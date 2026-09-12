//! Adapter for the pinned HyCC reader's mixed Boolean/integer export.
//! All integer lanes are little endian; public byte lanes remain MSB first.
//! No plaintext witness evaluation is exposed here.
use super::{Circuit, Constraint, Wire};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedCircuit {
    version: u32,
    inputs: Vec<(u64, usize)>,
    gates: Vec<Gate>,
    outputs: Vec<(u64, u8)>,
    input_groups: Vec<Group>,
    output_groups: Vec<Group>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Gate {
    id: u64,
    kind: String,
    width: usize,
    value: u64,
    inputs: Vec<(u64, u8)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    name_hex: String,
    indices: Vec<usize>,
}

fn group_name(group: &Group) -> Result<String, String> {
    let bytes = hex::decode(&group.name_hex).map_err(|_| "typed group hex")?;
    if hex::encode(&bytes) != group.name_hex || bytes.is_empty() {
        return Err("typed group name".into());
    }
    String::from_utf8(bytes).map_err(|_| "typed group UTF8".into())
}

fn width(n: usize) -> Result<(), String> {
    if (1..=64).contains(&n) {
        Ok(())
    } else {
        Err("typed lane width".into())
    }
}

fn add_mod(c: &mut Circuit, a: &[Wire], b: &[Wire]) -> Result<Vec<Wire>, String> {
    if a.len() != b.len() {
        return Err("typed add width".into());
    }
    width(a.len())?;
    let mut out = Vec::with_capacity(a.len());
    let mut carry = Wire::Constant(false);
    for (x, y) in a.chunks(32).zip(b.chunks(32)) {
        let sum = c.add_bits_le(x, y)?;
        let with_carry = c.add_bits_le(&sum, &[carry])?;
        out.extend_from_slice(&with_carry[..x.len()]);
        carry = with_carry[x.len()];
    }
    Ok(out)
}

fn mul_mod(c: &mut Circuit, a: &[Wire], b: &[Wire]) -> Result<Vec<Wire>, String> {
    if a.len() != b.len() {
        return Err("typed mul width".into());
    }
    width(a.len())?;
    let n = a.len();
    let mut out = vec![Wire::Constant(false); n];
    for (i, x) in a.chunks(32).enumerate() {
        for (j, y) in b.chunks(32).enumerate() {
            let offset = (i + j) * 32;
            if offset >= n {
                continue;
            }
            let product = c.multiply_bits_le(x, y)?;
            let mut term = vec![Wire::Constant(false); n];
            let count = product.len().min(n - offset);
            term[offset..offset + count].copy_from_slice(&product[..count]);
            out = add_mod(c, &out, &term)?;
        }
    }
    Ok(out)
}

impl TypedCircuit {
    /// Bind ALL named input lanes to existing circuit bits. The compiled C ABI
    /// defines grouping/order; no input is implicitly made public or discarded.
    pub fn append(
        &self,
        c: &mut Circuit,
        bindings: &BTreeMap<String, Vec<Wire>>,
    ) -> Result<BTreeMap<String, Vec<Wire>>, String> {
        if self.version != 1 || self.input_groups.len() != bindings.len() {
            return Err("typed circuit version/input groups".into());
        }
        let mut lanes = BTreeMap::<(u64, u8), Vec<Wire>>::new();
        let mut ids = BTreeSet::new();
        let mut used = BTreeSet::new();
        let mut names = BTreeSet::new();
        for group in &self.input_groups {
            let name = group_name(group)?;
            if !names.insert(name.clone()) {
                return Err("duplicate typed input name".into());
            }
            let bits = bindings.get(&name).ok_or("missing typed input")?;
            let mut offset = 0usize;
            for index in &group.indices {
                if !used.insert(*index) {
                    return Err("duplicate typed input".into());
                }
                let &(id, n) = self.inputs.get(*index).ok_or("typed input index")?;
                width(n)?;
                if id >> 62 != 1 || !ids.insert(id) {
                    return Err("typed input identity".into());
                }
                let end = offset.checked_add(n).ok_or("typed input overflow")?;
                let lane = bits.get(offset..end).ok_or("typed input length")?;
                lanes.insert((id, 0), lane.to_vec());
                offset = end;
            }
            if offset != bits.len() {
                return Err("excess typed input bits".into());
            }
        }
        if used.len() != self.inputs.len() {
            return Err("unbound typed input".into());
        }
        for g in &self.gates {
            width(g.width)?;
            if g.id >> 62 != 2 || !ids.insert(g.id) || (g.kind != "CONST" && g.value != 0) {
                return Err("typed gate identity/constant".into());
            }
            let args = g
                .inputs
                .iter()
                .map(|e| {
                    lanes
                        .get(e)
                        .cloned()
                        .ok_or_else(|| "typed forward/missing endpoint".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arity = match g.kind.as_str() {
                "ONE" | "CONST" => 0,
                "NOT" | "NEG" | "SPLIT" => 1,
                "AND" | "OR" | "XOR" | "ADD" | "SUB" | "MUL" => 2,
                "COMBINE" => g.width,
                _ => return Err("unknown typed gate".into()),
            };
            if args.len() != arity {
                return Err("typed gate arity".into());
            }
            if g.kind == "SPLIT" {
                if g.width != 1 {
                    return Err("typed split width".into());
                }
                for (pin, bit) in args[0].iter().enumerate() {
                    lanes.insert((g.id, pin as u8), vec![*bit]);
                }
                continue;
            }
            let bits = match g.kind.as_str() {
                "COMBINE" => {
                    if args.iter().any(|a| a.len() != 1) {
                        return Err("typed combine width".into());
                    }
                    args.iter().map(|a| a[0]).collect()
                }
                "ONE" => {
                    if g.width != 1 {
                        return Err("typed ONE width".into());
                    }
                    vec![Wire::Constant(true)]
                }
                "CONST" => {
                    if g.width < 64 && g.value >> g.width != 0 {
                        return Err("typed constant overflow".into());
                    }
                    (0..g.width)
                        .map(|i| Wire::Constant(g.value >> i & 1 != 0))
                        .collect()
                }
                "NOT" | "AND" | "OR" | "XOR" => {
                    if g.width != 1 || args.iter().any(|a| a.len() != 1) {
                        return Err("typed boolean width".into());
                    }
                    let a = args[0][0];
                    vec![match g.kind.as_str() {
                        "NOT" => c.not(a),
                        "AND" => c.and(a, args[1][0]),
                        "XOR" => c.xor(a, args[1][0]),
                        _ => {
                            let x = c.xor(a, args[1][0]);
                            let y = c.and(a, args[1][0]);
                            c.xor(x, y)
                        }
                    }]
                }
                _ => {
                    if args.iter().any(|a| a.len() != g.width) {
                        return Err("typed arithmetic width".into());
                    }
                    match g.kind.as_str() {
                        "ADD" => add_mod(c, &args[0], &args[1])?,
                        "MUL" => mul_mod(c, &args[0], &args[1])?,
                        "SUB" | "NEG" => {
                            let negand = if g.kind == "SUB" { &args[1] } else { &args[0] };
                            let inverse: Vec<_> = negand.iter().map(|w| c.not(*w)).collect();
                            let mut one = vec![Wire::Constant(false); g.width];
                            one[0] = Wire::Constant(true);
                            let negative = add_mod(c, &inverse, &one)?;
                            if g.kind == "SUB" {
                                add_mod(c, &args[0], &negative)?
                            } else {
                                negative
                            }
                        }
                        _ => return Err("typed arithmetic kind".into()),
                    }
                }
            };
            lanes.insert((g.id, 0), bits);
        }
        let mut outputs = BTreeMap::new();
        used.clear();
        for group in &self.output_groups {
            let mut bits = Vec::new();
            for index in &group.indices {
                if !used.insert(*index) {
                    return Err("duplicate typed output".into());
                }
                let endpoint = self.outputs.get(*index).ok_or("typed output index")?;
                bits.extend_from_slice(lanes.get(endpoint).ok_or("typed output endpoint")?);
            }
            if outputs.insert(group_name(group)?, bits).is_some() {
                return Err("duplicate typed output name".into());
            }
        }
        if used.len() != self.outputs.len() {
            return Err("unbound typed output".into());
        }
        let valid = outputs
            .get("OUTPUT_assertions")
            .ok_or("missing compiler validity")?;
        if valid.len() != 8 {
            return Err("compiler validity width".into());
        }
        for (i, bit) in valid.iter().enumerate() {
            c.constraints
                .push(Constraint::Equal(*bit, Wire::Constant(i == 0)));
        }
        Ok(outputs)
    }
}
