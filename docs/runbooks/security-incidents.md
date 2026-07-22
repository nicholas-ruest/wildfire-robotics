# Security incident playbooks

Every playbook begins with incident declaration, safety/incident-command coordination, immutable timeline and evidence preservation. Containment may narrow authority or quarantine identities/releases but must not erase safety audit or silently interrupt an active safe state. Recovery requires current authority; restored state is not authority.

| Scenario | Detect and contain | Recover and prove |
|---|---|---|
| Signing-key compromise | Freeze promotion, disable signer, deny affected identity, inventory signatures/digests, preserve transparency/HSM audit | Rotate under independent approval, rebuild/re-scan/re-sign clean artifacts, update admission trust, revoke affected releases, verify old signer denied |
| Cloud credential theft | Revoke sessions/keys, isolate principal/workload, snapshot audit/config, restrict network and JIT | Rotate dependent credentials, reconcile GitOps/IaC, hunt persistence, verify cross-environment denial and no unauthorized data movement |
| Tenant breach | Fence tenant identities/tokens/keys, prevent exports, preserve scoped evidence, notify privacy/security owners | Rotate tenant keys, rebuild projections/caches, validate cross-tenant isolation, legal/contract notification and monitored re-enable |
| Malicious image/dependency | Block digest/repository/signer, stop rollout, quarantine workloads and builder, preserve image/SBOM/provenance | Rebuild on clean ephemeral builder, replace dependency, scan/sign/promote new digest, prove admission rejects compromised digest |
| Kubernetes compromise | Isolate cluster/account, stop GitOps promotion, revoke workload/admin identity, collect control-plane/node evidence | Recreate from reviewed desired state in clean account/cluster, restore data through recovery order, rotate trust roots, do not import unverified cluster state |
| Ransomware/backup attack | Isolate workload/admin credentials and affected stores, enforce object locks, preserve malware/forensic copies | Restore verified pre-event immutable copies to new stores; rotate keys; reconcile integrity/gaps; measure RPO/RTO; never overwrite evidence |
| Broker command injection | Pause command consumers/publishers, revoke producer identity, quarantine subjects/messages, fence command epoch | Restore facts idempotently; classify and quarantine all commands; require current authorization and new command IDs; prove no blind replay |
| Device identity compromise | Revoke certificate/trust, quarantine device and capability, notify Fleet/Safety, preserve telemetry/attestation | Re-provision hardware-backed identity after inspection, re-attest/re-certify capability, reject old credential and stale authority |
| Canadian residency breach | Stop affected replication/export/processing, preserve route/config/audit, notify privacy/legal/customer owners | Move to approved Canadian services, rotate exposed keys, verify data and backups/logs/KMS placement, delete unauthorized copies where lawful and record evidence |

Closure requires root cause, affected asset/tenant/region manifest, safety impact, notification decision, containment/recovery timestamps, evidence digests, residual risk, corrective actions, independent review, and a scheduled recurrence exercise.
