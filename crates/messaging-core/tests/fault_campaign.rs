//! Deterministic fault campaigns for the prompt-07 exit gate.

use edge_reconciliation::{
    AggregateDisposition, AggregateTracker, AuthorityFact, AuthorityState, ReconcileDecision,
    ReplicaVersion,
};
use messaging_core::{
    delivery::{DeliveryDisposition, FailureClass, RetryPolicy},
    store_forward::{EnqueueOutcome, StoreForwardQueue, TelemetryTier},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct DurableBrokerModel {
    online: bool,
    facts: BTreeMap<String, Vec<u8>>,
}

impl DurableBrokerModel {
    fn start(&mut self) {
        self.online = true;
    }
    fn partition(&mut self) {
        self.online = false;
    }
    fn restart(&mut self) {
        self.online = true;
    }
    fn publish(&mut self, id: &str, payload: &[u8]) -> Result<bool, &'static str> {
        if !self.online {
            return Err("unavailable");
        }
        match self.facts.get(id) {
            Some(existing) if existing == payload => Ok(false),
            Some(_) => Err("contradictory duplicate"),
            None => {
                self.facts.insert(id.into(), payload.into());
                Ok(true)
            }
        }
    }
}

#[test]
fn partition_restart_and_reconnect_burst_preserve_safety_facts_and_one_effect()
-> Result<(), Box<dyn std::error::Error>> {
    let mut broker = DurableBrokerModel::default();
    broker.start();
    assert!(broker.publish("safety-1", b"grounded")?);
    broker.partition();
    let mut spool = StoreForwardQueue::new(2_000, 2_000 * 32, 1_000)?;
    for sequence in 0..1_000_u32 {
        let mut payload = sequence.to_be_bytes().to_vec();
        payload.extend_from_slice(b"-grounded");
        assert_eq!(
            spool.enqueue(TelemetryTier::SafetyCritical, payload)?,
            EnqueueOutcome::Accepted
        );
    }
    assert!(broker.publish("offline", b"fact").is_err());
    broker.restart();
    assert!(!broker.publish("safety-1", b"grounded")?);
    let mut effects = BTreeSet::new();
    let mut sequence = 0_u32;
    while let Some(item) = spool.pop() {
        let id = format!("reconnect-{sequence}");
        assert!(broker.publish(&id, &item.payload)?);
        assert!(effects.insert(id.clone()));
        assert!(!broker.publish(&id, &item.payload)?);
        assert!(!effects.insert(id));
        sequence += 1;
    }
    assert_eq!(effects.len(), 1_000);
    assert_eq!(broker.facts.len(), 1_001);
    Ok(())
}

#[test]
fn duplication_reordering_delay_and_poison_never_skip_or_hot_loop()
-> Result<(), Box<dyn std::error::Error>> {
    let mut tracker = AggregateTracker::default();
    assert_eq!(
        tracker.classify(3, [3; 32]),
        AggregateDisposition::Gap {
            expected: 1,
            actual: 3
        }
    );
    tracker.record_committed(1, [1; 32])?;
    assert_eq!(
        tracker.classify(1, [1; 32]),
        AggregateDisposition::Duplicate
    );
    tracker.record_committed(2, [2; 32])?;
    tracker.record_committed(3, [3; 32])?;
    let retries = RetryPolicy::new(4, 10, 1_000, 25)?;
    assert!(matches!(
        retries.disposition("delayed", 2, FailureClass::Retryable),
        DeliveryDisposition::Nak {
            delay_millis: 10..=1_000
        }
    ));
    assert!(matches!(
        retries.disposition("poison", 1, FailureClass::Permanent),
        DeliveryDisposition::Quarantine { .. }
    ));
    assert!(matches!(
        retries.disposition("exhausted", 4, FailureClass::Retryable),
        DeliveryDisposition::Quarantine { .. }
    ));
    Ok(())
}

#[test]
fn stricter_authority_conflict_wins_independent_of_arrival_order()
-> Result<(), Box<dyn std::error::Error>> {
    let progression =
        AuthorityFact::new(ReplicaVersion::new("cloud", 99)?, AuthorityState::Permitted);
    let grounded = AuthorityFact::new(ReplicaVersion::new("station", 1)?, AuthorityState::Grounded);
    assert_eq!(
        progression.reconcile(&grounded),
        ReconcileDecision::ApplyRemote(grounded.clone())
    );
    assert_eq!(
        grounded.reconcile(&progression),
        ReconcileDecision::KeepLocal
    );
    Ok(())
}
