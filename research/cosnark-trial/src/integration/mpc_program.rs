//! Rust-owned generator for the pinned official MP-SPDZ compiler. This is a
//! domain adapter, not a replacement MPC implementation. Every secret opening
//! uses native checked reveal. No original witness or individual share opens.
use crate::{
    algebra::{eq_table, fold_table},
    field::F,
    integration::circuit::{self, Linear},
    pcs::{self, PADDED},
};
use ark_ff::{BigInteger, PrimeField};
use std::fmt::Write;

pub const STATE_SIZE: usize = 8 * PADDED + 3 * 2048;
pub fn prime() -> String {
    F::MODULUS.to_string()
}
fn base() -> String {
    // The official getter records the selected setting in the schedule even
    // for a linear round that does not otherwise invoke a masking helper.
    let mut s=String::from("from Compiler.types import sint, cint, Array\nfrom Compiler.library import print_ln, runtime_error_if\nprogram.bit_length = 64\nprogram.set_security(128)\nassert program.security == 128\n");
    s.push_str("views = [[sint.get_input_from(p).reveal() for j in range(8)] for p in range(7)]\n");
    s.push_str("runtime_error_if(sum(views[p][j] != views[0][j] for p in range(1,7) for j in range(8)) != 0, 'challenge view mismatch')\n");
    s
}
fn load() -> String {
    let mut s = base();
    writeln!(
        s,
        "stop, stored = sint.read_from_file(0, n_items=1, size={STATE_SIZE})"
    )
    .unwrap();
    writeln!(
        s,
        "state = Array({STATE_SIZE}, sint)\nstate.assign_vector(stored[0])"
    )
    .unwrap();
    entropy_guard(&mut s, "after persistence");
    s
}
fn entropy_guard(s: &mut String, phase: &str) {
    let mut exponent = F::MODULUS;
    exponent.sub_with_borrow(&<F as PrimeField>::BigInt::from(1u64));
    writeln!(s,"runtime_error_if((state[32768] ** {exponent}).reveal() != 1, 'mask entropy missing {phase}')").unwrap();
}
fn linear(v: &Linear, offset: usize) -> String {
    if v.is_empty() {
        return "sint(0)".into();
    }
    v.iter()
        .map(|(i, c)| format!("state[{}] * {}", offset + i, c))
        .collect::<Vec<_>>()
        .join(" + ")
}
fn emit(s: &mut String, outputs: &[String]) {
    writeln!(
        s,
        "released = [x.reveal() for x in [{}]]",
        outputs.join(",")
    )
    .unwrap();
    s.push_str("for x in released:\n    print_ln('TRIAL_PUBLIC %s', x)\n");
}
fn array(s: &mut String, name: &str, values: &[F]) {
    writeln!(
        s,
        "{name} = Array.create_from(cint(x) for x in [{}])",
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
}
fn native_dot(s: &mut String, name: &str, offset: usize, weights: &[F], other: Option<(usize, F)>) {
    // Only public, exactly-zero coefficients are omitted. The Fourier-derived
    // functional is sparse even though its coefficient domain has 16384 slots.
    // Materializing all zero entries under -M creates quadratic compiler memory
    // pressure. This is the identical full dot product, not fewer proof queries.
    let terms = weights
        .iter()
        .enumerate()
        .filter(|(_, w)| **w != F::from(0u64))
        .map(|(i, w)| {
            let value = if let Some((mask, b)) = other {
                format!("(state[{}] + state[{}] * {b})", offset + i, mask + i)
            } else {
                format!("state[{}]", offset + i)
            };
            format!("{value} * {w}")
        })
        .collect::<Vec<_>>();
    let expression = if terms.is_empty() {
        "sint(0)".to_owned()
    } else {
        terms.join(" + ")
    };
    writeln!(s, "{name} = {expression}").unwrap();
}

pub fn initialize(no_fill: bool, resumed: bool) -> String {
    initialize_inner(no_fill, resumed, false)
}

/// The existing relation with explicit six-field private Shamir inputs instead
/// of synthetic balances/quantity/price. No cross-field persistence conversion.
pub fn initialize_owned(no_fill: bool, resumed: bool) -> String {
    initialize_inner(no_fill, resumed, true)
}

fn initialize_inner(no_fill: bool, resumed: bool, owned: bool) -> String {
    let mut s = base();
    if resumed {
        writeln!(s,"stop, old = sint.read_from_file(0, n_items=1, size={STATE_SIZE})\nprevious = Array({STATE_SIZE}, sint)\nprevious.assign_vector(old[0])").unwrap();
    }
    writeln!(
        s,
        "parts = [[sint.get_input_from(p) for j in range({})] for p in range(7)]",
        if owned { 6 } else { 3 }
    )
    .unwrap();
    if owned {
        // Public Lagrange coefficients for degree-two evaluations at 1,2,3.
        // Validate all other supplied evaluations inside malicious MPC and
        // reveal only a Boolean failure, never a residual or an input value.
        let mut exponent = F::MODULUS;
        exponent.sub_with_borrow(&<F as PrimeField>::BigInt::from(1u64));
        for x in 4..=7 {
            let a = (x - 2) * (x - 3) / 2;
            let b = -((x - 1) * (x - 3));
            let c = (x - 1) * (x - 2) / 2;
            writeln!(s, "for j in range(6):\n    residual = parts[{}][j] - ({}*parts[0][j] + {}*parts[1][j] + {}*parts[2][j])\n    runtime_error_if((residual ** {}).reveal() != 0, 'owner input polynomial mismatch')", x - 1, a, b, c, exponent).unwrap();
        }
        s.push_str(
            "owner_values = [3*parts[0][j] - 3*parts[1][j] + parts[2][j] for j in range(6)]\n",
        );
    }
    writeln!(s,"randoms = Array({}, sint)\nrandoms.assign_vector(sum(sint.get_input_from(p, size={}) for p in range(7)))",8*PADDED,8*PADDED).unwrap();
    writeln!(s,"state = Array({STATE_SIZE}, sint)\nstate.assign_all(0)\nstate.assign_vector(randoms.get_vector(),base=0)").unwrap();
    writeln!(s, "for i in range({}):\n    state[i] = 0", circuit::Z_LEN).unwrap();
    if resumed {
        writeln!(
            s,
            "state.assign_vector(previous.get_vector(base={},size={PADDED}),base={})",
            6 * PADDED,
            4 * PADDED
        )
        .unwrap();
    } else if owned {
        for j in 0..4 {
            writeln!(s, "state[{}] = owner_values[{j}]", 4 * PADDED + j).unwrap();
        }
    } else {
        // Fresh distributed synthetic issuance, separately pinned by genesis.
        for j in 0..4 {
            writeln!(
                s,
                "state[{}] = {} + sum(p[{}] for p in parts)",
                4 * PADDED + j,
                if j == 0 || j == 2 { 1_000_000 } else { 1000 },
                j % 2
            )
            .unwrap();
        }
    }
    if owned {
        s.push_str("state[0] = 1\nstate[9] = owner_values[4]\nstate[10] = owner_values[5]\nstate[11] = state[9] * state[10]\n");
    } else {
        s.push_str("state[0] = 1\nstate[9] = 1 + sum(p[0] for p in parts)\nstate[10] = 1 + sum(p[1] for p in parts)\nstate[11] = state[9] * state[10] + parts[0][2]\n");
    }
    writeln!(s, "state[16] = {}", u8::from(!no_fill)).unwrap();
    for j in 0..4 {
        writeln!(s,"state[{}] = state[{}]\nstate[{}] = state[{}] {} state[{}]*state[16]\nstate[{}] = state[{}]",1+j,4*PADDED+j,5+j,1+j,if j%2==0{"-"}else{"+"},if j<2{9}else{11},6*PADDED+j,5+j).unwrap();
    }
    for j in 0..2 {
        writeln!(
            s,
            "state[{}] = state[{}]\nstate[{}] = state[{}]",
            12 + j,
            4 * PADDED + 4 + j,
            14 + j,
            6 * PADDED + 4 + j
        )
        .unwrap();
    }
    for value in 1..=11 {
        writeln!(s,"bits = state[{value}].bit_decompose(64)\nfor i in range(64):\n    state[{}+i] = bits[i]",32+(value-1)*64).unwrap();
    }
    for (i, index) in (circuit::R1..circuit::R3 + 3).enumerate() {
        writeln!(s, "state[{index}] = randoms[{i}]").unwrap();
    }
    writeln!(s,"for i in range({}):\n    state[{}+i] = 0\nfor i in range({}):\n    state[{}+i] = randoms[{}+i]",circuit::MASK_MESSAGE,2*PADDED,circuit::MASK_LEN,2*PADDED,2*PADDED).unwrap();
    let rows = circuit::rows(no_fill);
    s.push_str("residuals = Array(1024,sint)\n");
    for (i, row) in rows.iter().enumerate() {
        for (j, which) in [&row.a, &row.b, &row.c].into_iter().enumerate() {
            writeln!(
                s,
                "state[{}] = {}",
                8 * PADDED + j * 2048 + i,
                linear(which, 0)
            )
            .unwrap();
        }
        writeln!(
            s,
            "residuals[{i}] = state[{}]*state[{}]-state[{}]",
            8 * PADDED + i,
            8 * PADDED + 2048 + i,
            8 * PADDED + 4096 + i
        )
        .unwrap();
    }
    for (j, (start, length)) in [(circuit::R1, 2), (circuit::R2, 2), (circuit::R3, 3)]
        .into_iter()
        .enumerate()
    {
        for i in 0..length {
            writeln!(
                s,
                "state[{}] = state[{}]",
                8 * PADDED + j * 2048 + 1024 + i,
                start + i
            )
            .unwrap();
        }
    }
    // Exact field zero detection; unlike a short-integer comparison this does
    // not assume that a malicious input already lies in the claimed range.
    let mut exponent = F::MODULUS;
    exponent.sub_with_borrow(&<F as PrimeField>::BigInt::from(1u64));
    writeln!(s,"nonzero = Array(1024,sint)\nnonzero.assign_vector(residuals.get_vector() ** {exponent})\ncount = sum(nonzero[i] for i in range(1024))\nvalid = 1 - count ** {exponent}\nruntime_error_if(valid.reveal() != 1, 'invalid financial witness')").unwrap();
    entropy_guard(&mut s, "before persistence");
    s.push_str("sint.write_to_file(state.get_vector(), position=0)\nprint_ln('TRIAL_READY')\n");
    s
}

pub fn book_claims(old: &[F], new: &[F]) -> String {
    let mut s = load();
    native_dot(&mut s, "old_claim", 4 * PADDED, old, None);
    native_dot(&mut s, "new_claim", 6 * PADDED, new, None);
    emit(&mut s, &["old_claim".into(), "new_claim".into()]);
    s
}

pub fn initial_sums(tau: &[F]) -> String {
    let mut s = load();
    let weights = eq_table(tau);
    let terms = (0..3)
        .map(|i| {
            let left = if i < 2 {
                format!("state[{}]*state[{}]", circuit::R1 + i, circuit::R2 + i)
            } else {
                "sint(0)".into()
            };
            format!("{} * ({left} - state[{}])", weights[i], circuit::R3 + i)
        })
        .collect::<Vec<_>>()
        .join(" + ");
    writeln!(
        s,
        "k = {terms}\nks = state[{}]*2048 + sum(state[{}+i]*1024 for i in range(1,{}))",
        2 * PADDED,
        2 * PADDED,
        circuit::MASK_LEN
    )
    .unwrap();
    emit(&mut s, &["k".into(), "ks".into()]);
    s
}

pub fn constraint_round(tau: &[F], prefix: &[F], batch: F) -> String {
    assert!(prefix.len() < circuit::SUMCHECK_VARS);
    let mut s = load();
    let mut equality = eq_table(tau);
    for r in prefix {
        equality = fold_table(&equality, *r);
    }
    // All witness restrictions remain operations on native secret registers.
    for (j, name) in ["left", "right", "output"].into_iter().enumerate() {
        writeln!(
            s,
            "{name} = [state[{}+i] for i in range(2048)]",
            8 * PADDED + j * 2048
        )
        .unwrap();
        for r in prefix {
            writeln!(s,"{name} = [{name}[2*i] + {r}*({name}[2*i+1]-{name}[2*i]) for i in range(len({name})//2)]").unwrap();
        }
    }
    let count = equality.len() / 2;
    let rounds_left = circuit::SUMCHECK_VARS - prefix.len() - 1;
    let remaining_scale = F::from(1u64 << rounds_left);
    let half_scale = remaining_scale * <F as ark_ff::Field>::inverse(&F::from(2u64)).unwrap();
    let mut prefix_mask = String::from("state[32768]");
    for (i, r) in prefix.iter().enumerate() {
        let mut power = *r;
        for d in 0..3 {
            write!(
                prefix_mask,
                " + state[{}]*{}",
                2 * PADDED + 1 + 3 * i + d,
                power
            )
            .unwrap();
            power *= r;
        }
    }
    let mut outputs = Vec::new();
    for t in 0..4 {
        let at = F::from(t as u64);
        let reduced = fold_table(&equality, at);
        array(&mut s, &format!("eq{t}"), &reduced);
        writeln!(s,"terms{t} = [(left[2*i]+{t}*(left[2*i+1]-left[2*i]))*(right[2*i]+{t}*(right[2*i+1]-right[2*i]))-(output[2*i]+{t}*(output[2*i+1]-output[2*i])) for i in range({count})]").unwrap();
        let mut current = prefix_mask.clone();
        let mut power = at;
        for d in 0..3 {
            write!(
                current,
                " + state[{}]*{}",
                2 * PADDED + 1 + 3 * prefix.len() + d,
                power
            )
            .unwrap();
            power *= at;
        }
        let tail = (prefix.len() + 1..circuit::SUMCHECK_VARS)
            .flat_map(|i| (0..3).map(move |d| format!("state[{}]", 2 * PADDED + 1 + 3 * i + d)))
            .collect::<Vec<_>>();
        let tail = if tail.is_empty() {
            "sint(0)".into()
        } else {
            tail.join(" + ")
        };
        writeln!(s,"h{t} = sum(terms{t}[i]*eq{t}[i] for i in range({count})) + {batch}*({remaining_scale}*({current})+{half_scale}*({tail}))").unwrap();
        outputs.push(format!("h{t}"));
    }
    emit(&mut s, &outputs);
    s
}

pub fn endpoints(point: &[F]) -> String {
    let mut s = load();
    let weights = eq_table(point);
    for (j, name) in ["yl", "yr", "yo"].into_iter().enumerate() {
        native_dot(&mut s, name, 8 * PADDED + j * 2048, &weights, None);
    }
    native_dot(
        &mut s,
        "ym",
        2 * PADDED,
        &circuit::mask_opening_weights(point),
        None,
    );
    emit(
        &mut s,
        &["yl".into(), "yr".into(), "yo".into(), "ym".into()],
    );
    s
}

pub fn pcs_mask_claim(kind: usize, weights: &[F]) -> String {
    let mut s = load();
    native_dot(&mut s, "yr", (kind + 1) * PADDED, weights, None);
    emit(&mut s, &["yr".into()]);
    s
}

pub fn pcs_reply(kind: usize, weights: &[F], batch: F) -> String {
    let mut s = load();
    for at in 0..3 {
        native_dot(
            &mut s,
            &format!("h{at}"),
            kind * PADDED,
            &pcs::reply_message_weights(weights, F::from(at as u64)),
            Some(((kind + 1) * PADDED, batch)),
        );
    }
    emit(&mut s, &["h0".into(), "h1".into(), "h2".into()]);
    s
}
