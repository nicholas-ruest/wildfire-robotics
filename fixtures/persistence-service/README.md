# Expand–migrate–contract fixture

This fixture demonstrates ADR-041 with a two-version compatibility window. Each immutable migration owns one phase, acquires the context advisory lock, and bounds PostgreSQL lock and statement time. The application fixture demonstrates legacy read, dual read/write, new-authoritative read, idempotent backfill, monotonic checkpointing, and evidence-gated contract.

Run `cargo test --manifest-path fixtures/persistence-service/Cargo.toml` and `cargo run --manifest-path tools/migration-check/Cargo.toml`. Production promotion additionally requires measured production-scale rehearsal and real backup, retention/legal-review, fleet-version, and restore evidence; the identifiers here are explicitly fixture-only.
