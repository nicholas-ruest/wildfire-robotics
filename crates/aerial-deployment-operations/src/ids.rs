use crate::DomainError;
use serde::{Deserialize, Serialize};
fn validate(value: &str) -> Result<String, DomainError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(DomainError::Empty);
    }
    if value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
    {
        return Err(DomainError::InvalidIdentifier);
    }
    Ok(value.to_owned())
}
macro_rules! opaque_ids {($($name:ident),+ $(,)?)=>{$(#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]#[serde(try_from="String",into="String")]pub struct $name(String);impl $name{pub fn new(value:&str)->Result<Self,DomainError>{validate(value).map(Self)}#[must_use]pub fn as_str(&self)->&str{&self.0}}impl TryFrom<String> for $name{type Error=DomainError;fn try_from(value:String)->Result<Self,Self::Error>{Self::new(&value)}}impl From<$name> for String{fn from(value:$name)->Self{value.0}})+}}
opaque_ids!(
    BlanketConfigurationId,
    MaterialRevisionId,
    PanelId,
    JointId,
    VentId,
    AnchorId,
    TetherId,
    ReelId,
    ParafoilId,
    CradleId,
    RobotId,
    AircraftConfigurationId,
    PayloadManifestId,
    ReleaseCorridorId,
    FootprintId,
    ExclusionZoneId,
    JettisonZoneId,
    EmergencyLandingZoneId,
    AssemblyId,
    AerialDropMissionId,
    ReleaseAuthorizationId,
    AirborneDeploymentId,
    GroundInstallationId,
    EvidenceId,
    OddId,
    CommandId,
    EventId,
    ComponentId,
    EffectivenessStudyId
);

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn rejects_ambiguous_or_unbounded_identifiers() {
        assert_eq!(PanelId::new(""), Err(DomainError::Empty));
        assert_eq!(
            PanelId::new("panel/../../x"),
            Err(DomainError::InvalidIdentifier)
        );
        assert!(PanelId::new("panel-001:rev-a").is_ok());
    }
    #[test]
    fn identifier_types_are_not_interchangeable() {
        let panel = PanelId::new("same").unwrap();
        let tether = TetherId::new("same").unwrap();
        assert_eq!(panel.as_str(), tether.as_str());
    }
}
