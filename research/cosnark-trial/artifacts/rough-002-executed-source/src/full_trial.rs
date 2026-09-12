//! Atomic closed-plan launch and public postflight for the whole candidate.
use crate::{algebra::{pack,unpack},circuit::Statement,field::F,mpc_program,native::Native,prover,transcript::Transcript,verifier::{self,Proof}};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fs::{self,OpenOptions},io::Write,path::Path,process::Command,time::{Instant,SystemTime,UNIX_EPOCH}};

fn hash(bytes:&[u8])->String { hex::encode(Sha256::digest(bytes)) }
fn flip(hex:&mut String) { let old=hex.as_bytes()[0]; hex.replace_range(..1,if old==b'0'{"1"}else{"0"}); }

pub fn verify_file(proof_path:&Path,statement_path:&Path)->Result<(),String> {
    if fs::metadata(proof_path).map_err(|_|"proof metadata")?.len()>512*1024*1024 { return Err("proof size limit".into()); }
    let proof:Proof=serde_json::from_slice(&fs::read(proof_path).map_err(|_|"public proof read")?).map_err(|_|"public proof syntax")?;
    let statement:Statement=serde_json::from_slice(&fs::read(statement_path).map_err(|_|"public statement read")?).map_err(|_|"public statement syntax")?;
    verifier::verify(&proof,&statement)
}

pub fn execute(engine:&Path,output:&Path)->Result<(),String> {
    let contract_bytes=fs::read("contract.json").map_err(|_|"contract missing")?;
    let manifest_bytes=fs::read("experiment-002.json").map_err(|_|"experiment manifest missing")?;
    let contract:Value=serde_json::from_slice(&contract_bytes).map_err(|_|"contract syntax")?;
    let manifest:Value=serde_json::from_slice(&manifest_bytes).map_err(|_|"manifest syntax")?;
    let contract_hash=hash(&contract_bytes);
    if manifest["contract_sha256"]!=contract_hash || manifest["contract_id"]!=contract["contract_id"]
        || manifest["closed_plan"]!=json!(["RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC"])
        || manifest["rough_command"]!="internal:full_native_prover"
        || manifest["negative_cases"]!=contract["negative_cases"]
        || contract["hard_boundaries"]["mpc_parties"]!=7
        || contract["hard_boundaries"]["max_corrupt_mpc_parties"]!=2
        || contract["hard_boundaries"]["private_witness_reconstruction_at_coordinator"]!=false {
        return Err("whole-candidate research preflight failed".into());
    }
    fs::create_dir(output).map_err(|_|"fresh whole-candidate output directory required")?;
    let _claim=OpenOptions::new().write(true).create_new(true).open(output.join("run.claim")).map_err(|_|"atomic run claim")?;
    let started=Instant::now();
    let at=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|"system time")?.as_secs();
    let statement=Statement::smoke(format!("cosnark-rough-002-{at}"));
    let statement_path=output.join("statement.json");
    fs::write(&statement_path,serde_json::to_vec_pretty(&statement).unwrap()).map_err(|_|"public statement output")?;
    let mut native=Native::new(engine,23840)?;
    let mut negatives=BTreeMap::new();
    let mut standalone=false;
    let mut proof_hash=None;
    let result=(||->Result<(),String> {
        let proof=prover::prove(&mut native,&statement)?;
        let proof_path=output.join("proof.json");
        let bytes=serde_json::to_vec(&proof).map_err(|_|"public proof encoding")?;
        proof_hash=Some(hash(&bytes));
        fs::write(&proof_path,bytes).map_err(|_|"public proof output")?;
        standalone=Command::new(std::env::current_exe().map_err(|_|"verifier executable")?)
            .arg("verify").arg(&proof_path).arg(&statement_path).status().map_err(|_|"standalone verifier launch")?.success();
        if !standalone { return Err("standalone public-only verifier rejected honest proof".into()); }
        macro_rules! mutation { ($name:expr,$change:expr) => {{
            let mut changed=proof.clone(); $change(&mut changed);
            negatives.insert($name.to_string(),verifier::verify(&changed,&statement).is_err());
        }}; }
        let mut changed=proof.clone(); let mut another=statement.clone(); another.deployment_id.push_str("-different-public-instance"); changed.statement=another.clone();
        negatives.insert("changed public statement".into(),verifier::verify(&changed,&another).is_err());
        mutation!("changed proof commitment root",|p:&mut Proof|flip(&mut p.roots[0].roots[0]));
        mutation!("changed codeword opening",|p:&mut Proof|flip(&mut p.witness_opening.queries[0][0].original[0].value));
        mutation!("changed final evaluation claim",|p:&mut Proof|{
            let mut y=unpack(&p.endpoint_claims,4).unwrap(); y[0]+=F::from(1u64); p.endpoint_claims=pack(&y);
        });
        mutation!("changed deployment mode or identifier",|p:&mut Proof|p.statement.deployment_mode="classical".into());
        let mut test_view=Transcript::new(b"native-two-party-context-negative");
        test_view.absorb(b"proof",proof.transcript_sha512.as_bytes());
        let rejected=native.run(mpc_program::endpoints(&vec![F::from(2u64);7]),test_view.view(),false,false,true).is_err();
        let correct_marker=native.records.last().and_then(|r|r.rejection_reason.as_deref())==Some("challenge view mismatch");
        negatives.insert("inconsistent per-party challenge view".into(),rejected && correct_marker);
        let rejected=native.run(mpc_program::initialize(),test_view.view(),true,true,false).is_err();
        let correct_marker=native.records.last().and_then(|r|r.rejection_reason.as_deref())==Some("invalid financial witness");
        negatives.insert("invalid witness".into(),rejected && correct_marker);
        if negatives.len()!=7 || negatives.values().any(|pass|!*pass) { return Err("declared negative gate failed".into()); }
        Ok(())
    })();
    let mut inputs=serde_json::Map::new();
    for path in ["Cargo.toml","Cargo.lock","contract.json","experiment-002.json","src/lib.rs","src/main.rs","src/field.rs","src/algebra.rs","src/circuit.rs","src/transcript.rs","src/pcs.rs","src/verifier.rs","src/mpc_program.rs","src/party.rs","src/native.rs","src/prover.rs","src/full_trial.rs"] {
        inputs.insert(path.into(),json!(hash(&fs::read(path).map_err(|_|"source identity input missing")?)));
    }
    let passed=result.is_ok();
    let receipt=json!({
        "experiment_id":manifest["experiment_id"],"contract_id":contract["contract_id"],"contract_sha256":contract_hash,
        "manifest_sha256":hash(&manifest_bytes),"stage":contract["stage"],"verdict":if passed{"smoke_only"}else{"blocked"},
        "primary_metric_observed":standalone,"primary_metric_value":if standalone{Some(usize::from(passed))}else{None},
        "point_prediction":1,"started_unix_seconds":at,"duration_seconds":started.elapsed().as_secs_f64(),
        "standalone_public_verifier_accepted":standalone,"negative_cases":negatives,"native_runs":native.records,
        "proof_sha256":proof_hash,"source_inputs":inputs,"failure":result.err(),
        "coordinator_private_witness_api":false,"private_inputs_generated_in_per_party_children":true,
        "native_parties":7,"native_corrupt_threshold":2,"single_host":true,"independent_operator_deployment":false,
        "production_security_or_post_quantum_parameter_promotion":false,
        "earliest_unresolved_gate":if passed{"Independent validation of the normalized/masking composition and security parameters; operational financial commitment adapters, DeFMI transaction/restart lifecycle and independent-operator acceptance remain unverified."}else{"Complete the failed whole-candidate execution; do not replace it with arithmetic-only evidence."}
    });
    fs::write(output.join("receipt.json"),serde_json::to_vec_pretty(&receipt).unwrap()).map_err(|_|"public receipt output")?;
    let mut ledger=OpenOptions::new().create(true).append(true).open("artifacts/ledger.jsonl").map_err(|_|"append-only research ledger")?;
    writeln!(ledger,"{}",serde_json::to_string(&receipt).unwrap()).map_err(|_|"append result")?;
    ledger.sync_all().map_err(|_|"sync result")?;
    println!("{}",serde_json::to_string_pretty(&json!({"verdict":receipt["verdict"],"standalone_public_verifier_accepted":standalone,"negative_cases":receipt["negative_cases"],"duration_seconds":receipt["duration_seconds"],"receipt":output.join("receipt.json"),"failure":receipt["failure"]})).unwrap());
    if passed { Ok(()) } else { Err("whole-candidate run blocked; see its public receipt".into()) }
}
