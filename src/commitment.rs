//! Salted hash commitments using upstream SHA-512, not an algebraic substitute
//! for Pedersen. No homomorphic operations or proof of opening are supplied here.
//!
//! A future ZK relation must check these exact bytes *inside* its proved
//! computation. Checking an opening in this API discloses it to the verifier.
use crate::{
    error::{CryptoError, Result},
    mode::DeploymentCryptoPolicy,
};
use sha2::{Digest, Sha512};
use zeroize::Zeroizing;

pub const HASH_COMMITMENT_BYTES: usize = 64;
pub const OPENING_SALT_BYTES: usize = 64;
pub const MAX_COMMITTED_BYTES: usize = 1 << 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HashCommitment(pub [u8; HASH_COMMITMENT_BYTES]);

/// Deliberately has no Debug, Clone, or serialization implementation.
pub struct HashOpening {
    salt: Zeroizing<[u8; OPENING_SALT_BYTES]>,
}

impl HashOpening {
    pub fn generate() -> Result<Self> {
        let mut salt = Zeroizing::new([0; OPENING_SALT_BYTES]);
        getrandom_04::fill(salt.as_mut()).map_err(|_| CryptoError::Randomness)?;
        Ok(Self { salt })
    }

    /// A custody layer or secret-shared proving input may access these bytes.
    /// They must never be put in public state or a public proof statement.
    pub fn secret_bytes(&self) -> &[u8; OPENING_SALT_BYTES] {
        &self.salt
    }
}

impl HashCommitment {
    pub fn commit(
        policy: &DeploymentCryptoPolicy,
        object_context: &[u8; 32],
        value: &[u8],
        opening: &HashOpening,
    ) -> Result<Self> {
        if *object_context == [0; 32] || value.is_empty() || value.len() > MAX_COMMITTED_BYTES {
            return Err(CryptoError::InvalidEncoding);
        }
        let mut hash = Sha512::new();
        hash.update(b"ZKFMI:SALTED-COMMITMENT:SHA512:v1");
        hash.update(policy.encode()?);
        hash.update(object_context);
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value);
        hash.update(opening.secret_bytes());
        Ok(Self(hash.finalize().into()))
    }

    pub fn verify_opening(
        &self,
        policy: &DeploymentCryptoPolicy,
        object_context: &[u8; 32],
        value: &[u8],
        opening: &HashOpening,
    ) -> Result<()> {
        let expected = Self::commit(policy, object_context, value, opening)?;
        if *self != expected {
            return Err(CryptoError::InvalidProof);
        }
        Ok(())
    }
}
