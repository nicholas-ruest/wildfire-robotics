#![forbid(unsafe_code)]
//! Generated bindings are isolated here so domain crates never depend on
//! transport representations. Adapters may map these wire types at context
//! boundaries.

pub mod conformance;

/// Generated bindings for the `wildfire.v1` Protobuf package.
#[allow(missing_docs, clippy::all, clippy::pedantic)]
pub mod wildfire {
    /// Version 1 of the canonical published language.
    #[allow(missing_docs, clippy::doc_markdown, clippy::must_use_candidate)]
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/wildfire.v1.rs"));
    }
}

/// Descriptor set for registry inspection and reflection-based tooling.
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wildfire_descriptor.bin"));
