use serde_json::json;
use zkfmi_crypto::{
    backend::{Ed25519Signer, Provider},
    canonical::SigningPreimage,
    error::CryptoError,
    hybrid::signature::HybridSigner,
    key::{
        KeyId, KeyPurpose, KeyRecord, KeyRegistry, ParticipantId, RegistrySnapshot, RotationProof,
    },
    suite::{Suite, SuiteId, Version},
    traits::Signer,
};

fn record(signer: &dyn Signer, id: &str, version: u32) -> KeyRecord {
    KeyRecord {
        participant_id: ParticipantId::new("participant-1").unwrap(),
        key_id: KeyId::new(id).unwrap(),
        suite: signer.suite(),
        key_version: version,
        purpose: KeyPurpose::Quote,
        public_key: signer.public_key(),
        not_before: 10,
        not_after: 100,
        revoked_at: None,
        rotation_proof: None,
        dekyx_binding: Some("issuer:epoch:reference".into()),
    }
}

fn transition() -> (KeyRegistry, KeyRecord) {
    let a = Ed25519Signer::from_seed(&[1; 32]);
    let b = Ed25519Signer::from_seed(&[2; 32]);
    let old = record(&a, "old", 1);
    let mut new = record(&b, "new", 2);
    new.rotation_proof = Some(RotationProof::create(&old, &new, &a, &b).unwrap());
    let mut registry = KeyRegistry::default();
    registry.insert_initial(old).unwrap();
    (registry, new)
}

#[test]
fn validity_is_start_inclusive_end_exclusive_and_revocation_inclusive() {
    let signer = Ed25519Signer::from_seed(&[1; 32]);
    let mut key = record(&signer, "a", 1);
    assert_eq!(key.valid_at(9), Err(CryptoError::NotYetValid));
    assert!(key.valid_at(10).is_ok());
    assert!(key.valid_at(99).is_ok());
    assert_eq!(key.valid_at(100), Err(CryptoError::Expired));
    key.revoked_at = Some(50);
    assert!(key.valid_at(49).is_ok());
    assert_eq!(key.valid_at(50), Err(CryptoError::Revoked));
    key.not_after = key.not_before;
    assert!(key.validate().is_err());
}

#[test]
fn unknown_key_version_wrong_purpose_and_revocation_fail_closed() {
    let (mut registry, _) = transition();
    let id = KeyId::new("old").unwrap();
    assert_eq!(
        registry.lookup(&id, 2, KeyPurpose::Quote, 20),
        Err(CryptoError::UnknownVersion)
    );
    assert_eq!(
        registry.lookup(&id, 1, KeyPurpose::Governance, 20),
        Err(CryptoError::InvalidPurpose)
    );
    assert_eq!(
        registry.get(&KeyId::new("missing").unwrap()),
        Err(CryptoError::NotFound)
    );
    registry.revoke(&id, 20).unwrap();
    registry.revoke(&id, 30).unwrap();
    assert_eq!(registry.get(&id).unwrap().revoked_at, Some(20));
    assert_eq!(
        registry.lookup(&id, 1, KeyPurpose::Quote, 20),
        Err(CryptoError::Revoked)
    );
}

#[test]
fn valid_bidirectional_rotation_preserves_participant_and_retires_old_key() {
    let (mut registry, new) = transition();
    let participant = new.participant_id.clone();
    registry
        .rotate(new, 20, &Provider::rustcrypto().unwrap())
        .unwrap();
    let old = registry.get(&KeyId::new("old").unwrap()).unwrap();
    assert_eq!(old.revoked_at, Some(20));
    let new = registry
        .lookup(&KeyId::new("new").unwrap(), 2, KeyPurpose::Quote, 20)
        .unwrap();
    assert_eq!(new.participant_id, participant);
    assert_eq!(old.participant_id, participant);
}

#[test]
fn rotation_rejects_either_missing_invalid_or_swapped_direction_atomically() {
    for case in 0..5 {
        let (mut registry, mut new) = transition();
        let proof = new.rotation_proof.as_mut().unwrap();
        match case {
            0 => proof.old_to_new.clear(),
            1 => proof.new_to_old.clear(),
            2 => proof.old_to_new[0] ^= 1,
            3 => proof.new_to_old[0] ^= 1,
            _ => std::mem::swap(&mut proof.old_to_new, &mut proof.new_to_old),
        }
        assert!(registry
            .rotate(new, 20, &Provider::rustcrypto().unwrap())
            .is_err());
        assert!(registry
            .get(&KeyId::new("old").unwrap())
            .unwrap()
            .valid_at(20)
            .is_ok());
        assert!(registry.get(&KeyId::new("new").unwrap()).is_err());
    }
}

#[test]
fn rotation_binds_new_key_identity_purpose_validity_and_dekyx_reference() {
    for case in 0..8 {
        let (mut registry, mut new) = transition();
        match case {
            0 => new.participant_id = ParticipantId::new("other-participant").unwrap(),
            1 => new.key_id = KeyId::new("other-new-key").unwrap(),
            2 => new.key_version = 3,
            3 => new.purpose = KeyPurpose::Governance,
            4 => new.not_after += 1,
            5 => new.dekyx_binding = Some("other-issuer".into()),
            6 => new.public_key[0] ^= 1,
            _ => new.rotation_proof.as_mut().unwrap().old_key_version = 2,
        }
        assert!(
            registry
                .rotate(new, 20, &Provider::rustcrypto().unwrap())
                .is_err(),
            "case {case}"
        );
    }
}

#[test]
fn rotation_rejects_stale_revoked_expired_or_future_keys_and_replay() {
    let provider = Provider::rustcrypto().unwrap();
    let (mut registry, new) = transition();
    registry.revoke(&KeyId::new("old").unwrap(), 20).unwrap();
    assert!(registry.rotate(new, 20, &provider).is_err());
    let (mut registry, new) = transition();
    assert!(registry.rotate(new, 100, &provider).is_err());
    let (mut registry, new) = transition();
    assert!(registry.rotate(new, 9, &provider).is_err());
    let (mut registry, new) = transition();
    registry.rotate(new.clone(), 20, &provider).unwrap();
    assert_eq!(
        registry.rotate(new, 21, &provider),
        Err(CryptoError::DuplicateKey)
    );
}

#[test]
fn classical_to_hybrid_rotation_requires_both_successor_signatures() {
    let a = Ed25519Signer::from_seed(&[1; 32]);
    let b = HybridSigner::generate().unwrap();
    let old = record(&a, "old", 1);
    let mut new = record(&b, "new", 2);
    new.rotation_proof = Some(RotationProof::create(&old, &new, &a, &b).unwrap());
    let mut registry = KeyRegistry::default();
    registry.insert_initial(old).unwrap();
    let mut invalid = new.clone();
    invalid.rotation_proof.as_mut().unwrap().new_to_old[70] ^= 1;
    assert!(registry
        .rotate(invalid, 20, &Provider::rustcrypto().unwrap())
        .is_err());
    registry
        .rotate(new, 20, &Provider::rustcrypto().unwrap())
        .unwrap();
}

#[test]
fn pq_to_classical_only_rotation_is_rejected() {
    let a = HybridSigner::generate().unwrap();
    let b = Ed25519Signer::from_seed(&[1; 32]);
    let old = record(&a, "old", 1);
    let mut new = record(&b, "new", 2);
    new.rotation_proof = Some(RotationProof::create(&old, &new, &a, &b).unwrap());
    let mut registry = KeyRegistry::default();
    registry.insert_initial(old).unwrap();
    assert_eq!(
        registry.rotate(new, 20, &Provider::rustcrypto().unwrap()),
        Err(CryptoError::InvalidRotation)
    );
}

#[test]
fn preimage_verification_applies_key_lifecycle_suite_purpose_and_expiry() {
    let signer = HybridSigner::generate().unwrap();
    let key = record(&signer, "hybrid", 1);
    let id = key.key_id.clone();
    let mut registry = KeyRegistry::default();
    registry.insert_initial(key).unwrap();
    let provider = Provider::rustcrypto().unwrap();
    let mut p = SigningPreimage {
        protocol: "qomm".into(),
        protocol_version: Version::V1,
        network_id: "net".into(),
        deployment_id: "defmi".into(),
        contract_id: "rfq".into(),
        tx_kind: KeyPurpose::Quote,
        object_ids: vec!["quote-1".into()],
        sequence_or_nonce: 1,
        expires_at: 30,
        suite: signer.suite(),
        body_hash: [3; 32],
    };
    let sig = signer.sign(p.tx_kind, &p.encode().unwrap()).unwrap();
    assert!(registry
        .verify_preimage(&id, 1, 20, &p, &sig, &provider)
        .is_ok());
    assert!(registry
        .verify_preimage(&id, 1, 30, &p, &sig, &provider)
        .is_err());
    assert!(registry
        .verify_preimage(&id, 2, 20, &p, &sig, &provider)
        .is_err());
    p.tx_kind = KeyPurpose::SettlementInstruction;
    assert!(registry
        .verify_preimage(&id, 1, 20, &p, &sig, &provider)
        .is_err());
    p.tx_kind = KeyPurpose::Quote;
    p.suite = Suite::new(SuiteId::Ed25519);
    assert!(registry
        .verify_preimage(&id, 1, 20, &p, &sig, &provider)
        .is_err());
    p.suite = signer.suite();
    registry.revoke(&id, 20).unwrap();
    assert!(registry
        .verify_preimage(&id, 1, 20, &p, &sig, &provider)
        .is_err());
}

#[test]
fn public_records_are_strict_serde_and_identifiers_are_nonempty() {
    let (registry, new) = transition();
    assert_eq!(
        serde_json::from_value::<KeyRecord>(serde_json::to_value(&new).unwrap()).unwrap(),
        new
    );
    for path in ["extra", "suite", "rotation_proof"] {
        let mut v = serde_json::to_value(&new).unwrap();
        if path == "extra" {
            v[path] = json!(1);
        } else {
            v[path]["extra"] = json!(1);
        }
        assert!(serde_json::from_value::<KeyRecord>(v).is_err());
    }
    let mut v = serde_json::to_value(&new).unwrap();
    v["rotation_proof"]["version"] = json!(2);
    assert!(serde_json::from_value::<KeyRecord>(v).is_err());
    assert!(serde_json::from_str::<ParticipantId>("\"\"").is_err());
    assert!(KeyId::new("bad\nidentifier").is_err());
    let mut snapshot = serde_json::to_value(registry.snapshot()).unwrap();
    snapshot["extra"] = json!(1);
    assert!(serde_json::from_value::<RegistrySnapshot>(snapshot).is_err());
}

#[test]
fn enrollment_rejects_invalid_shapes_and_cannot_overwrite_existing_keys() {
    let (mut registry, mut new) = transition();
    assert!(registry.insert_initial(new.clone()).is_err());
    new.key_version = 1;
    new.rotation_proof = None;
    new.public_key.clear();
    assert!(registry.insert_initial(new).is_err());
    let existing = registry.get(&KeyId::new("old").unwrap()).unwrap().clone();
    assert_eq!(
        registry.insert_initial(existing),
        Err(CryptoError::DuplicateKey)
    );
}
