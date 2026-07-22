#![allow(missing_docs, clippy::unwrap_used)]
use aerial_deployment_operations::{
    AerialPayloadInterface, AircraftConfigurationId, AircraftIdentity, AircraftInterfaceVersion,
    ComponentId, ComponentLoad, DomainError, EvidenceBackedAircraftAdapter, EvidenceId,
    EvidenceRef, LoadEnvelope, LoadKind, Mass, PayloadManifest, PayloadManifestId, Position3,
    ReconciliationState,
};
use chrono::{TimeZone, Utc};

fn evidence(expires: i64) -> EvidenceRef {
    EvidenceRef::new(
        EvidenceId::new("aircraft-load-analysis-7").unwrap(),
        &format!("sha256:{}", "a".repeat(64)),
        "evidence://engineering/load-analysis/7",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Some(Utc.timestamp_opt(expires, 0).single().unwrap()),
    )
    .unwrap()
}

#[test]
fn should_compute_mass_moment_and_cg_with_conservative_uncertainty() {
    let manifest = PayloadManifest::draft(
        PayloadManifestId::new("manifest-1").unwrap(),
        vec![
            ComponentLoad::new(
                ComponentId::new("a").unwrap(),
                "serial-a",
                Mass::new(100.0, 1.0).unwrap(),
                Position3::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
            ComponentLoad::new(
                ComponentId::new("b").unwrap(),
                "serial-b",
                Mass::new(300.0, 3.0).unwrap(),
                Position3::new(3.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    assert!(
        (manifest.mass_balance().unwrap().total_mass.nominal_kg() - 400.0).abs() < f64::EPSILON
    );
    assert!((manifest.mass_balance().unwrap().longitudinal_cg_m - 2.5).abs() < f64::EPSILON);
}

#[test]
fn should_reject_stale_evidence_and_interface_mismatch() {
    let manifest = PayloadManifest::draft(
        PayloadManifestId::new("m").unwrap(),
        vec![
            ComponentLoad::new(
                ComponentId::new("a").unwrap(),
                "s",
                Mass::new(10.0, 0.1).unwrap(),
                Position3::new(0.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let envelope = LoadEnvelope::new(
        AircraftIdentity::new(
            "C17-fixture",
            "tail-fixture",
            AircraftConfigurationId::new("cfg").unwrap(),
        )
        .unwrap(),
        AircraftInterfaceVersion::new(2, 0),
        evidence(20),
        Mass::new(100.0, 0.0).unwrap(),
        vec![LoadKind::Floor {
            station: "A".into(),
            maximum_kg_per_m2: 1000.0,
        }],
    )
    .unwrap();
    assert_eq!(
        manifest.validate_for(
            &envelope,
            AircraftInterfaceVersion::new(1, 0),
            Utc.timestamp_opt(15, 0).single().unwrap()
        ),
        Err(DomainError::InterfaceVersionMismatch)
    );
    assert_eq!(
        manifest.validate_for(
            &envelope,
            AircraftInterfaceVersion::new(2, 0),
            Utc.timestamp_opt(20, 0).single().unwrap()
        ),
        Err(DomainError::AircraftEvidenceStale)
    );
    assert_eq!(
        manifest.validate_for(
            &envelope,
            AircraftInterfaceVersion::new(2, 0),
            Utc.timestamp_opt(15, 0).single().unwrap()
        ),
        Err(DomainError::IncompleteLoadingPlan)
    );
}

#[test]
fn should_require_exact_serial_reconciliation() {
    let mut manifest = PayloadManifest::draft(
        PayloadManifestId::new("m").unwrap(),
        vec![
            ComponentLoad::new(
                ComponentId::new("a").unwrap(),
                "serial-a",
                Mass::new(10.0, 0.0).unwrap(),
                Position3::new(0.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(
        manifest.reconcile(ReconciliationState::Inspected, &["other"]),
        Err(DomainError::ManifestSubstitution)
    );
}

#[test]
fn should_reject_unapproved_floor_load_and_accept_ordered_reconciliation() {
    use aerial_deployment_operations::{
        AircraftLoadingConstraints, MechanicalInterface, PackedGeometry, ServiceLimits,
    };
    use std::collections::BTreeSet;
    let mut manifest = PayloadManifest::draft(
        PayloadManifestId::new("m").unwrap(),
        vec![
            ComponentLoad::new(
                ComponentId::new("a").unwrap(),
                "serial-a",
                Mass::new(10.0, 0.0).unwrap(),
                Position3::new(0.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    manifest
        .set_loading_plan(
            PackedGeometry::new(1.0, 1.0, 1.0).unwrap(),
            vec![LoadKind::Floor {
                station: "A".into(),
                maximum_kg_per_m2: 51.0,
            }],
            vec!["A".into()],
            MechanicalInterface {
                restraint_revision: "r1".into(),
                extraction_revision: "e1".into(),
            },
            ServiceLimits {
                electrical_revision: "p1".into(),
                maximum_voltage_v: 28.0,
                maximum_current_a: 10.0,
                data_revision: "d1".into(),
                maximum_data_rate_mbps: 100.0,
                environmental_revision: "env1".into(),
                minimum_temperature_c: -40.0,
                maximum_temperature_c: 60.0,
            },
            vec![],
        )
        .unwrap();
    let envelope = LoadEnvelope::new(
        AircraftIdentity::new("type", "tail", AircraftConfigurationId::new("cfg").unwrap())
            .unwrap(),
        AircraftInterfaceVersion::new(1, 0),
        evidence(100),
        Mass::new(20.0, 0.0).unwrap(),
        vec![LoadKind::Floor {
            station: "A".into(),
            maximum_kg_per_m2: 50.0,
        }],
    )
    .unwrap()
    .with_constraints(AircraftLoadingConstraints {
        maximum_geometry: PackedGeometry::new(2.0, 2.0, 2.0).unwrap(),
        longitudinal_cg_range_m: (-1.0, 1.0),
        maximum_absolute_moments_kg_m: Position3::new(100.0, 100.0, 100.0).unwrap(),
        mechanical_interface: MechanicalInterface {
            restraint_revision: "r1".into(),
            extraction_revision: "e1".into(),
        },
        service_limits: ServiceLimits {
            electrical_revision: "p1".into(),
            maximum_voltage_v: 28.0,
            maximum_current_a: 10.0,
            data_revision: "d1".into(),
            maximum_data_rate_mbps: 100.0,
            environmental_revision: "env1".into(),
            minimum_temperature_c: -40.0,
            maximum_temperature_c: 60.0,
        },
        loading_stations: BTreeSet::from(["A".into()]),
        permitted_hazardous_contents: BTreeSet::new(),
    })
    .unwrap();
    assert_eq!(
        manifest.validate_for(
            &envelope,
            AircraftInterfaceVersion::new(1, 0),
            Utc.timestamp_opt(50, 0).single().unwrap()
        ),
        Err(DomainError::LoadEnvelopeExceeded)
    );
    manifest
        .reconcile(ReconciliationState::Inspected, &["serial-a"])
        .unwrap();
    manifest
        .reconcile(ReconciliationState::Loaded, &["serial-a"])
        .unwrap();
    manifest
        .reconcile(ReconciliationState::Retained, &["serial-a"])
        .unwrap();
    assert_eq!(manifest.state(), ReconciliationState::Retained);
}

#[test]
fn should_scope_approved_evidence_to_exact_aircraft_tail_and_configuration() {
    let approved = AircraftIdentity::new(
        "type",
        "tail-1",
        AircraftConfigurationId::new("cfg-1").unwrap(),
    )
    .unwrap();
    let adapter = EvidenceBackedAircraftAdapter::new(
        LoadEnvelope::new(
            approved,
            AircraftInterfaceVersion::new(1, 0),
            evidence(100),
            Mass::new(20.0, 0.0).unwrap(),
            vec![],
        )
        .unwrap(),
    );
    let substituted = AircraftIdentity::new(
        "type",
        "tail-2",
        AircraftConfigurationId::new("cfg-1").unwrap(),
    )
    .unwrap();
    assert_eq!(
        adapter.approved_envelope(&substituted, Utc.timestamp_opt(50, 0).single().unwrap()),
        Err(DomainError::EvidenceMismatch)
    );
}

#[test]
fn should_reject_cg_outside_evidence_controlled_loading_envelope() {
    use aerial_deployment_operations::{
        AircraftLoadingConstraints, MechanicalInterface, PackedGeometry, ServiceLimits,
    };
    use std::collections::BTreeSet;
    let mechanical = MechanicalInterface {
        restraint_revision: "r1".into(),
        extraction_revision: "x1".into(),
    };
    let services = ServiceLimits {
        electrical_revision: "e1".into(),
        maximum_voltage_v: 28.0,
        maximum_current_a: 2.0,
        data_revision: "d1".into(),
        maximum_data_rate_mbps: 10.0,
        environmental_revision: "env1".into(),
        minimum_temperature_c: -20.0,
        maximum_temperature_c: 50.0,
    };
    let mut manifest = PayloadManifest::draft(
        PayloadManifestId::new("cg-manifest").unwrap(),
        vec![
            ComponentLoad::new(
                ComponentId::new("a").unwrap(),
                "serial-a",
                Mass::new(10.0, 0.1).unwrap(),
                Position3::new(2.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    manifest
        .set_loading_plan(
            PackedGeometry::new(1.0, 1.0, 1.0).unwrap(),
            vec![LoadKind::Floor {
                station: "A".into(),
                maximum_kg_per_m2: 10.0,
            }],
            vec!["A".into()],
            mechanical.clone(),
            services.clone(),
            vec![],
        )
        .unwrap();
    let envelope = LoadEnvelope::new(
        AircraftIdentity::new("type", "tail", AircraftConfigurationId::new("cfg").unwrap())
            .unwrap(),
        AircraftInterfaceVersion::new(1, 0),
        evidence(100),
        Mass::new(20.0, 0.0).unwrap(),
        vec![LoadKind::Floor {
            station: "A".into(),
            maximum_kg_per_m2: 20.0,
        }],
    )
    .unwrap()
    .with_constraints(AircraftLoadingConstraints {
        maximum_geometry: PackedGeometry::new(2.0, 2.0, 2.0).unwrap(),
        longitudinal_cg_range_m: (-1.0, 1.0),
        maximum_absolute_moments_kg_m: Position3::new(1000.0, 1000.0, 1000.0).unwrap(),
        mechanical_interface: mechanical,
        service_limits: services,
        loading_stations: BTreeSet::from(["A".into()]),
        permitted_hazardous_contents: BTreeSet::new(),
    })
    .unwrap();
    assert_eq!(
        manifest.validate_for(
            &envelope,
            AircraftInterfaceVersion::new(1, 0),
            Utc.timestamp_opt(50, 0).single().unwrap()
        ),
        Err(DomainError::LoadEnvelopeExceeded)
    );
}
