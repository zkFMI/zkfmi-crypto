use zkfmi_crypto::{
    commitment::{HashCommitment, HashOpening, MAX_COMMITTED_BYTES},
    mode::{DeploymentCryptoPolicy, PqcMode},
    suite::Version,
};

fn policy() -> DeploymentCryptoPolicy {
    DeploymentCryptoPolicy {
        version: Version::V1,
        deployment_id: "hash-commitment-test".into(),
        mode: PqcMode::On,
    }
}

#[test]
fn opening_binds_value_object_network_and_mode() {
    let p = policy();
    let opening = HashOpening::generate().unwrap();
    let value = 15722u64.to_be_bytes();
    let commitment = HashCommitment::commit(&p, &[1; 32], &value, &opening).unwrap();
    commitment
        .verify_opening(&p, &[1; 32], &value, &opening)
        .unwrap();
    assert!(commitment
        .verify_opening(&p, &[1; 32], &15723u64.to_be_bytes(), &opening)
        .is_err());
    assert!(commitment
        .verify_opening(&p, &[2; 32], &value, &opening)
        .is_err());
    let mut other = p.clone();
    other.deployment_id = "other".into();
    assert!(commitment
        .verify_opening(&other, &[1; 32], &value, &opening)
        .is_err());
    other = p.clone();
    other.mode = PqcMode::Off;
    assert!(commitment
        .verify_opening(&other, &[1; 32], &value, &opening)
        .is_err());
    let fresh = HashOpening::generate().unwrap();
    assert_ne!(
        commitment,
        HashCommitment::commit(&p, &[1; 32], &value, &fresh).unwrap()
    );
    assert!(commitment
        .verify_opening(&p, &[1; 32], &value, &fresh)
        .is_err());
}

#[test]
fn malformed_context_and_payload_are_rejected() {
    let p = policy();
    let opening = HashOpening::generate().unwrap();
    assert!(HashCommitment::commit(&p, &[0; 32], b"value", &opening).is_err());
    assert!(HashCommitment::commit(&p, &[1; 32], b"", &opening).is_err());
    assert!(
        HashCommitment::commit(&p, &[1; 32], &vec![1; MAX_COMMITTED_BYTES + 1], &opening).is_err()
    );
}
