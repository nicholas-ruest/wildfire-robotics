//! Exact-scope evidence cases, compliance ports, and promotion process manager.
#![allow(missing_docs)]

use crate::{Digest, Hazard, OperationalDesignDomain, SafetyOccurrence};
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Exact immutable subject of an assurance decision (SA-INV-003).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionScope {
    pub release_digest: Digest,
    pub configuration_digest: Digest,
    pub hardware_digests: BTreeSet<Digest>,
    pub capability: String,
    pub capability_version: u64,
    pub odd_id: EntityId,
    pub odd_version: u64,
}

impl PromotionScope {
    pub fn new(
        release_digest: Digest,
        configuration_digest: Digest,
        hardware_digests: impl IntoIterator<Item = Digest>,
        capability: impl Into<String>,
        capability_version: u64,
        odd_id: EntityId,
        odd_version: u64,
    ) -> Result<Self, PromotionError> {
        let hardware_digests = hardware_digests.into_iter().collect::<BTreeSet<_>>();
        let capability = capability.into();
        if release_digest == [0; 32]
            || configuration_digest == [0; 32]
            || hardware_digests.is_empty()
            || hardware_digests.contains(&[0; 32])
            || capability.trim().is_empty()
            || capability_version == 0
            || odd_version == 0
        {
            return Err(PromotionError::InvalidScope);
        }
        Ok(Self {
            release_digest,
            configuration_digest,
            hardware_digests,
            capability,
            capability_version,
            odd_id,
            odd_version,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EvidenceKind {
    RequirementsTrace,
    HazardMitigation,
    Simulation,
    SoftwareInLoop,
    HardwareInLoop,
    ControlledField,
    SecurityReview,
    RollbackPlan,
    SupportPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceStatus {
    Current,
    Failed,
    Stale,
    Contradictory,
}

/// Immutable, content-addressed evidence bound to the exact promotion subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    pub id: String,
    pub kind: EvidenceKind,
    pub digest: Digest,
    pub status: EvidenceStatus,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub assumption_valid: bool,
}

impl EvidenceRecord {
    #[must_use]
    pub fn current_at(&self, now: DateTime<Utc>) -> bool {
        self.digest != [0; 32]
            && self.status == EvidenceStatus::Current
            && self.assumption_valid
            && now >= self.valid_from
            && now < self.expires_at
    }
}

/// Independent human review bound to a frozen evidence case revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndependentApproval {
    pub id: String,
    pub reviewer: EntityId,
    pub case_author: EntityId,
    pub competency: String,
    pub approved_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub case_revision: u64,
}

impl IndependentApproval {
    #[must_use]
    pub fn current_for(&self, revision: u64, now: DateTime<Utc>) -> bool {
        self.reviewer != self.case_author
            && !self.competency.trim().is_empty()
            && self.case_revision == revision
            && now >= self.approved_at
            && now < self.expires_at
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceCaseState {
    Assembling,
    Review,
    Complete,
    Approved,
    Rejected,
    Stale,
}

/// Evidence case with enumerated completeness gaps and immutable exact scope.
#[derive(Clone, Debug)]
pub struct EvidenceCase {
    pub id: EntityId,
    pub scope: PromotionScope,
    pub author: EntityId,
    pub revision: u64,
    pub state: EvidenceCaseState,
    evidence: BTreeMap<String, EvidenceRecord>,
    approval: Option<IndependentApproval>,
}

impl EvidenceCase {
    #[must_use]
    pub fn open(id: EntityId, scope: PromotionScope, author: EntityId) -> Self {
        Self {
            id,
            scope,
            author,
            revision: 1,
            state: EvidenceCaseState::Assembling,
            evidence: BTreeMap::new(),
            approval: None,
        }
    }
    pub fn link(&mut self, evidence: EvidenceRecord) -> Result<(), PromotionError> {
        if self.state != EvidenceCaseState::Assembling
            || evidence.id.trim().is_empty()
            || evidence.digest == [0; 32]
            || self.evidence.contains_key(&evidence.id)
        {
            return Err(PromotionError::InvalidEvidence);
        }
        self.evidence.insert(evidence.id.clone(), evidence);
        Ok(())
    }
    #[must_use]
    pub fn gaps(&self, now: DateTime<Utc>) -> Vec<EvidenceKind> {
        required_evidence()
            .into_iter()
            .filter(|kind| {
                !self
                    .evidence
                    .values()
                    .any(|record| record.kind == *kind && record.current_at(now))
            })
            .collect()
    }
    pub fn submit(&mut self, now: DateTime<Utc>) -> Result<(), PromotionError> {
        if self.state != EvidenceCaseState::Assembling || !self.gaps(now).is_empty() {
            return Err(PromotionError::IncompleteEvidence);
        }
        self.state = EvidenceCaseState::Review;
        Ok(())
    }
    pub fn complete_review(
        &mut self,
        approval: IndependentApproval,
        now: DateTime<Utc>,
    ) -> Result<(), PromotionError> {
        if self.state != EvidenceCaseState::Review
            || !approval.current_for(self.revision, now)
            || approval.case_author != self.author
        {
            return Err(PromotionError::InvalidApproval);
        }
        self.approval = Some(approval);
        self.state = EvidenceCaseState::Complete;
        Ok(())
    }
    pub fn approve(&mut self, now: DateTime<Utc>) -> Result<(), PromotionError> {
        if self.state != EvidenceCaseState::Complete
            || !self.gaps(now).is_empty()
            || !self
                .approval
                .as_ref()
                .is_some_and(|a| a.current_for(self.revision, now))
        {
            return Err(PromotionError::InvalidApproval);
        }
        self.state = EvidenceCaseState::Approved;
        Ok(())
    }
    pub fn reject(&mut self) -> Result<(), PromotionError> {
        if self.state != EvidenceCaseState::Review {
            return Err(PromotionError::InvalidTransition);
        }
        self.state = EvidenceCaseState::Rejected;
        Ok(())
    }
    pub fn mark_stale(&mut self) {
        self.state = EvidenceCaseState::Stale;
    }
    #[must_use]
    pub fn is_current_approved(&self, now: DateTime<Utc>) -> bool {
        self.state == EvidenceCaseState::Approved
            && self.gaps(now).is_empty()
            && self
                .approval
                .as_ref()
                .is_some_and(|a| a.current_for(self.revision, now))
    }
}

fn required_evidence() -> BTreeSet<EvidenceKind> {
    BTreeSet::from([
        EvidenceKind::RequirementsTrace,
        EvidenceKind::HazardMitigation,
        EvidenceKind::Simulation,
        EvidenceKind::SoftwareInLoop,
        EvidenceKind::HardwareInLoop,
        EvidenceKind::ControlledField,
        EvidenceKind::SecurityReview,
        EvidenceKind::RollbackPlan,
        EvidenceKind::SupportPlan,
    ])
}

/// Opaque result from a specialist-managed, replaceable compliance matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceAssessment {
    pub matrix_revision: String,
    pub applicable_items_complete: bool,
    pub specialist_approval_expires_at: DateTime<Utc>,
}

/// The domain never embeds or guesses legal obligations.
pub trait ComplianceMatrixPort {
    type Error;
    fn assess(
        &self,
        scope: &PromotionScope,
        at: DateTime<Utc>,
    ) -> Result<ComplianceAssessment, Self::Error>;
}

/// Atomic publication boundary for offline-verifiable constraint bundles.
pub trait ConstraintBundlePublisher {
    type Error;
    /// Persists and publishes the complete bundle as one logical operation.
    fn publish(&self, bundle: &ConstraintBundle) -> Result<(), Self::Error>;
}

/// Time-bounded governed item surfaced by the expiry/review query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpiringItem {
    pub id: String,
    pub kind: ExpiringKind,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ExpiringKind {
    Evidence,
    Approval,
    ConstraintBundle,
    Compliance,
}

/// Deterministic read model for review schedulers; the exclusive boundary is due.
#[must_use]
pub fn due_reviews(
    items: impl IntoIterator<Item = ExpiringItem>,
    through: DateTime<Utc>,
) -> Vec<ExpiringItem> {
    let mut due = items
        .into_iter()
        .filter(|item| item.expires_at <= through)
        .collect::<Vec<_>>();
    due.sort_by(|left, right| {
        left.expires_at
            .cmp(&right.expires_at)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    due
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PromotionBlocker {
    Evidence,
    Approval,
    Hazard,
    Occurrence,
    Compliance,
    Odd,
    ConfigurationChanged,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromotionStatus {
    Candidate,
    Approved,
    Suspended,
    Narrowed,
    RolledBack,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromotionOutcome {
    Promoted,
    Blocked(BTreeSet<PromotionBlocker>),
    Suspended(BTreeSet<PromotionBlocker>),
    NarrowOdd,
    Rollback,
}

/// Stateful, exact-scope process manager. Blockers dominate approvals and never auto-restore.
#[derive(Clone, Debug)]
pub struct PromotionProcessManager {
    pub scope: PromotionScope,
    pub status: PromotionStatus,
    version: u64,
}

impl PromotionProcessManager {
    #[must_use]
    pub const fn new(scope: PromotionScope) -> Self {
        Self {
            scope,
            status: PromotionStatus::Candidate,
            version: 1,
        }
    }
    pub fn evaluate<C: ComplianceMatrixPort>(
        &mut self,
        case: &EvidenceCase,
        hazards: &[Hazard],
        odd: &OperationalDesignDomain,
        occurrences: &[SafetyOccurrence],
        compliance: &C,
        now: DateTime<Utc>,
    ) -> PromotionOutcome {
        let mut blockers = BTreeSet::new();
        if case.scope != self.scope || !case.is_current_approved(now) {
            blockers.insert(PromotionBlocker::Evidence);
        }
        if hazards.iter().any(|h| !h.promotion_ready_at(now)) {
            blockers.insert(PromotionBlocker::Hazard);
        }
        if !odd.is_approved()
            || odd.id() != &self.scope.odd_id
            || odd.version() != self.scope.odd_version
        {
            blockers.insert(PromotionBlocker::Odd);
        }
        if occurrences
            .iter()
            .any(|o| o.blocks_scope(&self.scope.capability))
        {
            blockers.insert(PromotionBlocker::Occurrence);
        }
        match compliance.assess(&self.scope, now) {
            Ok(v) if v.applicable_items_complete && now < v.specialist_approval_expires_at => {}
            _ => {
                blockers.insert(PromotionBlocker::Compliance);
            }
        }
        self.version = self.version.saturating_add(1);
        if blockers.is_empty() {
            self.status = PromotionStatus::Approved;
            PromotionOutcome::Promoted
        } else {
            self.status = if self.status == PromotionStatus::Approved {
                PromotionStatus::Suspended
            } else {
                PromotionStatus::Candidate
            };
            if self.status == PromotionStatus::Suspended {
                PromotionOutcome::Suspended(blockers)
            } else {
                PromotionOutcome::Blocked(blockers)
            }
        }
    }
    pub fn configuration_changed(&mut self, observed: Digest) -> PromotionOutcome {
        if observed == self.scope.configuration_digest {
            return if self.status == PromotionStatus::Approved {
                PromotionOutcome::Promoted
            } else {
                PromotionOutcome::Blocked(BTreeSet::from([PromotionBlocker::Approval]))
            };
        }
        self.status = PromotionStatus::Suspended;
        self.version = self.version.saturating_add(1);
        PromotionOutcome::Suspended(BTreeSet::from([PromotionBlocker::ConfigurationChanged]))
    }
    pub fn narrow_odd(&mut self) -> PromotionOutcome {
        self.status = PromotionStatus::Narrowed;
        self.version = self.version.saturating_add(1);
        PromotionOutcome::NarrowOdd
    }
    pub fn rollback(&mut self) -> PromotionOutcome {
        self.status = PromotionStatus::RolledBack;
        self.version = self.version.saturating_add(1);
        PromotionOutcome::Rollback
    }
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }
}

/// Signed, content-addressed bundle suitable for atomic offline publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstraintBundle {
    pub sequence: u64,
    pub scope_digest: Digest,
    pub constraint_digests: BTreeSet<Digest>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signer: EntityId,
    pub signature: String,
}
impl ConstraintBundle {
    pub fn new(
        sequence: u64,
        scope_digest: Digest,
        constraint_digests: impl IntoIterator<Item = Digest>,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        signer: EntityId,
        signature: impl Into<String>,
    ) -> Result<Self, PromotionError> {
        let constraint_digests = constraint_digests.into_iter().collect::<BTreeSet<_>>();
        let signature = signature.into();
        if sequence == 0
            || scope_digest == [0; 32]
            || constraint_digests.is_empty()
            || constraint_digests.contains(&[0; 32])
            || created_at >= expires_at
            || signature.trim().len() < 16
        {
            return Err(PromotionError::InvalidBundle);
        }
        Ok(Self {
            sequence,
            scope_digest,
            constraint_digests,
            created_at,
            expires_at,
            signer,
            signature,
        })
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum PromotionError {
    #[error("promotion scope is invalid")]
    InvalidScope,
    #[error("evidence record is invalid")]
    InvalidEvidence,
    #[error("evidence is incomplete, stale, failed, or contradictory")]
    IncompleteEvidence,
    #[error("independent approval is invalid or expired")]
    InvalidApproval,
    #[error("invalid evidence-case transition")]
    InvalidTransition,
    #[error("constraint bundle is invalid")]
    InvalidBundle,
}
