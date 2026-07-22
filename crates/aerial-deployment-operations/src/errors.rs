use thiserror::Error;
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("required value is empty")]
    Empty,
    #[error("identifier contains unsupported characters or exceeds 128 bytes")]
    InvalidIdentifier,
    #[error("digest must be a lowercase sha256 value")]
    InvalidDigest,
    #[error("evidence reference is incomplete or temporally invalid")]
    InvalidEvidence,
    #[error("scope is incomplete")]
    InvalidScope,
    #[error("a configuration binding is incomplete or contains duplicate items")]
    InvalidConfiguration,
    #[error("the requested qualification stage is not the next stage")]
    QualificationStageSkipped,
    #[error("qualification evidence does not bind the exact configuration and stage")]
    EvidenceMismatch,
    #[error("qualification evidence is expired or not yet valid")]
    EvidenceExpired,
    #[error("qualification is suspended until all invalidating conditions are resolved")]
    QualificationSuspended,
    #[error("qualification evidence contains unexplained variance")]
    UnexplainedVariance,
    #[error("qualification evidence has an unresolved occurrence")]
    UnresolvedOccurrence,
    #[error("the lifecycle transition is not permitted")]
    InvalidTransition,
    #[error("the serialized component is absent or duplicated")]
    InvalidComponent,
    #[error("a physical quantity is non-finite, negative, or dimensionally invalid")]
    InvalidQuantity,
    #[error("the aircraft interface version does not match the approved contract")]
    InterfaceVersionMismatch,
    #[error("authoritative aircraft engineering evidence is missing or stale")]
    AircraftEvidenceStale,
    #[error("the manifest exceeds an approved aircraft or loading interface limit")]
    LoadEnvelopeExceeded,
    #[error("the inspected component serials differ from the planned manifest")]
    ManifestSubstitution,
    #[error("the manifest reconciliation transition is invalid")]
    InvalidReconciliation,
    #[error("the payload loading plan or approved aircraft constraints are incomplete")]
    IncompleteLoadingPlan,
    #[error("the mission binding is incomplete, ambiguous, or inconsistent")]
    InvalidMissionBinding,
    #[error("the source observation is absent, stale, or below its required confidence")]
    UnsafeOrStaleObservation,
    #[error("the release decision does not bind the current canonical command digest")]
    ReleaseDigestMismatch,
    #[error("the release decision is a hold, veto, or abort")]
    ReleaseInhibited,
    #[error("the aggregate version is stale")]
    VersionConflict,
    #[error("the command was already applied with different content")]
    ReplayConflict,
    #[error("the requested contingency was not pre-authorized or is not least-harm")]
    ContingencyNotAuthorized,
    #[error("the point of no return or release boundary forbids this operation")]
    OperationalBoundaryCrossed,
    #[error("an airborne transition margin is absent, unsafe, or stale")]
    UnsafeTransitionMargin,
    #[error("the requested local cohort is absent or exceeds its bounded limit")]
    InvalidCohort,
    #[error("safe-sector evidence is required before jettison")]
    SafeSectorNotConfirmed,
    #[error("advisory output is malformed")]
    InvalidAdvisory,
    #[error("the ground zone is absent or exceeds the installation's bounded limit")]
    InvalidGroundZone,
    #[error("ground work is inhibited by an unsafe, uncertain, or stale prerequisite")]
    GroundWorkInhibited,
    #[error("the sensor suite is incomplete, uncertain beyond its bound, or stale")]
    GroundSensingInhibited,
    #[error("the requested ground policy action is not bounded for the affected zone")]
    GroundPolicyInhibited,
    #[error("the recovery record is incomplete or inconsistent")]
    InvalidRecoveryRecord,
    #[error("the serialized item is not present in the recovery ledger")]
    UnknownSerializedItem,
    #[error("the item cannot be closed without authority, search evidence, and a hazard notice")]
    RecoveryClosureInhibited,
    #[error("the external handoff acknowledgement is invalid or conflicts with the request")]
    InvalidHandoff,
}
