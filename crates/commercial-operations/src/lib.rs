#![forbid(unsafe_code)]
//! commercial operations bounded context.

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "commercial-operations";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_name_is_stable() {
        assert_eq!(CONTEXT_NAME, "commercial-operations");
    }
}
