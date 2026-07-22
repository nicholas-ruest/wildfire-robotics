#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]

use digital_twin::{
    BlanketState, DeterministicTwin, EvidenceCapturePort, Fault, Fidelity, HardwareFrame,
    HardwarePort, PromotionAnswer, Scenario, ScenarioLink, TwinDomain, ValidityClaim,
};

const KEY: &[u8] = b"test-only-evidence-signing-key-32";

fn scenario(id: &str, seed: u64, fault: Fault) -> Scenario {
    Scenario::new(
        id,
        seed,
        fault,
        ScenarioLink::new("REQ-29-001", "HAZ-29-001", "ADO-INV-005").unwrap(),
    )
    .unwrap()
}

#[test]
fn should_replay_identically_for_same_seed_and_scenario() {
    let scenario = scenario("SCN-NETWORK-LOSS", 42, Fault::NetworkLoss);
    let first = DeterministicTwin::standard().run(&scenario).unwrap();
    let second = DeterministicTwin::standard().run(&scenario).unwrap();
    assert_eq!(first, second);
}

#[test]
fn should_model_complete_nominal_blanket_lifecycle() {
    let result = DeterministicTwin::standard()
        .run(&scenario("SCN-NOMINAL", 41, Fault::Nominal))
        .unwrap();
    assert_eq!(result.final_blanket_state, BlanketState::Anchored);
    assert!(
        result
            .visited_blanket_states
            .contains(&BlanketState::Expanded)
    );
    assert!(
        result
            .visited_blanket_states
            .contains(&BlanketState::TerrainAligned)
    );
}

#[test]
fn should_integrate_every_required_domain() {
    let twin = DeterministicTwin::standard();
    assert_eq!(twin.domains().len(), TwinDomain::ALL.len());
}

#[test]
fn should_reach_minimum_risk_for_every_named_fault() {
    let twin = DeterministicTwin::standard();
    for (index, fault) in Fault::ALL.into_iter().enumerate() {
        let result = twin
            .run(&scenario(&format!("SCN-{index}"), index as u64 + 1, fault))
            .unwrap();
        assert!(result.minimum_risk_reached, "fault {fault:?}");
    }
}

#[test]
fn should_isolate_panel_and_account_recovery_after_entanglement() {
    let result = DeterministicTwin::standard()
        .run(&scenario("SCN-ENTANGLEMENT", 7, Fault::Entanglement))
        .unwrap();
    assert_eq!(result.final_blanket_state, BlanketState::PanelIsolated);
    assert!(result.recovery.all_components_accounted());
}

#[test]
fn should_fail_closed_when_trace_link_is_missing() {
    assert!(ScenarioLink::new("", "HAZ-29-001", "ADO-INV-005").is_err());
}

#[derive(Default)]
struct LoopbackPort(Vec<HardwareFrame>);

impl HardwarePort for LoopbackPort {
    fn exchange(&mut self, frame: HardwareFrame) -> Result<HardwareFrame, String> {
        self.0.push(frame.clone());
        Ok(frame)
    }
}

#[derive(Default)]
struct Capture(Vec<Vec<u8>>);

impl EvidenceCapturePort for Capture {
    fn capture(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.0.push(bytes.to_vec());
        Ok(())
    }
}

#[test]
fn should_capture_standardized_sil_and_hitl_evidence() {
    let scenario = scenario("SCN-CLOCK", 9, Fault::ClockLoss);
    let mut hardware = LoopbackPort::default();
    let mut capture = Capture::default();
    let result = DeterministicTwin::standard()
        .run_with_port(
            &scenario,
            Fidelity::HardwareInLoop,
            &mut hardware,
            &mut capture,
        )
        .unwrap();
    assert_eq!(hardware.0.len(), result.tick_count as usize);
    assert_eq!(capture.0.len(), result.tick_count as usize);
}

#[test]
fn should_sign_bundle_and_answer_only_simulation_promotion_query() {
    let twin = DeterministicTwin::standard();
    let scenarios = Fault::ALL
        .into_iter()
        .enumerate()
        .map(|(i, fault)| scenario(&format!("SCN-{i}"), i as u64 + 11, fault))
        .collect::<Vec<_>>();
    let bundle = twin
        .campaign(
            &scenarios,
            ValidityClaim::new("model-29.1", [0.0, 35.0], 0.85, Vec::<String>::new()).unwrap(),
            KEY,
        )
        .unwrap();
    assert!(bundle.verify(KEY));
    assert_eq!(
        bundle.promotion_answer(KEY),
        PromotionAnswer::SimulationEvidenceComplete
    );
    assert!(matches!(
        bundle.promotion_answer(b"wrong-verification-key"),
        PromotionAnswer::Incomplete { .. }
    ));
    assert!(!bundle.grants_operational_authority());
}

#[test]
fn should_report_validity_gap_and_deny_complete_answer() {
    let twin = DeterministicTwin::standard();
    let bundle = twin
        .campaign(
            &[scenario("SCN-ONE", 1, Fault::NetworkLoss)],
            ValidityClaim::new("model-29.1", [0.0, 35.0], 0.3, ["unvalidated fire model"]).unwrap(),
            KEY,
        )
        .unwrap();
    assert!(matches!(
        bundle.promotion_answer(KEY),
        PromotionAnswer::Incomplete { .. }
    ));
}

#[test]
fn should_reject_tampered_evidence_at_promotion_query() {
    let twin = DeterministicTwin::standard();
    let scenarios = Fault::ALL
        .into_iter()
        .enumerate()
        .map(|(i, fault)| scenario(&format!("SCN-{i}"), i as u64 + 100, fault))
        .collect::<Vec<_>>();
    let mut bundle = twin
        .campaign(
            &scenarios,
            ValidityClaim::new("model-29.1", [-5.0, 40.0], 0.9, Vec::<String>::new()).unwrap(),
            KEY,
        )
        .unwrap();
    bundle.results[0].compensation.push_str("tampered");
    assert!(matches!(
        bundle.promotion_answer(KEY),
        PromotionAnswer::Incomplete { .. }
    ));
}
