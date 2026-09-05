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
        ("qomm-defmi", ("defmi", 25)),
        ("qomm-proofs", ("defmi", 12)),
        ("qomm-harness", ("defmi", 12)),
        ("qomm-zk", ("defmi", 9)),
        ("oclob-node", ("oclob", 9)),
        ("qomm-avalanche-vm", ("defmi", 8)),
        ("qomm-zkpi", ("defmi", 7)),
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
        let explanation = if src_and_tests == handoff { "発注件数とsrc+tests全対象集合が一致" }
        else if src_non_hash == handoff { "発注件数はsrc内の非ハッシュ参照集合と一致。今回の全対象はhash-onlyとtestsも含む" }
        else { "発注件数とは不一致。src+tests、srcのみ、src非ハッシュを併記。発注時のファイル集合が無いため残差の原因は断定しない" }.into();
        CountComparison { crate_name: name.into(), repo: repo.into(), handoff, all_rust, src_and_tests, src_all, src_non_hash, explanation }
    }).collect()
}

fn markdown(entries: &[Entry], counts: &[CountComparison], sources: &Sources) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("# 暗号利用棚卸し\n\nこのファイルは `crypto_inventory.json`、`count_reconciliation.json`、`source_manifest.json` からRustツールで生成した。手編集しない。\n\n");
    let total_files = entries
        .iter()
        .map(|e| (&e.repo, &e.file))
        .collect::<BTreeSet<_>>()
        .len();
    writeln!(out, "固定スナップショット: `{}`。対象7リポジトリ、{}ファイル、{} primitiveエントリ。CargoとRust入力{}ファイルのSHA-256をsource_manifestに保持する。\n", sources.captured_at, total_files, entries.len(), sources.files.len()).unwrap();
    out.push_str("Rustソースの外部crate識別子を字句検索する。`.gitignore`で除外されない `.rs` が対象で、src/tests/examplesを含む。コメント・再エクスポート上の参照も保持する。`frost_*` は実際の外部crate `frost_core` / `frost_ristretto255` を検出し、RPC名やローカル関数名をcrate利用と混同しない。ownerはリポジトリ/crateというコード責任境界を示し、人員を新たに指名するものではない。\n\n量子分類はprimitiveが担う保証の分類。Shamirの条件付き完全秘匿性、Pedersenの完全秘匿性と破られる束縛性、ハッシュの汎用量子探索による余裕低下を分ける。移行順と用途は計画書とcrate/moduleの役割に基づくP0の割当てで、本番の承認ポリシーではない。新クレートへの配線はしていない。\n\n## 集計と原本\n\n| repo | HEAD | 対象ファイル | primitive行 |\n| --- | --- | ---: | ---: |\n");
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
    out.push_str("\n## 発注書件数との照合\n\n共通crateの代表はdefmi側。複製を全リポジトリ合算して発注数と比べない。下表は同じスナップショットから集計した。\n\n| crate (repo) | 発注 | src+tests 全対象 | src全対象 | src非ハッシュ | 説明 |\n| --- | ---: | ---: | ---: | ---: | --- |\n");
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
    out.push_str("\n## ファイル単位の記録\n\n各ファイルのnotesに検出行と原本SHA-256を残す。リンク先の原本は他タスクで変わり得るため、確定した内容はsource_manifestのハッシュに対応する専用スナップショットで確認する。\n");
    let mut group = (String::new(), String::new());
    let escape = |s: &str| s.replace('|', "\\|").replace('\n', " ");
    for e in entries {
        if group != (e.repo.clone(), e.crate_name.clone()) {
            group = (e.repo.clone(), e.crate_name.clone());
            writeln!(out, "\n### {}/{}\n\nowner: `{}`\n\n| file | primitive / suite | 用途 | quantum_status | 交換候補 / phase | notes |\n| --- | --- | --- | --- | --- | --- |", e.repo, e.crate_name, e.owner).unwrap();
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
        ("DeKYX参加者資格・発行者・用途限定認可", Phase::P3)
    } else if repo == "deccp" || file.contains("cross_domain") || file.contains("deccp") {
        ("保証枠・担保・クロスDeFMI決済承認", Phase::P4)
    } else {
        match name {
            "qomm-transport" => ("法人・MPCノード間メッセージと鍵ライフサイクル", Phase::P2),
            "qomm-zkpi" | "zkpi-defmi-sdk" => ("zkPI・決済指図・予約証跡", Phase::P2),
            "qomm-defmi" | "qomm-avalanche-vm" => ("DeFMI台帳・決済実行・確定性", Phase::P2),
            "qomm-proofs" | "qomm-zk" => ("秘密値・コミットメント・公開検証", Phase::P5),
            "qomm-audit" => ("ノード監査署名・公開監査記録", Phase::P2),
            "qomm-mpc" => ("秘密分散入力とMPC実行", Phase::P5),
            "qomm-harness" | "qomm-demo" => ("統合実行・検証・デモ境界", Phase::P2),
            "oclob-node" => ("OCLOBノード認証・注文処理・合意", Phase::P2),
            "oclob-settlement" => ("OCLOBとDeFMIの決済証跡", Phase::P2),
            _ if name.starts_with("oclob-") => ("OCLOB注文・参加者・MPC境界", Phase::P2),
            _ if repo == "aethel" => ("Aethel債権・参加者署名", Phase::P2),
            _ => ("プロトコルの識別・検証補助", Phase::P6),
        }
    };
    let kind = if file.contains("/tests/") || file.ends_with("_tests.rs") {
        "試験"
    } else if file.contains("/benches/") {
        "計測用コード"
    } else if file.contains("/examples/") {
        "例"
    } else {
        "実装"
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
            "署名生成・検証・公開鍵取扱い",
            BrokenByQuantum,
            "Ed25519 + ML-DSA-65 (AND verification)",
            signature_phase,
            "Shor攻撃で署名真正性が破られる。用途・鍵ID・有効期間を維持して移行",
        )],
        "frost_core" | "frost_ristretto255" => vec![make(
            primitive,
            "FrostRistretto255",
            "しきい値署名・DKG",
            BrokenByQuantum,
            "現行FROST + k本の独立ML-DSA-65署名",
            Phase::P2,
            "標準PQしきい値署名を実装済みとは扱わない",
        )],
        "curve25519_dalek" => {
            if file.ends_with("/shamir.rs") && !has(text, "RistrettoPoint") {
                vec![make(primitive, "Scalar / Shamir finite-field sharing", "有限体上の分散・補間・誤り位置特定", InformationTheoretic, "秘密分散を維持; 認証・通信・束縛性を別途交換", Phase::P5, "適切な一様乱数としきい値未満の観測に対する秘匿性。周辺のPedersen束縛性や通信認証まで量子耐性を主張しない")]
            } else {
                vec![make(primitive, "PedersenRistretto255 / Scalar / Ristretto255", "群演算・コミットメント・離散対数に基づく証明", BrokenByQuantum, "P5のハッシュベース公開検証・PQ commitment候補", Phase::P5, "Scalarという型だけで破られるとは扱わず、利用先の群ベース束縛性・証明健全性を分類。Pedersenの完全秘匿性は別の保証")]
            }
        }
        "bulletproofs" => vec![make(
            primitive,
            "BulletproofsRistretto255",
            "range proof・群ベース公開検証",
            BrokenByQuantum,
            "期間単位のハッシュベース証明候補",
            Phase::P5,
            "証明健全性の移行。現行証明の秘匿性と健全性を分けて確認",
        )],
        "merlin" => vec![make(
            primitive,
            "Merlin / STROBE transcript",
            "Fiat-Shamir transcriptとドメイン分離",
            GroverOnly,
            "ドメイン分離を維持してP5証明のtranscriptへ移行",
            Phase::P5,
            "ハッシュ系transcript自体の分類。これを使う離散対数証明は別途broken_by_quantum",
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
            vec![make(primitive, &suite, "本文・状態・識別子ハッシュまたはXOF", GroverOnly, "当面維持; 安全性余裕に応じSHA-384/SHA-512/SHAKE256を評価", Phase::P6, "Grover等の汎用量子探索を考慮する分類。collision安全性を含む必要強度の判断はP6で保持し、無条件に安全とは扱わない")]
        }
        "openssl" => {
            let mut output = Vec::new();
            if text.contains("openssl::ssl") || has(text, "SslStream") {
                output.push(make(
                    "openssl::tls",
                    "X25519Tls13 / classical TLS 1.3",
                    "ノード間・RPC接続の鍵交換",
                    BrokenByQuantum,
                    "X25519MLKEM768固定 (P1)",
                    Phase::P1,
                    "現在のTLS groupsとdowngrade制御はP1で変更。P0では未変更",
                ));
            }
            if has(text, "X509") || has(text, "X509Req") || text.contains("generate_ed25519") {
                output.push(make(
                    "openssl::certificate",
                    "Ed25519 / X.509",
                    "mTLS認証・CA・CSR・証明書ライフサイクル",
                    BrokenByQuantum,
                    "ML-DSA証明書単独または二重証明書 (判断待ち)",
                    Phase::P1,
                    "鍵交換のPQ化と認証方式は別の移行判断",
                ));
            }
            if text.contains("Id::X25519") || text.contains("generate_x25519") {
                output.push(make(
                    "openssl::x25519",
                    "X25519",
                    "選択開示・注文・capability envelopeの鍵配送",
                    BrokenByQuantum,
                    "X25519 + ML-KEM-768 の鍵材料結合",
                    Phase::P1,
                    "TLS外のenvelopeも保存後復号の対象。P0では既存コードへ配線しない",
                ));
            }
            if text.contains("PKey::hmac") {
                output.push(make(
                    "openssl::hmac",
                    "HMAC-SHA256",
                    "admission capabilityのMAC",
                    GroverOnly,
                    "鍵長と用途分離を維持しP6で安全性余裕を確認",
                    Phase::P6,
                    "公開鍵署名とMACを混同しない。MAC鍵の配送は別の信頼境界",
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
                    "秘密鍵保管またはenvelope本文の対称暗号",
                    GroverOnly,
                    "維持; KEMと鍵管理を別途更新",
                    Phase::P6,
                    "対称暗号・passphrase KDFの強度と非対称鍵交換を別に扱う",
                ));
            }
            if text.contains("openssl::sha::sha1") {
                output.push(make(
                    "openssl::sha1",
                    "SHA-1 / WebSocket handshake",
                    "WebSocket Acceptヘッダ (署名用途ではない)",
                    GroverOnly,
                    "プロトコル要件を維持; 認証・暗号に転用しない",
                    Phase::P6,
                    "SHA-1には既知の古典的collision問題もある。このenumはその問題を否定しない",
                ));
            }
            if output.is_empty() {
                output.push(make(
                    primitive,
                    "OpenSSL key / certificate handling",
                    "暗号鍵または証明書の入出力",
                    BrokenByQuantum,
                    "呼出元のP1方式・認証ポリシーへ追従",
                    Phase::P1,
                    "lexical referenceを漏れなく残す保守的分類; アルゴリズムは呼出元に依存",
                ));
            }
            output
        }
        _ => unreachable!("fixed primitive list"),
    }
}
