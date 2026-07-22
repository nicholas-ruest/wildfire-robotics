#![allow(missing_docs, clippy::must_use_candidate)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point3 {
    pub x_mm: i32,
    pub y_mm: i32,
    pub z_mm: i32,
}
impl Point3 {
    pub fn distance_squared(self, o: Self) -> u64 {
        u64::from(self.x_mm.abs_diff(o.x_mm)).pow(2)
            + u64::from(self.y_mm.abs_diff(o.y_mm)).pow(2)
            + u64::from(self.z_mm.abs_diff(o.z_mm)).pow(2)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerKind {
    Px4,
    ArduPilot,
}
pub trait FlightControllerFacade {
    fn kind(&self) -> ControllerKind;
    fn encode(&self, command: &FlightCommand) -> ControllerSetpoint;
}
pub struct Px4Facade;
pub struct ArduPilotFacade;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerSetpoint {
    pub frame: &'static str,
    pub target: Point3,
    pub mode: FlightMode,
}
impl FlightControllerFacade for Px4Facade {
    fn kind(&self) -> ControllerKind {
        ControllerKind::Px4
    }
    fn encode(&self, c: &FlightCommand) -> ControllerSetpoint {
        ControllerSetpoint {
            frame: "NED",
            target: c.target,
            mode: c.mode,
        }
    }
}
impl FlightControllerFacade for ArduPilotFacade {
    fn kind(&self) -> ControllerKind {
        ControllerKind::ArduPilot
    }
    fn encode(&self, c: &FlightCommand) -> ControllerSetpoint {
        ControllerSetpoint {
            frame: "GLOBAL_RELATIVE_ALT",
            target: c.target,
            mode: c.mode,
        }
    }
}
#[derive(Debug, Clone)]
pub struct UavState {
    pub id: String,
    pub position: Point3,
    pub energy_bps: u16,
    pub link_quality_bps: u16,
    pub controller: ControllerKind,
}
#[derive(Debug, Clone)]
pub struct Geofence {
    pub min: Point3,
    pub max: Point3,
}
impl Geofence {
    fn contains(&self, p: Point3) -> bool {
        p.x_mm >= self.min.x_mm
            && p.x_mm <= self.max.x_mm
            && p.y_mm >= self.min.y_mm
            && p.y_mm <= self.max.y_mm
            && p.z_mm >= self.min.z_mm
            && p.z_mm <= self.max.z_mm
    }
}
#[derive(Debug, Clone)]
pub struct AuthorityEnvelope {
    pub mission_id: String,
    pub issued_tick: u64,
    pub expires_tick: u64,
    pub geofence: Geofence,
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoordinationError {
    #[error("cohort invalid")]
    InvalidCohort,
    #[error("authority stale")]
    StaleAuthority,
    #[error("geofence violation")]
    GeofenceViolation,
    #[error("link route unavailable")]
    NoLinkRoute,
    #[error("invalid authority envelope")]
    InvalidAuthority,
    #[error("task lies outside authorized airspace")]
    InvalidTask,
    #[error("consensus unavailable")]
    NoQuorum,
    #[error("link map stale")]
    StaleLinkMap,
    #[error("return reserve insufficient")]
    InsufficientReturnEnergy,
}
#[derive(Debug, Clone)]
pub struct CohortCell {
    pub leader: String,
    pub members: Vec<UavState>,
}
pub struct CohortPlan {
    cells: Vec<CohortCell>,
}
impl CohortPlan {
    pub fn hierarchical(mut members: Vec<UavState>, max: usize) -> Result<Self, CoordinationError> {
        if members.is_empty() || max == 0 || max > 16 {
            return Err(CoordinationError::InvalidCohort);
        }
        members.sort_by(|a, b| a.id.cmp(&b.id));
        let cells = members
            .chunks(max)
            .map(|m| CohortCell {
                leader: m[0].id.clone(),
                members: m.to_vec(),
            })
            .collect();
        Ok(Self { cells })
    }
    pub fn cells(&self) -> &[CohortCell] {
        &self.cells
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightMode {
    Formation,
    Coverage,
    Relay,
    ReturnToLaunch,
    HoldThenLand,
}
#[derive(Debug, Clone)]
pub enum PlatformTask {
    Reconnaissance { target: Point3 },
    Coverage { min: Point3, max: Point3 },
    Relay { position: Point3 },
}
#[derive(Debug, Clone)]
pub struct TaskAssignment {
    pub uav_id: String,
    pub task_index: usize,
}
#[derive(Debug, Clone)]
pub struct FlightCommand {
    pub uav_id: String,
    pub target: Point3,
    pub mode: FlightMode,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationAuthority {
    InheritedOnly,
}
#[derive(Debug)]
pub struct CoordinationOutcome {
    pub commands: Vec<FlightCommand>,
    pub assignments: Vec<TaskAssignment>,
    pub authority: CoordinationAuthority,
    pub collision_avoidance_applied: bool,
}
pub trait UavCoordinationPort {
    fn coordinate(
        &self,
        uavs: &[UavState],
        authority: &AuthorityEnvelope,
        tick: u64,
        tasks: &[PlatformTask],
    ) -> Result<CoordinationOutcome, CoordinationError>;
}
#[derive(Default)]
pub struct ConventionalCoordinator;
impl UavCoordinationPort for ConventionalCoordinator {
    fn coordinate(
        &self,
        uavs: &[UavState],
        a: &AuthorityEnvelope,
        tick: u64,
        tasks: &[PlatformTask],
    ) -> Result<CoordinationOutcome, CoordinationError> {
        if a.mission_id.is_empty()
            || a.issued_tick >= a.expires_tick
            || a.geofence.min.x_mm > a.geofence.max.x_mm
            || a.geofence.min.y_mm > a.geofence.max.y_mm
            || a.geofence.min.z_mm > a.geofence.max.z_mm
        {
            return Err(CoordinationError::InvalidAuthority);
        }
        if tick < a.issued_tick || tick >= a.expires_tick {
            return Err(CoordinationError::StaleAuthority);
        }
        if uavs.iter().any(|u| !a.geofence.contains(u.position)) {
            return Err(CoordinationError::GeofenceViolation);
        }
        for task in tasks {
            let valid = match task {
                PlatformTask::Reconnaissance { target }
                | PlatformTask::Relay { position: target } => a.geofence.contains(*target),
                PlatformTask::Coverage { min, max } => {
                    min.x_mm <= max.x_mm
                        && min.y_mm <= max.y_mm
                        && min.z_mm <= max.z_mm
                        && a.geofence.contains(*min)
                        && a.geofence.contains(*max)
                }
            };
            if !valid {
                return Err(CoordinationError::InvalidTask);
            }
        }
        let mut sorted = uavs.to_vec();
        sorted.sort_by(|a, b| a.id.cmp(&b.id));
        let mut commands = Vec::new();
        for (i, u) in sorted.iter().enumerate() {
            let mode = if u.energy_bps <= 2000 {
                FlightMode::ReturnToLaunch
            } else if u.link_quality_bps == 0 {
                FlightMode::HoldThenLand
            } else {
                FlightMode::Formation
            };
            let target = match tasks.get(i % tasks.len().max(1)) {
                Some(
                    PlatformTask::Reconnaissance { target }
                    | PlatformTask::Relay { position: target },
                ) => *target,
                Some(PlatformTask::Coverage { min, .. }) => *min,
                None => Point3 {
                    x_mm: u.position.x_mm + i32::try_from(i).unwrap_or(i32::MAX) * 5000,
                    y_mm: u.position.y_mm,
                    z_mm: u.position.z_mm,
                },
            };
            let mut target = target;
            while commands
                .iter()
                .any(|c: &FlightCommand| c.target.distance_squared(target) < 25_000_000)
            {
                target.x_mm = target.x_mm.saturating_add(5000);
                if !a.geofence.contains(target) {
                    target.x_mm = target.x_mm.saturating_sub(10_000);
                }
            }
            commands.push(FlightCommand {
                uav_id: u.id.clone(),
                target,
                mode,
            });
        }
        let assignments = tasks
            .iter()
            .enumerate()
            .filter_map(|(i, _)| {
                sorted.get(i % sorted.len().max(1)).map(|u| TaskAssignment {
                    uav_id: u.id.clone(),
                    task_index: i,
                })
            })
            .collect();
        Ok(CoordinationOutcome {
            commands,
            assignments,
            authority: CoordinationAuthority::InheritedOnly,
            collision_avoidance_applied: uavs.len() > 1,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceClass {
    CommandAndControl,
    Telemetry,
    Payload,
}
pub struct LinkSample {
    pub from: String,
    pub to: String,
    pub quality_bps: u16,
}
pub struct RelayCandidate {
    pub id: String,
    pub a_quality_bps: u16,
    pub b_quality_bps: u16,
}
#[derive(Debug)]
pub struct LinkRoute {
    pub service: ServiceClass,
    pub handoff: Option<String>,
    pub effective_quality_bps: u16,
}
pub struct LinkQualityMap {
    samples: Vec<LinkSample>,
    version: u64,
    as_of_tick: u64,
    expires_tick: u64,
}
impl LinkQualityMap {
    pub fn new(samples: Vec<LinkSample>) -> Result<Self, CoordinationError> {
        Self::versioned(1, 0, u64::MAX, samples)
    }
    pub fn versioned(
        version: u64,
        as_of_tick: u64,
        expires_tick: u64,
        samples: Vec<LinkSample>,
    ) -> Result<Self, CoordinationError> {
        if samples
            .iter()
            .any(|s| s.from.is_empty() || s.to.is_empty() || s.quality_bps > 10_000)
            || version == 0
            || as_of_tick >= expires_tick
        {
            return Err(CoordinationError::NoLinkRoute);
        }
        Ok(Self {
            samples,
            version,
            as_of_tick,
            expires_tick,
        })
    }
    pub fn route(
        &self,
        service: ServiceClass,
        from: &str,
        to: &str,
        relays: &[RelayCandidate],
    ) -> Result<LinkRoute, CoordinationError> {
        self.route_at(service, from, to, relays, self.as_of_tick)
    }
    pub fn route_at(
        &self,
        service: ServiceClass,
        from: &str,
        to: &str,
        relays: &[RelayCandidate],
        tick: u64,
    ) -> Result<LinkRoute, CoordinationError> {
        if tick < self.as_of_tick || tick >= self.expires_tick {
            return Err(CoordinationError::StaleLinkMap);
        }
        if relays
            .iter()
            .any(|r| r.id.is_empty() || r.a_quality_bps > 10_000 || r.b_quality_bps > 10_000)
        {
            return Err(CoordinationError::NoLinkRoute);
        }
        let direct = self
            .samples
            .iter()
            .find(|s| s.from == from && s.to == to)
            .map_or(0, |s| s.quality_bps);
        let mut ranked = relays
            .iter()
            .map(|r| (r.a_quality_bps.min(r.b_quality_bps), &r.id))
            .collect::<Vec<_>>();
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
        let best = ranked.first().copied();
        if let Some((q, id)) = best.filter(|x| x.0 > direct) {
            return Ok(LinkRoute {
                service,
                handoff: Some(id.clone()),
                effective_quality_bps: q,
            });
        }
        if direct == 0 {
            return Err(CoordinationError::NoLinkRoute);
        }
        Ok(LinkRoute {
            service,
            handoff: None,
            effective_quality_bps: direct,
        })
    }
    pub fn version(&self) -> u64 {
        self.version
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnergyRequirement {
    pub mission_bps: u16,
    pub return_bps: u16,
    pub reserve_bps: u16,
    pub uncertainty_bps: u16,
}
impl EnergyRequirement {
    pub fn admit(&self, available_bps: u16) -> Result<(), CoordinationError> {
        let required = u32::from(self.mission_bps)
            + u32::from(self.return_bps)
            + u32::from(self.reserve_bps)
            + u32::from(self.uncertainty_bps);
        if u32::from(available_bps) < required {
            return Err(CoordinationError::InsufficientReturnEnergy);
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct ConsensusView {
    pub term: u64,
    pub leader_id: String,
    pub observed_tick: u64,
    pub voters: usize,
    pub cohort_size: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusOutcome {
    CommittedInheritedPlan,
    HoldSafe,
}
pub struct BoundedConsensus {
    current_term: u64,
    max_age_ticks: u64,
}
impl BoundedConsensus {
    pub fn new(term: u64, max_age_ticks: u64) -> Self {
        Self {
            current_term: term,
            max_age_ticks,
        }
    }
    pub fn apply(
        &mut self,
        view: &ConsensusView,
        tick: u64,
        active_members: &[String],
    ) -> ConsensusOutcome {
        let quorum = view.cohort_size / 2 + 1;
        if view.term < self.current_term
            || tick.saturating_sub(view.observed_tick) > self.max_age_ticks
            || view.voters < quorum
            || !active_members.contains(&view.leader_id)
        {
            return ConsensusOutcome::HoldSafe;
        }
        self.current_term = view.term;
        ConsensusOutcome::CommittedInheritedPlan
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentalStatus {
    DisabledPendingPromotion,
}
pub struct ExperimentalCoordinator;
impl ExperimentalCoordinator {
    pub fn status() -> ExperimentalStatus {
        ExperimentalStatus::DisabledPendingPromotion
    }
}
