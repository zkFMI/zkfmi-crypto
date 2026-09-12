#![cfg(all(feature = "native-prover", feature = "pqc-note-circuit"))]
//! Deterministic integration test of the generic native proof plumbing. This
//! public known-answer fixture is NOT an operational anonymous-note experiment
//! or evidence that the QOMM/OCLOB/DeFMI acceptance target has been achieved.
use ark_ff::UniformRand;
use rand::rngs::OsRng;
use sha2::{Digest, Sha512};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
    process::Command,
};
use zkfmi_cosnark_trial::{
    algebra::pack,
    field::F,
    integration::{
        circuit::QUERY_AGREEMENT_PROTOCOL,
        query_agreement::{generate_owner_key_file, TrustedRoster, HYBRID_SUITE},
    },
    note_circuit::{Circuit, Constraint, Wire},
    note_native::{Launch, PrivateInput},
    note_piop::{self, Statement, PROTOCOL},
};

fn write_new(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

#[test]
#[ignore = "requires pinned native hybrid engine and isolated remote run directory"]
fn seven_actual_native_parties_prove_and_verify_dynamic_relation() {
    let mut circuit = Circuit::default();
    let input = circuit.private_bytes(1);
    circuit
        .assert_equal(&input, &Circuit::constant_bytes(&[0xa3]))
        .unwrap();
    run_native_relation(
        circuit,
        &[0xa3],
        b"known-answer a3; native plumbing test only",
    );
}

#[test]
#[ignore = "requires pinned native hybrid engine and isolated remote run directory"]
fn seven_actual_native_parties_prove_packed_integer_relation() {
    let a = 0xffff_ffe1u32;
    let b = 0xff00_123bu32;
    let inputs: Vec<u8> = a.to_be_bytes().into_iter().chain(b.to_be_bytes()).collect();
    let mut circuit = Circuit::default();
    let private = circuit.private_bytes(8);
    let left: Vec<_> = private[..4].iter().flatten().rev().copied().collect();
    let right: Vec<_> = private[4..].iter().flatten().rev().copied().collect();
    for (wires, expected) in [
        (
            circuit.multiply_bits_le(&left, &right).unwrap(),
            u64::from(a) * u64::from(b),
        ),
        (
            circuit.add_bits_le(&left, &right).unwrap(),
            u64::from(a) + u64::from(b),
        ),
    ] {
        for (bit, wire) in wires.into_iter().enumerate() {
            circuit.constraints.push(Constraint::Equal(
                wire,
                Wire::Constant((expected >> bit) & 1 == 1),
            ));
        }
    }
    run_native_relation(
        circuit,
        &inputs,
        b"known-answer packed uint32 product and sum; native plumbing only",
    );
}

fn run_native_relation(circuit: Circuit, known_inputs: &[u8], application: &[u8]) {
    let output = std::env::var("NOTE_NATIVE_TEST_ROOT").expect("unique isolated native test root");
    let output = Path::new(&output);
    fs::create_dir(output).unwrap();
    fs::set_permissions(output, fs::Permissions::from_mode(0o700)).unwrap();
    let template = std::env::var("NOTE_NATIVE_TEST_ENGINE").expect("pinned engine template");
    qomm_mpc::engine_policy::verify(Path::new(&template)).unwrap();
    let engine = output.join("engine");
    fs::create_dir(&engine).unwrap();
    assert!(Command::new("cp")
        .arg("-a")
        .arg(Path::new(&template).join("."))
        .arg(&engine)
        .status()
        .unwrap()
        .success());
    let arithmetic = std::env::var("NOTE_NATIVE_ARITHMETIC_TEMPLATE")
        .expect("pinned arithmetic-128 public artifacts");
    for name in [
        "libSPDZ.so",
        "malicious-shamir-party.x",
        ".pqc-tls.sha256",
        "CONFIG.mine",
    ] {
        fs::copy(Path::new(&arithmetic).join(name), engine.join(name)).unwrap();
    }
    qomm_mpc::engine_policy::verify(&engine).unwrap();
    let mut paths = Vec::new();
    let mut owners = Vec::new();
    for party in 0..7 {
        let directory = output.join(format!("owner-{party}"));
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
        let path = directory.join("query.key");
        owners.push(generate_owner_key_file(&path, party).unwrap());
        paths.push(path);
    }
    let roster = TrustedRoster {
        protocol: QUERY_AGREEMENT_PROTOCOL.into(),
        suite: HYBRID_SUITE.into(),
        roster_id: "deterministic-note-native-test".into(),
        owners,
    };
    let roster_path = output.join("roster.json");
    write_new(&roster_path, &serde_json::to_vec(&roster).unwrap());
    let statement = Statement {
        protocol: PROTOCOL.into(),
        circuit_sha512: hex::encode(circuit.fingerprint()),
        application_statement_sha512: hex::encode(Sha512::digest(application)),
        query_roster_sha512: roster.sha512().unwrap(),
        session_sha512: hex::encode(Sha512::digest(output.as_os_str().as_encoded_bytes())),
    };
    let mut shares: Vec<Vec<F>> = (0..7).map(|_| Vec::new()).collect();
    for bit in 0..known_inputs.len() * 8 {
        let value = F::from(((known_inputs[bit / 8] >> (7 - bit % 8)) & 1) as u64);
        let a = F::rand(&mut OsRng);
        let b = F::rand(&mut OsRng);
        for (party, row) in shares.iter_mut().enumerate() {
            let x = F::from((party + 1) as u64);
            row.push(value + a * x + b * x * x);
        }
    }
    let mut private_input_paths = Vec::new();
    let mut private_input_sha512 = Vec::new();
    for (party, row) in shares.into_iter().enumerate() {
        let input = PrivateInput {
            party,
            application_statement_sha512: statement.application_statement_sha512.clone(),
            circuit_sha512: statement.circuit_sha512.clone(),
            session_sha512: statement.session_sha512.clone(),
            shares: pack(&row),
        };
        let bytes = serde_json::to_vec(&input).unwrap();
        let path = output.join(format!("owner-{party}/input.json"));
        write_new(&path, &bytes);
        private_input_sha512.push(hex::encode(Sha512::digest(&bytes)));
        private_input_paths.push(path);
    }
    let launch = Launch {
        statement: statement.clone(),
        circuit,
        roster_path,
        owner_key_paths: paths,
        private_input_paths,
        private_input_sha512,
    };
    let config = output.join("launch.json");
    let bytes = serde_json::to_vec(&launch).unwrap();
    write_new(&config, &bytes);
    let pin = hex::encode(Sha512::digest(&bytes));
    let native_log = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(output.join("native-run.log"))
        .unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_zkfmi-cosnark-trial"))
        .arg("note-prove")
        .arg(&engine)
        .arg("15820")
        .arg(&config)
        .arg(pin)
        .arg(output.join("result"))
        .stdout(native_log.try_clone().unwrap())
        .stderr(native_log)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "native test failed; inspect the owner-local logs in the isolated test directory"
    );
    let proof: note_piop::Proof =
        serde_json::from_slice(&fs::read(output.join("result/proof.json")).unwrap()).unwrap();
    note_piop::verify(&statement, &launch.circuit, &proof).unwrap();
    let mut different = statement.clone();
    different.application_statement_sha512 =
        hex::encode(Sha512::digest(b"wrong application statement"));
    assert!(note_piop::verify(&different, &launch.circuit, &proof).is_err());
    let mut changed = proof.clone();
    changed.prefix.endpoint_claims.replace_range(
        ..1,
        if proof.prefix.endpoint_claims.starts_with('0') {
            "1"
        } else {
            "0"
        },
    );
    assert!(note_piop::verify(&statement, &launch.circuit, &changed).is_err());
    let records: Vec<serde_json::Value> =
        serde_json::from_slice(&fs::read(output.join("result/native.json")).unwrap()).unwrap();
    assert!(!records.is_empty());
    for record in records {
        assert_eq!(
            record["party_exit_codes"],
            serde_json::json!([0, 0, 0, 0, 0, 0, 0])
        );
        assert_eq!(record["seven_context_views_equal"], true);
    }
}
