//! Canonical fixture/file adapter and provider anti-corruption port.
#![allow(missing_docs)]
use crate::{Digest, HazardError};
use sha2::{Digest as _, Sha256};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRecord {
    pub provider_identity: String,
    pub source_version: String,
    pub payload: String,
    pub checksum: Digest,
    pub license: String,
}
pub trait ProviderAdapter {
    type Error;
    fn fetch(&mut self, cursor: Option<&str>) -> Result<Vec<ProviderRecord>, Self::Error>;
}
#[derive(Clone, Debug, Default)]
pub struct CanonicalFileAdapter {
    records: Vec<ProviderRecord>,
    outage: bool,
}
impl CanonicalFileAdapter {
    pub fn parse(input: &str) -> Result<Self, HazardError> {
        if input.len() > 8 * 1024 * 1024 {
            return Err(HazardError::InvalidObservation);
        }
        let mut records = Vec::new();
        for line in input.lines().filter(|l| !l.trim().is_empty()) {
            if records.len() >= 100_000 {
                return Err(HazardError::InvalidObservation);
            }
            let fields = line.split('|').collect::<Vec<_>>();
            if fields.len() != 5 || fields.iter().any(|v| v.trim().is_empty()) {
                return Err(HazardError::InvalidObservation);
            }
            let bytes = hex_digest(fields[3])?;
            let actual: Digest = Sha256::digest(fields[2].as_bytes()).into();
            if actual != bytes {
                return Err(HazardError::InvalidObservation);
            }
            records.push(ProviderRecord {
                provider_identity: fields[0].into(),
                source_version: fields[1].into(),
                payload: fields[2].into(),
                checksum: bytes,
                license: fields[4].into(),
            });
        }
        Ok(Self {
            records,
            outage: false,
        })
    }
    pub fn set_outage(&mut self, value: bool) {
        self.outage = value;
    }
}
impl ProviderAdapter for CanonicalFileAdapter {
    type Error = HazardError;
    fn fetch(&mut self, _: Option<&str>) -> Result<Vec<ProviderRecord>, Self::Error> {
        if self.outage {
            return Err(HazardError::InvalidTransition);
        }
        Ok(self.records.clone())
    }
}
fn hex_digest(value: &str) -> Result<Digest, HazardError> {
    if value.len() != 64 {
        return Err(HazardError::InvalidObservation);
    }
    let mut out = [0u8; 32];
    for (i, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(chunk).map_err(|_| HazardError::InvalidObservation)?;
        out[i] = u8::from_str_radix(text, 16).map_err(|_| HazardError::InvalidObservation)?;
    }
    if out == [0; 32] {
        return Err(HazardError::InvalidObservation);
    }
    Ok(out)
}
