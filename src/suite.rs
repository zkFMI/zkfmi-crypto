//! Closed suite identifiers. Sizes are raw primitive bytes, excluding wire framing.
use crate::error::{CryptoError, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub enum Version {
    #[default]
    V1,
}

impl TryFrom<u16> for Version {
    type Error = CryptoError;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            1 => Ok(Self::V1),
            _ => Err(CryptoError::UnknownVersion),
        }
    }
}

impl From<Version> for u16 {
    fn from(_: Version) -> Self {
        1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SuiteId {
    Ed25519,
    FrostRistretto255,
    PedersenRistretto255,
    BulletproofsRistretto255,
    X25519Tls13,
    Sha256,
    Sha512,
    Shake128,
    MlKem768,
    MlDsa65,
    MlDsa44,
    SlhDsaSha2_128s,
    X25519MlKem768,
    Ed25519MlDsa65,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub id: SuiteId,
    pub version: Version,
}

impl Suite {
    pub const fn new(id: SuiteId) -> Self {
        Self {
            id,
            version: Version::V1,
        }
    }

    pub fn encode(self) -> [u8; 4] {
        let id = self.id.code().to_be_bytes();
        [id[0], id[1], 0, 1]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CryptoPurpose {
    Signature,
    Kem,
    Commitment,
    Proof,
    Hash,
    Transport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteMetadata {
    /// True denotes the intended PQ primitive family, not implementation certification.
    pub post_quantum: bool,
    pub purpose: CryptoPurpose,
    pub public_key_bytes: Option<usize>,
    pub signature_bytes: Option<usize>,
    pub ciphertext_bytes: Option<usize>,
}

pub const ML_KEM_768_EK_BYTES: usize = 1184;
pub const ML_KEM_768_DK_BYTES: usize = 2400;
pub const ML_KEM_768_CT_BYTES: usize = 1088;
pub const ML_DSA_44_PK_BYTES: usize = 1312;
pub const ML_DSA_44_SIG_BYTES: usize = 2420;
pub const ML_DSA_65_PK_BYTES: usize = 1952;
pub const ML_DSA_65_SIG_BYTES: usize = 3309;
pub const ML_DSA_87_PK_BYTES: usize = 2592;
pub const ML_DSA_87_SIG_BYTES: usize = 4627;
pub const SLH_DSA_SHA2_128S_PK_BYTES: usize = 32;
pub const SLH_DSA_SHA2_128S_SIG_BYTES: usize = 7856;

impl SuiteId {
    /// Stable wire discriminants. Never infer these from enum declaration order.
    pub const fn code(self) -> u16 {
        match self {
            Self::Ed25519 => 1,
            Self::FrostRistretto255 => 2,
            Self::PedersenRistretto255 => 3,
            Self::BulletproofsRistretto255 => 4,
            Self::X25519Tls13 => 5,
            Self::Sha256 => 6,
            Self::Sha512 => 7,
            Self::Shake128 => 8,
            Self::MlKem768 => 0x101,
            Self::MlDsa65 => 0x102,
            Self::MlDsa44 => 0x103,
            Self::SlhDsaSha2_128s => 0x104,
            Self::X25519MlKem768 => 0x201,
            Self::Ed25519MlDsa65 => 0x202,
        }
    }

    pub const fn metadata(self) -> SuiteMetadata {
        use CryptoPurpose::*;
        let (post_quantum, purpose, public_key_bytes, signature_bytes, ciphertext_bytes) =
            match self {
                Self::Ed25519 | Self::FrostRistretto255 => {
                    (false, Signature, Some(32), Some(64), None)
                }
                Self::PedersenRistretto255 => (false, Commitment, None, None, None),
                Self::BulletproofsRistretto255 => (false, Proof, None, None, None),
                Self::X25519Tls13 => (false, Transport, Some(32), None, Some(32)),
                Self::Sha256 | Self::Sha512 | Self::Shake128 => (true, Hash, None, None, None),
                Self::MlKem768 => (
                    true,
                    Kem,
                    Some(ML_KEM_768_EK_BYTES),
                    None,
                    Some(ML_KEM_768_CT_BYTES),
                ),
                Self::MlDsa65 => (
                    true,
                    Signature,
                    Some(ML_DSA_65_PK_BYTES),
                    Some(ML_DSA_65_SIG_BYTES),
                    None,
                ),
                Self::MlDsa44 => (
                    true,
                    Signature,
                    Some(ML_DSA_44_PK_BYTES),
                    Some(ML_DSA_44_SIG_BYTES),
                    None,
                ),
                Self::SlhDsaSha2_128s => (
                    true,
                    Signature,
                    Some(SLH_DSA_SHA2_128S_PK_BYTES),
                    Some(SLH_DSA_SHA2_128S_SIG_BYTES),
                    None,
                ),
                Self::X25519MlKem768 => (
                    true,
                    Transport,
                    Some(32 + ML_KEM_768_EK_BYTES),
                    None,
                    Some(32 + ML_KEM_768_CT_BYTES),
                ),
                Self::Ed25519MlDsa65 => (
                    true,
                    Signature,
                    Some(32 + ML_DSA_65_PK_BYTES),
                    Some(64 + ML_DSA_65_SIG_BYTES),
                    None,
                ),
            };
        SuiteMetadata {
            post_quantum,
            purpose,
            public_key_bytes,
            signature_bytes,
            ciphertext_bytes,
        }
    }
}

impl TryFrom<u16> for SuiteId {
    type Error = CryptoError;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            1 => Ok(Self::Ed25519),
            2 => Ok(Self::FrostRistretto255),
            3 => Ok(Self::PedersenRistretto255),
            4 => Ok(Self::BulletproofsRistretto255),
            5 => Ok(Self::X25519Tls13),
            6 => Ok(Self::Sha256),
            7 => Ok(Self::Sha512),
            8 => Ok(Self::Shake128),
            0x101 => Ok(Self::MlKem768),
            0x102 => Ok(Self::MlDsa65),
            0x103 => Ok(Self::MlDsa44),
            0x104 => Ok(Self::SlhDsaSha2_128s),
            0x201 => Ok(Self::X25519MlKem768),
            0x202 => Ok(Self::Ed25519MlDsa65),
            _ => Err(CryptoError::UnsupportedSuite),
        }
    }
}
