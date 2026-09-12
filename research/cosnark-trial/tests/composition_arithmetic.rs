//! Public-fixture unit checks only. Native private end-to-end acceptance is
//! recorded separately by the full runner; these tests do not replace it.
use ark_ff::{FftField, Field};
use ark_poly::EvaluationDomain;
use zkfmi_cosnark_trial::{
    algebra::{bounded_decode, dot, eq_table, fold_table, mle, normalized_rs_fold},
    circuit::{self, Linear},
    field::{domain, F},
    pcs,
    transcript::{self, Merkle, Transcript},
};

fn values(n: usize) -> Vec<F> {
    (0..n)
        .map(|i| F::from((i * i + 3 * i + 7) as u64))
        .collect()
}

#[test]
fn equality_table_and_low_bit_restriction_are_dual() {
    let table = values(16);
    let point = values(4);
    assert_eq!(mle(&table, &point), dot(&table, &eq_table(&point)));
    assert_eq!(eq_table(&point).iter().copied().sum::<F>(), F::from(1u64));
}

#[test]
fn coset_fold_matches_coefficient_restriction_including_endpoints() {
    let coefficients = values(16);
    let mut padded = coefficients.clone();
    padded.resize(64, F::from(0u64));
    let code = domain(64)
        .unwrap()
        .get_coset(F::GENERATOR)
        .unwrap()
        .fft(&padded);
    for challenge in [F::from(0u64), F::from(1u64), F::from(13u64)] {
        let folded = normalized_rs_fold(&code, challenge).unwrap();
        assert_eq!(
            bounded_decode(&folded, 8).unwrap(),
            fold_table(&coefficients, challenge)
        );
        assert!(bounded_decode(&folded, 7).is_err());
    }
}

#[test]
fn message_and_coefficient_linear_functionals_match() {
    let message = values(pcs::PADDED);
    let weights = values(128);
    let coefficients = domain(pcs::PADDED).unwrap().ifft(&message);
    assert_eq!(
        dot(&weights, &message[..128]),
        dot(&pcs::coefficient_weights(&weights), &coefficients)
    );
    let h0 = dot(
        &pcs::reply_message_weights(&weights, F::from(0u64)),
        &message,
    );
    let h1 = dot(
        &pcs::reply_message_weights(&weights, F::from(1u64)),
        &message,
    );
    assert_eq!(h0 + h1, dot(&weights, &message[..128]));
}

#[test]
fn separable_mask_boolean_sum_has_declared_scaling() {
    let coefficients = values(circuit::MASK_MESSAGE);
    let actual = (0..128)
        .map(|index| {
            let point = (0..7)
                .map(|bit| F::from(((index >> bit) & 1) as u64))
                .collect::<Vec<_>>();
            dot(&coefficients, &circuit::mask_opening_weights(&point))
        })
        .sum::<F>();
    let expected = coefficients[0] * F::from(128u64)
        + coefficients[1..circuit::MASK_LEN]
            .iter()
            .copied()
            .sum::<F>()
            * F::from(64u64);
    assert_eq!(actual, expected);
}

fn financial_fixture() -> Vec<F> {
    let mut z = vec![F::from(0u64); circuit::Z_LEN];
    let amount = 37u64;
    let price = 53u64;
    let notional = amount * price;
    let remainder = circuit::RESERVE - notional;
    z[..5].copy_from_slice(&[1, amount, price, notional, remainder].map(F::from));
    for (value, start, bits) in [(amount, 5, 8), (price, 13, 8), (remainder, 21, 16)] {
        for bit in 0..bits {
            z[start + bit] = F::from((value >> bit) & 1);
        }
    }
    for (i, value) in z[circuit::R1..circuit::R3 + 3].iter_mut().enumerate() {
        *value = F::from((i + 5) as u64);
    }
    z
}

fn linear(row: &Linear, witness: &[F]) -> F {
    row.iter().map(|(i, value)| witness[*i] * value).sum()
}

#[test]
fn financial_relation_detects_inconsistent_notional() {
    let mut z = financial_fixture();
    let valid = |z: &[F]| {
        circuit::rows()
            .iter()
            .all(|row| linear(&row.a, z) * linear(&row.b, z) == linear(&row.c, z))
    };
    assert!(valid(&z));
    z[3] += F::from(1u64);
    assert!(!valid(&z));
}

#[test]
fn collapsed_matrix_relation_binds_constant_and_padding_masks() {
    let z = financial_fixture();
    let point = values(7);
    let batch = values(4);
    let mut endpoint = Vec::new();
    for (matrix, offset, length) in [
        (0, circuit::R1, 2),
        (1, circuit::R2, 2),
        (2, circuit::R3, 3),
    ] {
        let mut table = circuit::rows()
            .iter()
            .map(|row| linear([&row.a, &row.b, &row.c][matrix], &z))
            .collect::<Vec<_>>();
        table.resize(128, F::from(0u64));
        table[64..64 + length].copy_from_slice(&z[offset..offset + length]);
        endpoint.push(mle(&table, &point));
    }
    assert_eq!(
        dot(&circuit::witness_opening_weights(&point, &batch), &z),
        dot(&endpoint, &batch[..3]) + batch[3]
    );
}

#[test]
fn merkle_opening_binds_index_domain_value_and_salt() {
    let tree = Merkle::new(b"public-unit-oracle".to_vec(), values(16));
    let opening = tree.open(5);
    assert_eq!(
        transcript::verify(&tree.root(), tree.domain(), 16, 5, &opening).unwrap(),
        values(16)[5]
    );
    assert!(transcript::verify(&tree.root(), tree.domain(), 16, 6, &opening).is_err());
    assert!(transcript::verify(&tree.root(), b"different-oracle", 16, 5, &opening).is_err());
    let mut changed = opening.clone();
    changed.salt = "00".repeat(64);
    assert!(transcript::verify(&tree.root(), tree.domain(), 16, 5, &changed).is_err());
    changed = opening;
    changed.value = "ff".repeat(32);
    assert!(transcript::verify(&tree.root(), tree.domain(), 16, 5, &changed).is_err());
}

#[test]
fn transcript_binds_statement_and_message_boundaries() {
    let mut first = Transcript::new(b"instance-one");
    let mut second = Transcript::new(b"instance-two");
    assert_ne!(first.field(b"challenge"), second.field(b"challenge"));
    let mut first = Transcript::new(b"same");
    let mut second = first.clone();
    first.absorb(b"ab", b"c");
    second.absorb(b"a", b"bc");
    assert_ne!(first.view(), second.view());
    assert_ne!(F::GENERATOR.pow([pcs::CODE as u64]), F::from(1u64));
}
