//! Both signature components are mandatory and domain-bound to the hybrid suite.
use crate::{
    backend::{Ed25519Signer, Ed25519Verifier, MlDsa65Signer, MlDsa65Verifier},
    error::{CryptoError, Result},
    key::KeyPurpose,
    suite::{Suite, SuiteId, ML_DSA_65_PK_BYTES, ML_DSA_65_SIG_BYTES},
    traits::{Signer, Verifier},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HybridSignature {
    pub classical: Vec<u8>,
    pub pq: Vec<u8>,
}

impl HybridSignature {
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.classical.len() != 64 || self.pq.len() != ML_DSA_65_SIG_BYTES {
            return Err(CryptoError::InvalidSignature);
        }
        Ok([self.classical.as_slice(), self.pq.as_slice()].concat())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 64 + ML_DSA_65_SIG_BYTES {
            return Err(CryptoError::InvalidSignature);
        }
        Ok(Self {
            classical: bytes[..64].to_vec(),
            pq: bytes[64..].to_vec(),
        })
    }
}

pub struct HybridSigner {
    classical: Ed25519Signer,
    pq: MlDsa65Signer,
}

impl HybridSigner {
    pub fn generate() -> Result<Self> {
        Ok(Self::new(
            Ed25519Signer::generate()?,
            MlDsa65Signer::generate()?,
        ))
    }
    pub fn new(classical: Ed25519Signer, pq: MlDsa65Signer) -> Self {
        Self { classical, pq }
    }
    pub fn sign_hybrid(&self, purpose: KeyPurpose, message: &[u8]) -> Result<HybridSignature> {
        Ok(HybridSignature {
            classical: self
                .classical
                .sign_in_suite(purpose, self.suite(), message)?,
            pq: self.pq.sign_in_suite(purpose, self.suite(), message)?,
        })
    }
}

impl Signer for HybridSigner {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::Ed25519MlDsa65)
    }
    fn public_key(&self) -> Vec<u8> {
        [self.classical.public_key(), self.pq.public_key()].concat()
    }
    fn sign(&self, purpose: KeyPurpose, message: &[u8]) -> Result<Vec<u8>> {
        self.sign_hybrid(purpose, message)?.encode()
    }
}

pub struct HybridVerifier;

impl HybridVerifier {
    pub fn verify_hybrid(
        &self,
        purpose: KeyPurpose,
        public_key: &[u8],
        message: &[u8],
        signature: &HybridSignature,
    ) -> Result<()> {
        if public_key.len() != 32 + ML_DSA_65_PK_BYTES {
            return Err(CryptoError::InvalidKey);
        }
        if signature.classical.len() != 64 || signature.pq.len() != ML_DSA_65_SIG_BYTES {
            return Err(CryptoError::InvalidSignature);
        }
        // Evaluate both components; never convert a PQ error into classical-only acceptance.
        let classical = Ed25519Verifier::verify_in_suite(
            purpose,
            self.suite(),
            &public_key[..32],
            message,
            &signature.classical,
        );
        let pq = MlDsa65Verifier::verify_in_suite(
            purpose,
            self.suite(),
            &public_key[32..],
            message,
            &signature.pq,
        );
        classical.and(pq)
    }
}

impl Verifier for HybridVerifier {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::Ed25519MlDsa65)
    }
    fn verify(
        &self,
        purpose: KeyPurpose,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        self.verify_hybrid(
            purpose,
            public_key,
            message,
            &HybridSignature::decode(signature)?,
        )
    }
}
