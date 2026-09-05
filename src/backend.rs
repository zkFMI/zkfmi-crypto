//! RustCrypto adapters. Ed25519/X25519 use rand_core 0.6; PQ adapters use OS bytes / rand_core 0.10.
use crate::{
    canonical::put_bytes,
    error::{CryptoError, Result},
    key::KeyPurpose,
    suite::{Suite, SuiteId},
    traits::{
        CryptoProvider, Encapsulation, KemDecapsulator, KemEncapsulator, ProofVerifier,
        SecretBytes, Signer, Verifier,
    },
};
use ed25519_dalek::Signer as _;
use ml_dsa::Keypair as _;
use ml_kem::{Decapsulate as _, KeyExport as _, MlKem768};
use rand_core_06::RngCore as _;
use std::collections::BTreeMap;
use zeroize::{Zeroize, Zeroizing};

#[cfg(test)]
#[path = "backend_tests.rs"]
mod tests;

pub(crate) fn signature_context(purpose: KeyPurpose, suite: Suite) -> Vec<u8> {
    let mut out = b"ZKFMI:SIGNATURE:v1".to_vec();
    out.extend_from_slice(&purpose.code().to_be_bytes());
    out.extend_from_slice(&suite.encode());
    out
}

fn classical_message(context: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    let mut out = b"ZKFMI:ED25519-CONTEXT:v1".to_vec();
    put_bytes(&mut out, context)?;
    put_bytes(&mut out, message)?;
    Ok(out)
}

pub struct Ed25519Signer {
    key: ed25519_dalek::SigningKey,
}

impl Ed25519Signer {
    pub fn generate() -> Result<Self> {
        let mut seed = Zeroizing::new([0u8; 32]);
        rand_core_06::OsRng
            .try_fill_bytes(seed.as_mut())
            .map_err(|_| CryptoError::Randomness)?;
        Ok(Self::from_seed(&seed))
    }

    /// The caller owns and must erase its seed; use generate for fresh OS-random keys.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            key: ed25519_dalek::SigningKey::from_bytes(seed),
        }
    }

    pub(crate) fn sign_in_suite(
        &self,
        purpose: KeyPurpose,
        suite: Suite,
        message: &[u8],
    ) -> Result<Vec<u8>> {
        Ok(self
            .key
            .sign(&classical_message(
                &signature_context(purpose, suite),
                message,
            )?)
            .to_bytes()
            .to_vec())
    }
}

impl Signer for Ed25519Signer {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::Ed25519)
    }
    fn public_key(&self) -> Vec<u8> {
        self.key.verifying_key().to_bytes().to_vec()
    }
    fn sign(&self, purpose: KeyPurpose, message: &[u8]) -> Result<Vec<u8>> {
        self.sign_in_suite(purpose, self.suite(), message)
    }
}

pub struct Ed25519Verifier;

fn verify_ed25519_raw(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<()> {
    let encoded: &[u8; 32] = public_key.try_into().map_err(|_| CryptoError::InvalidKey)?;
    let key =
        ed25519_dalek::VerifyingKey::from_bytes(encoded).map_err(|_| CryptoError::InvalidKey)?;
    let sig = ed25519_dalek::Signature::from_slice(signature)
        .map_err(|_| CryptoError::InvalidSignature)?;
    key.verify_strict(message, &sig)
        .map_err(|_| CryptoError::InvalidSignature)
}

impl Ed25519Verifier {
    pub(crate) fn verify_in_suite(
        purpose: KeyPurpose,
        suite: Suite,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        verify_ed25519_raw(
            public_key,
            &classical_message(&signature_context(purpose, suite), message)?,
            signature,
        )
    }
}

impl Verifier for Ed25519Verifier {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::Ed25519)
    }
    fn verify(
        &self,
        purpose: KeyPurpose,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        Self::verify_in_suite(purpose, self.suite(), public_key, message, signature)
    }
}

pub struct MlDsa65Signer {
    key: ml_dsa::SigningKey<ml_dsa::MlDsa65>,
}

impl MlDsa65Signer {
    pub fn generate() -> Result<Self> {
        let mut seed = Zeroizing::new([0u8; 32]);
        getrandom_04::fill(seed.as_mut()).map_err(|_| CryptoError::Randomness)?;
        Ok(Self::from_seed(&seed))
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let bytes = Zeroizing::new(ml_dsa::Seed::from(*seed));
        Self {
            key: ml_dsa::SigningKey::from_seed(&bytes),
        }
    }

    pub(crate) fn sign_in_suite(
        &self,
        purpose: KeyPurpose,
        suite: Suite,
        message: &[u8],
    ) -> Result<Vec<u8>> {
        self.key
            .expanded_key()
            .sign_randomized(
                message,
                &signature_context(purpose, suite),
                &mut getrandom_04::SysRng,
            )
            .map(|sig| sig.encode().to_vec())
            .map_err(|_| CryptoError::Randomness)
    }
}

impl Signer for MlDsa65Signer {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::MlDsa65)
    }
    fn public_key(&self) -> Vec<u8> {
        self.key.verifying_key().encode().to_vec()
    }
    fn sign(&self, purpose: KeyPurpose, message: &[u8]) -> Result<Vec<u8>> {
        self.sign_in_suite(purpose, self.suite(), message)
    }
}

pub struct MlDsa65Verifier;

fn verify_mldsa65_context(
    public_key: &[u8],
    message: &[u8],
    context: &[u8],
    signature: &[u8],
) -> Result<()> {
    let encoded = ml_dsa::EncodedVerifyingKey::<ml_dsa::MlDsa65>::try_from(public_key)
        .map_err(|_| CryptoError::InvalidKey)?;
    let key = ml_dsa::VerifyingKey::<ml_dsa::MlDsa65>::decode(&encoded);
    let sig = ml_dsa::Signature::<ml_dsa::MlDsa65>::try_from(signature)
        .map_err(|_| CryptoError::InvalidSignature)?;
    if key.verify_with_context(message, context, &sig) {
        Ok(())
    } else {
        Err(CryptoError::InvalidSignature)
    }
}

impl MlDsa65Verifier {
    pub(crate) fn verify_in_suite(
        purpose: KeyPurpose,
        suite: Suite,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        verify_mldsa65_context(
            public_key,
            message,
            &signature_context(purpose, suite),
            signature,
        )
    }
}

impl Verifier for MlDsa65Verifier {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::MlDsa65)
    }
    fn verify(
        &self,
        purpose: KeyPurpose,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        Self::verify_in_suite(purpose, self.suite(), public_key, message, signature)
    }
}

pub struct MlKem768Key {
    key: ml_kem::DecapsulationKey<MlKem768>,
}

impl MlKem768Key {
    pub fn generate() -> Result<Self> {
        let mut seed = Zeroizing::new([0u8; 64]);
        getrandom_04::fill(seed.as_mut()).map_err(|_| CryptoError::Randomness)?;
        Ok(Self::from_seed(&seed))
    }

    pub fn from_seed(seed: &[u8; 64]) -> Self {
        let bytes = Zeroizing::new(ml_kem::Seed::from(*seed));
        Self {
            key: ml_kem::DecapsulationKey::from_seed(*bytes),
        }
    }
}

impl KemDecapsulator for MlKem768Key {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::MlKem768)
    }
    fn public_key(&self) -> Vec<u8> {
        self.key.encapsulation_key().to_bytes().to_vec()
    }
    fn decapsulate(&self, ciphertext: &[u8]) -> Result<SecretBytes> {
        let ct = ml_kem::Ciphertext::<MlKem768>::try_from(ciphertext)
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        let mut shared = self.key.decapsulate(&ct);
        let result = Zeroizing::new(shared.to_vec());
        shared.zeroize();
        Ok(result)
    }
}

pub struct MlKem768Encapsulator;

impl KemEncapsulator for MlKem768Encapsulator {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::MlKem768)
    }
    fn encapsulate(&self, public_key: &[u8]) -> Result<Encapsulation> {
        let encoded = ml_kem::Key::<ml_kem::EncapsulationKey<MlKem768>>::try_from(public_key)
            .map_err(|_| CryptoError::InvalidKey)?;
        let key = ml_kem::EncapsulationKey::<MlKem768>::new(&encoded)
            .map_err(|_| CryptoError::InvalidKey)?;
        // FIPS 203's m is generated independently from the OS at this backend boundary.
        let mut randomness = Zeroizing::new(ml_kem::B32::default());
        getrandom_04::fill(randomness.as_mut()).map_err(|_| CryptoError::Randomness)?;
        let (ct, mut shared) = key.encapsulate_deterministic(&randomness);
        let shared_secret = Zeroizing::new(shared.to_vec());
        shared.zeroize();
        Ok(Encapsulation {
            ciphertext: ct.to_vec(),
            shared_secret,
        })
    }
}

/// Only explicitly registered capabilities resolve. Knowing a SuiteId does not enable it.
#[derive(Default)]
pub struct Provider {
    verifiers: BTreeMap<Suite, Box<dyn Verifier>>,
    encapsulators: BTreeMap<Suite, Box<dyn KemEncapsulator>>,
    proof_verifiers: BTreeMap<Suite, Box<dyn ProofVerifier>>,
}

impl Provider {
    pub fn register_verifier(&mut self, verifier: Box<dyn Verifier>) -> Result<()> {
        if self.verifiers.contains_key(&verifier.suite()) {
            return Err(CryptoError::DuplicateKey);
        }
        self.verifiers.insert(verifier.suite(), verifier);
        Ok(())
    }
    pub fn register_kem(&mut self, encapsulator: Box<dyn KemEncapsulator>) -> Result<()> {
        if self.encapsulators.contains_key(&encapsulator.suite()) {
            return Err(CryptoError::DuplicateKey);
        }
        self.encapsulators
            .insert(encapsulator.suite(), encapsulator);
        Ok(())
    }
    pub fn register_proof_verifier(&mut self, verifier: Box<dyn ProofVerifier>) -> Result<()> {
        if self.proof_verifiers.contains_key(&verifier.suite()) {
            return Err(CryptoError::DuplicateKey);
        }
        self.proof_verifiers.insert(verifier.suite(), verifier);
        Ok(())
    }
    pub fn rustcrypto() -> Result<Self> {
        let mut out = Self::default();
        out.register_verifier(Box::new(Ed25519Verifier))?;
        out.register_verifier(Box::new(MlDsa65Verifier))?;
        out.register_verifier(Box::new(crate::hybrid::signature::HybridVerifier))?;
        out.register_kem(Box::new(MlKem768Encapsulator))?;
        out.register_kem(Box::new(crate::hybrid::kem::HybridKemEncapsulator))?;
        Ok(out)
    }
}

impl CryptoProvider for Provider {
    fn verifier(&self, suite: Suite) -> Result<&dyn Verifier> {
        self.verifiers
            .get(&suite)
            .map(Box::as_ref)
            .ok_or(CryptoError::UnsupportedSuite)
    }
    fn kem_encapsulator(&self, suite: Suite) -> Result<&dyn KemEncapsulator> {
        self.encapsulators
            .get(&suite)
            .map(Box::as_ref)
            .ok_or(CryptoError::UnsupportedSuite)
    }
    fn proof_verifier(&self, suite: Suite) -> Result<&dyn ProofVerifier> {
        self.proof_verifiers
            .get(&suite)
            .map(Box::as_ref)
            .ok_or(CryptoError::UnsupportedSuite)
    }
}
