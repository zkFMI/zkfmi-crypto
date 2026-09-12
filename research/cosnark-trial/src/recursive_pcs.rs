//! Multi-round form of the existing normalized coCode opening. Each coefficient
//! fold is paired with an inner-product sumcheck, an authenticated RS fold and
//! the next committed oracle. Only the final small MASKED oracle is revealed.
//! Arithmetic uses arkworks; the existing salted SHA-512 Merkle format is kept.
use crate::{
    algebra::{dot, fold_table, lagrange_at, pack, small_polynomial, unpack},
    field::{domain, F},
    transcript::{self, BatchOpening, Transcript},
};
use ark_ff::{FftField, Field};
use ark_poly::EvaluationDomain;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PARTIES: usize = 7;
pub const CORRUPT: usize = 2;
pub const QUERIES: usize = 512;
pub const TERMINAL: usize = 16;
const PADDING_RESERVE: usize = 16_384;
pub const PROTOCOL: &str = "zkfmi-cocode-note-recursive-pcs-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub message: usize,
    pub padded: usize,
}

impl Parameters {
    /// Retain at least the previous adapter's entire padding allocation as
    /// fresh random message padding. Query count and corruption threshold are
    /// fixed protocol constants, not options selected by the proof producer.
    pub fn for_message(message: usize) -> Result<Self, String> {
        let padded = message
            .checked_add(PADDING_RESERVE)
            .and_then(usize::checked_next_power_of_two)
            .ok_or("recursive PCS size overflow")?;
        let result = Self { message, padded };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(self) -> Result<(), String> {
        if self.message == 0
            || !self.padded.is_power_of_two()
            || self.padded < TERMINAL * 2
            || self
                .padded
                .checked_sub(self.message)
                .is_none_or(|n| n < PADDING_RESERVE)
            || !self
                .message
                .checked_add(PADDING_RESERVE)
                .and_then(usize::checked_next_power_of_two)
                .is_some_and(|n| n == self.padded)
        {
            return Err("recursive PCS parameter mismatch".into());
        }
        let code = self
            .padded
            .checked_mul(4)
            .ok_or("recursive PCS code overflow")?;
        domain(code).map_err(str::to_owned)?;
        if F::GENERATOR.pow([code as u64]) == F::from(1u64) {
            return Err("recursive PCS coset overlaps secret message domain".into());
        }
        Ok(())
    }
    pub fn rounds(self) -> usize {
        (self.padded / TERMINAL).trailing_zeros() as usize
    }
    pub fn code(self, round: usize) -> usize {
        (self.padded * 4) >> round
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Round {
    pub sumcheck: String,
    /// Seven roots except at the terminal round, whose complete small row is
    /// absorbed before queries instead. No unauthenticated intermediate row.
    pub next_roots: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Queries {
    pub original: Vec<BatchOpening>,
    pub mask: Vec<BatchOpening>,
    /// Nonterminal folded layers, each containing exactly seven multiproofs.
    pub folded: Vec<Vec<BatchOpening>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proof {
    pub mask_claim: String,
    pub rounds: Vec<Round>,
    pub terminal_rows: Vec<String>,
    pub queries: Queries,
}

/// Public request to the existing all-owner challenge-agreement boundary. The
/// native implementation must authorize the complete transcript and derive
/// every layer's query set from these seeds, before returning any openings.
pub struct QueryPlan {
    pub transcript: [u8; 64],
    pub parameters: Parameters,
    pub kind: usize,
    pub seeds: Vec<usize>,
}

/// Coordinator-facing interface: every returned scalar is a jointly opened
/// protocol message. No method returns an original coefficient, input, share
/// vector, seed or private witness. Native malicious MPC supplies these calls.
pub trait DistributedOpening {
    fn mask_claim(&mut self, coefficient_weights: &[F]) -> Result<F, String>;
    fn aggregate(&mut self, challenge: F) -> Result<(), String>;
    fn sumcheck(&mut self, round: usize, coefficient_weights: &[F]) -> Result<[F; 3], String>;
    fn fold_commit(&mut self, round: usize, challenge: F) -> Result<Vec<String>, String>;
    fn terminal_rows(&mut self) -> Result<Vec<String>, String>;
    fn query_after_agreement(&mut self, plan: &QueryPlan) -> Result<Queries, String>;
}

pub fn oracle_domain(kind: usize, party: usize, layer: usize) -> Vec<u8> {
    format!("{PROTOCOL}:kind:{kind}:party:{party}:layer:{layer}").into_bytes()
}

pub(crate) fn absorb_roots(t: &mut Transcript, roots: &[String]) -> Result<(), String> {
    if roots.len() != PARTIES {
        return Err("recursive PCS root roster".into());
    }
    for (party, root) in roots.iter().enumerate() {
        let raw = hex::decode(root).map_err(|_| "recursive PCS root hex")?;
        if raw.len() != 64 || hex::encode(&raw) != *root {
            return Err("recursive PCS root encoding".into());
        }
        t.absorb(b"recursive-party", &(party as u64).to_le_bytes());
        t.absorb(b"recursive-root", &raw);
    }
    Ok(())
}

pub(crate) fn start(
    t: &mut Transcript,
    p: Parameters,
    kind: usize,
    original: &[String],
    mask: &[String],
    claim: F,
) -> Result<(), String> {
    p.validate()?;
    if ![0, 2].contains(&kind) {
        return Err("recursive PCS oracle kind".into());
    }
    t.absorb(b"pcs-family", PROTOCOL.as_bytes());
    t.absorb(
        b"pcs-parameters",
        &serde_json::to_vec(&p).map_err(|_| "PCS parameters")?,
    );
    t.absorb(b"pcs-kind", &(kind as u64).to_le_bytes());
    absorb_roots(t, original)?;
    absorb_roots(t, mask)?;
    t.absorb_fields(b"pcs-claim", &[claim]);
    Ok(())
}

pub fn coefficient_weights(p: Parameters, message_weights: &[F]) -> Result<Vec<F>, String> {
    p.validate()?;
    if message_weights.len() != p.message {
        return Err("recursive PCS weight length".into());
    }
    let mut weights = message_weights.to_vec();
    weights.resize(p.padded, F::from(0u64));
    domain(p.padded)
        .map_err(str::to_owned)?
        .fft_in_place(&mut weights);
    Ok(weights)
}

pub(crate) fn bind_weights(t: &mut Transcript, weights: &[F]) {
    use sha2::{Digest, Sha512};
    let mut hash = Sha512::new();
    hash.update(b"ZKFMI:PCS-LINEAR-FORM:v1");
    hash.update((weights.len() as u64).to_le_bytes());
    for weight in weights {
        hash.update(crate::field::encode(*weight));
    }
    t.absorb(b"pcs-linear-form", &hash.finalize());
}

pub(crate) fn terminal(t: &mut Transcript, rows: &[String]) -> Result<Vec<Vec<F>>, String> {
    if rows.len() != PARTIES {
        return Err("recursive PCS terminal roster".into());
    }
    rows.iter()
        .enumerate()
        .map(|(party, row)| {
            let values = unpack(row, TERMINAL * 4)?;
            t.absorb(b"terminal-party", &(party as u64).to_le_bytes());
            t.absorb(b"terminal-row", &transcript::digest(row.as_bytes()));
            Ok(values)
        })
        .collect()
}

/// The transcript supplied by the note PIOP already binds the exact statement,
/// expected circuit, authoritative state, deployment and registered owner set.
#[allow(clippy::too_many_arguments)]
pub fn prove(
    native: &mut impl DistributedOpening,
    t: &mut Transcript,
    p: Parameters,
    kind: usize,
    original: &[String],
    mask: &[String],
    message_weights: &[F],
    claim: F,
) -> Result<Proof, String> {
    let checkpoint = t.clone();
    start(t, p, kind, original, mask, claim)?;
    bind_weights(t, message_weights);
    let mut weights = coefficient_weights(p, message_weights)?;
    let mask_claim = native.mask_claim(&weights)?;
    t.absorb_fields(b"pcs-mask-claim", &[mask_claim]);
    let aggregation = t.field(b"pcs-aggregation");
    native.aggregate(aggregation)?;
    let mut current_claim = claim + aggregation * mask_claim;
    let mut rounds = Vec::with_capacity(p.rounds());
    for round in 0..p.rounds() {
        let h = native.sumcheck(round, &weights)?;
        if h[0] + h[1] != current_claim {
            return Err("native recursive PCS sumcheck identity".into());
        }
        t.absorb(b"pcs-round", &(round as u64).to_le_bytes());
        t.absorb_fields(b"pcs-h", &h);
        let challenge = t.field(b"pcs-fold");
        current_claim = small_polynomial(&h, challenge);
        weights = fold_table(&weights, challenge);
        let next_roots = native.fold_commit(round, challenge)?;
        if round + 1 == p.rounds() {
            if !next_roots.is_empty() {
                return Err("unexpected terminal roots".into());
            }
        } else {
            absorb_roots(t, &next_roots)?;
        }
        rounds.push(Round {
            sumcheck: pack(&h),
            next_roots,
        });
    }
    let terminal_rows = native.terminal_rows()?;
    terminal(t, &terminal_rows)?;
    let seeds = (0..QUERIES).map(|_| t.query(p.code(0) / 2)).collect();
    let queries = native.query_after_agreement(&QueryPlan {
        transcript: t.view(),
        parameters: p,
        kind,
        seeds,
    })?;
    let proof = Proof {
        mask_claim: pack(&[mask_claim]),
        rounds,
        terminal_rows,
        queries,
    };
    let mut checked = checkpoint;
    verify(
        &mut checked,
        p,
        kind,
        original,
        mask,
        message_weights,
        claim,
        &proof,
    )?;
    if checked.view() != t.view() {
        return Err("recursive PCS transcript disagreement".into());
    }
    Ok(proof)
}

pub(crate) fn query_indices(p: Parameters, round: usize, seeds: &[usize]) -> Vec<usize> {
    let half = p.code(round) / 2;
    seeds
        .iter()
        .flat_map(|seed| {
            let i = seed & (half - 1);
            [i, i + half]
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn verify(
    t: &mut Transcript,
    p: Parameters,
    kind: usize,
    original: &[String],
    mask: &[String],
    message_weights: &[F],
    claim: F,
    proof: &Proof,
) -> Result<(), String> {
    start(t, p, kind, original, mask, claim)?;
    bind_weights(t, message_weights);
    if proof.rounds.len() != p.rounds()
        || proof.queries.original.len() != PARTIES
        || proof.queries.mask.len() != PARTIES
        || proof.queries.folded.len() + 1 != p.rounds()
        || proof
            .queries
            .folded
            .iter()
            .any(|layer| layer.len() != PARTIES)
    {
        return Err("recursive PCS proof shape".into());
    }
    let mut weights = coefficient_weights(p, message_weights)?;
    let mask_claim = unpack(&proof.mask_claim, 1)?[0];
    t.absorb_fields(b"pcs-mask-claim", &[mask_claim]);
    let aggregation = t.field(b"pcs-aggregation");
    let mut current_claim = claim + aggregation * mask_claim;
    let mut challenges = Vec::with_capacity(p.rounds());
    for (round, step) in proof.rounds.iter().enumerate() {
        let h = unpack(&step.sumcheck, 3)?;
        if h[0] + h[1] != current_claim {
            return Err("recursive PCS sumcheck identity".into());
        }
        t.absorb(b"pcs-round", &(round as u64).to_le_bytes());
        t.absorb_fields(b"pcs-h", &h);
        let r = t.field(b"pcs-fold");
        current_claim = small_polynomial(&h, r);
        weights = fold_table(&weights, r);
        challenges.push(r);
        if round + 1 == p.rounds() {
            if !step.next_roots.is_empty() {
                return Err("recursive terminal roots".into());
            }
        } else {
            absorb_roots(t, &step.next_roots)?;
        }
    }
    let final_values = terminal(t, &proof.terminal_rows)?;
    let mut coset = F::GENERATOR;
    for _ in 0..p.rounds() {
        coset.square_in_place();
    }
    let d = domain(TERMINAL * 4)
        .map_err(str::to_owned)?
        .get_coset(coset)
        .ok_or("recursive terminal coset")?;
    let mut decoded = Vec::with_capacity(PARTIES);
    for row in &final_values {
        let mut coefficients = d.ifft(row);
        if coefficients[TERMINAL..].iter().any(|x| *x != F::from(0u64)) {
            return Err("recursive RS terminal degree".into());
        }
        coefficients.truncate(TERMINAL);
        decoded.push(coefficients);
    }
    let basis = [F::from(1u64), F::from(2u64), F::from(3u64)];
    for party in CORRUPT + 1..PARTIES {
        let lambda = lagrange_at(&basis, F::from((party + 1) as u64));
        for (column, value) in decoded[party].iter().enumerate() {
            if *value
                != (0..=CORRUPT)
                    .map(|i| lambda[i] * decoded[i][column])
                    .sum::<F>()
            {
                return Err("recursive tensor sharing degree".into());
            }
        }
    }
    let zero = lagrange_at(&basis, F::from(0u64));
    let joint: Vec<F> = (0..TERMINAL)
        .map(|column| (0..=CORRUPT).map(|i| zero[i] * decoded[i][column]).sum())
        .collect();
    if dot(&joint, &weights) != current_claim {
        return Err("recursive terminal inner product".into());
    }
    let seeds: Vec<usize> = (0..QUERIES).map(|_| t.query(p.code(0) / 2)).collect();
    let initial_indices = query_indices(p, 0, &seeds);
    let mut originals = Vec::with_capacity(PARTIES);
    let mut masks = Vec::with_capacity(PARTIES);
    for party in 0..PARTIES {
        originals.push(transcript::verify_many(
            &original[party],
            &oracle_domain(kind, party, 0),
            p.code(0),
            &initial_indices,
            &proof.queries.original[party],
        )?);
        masks.push(transcript::verify_many(
            &mask[party],
            &oracle_domain(kind + 1, party, 0),
            p.code(0),
            &initial_indices,
            &proof.queries.mask[party],
        )?);
    }
    let mut folded: Vec<Vec<BTreeMap<usize, F>>> = Vec::with_capacity(p.rounds() - 1);
    for layer in 1..p.rounds() {
        let indices = query_indices(p, layer, &seeds);
        let values = (0..PARTIES)
            .map(|party| {
                transcript::verify_many(
                    &proof.rounds[layer - 1].next_roots[party],
                    &oracle_domain(kind, party, layer),
                    p.code(layer),
                    &indices,
                    &proof.queries.folded[layer - 1][party],
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        folded.push(values);
    }
    let inv2 = F::from(2u64).inverse().unwrap();
    let mut coset = F::GENERATOR;
    for round in 0..p.rounds() {
        let size = p.code(round);
        let d = domain(size)
            .map_err(str::to_owned)?
            .get_coset(coset)
            .ok_or("recursive query coset")?;
        for seed in &seeds {
            let index = seed & (size / 2 - 1);
            for party in 0..PARTIES {
                let current = |at: usize| {
                    if round == 0 {
                        originals[party][&at] + aggregation * masks[party][&at]
                    } else {
                        folded[round - 1][party][&at]
                    }
                };
                let left = current(index);
                let right = current(index + size / 2);
                let even = (left + right) * inv2;
                let odd = (left - right) * inv2 * d.element(index).inverse().unwrap();
                let next = if round + 1 == p.rounds() {
                    final_values[party][index]
                } else {
                    folded[round][party][&index]
                };
                if next != (F::from(1u64) - challenges[round]) * even + challenges[round] * odd {
                    return Err("recursive authenticated fold".into());
                }
            }
        }
        coset.square_in_place();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Pure algebraic protocol tests. This fixture intentionally has all test
    // coefficients and is NOT native MPC or an eligible end-to-end run.
    use super::*;
    use crate::transcript::Merkle;
    use ark_ff::UniformRand;
    use rand::rngs::OsRng;

    fn encode_coset(coefficients: &[F], layer: usize) -> Vec<F> {
        let mut shift = F::GENERATOR;
        for _ in 0..layer {
            shift.square_in_place();
        }
        let d = domain(coefficients.len() * 4)
            .unwrap()
            .get_coset(shift)
            .unwrap();
        d.fft(coefficients)
    }

    struct AlgebraFixture {
        p: Parameters,
        original: Vec<Merkle>,
        masks: Vec<Merkle>,
        original_coefficients: Vec<Vec<F>>,
        mask_coefficients: Vec<Vec<F>>,
        current: Vec<Vec<F>>,
        folded: Vec<Vec<Merkle>>,
        terminal: Vec<String>,
    }

    impl AlgebraFixture {
        fn new(p: Parameters) -> Self {
            let make = |is_mask: bool| {
                let mut evaluations: Vec<_> =
                    (0..PARTIES).map(|_| Vec::with_capacity(p.padded)).collect();
                for column in 0..p.padded {
                    let secret = if is_mask || column >= p.message {
                        F::rand(&mut OsRng)
                    } else {
                        F::from((column + 11) as u64)
                    };
                    let first = F::rand(&mut OsRng);
                    let second = F::rand(&mut OsRng);
                    for (party, row) in evaluations.iter_mut().enumerate() {
                        let x = F::from((party + 1) as u64);
                        row.push(secret + first * x + second * x * x);
                    }
                }
                evaluations
                    .into_iter()
                    .map(|row| domain(p.padded).unwrap().ifft(&row))
                    .collect::<Vec<_>>()
            };
            let original_coefficients = make(false);
            let mask_coefficients = make(true);
            let original = original_coefficients
                .iter()
                .enumerate()
                .map(|(party, row)| Merkle::new(oracle_domain(0, party, 0), encode_coset(row, 0)))
                .collect();
            let masks = mask_coefficients
                .iter()
                .enumerate()
                .map(|(party, row)| Merkle::new(oracle_domain(1, party, 0), encode_coset(row, 0)))
                .collect();
            Self {
                p,
                original,
                masks,
                original_coefficients,
                mask_coefficients,
                current: vec![],
                folded: vec![],
                terminal: vec![],
            }
        }
        fn joint(values: &[F]) -> F {
            F::from(3u64) * values[0] - F::from(3u64) * values[1] + values[2]
        }
    }

    impl DistributedOpening for AlgebraFixture {
        fn mask_claim(&mut self, weights: &[F]) -> Result<F, String> {
            Ok(Self::joint(
                &self
                    .mask_coefficients
                    .iter()
                    .map(|row| dot(row, weights))
                    .collect::<Vec<_>>(),
            ))
        }
        fn aggregate(&mut self, b: F) -> Result<(), String> {
            self.current = self
                .original_coefficients
                .iter()
                .zip(&self.mask_coefficients)
                .map(|(a, m)| a.iter().zip(m).map(|(a, m)| *a + b * m).collect())
                .collect();
            Ok(())
        }
        fn sumcheck(&mut self, _round: usize, weights: &[F]) -> Result<[F; 3], String> {
            Ok(std::array::from_fn(|at| {
                let at = F::from(at as u64);
                let weights = fold_table(weights, at);
                Self::joint(
                    &self
                        .current
                        .iter()
                        .map(|row| dot(&fold_table(row, at), &weights))
                        .collect::<Vec<_>>(),
                )
            }))
        }
        fn fold_commit(&mut self, round: usize, r: F) -> Result<Vec<String>, String> {
            self.current = self.current.iter().map(|row| fold_table(row, r)).collect();
            if round + 1 == self.p.rounds() {
                self.terminal = self
                    .current
                    .iter()
                    .map(|row| pack(&encode_coset(row, round + 1)))
                    .collect();
                Ok(vec![])
            } else {
                let trees: Vec<Merkle> = self
                    .current
                    .iter()
                    .enumerate()
                    .map(|(party, row)| {
                        Merkle::new(
                            oracle_domain(0, party, round + 1),
                            encode_coset(row, round + 1),
                        )
                    })
                    .collect();
                let roots = trees.iter().map(Merkle::root).collect();
                self.folded.push(trees);
                Ok(roots)
            }
        }
        fn terminal_rows(&mut self) -> Result<Vec<String>, String> {
            Ok(self.terminal.clone())
        }
        fn query_after_agreement(&mut self, plan: &QueryPlan) -> Result<Queries, String> {
            let initial = query_indices(plan.parameters, 0, &plan.seeds);
            Ok(Queries {
                original: self
                    .original
                    .iter()
                    .map(|tree| tree.open_many(&initial))
                    .collect::<Result<_, _>>()?,
                mask: self
                    .masks
                    .iter()
                    .map(|tree| tree.open_many(&initial))
                    .collect::<Result<_, _>>()?,
                folded: self
                    .folded
                    .iter()
                    .enumerate()
                    .map(|(i, layer)| {
                        let indices = query_indices(plan.parameters, i + 1, &plan.seeds);
                        layer
                            .iter()
                            .map(|tree| tree.open_many(&indices))
                            .collect::<Result<_, _>>()
                    })
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    #[test]
    fn recursive_algebra_and_authenticated_layers_reject_tampering() {
        let p = Parameters::for_message(8).unwrap();
        let weights: Vec<F> = (1u64..=8).map(F::from).collect();
        let message: Vec<F> = (11u64..19).map(F::from).collect();
        let claim = dot(&message, &weights);
        let mut fixture = AlgebraFixture::new(p);
        let original: Vec<String> = fixture.original.iter().map(Merkle::root).collect();
        let mask: Vec<String> = fixture.masks.iter().map(Merkle::root).collect();
        let initial = Transcript::new(b"algebra-fixture-not-native-acceptance");
        let mut transcript = initial.clone();
        let proof = prove(
            &mut fixture,
            &mut transcript,
            p,
            0,
            &original,
            &mask,
            &weights,
            claim,
        )
        .unwrap();
        assert_eq!(proof.terminal_rows[0].len(), TERMINAL * 4 * 64);
        for mutation in 0..5 {
            let mut bad = proof.clone();
            match mutation {
                0 => bad.rounds[0].sumcheck = pack(&[F::from(0u64); 3]),
                1 => bad.rounds[0].next_roots[0] = "00".repeat(64),
                2 => bad.queries.original[0].leaves[0].value = pack(&[F::from(0u64)]),
                3 => bad.queries.folded[0][0].siblings.push("00".repeat(64)),
                _ => bad.terminal_rows[0] = pack(&vec![F::from(0u64); TERMINAL * 4]),
            }
            assert!(verify(
                &mut initial.clone(),
                p,
                0,
                &original,
                &mask,
                &weights,
                claim,
                &bad
            )
            .is_err());
        }
        let mut changed = weights.clone();
        changed[0] += F::from(1u64);
        assert!(verify(
            &mut initial.clone(),
            p,
            0,
            &original,
            &mask,
            &changed,
            claim,
            &proof
        )
        .is_err());
        let mut changed = p;
        changed.padded *= 2;
        assert!(verify(
            &mut initial.clone(),
            changed,
            0,
            &original,
            &mask,
            &weights,
            claim,
            &proof
        )
        .is_err());
    }
}
