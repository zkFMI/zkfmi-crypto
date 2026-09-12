//! One-round, fully revealed final-oracle coPCS adapter. The deliberately large
//! terminal oracle avoids an unimplemented sparse-matrix or recursive verifier.
use crate::{algebra::{bounded_decode,dot,fold_table,lagrange_at,small_polynomial,unpack}, circuit::PARTIES, field::{domain,F}, transcript::{self,Opening,Transcript}};
use ark_ff::Field;
use ark_poly::EvaluationDomain;
use serde::{Deserialize,Serialize};

pub const PADDED: usize = 16_384;
pub const CODE: usize = PADDED * 4;
pub const QUERIES: usize = 512;

#[derive(Clone,Serialize,Deserialize)]
pub struct NodeRoots { pub roots: Vec<String> }
#[derive(Clone,Serialize,Deserialize)]
pub struct PairOpening { pub original: [Opening;2], pub mask: [Opening;2] }
#[derive(Clone,Serialize,Deserialize)]
pub struct PcsProof {
    pub mask_claim: String,
    pub sumcheck: String,
    pub final_rows: Vec<String>,
    pub queries: Vec<Vec<PairOpening>>,
}

pub fn oracle_domain(kind:usize,party:usize)->Vec<u8> {
    format!("zkfmi-cocode-v1-oracle-{kind}-party-{party}").into_bytes()
}

/// Transform a linear functional on the message evaluation table into one on
/// the coefficients: sum_i lambda_i p(zeta^i) = sum_j FFT(lambda)_j coeff_j.
pub fn coefficient_weights(message_weights:&[F])->Vec<F> {
    assert!(message_weights.len()<=PADDED);
    let mut weights=message_weights.to_vec(); weights.resize(PADDED,F::from(0u64));
    domain(PADDED).unwrap().fft(&weights)
}

/// Public message-basis weights for h(T). They allow native MPC to compute the
/// sumcheck replies without exporting any private coefficient or partial share.
pub fn reply_message_weights(message_weights:&[F],at:F)->Vec<F> {
    let weights=coefficient_weights(message_weights);
    let reduced=fold_table(&weights,at);
    let mut coefficient=Vec::with_capacity(PADDED);
    for weight in reduced { coefficient.push((F::from(1u64)-at)*weight); coefficient.push(at*weight); }
    domain(PADDED).unwrap().ifft(&coefficient)
}

pub fn absorb_final_rows(transcript:&mut Transcript,rows:&[String])->Result<Vec<Vec<F>>,String> {
    if rows.len()!=PARTIES { return Err("wrong final-row count".into()); }
    rows.iter().enumerate().map(|(i,row)| {
        let values=unpack(row,CODE/2)?;
        transcript.absorb(b"final-party", &(i as u64).to_le_bytes());
        transcript.absorb(b"final-row-hash", &transcript::digest(row.as_bytes()));
        Ok(values)
    }).collect()
}

pub fn verify(
    transcript:&mut Transcript,roots:&[NodeRoots],kind:usize,
    weights:&[F],claim:F,proof:&PcsProof,
)->Result<(),String> {
    if kind!=0 && kind!=2 { return Err("invalid oracle kind".into()); }
    if roots.len()!=PARTIES || roots.iter().any(|r|r.roots.len()!=4) { return Err("root roster shape".into()); }
    transcript.absorb(b"pcs-kind",&(kind as u64).to_le_bytes());
    transcript.absorb_fields(b"pcs-claim",&[claim]);
    let mask_claim=unpack(&proof.mask_claim,1)?[0];
    transcript.absorb_fields(b"pcs-mask-claim",&[mask_claim]);
    let b=transcript.field(b"pcs-aggregation");
    let h=unpack(&proof.sumcheck,3)?;
    if h[0]+h[1]!=claim+b*mask_claim { return Err("PCS sumcheck initial identity".into()); }
    transcript.absorb_fields(b"pcs-h",&h);
    let r=transcript.field(b"pcs-fold");
    let rows=absorb_final_rows(transcript,&proof.final_rows)?;
    let decoded=rows.iter().map(|row|bounded_decode(row,PADDED/2)).collect::<Result<Vec<_>,_>>()?;
    // The original k=1, t=2 sharing dimension uses evaluation points 1..7.
    // Require ALL rows to fit one degree-two x polynomial. Interpolating seven
    // arbitrary rows with a degree-six polynomial would hide malicious edits.
    let basis=[F::from(1u64),F::from(2u64),F::from(3u64)];
    for party in 3..PARTIES {
        let lambda=lagrange_at(&basis,F::from((party+1) as u64));
        for column in 0..PADDED/2 {
            if decoded[party][column] != (0..3).map(|i|lambda[i]*decoded[i][column]).sum::<F>() {
                return Err("tensor code x-degree bound".into());
            }
        }
    }
    let zero=lagrange_at(&basis,F::from(0u64));
    let joint:Vec<F>=(0..PADDED/2).map(|j|(0..3).map(|i|zero[i]*decoded[i][j]).sum()).collect();
    if dot(&joint,&fold_table(&coefficient_weights(weights),r))!=small_polynomial(&h,r) {
        return Err("PCS final evaluation identity".into());
    }
    if proof.queries.len()!=QUERIES { return Err("PCS query count".into()); }
    let d=domain(CODE).unwrap(); let inv2=F::from(2u64).inverse().unwrap();
    for query in &proof.queries {
        let index=transcript.query(CODE/2);
        if query.len()!=PARTIES { return Err("PCS query roster".into()); }
        for (party,opening) in query.iter().enumerate() {
            let mut aggregate=[F::from(0u64);2];
            for side in 0..2 {
                let at=index+side*(CODE/2);
                let original=transcript::verify(&roots[party].roots[kind],&oracle_domain(kind,party),CODE,at,&opening.original[side])?;
                let mask=transcript::verify(&roots[party].roots[kind+1],&oracle_domain(kind+1,party),CODE,at,&opening.mask[side])?;
                aggregate[side]=original+b*mask;
            }
            let even=(aggregate[0]+aggregate[1])*inv2;
            let odd=(aggregate[0]-aggregate[1])*inv2*d.element(index).inverse().unwrap();
            if rows[party][index]!=(F::from(1u64)-r)*even+r*odd {
                return Err("authenticated normalized fold query".into());
            }
        }
    }
    Ok(())
}
