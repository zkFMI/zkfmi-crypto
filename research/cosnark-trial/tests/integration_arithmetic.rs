//! Public fixtures for deterministic arithmetic/schema checks only. These are
//! unit aids; acceptance uses actual native MPC and canonical State.apply.
use zkfmi_cosnark_trial::{
    algebra::dot,
    field::F,
    integration::{
        circuit::{self, Statement},
        SettlementStatement,
    },
};

fn witness(old: [u128; 4], new: [u128; 4], q: u128, p: u128, no_fill: bool) -> Vec<F> {
    let mut w = vec![F::from(0u64); circuit::Z_LEN];
    w[0] = F::from(1u64);
    let values: Vec<u128> = old.into_iter().chain(new).chain([q, p, q * p]).collect();
    for (i, v) in values.into_iter().enumerate() {
        w[1 + i] = F::from(v);
        for bit in 0..64 {
            w[32 + 64 * i + bit] = F::from(((v >> bit) & 1) as u64);
        }
    }
    w[16] = F::from(u64::from(!no_fill));
    w
}
fn valid(w: &[F], no_fill: bool) -> bool {
    let linear = |l: &circuit::Linear| l.iter().map(|(i, c)| w[*i] * c).sum::<F>();
    circuit::rows(no_fill)
        .iter()
        .all(|r| linear(&r.a) * linear(&r.b) == linear(&r.c))
}
fn statement() -> SettlementStatement {
    SettlementStatement {
        deployment_id: "unit-deployment".into(),
        book_id: "book".into(),
        operation_id: "op".into(),
        sequence: 1,
        before_commitment: "a".repeat(128),
        after_commitment: "b".repeat(128),
        no_fill: false,
    }
}

#[test]
fn full_u64_boundaries_and_atomic_conservation() {
    let m = u64::MAX as u128;
    assert!(valid(
        &witness([m, 0, m, 0], [m - 1, 1, m - 1, 1], 1, 1, false),
        false
    ));
}
#[test]
fn no_fill_preserves_balances_even_with_high_bit_quantities() {
    let old = [7, 8, 9, 10];
    assert!(valid(&witness(old, old, 1 << 63, 1, true), true));
    assert!(!valid(&witness(old, [6, 9, 8, 11], 1, 1, true), true));
}
#[test]
fn product_overflow_is_not_accepted_as_modular_integer_arithmetic() {
    let old = [7, 8, 9, 10];
    assert!(!valid(&witness(old, old, u64::MAX as u128, 2, true), true));
}
#[test]
fn negative_balance_cannot_masquerade_as_u64_max() {
    let mut w = witness([0, 0, 1, 0], [u64::MAX as u128, 1, 0, 1], 1, 1, false);
    w[5] = -F::from(1u64);
    assert!(!valid(&w, false));
}
#[test]
fn non_boolean_bits_and_wrong_product_are_rejected() {
    let mut w = witness([10, 0, 20, 0], [8, 2, 14, 6], 2, 3, false);
    assert!(valid(&w, false));
    w[32] = F::from(2u64);
    assert!(!valid(&w, false));
    let mut w = witness([10, 0, 20, 0], [8, 2, 14, 6], 2, 3, false);
    w[11] += F::from(1u64);
    assert!(!valid(&w, false));
}
#[test]
fn public_fill_flag_is_part_of_the_bound_circuit() {
    assert_ne!(circuit::fingerprint(false), circuit::fingerprint(true));
    let old = [7, 8, 9, 10];
    assert!(!valid(&witness(old, old, 1, 1, true), false));
}
#[test]
fn book_linear_claims_bind_the_same_balance_and_blind_coordinates() {
    let mut w = witness([10, 0, 20, 0], [8, 2, 14, 6], 2, 3, false);
    for (i, v) in [101, 103, 107, 109].into_iter().enumerate() {
        w[12 + i] = F::from(v as u64);
    }
    let old: Vec<F> = (2u64..8).map(F::from).collect();
    let new: Vec<F> = (11u64..17).map(F::from).collect();
    let batch = [F::from(17u64), F::from(19u64)];
    let mut weights = vec![F::from(0u64); circuit::Z_LEN];
    circuit::bind_book_weights(&mut weights, &old, &new, &batch);
    let before: Vec<F> = w[1..5].iter().chain(&w[12..14]).copied().collect();
    let after: Vec<F> = w[5..9].iter().chain(&w[14..16]).copied().collect();
    assert_eq!(
        dot(&weights, &w),
        batch[0] * dot(&old, &before) + batch[1] * dot(&new, &after)
    );
}
#[test]
fn unsupported_modes_sequences_and_noncanonical_commitments_fail_closed() {
    let expected = statement();
    expected.validate().unwrap();
    let mut s = Statement::new(expected.clone());
    s.validate().unwrap();
    s.deployment_mode = "classical".into();
    assert!(s.validate().is_err());
    let mut s = expected.clone();
    s.sequence = 0;
    assert!(s.validate().is_err());
    let mut s = expected.clone();
    s.sequence = u64::MAX;
    assert!(s.validate().is_err());
    let mut s = expected;
    s.after_commitment = "A".repeat(128);
    assert!(s.validate().is_err());
}

#[test]
fn legacy_and_query_agreement_protocols_are_explicit_and_not_interchangeable() {
    let legacy = Statement::new(statement());
    legacy.validate_legacy().unwrap();
    let legacy_json = serde_json::to_value(&legacy).unwrap();
    assert!(legacy_json.get("query_roster_sha512").is_none());
    assert!(legacy.validate_query_agreement(&"11".repeat(64)).is_err());

    let v2 = Statement::new_query_agreement(statement(), &"11".repeat(64)).unwrap();
    v2.validate_query_agreement(&"11".repeat(64)).unwrap();
    assert!(v2.validate_query_agreement(&"22".repeat(64)).is_err());
    assert!(v2.validate_legacy().is_err());
    assert_ne!(legacy.protocol, v2.protocol);
}
