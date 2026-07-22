//! Prompt 17 robot-care safety, provenance, and recovery invariants.
#![allow(clippy::unwrap_used)]
use chrono::{Duration, Utc};
use robot_care_recovery::*;
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};
fn policy() -> ServicePolicy {
    let mut rules = BTreeMap::new();
    rules.insert("drive".into(), BTreeSet::from(["inspect:meter-v2".into()]));
    let mut p = ServicePolicy::define(EntityId::new(), [1; 32], 100, rules).unwrap();
    p.transition(PolicyState::Draft, PolicyState::Validated)
        .unwrap();
    p.transition(PolicyState::Validated, PolicyState::Approved)
        .unwrap();
    p.transition(PolicyState::Approved, PolicyState::Effective)
        .unwrap();
    p
}
fn assessment(hazard: HazardState) -> RecoveryAssessment {
    RecoveryAssessment {
        scene_odd: SafetyFact::Passed,
        route_odd: SafetyFact::Passed,
        communications: SafetyFact::Passed,
        medic_capable: SafetyFact::Passed,
        mass_compatible: SafetyFact::Passed,
        lift_tow_cradle_compatible: SafetyFact::Passed,
        energy_isolated: SafetyFact::Passed,
        tools_stabilized: SafetyFact::Passed,
        destination_reserved: SafetyFact::Passed,
        fallback_ready: SafetyFact::Passed,
        human_rescue_clear: SafetyFact::Passed,
        exclusion_clear: SafetyFact::Passed,
        hazard,
    }
}
fn part(serial: &str) -> InstalledPart {
    InstalledPart {
        serial: serial.into(),
        provenance_digest: [3; 32],
        module: "drive".into(),
        cannibalized: false,
    }
}

#[test]
fn overdue_critical_maintenance_removes_eligibility() {
    let due = Utc::now() - Duration::seconds(1);
    let mut plan =
        MaintenancePlan::propose(EntityId::new(), EntityId::new(), due, [1; 32], 100, 500).unwrap();
    assert!(plan.evaluate_due(Utc::now()));
    assert!(!plan.eligible());
}
#[test]
fn maintenance_robot_requires_promoted_compatible_procedure_and_isolation() {
    let mut order = WorkOrder::open(
        EntityId::new(),
        EntityId::new(),
        "drive",
        "inspect",
        "meter-v2",
    );
    assert_eq!(
        order.start(&policy(), false),
        Err(CareError::ProcedureDenied)
    );
    order.start(&policy(), true).unwrap();
    order.install(part("part-1")).unwrap();
    assert_eq!(
        order.install(part("part-1")),
        Err(CareError::IncompleteRepair)
    );
    let mut sim = DeterministicMaintenanceSimulator::default();
    sim.execute(&order).unwrap();
}
#[test]
fn every_unknown_heat_or_contamination_hazard_requires_quarantine_transport() {
    for hazard in [
        HazardState::Unknown,
        HazardState::HeatExposed,
        HazardState::SwollenOrLeaking,
        HazardState::ElectricalUnsafe,
        HazardState::Contaminated,
        HazardState::StructurallyUnstable,
    ] {
        assert!(hazard.requires_quarantine());
        let asset = EntityId::new();
        let mut mission = RecoveryMission::request(EntityId::new(), asset, "fire-zone-a").unwrap();
        mission.authorize(&assessment(hazard)).unwrap();
        assert!(mission.quarantine_transport);
    }
}
#[test]
fn human_rescue_exclusion_and_incompatible_recovery_always_deny() {
    for mutation in 0..4 {
        let mut a = assessment(HazardState::KnownSafe);
        match mutation {
            0 => a.human_rescue_clear = SafetyFact::Failed,
            1 => a.exclusion_clear = SafetyFact::Failed,
            2 => a.lift_tow_cradle_compatible = SafetyFact::Failed,
            _ => a.destination_reserved = SafetyFact::Failed,
        }
        let mut mission =
            RecoveryMission::request(EntityId::new(), EntityId::new(), "hospital").unwrap();
        assert_eq!(mission.authorize(&a), Err(CareError::UnsafeRecovery));
    }
}
#[test]
fn medic_failure_retreats_without_claiming_recovery() {
    let asset = EntityId::new();
    let mut sim = DeterministicMedicSimulator::new(assessment(HazardState::KnownSafe));
    sim.stabilize(&asset).unwrap();
    sim.fail_recovery = true;
    assert_eq!(sim.recover(&asset), Err(CareError::UnsafeRecovery));
    sim.safe_retreat().unwrap();
    assert!(sim.retreated);
    assert!(!sim.recovered(&asset));
}
#[test]
fn hospital_saturation_never_spills_into_ordinary_capacity() {
    let mut capacity = HospitalCapacity::default();
    capacity.configure("fire-separated", 1);
    capacity.admit("fire-separated").unwrap();
    assert_eq!(
        capacity.admit("fire-separated"),
        Err(CareError::CapacityUnavailable)
    );
    assert_eq!(
        capacity.admit("ordinary"),
        Err(CareError::CapacityUnavailable)
    );
}
#[test]
fn quarantine_clearance_requires_independent_multiple_evidence() {
    let mut case =
        QuarantineCase::open(EntityId::new(), EntityId::new(), "fire-zone", true).unwrap();
    case.isolate().unwrap();
    assert_eq!(
        case.clear([[1; 32]], true),
        Err(CareError::QuarantineRequired)
    );
    case.clear([[1; 32], [2; 32]], true).unwrap();
    assert_eq!(case.state, QuarantineState::Cleared);
}
#[test]
fn repair_recertification_requires_parts_calibration_burn_in_health_and_clearance() {
    let mut repair = RepairCase::admit(EntityId::new(), EntityId::new());
    repair.begin_repair().unwrap();
    repair.install(part("serial-1")).unwrap();
    assert_eq!(
        repair.recertify([4; 32], EntityId::new(), true, true),
        Err(CareError::RecertificationDenied)
    );
    repair.calibrate([5; 32]).unwrap();
    repair.burn_in([6; 32]).unwrap();
    assert_eq!(
        repair.recertify([4; 32], EntityId::new(), true, false),
        Err(CareError::RecertificationDenied)
    );
    repair
        .recertify([4; 32], EntityId::new(), true, true)
        .unwrap();
    assert_eq!(repair.state, RepairState::Recertified);
}
#[test]
fn retirement_cannot_close_without_depower_revocation_sanitization_and_salvage_custody() {
    let mut case = RetirementCase::propose(EntityId::new(), EntityId::new());
    case.approve().unwrap();
    assert_eq!(case.depower(false), Err(CareError::RetirementDenied));
    case.depower(true).unwrap();
    assert_eq!(case.sanitize(false, true), Err(CareError::RetirementDenied));
    case.sanitize(true, true).unwrap();
    assert_eq!(case.close(), Err(CareError::RetirementDenied));
    case.salvage("hazmat recycler", [7; 32]).unwrap();
    case.close().unwrap();
    assert_eq!(case.state, RetirementState::Closed);
}
