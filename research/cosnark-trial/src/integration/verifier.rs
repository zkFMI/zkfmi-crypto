//! Standalone verifier: takes no private file, MPC session, signature, or key.
use crate::{
    algebra::{dot, eq_table, small_polynomial, unpack},
    field::F,
    integration::circuit::{self, Statement, SUMCHECK_VARS},
    pcs::{self, NodeRoots, PcsProof},
    transcript::Transcript,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Proof {
    pub statement: Statement,
    pub roots: Vec<NodeRoots>,
    pub book_claims: String,
    pub initial_mask_sums: String,
    pub sumcheck_rounds: Vec<String>,
    pub endpoint_claims: String,
    pub mask_opening: PcsProof,
    pub book_openings: Vec<PcsProof>,
    pub witness_opening: PcsProof,
    pub transcript_sha512: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstraintPrefix {
    pub statement: Statement,
    pub roots: Vec<NodeRoots>,
    pub book_claims: String,
    pub initial_mask_sums: String,
    pub sumcheck_rounds: Vec<String>,
    pub endpoint_claims: String,
}

pub fn start(statement: &Statement, roots: &[NodeRoots]) -> Result<Transcript, String> {
    statement.validate()?;
    if super::book_commitment(roots, 4)? != statement.settlement.before_commitment
        || super::book_commitment(roots, 6)? != statement.settlement.after_commitment
    {
        return Err("authoritative book commitment mismatch".into());
    }
    if roots.len() != circuit::PARTIES
        || roots.iter().any(|r| {
            r.roots.len() != 8
                || r.roots
                    .iter()
                    .any(|h| h.len() != 128 || hex::decode(h).is_err())
        })
    {
        return Err("commitment root roster".into());
    }
    let mut t = Transcript::new(&serde_json::to_vec(statement).map_err(|_| "statement encoding")?);
    t.absorb(
        b"commitment-roots",
        &serde_json::to_vec(roots).map_err(|_| "root encoding")?,
    );
    Ok(t)
}

pub fn constraint_transcript(
    proof: &ConstraintPrefix,
) -> Result<(Transcript, Vec<F>, Vec<F>), String> {
    let mut t = start(&proof.statement, &proof.roots)?;
    let _ = book_weights(&mut t);
    let claims = unpack(&proof.book_claims, 2)?;
    t.absorb_fields(b"book-claims", &claims);
    let mut tau = t.fields(b"constraint-weight", 10);
    let sums = unpack(&proof.initial_mask_sums, 2)?;
    t.absorb_fields(b"initial-mask-sums", &sums);
    let pad = t.field(b"constraint-padding");
    tau.push(pad);
    let mask_batch = t.field(b"sumcheck-mask-batch");
    let mut claim = pad * sums[0] + mask_batch * sums[1];
    if proof.sumcheck_rounds.len() != SUMCHECK_VARS {
        return Err("sumcheck round count".into());
    }
    let mut point = Vec::new();
    for round in &proof.sumcheck_rounds {
        let h = unpack(round, 4)?;
        if h[0] + h[1] != claim {
            return Err("constraint sumcheck identity".into());
        }
        t.absorb_fields(b"constraint-h", &h);
        let r = t.field(b"constraint-round");
        claim = small_polynomial(&h, r);
        point.push(r);
    }
    let endpoints = unpack(&proof.endpoint_claims, 4)?;
    let equality: F = tau
        .iter()
        .zip(&point)
        .map(|(a, b)| *a * b + (F::from(1u64) - a) * (F::from(1u64) - b))
        .product();
    if claim != equality * (endpoints[0] * endpoints[1] - endpoints[2]) + mask_batch * endpoints[3]
    {
        return Err("constraint sumcheck endpoint".into());
    }
    t.absorb_fields(b"constraint-endpoints", &endpoints);
    Ok((t, point, endpoints))
}

pub fn verify(proof: &Proof, expected: &super::SettlementStatement) -> Result<(), String> {
    proof.statement.validate_legacy()?;
    verify_body(proof, expected)
}

pub fn verify_query_agreement(
    proof: &Proof,
    expected: &super::SettlementStatement,
    trusted_roster_sha512: &str,
) -> Result<(), String> {
    proof
        .statement
        .validate_query_agreement(trusted_roster_sha512)?;
    verify_body(proof, expected)
}

fn verify_body(proof: &Proof, expected: &super::SettlementStatement) -> Result<(), String> {
    expected.validate()?;
    if &proof.statement.settlement != expected {
        return Err("external statement mismatch".into());
    }
    let prefix = ConstraintPrefix {
        statement: proof.statement.clone(),
        roots: proof.roots.clone(),
        book_claims: proof.book_claims.clone(),
        initial_mask_sums: proof.initial_mask_sums.clone(),
        sumcheck_rounds: proof.sumcheck_rounds.clone(),
        endpoint_claims: proof.endpoint_claims.clone(),
    };
    let (mut t, weights, target) =
        opening_context(&prefix, Some(&proof.mask_opening), &proof.book_openings, 0)?;
    pcs::verify_roster(
        &mut t,
        &proof.roots,
        0,
        &weights,
        target,
        &proof.witness_opening,
        8,
    )?;
    if proof.transcript_sha512 != hex::encode(t.view()) {
        return Err("final transcript binding".into());
    }
    Ok(())
}

pub fn book_weights(t: &mut Transcript) -> [Vec<F>; 2] {
    // Two independent, full-field blinds hide the two permitted lifetime
    // linear claims of a book (output proof and next input proof).
    [
        t.fields(b"previous-book-linear-weights", 6),
        t.fields(b"next-book-linear-weights", 6),
    ]
}

pub fn opening_context(
    prefix: &ConstraintPrefix,
    mask: Option<&PcsProof>,
    books: &[PcsProof],
    kind: usize,
) -> Result<(Transcript, Vec<F>, F), String> {
    let (mut t, point, ends) = constraint_transcript(prefix)?;
    if kind == 2 {
        if mask.is_some() || !books.is_empty() {
            return Err("unexpected prior opening".into());
        }
        return Ok((t, circuit::mask_opening_weights(&point), ends[3]));
    }
    pcs::verify_roster(
        &mut t,
        &prefix.roots,
        2,
        &circuit::mask_opening_weights(&point),
        ends[3],
        mask.ok_or("missing mask opening")?,
        8,
    )?;
    let mut initial = start(&prefix.statement, &prefix.roots)?;
    let weights = book_weights(&mut initial);
    let claims = unpack(&prefix.book_claims, 2)?;
    let required = match kind {
        4 => 0,
        6 => 1,
        0 => 2,
        _ => return Err("opening kind".into()),
    };
    if books.len() != required {
        return Err("prior book opening count".into());
    }
    for (i, p) in books.iter().enumerate() {
        pcs::verify_roster(
            &mut t,
            &prefix.roots,
            4 + 2 * i,
            &weights[i],
            claims[i],
            p,
            8,
        )?;
    }
    if kind != 0 {
        return Ok((t, weights[required].clone(), claims[required]));
    }
    let batching = t.fields(b"linear-relation-batch", 4);
    let book_batch = t.fields(b"book-binding-batch", 2);
    let mut w =
        circuit::witness_opening_weights(&point, &batching, prefix.statement.settlement.no_fill);
    circuit::bind_book_weights(&mut w, &weights[0], &weights[1], &book_batch);
    let target = dot(&ends[..3], &batching[..3]) + batching[3] + dot(&claims, &book_batch);
    Ok((t, w, target))
}

/// Public coefficient tables used by both the prover and verifier adapters.
pub fn equality_weights(point: &[F]) -> Vec<F> {
    eq_table(point)
}
