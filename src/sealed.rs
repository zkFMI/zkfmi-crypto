//! Authenticated recipient delivery for stored note and opening payloads.
//!
//! The existing hybrid KEM establishes an independent key per envelope.
//! RustCrypto AES-GCM implements authenticated encryption. This adapter binds
//! the closed purpose, caller's object context and recipient public key. It
//! does not authorize a recipient key: the calling protocol must pin that key.
use crate::{
    error::{CryptoError, Result},
    hybrid::kem::{HybridCiphertext, HybridKemEncapsulator, HybridKemKey},
    suite::{Suite, SuiteId},
    traits::{KemDecapsulator, KemEncapsulator, SecretBytes},
};
use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce, Tag,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const RECIPIENT_PUBLIC_BYTES: usize = 1216;
pub const MAX_PAYLOAD_BYTES: usize = 1024;
const DOMAIN: &[u8] = b"ZKFMI:RECIPIENT-ENVELOPE:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SealingPurpose {
    NoteOpening,
    ThresholdOpeningShare,
    CredentialCustody,
}

impl SealingPurpose {
    fn code(self) -> u16 {
        match self {
            Self::NoteOpening => 1,
            Self::ThresholdOpeningShare => 2,
            Self::CredentialCustody => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealedMessage {
    pub version: u16,
    pub suite: Suite,
    pub purpose: SealingPurpose,
    pub kem_ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub tag: [u8; 16],
}

fn associated_data(
    purpose: SealingPurpose,
    context: &[u8; 32],
    recipient: &[u8],
) -> Result<Vec<u8>> {
    if recipient.len() != RECIPIENT_PUBLIC_BYTES || *context == [0; 32] {
        return Err(CryptoError::InvalidEncoding);
    }
    let mut data = DOMAIN.to_vec();
    data.extend_from_slice(&1_u16.to_be_bytes());
    data.extend_from_slice(&Suite::new(SuiteId::X25519MlKem768).encode());
    data.extend_from_slice(&purpose.code().to_be_bytes());
    data.extend_from_slice(context);
    data.extend_from_slice(recipient);
    Ok(data)
}

impl SealedMessage {
    pub fn validate(&self, purpose: SealingPurpose, payload_bytes: usize) -> Result<()> {
        if self.version != 1 {
            return Err(CryptoError::UnknownVersion);
        }
        if self.suite != Suite::new(SuiteId::X25519MlKem768) {
            return Err(CryptoError::UnsupportedSuite);
        }
        if self.purpose != purpose {
            return Err(CryptoError::InvalidPurpose);
        }
        if payload_bytes == 0
            || payload_bytes > MAX_PAYLOAD_BYTES
            || self.ciphertext.len() != payload_bytes
        {
            return Err(CryptoError::InvalidCiphertext);
        }
        HybridCiphertext::decode(&self.kem_ciphertext)?;
        Ok(())
    }

    pub fn seal(
        recipient: &[u8],
        purpose: SealingPurpose,
        context: &[u8; 32],
        plaintext: &[u8],
    ) -> Result<Self> {
        if plaintext.is_empty() || plaintext.len() > MAX_PAYLOAD_BYTES {
            return Err(CryptoError::InvalidEncoding);
        }
        let aad = associated_data(purpose, context, recipient)?;
        let encapsulated = HybridKemEncapsulator.encapsulate(recipient)?;
        let cipher = Aes256Gcm::new_from_slice(&encapsulated.shared_secret)
            .map_err(|_| CryptoError::InvalidKey)?;
        let mut nonce = [0; 12];
        getrandom_04::fill(&mut nonce).map_err(|_| CryptoError::Randomness)?;
        let mut buffer = Zeroizing::new(plaintext.to_vec());
        let tag = cipher
            .encrypt_in_place_detached(Nonce::from_slice(&nonce), &aad, &mut buffer)
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        Ok(Self {
            version: 1,
            suite: Suite::new(SuiteId::X25519MlKem768),
            purpose,
            kem_ciphertext: encapsulated.ciphertext,
            nonce,
            ciphertext: buffer.to_vec(),
            tag: tag.into(),
        })
    }

    pub fn open(
        &self,
        recipient: &HybridKemKey,
        purpose: SealingPurpose,
        context: &[u8; 32],
        payload_bytes: usize,
    ) -> Result<SecretBytes> {
        self.validate(purpose, payload_bytes)?;
        let aad = associated_data(purpose, context, &recipient.public_key())?;
        let key = recipient.decapsulate(&self.kem_ciphertext)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::InvalidKey)?;
        let mut buffer = Zeroizing::new(self.ciphertext.clone());
        cipher
            .decrypt_in_place_detached(
                Nonce::from_slice(&self.nonce),
                &aad,
                &mut buffer,
                Tag::from_slice(&self.tag),
            )
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        Ok(buffer)
    }

    /// Fixed-width canonical bytes after the caller validates its exact payload
    /// size. Application state commitments must bind every returned byte.
    pub fn binding_bytes(&self) -> Vec<u8> {
        let mut out = DOMAIN.to_vec();
        out.extend_from_slice(&self.version.to_be_bytes());
        out.extend_from_slice(&self.suite.encode());
        out.extend_from_slice(&self.purpose.code().to_be_bytes());
        out.extend_from_slice(&(self.kem_ciphertext.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.kem_ciphertext);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&(self.ciphertext.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.ciphertext);
        out.extend_from_slice(&self.tag);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipient_and_object_binding_reject_changes_to_either_kem_component_or_ciphertext() {
        let key = HybridKemKey::generate().unwrap();
        let other = HybridKemKey::generate().unwrap();
        let context = [7; 32];
        let purpose = SealingPurpose::NoteOpening;
        let envelope = SealedMessage::seal(&key.public_key(), purpose, &context, &[3; 64]).unwrap();
        assert_eq!(
            &*envelope.open(&key, purpose, &context, 64).unwrap(),
            &[3; 64]
        );
        assert!(envelope.open(&other, purpose, &context, 64).is_err());
        assert!(envelope.open(&key, purpose, &[8; 32], 64).is_err());
        assert!(envelope
            .open(&key, SealingPurpose::ThresholdOpeningShare, &context, 64)
            .is_err());
        for field in 0..8 {
            let mut bad = envelope.clone();
            match field {
                0 => bad.kem_ciphertext[0] ^= 1,
                1 => bad.kem_ciphertext[32] ^= 1,
                2 => bad.nonce[0] ^= 1,
                3 => bad.ciphertext[0] ^= 1,
                4 => bad.tag[0] ^= 1,
                5 => bad.version = 0,
                6 => bad.suite = Suite::new(SuiteId::Ed25519),
                _ => {
                    bad.kem_ciphertext.pop();
                }
            }
            assert!(bad.open(&key, purpose, &context, 64).is_err());
        }
        assert!(envelope.open(&key, purpose, &context, 63).is_err());
    }

    #[cfg(feature = "tls")]
    #[test]
    fn openssl_independently_opens_the_rustcrypto_aes_gcm_payload() {
        let key = HybridKemKey::generate().unwrap();
        let context = [19; 32];
        let purpose = SealingPurpose::ThresholdOpeningShare;
        let envelope =
            SealedMessage::seal(&key.public_key(), purpose, &context, &[21; 64]).unwrap();
        let shared = key.decapsulate(&envelope.kem_ciphertext).unwrap();
        let aad = associated_data(purpose, &context, &key.public_key()).unwrap();
        let plaintext = Zeroizing::new(
            openssl::symm::decrypt_aead(
                openssl::symm::Cipher::aes_256_gcm(),
                &shared,
                Some(&envelope.nonce),
                &aad,
                &envelope.ciphertext,
                &envelope.tag,
            )
            .unwrap(),
        );
        assert_eq!(&*plaintext, &[21; 64]);
    }
}
