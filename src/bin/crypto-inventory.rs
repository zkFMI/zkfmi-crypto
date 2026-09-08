//! Deterministic inventory tool. Read-only input; all outputs belong to this repository.
#![forbid(unsafe_code)]
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const REPOS: [&str; 7] = ["qomm", "zkpi", "defmi", "oclob", "dekyx", "deccp", "aethel"];
// External crate identifiers; frost RPC/function names are not crate dependencies.
const GREP_PATTERN: &str = r"(^|[^[:alnum:]_])(ed25519_dalek|frost_(core|ristretto255)|curve25519_dalek|bulletproofs|merlin|openssl|sha2|sha3)([^[:alnum:]_]|$)";
const PRIMITIVES: [&str; 9] = [
    "ed25519_dalek",
    "frost_core",
    "frost_ristretto255",
    "curve25519_dalek",
    "bulletproofs",
    "merlin",
    "openssl",
    "sha2",
    "sha3",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum QuantumStatus {
    InformationTheoretic,
    BrokenByQuantum,
    GroverOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum Phase {
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    repo: String,
    #[serde(rename = "crate")]
    crate_name: String,
    file: String,
    primitive: String,
    current_suite: String,
    purpose: String,
    owner: String,
    quantum_status: QuantumStatus,
    replacement_candidate: String,
    migration_phase: Phase,
    notes: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFile {
    repo: String,
    file: String,
    sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRepo {
    repo: String,
    head: String,
    dirty_paths: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sources {
    captured_at: String,
    grep_pattern: String,
    repositories: Vec<SourceRepo>,
    files: Vec<SourceFile>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CountComparison {
    crate_name: String,
    repo: String,
    handoff: usize,
    all_rust: usize,
    src_and_tests: usize,
    src_all: usize,
    src_non_hash: usize,
    explanation: String,
}

fn scan(root: &Path) -> Result<(Vec<Entry>, Sources)> {
    let mut output = Vec::new();
    let mut sources = Sources {
        captured_at: fs::read_to_string(root.join("captured_at.txt"))?
            .trim()
            .into(),
        grep_pattern: GREP_PATTERN.into(),
        repositories: Vec::new(),
        files: Vec::new(),
    };
    let patterns: Vec<_> = PRIMITIVES
        .iter()
        .map(|name| {
            Regex::new(&format!(r"\b{}\b", regex::escape(name))).expect("fixed literal regex")
        })
        .collect();
    for repo in REPOS {
        let repo_root = root.join(repo);
        sources.repositories.push(SourceRepo {
            repo: repo.into(),
            head: fs::read_to_string(root.join(format!("{repo}.head")))?
                .trim()
                .into(),
            dirty_paths: fs::read_to_string(root.join(format!("{repo}.status")))?
                .lines()
                .map(str::to_owned)
                .collect(),
        });
        for path in files(&repo_root)? {
            let relative = path
                .strip_prefix(&repo_root)?
                .to_str()
                .ok_or("non-UTF8 source path")?;
            let bytes = fs::read(&path)?;
            sources.files.push(SourceFile {
                repo: repo.into(),
                file: relative.into(),
                sha256: hash(&bytes),
            });
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let content = std::str::from_utf8(&bytes)?;
            let name = crate_name(&path, &repo_root)?;
            for (primitive, pattern) in PRIMITIVES.iter().zip(&patterns) {
                let lines: Vec<_> = content
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| pattern.is_match(line))
                    .map(|(i, _)| i + 1)
                    .collect();
                if !lines.is_empty() {
                    output.extend(classify(repo, &name, relative, primitive, content, &lines));
                }
            }
        }
    }
    output.sort_by(|a, b| {
        (&a.repo, &a.crate_name, &a.file, &a.primitive).cmp(&(
            &b.repo,
            &b.crate_name,
            &b.file,
            &b.primitive,
        ))
    });
    Ok((output, sources))
}

fn under(file: &str, directory: &str) -> bool {
    Path::new(file)
        .components()
        .any(|c| c.as_os_str() == directory)
}

fn comparisons(entries: &[Entry]) -> Vec<CountComparison> {
    let mut expected = BTreeMap::from([
        ("qomm-transport", ("defmi", 34)),
        ("defmi", ("defmi", 25)),
        ("qomm-proofs", ("defmi", 12)),
        ("qomm-harness", ("defmi", 12)),
        ("zkfmi-zk", ("defmi", 9)),
        ("oclob-node", ("oclob", 9)),
        ("qomm-avalanche-vm", ("defmi", 8)),
        ("zkpi", ("defmi", 7)),
        ("zkpi-defmi-sdk", ("defmi", 3)),
        ("oclob-settlement", ("oclob", 3)),
        ("oclob-demo", ("oclob", 3)),
        ("qomm-audit", ("defmi", 3)),
        ("aethel-provider-sdk", ("aethel", 3)),
        ("dekyx-core", ("dekyx", 1)),
        ("deccp-core", ("deccp", 1)),
    ]);
    for entry in entries
        .iter()
        .filter(|e| e.repo == "oclob" && e.crate_name.starts_with("oclob-"))
    {
        expected
            .entry(entry.crate_name.as_str())
            .or_insert(("oclob", 1));
    }
    expected.into_iter().map(|(name, (repo, handoff))| {
        let selected: Vec<_> = entries.iter().filter(|e| e.repo == repo && e.crate_name == name).collect();
        let unique = |predicate: &dyn Fn(&Entry) -> bool| selected.iter().filter(|e| predicate(e)).map(|e| &e.file).collect::<BTreeSet<_>>().len();
        let all_rust = unique(&|_| true);
        let src_and_tests = unique(&|e| under(&e.file, "src") || under(&e.file, "tests"));
        let src_all = unique(&|e| under(&e.file, "src"));
        let src_non_hash = unique(&|e| under(&e.file, "src") && e.primitive != "sha2" && e.primitive != "sha3");
        let explanation = if src_and_tests == handoff { "The work-order count matches the complete src+tests set" }
        else if src_non_hash == handoff { "The work-order count matches non-hash references in src. The current complete set also includes hash-only files and tests" }
        else { "The count differs from the work order. Counts for src+tests, src only, and non-hash src are shown. The original file set was not provided, so the cause of the residual difference is undetermined" }.into();
        CountComparison { crate_name: name.into(), repo: repo.into(), handoff, all_rust, src_and_tests, src_all, src_non_hash, explanation }
    }).collect()
}

fn markdown(entries: &[Entry], counts: &[CountComparison], sources: &Sources) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("# Cryptographic usage inventory\n\nThis file is generated by the Rust tool from `crypto_inventory.json`, `count_reconciliation.json`, and `source_manifest.json`. Do not edit it manually.\n\n");
    let total_files = entries
        .iter()
        .map(|e| (&e.repo, &e.file))
        .collect::<BTreeSet<_>>()
        .len();
    writeln!(out, "Fixed snapshot: `{}`. Covers 7 repositories, {} files, and {} primitive entries. SHA-256 hashes of {} Cargo and Rust input files are retained in source_manifest.\n", sources.captured_at, total_files, entries.len(), sources.files.len()).unwrap();
    out.push_str("External crate identifiers are found by lexical search in Rust source. The scope includes `.rs` files not excluded by `.gitignore`, including src/tests/examples. References in comments and re-exports are retained. `frost_*` detects the actual external crates `frost_core` / `frost_ristretto255`; RPC names and local function names are not treated as crate usage. The owner identifies a repository/crate code responsibility boundary, not a newly assigned individual.\n\nQuantum classifications describe the guarantees provided by each primitive. Shamir's conditional perfect secrecy, Pedersen's perfect hiding and vulnerable binding, and reduced hash security margins under generic quantum search are treated separately. Migration order and purpose are P0 assignments based on the plan and the roles of crates/modules, not production authorization policies. The new crate is not integrated into existing services.\n\n## Summary and source snapshots\n\n| repo | HEAD | included files | primitive rows |\n| --- | --- | ---: | ---: |\n");
    for repo in &sources.repositories {
        let selected: Vec<_> = entries.iter().filter(|e| e.repo == repo.repo).collect();
        let count = selected
            .iter()
            .map(|e| &e.file)
            .collect::<BTreeSet<_>>()
            .len();
        writeln!(
            out,
            "| {} | `{}` | {} | {} |",
            repo.repo,
            repo.head,
            count,
            selected.len()
        )
        .unwrap();
    }
    out.push_str("\n## Reconciliation with work-order counts\n\nThe defmi copies represent shared crates. Duplicated copies across repositories are not summed when comparing against the work-order counts. The following table uses the same fixed snapshot.\n\n| crate (repo) | work order | all src+tests | all src | non-hash src | explanation |\n| --- | ---: | ---: | ---: | ---: | --- |\n");
    for c in counts {
        writeln!(
            out,
            "| {} ({}) | {} | {} | {} | {} | {} |",
            c.crate_name,
            c.repo,
            c.handoff,
            c.src_and_tests,
            c.src_all,
            c.src_non_hash,
            c.explanation
        )
        .unwrap();
    }
    out.push_str("\n## File-level records\n\nEach file's notes retain the detected line numbers and original SHA-256. Linked source files may change through other tasks; verify the recorded content against the dedicated snapshot matching the hashes in source_manifest.\n");
    let mut group = (String::new(), String::new());
    let escape = |s: &str| s.replace('|', "\\|").replace('\n', " ");
    for e in entries {
        if group != (e.repo.clone(), e.crate_name.clone()) {
            group = (e.repo.clone(), e.crate_name.clone());
            writeln!(out, "\n### {}/{}\n\nowner: `{}`\n\n| file | primitive / suite | purpose | quantum_status | replacement candidate / phase | notes |\n| --- | --- | --- | --- | --- | --- |", e.repo, e.crate_name, e.owner).unwrap();
        }
        let status = serde_json::to_value(&e.quantum_status).expect("enum JSON");
        writeln!(
            out,
            "| [{}](../../{}/{}) | {} / {} | {} | {} | {} / {:?} | {} |",
            escape(&e.file),
            e.repo,
            e.file,
            e.primitive,
            escape(&e.current_suite),
            escape(&e.purpose),
            status.as_str().expect("enum string"),
            escape(&e.replacement_candidate),
            e.migration_phase,
            escape(&e.notes)
        )
        .unwrap();
    }
    out
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}

fn generate(root: &Path, output: &Path) -> Result<()> {
    let (entries, sources) = scan(root)?;
    if entries.is_empty() {
        return Err("empty inventory".into());
    }
    let counts = comparisons(&entries);
    fs::create_dir_all(output)?;
    write_json(&output.join("crypto_inventory.json"), &entries)?;
    write_json(&output.join("source_manifest.json"), &sources)?;
    write_json(&output.join("count_reconciliation.json"), &counts)?;
    fs::write(
        output.join("CRYPTO_INVENTORY.md"),
        markdown(&entries, &counts, &sources),
    )?;
    println!("generated {} primitive entries", entries.len());
    Ok(())
}

fn check(root: &Path, output: &Path) -> Result<()> {
    let entries: Vec<Entry> =
        serde_json::from_slice(&fs::read(output.join("crypto_inventory.json"))?)?;
    let sources: Sources = serde_json::from_slice(&fs::read(output.join("source_manifest.json"))?)?;
    let counts: Vec<CountComparison> =
        serde_json::from_slice(&fs::read(output.join("count_reconciliation.json"))?)?;
    if sources.grep_pattern != GREP_PATTERN {
        return Err("stale primitive detection contract".into());
    }
    let indexed: BTreeSet<_> = entries
        .iter()
        .map(|e| (e.repo.clone(), e.file.clone()))
        .collect();
    let identities: BTreeSet<_> = entries
        .iter()
        .map(|e| (&e.repo, &e.file, &e.primitive))
        .collect();
    if identities.len() != entries.len() {
        return Err("duplicate inventory entry".into());
    }
    let mut observed = BTreeSet::new();
    let mut all_inputs = BTreeSet::new();
    for repo in REPOS {
        let repo_root = root.join(repo);
        let paths = files(&repo_root)?;
        for path in &paths {
            all_inputs.insert((
                repo.to_owned(),
                path.strip_prefix(&repo_root)?
                    .to_str()
                    .ok_or("non-UTF8")?
                    .to_owned(),
            ));
        }
        let rust: Vec<_> = paths
            .iter()
            .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
            .map(|p| p.strip_prefix(&repo_root).expect("child path"))
            .collect();
        if rust.is_empty() {
            return Err(format!("no Rust sources for {repo}").into());
        }
        // Independent GNU grep detector: the JSON generator uses Rust regex, not this command.
        let result = Command::new("grep")
            .args(["-El", GREP_PATTERN, "--"])
            .args(rust)
            .current_dir(&repo_root)
            .output()?;
        if !matches!(result.status.code(), Some(0 | 1)) {
            return Err(format!(
                "grep failed for {repo}: {}",
                String::from_utf8_lossy(&result.stderr)
            )
            .into());
        }
        for file in std::str::from_utf8(&result.stdout)?.lines() {
            observed.insert((repo.to_owned(), file.to_owned()));
        }
    }
    let missing: Vec<_> = observed.difference(&indexed).collect();
    let stale: Vec<_> = indexed.difference(&observed).collect();
    println!(
        "grep_files={} inventory_files={} primitive_entries={} missing={} stale={}",
        observed.len(),
        indexed.len(),
        entries.len(),
        missing.len(),
        stale.len()
    );
    if !missing.is_empty() || !stale.is_empty() {
        return Err(format!("inventory mismatch: missing={missing:?}; stale={stale:?}").into());
    }
    let recorded_inputs: BTreeSet<_> = sources
        .files
        .iter()
        .map(|f| (f.repo.clone(), f.file.clone()))
        .collect();
    if recorded_inputs != all_inputs || sources.files.len() != recorded_inputs.len() {
        return Err("source file manifest mismatch".into());
    }
    for input in &sources.files {
        if hash(&fs::read(root.join(&input.repo).join(&input.file))?) != input.sha256 {
            return Err(format!("source hash mismatch: {}/{}", input.repo, input.file).into());
        }
    }
    let (regenerated, _) = scan(root)?;
    if entries != regenerated {
        return Err(
            "per-primitive metadata differs from captured source and classification rules".into(),
        );
    }
    if counts != comparisons(&entries) {
        return Err("count reconciliation differs from inventory JSON".into());
    }
    if fs::read_to_string(output.join("CRYPTO_INVENTORY.md"))?
        != markdown(&entries, &counts, &sources)
    {
        return Err("generated Markdown differs from canonical JSON".into());
    }
    println!(
        "source_hashes={} markdown_from_json=PASS inventory_check=PASS",
        sources.files.len()
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: crypto-inventory (generate|check) SOURCE_SNAPSHOT OUTPUT_DIRECTORY".into(),
        );
    }
    match args[1].as_str() {
        "generate" => generate(Path::new(&args[2]), Path::new(&args[3])),
        "check" => check(Path::new(&args[2]), Path::new(&args[3])),
        _ => Err("unknown inventory command".into()),
    }
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut output = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            if !matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some("target" | ".git" | ".cache" | "vendor")
            ) {
                output.extend(files(&path)?);
            }
        } else if path.extension().is_some_and(|e| e == "rs")
            || path.file_name().is_some_and(|f| f == "Cargo.toml")
        {
            output.push(path);
        }
    }
    output.sort();
    Ok(output)
}

fn crate_name(file: &Path, repo_root: &Path) -> Result<String> {
    let mut directory = file.parent().ok_or("missing parent")?;
    loop {
        let manifest = directory.join("Cargo.toml");
        if manifest.is_file() {
            let parsed: toml::Value = toml::from_str(&fs::read_to_string(manifest)?)?;
            if let Some(name) = parsed
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(toml::Value::as_str)
            {
                return Ok(name.to_owned());
            }
        }
        if directory == repo_root {
            return Ok("standalone-rust".into());
        }
        directory = directory.parent().ok_or("crate outside repository")?;
    }
}

fn context(repo: &str, name: &str, file: &str) -> (String, Phase) {
    let (domain, phase) = if repo == "dekyx" || file.contains("kyb") || file.contains("kyx") {
        (
            "DeKYX participant qualifications, issuers, and purpose-scoped authorization",
            Phase::P3,
        )
    } else if repo == "deccp" || file.contains("cross_domain") || file.contains("deccp") {
        (
            "Guarantee limits, collateral, and cross-DeFMI settlement authorization",
            Phase::P4,
        )
    } else {
        match name {
            "qomm-transport" | "zkpi-committee" => (
                "Messages between corporate and MPC nodes, and key lifecycles",
                Phase::P2,
            ),
            "zkpi" | "zkpi-defmi-sdk" => (
                "zkPI, settlement instructions, and reservation evidence",
                Phase::P2,
            ),
            "defmi" | "qomm-avalanche-vm" | "defmi-avalanche-vm" => (
                "DeFMI ledger, settlement execution, and finality",
                Phase::P2,
            ),
            "qomm-proofs" | "zkpi-proofs" | "zkfmi-zk" => (
                "Secret values, commitments, and public verification",
                Phase::P5,
            ),
            "qomm-audit" => ("Node audit signatures and public audit records", Phase::P2),
            "qomm-mpc" => ("Secret-shared inputs and MPC execution", Phase::P5),
            "qomm-harness" | "zkpi-harness" | "defmi-harness" | "qomm-demo" => (
                "Integrated execution, verification, and demo boundaries",
                Phase::P2,
            ),
            "oclob-node" => (
                "OCLOB node authentication, order processing, and consensus",
                Phase::P2,
            ),
            "oclob-settlement" => ("Settlement evidence between OCLOB and DeFMI", Phase::P2),
            _ if name.starts_with("oclob-") => {
                ("OCLOB order, participant, and MPC boundaries", Phase::P2)
            }
            _ if repo == "aethel" => ("Aethel receivables and participant signatures", Phase::P2),
            _ => (
                "Protocol identification and verification support",
                Phase::P6,
            ),
        }
    };
    let kind = if file.contains("/tests/") || file.ends_with("_tests.rs") {
        "tests"
    } else if file.contains("/benches/") {
        "benchmark code"
    } else if file.contains("/examples/") {
        "examples"
    } else {
        "implementation"
    };
    (format!("{domain} / {kind}"), phase)
}

fn has(text: &str, token: &str) -> bool {
    Regex::new(&format!(r"\b{}\b", regex::escape(token)))
        .expect("literal regex")
        .is_match(text)
}

fn classify(
    repo: &str,
    name: &str,
    file: &str,
    primitive: &str,
    text: &str,
    lines: &[usize],
) -> Vec<Entry> {
    let (domain, signature_phase) = context(repo, name, file);
    let source_hash = hash(text.as_bytes());
    let make = |primitive: &str,
                suite: &str,
                role: &str,
                status: QuantumStatus,
                candidate: &str,
                phase: Phase,
                note: &str| Entry {
        repo: repo.into(),
        crate_name: name.into(),
        file: file.into(),
        primitive: primitive.into(),
        current_suite: suite.into(),
        purpose: format!("{domain}: {role}"),
        owner: format!("{repo}/{name}"),
        quantum_status: status,
        replacement_candidate: candidate.into(),
        migration_phase: phase,
        notes: format!("{note}; lexical reference lines={lines:?}; source_sha256={source_hash}"),
    };
    use QuantumStatus::*;
    match primitive {
        "ed25519_dalek" => vec![make(
            primitive,
            "Ed25519",
            "Signature generation, verification, and public-key handling",
            BrokenByQuantum,
            "Ed25519 + ML-DSA-65 (AND verification)",
            signature_phase,
            "Shor's algorithm breaks signature authenticity. Preserve purposes, key IDs, and validity intervals during migration",
        )],
        "frost_core" | "frost_ristretto255" => vec![make(
            primitive,
            "FrostRistretto255",
            "Threshold signatures and DKG",
            BrokenByQuantum,
            "Existing FROST + k independent ML-DSA-65 signatures",
            Phase::P2,
            "This is not an implemented standardized PQ threshold signature",
        )],
        "curve25519_dalek" => {
            if file.ends_with("/shamir.rs") && !has(text, "RistrettoPoint") {
                vec![make(primitive, "Scalar / Shamir finite-field sharing", "Finite-field sharing, interpolation, and error location", InformationTheoretic, "Retain secret sharing; replace authentication, transport, and binding separately", Phase::P5, "Secrecy assumes suitable uniform randomness and observation below the threshold. This does not claim quantum resistance for surrounding Pedersen binding or transport authentication")]
            } else {
                vec![make(primitive, "PedersenRistretto255 / Scalar / Ristretto255", "Group operations, commitments, and discrete-log-based proofs", BrokenByQuantum, "P5 hash-based public verification and PQ commitment candidates", Phase::P5, "The Scalar type alone is not classified as broken; classification concerns group-based binding and proof soundness at its use sites. Pedersen's perfect hiding is a separate guarantee")]
            }
        }
        "bulletproofs" => vec![make(
            primitive,
            "BulletproofsRistretto255",
            "Range proofs and group-based public verification",
            BrokenByQuantum,
            "Period-based hash-proof candidates",
            Phase::P5,
            "Migrate proof soundness. Evaluate the confidentiality and soundness of existing proofs separately",
        )],
        "merlin" => vec![make(
            primitive,
            "Merlin / STROBE transcript",
            "Fiat-Shamir transcripts and domain separation",
            GroverOnly,
            "Preserve domain separation while migrating to P5 proof transcripts",
            Phase::P5,
            "This classifies the hash-based transcript itself. Discrete-log proofs using it are separately broken_by_quantum",
        )],
        "sha2" | "sha3" => {
            let candidates = if primitive == "sha2" {
                vec!["Sha256", "Sha384", "Sha512"]
            } else {
                vec!["Sha3_256", "Sha3_512", "Shake128", "Shake256", "Keccak256"]
            };
            let algorithms: Vec<_> = candidates
                .into_iter()
                .filter(|name| has(text, name))
                .collect();
            let suite = if algorithms.is_empty() {
                format!("{primitive} (see upstream call site)")
            } else {
                algorithms.join(" + ")
            };
            vec![make(primitive, &suite, "Body, state, and identifier hashes or XOFs", GroverOnly, "Retain for now; evaluate SHA-384/SHA-512/SHAKE256 according to required security margins", Phase::P6, "Classification accounts for generic quantum search such as Grover's algorithm. Required strength, including collision resistance, remains a P6 decision; unconditional security is not assumed")]
        }
        "openssl" => {
            let mut output = Vec::new();
            if text.contains("openssl::ssl") || has(text, "SslStream") {
                output.push(make(
                    "openssl::tls",
                    "X25519Tls13 / classical TLS 1.3",
                    "Key exchange for node-to-node and RPC connections",
                    BrokenByQuantum,
                    "Pin X25519MLKEM768 (P1)",
                    Phase::P1,
                    "Current TLS groups and downgrade controls change in P1. They are unchanged in P0",
                ));
            }
            if has(text, "X509") || has(text, "X509Req") || text.contains("generate_ed25519") {
                output.push(make(
                    "openssl::certificate",
                    "Ed25519 / X.509",
                    "mTLS authentication, CA, CSR, and certificate lifecycles",
                    BrokenByQuantum,
                    "ML-DSA-only certificates or dual certificates (decision pending)",
                    Phase::P1,
                    "Post-quantum key exchange and authentication are separate migration decisions",
                ));
            }
            if text.contains("Id::X25519") || text.contains("generate_x25519") {
                output.push(make(
                    "openssl::x25519",
                    "X25519",
                    "Key distribution for selective-disclosure, order, and capability envelopes",
                    BrokenByQuantum,
                    "Combine X25519 + ML-KEM-768 key material",
                    Phase::P1,
                    "Envelopes outside TLS are also exposed to store-now-decrypt-later attacks. P0 does not wire this into existing code",
                ));
            }
            if text.contains("PKey::hmac") {
                output.push(make(
                    "openssl::hmac",
                    "HMAC-SHA256",
                    "Admission capability MAC",
                    GroverOnly,
                    "Preserve key lengths and purpose separation; review security margins in P6",
                    Phase::P6,
                    "Do not conflate public-key signatures and MACs. Distribution of the MAC key is a separate trust boundary",
                ));
            }
            if text.contains("openssl::symm") {
                let cipher = if text.contains("Cipher::chacha20_poly1305") {
                    "ChaCha20-Poly1305 / envelope encryption"
                } else if text.contains("Cipher::aes_256_gcm") {
                    "AES-256-GCM / key-store encryption"
                } else {
                    "OpenSSL symmetric encryption (see call site)"
                };
                output.push(make(
                    "openssl::symmetric",
                    cipher,
                    "Symmetric encryption for secret-key storage or envelope bodies",
                    GroverOnly,
                    "Retain; update the KEM and key management separately",
                    Phase::P6,
                    "Treat symmetric encryption and passphrase-KDF strength separately from asymmetric key exchange",
                ));
            }
            if text.contains("openssl::sha::sha1") {
                output.push(make(
                    "openssl::sha1",
                    "SHA-1 / WebSocket handshake",
                    "WebSocket Accept header (not a signature use)",
                    GroverOnly,
                    "Retain protocol requirements; do not reuse for authentication or encryption",
                    Phase::P6,
                    "SHA-1 also has known classical collision weaknesses. This enum does not deny those weaknesses",
                ));
            }
            if output.is_empty() {
                output.push(make(
                    primitive,
                    "OpenSSL key / certificate handling",
                    "Cryptographic key or certificate input/output",
                    BrokenByQuantum,
                    "Follow the caller's P1 algorithms and authentication policy",
                    Phase::P1,
                    "Conservative classification retains every lexical reference; the algorithm depends on the caller",
                ));
            }
            output
        }
        _ => unreachable!("fixed primitive list"),
    }
}
