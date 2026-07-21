# ADR-047: Vehicle firmware, OTA, and fleet compatibility

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: vehicle, firmware, operations

## Context

Mixed hardware and intermittently connected fleets require safe, recoverable software and firmware lifecycle management.

## Decision

Maintain signed hardware/software bills of material and a compatibility matrix for controller, bootloader, firmware, OS, ROS, adapter, payload, configuration, and capability. Updates use authenticated staged rollout, prerequisite checks, power/connectivity gates, A/B or equivalent recoverable installation, post-update attestation, health observation, and automatic rollback where safe. Active missions prohibit non-emergency updates. Unsupported or vulnerable combinations lose capability eligibility. Physical service procedures cover devices that cannot recover remotely.

## Consequences

### Positive
- Prevents incompatible fleet states and limits update blast radius.
### Negative
- Long-lived mixed versions and vendor constraints increase testing.
### Neutral
- Rollback may also require configuration and data compatibility.

## Links
- [ADR-008](ADR-008-px4-first-uas-integration-with-ardupilot-adapters.md)
- [ADR-038](ADR-038-secure-software-supply-chain.md)
