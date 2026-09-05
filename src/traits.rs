//! Backend-neutral byte APIs. No rand_core or signature trait crosses this boundary.
use crate::{error::Result, key::KeyPurpose, suite::Suite};
use zeroize::Zeroizing;

pub type SecretBytes = Zeroizing<Vec<u8>>;

pub struct Encapsulation {
    pub ciphertext: Vec<u8>,
    pub shared_secret: SecretBytes,
}

pub trait Signer: Send + Sync {
    fn suite(&self) -> Suite;
    fn public_key(&self) -> Vec<u8>;
    fn sign(&self, purpose: KeyPurpose, message: &[u8]) -> Result<Vec<u8>>;
}

pub trait Verifier: Send + Sync {
    fn suite(&self) -> Suite;
    fn verify(
        &self,
        purpose: KeyPurpose,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<()>;
}

pub trait KemEncapsulator: Send + Sync {
    fn suite(&self) -> Suite;
    fn encapsulate(&self, public_key: &[u8]) -> Result<Encapsulation>;
}

pub trait KemDecapsulator: Send + Sync {
    fn suite(&self) -> Suite;
    fn public_key(&self) -> Vec<u8>;
    fn decapsulate(&self, ciphertext: &[u8]) -> Result<SecretBytes>;
}

pub trait ProofVerifier: Send + Sync {
    fn suite(&self) -> Suite;
    fn verify_proof(&self, statement: &[u8], proof: &[u8]) -> Result<()>;
}

pub trait CryptoProvider: Send + Sync {
    fn verifier(&self, suite: Suite) -> Result<&dyn Verifier>;
    fn kem_encapsulator(&self, suite: Suite) -> Result<&dyn KemEncapsulator>;
    fn proof_verifier(&self, suite: Suite) -> Result<&dyn ProofVerifier>;
}
