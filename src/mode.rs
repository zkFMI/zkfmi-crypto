//! One deployment-wide policy, not a peer-negotiated downgrade preference.
//!
//! A policy is included in the signed protocol context and persisted with state.
//! Parsing a policy is not evidence that its proof backend is available.
use crate::{
    backend::{Ed25519Signer, Ed25519Verifier},
    hybrid::signature::{HybridSigner, HybridVerifier},
    key::KeyPurpose,
    traits::{Signer, Verifier},
};
use crate::{
    error::{CryptoError, Result},
    suite::{Suite, SuiteId, Version},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PqcMode {
    Off,
    On,
}

impl FromStr for PqcMode {
    type Err = CryptoError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "off" => Ok(Self::Off),
            "on" => Ok(Self::On),
            _ => Err(CryptoError::InvalidEncoding),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentCryptoPolicy {
    pub version: Version,
    pub deployment_id: String,
    pub mode: PqcMode,
}

impl DeploymentCryptoPolicy {
    pub fn validate(&self) -> Result<()> {
        if self.deployment_id.is_empty() || self.deployment_id.len() > 255 {
            return Err(CryptoError::InvalidEncoding);
        }
        Ok(())
    }

    /// Unambiguous bytes to bind into a protocol's authenticated context.
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = b"ZKFMI:DEPLOYMENT-CRYPTO:v1".to_vec();
        bytes.extend_from_slice(&u16::from(self.version).to_be_bytes());
        bytes.push(match self.mode {
            PqcMode::Off => 0,
            PqcMode::On => 1,
        });
        bytes.extend_from_slice(&(self.deployment_id.len() as u32).to_be_bytes());
        bytes.extend_from_slice(self.deployment_id.as_bytes());
        Ok(bytes)
    }

    /// Used for both peer admission and reopening a persisted deployment.
    /// A change requires an explicit state migration, never an environment edit.
    pub fn require_same(&self, other: &Self) -> Result<()> {
        self.validate()?;
        other.validate()?;
        if self != other {
            return Err(CryptoError::InvalidPurpose);
        }
        Ok(())
    }

    pub const fn signing_suite(&self) -> Suite {
        Suite::new(match self.mode {
            PqcMode::Off => SuiteId::Ed25519,
            PqcMode::On => SuiteId::Ed25519MlDsa65,
        })
    }

    /// Fresh keys for a new deployment. Existing custody imports its own key
    /// through `sign`; toggling the mode never rotates stored keys implicitly.
    pub fn generate_signer(&self) -> Result<Box<dyn Signer>> {
        self.validate()?;
        match self.mode {
            PqcMode::Off => Ok(Box::new(Ed25519Signer::generate()?)),
            PqcMode::On => Ok(Box::new(HybridSigner::generate()?)),
        }
    }

    fn signing_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        let mut bytes = self.encode()?;
        bytes.extend_from_slice(&(message.len() as u64).to_be_bytes());
        bytes.extend_from_slice(message);
        Ok(bytes)
    }

    pub fn sign(
        &self,
        signer: &dyn Signer,
        purpose: KeyPurpose,
        message: &[u8],
    ) -> Result<Vec<u8>> {
        if signer.suite() != self.signing_suite() {
            return Err(CryptoError::UnsupportedSuite);
        }
        signer.sign(purpose, &self.signing_message(message)?)
    }

    /// The public key must come from the application's authoritative registry,
    /// not from the party submitting the signature.
    pub fn verify(
        &self,
        purpose: KeyPurpose,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        let message = self.signing_message(message)?;
        match self.mode {
            PqcMode::Off => Ed25519Verifier.verify(purpose, public_key, &message, signature),
            PqcMode::On => HybridVerifier.verify(purpose, public_key, &message, signature),
        }
    }

    /// Proof dispatchers must implement both binding and witness privacy
    /// against quantum attackers. An outer PQ signature does not qualify.
    pub fn require_proof_security(&self, security: ProofSecurity) -> Result<()> {
        match (self.mode, security) {
            (PqcMode::Off, ProofSecurity::Classical)
            | (PqcMode::On, ProofSecurity::PostQuantum) => Ok(()),
            _ => Err(CryptoError::UnsupportedSuite),
        }
    }
}

/// A backend's reviewed security class; not derived from a wire-supplied flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofSecurity {
    Classical,
    PostQuantum,
}
