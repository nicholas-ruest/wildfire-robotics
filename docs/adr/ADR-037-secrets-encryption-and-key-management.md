# ADR-037: Secrets, encryption, and key management

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: security, cryptography

## Context

Credentials and sensitive operational data must remain protected in cloud, station, vehicle, backup, and support paths.

## Decision

Store secrets only in approved secret managers or hardware-backed device stores; never in source, images, logs, telemetry, or ordinary configuration. Encrypt data in transit and at rest using approved current algorithms. Central KMS/HSM protects service and signing keys; envelope encryption enables scoped rotation and erasure. Define key ownership, purpose, access policy, rotation period, compromise playbook, escrow prohibition/exception, backup, destruction evidence, and cryptographic inventory. Secret scanning and key-policy tests block release.

## Consequences

### Positive
- Limits credential exposure and supplies auditable cryptographic controls.
### Negative
- Key loss or unavailable KMS requires carefully tested degraded behavior.
### Neutral
- Encryption does not replace minimization or authorization.

## Links
- [ADR-034](ADR-034-workload-device-identity-and-pki.md)
- [ADR-028](ADR-028-data-classification-retention-and-deletion-policy.md)
