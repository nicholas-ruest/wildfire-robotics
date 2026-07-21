//! Adversarial lifecycle tests for `IA-INV-001` and `IA-INV-003`.

use crate::domain::*;
use chrono::{DateTime, Duration, Utc};
use shared_kernel::PrincipalId;
use uuid::Uuid;

fn principal(value: u128) -> PrincipalId {
    PrincipalId::from_uuid(Uuid::from_u128(value))
}
fn now() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::days(20_000)
}
fn provenance() -> Result<VerifiedProvenance, IdentityError> {
    VerifiedProvenance::new(
        "idp",
        "unique-subject",
        "evidence",
        now(),
        AuthenticatorAssurance::MultiFactor,
    )
}
fn attestation(root: &str, key: &str) -> AttestationEvidence {
    AttestationEvidence {
        evidence_id: "attestation-1".into(),
        hardware_key_id: key.into(),
        trust_root_version: root.into(),
        software_version: "1.2.3".into(),
        configuration_digest: "a".repeat(64),
        attested_at: now() - Duration::minutes(1),
        expires_at: now() + Duration::minutes(5),
        hardware_backed: true,
        compatible_and_not_revoked: true,
    }
}

#[test]
fn principal_requires_verified_provenance_before_activation() {
    let mut value = Principal::register(principal(1), PrincipalKind::Human, principal(2));
    assert_eq!(
        value.activate(),
        Err(IdentityError::IllegalPrincipalTransition)
    );
}

#[test]
fn principal_transition_table_denies_reactivation_after_disable() -> Result<(), IdentityError> {
    let mut value = Principal::register(principal(1), PrincipalKind::Workload, principal(2));
    value.verify_identity(provenance()?)?;
    value.activate()?;
    value.suspend("investigation")?;
    value.activate()?;
    value.disable("retired")?;
    assert_eq!(
        value.activate(),
        Err(IdentityError::IllegalPrincipalTransition)
    );
    assert_eq!(
        value.suspend("again"),
        Err(IdentityError::IllegalPrincipalTransition)
    );
    Ok(())
}

#[test]
fn device_rejects_software_revocation_nonhardware_and_expired_evidence() -> Result<(), IdentityError>
{
    for mut evidence in [
        attestation("root-1", "key-1"),
        attestation("root-1", "key-2"),
        attestation("root-1", "key-3"),
    ] {
        let mut device =
            DeviceIdentity::manufactured(DeviceId::from_opaque_id(principal(3)), principal(2));
        device.begin_enrollment()?;
        if evidence.hardware_key_id == "key-1" {
            evidence.hardware_backed = false;
        }
        if evidence.hardware_key_id == "key-2" {
            evidence.compatible_and_not_revoked = false;
        }
        if evidence.hardware_key_id == "key-3" {
            evidence.expires_at = now();
        }
        assert!(!device.is_trusted_at(now()));
        assert!(device.attest(evidence, now()).is_err());
    }
    Ok(())
}

#[test]
fn trusted_device_expires_and_key_rotation_requires_a_new_key() -> Result<(), IdentityError> {
    let mut device = DeviceIdentity::imported(DeviceId::from_opaque_id(principal(3)), principal(2));
    device.begin_enrollment()?;
    device.attest(attestation("root-1", "key-1"), now())?;
    assert!(device.is_trusted_at(now()));
    assert!(!device.is_trusted_at(now() + Duration::minutes(6)));
    assert_eq!(
        device.rotate_key(attestation("root-2", "key-1"), now()),
        Err(IdentityError::KeyWasNotRotated)
    );
    device.rotate_key(attestation("root-2", "key-2"), now())?;
    assert!(device.is_trusted_at(now()));
    Ok(())
}

#[test]
fn quarantine_requires_fresh_attestation_before_trust_returns() -> Result<(), IdentityError> {
    let mut device =
        DeviceIdentity::manufactured(DeviceId::from_opaque_id(principal(3)), principal(2));
    device.begin_enrollment()?;
    device.attest(attestation("root-1", "key-1"), now())?;
    device.quarantine("root compromise")?;
    assert!(!device.is_trusted_at(now()));
    assert_eq!(
        device.attest(attestation("root-2", "key-2"), now()),
        Err(IdentityError::IllegalDeviceTransition)
    );
    device.begin_enrollment()?;
    device.attest(attestation("root-2", "key-2"), now())?;
    assert!(device.is_trusted_at(now()));
    Ok(())
}

#[test]
fn credentials_must_be_short_lived_and_root_attributed() {
    let valid = IssuedCredential {
        credential_id: "credential".into(),
        issuer: "pki".into(),
        issued_at: now(),
        expires_at: now() + Duration::minutes(5),
        trust_root_version: "root-2".into(),
    };
    assert!(valid.validate(Duration::minutes(10)).is_ok());
    let excessive = IssuedCredential {
        expires_at: now() + Duration::hours(1),
        ..valid.clone()
    };
    assert_eq!(
        excessive.validate(Duration::minutes(10)),
        Err(IdentityError::CredentialLifetimeInvalid)
    );
    let unknown_root = IssuedCredential {
        trust_root_version: String::new(),
        ..valid
    };
    assert_eq!(
        unknown_root.validate(Duration::minutes(10)),
        Err(IdentityError::InvalidValue)
    );
}
