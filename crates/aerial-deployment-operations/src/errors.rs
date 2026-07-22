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
}
