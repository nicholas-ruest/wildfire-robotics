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
}
