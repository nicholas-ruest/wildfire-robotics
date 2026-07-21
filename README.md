# Wildfire Robotics

Canada-first, safety-gated wildfire intelligence and robotics platform.

The authoritative implementation is a Rust 2024 Cargo workspace. Rust owns domain logic,
services, edge/robot gateways, safety controls, ingestion, simulation, and platform tooling.
TypeScript is reserved for the future browser operator console and generated clients under
[ADR-016](docs/adr/ADR-016-rust-first-implementation-language.md).

## Developer verification

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p architecture-check
```

Architecture and production gates are documented in [docs/architecture](docs/architecture/README.md).
