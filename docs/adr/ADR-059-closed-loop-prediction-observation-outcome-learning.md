# ADR-059: Closed-loop prediction, observation, and outcome learning

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: machine-learning, drones, evidence

## Context

The platform must improve lightning ignition, fire evolution, route, resource, and collaboration predictions from what drones and robots subsequently observe.

## Decision

Implement a lineage-complete loop: authoritative inputs → immutable baseline/ML prediction → uncertainty-aware reconnaissance value → authorized drone/robot observation mission → calibrated media/telemetry/outcome ingestion → RuPixel-assisted retrieval and dataset construction → human/algorithmic label with confidence → prediction/outcome alignment → error and operational-utility analysis → candidate retraining → offline/shadow/champion-challenger validation → approved model/policy promotion → monitored rollout. Dataset snapshots are immutable and split by incident, geography, time, and fire year to prevent leakage. Record misses, unobserved/censored areas, sampling policy, interventions, and negative outcomes to limit feedback and survivorship bias. No online learning directly changes operational models, robot relationships, routes, or autonomy authority.

## Consequences

### Positive
- Creates measurable learning from predictions and real-world outcomes.
### Negative
- Causal attribution, delayed labels, biased reconnaissance, and changing sensors are difficult.
### Neutral
- Iteration improves only when evaluation demonstrates generalization and operational value.

## Links
- [ADR-046](ADR-046-digital-twin-scenario-and-test-evidence.md)
- [ADR-056](ADR-056-authoritative-lightning-intelligence-and-ml-enhancement.md)
- [ADR-057](ADR-057-rupixel-visual-evidence-and-outcome-retrieval.md)
