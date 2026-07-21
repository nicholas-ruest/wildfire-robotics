#![forbid(unsafe_code)]
//! suppression operations bounded context.

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "suppression-operations";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_name_is_stable() {
        assert_eq!(CONTEXT_NAME, "suppression-operations");
    }
}
