//! Rust-owned, variable-size adapter to the pinned official MP-SPDZ engine.
//! Secret inputs are degree-two shares supplied separately by seven workers.
//! This module only emits public programs; it has no plaintext witness API.
use crate::{
    field::{domain, F},
    note_circuit::{Circuit, Constraint, Wire},
    note_piop::Relation,
    recursive_pcs::Parameters,
};
use ark_ff::{BigInteger, PrimeField};
use ark_poly::EvaluationDomain;
use std::fmt::Write;

pub struct Layout {
    pub parameters: Parameters,
    pub input_fields: usize,
    pub check_fields: usize,
    pub stored_fields: usize,
}

impl Layout {
    pub fn for_circuit(circuit: &Circuit) -> Result<Self, String> {
        let relation = Relation::new(circuit)?;
        let checks = circuit
            .constraints
            .iter()
            .filter(|constraint| {
                matches!(constraint, Constraint::Boolean(_) | Constraint::Equal(_, _))
            })
            .count();
        Ok(Self {
            parameters: relation.witness,
            input_fields: circuit.inputs.len(),
            check_fields: checks
                .checked_add(
                    circuit
                        .inputs
                        .len()
                        .checked_mul(4)
                        .ok_or("note input shape overflow")?,
                )
                .ok_or("note check shape overflow")?,
            stored_fields: relation.state_size(),
        })
    }
    pub fn state_size(&self) -> usize {
        self.stored_fields
    }
    pub fn entropy_fields(&self) -> usize {
        self.state_size() + self.check_fields
    }
}

fn base() -> String {
    String::from("from Compiler.types import sint, cint, Array\nfrom Compiler.library import print_ln, runtime_error_if\nprogram.bit_length = 64\nprogram.set_security(128)\nassert program.security == 128\nviews = [[sint.get_input_from(p).reveal() for j in range(8)] for p in range(7)]\nruntime_error_if(sum(views[p][j] != views[0][j] for p in range(1,7) for j in range(8)) != 0, 'challenge view mismatch')\n")
}

fn wire(w: Wire) -> String {
    match w {
        Wire::Constant(bit) => format!("sint({})", u8::from(bit)),
        Wire::Variable(i) => format!("state[{i}]"),
    }
}

/// Validate input sharing and all asserted bits/equalities with independent
/// secret uniform field masks, contributed by every worker. Only a Boolean
/// failure is revealed. Gate rows hold by construction in malicious MPC.
pub fn initialize(circuit: &Circuit) -> Result<String, String> {
    let layout = Layout::for_circuit(circuit)?;
    let relation = Relation::new(circuit)?;
    let mut s = base();
    writeln!(
        s,
        "# Exact public circuit SHA-512: {}",
        hex::encode(circuit.fingerprint())
    )
    .unwrap();
    writeln!(
        s,
        "parts = [[sint.get_input_from(p) for j in range({})] for p in range(7)]",
        layout.input_fields
    )
    .unwrap();
    writeln!(s, "entropy = Array({}, sint)\nentropy.assign_vector(sum(sint.get_input_from(p, size={}) for p in range(7)))", layout.entropy_fields(), layout.entropy_fields()).unwrap();
    writeln!(s, "state = Array({}, sint)\nstate.assign_vector(entropy.get_vector(base=0, size={}))\nstate[0] = 1\ncheck = sint(0)", layout.state_size(), layout.state_size()).unwrap();
    let mut check = layout.state_size();
    for x in 4..=7 {
        let a = (x - 2) * (x - 3) / 2;
        let b = -((x - 1) * (x - 3));
        let c = (x - 1) * (x - 2) / 2;
        writeln!(s, "for j in range({}):\n    check += (parts[{}][j] - ({a}*parts[0][j] + {b}*parts[1][j] + {c}*parts[2][j])) * entropy[{check}+j]", layout.input_fields, x-1).unwrap();
        check += layout.input_fields;
    }
    for (j, input) in circuit.inputs.iter().enumerate() {
        writeln!(
            s,
            "{} = 3*parts[0][{j}] - 3*parts[1][{j}] + parts[2][{j}]",
            wire(*input)
        )
        .unwrap();
    }
    for constraint in &circuit.constraints {
        match *constraint {
            Constraint::And { a, b, out } => {
                writeln!(s, "{} = {} * {}", wire(out), wire(a), wire(b)).unwrap()
            }
            Constraint::Xor { a, b, out } => writeln!(
                s,
                "{} = {} + {} - 2*{}*{}",
                wire(out),
                wire(a),
                wire(b),
                wire(a),
                wire(b)
            )
            .unwrap(),
            Constraint::Not { input, out } => {
                writeln!(s, "{} = 1 - {}", wire(out), wire(input)).unwrap()
            }
            Constraint::Boolean(a) => {
                writeln!(
                    s,
                    "check += {} * ({} - 1) * entropy[{check}]",
                    wire(a),
                    wire(a)
                )
                .unwrap();
                check += 1;
            }
            Constraint::Equal(a, b) => {
                writeln!(s, "check += ({} - {}) * entropy[{check}]", wire(a), wire(b)).unwrap();
                check += 1;
            }
            Constraint::PackedProduct {
                ref a,
                ref b,
                ref out,
            }
            | Constraint::PackedAdd {
                ref a,
                ref b,
                ref out,
            } => {
                let packed = |bits: &[Wire]| {
                    bits.iter()
                        .enumerate()
                        .map(|(i, bit)| format!("({} * {})", wire(*bit), 1u128 << i))
                        .collect::<Vec<_>>()
                        .join(" + ")
                };
                let operation = if matches!(constraint, Constraint::PackedProduct { .. }) {
                    "*"
                } else {
                    "+"
                };
                // The validated full result has <=64 bits. Keep the existing
                // 128-bit statistical security; do not decompose 128-bit words
                // with an undersized prime/masking interval.
                writeln!(
                    s,
                    "integer_bits = (({}) {operation} ({})).bit_decompose({})",
                    packed(a),
                    packed(b),
                    out.len()
                )
                .unwrap();
                for (i, output) in out.iter().enumerate() {
                    writeln!(s, "{} = integer_bits[{i}]", wire(*output)).unwrap();
                }
            }
        }
    }
    if check != layout.entropy_fields() {
        return Err("note native entropy layout".into());
    }
    // The seven witness blinding coordinates and polynomial-mask coefficients
    // retain independent entropy. Native matrix rows are not public outputs.
    for which in 0..3 {
        let offset = relation.row_offset(which);
        writeln!(
            s,
            "for i in range({}):\n    state[{offset}+i] = 0",
            relation.row_count()
        )
        .unwrap();
        for row in
            (0..circuit.constraints.len()).chain(relation.constraints..relation.constraints + 3)
        {
            let terms = relation.row_linear(row, which)?;
            if !terms.is_empty() {
                writeln!(s, "state[{}] = {}", offset + row, linear(&terms)).unwrap();
            }
        }
    }
    let mut exponent = F::MODULUS;
    exponent.sub_with_borrow(&<F as PrimeField>::BigInt::from(1u64));
    writeln!(s, "runtime_error_if((check ** {exponent}).reveal() != 0, 'invalid private note circuit')\nsint.write_to_file(state.get_vector(), position=0)\nprint_ln('TRIAL_READY')").unwrap();
    Ok(s)
}

/// Transpose the exact arkworks inverse FFT. The native persistence contains
/// message evaluations, while recursive PCS works on polynomial coefficients.
fn message_functional(coefficient_weights: &[F]) -> Result<Vec<F>, String> {
    Ok(domain(coefficient_weights.len())
        .map_err(str::to_owned)?
        .ifft(coefficient_weights))
}

/// Lift a functional on a folded coefficient vector to the original vector.
/// Same adjacent-pair convention as algebra::fold_table; no opened shares.
pub fn lift_functional(weights: &[F], folds: &[F]) -> Vec<F> {
    let mut result = weights.to_vec();
    for r in folds.iter().rev() {
        result = result
            .iter()
            .flat_map(|w| [(F::from(1u64) - r) * w, *r * w])
            .collect();
    }
    result
}

pub fn linear_openings(
    relation: &Relation<'_>,
    kind: usize,
    functionals: &[Vec<F>],
    mask_only: bool,
    aggregation: F,
) -> Result<String, String> {
    let parameters = match kind {
        0 => relation.witness,
        2 => relation.mask,
        _ => return Err("note native PCS kind".into()),
    };
    parameters.validate()?;
    if functionals.is_empty()
        || functionals.len() > 3
        || functionals.iter().any(|v| v.len() != parameters.padded)
    {
        return Err("note native public functional shape".into());
    }
    let mut s = load(relation);
    let offset = relation.oracle_offset(kind)?;
    let mask_offset = relation.oracle_offset(kind + 1)?;
    for functional in functionals {
        let weights = message_functional(functional)?;
        s.push_str("value = sint(0)\n");
        for (i, weight) in weights
            .iter()
            .enumerate()
            .filter(|(_, w)| **w != F::from(0u64))
        {
            let mask = i + mask_offset;
            let i = i + offset;
            if mask_only {
                writeln!(s, "value += state[{mask}] * {weight}").unwrap();
            } else {
                writeln!(
                    s,
                    "value += (state[{i}] + {aggregation}*state[{mask}]) * {weight}"
                )
                .unwrap();
            }
        }
        s.push_str("print_ln('TRIAL_PUBLIC %s', value.reveal())\n");
    }
    Ok(s)
}

fn load(relation: &Relation<'_>) -> String {
    let mut s = base();
    let size = relation.state_size();
    writeln!(s,"stop, stored = sint.read_from_file(0, n_items=1, size={size})\nstate = Array({size}, sint)\nstate.assign_vector(stored[0])").unwrap();
    s
}

fn linear(terms: &[(usize, F)]) -> String {
    if terms.is_empty() {
        "sint(0)".into()
    } else {
        terms
            .iter()
            .map(|(i, c)| format!("state[{i}]*{c}"))
            .collect::<Vec<_>>()
            .join(" + ")
    }
}

fn release(s: &mut String, expressions: &[String]) {
    for expression in expressions {
        writeln!(s, "print_ln('TRIAL_PUBLIC %s', ({expression}).reveal())").unwrap();
    }
}

pub fn initial_sums(relation: &Relation<'_>, tau: &[F]) -> Result<String, String> {
    use crate::algebra::eq_table;
    if tau.len() + 1 != relation.rounds {
        return Err("note initial weight shape".into());
    }
    let mut s = load(relation);
    let beta = eq_table(tau);
    let mask = relation.oracle_offset(2)?;
    let terms = (0..3)
        .map(|i| {
            let product = if i < 2 {
                format!(
                    "state[{}]*state[{}]",
                    relation.blind(0) + i,
                    relation.blind(1) + i
                )
            } else {
                "sint(0)".into()
            };
            format!("{}*({product}-state[{}])", beta[i], relation.blind(2) + i)
        })
        .collect::<Vec<_>>()
        .join(" + ");
    let mask_sum = format!(
        "state[{mask}]*{} + sum(state[{mask}+i]*{} for i in range(1,{}))",
        relation.row_count(),
        relation.constraints,
        relation.mask.message
    );
    release(&mut s, &[terms, mask_sum]);
    Ok(s)
}

pub fn constraint_round(
    relation: &Relation<'_>,
    tau: &[F],
    prefix: &[F],
    batch: F,
) -> Result<String, String> {
    use crate::algebra::{eq_table, fold_table};
    if tau.len() != relation.rounds || prefix.len() >= relation.rounds {
        return Err("note constraint round shape".into());
    }
    let mut s = load(relation);
    let mut equality = eq_table(tau);
    for r in prefix {
        equality = fold_table(&equality, *r);
    }
    for (which, name) in ["left", "right", "output"].iter().enumerate() {
        writeln!(
            s,
            "{name} = [state[{}+i] for i in range({})]",
            relation.row_offset(which),
            relation.row_count()
        )
        .unwrap();
        for r in prefix {
            writeln!(s,"{name} = [{name}[2*i] + {r}*({name}[2*i+1]-{name}[2*i]) for i in range(len({name})//2)]").unwrap();
        }
    }
    let mask = relation.oracle_offset(2)?;
    let count = equality.len() / 2;
    let remaining = F::from(1u64 << (relation.rounds - prefix.len() - 1));
    let half = remaining / F::from(2u64);
    let mut fixed = vec![(mask, F::from(1u64))];
    for (i, r) in prefix.iter().enumerate() {
        let mut power = *r;
        for d in 0..3 {
            fixed.push((mask + 1 + 3 * i + d, power));
            power *= r;
        }
    }
    let tail = (prefix.len() + 1..relation.rounds)
        .flat_map(|i| (0..3).map(move |d| (mask + 1 + 3 * i + d, half)))
        .collect::<Vec<_>>();
    for at in 0..4 {
        let r = F::from(at as u64);
        let weights = fold_table(&equality, r);
        writeln!(
            s,
            "eq = [{}]",
            weights
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
        writeln!(s,"value = sum(eq[i]*((left[2*i]+{r}*(left[2*i+1]-left[2*i]))*(right[2*i]+{r}*(right[2*i+1]-right[2*i]))-(output[2*i]+{r}*(output[2*i+1]-output[2*i]))) for i in range({count}))").unwrap();
        let mut terms = fixed
            .iter()
            .map(|(i, c)| (*i, *c * remaining))
            .collect::<Vec<_>>();
        let mut power = r;
        for d in 0..3 {
            terms.push((mask + 1 + 3 * prefix.len() + d, remaining * power));
            power *= r;
        }
        terms.extend_from_slice(&tail);
        release(&mut s, &[format!("value + {batch}*({})", linear(&terms))]);
    }
    Ok(s)
}

pub fn endpoints(relation: &Relation<'_>, point: &[F]) -> Result<String, String> {
    use crate::algebra::eq_table;
    if point.len() != relation.rounds {
        return Err("note endpoint shape".into());
    }
    let mut s = load(relation);
    let weights = eq_table(point);
    for which in 0..3 {
        release(
            &mut s,
            &[linear(
                &weights
                    .iter()
                    .enumerate()
                    .map(|(i, w)| (relation.row_offset(which) + i, *w))
                    .collect::<Vec<_>>(),
            )],
        );
    }
    let mask = relation.oracle_offset(2)?;
    release(
        &mut s,
        &[linear(
            &relation
                .mask_weights(point)?
                .iter()
                .enumerate()
                .map(|(i, w)| (mask + i, *w))
                .collect::<Vec<_>>(),
        )],
    );
    Ok(s)
}

pub fn sumcheck_functionals(weights: &[F], prior_folds: &[F]) -> Result<Vec<Vec<F>>, String> {
    if weights.len() < 2 || !weights.len().is_power_of_two() {
        return Err("note sumcheck weights".into());
    }
    Ok((0..3)
        .map(|point| {
            let r = F::from(point as u64);
            let mut lifted = Vec::with_capacity(weights.len());
            for pair in weights.chunks_exact(2) {
                let w = (F::from(1u64) - r) * pair[0] + r * pair[1];
                lifted.extend([(F::from(1u64) - r) * w, r * w]);
            }
            lift_functional(&lifted, prior_folds)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::{dot, fold_table};
    #[test]
    fn native_functionals_match_recursive_coefficient_protocol() {
        let values: Vec<F> = (0..32).map(|i| F::from((i * i + 17) as u64)).collect();
        let coefficients = domain(32).unwrap().ifft(&values);
        let folds = [F::from(19u64), F::from(23u64)];
        let folded = folds
            .iter()
            .fold(coefficients.clone(), |v, r| fold_table(&v, *r));
        let weights: Vec<F> = (0..8).map(|i| F::from((i + 2) as u64)).collect();
        for (point, functional) in sumcheck_functionals(&weights, &folds)
            .unwrap()
            .iter()
            .enumerate()
        {
            let r = F::from(point as u64);
            let expected = dot(&fold_table(&folded, r), &fold_table(&weights, r));
            assert_eq!(dot(&coefficients, functional), expected);
            assert_eq!(
                dot(&values, &message_functional(functional).unwrap()),
                expected
            );
        }
    }
    #[test]
    fn native_initialization_is_dynamic_and_has_no_plaintext_release() {
        let mut circuit = Circuit::default();
        let byte = circuit.private_bytes(1);
        circuit
            .assert_equal(&byte, &Circuit::constant_bytes(&[3]))
            .unwrap();
        let layout = Layout::for_circuit(&circuit).unwrap();
        assert_eq!(layout.input_fields, 8);
        let source = initialize(&circuit).unwrap();
        assert!(source.contains("invalid private note circuit"));
        assert!(!source.contains("TRIAL_PUBLIC"));
        assert_eq!(source.matches(".reveal()").count(), 2);
        circuit.inputs.push(Wire::Variable(0));
        assert!(initialize(&circuit).is_err());
    }
}
