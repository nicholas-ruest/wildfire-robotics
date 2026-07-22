#![allow(missing_docs, clippy::unwrap_used)]
use vehicle_integration::*;
fn uav(id: &str, x: i32, energy: u16) -> UavState {
    UavState {
        id: id.into(),
        position: Point3 {
            x_mm: x,
            y_mm: 0,
            z_mm: 100_000,
        },
        energy_bps: energy,
        link_quality_bps: 9000,
        controller: ControllerKind::Px4,
    }
}
fn authority() -> AuthorityEnvelope {
    AuthorityEnvelope {
        mission_id: "m1".into(),
        issued_tick: 90,
        expires_tick: 110,
        geofence: Geofence {
            min: Point3 {
                x_mm: -100_000,
                y_mm: -100_000,
                z_mm: 50_000,
            },
            max: Point3 {
                x_mm: 100_000,
                y_mm: 100_000,
                z_mm: 120_000,
            },
        },
    }
}
#[test]
fn cohort_is_bounded_and_hierarchical() {
    let c = CohortPlan::hierarchical(
        (0..17)
            .map(|i| uav(&format!("u{i}"), i * 5000, 9000))
            .collect(),
        8,
    )
    .unwrap();
    assert_eq!(c.cells().len(), 3);
    assert!(c.cells().iter().all(|x| x.members.len() <= 8));
}
#[test]
fn rejects_stale_authority_and_geofence_breach() {
    let coordinator = ConventionalCoordinator;
    let mut stale = authority();
    stale.expires_tick = 99;
    assert_eq!(
        coordinator
            .coordinate(&[uav("u1", 0, 9000)], &stale, 100, &[])
            .unwrap_err(),
        CoordinationError::StaleAuthority
    );
    let mut outside = uav("u1", 200_000, 9000);
    outside.position.z_mm = 100_000;
    assert_eq!(
        coordinator
            .coordinate(&[outside], &authority(), 100, &[])
            .unwrap_err(),
        CoordinationError::GeofenceViolation
    );
}
#[test]
fn expiry_is_exclusive_and_invalid_envelope_rejected() {
    let c = ConventionalCoordinator;
    assert_eq!(
        c.coordinate(&[uav("u", 0, 9000)], &authority(), 110, &[])
            .unwrap_err(),
        CoordinationError::StaleAuthority
    );
    let mut invalid = authority();
    invalid.geofence.min.x_mm = 1;
    invalid.geofence.max.x_mm = 0;
    assert_eq!(
        c.coordinate(&[uav("u", 0, 9000)], &invalid, 100, &[])
            .unwrap_err(),
        CoordinationError::InvalidAuthority
    );
}
#[test]
fn rejects_task_outside_airspace_and_reversed_coverage() {
    let c = ConventionalCoordinator;
    let outside = PlatformTask::Reconnaissance {
        target: Point3 {
            x_mm: 200_000,
            y_mm: 0,
            z_mm: 100_000,
        },
    };
    assert_eq!(
        c.coordinate(&[uav("u", 0, 9000)], &authority(), 100, &[outside])
            .unwrap_err(),
        CoordinationError::InvalidTask
    );
    let reversed = PlatformTask::Coverage {
        min: Point3 {
            x_mm: 10,
            y_mm: 0,
            z_mm: 100_000,
        },
        max: Point3 {
            x_mm: 0,
            y_mm: 0,
            z_mm: 100_000,
        },
    };
    assert_eq!(
        c.coordinate(&[uav("u", 0, 9000)], &authority(), 100, &[reversed])
            .unwrap_err(),
        CoordinationError::InvalidTask
    );
}
#[test]
fn allocates_recon_tasks_without_creating_authority() {
    let out = ConventionalCoordinator
        .coordinate(
            &[uav("u1", 0, 9000)],
            &authority(),
            100,
            &[PlatformTask::Reconnaissance {
                target: Point3 {
                    x_mm: 10_000,
                    y_mm: 0,
                    z_mm: 100_000,
                },
            }],
        )
        .unwrap();
    assert_eq!(out.assignments.len(), 1);
    assert_eq!(out.authority, CoordinationAuthority::InheritedOnly);
}
#[test]
fn collision_avoidance_separates_formation() {
    let out = ConventionalCoordinator
        .coordinate(
            &[uav("a", 0, 9000), uav("b", 100, 9000)],
            &authority(),
            100,
            &[],
        )
        .unwrap();
    assert!(
        out.commands
            .windows(2)
            .all(|w| w[0].target.distance_squared(w[1].target) >= 25_000_000)
    );
}
#[test]
fn collision_avoidance_separates_converging_task_targets() {
    let same = Point3 {
        x_mm: 0,
        y_mm: 0,
        z_mm: 100_000,
    };
    let out = ConventionalCoordinator
        .coordinate(
            &[uav("a", 0, 9000), uav("b", 5000, 9000)],
            &authority(),
            100,
            &[
                PlatformTask::Reconnaissance { target: same },
                PlatformTask::Reconnaissance { target: same },
            ],
        )
        .unwrap();
    assert!(
        out.commands[0]
            .target
            .distance_squared(out.commands[1].target)
            >= 25_000_000
    );
}
#[test]
fn low_reserve_returns_and_total_loss_fails_safe() {
    let mut low = uav("low", 0, 1500);
    low.link_quality_bps = 0;
    let out = ConventionalCoordinator
        .coordinate(&[low], &authority(), 100, &[])
        .unwrap();
    assert_eq!(out.commands[0].mode, FlightMode::ReturnToLaunch);
    let mut lost = uav("lost", 0, 9000);
    lost.link_quality_bps = 0;
    let out = ConventionalCoordinator
        .coordinate(&[lost], &authority(), 100, &[])
        .unwrap();
    assert_eq!(out.commands[0].mode, FlightMode::HoldThenLand);
}
#[test]
fn relay_handoff_preserves_priority_service() {
    let map = LinkQualityMap::new(vec![LinkSample {
        from: "a".into(),
        to: "b".into(),
        quality_bps: 2000,
    }])
    .unwrap();
    let route = map
        .route(
            ServiceClass::CommandAndControl,
            "a",
            "b",
            &[RelayCandidate {
                id: "relay".into(),
                a_quality_bps: 9000,
                b_quality_bps: 8000,
            }],
        )
        .unwrap();
    assert_eq!(route.handoff, Some("relay".into()));
    assert_eq!(route.service, ServiceClass::CommandAndControl);
}
#[test]
fn stale_link_map_and_invalid_relay_fail_closed() {
    let map = LinkQualityMap::versioned(
        2,
        10,
        20,
        vec![LinkSample {
            from: "a".into(),
            to: "b".into(),
            quality_bps: 5000,
        }],
    )
    .unwrap();
    assert_eq!(map.version(), 2);
    assert_eq!(
        map.route_at(ServiceClass::Telemetry, "a", "b", &[], 20)
            .unwrap_err(),
        CoordinationError::StaleLinkMap
    );
    assert_eq!(
        map.route_at(
            ServiceClass::Telemetry,
            "a",
            "b",
            &[RelayCandidate {
                id: "r".into(),
                a_quality_bps: 10_001,
                b_quality_bps: 1
            }],
            15
        )
        .unwrap_err(),
        CoordinationError::NoLinkRoute
    );
}
#[test]
fn energy_admission_is_monotonic_at_reserve_boundary() {
    let e = EnergyRequirement {
        mission_bps: 3000,
        return_bps: 2000,
        reserve_bps: 1000,
        uncertainty_bps: 500,
    };
    assert_eq!(
        e.admit(6499),
        Err(CoordinationError::InsufficientReturnEnergy)
    );
    assert!(e.admit(6500).is_ok());
    assert!(e.admit(7000).is_ok());
}
#[test]
fn consensus_falls_back_on_leader_loss_stale_term_and_partition() {
    let mut c = BoundedConsensus::new(4, 5);
    let base = ConsensusView {
        term: 4,
        leader_id: "a".into(),
        observed_tick: 100,
        voters: 2,
        cohort_size: 3,
    };
    assert_eq!(
        c.apply(&base, 101, &["a".into(), "b".into()]),
        ConsensusOutcome::CommittedInheritedPlan
    );
    let mut stale = base.clone();
    stale.term = 3;
    assert_eq!(
        c.apply(&stale, 101, &["a".into()]),
        ConsensusOutcome::HoldSafe
    );
    let mut partition = base.clone();
    partition.voters = 1;
    assert_eq!(
        c.apply(&partition, 101, &["a".into()]),
        ConsensusOutcome::HoldSafe
    );
    assert_eq!(
        c.apply(&base, 101, &["b".into()]),
        ConsensusOutcome::HoldSafe
    );
}
#[test]
fn controller_facades_preserve_failsafe_mode() {
    let cmd = FlightCommand {
        uav_id: "u".into(),
        target: Point3 {
            x_mm: 1,
            y_mm: 2,
            z_mm: 3,
        },
        mode: FlightMode::HoldThenLand,
    };
    assert_eq!(Px4Facade.encode(&cmd).frame, "NED");
    assert_eq!(ArduPilotFacade.encode(&cmd).frame, "GLOBAL_RELATIVE_ALT");
    assert_eq!(Px4Facade.encode(&cmd).mode, FlightMode::HoldThenLand);
}
#[test]
fn mappo_and_ruv_drone_are_disabled() {
    assert_eq!(
        ExperimentalCoordinator::status(),
        ExperimentalStatus::DisabledPendingPromotion
    );
}
