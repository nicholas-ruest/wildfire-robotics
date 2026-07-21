//! Route, delivery, and process-manager lifecycle with semantic compensation.
#![allow(missing_docs)]
use crate::{CustodyChain, CustodyTransfer, LogisticsError};
use chrono::{DateTime, Utc};
use shared_kernel::{EntityId, TimeWindow};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteState {
    Proposed,
    Validated,
    Active,
    Blocked,
    Expired,
}
#[derive(Clone, Debug)]
pub struct Route {
    pub id: EntityId,
    envelope_digest: [u8; 32],
    restriction_digest: [u8; 32],
    validity: TimeWindow,
    safe_stop: String,
    reroute_authorized: bool,
    pub state: RouteState,
}
impl Route {
    pub fn propose(
        id: EntityId,
        envelope_digest: [u8; 32],
        restriction_digest: [u8; 32],
        validity: TimeWindow,
        safe_stop: impl Into<String>,
        reroute_authorized: bool,
    ) -> Result<Self, LogisticsError> {
        let safe_stop = safe_stop.into();
        if envelope_digest == [0; 32]
            || restriction_digest == [0; 32]
            || safe_stop.trim().is_empty()
        {
            return Err(LogisticsError::UnsafeRoute);
        }
        Ok(Self {
            id,
            envelope_digest,
            restriction_digest,
            validity,
            safe_stop,
            reroute_authorized,
            state: RouteState::Proposed,
        })
    }
    pub fn validate(
        &mut self,
        current_envelope: [u8; 32],
        current_restrictions: [u8; 32],
        now: DateTime<Utc>,
    ) -> Result<(), LogisticsError> {
        if self.state != RouteState::Proposed
            || self.envelope_digest != current_envelope
            || self.restriction_digest != current_restrictions
            || !self.validity.contains(now)
        {
            return Err(LogisticsError::UnsafeRoute);
        }
        self.state = RouteState::Validated;
        Ok(())
    }
    pub fn activate(&mut self) -> Result<(), LogisticsError> {
        if self.state != RouteState::Validated {
            return Err(LogisticsError::UnsafeRoute);
        }
        self.state = RouteState::Active;
        Ok(())
    }
    pub fn block(&mut self) -> RouteCompensation {
        self.state = RouteState::Blocked;
        if self.reroute_authorized {
            RouteCompensation::AuthorizedReroute
        } else {
            RouteCompensation::SafeStop(self.safe_stop.clone())
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteCompensation {
    AuthorizedReroute,
    SafeStop(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DependencyState {
    Current,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteOption {
    pub route_id: EntityId,
    pub duration_seconds: u64,
    pub risk_micros: u64,
    pub envelope: DependencyState,
    pub restrictions: DependencyState,
    pub communications: DependencyState,
    pub energy: DependencyState,
    pub maintenance: DependencyState,
    pub safe_stops: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteRecommendation {
    pub route_id: EntityId,
    pub assumptions: Vec<String>,
    pub alternatives: Vec<EntityId>,
}

pub trait RoutingOptimizer {
    fn choose(&self, options: &[RouteOption]) -> Result<RouteRecommendation, LogisticsError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DeterministicRoutingOptimizer;
impl RoutingOptimizer for DeterministicRoutingOptimizer {
    fn choose(&self, options: &[RouteOption]) -> Result<RouteRecommendation, LogisticsError> {
        let mut feasible = options
            .iter()
            .filter(|option| {
                option.envelope == DependencyState::Current
                    && option.restrictions == DependencyState::Current
                    && option.communications == DependencyState::Current
                    && option.energy == DependencyState::Current
                    && option.maintenance == DependencyState::Current
                    && !option.safe_stops.is_empty()
            })
            .collect::<Vec<_>>();
        feasible.sort_by_key(|option| {
            (
                option.risk_micros,
                option.duration_seconds,
                option.route_id.to_string(),
            )
        });
        let selected = feasible.first().ok_or(LogisticsError::UnsafeRoute)?;
        Ok(RouteRecommendation {
            route_id: selected.route_id.clone(),
            assumptions: vec![
                "ODD and restriction snapshots remain current".into(),
                "communications, energy, and maintenance are hard constraints".into(),
            ],
            alternatives: feasible
                .iter()
                .skip(1)
                .map(|option| option.route_id.clone())
                .collect(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryState {
    Prepared,
    InCustody,
    InTransit,
    HandedOver,
    Rejected,
    Lost,
}
#[derive(Clone, Debug)]
pub struct Delivery {
    pub id: EntityId,
    pub item_id: EntityId,
    pub route_id: EntityId,
    pub state: DeliveryState,
    pub attempts: u32,
    pub escalation: Option<String>,
}
impl Delivery {
    #[must_use]
    pub fn prepare(id: EntityId, item_id: EntityId, route_id: EntityId) -> Self {
        Self {
            id,
            item_id,
            route_id,
            state: DeliveryState::Prepared,
            attempts: 0,
            escalation: None,
        }
    }
    pub fn transfer(
        &mut self,
        route: &Route,
        chain: &mut CustodyChain,
        transfer: CustodyTransfer,
    ) -> Result<bool, LogisticsError> {
        if !matches!(
            self.state,
            DeliveryState::Prepared | DeliveryState::InCustody | DeliveryState::InTransit
        ) || route.state != RouteState::Active
            || transfer.item_id != self.item_id
        {
            return Err(LogisticsError::InvalidDelivery);
        }
        let added = chain.append(transfer)?;
        if added {
            self.state = DeliveryState::InCustody;
        }
        Ok(added)
    }
    pub fn dispatch(&mut self, route: &Route) -> Result<(), LogisticsError> {
        if self.state != DeliveryState::InCustody || route.state != RouteState::Active {
            return Err(LogisticsError::InvalidDelivery);
        }
        self.state = DeliveryState::InTransit;
        Ok(())
    }
    pub fn handover(
        &mut self,
        chain: &mut CustodyChain,
        transfer: CustodyTransfer,
    ) -> Result<bool, LogisticsError> {
        if self.state != DeliveryState::InTransit || transfer.item_id != self.item_id {
            return Err(LogisticsError::InvalidDelivery);
        }
        let added = chain.append(transfer)?;
        if added {
            self.state = DeliveryState::HandedOver;
        }
        Ok(added)
    }
    pub fn route_failed(&mut self, compensation: RouteCompensation) {
        self.attempts = self.attempts.saturating_add(1);
        self.escalation = Some(match compensation {
            RouteCompensation::AuthorizedReroute => "authorized reroute required".into(),
            RouteCompensation::SafeStop(place) => format!("safe stop at {place}"),
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogisticsMissionState {
    Draft,
    Validated,
    Authorized,
    Dispatched,
    Delivered,
    Aborted,
    Failed,
}
#[derive(Clone, Debug)]
pub struct LogisticsMission {
    pub id: EntityId,
    pub correlation_id: EntityId,
    pub authority_digest: [u8; 32],
    pub deadline: DateTime<Utc>,
    pub state: LogisticsMissionState,
    pub version: u64,
}
impl LogisticsMission {
    pub fn plan(
        id: EntityId,
        correlation_id: EntityId,
        authority_digest: [u8; 32],
        deadline: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<Self, LogisticsError> {
        if authority_digest == [0; 32] || deadline <= now {
            return Err(LogisticsError::InvalidDelivery);
        }
        Ok(Self {
            id,
            correlation_id,
            authority_digest,
            deadline,
            state: LogisticsMissionState::Draft,
            version: 1,
        })
    }
    pub fn transition(
        &mut self,
        from: LogisticsMissionState,
        to: LogisticsMissionState,
        now: DateTime<Utc>,
    ) -> Result<(), LogisticsError> {
        if self.state != from
            || now >= self.deadline
            || matches!(
                self.state,
                LogisticsMissionState::Aborted
                    | LogisticsMissionState::Failed
                    | LogisticsMissionState::Delivered
            )
        {
            return Err(LogisticsError::InvalidDelivery);
        }
        self.version = self
            .version
            .checked_add(1)
            .ok_or(LogisticsError::VersionExhausted)?;
        self.state = to;
        Ok(())
    }
    pub fn abort(&mut self) {
        self.state = LogisticsMissionState::Aborted;
        self.version = self.version.saturating_add(1);
    }
}
