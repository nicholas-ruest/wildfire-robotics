# Documentation Drift Report

- **Checked:** 2026-07-21
- **Scope:** `.plans/deep-research.md`, architecture, ADR, DDD, implementation, operations, and bounded-context source boundaries
- **Result:** The architecture package now contains a complete initial decision catalog and tactical domain baseline. Implementation and external-evidence drift remains explicitly open because production infrastructure, verified integrations, safety evidence, commercial operations, and deployments do not yet exist.

## Automated checks performed

- Sixty-eight sequential ADRs exist and are listed in the ADR index.
- Fourteen bounded contexts are specified. Twelve map to existing Rust workspace crates; Vegetation Management and Robot Care and Recovery are deliberately specified before their authoritative crates are created.
- Every context specifies aggregate lifecycles, commands/events, owned values, numbered invariants, ports, read models, and failure policy under a common tactical standard.
- Cross-context envelopes, delivery rules, published language, process managers, compensation, and traceability are governed explicitly.
- Million-asset hierarchy, ruv-drone integration, aerial relay, authoritative lightning-plus-ML, RuPixel imagery retrieval, bounded RVM collaboration, closed-loop learning, supply-chain digital thread, ROI analytics, and vegetation robots are explicit governed requirements.
- Distributed solar microgrid habitats, million-battery charging, standardized transport pods, hybrid-electric autonomous carriers, and 100,000-robot mobilization waves are explicit and governed by a parametric physical-scale model.
- Autonomous habitat maintenance robots, small medic recovery pods, hazardous-damage quarantine, regional robot hospitals, recertification, salvage and retirement are explicit governed capabilities.
- Local ADR Markdown targets resolve.
- No changed file contains whitespace errors (`git diff --check`).
- No authored documentation/source file exceeds the repository’s 500-line limit.

## Open drift that blocks a production claim

- ADRs have no named deciders and remain proposed.
- Context crates do not yet implement the specified aggregates, invariants, workflows, persistence, adapters, contracts, or SLOs.
- No build/test toolchain, runtime infrastructure, data integration, digital twin, hardware integration, safety case, field evidence, or commercial operations exist.
- External licensing, regulatory, partner, hardware, and Canadian ROI-method approvals remain prerequisites.

This report must be regenerated after material code or contract changes. A passing document check never substitutes for cyber-physical validation.
