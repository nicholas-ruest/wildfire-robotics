#![allow(clippy::expect_used, missing_docs)]

use aerial_deployment_operations::*;
use chrono::{DateTime, Duration, Utc};

const DIGEST_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SIGNATURE: &str = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn at() -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).expect("fixture timestamp is valid")
}

fn revision<T>(item: T) -> RevisionBinding<T> {
    RevisionBinding::new(item, "revision-1").expect("fixture revision is valid")
}

fn binding(id: &str, digest: &str, odd: &str) -> ConfigurationBinding {
    ConfigurationBinding::new(
        BlanketConfigurationId::new(id).expect("fixture id is valid"),
        digest,
        MaterialRevisionId::new("material-r1").expect("fixture id is valid"),
        vec![revision(PanelId::new("panel-1").expect("valid"))],
        vec![revision(JointId::new("joint-1").expect("valid"))],
        vec![revision(VentId::new("vent-1").expect("valid"))],
        vec![revision(AnchorId::new("anchor-1").expect("valid"))],
        vec![revision(TetherId::new("tether-1").expect("valid"))],
        vec![revision(ReelId::new("reel-1").expect("valid"))],
        vec![revision(ParafoilId::new("parafoil-1").expect("valid"))],
        vec![revision(CradleId::new("cradle-1").expect("valid"))],
        vec![revision(RobotId::new("robot-1").expect("valid"))],
        "geometry-r1",
        "mass-properties-r1",
        OddId::new(odd).expect("fixture id is valid"),
    )
    .expect("complete binding is valid")
}

fn evidence(
    digest: &str,
    stage: QualificationStage,
    expires_at: DateTime<Utc>,
    variance: TestVariance,
    occurrence_resolved: bool,
) -> SignedQualificationEvidence {
    let artifact = EvidenceRef::new(
        EvidenceId::new(&format!("evidence-{stage:?}")).expect("fixture id is valid"),
        DIGEST_A,
        "object://qualification/artifact",
        at() - Duration::hours(1),
        Some(expires_at),
    )
    .expect("fixture evidence is valid");
    SignedQualificationEvidence::new(
        artifact,
        digest,
        stage,
        "independent-qualification-authority",
        SIGNATURE,
        variance,
        occurrence_resolved,
    )
    .expect("fixture signature is valid")
}

fn promote(configuration: &mut BlanketConfiguration, target: QualificationStage) {
    configuration
        .promote(
            target,
            vec![evidence(
                DIGEST_A,
                target,
                at() + Duration::hours(1),
                TestVariance::None,
                true,
            )],
            at(),
        )
        .expect("sequential promotion succeeds");
}

#[test]
fn should_bind_every_exact_revision_and_reject_duplicate_or_missing_components() {
    let valid = binding("configuration-a", DIGEST_A, "odd-a");
    assert_eq!(valid.panels()[0].revision(), "revision-1");

    let duplicate_panels = vec![
        revision(PanelId::new("panel-1").expect("valid")),
        revision(PanelId::new("panel-1").expect("valid")),
    ];
    let result = ConfigurationBinding::new(
        BlanketConfigurationId::new("bad").expect("valid"),
        DIGEST_A,
        MaterialRevisionId::new("material-r1").expect("valid"),
        duplicate_panels,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        "geometry-r1",
        "mass-r1",
        OddId::new("odd-a").expect("valid"),
    );
    assert_eq!(result, Err(DomainError::InvalidConfiguration));
}

#[test]
fn should_walk_the_entire_ladder_without_skipping_or_regressing() {
    let mut configuration =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    let ladder = [
        QualificationStage::CouponMaterial,
        QualificationStage::Component,
        QualificationStage::GroundMultiPanel,
        QualificationStage::LowDrop,
        QualificationStage::SubscaleExtraction,
        QualificationStage::Sil,
        QualificationStage::Hitl,
        QualificationStage::InstrumentedRange,
        QualificationStage::AircraftGroundExtraction,
        QualificationStage::PartialScaleFlight,
        QualificationStage::FullSystemCandidate,
    ];
    for stage in ladder {
        promote(&mut configuration, stage);
        assert_eq!(configuration.stage(), stage);
    }
    assert_eq!(
        configuration.promote(QualificationStage::PartialScaleFlight, Vec::new(), at()),
        Err(DomainError::QualificationStageSkipped)
    );
}

#[test]
fn should_reject_stage_skip_and_evidence_for_another_exact_configuration() {
    let mut configuration =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    assert_eq!(
        configuration.promote(QualificationStage::Component, Vec::new(), at()),
        Err(DomainError::QualificationStageSkipped)
    );
    assert_eq!(
        configuration.promote(
            QualificationStage::CouponMaterial,
            vec![evidence(
                DIGEST_B,
                QualificationStage::CouponMaterial,
                at() + Duration::hours(1),
                TestVariance::None,
                true,
            )],
            at(),
        ),
        Err(DomainError::EvidenceMismatch)
    );
}

#[test]
fn should_treat_expiry_as_an_exclusive_boundary_and_suspend_existing_qualification() {
    let mut configuration =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    let expiring = evidence(
        DIGEST_A,
        QualificationStage::CouponMaterial,
        at(),
        TestVariance::None,
        true,
    );
    assert_eq!(
        configuration.promote(QualificationStage::CouponMaterial, vec![expiring], at()),
        Err(DomainError::EvidenceExpired)
    );
    promote(&mut configuration, QualificationStage::CouponMaterial);
    assert!(!configuration.evaluate_evidence(at() + Duration::hours(1)));
    assert!(matches!(
        configuration.status(),
        QualificationStatus::Suspended(reasons)
            if reasons.contains(&SuspensionReason::ExpiredEvidence)
    ));
}

#[test]
fn should_reject_unexplained_variance_and_unresolved_occurrences() {
    for (variance, occurrence, expected) in [
        (
            TestVariance::Unexplained,
            true,
            DomainError::UnexplainedVariance,
        ),
        (
            TestVariance::Explained,
            false,
            DomainError::UnresolvedOccurrence,
        ),
    ] {
        let mut configuration =
            BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
                .expect("valid configuration");
        let result = configuration.promote(
            QualificationStage::CouponMaterial,
            vec![evidence(
                DIGEST_A,
                QualificationStage::CouponMaterial,
                at() + Duration::hours(1),
                variance,
                occurrence,
            )],
            at(),
        );
        assert_eq!(result, Err(expected));
    }
}

#[test]
fn should_require_requalification_after_suspension_or_changed_odd() {
    let mut configuration =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    promote(&mut configuration, QualificationStage::CouponMaterial);
    configuration.record_changed_odd();
    assert_eq!(
        configuration.promote(QualificationStage::Component, Vec::new(), at()),
        Err(DomainError::QualificationSuspended)
    );
    configuration
        .requalify(
            vec![evidence(
                DIGEST_A,
                QualificationStage::CouponMaterial,
                at() + Duration::hours(2),
                TestVariance::Explained,
                true,
            )],
            at(),
        )
        .expect("fresh exact-stage evidence restores qualification");
    assert_eq!(configuration.status(), &QualificationStatus::Active);
}

#[test]
fn should_not_preload_future_stage_evidence_during_requalification() {
    let mut configuration =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    promote(&mut configuration, QualificationStage::CouponMaterial);
    configuration.record_changed_odd();
    assert_eq!(
        configuration.requalify(
            vec![
                evidence(
                    DIGEST_A,
                    QualificationStage::CouponMaterial,
                    at() + Duration::hours(2),
                    TestVariance::None,
                    true,
                ),
                evidence(
                    DIGEST_A,
                    QualificationStage::Component,
                    at() + Duration::hours(2),
                    TestVariance::None,
                    true,
                ),
            ],
            at(),
        ),
        Err(DomainError::EvidenceMismatch)
    );
}

#[test]
fn should_never_inherit_stage_or_evidence_after_substitution() {
    let mut original =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    promote(&mut original, QualificationStage::CouponMaterial);
    let changed = original.substitute(binding("configuration-b", DIGEST_B, "odd-a"));
    assert_eq!(changed.stage(), QualificationStage::Concept);
    assert!(matches!(
        changed.status(),
        QualificationStatus::Suspended(reasons)
            if reasons.contains(&SuspensionReason::Substitution)
    ));
    let mut changed = changed;
    changed
        .begin_substitution_requalification()
        .expect("explicit assessment begins a new ladder");
    changed
        .promote(
            QualificationStage::CouponMaterial,
            vec![evidence(
                DIGEST_B,
                QualificationStage::CouponMaterial,
                at() + Duration::hours(1),
                TestVariance::None,
                true,
            )],
            at(),
        )
        .expect("new exact evidence starts the replacement ladder");
}

#[test]
fn should_track_panel_isolation_sacrificial_release_and_serialized_recovery() {
    let configuration =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    let mut assembly = MembraneAssembly::assemble(
        AssemblyId::new("assembly-a").expect("valid"),
        &configuration,
        vec![
            SerializedComponent::new("panel-serial-1", MembraneComponentKind::Panel)
                .expect("valid"),
            SerializedComponent::new("robot-serial-1", MembraneComponentKind::Robot)
                .expect("valid"),
        ],
    )
    .expect("unique serialized inventory");
    assembly
        .isolate_panel("panel-serial-1")
        .expect("panel exists");
    assembly
        .sacrificial_release("panel-serial-1")
        .expect("release is recorded");
    assembly
        .transition_recovery("robot-serial-1", RecoveryState::Deployed)
        .expect("deployment is recorded");
    assembly
        .transition_recovery("robot-serial-1", RecoveryState::SearchPending)
        .expect("search is recorded");
    assembly
        .transition_recovery("robot-serial-1", RecoveryState::Located)
        .expect("location is recorded");
    assembly
        .transition_recovery("robot-serial-1", RecoveryState::Quarantined)
        .expect("quarantine is recorded");
    assembly
        .transition_recovery("robot-serial-1", RecoveryState::Recovered)
        .expect("recovery is recorded");
    assert!(assembly.components()[0].isolated);
    assert_eq!(
        assembly.components()[0].recovery,
        RecoveryState::SacrificiallyReleased
    );
    assert_eq!(assembly.components()[1].recovery, RecoveryState::Recovered);
}

#[test]
fn should_reject_duplicate_serials_and_illegal_recovery_transitions() {
    let configuration =
        BlanketConfiguration::register(binding("configuration-a", DIGEST_A, "odd-a"))
            .expect("valid configuration");
    let duplicate = MembraneAssembly::assemble(
        AssemblyId::new("assembly-a").expect("valid"),
        &configuration,
        vec![
            SerializedComponent::new("same", MembraneComponentKind::Panel).expect("valid"),
            SerializedComponent::new("same", MembraneComponentKind::Robot).expect("valid"),
        ],
    );
    assert_eq!(duplicate, Err(DomainError::InvalidComponent));

    let mut assembly = MembraneAssembly::assemble(
        AssemblyId::new("assembly-b").expect("valid"),
        &configuration,
        vec![SerializedComponent::new("panel-1", MembraneComponentKind::Panel).expect("valid")],
    )
    .expect("valid assembly");
    assert_eq!(
        assembly.transition_recovery("panel-1", RecoveryState::Recovered),
        Err(DomainError::InvalidTransition)
    );
}
