use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CryptoError {
    UnknownVersion,
    UnsupportedSuite,
    InvalidEncoding,
    InvalidKey,
    InvalidSignature,
    InvalidCiphertext,
    InvalidPurpose,
    InvalidTime,
    NotFound,
    DuplicateKey,
    Revoked,
    NotYetValid,
    Expired,
    InvalidRotation,
    Randomness,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CryptoError {}

pub type Result<T> = std::result::Result<T, CryptoError>;
