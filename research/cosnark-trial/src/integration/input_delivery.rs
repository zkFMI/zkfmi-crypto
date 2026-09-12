//! Authenticated input admission at the intended MPC worker. Registration and
//! deployment snapshots must be selected by trusted application state, never
//! by a packet. This does not establish account or order-role authorization.
use super::owner_input::{OwnerInputBinding, PrivateOwnerInput, SealedOwnerInput};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;
use zkfmi_crypto::{
    backend::{Ed25519Signer, MlDsa65Signer},
    hybrid::{kem::HybridKemKey, signature::HybridSigner},
    key::{KeyPurpose, KeyRecord},
    mode::DeploymentCryptoPolicy,
    suite::{Suite, SuiteId},
    traits::{KemDecapsulator, Signer},
};

const MAX_PACKET: usize = 65_536;
const REPLAY_DOMAIN: &[u8] = b"zkfmi-cocode-owner-input-consumption-v1";

/// A pinned public registration/deployment snapshot supplied to the worker.
/// Private KEM material is deliberately absent: the application supplies a
/// resident decapsulation capability separately from this transport object.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedInputReceiver {
    pub version: u32,
    pub policy: DeploymentCryptoPolicy,
    pub party: usize,
    pub owner: KeyRecord,
    pub recipient: KeyRecord,
    pub replay_directory: PathBuf,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedReceiverLaunch {
    pub config_path: PathBuf,
    pub config_sha256: String,
    pub recipient_key_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedInputSender {
    pub version: u32,
    pub policy: DeploymentCryptoPolicy,
    pub owner: KeyRecord,
    /// Ordered party 0..6 recipient registration records, selected by the
    /// deployment, not recipient public keys supplied by a coordinator request.
    pub recipients: [KeyRecord; 7],
}

impl TrustedInputReceiver {
    pub fn validate(
        &self,
        party: usize,
        binding: &OwnerInputBinding,
        now: u64,
    ) -> Result<(), String> {
        binding.validate()?;
        self.owner
            .valid_at(now)
            .map_err(|_| "input owner registration is not valid")?;
        self.recipient
            .valid_at(now)
            .map_err(|_| "input recipient registration is not valid")?;
        if self.version != 1
            || party >= 7
            || self.party != party
            || self.policy != binding.policy
            || self.owner.purpose != KeyPurpose::SettlementInstruction
            || self.owner.suite != binding.policy.signing_suite()
            || self.recipient.purpose != KeyPurpose::Transport
            || self.recipient.suite != Suite::new(SuiteId::X25519MlKem768)
            || !self.replay_directory.is_absolute()
        {
            return Err("input receiver registration or policy mismatch".into());
        }
        Ok(())
    }

    pub fn read_pinned(path: &Path, expected_sha256: &str) -> Result<Self, String> {
        let bytes = read_bounded(path)?;
        if hex::encode(Sha256::digest(&bytes)) != expected_sha256 {
            return Err("input receiver snapshot pin mismatch".into());
        }
        serde_json::from_slice(&bytes).map_err(|_| "input receiver snapshot encoding".into())
    }
}

pub fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "input delivery file missing")?;
    if !metadata.is_file() || metadata.len() > MAX_PACKET as u64 {
        return Err("input delivery file bounds".into());
    }
    let file = File::open(path).map_err(|_| "input delivery file open")?;
    let metadata = file
        .metadata()
        .map_err(|_| "input delivery file metadata")?;
    if !metadata.is_file() || metadata.len() > MAX_PACKET as u64 {
        return Err("input delivery file bounds".into());
    }
    let mut bytes = Vec::new();
    file.take((MAX_PACKET + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "input delivery file read")?;
    if bytes.len() > MAX_PACKET {
        return Err("input delivery file bounds".into());
    }
    Ok(bytes)
}

/// Authenticate/decrypt only at the intended worker, then durably consume the
/// semantic operation before any input can reach native MPC. Re-encryption or
/// signing/recipient key rotation does not reset this operation's budget.
pub fn receive(
    packet: &[u8],
    party: usize,
    binding: &OwnerInputBinding,
    trusted: &TrustedInputReceiver,
    recipient: &dyn KemDecapsulator,
    now: u64,
) -> Result<PrivateOwnerInput, String> {
    trusted.validate(party, binding, now)?;
    if packet.is_empty()
        || packet.len() > MAX_PACKET
        || recipient.suite() != trusted.recipient.suite
        || recipient.public_key() != trusted.recipient.public_key
    {
        return Err("input delivery recipient or packet bounds".into());
    }
    let sealed: SealedOwnerInput =
        serde_json::from_slice(packet).map_err(|_| "sealed input packet encoding")?;
    let input = sealed.open_at_worker(party, binding, &trusted.owner, recipient, now)?;
    // This names the semantic authorization, not a ciphertext/signature/key
    // version. Never persist plaintext values or shares in the replay journal.
    let context = serde_json::to_vec(&(binding, party, &trusted.owner.participant_id))
        .map_err(|_| "input replay context encoding")?;
    let mut hash = Sha256::new();
    hash.update(REPLAY_DOMAIN);
    hash.update(context);
    let id = hex::encode(hash.finalize());
    let metadata = fs::symlink_metadata(&trusted.replay_directory)
        .map_err(|_| "private input replay directory missing")?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err("input replay directory must be private".into());
    }
    let mut marker = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(trusted.replay_directory.join(id))
        .map_err(|_| "input operation already consumed or journal unavailable")?;
    marker
        .write_all(b"consumed-v1\n")
        .map_err(|_| "input consumption write")?;
    marker.sync_all().map_err(|_| "input consumption sync")?;
    File::open(&trusted.replay_directory)
        .and_then(|dir| dir.sync_all())
        .map_err(|_| "input consumption directory sync")?;
    Ok(input)
}

fn private_bytes(path: &Path, limit: usize) -> Result<Zeroizing<Vec<u8>>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "private delivery file missing")?;
    if !metadata.is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > limit as u64
    {
        return Err("private delivery file permissions or bounds".into());
    }
    let file = File::open(path).map_err(|_| "private delivery file open")?;
    let metadata = file
        .metadata()
        .map_err(|_| "private delivery file metadata")?;
    if !metadata.is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > limit as u64
    {
        return Err("private delivery file permissions or bounds".into());
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "private delivery file read")?;
    if bytes.len() > limit {
        return Err("private delivery file bounds".into());
    }
    Ok(bytes)
}

/// Laboratory-only raw seed file adapter. Production workers should call
/// receive with their resident/keystore-backed decapsulation capability.
pub fn load_lab_recipient(path: &Path) -> Result<HybridKemKey, String> {
    let bytes = private_bytes(path, 96)?;
    let seed: &[u8; 96] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| "lab recipient key length")?;
    Ok(HybridKemKey::from_seed(seed))
}

pub fn clock_now() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|time| time.as_secs())
        .map_err(|_| "input admission clock".into())
}

/// Run inside a distinct laboratory key-owner process. Only public bytes are
/// returned; the newly generated seed stays in a private create-new file.
pub fn generate_lab_key(kind: &str, private: &Path, public: &Path) -> Result<(), String> {
    let length = match kind {
        "sender" => 64,
        "recipient" => 96,
        _ => return Err("lab input key kind".into()),
    };
    let mut seed = Zeroizing::new(vec![0u8; length]);
    rand::rngs::OsRng
        .try_fill_bytes(&mut seed)
        .map_err(|_| "input key randomness")?;
    let public_key = if kind == "sender" {
        let classical: &[u8; 32] = seed[..32].try_into().map_err(|_| "input signing seed")?;
        let pq: &[u8; 32] = seed[32..].try_into().map_err(|_| "input signing seed")?;
        HybridSigner::new(
            Ed25519Signer::from_seed(classical),
            MlDsa65Signer::from_seed(pq),
        )
        .public_key()
    } else {
        HybridKemKey::from_seed(seed.as_slice().try_into().map_err(|_| "input KEM seed")?)
            .public_key()
    };
    let mut secret_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(private)
        .map_err(|_| "fresh private input key required")?;
    secret_file
        .write_all(&seed)
        .map_err(|_| "private input key write")?;
    secret_file
        .sync_all()
        .map_err(|_| "private input key sync")?;
    let mut public_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o644)
        .open(public)
        .map_err(|_| "fresh public input key required")?;
    serde_json::to_writer(&mut public_file, &public_key).map_err(|_| "public input key write")?;
    public_file
        .sync_all()
        .map_err(|_| "public input key sync")?;
    Ok(())
}

/// Only the owner process reads the six clear values and its signing seed.
/// Outputs are signed ciphertext packets; no plaintext shares are written.
pub fn seal_lab_inputs(
    values_path: &Path,
    binding_path: &Path,
    sender_config: &Path,
    expected_config_sha256: &str,
    sender_key: &Path,
    output: &Path,
) -> Result<(), String> {
    use std::os::unix::fs::DirBuilderExt;
    let config_bytes = read_bounded(sender_config)?;
    if hex::encode(Sha256::digest(&config_bytes)) != expected_config_sha256 {
        return Err("input sender snapshot pin mismatch".into());
    }
    let config: TrustedInputSender =
        serde_json::from_slice(&config_bytes).map_err(|_| "input sender snapshot encoding")?;
    let binding: OwnerInputBinding = serde_json::from_slice(&read_bounded(binding_path)?)
        .map_err(|_| "input sender binding encoding")?;
    binding.validate()?;
    let now = clock_now()?;
    config
        .owner
        .valid_at(now)
        .map_err(|_| "input sender registration is not valid")?;
    if config.version != 1
        || config.policy != binding.policy
        || config.owner.purpose != KeyPurpose::SettlementInstruction
        || config.owner.suite != binding.policy.signing_suite()
    {
        return Err("input sender policy or purpose mismatch".into());
    }
    for (party, recipient) in config.recipients.iter().enumerate() {
        recipient
            .valid_at(now)
            .map_err(|_| "input recipient registration is not valid")?;
        if recipient.purpose != KeyPurpose::Transport
            || recipient.suite != Suite::new(SuiteId::X25519MlKem768)
            || config.recipients[..party].iter().any(|prior| {
                prior.public_key == recipient.public_key
                    || prior.participant_id == recipient.participant_id
            })
        {
            return Err("input recipients must be distinct registered KEM owners".into());
        }
    }
    let seed = private_bytes(sender_key, 64)?;
    if seed.len() != 64 {
        return Err("lab sender key length".into());
    }
    let signer = HybridSigner::new(
        Ed25519Signer::from_seed(seed[..32].try_into().map_err(|_| "lab sender seed")?),
        MlDsa65Signer::from_seed(seed[32..].try_into().map_err(|_| "lab sender seed")?),
    );
    if signer.public_key() != config.owner.public_key {
        return Err("input sender key does not match registration".into());
    }
    let bytes = private_bytes(values_path, 4096)?;
    let values = Zeroizing::new(
        serde_json::from_slice::<[u64; 6]>(&bytes).map_err(|_| "lab input values encoding")?,
    );
    let inputs = PrivateOwnerInput::share_at_owner(&values, &binding, &mut rand::rngs::OsRng)?;
    fs::DirBuilder::new()
        .mode(0o700)
        .create(output)
        .map_err(|_| "fresh sealed input output required")?;
    for (party, input) in inputs.into_iter().enumerate() {
        let packet = input.seal_for_worker(
            &config.owner,
            &signer,
            &config.recipients[party].public_key,
            now,
        )?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(output.join(format!("party-{party}.json")))
            .map_err(|_| "fresh sealed packet required")?;
        serde_json::to_writer(&mut file, &packet).map_err(|_| "sealed packet write")?;
        file.sync_all().map_err(|_| "sealed packet sync")?;
    }
    File::open(output)
        .and_then(|dir| dir.sync_all())
        .map_err(|_| "sealed output directory sync")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::DirBuilderExt;
    use zkfmi_crypto::{
        key::{KeyId, ParticipantId},
        mode::PqcMode,
    };

    struct Delivery {
        binding: OwnerInputBinding,
        owner: Box<dyn Signer>,
        recipient: HybridKemKey,
        trusted: TrustedInputReceiver,
    }

    impl Delivery {
        fn new() -> Self {
            let binding: OwnerInputBinding = serde_json::from_value(serde_json::json!({
                "policy":{"version":1,"deployment_id":"delivery-test","mode":"on"},
                "book_id":"book","operation_id":"fill","sequence":1,"no_fill":false,
            }))
            .unwrap();
            let owner = binding.policy.generate_signer().unwrap();
            let recipient = HybridKemKey::generate().unwrap();
            let record = |id: &str, suite, purpose, public_key| KeyRecord {
                participant_id: ParticipantId::new(id).unwrap(),
                key_id: KeyId::new("input-key").unwrap(),
                suite,
                key_version: 1,
                purpose,
                public_key,
                not_before: 1,
                not_after: 100,
                revoked_at: None,
                rotation_proof: None,
                dekyx_binding: None,
            };
            let directory = std::env::temp_dir().join(format!(
                "cocode-input-delivery-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            fs::DirBuilder::new()
                .mode(0o700)
                .create(&directory)
                .unwrap();
            let trusted = TrustedInputReceiver {
                version: 1,
                policy: binding.policy.clone(),
                party: 0,
                owner: record(
                    "owner",
                    owner.suite(),
                    KeyPurpose::SettlementInstruction,
                    owner.public_key(),
                ),
                recipient: record(
                    "recipient",
                    recipient.suite(),
                    KeyPurpose::Transport,
                    recipient.public_key(),
                ),
                replay_directory: directory,
            };
            Self {
                binding,
                owner,
                recipient,
                trusted,
            }
        }

        fn packet(&self) -> Vec<u8> {
            self.packet_and_fields().0
        }

        fn packet_and_fields(&self) -> (Vec<u8>, [crate::field::F; 6]) {
            let input = PrivateOwnerInput::share_at_owner(
                &[10, 20, 30, 40, 1, 2],
                &self.binding,
                &mut rand::rngs::OsRng,
            )
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
            let fields = input.fields().unwrap();
            let packet = serde_json::to_vec(
                &input
                    .seal_for_worker(
                        &self.trusted.owner,
                        self.owner.as_ref(),
                        &self.recipient.public_key(),
                        2,
                    )
                    .unwrap(),
            )
            .unwrap();
            (packet, fields)
        }

        fn count(&self) -> usize {
            fs::read_dir(&self.trusted.replay_directory)
                .unwrap()
                .count()
        }
    }

    impl Drop for Delivery {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.trusted.replay_directory).unwrap();
        }
    }

    #[test]
    fn admission_rejects_invalid_packets_without_consuming_authorization() {
        let delivery = Delivery::new();
        let (wire, expected) = delivery.packet_and_fields();
        let check = |packet: &[u8],
                     party,
                     binding: &OwnerInputBinding,
                     trusted: &TrustedInputReceiver,
                     recipient: &dyn KemDecapsulator,
                     now| {
            assert!(receive(packet, party, binding, trusted, recipient, now).is_err());
            assert_eq!(delivery.count(), 0);
        };
        check(
            &wire,
            1,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2,
        );
        let other_key = HybridKemKey::generate().unwrap();
        check(
            &wire,
            0,
            &delivery.binding,
            &delivery.trusted,
            &other_key,
            2,
        );
        let mut changed = delivery.binding.clone();
        changed.operation_id = "different-operation".into();
        check(
            &wire,
            0,
            &changed,
            &delivery.trusted,
            &delivery.recipient,
            2,
        );
        changed = delivery.binding.clone();
        changed.policy.mode = PqcMode::Off;
        check(
            &wire,
            0,
            &changed,
            &delivery.trusted,
            &delivery.recipient,
            2,
        );
        let mut registration = delivery.trusted.clone();
        registration.recipient.revoked_at = Some(2);
        check(
            &wire,
            0,
            &delivery.binding,
            &registration,
            &delivery.recipient,
            2,
        );
        registration = delivery.trusted.clone();
        registration.owner.purpose = KeyPurpose::Transport;
        check(
            &wire,
            0,
            &delivery.binding,
            &registration,
            &delivery.recipient,
            2,
        );
        check(
            &wire,
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            100,
        );
        let mut packet: serde_json::Value = serde_json::from_slice(&wire).unwrap();
        packet["signature"][0] = (packet["signature"][0].as_u64().unwrap() ^ 1).into();
        check(
            &serde_json::to_vec(&packet).unwrap(),
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2,
        );
        packet = serde_json::from_slice(&wire).unwrap();
        packet["envelope"]["ciphertext"][0] =
            (packet["envelope"]["ciphertext"][0].as_u64().unwrap() ^ 1).into();
        check(
            &serde_json::to_vec(&packet).unwrap(),
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2,
        );
        check(
            b"{}",
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2,
        );
        let opened = receive(
            &wire,
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2,
        )
        .unwrap();
        assert!(opened.fields().unwrap() == expected);
        assert_eq!(delivery.count(), 1);
    }

    #[test]
    fn semantic_consumption_survives_reencryption_registration_reload_and_key_rotation() {
        let mut delivery = Delivery::new();
        let first = delivery.packet();
        let second = delivery.packet();
        assert!(first != second);
        receive(
            &first,
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2,
        )
        .unwrap();
        // A new receiver object opens the already persisted journal; no in-memory
        // replay set is shared with the first admission.
        let reloaded: TrustedInputReceiver =
            serde_json::from_slice(&serde_json::to_vec(&delivery.trusted).unwrap()).unwrap();
        assert!(receive(
            &second,
            0,
            &delivery.binding,
            &reloaded,
            &delivery.recipient,
            2
        )
        .is_err());
        delivery.owner = delivery.binding.policy.generate_signer().unwrap();
        delivery.trusted.owner.public_key = delivery.owner.public_key();
        delivery.trusted.owner.key_version += 1;
        delivery.recipient = HybridKemKey::generate().unwrap();
        delivery.trusted.recipient.public_key = delivery.recipient.public_key();
        delivery.trusted.recipient.key_version += 1;
        let rotated = delivery.packet();
        assert!(receive(
            &rotated,
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2
        )
        .is_err());
        assert_eq!(delivery.count(), 1);
    }

    #[test]
    fn registration_pin_and_private_journal_permissions_fail_closed() {
        let delivery = Delivery::new();
        let path = delivery.trusted.replay_directory.join("registration.json");
        let bytes = serde_json::to_vec(&delivery.trusted).unwrap();
        fs::write(&path, &bytes).unwrap();
        let pin = hex::encode(Sha256::digest(&bytes));
        assert!(TrustedInputReceiver::read_pinned(&path, &pin).is_ok());
        assert!(TrustedInputReceiver::read_pinned(&path, &"00".repeat(32)).is_err());
        fs::remove_file(&path).unwrap();
        fs::set_permissions(
            &delivery.trusted.replay_directory,
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert!(receive(
            &delivery.packet(),
            0,
            &delivery.binding,
            &delivery.trusted,
            &delivery.recipient,
            2
        )
        .is_err());
        assert_eq!(delivery.count(), 0);
    }
}
