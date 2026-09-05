//! Publicly known deterministic keys. Never enroll these in a deployed service.
use crate::{
    backend::MlDsa65Signer,
    key::{KeyId, KeyPurpose, KeyRecord, ParticipantId},
    quorum::{QuorumApproval, QuorumMember, QuorumPolicy, SUITE},
    suite::Version,
    traits::Signer,
};

pub fn committee(classical_binding: [u8; 32]) -> QuorumPolicy {
    QuorumPolicy {
        version: Version::V1,
        epoch: 1,
        purpose: KeyPurpose::SettlementInstruction,
        context: b"public-deterministic-test-committee".to_vec(),
        classical_binding,
        threshold: 3,
        members: (1_u8..=7)
            .map(|node| QuorumMember {
                node: u16::from(node),
                key: KeyRecord {
                    participant_id: ParticipantId::new(format!("test-node-{node}")).unwrap(),
                    key_id: KeyId::new(format!("test-pq-key-{node}")).unwrap(),
                    suite: SUITE,
                    key_version: 1,
                    purpose: KeyPurpose::SettlementInstruction,
                    public_key: MlDsa65Signer::from_seed(&[node; 32]).public_key(),
                    not_before: 1,
                    not_after: 4_102_444_801,
                    revoked_at: None,
                    rotation_proof: None,
                    dekyx_binding: None,
                },
            })
            .collect(),
    }
}

pub fn approve(policy: &QuorumPolicy, message: &[u8], now: u64) -> QuorumApproval {
    let signatures = (1_u8..=3)
        .map(|node| {
            let signer = MlDsa65Signer::from_seed(&[node; 32]);
            policy
                .sign_member(u16::from(node), &signer, message, now)
                .unwrap()
        })
        .collect();
    policy.assemble(signatures, message, now).unwrap()
}

/// A publicly reconstructible fixture key. This is deliberately NOT a secure
/// derivation: it exists only to attach reproducible PQ keys to test fixtures.
/// Never use this feature or these keys to enroll a deployed issuer.
pub fn public_fixture_pq_key(classical_public: &[u8; 32]) -> MlDsa65Signer {
    use sha2::{Digest, Sha256};
    let seed = Sha256::new()
        .chain_update(b"ZKFMI:PUBLIC-TEST-ISSUER-KEY:v1")
        .chain_update(classical_public)
        .finalize()
        .into();
    MlDsa65Signer::from_seed(&seed)
}

/// Fixture-only independent test component, matching provider SDK fixtures.
/// Production signers must generate and persist their own independent PQ seed.
pub fn entity_pq_signer(classical_seed: &[u8; 32]) -> MlDsa65Signer {
    let seed = zeroize::Zeroizing::new(classical_seed.map(|byte| byte.wrapping_add(73)));
    MlDsa65Signer::from_seed(&seed)
}

/// Fixture-only hybrid authority. Never use test-support in deployed custody.
pub fn hybrid_signer(
    classical_seed: &[u8; 32],
) -> std::sync::Arc<crate::hybrid::signature::HybridSigner> {
    std::sync::Arc::new(crate::hybrid::signature::HybridSigner::new(
        crate::backend::Ed25519Signer::from_seed(classical_seed),
        entity_pq_signer(classical_seed),
    ))
}

/// Deterministic shape-only bytes for consensus wire vectors. These are not
/// authentic ciphertext and cannot be used as wallet delivery evidence.
pub fn note_envelope() -> crate::sealed::SealedMessage {
    crate::sealed::SealedMessage {
        version: 1,
        suite: crate::suite::Suite::new(crate::suite::SuiteId::X25519MlKem768),
        purpose: crate::sealed::SealingPurpose::NoteOpening,
        kem_ciphertext: vec![41; 1120],
        nonce: [43; 12],
        ciphertext: vec![47; 40],
        tag: [53; 16],
    }
}

/// Public fixtures only, never a production recipient enrollment.
pub fn opening_recipient_public() -> Vec<u8> {
    use crate::traits::KemDecapsulator;
    crate::hybrid::kem::HybridKemKey::from_seed(&[37; 96]).public_key()
}
/// Shape-only threshold ciphertext fixture, not an opening recovery test.
pub fn threshold_opening_envelope() -> crate::sealed::SealedMessage {
    let mut envelope = note_envelope();
    envelope.purpose = crate::sealed::SealingPurpose::ThresholdOpeningShare;
    envelope.ciphertext = vec![59; 64];
    envelope
}

/// Public acceptance-fixture material; provides no recipient secrecy. Real
/// deployments enroll independently generated keys from authenticated custody.
pub fn public_fixture_recipient_key(view: &[u8; 32]) -> crate::hybrid::kem::HybridKemKey {
    use sha2::{Digest, Sha256, Sha512};
    let mut seed = zeroize::Zeroizing::new([0u8; 96]);
    seed[..32].copy_from_slice(
        &Sha256::new()
            .chain_update(b"ZKFMI:TEST:RECIPIENT:X:v1")
            .chain_update(view)
            .finalize(),
    );
    seed[32..].copy_from_slice(
        &Sha512::new()
            .chain_update(b"ZKFMI:TEST:RECIPIENT:PQ:v1")
            .chain_update(view)
            .finalize(),
    );
    crate::hybrid::kem::HybridKemKey::from_seed(&seed)
}
