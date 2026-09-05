//! V1 uses big-endian fixed integers and u32 byte lengths/counts. Strings are exact UTF-8.
use crate::{
    error::{CryptoError, Result},
    key::KeyPurpose,
    suite::{Suite, Version},
};
use serde::{Deserialize, Serialize};

pub const DOMAIN: &[u8] = b"ZKFMI:CANONICAL:v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningPreimage {
    pub protocol: String,
    pub protocol_version: Version,
    pub network_id: String,
    pub deployment_id: String,
    pub contract_id: String,
    pub tx_kind: KeyPurpose,
    /// Order is significant; callers must use the protocol's defined object order.
    pub object_ids: Vec<String>,
    pub sequence_or_nonce: u64,
    /// Exclusive expiry in Unix seconds. At exactly expires_at the object is expired.
    pub expires_at: u64,
    pub suite: Suite,
    /// SHA-256 of the protocol-defined body bytes; the body encoding is caller-owned.
    pub body_hash: [u8; 32],
}

pub(crate) fn put_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<()> {
    let len = u32::try_from(value.len()).map_err(|_| CryptoError::InvalidEncoding)?;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(value);
    Ok(())
}

impl SigningPreimage {
    pub fn encode(&self) -> Result<Vec<u8>> {
        if [
            &self.protocol,
            &self.network_id,
            &self.deployment_id,
            &self.contract_id,
        ]
        .iter()
        .any(|s| s.is_empty())
            || self.object_ids.iter().any(String::is_empty)
        {
            return Err(CryptoError::InvalidEncoding);
        }
        let mut out = DOMAIN.to_vec();
        put_bytes(&mut out, self.protocol.as_bytes())?;
        out.extend_from_slice(&u16::from(self.protocol_version).to_be_bytes());
        for field in [&self.network_id, &self.deployment_id, &self.contract_id] {
            put_bytes(&mut out, field.as_bytes())?;
        }
        out.extend_from_slice(&self.tx_kind.code().to_be_bytes());
        let count =
            u32::try_from(self.object_ids.len()).map_err(|_| CryptoError::InvalidEncoding)?;
        out.extend_from_slice(&count.to_be_bytes());
        for id in &self.object_ids {
            put_bytes(&mut out, id.as_bytes())?;
        }
        out.extend_from_slice(&self.sequence_or_nonce.to_be_bytes());
        out.extend_from_slice(&self.expires_at.to_be_bytes());
        out.extend_from_slice(&self.suite.encode());
        out.extend_from_slice(&self.body_hash);
        Ok(out)
    }

    pub fn validate_at(&self, now: u64) -> Result<()> {
        if now >= self.expires_at {
            return Err(CryptoError::Expired);
        }
        self.encode().map(|_| ())
    }
}
