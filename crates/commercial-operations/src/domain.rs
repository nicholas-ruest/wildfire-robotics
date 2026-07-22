#![allow(missing_docs)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CommercialError {
    #[error("a required value was empty or invalid")]
    InvalidValue,
    #[error("tenant or region scope does not match")]
    TenantScopeMismatch,
    #[error("transition is not permitted from the current state")]
    InvalidTransition,
    #[error("effective terms overlap or are ambiguous")]
    AmbiguousTerms,
    #[error("no effective terms exist for the requested instant")]
    TermsNotEffective,
    #[error("the referenced ledger entry does not exist")]
    UnknownLedgerEntry,
    #[error("offboarding evidence is incomplete")]
    OffboardingIncomplete,
    #[error("arithmetic exceeded supported bounds")]
    ArithmeticOverflow,
    #[error("a replayed identity carried different immutable content")]
    ReplayConflict,
    #[error("meter reconciliation still contains declared gaps")]
    ReconciliationGap,
}

fn nonempty(value: &str) -> Result<String, CommercialError> {
    let value = value.trim();
    if value.is_empty() {
        Err(CommercialError::InvalidValue)
    } else {
        Ok(value.to_owned())
    }
}

/// Mandatory tenant and residency scope (`CO-INV-001`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantScope {
    tenant_id: String,
    region: String,
}
impl TenantScope {
    pub fn new(tenant_id: &str, region: &str) -> Result<Self, CommercialError> {
        Ok(Self {
            tenant_id: nonempty(tenant_id)?,
            region: nonempty(region)?,
        })
    }
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    #[must_use]
    pub fn region(&self) -> &str {
        &self.region
    }
    pub fn ensure(&self, other: &Self) -> Result<(), CommercialError> {
        if self == other {
            Ok(())
        } else {
            Err(CommercialError::TenantScopeMismatch)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationTier {
    Shared,
    DedicatedDatabase,
    DedicatedCluster,
    DedicatedAccount,
    Offline,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantState {
    Provisioning,
    Active,
    Suspended,
    Offboarding,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OffboardingEvidence {
    AccessRevoked { reference: String },
    ExportCompleted { reference: String },
    ResourcesIsolated { reference: String },
    RetentionResolved { reference: String },
    LegalHoldApplied { reference: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    id: String,
    scope: TenantScope,
    legal_name: String,
    isolation: IsolationTier,
    state: TenantState,
    evidence: Vec<OffboardingEvidence>,
    version: u64,
}
impl Tenant {
    pub fn onboard(
        id: &str,
        scope: TenantScope,
        isolation: IsolationTier,
        legal_name: &str,
    ) -> Result<Self, CommercialError> {
        Ok(Self {
            id: nonempty(id)?,
            scope,
            legal_name: nonempty(legal_name)?,
            isolation,
            state: TenantState::Provisioning,
            evidence: vec![],
            version: 0,
        })
    }
    pub fn activate(&mut self) -> Result<(), CommercialError> {
        self.transition(TenantState::Provisioning, TenantState::Active)
    }
    pub fn suspend_optional(&mut self) -> Result<(), CommercialError> {
        self.transition(TenantState::Active, TenantState::Suspended)
    }
    pub fn begin_offboarding(&mut self) -> Result<(), CommercialError> {
        if matches!(
            self.state,
            TenantState::Provisioning | TenantState::Active | TenantState::Suspended
        ) {
            self.state = TenantState::Offboarding;
            self.version += 1;
            Ok(())
        } else {
            Err(CommercialError::InvalidTransition)
        }
    }
    fn transition(&mut self, from: TenantState, to: TenantState) -> Result<(), CommercialError> {
        if self.state == from {
            self.state = to;
            self.version += 1;
            Ok(())
        } else {
            Err(CommercialError::InvalidTransition)
        }
    }
    pub fn record_offboarding(
        &mut self,
        evidence: OffboardingEvidence,
    ) -> Result<(), CommercialError> {
        if self.state != TenantState::Offboarding {
            return Err(CommercialError::InvalidTransition);
        }
        self.evidence.push(evidence);
        self.version += 1;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), CommercialError> {
        if self.state != TenantState::Offboarding || !self.offboarding_complete() {
            return Err(CommercialError::OffboardingIncomplete);
        }
        self.state = TenantState::Closed;
        self.version += 1;
        Ok(())
    }
    fn offboarding_complete(&self) -> bool {
        let access = self
            .evidence
            .iter()
            .any(|e| matches!(e, OffboardingEvidence::AccessRevoked { .. }));
        let export = self
            .evidence
            .iter()
            .any(|e| matches!(e, OffboardingEvidence::ExportCompleted { .. }));
        let isolated = self
            .evidence
            .iter()
            .any(|e| matches!(e, OffboardingEvidence::ResourcesIsolated { .. }));
        let retention = self
            .evidence
            .iter()
            .any(|e| matches!(e, OffboardingEvidence::RetentionResolved { .. }));
        access && export && isolated && retention
    }
    #[must_use]
    pub fn state(&self) -> TenantState {
        self.state
    }
    #[must_use]
    pub fn scope(&self) -> &TenantScope {
        &self.scope
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractState {
    Draft,
    Approved,
    Effective,
    Terminated,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractTerms {
    version: String,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    unit_price_minor: i64,
    currency: String,
}
impl ContractTerms {
    pub fn new(
        version: &str,
        starts_at: DateTime<Utc>,
        ends_at: Option<DateTime<Utc>>,
        unit_price_minor: i64,
        currency: &str,
    ) -> Result<Self, CommercialError> {
        if unit_price_minor < 0 || ends_at.is_some_and(|end| end <= starts_at) {
            return Err(CommercialError::InvalidValue);
        }
        Ok(Self {
            version: nonempty(version)?,
            starts_at,
            ends_at,
            unit_price_minor,
            currency: nonempty(currency)?,
        })
    }
    fn contains(&self, at: DateTime<Utc>) -> bool {
        at >= self.starts_at && self.ends_at.is_none_or(|end| at < end)
    }
    fn overlaps(&self, other: &Self) -> bool {
        self.starts_at < other.ends_at.unwrap_or(DateTime::<Utc>::MAX_UTC)
            && other.starts_at < self.ends_at.unwrap_or(DateTime::<Utc>::MAX_UTC)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    minor_units: i64,
    currency: String,
}
impl Money {
    #[must_use]
    pub fn minor_units(&self) -> i64 {
        self.minor_units
    }
    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    id: String,
    scope: TenantScope,
    currency: String,
    state: ContractState,
    terms: Vec<ContractTerms>,
    approved_by: Option<String>,
    approved_at: Option<DateTime<Utc>>,
    version: u64,
}
impl Contract {
    pub fn draft(id: &str, scope: TenantScope, currency: &str) -> Result<Self, CommercialError> {
        Ok(Self {
            id: nonempty(id)?,
            scope,
            currency: nonempty(currency)?,
            state: ContractState::Draft,
            terms: vec![],
            approved_by: None,
            approved_at: None,
            version: 0,
        })
    }
    pub fn approve(&mut self, actor: &str, at: DateTime<Utc>) -> Result<(), CommercialError> {
        if self.state != ContractState::Draft {
            return Err(CommercialError::InvalidTransition);
        }
        self.approved_by = Some(nonempty(actor)?);
        self.approved_at = Some(at);
        self.state = ContractState::Approved;
        self.version += 1;
        Ok(())
    }
    pub fn add_terms(&mut self, terms: ContractTerms) -> Result<(), CommercialError> {
        if self.state == ContractState::Terminated {
            return Err(CommercialError::InvalidTransition);
        }
        if terms.currency != self.currency {
            return Err(CommercialError::InvalidValue);
        }
        if self
            .terms
            .iter()
            .any(|t| t.version == terms.version || t.overlaps(&terms))
        {
            return Err(CommercialError::AmbiguousTerms);
        }
        self.terms.push(terms);
        self.terms.sort_by_key(|t| t.starts_at);
        self.state = ContractState::Effective;
        self.version += 1;
        Ok(())
    }
    pub fn amend(&mut self, terms: ContractTerms) -> Result<(), CommercialError> {
        self.add_terms(terms)
    }
    pub fn terminate(&mut self) -> Result<(), CommercialError> {
        if self.state == ContractState::Terminated {
            return Err(CommercialError::InvalidTransition);
        }
        self.state = ContractState::Terminated;
        self.version += 1;
        Ok(())
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub fn state(&self) -> ContractState {
        self.state
    }
    pub fn effective_terms(&self, at: DateTime<Utc>) -> Result<&ContractTerms, CommercialError> {
        let mut found = self.terms.iter().filter(|terms| terms.contains(at));
        let first = found.next().ok_or(CommercialError::TermsNotEffective)?;
        if found.next().is_some() {
            return Err(CommercialError::AmbiguousTerms);
        }
        Ok(first)
    }
    pub fn rate(&self, quantity: i64, at: DateTime<Utc>) -> Result<Money, CommercialError> {
        let matches: Vec<_> = self.terms.iter().filter(|t| t.contains(at)).collect();
        if matches.len() != 1 {
            return Err(CommercialError::TermsNotEffective);
        }
        let minor_units = quantity
            .checked_mul(matches[0].unit_price_minor)
            .ok_or(CommercialError::ArithmeticOverflow)?;
        Ok(Money {
            minor_units,
            currency: self.currency.clone(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkClass {
    OptionalNew,
    ActiveAuthorizedSafety,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkDecision {
    Permit,
    DenyOptional,
    ContinueAndReconcile,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntitlementState {
    Active,
    Suspended,
    Exhausted,
    Expired,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    id: String,
    scope: TenantScope,
    capability: String,
    quota: u64,
    reserved: u64,
    expires_at: DateTime<Utc>,
    state: EntitlementState,
    reason: Option<String>,
}
impl Entitlement {
    pub fn grant(
        id: &str,
        scope: TenantScope,
        capability: &str,
        quota: u64,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, CommercialError> {
        if quota == 0 {
            return Err(CommercialError::InvalidValue);
        }
        Ok(Self {
            id: nonempty(id)?,
            scope,
            capability: nonempty(capability)?,
            quota,
            reserved: 0,
            expires_at,
            state: EntitlementState::Active,
            reason: None,
        })
    }
    pub fn reserve(&mut self, quantity: u64, at: DateTime<Utc>) -> Result<(), CommercialError> {
        if self.decision(WorkClass::OptionalNew, at) != WorkDecision::Permit
            || self
                .reserved
                .checked_add(quantity)
                .is_none_or(|v| v > self.quota)
        {
            return Err(CommercialError::InvalidTransition);
        }
        self.reserved += quantity;
        Ok(())
    }
    pub fn release(&mut self, quantity: u64) -> Result<(), CommercialError> {
        self.reserved = self
            .reserved
            .checked_sub(quantity)
            .ok_or(CommercialError::InvalidValue)?;
        Ok(())
    }
    pub fn suspend_optional(&mut self, reason: &str) -> Result<(), CommercialError> {
        self.reason = Some(nonempty(reason)?);
        self.state = EntitlementState::Suspended;
        Ok(())
    }
    #[must_use]
    pub fn decision(&self, class: WorkClass, at: DateTime<Utc>) -> WorkDecision {
        if class == WorkClass::ActiveAuthorizedSafety {
            return WorkDecision::ContinueAndReconcile;
        }
        if at >= self.expires_at
            || self.state != EntitlementState::Active
            || self.reserved >= self.quota
        {
            WorkDecision::DenyOptional
        } else {
            WorkDecision::Permit
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageFact {
    source_id: String,
    scope: TenantScope,
    quantity: i64,
    occurred_at: DateTime<Utc>,
    content_digest: String,
}
impl UsageFact {
    pub fn new(
        source_id: &str,
        scope: TenantScope,
        quantity: i64,
        occurred_at: DateTime<Utc>,
        content_digest: &str,
    ) -> Result<Self, CommercialError> {
        if quantity <= 0 {
            return Err(CommercialError::InvalidValue);
        }
        Ok(Self {
            source_id: nonempty(source_id)?,
            scope,
            quantity,
            occurred_at,
            content_digest: nonempty(content_digest)?,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerKind {
    Usage,
    Adjustment { original_id: String, reason: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    id: String,
    source_id: String,
    quantity: i64,
    occurred_at: DateTime<Utc>,
    kind: LedgerKind,
}
impl LedgerEntry {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordOutcome {
    Recorded,
    Duplicate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeterState {
    Open,
    Closing,
    Reconciled,
    Rated,
    Invoiced,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RatingLineage {
    pub contract_version: String,
    pub currency: String,
    pub algorithm: String,
    pub rounding: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invoice {
    id: String,
    amount: Money,
    lineage: RatingLineage,
    entry_count: usize,
}
impl Invoice {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn amount(&self) -> &Money {
        &self.amount
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meter {
    id: String,
    scope: TenantScope,
    unit: String,
    entries: Vec<LedgerEntry>,
    dedup: HashMap<String, String>,
    state: MeterState,
    declared_gaps: Vec<String>,
    rating: Option<(Money, RatingLineage)>,
    invoice: Option<Invoice>,
}
impl Meter {
    pub fn open(id: &str, scope: TenantScope, unit: &str) -> Result<Self, CommercialError> {
        Ok(Self {
            id: nonempty(id)?,
            scope,
            unit: nonempty(unit)?,
            entries: vec![],
            dedup: HashMap::new(),
            state: MeterState::Open,
            declared_gaps: vec![],
            rating: None,
            invoice: None,
        })
    }
    pub fn record(&mut self, fact: UsageFact) -> Result<RecordOutcome, CommercialError> {
        self.scope.ensure(&fact.scope)?;
        if self.state != MeterState::Open {
            return Err(CommercialError::InvalidTransition);
        }
        if let Some(digest) = self.dedup.get(&fact.source_id) {
            return if digest == &fact.content_digest {
                Ok(RecordOutcome::Duplicate)
            } else {
                Err(CommercialError::ReplayConflict)
            };
        }
        self.dedup
            .insert(fact.source_id.clone(), fact.content_digest.clone());
        self.entries.push(LedgerEntry {
            id: fact.source_id.clone(),
            source_id: fact.source_id,
            quantity: fact.quantity,
            occurred_at: fact.occurred_at,
            kind: LedgerKind::Usage,
        });
        Ok(RecordOutcome::Recorded)
    }
    pub fn adjust(
        &mut self,
        source_id: &str,
        original_id: &str,
        quantity: i64,
        reason: &str,
        at: DateTime<Utc>,
    ) -> Result<RecordOutcome, CommercialError> {
        if quantity == 0 {
            return Err(CommercialError::InvalidValue);
        }
        let adjustment_digest = format!("adjustment:{original_id}:{quantity}:{reason}");
        if let Some(existing) = self.dedup.get(source_id) {
            return if existing == &adjustment_digest {
                Ok(RecordOutcome::Duplicate)
            } else {
                Err(CommercialError::ReplayConflict)
            };
        }
        if !self.entries.iter().any(|e| e.id == original_id) {
            return Err(CommercialError::UnknownLedgerEntry);
        }
        let source_id = nonempty(source_id)?;
        self.dedup.insert(source_id.clone(), adjustment_digest);
        self.entries.push(LedgerEntry {
            id: source_id.clone(),
            source_id,
            quantity,
            occurred_at: at,
            kind: LedgerKind::Adjustment {
                original_id: original_id.to_owned(),
                reason: nonempty(reason)?,
            },
        });
        Ok(RecordOutcome::Recorded)
    }
    pub fn begin_closing(&mut self, gaps: Vec<String>) -> Result<(), CommercialError> {
        if self.state != MeterState::Open {
            return Err(CommercialError::InvalidTransition);
        }
        self.declared_gaps = gaps;
        self.state = MeterState::Closing;
        Ok(())
    }
    pub fn reconcile_gap(&mut self, gap: &str) -> Result<(), CommercialError> {
        if self.state != MeterState::Closing {
            return Err(CommercialError::InvalidTransition);
        }
        self.declared_gaps.retain(|g| g != gap);
        Ok(())
    }
    pub fn reconcile(&mut self) -> Result<(), CommercialError> {
        if self.state != MeterState::Closing {
            return Err(CommercialError::InvalidTransition);
        }
        if !self.declared_gaps.is_empty() {
            return Err(CommercialError::ReconciliationGap);
        }
        self.state = MeterState::Reconciled;
        Ok(())
    }
    pub fn rate_window(
        &mut self,
        contract: &Contract,
        at: DateTime<Utc>,
    ) -> Result<(), CommercialError> {
        if self.state != MeterState::Reconciled {
            return Err(CommercialError::InvalidTransition);
        }
        self.scope.ensure(&contract.scope)?;
        let terms = contract.effective_terms(at)?;
        let amount = contract.rate(self.net_quantity(), at)?;
        self.rating = Some((
            amount,
            RatingLineage {
                contract_version: terms.version.clone(),
                currency: terms.currency.clone(),
                algorithm: "quantity-times-unit-price/v1".into(),
                rounding: "integer-minor-units-exact".into(),
            },
        ));
        self.state = MeterState::Rated;
        Ok(())
    }
    pub fn finalize_invoice(&mut self, id: &str) -> Result<&Invoice, CommercialError> {
        if self.state != MeterState::Rated {
            return Err(CommercialError::InvalidTransition);
        }
        let (amount, lineage) = self
            .rating
            .clone()
            .ok_or(CommercialError::InvalidTransition)?;
        self.invoice = Some(Invoice {
            id: nonempty(id)?,
            amount,
            lineage,
            entry_count: self.entries.len(),
        });
        self.state = MeterState::Invoiced;
        self.invoice
            .as_ref()
            .ok_or(CommercialError::InvalidTransition)
    }
    #[must_use]
    pub fn state(&self) -> MeterState {
        self.state
    }
    #[must_use]
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }
    #[must_use]
    pub fn net_quantity(&self) -> i64 {
        self.entries
            .iter()
            .fold(0_i64, |sum, e| sum.saturating_add(e.quantity))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportState {
    Open,
    Triaged,
    Investigating,
    Resolved,
    Closed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticGrant {
    id: String,
    principal: String,
    expires_at: DateTime<Utc>,
}
impl DiagnosticGrant {
    pub fn new(
        id: &str,
        principal: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, CommercialError> {
        Ok(Self {
            id: nonempty(id)?,
            principal: nonempty(principal)?,
            expires_at,
        })
    }
    #[must_use]
    pub fn permits_vehicle_commands(&self) -> bool {
        false
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportCase {
    id: String,
    scope: TenantScope,
    opened_by: String,
    severity: Severity,
    summary: String,
    state: SupportState,
    grants: Vec<DiagnosticGrant>,
}
impl SupportCase {
    pub fn open(
        id: &str,
        scope: TenantScope,
        opened_by: &str,
        severity: Severity,
        summary: &str,
    ) -> Result<Self, CommercialError> {
        Ok(Self {
            id: nonempty(id)?,
            scope,
            opened_by: nonempty(opened_by)?,
            severity,
            summary: nonempty(summary)?,
            state: SupportState::Open,
            grants: vec![],
        })
    }
    pub fn authorize_diagnostic(
        &mut self,
        request_scope: &TenantScope,
        grant: &DiagnosticGrant,
    ) -> Result<(), CommercialError> {
        self.scope.ensure(request_scope)?;
        self.grants.push(grant.clone());
        Ok(())
    }
    pub fn triage(&mut self, severity: Severity) -> Result<(), CommercialError> {
        if self.state != SupportState::Open {
            return Err(CommercialError::InvalidTransition);
        }
        self.severity = severity;
        self.state = SupportState::Triaged;
        Ok(())
    }
    pub fn escalate(&mut self) -> Result<(), CommercialError> {
        if !matches!(
            self.state,
            SupportState::Triaged | SupportState::Investigating
        ) {
            return Err(CommercialError::InvalidTransition);
        }
        self.state = SupportState::Investigating;
        Ok(())
    }
    pub fn resolve(&mut self) -> Result<(), CommercialError> {
        if self.state != SupportState::Investigating {
            return Err(CommercialError::InvalidTransition);
        }
        self.state = SupportState::Resolved;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), CommercialError> {
        if self.state != SupportState::Resolved {
            return Err(CommercialError::InvalidTransition);
        }
        self.state = SupportState::Closed;
        Ok(())
    }
    pub fn reopen(&mut self) -> Result<(), CommercialError> {
        if self.state != SupportState::Closed {
            return Err(CommercialError::InvalidTransition);
        }
        self.state = SupportState::Investigating;
        Ok(())
    }
    #[must_use]
    pub fn can_access_diagnostics(
        &self,
        principal: &str,
        now: DateTime<Utc>,
        grant: &DiagnosticGrant,
    ) -> bool {
        grant.principal == principal && now < grant.expires_at && self.grants.contains(grant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentState {
    Draft,
    Sourced,
    Modeled,
    Reviewed,
    Published,
    Superseded,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestmentFact {
    pub id: String,
    pub digest: String,
    pub observed_at: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanCounterfactual {
    pub version: String,
    pub digest: String,
    pub description: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationEvidence {
    pub reviewer: String,
    pub evidence_digest: String,
    pub published_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentCase {
    id: String,
    scope: TenantScope,
    purpose: String,
    state: InvestmentState,
    facts: Vec<InvestmentFact>,
    counterfactual: Option<HumanCounterfactual>,
    predictive_result_digest: Option<String>,
    publication: Option<PublicationEvidence>,
    version: u64,
}
impl InvestmentCase {
    pub fn create(id: &str, scope: TenantScope, purpose: &str) -> Result<Self, CommercialError> {
        Ok(Self {
            id: nonempty(id)?,
            scope,
            purpose: nonempty(purpose)?,
            state: InvestmentState::Draft,
            facts: vec![],
            counterfactual: None,
            predictive_result_digest: None,
            publication: None,
            version: 0,
        })
    }
    pub fn link_fact(&mut self, fact: InvestmentFact) -> Result<(), CommercialError> {
        if !matches!(
            self.state,
            InvestmentState::Draft | InvestmentState::Sourced
        ) {
            return Err(CommercialError::InvalidTransition);
        }
        nonempty(&fact.id)?;
        nonempty(&fact.digest)?;
        if self.facts.iter().any(|f| f.id == fact.id) {
            return Err(CommercialError::ReplayConflict);
        }
        self.facts.push(fact);
        self.state = InvestmentState::Sourced;
        self.version += 1;
        Ok(())
    }
    pub fn define_human_counterfactual(
        &mut self,
        value: HumanCounterfactual,
    ) -> Result<(), CommercialError> {
        if self.state != InvestmentState::Sourced {
            return Err(CommercialError::InvalidTransition);
        }
        nonempty(&value.version)?;
        nonempty(&value.digest)?;
        nonempty(&value.description)?;
        self.counterfactual = Some(value);
        self.version += 1;
        Ok(())
    }
    pub fn link_predictive_result(&mut self, digest: &str) -> Result<(), CommercialError> {
        if self.state != InvestmentState::Sourced
            || self.facts.is_empty()
            || self.counterfactual.is_none()
        {
            return Err(CommercialError::InvalidTransition);
        }
        self.predictive_result_digest = Some(nonempty(digest)?);
        self.state = InvestmentState::Modeled;
        self.version += 1;
        Ok(())
    }
    pub fn review(&mut self) -> Result<(), CommercialError> {
        if self.state != InvestmentState::Modeled {
            return Err(CommercialError::InvalidTransition);
        }
        self.state = InvestmentState::Reviewed;
        self.version += 1;
        Ok(())
    }
    pub fn publish(&mut self, evidence: PublicationEvidence) -> Result<(), CommercialError> {
        if self.state != InvestmentState::Reviewed {
            return Err(CommercialError::InvalidTransition);
        }
        nonempty(&evidence.reviewer)?;
        nonempty(&evidence.evidence_digest)?;
        self.publication = Some(evidence);
        self.state = InvestmentState::Published;
        self.version += 1;
        Ok(())
    }
    pub fn supersede(&mut self) -> Result<(), CommercialError> {
        if self.state != InvestmentState::Published {
            return Err(CommercialError::InvalidTransition);
        }
        self.state = InvestmentState::Superseded;
        self.version += 1;
        Ok(())
    }
    #[must_use]
    pub fn state(&self) -> InvestmentState {
        self.state
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
}
