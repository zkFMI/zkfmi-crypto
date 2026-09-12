//! Atomic result-first integration runner. Outputs contain public evidence only.
use super::{native::Native, prover, verify_serialized, SettlementStatement};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn put(path: &Path, v: &impl serde::Serialize) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(v).map_err(|_| "receipt encode")?,
    )
    .map_err(|_| "public evidence write".into())
}
fn fresh_engine(
    parent: &Path,
    name: &str,
    prior: Option<&Path>,
) -> Result<std::path::PathBuf, String> {
    let root = parent.join(name);
    fs::create_dir(&root).map_err(|_| "fresh private engine required")?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
        .map_err(|_| "engine permissions")?;
    if !Command::new("cp")
        .args(["-a", "/trial/clean-engine/."])
        .arg(&root)
        .status()
        .map_err(|_| "clean engine fixture copy")?
        .success()
    {
        return Err("clean engine fixture copy failed".into());
    }
    for name in [
        "libSPDZ.so",
        "malicious-shamir-party.x",
        ".pqc-tls.sha256",
        "CONFIG.mine",
    ] {
        fs::copy(
            Path::new("/trial/native-template").join(name),
            root.join(name),
        )
        .map_err(|_| "pinned native engine artifact copy")?;
    }
    fs::create_dir_all(root.join("Persistence")).map_err(|_| "private persistence directory")?;
    if let Some(prior) = prior {
        for p in 0..7 {
            // Kernel copies only each owner's bytes; no decoding or private
            // witness/share return path exists in this coordinator.
            let claim = prior.join(format!("book-next-consumed-P{p}"));
            let mut marker = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(claim)
                .map_err(|_| "prior book capability already consumed")?;
            marker
                .write_all(name.as_bytes())
                .map_err(|_| "book consumption receipt")?;
            marker.sync_all().map_err(|_| "book consumption sync")?;
            if !prior.join(format!("book-query-used-6-P{p}")).exists() {
                return Err("prior output book was never proved".into());
            }
            let f = format!("Transactions-P{p}.data");
            fs::copy(
                prior.join("Persistence").join(&f),
                root.join("Persistence").join(f),
            )
            .map_err(|_| "owner persistence recovery")?;
            fs::copy(
                prior.join(format!("book-next-salts-P{p}")),
                root.join(format!("book-prior-salts-P{p}")),
            )
            .map_err(|_| "owner commitment salt recovery")?;
        }
    }
    Ok(root)
}
fn source_hashes(
    path: &Path,
    base: &Path,
    out: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    for e in fs::read_dir(path).map_err(|_| "source directory")? {
        let e = e.map_err(|_| "source entry")?;
        let p = e.path();
        if p.is_dir() {
            source_hashes(&p, base, out)?;
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.insert(
                p.strip_prefix(base).unwrap_or(&p).to_string_lossy().into(),
                hash(&fs::read(p).map_err(|_| "source read")?),
            );
        }
    }
    Ok(())
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryAgreementRunConfig {
    pub roster_path: PathBuf,
    pub roster_sha512: String,
    pub owner_key_paths: [PathBuf; 7],
}

/// Trusted laboratory launch data: paths and public context only. The
/// coordinator does not read any of the seven private input files.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedRunInput {
    pub binding_path: PathBuf,
    pub input_paths: [PathBuf; 7],
    pub receivers: Option<[super::input_delivery::PinnedReceiverLaunch; 7]>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedRunConfig {
    pub query: QueryAgreementRunConfig,
    pub inputs: BTreeMap<String, OwnedRunInput>,
}

fn launch_native(
    engine: &Path,
    resumed: bool,
    agreement: Option<&QueryAgreementRunConfig>,
    owned: Option<&OwnedRunInput>,
) -> Result<Native, String> {
    if let Some(input) = owned {
        let config = agreement.ok_or("owned inputs require query agreement")?;
        if let Some(receivers) = &input.receivers {
            return Native::new_query_agreement_with_sealed_inputs(
                engine,
                23840,
                resumed,
                &config.roster_path,
                &config.roster_sha512,
                &config.owner_key_paths,
                &input.binding_path,
                &input.input_paths,
                receivers,
            );
        }
        return Native::new_query_agreement_with_inputs(
            engine,
            23840,
            resumed,
            &config.roster_path,
            &config.roster_sha512,
            &config.owner_key_paths,
            &input.binding_path,
            &input.input_paths,
        );
    }
    match agreement {
        None => Native::new(engine, 23840, resumed),
        Some(config) => Native::new_query_agreement(
            engine,
            23840,
            resumed,
            &config.roster_path,
            &config.roster_sha512,
            &config.owner_key_paths,
        ),
    }
}

fn verify_candidate(
    proof: &super::verifier::Proof,
    expected: &SettlementStatement,
    agreement: Option<&QueryAgreementRunConfig>,
) -> Result<(), String> {
    match agreement {
        None => super::verifier::verify(proof, expected),
        Some(config) => {
            super::verifier::verify_query_agreement(proof, expected, &config.roster_sha512)
        }
    }
}

fn prove_candidate(
    native: &mut Native,
    statement: SettlementStatement,
    resumed: bool,
    invalid: bool,
    agreement: Option<&QueryAgreementRunConfig>,
) -> Result<super::verifier::Proof, String> {
    if agreement.is_some() {
        prover::prove_query_agreement(native, statement, resumed, invalid)
    } else {
        prover::prove(native, statement, resumed, invalid)
    }
}

pub fn execute(
    private: &Path,
    public: &Path,
    manifest_path: &Path,
    canonical: &Path,
) -> Result<(), String> {
    execute_inner(private, public, manifest_path, canonical, None, None)
}

pub fn execute_query_agreement(
    private: &Path,
    public: &Path,
    manifest_path: &Path,
    canonical: &Path,
    config_path: &Path,
) -> Result<(), String> {
    let config: QueryAgreementRunConfig = serde_json::from_slice(
        &fs::read(config_path).map_err(|_| "trusted query launch configuration missing")?,
    )
    .map_err(|_| "trusted query launch configuration syntax")?;
    super::query_agreement::TrustedRoster::read_pinned(&config.roster_path, &config.roster_sha512)?;
    execute_inner(
        private,
        public,
        manifest_path,
        canonical,
        Some(&config),
        None,
    )
}

pub fn execute_owned_inputs(
    private: &Path,
    public: &Path,
    manifest_path: &Path,
    canonical: &Path,
    config_path: &Path,
) -> Result<(), String> {
    let config_bytes = fs::read(config_path).map_err(|_| "owned launch config missing")?;
    let config: OwnedRunConfig =
        serde_json::from_slice(&config_bytes).map_err(|_| "owned launch config syntax")?;
    let manifest: Value =
        serde_json::from_slice(&fs::read(manifest_path).map_err(|_| "owned manifest missing")?)
            .map_err(|_| "owned manifest syntax")?;
    if manifest["launch_config_sha256"] != hash(&config_bytes)
        || manifest["native_binary_sha256"]
            != hash(
                &fs::read(std::env::current_exe().map_err(|_| "owned binary identity")?)
                    .map_err(|_| "owned binary read")?,
            )
        || manifest["canonical_binary_sha256"]
            != hash(&fs::read(canonical).map_err(|_| "owned canonical binary read")?)
        || config.inputs.keys().map(String::as_str).collect::<Vec<_>>()
            != vec!["fill", "invalid", "nofill"]
    {
        return Err("owned launch configuration preflight failed".into());
    }
    super::query_agreement::TrustedRoster::read_pinned(
        &config.query.roster_path,
        &config.query.roster_sha512,
    )?;
    let sealed = config
        .inputs
        .values()
        .any(|input| input.receivers.is_some());
    if sealed
        && config
            .inputs
            .values()
            .any(|input| input.receivers.is_none())
    {
        return Err("sealed input execution cannot mix in raw file fallback".into());
    }
    // Only public binding files are read here. Expected contexts come from the
    // sealed experiment, not from the input payloads.
    for (name, input) in &config.inputs {
        let bytes = fs::read(&input.binding_path).map_err(|_| "owned public binding missing")?;
        let binding: super::owner_input::OwnerInputBinding =
            serde_json::from_slice(&bytes).map_err(|_| "owned public binding syntax")?;
        binding.validate()?;
        if let Some(receivers) = &input.receivers {
            let pins: Vec<_> = receivers
                .iter()
                .map(|receiver| &receiver.config_sha256)
                .collect();
            if manifest["receiver_config_sha256"][name] != json!(pins) {
                return Err("sealed receiver manifest pins differ".into());
            }
            for (party, receiver) in receivers.iter().enumerate() {
                let trusted = super::input_delivery::TrustedInputReceiver::read_pinned(
                    &receiver.config_path,
                    &receiver.config_sha256,
                )?;
                trusted.validate(party, &binding, super::input_delivery::clock_now()?)?;
            }
        }
        if manifest["binding_sha256"][name] != hash(&bytes)
            || manifest["experiment_id"] != binding.policy.deployment_id
            || binding.book_id != "private-four-account-book"
            || binding.operation_id != *name
            || binding.sequence != if name == "nofill" { 2 } else { 1 }
            || binding.no_fill != (name == "nofill")
        {
            return Err("owned public binding preflight failed".into());
        }
    }
    execute_inner(
        private,
        public,
        manifest_path,
        canonical,
        Some(&config.query),
        Some(&config),
    )
}

fn execute_inner(
    private: &Path,
    public: &Path,
    manifest_path: &Path,
    canonical: &Path,
    agreement: Option<&QueryAgreementRunConfig>,
    owned: Option<&OwnedRunConfig>,
) -> Result<(), String> {
    let sealed = owned.is_some_and(|config| config.inputs["fill"].receivers.is_some());
    let contract_path = if sealed {
        "integration-sealed-contract.json"
    } else if owned.is_some() {
        "integration-owned-contract.json"
    } else if agreement.is_some() {
        "integration-query-contract.json"
    } else {
        "integration-contract.json"
    };
    let contract_bytes = fs::read(contract_path).map_err(|_| "integration contract missing")?;
    let contract: Value =
        serde_json::from_slice(&contract_bytes).map_err(|_| "integration contract syntax")?;
    let manifest_bytes = fs::read(manifest_path).map_err(|_| "integration manifest missing")?;
    let manifest: Value =
        serde_json::from_slice(&manifest_bytes).map_err(|_| "integration manifest syntax")?;
    if manifest["contract_sha256"] != hash(&contract_bytes)
        || manifest["contract_id"] != contract["contract_id"]
        || manifest["closed_plan"] != json!(["RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC"])
        || manifest["rough_command"] != "internal:native_to_canonical_defmi"
        || manifest["negative_gates"] != contract["negative_gates"]
        || manifest["point_prediction"] != 1
        || contract["hard_boundaries"]["mpc_parties"] != 7
        || contract["hard_boundaries"]["max_corrupt_mpc_parties"] != 2
        || contract["hard_boundaries"]["private_witness_reconstruction_at_coordinator"] != false
        || !canonical.is_file()
        || (owned.is_some() && contract["hard_boundaries"]["owner_input_fixture_fallback"] != false)
        || (sealed && contract["hard_boundaries"]["authenticated_encrypted_worker_input"] != true)
        || agreement.is_some_and(|config| {
            manifest["trusted_query_roster_sha512"] != config.roster_sha512
                || contract["hard_boundaries"]["all_seven_hybrid_query_agreement"] != true
        })
    {
        return Err("integration preflight failed; no experiment executed".into());
    }
    let mut sources = BTreeMap::new();
    source_hashes(Path::new("src"), Path::new("."), &mut sources)?;
    for p in ["Cargo.toml", "Cargo.lock", contract_path] {
        sources.insert(p.into(), hash(&fs::read(p).map_err(|_| "source identity")?));
    }
    fs::create_dir(public).map_err(|_| "fresh public output required")?;
    let mut claim = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(public.join("run.claim"))
        .map_err(|_| "atomic integration claim")?;
    claim
        .write_all(hash(&manifest_bytes).as_bytes())
        .map_err(|_| "claim binding")?;
    claim.sync_all().map_err(|_| "claim sync")?;
    fs::create_dir(private).map_err(|_| "fresh private output required")?;
    fs::set_permissions(private, fs::Permissions::from_mode(0o700))
        .map_err(|_| "private parent permissions")?;
    let started = Instant::now();
    let at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "clock")?
        .as_secs();
    let mut proofs = Vec::new();
    let mut negatives = BTreeMap::new();
    let mut canonical_receipt = Value::Null;
    let result = (|| -> Result<(), String> {
        let mut previous = None;
        let mut previous_commitment = None;
        for (name, seq, no_fill) in [("fill", 1, false), ("nofill", 2, true)] {
            let engine = fresh_engine(private, name, previous.as_deref())?;
            let out = public.join(name);
            fs::create_dir(&out).map_err(|_| "fresh public proof directory")?;
            let statement = SettlementStatement {
                deployment_id: manifest["experiment_id"]
                    .as_str()
                    .ok_or("experiment identifier")?
                    .into(),
                book_id: "private-four-account-book".into(),
                operation_id: name.into(),
                sequence: seq,
                before_commitment: "0".repeat(128),
                after_commitment: "0".repeat(128),
                no_fill,
            };
            let mut native =
                launch_native(&engine, seq > 1, agreement, owned.map(|c| &c.inputs[name]))?;
            let proof_result = prove_candidate(&mut native, statement, seq > 1, false, agreement);
            put(&out.join("native.json"), &native.records)?;
            let proof = proof_result?;
            if previous_commitment
                .as_ref()
                .is_some_and(|c| c != &proof.statement.settlement.before_commitment)
            {
                return Err("recovered book commitment differs from accepted output".into());
            }
            let bytes = serde_json::to_vec(&proof).map_err(|_| "public proof encoding")?;
            fs::write(out.join("proof.json"), &bytes).map_err(|_| "public proof output")?;
            put(&out.join("statement.json"), &proof.statement.settlement)?;
            let mut verifier_command =
                Command::new(std::env::current_exe().map_err(|_| "verifier executable")?);
            verifier_command
                .arg(if agreement.is_some() {
                    "integration-verify-v2"
                } else {
                    "integration-verify"
                })
                .arg(out.join("proof.json"))
                .arg(out.join("statement.json"));
            if let Some(config) = agreement {
                verifier_command.arg(&config.roster_sha512);
            }
            let verifier_status = verifier_command
                .status()
                .map_err(|_| "public verifier launch")?;
            if !verifier_status.success() {
                return Err("separate public verifier rejected".into());
            }
            if seq == 1 {
                if agreement.is_some() {
                    let mut changed = proof.clone();
                    let old = changed.statement.query_roster_sha512.as_bytes()[0];
                    changed
                        .statement
                        .query_roster_sha512
                        .replace_range(..1, if old == b'0' { "1" } else { "0" });
                    negatives.insert(
                        "changed trusted query roster",
                        verify_candidate(&changed, &proof.statement.settlement, agreement).is_err(),
                    );
                    negatives.insert(
                        "v2 proof rejected by explicit legacy verifier",
                        super::verifier::verify(&proof, &proof.statement.settlement).is_err(),
                    );
                }
                let mut changed = proof.clone();
                changed.witness_opening.queries[0][0].original[0]
                    .value
                    .replace_range(..1, "g");
                negatives.insert(
                    "changed codeword opening",
                    verify_candidate(&changed, &proof.statement.settlement, agreement).is_err(),
                );
                let mut changed = proof.clone();
                changed.statement.deployment_mode = "classical".into();
                negatives.insert(
                    "changed proof deployment mode",
                    verify_candidate(&changed, &proof.statement.settlement, agreement).is_err(),
                );
                let mut changed = proof.clone();
                let old = changed.statement.settlement.after_commitment.as_bytes()[0];
                changed
                    .statement
                    .settlement
                    .after_commitment
                    .replace_range(..1, if old == b'0' { "1" } else { "0" });
                negatives.insert(
                    "changed output commitment with matching external statement",
                    verify_candidate(&changed, &changed.statement.settlement, agreement).is_err(),
                );
            }
            let guards =
                native.reject_replayed_queries(proof.witness_opening.final_rows.clone())?;
            if guards.len() != 7 || guards.iter().any(|b| !*b) {
                return Err("owner one-shot guard failed".into());
            }
            proofs.push(json!({"operation":name,"proof_sha256":hash(&bytes),"proof_bytes":bytes.len(),"statement":proof.statement.settlement,"owner_query_replay_rejected":guards}));
            previous_commitment = Some(proof.statement.settlement.after_commitment.clone());
            previous = Some(engine);
        }
        let mut canonical_command = Command::new(canonical);
        canonical_command.arg(public);
        if let Some(config) = agreement {
            canonical_command
                .arg("--query-roster-sha512")
                .arg(&config.roster_sha512);
        }
        let status = canonical_command
            .status()
            .map_err(|_| "canonical acceptance executable")?;
        canonical_receipt = serde_json::from_slice(
            &fs::read(public.join("canonical-receipt.json"))
                .map_err(|_| "canonical receipt missing")?,
        )
        .map_err(|_| "canonical receipt syntax")?;
        if !status.success() || canonical_receipt["accepted"] != true {
            return Err("canonical DeFMI settlement/recovery gate failed".into());
        }
        if agreement.is_some_and(|config| {
            canonical_receipt["trusted_query_roster_sha512"] != config.roster_sha512
                || canonical_receipt["proof_protocol"] != super::circuit::QUERY_AGREEMENT_PROTOCOL
        }) {
            return Err("canonical receipt does not bind the expected v2 query roster".into());
        }
        let engine = fresh_engine(private, "invalid", None)?;
        let mut native = launch_native(
            &engine,
            false,
            agreement,
            owned.map(|c| &c.inputs["invalid"]),
        )?;
        let statement = SettlementStatement {
            deployment_id: manifest["experiment_id"].as_str().unwrap().into(),
            book_id: "private-four-account-book".into(),
            operation_id: "invalid".into(),
            sequence: 1,
            before_commitment: "0".repeat(128),
            after_commitment: "0".repeat(128),
            no_fill: false,
        };
        let invalid =
            prove_candidate(&mut native, statement, false, owned.is_none(), agreement).is_err();
        put(&public.join("invalid-native.json"), &native.records)?;
        negatives.insert(
            if owned.is_some() {
                "owner-supplied negative balance"
            } else {
                "invalid private product"
            },
            invalid
                && native.records.iter().any(|r| {
                    r.rejection_reason.as_deref() == Some("invalid financial witness")
                        && r.party_exit_codes == vec![Some(1); 7]
                }),
        );
        if negatives.values().any(|b| !*b) {
            return Err("proof/native negative gate failed".into());
        }
        Ok(())
    })();
    let receipt = json!({"contract_id":contract["contract_id"],"contract_sha256":hash(&contract_bytes),"manifest_sha256":hash(&manifest_bytes),"experiment_id":manifest["experiment_id"],"stage":"RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC","verdict":if result.is_ok(){"smoke_only"}else{"blocked"},"primary_metric_value":u8::from(result.is_ok()),"point_prediction":1,"started_unix_seconds":at,"seconds":started.elapsed().as_secs_f64(),"proofs":proofs,"proof_negative_gates":negatives,"canonical_receipt":canonical_receipt,"source_sha256":sources,"binary_sha256":hash(&fs::read(std::env::current_exe().map_err(|_|"binary identity")?).map_err(|_|"binary read")?),"canonical_binary_sha256":hash(&fs::read(canonical).map_err(|_|"canonical binary identity")?),"error":result.as_ref().err(),"earliest_unresolved_gate":if result.is_ok(){"Independent composition, quantum parameters, operator isolation and venue-specific adoption; no production promotion."}else{"Actual native-to-canonical integration completion."}});
    let mut receipt = receipt;
    if owned.is_some() {
        receipt["input_path"] = json!(if sealed {
            "integration-worker-v2-sealed; no raw file fallback"
        } else {
            "integration-worker-v2-owned; no fixture fallback"
        });
        receipt["launch_config_sha256"] = manifest["launch_config_sha256"].clone();
        receipt["binding_sha256"] = manifest["binding_sha256"].clone();
        receipt["input_provenance"] = json!(if sealed {
            "laboratory owner process; signed hybrid-encrypted packets, externally pinned registration, worker-local authentication/decryption and durable one-shot consumption; not operational venue enrollment or account/order authority"
        } else {
            "laboratory owner process, private local files; not authenticated venue enrollment or encrypted worker transport"
        });
        if sealed {
            receipt["receiver_config_sha256"] = manifest["receiver_config_sha256"].clone();
        }
    }
    if let Some(config) = agreement {
        receipt["trusted_query_roster_sha512"] = json!(config.roster_sha512);
        receipt["operator_isolation"] =
            json!("laboratory-only, same-host launcher; not independent operator custody");
    }
    put(&public.join("receipt.json"), &receipt)?;
    let mut ledger = OpenOptions::new()
        .append(true)
        .create(true)
        .open("integration-ledger.jsonl")
        .map_err(|_| "integration ledger")?;
    writeln!(ledger, "{}", serde_json::to_string(&receipt).unwrap())
        .map_err(|_| "integration ledger append")?;
    ledger.sync_all().map_err(|_| "integration ledger sync")?;
    println!(
        "integration verdict={} seconds={:.3}",
        receipt["verdict"],
        started.elapsed().as_secs_f64()
    );
    result
}
pub fn verify_files(proof: &Path, statement: &Path) -> Result<(), String> {
    if fs::metadata(proof).map_err(|_| "proof metadata")?.len() > 256 * 1024 * 1024 {
        return Err("proof size".into());
    }
    let statement: SettlementStatement =
        serde_json::from_slice(&fs::read(statement).map_err(|_| "statement read")?)
            .map_err(|_| "statement syntax")?;
    verify_serialized(&fs::read(proof).map_err(|_| "proof read")?, &statement)
}

pub fn verify_query_agreement_files(
    proof: &Path,
    statement: &Path,
    trusted_roster_sha512: &str,
) -> Result<(), String> {
    if fs::metadata(proof).map_err(|_| "proof metadata")?.len() > 256 * 1024 * 1024 {
        return Err("proof size".into());
    }
    let statement: SettlementStatement =
        serde_json::from_slice(&fs::read(statement).map_err(|_| "statement read")?)
            .map_err(|_| "statement syntax")?;
    super::verify_serialized_query_agreement(
        &fs::read(proof).map_err(|_| "proof read")?,
        &statement,
        trusted_roster_sha512,
    )
}
