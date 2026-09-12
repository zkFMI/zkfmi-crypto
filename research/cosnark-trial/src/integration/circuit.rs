//! Four ordered private balances: asset seller/buyer, cash buyer/seller.
use super::SettlementStatement;
use crate::{
    algebra::eq_table,
    field::{encode, F},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

pub const LEGACY_PROTOCOL: &str = "cocode-defmi-private-dvp-research-v1";
pub const QUERY_AGREEMENT_PROTOCOL: &str =
    "cocode-defmi-private-dvp-research-v2-hybrid-query-agreement";
pub const PARTIES: usize = 7;
pub const CORRUPT: usize = 2;
pub const WITNESS: usize = 1024;
pub const CONSTRAINTS: usize = 1024;
pub const Z_LEN: usize = 2048;
pub const R1: usize = 1024;
pub const R2: usize = 1026;
pub const R3: usize = 1028;
pub const SUMCHECK_VARS: usize = 11;
pub const SUMCHECK_DEGREE: usize = 3;
pub const MASK_LEN: usize = 1 + SUMCHECK_VARS * 3;
pub const MASK_MESSAGE: usize = 64;
// w1..4 old balances, w5..8 new balances, w9 q, w10 p, w11 q*p;
// w12..13 old two-use claim blinds, w14..15 new blinds, w16 fill flag.
pub type Linear = Vec<(usize, F)>;
#[derive(Clone)]
pub struct Row {
    pub a: Linear,
    pub b: Linear,
    pub c: Linear,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Statement {
    pub protocol: String,
    pub deployment_mode: String,
    pub circuit_sha512: String,
    pub parties: usize,
    pub max_corrupt: usize,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub query_roster_sha512: String,
    pub settlement: SettlementStatement,
}
impl Statement {
    pub fn new(settlement: SettlementStatement) -> Self {
        Self {
            protocol: LEGACY_PROTOCOL.into(),
            deployment_mode: "pq-only-research".into(),
            circuit_sha512: fingerprint(settlement.no_fill),
            parties: PARTIES,
            max_corrupt: CORRUPT,
            query_roster_sha512: String::new(),
            settlement,
        }
    }

    pub fn new_query_agreement(
        settlement: SettlementStatement,
        query_roster_sha512: &str,
    ) -> Result<Self, String> {
        validate_sha512(query_roster_sha512)?;
        Ok(Self {
            protocol: QUERY_AGREEMENT_PROTOCOL.into(),
            deployment_mode: "pq-only-research".into(),
            circuit_sha512: fingerprint(settlement.no_fill),
            parties: PARTIES,
            max_corrupt: CORRUPT,
            query_roster_sha512: query_roster_sha512.into(),
            settlement,
        })
    }

    pub fn validate_legacy(&self) -> Result<(), String> {
        self.settlement.validate()?;
        if self != &Self::new(self.settlement.clone()) {
            return Err("unsupported legacy integration statement".into());
        }
        Ok(())
    }

    pub fn validate_query_agreement(&self, trusted_roster_sha512: &str) -> Result<(), String> {
        self.settlement.validate()?;
        validate_sha512(trusted_roster_sha512)?;
        let expected = Self::new_query_agreement(self.settlement.clone(), trusted_roster_sha512)?;
        if self != &expected {
            return Err("unsupported query-agreement integration statement".into());
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.protocol.as_str() {
            LEGACY_PROTOCOL => self.validate_legacy(),
            QUERY_AGREEMENT_PROTOCOL => {
                let roster = self.query_roster_sha512.clone();
                self.validate_query_agreement(&roster)
            }
            _ => Err("unsupported integration protocol".into()),
        }
    }
}

fn validate_sha512(value: &str) -> Result<(), String> {
    if value.len() != 128 {
        return Err("invalid query roster SHA-512".into());
    }
    let decoded = hex::decode(value).map_err(|_| "invalid query roster SHA-512")?;
    if hex::encode(decoded) != value {
        return Err("invalid query roster SHA-512".into());
    }
    Ok(())
}
pub fn rows(no_fill: bool) -> Vec<Row> {
    let o = F::from(1u64);
    let mut r = vec![Row {
        a: vec![(9, o)],
        b: vec![(10, o)],
        c: vec![(11, o)],
    }];
    for (old, new, quantity, sign) in [(1, 5, 9, o), (2, 6, 9, -o), (3, 7, 11, o), (4, 8, 11, -o)] {
        r.push(Row {
            a: vec![(quantity, sign)],
            b: vec![(16, o)],
            c: vec![(old, o), (new, -o)],
        });
    }
    r.push(Row {
        a: vec![(16, o), (0, -F::from(u64::from(!no_fill)))],
        b: vec![(0, o)],
        c: vec![],
    });
    for value in 1..=11 {
        let start = 32 + (value - 1) * 64;
        let mut a = vec![(value, o)];
        a.extend((0..64).map(|b| (start + b, -F::from(1u64 << b))));
        r.push(Row {
            a,
            b: vec![(0, o)],
            c: vec![],
        });
        for b in 0..64 {
            r.push(Row {
                a: vec![(start + b, o)],
                b: vec![(start + b, o), (0, -o)],
                c: vec![],
            });
        }
    }
    r.resize_with(CONSTRAINTS, || Row {
        a: vec![],
        b: vec![],
        c: vec![],
    });
    r
}
pub fn fingerprint(no_fill: bool) -> String {
    let mut h = Sha512::new();
    h.update(b"zkfmi-four-account-u64-dvp-r1cs-v1");
    for r in rows(no_fill) {
        for l in [&r.a, &r.b, &r.c] {
            h.update((l.len() as u64).to_le_bytes());
            for (i, v) in l {
                h.update((*i as u64).to_le_bytes());
                h.update(encode(*v));
            }
        }
    }
    hex::encode(h.finalize())
}
pub fn witness_opening_weights(point: &[F], batch: &[F], no_fill: bool) -> Vec<F> {
    let beta = eq_table(&point[..10]);
    let pad = point[10];
    let mut w = vec![F::from(0u64); Z_LEN];
    for (i, r) in rows(no_fill).iter().enumerate() {
        for (l, b) in [&r.a, &r.b, &r.c].into_iter().zip(batch) {
            for (c, v) in l {
                w[*c] += (F::from(1u64) - pad) * beta[i] * b * v;
            }
        }
    }
    for (o, n, b) in [(R1, 2, batch[0]), (R2, 2, batch[1]), (R3, 3, batch[2])] {
        for i in 0..n {
            w[o + i] += pad * beta[i] * b;
        }
    }
    w[0] += batch[3];
    w
}
pub fn mask_opening_weights(point: &[F]) -> Vec<F> {
    let mut w = vec![F::from(0u64); MASK_MESSAGE];
    w[0] = F::from(1u64);
    for (i, r) in point.iter().enumerate() {
        let mut p = *r;
        for d in 0..3 {
            w[1 + 3 * i + d] = p;
            p *= r;
        }
    }
    w
}
pub fn bind_book_weights(w: &mut [F], old: &[F], new: &[F], batch: &[F]) {
    for j in 0..4 {
        w[1 + j] += batch[0] * old[j];
        w[5 + j] += batch[1] * new[j];
    }
    for j in 0..2 {
        w[12 + j] += batch[0] * old[4 + j];
        w[14 + j] += batch[1] * new[4 + j];
    }
}
