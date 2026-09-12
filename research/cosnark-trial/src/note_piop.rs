//! Variable-size adaptation of the existing masked R1CS sumcheck, composed
//! with recursive coCode openings. A caller supplies the expected circuit and
//! statement independently of the proof. This is a research protocol, not a
//! claim of a new formal malicious-security/QROM composition theorem.
use crate::{
    algebra::{dot, eq_table, small_polynomial, unpack},
    field::F,
    note_circuit::{Circuit, Linear},
    recursive_pcs::{self, Parameters},
    transcript::Transcript,
};
use serde::{Deserialize, Serialize};

pub const PROTOCOL: &str = "zkfmi-private-note-r1cs-recursive-v1";

/// Both digests are independently pinned by the deployment/wallet launch, not
/// copied out of a submitted proof. Statement bytes bind the canonical parent,
/// operation, complete public ciphertexts, nullifiers and reservation heads.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Statement {
    pub protocol: String,
    pub circuit_sha512: String,
    pub application_statement_sha512: String,
    pub query_roster_sha512: String,
    pub session_sha512: String,
}

fn digest(value: &str) -> Result<(), String> {
    if value.len() != 128
        || hex::decode(value).is_err()
        || value
            .bytes()
            .any(|b| !b.is_ascii_digit() && !(b'a'..=b'f').contains(&b))
        || value.bytes().all(|b| b == b'0')
    {
        return Err("note statement canonical nonzero digest".into());
    }
    Ok(())
}

impl Statement {
    pub fn validate(&self, circuit: &Circuit) -> Result<(), String> {
        circuit.validate()?;
        if self.protocol != PROTOCOL || self.circuit_sha512 != hex::encode(circuit.fingerprint()) {
            return Err("note statement circuit or protocol mismatch".into());
        }
        for v in [
            &self.circuit_sha512,
            &self.application_statement_sha512,
            &self.query_roster_sha512,
            &self.session_sha512,
        ] {
            digest(v)?;
        }
        Ok(())
    }
}

pub struct Relation<'a> {
    pub circuit: &'a Circuit,
    pub constraints: usize,
    pub rounds: usize,
    pub witness: Parameters,
    pub mask: Parameters,
}

impl<'a> Relation<'a> {
    pub fn new(circuit: &'a Circuit) -> Result<Self, String> {
        circuit.validate()?;
        let constraints = circuit
            .constraints
            .len()
            .max(4)
            .checked_next_power_of_two()
            .ok_or("note constraint count overflow")?;
        let rows = constraints
            .checked_mul(2)
            .ok_or("note constraint row overflow")?;
        let rounds = rows.trailing_zeros() as usize;
        let witness = Parameters::for_message(
            circuit
                .variables
                .checked_add(7)
                .ok_or("note witness overflow")?,
        )?;
        let mask = Parameters::for_message(1 + rounds * 3)?;
        Ok(Self {
            circuit,
            constraints,
            rounds,
            witness,
            mask,
        })
    }
    pub fn blind(&self, which: usize) -> usize {
        self.circuit.variables + [0, 2, 4][which]
    }
    pub fn row_count(&self) -> usize {
        2 * self.constraints
    }
    pub fn state_size(&self) -> usize {
        2 * self.witness.padded + 2 * self.mask.padded + 3 * self.row_count()
    }
    pub fn oracle_offset(&self, kind: usize) -> Result<usize, String> {
        match kind {
            0 => Ok(0),
            1 => Ok(self.witness.padded),
            2 => Ok(2 * self.witness.padded),
            3 => Ok(2 * self.witness.padded + self.mask.padded),
            _ => Err("note oracle kind".into()),
        }
    }
    pub fn row_offset(&self, which: usize) -> usize {
        2 * self.witness.padded + 2 * self.mask.padded + which * self.row_count()
    }
    pub fn mask_weights(&self, point: &[F]) -> Result<Vec<F>, String> {
        if point.len() != self.rounds {
            return Err("note mask point shape".into());
        }
        let mut result = vec![F::from(0u64); self.mask.message];
        result[0] = F::from(1u64);
        for (i, r) in point.iter().enumerate() {
            let mut power = *r;
            for d in 0..3 {
                result[1 + 3 * i + d] = power;
                power *= r;
            }
        }
        Ok(result)
    }
    pub fn witness_weights(&self, point: &[F], batch: &[F]) -> Result<Vec<F>, String> {
        if point.len() != self.rounds || batch.len() != 4 {
            return Err("note witness point shape".into());
        }
        let beta = eq_table(&point[..self.rounds - 1]);
        let pad = point[self.rounds - 1];
        let mut result = vec![F::from(0u64); self.witness.message];
        for (i, constraint) in self.circuit.constraints.iter().enumerate() {
            let row = constraint.row();
            for (linear, b) in [&row.a, &row.b, &row.c].into_iter().zip(batch) {
                for (column, value) in linear {
                    result[*column] += (F::from(1u64) - pad) * beta[i] * b * value;
                }
            }
        }
        for (which, count) in [2, 2, 3].into_iter().enumerate() {
            for (i, beta_i) in beta.iter().enumerate().take(count) {
                result[self.blind(which) + i] += pad * beta_i * batch[which];
            }
        }
        result[0] += batch[3];
        Ok(result)
    }
    pub fn row_linear(&self, row: usize, which: usize) -> Result<Linear, String> {
        if row >= self.row_count() || which >= 3 {
            return Err("note matrix index".into());
        }
        if row < self.circuit.constraints.len() {
            let row = self.circuit.constraints[row].row();
            return Ok(match which {
                0 => row.a,
                1 => row.b,
                _ => row.c,
            });
        }
        if row >= self.constraints && row - self.constraints < [2, 2, 3][which] {
            return Ok(vec![(
                self.blind(which) + row - self.constraints,
                F::from(1u64),
            )]);
        }
        Ok(vec![])
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub statement: Statement,
    /// Four oracle kinds, each containing all seven ordered party roots.
    pub roots: Vec<Vec<String>>,
    pub initial_mask_sums: String,
    pub sumcheck_rounds: Vec<String>,
    pub endpoint_claims: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proof {
    pub prefix: Prefix,
    pub mask_opening: recursive_pcs::Proof,
    pub witness_opening: recursive_pcs::Proof,
    pub transcript_sha512: String,
}

pub fn start(
    expected: &Statement,
    circuit: &Circuit,
    roots: &[Vec<String>],
) -> Result<Transcript, String> {
    expected.validate(circuit)?;
    if roots.len() != 4 || roots.iter().any(|r| r.len() != 7) {
        return Err("note root roster shape".into());
    }
    for root in roots.iter().flatten() {
        digest(root)?;
    }
    let mut t =
        Transcript::new(&serde_json::to_vec(expected).map_err(|_| "note statement encoding")?);
    t.absorb(
        b"note-root-roster",
        &serde_json::to_vec(roots).map_err(|_| "note roots encoding")?,
    );
    Ok(t)
}

/// Not constructible from caller-provided weights or a claimed 'verified'
/// flag. Owners use the independently checked prefix to authorize openings.
pub struct OpeningContext {
    pub(crate) transcript: Transcript,
    pub(crate) parameters: Parameters,
    pub(crate) kind: usize,
    pub(crate) weights: Vec<F>,
    pub(crate) claim: F,
}

pub fn opening_context(
    expected: &Statement,
    circuit: &Circuit,
    prefix: &Prefix,
    prior_mask: Option<&recursive_pcs::Proof>,
) -> Result<OpeningContext, String> {
    if &prefix.statement != expected {
        return Err("note external statement mismatch".into());
    }
    let relation = Relation::new(circuit)?;
    let mut t = start(expected, circuit, &prefix.roots)?;
    let mut tau = t.fields(b"constraint-weight", relation.rounds - 1);
    let sums = unpack(&prefix.initial_mask_sums, 2)?;
    t.absorb_fields(b"initial-mask-sums", &sums);
    let pad = t.field(b"constraint-padding");
    tau.push(pad);
    let batch = t.field(b"sumcheck-mask-batch");
    let mut claim = pad * sums[0] + batch * sums[1];
    if prefix.sumcheck_rounds.len() != relation.rounds {
        return Err("note constraint round count".into());
    }
    let mut point = Vec::with_capacity(relation.rounds);
    for round in &prefix.sumcheck_rounds {
        let h = unpack(round, 4)?;
        if h[0] + h[1] != claim {
            return Err("note constraint sumcheck identity".into());
        }
        t.absorb_fields(b"constraint-h", &h);
        let r = t.field(b"constraint-round");
        claim = small_polynomial(&h, r);
        point.push(r);
    }
    let endpoints = unpack(&prefix.endpoint_claims, 4)?;
    let equality: F = tau
        .iter()
        .zip(&point)
        .map(|(a, b)| *a * b + (F::from(1u64) - a) * (F::from(1u64) - b))
        .product();
    if claim != equality * (endpoints[0] * endpoints[1] - endpoints[2]) + batch * endpoints[3] {
        return Err("note constraint endpoint identity".into());
    }
    t.absorb_fields(b"constraint-endpoints", &endpoints);
    let weights = relation.mask_weights(&point)?;
    if let Some(mask) = prior_mask {
        recursive_pcs::verify(
            &mut t,
            relation.mask,
            2,
            &prefix.roots[2],
            &prefix.roots[3],
            &weights,
            endpoints[3],
            mask,
        )?;
        let batching = t.fields(b"linear-relation-batch", 4);
        Ok(OpeningContext {
            transcript: t,
            parameters: relation.witness,
            kind: 0,
            weights: relation.witness_weights(&point, &batching)?,
            claim: dot(&endpoints[..3], &batching[..3]) + batching[3],
        })
    } else {
        Ok(OpeningContext {
            transcript: t,
            parameters: relation.mask,
            kind: 2,
            weights,
            claim: endpoints[3],
        })
    }
}

pub fn verify(expected: &Statement, circuit: &Circuit, proof: &Proof) -> Result<(), String> {
    let mut context = opening_context(expected, circuit, &proof.prefix, Some(&proof.mask_opening))?;
    recursive_pcs::verify(
        &mut context.transcript,
        context.parameters,
        context.kind,
        &proof.prefix.roots[0],
        &proof.prefix.roots[1],
        &context.weights,
        context.claim,
        &proof.witness_opening,
    )?;
    if proof.transcript_sha512 != hex::encode(context.transcript.view()) {
        return Err("note final transcript mismatch".into());
    }
    Ok(())
}
