//! CoCode/Spartan trial composition over native malicious MPC. All scalar
//! outputs below are masked protocol messages; no original witness enters this
//! coordinator's memory or input/output API.
use crate::{
    algebra::{pack, small_polynomial},
    field::F,
    integration::circuit::{self, Statement, LEGACY_PROTOCOL, QUERY_AGREEMENT_PROTOCOL},
    integration::mpc_program,
    integration::native::Native,
    integration::party::{Authorization, Request},
    integration::verifier::{self, ConstraintPrefix, Proof},
    pcs::{self, NodeRoots, PairOpening, PcsProof, CODE, QUERIES},
    transcript::Transcript,
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
    prior_books: &[PcsProof],
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
        prior_book_openings: prior_books.to_vec(),
        mask_claim: pack(&[mask_claim]),
        sumcheck: pack(&h),
    };
    let final_rows: Vec<String> = native.broadcast(&Request::Fold {
        kind,
        authorization: Box::new(authorization),
    })?;
    pcs::absorb_final_rows(t, &final_rows)?;
    let indices: Vec<usize> = (0..QUERIES).map(|_| t.query(CODE / 2)).collect();
    let by_party: Vec<Vec<PairOpening>> = if prefix.statement.protocol == QUERY_AGREEMENT_PROTOCOL {
        native.query_with_agreement(kind, indices, final_rows.clone())?
    } else {
        native.broadcast(&Request::Query {
            kind,
            indices,
            final_rows: final_rows.clone(),
        })?
    };
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
    pcs::verify_roster(&mut checked, roots, kind, weights, claim, &proof, 8)?;
    if checked.view() != t.view() {
        return Err("prover/verifier PCS transcript mismatch".into());
    }
    Ok(proof)
}

pub fn prove(
    native: &mut Native,
    settlement: super::SettlementStatement,
    resumed: bool,
    invalid: bool,
) -> Result<Proof, String> {
    prove_with_statement(native, Statement::new(settlement), resumed, invalid)
}

pub fn prove_query_agreement(
    native: &mut Native,
    settlement: super::SettlementStatement,
    resumed: bool,
    invalid: bool,
) -> Result<Proof, String> {
    let roster_sha512 = native
        .query_roster_sha512()
        .ok_or("v2 proof requires a pinned query-agreement launch")?
        .to_owned();
    let statement = Statement::new_query_agreement(settlement, &roster_sha512)?;
    prove_with_statement(native, statement, resumed, invalid)
}

fn prove_with_statement(
    native: &mut Native,
    mut statement: Statement,
    resumed: bool,
    invalid: bool,
) -> Result<Proof, String> {
    match (statement.protocol.as_str(), native.query_roster_sha512()) {
        (LEGACY_PROTOCOL, None) => statement.validate_legacy()?,
        (QUERY_AGREEMENT_PROTOCOL, Some(roster)) => statement.validate_query_agreement(roster)?,
        _ => return Err("proof protocol does not match native worker launch".into()),
    }
    let initial =
        Transcript::new(&serde_json::to_vec(&statement).map_err(|_| "statement serialization")?);
    let initialization = native.initialization_source(&statement, resumed)?;
    native.run(initialization, initial.view(), true, invalid, false)?;
    eprintln!("coSNARK: native seven-party private witness initialized and checked");
    let roots = native.roots()?;
    statement.settlement.before_commitment = super::book_commitment(&roots, 4)?;
    statement.settlement.after_commitment = super::book_commitment(&roots, 6)?;
    statement.validate()?;
    let _: Vec<serde_json::Value> = native.broadcast(&Request::BindStatement {
        statement: statement.clone(),
    })?;
    let mut t = verifier::start(&statement, &roots)?;
    let book_weights = verifier::book_weights(&mut t);
    let book_claims = count(
        native.run(
            mpc_program::book_claims(&book_weights[0], &book_weights[1]),
            t.view(),
            false,
            false,
            false,
        )?,
        2,
    )?;
    t.absorb_fields(b"book-claims", &book_claims);
    let mut tau = t.fields(b"constraint-weight", 10);
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
        book_claims: pack(&book_claims),
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
        &[],
    )?;
    eprintln!("coSNARK: masked sumcheck polynomial opening verified");
    let mut book_openings = Vec::new();
    for i in 0..2 {
        let p = open(
            native,
            &mut t,
            &roots,
            4 + 2 * i,
            &book_weights[i],
            book_claims[i],
            &prefix,
            Some(&mask_opening),
            &book_openings,
        )?;
        book_openings.push(p);
    }
    let (checked, weights, target) =
        verifier::opening_context(&prefix, Some(&mask_opening), &book_openings, 0)?;
    // opening_context also draws the two final batching challenges.
    let _ = t.fields(b"linear-relation-batch", 4);
    let _ = t.fields(b"book-binding-batch", 2);
    if checked.view() != t.view() {
        return Err("book binding transcript mismatch".into());
    }
    let witness_opening = open(
        native,
        &mut t,
        &roots,
        0,
        &weights,
        target,
        &prefix,
        Some(&mask_opening),
        &book_openings,
    )?;
    let proof = Proof {
        statement: statement.clone(),
        roots,
        book_claims: pack(&book_claims),
        initial_mask_sums: pack(&sums),
        sumcheck_rounds: rounds,
        endpoint_claims: pack(&endpoints),
        mask_opening,
        book_openings,
        witness_opening,
        transcript_sha512: hex::encode(t.view()),
    };
    if statement.protocol == QUERY_AGREEMENT_PROTOCOL {
        verifier::verify_query_agreement(
            &proof,
            &statement.settlement,
            &statement.query_roster_sha512,
        )?;
    } else {
        verifier::verify(&proof, &statement.settlement)?;
    }
    eprintln!("coSNARK: complete public proof accepted by the standalone verifier logic");
    Ok(proof)
}
