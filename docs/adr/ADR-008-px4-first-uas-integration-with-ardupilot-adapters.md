# ADR-008: PX4-First UAS Integration with ArduPilot Adapters

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: uas, px4, ardupilot

## Context

PX4 has strong ROS 2 integration; ArduPilot supports broader vehicle types. Neither airframe control nor a ground-control station should be rebuilt.

## Decision

Use PX4 as the primary UAS autopilot with XRCE-DDS/ROS 2 where supported, MAVLink/MAVSDK for stable supervisory integration, and QGroundControl for authorized maintenance/flight workflows. Support ArduPilot behind the same vehicle capability contract when its platform coverage is required. Pin certified configurations and separate safety-critical autopilot parameters from mission software.

## Consequences

### Positive
- Mature flight control and an adapter path across vehicle types.

### Negative
- Dual-stack compatibility and configuration assurance increase test scope.

### Neutral
- Autopilot adoption does not imply swarm or wildfire certification.

## Links
- [ADR-004](ADR-004-ros2-dds-for-robot-internal-middleware.md)
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
