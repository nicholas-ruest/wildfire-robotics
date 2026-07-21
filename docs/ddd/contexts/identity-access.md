# Identity and Access Context

## Purpose

Authenticate people, workloads, devices, and approvals and enforce scoped least privilege.

## Model

- **Aggregates:** Principal, DeviceIdentity, RoleGrant, Approval.
- **Core invariant:** No shared operational identity; credentials expire/rotate; actuation uses step-up/separation of duties; revocation is auditable and distributable offline.
- **Primary workflow:** Enroll/attest -> grant scoped role -> authorize -> record approval -> rotate/revoke.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Principal | invited → active → suspended → disabled | RegisterPrincipal, VerifyIdentity, ActivatePrincipal, SuspendPrincipal | PrincipalActivated, PrincipalSuspended |
| DeviceIdentity | manufactured/imported → enrolling → trusted → quarantined/revoked/retired | EnrollDevice, AttestDevice, RotateDeviceKey, QuarantineDevice, RevokeTrust | DeviceEnrolled, DeviceTrustChanged |
| RoleGrant | requested → approved → active → expired/revoked | RequestGrant, ApproveGrant, ActivateGrant, RevokeGrant | AuthorityGrantChanged |
| Approval | requested → pending → approved/rejected → consumed/expired | RequestApproval, RecordDecision, ConsumeApproval, ExpireApproval | ApprovalGranted, ApprovalConsumed |

Owned values include principal type, verified attributes/qualification, authenticator assurance, device hardware key/attestation, trust domain, role/permission, tenant/incident/resource/geographic scope, validity, policy version, approver separation, revocation reason, and offline-bundle sequence.

## Invariants

- `IA-INV-001`: Every person/workload/device has a unique non-shared identity with verified provenance and lifecycle owner.
- `IA-INV-002`: Grants are least-privilege, scoped, time-bounded, policy-versioned, approved by authorized principals, and cannot self-approve where separation is required.
- `IA-INV-003`: Device trust requires current hardware-backed attestation and compatible non-revoked software/configuration.
- `IA-INV-004`: Approval is purpose-bound, single-use where specified, unforgeable, expiring, and cannot be replayed across scope or payload digest.
- `IA-INV-005`: Offline bundles are signed, monotonically versioned, expiring, and can retain/narrow but never expand authority after loss of the authority service.

## Ports and read models

Ports cover human identity provider/MFA, workload identity, PKI/HSM, hardware attestation, qualification source, policy decision, notification, and audit. Read models expose access inventory, expiring grants/certificates, separation conflicts, device trust, revocation distribution, and privileged activity.

## Boundary and failure policy

Publishes trust and grant changes through the [integration registry](../integration-contracts.md). Credential compromise, attestation mismatch, expired offline bundle, unavailable authority, or revocation-distribution uncertainty denies expansion, uses only current cached restrictions, quarantines affected identities where warranted, and initiates response.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
