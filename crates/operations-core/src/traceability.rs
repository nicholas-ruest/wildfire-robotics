//! Immutable governed records and fail-closed assurance traceability queries.

use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Digest of canonical record bytes.
pub type RecordDigest = [u8; 32];

/// Governed artifact classes used by promotion decisions.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArtifactKind {
    /// Protected source revision.
    Source,
    /// Reproducible build output.
    BuildArtifact,
    /// Software bill of materials.
    Sbom,
    /// Deployable release.
    Release,
    /// Exact runtime configuration.
    Configuration,
    /// Immutable model release.
    Model,
    /// Product capability.
    Capability,
    /// Requirement with acceptance measure.
    Requirement,
    /// Executable domain invariant.
    Invariant,
    /// Governed hazard.
    Hazard,
    /// Governed threat.
    Threat,
    /// Architecture decision.
    Decision,
    /// Executed test.
    Test,
    /// Content-addressed evidence.
    Evidence,
    /// Approved operational design domain.
    OperationalDomain,
    /// Exact deployed state.
    Deployment,
}

/// Immutable identity and content hash for one artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernedArtifact {
    /// Namespace-qualified immutable ID.
    pub id: String,
    /// Artifact class.
    pub kind: ArtifactKind,
    /// Digest of canonical content.
    pub digest: RecordDigest,
    /// Creation instant.
    pub created_at: DateTime<Utc>,
}

impl GovernedArtifact {
    /// Creates a validated artifact identity.
    pub fn new(
        id: impl Into<String>,
        kind: ArtifactKind,
        digest: RecordDigest,
        created_at: DateTime<Utc>,
    ) -> Result<Self, TraceabilityError> {
        let id = id.into();
        validate_text(&id)?;
        validate_digest(digest)?;
        Ok(Self {
            id,
            kind,
            digest,
            created_at,
        })
    }
}

/// Current trust state of a traceability relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceStatus {
    /// Link is current and mutually consistent.
    Current,
    /// Evidence or an assumption passed its review boundary.
    Stale,
    /// Linked facts disagree.
    Contradictory,
    /// Link was explicitly revoked.
    Revoked,
}

/// Directed relationship from a release to supporting material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceLink {
    /// Source artifact ID.
    pub from: String,
    /// Target artifact ID.
    pub to: String,
    /// Governed status.
    pub status: EvidenceStatus,
    /// Last verification instant.
    pub verified_at: DateTime<Utc>,
    /// Exclusive review expiry.
    pub expires_at: Option<DateTime<Utc>>,
}

impl EvidenceLink {
    /// Creates a current link.
    #[must_use]
    pub fn current(
        from: impl Into<String>,
        to: impl Into<String>,
        verified_at: DateTime<Utc>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            status: EvidenceStatus::Current,
            verified_at,
            expires_at: None,
        }
    }
}

/// Approval category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalKind {
    /// Independent safety assurance review.
    IndependentSafety,
    /// Security review.
    Security,
    /// Operational release authority.
    Operations,
}

/// Attributable, scoped, expiring human approval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approval {
    /// Immutable approval ID.
    pub id: String,
    /// Exact subject artifact.
    pub subject_id: String,
    /// Human reviewer identity.
    pub reviewer: String,
    /// Review category.
    pub kind: ApprovalKind,
    /// Authority scope.
    pub authority: String,
    /// Decision instant.
    pub approved_at: DateTime<Utc>,
    /// Exclusive validity expiry.
    pub expires_at: DateTime<Utc>,
    /// Explicit revocation state.
    pub revoked: bool,
}

impl Approval {
    /// Creates a validated active approval.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        subject_id: impl Into<String>,
        reviewer: impl Into<String>,
        kind: ApprovalKind,
        authority: impl Into<String>,
        approved_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, TraceabilityError> {
        let value = Self {
            id: id.into(),
            subject_id: subject_id.into(),
            reviewer: reviewer.into(),
            kind,
            authority: authority.into(),
            approved_at,
            expires_at,
            revoked: false,
        };
        for field in [
            &value.id,
            &value.subject_id,
            &value.reviewer,
            &value.authority,
        ] {
            validate_text(field)?;
        }
        if approved_at >= expires_at {
            return Err(TraceabilityError::InvalidRecord);
        }
        Ok(value)
    }
}

/// Exact inputs to the mechanical promotion query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionRequest {
    /// Candidate release.
    pub release_id: String,
    /// Candidate configuration.
    pub configuration_id: String,
    /// Capability being promoted.
    pub capability_id: String,
    /// Approved operational domain.
    pub odd_id: String,
    /// Exact hardware revisions.
    pub hardware_ids: Vec<String>,
    /// Accountable promotion authority.
    pub authority: String,
    /// Evaluation instant.
    pub evaluated_at: DateTime<Utc>,
}

impl PromotionRequest {
    /// Convenience constructor for deterministic adapters and tests.
    #[must_use]
    pub fn fixture(
        release: &str,
        configuration: &str,
        capability: &str,
        odd: &str,
        authority: &str,
        evaluated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            release_id: release.into(),
            configuration_id: configuration.into(),
            capability_id: capability.into(),
            odd_id: odd.into(),
            hardware_ids: vec!["fixture-hardware".into()],
            authority: authority.into(),
            evaluated_at,
        }
    }
}

/// Successful, auditable answer to the promotion query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionDecision {
    /// Exact release approved by the query.
    pub release_id: String,
    /// Exact configuration.
    pub configuration_id: String,
    /// Exact capability.
    pub capability_id: String,
    /// Exact ODD.
    pub odd_id: String,
    /// Exact hardware revisions.
    pub hardware_ids: Vec<String>,
    /// Approval IDs authorizing promotion.
    pub approval_ids: Vec<String>,
    /// Content-addressed artifacts traversed by the decision.
    pub artifact_digests: BTreeMap<String, RecordDigest>,
}

/// Framework-neutral evidence graph domain model.
#[derive(Clone, Debug, Default)]
pub struct EvidenceGraph {
    artifacts: BTreeMap<String, GovernedArtifact>,
    links: BTreeMap<(String, String), EvidenceLink>,
    approvals: BTreeMap<String, Approval>,
}

impl EvidenceGraph {
    /// Creates an empty graph.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            artifacts: BTreeMap::new(),
            links: BTreeMap::new(),
            approvals: BTreeMap::new(),
        }
    }

    /// Inserts an immutable artifact, rejecting identifier reuse.
    pub fn insert_artifact(&mut self, artifact: GovernedArtifact) -> Result<(), TraceabilityError> {
        if self
            .artifacts
            .insert(artifact.id.clone(), artifact)
            .is_some()
        {
            return Err(TraceabilityError::DuplicateIdentifier);
        }
        Ok(())
    }

    /// Inserts a link only when both endpoints exist.
    pub fn insert_link(&mut self, link: EvidenceLink) -> Result<(), TraceabilityError> {
        if !self.artifacts.contains_key(&link.from) {
            return Err(TraceabilityError::UnknownArtifact(link.from));
        }
        if !self.artifacts.contains_key(&link.to) {
            return Err(TraceabilityError::UnknownArtifact(link.to));
        }
        let key = (link.from.clone(), link.to.clone());
        if self.links.insert(key, link).is_some() {
            return Err(TraceabilityError::DuplicateIdentifier);
        }
        Ok(())
    }

    /// Inserts an attributable approval without allowing ID reuse.
    pub fn insert_approval(&mut self, approval: Approval) -> Result<(), TraceabilityError> {
        if !self.artifacts.contains_key(&approval.subject_id) {
            return Err(TraceabilityError::UnknownArtifact(approval.subject_id));
        }
        if self
            .approvals
            .insert(approval.id.clone(), approval)
            .is_some()
        {
            return Err(TraceabilityError::DuplicateIdentifier);
        }
        Ok(())
    }

    /// Applies a governed status update to a known relationship.
    pub fn set_link_status(
        &mut self,
        from: &str,
        to: &str,
        status: EvidenceStatus,
    ) -> Result<(), TraceabilityError> {
        let link = self
            .links
            .get_mut(&(from.to_owned(), to.to_owned()))
            .ok_or_else(|| TraceabilityError::MissingLink {
                release_id: from.into(),
                kind: None,
            })?;
        link.status = status;
        Ok(())
    }

    /// Answers the release promotion query or fails closed.
    #[allow(clippy::too_many_lines)]
    pub fn evaluate_promotion(
        &self,
        request: &PromotionRequest,
    ) -> Result<PromotionDecision, TraceabilityError> {
        for field in [
            &request.release_id,
            &request.configuration_id,
            &request.capability_id,
            &request.odd_id,
            &request.authority,
        ] {
            validate_text(field)?;
        }
        if request.hardware_ids.is_empty()
            || request
                .hardware_ids
                .iter()
                .any(|id| validate_text(id).is_err())
        {
            return Err(TraceabilityError::MissingHardware);
        }
        self.require_kind(&request.release_id, ArtifactKind::Release)?;
        self.require_kind(&request.configuration_id, ArtifactKind::Configuration)?;
        self.require_kind(&request.capability_id, ArtifactKind::Capability)?;
        self.require_kind(&request.odd_id, ArtifactKind::OperationalDomain)?;

        let release_links: Vec<_> = self
            .links
            .values()
            .filter(|link| link.from == request.release_id)
            .collect();
        for link in &release_links {
            if link.status == EvidenceStatus::Stale
                || link
                    .expires_at
                    .is_some_and(|expiry| request.evaluated_at >= expiry)
            {
                return Err(TraceabilityError::StaleLink {
                    from: link.from.clone(),
                    to: link.to.clone(),
                });
            }
            if link.status == EvidenceStatus::Contradictory {
                return Err(TraceabilityError::ContradictoryLink {
                    from: link.from.clone(),
                    to: link.to.clone(),
                });
            }
            if link.status == EvidenceStatus::Revoked {
                return Err(TraceabilityError::RevokedLink {
                    from: link.from.clone(),
                    to: link.to.clone(),
                });
            }
        }

        let required = [
            ArtifactKind::Source,
            ArtifactKind::BuildArtifact,
            ArtifactKind::Sbom,
            ArtifactKind::Configuration,
            ArtifactKind::Capability,
            ArtifactKind::Requirement,
            ArtifactKind::Invariant,
            ArtifactKind::Hazard,
            ArtifactKind::Threat,
            ArtifactKind::Decision,
            ArtifactKind::Test,
            ArtifactKind::Evidence,
            ArtifactKind::OperationalDomain,
            ArtifactKind::Deployment,
        ];
        let linked_kinds: BTreeSet<_> = release_links
            .iter()
            .filter_map(|link| self.artifacts.get(&link.to).map(|artifact| artifact.kind))
            .collect();
        for kind in required {
            if !linked_kinds.contains(&kind) {
                return Err(TraceabilityError::MissingLink {
                    release_id: request.release_id.clone(),
                    kind: Some(kind),
                });
            }
        }
        for exact in [
            &request.configuration_id,
            &request.capability_id,
            &request.odd_id,
        ] {
            if !release_links.iter().any(|link| &link.to == exact) {
                return Err(TraceabilityError::WrongPromotionScope((*exact).clone()));
            }
        }

        let approvals: Vec<_> = self
            .approvals
            .values()
            .filter(|approval| {
                approval.subject_id == request.release_id
                    && approval.kind == ApprovalKind::IndependentSafety
                    && approval.authority == request.authority
                    && !approval.revoked
                    && approval.approved_at <= request.evaluated_at
                    && request.evaluated_at < approval.expires_at
                    && approval.reviewer != request.authority
            })
            .collect();
        if approvals.is_empty() {
            return Err(TraceabilityError::Unapproved(request.release_id.clone()));
        }
        let artifact_digests = std::iter::once(&request.release_id)
            .chain(release_links.iter().map(|link| &link.to))
            .filter_map(|id| {
                self.artifacts
                    .get(id)
                    .map(|artifact| (artifact.id.clone(), artifact.digest))
            })
            .collect();
        Ok(PromotionDecision {
            release_id: request.release_id.clone(),
            configuration_id: request.configuration_id.clone(),
            capability_id: request.capability_id.clone(),
            odd_id: request.odd_id.clone(),
            hardware_ids: request.hardware_ids.clone(),
            approval_ids: approvals
                .iter()
                .map(|approval| approval.id.clone())
                .collect(),
            artifact_digests,
        })
    }

    fn require_kind(&self, id: &str, expected: ArtifactKind) -> Result<(), TraceabilityError> {
        let artifact = self
            .artifacts
            .get(id)
            .ok_or_else(|| TraceabilityError::UnknownArtifact(id.into()))?;
        if artifact.kind != expected {
            return Err(TraceabilityError::WrongArtifactKind {
                id: id.into(),
                expected,
            });
        }
        Ok(())
    }
}

macro_rules! immutable_record {
    ($name:ident, $doc:literal, { $($field:ident),+ $(,)? }) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            /// Immutable record identifier.
            pub id: String,
            $(#[doc = concat!("Governed `", stringify!($field), "` identifier.")]
            pub $field: String,)+
            /// Record creation instant.
            pub created_at: DateTime<Utc>,
            /// Digest of exact canonical record bytes.
            pub digest: RecordDigest,
        }
        impl $name {
            /// Creates a validated immutable record.
            #[allow(clippy::too_many_arguments)]
            pub fn new(id: impl Into<String>, $($field: impl Into<String>,)+ created_at: DateTime<Utc>, digest: RecordDigest) -> Result<Self, TraceabilityError> {
                let value = Self { id: id.into(), $($field: $field.into(),)+ created_at, digest };
                validate_text(&value.id)?;
                $(validate_text(&value.$field)?;)+
                validate_digest(value.digest)?;
                Ok(value)
            }
            /// Verifies an independently calculated canonical digest.
            pub fn verify_identity(&self, actual: RecordDigest) -> Result<(), TraceabilityError> {
                if actual == self.digest { Ok(()) } else { Err(TraceabilityError::DigestMismatch { id: self.id.clone() }) }
            }
        }
    }
}

immutable_record!(ReleaseRecord, "Immutable software release record.", {
    source_id,
    artifact_id,
    configuration_id,
    sbom_id
});
immutable_record!(ConfigurationRecord, "Immutable activated configuration record.", {
    schema_version,
    owner,
    rollout_scope
});
immutable_record!(ModelRecord, "Immutable model release record with lineage.", {
    training_source_id,
    training_data_id,
    odd_id,
    approval_id
});
immutable_record!(DeploymentRecord, "Immutable deployed-state record.", {
    release_id,
    configuration_id,
    environment,
    tenant_region,
    actor_id
});

fn validate_text(value: &str) -> Result<(), TraceabilityError> {
    if value.trim().is_empty() || value.len() > 4096 {
        Err(TraceabilityError::InvalidRecord)
    } else {
        Ok(())
    }
}

fn validate_digest(digest: RecordDigest) -> Result<(), TraceabilityError> {
    if digest == [0; 32] {
        Err(TraceabilityError::InvalidRecord)
    } else {
        Ok(())
    }
}

/// Stable fail-closed traceability and record errors.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TraceabilityError {
    /// A record contains invalid or zero-digest content.
    #[error("governed record is invalid")]
    InvalidRecord,
    /// An immutable identifier was reused.
    #[error("immutable identifier is duplicated")]
    DuplicateIdentifier,
    /// An artifact endpoint does not exist.
    #[error("unknown artifact: {0}")]
    UnknownArtifact(String),
    /// An exact input has the wrong artifact class.
    #[error("artifact {id} has the wrong kind; expected {expected:?}")]
    WrongArtifactKind {
        /// Artifact identifier.
        id: String,
        /// Required class.
        expected: ArtifactKind,
    },
    /// A required trace relationship is absent.
    #[error("release {release_id} is missing a required {kind:?} link")]
    MissingLink {
        /// Candidate release.
        release_id: String,
        /// Missing class, when known.
        kind: Option<ArtifactKind>,
    },
    /// A relationship passed its review boundary.
    #[error("traceability link {from} -> {to} is stale")]
    StaleLink {
        /// Source ID.
        from: String,
        /// Target ID.
        to: String,
    },
    /// Linked evidence disagrees.
    #[error("traceability link {from} -> {to} is contradictory")]
    ContradictoryLink {
        /// Source ID.
        from: String,
        /// Target ID.
        to: String,
    },
    /// A relationship was explicitly revoked.
    #[error("traceability link {from} -> {to} is revoked")]
    RevokedLink {
        /// Source ID.
        from: String,
        /// Target ID.
        to: String,
    },
    /// No current independent human approval matched the authority.
    #[error("release is not independently approved: {0}")]
    Unapproved(String),
    /// Exact configuration/capability/ODD is not linked.
    #[error("promotion scope is not linked to candidate: {0}")]
    WrongPromotionScope(String),
    /// Exact hardware inventory is absent.
    #[error("promotion request has no exact hardware revisions")]
    MissingHardware,
    /// Canonical bytes differ from the immutable digest.
    #[error("immutable record digest differs: {id}")]
    DigestMismatch {
        /// Record identifier.
        id: String,
    },
}
