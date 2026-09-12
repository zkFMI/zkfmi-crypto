//! Rust-owned generator for the pinned official MP-SPDZ compiler. This is a
//! domain adapter, not a replacement MPC implementation. Every secret opening
//! uses native checked reveal. No original witness or individual share opens.
use crate::{algebra::{eq_table,fold_table}, circuit::{self,Linear}, field::F, pcs::{self,PADDED}};
use ark_ff::{BigInteger,PrimeField};
use std::fmt::Write;

pub const STATE_SIZE: usize = 4*PADDED + 3*128;
pub fn prime()->String { F::MODULUS.to_string() }
fn base()->String {
    // The official getter records the selected setting in the schedule even
    // for a linear round that does not otherwise invoke a masking helper.
    let mut s=String::from("from Compiler.types import sint, cint, Array\nfrom Compiler.library import print_ln, runtime_error_if\nprogram.bit_length = 64\nprogram.set_security(128)\nassert program.security == 128\n");
    s.push_str("views = [[sint.get_input_from(p).reveal() for j in range(8)] for p in range(7)]\n");
    s.push_str("runtime_error_if(sum(views[p][j] != views[0][j] for p in range(1,7) for j in range(8)) != 0, 'challenge view mismatch')\n");
    s
}
fn load()->String {
    let mut s=base();
    writeln!(s,"stop, stored = sint.read_from_file(0, n_items=1, size={STATE_SIZE})").unwrap();
    writeln!(s,"state = Array({STATE_SIZE}, sint)\nstate.assign_vector(stored[0])").unwrap();
    entropy_guard(&mut s,"after persistence");
    s
}
fn entropy_guard(s:&mut String,phase:&str) {
    let mut exponent=F::MODULUS; exponent.sub_with_borrow(&<F as PrimeField>::BigInt::from(1u64));
    writeln!(s,"runtime_error_if((state[32768] ** {exponent}).reveal() != 1, 'mask entropy missing {phase}')").unwrap();
}
fn linear(v:&Linear,offset:usize)->String {
    if v.is_empty() { return "sint(0)".into(); }
    v.iter().map(|(i,c)|format!("state[{}] * {}",offset+i,c)).collect::<Vec<_>>().join(" + ")
}
fn emit(s:&mut String,outputs:&[String]) {
    writeln!(s,"released = [x.reveal() for x in [{}]]",outputs.join(",")).unwrap();
    s.push_str("for x in released:\n    print_ln('TRIAL_PUBLIC %s', x)\n");
}
fn array(s:&mut String,name:&str,values:&[F]) {
    writeln!(s,"{name} = Array.create_from(cint(x) for x in [{}])",values.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")).unwrap();
}
fn native_dot(s:&mut String,name:&str,offset:usize,weights:&[F],other:Option<(usize,F)>) {
    array(s,&format!("{name}_weights"),weights);
    let expression=if let Some((mask,b))=other { format!("state.get_vector(base={offset}, size={}) + state.get_vector(base={mask}, size={}) * {b}",weights.len(),weights.len()) }
        else { format!("state.get_vector(base={offset}, size={})",weights.len()) };
    writeln!(s,"{name} = (({expression}) * {name}_weights.get_vector()).sum()").unwrap();
}

pub fn initialize()->String {
    let mut s=base();
    s.push_str("parts = [[sint.get_input_from(p) for j in range(3)] for p in range(7)]\n");
    writeln!(s,"randoms = Array({}, sint)\nrandoms.assign_vector(sum(sint.get_input_from(p, size={}) for p in range(7)))",4*PADDED,4*PADDED).unwrap();
    writeln!(s,"state = Array({STATE_SIZE}, sint)\nstate.assign_all(0)\nstate.assign_vector(randoms.get_vector(),base=0)").unwrap();
    writeln!(s,"for i in range({}):\n    state[i] = 0",circuit::Z_LEN).unwrap();
    s.push_str("amount = 1 + sum(p[0] for p in parts)\nprice = 1 + sum(p[1] for p in parts)\nnotional = amount * price + parts[0][2]\nremainder = 65535 - notional\nstate[0] = 1\nstate[1] = amount\nstate[2] = price\nstate[3] = notional\nstate[4] = remainder\n");
    for (variable,start,bits) in [("amount",5,8),("price",13,8),("remainder",21,16)] {
        writeln!(s,"bits = {variable}.bit_decompose({bits})\nfor i in range({bits}):\n    state[{}+i] = bits[i]",start).unwrap();
    }
    for (i,index) in (circuit::R1..circuit::R3+3).enumerate() { writeln!(s,"state[{index}] = randoms[{i}]").unwrap(); }
    writeln!(s,"for i in range({}):\n    state[{}+i] = 0\nfor i in range({}):\n    state[{}+i] = randoms[{}+i]",circuit::MASK_MESSAGE,2*PADDED,circuit::MASK_LEN,2*PADDED,2*PADDED).unwrap();
    let rows=circuit::rows();
    s.push_str("residuals = Array(64,sint)\n");
    for (i,row) in rows.iter().enumerate() {
        for (j,which) in [&row.a,&row.b,&row.c].into_iter().enumerate() {
            writeln!(s,"state[{}] = {}",4*PADDED+j*128+i,linear(which,0)).unwrap();
        }
        writeln!(s,"residuals[{i}] = state[{}]*state[{}]-state[{}]",4*PADDED+i,4*PADDED+128+i,4*PADDED+256+i).unwrap();
    }
    for (j,(start,length)) in [(circuit::R1,2),(circuit::R2,2),(circuit::R3,3)].into_iter().enumerate() {
        for i in 0..length { writeln!(s,"state[{}] = state[{}]",4*PADDED+j*128+64+i,start+i).unwrap(); }
    }
    // Exact field zero detection; unlike a short-integer comparison this does
    // not assume that a malicious input already lies in the claimed range.
    let mut exponent=F::MODULUS; exponent.sub_with_borrow(&<F as PrimeField>::BigInt::from(1u64));
    writeln!(s,"nonzero = Array(64,sint)\nnonzero.assign_vector(residuals.get_vector() ** {exponent})\ncount = sum(nonzero[i] for i in range(64))\nvalid = 1 - count ** {exponent}\nruntime_error_if(valid.reveal() != 1, 'invalid financial witness')").unwrap();
    entropy_guard(&mut s,"before persistence");
    s.push_str("sint.write_to_file(state.get_vector())\nprint_ln('TRIAL_READY')\n");
    s
}

pub fn initial_sums(tau:&[F])->String {
    let mut s=load(); let weights=eq_table(tau);
    let terms=(0..3).map(|i| {
        let left=if i<2 { format!("state[{}]*state[{}]",circuit::R1+i,circuit::R2+i) } else { "sint(0)".into() };
        format!("{} * ({left} - state[{}])",weights[i],circuit::R3+i)
    }).collect::<Vec<_>>().join(" + ");
    writeln!(s,"k = {terms}\nks = state[{}]*128 + sum(state[{}+i]*64 for i in range(1,{}))",2*PADDED,2*PADDED,circuit::MASK_LEN).unwrap();
    emit(&mut s,&["k".into(),"ks".into()]); s
}

pub fn constraint_round(tau:&[F],prefix:&[F],batch:F)->String {
    assert!(prefix.len()<circuit::SUMCHECK_VARS);
    let mut s=load();
    let mut equality=eq_table(tau);
    for r in prefix { equality=fold_table(&equality,*r); }
    // All witness restrictions remain operations on native secret registers.
    for (j,name) in ["left","right","output"].into_iter().enumerate() {
        writeln!(s,"{name} = [state[{}+i] for i in range(128)]",4*PADDED+j*128).unwrap();
        for r in prefix { writeln!(s,"{name} = [{name}[2*i] + {r}*({name}[2*i+1]-{name}[2*i]) for i in range(len({name})//2)]").unwrap(); }
    }
    let count=equality.len()/2;
    let rounds_left=circuit::SUMCHECK_VARS-prefix.len()-1;
    let remaining_scale=F::from(1u64<<rounds_left);
    let half_scale=remaining_scale * <F as ark_ff::Field>::inverse(&F::from(2u64)).unwrap();
    let mut prefix_mask=String::from("state[32768]");
    for (i,r) in prefix.iter().enumerate() {
        let mut power=*r;
        for d in 0..3 { write!(prefix_mask," + state[{}]*{}",2*PADDED+1+3*i+d,power).unwrap(); power*=r; }
    }
    let mut outputs=Vec::new();
    for t in 0..4 {
        let at=F::from(t as u64); let reduced=fold_table(&equality,at);
        array(&mut s,&format!("eq{t}"),&reduced);
        writeln!(s,"terms{t} = [(left[2*i]+{t}*(left[2*i+1]-left[2*i]))*(right[2*i]+{t}*(right[2*i+1]-right[2*i]))-(output[2*i]+{t}*(output[2*i+1]-output[2*i])) for i in range({count})]").unwrap();
        let mut current=prefix_mask.clone(); let mut power=at;
        for d in 0..3 { write!(current," + state[{}]*{}",2*PADDED+1+3*prefix.len()+d,power).unwrap(); power*=at; }
        let tail=(prefix.len()+1..circuit::SUMCHECK_VARS).flat_map(|i|(0..3).map(move |d|format!("state[{}]",2*PADDED+1+3*i+d))).collect::<Vec<_>>();
        let tail=if tail.is_empty(){"sint(0)".into()}else{tail.join(" + ")};
        writeln!(s,"h{t} = sum(terms{t}[i]*eq{t}[i] for i in range({count})) + {batch}*({remaining_scale}*({current})+{half_scale}*({tail}))").unwrap();
        outputs.push(format!("h{t}"));
    }
    emit(&mut s,&outputs); s
}

pub fn endpoints(point:&[F])->String {
    let mut s=load(); let weights=eq_table(point);
    for (j,name) in ["yl","yr","yo"].into_iter().enumerate() { native_dot(&mut s,name,4*PADDED+j*128,&weights,None); }
    native_dot(&mut s,"ym",2*PADDED,&circuit::mask_opening_weights(point),None);
    emit(&mut s,&["yl".into(),"yr".into(),"yo".into(),"ym".into()]); s
}

pub fn pcs_mask_claim(kind:usize,weights:&[F])->String {
    let mut s=load(); native_dot(&mut s,"yr",(kind+1)*PADDED,weights,None); emit(&mut s,&["yr".into()]); s
}

pub fn pcs_reply(kind:usize,weights:&[F],batch:F)->String {
    let mut s=load();
    for at in 0..3 { native_dot(&mut s,&format!("h{at}"),kind*PADDED,&pcs::reply_message_weights(weights,F::from(at as u64)),Some(((kind+1)*PADDED,batch))); }
    emit(&mut s,&["h0".into(),"h1".into(),"h2".into()]); s
}
