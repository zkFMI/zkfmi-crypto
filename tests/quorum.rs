use zkfmi_crypto::{
    backend::MlDsa65Signer,
    key::{KeyId, KeyPurpose, KeyRecord, ParticipantId},
    quorum::{QuorumMember, QuorumPolicy, SUITE},
    suite::Version,
    traits::Signer,
};

fn committee() -> (QuorumPolicy, Vec<MlDsa65Signer>) {
    let signers = (1_u8..=7)
        .map(|seed| MlDsa65Signer::from_seed(&[seed; 32]))
        .collect::<Vec<_>>();
    let policy = QuorumPolicy {
        version: Version::V1,
        epoch: 7,
        purpose: KeyPurpose::SettlementInstruction,
        context: b"test-network/venue-a/defmi-a".to_vec(),
        classical_binding: [9; 32],
        threshold: 3,
        members: signers
            .iter()
            .enumerate()
            .map(|(index, signer)| QuorumMember {
                node: index as u16 + 1,
                key: KeyRecord {
                    participant_id: ParticipantId::new(format!("operator-{index}")).unwrap(),
                    key_id: KeyId::new(format!("key-{index}")).unwrap(),
                    suite: SUITE,
                    key_version: 2,
                    purpose: KeyPurpose::SettlementInstruction,
                    public_key: signer.public_key(),
                    not_before: 100,
                    not_after: 200,
                    revoked_at: None,
                    rotation_proof: None,
                    dekyx_binding: None,
                },
            })
            .collect(),
    };
    (policy, signers)
}

#[test]
fn distinct_nodes_approve_exactly_one_committee_and_message() {
    let (policy, signers) = committee();
    let message = b"canonical settlement instruction";
    let signatures = [1_u16, 4, 7].map(|node| {
        policy
            .sign_member(node, &signers[node as usize - 1], message, 150)
            .unwrap()
    });
    let approval = policy.assemble(signatures.to_vec(), message, 150).unwrap();
    policy.verify(&approval, message, 150).unwrap();
    assert!(policy
        .verify(&approval, b"another settlement", 150)
        .is_err());
    let mut other = policy.clone();
    other.epoch += 1;
    assert!(other.verify(&approval, message, 150).is_err());
    other = policy.clone();
    other.context.push(1);
    assert!(other.verify(&approval, message, 150).is_err());
    other = policy.clone();
    other.classical_binding[0] ^= 1;
    assert!(other.verify(&approval, message, 150).is_err());
    assert!(policy
        .assemble(signatures[..2].to_vec(), message, 150)
        .is_err());
    assert!(policy
        .assemble(vec![signatures[0].clone(); 3], message, 150)
        .is_err());
}

#[test]
fn rotation_expiry_and_revocation_fail_closed() {
    let (policy, signers) = committee();
    let message = b"key lifecycle";
    let signature = policy.sign_member(1, &signers[0], message, 150).unwrap();
    assert!(policy.verify_member(&signature, message, 99).is_err());
    assert!(policy.verify_member(&signature, message, 200).is_err());
    let mut wrong = signature.clone();
    wrong.key_version += 1;
    assert!(policy.verify_member(&wrong, message, 150).is_err());
    let mut revoked = policy.clone();
    revoked.members[0].key.revoked_at = Some(150);
    assert!(revoked.verify_member(&signature, message, 150).is_err());
    let mut purpose = policy.clone();
    purpose.purpose = KeyPurpose::Governance;
    assert!(purpose.validate().is_err());
}

#[test]
fn one_identity_or_key_cannot_fill_multiple_node_slots() {
    let (policy, _) = committee();
    let mut duplicate = policy.clone();
    duplicate.members[1].key.public_key = duplicate.members[0].key.public_key.clone();
    assert!(duplicate.validate().is_err());
    duplicate = policy.clone();
    duplicate.members[1].key.participant_id = duplicate.members[0].key.participant_id.clone();
    assert!(duplicate.validate().is_err());
    duplicate = policy.clone();
    duplicate.members[1].node = duplicate.members[0].node;
    assert!(duplicate.validate().is_err());
}
