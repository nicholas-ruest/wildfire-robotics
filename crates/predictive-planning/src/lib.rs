#![forbid(unsafe_code)]
//! predictive planning bounded context.

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "predictive-planning";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_name_is_stable() {
        assert_eq!(CONTEXT_NAME, "predictive-planning");
    }
}
