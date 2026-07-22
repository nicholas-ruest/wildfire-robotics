use crate::{
    AssemblyId, BlanketConfigurationId, CradleId, DomainError, EvidenceRef, JointId,
    MaterialRevisionId, OddId, PanelId, ParafoilId, QualificationStage, ReelId, RobotId, TetherId,
    VentId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionBinding<T> {
    pub item: T,
    revision: String,
}

impl<T> RevisionBinding<T> {
    pub fn new(item: T, revision: &str) -> Result<Self, DomainError> {
        let revision = revision.trim();
        if revision.is_empty() || revision.len() > 128 {
            return Err(DomainError::InvalidConfiguration);
        }
        Ok(Self {
            item,
            revision: revision.to_owned(),
        })
    }

    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationBinding {
    id: BlanketConfigurationId,
    digest: String,
    pub material: MaterialRevisionId,
    panels: Vec<RevisionBinding<PanelId>>,
    pub joints: Vec<RevisionBinding<JointId>>,
    pub vents: Vec<RevisionBinding<VentId>>,
    pub anchors: Vec<RevisionBinding<crate::AnchorId>>,
    pub tethers: Vec<RevisionBinding<TetherId>>,
    pub reels: Vec<RevisionBinding<ReelId>>,
    pub parafoils: Vec<RevisionBinding<ParafoilId>>,
    pub cradles: Vec<RevisionBinding<CradleId>>,
    pub robots: Vec<RevisionBinding<RobotId>>,
    pub geometry_revision: String,
    pub mass_properties_revision: String,
    pub odd: OddId,
}

impl ConfigurationBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: BlanketConfigurationId,
        digest: &str,
        material: MaterialRevisionId,
        panels: Vec<RevisionBinding<PanelId>>,
        joints: Vec<RevisionBinding<JointId>>,
        vents: Vec<RevisionBinding<VentId>>,
        anchors: Vec<RevisionBinding<crate::AnchorId>>,
        tethers: Vec<RevisionBinding<TetherId>>,
        reels: Vec<RevisionBinding<ReelId>>,
        parafoils: Vec<RevisionBinding<ParafoilId>>,
        cradles: Vec<RevisionBinding<CradleId>>,
        robots: Vec<RevisionBinding<RobotId>>,
        geometry_revision: &str,
        mass_properties_revision: &str,
        odd: OddId,
    ) -> Result<Self, DomainError> {
        let collections_valid = [
            unique(&panels),
            unique(&joints),
            unique(&vents),
            unique(&anchors),
            unique(&tethers),
            unique(&reels),
            unique(&parafoils),
            unique(&cradles),
            unique(&robots),
        ]
        .into_iter()
        .all(|valid| valid);
        if !valid_digest(digest)
            || geometry_revision.trim().is_empty()
            || mass_properties_revision.trim().is_empty()
            || !collections_valid
        {
            return Err(DomainError::InvalidConfiguration);
        }
        Ok(Self {
            id,
            digest: digest.to_owned(),
            material,
            panels,
            joints,
            vents,
            anchors,
            tethers,
            reels,
            parafoils,
            cradles,
            robots,
            geometry_revision: geometry_revision.trim().to_owned(),
            mass_properties_revision: mass_properties_revision.trim().to_owned(),
            odd,
        })
    }

    #[must_use]
    pub fn id(&self) -> &BlanketConfigurationId {
        &self.id
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    #[must_use]
    pub fn panels(&self) -> &[RevisionBinding<PanelId>] {
        &self.panels
    }
}

fn unique<T: core::fmt::Debug>(items: &[RevisionBinding<T>]) -> bool {
    if items.is_empty() {
        return false;
    }
    let mut seen = BTreeSet::new();
    items.iter().all(|item| {
        let encoded = format!("{:?}", item.item);
        seen.insert(encoded)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestVariance {
    None,
    Explained,
    Unexplained,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedQualificationEvidence {
    pub artifact: EvidenceRef,
    pub configuration_digest: String,
    pub stage: QualificationStage,
    signer: String,
    signature_digest: String,
    pub variance: TestVariance,
    pub occurrence_resolved: bool,
}

impl SignedQualificationEvidence {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        artifact: EvidenceRef,
        configuration_digest: &str,
        stage: QualificationStage,
        signer: &str,
        signature_digest: &str,
        variance: TestVariance,
        occurrence_resolved: bool,
    ) -> Result<Self, DomainError> {
        if !valid_digest(configuration_digest)
            || !valid_digest(signature_digest)
            || signer.trim().is_empty()
            || matches!(
                stage,
                QualificationStage::Concept
                    | QualificationStage::Suspended
                    | QualificationStage::Retired
            )
        {
            return Err(DomainError::InvalidEvidence);
        }
        Ok(Self {
            artifact,
            configuration_digest: configuration_digest.to_owned(),
            stage,
            signer: signer.trim().to_owned(),
            signature_digest: signature_digest.to_owned(),
            variance,
            occurrence_resolved,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SuspensionReason {
    Substitution,
    ExpiredEvidence,
    UnexplainedVariance,
    UnresolvedOccurrence,
    ChangedOdd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualificationStatus {
    Active,
    Suspended(BTreeSet<SuspensionReason>),
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlanketConfiguration {
    binding: ConfigurationBinding,
    stage: QualificationStage,
    evidence: Vec<SignedQualificationEvidence>,
    status: QualificationStatus,
}

impl BlanketConfiguration {
    pub fn register(binding: ConfigurationBinding) -> Result<Self, DomainError> {
        Ok(Self {
            binding,
            stage: QualificationStage::Concept,
            evidence: Vec::new(),
            status: QualificationStatus::Active,
        })
    }

    #[must_use]
    pub const fn stage(&self) -> QualificationStage {
        self.stage
    }

    #[must_use]
    pub fn status(&self) -> &QualificationStatus {
        &self.status
    }

    #[must_use]
    pub fn binding(&self) -> &ConfigurationBinding {
        &self.binding
    }

    pub fn promote(
        &mut self,
        target: QualificationStage,
        evidence: Vec<SignedQualificationEvidence>,
        at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if !matches!(self.status, QualificationStatus::Active) {
            return Err(DomainError::QualificationSuspended);
        }
        if self.stage.next() != Some(target) {
            return Err(DomainError::QualificationStageSkipped);
        }
        if self
            .evidence
            .iter()
            .any(|item| !item.artifact.is_current_at(at))
        {
            self.suspend(SuspensionReason::ExpiredEvidence);
            return Err(DomainError::EvidenceExpired);
        }
        self.validate_evidence(target, &evidence, at)?;
        self.evidence.extend(evidence);
        self.stage = target;
        Ok(())
    }

    fn validate_evidence(
        &self,
        target: QualificationStage,
        evidence: &[SignedQualificationEvidence],
        at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if evidence.is_empty()
            || evidence.iter().any(|item| {
                item.configuration_digest != self.binding.digest || item.stage != target
            })
        {
            return Err(DomainError::EvidenceMismatch);
        }
        if evidence.iter().any(|item| !item.artifact.is_current_at(at)) {
            return Err(DomainError::EvidenceExpired);
        }
        if evidence
            .iter()
            .any(|item| item.variance == TestVariance::Unexplained)
        {
            return Err(DomainError::UnexplainedVariance);
        }
        if evidence.iter().any(|item| !item.occurrence_resolved) {
            return Err(DomainError::UnresolvedOccurrence);
        }
        Ok(())
    }

    pub fn evaluate_evidence(&mut self, at: DateTime<Utc>) -> bool {
        if self
            .evidence
            .iter()
            .any(|item| !item.artifact.is_current_at(at))
        {
            self.suspend(SuspensionReason::ExpiredEvidence);
            return false;
        }
        true
    }

    pub fn suspend(&mut self, reason: SuspensionReason) {
        match &mut self.status {
            QualificationStatus::Suspended(reasons) => {
                reasons.insert(reason);
            }
            QualificationStatus::Active => {
                self.status = QualificationStatus::Suspended(BTreeSet::from([reason]));
            }
            QualificationStatus::Retired => {}
        }
    }

    pub fn requalify(
        &mut self,
        replacement_evidence: Vec<SignedQualificationEvidence>,
        at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if !matches!(self.status, QualificationStatus::Suspended(_)) {
            return Err(DomainError::InvalidTransition);
        }
        let required = stages_through(self.stage);
        if required.is_empty()
            || replacement_evidence
                .iter()
                .any(|item| !required.contains(&item.stage))
            || required
                .iter()
                .any(|stage| !replacement_evidence.iter().any(|item| item.stage == *stage))
        {
            return Err(DomainError::EvidenceMismatch);
        }
        for stage in required {
            let stage_evidence = replacement_evidence
                .iter()
                .filter(|item| item.stage == stage)
                .cloned()
                .collect::<Vec<_>>();
            self.validate_evidence(stage, &stage_evidence, at)?;
        }
        self.evidence = replacement_evidence;
        self.status = QualificationStatus::Active;
        Ok(())
    }

    #[must_use]
    pub fn substitute(&self, replacement: ConfigurationBinding) -> Self {
        Self {
            binding: replacement,
            stage: QualificationStage::Concept,
            evidence: Vec::new(),
            status: QualificationStatus::Suspended(BTreeSet::from([
                SuspensionReason::Substitution,
            ])),
        }
    }

    pub fn record_changed_odd(&mut self) {
        self.suspend(SuspensionReason::ChangedOdd);
    }

    pub fn record_unexplained_variance(&mut self) {
        self.suspend(SuspensionReason::UnexplainedVariance);
    }

    pub fn record_unresolved_occurrence(&mut self) {
        self.suspend(SuspensionReason::UnresolvedOccurrence);
    }

    pub fn begin_substitution_requalification(&mut self) -> Result<(), DomainError> {
        let QualificationStatus::Suspended(reasons) = &self.status else {
            return Err(DomainError::InvalidTransition);
        };
        if self.stage != QualificationStage::Concept
            || reasons != &BTreeSet::from([SuspensionReason::Substitution])
        {
            return Err(DomainError::QualificationSuspended);
        }
        self.status = QualificationStatus::Active;
        Ok(())
    }

    pub fn retire(&mut self) {
        self.status = QualificationStatus::Retired;
    }
}

fn stages_through(stage: QualificationStage) -> Vec<QualificationStage> {
    let mut stages = Vec::new();
    let mut cursor = QualificationStage::Concept;
    while let Some(next) = cursor.next() {
        stages.push(next);
        if next == stage {
            break;
        }
        cursor = next;
    }
    stages
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembraneComponentKind {
    Panel,
    Joint,
    Vent,
    Anchor,
    Tether,
    Reel,
    Parafoil,
    Cradle,
    Robot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    Packed,
    Deployed,
    SearchPending,
    Located,
    Quarantined,
    Recovered,
    SacrificiallyReleased,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponent {
    serial: String,
    pub kind: MembraneComponentKind,
    pub recovery: RecoveryState,
    pub isolated: bool,
}

impl SerializedComponent {
    pub fn new(serial: &str, kind: MembraneComponentKind) -> Result<Self, DomainError> {
        let serial = serial.trim();
        if serial.is_empty() || serial.len() > 128 {
            return Err(DomainError::InvalidComponent);
        }
        Ok(Self {
            serial: serial.to_owned(),
            kind,
            recovery: RecoveryState::Packed,
            isolated: false,
        })
    }

    #[must_use]
    pub fn serial(&self) -> &str {
        &self.serial
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembraneAssembly {
    pub id: AssemblyId,
    pub configuration: BlanketConfigurationId,
    pub configuration_digest: String,
    components: Vec<SerializedComponent>,
}

impl MembraneAssembly {
    pub fn assemble(
        id: AssemblyId,
        configuration: &BlanketConfiguration,
        components: Vec<SerializedComponent>,
    ) -> Result<Self, DomainError> {
        let mut serials = BTreeSet::new();
        if components.is_empty() || !components.iter().all(|item| serials.insert(&item.serial)) {
            return Err(DomainError::InvalidComponent);
        }
        Ok(Self {
            id,
            configuration: configuration.binding.id.clone(),
            configuration_digest: configuration.binding.digest.clone(),
            components,
        })
    }

    pub fn isolate_panel(&mut self, serial: &str) -> Result<(), DomainError> {
        let component = self.component_mut(serial)?;
        if component.kind != MembraneComponentKind::Panel {
            return Err(DomainError::InvalidComponent);
        }
        component.isolated = true;
        Ok(())
    }

    pub fn sacrificial_release(&mut self, serial: &str) -> Result<(), DomainError> {
        let component = self.component_mut(serial)?;
        if component.recovery == RecoveryState::Recovered {
            return Err(DomainError::InvalidTransition);
        }
        component.recovery = RecoveryState::SacrificiallyReleased;
        Ok(())
    }

    pub fn transition_recovery(
        &mut self,
        serial: &str,
        target: RecoveryState,
    ) -> Result<(), DomainError> {
        let component = self.component_mut(serial)?;
        let allowed = matches!(
            (component.recovery, target),
            (RecoveryState::Packed, RecoveryState::Deployed)
                | (RecoveryState::Deployed, RecoveryState::SearchPending)
                | (RecoveryState::SearchPending, RecoveryState::Located)
                | (
                    RecoveryState::Located,
                    RecoveryState::Quarantined | RecoveryState::Recovered
                )
                | (RecoveryState::Quarantined, RecoveryState::Recovered)
        );
        if !allowed {
            return Err(DomainError::InvalidTransition);
        }
        component.recovery = target;
        Ok(())
    }

    #[must_use]
    pub fn components(&self) -> &[SerializedComponent] {
        &self.components
    }

    fn component_mut(&mut self, serial: &str) -> Result<&mut SerializedComponent, DomainError> {
        self.components
            .iter_mut()
            .find(|item| item.serial == serial)
            .ok_or(DomainError::InvalidComponent)
    }
}
