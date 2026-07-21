#![forbid(unsafe_code)]
//! identity access bounded context.

pub mod approval;
pub mod crypto;
pub mod domain;
pub mod grant;
pub mod offline;
pub mod policy;
pub mod ports;
pub mod verifier;

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "identity-access";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_name_is_stable() {
        assert_eq!(CONTEXT_NAME, "identity-access");
    }
}

#[cfg(test)]
mod domain_tests;
