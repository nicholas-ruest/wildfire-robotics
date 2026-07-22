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
}
