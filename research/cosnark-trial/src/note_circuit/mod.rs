//! Variable-size boolean-to-R1CS compilation for the operational note relation.
//! The SHA-512 gate core is the pinned Bristol circuit, not a host hash standing
//! in for a private computation. This module exposes only the public circuit;
//! private trace evaluation belongs to MPC workers, not a coordinator API.
mod bristol;
pub mod typed;

use crate::field::F;
use bristol::Gate;
use sha2::{Digest, Sha512};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Wire {
    Constant(bool),
    Variable(usize),
}

pub type Byte = [Wire; 8]; // network byte order, most significant bit first
pub type Linear = Vec<(usize, F)>;
pub struct Row {
    pub a: Linear,
    pub b: Linear,
    pub c: Linear,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Constraint {
    Boolean(Wire),
    And {
        a: Wire,
        b: Wire,
        out: Wire,
    },
    Xor {
        a: Wire,
        b: Wire,
        out: Wire,
    },
    Not {
        input: Wire,
        out: Wire,
    },
    Equal(Wire, Wire),
    /// Little-endian unsigned bits. Inputs are at most 32 bits each, output is
    /// their full (at most 64-bit) integer product, never a reduced field value.
    PackedProduct {
        a: Box<[Wire]>,
        b: Box<[Wire]>,
        out: Box<[Wire]>,
    },
    /// Full unsigned sum, including its carry bit; at most 64 output bits.
    PackedAdd {
        a: Box<[Wire]>,
        b: Box<[Wire]>,
        out: Box<[Wire]>,
    },
}

fn term(wire: Wire, coefficient: i64) -> Linear {
    let coefficient = if coefficient >= 0 {
        F::from(coefficient as u64)
    } else {
        -F::from(coefficient.unsigned_abs())
    };
    match wire {
        Wire::Constant(false) => vec![],
        Wire::Constant(true) => vec![(0, coefficient)],
        Wire::Variable(index) => vec![(index, coefficient)],
    }
}

impl Constraint {
    pub fn row(&self) -> Row {
        let one = || vec![(0, F::from(1u64))];
        match *self {
            Self::Boolean(wire) => {
                let mut b = term(wire, 1);
                b.extend(term(Wire::Constant(true), -1));
                Row {
                    a: term(wire, 1),
                    b,
                    c: vec![],
                }
            }
            Self::And { a, b, out } => Row {
                a: term(a, 1),
                b: term(b, 1),
                c: term(out, 1),
            },
            Self::Xor { a, b, out } => {
                // In the odd prime field: 2*a*b = a+b-out. Booleanity follows
                // inductively from the input constraints and these gate rows.
                let mut c = term(a, 1);
                c.extend(term(b, 1));
                c.extend(term(out, -1));
                Row {
                    a: term(a, 2),
                    b: term(b, 1),
                    c,
                }
            }
            Self::Not { input, out } => {
                let mut a = one();
                a.extend(term(input, -1));
                Row {
                    a,
                    b: one(),
                    c: term(out, 1),
                }
            }
            Self::Equal(a, b) => Row {
                a: term(a, 1),
                b: one(),
                c: term(b, 1),
            },
            Self::PackedProduct {
                ref a,
                ref b,
                ref out,
            } => Row {
                a: packed_linear(a),
                b: packed_linear(b),
                c: packed_linear(out),
            },
            Self::PackedAdd {
                ref a,
                ref b,
                ref out,
            } => {
                let mut left = packed_linear(a);
                left.extend(packed_linear(b));
                Row {
                    a: left,
                    b: one(),
                    c: packed_linear(out),
                }
            }
        }
    }
}

fn packed_linear(bits: &[Wire]) -> Linear {
    let mut coefficient = F::from(1u64);
    let mut result = Vec::with_capacity(bits.len());
    for bit in bits {
        match *bit {
            Wire::Constant(false) => {}
            Wire::Constant(true) => result.push((0, coefficient)),
            Wire::Variable(i) => result.push((i, coefficient)),
        }
        coefficient += coefficient;
    }
    result
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circuit {
    /// Index zero is the public constant one, never an owner input.
    pub variables: usize,
    pub inputs: Vec<Wire>,
    pub constraints: Vec<Constraint>,
}

impl Default for Circuit {
    fn default() -> Self {
        Self {
            variables: 1,
            inputs: vec![],
            constraints: vec![],
        }
    }
}

impl Circuit {
    /// Validate the executable topology before admitting a circuit to native
    /// MPC. Public struct fields must not permit reading an undefined wire or
    /// overwriting an input, the constant one, or an earlier gate output.
    pub fn validate(&self) -> Result<(), String> {
        if self.variables == 0 {
            return Err("empty note circuit".into());
        }
        let mut defined = vec![false; self.variables];
        defined[0] = true;
        for input in &self.inputs {
            match *input {
                Wire::Variable(i) if i < defined.len() && !defined[i] => defined[i] = true,
                _ => return Err("note circuit input topology".into()),
            }
        }
        let mut boolean_inputs = std::collections::BTreeSet::new();
        let mut packed_outputs = std::collections::BTreeSet::new();
        for constraint in &self.constraints {
            let (operands, outputs) = match *constraint {
                Constraint::Boolean(a) => {
                    if let Wire::Variable(i) = a {
                        boolean_inputs.insert(i);
                    }
                    (vec![a], vec![])
                }
                Constraint::Equal(a, b) => (vec![a, b], vec![]),
                Constraint::And { a, b, out } | Constraint::Xor { a, b, out } => {
                    (vec![a, b], vec![out])
                }
                Constraint::Not { input, out } => (vec![input], vec![out]),
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
                    let product = matches!(constraint, Constraint::PackedProduct { .. });
                    let width = if product {
                        a.len() + b.len()
                    } else {
                        a.len().max(b.len()) + 1
                    };
                    let input_limit = if product { 32 } else { 63 };
                    if a.is_empty()
                        || b.is_empty()
                        || a.len() > input_limit
                        || b.len() > input_limit
                        || out.len() != width
                        || width > 64
                    {
                        return Err("packed integer circuit width".into());
                    }
                    for wire in out.iter() {
                        if let Wire::Variable(i) = wire {
                            packed_outputs.insert(*i);
                        }
                    }
                    (a.iter().chain(b.iter()).copied().collect(), out.to_vec())
                }
            };
            for operand in operands {
                if let Wire::Variable(i) = operand {
                    if !defined.get(i).copied().unwrap_or(false) {
                        return Err("note circuit forward or missing wire".into());
                    }
                }
            }
            for output in outputs {
                match output {
                    Wire::Variable(i) if i < defined.len() && !defined[i] => defined[i] = true,
                    _ => return Err("note circuit output topology".into()),
                }
            }
        }
        if defined.iter().any(|v| !v)
            || !packed_outputs.is_subset(&boolean_inputs)
            || self.inputs.iter().any(|wire| match wire {
                Wire::Variable(i) => !boolean_inputs.contains(i),
                Wire::Constant(_) => true,
            })
        {
            return Err("note circuit undefined or non-boolean input".into());
        }
        Ok(())
    }

    fn variable(&mut self) -> Wire {
        let wire = Wire::Variable(self.variables);
        self.variables += 1;
        wire
    }

    pub fn private_byte(&mut self) -> Byte {
        std::array::from_fn(|_| {
            let wire = self.variable();
            self.inputs.push(wire);
            self.constraints.push(Constraint::Boolean(wire));
            wire
        })
    }

    pub fn private_bytes(&mut self, count: usize) -> Vec<Byte> {
        (0..count).map(|_| self.private_byte()).collect()
    }

    /// Exact bounded integer arithmetic over the existing BN254 scalar FIELD
    /// (not a curve proof). All full products/sums fit below 2^64, far below its
    /// modulus. Explicit output-bit constraints prevent a forged decomposition.
    pub fn multiply_bits_le(&mut self, a: &[Wire], b: &[Wire]) -> Result<Vec<Wire>, String> {
        self.packed_integer(a, b, true)
    }

    pub fn add_bits_le(&mut self, a: &[Wire], b: &[Wire]) -> Result<Vec<Wire>, String> {
        self.packed_integer(a, b, false)
    }

    fn packed_integer(
        &mut self,
        a: &[Wire],
        b: &[Wire],
        product: bool,
    ) -> Result<Vec<Wire>, String> {
        let limit = if product { 32 } else { 63 };
        if a.is_empty() || b.is_empty() || a.len() > limit || b.len() > limit {
            return Err("packed integer input width".into());
        }
        let width = if product {
            a.len() + b.len()
        } else {
            a.len().max(b.len()) + 1
        };
        if product
            && (a.iter().all(|w| *w == Wire::Constant(false))
                || b.iter().all(|w| *w == Wire::Constant(false)))
        {
            return Ok(vec![Wire::Constant(false); width]);
        }
        let out: Vec<_> = (0..width).map(|_| self.variable()).collect();
        self.constraints.push(if product {
            Constraint::PackedProduct {
                a: a.into(),
                b: b.into(),
                out: out.clone().into(),
            }
        } else {
            Constraint::PackedAdd {
                a: a.into(),
                b: b.into(),
                out: out.clone().into(),
            }
        });
        self.constraints
            .extend(out.iter().copied().map(Constraint::Boolean));
        Ok(out)
    }

    pub fn constant_bytes(bytes: &[u8]) -> Vec<Byte> {
        bytes
            .iter()
            .map(|byte| std::array::from_fn(|bit| Wire::Constant(byte & (1 << (7 - bit)) != 0)))
            .collect()
    }

    pub fn and(&mut self, a: Wire, b: Wire) -> Wire {
        match (a, b) {
            (Wire::Constant(false), _) | (_, Wire::Constant(false)) => Wire::Constant(false),
            (Wire::Constant(true), wire) | (wire, Wire::Constant(true)) => wire,
            _ if a == b => a,
            _ => {
                let out = self.variable();
                self.constraints.push(Constraint::And { a, b, out });
                out
            }
        }
    }

    pub fn not(&mut self, input: Wire) -> Wire {
        match input {
            Wire::Constant(value) => Wire::Constant(!value),
            _ => {
                let out = self.variable();
                self.constraints.push(Constraint::Not { input, out });
                out
            }
        }
    }

    pub fn xor(&mut self, a: Wire, b: Wire) -> Wire {
        match (a, b) {
            (Wire::Constant(false), wire) | (wire, Wire::Constant(false)) => wire,
            (Wire::Constant(true), wire) | (wire, Wire::Constant(true)) => self.not(wire),
            _ if a == b => Wire::Constant(false),
            _ => {
                let out = self.variable();
                self.constraints.push(Constraint::Xor { a, b, out });
                out
            }
        }
    }

    pub fn assert_equal(&mut self, left: &[Byte], right: &[Byte]) -> Result<(), String> {
        if left.len() != right.len() {
            return Err("circuit equality width".into());
        }
        for (a, b) in left.iter().flatten().zip(right.iter().flatten()) {
            self.constraints.push(Constraint::Equal(*a, *b));
        }
        Ok(())
    }

    fn sha512_compress(
        &mut self,
        block_msb: &[Wire],
        state_lsb: &[Wire],
    ) -> Result<Vec<Wire>, String> {
        if block_msb.len() != 1024 || state_lsb.len() != 512 {
            return Err("SHA-512 circuit input width".into());
        }
        let reference = bristol::sha512()?;
        let mut wires = vec![Wire::Constant(false); reference.wires];
        // Bristol treats each complete input value as a little-endian integer.
        // This is the same chunk/state convention used by MP-SPDZ's SHA2
        // Bristol adapter; external bytes and digest bytes remain big endian.
        for (to, from) in wires[..1024].iter_mut().zip(block_msb.iter().rev()) {
            *to = *from;
        }
        wires[1024..1536].copy_from_slice(state_lsb);
        for gate in &reference.gates {
            match *gate {
                Gate::And(a, b, out) => wires[out] = self.and(wires[a], wires[b]),
                Gate::Xor(a, b, out) => wires[out] = self.xor(wires[a], wires[b]),
                Gate::Inv(a, out) => wires[out] = self.not(wires[a]),
                Gate::Copy(a, out) => wires[out] = wires[a],
                Gate::Constant(value, out) => wires[out] = Wire::Constant(value),
            }
        }
        Ok(wires[reference.wires - 512..].to_vec())
    }

    pub fn sha512(&mut self, message: &[Byte]) -> Result<Vec<Byte>, String> {
        // FIPS 180-4 SHA-512 IV; only padding/byte adaptation is implemented
        // here. All compression gates come from the unmodified licensed core.
        let iv: [u64; 8] = [
            0x6a09e667f3bcc908,
            0xbb67ae8584caa73b,
            0x3c6ef372fe94f82b,
            0xa54ff53a5f1d36f1,
            0x510e527fade682d1,
            0x9b05688c2b3e6c1f,
            0x1f83d9abfb41bd6b,
            0x5be0cd19137e2179,
        ];
        let iv_bytes: Vec<u8> = iv.iter().flat_map(|word| word.to_be_bytes()).collect();
        let mut state: Vec<Wire> = Self::constant_bytes(&iv_bytes)
            .into_iter()
            .flatten()
            .rev()
            .collect();
        let mut padded: Vec<Wire> = message.iter().flatten().copied().collect();
        let bit_length = padded.len() as u128;
        padded.push(Wire::Constant(true));
        while padded.len() % 1024 != 896 {
            padded.push(Wire::Constant(false));
        }
        padded.extend(
            Self::constant_bytes(&bit_length.to_be_bytes())
                .into_iter()
                .flatten(),
        );
        for block in padded.chunks_exact(1024) {
            state = self.sha512_compress(block, &state)?;
        }
        state.reverse();
        Ok(state
            .chunks_exact(8)
            .map(|byte| byte.try_into().expect("exact output byte"))
            .collect())
    }

    /// Exact identity preimage shared with DeFMI's PQC wallet, including full
    /// domain/policy/body lengths. A host-computed hash is not a constraint.
    pub fn identity(
        &mut self,
        domain: &[u8],
        policy: &[u8],
        body: &[Byte],
    ) -> Result<Vec<Byte>, String> {
        let mut bytes = Self::constant_bytes(&(domain.len() as u64).to_be_bytes());
        bytes.extend(Self::constant_bytes(domain));
        bytes.extend(Self::constant_bytes(&(policy.len() as u64).to_be_bytes()));
        bytes.extend(Self::constant_bytes(policy));
        bytes.extend(Self::constant_bytes(&(body.len() as u64).to_be_bytes()));
        bytes.extend_from_slice(body);
        self.sha512(&bytes)
    }

    /// Hash every constraint and input location, not just the circuit's size.
    pub fn fingerprint(&self) -> [u8; 64] {
        let mut hash = Sha512::new();
        hash.update(b"ZKFMI:PQC-NOTE-R1CS:v1");
        hash.update((self.variables as u64).to_be_bytes());
        hash.update((self.inputs.len() as u64).to_be_bytes());
        let wire = |hash: &mut Sha512, value: Wire| match value {
            Wire::Constant(value) => {
                hash.update([0, u8::from(value)]);
            }
            Wire::Variable(index) => {
                hash.update([1]);
                hash.update((index as u64).to_be_bytes());
            }
        };
        for input in &self.inputs {
            wire(&mut hash, *input);
        }
        hash.update((self.constraints.len() as u64).to_be_bytes());
        for constraint in &self.constraints {
            match *constraint {
                Constraint::Boolean(a) => {
                    hash.update([0]);
                    wire(&mut hash, a);
                }
                Constraint::And { a, b, out } => {
                    hash.update([1]);
                    for w in [a, b, out] {
                        wire(&mut hash, w);
                    }
                }
                Constraint::Xor { a, b, out } => {
                    hash.update([2]);
                    for w in [a, b, out] {
                        wire(&mut hash, w);
                    }
                }
                Constraint::Not { input, out } => {
                    hash.update([3]);
                    for w in [input, out] {
                        wire(&mut hash, w);
                    }
                }
                Constraint::Equal(a, b) => {
                    hash.update([4]);
                    for w in [a, b] {
                        wire(&mut hash, w);
                    }
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
                    hash.update([if matches!(constraint, Constraint::PackedProduct { .. }) {
                        5
                    } else {
                        6
                    }]);
                    for bits in [a, b, out] {
                        hash.update((bits.len() as u64).to_be_bytes());
                        for bit in bits.iter() {
                            wire(&mut hash, *bit);
                        }
                    }
                }
            }
        }
        hash.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(wire: Wire, values: &[F]) -> F {
        match wire {
            Wire::Constant(value) => F::from(u64::from(value)),
            Wire::Variable(i) => values[i],
        }
    }

    // Public known-answer fixtures only. This is not a production centralized
    // witness-generation API and is absent outside cfg(test).
    fn trace(circuit: &Circuit, inputs: &[u8]) -> Vec<F> {
        assert_eq!(circuit.inputs.len(), inputs.len() * 8);
        let mut values = vec![F::from(0u64); circuit.variables];
        values[0] = F::from(1u64);
        for (wire, bit) in circuit.inputs.iter().zip(
            inputs
                .iter()
                .flat_map(|byte| (0..8).map(move |bit| byte >> (7 - bit) & 1)),
        ) {
            if let Wire::Variable(i) = wire {
                values[*i] = F::from(bit as u64);
            } else {
                panic!("input constant");
            }
        }
        for constraint in &circuit.constraints {
            if let Constraint::PackedProduct { a, b, out } | Constraint::PackedAdd { a, b, out } =
                constraint
            {
                let packed = |bits: &[Wire]| {
                    bits.iter().enumerate().fold(0u128, |acc, (i, bit)| {
                        let value = value(*bit, &values);
                        assert!(value == F::from(0u64) || value == F::from(1u64));
                        acc | (u128::from(value == F::from(1u64)) << i)
                    })
                };
                let integer = if matches!(constraint, Constraint::PackedProduct { .. }) {
                    packed(a) * packed(b)
                } else {
                    packed(a) + packed(b)
                };
                for (i, bit) in out.iter().enumerate() {
                    if let Wire::Variable(index) = bit {
                        values[*index] = F::from(((integer >> i) & 1) as u64);
                    }
                }
            }
            let computed = match *constraint {
                Constraint::And { a, b, out } => Some((out, value(a, &values) * value(b, &values))),
                Constraint::Xor { a, b, out } => Some((
                    out,
                    value(a, &values) + value(b, &values)
                        - F::from(2u64) * value(a, &values) * value(b, &values),
                )),
                Constraint::Not { input, out } => {
                    Some((out, F::from(1u64) - value(input, &values)))
                }
                _ => None,
            };
            if let Some((Wire::Variable(i), v)) = computed {
                values[i] = v;
            }
        }
        values
    }

    fn satisfied(circuit: &Circuit, values: &[F]) -> bool {
        circuit.constraints.iter().all(|constraint| {
            let row = constraint.row();
            let eval = |terms: &Linear| terms.iter().map(|(i, c)| values[*i] * c).sum::<F>();
            eval(&row.a) * eval(&row.b) == eval(&row.c)
        })
    }

    #[test]
    fn actual_hycc_typed_uint64_product_matches_all_boundary_pairs() {
        let export = include_bytes!("../../vendor/hycc-typed-mul64.json");
        assert_eq!(
            hex::encode(sha2::Sha256::digest(export)),
            "3e88503b8a1d79b535f07f9c7b6b79797d13988438e7f43b3d3e9f6ad470699d"
        );
        let typed: typed::TypedCircuit = serde_json::from_slice(export).unwrap();
        let mut circuit = Circuit::default();
        let private = circuit.private_bytes(16);
        let bindings = std::collections::BTreeMap::from([
            (
                "INPUT_A_left".into(),
                private[..8].iter().flatten().rev().copied().collect(),
            ),
            (
                "INPUT_A_right".into(),
                private[8..].iter().flatten().rev().copied().collect(),
            ),
        ]);
        let outputs = typed.append(&mut circuit, &bindings).unwrap();
        circuit.validate().unwrap();
        let product = &outputs["OUTPUT_product"];
        assert_eq!(product.len(), 64);
        let cases = [
            0,
            1,
            2,
            u32::MAX as u64,
            1 << 32,
            (1 << 63) - 1,
            1 << 63,
            u64::MAX,
        ];
        for a in cases {
            for b in cases {
                let bytes: Vec<_> = a.to_be_bytes().into_iter().chain(b.to_be_bytes()).collect();
                let values = trace(&circuit, &bytes);
                assert!(satisfied(&circuit, &values));
                let actual = product.iter().enumerate().fold(0u64, |acc, (i, bit)| {
                    acc | (u64::from(value(*bit, &values) == F::from(1u64)) << i)
                });
                assert_eq!(actual, a.wrapping_mul(b));
            }
        }
        // Output-assertion coverage is mandatory, not a caller convention.
        let mut bad: serde_json::Value = serde_json::from_slice(export).unwrap();
        bad["output_groups"][0]["name_hex"] = serde_json::json!(hex::encode("omitted_validity"));
        let bad: typed::TypedCircuit = serde_json::from_value(bad).unwrap();
        assert!(bad.append(&mut circuit, &bindings).is_err());
    }

    #[test]
    fn packed_integer_rows_are_full_width_and_reject_forged_bits() {
        let mut circuit = Circuit::default();
        let private = circuit.private_bytes(8);
        let a: Vec<_> = private[..4].iter().flatten().rev().copied().collect();
        let b: Vec<_> = private[4..].iter().flatten().rev().copied().collect();
        let product = circuit.multiply_bits_le(&a, &b).unwrap();
        let sum = circuit.add_bits_le(&a, &b).unwrap();
        circuit.validate().unwrap();
        assert_eq!(product.len(), 64);
        assert_eq!(sum.len(), 33);
        let cases = [0, 1, 2, 0xffff, 0x10000, 0x7fff_ffff, 0x8000_0000, u32::MAX];
        for left in cases {
            for right in cases {
                let bytes: Vec<_> = left
                    .to_be_bytes()
                    .into_iter()
                    .chain(right.to_be_bytes())
                    .collect();
                let values = trace(&circuit, &bytes);
                assert!(satisfied(&circuit, &values));
                let integer = |bits: &[Wire]| {
                    bits.iter().enumerate().fold(0u64, |acc, (i, bit)| {
                        acc | (u64::from(value(*bit, &values) == F::from(1u64)) << i)
                    })
                };
                assert_eq!(integer(&product), u64::from(left) * u64::from(right));
                assert_eq!(integer(&sum), u64::from(left) + u64::from(right));
                for bit in [product[0], product[63], sum[32]] {
                    let mut forged = values.clone();
                    if let Wire::Variable(i) = bit {
                        forged[i] += F::from(1u64);
                    }
                    assert!(!satisfied(&circuit, &forged));
                }
            }
        }
        let fingerprint = circuit.fingerprint();
        circuit
            .constraints
            .retain(|c| !matches!(c, Constraint::Boolean(w) if *w == product[63]));
        assert!(circuit.validate().is_err());
        assert_ne!(fingerprint, circuit.fingerprint());
    }

    #[test]
    fn packed_integer_widths_cannot_wrap_the_proof_field_or_masking_interval() {
        let mut circuit = Circuit::default();
        let a = circuit
            .private_bytes(8)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        assert!(circuit.multiply_bits_le(&a, &a).is_err());
        assert!(circuit.add_bits_le(&a, &a).is_err());
        assert!(circuit.multiply_bits_le(&[], &a[..8]).is_err());
    }

    #[test]
    fn licensed_sha512_circuit_matches_rustcrypto_across_padding_boundaries() {
        for length in [0, 1, 111, 112, 128, 232] {
            let message: Vec<u8> = (0..length).map(|i| (i * 73 + 19) as u8).collect();
            let mut circuit = Circuit::default();
            let private = circuit.private_bytes(message.len());
            let output = circuit.sha512(&private).unwrap();
            let expected: [u8; 64] = Sha512::digest(&message).into();
            circuit
                .assert_equal(&output, &Circuit::constant_bytes(&expected))
                .unwrap();
            let mut values = trace(&circuit, &message);
            let actual: Vec<u8> = output
                .iter()
                .map(|byte| {
                    byte.iter().fold(0u8, |acc, wire| {
                        let bit = value(*wire, &values);
                        assert!(bit == F::from(0u64) || bit == F::from(1u64));
                        (acc << 1) | u8::from(bit == F::from(1u64))
                    })
                })
                .collect();
            assert_eq!(actual, expected, "message length {length}");
            assert!(satisfied(&circuit, &values));
            if let Some(Wire::Variable(i)) = output
                .iter()
                .flatten()
                .find(|w| matches!(w, Wire::Variable(_)))
            {
                values[*i] += F::from(1u64);
                assert!(!satisfied(&circuit, &values));
            }
        }
    }

    #[test]
    fn private_identity_is_constrained_not_only_host_metadata() {
        let body = [3u8; 64];
        let policy = b"known-answer-policy";
        let domain = b"DEFMI:PQC-NOTE-SPENDING-KEY:v1";
        let mut circuit = Circuit::default();
        let private = circuit.private_bytes(body.len());
        let output = circuit.identity(domain, policy, &private).unwrap();
        let mut host = Sha512::new();
        host.update((domain.len() as u64).to_be_bytes());
        host.update(domain);
        host.update((policy.len() as u64).to_be_bytes());
        host.update(policy);
        host.update((body.len() as u64).to_be_bytes());
        host.update(body);
        circuit
            .assert_equal(&output, &Circuit::constant_bytes(&host.finalize()))
            .unwrap();
        assert!(satisfied(&circuit, &trace(&circuit, &body)));
        assert!(!satisfied(&circuit, &trace(&circuit, &[4u8; 64])));
        assert!(circuit.variables > 1024);
    }
}
