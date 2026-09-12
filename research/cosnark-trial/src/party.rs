//! Private per-party worker. The coordinator receives roots, masked final
//! codewords and authenticated queries, never this worker's original shares.
use crate::{
    algebra::{dot, normalized_rs_fold, pack, unpack},
    circuit::{self, Statement},
    field::{decode, domain, interpolate_row, F},
    mpc_program::STATE_SIZE,
    pcs::{self, NodeRoots, PairOpening, PcsProof, CODE, PADDED},
    transcript::{Merkle, Transcript},
    verifier::{self, ConstraintPrefix},
};
use ark_ff::{BigInteger, FftField, Field, PrimeField, UniformRand};
use ark_poly::EvaluationDomain;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
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
    Shutdown,
}

#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub prefix: ConstraintPrefix,
    pub prior_mask_opening: Option<PcsProof>,
    pub mask_claim: String,
    pub sumcheck: String,
}

fn authorize(
    statement: &Statement,
    oracles: &[Merkle],
    party: usize,
    kind: usize,
    auth: &Authorization,
) -> Result<(Transcript, F, F), String> {
    if &auth.prefix.statement != statement
        || auth.prefix.roots.len() != 7
        || auth.prefix.roots[party].roots != oracles.iter().map(Merkle::root).collect::<Vec<_>>()
    {
        return Err("owner statement or commitment view changed".into());
    }
    let (mut t, point, ends) = verifier::constraint_transcript(&auth.prefix)?;
    let claim = if kind == 2 {
        if auth.prior_mask_opening.is_some() {
            return Err("unexpected prior opening".into());
        }
        ends[3]
    } else {
        let prior = auth
            .prior_mask_opening
            .as_ref()
            .ok_or("missing prior mask opening")?;
        pcs::verify(
            &mut t,
            &auth.prefix.roots,
            2,
            &circuit::mask_opening_weights(&point),
            ends[3],
            prior,
        )?;
        let batch = t.fields(b"linear-relation-batch", 4);
        dot(&ends[..3], &batch[..3]) + batch[3]
    };
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
    Ok((t, b, r))
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
) -> Result<(), String> {
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
        for _ in 0..2 {
            writeln!(file, "{}", 1 + OsRng.next_u32() % 15).map_err(|_| "write owner input")?;
        }
        writeln!(file, "{}", usize::from(invalid && party == 0))
            .map_err(|_| "write owner fixture flag")?;
        for _ in 0..4 * PADDED {
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
    for kind in 0..4 {
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
        oracles.push(Merkle::new(pcs::oracle_domain(kind, party), codeword));
    }
    Ok(oracles)
}

pub fn serve(root: &Path, party: usize) -> Result<(), String> {
    if party >= 7 {
        return Err("invalid private worker index".into());
    }
    fs::set_permissions(root.join("Persistence"), fs::Permissions::from_mode(0o700))
        .map_err(|_| "private persistence permissions")?;
    let mut oracles = Vec::new();
    let mut statement = None;
    let mut committed = false;
    let mut folded = [false; 2];
    let mut queried = [false; 2];
    let mut cursors: [Option<Transcript>; 2] = [None, None];
    let mut own_final: [Option<String>; 2] = [None, None];
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
                if statement.is_some() {
                    return Err("owner statement is already bound".into());
                }
                given.validate()?;
                statement = Some(given);
                serde_json::json!({"party":party,"bound":true})
            }
            Request::Input {
                view,
                initial,
                invalid,
            } => {
                write_input(root, party, &view, initial, invalid)?;
                serde_json::json!({"party":party,"view":view})
            }
            Request::Commit => {
                if committed || statement.is_none() {
                    return Err(
                        "owner commitment is one-shot and requires a bound statement".into(),
                    );
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
                if (kind != 0 && kind != 2)
                    || oracles.len() != 4
                    || folded[kind / 2]
                    || (kind == 0 && !queried[1])
                {
                    return Err("worker fold is out of order or its mask was consumed".into());
                }
                let (cursor, b, r) = authorize(
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
                    .map(|(v, m)| *v + b * m)
                    .collect();
                let final_row = pack(&normalized_rs_fold(&aggregate, r)?);
                folded[kind / 2] = true;
                cursors[kind / 2] = Some(cursor);
                own_final[kind / 2] = Some(final_row.clone());
                serde_json::json!(final_row)
            }
            Request::Query {
                kind,
                indices,
                final_rows,
            } => {
                if (kind != 0 && kind != 2)
                    || oracles.len() != 4
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
            Request::Shutdown => break,
        };
        serde_json::to_writer(&mut stdout, &answer).map_err(|_| "worker response")?;
        writeln!(stdout).map_err(|_| "worker response")?;
        stdout.flush().map_err(|_| "worker response")?;
    }
    Ok(())
}
