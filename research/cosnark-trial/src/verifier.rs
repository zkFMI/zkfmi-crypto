//! Standalone verifier: takes no private file, MPC session, signature, or key.
use crate::{
    algebra::{dot, eq_table, small_polynomial, unpack},
    circuit::{self, Statement, SUMCHECK_VARS},
    field::F,
    pcs::{self, NodeRoots, PcsProof},
    transcript::Transcript,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Proof {
    pub statement: Statement,
    pub roots: Vec<NodeRoots>,
    pub initial_mask_sums: String,
    pub sumcheck_rounds: Vec<String>,
    pub endpoint_claims: String,
    pub mask_opening: PcsProof,
    pub witness_opening: PcsProof,
    pub transcript_sha512: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstraintPrefix {
    pub statement: Statement,
    pub roots: Vec<NodeRoots>,
    pub initial_mask_sums: String,
    pub sumcheck_rounds: Vec<String>,
    pub endpoint_claims: String,
}

pub fn start(statement: &Statement, roots: &[NodeRoots]) -> Result<Transcript, String> {
    statement.validate()?;
    if roots.len() != circuit::PARTIES
        || roots.iter().any(|r| {
            r.roots.len() != 4
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
    let mut tau = t.fields(b"constraint-weight", 6);
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

pub fn verify(proof: &Proof, expected: &Statement) -> Result<(), String> {
    if &proof.statement != expected {
        return Err("external statement mismatch".into());
    }
    let prefix = ConstraintPrefix {
        statement: proof.statement.clone(),
        roots: proof.roots.clone(),
        initial_mask_sums: proof.initial_mask_sums.clone(),
        sumcheck_rounds: proof.sumcheck_rounds.clone(),
        endpoint_claims: proof.endpoint_claims.clone(),
    };
    let (mut t, point, endpoints) = constraint_transcript(&prefix)?;
    pcs::verify(
        &mut t,
        &proof.roots,
        2,
        &circuit::mask_opening_weights(&point),
        endpoints[3],
        &proof.mask_opening,
    )?;
    let batching = t.fields(b"linear-relation-batch", 4);
    let weights = circuit::witness_opening_weights(&point, &batching);
    let target = dot(&endpoints[..3], &batching[..3]) + batching[3];
    pcs::verify(
        &mut t,
        &proof.roots,
        0,
        &weights,
        target,
        &proof.witness_opening,
    )?;
    if proof.transcript_sha512 != hex::encode(t.view()) {
        return Err("final transcript binding".into());
    }
    Ok(())
}

/// Public coefficient tables used by both the prover and verifier adapters.
pub fn equality_weights(point: &[F]) -> Vec<F> {
    eq_table(point)
}
