//! Fixed, public, bounded financial R1CS for the first whole-candidate smoke.
//! It is not an integration with an operational order or settlement store.
use crate::{
    algebra::eq_table,
    field::{encode, F},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

pub const PARTIES: usize = 7;
pub const CORRUPT: usize = 2;
pub const WITNESS: usize = 64;
pub const CONSTRAINTS: usize = 64;
pub const Z_LEN: usize = 128;
pub const R1: usize = 64;
pub const R2: usize = 66;
pub const R3: usize = 68;
pub const SUMCHECK_VARS: usize = 7;
pub const SUMCHECK_DEGREE: usize = 3;
pub const MASK_LEN: usize = 1 + SUMCHECK_VARS * SUMCHECK_DEGREE;
pub const MASK_MESSAGE: usize = 32;
pub const RESERVE: u64 = 65_535;

pub type Linear = Vec<(usize, F)>;

#[derive(Clone)]
pub struct Row {
    pub a: Linear,
    pub b: Linear,
    pub c: Linear,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Statement {
    pub protocol: String,
    pub deployment_mode: String,
    pub deployment_id: String,
    pub circuit_sha512: String,
    pub reserve: u64,
    pub parties: usize,
    pub max_corrupt: usize,
}

impl Statement {
    pub fn smoke(deployment_id: String) -> Self {
        Self {
            protocol: "cocode-normalized-malicious-mpc-research-v1".into(),
            deployment_mode: "pq-only".into(),
            deployment_id,
            circuit_sha512: fingerprint(),
            reserve: RESERVE,
            parties: PARTIES,
            max_corrupt: CORRUPT,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self != &Self::smoke(self.deployment_id.clone())
            || self.deployment_id.is_empty()
            || self.deployment_id.len() > 128
        {
            return Err("unsupported statement, circuit or deployment mode".into());
        }
        Ok(())
    }
}

pub fn rows() -> Vec<Row> {
    let one = F::from(1u64);
    let mut rows = vec![
        Row {
            a: vec![(1, one)],
            b: vec![(2, one)],
            c: vec![(3, one)],
        },
        Row {
            a: vec![(3, one), (4, one), (0, -F::from(RESERVE))],
            b: vec![(0, one)],
            c: vec![],
        },
    ];
    for (value, start, bits) in [(1, 5, 8), (2, 13, 8), (4, 21, 16)] {
        let mut a = vec![(value, one)];
        a.extend((0..bits).map(|bit| (start + bit, -F::from(1u64 << bit))));
        rows.push(Row {
            a,
            b: vec![(0, one)],
            c: vec![],
        });
        for bit in 0..bits {
            rows.push(Row {
                a: vec![(start + bit, one)],
                b: vec![(start + bit, one), (0, -one)],
                c: vec![],
            });
        }
    }
    rows.resize_with(CONSTRAINTS, || Row {
        a: vec![],
        b: vec![],
        c: vec![],
    });
    rows
}

pub fn fingerprint() -> String {
    let mut hash = Sha512::new();
    hash.update(b"zkfmi-financial-smoke-r1cs-v1");
    for row in rows() {
        for linear in [&row.a, &row.b, &row.c] {
            hash.update((linear.len() as u64).to_le_bytes());
            for (column, value) in linear {
                hash.update((*column as u64).to_le_bytes());
                hash.update(encode(*value));
            }
        }
    }
    hex::encode(hash.finalize())
}

/// Collapse the second Spartan relation to one general linear-functional
/// opening of a single joint commitment (w,r1,r2,r3). This intentionally uses
/// O(nnz) public verifier work rather than the paper's sparse-matrix optimization.
/// The random constant-column check binds w[0]=1 without revealing private w.
pub fn witness_opening_weights(point: &[F], batching: &[F]) -> Vec<F> {
    assert_eq!(point.len(), SUMCHECK_VARS);
    assert_eq!(batching.len(), 4);
    let beta = eq_table(&point[..6]);
    let pad = point[6];
    let mut weights = vec![F::from(0u64); Z_LEN];
    for (i, row) in rows().iter().enumerate() {
        for (matrix, coefficient) in [&row.a, &row.b, &row.c].into_iter().zip(batching) {
            for (column, value) in matrix {
                weights[*column] += (F::from(1u64) - pad) * beta[i] * coefficient * value;
            }
        }
    }
    for (offset, length, coefficient) in [
        (R1, 2, batching[0]),
        (R2, 2, batching[1]),
        (R3, 3, batching[2]),
    ] {
        for i in 0..length {
            weights[offset + i] += pad * beta[i] * coefficient;
        }
    }
    weights[0] += batching[3];
    weights
}

/// Libra-style degree-three separable mask, committed by its coefficient
/// vector. Its degree matches the complete masked constraint sumcheck.
pub fn mask_opening_weights(point: &[F]) -> Vec<F> {
    assert_eq!(point.len(), SUMCHECK_VARS);
    let mut weights = vec![F::from(0u64); MASK_MESSAGE];
    weights[0] = F::from(1u64);
    for (i, r) in point.iter().enumerate() {
        let mut power = *r;
        for degree in 0..SUMCHECK_DEGREE {
            weights[1 + i * SUMCHECK_DEGREE + degree] = power;
            power *= r;
        }
    }
    weights
}
