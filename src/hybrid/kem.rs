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

    /// Import independently generated X25519 (32 bytes) and ML-KEM (64 bytes)
    /// seeds from an authenticated encrypted keystore. The caller must erase
    /// its input buffer; no serialization of secret keys is provided here.
    pub fn from_seed(seed: &[u8; 96]) -> Self {
        let classical = Zeroizing::new(<[u8; 32]>::try_from(&seed[..32]).expect("fixed seed"));
        let pq = Zeroizing::new(<[u8; 64]>::try_from(&seed[32..]).expect("fixed seed"));
        Self {
            classical: StaticSecret::from(*classical),
            pq: MlKem768Key::from_seed(&pq),
        }
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

#[cfg(feature = "private-proof-witness")]
impl HybridKemEncapsulator {
    /// Sender-private proof adapter; both independently sampled seeds must stay
    /// private. It uses the same upstream X25519/ML-KEM/HKDF implementations as
    /// the ordinary encapsulator, not a host assertion of encryption validity.
    pub(crate) fn encapsulate_with_coins(
        &self,
        public_key: &[u8],
        coins: &[u8; 64],
    ) -> Result<Encapsulation> {
        if public_key.len() != 32 + ML_KEM_768_EK_BYTES {
            return Err(CryptoError::InvalidKey);
        }
        let seed = Zeroizing::new(<[u8; 32]>::try_from(&coins[..32]).expect("fixed seed"));
        let pq_coins = Zeroizing::new(<[u8; 32]>::try_from(&coins[32..]).expect("fixed coins"));
        let ephemeral = StaticSecret::from(*seed);
        let public = PublicKey::from(<[u8; 32]>::try_from(&public_key[..32]).expect("checked key"));
        let shared_x = ephemeral.diffie_hellman(&public);
        if !shared_x.was_contributory() {
            return Err(CryptoError::InvalidKey);
        }
        let pq = MlKem768Encapsulator.encapsulate_with_coins(&public_key[32..], &pq_coins)?;
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
