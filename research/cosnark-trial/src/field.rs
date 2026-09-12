//! Finite-field and Reed-Solomon arithmetic from arkworks 0.5.0.
//!
//! Uses only the BN254 scalar *field*, not a curve, pairing, commitment or
//! discrete-log assumption. Source: https://github.com/arkworks-rs/algebra
//! and https://github.com/arkworks-rs/curves, MIT/Apache-2.0.
use ark_ff::{BigInteger, PrimeField};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

pub type F = ark_bn254::Fr;

pub fn encode(value: F) -> [u8; 32] {
    let bytes = value.into_bigint().to_bytes_le();
    let mut out = [0; 32];
    out[..bytes.len()].copy_from_slice(&bytes);
    out
}

/// Reject non-canonical encodings rather than reducing attacker input modulo p.
pub fn decode(bytes: [u8; 32]) -> Option<F> {
    let value = F::from_le_bytes_mod_order(&bytes);
    (encode(value) == bytes).then_some(value)
}

pub fn domain(size: usize) -> Result<Radix2EvaluationDomain<F>, &'static str> {
    if !size.is_power_of_two() {
        return Err("domain size is not a power of two");
    }
    let domain = Radix2EvaluationDomain::new(size).ok_or("unsupported domain")?;
    if domain.size() != size {
        return Err("domain size changed");
    }
    Ok(domain)
}

/// Interpolate the complete, already padded local row. Padding randomness must
/// come from the declared distributed functionality, not this arithmetic layer.
pub fn interpolate_row(values: &[F]) -> Result<Vec<F>, &'static str> {
    Ok(domain(values.len())?.ifft(values))
}

pub fn encode_row(coefficients: &[F], size: usize) -> Result<Vec<F>, &'static str> {
    if coefficients.len() > size {
        return Err("polynomial exceeds evaluation domain");
    }
    let mut padded = coefficients.to_vec();
    padded.resize(size, F::from(0u64));
    domain(size)?.fft_in_place(&mut padded);
    Ok(padded)
}
