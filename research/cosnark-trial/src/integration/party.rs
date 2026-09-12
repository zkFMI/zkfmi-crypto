//! Private per-party worker. The coordinator receives roots, masked final
//! codewords and authenticated queries, never this worker's original shares.
use crate::{
    algebra::{normalized_rs_fold, pack, unpack},
    field::{decode, domain, interpolate_row, F},
    integration::{
        circuit::{Statement, LEGACY_PROTOCOL, QUERY_AGREEMENT_PROTOCOL},
        mpc_program::STATE_SIZE,
        owner_input::{OwnerInputBinding, PrivateOwnerInput},
        query_agreement::{
            self, DurableQueryBudget, OwnerApproval, OwnerIdentity, QueryAgreementContext,
            TrustedRoster,
        },
        verifier::{self, ConstraintPrefix},
    },
    pcs::{self, NodeRoots, PairOpening, PcsProof, CODE, PADDED},
    transcript::{Merkle, Transcript},
};
use ark_ff::{BigInteger, FftField, Field, PrimeField, UniformRand};
use ark_poly::EvaluationDomain;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum Request {
    BindStatement {
        statement: Statement,
    },
    Input {
        view: String,
        initial: bool,
        invalid: bool,
    },
    Commit,
    Fold {
        kind: usize,
        authorization: Box<Authorization>,
    },
    Query {
        kind: usize,
        indices: Vec<usize>,
        final_rows: Vec<String>,
    },
    PrepareQuery {
        kind: usize,
        indices: Vec<usize>,
        final_rows: Vec<String>,
    },
    QueryWithAgreement {
        kind: usize,
        approvals: Vec<OwnerApproval>,
    },
    Shutdown,
}

#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub prefix: ConstraintPrefix,
    pub prior_mask_opening: Option<PcsProof>,
    pub prior_book_openings: Vec<PcsProof>,
    pub mask_claim: String,
    pub sumcheck: String,
}

struct AuthorizedOpening {
    cursor: Transcript,
    aggregation_challenge: F,
    fold_challenge: F,
    claim: F,
    mask_claim: F,
    sumcheck: Vec<F>,
}

fn authorize(
    statement: &Statement,
    oracles: &[Merkle],
    party: usize,
    kind: usize,
    auth: &Authorization,
) -> Result<AuthorizedOpening, String> {
    if &auth.prefix.statement != statement
        || auth.prefix.roots.len() != 7
        || auth.prefix.roots[party].roots != oracles.iter().map(Merkle::root).collect::<Vec<_>>()
    {
        return Err("owner statement or commitment view changed".into());
    }
    let (mut t, _, claim) = verifier::opening_context(
        &auth.prefix,
        auth.prior_mask_opening.as_ref(),
        &auth.prior_book_openings,
        kind,
    )?;
    t.absorb(b"pcs-kind", &(kind as u64).to_le_bytes());
    t.absorb_fields(b"pcs-claim", &[claim]);
    let yr = unpack(&auth.mask_claim, 1)?[0];
    t.absorb_fields(b"pcs-mask-claim", &[yr]);
    let b = t.field(b"pcs-aggregation");
    let h = unpack(&auth.sumcheck, 3)?;
    if h[0] + h[1] != claim + b * yr {
        return Err("owner PCS sumcheck identity".into());
    }
    t.absorb_fields(b"pcs-h", &h);
    let r = t.field(b"pcs-fold");
    if b == F::from(0u64) {
        return Err("zero masking challenge requires a fresh proof session".into());
    }
    Ok(AuthorizedOpening {
        cursor: t,
        aggregation_challenge: b,
        fold_challenge: r,
        claim,
        mask_claim: yr,
        sumcheck: h,
    })
}

struct QueryAgreementOwner {
    roster: TrustedRoster,
    identity: OwnerIdentity,
    roster_sha512: String,
    state_root: PathBuf,
}

struct PreparedQuery {
    digest_sha512: String,
    indices: Vec<usize>,
    budget: DurableQueryBudget,
}

fn persist_book_query_marker(
    root: &Path,
    party: usize,
    kind: usize,
    body: &[u8],
) -> Result<(), String> {
    if kind != 4 && kind != 6 {
        return Ok(());
    }
    let path = root.join(format!("book-query-used-{kind}-P{party}"));
    let mut marker = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "owner book query lifetime budget consumed")?;
    marker
        .write_all(body)
        .and_then(|_| marker.sync_all())
        .map_err(|_| "owner query receipt")?;
    fs::File::open(root)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| "owner query receipt directory sync".to_string())
}

fn context_values(view: &str) -> Result<Vec<u64>, String> {
    let bytes = hex::decode(view).map_err(|_| "context hex")?;
    if bytes.len() != 64 {
        return Err("context length".into());
    }
    Ok(bytes
        .chunks_exact(8)
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .collect())
}

fn write_input(
    root: &Path,
    party: usize,
    view: &str,
    initial: bool,
    invalid: bool,
    owner_input: Option<&PrivateOwnerInput>,
) -> Result<(), String> {
    if owner_input.is_some() && invalid {
        return Err("fixture fault injection is forbidden for owner inputs".into());
    }
    let path = root
        .join("Player-Data")
        .join(format!("CosnarkInput-P{party}-0"));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "owner input file")?;
    for value in context_values(view)? {
        writeln!(file, "{value}").map_err(|_| "write public context")?;
    }
    if initial {
        // Each worker generates its own input contribution and independent
        // entropy locally. Neither appears in a coordinator request/response.
        if let Some(input) = owner_input {
            for value in input.fields()? {
                writeln!(file, "{value}").map_err(|_| "write private contribution")?;
            }
        } else {
            for _ in 0..2 {
                writeln!(file, "{}", 1 + OsRng.next_u32() % 15).map_err(|_| "write owner input")?;
            }
            writeln!(file, "{}", usize::from(invalid && party == 0))
                .map_err(|_| "write owner fixture flag")?;
        }
        for _ in 0..8 * PADDED {
            writeln!(file, "{}", F::rand(&mut OsRng)).map_err(|_| "write owner entropy")?;
        }
    }
    file.sync_all().map_err(|_| "sync owner input".to_string())
}

fn load_own_oracles(root: &Path, party: usize) -> Result<Vec<Merkle>, String> {
    let path = root
        .join("Persistence")
        .join(format!("Transactions-P{party}.data"));
    let saved = qomm_mpc::persistence::read(&path, party).map_err(|_| "owner persistence parse")?;
    if saved.shares.len() != STATE_SIZE
        || saved.element_bytes != 32
        || saved.prime.to_bytes_be() != F::MODULUS.to_bytes_be()
    {
        return Err("owner persistence field or shape mismatch".into());
    }
    let conversion = if saved.montgomery {
        F::from(2u64).pow([256u64]).inverse().unwrap()
    } else {
        F::from(1u64)
    };
    let mut oracles = Vec::new();
    for kind in 0..8 {
        let row = saved.shares[kind * PADDED..(kind + 1) * PADDED]
            .iter()
            .map(|v| {
                let bytes = v.to_bytes_le(32).map_err(|_| "owner share width")?;
                Ok(decode(bytes.try_into().unwrap()).ok_or("owner share encoding")? * conversion)
            })
            .collect::<Result<Vec<F>, String>>()?;
        let coefficients = interpolate_row(&row).map_err(str::to_owned)?;
        // Query points must not include the interpolation points containing the
        // secret message. A full multiplicative-generator coset is disjoint
        // from the CODE-point subgroup (and hence from its PADDED subgroup).
        if F::GENERATOR.pow([CODE as u64]) == F::from(1u64) {
            return Err("query and secret-message domains overlap".into());
        }
        let d = domain(CODE)
            .map_err(str::to_owned)?
            .get_coset(F::GENERATOR)
            .ok_or("invalid oracle coset")?;
        let mut coefficients = coefficients;
        coefficients.resize(CODE, F::from(0u64));
        let codeword = d.fft(&coefficients);
        let prior_salts = root.join(format!("book-prior-salts-P{party}"));
        let oracle = if kind == 4 && prior_salts.exists() {
            let bytes = fs::read(&prior_salts).map_err(|_| "owner prior salts")?;
            if bytes.len() != CODE * 64 {
                return Err("owner prior salt shape".into());
            }
            Merkle::with_salts(
                pcs::oracle_domain(kind, party),
                codeword,
                bytes
                    .chunks_exact(64)
                    .map(|s| s.try_into().unwrap())
                    .collect(),
            )
        } else {
            Merkle::new(pcs::oracle_domain(kind, party), codeword)
        };
        if kind == 6 {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(root.join(format!("book-next-salts-P{party}")))
                .map_err(|_| "fresh owner next salts")?;
            for salt in oracle.salts() {
                f.write_all(salt).map_err(|_| "owner salt persistence")?;
            }
            f.sync_all().map_err(|_| "owner salt sync")?;
        }
        oracles.push(oracle);
    }
    Ok(oracles)
}

pub fn serve(root: &Path, party: usize) -> Result<(), String> {
    serve_inner(root, party, None, None)
}

pub fn serve_query_agreement(
    root: &Path,
    party: usize,
    roster_path: &Path,
    expected_roster_sha512: &str,
    owner_key_path: &Path,
) -> Result<(), String> {
    let roster = TrustedRoster::read_pinned(roster_path, expected_roster_sha512)?;
    let identity = OwnerIdentity::load(owner_key_path, party, &roster, expected_roster_sha512)?;
    let state_root = identity.state_root().to_path_buf();
    serve_inner(
        root,
        party,
        Some(QueryAgreementOwner {
            roster,
            identity,
            roster_sha512: expected_roster_sha512.into(),
            state_root,
        }),
        None,
    )
}

/// The trusted launcher provides public binding and a path into this owner's
/// private boundary. No private file contents cross the coordinator channel.
pub fn serve_query_agreement_with_input(
    root: &Path,
    party: usize,
    roster_path: &Path,
    expected_roster_sha512: &str,
    owner_key_path: &Path,
    binding_path: &Path,
    input_path: &Path,
) -> Result<(), String> {
    let binding: OwnerInputBinding = serde_json::from_slice(
        &fs::read(binding_path).map_err(|_| "public owner binding missing")?,
    )
    .map_err(|_| "public owner binding encoding")?;
    let input = PrivateOwnerInput::load(input_path, party, &binding)?;
    let roster = TrustedRoster::read_pinned(roster_path, expected_roster_sha512)?;
    let identity = OwnerIdentity::load(owner_key_path, party, &roster, expected_roster_sha512)?;
    let state_root = identity.state_root().to_path_buf();
    serve_inner(
        root,
        party,
        Some(QueryAgreementOwner {
            roster,
            identity,
            roster_sha512: expected_roster_sha512.into(),
            state_root,
        }),
        Some(input),
    )
}

/// Laboratory key-file wrapper around the resident-key input delivery API.
/// Snapshot/packet authentication and durable consumption happen before the
/// worker processes any coordinator requests or writes native input shares.
#[allow(clippy::too_many_arguments)]
pub fn serve_query_agreement_with_sealed_input(
    root: &Path,
    party: usize,
    roster_path: &Path,
    expected_roster_sha512: &str,
    owner_key_path: &Path,
    binding_path: &Path,
    input_path: &Path,
    receiver: &super::input_delivery::PinnedReceiverLaunch,
) -> Result<(), String> {
    use super::input_delivery;
    let binding: OwnerInputBinding =
        serde_json::from_slice(&input_delivery::read_bounded(binding_path)?)
            .map_err(|_| "sealed input public binding encoding")?;
    let trusted = input_delivery::TrustedInputReceiver::read_pinned(
        &receiver.config_path,
        &receiver.config_sha256,
    )?;
    let key = input_delivery::load_lab_recipient(&receiver.recipient_key_path)?;
    let input = input_delivery::receive(
        &input_delivery::read_bounded(input_path)?,
        party,
        &binding,
        &trusted,
        &key,
        input_delivery::clock_now()?,
    )?;
    let roster = TrustedRoster::read_pinned(roster_path, expected_roster_sha512)?;
    let identity = OwnerIdentity::load(owner_key_path, party, &roster, expected_roster_sha512)?;
    let state_root = identity.state_root().to_path_buf();
    serve_inner(
        root,
        party,
        Some(QueryAgreementOwner {
            roster,
            identity,
            roster_sha512: expected_roster_sha512.into(),
            state_root,
        }),
        Some(input),
    )
}

fn serve_inner(
    root: &Path,
    party: usize,
    agreement_owner: Option<QueryAgreementOwner>,
    owner_input: Option<PrivateOwnerInput>,
) -> Result<(), String> {
    if party >= 7 {
        return Err("invalid private worker index".into());
    }
    fs::set_permissions(root.join("Persistence"), fs::Permissions::from_mode(0o700))
        .map_err(|_| "private persistence permissions")?;
    let mut oracles = Vec::new();
    let mut statement = None;
    let mut committed = false;
    let mut initialized = false;
    let mut folded = [false; 4];
    let mut queried = [false; 4];
    let mut cursors: [Option<Transcript>; 4] = [None, None, None, None];
    let mut own_final: [Option<String>; 4] = [None, None, None, None];
    let mut agreement_contexts: [Option<QueryAgreementContext>; 4] = std::array::from_fn(|_| None);
    let mut prepared_queries: [Option<PreparedQuery>; 4] = std::array::from_fn(|_| None);
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).map_err(|_| "worker input")? == 0 {
            break;
        }
        if line.len() > 512 * 1024 * 1024 {
            return Err("worker request too large".into());
        }
        let request: Request =
            serde_json::from_str(&line).map_err(|_| "worker request encoding")?;
        let answer = match request {
            Request::BindStatement { statement: given } => {
                if statement.is_some() || !committed {
                    return Err("owner statement is already bound".into());
                }
                match (&agreement_owner, given.protocol.as_str()) {
                    (None, LEGACY_PROTOCOL) => given.validate_legacy()?,
                    (Some(owner), QUERY_AGREEMENT_PROTOCOL) => {
                        given.validate_query_agreement(&owner.roster_sha512)?
                    }
                    _ => return Err("worker launch protocol does not match statement".into()),
                }
                if let Some(input) = &owner_input {
                    input.require_statement(&given)?;
                }
                statement = Some(given);
                serde_json::json!({"party":party,"bound":true})
            }
            Request::Input {
                view,
                initial,
                invalid,
            } => {
                if initial && initialized && owner_input.is_some() {
                    return Err("owner input initialization is one-shot".into());
                }
                write_input(root, party, &view, initial, invalid, owner_input.as_ref())?;
                initialized |= initial;
                serde_json::json!({"party":party,"view":view})
            }
            Request::Commit => {
                if committed {
                    return Err("owner commitment is one-shot".into());
                }
                oracles = load_own_oracles(root, party)?;
                committed = true;
                serde_json::to_value(NodeRoots {
                    roots: oracles.iter().map(Merkle::root).collect(),
                })
                .unwrap()
            }
            Request::Fold {
                kind,
                authorization,
            } => {
                if (kind >= 8 || !kind.is_multiple_of(2))
                    || oracles.len() != 8
                    || folded[kind / 2]
                    || (kind == 4 && !queried[1])
                    || (kind == 6 && !queried[2])
                    || (kind == 0 && !queried[3])
                {
                    return Err("worker fold is out of order or its mask was consumed".into());
                }
                let authorized = authorize(
                    statement.as_ref().ok_or("owner statement missing")?,
                    &oracles,
                    party,
                    kind,
                    &authorization,
                )?;
                let aggregate: Vec<F> = oracles[kind]
                    .values()
                    .iter()
                    .zip(oracles[kind + 1].values())
                    .map(|(v, m)| *v + authorized.aggregation_challenge * m)
                    .collect();
                let final_row = pack(&normalized_rs_fold(&aggregate, authorized.fold_challenge)?);
                let agreement_context = if agreement_owner.is_some() {
                    Some(query_agreement::query_context(
                        statement.as_ref().ok_or("owner statement missing")?,
                        &authorization.prefix.roots,
                        kind,
                        [authorized.claim, authorized.mask_claim],
                        &authorized.sumcheck,
                        [authorized.aggregation_challenge, authorized.fold_challenge],
                        &authorized.cursor.view(),
                    )?)
                } else {
                    None
                };
                folded[kind / 2] = true;
                cursors[kind / 2] = Some(authorized.cursor);
                own_final[kind / 2] = Some(final_row.clone());
                agreement_contexts[kind / 2] = agreement_context;
                serde_json::json!(final_row)
            }
            Request::Query {
                kind,
                indices,
                final_rows,
            } => {
                if agreement_owner.is_some() {
                    return Err("legacy query request forbidden for query-agreement worker".into());
                }
                if (kind >= 8 || !kind.is_multiple_of(2))
                    || oracles.len() != 8
                    || indices.len() != pcs::QUERIES
                    || indices.iter().any(|i| *i >= CODE / 2)
                    || !folded[kind / 2]
                    || queried[kind / 2]
                {
                    return Err(
                        "worker query budget was consumed or request is out of order".into(),
                    );
                }
                if final_rows.len() != 7 || own_final[kind / 2].as_ref() != Some(&final_rows[party])
                {
                    return Err("owner final-row view changed".into());
                }
                let mut cursor = cursors[kind / 2]
                    .take()
                    .ok_or("owner transcript cursor missing")?;
                pcs::absorb_final_rows(&mut cursor, &final_rows)?;
                let expected: Vec<usize> =
                    (0..pcs::QUERIES).map(|_| cursor.query(CODE / 2)).collect();
                if indices != expected {
                    return Err("owner Fiat-Shamir query view mismatch".into());
                }
                queried[kind / 2] = true;
                persist_book_query_marker(
                    root,
                    party,
                    kind,
                    hex::encode(cursor.view()).as_bytes(),
                )?;
                let answers: Vec<PairOpening> = indices
                    .iter()
                    .map(|i| PairOpening {
                        original: [oracles[kind].open(*i), oracles[kind].open(*i + CODE / 2)],
                        mask: [
                            oracles[kind + 1].open(*i),
                            oracles[kind + 1].open(*i + CODE / 2),
                        ],
                    })
                    .collect();
                serde_json::to_value(answers).unwrap()
            }
            Request::PrepareQuery {
                kind,
                indices,
                final_rows,
            } => {
                let owner = agreement_owner
                    .as_ref()
                    .ok_or("query agreement request requires a pinned owner launch")?;
                if (kind >= 8 || !kind.is_multiple_of(2))
                    || oracles.len() != 8
                    || indices.len() != pcs::QUERIES
                    || indices.iter().any(|i| *i >= CODE / 2)
                    || !folded[kind / 2]
                    || queried[kind / 2]
                    || prepared_queries[kind / 2].is_some()
                {
                    return Err(
                        "worker query budget was consumed, prepared, or request is out of order"
                            .into(),
                    );
                }
                if final_rows.len() != 7 || own_final[kind / 2].as_ref() != Some(&final_rows[party])
                {
                    return Err("owner final-row view changed".into());
                }
                let mut cursor = cursors[kind / 2]
                    .as_ref()
                    .ok_or("owner transcript cursor missing")?
                    .clone();
                pcs::absorb_final_rows(&mut cursor, &final_rows)?;
                let expected: Vec<usize> =
                    (0..pcs::QUERIES).map(|_| cursor.query(CODE / 2)).collect();
                if indices != expected {
                    return Err("owner Fiat-Shamir query view mismatch".into());
                }
                let context = agreement_contexts[kind / 2]
                    .as_ref()
                    .ok_or("owner query agreement context missing")?;
                let digest_sha512 =
                    query_agreement::query_digest(context, &final_rows, &indices, &cursor.view())?;
                let budget = DurableQueryBudget::reserve(
                    &owner.state_root,
                    party,
                    statement.as_ref().ok_or("owner statement missing")?,
                    &owner.roster_sha512,
                    kind,
                    &digest_sha512,
                )?;
                let approval = owner.identity.approve(&digest_sha512)?;
                prepared_queries[kind / 2] = Some(PreparedQuery {
                    digest_sha512,
                    indices,
                    budget,
                });
                serde_json::to_value(approval).map_err(|_| "owner approval encoding")?
            }
            Request::QueryWithAgreement { kind, approvals } => {
                let owner = agreement_owner
                    .as_ref()
                    .ok_or("query agreement request requires a pinned owner launch")?;
                if (kind >= 8 || !kind.is_multiple_of(2))
                    || oracles.len() != 8
                    || !folded[kind / 2]
                    || queried[kind / 2]
                {
                    return Err("worker agreed query request is out of order".into());
                }
                let prepared = prepared_queries[kind / 2]
                    .as_ref()
                    .ok_or("owner query was not durably prepared")?;
                query_agreement::verify_all_approvals(
                    &owner.roster,
                    &owner.roster_sha512,
                    &prepared.digest_sha512,
                    &approvals,
                )?;
                // Both receipts are persisted before any call to Merkle::open.
                persist_book_query_marker(root, party, kind, prepared.digest_sha512.as_bytes())?;
                prepared.budget.consume(&approvals)?;
                queried[kind / 2] = true;
                cursors[kind / 2] = None;
                own_final[kind / 2] = None;
                agreement_contexts[kind / 2] = None;
                let prepared = prepared_queries[kind / 2]
                    .take()
                    .ok_or("owner prepared query disappeared")?;
                let answers: Vec<PairOpening> = prepared
                    .indices
                    .iter()
                    .map(|i| PairOpening {
                        original: [oracles[kind].open(*i), oracles[kind].open(*i + CODE / 2)],
                        mask: [
                            oracles[kind + 1].open(*i),
                            oracles[kind + 1].open(*i + CODE / 2),
                        ],
                    })
                    .collect();
                serde_json::to_value(answers).unwrap()
            }
            Request::Shutdown => break,
        };
        serde_json::to_writer(&mut stdout, &answer).map_err(|_| "worker response")?;
        writeln!(stdout).map_err(|_| "worker response")?;
        stdout.flush().map_err(|_| "worker response")?;
    }
    Ok(())
}
