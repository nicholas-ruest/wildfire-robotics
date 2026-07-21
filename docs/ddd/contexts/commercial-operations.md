# Commercial Operations Context

## Purpose

Manage tenants, contracts, entitlements, metering, economics, and support without entering the safety control path.

## Model

- **Aggregates:** Tenant, Contract, Entitlement, Meter, SupportCase.
- **Core invariant:** Tenant isolation is universal; usage is immutable; commercial state cannot delay/revoke an active safety action; contractual retention is enforced.
- **Primary workflow:** Onboard -> configure isolation/entitlement -> consume usage facts -> rate/report -> support/offboard.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Tenant | prospect → provisioning → active → suspended → offboarding → closed | OnboardTenant, ActivateTenant, SuspendOptionalWork, BeginOffboarding, CloseTenant | TenantActivated, TenantOffboarded |
| Contract | draft → approved → effective → amended/expired/terminated | CreateContract, ApproveContract, AmendContract, TerminateContract | ContractBecameEffective |
| Entitlement | scheduled → active → exhausted/suspended → expired | GrantEntitlement, ChangeEntitlement, ReserveQuota, ReleaseQuota | EntitlementChanged |
| Meter | open → closing → reconciled → rated → invoiced | RecordUsage, CloseWindow, ReconcileUsage, RateUsage, FinalizeInvoice | UsageRated, InvoiceFinalized |
| SupportCase | open → triaged → investigating → resolved/closed/reopened | OpenCase, SetSeverity, EscalateCase, RecordResolution, CloseCase | SupportEscalated, SupportCaseResolved |
| InvestmentCase | draft → sourced → modeled → reviewed → published → superseded | CreateInvestmentCase, LinkCostFacts, DefineCounterfactual, RunScenario, PublishInvestmentCase | InvestmentCasePublished |

Owned values include legal customer/account, region/residency, isolation tier, product catalog/contract version, effective terms, entitlement/quota, immutable usage ledger, price/tax/currency, invoice/adjustment, support service level, consented diagnostic access, maintenance/warranty obligation, and offboarding evidence. Investment cases own decision purpose, strategies, human-capital counterfactual, lifecycle horizon, cost/outcome fact manifests, assumptions, uncertainty/sensitivity and linked Predictive Planning result.

## Invariants

- `CO-INV-001`: Tenant scope is mandatory and isolated in every owned record, ledger entry, job, export, and support action.
- `CO-INV-002`: Contract, entitlement, price, tax, and currency rules are effective-dated and historical rating is reproducible.
- `CO-INV-003`: Usage is immutable, deduplicated, source-linked, reconcilable, and adjusted by compensating entries rather than mutation.
- `CO-INV-004`: Billing, entitlement, suspension, or support failure cannot interrupt or revoke an active authorized safety action.
- `CO-INV-005`: Offboarding proves access revocation, authorized export, resource isolation, and retention/deletion/legal-hold outcomes.
- `CO-INV-006`: A published investment case is reproducible from immutable actual-cost/usage/outcome facts and a versioned model; it clearly separates observed facts, assumptions, predictions, and causal claims.

## Ports and read models

Ports integrate CRM, contract/e-signature, tax, payment/accounting, customer identity, provisioning, status/notification, support/field service, and data lifecycle. Read models expose tenant health, entitlement/quota, usage lineage, revenue/cost, invoices/disputes, SLA performance, assets/warranties, and offboarding progress.

## Boundary and failure policy

Owns tenant onboarding/offboarding, usage rating, billing, and support processes under ADR-050/051. Metering lag, billing/tax failure, entitlement disagreement, payment failure, or support outage continues authorized incident safety operations, blocks only optional new work when policy permits, and reconciles asynchronously.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
