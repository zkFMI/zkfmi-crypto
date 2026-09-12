//! Executable necessary-completeness check, NOT a SNARK implementation.
//!
//! Independent transcription of the equations in ePrint 2026/729, Appendix B,
//! Construction 4(1c, 1f, 3), and Appendix E Lemma 5. No author source code is
//! incorporated. All fixtures are PUBLIC. Seven rows do not constitute an MPC
//! execution or demonstrate secrecy against two malicious parties.
use crate::field::{domain, encode, encode_row, F};
use ark_ff::Field;
use ark_poly::EvaluationDomain;
use serde::Serialize;

fn affine_pair(a: F, b: F, r: F) -> F {
    (F::from(1u64) - r) * a + r * b
}

fn inner(a: &[F], b: &[F]) -> F {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| *x * y).sum()
}

fn restrict(values: &[F], r: F) -> Vec<F> {
    values
        .chunks_exact(2)
        .map(|p| affine_pair(p[0], p[1], r))
        .collect()
}

/// Appendix B: E(Y) + r O(Y), computed from an actual FFT codeword.
fn rs_fold(codeword: &[F], r: F, normalize_even: bool) -> Vec<F> {
    let n = codeword.len();
    let d = domain(n).expect("fixed radix-two domain");
    let half = F::from(2u64).inverse().unwrap();
    (0..n / 2)
        .map(|i| {
            let y = d.element(i);
            let even = (codeword[i] + codeword[i + n / 2]) * half;
            let odd = (codeword[i] - codeword[i + n / 2]) * half * y.inverse().unwrap();
            let even_scale = if normalize_even {
                F::from(1u64) - r
            } else {
                F::from(1u64)
            };
            even_scale * even + r * odd
        })
        .collect()
}

fn decode_bounded(code: &[F], bound: usize) -> Option<Vec<F>> {
    let coefficients = domain(code.len()).ok()?.ifft(code);
    if coefficients[bound..].iter().any(|v| *v != F::from(0u64)) {
        return None;
    }
    Some(coefficients[..bound].to_vec())
}

/// Interpolation at zero of degree <= 6, used only for public fixture rows.
fn combine(rows: &[Vec<F>]) -> Vec<F> {
    let mut out = vec![F::from(0u64); rows[0].len()];
    for (i, row) in rows.iter().enumerate() {
        let x = F::from((i + 1) as u64);
        let mut weight = F::from(1u64);
        for j in 0..rows.len() {
            if i != j {
                let y = F::from((j + 1) as u64);
                weight *= -y * (x - y).inverse().unwrap();
            }
        }
        for (v, item) in out.iter_mut().zip(row) {
            *v += weight * item;
        }
    }
    out
}

#[derive(Serialize)]
pub struct Observation {
    pub fixture: &'static str,
    pub full_candidate_executed: bool,
    pub malicious_mpc_executed: bool,
    pub zero_knowledge_demonstrated: bool,
    pub security_parameters_validated: bool,
    pub public_rows: usize,
    pub public_coefficients_per_row: usize,
    pub rs_encoding_points_per_row: usize,
    pub sumcheck_initial_identity: bool,
    pub literal_fold_rows_in_degree_bound: usize,
    pub literal_fold_matches_even_plus_r_odd: usize,
    pub literal_final_evaluation_matches_sumcheck: bool,
    pub sumcheck_claim_hex_le: String,
    pub literal_decoded_claim_hex_le: String,
    pub normalized_control_matches_sumcheck: bool,
    pub normalized_control_rows_in_degree_bound: usize,
    pub normalized_endpoint_controls_pass: bool,
    pub verdict: &'static str,
    pub blocker: &'static str,
}

pub fn run() -> Observation {
    const ROWS: usize = 7;
    const WIDTH: usize = 8;
    const CODE: usize = 32;
    // Fixed public fixtures make the mismatch exactly reproducible. These are
    // not private shares, production padding, or zero-knowledge randomness.
    let mut message = Vec::new();
    let mut mask = Vec::new();
    for party in 1..=ROWS {
        let x = F::from(party as u64);
        message.push(
            (0..WIDTH)
                .map(|j| {
                    F::from((j + 2) as u64)
                        + x * F::from((3 * j + 1) as u64)
                        + x.square() * F::from((j + 5) as u64)
                })
                .collect::<Vec<_>>(),
        );
        mask.push(
            (0..WIDTH)
                .map(|j| {
                    F::from((2 * j + 3) as u64)
                        + x * F::from((j + 9) as u64)
                        + x.square() * F::from((j + 1) as u64)
                })
                .collect::<Vec<_>>(),
        );
    }
    // A publicly specified separable coefficient-weight tensor. For k=1 the
    // x-direction evaluation is interpolation at zero. Each y pair uses the
    // same first-variable factor, as in the transformed inner-product claim.
    let weights: Vec<F> = [1u64, 3, 5, 15, 7, 21, 35, 105]
        .into_iter()
        .map(F::from)
        .collect();
    let b = F::from(11u64);
    let r = F::from(13u64);
    let aggregate: Vec<Vec<F>> = message
        .iter()
        .zip(&mask)
        .map(|(a, z)| a.iter().zip(z).map(|(a, z)| *a + b * z).collect())
        .collect();
    let joint = combine(&aggregate);
    let initial = inner(&combine(&message), &weights) + b * inner(&combine(&mask), &weights);
    let h = |challenge: F| inner(&restrict(&joint, challenge), &restrict(&weights, challenge));
    let expected = h(r);
    let mut literal_rows = Vec::new();
    let mut corrected_rows = Vec::new();
    let mut literal_degrees = 0;
    let mut corrected_degrees = 0;
    let mut literal_formula = 0;
    let mut endpoints = true;
    for row in &aggregate {
        let encoded = encode_row(row, CODE).unwrap();
        let literal = decode_bounded(&rs_fold(&encoded, r, false), WIDTH / 2);
        let corrected = decode_bounded(&rs_fold(&encoded, r, true), WIDTH / 2);
        literal_degrees += usize::from(literal.is_some());
        corrected_degrees += usize::from(corrected.is_some());
        let literal = literal.expect("fold degree bound");
        let corrected = corrected.expect("normalized fold degree bound");
        literal_formula += usize::from(
            literal
                == row
                    .chunks_exact(2)
                    .map(|p| p[0] + r * p[1])
                    .collect::<Vec<_>>(),
        );
        for endpoint in [F::from(0u64), F::from(1u64)] {
            endpoints &= decode_bounded(&rs_fold(&encoded, endpoint, true), WIDTH / 2).unwrap()
                == restrict(row, endpoint);
        }
        literal_rows.push(literal);
        corrected_rows.push(corrected);
    }
    let literal_claim = inner(&combine(&literal_rows), &restrict(&weights, r));
    let control_claim = inner(&combine(&corrected_rows), &restrict(&weights, r));
    Observation {
        fixture: "PUBLIC exact-field one-round Construction 4 necessary equality; not complete coPCS",
        full_candidate_executed: false,
        malicious_mpc_executed: false,
        zero_knowledge_demonstrated: false,
        security_parameters_validated: false,
        public_rows: ROWS,
        public_coefficients_per_row: WIDTH,
        rs_encoding_points_per_row: CODE,
        sumcheck_initial_identity: initial == h(F::from(0u64)) + h(F::from(1u64)),
        literal_fold_rows_in_degree_bound: literal_degrees,
        literal_fold_matches_even_plus_r_odd: literal_formula,
        literal_final_evaluation_matches_sumcheck: literal_claim == expected,
        sumcheck_claim_hex_le: hex::encode(encode(expected)),
        literal_decoded_claim_hex_le: hex::encode(encode(literal_claim)),
        normalized_control_matches_sumcheck: control_claim == expected,
        normalized_control_rows_in_degree_bound: corrected_degrees,
        normalized_endpoint_controls_pass: endpoints,
        verdict: "blocked",
        blocker: "The literal Appendix B fold does not restrict the coefficient multilinear extension at the same challenge used in Construction 4. The normalized control is a local equation adjustment, not an established malicious-secure coSNARK implementation.",
    }
}
