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
