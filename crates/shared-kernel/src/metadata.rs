//! Content identity, classification, version, and evidence metadata primitives.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

const SHA256_BYTES: usize = 32;
const SHA256_HEX_CHARS: usize = SHA256_BYTES * 2;
const MAX_REFERENCE_ID_CHARS: usize = 128;
const MAX_MEDIA_TYPE_CHARS: usize = 127;

/// Failures encountered while validating technical metadata at a boundary.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum MetadataError {
    /// A digest was not a canonical, lower-case SHA-256 digest.
    #[error("digest must be `sha256:` followed by 64 lower-case hexadecimal characters")]
    InvalidSha256Digest,
    /// A semantic version did not follow the supported `SemVer` 2.0 grammar.
    #[error("semantic version is invalid")]
    InvalidSemanticVersion,
    /// A governed reference identifier was malformed or used the wrong namespace.
    #[error("governed reference identifier is invalid")]
    InvalidReferenceId,
    /// An artifact media type was blank, malformed, or too long.
    #[error("artifact media type is invalid")]
    InvalidMediaType,
}

/// The SHA-256 digest of immutable content.
///
/// Text input and output use the canonical `sha256:<lower-case hex>` form. This
/// type validates existing digests; hashing bytes remains an injected boundary
/// concern so the shared kernel does not depend on a hashing implementation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentDigest([u8; SHA256_BYTES]);

impl ContentDigest {
    /// Constructs a digest from an already computed SHA-256 byte array.
    #[must_use]
    pub const fn from_sha256_bytes(bytes: [u8; SHA256_BYTES]) -> Self {
        Self(bytes)
    }

    /// Returns the raw SHA-256 bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; SHA256_BYTES] {
        &self.0
    }
}

impl FromStr for ContentDigest {
    type Err = MetadataError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value
            .strip_prefix("sha256:")
            .filter(|hex| hex.len() == SHA256_HEX_CHARS)
            .ok_or(MetadataError::InvalidSha256Digest)?;
        let mut bytes = [0_u8; SHA256_BYTES];
        for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
            bytes[index] = decode_hex(pair[0])?
                .checked_mul(16)
                .and_then(|high| high.checked_add(decode_hex(pair[1]).ok()?))
                .ok_or(MetadataError::InvalidSha256Digest)?;
        }
        Ok(Self(bytes))
    }
}

fn decode_hex(value: u8) -> Result<u8, MetadataError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(MetadataError::InvalidSha256Digest),
    }
}

impl fmt::Display for ContentDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("sha256:")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Sensitivity assigned to data when it is created (ADR-028).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataClassification {
    /// Approved for unrestricted disclosure.
    Public,
    /// Operational data intended only for the organization.
    Internal,
    /// Sensitive data limited to explicitly authorized consumers.
    Confidential,
    /// Highest-sensitivity data requiring purpose-specific authorization.
    Restricted,
}

/// A validated Semantic Versioning 2.0 version.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SemanticVersion {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Option<String>,
    build_metadata: Option<String>,
}

impl SemanticVersion {
    /// Creates a stable semantic version without pre-release or build metadata.
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    /// Returns the major component.
    #[must_use]
    pub const fn major(&self) -> u64 {
        self.major
    }
    /// Returns the minor component.
    #[must_use]
    pub const fn minor(&self) -> u64 {
        self.minor
    }
    /// Returns the patch component.
    #[must_use]
    pub const fn patch(&self) -> u64 {
        self.patch
    }
    /// Returns the optional pre-release component.
    #[must_use]
    pub fn pre_release(&self) -> Option<&str> {
        self.pre_release.as_deref()
    }
    /// Returns the optional build metadata component.
    #[must_use]
    pub fn build_metadata(&self) -> Option<&str> {
        self.build_metadata.as_deref()
    }
}

impl FromStr for SemanticVersion {
    type Err = MetadataError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (core_and_pre, build_metadata) = split_optional(value, '+')?;
        let (core, pre_release) = split_optional(core_and_pre, '-')?;
        let mut numbers = core.split('.');
        let major = parse_core_number(numbers.next())?;
        let minor = parse_core_number(numbers.next())?;
        let patch = parse_core_number(numbers.next())?;
        if numbers.next().is_some() {
            return Err(MetadataError::InvalidSemanticVersion);
        }
        validate_identifiers(pre_release, true)?;
        validate_identifiers(build_metadata, false)?;
        Ok(Self {
            major,
            minor,
            patch,
            pre_release: pre_release.map(str::to_owned),
            build_metadata: build_metadata.map(str::to_owned),
        })
    }
}

fn split_optional(value: &str, separator: char) -> Result<(&str, Option<&str>), MetadataError> {
    let mut pieces = value.split(separator);
    let first = pieces.next().ok_or(MetadataError::InvalidSemanticVersion)?;
    let second = pieces.next();
    if pieces.next().is_some() || second == Some("") {
        return Err(MetadataError::InvalidSemanticVersion);
    }
    Ok((first, second))
}

fn parse_core_number(value: Option<&str>) -> Result<u64, MetadataError> {
    let value = value.ok_or(MetadataError::InvalidSemanticVersion)?;
    if value.is_empty() || (value.len() > 1 && value.starts_with('0')) {
        return Err(MetadataError::InvalidSemanticVersion);
    }
    value
        .parse()
        .map_err(|_| MetadataError::InvalidSemanticVersion)
}

fn validate_identifiers(
    value: Option<&str>,
    forbid_numeric_leading_zero: bool,
) -> Result<(), MetadataError> {
    let Some(value) = value else {
        return Ok(());
    };
    for identifier in value.split('.') {
        let valid_chars = !identifier.is_empty()
            && identifier
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
        let numeric_leading_zero = forbid_numeric_leading_zero
            && identifier.bytes().all(|byte| byte.is_ascii_digit())
            && identifier.len() > 1
            && identifier.starts_with('0');
        if !valid_chars || numeric_leading_zero {
            return Err(MetadataError::InvalidSemanticVersion);
        }
    }
    Ok(())
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre_release) = &self.pre_release {
            write!(formatter, "-{pre_release}")?;
        }
        if let Some(build_metadata) = &self.build_metadata {
            write!(formatter, "+{build_metadata}")?;
        }
        Ok(())
    }
}

impl TryFrom<String> for SemanticVersion {
    type Error = MetadataError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<SemanticVersion> for String {
    fn from(value: SemanticVersion) -> Self {
        value.to_string()
    }
}

/// Immutable reference to a governed evidence item and its verified content.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct EvidenceReference {
    evidence_id: String,
    digest: ContentDigest,
}

#[derive(Deserialize)]
struct EvidenceReferenceWire {
    evidence_id: String,
    digest: ContentDigest,
}

impl<'de> Deserialize<'de> for EvidenceReference {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = EvidenceReferenceWire::deserialize(deserializer)?;
        Self::new(wire.evidence_id, wire.digest).map_err(serde::de::Error::custom)
    }
}

impl EvidenceReference {
    /// Validates an `EVD-` namespaced evidence identifier.
    pub fn new(
        evidence_id: impl Into<String>,
        digest: ContentDigest,
    ) -> Result<Self, MetadataError> {
        let evidence_id = evidence_id.into();
        validate_reference_id(&evidence_id, "EVD-")?;
        Ok(Self {
            evidence_id,
            digest,
        })
    }
    /// Returns the governed evidence identifier.
    #[must_use]
    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }
    /// Returns the expected immutable content digest.
    #[must_use]
    pub const fn digest(&self) -> ContentDigest {
        self.digest
    }
}

/// Immutable reference to a content-addressed artifact.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct ArtifactReference {
    artifact_id: String,
    digest: ContentDigest,
    media_type: String,
}

#[derive(Deserialize)]
struct ArtifactReferenceWire {
    artifact_id: String,
    digest: ContentDigest,
    media_type: String,
}

impl<'de> Deserialize<'de> for ArtifactReference {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ArtifactReferenceWire::deserialize(deserializer)?;
        Self::new(wire.artifact_id, wire.digest, wire.media_type).map_err(serde::de::Error::custom)
    }
}

impl ArtifactReference {
    /// Validates an `ART-` identifier and its IANA-style media type.
    pub fn new(
        artifact_id: impl Into<String>,
        digest: ContentDigest,
        media_type: impl Into<String>,
    ) -> Result<Self, MetadataError> {
        let artifact_id = artifact_id.into();
        let media_type = media_type.into();
        validate_reference_id(&artifact_id, "ART-")?;
        validate_media_type(&media_type)?;
        Ok(Self {
            artifact_id,
            digest,
            media_type,
        })
    }
    /// Returns the governed artifact identifier.
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
    /// Returns the expected immutable content digest.
    #[must_use]
    pub const fn digest(&self) -> ContentDigest {
        self.digest
    }
    /// Returns the declared media type.
    #[must_use]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }
}

fn validate_reference_id(value: &str, namespace: &str) -> Result<(), MetadataError> {
    let suffix = value
        .strip_prefix(namespace)
        .ok_or(MetadataError::InvalidReferenceId)?;
    if value.len() > MAX_REFERENCE_ID_CHARS
        || suffix.is_empty()
        || !suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(MetadataError::InvalidReferenceId);
    }
    Ok(())
}

fn validate_media_type(value: &str) -> Result<(), MetadataError> {
    let Some((kind, subtype)) = value.split_once('/') else {
        return Err(MetadataError::InvalidMediaType);
    };
    let valid = !kind.is_empty()
        && !subtype.is_empty()
        && !subtype.contains('/')
        && value.len() <= MAX_MEDIA_TYPE_CHARS
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control());
    if !valid {
        return Err(MetadataError::InvalidMediaType);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO_DIGEST: &str =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn digest_round_trips_canonical_text_for_many_byte_patterns() -> Result<(), MetadataError> {
        for seed in 0_u8..=u8::MAX {
            let digest = ContentDigest::from_sha256_bytes([seed; SHA256_BYTES]);
            assert_eq!(digest.to_string().parse::<ContentDigest>()?, digest);
        }
        Ok(())
    }

    #[test]
    fn digest_rejects_noncanonical_or_wrong_length_text() {
        for invalid in [
            "",
            "sha256:00",
            "SHA256:0000",
            "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ] {
            assert_eq!(
                invalid.parse::<ContentDigest>(),
                Err(MetadataError::InvalidSha256Digest)
            );
        }
    }

    #[test]
    fn semantic_version_round_trips_full_semver() -> Result<(), MetadataError> {
        let text = "12.34.56-alpha.1+linux-x86-64";
        let version: SemanticVersion = text.parse()?;
        assert_eq!(version.to_string(), text);
        Ok(())
    }

    #[test]
    fn semantic_version_rejects_semver_boundary_violations() {
        for invalid in [
            "1.0",
            "01.0.0",
            "1.0.0-01",
            "1.0.0-",
            "1.0.0+a+b",
            "1.0.0-ä",
        ] {
            assert_eq!(
                invalid.parse::<SemanticVersion>(),
                Err(MetadataError::InvalidSemanticVersion)
            );
        }
    }

    #[test]
    fn evidence_and_artifact_references_enforce_namespaces() -> Result<(), MetadataError> {
        let digest: ContentDigest = ZERO_DIGEST.parse()?;
        assert!(EvidenceReference::new("EVD-release-42", digest).is_ok());
        assert_eq!(
            EvidenceReference::new("ART-release-42", digest),
            Err(MetadataError::InvalidReferenceId)
        );
        assert!(
            ArtifactReference::new("ART-flight-log", digest, "application/octet-stream").is_ok()
        );
        assert_eq!(
            ArtifactReference::new("EVD-flight-log", digest, "application/octet-stream"),
            Err(MetadataError::InvalidReferenceId)
        );
        Ok(())
    }

    #[test]
    fn artifact_reference_rejects_invalid_media_type() -> Result<(), MetadataError> {
        let digest: ContentDigest = ZERO_DIGEST.parse()?;
        assert_eq!(
            ArtifactReference::new("ART-flight-log", digest, "application json"),
            Err(MetadataError::InvalidMediaType)
        );
        Ok(())
    }
}
