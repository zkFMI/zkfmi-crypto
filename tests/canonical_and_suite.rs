use ml_dsa::Keypair as _;
use ml_kem::KeyExport as _;
use serde_json::json;
use zkfmi_crypto::{
    canonical::{SigningPreimage, DOMAIN},
    error::CryptoError,
    key::KeyPurpose,
    suite::*,
};

fn base() -> SigningPreimage {
    SigningPreimage {
        protocol: "q".into(),
        protocol_version: Version::V1,
        network_id: "n".into(),
        deployment_id: "d".into(),
        contract_id: "c".into(),
        tx_kind: KeyPurpose::Quote,
        object_ids: vec![],
        sequence_or_nonce: 0,
        expires_at: 1,
        suite: Suite::new(SuiteId::Ed25519),
        body_hash: [0; 32],
    }
}

#[test]
fn golden_minimal_quote() {
    let expected = concat!(
        "5a4b464d493a43414e4f4e4943414c3a7631",
        "0000000171",
        "0001",
        "000000016e",
        "0000000164",
        "0000000163",
        "0001",
        "00000000",
        "0000000000000000",
        "0000000000000001",
        "00010001",
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert_eq!(hex::encode(base().encode().unwrap()), expected);
}

#[test]
fn golden_hybrid_settlement_two_objects() {
    let mut p = base();
    p.tx_kind = KeyPurpose::SettlementInstruction;
    p.object_ids = vec!["a".into(), "bc".into()];
    p.sequence_or_nonce = 42;
    p.expires_at = 256;
    p.suite = Suite::new(SuiteId::Ed25519MlDsa65);
    p.body_hash = [255; 32];
    let expected = concat!(
        "5a4b464d493a43414e4f4e4943414c3a7631",
        "0000000171",
        "0001",
        "000000016e",
        "0000000164",
        "0000000163",
        "0002",
        "00000002",
        "0000000161",
        "000000026263",
        "000000000000002a",
        "0000000000000100",
        "02020001",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    assert_eq!(hex::encode(p.encode().unwrap()), expected);
}

#[test]
fn golden_utf8_and_u64_limits() {
    let mut p = base();
    p.network_id = "jp".into();
    p.deployment_id = "dev".into();
    p.contract_id = "合".into();
    p.tx_kind = KeyPurpose::KeyRotation;
    p.object_ids = vec!["é".into()];
    p.sequence_or_nonce = u64::MAX;
    p.expires_at = u64::MAX;
    p.suite = Suite::new(SuiteId::MlDsa65);
    p.body_hash = std::array::from_fn(|i| i as u8);
    let expected = concat!(
        "5a4b464d493a43414e4f4e4943414c3a7631",
        "0000000171",
        "0001",
        "000000026a70",
        "00000003646576",
        "00000003e59088",
        "0003",
        "00000001",
        "00000002c3a9",
        "ffffffffffffffff",
        "ffffffffffffffff",
        "01020001",
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    );
    assert_eq!(hex::encode(p.encode().unwrap()), expected);
}

#[test]
fn all_replay_boundaries_change_the_preimage() {
    let p = base();
    let original = p.encode().unwrap();
    let mut variants = Vec::new();
    let mut v = p.clone();
    v.protocol.push('2');
    variants.push(v);
    let mut v = p.clone();
    v.network_id.push('2');
    variants.push(v);
    let mut v = p.clone();
    v.deployment_id.push('2');
    variants.push(v);
    let mut v = p.clone();
    v.contract_id.push('2');
    variants.push(v);
    let mut v = p.clone();
    v.tx_kind = KeyPurpose::SettlementInstruction;
    variants.push(v);
    let mut v = p.clone();
    v.object_ids.push("q1".into());
    variants.push(v);
    let mut v = p.clone();
    v.sequence_or_nonce += 1;
    variants.push(v);
    let mut v = p.clone();
    v.expires_at += 1;
    variants.push(v);
    let mut v = p.clone();
    v.suite = Suite::new(SuiteId::MlDsa65);
    variants.push(v);
    let mut v = p.clone();
    v.body_hash[0] = 1;
    variants.push(v);
    for variant in variants {
        assert_ne!(variant.encode().unwrap(), original);
    }
    assert!(original.starts_with(DOMAIN));
}

#[test]
fn length_framing_prevents_concatenation_ambiguity() {
    let mut left = base();
    let mut right = base();
    left.object_ids = vec!["ab".into(), "c".into()];
    right.object_ids = vec!["a".into(), "bc".into()];
    assert_ne!(left.encode().unwrap(), right.encode().unwrap());
    right.object_ids.reverse();
    assert_ne!(left.encode().unwrap(), right.encode().unwrap());
}

#[test]
fn canonical_rejects_expiry_and_empty_identifiers() {
    assert!(base().validate_at(0).is_ok());
    assert_eq!(base().validate_at(1), Err(CryptoError::Expired));
    let mut invalid = base();
    invalid.network_id.clear();
    assert_eq!(invalid.encode(), Err(CryptoError::InvalidEncoding));
}

#[test]
fn wire_rejects_unknown_fields_suites_and_versions() {
    let p = base();
    let valid = serde_json::to_value(&p).unwrap();
    assert_eq!(
        serde_json::from_value::<SigningPreimage>(valid.clone()).unwrap(),
        p
    );
    let mut v = valid.clone();
    v["surprise"] = json!(true);
    assert!(serde_json::from_value::<SigningPreimage>(v).is_err());
    let mut v = valid.clone();
    v["protocol_version"] = json!(2);
    assert!(serde_json::from_value::<SigningPreimage>(v).is_err());
    let mut v = valid.clone();
    v["suite"]["version"] = json!(0);
    assert!(serde_json::from_value::<SigningPreimage>(v).is_err());
    let mut v = valid.clone();
    v["suite"]["id"] = json!("FutureSuite");
    assert!(serde_json::from_value::<SigningPreimage>(v).is_err());
    let mut v = valid;
    v["suite"]["extension"] = json!(1);
    assert!(serde_json::from_value::<SigningPreimage>(v).is_err());
    assert!(SuiteId::try_from(0xffff).is_err());
    assert!(Version::try_from(2).is_err());
}

#[test]
fn fips_sizes_match_actual_rustcrypto_encodings() {
    let seed = ml_dsa::Seed::from([17; 32]);
    let s44 = ml_dsa::SigningKey::<ml_dsa::MlDsa44>::from_seed(&seed);
    let s65 = ml_dsa::SigningKey::<ml_dsa::MlDsa65>::from_seed(&seed);
    let s87 = ml_dsa::SigningKey::<ml_dsa::MlDsa87>::from_seed(&seed);
    assert_eq!(s44.verifying_key().encode().len(), ML_DSA_44_PK_BYTES);
    assert_eq!(s65.verifying_key().encode().len(), ML_DSA_65_PK_BYTES);
    assert_eq!(s87.verifying_key().encode().len(), ML_DSA_87_PK_BYTES);
    assert_eq!(
        s44.expanded_key()
            .sign_deterministic(b"sizes", &[])
            .unwrap()
            .encode()
            .len(),
        ML_DSA_44_SIG_BYTES
    );
    assert_eq!(
        s65.expanded_key()
            .sign_deterministic(b"sizes", &[])
            .unwrap()
            .encode()
            .len(),
        ML_DSA_65_SIG_BYTES
    );
    assert_eq!(
        s87.expanded_key()
            .sign_deterministic(b"sizes", &[])
            .unwrap()
            .encode()
            .len(),
        ML_DSA_87_SIG_BYTES
    );
    let dk = ml_kem::DecapsulationKey::<ml_kem::MlKem768>::from_seed(ml_kem::Seed::from([19; 64]));
    let ek = dk.encapsulation_key();
    assert_eq!(ek.to_bytes().len(), ML_KEM_768_EK_BYTES);
    assert_eq!(
        ek.encapsulate_deterministic(&ml_kem::B32::from([21; 32]))
            .0
            .len(),
        ML_KEM_768_CT_BYTES
    );
    #[allow(deprecated)]
    {
        use ml_kem::ExpandedKeyEncoding as _;
        assert_eq!(dk.to_expanded_bytes().len(), ML_KEM_768_DK_BYTES);
    }
    assert_eq!(
        SuiteId::SlhDsaSha2_128s.metadata().public_key_bytes,
        Some(32)
    );
    assert_eq!(
        SuiteId::SlhDsaSha2_128s.metadata().signature_bytes,
        Some(7856)
    );
    assert_eq!(3 * ML_DSA_65_SIG_BYTES, 9927);
    assert_eq!(3 * ML_DSA_44_SIG_BYTES, 7260);
}
