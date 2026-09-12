//! CoCode/Spartan trial composition over native malicious MPC. All scalar
//! outputs below are masked protocol messages; no original witness enters this
//! coordinator's memory or input/output API.
use crate::{
    algebra::{dot, pack, small_polynomial},
    circuit::{self, Statement},
    field::F,
    mpc_program,
    native::Native,
    party::{Authorization, Request},
    pcs::{self, NodeRoots, PairOpening, PcsProof, CODE, QUERIES},
    transcript::Transcript,
    verifier::{self, ConstraintPrefix, Proof},
};

fn count(values: Vec<F>, expected: usize) -> Result<Vec<F>, String> {
    if values.len() != expected {
        return Err("native public output count".into());
    }
    Ok(values)
}

// Keep the transcript, statement authorization, and prior opening explicit at
// the two call sites; they represent distinct protocol binding obligations.
#[allow(clippy::too_many_arguments)]
fn open(
    native: &mut Native,
    t: &mut Transcript,
    roots: &[NodeRoots],
    kind: usize,
    weights: &[F],
    claim: F,
    prefix: &ConstraintPrefix,
    prior: Option<&PcsProof>,
) -> Result<PcsProof, String> {
    // Preserve a verifier checkpoint: locally verify each completed opening
    // against the same transcript before allowing the next protocol phase.
    let before = t.clone();
    t.absorb(b"pcs-kind", &(kind as u64).to_le_bytes());
    t.absorb_fields(b"pcs-claim", &[claim]);
    let mask_claim = count(
        native.run(
            mpc_program::pcs_mask_claim(kind, weights),
            t.view(),
            false,
            false,
            false,
        )?,
        1,
    )?[0];
    t.absorb_fields(b"pcs-mask-claim", &[mask_claim]);
    let b = t.field(b"pcs-aggregation");
    let h = count(
        native.run(
            mpc_program::pcs_reply(kind, weights, b),
            t.view(),
            false,
            false,
            false,
        )?,
        3,
    )?;
    if h[0] + h[1] != claim + b * mask_claim {
        return Err("honest PCS initial equality failed".into());
    }
    t.absorb_fields(b"pcs-h", &h);
    t.field(b"pcs-fold");
    let authorization = Authorization {
        prefix: prefix.clone(),
        prior_mask_opening: prior.cloned(),
        mask_claim: pack(&[mask_claim]),
        sumcheck: pack(&h),
    };
    let final_rows: Vec<String> = native.broadcast(&Request::Fold {
        kind,
        authorization: Box::new(authorization),
    })?;
    pcs::absorb_final_rows(t, &final_rows)?;
    let indices: Vec<usize> = (0..QUERIES).map(|_| t.query(CODE / 2)).collect();
    let by_party: Vec<Vec<PairOpening>> = native.broadcast(&Request::Query {
        kind,
        indices,
        final_rows: final_rows.clone(),
    })?;
    if by_party.iter().any(|q| q.len() != QUERIES) {
        return Err("owner query response length".into());
    }
    let queries = (0..QUERIES)
        .map(|q| by_party.iter().map(|p| p[q].clone()).collect())
        .collect();
    let proof = PcsProof {
        mask_claim: pack(&[mask_claim]),
        sumcheck: pack(&h),
        final_rows,
        queries,
    };
    let mut checked = before;
    pcs::verify(&mut checked, roots, kind, weights, claim, &proof)?;
    if checked.view() != t.view() {
        return Err("prover/verifier PCS transcript mismatch".into());
    }
    Ok(proof)
}

pub fn prove(native: &mut Native, statement: &Statement) -> Result<Proof, String> {
    statement.validate()?;
    let _: Vec<serde_json::Value> = native.broadcast(&Request::BindStatement {
        statement: statement.clone(),
    })?;
    let initial =
        Transcript::new(&serde_json::to_vec(statement).map_err(|_| "statement serialization")?);
    native.run(
        mpc_program::initialize(),
        initial.view(),
        true,
        false,
        false,
    )?;
    eprintln!("coSNARK: native seven-party private witness initialized and checked");
    let roots = native.roots()?;
    let mut t = verifier::start(statement, &roots)?;
    let mut tau = t.fields(b"constraint-weight", 6);
    let sums = count(
        native.run(
            mpc_program::initial_sums(&tau),
            t.view(),
            false,
            false,
            false,
        )?,
        2,
    )?;
    t.absorb_fields(b"initial-mask-sums", &sums);
    let pad = t.field(b"constraint-padding");
    tau.push(pad);
    let batch = t.field(b"sumcheck-mask-batch");
    let mut claim = pad * sums[0] + batch * sums[1];
    let mut point = Vec::new();
    let mut rounds = Vec::new();
    for round in 0..circuit::SUMCHECK_VARS {
        let h = count(
            native.run(
                mpc_program::constraint_round(&tau, &point, batch),
                t.view(),
                false,
                false,
                false,
            )?,
            4,
        )?;
        if h[0] + h[1] != claim {
            return Err(format!("honest financial sumcheck round {round} failed"));
        }
        t.absorb_fields(b"constraint-h", &h);
        let r = t.field(b"constraint-round");
        claim = small_polynomial(&h, r);
        point.push(r);
        rounds.push(pack(&h));
        eprintln!(
            "coSNARK: native masked constraint round {}/{}",
            round + 1,
            circuit::SUMCHECK_VARS
        );
    }
    let endpoints = count(
        native.run(
            mpc_program::endpoints(&point),
            t.view(),
            false,
            false,
            false,
        )?,
        4,
    )?;
    t.absorb_fields(b"constraint-endpoints", &endpoints);
    let prefix = ConstraintPrefix {
        statement: statement.clone(),
        roots: roots.clone(),
        initial_mask_sums: pack(&sums),
        sumcheck_rounds: rounds.clone(),
        endpoint_claims: pack(&endpoints),
    };
    let mask_opening = open(
        native,
        &mut t,
        &roots,
        2,
        &circuit::mask_opening_weights(&point),
        endpoints[3],
        &prefix,
        None,
    )?;
    eprintln!("coSNARK: masked sumcheck polynomial opening verified");
    let batching = t.fields(b"linear-relation-batch", 4);
    let weights = circuit::witness_opening_weights(&point, &batching);
    let target = dot(&endpoints[..3], &batching[..3]) + batching[3];
    let witness_opening = open(
        native,
        &mut t,
        &roots,
        0,
        &weights,
        target,
        &prefix,
        Some(&mask_opening),
    )?;
    let proof = Proof {
        statement: statement.clone(),
        roots,
        initial_mask_sums: pack(&sums),
        sumcheck_rounds: rounds,
        endpoint_claims: pack(&endpoints),
        mask_opening,
        witness_opening,
        transcript_sha512: hex::encode(t.view()),
    };
    verifier::verify(&proof, statement)?;
    eprintln!("coSNARK: complete public proof accepted by the standalone verifier logic");
    Ok(proof)
}
