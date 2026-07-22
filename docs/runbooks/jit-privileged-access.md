# Just-in-time privileged access

Standing production privilege is prohibited. Normal tenant administrators cannot grant platform privilege, and security monitoring remains separate from tenant administration.

## Request and approval

1. The requester opens an immutable ticket naming incident/change, purpose, production environment, exact resources/actions, tenant or region scope, requested start and duration, and expected reconciliation.
2. Identity verifies phishing-resistant MFA, current employment/on-call status, device posture, training, and separation from the approver. The requester cannot approve their own grant.
3. An environment owner and, for protected or safety-relevant data, the data/incident authority approve the narrowest role. Maximum routine duration is one hour; extension is a new approval. Break-glass access requires a declared incident and immediate security notification.
4. Automation issues a short-lived workload/session credential bound to principal, ticket, purpose, environment, role, source device, start, expiry, and approval IDs. It cannot mint operational authority or vehicle command credentials.

## Session controls

- Record authentication, grant, commands/API calls, resources, exports, policy decisions, errors, and session end in the immutable security audit without recording secret values.
- Deny cross-environment, cross-region, cross-tenant, wildcard, delegation, credential export, and privilege-escalation attempts unless separately approved and recorded.
- Prefer read-only access. Mutating work uses reviewed commands and captures before/after state. Production GitOps mutation is emergency-only.
- Security may revoke immediately. Expiry is enforced by the target and identity provider, not only the user interface.

## Closure and reconciliation

1. Revoke the grant and active sessions; verify cached credentials and tokens fail.
2. Reconcile every direct mutation against GitOps. Revert it or submit a reviewed desired-state change; unexplained drift is an incident.
3. Attach session audit digest, changed-resource manifest, reconciliation result, evidence exports, and reviewer sign-off to the ticket.
4. Alert on overdue session, extension, failed revocation, unaudited action, scope denial, or unreconciled drift. Retain evidence under security-record policy.

Quarterly exercises must prove self-approval denial, expired/revoked credential denial, cross-environment denial, automatic expiry, audit completeness, and GitOps reconciliation.
