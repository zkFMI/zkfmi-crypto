//! Public key metadata. Enrollment, snapshot authenticity and the clock are caller trust boundaries.
use crate::{
    canonical::{put_bytes, SigningPreimage},
    error::{CryptoError, Result},
    suite::{CryptoPurpose, Suite, Version},
    traits::{CryptoProvider, Signer},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! identifier {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                Self::try_from(value.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = CryptoError;
            fn try_from(value: String) -> Result<Self> {
                if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
                    return Err(CryptoError::InvalidEncoding);
                }
                Ok(Self(value))
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

identifier!(ParticipantId);
identifier!(KeyId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyPurpose {
    Quote,
    SettlementInstruction,
    KeyRotation,
    Governance,
    Transport,
    Attestation,
    Order,
    AuditCheckpoint,
}

impl KeyPurpose {
    pub const fn code(self) -> u16 {
        match self {
            Self::Quote => 1,
            Self::SettlementInstruction => 2,
            Self::KeyRotation => 3,
            Self::Governance => 4,
            Self::Transport => 5,
            Self::Attestation => 6,
            Self::Order => 7,
            Self::AuditCheckpoint => 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotationProof {
    pub version: Version,
    pub old_key_id: KeyId,
    pub old_key_version: u32,
    pub old_to_new: Vec<u8>,
    pub new_to_old: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyRecord {
    pub participant_id: ParticipantId,
    pub key_id: KeyId,
    pub suite: Suite,
    /// Monotonic key generation, distinct from the closed suite/wire version.
    pub key_version: u32,
    pub purpose: KeyPurpose,
    pub public_key: Vec<u8>,
    pub not_before: u64,
    pub not_after: u64,
    pub revoked_at: Option<u64>,
    pub rotation_proof: Option<RotationProof>,
    /// Opaque reference. This crate does not verify a DeKYX identity claim.
    pub dekyx_binding: Option<String>,
}

impl KeyRecord {
    pub fn validate(&self) -> Result<()> {
        if self.key_version == 0 || self.not_before >= self.not_after {
            return Err(CryptoError::InvalidTime);
        }
        let meta = self.suite.id.metadata();
        if meta.public_key_bytes != Some(self.public_key.len()) {
            return Err(CryptoError::InvalidKey);
        }
        match (meta.purpose, self.purpose) {
            (CryptoPurpose::Signature, _) => (),
            (CryptoPurpose::Transport | CryptoPurpose::Kem, KeyPurpose::Transport) => (),
            _ => return Err(CryptoError::InvalidPurpose),
        }
        Ok(())
    }

    /// Valid on [not_before, not_after), and only while now < revoked_at, if present.
    pub fn valid_at(&self, now: u64) -> Result<()> {
        self.validate()?;
        if self.revoked_at.is_some_and(|at| now >= at) {
            return Err(CryptoError::Revoked);
        }
        if now < self.not_before {
            return Err(CryptoError::NotYetValid);
        }
        if now >= self.not_after {
            return Err(CryptoError::Expired);
        }
        Ok(())
    }

    fn rotation_binding(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut out = Vec::new();
        put_bytes(&mut out, self.participant_id.as_str().as_bytes())?;
        put_bytes(&mut out, self.key_id.as_str().as_bytes())?;
        out.extend_from_slice(&self.suite.encode());
        out.extend_from_slice(&self.key_version.to_be_bytes());
        out.extend_from_slice(&self.purpose.code().to_be_bytes());
        put_bytes(&mut out, &self.public_key)?;
        out.extend_from_slice(&self.not_before.to_be_bytes());
        out.extend_from_slice(&self.not_after.to_be_bytes());
        match &self.dekyx_binding {
            None => out.push(0),
            Some(binding) => {
                out.push(1);
                put_bytes(&mut out, binding.as_bytes())?;
            }
        }
        Ok(out)
    }
}

fn rotation_messages(old: &KeyRecord, new: &KeyRecord) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut authorize = b"ZKFMI:KEY-ROTATION:AUTHORIZE:v1".to_vec();
    let mut acknowledge = b"ZKFMI:KEY-ROTATION:ACKNOWLEDGE:v1".to_vec();
    for out in [&mut authorize, &mut acknowledge] {
        put_bytes(out, &old.rotation_binding()?)?;
        put_bytes(out, &new.rotation_binding()?)?;
    }
    Ok((authorize, acknowledge))
}

impl RotationProof {
    /// Both directions bind the complete key transition, including the old ID and new metadata.
    pub fn create(
        old: &KeyRecord,
        new: &KeyRecord,
        old_signer: &dyn Signer,
        new_signer: &dyn Signer,
    ) -> Result<Self> {
        if old_signer.suite() != old.suite
            || new_signer.suite() != new.suite
            || old_signer.public_key() != old.public_key
            || new_signer.public_key() != new.public_key
        {
            return Err(CryptoError::InvalidKey);
        }
        let (authorize, acknowledge) = rotation_messages(old, new)?;
        Ok(Self {
            version: Version::V1,
            old_key_id: old.key_id.clone(),
            old_key_version: old.key_version,
            old_to_new: old_signer.sign(KeyPurpose::KeyRotation, &authorize)?,
            new_to_old: new_signer.sign(KeyPurpose::KeyRotation, &acknowledge)?,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrySnapshot {
    pub version: Version,
    pub records: Vec<KeyRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct KeyRegistry {
    records: BTreeMap<KeyId, KeyRecord>,
}

impl KeyRegistry {
    /// Administrative trust anchor enrollment, not a self-service registration endpoint.
    pub fn insert_initial(&mut self, record: KeyRecord) -> Result<()> {
        record.validate()?;
        if record.key_version != 1 || record.rotation_proof.is_some() || record.revoked_at.is_some()
        {
            return Err(CryptoError::InvalidRotation);
        }
        if self.records.contains_key(&record.key_id) {
            return Err(CryptoError::DuplicateKey);
        }
        self.records.insert(record.key_id.clone(), record);
        Ok(())
    }

    pub fn get(&self, key_id: &KeyId) -> Result<&KeyRecord> {
        self.records.get(key_id).ok_or(CryptoError::NotFound)
    }

    pub fn lookup(
        &self,
        key_id: &KeyId,
        key_version: u32,
        purpose: KeyPurpose,
        now: u64,
    ) -> Result<&KeyRecord> {
        let record = self.get(key_id)?;
        if record.key_version != key_version {
            return Err(CryptoError::UnknownVersion);
        }
        if record.purpose != purpose {
            return Err(CryptoError::InvalidPurpose);
        }
        record.valid_at(now)?;
        Ok(record)
    }

    /// Administrative revocation can only move the effective revocation time earlier.
    pub fn revoke(&mut self, key_id: &KeyId, at: u64) -> Result<()> {
        let record = self.records.get_mut(key_id).ok_or(CryptoError::NotFound)?;
        record.revoked_at = Some(record.revoked_at.map_or(at, |old| old.min(at)));
        Ok(())
    }

    pub fn verify_preimage(
        &self,
        key_id: &KeyId,
        key_version: u32,
        now: u64,
        preimage: &SigningPreimage,
        signature: &[u8],
        provider: &dyn CryptoProvider,
    ) -> Result<()> {
        preimage.validate_at(now)?;
        let key = self.lookup(key_id, key_version, preimage.tx_kind, now)?;
        if key.suite != preimage.suite {
            return Err(CryptoError::UnsupportedSuite);
        }
        provider.verifier(key.suite)?.verify(
            key.purpose,
            &key.public_key,
            &preimage.encode()?,
            signature,
        )
    }

    /// Atomically verify both signatures, insert the successor and retire the old key at now.
    pub fn rotate(
        &mut self,
        new: KeyRecord,
        now: u64,
        provider: &dyn CryptoProvider,
    ) -> Result<()> {
        new.valid_at(now)?;
        if new.revoked_at.is_some() {
            return Err(CryptoError::InvalidRotation);
        }
        if self.records.contains_key(&new.key_id) {
            return Err(CryptoError::DuplicateKey);
        }
        let proof = new
            .rotation_proof
            .as_ref()
            .ok_or(CryptoError::InvalidRotation)?;
        let old = self.get(&proof.old_key_id)?;
        old.valid_at(now)?;
        if old.key_version != proof.old_key_version
            || old.key_version.checked_add(1) != Some(new.key_version)
            || old.participant_id != new.participant_id
            || old.purpose != new.purpose
            || old.public_key == new.public_key
        {
            return Err(CryptoError::InvalidRotation);
        }
        // No migration back to a classical-only signature suite through the rotation API.
        if old.suite.id.metadata().post_quantum && !new.suite.id.metadata().post_quantum {
            return Err(CryptoError::InvalidRotation);
        }
        let (authorize, acknowledge) = rotation_messages(old, &new)?;
        provider.verifier(old.suite)?.verify(
            KeyPurpose::KeyRotation,
            &old.public_key,
            &authorize,
            &proof.old_to_new,
        )?;
        provider.verifier(new.suite)?.verify(
            KeyPurpose::KeyRotation,
            &new.public_key,
            &acknowledge,
            &proof.new_to_old,
        )?;
        let old_id = old.key_id.clone();
        self.revoke(&old_id, now)?;
        self.records.insert(new.key_id.clone(), new);
        Ok(())
    }

    pub fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            version: Version::V1,
            records: self.records.values().cloned().collect(),
        }
    }
}
