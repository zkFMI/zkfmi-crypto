//! PQ quorum adapter. The caller authenticates enrollment and the expected
//! committee; this module never trusts a policy supplied inside an approval.

use crate::{
    backend::MlDsa65Verifier,
    canonical::put_bytes,
    error::{CryptoError, Result},
    key::{KeyId, KeyPurpose, KeyRecord},
    suite::{Suite, SuiteId, Version},
    traits::{Signer, Verifier},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const SUITE: Suite = Suite::new(SuiteId::MlDsa65);
const APPROVAL_MAGIC: &[u8; 8] = b"ZKPQQRM1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuorumMember {
    pub node: u16,
    pub key: KeyRecord,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuorumPolicy {
    pub version: Version,
    pub epoch: u64,
    pub purpose: KeyPurpose,
    /// Canonical protocol/deployment context, supplied by the enrollment owner.
    pub context: Vec<u8>,
    /// SHA-256 of the canonical classical committee package. Its verifier
    /// remains mandatory at the service boundary (AND composition).
    pub classical_binding: [u8; 32],
    pub threshold: u16,
    /// Strictly increasing one-based node IDs; no implicit reordering.
    pub members: Vec<QuorumMember>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberApproval {
    pub node: u16,
    pub key_id: KeyId,
    pub key_version: u32,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuorumApproval {
    pub version: Version,
    pub suite: Suite,
    pub epoch: u64,
    pub committee: [u8; 32],
    pub signatures: Vec<MemberApproval>,
}

impl QuorumApproval {
    fn validate_encoding(&self) -> Result<()> {
        if self.suite != SUITE
            || self.epoch == 0
            || self.committee == [0; 32]
            || self.signatures.is_empty()
            || self.signatures.len() > 64
        {
            return Err(CryptoError::InvalidEncoding);
        }
        let mut previous = 0;
        for member in &self.signatures {
            if member.node <= previous
                || member.key_version == 0
                || member.signature.len() != crate::suite::ML_DSA_65_SIG_BYTES
            {
                return Err(CryptoError::InvalidEncoding);
            }
            previous = member.node;
        }
        Ok(())
    }

    /// Fixed big-endian scalars, bounded length-prefixed key IDs, and fixed
    /// ML-DSA-65 signature lengths. No recursive or negotiable container.
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate_encoding()?;
        let mut out = APPROVAL_MAGIC.to_vec();
        out.extend_from_slice(&u16::from(self.version).to_be_bytes());
        out.extend_from_slice(&self.suite.encode());
        out.extend_from_slice(&self.epoch.to_be_bytes());
        out.extend_from_slice(&self.committee);
        out.extend_from_slice(&(self.signatures.len() as u16).to_be_bytes());
        for member in &self.signatures {
            out.extend_from_slice(&member.node.to_be_bytes());
            put_bytes(&mut out, member.key_id.as_str().as_bytes())?;
            out.extend_from_slice(&member.key_version.to_be_bytes());
            out.extend_from_slice(&member.signature);
        }
        Ok(out)
    }

    pub fn decode(raw: &[u8]) -> Result<Self> {
        struct Reader<'a>(&'a [u8]);
        impl<'a> Reader<'a> {
            fn take(&mut self, count: usize) -> Result<&'a [u8]> {
                let (head, tail) = self
                    .0
                    .split_at_checked(count)
                    .ok_or(CryptoError::InvalidEncoding)?;
                self.0 = tail;
                Ok(head)
            }
            fn u16(&mut self) -> Result<u16> {
                Ok(u16::from_be_bytes(
                    self.take(2)?
                        .try_into()
                        .map_err(|_| CryptoError::InvalidEncoding)?,
                ))
            }
            fn u32(&mut self) -> Result<u32> {
                Ok(u32::from_be_bytes(
                    self.take(4)?
                        .try_into()
                        .map_err(|_| CryptoError::InvalidEncoding)?,
                ))
            }
        }
        if raw.len() > 1 << 20 {
            return Err(CryptoError::InvalidEncoding);
        }
        let mut reader = Reader(raw);
        if reader.take(8)? != APPROVAL_MAGIC {
            return Err(CryptoError::InvalidEncoding);
        }
        let version = Version::try_from(reader.u16()?)?;
        if reader.take(4)? != SUITE.encode() {
            return Err(CryptoError::UnsupportedSuite);
        }
        let epoch = u64::from_be_bytes(
            reader
                .take(8)?
                .try_into()
                .map_err(|_| CryptoError::InvalidEncoding)?,
        );
        let committee = reader
            .take(32)?
            .try_into()
            .map_err(|_| CryptoError::InvalidEncoding)?;
        let count = reader.u16()?;
        if count == 0 || count > 64 {
            return Err(CryptoError::InvalidEncoding);
        }
        let mut signatures = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            let node = reader.u16()?;
            let length = reader.u32()? as usize;
            if length == 0 || length > 256 {
                return Err(CryptoError::InvalidEncoding);
            }
            let key_id = KeyId::new(
                std::str::from_utf8(reader.take(length)?)
                    .map_err(|_| CryptoError::InvalidEncoding)?,
            )?;
            let key_version = reader.u32()?;
            let signature = reader.take(crate::suite::ML_DSA_65_SIG_BYTES)?.to_vec();
            signatures.push(MemberApproval {
                node,
                key_id,
                key_version,
                signature,
            });
        }
        if !reader.0.is_empty() {
            return Err(CryptoError::InvalidEncoding);
        }
        let result = Self {
            version,
            suite: SUITE,
            epoch,
            committee,
            signatures,
        };
        result.validate_encoding()?;
        Ok(result)
    }
}

impl QuorumPolicy {
    pub fn validate(&self) -> Result<()> {
        if self.epoch == 0
            || self.context.is_empty()
            || self.context.len() > 1024
            || self.classical_binding == [0; 32]
            || self.members.len() > 64
            || self.threshold == 0
            || usize::from(self.threshold) > self.members.len()
        {
            return Err(CryptoError::InvalidEncoding);
        }
        let mut previous = 0;
        let mut participants = BTreeSet::new();
        let mut keys = BTreeSet::new();
        let mut public_keys = BTreeSet::new();
        for member in &self.members {
            member.key.validate()?;
            if member.node <= previous
                || member.key.suite != SUITE
                || member.key.purpose != self.purpose
                || !participants.insert(&member.key.participant_id)
                || !keys.insert(&member.key.key_id)
                || !public_keys.insert(&member.key.public_key)
            {
                return Err(CryptoError::InvalidKey);
            }
            previous = member.node;
        }
        Ok(())
    }

    /// Bind immutable enrollment data, validity and revocation metadata. A
    /// rotation/revocation changes this digest and requires an updated trusted
    /// policy. Opaque DeKYX references are not a verified credential here.
    pub fn digest(&self) -> Result<[u8; 32]> {
        self.validate()?;
        let mut bytes = b"ZKFMI:PQ-QUORUM-POLICY:v1".to_vec();
        bytes.extend_from_slice(&u16::from(self.version).to_be_bytes());
        bytes.extend_from_slice(&self.epoch.to_be_bytes());
        bytes.extend_from_slice(&self.purpose.code().to_be_bytes());
        put_bytes(&mut bytes, &self.context)?;
        bytes.extend_from_slice(&self.classical_binding);
        bytes.extend_from_slice(&self.threshold.to_be_bytes());
        bytes.extend_from_slice(&(self.members.len() as u16).to_be_bytes());
        for member in &self.members {
            bytes.extend_from_slice(&member.node.to_be_bytes());
            let key = &member.key;
            put_bytes(&mut bytes, key.participant_id.as_str().as_bytes())?;
            put_bytes(&mut bytes, key.key_id.as_str().as_bytes())?;
            bytes.extend_from_slice(&key.suite.encode());
            bytes.extend_from_slice(&key.key_version.to_be_bytes());
            bytes.extend_from_slice(&key.purpose.code().to_be_bytes());
            put_bytes(&mut bytes, &key.public_key)?;
            bytes.extend_from_slice(&key.not_before.to_be_bytes());
            bytes.extend_from_slice(&key.not_after.to_be_bytes());
            match key.revoked_at {
                Some(at) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&at.to_be_bytes());
                }
                None => bytes.push(0),
            }
            match &key.dekyx_binding {
                Some(binding) => {
                    bytes.push(1);
                    put_bytes(&mut bytes, binding.as_bytes())?;
                }
                None => bytes.push(0),
            }
        }
        Ok(Sha256::digest(bytes).into())
    }

    fn member(&self, node: u16) -> Result<&QuorumMember> {
        self.members
            .iter()
            .find(|member| member.node == node)
            .ok_or(CryptoError::NotFound)
    }

    fn preimage(&self, member: &QuorumMember, message: &[u8]) -> Result<Vec<u8>> {
        if message.is_empty() || message.len() > 1 << 20 {
            return Err(CryptoError::InvalidEncoding);
        }
        let mut out = b"ZKFMI:PQ-QUORUM-APPROVAL:v1".to_vec();
        out.extend_from_slice(&SUITE.encode());
        out.extend_from_slice(&self.digest()?);
        out.extend_from_slice(&member.node.to_be_bytes());
        put_bytes(&mut out, member.key.key_id.as_str().as_bytes())?;
        out.extend_from_slice(&member.key.key_version.to_be_bytes());
        put_bytes(&mut out, message)?;
        Ok(out)
    }

    pub fn sign_member(
        &self,
        node: u16,
        signer: &dyn Signer,
        message: &[u8],
        now: u64,
    ) -> Result<MemberApproval> {
        self.validate()?;
        let member = self.member(node)?;
        member.key.valid_at(now)?;
        if signer.suite() != SUITE || signer.public_key() != member.key.public_key {
            return Err(CryptoError::InvalidKey);
        }
        Ok(MemberApproval {
            node,
            key_id: member.key.key_id.clone(),
            key_version: member.key.key_version,
            signature: signer.sign(self.purpose, &self.preimage(member, message)?)?,
        })
    }

    pub fn verify_member(&self, approval: &MemberApproval, message: &[u8], now: u64) -> Result<()> {
        self.member(approval.node)?.key.valid_at(now)?;
        self.verify_member_signature(approval, message)
    }

    fn verify_member_signature(&self, approval: &MemberApproval, message: &[u8]) -> Result<()> {
        self.validate()?;
        let member = self.member(approval.node)?;
        if member.key.key_id != approval.key_id || member.key.key_version != approval.key_version {
            return Err(CryptoError::InvalidKey);
        }
        MlDsa65Verifier.verify(
            self.purpose,
            &member.key.public_key,
            &self.preimage(member, message)?,
            &approval.signature,
        )
    }

    pub fn assemble(
        &self,
        signatures: Vec<MemberApproval>,
        message: &[u8],
        now: u64,
    ) -> Result<QuorumApproval> {
        let approval = QuorumApproval {
            version: Version::V1,
            suite: SUITE,
            epoch: self.epoch,
            committee: self.digest()?,
            signatures,
        };
        self.verify(&approval, message, now)?;
        Ok(approval)
    }

    pub fn verify(&self, approval: &QuorumApproval, message: &[u8], now: u64) -> Result<()> {
        self.verify_archived_signatures(approval, message)?;
        for signature in &approval.signatures {
            self.member(signature.node)?.key.valid_at(now)?;
        }
        Ok(())
    }

    /// Cryptographic integrity for stored evidence only. This deliberately does
    /// not authorize execution or establish validity at any time. Live callers
    /// must use `verify` with their trusted clock and current enrolled policy.
    pub fn verify_archived_signatures(
        &self,
        approval: &QuorumApproval,
        message: &[u8],
    ) -> Result<()> {
        if approval.suite != SUITE
            || approval.epoch != self.epoch
            || approval.committee != self.digest()?
            || approval.signatures.len() < usize::from(self.threshold)
            || approval.signatures.len() > self.members.len()
        {
            return Err(CryptoError::InvalidSignature);
        }
        let mut previous = 0;
        for signature in &approval.signatures {
            if signature.node <= previous {
                return Err(CryptoError::DuplicateKey);
            }
            previous = signature.node;
            self.verify_member_signature(signature, message)?;
        }
        Ok(())
    }
}
