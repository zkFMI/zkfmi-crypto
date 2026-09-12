//! Research-only canonical settlement binding. No production security claim.
pub mod circuit;
#[cfg(feature = "owner-input")]
pub mod input_delivery;
#[cfg(feature = "native-prover")]
pub mod live_run;
#[cfg(feature = "native-prover")]
pub mod mpc_program;
#[cfg(feature = "native-prover")]
pub mod native;
#[cfg(feature = "owner-input")]
pub mod owner_input;
#[cfg(feature = "native-prover")]
pub mod party;
#[cfg(feature = "native-prover")]
pub mod prover;
#[cfg(feature = "native-prover")]
pub mod query_agreement;
#[cfg(feature = "native-prover")]
pub mod run;
pub mod verifier;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SettlementStatement {
    pub deployment_id: String,
    pub book_id: String,
    pub operation_id: String,
    pub sequence: u64,
    pub before_commitment: String,
    pub after_commitment: String,
    pub no_fill: bool,
}

impl SettlementStatement {
    pub fn validate(&self) -> Result<(), String> {
        for id in [&self.deployment_id, &self.book_id, &self.operation_id] {
            if id.is_empty() || id.len() > 128 || !id.is_ascii() {
                return Err("invalid settlement identifier".into());
            }
        }
        if self.sequence == 0 || self.sequence == u64::MAX {
            return Err("invalid settlement sequence".into());
        }
        for h in [&self.before_commitment, &self.after_commitment] {
            if h.len() != 128
                || h.bytes()
                    .any(|b| !b.is_ascii_digit() && !(b'a'..=b'f').contains(&b))
            {
                return Err("invalid canonical book commitment".into());
            }
        }
        Ok(())
    }
}

pub fn book_commitment(roots: &[crate::pcs::NodeRoots], kind: usize) -> Result<String, String> {
    if roots.len() != 7 || ![4, 6].contains(&kind) || roots.iter().any(|r| r.roots.len() != 8) {
        return Err("book commitment roster".into());
    }
    let mut hash = Sha512::new();
    hash.update(b"zkfmi-cocode-private-book-v1");
    for (party, row) in roots.iter().enumerate() {
        hash.update((party as u64).to_le_bytes());
        let raw = hex::decode(&row.roots[kind]).map_err(|_| "book root encoding")?;
        if raw.len() != 64 {
            return Err("book root length".into());
        }
        hash.update(raw);
    }
    Ok(hex::encode(hash.finalize()))
}

pub fn verify_serialized(bytes: &[u8], expected: &SettlementStatement) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > 256 * 1024 * 1024 {
        return Err("integration proof size".into());
    }
    let proof: verifier::Proof =
        serde_json::from_slice(bytes).map_err(|_| "integration proof encoding")?;
    verifier::verify(&proof, expected)
}

pub fn verify_serialized_query_agreement(
    bytes: &[u8],
    expected: &SettlementStatement,
    trusted_roster_sha512: &str,
) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > 256 * 1024 * 1024 {
        return Err("integration proof size".into());
    }
    let proof: verifier::Proof =
        serde_json::from_slice(bytes).map_err(|_| "integration proof encoding")?;
    verifier::verify_query_agreement(&proof, expected, trusted_roster_sha512)
}

pub fn serialized_statement(bytes: &[u8]) -> Result<SettlementStatement, String> {
    if bytes.is_empty() || bytes.len() > 256 * 1024 * 1024 {
        return Err("integration proof size".into());
    }
    let proof: verifier::Proof =
        serde_json::from_slice(bytes).map_err(|_| "integration proof encoding")?;
    proof.statement.validate_legacy()?;
    Ok(proof.statement.settlement)
}

pub fn serialized_query_agreement_statement(
    bytes: &[u8],
    trusted_roster_sha512: &str,
) -> Result<SettlementStatement, String> {
    if bytes.is_empty() || bytes.len() > 256 * 1024 * 1024 {
        return Err("integration proof size".into());
    }
    let proof: verifier::Proof =
        serde_json::from_slice(bytes).map_err(|_| "integration proof encoding")?;
    proof
        .statement
        .validate_query_agreement(trusted_roster_sha512)?;
    Ok(proof.statement.settlement)
}
