//! Private per-party worker. The coordinator receives roots, masked final
//! codewords and authenticated queries, never this worker's original shares.
use crate::{algebra::{normalized_rs_fold,pack,unpack}, field::{decode,encode_row,interpolate_row,F}, mpc_program::STATE_SIZE, pcs::{self,NodeRoots,PairOpening,CODE,PADDED}, transcript::Merkle};
use ark_ff::{BigInteger,Field,PrimeField,UniformRand};
use rand::{rngs::OsRng,RngCore};
use serde::{Deserialize,Serialize};
use std::{fs::{self,OpenOptions},io::{BufRead,BufReader,Write},os::unix::fs::{OpenOptionsExt,PermissionsExt},path::Path};

#[derive(Serialize,Deserialize)]
#[serde(tag="command")]
pub enum Request {
    Input { view:String, initial:bool, invalid:bool },
    Commit,
    Fold { kind:usize, batch:String, challenge:String },
    Query { kind:usize, indices:Vec<usize> },
    Shutdown,
}

fn context_values(view:&str)->Result<Vec<u64>,String> {
    let bytes=hex::decode(view).map_err(|_|"context hex")?;
    if bytes.len()!=64 { return Err("context length".into()); }
    Ok(bytes.chunks_exact(8).map(|b|u64::from_le_bytes(b.try_into().unwrap())).collect())
}

fn write_input(root:&Path,party:usize,view:&str,initial:bool,invalid:bool)->Result<(),String> {
    let path=root.join("Player-Data").join(format!("CosnarkInput-P{party}-0"));
    let mut file=OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path).map_err(|_|"owner input file")?;
    for value in context_values(view)? { writeln!(file,"{value}").map_err(|_|"write public context")?; }
    if initial {
        // Each worker generates its own input contribution and independent
        // entropy locally. Neither appears in a coordinator request/response.
        for _ in 0..2 { writeln!(file,"{}",1+OsRng.next_u32()%15).map_err(|_|"write owner input")?; }
        writeln!(file,"{}",usize::from(invalid && party==0)).map_err(|_|"write owner fixture flag")?;
        for _ in 0..4*PADDED { writeln!(file,"{}",F::rand(&mut OsRng)).map_err(|_|"write owner entropy")?; }
    }
    file.sync_all().map_err(|_|"sync owner input".to_string())
}

fn load_own_oracles(root:&Path,party:usize)->Result<Vec<Merkle>,String> {
    let path=root.join("Persistence").join(format!("Transactions-P{party}.data"));
    let saved=qomm_mpc::persistence::read(&path,party).map_err(|_|"owner persistence parse")?;
    if saved.shares.len()!=STATE_SIZE || saved.element_bytes!=32 || saved.prime.to_bytes_be()!=F::MODULUS.to_bytes_be() {
        return Err("owner persistence field or shape mismatch".into());
    }
    let conversion=if saved.montgomery { F::from(2u64).pow([256u64]).inverse().unwrap() } else { F::from(1u64) };
    let mut oracles=Vec::new();
    for kind in 0..4 {
        let row=saved.shares[kind*PADDED..(kind+1)*PADDED].iter().map(|v| {
            let bytes=v.to_bytes_le(32).map_err(|_|"owner share width")?;
            Ok(decode(bytes.try_into().unwrap()).ok_or("owner share encoding")?*conversion)
        }).collect::<Result<Vec<F>,String>>()?;
        let coefficients=interpolate_row(&row).map_err(str::to_owned)?;
        let codeword=encode_row(&coefficients,CODE).map_err(str::to_owned)?;
        oracles.push(Merkle::new(pcs::oracle_domain(kind,party),codeword));
    }
    Ok(oracles)
}

pub fn serve(root:&Path,party:usize)->Result<(),String> {
    if party>=7 { return Err("invalid private worker index".into()); }
    fs::set_permissions(root.join("Persistence"),fs::Permissions::from_mode(0o700)).map_err(|_|"private persistence permissions")?;
    let mut oracles=Vec::new();
    let stdin=std::io::stdin(); let mut reader=BufReader::new(stdin.lock());
    let stdout=std::io::stdout(); let mut stdout=stdout.lock();
    loop {
        let mut line=String::new(); if reader.read_line(&mut line).map_err(|_|"worker input")?==0 { break; }
        if line.len()>8*1024*1024 { return Err("worker request too large".into()); }
        let request:Request=serde_json::from_str(&line).map_err(|_|"worker request encoding")?;
        let answer=match request {
            Request::Input{view,initial,invalid} => { write_input(root,party,&view,initial,invalid)?; serde_json::json!({"party":party,"view":view}) },
            Request::Commit => { oracles=load_own_oracles(root,party)?; serde_json::to_value(NodeRoots{roots:oracles.iter().map(Merkle::root).collect()}).unwrap() },
            Request::Fold{kind,batch,challenge} => {
                if (kind!=0 && kind!=2)||oracles.len()!=4 { return Err("worker oracle phase".into()); }
                let b=unpack(&batch,1)?[0]; let r=unpack(&challenge,1)?[0];
                let aggregate:Vec<F>=oracles[kind].values().iter().zip(oracles[kind+1].values()).map(|(v,m)|*v+b*m).collect();
                serde_json::json!(pack(&normalized_rs_fold(&aggregate,r)?))
            },
            Request::Query{kind,indices} => {
                if (kind!=0 && kind!=2)||oracles.len()!=4||indices.len()!=pcs::QUERIES||indices.iter().any(|i|*i>=CODE/2) { return Err("worker query shape".into()); }
                let answers:Vec<PairOpening>=indices.iter().map(|i|PairOpening{
                    original:[oracles[kind].open(*i),oracles[kind].open(*i+CODE/2)],
                    mask:[oracles[kind+1].open(*i),oracles[kind+1].open(*i+CODE/2)],
                }).collect(); serde_json::to_value(answers).unwrap()
            },
            Request::Shutdown => break,
        };
        serde_json::to_writer(&mut stdout,&answer).map_err(|_|"worker response")?;
        writeln!(stdout).map_err(|_|"worker response")?; stdout.flush().map_err(|_|"worker response")?;
    }
    Ok(())
}
