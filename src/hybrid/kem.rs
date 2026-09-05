//! Standalone P0 combiner, not a TLS implementation or an authenticated handshake.
use crate::{
    backend::{MlKem768Encapsulator, MlKem768Key},
    error::{CryptoError, Result},
    suite::{Suite, SuiteId, ML_KEM_768_CT_BYTES, ML_KEM_768_EK_BYTES},
    traits::{Encapsulation, KemDecapsulator, KemEncapsulator, SecretBytes},
};
use hkdf::Hkdf;
use rand_core_06::RngCore as _;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroizing;

pub const DOMAIN: &[u8] = b"ZKFMI:HYBRID-KEM:v1";
pub const INFO: &[u8] = b"ZKFMI:HYBRID-KEM:SESSION:v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HybridCiphertext {
    pub classical: Vec<u8>,
    pub pq: Vec<u8>,
}

impl HybridCiphertext {
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.classical.len() != 32 || self.pq.len() != ML_KEM_768_CT_BYTES {
            return Err(CryptoError::InvalidCiphertext);
        }
        Ok([self.classical.as_slice(), self.pq.as_slice()].concat())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 + ML_KEM_768_CT_BYTES {
            return Err(CryptoError::InvalidCiphertext);
        }
        Ok(Self {
            classical: bytes[..32].to_vec(),
            pq: bytes[32..].to_vec(),
        })
    }
}

/// HKDF-SHA256(salt=DOMAIN, ikm=ss_x || ss_pq || ct_x || ct_pq || suite_id_and_version,
/// info=INFO, L=32). All fields are fixed length; suite encoding is four big-endian bytes.
pub fn combine(
    x_secret: &[u8; 32],
    pq_secret: &[u8; 32],
    ciphertext: &HybridCiphertext,
    suite: Suite,
) -> Result<SecretBytes> {
    if suite != Suite::new(SuiteId::X25519MlKem768) {
        return Err(CryptoError::UnsupportedSuite);
    }
    let ct = ciphertext.encode()?;
    let mut ikm = Zeroizing::new(Vec::with_capacity(64 + ct.len() + 4));
    ikm.extend_from_slice(x_secret);
    ikm.extend_from_slice(pq_secret);
    ikm.extend_from_slice(&ct);
    ikm.extend_from_slice(&suite.encode());
    let hkdf = Hkdf::<Sha256>::new(Some(DOMAIN), &ikm);
    let mut output = Zeroizing::new(vec![0u8; 32]);
    hkdf.expand(INFO, &mut output)
        .map_err(|_| CryptoError::InvalidEncoding)?;
    Ok(output)
}

fn generate_x25519() -> Result<StaticSecret> {
    let mut seed = Zeroizing::new([0u8; 32]);
    rand_core_06::OsRng
        .try_fill_bytes(seed.as_mut())
        .map_err(|_| CryptoError::Randomness)?;
    Ok(StaticSecret::from(*seed))
}

pub struct HybridKemKey {
    classical: StaticSecret,
    pq: MlKem768Key,
}

impl HybridKemKey {
    pub fn generate() -> Result<Self> {
        Ok(Self {
            classical: generate_x25519()?,
            pq: MlKem768Key::generate()?,
        })
    }
}

impl KemDecapsulator for HybridKemKey {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::X25519MlKem768)
    }
    fn public_key(&self) -> Vec<u8> {
        [
            PublicKey::from(&self.classical).as_bytes().as_slice(),
            self.pq.public_key().as_slice(),
        ]
        .concat()
    }
    fn decapsulate(&self, ciphertext: &[u8]) -> Result<SecretBytes> {
        let ct = HybridCiphertext::decode(ciphertext)?;
        let x_bytes: [u8; 32] = ct
            .classical
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        let shared_x = self.classical.diffie_hellman(&PublicKey::from(x_bytes));
        if !shared_x.was_contributory() {
            return Err(CryptoError::InvalidCiphertext);
        }
        let shared_pq = self.pq.decapsulate(&ct.pq)?;
        combine(
            shared_x.as_bytes(),
            shared_pq
                .as_slice()
                .try_into()
                .map_err(|_| CryptoError::InvalidCiphertext)?,
            &ct,
            self.suite(),
        )
    }
}

pub struct HybridKemEncapsulator;

impl KemEncapsulator for HybridKemEncapsulator {
    fn suite(&self) -> Suite {
        Suite::new(SuiteId::X25519MlKem768)
    }
    fn encapsulate(&self, public_key: &[u8]) -> Result<Encapsulation> {
        if public_key.len() != 32 + ML_KEM_768_EK_BYTES {
            return Err(CryptoError::InvalidKey);
        }
        let x_bytes: [u8; 32] = public_key[..32]
            .try_into()
            .map_err(|_| CryptoError::InvalidKey)?;
        let ephemeral = generate_x25519()?;
        let shared_x = ephemeral.diffie_hellman(&PublicKey::from(x_bytes));
        if !shared_x.was_contributory() {
            return Err(CryptoError::InvalidKey);
        }
        let pq = MlKem768Encapsulator.encapsulate(&public_key[32..])?;
        let ct = HybridCiphertext {
            classical: PublicKey::from(&ephemeral).to_bytes().to_vec(),
            pq: pq.ciphertext,
        };
        let shared_secret = combine(
            shared_x.as_bytes(),
            pq.shared_secret
                .as_slice()
                .try_into()
                .map_err(|_| CryptoError::InvalidKey)?,
            &ct,
            self.suite(),
        )?;
        Ok(Encapsulation {
            ciphertext: ct.encode()?,
            shared_secret,
        })
    }
}
