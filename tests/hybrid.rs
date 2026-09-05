use zkfmi_crypto::{
    backend::{
        Ed25519Signer, Ed25519Verifier, MlDsa65Signer, MlDsa65Verifier, MlKem768Encapsulator,
        MlKem768Key, Provider,
    },
    error::CryptoError,
    hybrid::{
        kem::{combine, HybridCiphertext, HybridKemEncapsulator, HybridKemKey},
        signature::{HybridSignature, HybridSigner, HybridVerifier},
    },
    key::KeyPurpose,
    suite::{Suite, SuiteId},
    traits::{CryptoProvider, KemDecapsulator, KemEncapsulator, Signer, Verifier},
};

fn signer() -> HybridSigner {
    HybridSigner::new(
        Ed25519Signer::from_seed(&[11; 32]),
        MlDsa65Signer::from_seed(&[13; 32]),
    )
}

fn quadrant(bad_classical: bool, bad_pq: bool) {
    let signer = signer();
    let mut signature = signer
        .sign_hybrid(KeyPurpose::Quote, b"same message")
        .unwrap();
    if bad_classical {
        signature.classical[0] ^= 1;
    }
    if bad_pq {
        signature.pq[0] ^= 1;
    }
    let result = HybridVerifier.verify_hybrid(
        KeyPurpose::Quote,
        &signer.public_key(),
        b"same message",
        &signature,
    );
    assert_eq!(result.is_ok(), !bad_classical && !bad_pq);
}

#[test]
fn signature_accepts_both_valid() {
    quadrant(false, false);
}
#[test]
fn signature_rejects_classical_invalid() {
    quadrant(true, false);
}
#[test]
fn signature_rejects_pq_invalid() {
    quadrant(false, true);
}
#[test]
fn signature_rejects_both_invalid() {
    quadrant(true, true);
}

#[test]
fn signature_rejects_missing_swapped_truncated_and_appended_components() {
    let signer = signer();
    let pk = signer.public_key();
    let sig = signer.sign_hybrid(KeyPurpose::Quote, b"m").unwrap();
    let verify =
        |s: &HybridSignature| HybridVerifier.verify_hybrid(KeyPurpose::Quote, &pk, b"m", s);
    let mut bad = sig.clone();
    bad.pq.clear();
    assert!(verify(&bad).is_err());
    let mut bad = sig.clone();
    bad.classical.clear();
    assert!(verify(&bad).is_err());
    let bad = HybridSignature {
        classical: sig.pq.clone(),
        pq: sig.classical.clone(),
    };
    assert!(verify(&bad).is_err());
    let wire = sig.encode().unwrap();
    assert!(HybridSignature::decode(&wire[..wire.len() - 1]).is_err());
    let mut extended = wire;
    extended.push(0);
    assert!(HybridSignature::decode(&extended).is_err());
    let mut value = serde_json::to_value(&sig).unwrap();
    value.as_object_mut().unwrap().remove("pq");
    assert!(serde_json::from_value::<HybridSignature>(value).is_err());
    let mut value = serde_json::to_value(&sig).unwrap();
    value["future"] = serde_json::json!(1);
    assert!(serde_json::from_value::<HybridSignature>(value).is_err());
}

#[test]
fn signature_binds_purpose_message_and_hybrid_suite_context() {
    let signer = signer();
    let pk = signer.public_key();
    let sig = signer.sign_hybrid(KeyPurpose::Quote, b"m").unwrap();
    assert!(HybridVerifier
        .verify_hybrid(KeyPurpose::SettlementInstruction, &pk, b"m", &sig)
        .is_err());
    assert!(HybridVerifier
        .verify_hybrid(KeyPurpose::Quote, &pk, b"other", &sig)
        .is_err());
    assert!(Ed25519Verifier
        .verify(KeyPurpose::Quote, &pk[..32], b"m", &sig.classical)
        .is_err());
    assert!(MlDsa65Verifier
        .verify(KeyPurpose::Quote, &pk[32..], b"m", &sig.pq)
        .is_err());
}

#[test]
fn independent_standalone_signatures_cannot_be_lifted_into_hybrid() {
    let ed = Ed25519Signer::from_seed(&[11; 32]);
    let pq = MlDsa65Signer::from_seed(&[13; 32]);
    let sig = HybridSignature {
        classical: ed.sign(KeyPurpose::Quote, b"m").unwrap(),
        pq: pq.sign(KeyPurpose::Quote, b"m").unwrap(),
    };
    assert!(HybridVerifier
        .verify_hybrid(KeyPurpose::Quote, &signer().public_key(), b"m", &sig)
        .is_err());
}

#[test]
fn fresh_os_keys_work_through_backend_neutral_provider() {
    let provider = Provider::rustcrypto().unwrap();
    for signer in [
        Box::new(Ed25519Signer::generate().unwrap()) as Box<dyn Signer>,
        Box::new(MlDsa65Signer::generate().unwrap()),
        Box::new(HybridSigner::generate().unwrap()),
    ] {
        let sig = signer.sign(KeyPurpose::Governance, b"governance").unwrap();
        let verifier = provider.verifier(signer.suite()).unwrap();
        assert!(verifier
            .verify(
                KeyPurpose::Governance,
                &signer.public_key(),
                b"governance",
                &sig
            )
            .is_ok());
        assert!(verifier
            .verify(KeyPurpose::Quote, &signer.public_key(), b"governance", &sig)
            .is_err());
    }
}

#[test]
fn provider_rejects_unregistered_capabilities_and_duplicate_registration() {
    let mut p = Provider::default();
    assert!(p.verifier(Suite::new(SuiteId::Ed25519)).is_err());
    p.register_verifier(Box::new(Ed25519Verifier)).unwrap();
    assert_eq!(
        p.register_verifier(Box::new(Ed25519Verifier)),
        Err(CryptoError::DuplicateKey)
    );
    let p = Provider::rustcrypto().unwrap();
    assert!(p.verifier(Suite::new(SuiteId::MlDsa44)).is_err());
    assert!(p.verifier(Suite::new(SuiteId::FrostRistretto255)).is_err());
    assert!(p
        .proof_verifier(Suite::new(SuiteId::BulletproofsRistretto255))
        .is_err());
    assert!(p
        .kem_encapsulator(Suite::new(SuiteId::X25519Tls13))
        .is_err());
}

#[test]
fn kem_real_roundtrip_and_ciphertext_integrity() {
    let key = HybridKemKey::generate().unwrap();
    let result = HybridKemEncapsulator
        .encapsulate(&key.public_key())
        .unwrap();
    assert_eq!(key.public_key().len(), 1216);
    assert_eq!(result.ciphertext.len(), 1120);
    let received = key.decapsulate(&result.ciphertext).unwrap();
    assert!(received.as_slice() == result.shared_secret.as_slice());
    let mut modified = result.ciphertext.clone();
    modified[33] ^= 1;
    // FIPS 203 implicit rejection returns a pseudorandom secret, never a classical fallback.
    let rejected_secret = key.decapsulate(&modified).unwrap();
    assert!(rejected_secret.as_slice() != result.shared_secret.as_slice());
    assert!(key.decapsulate(&result.ciphertext[..1119]).is_err());
    let other_key = HybridKemKey::generate().unwrap();
    assert!(
        other_key
            .decapsulate(&result.ciphertext)
            .unwrap()
            .as_slice()
            != received.as_slice()
    );
}

#[test]
fn kem_rejects_low_order_x25519_and_bad_mlkem_keys() {
    let key = HybridKemKey::generate().unwrap();
    let mut pk = key.public_key();
    pk[..32].fill(0);
    assert!(HybridKemEncapsulator.encapsulate(&pk).is_err());
    let mut pk = key.public_key();
    pk[32..].fill(255);
    assert!(HybridKemEncapsulator.encapsulate(&pk).is_err());
    let mut ct = HybridKemEncapsulator
        .encapsulate(&key.public_key())
        .unwrap()
        .ciphertext;
    ct[..32].fill(0);
    assert!(key.decapsulate(&ct).is_err());
    assert!(HybridKemEncapsulator
        .encapsulate(&key.public_key()[..1215])
        .is_err());
}

#[test]
fn kem_combiner_depends_on_both_secrets_and_both_ciphertexts() {
    let suite = Suite::new(SuiteId::X25519MlKem768);
    let ct = HybridCiphertext {
        classical: vec![3; 32],
        pq: vec![4; 1088],
    };
    let baseline = combine(&[1; 32], &[2; 32], &ct, suite).unwrap();
    assert!(combine(&[0; 32], &[2; 32], &ct, suite).unwrap().as_slice() != baseline.as_slice());
    assert!(combine(&[1; 32], &[0; 32], &ct, suite).unwrap().as_slice() != baseline.as_slice());
    let mut changed = ct.clone();
    changed.classical[0] ^= 1;
    assert!(
        combine(&[1; 32], &[2; 32], &changed, suite)
            .unwrap()
            .as_slice()
            != baseline.as_slice()
    );
    let mut changed = ct.clone();
    changed.pq[0] ^= 1;
    assert!(
        combine(&[1; 32], &[2; 32], &changed, suite)
            .unwrap()
            .as_slice()
            != baseline.as_slice()
    );
    assert!(combine(&[1; 32], &[2; 32], &ct, Suite::new(SuiteId::MlKem768)).is_err());
}

#[test]
fn standalone_mlkem_uses_real_backend_and_strict_lengths() {
    let key = MlKem768Key::generate().unwrap();
    let result = MlKem768Encapsulator.encapsulate(&key.public_key()).unwrap();
    assert!(
        key.decapsulate(&result.ciphertext).unwrap().as_slice() == result.shared_secret.as_slice()
    );
    assert!(key.decapsulate(&[]).is_err());
    assert!(MlKem768Encapsulator.encapsulate(&[]).is_err());
}

#[test]
fn ciphertext_wire_rejects_unknown_fields_and_missing_parts() {
    let ct = HybridCiphertext {
        classical: vec![3; 32],
        pq: vec![4; 1088],
    };
    assert_eq!(HybridCiphertext::decode(&ct.encode().unwrap()).unwrap(), ct);
    let mut value = serde_json::to_value(&ct).unwrap();
    value["extra"] = serde_json::json!(1);
    assert!(serde_json::from_value::<HybridCiphertext>(value).is_err());
    let mut value = serde_json::to_value(&ct).unwrap();
    value.as_object_mut().unwrap().remove("classical");
    assert!(serde_json::from_value::<HybridCiphertext>(value).is_err());
}
