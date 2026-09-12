//! Independent, publicly published NIST/RFC fixtures; no production secret material.
use super::*;
use serde_json::Value;

#[test]
fn portable_c_reference_encapsulation_matches_rustcrypto() {
    // Pinned upstream public known answer, executed through the same portable
    // C source/configuration as the circuit adapter. No operational secret.
    let vector = include_bytes!("../tests/vectors/mlkem-native-768-known-answer.bin");
    assert_eq!(vector.len(), 32 + 1184 + 32 + 1088);
    let coins: &[u8; 32] = vector[..32].try_into().unwrap();
    let output = MlKem768Encapsulator
        .encapsulate_with_coins(&vector[32..1216], coins)
        .unwrap();
    assert_eq!(&*output.shared_secret, &vector[1216..1248]);
    assert_eq!(&output.ciphertext, &vector[1248..]);
}

#[test]
fn portable_bearssl_x25519_matches_dalek_with_nonuniform_scalar() {
    use x25519_dalek::{PublicKey, StaticSecret};
    let vector = include_bytes!("../tests/vectors/bearssl-x25519-known-answer.bin");
    assert_eq!(vector.len(), 128);
    let seed: [u8; 32] = vector[..32].try_into().unwrap();
    assert_ne!(seed[0], seed[31]);
    let key = StaticSecret::from(seed);
    let recipient = PublicKey::from(<[u8; 32]>::try_from(&vector[32..64]).unwrap());
    assert_eq!(PublicKey::from(&key).as_bytes(), &vector[64..96]);
    let shared = key.diffie_hellman(&recipient);
    assert!(shared.was_contributory());
    assert_eq!(shared.as_bytes(), &vector[96..]);
}

fn bytes(test: &Value, field: &str) -> Vec<u8> {
    hex::decode(test[field].as_str().unwrap()).unwrap()
}

fn sig_case(id: u64) {
    let document: Value =
        serde_json::from_str(include_str!("../tests/vectors/ml-dsa-65-sigver.json")).unwrap();
    assert_eq!(document["algorithm"], "ML-DSA");
    let group = &document["testGroups"][0];
    assert_eq!(group["parameterSet"], "ML-DSA-65");
    assert_eq!(group["signatureInterface"], "external");
    assert_eq!(group["preHash"], "pure");
    assert_eq!(group["tests"].as_array().unwrap().len(), 4);
    let t = group["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["tcId"] == id)
        .unwrap();
    let result = verify_mldsa65_context(
        &bytes(t, "pk"),
        &bytes(t, "message"),
        &bytes(t, "context"),
        &bytes(t, "signature"),
    );
    assert_eq!(
        result.is_ok(),
        t["testPassed"].as_bool().unwrap(),
        "ACVP sigVer tcId={id}"
    );
}

#[test]
fn acvp_mldsa65_31_modified_z() {
    sig_case(31);
}
#[test]
fn acvp_mldsa65_33_valid() {
    sig_case(33);
}
#[test]
fn acvp_mldsa65_42_modified_commitment_max_context() {
    sig_case(42);
}
#[test]
fn acvp_mldsa65_43_valid() {
    sig_case(43);
}

#[allow(deprecated)] // NIST ACVP publishes the FIPS expanded dk, not the newer seed serialization.
fn kem_case(id: u64, function: &str) {
    let document: Value =
        serde_json::from_str(include_str!("../tests/vectors/ml-kem-768-encapdecap.json")).unwrap();
    assert_eq!(document["algorithm"], "ML-KEM");
    let group = document["testGroups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["function"] == function)
        .unwrap();
    assert_eq!(group["parameterSet"], "ML-KEM-768");
    assert_eq!(group["tests"].as_array().unwrap().len(), 2);
    let t = group["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["tcId"] == id)
        .unwrap();
    let encoded =
        ml_kem::ExpandedDecapsulationKey::<MlKem768>::try_from(bytes(t, "dk").as_slice()).unwrap();
    let key = MlKem768Key {
        key: ml_kem::DecapsulationKey::from_expanded(&encoded).unwrap(),
    };
    assert_eq!(key.public_key(), bytes(t, "ek"));
    let actual = key.decapsulate(&bytes(t, "c")).unwrap();
    assert!(
        actual.as_slice() == bytes(t, "k").as_slice(),
        "ACVP decapsulation tcId={id}"
    );
    if function == "encapsulation" {
        let m = ml_kem::B32::try_from(bytes(t, "m").as_slice()).unwrap();
        // Exercise the same upstream encapsulation operation as the OS-random adapter.
        let (ct, secret) = key.key.encapsulation_key().encapsulate_deterministic(&m);
        assert_eq!(ct.as_slice(), bytes(t, "c"));
        assert!(secret.as_slice() == bytes(t, "k").as_slice());
    }
}

#[test]
fn acvp_mlkem768_26_encapsulation() {
    kem_case(26, "encapsulation");
}
#[test]
fn acvp_mlkem768_27_encapsulation() {
    kem_case(27, "encapsulation");
}
#[test]
fn acvp_mlkem768_86_valid_decapsulation() {
    kem_case(86, "decapsulation");
}
#[test]
fn acvp_mlkem768_88_implicit_rejection() {
    kem_case(88, "decapsulation");
}

#[test]
fn rfc8032_ed25519_test_1() {
    let t: Value =
        serde_json::from_str(include_str!("../tests/vectors/rfc8032-ed25519-1.json")).unwrap();
    let public = bytes(&t, "public_key");
    let signature = bytes(&t, "signature");
    let message = bytes(&t, "message");
    verify_ed25519_raw(&public, &message, &signature).unwrap();
    let seed: [u8; 32] = bytes(&t, "seed").try_into().unwrap();
    let signer = Ed25519Signer::from_seed(&seed);
    assert_eq!(signer.public_key(), public);
    assert_eq!(signer.key.sign(&message).to_bytes().as_slice(), signature);
    let mut invalid = signature;
    invalid[0] ^= 1;
    assert!(verify_ed25519_raw(&public, &message, &invalid).is_err());
}
