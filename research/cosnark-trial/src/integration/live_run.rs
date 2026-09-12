//! Atomic, manifest-bound live Avalanche verification of existing real proofs.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(hex::encode(hash.finalize()))
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn put(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn sources(path: &Path, out: &mut BTreeMap<String, String>) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            sources(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.insert(path.to_string_lossy().into(), hash_file(&path)?);
        }
    }
    Ok(())
}

fn equal_restart_roots(before: &Value, after: &Value) -> bool {
    before == after
        && before.as_array().is_some_and(|roots| {
            roots.len() == 5
                && roots.iter().all(|root| {
                    root == &roots[0]
                        && root.as_str().is_some_and(|value| {
                            value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
                        })
                })
        })
}

fn pending_restart_valid(canonical: &Value) -> bool {
    let pending = &canonical["live_network"]["pending_upload_restart"];
    let recovered = &canonical["pending_upload_recovery"];
    pending["accepted"] == true
        && pending["state_bytes"]
            .as_u64()
            .is_some_and(|n| n > 67_108_864)
        && pending["state_bytes"] == recovered["state_bytes"]
        && pending["proof_uploaded_chunks"]
            .as_u64()
            .is_some_and(|n| n > 0)
        && pending["proof_uploaded_chunks"] == pending["proof_expected_chunks"]
        && pending["proof_uploaded_chunks"] == recovered["proof_uploaded_chunks"]
        && pending["roots_before_restart"][0] == recovered["state_root"]
        && equal_restart_roots(
            &pending["roots_before_restart"],
            &pending["roots_after_restart"],
        )
}

pub fn execute(
    output: &Path,
    manifest_path: &Path,
    script: &Path,
    vm: &Path,
    driver: &Path,
    genesis: &Path,
) -> Result<(), String> {
    execute_inner(
        output,
        manifest_path,
        script,
        vm,
        driver,
        genesis,
        Path::new("live-contract.json"),
    )
}

pub fn execute_query_agreement(
    output: &Path,
    manifest_path: &Path,
    script: &Path,
    vm: &Path,
    driver: &Path,
    genesis: &Path,
) -> Result<(), String> {
    execute_inner(
        output,
        manifest_path,
        script,
        vm,
        driver,
        genesis,
        Path::new("live-query-contract.json"),
    )
}

pub fn execute_greenfield(
    output: &Path,
    manifest_path: &Path,
    script: &Path,
    vm: &Path,
    driver: &Path,
    genesis: &Path,
) -> Result<(), String> {
    execute_inner(
        output,
        manifest_path,
        script,
        vm,
        driver,
        genesis,
        Path::new("greenfield-live-contract.json"),
    )
}

fn execute_inner(
    output: &Path,
    manifest_path: &Path,
    script: &Path,
    vm: &Path,
    driver: &Path,
    genesis: &Path,
    contract_path: &Path,
) -> Result<(), String> {
    let contract = read_json(contract_path)?;
    let manifest = read_json(manifest_path)?;
    let contract_sha = hash_file(contract_path)?;
    let manifest_sha = hash_file(manifest_path)?;
    if manifest["contract_id"] != contract["contract_id"]
        || manifest["contract_sha256"] != contract_sha
        || manifest["closed_plan"] != json!(["RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC"])
        || manifest["rough_command"] != "internal:live_avalanche"
        || manifest["point_prediction"] != 1
        || contract["hard_boundaries"]["validators"] != 5
        || contract["hard_boundaries"]["operator_hosts"] != 1
        || contract["hard_boundaries"]["production_promotion"] != false
    {
        return Err("live preflight failed; no network started".into());
    }
    let query_pin = if contract["hard_boundaries"]["all_seven_hybrid_query_agreement"] == true {
        let pin = manifest["trusted_query_roster_sha512"]
            .as_str()
            .ok_or("live v2 roster pin missing")?;
        if pin.len() != 128
            || !pin
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("live v2 roster pin is not canonical".into());
        }
        Some(pin)
    } else {
        if manifest.get("trusted_query_roster_sha512").is_some() {
            return Err("query-agreement live proof requires the explicit v2 contract".into());
        }
        None
    };
    let proof_dir = Path::new(
        manifest["public_proof_directory"]
            .as_str()
            .ok_or("proof directory")?,
    );
    let mut inputs = BTreeMap::new();
    let deployment_policy = if contract["hard_boundaries"]["greenfield_deployment_policy"] == true {
        let path = Path::new(
            manifest["deployment_policy_file"]
                .as_str()
                .ok_or("greenfield policy file missing")?,
        );
        let policy: zkfmi_crypto::mode::DeploymentCryptoPolicy =
            serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        policy.validate().map_err(|e| e.to_string())?;
        let value = serde_json::to_value(&policy).map_err(|e| e.to_string())?;
        let digest = hash_file(path)?;
        if manifest["deployment_policy_sha256"] != digest
            || manifest["genesis_config_sha256"] != hash_file(genesis)?
            || manifest["deployment_crypto_policy"] != value
            || value["mode"] != contract["hard_boundaries"]["pqc_mode"]
            || read_json(genesis)?["deployment_crypto_policy"] != value
            || contract["hard_boundaries"]["existing_ledger_migration"] != false
            || contract["hard_boundaries"]["prior_ledger_or_snapshot_import"] != false
        {
            return Err("greenfield deployment/genesis policy preflight failed".into());
        }
        inputs.insert("deployment_policy".into(), digest);
        Some((path, value))
    } else {
        if manifest.get("deployment_crypto_policy").is_some() {
            return Err(
                "shared deployment policy requires the explicit greenfield contract".into(),
            );
        }
        None
    };
    for relative in [
        "receipt.json",
        "fill/proof.json",
        "fill/statement.json",
        "nofill/proof.json",
        "nofill/statement.json",
    ] {
        let digest = hash_file(&proof_dir.join(relative))?;
        if manifest["public_input_sha256"][relative] != digest {
            return Err(format!("frozen public input changed: {relative}"));
        }
        inputs.insert(format!("public/{relative}"), digest);
    }
    if let Some(pin) = query_pin {
        let native_receipt = read_json(&proof_dir.join("receipt.json"))?;
        if native_receipt["verdict"] != "smoke_only"
            || native_receipt["primary_metric_value"] != 1
            || native_receipt["trusted_query_roster_sha512"] != pin
            || native_receipt["canonical_receipt"]["trusted_query_roster_sha512"] != pin
            || native_receipt["canonical_receipt"]["proof_protocol"]
                != super::circuit::QUERY_AGREEMENT_PROTOCOL
        {
            return Err(
                "live v2 requires a completed native/canonical receipt under the pinned roster"
                    .into(),
            );
        }
    }
    if let Some((_, policy)) = &deployment_policy {
        for operation in ["fill", "nofill"] {
            if read_json(&proof_dir.join(operation).join("statement.json"))?["deployment_id"]
                != policy["deployment_id"]
            {
                return Err("greenfield policy and native proof deployment differ".into());
            }
        }
    }
    for (name, path) in [
        ("network_runner", script),
        ("vm", vm),
        ("canonical_driver", driver),
        ("genesis_config", genesis),
    ] {
        inputs.insert(name.into(), hash_file(path)?);
    }
    let mut source_hashes = BTreeMap::new();
    sources(Path::new("src"), &mut source_hashes)?;
    for p in ["Cargo.toml", "Cargo.lock"] {
        source_hashes.insert(p.into(), hash_file(Path::new(p))?);
    }
    fs::create_dir(output).map_err(|_| "fresh live output directory required")?;
    let mut claim = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("run.claim"))
        .map_err(|e| e.to_string())?;
    claim
        .write_all(manifest_sha.as_bytes())
        .map_err(|e| e.to_string())?;
    claim.sync_all().map_err(|e| e.to_string())?;
    let at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let started = Instant::now();
    let mut canonical = Value::Null;
    let result = (|| -> Result<(), String> {
        let mut command = Command::new("bash");
        command
            .arg(script)
            .arg(output)
            .arg(proof_dir)
            .arg(vm)
            .arg(driver)
            .arg(genesis)
            .env_remove("COCODE_LIVE_QUERY_ROSTER_SHA512")
            .env_remove("COCODE_LIVE_DEPLOYMENT_POLICY");
        if let Some(pin) = query_pin {
            command.env("COCODE_LIVE_QUERY_ROSTER_SHA512", pin);
        }
        if let Some((path, _)) = &deployment_policy {
            command.env("COCODE_LIVE_DEPLOYMENT_POLICY", path);
        }
        let status = command.status().map_err(|e| e.to_string())?;
        canonical = read_json(&output.join("canonical-receipt.json"))?;
        let live = &canonical["live_network"];
        if !status.success()
            || canonical["accepted"] != true
            || live["accepted"] != true
            || live["node_count"] != 5
            || !equal_restart_roots(&live["roots_before_restart"], &live["roots_after_restart"])
        {
            return Err("actual five-validator acceptance/restart gate failed".into());
        }
        if manifest["require_pending_upload_restart"] == true && !pending_restart_valid(&canonical)
        {
            return Err("large pending upload did not survive actual validator restart".into());
        }
        if query_pin.is_some_and(|pin| {
            canonical["trusted_query_roster_sha512"] != pin
                || canonical["proof_protocol"] != super::circuit::QUERY_AGREEMENT_PROTOCOL
        }) {
            return Err("live canonical receipt did not bind the expected v2 roster".into());
        }
        if let Some((_, policy)) = &deployment_policy {
            if canonical["deployment_crypto_policy"] != *policy
                || live["deployment_crypto_policy"] != *policy
            {
                return Err(
                    "live canonical/network receipt omitted or changed the genesis policy".into(),
                );
            }
        }
        Ok(())
    })();
    let receipt = json!({"experiment_id":manifest["experiment_id"],"contract_id":contract["contract_id"],
        "contract_sha256":contract_sha,"manifest_sha256":manifest_sha,"stage":contract["stage"],
        "started_unix_seconds":at,"seconds":started.elapsed().as_secs_f64(),"point_prediction":1,
        "primary_metric_value":u8::from(result.is_ok()),"verdict":if result.is_ok(){"smoke_only"}else{"blocked"},
        "canonical_receipt":canonical,"input_sha256":inputs,"source_sha256":source_hashes,
        "binary_sha256":hash_file(&std::env::current_exe().map_err(|e|e.to_string())?)?,
        "error":result.as_ref().err(),"earliest_unresolved_gate":if result.is_ok(){"Independent security/composition/quantum parameters, independent operators and full application/venue PQC migration remain unresolved."}else{"Actual five-validator proof finalization and restart recovery."}});
    put(&output.join("live-receipt.json"), &receipt)?;
    let mut ledger = OpenOptions::new()
        .create(true)
        .append(true)
        .open("live-ledger.jsonl")
        .map_err(|e| e.to_string())?;
    writeln!(
        ledger,
        "{}",
        serde_json::to_string(&receipt).map_err(|e| e.to_string())?
    )
    .map_err(|e| e.to_string())?;
    ledger.sync_all().map_err(|e| e.to_string())?;
    println!(
        "live verdict={} seconds={:.3}",
        receipt["verdict"],
        started.elapsed().as_secs_f64()
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic receipt-shape tests only; never live integration evidence.
    fn pending_receipt() -> Value {
        let root = "a".repeat(64);
        json!({"pending_upload_recovery":{"state_bytes":257_000_000,
            "proof_uploaded_chunks":367,"state_root":root},
            "live_network":{"pending_upload_restart":{"accepted":true,
                "state_bytes":257_000_000,"proof_uploaded_chunks":367,
                "proof_expected_chunks":367,"roots_before_restart":vec![root.clone();5],
                "roots_after_restart":vec![root;5]}}})
    }

    #[test]
    fn requires_five_equal_hex_roots_on_both_sides_of_restart() {
        let valid = json!(vec!["a".repeat(64); 5]);
        assert!(equal_restart_roots(&valid, &valid));
        for invalid in [Value::Null, json!([]), json!(vec!["g".repeat(64); 5])] {
            assert!(!equal_restart_roots(&invalid, &invalid));
        }
        let mut changed = valid.clone();
        changed[4] = json!("b".repeat(64));
        assert!(!equal_restart_roots(&valid, &changed));
        assert!(!equal_restart_roots(&changed, &changed));
    }

    #[test]
    fn pending_restart_requires_large_complete_upload_and_expected_root() {
        let canonical = pending_receipt();
        assert!(pending_restart_valid(&canonical));
        assert!(!pending_restart_valid(&json!({})));
        for (field, value) in [
            ("accepted", json!(false)),
            ("state_bytes", json!(67_108_864)),
            ("proof_uploaded_chunks", json!(366)),
            ("proof_expected_chunks", json!(368)),
            ("roots_after_restart", json!(vec!["b".repeat(64); 5])),
        ] {
            let mut invalid = canonical.clone();
            invalid["live_network"]["pending_upload_restart"][field] = value;
            assert!(!pending_restart_valid(&invalid));
        }
        let mut invalid = canonical;
        invalid["pending_upload_recovery"]["state_root"] = json!("b".repeat(64));
        assert!(!pending_restart_valid(&invalid));
    }
}
