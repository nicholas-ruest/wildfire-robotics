#![allow(missing_docs, clippy::unwrap_used)]
use suppression_operations::*;
fn envelope() -> ActuationEnvelope {
    ActuationEnvelope::new(
        Fence {
            min_x_mm: 0,
            max_x_mm: 10000,
            min_y_mm: 0,
            max_y_mm: 10000,
        },
        5000,
        1000,
        500,
        8000,
        ToolKind::WaterNozzle,
    )
    .unwrap()
}
fn authority() -> AuthoritySnapshot {
    AuthoritySnapshot {
        incident_id: "i".into(),
        mission_id: "m".into(),
        fence_digest: [1; 32],
        issued_tick: 10,
        expires_tick: 20,
    }
}
fn prepared_with(envelope: ActuationEnvelope, target_point: Point) -> Operation {
    let candidate = Target::<Candidate>::candidate("t", target_point);
    let verified = candidate
        .verify(VerificationEvidence {
            digest: [2; 32],
            uncertainty_mm: 10,
        })
        .unwrap();
    let approved = verified
        .approve(TargetApproval {
            authority_id: "i".into(),
            digest: [3; 32],
        })
        .unwrap();
    let plan = SuppressionPlan::new(
        "p",
        approved,
        envelope,
        CapabilitySnapshot {
            id: "cap".into(),
            promoted: true,
            odd_valid: true,
            digest: [4; 32],
        },
        AgentBatch {
            id: "water-1".into(),
            approved: true,
            digest: [5; 32],
        },
    )
    .unwrap();
    Operation::prepare("op", plan, OperationMode::Simulation).unwrap()
}
fn prepared() -> Operation {
    prepared_with(
        envelope(),
        Point {
            x_mm: 100,
            y_mm: 100,
        },
    )
}
fn stop() -> StopAttestation {
    StopAttestation {
        channel_id: "independent-stop".into(),
        config_digest: [8; 32],
        proof_tick: 14,
        expires_tick: 18,
        response_bound_ms: 50,
        healthy: true,
        readback: true,
    }
}
fn approvals(digest: [u8; 32]) -> [ApprovalEvidence; 2] {
    [
        ApprovalEvidence::qualified(
            "operator",
            "cred-op",
            "i",
            "m",
            "suppression-arm",
            14,
            18,
            30,
            true,
            digest,
            [6; 32],
        ),
        ApprovalEvidence::qualified(
            "supervisor",
            "cred-sup",
            "i",
            "m",
            "suppression-arm",
            14,
            18,
            30,
            false,
            digest,
            [7; 32],
        ),
    ]
}
fn armed() -> Operation {
    let mut op = prepared();
    let digest = op.arming_digest(&authority(), 15).unwrap();
    op.arm(&authority(), 15, approvals(digest), &stop())
        .unwrap();
    op
}
#[test]
fn typed_target_cannot_skip_verification_and_approval() {
    let c = Target::<Candidate>::candidate("t", Point { x_mm: 0, y_mm: 0 });
    assert_eq!(
        c.verify(VerificationEvidence {
            digest: [0; 32],
            uncertainty_mm: 1
        })
        .unwrap_err(),
        SuppressionError::InvalidTarget
    );
}
#[test]
fn arming_requires_current_authority_capability_batch_and_two_distinct_approvers() {
    let mut op = armed();
    assert_eq!(op.state(), OperationState::Armed);
    op.inhibit(InhibitReason::LostSupervision, 15);
    assert_eq!(op.state(), OperationState::Inhibited);
}
#[test]
fn arming_rejects_stale_or_unhealthy_stop_proof() {
    for mutate in 0..2 {
        let mut op = prepared();
        let d = op.arming_digest(&authority(), 15).unwrap();
        let mut s = stop();
        if mutate == 0 {
            s.expires_tick = 15;
        } else {
            s.healthy = false;
        }
        assert_eq!(
            op.arm(&authority(), 15, approvals(d), &s).unwrap_err(),
            SuppressionError::InvalidApproval
        );
    }
}
#[test]
fn arming_rejects_identity_requester_scope_purpose_digest_and_expiry_failures() {
    for case in 0..7 {
        let mut op = prepared();
        let d = op.arming_digest(&authority(), 15).unwrap();
        let mut a = approvals(d);
        match case {
            0 => a[1].principal = a[0].principal.clone(),
            1 => a[1].credential_id = a[0].credential_id.clone(),
            2 => a[1].requester = true,
            3 => a[1].mission_id = "wrong".into(),
            4 => a[1].purpose = "observe".into(),
            5 => a[1].arming_digest = [0; 32],
            _ => a[1].qualification_expires_tick = 15,
        }
        assert_eq!(
            op.arm(&authority(), 15, a, &stop()).unwrap_err(),
            SuppressionError::InvalidApproval
        );
    }
}
#[test]
fn canonical_arming_digest_changes_for_envelope_target_and_authority() {
    let base = prepared().arming_digest(&authority(), 15).unwrap();
    let changed_envelope = ActuationEnvelope::new(
        Fence {
            min_x_mm: 0,
            max_x_mm: 10000,
            min_y_mm: 0,
            max_y_mm: 10000,
        },
        4999,
        1000,
        500,
        8000,
        ToolKind::WaterNozzle,
    )
    .unwrap();
    assert_ne!(
        base,
        prepared_with(
            changed_envelope,
            Point {
                x_mm: 100,
                y_mm: 100
            }
        )
        .arming_digest(&authority(), 15)
        .unwrap()
    );
    assert_ne!(
        base,
        prepared_with(
            envelope(),
            Point {
                x_mm: 101,
                y_mm: 100
            }
        )
        .arming_digest(&authority(), 15)
        .unwrap()
    );
    let mut a = authority();
    a.fence_digest = [9; 32];
    assert_ne!(base, prepared().arming_digest(&a, 15).unwrap());
}
#[test]
fn every_fault_latches_and_records_occurrence() {
    for reason in [
        InhibitReason::Intrusion,
        InhibitReason::Uncertainty,
        InhibitReason::SpatialBreach,
        InhibitReason::EnvironmentalBreach,
        InhibitReason::DoseBreach,
        InhibitReason::RateBreach,
        InhibitReason::PressureBreach,
        InhibitReason::ToolFault,
        InhibitReason::SensorFault,
        InhibitReason::ActuatorFault,
        InhibitReason::LostSupervision,
        InhibitReason::StopRequested,
    ] {
        let mut op = armed();
        op.inhibit(reason, 16);
        assert_eq!(op.state(), OperationState::Inhibited);
        assert_eq!(op.occurrences().last().unwrap().reason, reason);
    }
}
#[test]
fn continuously_monitors_envelope_and_stops_flow() {
    let mut op = armed();
    let mut normal = SimulatedActuator::default();
    let mut stop = SimulatedIndependentInhibit::default();
    op.apply(
        Command {
            target: Point {
                x_mm: 100,
                y_mm: 100,
            },
            dose_ml: 6000,
            rate_ml_s: 100,
            pressure_kpa: 100,
            tool: ToolKind::WaterNozzle,
        },
        Measurements {
            position: Point {
                x_mm: 100,
                y_mm: 100,
            },
            environment_bps: 100,
            supervision_fresh: true,
            sensor_ok: true,
            uncertainty_mm: 1,
        },
        16,
        &mut normal,
        &mut stop,
    )
    .unwrap_err();
    assert_eq!(normal.flow_ml, 0);
    assert!(stop.stop_calls > 0);
}
#[test]
fn independent_stop_failure_is_a_recorded_safe_failure() {
    let mut op = armed();
    let mut normal = SimulatedActuator::default();
    let mut stop = SimulatedIndependentInhibit {
        fail: true,
        ..Default::default()
    };
    assert_eq!(
        op.emergency_stop(16, &mut normal, &mut stop).unwrap_err(),
        SuppressionError::IndependentStopFailed
    );
    assert_eq!(normal.flow_ml, 0);
    assert_eq!(op.state(), OperationState::InhibitUnconfirmed);
    assert!(
        op.occurrences()
            .iter()
            .any(|o| o.reason == InhibitReason::IndependentStopFault)
    );
}
#[test]
fn cumulative_dose_and_actuator_failure_invoke_independent_stop() {
    let mut op = armed();
    let mut normal = SimulatedActuator::default();
    let mut stop = SimulatedIndependentInhibit::default();
    let command = Command {
        target: Point {
            x_mm: 100,
            y_mm: 100,
        },
        dose_ml: 3000,
        rate_ml_s: 100,
        pressure_kpa: 100,
        tool: ToolKind::WaterNozzle,
    };
    let m = Measurements {
        position: command.target,
        environment_bps: 1,
        supervision_fresh: true,
        sensor_ok: true,
        uncertainty_mm: 1,
    };
    op.apply(command, m, 16, &mut normal, &mut stop).unwrap();
    assert_eq!(
        op.apply(command, m, 17, &mut normal, &mut stop)
            .unwrap_err(),
        SuppressionError::EnvelopeBreach
    );
    let mut op = armed();
    let mut failed = SimulatedActuator {
        fail: true,
        ..Default::default()
    };
    let mut independent = SimulatedIndependentInhibit::default();
    assert!(
        op.apply(command, m, 16, &mut failed, &mut independent)
            .is_err()
    );
    assert_eq!(independent.stop_calls, 1);
    assert_eq!(
        op.occurrences().last().unwrap().reason,
        InhibitReason::ActuatorFault
    );
}
#[test]
fn commanded_and_measured_effects_are_append_only_with_uncertainty() {
    let mut op = armed();
    let mut normal = SimulatedActuator::default();
    let mut stop = SimulatedIndependentInhibit::default();
    op.apply(
        Command {
            target: Point {
                x_mm: 100,
                y_mm: 100,
            },
            dose_ml: 100,
            rate_ml_s: 50,
            pressure_kpa: 100,
            tool: ToolKind::WaterNozzle,
        },
        Measurements {
            position: Point {
                x_mm: 101,
                y_mm: 99,
            },
            environment_bps: 100,
            supervision_fresh: true,
            sensor_ok: true,
            uncertainty_mm: 5,
        },
        16,
        &mut normal,
        &mut stop,
    )
    .unwrap();
    op.record_effect(MeasuredEffect {
        measured_ml: 95,
        uncertainty_ml: 3,
        residual_flow_ml: 2,
        evidence_digest: [9; 32],
    })
    .unwrap();
    assert_eq!(op.effects().len(), 1);
    assert_eq!(op.effects()[0].commanded_ml, 100);
}
#[test]
fn production_autonomy_is_not_available() {
    assert_eq!(
        Operation::prepare("x", armed().plan().clone(), OperationMode::Unsupervised).unwrap_err(),
        SuppressionError::ModeForbidden
    );
}
