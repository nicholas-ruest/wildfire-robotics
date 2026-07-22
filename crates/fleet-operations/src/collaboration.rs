//! Bounded, advisory fleet-collaboration profiles and runtimes.
#![allow(missing_docs, clippy::must_use_candidate)]
use hmac::{Hmac, Mac as _};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const MAX_EVIDENCE: usize = 4_096;
const MAX_MEMBERS: usize = 4_096;
const MAX_COHORT: usize = 16;
const STALE_AFTER_TICKS: u64 = 500;
const MAX_PER_WITNESS_KIND: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CollaborationError {
    #[error("invalid evidence")]
    InvalidEvidence,
    #[error("profile stale or poisoned")]
    InvalidProfile,
    #[error("invalid lifecycle transition")]
    InvalidTransition,
    #[error("invalid checkpoint")]
    InvalidCheckpoint,
    #[error("runtime unavailable")]
    RuntimeUnavailable,
    #[error("member is outside the capability-gated partition")]
    IneligibleMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum EvidenceKind {
    Cooperation,
    Proximity,
    Communication,
    Complementarity,
    Handoff,
    SafetyOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipEvidence {
    a: String,
    b: String,
    context: String,
    kind: EvidenceKind,
    tick: u64,
    value_bps: u16,
    witness: String,
    event_id: String,
    key_id: String,
    signature: [u8; 32],
}
impl RelationshipEvidence {
    #[allow(clippy::too_many_arguments)]
    pub fn sign(
        a: &str,
        b: &str,
        context: &str,
        kind: EvidenceKind,
        tick: u64,
        value_bps: u16,
        witness: &str,
        key: [u8; 32],
    ) -> Result<Self, CollaborationError> {
        let event_id = format!("{witness}:{tick}:{}", kind as u8);
        Self::sign_event(
            a,
            b,
            context,
            kind,
            tick,
            value_bps,
            witness,
            &event_id,
            "legacy-fixture",
            key,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn sign_event(
        a: &str,
        b: &str,
        context: &str,
        kind: EvidenceKind,
        tick: u64,
        value_bps: u16,
        witness: &str,
        event_id: &str,
        key_id: &str,
        key: [u8; 32],
    ) -> Result<Self, CollaborationError> {
        if [a, b, context, witness, event_id, key_id].contains(&"")
            || a == b
            || key == [0; 32]
            || value_bps > 10_000
        {
            return Err(CollaborationError::InvalidEvidence);
        }
        let mut e = Self {
            a: a.into(),
            b: b.into(),
            context: context.into(),
            kind,
            tick,
            value_bps,
            witness: witness.into(),
            event_id: event_id.into(),
            key_id: key_id.into(),
            signature: [0; 32],
        };
        e.signature = e.expected(&key);
        Ok(e)
    }
    fn expected(&self, key: &[u8; 32]) -> [u8; 32] {
        let Ok(mut h) = Hmac::<Sha256>::new_from_slice(key) else {
            return [0; 32];
        };
        h.update(b"wildfire-collaboration-evidence-v1");
        for value in [
            &self.a,
            &self.b,
            &self.context,
            &self.witness,
            &self.event_id,
        ] {
            h.update(&(value.len() as u64).to_be_bytes());
            h.update(value.as_bytes());
        }
        h.update(&[self.kind as u8]);
        h.update(&self.tick.to_be_bytes());
        h.update(&self.value_bps.to_be_bytes());
        h.finalize().into_bytes().into()
    }
    pub fn verify_with_key(&self, key: [u8; 32]) -> bool {
        self.signature == self.expected(&key)
    }
    pub fn context(&self) -> &str {
        &self.context
    }
    pub fn endpoints(&self) -> (&str, &str) {
        (&self.a, &self.b)
    }
    pub fn event_id(&self) -> &str {
        &self.event_id
    }
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_transport(
        a: String,
        b: String,
        context: String,
        kind: EvidenceKind,
        tick: u64,
        value_bps: u16,
        witness: String,
        event_id: String,
        key_id: String,
        signature: [u8; 32],
    ) -> Result<Self, CollaborationError> {
        if [&a, &b, &context, &witness, &event_id, &key_id]
            .iter()
            .any(|v| v.is_empty())
            || a == b
            || value_bps > 10_000
        {
            return Err(CollaborationError::InvalidEvidence);
        }
        Ok(Self {
            a,
            b,
            context,
            kind,
            tick,
            value_bps,
            witness,
            event_id,
            key_id,
            signature,
        })
    }
    pub fn signature(&self) -> [u8; 32] {
        self.signature
    }
}

pub trait EvidenceKeyResolver {
    fn resolve(&self, key_id: &str, witness: &str) -> Option<[u8; 32]>;
}
struct LegacyFixtureResolver;
impl EvidenceKeyResolver for LegacyFixtureResolver {
    fn resolve(&self, key_id: &str, _: &str) -> Option<[u8; 32]> {
        (key_id == "legacy-fixture").then_some([7; 32])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileState {
    Collecting,
    Evaluated,
    Active,
    Stale,
    Suspended,
    Retired,
}
#[derive(Debug, Clone)]
pub struct CollaborationProfile {
    id: String,
    context: String,
    evidence: Vec<RelationshipEvidence>,
    base_score_bps: u16,
    uncertainty_bps: u16,
    as_of_tick: u64,
    state: ProfileState,
    model_version: u64,
}
impl CollaborationProfile {
    pub fn build(
        id: &str,
        evidence: Vec<RelationshipEvidence>,
        as_of: u64,
    ) -> Result<Self, CollaborationError> {
        let mut profile = Self::evaluate(id, evidence, as_of, 1)?;
        profile.promote()?;
        Ok(profile)
    }
    pub fn evaluate(
        id: &str,
        evidence: Vec<RelationshipEvidence>,
        as_of: u64,
        model_version: u64,
    ) -> Result<Self, CollaborationError> {
        Self::evaluate_verified(id, evidence, as_of, model_version, &LegacyFixtureResolver)
    }
    pub fn evaluate_verified(
        id: &str,
        evidence: Vec<RelationshipEvidence>,
        as_of: u64,
        model_version: u64,
        keys: &impl EvidenceKeyResolver,
    ) -> Result<Self, CollaborationError> {
        if id.is_empty()
            || evidence.is_empty()
            || evidence.len() > MAX_EVIDENCE
            || model_version == 0
        {
            return Err(CollaborationError::InvalidProfile);
        }
        let context = evidence[0].context.clone();
        let endpoints = canonical_pair(&evidence[0].a, &evidence[0].b);
        let mut events = BTreeSet::new();
        let mut counts: BTreeMap<(String, EvidenceKind), usize> = BTreeMap::new();
        for e in &evidence {
            if keys
                .resolve(&e.key_id, &e.witness)
                .is_none_or(|key| !e.verify_with_key(key))
                || e.tick > as_of
                || e.context != context
                || canonical_pair(&e.a, &e.b) != endpoints
                || !events.insert(e.event_id.clone())
            {
                return Err(CollaborationError::InvalidProfile);
            }
            let count = counts.entry((e.witness.clone(), e.kind)).or_default();
            *count += 1;
            if *count > MAX_PER_WITNESS_KIND {
                return Err(CollaborationError::InvalidProfile);
            }
        }
        let unique_witnesses = evidence
            .iter()
            .map(|e| e.witness.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        let distinct_kinds = evidence
            .iter()
            .map(|e| e.kind)
            .collect::<BTreeSet<_>>()
            .len();
        let raw =
            evidence.iter().map(|e| u64::from(e.value_bps)).sum::<u64>() / evidence.len() as u64;
        let diversity = (unique_witnesses.min(4) * 750 + distinct_kinds.min(6) * 250) as u64;
        let score = u16::try_from(raw.min(4_000 + diversity).min(8_000))
            .map_err(|_| CollaborationError::InvalidProfile)?;
        let uncertainty = u16::try_from(
            (4_000usize
                .saturating_sub(unique_witnesses.min(4) * 500 + distinct_kinds.min(6) * 200))
            .max(500),
        )
        .map_err(|_| CollaborationError::InvalidProfile)?;
        Ok(Self {
            id: id.into(),
            context,
            evidence,
            base_score_bps: score,
            uncertainty_bps: uncertainty,
            as_of_tick: as_of,
            state: ProfileState::Evaluated,
            model_version,
        })
    }
    pub fn promote(&mut self) -> Result<(), CollaborationError> {
        if self.state != ProfileState::Evaluated {
            return Err(CollaborationError::InvalidTransition);
        }
        self.state = ProfileState::Active;
        Ok(())
    }
    pub fn revoke(&mut self) -> Result<(), CollaborationError> {
        if matches!(self.state, ProfileState::Retired) {
            return Err(CollaborationError::InvalidTransition);
        }
        self.state = ProfileState::Suspended;
        Ok(())
    }
    pub fn retire(&mut self) {
        self.state = ProfileState::Retired;
    }
    pub fn refresh_state(&mut self, tick: u64) {
        if tick.saturating_sub(self.as_of_tick) > STALE_AFTER_TICKS
            && self.state == ProfileState::Active
        {
            self.state = ProfileState::Stale;
        }
    }
    pub fn score_at(&self, tick: u64) -> u16 {
        if tick < self.as_of_tick {
            return 0;
        }
        let age = tick - self.as_of_tick;
        u16::try_from(u64::from(self.base_score_bps) * 10_000 / (10_000 + age.saturating_mul(100)))
            .unwrap_or(0)
    }
    pub fn uncertainty_bps(&self) -> u16 {
        self.uncertainty_bps
    }
    pub fn privileges(&self) -> PlatformPrivileges {
        PlatformPrivileges::None
    }
    pub fn state(&self) -> ProfileState {
        self.state
    }
    pub fn evidence_count(&self) -> usize {
        self.evidence.len()
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn context(&self) -> &str {
        &self.context
    }
    pub fn model_version(&self) -> u64 {
        self.model_version
    }
    fn pair(&self) -> (&str, &str) {
        self.evidence[0].endpoints()
    }
}
fn canonical_pair<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b { (a, b) } else { (b, a) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformPrivileges {
    None,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationAuthority {
    AdvisoryOnly,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationSource {
    Conventional,
    ConventionalFallback,
    External,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CohortRecommendation {
    pub cohorts: Vec<Vec<String>>,
    pub split_advised: bool,
    pub merge_advised: bool,
    pub authority: RecommendationAuthority,
    pub source: RecommendationSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionMember<'a> {
    pub id: &'a str,
    pub tenant: &'a str,
    pub capability: &'a str,
    pub epoch: u64,
    pub eligible: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionGate<'a> {
    pub tenant: &'a str,
    pub capability: &'a str,
    pub epoch: u64,
}

pub trait CollaborationRuntimePort {
    fn recommend(
        &self,
        members: &[&str],
        profiles: &[CollaborationProfile],
        max_cohort: usize,
        tick: u64,
    ) -> Result<CohortRecommendation, CollaborationError>;
    fn fallback(
        &self,
        members: &[&str],
        max_cohort: usize,
    ) -> Result<CohortRecommendation, CollaborationError> {
        conventional(
            members,
            &[],
            max_cohort,
            0,
            RecommendationSource::ConventionalFallback,
        )
    }
}
pub struct ConventionalCollaborationRuntime;
impl CollaborationRuntimePort for ConventionalCollaborationRuntime {
    fn recommend(
        &self,
        m: &[&str],
        p: &[CollaborationProfile],
        max: usize,
        tick: u64,
    ) -> Result<CohortRecommendation, CollaborationError> {
        let invalid = p.iter().any(|x| {
            tick < x.as_of_tick
                || tick - x.as_of_tick > STALE_AFTER_TICKS
                || x.state != ProfileState::Active
        });
        conventional(
            m,
            if invalid { &[] } else { p },
            max,
            tick,
            if invalid {
                RecommendationSource::ConventionalFallback
            } else {
                RecommendationSource::Conventional
            },
        )
    }
}
impl ConventionalCollaborationRuntime {
    pub fn recommend_partition(
        &self,
        members: &[PartitionMember<'_>],
        gate: &PartitionGate<'_>,
        profiles: &[CollaborationProfile],
        max: usize,
        tick: u64,
    ) -> Result<CohortRecommendation, CollaborationError> {
        if members.iter().any(|m| {
            !m.eligible
                || m.tenant != gate.tenant
                || m.capability != gate.capability
                || m.epoch != gate.epoch
        }) {
            return Err(CollaborationError::IneligibleMember);
        }
        let ids = members.iter().map(|m| m.id).collect::<Vec<_>>();
        self.recommend(&ids, profiles, max, tick)
    }
}
fn conventional(
    members: &[&str],
    profiles: &[CollaborationProfile],
    max: usize,
    tick: u64,
    source: RecommendationSource,
) -> Result<CohortRecommendation, CollaborationError> {
    if members.is_empty()
        || members.len() > MAX_MEMBERS
        || max == 0
        || max > MAX_COHORT
        || members.iter().any(|m| m.is_empty())
    {
        return Err(CollaborationError::RuntimeUnavailable);
    }
    let mut remaining = members.iter().copied().collect::<BTreeSet<_>>();
    if remaining.len() != members.len() {
        return Err(CollaborationError::RuntimeUnavailable);
    }
    let mut weights = BTreeMap::new();
    for p in profiles {
        let (a, b) = p.pair();
        if remaining.contains(a) && remaining.contains(b) {
            weights.insert(
                canonical_pair(a, b),
                p.score_at(tick).saturating_sub(p.uncertainty_bps),
            );
        }
    }
    let mut cohorts = Vec::new();
    while let Some(seed) = remaining.pop_first() {
        let mut cohort = vec![seed.to_owned()];
        while cohort.len() < max && !remaining.is_empty() {
            let next = remaining.iter().copied().max_by_key(|candidate| {
                let affinity = cohort
                    .iter()
                    .map(|m| {
                        weights
                            .get(&canonical_pair(m, candidate))
                            .copied()
                            .unwrap_or(0)
                    })
                    .sum::<u16>();
                (affinity, std::cmp::Reverse(*candidate))
            });
            if let Some(n) = next {
                remaining.remove(n);
                cohort.push(n.to_owned());
            } else {
                break;
            }
        }
        cohort.sort();
        cohorts.push(cohort);
    }
    Ok(CohortRecommendation {
        split_advised: cohorts.len() > 1,
        merge_advised: cohorts.len() == 1 && members.len() < max.div_ceil(2),
        cohorts,
        authority: RecommendationAuthority::AdvisoryOnly,
        source,
    })
}

pub struct FailingRuntime;
impl CollaborationRuntimePort for FailingRuntime {
    fn recommend(
        &self,
        _: &[&str],
        _: &[CollaborationProfile],
        _: usize,
        _: u64,
    ) -> Result<CohortRecommendation, CollaborationError> {
        Err(CollaborationError::RuntimeUnavailable)
    }
}
pub struct FaultTolerantRuntime<P, F> {
    pub primary: P,
    pub fallback: F,
}
impl<P: CollaborationRuntimePort, F: CollaborationRuntimePort> CollaborationRuntimePort
    for FaultTolerantRuntime<P, F>
{
    fn recommend(
        &self,
        m: &[&str],
        p: &[CollaborationProfile],
        max: usize,
        tick: u64,
    ) -> Result<CohortRecommendation, CollaborationError> {
        self.primary
            .recommend(m, p, max, tick)
            .or_else(|_| self.fallback.fallback(m, max))
    }
}

pub trait ExternalRecommendationPort {
    fn external_recommend(
        &self,
        members: &[&str],
        profiles: &[CollaborationProfile],
        max_cohort: usize,
        tick: u64,
    ) -> Result<CohortRecommendation, CollaborationError>;
}
pub struct GovernedExternalAdapter<E, F> {
    pub external: E,
    pub fallback: F,
}
impl<E: ExternalRecommendationPort, F: CollaborationRuntimePort> CollaborationRuntimePort
    for GovernedExternalAdapter<E, F>
{
    fn recommend(
        &self,
        members: &[&str],
        profiles: &[CollaborationProfile],
        max: usize,
        tick: u64,
    ) -> Result<CohortRecommendation, CollaborationError> {
        let external = self
            .external
            .external_recommend(members, profiles, max, tick);
        match external.and_then(|r| validate_external(r, members, max)) {
            Ok(r) => Ok(r),
            Err(_) => self.fallback.fallback(members, max),
        }
    }
}
fn validate_external(
    r: CohortRecommendation,
    members: &[&str],
    max: usize,
) -> Result<CohortRecommendation, CollaborationError> {
    if max == 0
        || max > MAX_COHORT
        || r.authority != RecommendationAuthority::AdvisoryOnly
        || r.source != RecommendationSource::External
        || r.cohorts.is_empty()
        || r.cohorts.iter().any(|c| c.is_empty() || c.len() > max)
    {
        return Err(CollaborationError::InvalidProfile);
    }
    let expected = members.iter().copied().collect::<BTreeSet<_>>();
    let actual = r
        .cohorts
        .iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if actual.len() != members.len() || actual.iter().copied().collect::<BTreeSet<_>>() != expected
    {
        return Err(CollaborationError::InvalidProfile);
    }
    Ok(r)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphCheckpoint {
    epoch: u64,
    nodes: Vec<String>,
    runtime_digest: [u8; 32],
    previous_digest: Option<[u8; 32]>,
    digest: [u8; 32],
}
impl GraphCheckpoint {
    pub fn seal(
        epoch: u64,
        nodes: Vec<String>,
        runtime: [u8; 32],
    ) -> Result<Self, CollaborationError> {
        Self::seal_after(epoch, nodes, runtime, None)
    }
    pub fn seal_after(
        epoch: u64,
        mut nodes: Vec<String>,
        runtime: [u8; 32],
        previous: Option<[u8; 32]>,
    ) -> Result<Self, CollaborationError> {
        if epoch == 0
            || nodes.is_empty()
            || nodes.len() > MAX_MEMBERS
            || runtime == [0; 32]
            || nodes.iter().any(String::is_empty)
        {
            return Err(CollaborationError::InvalidCheckpoint);
        }
        nodes.sort();
        nodes.dedup();
        let digest = checkpoint_digest(epoch, &nodes, runtime, previous);
        Ok(Self {
            epoch,
            nodes,
            runtime_digest: runtime,
            previous_digest: previous,
            digest,
        })
    }
    fn verified(&self) -> bool {
        self.digest
            == checkpoint_digest(
                self.epoch,
                &self.nodes,
                self.runtime_digest,
                self.previous_digest,
            )
    }
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }
}
fn checkpoint_digest(
    epoch: u64,
    nodes: &[String],
    runtime: [u8; 32],
    previous: Option<[u8; 32]>,
) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"wildfire-collaboration-checkpoint-v1");
    h.update(epoch.to_be_bytes());
    h.update(runtime);
    h.update(previous.unwrap_or([0; 32]));
    for n in nodes {
        h.update((n.len() as u64).to_be_bytes());
        h.update(n);
    }
    h.finalize().into()
}
pub struct CheckpointStore {
    active: GraphCheckpoint,
    history: Vec<GraphCheckpoint>,
}
impl CheckpointStore {
    pub fn activate(c: GraphCheckpoint) -> Result<Self, CollaborationError> {
        if !c.verified() {
            return Err(CollaborationError::InvalidCheckpoint);
        }
        Ok(Self {
            active: c,
            history: Vec::new(),
        })
    }
    pub fn replace(
        &mut self,
        c: Result<GraphCheckpoint, CollaborationError>,
    ) -> Result<(), CollaborationError> {
        let c = c?;
        if !c.verified()
            || c.epoch <= self.active.epoch
            || c.previous_digest.is_some_and(|d| d != self.active.digest)
        {
            return Err(CollaborationError::InvalidCheckpoint);
        }
        self.history.push(std::mem::replace(&mut self.active, c));
        Ok(())
    }
    pub fn rollback(&mut self) -> Result<(), CollaborationError> {
        self.active = self
            .history
            .pop()
            .ok_or(CollaborationError::InvalidCheckpoint)?;
        Ok(())
    }
    pub fn active_epoch(&self) -> u64 {
        self.active.epoch
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalRuntimeStatus {
    DisabledPendingEvaluation,
}
pub struct RvmAdapter;
impl RvmAdapter {
    pub fn status() -> ExternalRuntimeStatus {
        ExternalRuntimeStatus::DisabledPendingEvaluation
    }
}
