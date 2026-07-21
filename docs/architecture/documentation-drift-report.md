# Documentation Drift Report

- **Checked:** 2026-07-21
- **Scope:** `.plans/deep-research.md`, architecture, ADR, DDD, implementation, operations, and bounded-context source boundaries
- **Result:** All bounded-context source boundaries and their ownership metadata now exist and are machine checked. Implementation and external-evidence drift remains explicitly open because production infrastructure, verified integrations, safety evidence, commercial operations, and deployments do not yet exist.

## Automated checks performed

- Seventy-four sequential ADRs exist and are listed in the ADR index.
- Fifteen bounded contexts are specified and map one-to-one to Rust workspace crates. The shared kernel is a sixteenth, non-context technical crate.
- `docs/architecture/context-ownership.toml` records each context's crate, schema, migration namespace, deployable, accountable team, governing ADRs, and invariant namespace.
- `cargo run -p architecture-check` validates all 74 ADR numbers, all 15 context boundaries, invariant namespaces, local documentation links, unique schema ownership, and domain dependency direction.
- Every context specifies aggregate lifecycles, commands/events, owned values, numbered invariants, ports, read models, and failure policy under a common tactical standard.
- Cross-context envelopes, delivery rules, published language, process managers, compensation, and traceability are governed explicitly.
- Million-asset hierarchy, ruv-drone integration, aerial relay, authoritative lightning-plus-ML, RuPixel imagery retrieval, bounded RVM collaboration, closed-loop learning, supply-chain digital thread, ROI analytics, and vegetation robots are explicit governed requirements.
- Distributed solar microgrid habitats, million-battery charging, standardized transport pods, hybrid-electric autonomous carriers, and 100,000-robot mobilization waves are explicit and governed by a parametric physical-scale model.
- Autonomous habitat maintenance robots, small medic recovery pods, hazardous-damage quarantine, regional robot hospitals, recertification, salvage and retirement are explicit governed capabilities.
- The experimental aerial fire blanket is governed as an aircraft-independent, modular, dual-authority, progressively deployed and fully accounted R&D capability with staged material/ground/airdrop/aircraft evidence gates.
- Local ADR Markdown targets resolve.
- No changed file contains whitespace errors (`git diff --check`).
- No authored documentation/source file exceeds the repository’s 500-line limit.

## Open drift that blocks a production claim

- ADRs have no named deciders and remain proposed.
- Context crates do not yet implement the specified aggregates, invariants, workflows, persistence, adapters, contracts, or SLOs.
- A preliminary Rust toolchain and partial domain experiments exist, but the deterministic Prompt 01 quality pipeline, runtime infrastructure, data integration, digital twin, hardware integration, complete safety case, field evidence, and commercial operations do not.
- Pre-existing experimental implementations in `shared-kernel`, `safety-assurance`, and `mission-control` run ahead of their Prompt 02, 08, and 13 exit gates. They are preserved user work, are not credited as completion of those prompts, and must be reconciled against the full specifications when those prompts execute.
- The former direct Mission Control dependency on Safety Assurance domain types was replaced by a Mission-owned authorization port. A contract adapter remains deferred to the contract/application prompts.
- External licensing, regulatory, partner, hardware, and Canadian ROI-method approvals remain prerequisites.

This report must be regenerated after material code or contract changes. A passing document check never substitutes for cyber-physical validation.
