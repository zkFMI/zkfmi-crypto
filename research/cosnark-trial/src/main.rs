use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let contract_bytes = fs::read("contract.json")?;
    let manifest_bytes = fs::read("experiment.json")?;
    let contract: Value = serde_json::from_slice(&contract_bytes)?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)?;
    let contract_hash = hash(&contract_bytes);
    if manifest["contract_sha256"] != contract_hash
        || contract["contract_id"] != manifest["contract_id"]
        || manifest["closed_plan"] != json!(["RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC"])
        || manifest["rough_command"] != "internal:fold_binding_probe"
        || contract["hard_boundaries"]["mpc_parties"] != 7
        || contract["hard_boundaries"]["max_corrupt_mpc_parties"] != 2
        || contract["hard_boundaries"]["private_witness_reconstruction_at_coordinator"] != false
    {
        return Err("research preflight failed; no run executed".into());
    }
    fs::create_dir_all("artifacts")?;
    // Atomic reservation and immediate in-process observation: no intermediate
    // prepared state permits unrelated diagnostics before this rough execution.
    let claim = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("artifacts/run.claim")?;
    let started = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let observed = zkfmi_cosnark_trial::fold_binding_probe::run();
    drop(claim);
    let observation = serde_json::to_vec_pretty(&observed)?;
    fs::write("artifacts/observation.json", &observation)?;
    let mut inputs = serde_json::Map::new();
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "contract.json",
        "experiment.json",
        "src/main.rs",
        "src/lib.rs",
        "src/field.rs",
        "src/fold_binding_probe.rs",
    ] {
        inputs.insert(path.into(), json!(hash(&fs::read(path)?)));
    }
    let receipt = json!({
        "experiment_id": manifest["experiment_id"],
        "contract_id": contract["contract_id"],
        "contract_sha256": contract_hash,
        "manifest_sha256": hash(&manifest_bytes),
        "stage": contract["stage"],
        "started_unix_seconds": started,
        "finished_unix_seconds": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        "verdict": "blocked",
        "primary_metric_observed": false,
        "primary_metric_value": null,
        "declared_negative_cases_executed": 0,
        "observation_sha256": hash(&observation),
        "inputs": inputs,
        "earliest_unresolved_gate": "Resolve the coPCS fold-to-MLE binding; then execute the full private financial R1CS with seven-party malicious MPC and required negative cases.",
        "mpc_artifact": null,
        "public_proof_artifact": null,
        "reason": observed.blocker
    });
    fs::write(
        "artifacts/receipt.json",
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    let ledger_path = Path::new("artifacts/ledger.jsonl");
    let mut ledger = OpenOptions::new()
        .append(true)
        .create(true)
        .open(ledger_path)?;
    writeln!(ledger, "{}", serde_json::to_string(&receipt)?)?;
    println!("{}", serde_json::to_string_pretty(&observed)?);
    println!("receipt=artifacts/receipt.json; verdict=blocked; full_candidate_executed=false");
    Ok(())
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 1 {
        let result = match args[1].as_str() {
            #[cfg(feature = "pqc-note-circuit")]
            "note-prove" if args.len() == 7 => {
                args[3].parse::<u16>().map_err(|_|"note native port".to_string()).and_then(|port| {
                    zkfmi_cosnark_trial::note_native::prove_to_directory(Path::new(&args[2]),port,Path::new(&args[4]),&args[5],Path::new(&args[6]))
                })
            }
            #[cfg(feature = "pqc-note-circuit")]
            "note-worker" if args.len() == 6 => {
                args[3].parse::<usize>().map_err(|_|"note party index".to_string()).and_then(|party| {
                    zkfmi_cosnark_trial::note_native::serve(Path::new(&args[2]),party,Path::new(&args[4]),&args[5])
                })
            }
            "integration-lab-input-keygen" if args.len() == 5 => {
                zkfmi_cosnark_trial::integration::input_delivery::generate_lab_key(
                    &args[2], Path::new(&args[3]), Path::new(&args[4]),
                )
            }
            "integration-lab-seal-inputs" if args.len() == 8 => {
                zkfmi_cosnark_trial::integration::input_delivery::seal_lab_inputs(
                    Path::new(&args[2]), Path::new(&args[3]), Path::new(&args[4]),
                    &args[5], Path::new(&args[6]), Path::new(&args[7]),
                )
            }
            "integration-worker-v2-sealed" if args.len() == 12 => {
                args[3].parse::<usize>().map_err(|_| "worker index".to_string()).and_then(|party| {
                    zkfmi_cosnark_trial::integration::party::serve_query_agreement_with_sealed_input(
                        Path::new(&args[2]), party, Path::new(&args[4]), &args[5], Path::new(&args[6]),
                        Path::new(&args[7]), Path::new(&args[8]),
                        &zkfmi_cosnark_trial::integration::input_delivery::PinnedReceiverLaunch {
                            config_path: args[9].clone().into(), config_sha256: args[10].clone(),
                            recipient_key_path: args[11].clone().into(),
                        },
                    )
                })
            }
            "integration-lab-owner-inputs" if args.len() == 5 => {
                zkfmi_cosnark_trial::integration::owner_input::prepare_lab_files(
                    Path::new(&args[2]), Path::new(&args[3]), Path::new(&args[4]),
                )
            }
            "integration-full-owned" if args.len() == 7 => {
                zkfmi_cosnark_trial::integration::run::execute_owned_inputs(
                    Path::new(&args[2]), Path::new(&args[3]), Path::new(&args[4]),
                    Path::new(&args[5]), Path::new(&args[6]),
                )
            }
            "integration-live-greenfield" if args.len() == 8 => {
                zkfmi_cosnark_trial::integration::live_run::execute_greenfield(
                    Path::new(&args[2]), Path::new(&args[3]), Path::new(&args[4]),
                    Path::new(&args[5]), Path::new(&args[6]), Path::new(&args[7]),
                )
            }
            "integration-live-v2" if args.len() == 8 => {
                zkfmi_cosnark_trial::integration::live_run::execute_query_agreement(
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                    Path::new(&args[4]),
                    Path::new(&args[5]),
                    Path::new(&args[6]),
                    Path::new(&args[7]),
                )
            }
            "integration-full-v2" if args.len() == 7 => {
                zkfmi_cosnark_trial::integration::run::execute_query_agreement(
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                    Path::new(&args[4]),
                    Path::new(&args[5]),
                    Path::new(&args[6]),
                )
            }
            "integration-verify-v2" if args.len() == 5 => {
                zkfmi_cosnark_trial::integration::run::verify_query_agreement_files(
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                    &args[4],
                )
            }
            "integration-live" if args.len() == 8 => {
                zkfmi_cosnark_trial::integration::live_run::execute(
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                    Path::new(&args[4]),
                    Path::new(&args[5]),
                    Path::new(&args[6]),
                    Path::new(&args[7]),
                )
            }
            "integration-full" if args.len() == 6 => {
                zkfmi_cosnark_trial::integration::run::execute(
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                    Path::new(&args[4]),
                    Path::new(&args[5]),
                )
            }
            "integration-verify" if args.len() == 4 => {
                zkfmi_cosnark_trial::integration::run::verify_files(
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                )
            }
            "integration-worker" if args.len() == 4 => args[3]
                .parse::<usize>()
                .map_err(|_| "worker index".to_string())
                .and_then(|p| {
                    zkfmi_cosnark_trial::integration::party::serve(Path::new(&args[2]), p)
                }),
            "integration-worker-v2-owned" if args.len() == 9 => args[3]
                .parse::<usize>()
                .map_err(|_| "worker index".to_string())
                .and_then(|p| {
                    zkfmi_cosnark_trial::integration::party::serve_query_agreement_with_input(
                        Path::new(&args[2]), p, Path::new(&args[4]), &args[5],
                        Path::new(&args[6]), Path::new(&args[7]), Path::new(&args[8]),
                    )
                }),
            "integration-worker-v2" if args.len() == 7 => args[3]
                .parse::<usize>()
                .map_err(|_| "worker index".to_string())
                .and_then(|p| {
                    zkfmi_cosnark_trial::integration::party::serve_query_agreement(
                        Path::new(&args[2]),
                        p,
                        Path::new(&args[4]),
                        &args[5],
                        Path::new(&args[6]),
                    )
                }),
            "integration-query-owner-keygen" if args.len() == 4 => args[3]
                .parse::<usize>()
                .map_err(|_| "owner index".to_string())
                .and_then(|party| {
                    let enrollment = zkfmi_cosnark_trial::integration::query_agreement::generate_owner_key_file(
                        Path::new(&args[2]),
                        party,
                    )?;
                    println!(
                        "{}",
                        serde_json::to_string(&enrollment)
                            .map_err(|_| "owner enrollment encoding")?
                    );
                    Ok(())
                }),
            "integration-query-roster-digest" if args.len() == 3 => {
                zkfmi_cosnark_trial::integration::query_agreement::TrustedRoster::read_for_pinning(
                    Path::new(&args[2]),
                )
                .and_then(|roster| roster.sha512())
                .map(|digest| println!("{digest}"))
            }
            "worker" if args.len() == 4 => args[3]
                .parse::<usize>()
                .map_err(|_| "worker index".to_string())
                .and_then(|party| zkfmi_cosnark_trial::party::serve(Path::new(&args[2]), party)),
            "verify" if args.len() == 4 => zkfmi_cosnark_trial::full_trial::verify_file(
                Path::new(&args[2]),
                Path::new(&args[3]),
            ),
            "full" if args.len() == 4 || args.len() == 5 => {
                zkfmi_cosnark_trial::full_trial::execute(
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                    Path::new(
                        args.get(4)
                            .map(String::as_str)
                            .unwrap_or("experiment-002.json"),
                    ),
                )
            }
            _ => Err(
                "usage: full ENGINE NEW_OUTPUT | verify PROOF STATEMENT | worker ENGINE PARTY | integration-query-owner-keygen OWNER_KEY PARTY | integration-query-roster-digest ROSTER_JSON"
                    .into(),
            ),
        };
        if let Err(error) = result {
            eprintln!("trial error: {error}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(error) = execute() {
        eprintln!("trial error: {error}");
        std::process::exit(1);
    }
}
