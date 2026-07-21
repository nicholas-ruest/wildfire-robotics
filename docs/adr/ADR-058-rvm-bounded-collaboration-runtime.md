# ADR-058: RVM bounded collaboration runtime

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: runtime, collaboration, learning

## Context

Robot cohorts need isolated collaboration modules, attributable state changes, relationship graphs, and adaptive placement without allowing learned affinity to become operational authority.

## Decision

Evaluate `ruvnet/rvm` as an optional station/cohort runtime for capability-gated partitions, proof-gated transitions, hash-chained witnesses, communication edges, coherence scoring, split/merge, checkpoints, and sandboxed WASM collaboration modules. Represent observed robot relationships as time-decayed, context-labelled edges derived from successful cooperation, proximity, communications, capability complementarity, task handoff, and safety outcomes. RVM coherence may recommend locality, cohorting, or module placement; Mission Control remains authoritative for allocation and constraints. Learning uses signed outcome facts, bounded features, bias/poisoning defenses, exploration limits, rollback, and promoted policy versions. Do not deploy RVM in flight/drive safety loops until platform, timing, hardware, and assurance evidence supports it. Maintain a conventional graph/runtime fallback and pin all submodules/digests.

## Consequences

### Positive
- Provides a Rust-native isolation, witness, and graph/coherence substrate for adaptive collaboration.
### Negative
- RVM is a VM/runtime, not a proven million-robot learner; integration and assurance are substantial.
### Neutral
- Relationship scores advise collaboration and never confer identity, trust, authority, or safety approval.

## Links
- [ADR-035](ADR-035-policy-as-code-and-separation-of-duties.md)
- [ADR-053](ADR-053-million-asset-hierarchical-fleet-architecture.md)
- [RVM](https://github.com/ruvnet/rvm)
