//! Authenticated recipient delivery for stored note and opening payloads.
//!
//! The existing hybrid KEM establishes an independent key per envelope.
//! RustCrypto AES-GCM implements authenticated encryption. This adapter binds
//! the closed purpose, caller's object context and recipient public key. It
//! does not authorize a recipient key: the calling protocol must pin that key.
use crate::{
    error::{CryptoError, Result},
    hybrid::kem::{HybridCiphertext, HybridKemEncapsulator},
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

/// Sender-local secret entropy for the private encryption relation. This is
/// deliberately not Debug, Clone or Serialize. Never put it in a statement,
/// public artifact, coordinator configuration or recipient envelope.
#[cfg(feature = "private-proof-witness")]
pub struct SealingWitness {
    coins: Zeroizing<[u8; 64]>,
}

#[cfg(feature = "private-proof-witness")]
impl SealingWitness {
    /// X25519 seed then independent ML-KEM m. Only a sender-local secret
    /// sharing/keystore adapter may consume these bytes; borrowed to avoid an
    /// implicit unprotected copy. Public nonce is already in the envelope.
    pub fn private_coins(&self) -> &[u8; 64] {
        &self.coins
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SealingPurpose {
    NoteOpening,
    ThresholdOpeningShare,
    CredentialCustody,
    PrivateMpcInput,
}

impl SealingPurpose {
    fn code(self) -> u16 {
        match self {
            Self::NoteOpening => 1,
            Self::ThresholdOpeningShare => 2,
            Self::CredentialCustody => 3,
            Self::PrivateMpcInput => 4,
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
    /// Create an ordinary interoperable envelope and retain its sender-private
    /// randomness for proving the full encryption relation. This is NOT a proof
    /// or an alternative acceptance check. The ordinary seal API is unchanged.
    #[cfg(feature = "private-proof-witness")]
    pub fn seal_for_private_proof(
        recipient: &[u8],
        purpose: SealingPurpose,
        context: &[u8; 32],
        plaintext: &[u8],
    ) -> Result<(Self, SealingWitness)> {
        let mut coins = Zeroizing::new([0u8; 64]);
        // Separate OS requests keep the two KEM randomness sources explicit.
        getrandom_04::fill(&mut coins[..32]).map_err(|_| CryptoError::Randomness)?;
        getrandom_04::fill(&mut coins[32..]).map_err(|_| CryptoError::Randomness)?;
        let witness = SealingWitness { coins };
        let mut nonce = [0; 12];
        getrandom_04::fill(&mut nonce).map_err(|_| CryptoError::Randomness)?;
        let envelope =
            Self::seal_with_private_coins(recipient, purpose, context, plaintext, nonce, &witness)?;
        Ok((envelope, witness))
    }

    #[cfg(feature = "private-proof-witness")]
    fn seal_with_private_coins(
        recipient: &[u8],
        purpose: SealingPurpose,
        context: &[u8; 32],
        plaintext: &[u8],
        nonce: [u8; 12],
        witness: &SealingWitness,
    ) -> Result<Self> {
        if plaintext.is_empty() || plaintext.len() > MAX_PAYLOAD_BYTES {
            return Err(CryptoError::InvalidEncoding);
        }
        let aad = associated_data(purpose, context, recipient)?;
        let encapsulated =
            HybridKemEncapsulator.encapsulate_with_coins(recipient, &witness.coins)?;
        let cipher = Aes256Gcm::new_from_slice(&encapsulated.shared_secret)
            .map_err(|_| CryptoError::InvalidKey)?;
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

    /// The application supplies a trusted resident-key/keystore capability.
    /// The envelope and capability must both use the exact hybrid KEM suite.
    pub fn open(
        &self,
        recipient: &dyn KemDecapsulator,
        purpose: SealingPurpose,
        context: &[u8; 32],
        payload_bytes: usize,
    ) -> Result<SecretBytes> {
        self.validate(purpose, payload_bytes)?;
        if recipient.suite() != Suite::new(SuiteId::X25519MlKem768) {
            return Err(CryptoError::UnsupportedSuite);
        }
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
    use crate::hybrid::kem::HybridKemKey;

    #[test]
    fn portable_bearssl_combiner_aad_and_aead_match_rustcrypto() {
        // Public arithmetic boundary vector, not a valid KEM or note fixture.
        let vector = include_bytes!("../tests/vectors/bearssl-envelope-known-answer.bin");
        assert_eq!(vector.len(), 32 + 232 + 16);
        let ct = HybridCiphertext::decode(&[33; 1120]).unwrap();
        let key = crate::hybrid::kem::combine(
            &[11; 32],
            &[22; 32],
            &ct,
            Suite::new(SuiteId::X25519MlKem768),
        )
        .unwrap();
        assert_eq!(&*key, &vector[..32]);
        let aad = associated_data(SealingPurpose::NoteOpening, &[66; 32], &[55; 1216]).unwrap();
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let mut plaintext = [77; 232];
        let tag = cipher
            .encrypt_in_place_detached(Nonce::from_slice(&[44; 12]), &aad, &mut plaintext)
            .unwrap();
        assert_eq!(&plaintext, &vector[32..264]);
        assert_eq!(&tag[..], &vector[264..]);
    }

    #[test]
    #[cfg(feature = "private-proof-witness")]
    fn private_proof_entropy_replays_exact_envelope_and_opens_with_ordinary_key() {
        let key = HybridKemKey::generate().unwrap();
        let recipient = key.public_key();
        let context = [91; 32];
        let plaintext = [37; 232];
        let purpose = SealingPurpose::NoteOpening;
        let (envelope, witness) =
            SealedMessage::seal_for_private_proof(&recipient, purpose, &context, &plaintext)
                .unwrap();
        assert_eq!(
            &*envelope.open(&key, purpose, &context, 232).unwrap(),
            &plaintext
        );
        assert_eq!(
            envelope,
            SealedMessage::seal_with_private_coins(
                &recipient,
                purpose,
                &context,
                &plaintext,
                envelope.nonce,
                &witness
            )
            .unwrap()
        );
        for position in [0, 32] {
            let mut changed = SealingWitness {
                coins: Zeroizing::new(*witness.private_coins()),
            };
            changed.coins[position] ^= 0x80;
            let other = SealedMessage::seal_with_private_coins(
                &recipient,
                purpose,
                &context,
                &plaintext,
                envelope.nonce,
                &changed,
            )
            .unwrap();
            assert_ne!(envelope.kem_ciphertext, other.kem_ciphertext);
            assert_ne!(envelope.ciphertext, other.ciphertext);
        }
        let mut invalid = recipient;
        invalid[..32].fill(0);
        assert!(
            SealedMessage::seal_for_private_proof(&invalid, purpose, &context, &plaintext).is_err()
        );
    }

    #[test]
    fn resident_capability_must_use_the_exact_hybrid_suite() {
        struct WrongSuite(HybridKemKey);
        impl KemDecapsulator for WrongSuite {
            fn suite(&self) -> Suite {
                Suite::new(SuiteId::MlKem768)
            }
            fn public_key(&self) -> Vec<u8> {
                self.0.public_key()
            }
            fn decapsulate(&self, _: &[u8]) -> Result<SecretBytes> {
                panic!("wrong-suite capability must be rejected before decapsulation")
            }
        }
        let key = HybridKemKey::generate().unwrap();
        let context = [23; 32];
        let envelope = SealedMessage::seal(
            &key.public_key(),
            SealingPurpose::PrivateMpcInput,
            &context,
            &[1; 32],
        )
        .unwrap();
        assert!(envelope
            .open(
                &WrongSuite(key),
                SealingPurpose::PrivateMpcInput,
                &context,
                32
            )
            .is_err());
    }

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
