//! Bristol basic-format adapter for the pinned, unmodified SHA-512 circuit.
//!
//! Header/arity dispatch is adapted from Galois Swanky's reader at
//! ad4a93b412ca8f0d2a5ca033acbe43b30e940af4, edge/simple-arith-circuit/src/reader.rs.
//! See vendor/bristol-sha512/License.swanky.txt. Unlike that F2 circuit store,
//! retain original wire labels and require each input to be defined before use;
//! no first-use input permutation or synthetic constant/output wires are needed.
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub(super) enum Gate {
    And(usize, usize, usize),
    Xor(usize, usize, usize),
    Inv(usize, usize),
    Copy(usize, usize),
    Constant(bool, usize),
}

pub(super) struct Bristol {
    pub wires: usize,
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub gates: Vec<Gate>,
}

fn numbers(line: &str) -> Result<Vec<usize>, String> {
    line.split_whitespace()
        .map(|word| word.parse().map_err(|_| "invalid Bristol integer".into()))
        .collect()
}

fn widths(line: &str) -> Result<Vec<usize>, String> {
    let nums = numbers(line)?;
    match nums.split_first() {
        Some((&count, values)) if count == values.len() && count > 0 => Ok(values.to_vec()),
        _ => Err("invalid Bristol value widths".into()),
    }
}

impl Bristol {
    fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        let header = numbers(lines.next().ok_or("missing Bristol header")?)?;
        if header.len() != 2 || header[0] > 500_000 || header[1] > 502_048 {
            return Err("invalid Bristol circuit bounds".into());
        }
        let (gate_count, wires) = (header[0], header[1]);
        let inputs = widths(lines.next().ok_or("missing Bristol inputs")?)?;
        let outputs = widths(lines.next().ok_or("missing Bristol outputs")?)?;
        let sum = |v: &[usize]| v.iter().try_fold(0usize, |a, b| a.checked_add(*b));
        let input_count = sum(&inputs).ok_or("Bristol input overflow")?;
        let output_count = sum(&outputs).ok_or("Bristol output overflow")?;
        if input_count > wires || output_count > wires || input_count + gate_count != wires {
            return Err("Bristol wire cardinality".into());
        }
        if !lines
            .next()
            .ok_or("missing Bristol separator")?
            .trim()
            .is_empty()
        {
            return Err("nonempty Bristol separator".into());
        }
        let mut defined = vec![false; wires];
        defined[..input_count].fill(true);
        let mut gates = Vec::with_capacity(gate_count);
        for _ in 0..gate_count {
            let line = lines.next().ok_or("truncated Bristol gates")?;
            let (args, name) = line
                .trim()
                .rsplit_once(char::is_whitespace)
                .ok_or("missing Bristol gate name")?;
            let nums = numbers(args)?;
            let gate = match (name, nums.as_slice()) {
                ("AND", &[2, 1, a, b, out]) => Gate::And(a, b, out),
                ("XOR", &[2, 1, a, b, out]) => Gate::Xor(a, b, out),
                ("INV", &[1, 1, a, out]) => Gate::Inv(a, out),
                ("EQW", &[1, 1, a, out]) => Gate::Copy(a, out),
                // EQ's first argument is a literal, NOT an input wire label.
                ("EQ", &[1, 1, value, out]) if value <= 1 => Gate::Constant(value == 1, out),
                _ => return Err("invalid Bristol gate or arity".into()),
            };
            let (read, out): (Vec<usize>, usize) = match gate {
                Gate::And(a, b, out) | Gate::Xor(a, b, out) => (vec![a, b], out),
                Gate::Inv(a, out) | Gate::Copy(a, out) => (vec![a], out),
                Gate::Constant(_, out) => (vec![], out),
            };
            if read
                .iter()
                .any(|i| !defined.get(*i).copied().unwrap_or(false))
                || out >= wires
                || defined[out]
            {
                return Err("Bristol undefined input or duplicate output".into());
            }
            defined[out] = true;
            gates.push(gate);
        }
        if defined.iter().any(|set| !set) || lines.any(|line| !line.trim().is_empty()) {
            return Err("Bristol incomplete circuit or trailing data".into());
        }
        Ok(Self {
            wires,
            inputs,
            outputs,
            gates,
        })
    }
}

pub(super) fn sha512() -> Result<&'static Bristol, String> {
    static CIRCUIT: OnceLock<Result<Bristol, String>> = OnceLock::new();
    CIRCUIT
        .get_or_init(|| {
            let source = include_str!("../../vendor/bristol-sha512/sha512.txt");
            if hex::encode(Sha256::digest(source.as_bytes()))
                != "43cadaa91b437e99173c9ffa33b20f49db80bb0c7993e5fcedb8de1388a25df2"
            {
                return Err("SHA-512 reference circuit checksum mismatch".into());
            }
            let circuit = Bristol::parse(source)?;
            if circuit.inputs != [1024, 512]
                || circuit.outputs != [512]
                || circuit.gates.len() != 349_617
                || circuit.wires != 351_153
            {
                return Err("SHA-512 reference circuit shape mismatch".into());
            }
            Ok(circuit)
        })
        .as_ref()
        .map_err(Clone::clone)
}
