//! Hybrid owner authorization for the v2 query-agreement research protocol.
//!
//! Signatures authorize release of already-derived Merkle openings. They do not
//! replace PCS verification or add proof soundness. A trusted launcher must pin
//! the roster before starting an untrusted coordinator. The bundled same-host
//! launcher does not provide independent operator or private-key custody.

use crate::{
    algebra::{pack, unpack},
    field::{encode, F},
    integration::{
        circuit::{Statement, QUERY_AGREEMENT_PROTOCOL},
        SettlementStatement,
    },
    pcs::{NodeRoots, CODE, QUERIES},
    transcript::Hash,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{
    collections::BTreeSet,
    fs::{self, DirBuilder, File, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;
use zkfmi_crypto::{
    backend::{Ed25519Signer, MlDsa65Signer},
    hybrid::signature::{HybridSigner, HybridVerifier},
    key::KeyPurpose,
    suite::{Suite, SuiteId, ML_DSA_65_PK_BYTES, ML_DSA_65_SIG_BYTES},
    traits::{Signer, Verifier},
};

pub const OWNER_COUNT: usize = 7;
pub const HYBRID_SUITE: &str = "ed25519-ml-dsa-65-and-v1";
const OWNER_KEY_MAGIC: &[u8] = b"ZKFMI:COCODE:QUERY-OWNER-KEY:v2\0";
const OWNER_SEED_BYTES: usize = 64;
const HYBRID_PUBLIC_KEY_BYTES: usize = 32 + ML_DSA_65_PK_BYTES;
const HYBRID_SIGNATURE_BYTES: usize = 64 + ML_DSA_65_SIG_BYTES;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerEnrollment {
    pub party: usize,
    pub public_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedRoster {
    pub protocol: String,
    pub suite: String,
    pub roster_id: String,
    pub owners: Vec<OwnerEnrollment>,
}

fn canonical_hex(value: &str, bytes: usize, label: &'static str) -> Result<Vec<u8>, String> {
    if value.len() != bytes.checked_mul(2).ok_or("hex length overflow")? {
        return Err(label.into());
    }
    let decoded = hex::decode(value).map_err(|_| label.to_string())?;
    if hex::encode(&decoded) != value {
        return Err(label.into());
    }
    Ok(decoded)
}

fn absorb(hash: &mut Sha512, label: &[u8], value: &[u8]) {
    hash.update((label.len() as u64).to_le_bytes());
    hash.update(label);
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        })
}

impl TrustedRoster {
    pub fn validate(&self) -> Result<(), String> {
        if self.protocol != QUERY_AGREEMENT_PROTOCOL
            || self.suite != HYBRID_SUITE
            || !valid_identifier(&self.roster_id)
            || self.owners.len() != OWNER_COUNT
        {
            return Err("invalid trusted query roster metadata".into());
        }
        let mut keys = BTreeSet::new();
        for (party, owner) in self.owners.iter().enumerate() {
            if owner.party != party {
                return Err("trusted query roster must contain ordered distinct owners".into());
            }
            let key = canonical_hex(
                &owner.public_key,
                HYBRID_PUBLIC_KEY_BYTES,
                "invalid trusted hybrid owner public key",
            )?;
            if !keys.insert(key) {
                return Err("duplicate trusted hybrid owner public key".into());
            }
        }
        Ok(())
    }

    /// Canonical roster digest. JSON whitespace and map ordering are not trusted.
    pub fn sha512(&self) -> Result<String, String> {
        self.validate()?;
        let mut hash = Sha512::new();
        absorb(
            &mut hash,
            b"domain",
            b"ZKFMI:COCODE:TRUSTED-QUERY-ROSTER:v2",
        );
        absorb(&mut hash, b"protocol", self.protocol.as_bytes());
        absorb(&mut hash, b"suite-label", self.suite.as_bytes());
        absorb(
            &mut hash,
            b"suite-id",
            &Suite::new(SuiteId::Ed25519MlDsa65).encode(),
        );
        absorb(&mut hash, b"roster-id", self.roster_id.as_bytes());
        for owner in &self.owners {
            absorb(&mut hash, b"party", &(owner.party as u64).to_le_bytes());
            absorb(
                &mut hash,
                b"public-key",
                &canonical_hex(
                    &owner.public_key,
                    HYBRID_PUBLIC_KEY_BYTES,
                    "invalid trusted hybrid owner public key",
                )?,
            );
        }
        Ok(hex::encode(hash.finalize()))
    }

    /// Read and validate a roster during the trusted enrollment/pinning ceremony.
    /// Runtime callers must use `read_pinned` with an independently supplied pin.
    pub fn read_for_pinning(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|_| "trusted query roster read")?;
        if bytes.is_empty() || bytes.len() > 64 * 1024 {
            return Err("trusted query roster size".into());
        }
        let roster: Self =
            serde_json::from_slice(&bytes).map_err(|_| "trusted query roster encoding")?;
        roster.validate()?;
        Ok(roster)
    }

    /// The expected digest is an independent launch/deployment input. Reading a
    /// coordinator-provided roster without this pin is deliberately unsupported.
    pub fn read_pinned(path: &Path, expected_sha512: &str) -> Result<Self, String> {
        canonical_hex(expected_sha512, 64, "invalid trusted roster SHA-512 pin")?;
        let roster = Self::read_for_pinning(path)?;
        if roster.sha512()? != expected_sha512 {
            return Err("trusted query roster pin mismatch".into());
        }
        Ok(roster)
    }
}

fn signer_from_material(material: &[u8]) -> Result<HybridSigner, String> {
    if material.len() != OWNER_SEED_BYTES {
        return Err("owner query key size".into());
    }
    let classical: &[u8; 32] = material[..32]
        .try_into()
        .map_err(|_| "owner classical seed size")?;
    let pq: &[u8; 32] = material[32..]
        .try_into()
        .map_err(|_| "owner PQ seed size")?;
    Ok(HybridSigner::new(
        Ed25519Signer::from_seed(classical),
        MlDsa65Signer::from_seed(pq),
    ))
}

fn sync_parent(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| "owner query directory sync".to_string())
}

fn ensure_private_directory(path: &Path, label: &'static str) -> Result<(), String> {
    if !path.exists() {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .map_err(|_| label.to_string())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            sync_parent(parent)?;
        }
    }
    let metadata = fs::metadata(path).map_err(|_| label.to_string())?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(label.into());
    }
    Ok(())
}

/// Generate a new owner-local key file. The returned enrollment contains only
/// public bytes and can be transferred to the trusted roster administrator.
pub fn generate_owner_key_file(path: &Path, party: usize) -> Result<OwnerEnrollment, String> {
    if party >= OWNER_COUNT {
        return Err("invalid owner party".into());
    }
    let parent = path.parent().ok_or("owner query key parent")?;
    ensure_private_directory(parent, "owner query key directory must be private")?;
    let mut material = Zeroizing::new([0u8; OWNER_SEED_BYTES]);
    OsRng.fill_bytes(material.as_mut());
    let signer = signer_from_material(material.as_ref())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "fresh owner query key required")?;
    file.write_all(OWNER_KEY_MAGIC)
        .and_then(|_| file.write_all(material.as_ref()))
        .and_then(|_| file.sync_all())
        .map_err(|_| "owner query key persistence")?;
    sync_parent(parent)?;
    Ok(OwnerEnrollment {
        party,
        public_key: hex::encode(signer.public_key()),
    })
}

pub struct OwnerIdentity {
    party: usize,
    signer: HybridSigner,
    roster_sha512: String,
    state_root: PathBuf,
}

impl OwnerIdentity {
    /// Import an owner-local key after independently checking the pinned roster.
    pub fn load(
        path: &Path,
        party: usize,
        roster: &TrustedRoster,
        roster_sha512: &str,
    ) -> Result<Self, String> {
        if party >= OWNER_COUNT || roster.sha512()? != roster_sha512 {
            return Err("owner trusted roster mismatch".into());
        }
        let path = fs::canonicalize(path).map_err(|_| "owner query key path")?;
        let state_root = path.parent().ok_or("owner query key parent")?.to_path_buf();
        ensure_private_directory(&state_root, "owner query key directory must be private")?;
        let metadata = fs::metadata(&path).map_err(|_| "owner query key metadata")?;
        if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
            return Err("owner query key permissions".into());
        }
        let bytes = Zeroizing::new(fs::read(&path).map_err(|_| "owner query key read")?);
        if bytes.len() != OWNER_KEY_MAGIC.len() + OWNER_SEED_BYTES
            || &bytes[..OWNER_KEY_MAGIC.len()] != OWNER_KEY_MAGIC
        {
            return Err("owner query key encoding".into());
        }
        let signer = signer_from_material(&bytes[OWNER_KEY_MAGIC.len()..])?;
        if hex::encode(signer.public_key()) != roster.owners[party].public_key {
            return Err("owner private key does not match pinned roster".into());
        }
        Ok(Self {
            party,
            signer,
            roster_sha512: roster_sha512.into(),
            state_root,
        })
    }

    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    pub fn enrollment(&self) -> OwnerEnrollment {
        OwnerEnrollment {
            party: self.party,
            public_key: hex::encode(self.signer.public_key()),
        }
    }

    pub fn approve(&self, query_digest_sha512: &str) -> Result<OwnerApproval, String> {
        let message = approval_message(query_digest_sha512)?;
        let signature = self
            .signer
            .sign(KeyPurpose::AuditCheckpoint, &message)
            .map_err(|_| "hybrid owner query signature")?;
        Ok(OwnerApproval {
            party: self.party,
            roster_sha512: self.roster_sha512.clone(),
            query_digest_sha512: query_digest_sha512.into(),
            signature: hex::encode(signature),
        })
    }
}

#[derive(Clone)]
pub struct QueryAgreementContext {
    pub statement: Statement,
    pub roots: Vec<NodeRoots>,
    pub kind: usize,
    pub claim: String,
    pub mask_claim: String,
    pub sumcheck: String,
    pub aggregation_challenge: String,
    pub fold_challenge: String,
    pub transcript_before_rows_sha512: String,
}

fn canonical_field_vector(
    value: &str,
    count: usize,
    label: &'static str,
) -> Result<Vec<F>, String> {
    let fields = unpack(value, count).map_err(|_| label.to_string())?;
    if pack(&fields) != value {
        return Err(label.into());
    }
    Ok(fields)
}

fn absorb_statement(hash: &mut Sha512, statement: &Statement) -> Result<(), String> {
    absorb(
        hash,
        b"statement",
        &serde_json::to_vec(statement).map_err(|_| "query agreement statement encoding")?,
    );
    Ok(())
}

/// Digest the complete ordered query view. Callers must first run the normal PCS
/// row parser; this function additionally binds the exact row and index bytes.
pub fn query_digest(
    context: &QueryAgreementContext,
    final_rows: &[String],
    indices: &[usize],
    transcript_after_queries: &Hash,
) -> Result<String, String> {
    context
        .statement
        .validate_query_agreement(&context.statement.query_roster_sha512)?;
    if context.kind >= 8
        || !context.kind.is_multiple_of(2)
        || context.roots.len() != OWNER_COUNT
        || context.roots.iter().any(|row| row.roots.len() != 8)
        || final_rows.len() != OWNER_COUNT
        || indices.len() != QUERIES
        || indices.iter().any(|index| *index >= CODE / 2)
    {
        return Err("invalid complete query agreement view".into());
    }
    canonical_hex(
        &context.statement.query_roster_sha512,
        64,
        "invalid query roster digest",
    )?;
    canonical_hex(
        &context.transcript_before_rows_sha512,
        64,
        "invalid pre-row transcript digest",
    )?;
    canonical_field_vector(&context.claim, 1, "invalid query claim")?;
    canonical_field_vector(&context.mask_claim, 1, "invalid query mask claim")?;
    canonical_field_vector(&context.sumcheck, 3, "invalid query sumcheck")?;
    canonical_field_vector(
        &context.aggregation_challenge,
        1,
        "invalid query aggregation challenge",
    )?;
    canonical_field_vector(&context.fold_challenge, 1, "invalid query fold challenge")?;

    let mut hash = Sha512::new();
    absorb(&mut hash, b"domain", b"ZKFMI:COCODE:COMPLETE-QUERY-VIEW:v2");
    absorb(&mut hash, b"protocol", QUERY_AGREEMENT_PROTOCOL.as_bytes());
    absorb(&mut hash, b"hybrid-suite", HYBRID_SUITE.as_bytes());
    absorb(
        &mut hash,
        b"hybrid-suite-id",
        &Suite::new(SuiteId::Ed25519MlDsa65).encode(),
    );
    absorb(
        &mut hash,
        b"roster-sha512",
        &canonical_hex(
            &context.statement.query_roster_sha512,
            64,
            "invalid query roster digest",
        )?,
    );
    absorb_statement(&mut hash, &context.statement)?;
    absorb(
        &mut hash,
        b"session-deployment",
        context.statement.settlement.deployment_id.as_bytes(),
    );
    absorb(
        &mut hash,
        b"session-book",
        context.statement.settlement.book_id.as_bytes(),
    );
    absorb(
        &mut hash,
        b"session-operation",
        context.statement.settlement.operation_id.as_bytes(),
    );
    absorb(
        &mut hash,
        b"session-sequence",
        &context.statement.settlement.sequence.to_le_bytes(),
    );
    for (party, row) in context.roots.iter().enumerate() {
        absorb(&mut hash, b"root-party", &(party as u64).to_le_bytes());
        for (kind, root) in row.roots.iter().enumerate() {
            absorb(&mut hash, b"root-kind", &(kind as u64).to_le_bytes());
            absorb(
                &mut hash,
                b"root",
                &canonical_hex(root, 64, "invalid query commitment root")?,
            );
        }
    }
    absorb(&mut hash, b"pcs-kind", &(context.kind as u64).to_le_bytes());
    for (label, value, count) in [
        (b"claim".as_slice(), context.claim.as_str(), 1usize),
        (
            b"mask-claim".as_slice(),
            context.mask_claim.as_str(),
            1usize,
        ),
        (b"sumcheck".as_slice(), context.sumcheck.as_str(), 3usize),
        (
            b"aggregation-challenge".as_slice(),
            context.aggregation_challenge.as_str(),
            1usize,
        ),
        (
            b"fold-challenge".as_slice(),
            context.fold_challenge.as_str(),
            1usize,
        ),
    ] {
        for field in canonical_field_vector(value, count, "invalid query field vector")? {
            absorb(&mut hash, label, &encode(field));
        }
    }
    absorb(
        &mut hash,
        b"transcript-before-rows",
        &canonical_hex(
            &context.transcript_before_rows_sha512,
            64,
            "invalid pre-row transcript digest",
        )?,
    );
    for (party, row) in final_rows.iter().enumerate() {
        absorb(&mut hash, b"final-row-party", &(party as u64).to_le_bytes());
        absorb(&mut hash, b"final-row", row.as_bytes());
    }
    for (position, index) in indices.iter().enumerate() {
        absorb(
            &mut hash,
            b"query-position",
            &(position as u64).to_le_bytes(),
        );
        absorb(&mut hash, b"query-index", &(*index as u64).to_le_bytes());
    }
    absorb(
        &mut hash,
        b"transcript-after-queries",
        transcript_after_queries,
    );
    Ok(hex::encode(hash.finalize()))
}

fn approval_message(query_digest_sha512: &str) -> Result<Vec<u8>, String> {
    let digest = canonical_hex(query_digest_sha512, 64, "invalid complete query digest")?;
    let mut message = b"ZKFMI:COCODE:OWNER-QUERY-APPROVAL:v2".to_vec();
    message.extend_from_slice(&Suite::new(SuiteId::Ed25519MlDsa65).encode());
    message.extend_from_slice(&digest);
    Ok(message)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerApproval {
    pub party: usize,
    pub roster_sha512: String,
    pub query_digest_sha512: String,
    pub signature: String,
}

pub fn verify_all_approvals(
    roster: &TrustedRoster,
    expected_roster_sha512: &str,
    expected_query_digest_sha512: &str,
    approvals: &[OwnerApproval],
) -> Result<(), String> {
    if roster.sha512()? != expected_roster_sha512
        || approvals.len() != OWNER_COUNT
        || canonical_hex(
            expected_query_digest_sha512,
            64,
            "invalid expected query digest",
        )
        .is_err()
    {
        return Err("query approval roster, count, or digest mismatch".into());
    }
    let message = approval_message(expected_query_digest_sha512)?;
    let verifier = HybridVerifier;
    for (party, approval) in approvals.iter().enumerate() {
        if approval.party != party
            || approval.roster_sha512 != expected_roster_sha512
            || approval.query_digest_sha512 != expected_query_digest_sha512
        {
            return Err("missing, duplicate, or mismatched owner approval".into());
        }
        let public_key = canonical_hex(
            &roster.owners[party].public_key,
            HYBRID_PUBLIC_KEY_BYTES,
            "invalid owner approval public key",
        )?;
        let signature = canonical_hex(
            &approval.signature,
            HYBRID_SIGNATURE_BYTES,
            "invalid hybrid owner approval",
        )?;
        verifier
            .verify(
                KeyPurpose::AuditCheckpoint,
                &public_key,
                &message,
                &signature,
            )
            .map_err(|_| "hybrid owner approval verification")?;
    }
    Ok(())
}

fn session_marker_id(
    statement: &Statement,
    roster_sha512: &str,
    kind: usize,
) -> Result<String, String> {
    statement.validate_query_agreement(roster_sha512)?;
    let mut hash = Sha512::new();
    absorb(&mut hash, b"domain", b"ZKFMI:COCODE:QUERY-SESSION-KIND:v2");
    absorb(&mut hash, b"protocol", statement.protocol.as_bytes());
    absorb(
        &mut hash,
        b"roster-sha512",
        &canonical_hex(roster_sha512, 64, "invalid query roster digest")?,
    );
    // The durable budget is keyed by the externally assigned logical session,
    // not mutable commitment roots. Recommitting after a restart must not buy a
    // second query view for the same operation and sequence.
    absorb(
        &mut hash,
        b"session-deployment",
        statement.settlement.deployment_id.as_bytes(),
    );
    absorb(
        &mut hash,
        b"session-book",
        statement.settlement.book_id.as_bytes(),
    );
    absorb(
        &mut hash,
        b"session-operation",
        statement.settlement.operation_id.as_bytes(),
    );
    absorb(
        &mut hash,
        b"session-sequence",
        &statement.settlement.sequence.to_le_bytes(),
    );
    absorb(&mut hash, b"kind", &(kind as u64).to_le_bytes());
    Ok(hex::encode(hash.finalize()))
}

fn create_marker(path: &Path, body: &[u8], label: &'static str) -> Result<(), String> {
    let mut marker = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| label.to_string())?;
    marker
        .write_all(body)
        .and_then(|_| marker.sync_all())
        .map_err(|_| label.to_string())?;
    sync_parent(path.parent().ok_or("query marker parent")?)
}

pub struct DurableQueryBudget {
    directory: PathBuf,
    stem: String,
    query_digest_sha512: String,
}

impl DurableQueryBudget {
    #[cfg(feature = "pqc-note-circuit")]
    pub(crate) fn reserve_note(
        owner_root: &Path,
        party: usize,
        session_sha512: &str,
        roster_sha512: &str,
        kind: usize,
        query_digest_sha512: &str,
    ) -> Result<Self, String> {
        if party >= OWNER_COUNT || ![0, 2].contains(&kind) {
            return Err("note query owner or kind".into());
        }
        let session = canonical_hex(session_sha512, 64, "note query session digest")?;
        let roster = canonical_hex(roster_sha512, 64, "note query roster digest")?;
        canonical_hex(query_digest_sha512, 64, "note query digest")?;
        ensure_private_directory(owner_root, "note owner state must be private")?;
        let directory = owner_root.join("query-agreement-note-v1");
        ensure_private_directory(&directory, "note query state must be private")?;
        let directory = directory.join(format!("P{party}"));
        ensure_private_directory(&directory, "note query owner state must be private")?;
        let mut hash = Sha512::new();
        absorb(&mut hash, b"domain", b"ZKFMI:NOTE:QUERY-LIFETIME:v1");
        absorb(&mut hash, b"session", &session);
        absorb(&mut hash, b"roster", &roster);
        absorb(&mut hash, b"kind", &(kind as u64).to_le_bytes());
        let stem = hex::encode(hash.finalize());
        if directory.join(format!("{stem}.consumed")).exists() {
            return Err("note query lifetime budget consumed".into());
        }
        create_marker(
            &directory.join(format!("{stem}.prepared")),
            format!("{query_digest_sha512}\n").as_bytes(),
            "note query lifetime budget already prepared",
        )?;
        Ok(Self {
            directory,
            stem,
            query_digest_sha512: query_digest_sha512.into(),
        })
    }

    /// Reserve the session/kind before signing. Any failed or interrupted
    /// attempt remains consumed for privacy: callers must start a fresh session.
    pub fn reserve(
        owner_root: &Path,
        party: usize,
        statement: &Statement,
        roster_sha512: &str,
        kind: usize,
        query_digest_sha512: &str,
    ) -> Result<Self, String> {
        if party >= OWNER_COUNT || kind >= 8 || !kind.is_multiple_of(2) {
            return Err("invalid owner query budget identity".into());
        }
        statement.validate_query_agreement(roster_sha512)?;
        canonical_hex(query_digest_sha512, 64, "invalid owner query budget digest")?;
        ensure_private_directory(owner_root, "owner query state root must be private")?;
        let agreement_directory = owner_root.join("query-agreement-v2");
        ensure_private_directory(
            &agreement_directory,
            "owner query agreement directory must be private",
        )?;
        let directory = agreement_directory.join(format!("P{party}"));
        ensure_private_directory(&directory, "owner query budget directory must be private")?;
        let stem = session_marker_id(statement, roster_sha512, kind)?;
        let consumed = directory.join(format!("{stem}.consumed"));
        if consumed.exists() {
            return Err("owner query session/kind budget already consumed".into());
        }
        create_marker(
            &directory.join(format!("{stem}.prepared")),
            format!("{query_digest_sha512}\n").as_bytes(),
            "owner query session/kind budget already prepared",
        )?;
        Ok(Self {
            directory,
            stem,
            query_digest_sha512: query_digest_sha512.into(),
        })
    }

    /// Persist final consumption after all seven approvals verify and before an
    /// opening is computed or returned.
    pub fn consume(&self, approvals: &[OwnerApproval]) -> Result<(), String> {
        let prepared = self.directory.join(format!("{}.prepared", self.stem));
        if fs::read_to_string(prepared).map_err(|_| "owner prepared query marker read")?
            != format!("{}\n", self.query_digest_sha512)
        {
            return Err("owner prepared query marker mismatch".into());
        }
        let certificate =
            serde_json::to_vec(approvals).map_err(|_| "owner approval certificate encoding")?;
        let mut hash = Sha512::new();
        absorb(&mut hash, b"domain", b"ZKFMI:COCODE:QUERY-CERTIFICATE:v2");
        absorb(&mut hash, b"approvals", &certificate);
        let body = format!(
            "{}\n{}\n",
            self.query_digest_sha512,
            hex::encode(hash.finalize())
        );
        create_marker(
            &self.directory.join(format!("{}.consumed", self.stem)),
            body.as_bytes(),
            "owner query session/kind budget already consumed",
        )
    }
}

pub fn query_context(
    statement: &Statement,
    roots: &[NodeRoots],
    kind: usize,
    claims: [F; 2],
    sumcheck: &[F],
    challenges: [F; 2],
    transcript_before_rows: &Hash,
) -> Result<QueryAgreementContext, String> {
    if sumcheck.len() != 3 {
        return Err("query agreement sumcheck length".into());
    }
    Ok(QueryAgreementContext {
        statement: statement.clone(),
        roots: roots.to_vec(),
        kind,
        claim: pack(&[claims[0]]),
        mask_claim: pack(&[claims[1]]),
        sumcheck: pack(sumcheck),
        aggregation_challenge: pack(&[challenges[0]]),
        fold_challenge: pack(&[challenges[1]]),
        transcript_before_rows_sha512: hex::encode(transcript_before_rows),
    })
}

pub fn validate_settlement_session(statement: &SettlementStatement) -> Result<(), String> {
    statement.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{integration::SettlementStatement, pcs::NodeRoots};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn signer(party: usize) -> HybridSigner {
        HybridSigner::new(
            Ed25519Signer::from_seed(&[(party + 1) as u8; 32]),
            MlDsa65Signer::from_seed(&[(party + 17) as u8; 32]),
        )
    }

    fn roster() -> TrustedRoster {
        TrustedRoster {
            protocol: QUERY_AGREEMENT_PROTOCOL.into(),
            suite: HYBRID_SUITE.into(),
            roster_id: "deterministic-seven-owner-test".into(),
            owners: (0..OWNER_COUNT)
                .map(|party| OwnerEnrollment {
                    party,
                    public_key: hex::encode(signer(party).public_key()),
                })
                .collect(),
        }
    }

    fn statement(roster_sha512: &str) -> Statement {
        Statement::new_query_agreement(
            SettlementStatement {
                deployment_id: "query-agreement-test".into(),
                book_id: "book".into(),
                operation_id: "operation".into(),
                sequence: 1,
                before_commitment: "11".repeat(64),
                after_commitment: "22".repeat(64),
                no_fill: false,
            },
            roster_sha512,
        )
        .unwrap()
    }

    fn context(roster_sha512: &str) -> QueryAgreementContext {
        query_context(
            &statement(roster_sha512),
            &(0..OWNER_COUNT)
                .map(|party| NodeRoots {
                    roots: (0..8)
                        .map(|kind| format!("{:0128x}", 1 + party * 8 + kind))
                        .collect(),
                })
                .collect::<Vec<_>>(),
            2,
            [F::from(3u64), F::from(4u64)],
            &[F::from(5u64), F::from(6u64), F::from(7u64)],
            [F::from(8u64), F::from(9u64)],
            &[10u8; 64],
        )
        .unwrap()
    }

    fn approvals(roster: &TrustedRoster, digest: &str) -> Vec<OwnerApproval> {
        let roster_sha512 = roster.sha512().unwrap();
        (0..OWNER_COUNT)
            .map(|party| {
                let message = approval_message(digest).unwrap();
                OwnerApproval {
                    party,
                    roster_sha512: roster_sha512.clone(),
                    query_digest_sha512: digest.into(),
                    signature: hex::encode(
                        signer(party)
                            .sign(KeyPurpose::AuditCheckpoint, &message)
                            .unwrap(),
                    ),
                }
            })
            .collect()
    }

    fn flip_signature_byte(approval: &mut OwnerApproval, byte: usize) {
        let mut signature = hex::decode(&approval.signature).unwrap();
        signature[byte] ^= 1;
        approval.signature = hex::encode(signature);
    }

    #[test]
    fn seven_distinct_hybrid_approvals_are_mandatory() {
        let roster = roster();
        let roster_sha512 = roster.sha512().unwrap();
        let context = context(&roster_sha512);
        let rows: Vec<String> = (0..OWNER_COUNT)
            .map(|party| format!("row-{party}"))
            .collect();
        let indices: Vec<usize> = (0..QUERIES).map(|index| index % (CODE / 2)).collect();
        let digest = query_digest(&context, &rows, &indices, &[12u8; 64]).unwrap();
        let signed = approvals(&roster, &digest);
        verify_all_approvals(&roster, &roster_sha512, &digest, &signed).unwrap();

        let mut missing = signed.clone();
        missing.pop();
        assert!(verify_all_approvals(&roster, &roster_sha512, &digest, &missing).is_err());

        let mut duplicate = signed.clone();
        duplicate[6] = duplicate[5].clone();
        assert!(verify_all_approvals(&roster, &roster_sha512, &digest, &duplicate).is_err());

        let mut bad_classical = signed.clone();
        flip_signature_byte(&mut bad_classical[3], 0);
        assert!(verify_all_approvals(&roster, &roster_sha512, &digest, &bad_classical).is_err());

        let mut bad_pq = signed.clone();
        flip_signature_byte(&mut bad_pq[3], 64);
        assert!(verify_all_approvals(&roster, &roster_sha512, &digest, &bad_pq).is_err());
    }

    #[test]
    fn any_non_own_row_or_query_equivocation_changes_the_signed_digest() {
        let roster = roster();
        let roster_sha512 = roster.sha512().unwrap();
        let context = context(&roster_sha512);
        let rows: Vec<String> = (0..OWNER_COUNT)
            .map(|party| format!("row-{party}"))
            .collect();
        let indices: Vec<usize> = (0..QUERIES).map(|index| index % (CODE / 2)).collect();
        let expected = query_digest(&context, &rows, &indices, &[12u8; 64]).unwrap();

        let mut other_row = rows.clone();
        other_row[4].push_str("-equivocated");
        assert_ne!(
            expected,
            query_digest(&context, &other_row, &indices, &[12u8; 64]).unwrap()
        );

        let mut other_query = indices.clone();
        other_query[0] += 1;
        assert_ne!(
            expected,
            query_digest(&context, &rows, &other_query, &[12u8; 64]).unwrap()
        );
    }

    #[test]
    fn every_security_relevant_query_context_component_is_digest_bound() {
        let roster = roster();
        let roster_sha512 = roster.sha512().unwrap();
        let context = context(&roster_sha512);
        let rows: Vec<String> = (0..OWNER_COUNT)
            .map(|party| format!("row-{party}"))
            .collect();
        let indices: Vec<usize> = (0..QUERIES).map(|index| index % (CODE / 2)).collect();
        let after = [12u8; 64];
        let expected = query_digest(&context, &rows, &indices, &after).unwrap();
        let changed = |candidate: &QueryAgreementContext| {
            query_digest(candidate, &rows, &indices, &after).unwrap()
        };

        let mut candidate = context.clone();
        candidate.statement.settlement.operation_id = "other-operation".into();
        assert_ne!(expected, changed(&candidate));

        let mut candidate = context.clone();
        candidate.roots[0].roots[1] = "ff".repeat(64);
        assert_ne!(expected, changed(&candidate));

        let mut candidate = context.clone();
        candidate.kind = 4;
        assert_ne!(expected, changed(&candidate));

        for mutate in [
            |candidate: &mut QueryAgreementContext| candidate.claim = pack(&[F::from(13u64)]),
            |candidate: &mut QueryAgreementContext| candidate.mask_claim = pack(&[F::from(14u64)]),
            |candidate: &mut QueryAgreementContext| {
                candidate.sumcheck = pack(&[F::from(15u64), F::from(6u64), F::from(7u64)])
            },
            |candidate: &mut QueryAgreementContext| {
                candidate.aggregation_challenge = pack(&[F::from(16u64)])
            },
            |candidate: &mut QueryAgreementContext| {
                candidate.fold_challenge = pack(&[F::from(17u64)])
            },
            |candidate: &mut QueryAgreementContext| {
                candidate.transcript_before_rows_sha512 = "aa".repeat(64)
            },
        ] {
            let mut candidate = context.clone();
            mutate(&mut candidate);
            assert_ne!(expected, changed(&candidate));
        }

        assert_ne!(
            expected,
            query_digest(&context, &rows, &indices, &[18u8; 64]).unwrap()
        );
    }

    #[test]
    fn roster_pin_and_owner_keys_fail_closed() {
        let roster = roster();
        let digest = roster.sha512().unwrap();
        let mut duplicate = roster.clone();
        duplicate.owners[6].public_key = duplicate.owners[5].public_key.clone();
        assert!(duplicate.validate().is_err());

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "zkfmi-cocode-query-roster-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("roster.json");
        fs::write(&path, serde_json::to_vec(&roster).unwrap()).unwrap();
        assert_eq!(TrustedRoster::read_for_pinning(&path).unwrap(), roster);
        assert_eq!(TrustedRoster::read_pinned(&path, &digest).unwrap(), roster);
        assert!(TrustedRoster::read_pinned(&path, &"00".repeat(64)).is_err());

        let owner_directory = directory.join("owner-0-private");
        fs::create_dir(&owner_directory).unwrap();
        fs::set_permissions(&owner_directory, fs::Permissions::from_mode(0o700)).unwrap();
        let owner_key = owner_directory.join("query-approval.key");
        let enrollment = generate_owner_key_file(&owner_key, 0).unwrap();
        let mut enrolled_roster = roster.clone();
        enrolled_roster.owners[0] = enrollment.clone();
        let enrolled_digest = enrolled_roster.sha512().unwrap();
        let identity =
            OwnerIdentity::load(&owner_key, 0, &enrolled_roster, &enrolled_digest).unwrap();
        assert_eq!(identity.enrollment(), enrollment);
        assert_eq!(identity.state_root(), owner_directory.as_path());
        assert!(OwnerIdentity::load(&owner_key, 1, &enrolled_roster, &enrolled_digest).is_err());

        let insecure_directory = directory.join("insecure-owner");
        fs::create_dir(&insecure_directory).unwrap();
        fs::set_permissions(&insecure_directory, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(generate_owner_key_file(&insecure_directory.join("key"), 0).is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn prepared_and_consumed_markers_reject_replay_and_restart() {
        let roster = roster();
        let roster_sha512 = roster.sha512().unwrap();
        let statement = statement(&roster_sha512);
        let digest = "33".repeat(64);
        let signed = approvals(&roster, &digest);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zkfmi-cocode-query-budget-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();

        let budget =
            DurableQueryBudget::reserve(&root, 0, &statement, &roster_sha512, 2, &digest).unwrap();
        assert!(
            DurableQueryBudget::reserve(&root, 0, &statement, &roster_sha512, 2, &digest,).is_err()
        );
        budget.consume(&signed).unwrap();
        assert!(budget.consume(&signed).is_err());

        let mut recommitted = statement.clone();
        recommitted.settlement.before_commitment = "44".repeat(64);
        recommitted.settlement.after_commitment = "55".repeat(64);
        assert_eq!(
            session_marker_id(&statement, &roster_sha512, 2).unwrap(),
            session_marker_id(&recommitted, &roster_sha512, 2).unwrap()
        );
        assert!(DurableQueryBudget::reserve(
            &root,
            0,
            &recommitted,
            &roster_sha512,
            2,
            &"66".repeat(64),
        )
        .is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
