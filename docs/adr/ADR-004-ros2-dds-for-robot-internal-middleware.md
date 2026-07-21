# ADR-004: ROS 2/DDS for Robot-Internal Middleware

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: robotics, ros2, dds

## Context

The platform needs mature robotics messaging, tooling, simulation integration, and hardware ecosystem support. ROS 2 does not provide enterprise fleet orchestration.

## Decision

Use supported ROS 2 LTS distributions and DDS within robots and controlled incident-edge domains. Define strict QoS, namespace, time, frame, lifecycle, security, and message compatibility profiles. Bridge only curated telemetry and commands through a Fleet Gateway; never stretch a ROS domain across the public internet or treat ROS 2 as fleet management.

## Consequences

### Positive
- Adopts a mature robotics ecosystem while containing its trust and scaling boundary.

### Negative
- Gateway and profile conformance require dedicated engineering.

### Neutral
- Non-ROS devices participate through adapters.

## Links
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
- [ADR-008](ADR-008-px4-first-uas-integration-with-ardupilot-adapters.md)
