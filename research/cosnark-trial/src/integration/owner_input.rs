//! Explicit owner-local BN254 inputs. This file is a private input adapter,
//! not a conversion of an existing venue's Ed25519-field MPC persistence.
//! The launcher carries only paths and public bindings; it never reads shares.
use super::circuit::Statement;
use crate::field::{decode, encode, F};
use ark_ff::UniformRand;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{DirBuilder, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::Path,
};
use zeroize::{Zeroize, Zeroizing};
use zkfmi_crypto::{
    key::{KeyPurpose, KeyRecord},
    mode::{DeploymentCryptoPolicy, PqcMode},
    sealed::{SealedMessage, SealingPurpose},
    traits::{KemDecapsulator, Signer},
};

const INPUT_PAYLOAD_BYTES: usize = 6 * 32;
const INPUT_DELIVERY_DOMAIN: &[u8] = b"zkfmi-cocode-owner-input-delivery-v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OwnerInputBinding {
    pub policy: DeploymentCryptoPolicy,
    pub book_id: String,
    pub operation_id: String,
    pub sequence: u64,
    pub no_fill: bool,
}

impl OwnerInputBinding {
    pub fn validate(&self) -> Result<(), String> {
        self.policy
            .validate()
            .map_err(|_| "invalid owner input policy")?;
        if self.policy.mode != PqcMode::On
            || self.sequence == 0
            || [&self.book_id, &self.operation_id].iter().any(|value| {
                value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)
            })
        {
            return Err("invalid owner input binding".into());
        }
        Ok(())
    }

    pub fn require_statement(&self, statement: &Statement) -> Result<(), String> {
        self.validate()?;
        let settlement = &statement.settlement;
        if self.policy.deployment_id != settlement.deployment_id
            || self.book_id != settlement.book_id
            || self.operation_id != settlement.operation_id
            || self.sequence != settlement.sequence
            || self.no_fill != settlement.no_fill
        {
            return Err("owner input does not authorize this settlement context".into());
        }
        Ok(())
    }
}

/// Six fresh degree-two Shamir evaluations, encoded as canonical little-endian
/// BN254 field elements: four old balances, matched quantity and matched price.
/// Each owner worker receives only its own evaluation. Secure generation and
/// authenticated delivery from the actual participant are caller obligations;
/// this adapter does not claim that arbitrary files prove venue provenance.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateOwnerInput {
    version: u32,
    party: usize,
    binding: OwnerInputBinding,
    contributions: [String; 6],
}

/// Ciphertext-only transport object. No sender key is accepted from this
/// object: `open_at_worker` requires the registry record selected by the venue.
/// Registry authenticity, account ownership and durable replay consumption
/// remain application obligations; a valid signature alone does not grant them.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealedOwnerInput {
    version: u32,
    party: usize,
    binding: OwnerInputBinding,
    envelope: SealedMessage,
    signature: Vec<u8>,
}

fn delivery_context(
    binding: &OwnerInputBinding,
    party: usize,
    owner: &KeyRecord,
    recipient_public_key: &[u8],
    now: u64,
) -> Result<[u8; 32], String> {
    binding.validate()?;
    owner
        .valid_at(now)
        .map_err(|_| "owner signing key is not valid")?;
    if party >= 7
        || owner.purpose != KeyPurpose::SettlementInstruction
        || owner.suite != binding.policy.signing_suite()
    {
        return Err("owner delivery authority or party mismatch".into());
    }
    // This typed tuple has no maps/floats or caller-supplied canonical bytes.
    // Include the registry identity/version as well as both public keys.
    let canonical = serde_json::to_vec(&(
        binding,
        party,
        &owner.participant_id,
        &owner.key_id,
        owner.key_version,
        &owner.public_key,
        recipient_public_key,
    ))
    .map_err(|_| "owner delivery context encoding")?;
    let mut hash = Sha256::new();
    hash.update(INPUT_DELIVERY_DOMAIN);
    hash.update(canonical);
    Ok(hash.finalize().into())
}

fn delivery_message(context: &[u8; 32], envelope: &SealedMessage) -> Vec<u8> {
    let mut message = INPUT_DELIVERY_DOMAIN.to_vec();
    message.extend_from_slice(context);
    message.extend_from_slice(&envelope.binding_bytes());
    message
}

impl SealedOwnerInput {
    /// Called only in the intended worker. `owner` and `expected` must be
    /// selected by its trusted application state, not copied from the packet.
    /// Authenticate the entire ciphertext before attempting decryption.
    pub fn open_at_worker(
        &self,
        party: usize,
        expected: &OwnerInputBinding,
        owner: &KeyRecord,
        recipient: &dyn KemDecapsulator,
        now: u64,
    ) -> Result<PrivateOwnerInput, String> {
        if self.version != 1 || self.party != party || &self.binding != expected {
            return Err("sealed owner input binding mismatch".into());
        }
        let context = delivery_context(expected, party, owner, &recipient.public_key(), now)?;
        self.envelope
            .validate(SealingPurpose::PrivateMpcInput, INPUT_PAYLOAD_BYTES)
            .map_err(|_| "sealed owner input envelope")?;
        expected
            .policy
            .verify(
                KeyPurpose::SettlementInstruction,
                &owner.public_key,
                &delivery_message(&context, &self.envelope),
                &self.signature,
            )
            .map_err(|_| "sealed owner input authentication failed")?;
        let plaintext = self
            .envelope
            .open(
                recipient,
                SealingPurpose::PrivateMpcInput,
                &context,
                INPUT_PAYLOAD_BYTES,
            )
            .map_err(|_| "sealed owner input decryption failed")?;
        let input = PrivateOwnerInput {
            version: 1,
            party,
            binding: expected.clone(),
            contributions: std::array::from_fn(|column| {
                hex::encode(&plaintext[column * 32..(column + 1) * 32])
            }),
        };
        for value in &input.contributions {
            decode_contribution(value)?;
        }
        Ok(input)
    }
}

impl Drop for PrivateOwnerInput {
    fn drop(&mut self) {
        self.contributions.zeroize();
    }
}

impl PrivateOwnerInput {
    /// Consume one plaintext evaluation inside the participant process and
    /// encrypt it to a registry-pinned worker key using the existing hybrid
    /// KEM/AES-GCM adapter. Only the ciphertext and its hybrid signature leave.
    pub fn seal_for_worker(
        self,
        owner: &KeyRecord,
        signer: &dyn Signer,
        recipient_public_key: &[u8],
        now: u64,
    ) -> Result<SealedOwnerInput, String> {
        let context =
            delivery_context(&self.binding, self.party, owner, recipient_public_key, now)?;
        if self.version != 1
            || signer.public_key() != owner.public_key
            || signer.suite() != owner.suite
        {
            return Err("owner signing key does not match trusted record".into());
        }
        let mut plaintext = Zeroizing::new(Vec::with_capacity(INPUT_PAYLOAD_BYTES));
        for contribution in &self.contributions {
            plaintext.extend_from_slice(&encode(decode_contribution(contribution)?));
        }
        let envelope = SealedMessage::seal(
            recipient_public_key,
            SealingPurpose::PrivateMpcInput,
            &context,
            &plaintext,
        )
        .map_err(|_| "owner input encryption failed")?;
        let signature = self
            .binding
            .policy
            .sign(
                signer,
                KeyPurpose::SettlementInstruction,
                &delivery_message(&context, &envelope),
            )
            .map_err(|_| "owner input signing failed")?;
        Ok(SealedOwnerInput {
            version: 1,
            party: self.party,
            binding: self.binding.clone(),
            envelope,
            signature,
        })
    }

    /// Participant-edge adapter over the existing arkworks polynomial core
    /// (ark-poly 0.5.0, MIT/Apache-2.0). Call only inside the actual owner's
    /// process; all seven outputs must be encrypted to their respective nodes
    /// before crossing that boundary. This is not a coordinator input API.
    pub fn share_at_owner<R: RngCore + CryptoRng>(
        values: &[u64; 6],
        binding: &OwnerInputBinding,
        rng: &mut R,
    ) -> Result<[Self; 7], String> {
        binding.validate()?;
        let mut inputs: [Self; 7] = std::array::from_fn(|party| Self {
            version: 1,
            party,
            binding: binding.clone(),
            contributions: std::array::from_fn(|_| String::new()),
        });
        for (column, value) in values.iter().enumerate() {
            let mut polynomial = DensePolynomial::from_coefficients_vec(vec![
                F::from(*value),
                F::rand(rng),
                F::rand(rng),
            ]);
            for (party, input) in inputs.iter_mut().enumerate() {
                input.contributions[column] =
                    hex::encode(encode(polynomial.evaluate(&F::from((party + 1) as u64))));
            }
            polynomial.coeffs.zeroize();
        }
        Ok(inputs)
    }

    pub fn require_statement(&self, statement: &Statement) -> Result<(), String> {
        self.binding.require_statement(statement)
    }

    pub fn load(path: &Path, party: usize, binding: &OwnerInputBinding) -> Result<Self, String> {
        binding.validate()?;
        let metadata = std::fs::symlink_metadata(path).map_err(|_| "owner input missing")?;
        if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
            return Err("owner input must be a private regular file".into());
        }
        let file = File::open(path).map_err(|_| "owner input open failed")?;
        let opened = file.metadata().map_err(|_| "owner input metadata")?;
        if !opened.is_file() || opened.permissions().mode() & 0o077 != 0 || opened.len() > 16_384 {
            return Err("owner input file bounds".into());
        }
        let mut bytes = Zeroizing::new(Vec::new());
        file.take(16_385)
            .read_to_end(&mut bytes)
            .map_err(|_| "owner input read")?;
        if bytes.len() > 16_384 {
            return Err("owner input file bounds".into());
        }
        let input: Self = serde_json::from_slice(&bytes).map_err(|_| "owner input encoding")?;
        if input.version != 1 || party >= 7 || input.party != party || &input.binding != binding {
            return Err("owner input identity or binding mismatch".into());
        }
        for value in &input.contributions {
            decode_contribution(value)?;
        }
        Ok(input)
    }

    pub fn fields(&self) -> Result<[F; 6], String> {
        let mut fields = [F::from(0u64); 6];
        for (field, encoded) in fields.iter_mut().zip(&self.contributions) {
            *field = decode_contribution(encoded)?;
        }
        Ok(fields)
    }
}

fn decode_contribution(encoded: &str) -> Result<F, String> {
    if encoded.len() != 64 {
        return Err("owner contribution encoding".into());
    }
    let bytes = Zeroizing::new(hex::decode(encoded).map_err(|_| "owner contribution encoding")?);
    if hex::encode(&*bytes) != encoded {
        return Err("owner contribution is not canonical".into());
    }
    decode(
        bytes
            .as_slice()
            .try_into()
            .map_err(|_| "owner contribution width")?,
    )
    .ok_or_else(|| "owner contribution is outside the BN254 field".into())
}

/// Explicit same-host laboratory input preparation, run in a separate owner
/// process. The six balances/quantity/price are never command-line arguments or
/// stdout. These private files stay inside the laboratory trust boundary;
/// production transport must use seal_for_worker and authenticated enrollment.
pub fn prepare_lab_files(
    values_path: &Path,
    binding_path: &Path,
    output: &Path,
) -> Result<(), String> {
    let metadata =
        std::fs::symlink_metadata(values_path).map_err(|_| "lab owner values missing")?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
        return Err("lab owner values must be a private regular file".into());
    }
    let file = File::open(values_path).map_err(|_| "lab owner values open")?;
    let metadata = file.metadata().map_err(|_| "lab owner values metadata")?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 || metadata.len() > 4096 {
        return Err("lab owner values file bounds".into());
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(4097)
        .read_to_end(&mut bytes)
        .map_err(|_| "lab owner values read")?;
    if bytes.len() > 4096 {
        return Err("lab owner values file bounds".into());
    }
    let values = Zeroizing::new(
        serde_json::from_slice::<[u64; 6]>(&bytes).map_err(|_| "lab owner values encoding")?,
    );
    let binding: OwnerInputBinding = serde_json::from_slice(
        &std::fs::read(binding_path).map_err(|_| "lab owner public binding read")?,
    )
    .map_err(|_| "lab owner public binding encoding")?;
    let inputs = PrivateOwnerInput::share_at_owner(&values, &binding, &mut rand::rngs::OsRng)?;
    DirBuilder::new()
        .mode(0o700)
        .create(output)
        .map_err(|_| "fresh private lab output required")?;
    for (party, input) in inputs.iter().enumerate() {
        let encoded = Zeroizing::new(serde_json::to_vec(input).map_err(|_| "lab input encoding")?);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(output.join(format!("party-{party}.json")))
            .map_err(|_| "fresh lab party input")?;
        file.write_all(&encoded)
            .map_err(|_| "lab party input write")?;
        file.sync_all().map_err(|_| "lab party input sync")?;
    }
    File::open(output)
        .and_then(|dir| dir.sync_all())
        .map_err(|_| "lab output directory sync")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{circuit::Statement, SettlementStatement};
    use zkfmi_crypto::hybrid::kem::HybridKemKey;
    use zkfmi_crypto::key::{KeyId, ParticipantId};

    fn binding() -> OwnerInputBinding {
        serde_json::from_value(serde_json::json!({
            "policy":{"version":1,"deployment_id":"owner-input-test","mode":"on"},
            "book_id":"book","operation_id":"fill","sequence":1,"no_fill":false,
        }))
        .unwrap()
    }

    fn signing_record(signer: &dyn Signer) -> KeyRecord {
        KeyRecord {
            participant_id: ParticipantId::new("test-owner").unwrap(),
            key_id: KeyId::new("settlement-key").unwrap(),
            suite: signer.suite(),
            key_version: 1,
            purpose: KeyPurpose::SettlementInstruction,
            public_key: signer.public_key(),
            not_before: 1,
            not_after: 100,
            revoked_at: None,
            rotation_proof: None,
            dekyx_binding: None,
        }
    }

    #[test]
    fn signed_delivery_binds_owner_worker_and_operation_before_decryption() {
        let binding = binding();
        let signer = binding.policy.generate_signer().unwrap();
        let owner = signing_record(signer.as_ref());
        let recipient = HybridKemKey::generate().unwrap();
        let other_recipient = HybridKemKey::generate().unwrap();
        let inputs = PrivateOwnerInput::share_at_owner(
            &[10, 20, 30, 40, 1, 2],
            &binding,
            &mut rand::rngs::OsRng,
        )
        .unwrap();
        let input = inputs.into_iter().next().unwrap();
        let expected = input.contributions.clone();
        let packet = input
            .seal_for_worker(&owner, signer.as_ref(), &recipient.public_key(), 2)
            .unwrap();
        let wire = serde_json::to_vec(&packet).unwrap();
        let mut packet: SealedOwnerInput = serde_json::from_slice(&wire).unwrap();
        let opened = packet
            .open_at_worker(0, &binding, &owner, &recipient, 2)
            .unwrap();
        assert!(opened.contributions == expected);
        assert!(packet
            .open_at_worker(1, &binding, &owner, &recipient, 2)
            .is_err());
        assert!(packet
            .open_at_worker(0, &binding, &owner, &other_recipient, 2)
            .is_err());
        let mut another_operation = binding.clone();
        another_operation.operation_id = "another-operation".into();
        assert!(packet
            .open_at_worker(0, &another_operation, &owner, &recipient, 2)
            .is_err());
        let other_signer = binding.policy.generate_signer().unwrap();
        let other_owner = signing_record(other_signer.as_ref());
        assert!(packet
            .open_at_worker(0, &binding, &other_owner, &recipient, 2)
            .is_err());
        let mut revoked = owner.clone();
        revoked.revoked_at = Some(2);
        assert!(packet
            .open_at_worker(0, &binding, &revoked, &recipient, 2)
            .is_err());
        assert!(packet
            .open_at_worker(0, &binding, &owner, &recipient, 100)
            .is_err());
        packet.signature[0] ^= 1;
        assert!(packet
            .open_at_worker(0, &binding, &owner, &recipient, 2)
            .is_err());
        packet.signature[0] ^= 1;
        packet.envelope.ciphertext[0] ^= 1;
        assert!(packet
            .open_at_worker(0, &binding, &owner, &recipient, 2)
            .is_err());
    }

    #[test]
    fn owner_edge_uses_fresh_degree_two_evaluations_in_the_bn254_field() {
        let values = [1000, 5, 20_000, 7, 3, 101];
        let first =
            PrivateOwnerInput::share_at_owner(&values, &binding(), &mut rand::rngs::OsRng).unwrap();
        let second =
            PrivateOwnerInput::share_at_owner(&values, &binding(), &mut rand::rngs::OsRng).unwrap();
        assert_ne!(first[0].contributions, second[0].contributions);
        let fields: Vec<_> = first.iter().map(|input| input.fields().unwrap()).collect();
        for column in 0..6 {
            assert_eq!(
                fields[0][column] * F::from(3u64) - fields[1][column] * F::from(3u64)
                    + fields[2][column],
                F::from(values[column])
            );
            for x in 4u64..=7 {
                let expected = fields[0][column] * F::from((x - 2) * (x - 3) / 2)
                    - fields[1][column] * F::from((x - 1) * (x - 3))
                    + fields[2][column] * F::from((x - 1) * (x - 2) / 2);
                assert_eq!(fields[(x - 1) as usize][column], expected);
            }
        }
    }

    #[test]
    fn owner_binding_rejects_another_operation_or_policy_without_reading_secrets() {
        let binding = binding();
        let statement = Statement::new(SettlementStatement {
            deployment_id: binding.policy.deployment_id.clone(),
            book_id: binding.book_id.clone(),
            operation_id: binding.operation_id.clone(),
            sequence: 1,
            no_fill: false,
            before_commitment: "a".repeat(128),
            after_commitment: "b".repeat(128),
        });
        binding.require_statement(&statement).unwrap();
        let mut changed = statement.clone();
        changed.settlement.operation_id = "another-operation".into();
        assert!(binding.require_statement(&changed).is_err());
        let mut off = binding.clone();
        off.policy.mode = PqcMode::Off;
        assert!(off.require_statement(&statement).is_err());
        assert!(decode_contribution(&"ff".repeat(32)).is_err());
        assert!(decode_contribution(&"00".repeat(31)).is_err());
    }
}
