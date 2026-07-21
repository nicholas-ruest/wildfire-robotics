#![forbid(unsafe_code)]
//! station operations bounded context.

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "station-operations";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_name_is_stable() {
        assert_eq!(CONTEXT_NAME, "station-operations");
    }
}
