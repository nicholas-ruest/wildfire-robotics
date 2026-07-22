#![allow(clippy::expect_used)]
#![allow(missing_docs)]
use aerial_deployment_operations::*;
use chrono::{Duration, TimeZone, Utc};

fn now() -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(42_000, 0).single().expect("time")
}

fn safe_margins() -> TransitionMargins {
    TransitionMargins::new(
        now() - Duration::seconds(1),
        now() + Duration::seconds(5),
        [10; MarginKind::COUNT],
    )
    .expect("margins")
}

fn deployment(count: u8) -> AirborneDeployment {
    AirborneDeployment::new(
        AirborneDeploymentId::new("deployment-1").expect("id"),
        count,
        4,
    )
    .expect("deployment")
}

#[test]
fn model_covers_every_legal_and_illegal_transition() {
    let protocol = DeploymentPhase::PROTOCOL;
    for (index, from) in protocol.iter().copied().enumerate() {
        for to in protocol.iter().copied() {
            let mut subject = deployment(1);
            for target in protocol.iter().copied().skip(1).take(index) {
                subject
                    .transition(0, target, now(), safe_margins())
                    .expect("setup transition");
            }
            let result = subject.transition(0, to, now(), safe_margins());
            let expected = protocol.get(index + 1).copied() == Some(to);
            assert_eq!(result.is_ok(), expected, "{from:?} -> {to:?}");
        }
    }
}

#[test]
fn each_margin_is_measured_current_and_positive_at_every_transition() {
    for missing in 0..MarginKind::COUNT {
        let mut values = [10; MarginKind::COUNT];
        values[missing] = 0;
        assert_eq!(
            TransitionMargins::new(now(), now() + Duration::seconds(1), values),
            Err(DomainError::UnsafeTransitionMargin)
        );
    }
    let mut subject = deployment(1);
    let expired = TransitionMargins::new(
        now() - Duration::seconds(2),
        now() - Duration::seconds(1),
        [10; MarginKind::COUNT],
    )
    .expect("well formed historical measurement");
    assert_eq!(
        subject.transition(0, DeploymentPhase::Extracted, now(), expired),
        Err(DomainError::UnsafeTransitionMargin)
    );
}

#[test]
fn containment_is_idempotent_and_replay_conflicts_are_rejected() {
    let mut subject = deployment(2);
    let command = CommandId::new("contain-1").expect("id");
    assert_eq!(
        subject.contain(command.clone(), 0, ContainmentAction::Pause, true),
        Ok(ContainmentOutcome::Paused)
    );
    assert_eq!(
        subject.contain(command.clone(), 0, ContainmentAction::Pause, true),
        Ok(ContainmentOutcome::Paused)
    );
    assert_eq!(
        subject.contain(command, 0, ContainmentAction::Reef, true),
        Err(DomainError::ReplayConflict)
    );
    let unsafe_jettison = CommandId::new("jettison-unsafe").expect("id");
    assert_eq!(
        subject.contain(
            unsafe_jettison,
            0,
            ContainmentAction::SafeSectorJettison,
            false,
        ),
        Err(DomainError::SafeSectorNotConfirmed)
    );
}

#[test]
fn terminal_phases_are_irreversible_and_retain_cannot_claim_a_regression() {
    let mut subject = deployment(1);
    subject
        .transition(0, DeploymentPhase::Extracted, now(), safe_margins())
        .expect("extracted");
    assert_eq!(
        subject.contain(
            CommandId::new("late-retain").expect("id"),
            0,
            ContainmentAction::Retain,
            true,
        ),
        Err(DomainError::InvalidTransition)
    );
    subject
        .contain(
            CommandId::new("isolate").expect("id"),
            0,
            ContainmentAction::Isolate,
            true,
        )
        .expect("isolate");
    assert_eq!(
        subject.contain(
            CommandId::new("terminal-rewrite").expect("id"),
            0,
            ContainmentAction::EmergencyLand,
            true,
        ),
        Err(DomainError::InvalidTransition)
    );
}

#[test]
fn every_bounded_containment_command_has_a_stable_idempotent_result() {
    let cases = [
        (ContainmentAction::Retain, ContainmentOutcome::Retained),
        (ContainmentAction::Pause, ContainmentOutcome::Paused),
        (ContainmentAction::Reef, ContainmentOutcome::Reefed),
        (ContainmentAction::Vent, ContainmentOutcome::Vented),
        (ContainmentAction::Isolate, ContainmentOutcome::Isolated),
        (ContainmentAction::Breakaway, ContainmentOutcome::BrokenAway),
        (
            ContainmentAction::EmergencyLand,
            ContainmentOutcome::EmergencyLanding,
        ),
        (
            ContainmentAction::SafeSectorJettison,
            ContainmentOutcome::Jettisoned,
        ),
    ];
    for (index, (action, expected)) in cases.into_iter().enumerate() {
        let mut subject = deployment(1);
        let command = CommandId::new(&format!("contain-{index}")).expect("id");
        assert_eq!(
            subject.contain(command.clone(), 0, action, true),
            Ok(expected)
        );
        assert_eq!(subject.contain(command, 0, action, true), Ok(expected));
    }
}

#[test]
fn concurrent_faults_are_confined_to_their_bounded_local_cohorts() {
    for fault in FaultKind::ALL {
        let mut subject = deployment(3);
        subject
            .transition(0, DeploymentPhase::Extracted, now(), safe_margins())
            .expect("transition");
        subject
            .transition(1, DeploymentPhase::Extracted, now(), safe_margins())
            .expect("transition");
        subject.report_fault(0, fault).expect("fault");
        subject
            .transition(1, DeploymentPhase::Stabilized, now(), safe_margins())
            .expect("other cohort remains autonomous");
        assert_eq!(
            subject.cohort(0).expect("cohort").phase,
            DeploymentPhase::Isolated
        );
        assert_eq!(
            subject.cohort(1).expect("cohort").phase,
            DeploymentPhase::Stabilized
        );
    }
    assert!(
        AirborneDeployment::new(AirborneDeploymentId::new("too-large").expect("id"), 9, 4).is_err()
    );
}

struct HostileAdvisor;
impl AdvisoryAdapter for HostileAdvisor {
    fn advise(&self, _: &AdvisorySnapshot) -> AdvisoryOutput {
        AdvisoryOutput::new("learned", 10_000, vec![1, 2, 3]).expect("advice")
    }
}

#[test]
fn optional_learned_advice_has_no_transition_authority_or_trust_assignment() {
    let mut subject = deployment(1);
    let output = subject.collect_advice(0, &HostileAdvisor).expect("advice");
    assert_eq!(output.source(), "learned");
    assert_eq!(
        subject.cohort(0).expect("cohort").phase,
        DeploymentPhase::Retained
    );
    assert_eq!(
        subject.cohort(0).expect("cohort").trust,
        LocalTrust::Unassigned
    );
    assert!(
        subject
            .transition(0, DeploymentPhase::Extracted, now(), safe_margins())
            .is_ok()
    );
}

#[test]
fn hierarchy_is_summary_only_and_has_no_global_safe_flight_gate() {
    let mut subject = deployment(3);
    subject.report_fault(2, FaultKind::Network).expect("fault");
    let summary = subject.summary();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.isolated, 1);
    assert_eq!(summary.active, 2);
    subject
        .transition(0, DeploymentPhase::Extracted, now(), safe_margins())
        .expect("local transition ignores summary/network dependency");
}
