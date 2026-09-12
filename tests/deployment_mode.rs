use zkfmi_crypto::{
    mode::{DeploymentCryptoPolicy, PqcMode, ProofSecurity},
    suite::{SuiteId, Version},
};

fn policy(mode: PqcMode) -> DeploymentCryptoPolicy {
    DeploymentCryptoPolicy {
        version: Version::V1,
        deployment_id: "test-network".into(),
        mode,
    }
}

#[test]
fn switch_is_explicit_and_closed() {
    assert_eq!("on".parse::<PqcMode>().unwrap(), PqcMode::On);
    assert_eq!("off".parse::<PqcMode>().unwrap(), PqcMode::Off);
    for invalid in ["", "auto", "true", "ON", "legacy", " on"] {
        assert!(invalid.parse::<PqcMode>().is_err());
    }
    assert!(serde_json::from_str::<DeploymentCryptoPolicy>(
        r#"{"version":1,"deployment_id":"test-network"}"#
    )
    .is_err());
}

#[test]
fn policy_binds_mode_and_network_and_rejects_both_direction_changes() {
    let off = policy(PqcMode::Off);
    let on = policy(PqcMode::On);
    assert_ne!(off.encode().unwrap(), on.encode().unwrap());
    assert!(off.require_same(&on).is_err());
    assert!(on.require_same(&off).is_err());
    let mut other = on.clone();
    other.deployment_id = "other-network".into();
    assert!(on.require_same(&other).is_err());
    assert_ne!(on.encode().unwrap(), other.encode().unwrap());
    assert!(on.require_same(&on).is_ok());
}

#[test]
fn pq_authorization_cannot_launder_classical_proofs() {
    let on = policy(PqcMode::On);
    assert_eq!(on.signing_suite().id, SuiteId::Ed25519MlDsa65);
    assert!(on.require_proof_security(ProofSecurity::Classical).is_err());
    assert!(on
        .require_proof_security(ProofSecurity::PostQuantum)
        .is_ok());
    assert_eq!(policy(PqcMode::Off).signing_suite().id, SuiteId::Ed25519);
}

#[test]
fn both_modes_execute_real_signatures_and_bind_deployment() {
    use zkfmi_crypto::key::KeyPurpose;
    for mode in [PqcMode::Off, PqcMode::On] {
        let p = policy(mode);
        let signer = p.generate_signer().unwrap();
        let message = b"trade:quantity=7:price=15722";
        let signature = p
            .sign(signer.as_ref(), KeyPurpose::SettlementInstruction, message)
            .unwrap();
        p.verify(
            KeyPurpose::SettlementInstruction,
            &signer.public_key(),
            message,
            &signature,
        )
        .unwrap();
        assert!(p
            .verify(
                KeyPurpose::SettlementInstruction,
                &signer.public_key(),
                b"altered",
                &signature
            )
            .is_err());
        let mut other = p.clone();
        other.deployment_id = "other-network".into();
        assert!(other
            .verify(
                KeyPurpose::SettlementInstruction,
                &signer.public_key(),
                message,
                &signature
            )
            .is_err());
        other.mode = if mode == PqcMode::Off {
            PqcMode::On
        } else {
            PqcMode::Off
        };
        assert!(other
            .sign(signer.as_ref(), KeyPurpose::SettlementInstruction, message)
            .is_err());
        assert!(other
            .verify(
                KeyPurpose::SettlementInstruction,
                &signer.public_key(),
                message,
                &signature
            )
            .is_err());
    }
}
