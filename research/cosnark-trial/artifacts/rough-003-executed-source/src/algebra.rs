//! Arithmetic adapters around arkworks, including the explicitly normalized
//! coCode fold. The original equation probe remains unchanged for provenance.
use crate::field::{decode, domain, encode, F};
use ark_ff::{Field,FftField};
use ark_poly::EvaluationDomain;

pub fn dot(a: &[F], b: &[F]) -> F {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(a,b)| *a * b).sum()
}

pub fn fold_table(table: &[F], r: F) -> Vec<F> {
    assert!(table.len().is_power_of_two() && table.len() >= 2);
    table.chunks_exact(2).map(|v| v[0] + r * (v[1] - v[0])).collect()
}

pub fn mle(table: &[F], point: &[F]) -> F {
    assert_eq!(table.len(), 1usize << point.len());
    point.iter().fold(table.to_vec(), |v, r| fold_table(&v, *r))[0]
}

/// Boolean equality table, first challenge corresponding to the low bit.
pub fn eq_table(point: &[F]) -> Vec<F> {
    (0..(1usize << point.len())).map(|i| {
        point.iter().enumerate().map(|(bit, r)| {
            if i & (1 << bit) == 0 { F::from(1u64) - r } else { *r }
        }).product()
    }).collect()
}

pub fn lagrange_at(nodes: &[F], at: F) -> Vec<F> {
    nodes.iter().enumerate().map(|(i, x)| {
        nodes.iter().enumerate().filter(|(j, _)| *j != i)
            .map(|(_, y)| (at - y) * (*x - y).inverse().expect("distinct nodes"))
            .product()
    }).collect()
}

pub fn small_polynomial(values_at_integers: &[F], at: F) -> F {
    let nodes: Vec<F> = (0..values_at_integers.len()).map(|i| F::from(i as u64)).collect();
    dot(values_at_integers, &lagrange_at(&nodes, at))
}

/// Row-local restriction compatible with a coefficient-table MLE:
/// (1-r) E(Y) + r O(Y), with no division by 1-r or exceptional challenge.
pub fn normalized_rs_fold(codeword: &[F], r: F) -> Result<Vec<F>, String> {
    if codeword.len() < 2 { return Err("short codeword".into()); }
    let d = domain(codeword.len()).map_err(str::to_owned)?.get_coset(F::GENERATOR).ok_or("invalid code coset")?;
    let inv2 = F::from(2u64).inverse().unwrap();
    let n = codeword.len() / 2;
    Ok((0..n).map(|i| {
        let even = (codeword[i] + codeword[i+n]) * inv2;
        let odd = (codeword[i] - codeword[i+n]) * inv2 * d.element(i).inverse().unwrap();
        (F::from(1u64)-r)*even + r*odd
    }).collect())
}

pub fn bounded_decode(codeword: &[F], bound: usize) -> Result<Vec<F>, String> {
    if bound > codeword.len() { return Err("invalid degree bound".into()); }
    let d=domain(codeword.len()).map_err(str::to_owned)?.get_coset(F::GENERATOR.square()).ok_or("invalid folded coset")?;
    let mut coefficients = d.ifft(codeword);
    if coefficients[bound..].iter().any(|x| *x != F::from(0u64)) {
        return Err("Reed-Solomon degree bound failed".into());
    }
    coefficients.truncate(bound);
    Ok(coefficients)
}

pub fn pack(values: &[F]) -> String {
    hex::encode(values.iter().flat_map(|v| encode(*v)).collect::<Vec<u8>>())
}

pub fn unpack(encoded: &str, exact: usize) -> Result<Vec<F>, String> {
    if encoded.len() != exact.checked_mul(64).ok_or("field length overflow")? {
        return Err("noncanonical field vector length".into());
    }
    let bytes = hex::decode(encoded).map_err(|_| "invalid field hex")?;
    bytes.chunks_exact(32).map(|b| decode(b.try_into().unwrap()).ok_or("noncanonical field element".into())).collect()
}
