//! Native seven-party process orchestration. Secret input generation and raw
//! persistence loading live only in the dedicated per-party child workers.
use crate::{
    field::F,
    integration::{
        circuit::PARTIES,
        input_delivery::PinnedReceiverLaunch,
        mpc_program,
        owner_input::OwnerInputBinding,
        party::Request,
        query_agreement::{OwnerApproval, TrustedRoster},
    },
    pcs::{NodeRoots, PairOpening},
    transcript::Hash,
};
use qomm_mpc::{compiler::OfficialCompiler, engine_policy::EnginePin};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    str::FromStr,
    time::Instant,
};

pub struct Worker {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Worker {
    fn send(&mut self, request: &impl Serialize) -> Result<(), String> {
        serde_json::to_writer(&mut self.input, request).map_err(|_| "worker request write")?;
        writeln!(self.input).map_err(|_| "worker request newline")?;
        self.input.flush().map_err(|_| "worker flush".to_string())
    }
    fn receive<T: DeserializeOwned>(&mut self) -> Result<T, String> {
        let mut answer = String::new();
        if self
            .output
            .read_line(&mut answer)
            .map_err(|_| "worker response read")?
            == 0
        {
            return Err("private worker exited; see owner-local error log".into());
        }
        serde_json::from_str(&answer).map_err(|_| "worker public response shape".to_string())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.send(&Request::Shutdown);
        let _ = self.child.wait();
    }
}

#[derive(Serialize, Clone)]
pub struct RunRecord {
    pub program: String,
    pub source_sha256: String,
    pub compiled_artifacts: Vec<(String, String)>,
    pub party_exit_codes: Vec<Option<i32>>,
    pub seconds: f64,
    pub released_field_count: usize,
    pub seven_context_views_equal: bool,
    pub rejection_reason: Option<String>,
}

pub struct Native {
    root: PathBuf,
    compiler: OfficialCompiler,
    pin: EnginePin,
    workers: Vec<Worker>,
    counter: usize,
    pub records: Vec<RunRecord>,
    port: u16,
    query_roster_sha512: Option<String>,
    owner_input_binding: Option<OwnerInputBinding>,
    all_public_outputs: bool,
}

struct QueryAgreementLaunch {
    roster_path: PathBuf,
    roster_sha512: String,
    owner_key_paths: Vec<PathBuf>,
}

struct OwnerInputLaunch {
    binding: OwnerInputBinding,
    binding_path: PathBuf,
    input_paths: Vec<PathBuf>,
    receivers: Option<Vec<PinnedReceiverLaunch>>,
}
fn public_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn private_log(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "unique native log file".to_string())
}

impl Native {
    /// Separate note-worker protocol; legacy fixed-vector launches are
    /// unchanged. The public configuration and its independent pin are
    /// checked again inside each private owner worker before input release.
    #[cfg(feature = "pqc-note-circuit")]
    pub(crate) fn new_note_workers(
        root: &Path,
        port: u16,
        config: &Path,
        config_sha512: &str,
    ) -> Result<Self, String> {
        let root = fs::canonicalize(root).map_err(|_| "native note root missing")?;
        let pin = EnginePin::verify(&root)?;
        let compiler = OfficialCompiler::from_checkout(&root).map_err(|e| e.to_string())?;
        fs::create_dir_all(root.join("Persistence")).map_err(|_| "note persistence directory")?;
        fs::create_dir(root.join("cosnark-private-logs"))
            .map_err(|_| "fresh note native run required")?;
        let mut workers = Vec::new();
        for party in 0..PARTIES {
            if root
                .join("Persistence")
                .join(format!("Transactions-P{party}.data"))
                .exists()
            {
                return Err("note proof sessions require fresh private persistence".into());
            }
            let log = private_log(
                &root
                    .join("cosnark-private-logs")
                    .join(format!("worker-{party}.log")),
            )?;
            let mut child = Command::new(std::env::current_exe().map_err(|_| "note executable")?)
                .arg("note-worker")
                .arg(&root)
                .arg(party.to_string())
                .arg(config)
                .arg(config_sha512)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::from(log))
                .spawn()
                .map_err(|_| "launch private note worker")?;
            workers.push(Worker {
                input: child.stdin.take().unwrap(),
                output: BufReader::new(child.stdout.take().unwrap()),
                child,
            });
        }
        Ok(Self {
            root,
            compiler,
            pin,
            workers,
            counter: 0,
            records: vec![],
            port,
            query_roster_sha512: None,
            owner_input_binding: None,
            all_public_outputs: true,
        })
    }

    #[cfg(feature = "pqc-note-circuit")]
    pub(crate) fn note_broadcast<T: DeserializeOwned>(
        &mut self,
        request: &impl Serialize,
    ) -> Result<Vec<T>, String> {
        for worker in &mut self.workers {
            worker.send(request)?;
        }
        self.workers.iter_mut().map(Worker::receive).collect()
    }

    pub fn new(root: &Path, port: u16, resumed: bool) -> Result<Self, String> {
        Self::new_inner(root, port, resumed, None, None)
    }

    /// Same-host research adapter. The caller is the trusted launch boundary
    /// and must source the roster path and digest from deployment configuration,
    /// not from the coordinator. Key bytes are loaded only by owner workers.
    /// This launcher does not provide independent operator custody.
    pub fn new_query_agreement(
        root: &Path,
        port: u16,
        resumed: bool,
        roster_path: &Path,
        expected_roster_sha512: &str,
        owner_key_paths: &[PathBuf],
    ) -> Result<Self, String> {
        TrustedRoster::read_pinned(roster_path, expected_roster_sha512)?;
        if owner_key_paths.len() != PARTIES {
            return Err("query agreement launch requires seven owner key paths".into());
        }
        Self::new_inner(
            root,
            port,
            resumed,
            Some(QueryAgreementLaunch {
                roster_path: fs::canonicalize(roster_path)
                    .map_err(|_| "trusted query roster path")?,
                roster_sha512: expected_roster_sha512.into(),
                owner_key_paths: owner_key_paths.to_vec(),
            }),
            None,
        )
    }

    /// Actual input adapter: the coordinator receives paths/public context,
    /// never the contents of any party's contribution file. Missing inputs fail
    /// in that worker; the fixture generator is not a fallback for this launch.
    #[allow(clippy::too_many_arguments)]
    pub fn new_query_agreement_with_inputs(
        root: &Path,
        port: u16,
        resumed: bool,
        roster_path: &Path,
        expected_roster_sha512: &str,
        owner_key_paths: &[PathBuf],
        binding_path: &Path,
        input_paths: &[PathBuf],
    ) -> Result<Self, String> {
        TrustedRoster::read_pinned(roster_path, expected_roster_sha512)?;
        if owner_key_paths.len() != PARTIES || input_paths.len() != PARTIES {
            return Err("owner input launch requires seven key and input paths".into());
        }
        let binding: OwnerInputBinding = serde_json::from_slice(
            &fs::read(binding_path).map_err(|_| "public owner input binding missing")?,
        )
        .map_err(|_| "public owner input binding encoding")?;
        binding.validate()?;
        Self::new_inner(
            root,
            port,
            resumed,
            Some(QueryAgreementLaunch {
                roster_path: fs::canonicalize(roster_path).map_err(|_| "trusted roster path")?,
                roster_sha512: expected_roster_sha512.into(),
                owner_key_paths: owner_key_paths.to_vec(),
            }),
            Some(OwnerInputLaunch {
                binding,
                binding_path: fs::canonicalize(binding_path)
                    .map_err(|_| "public owner binding path")?,
                input_paths: input_paths.to_vec(),
                receivers: None,
            }),
        )
    }

    /// Ciphertext input files and pinned worker registration snapshots. Private
    /// KEM seeds are loaded only by the recipient worker's laboratory adapter.
    #[allow(clippy::too_many_arguments)]
    pub fn new_query_agreement_with_sealed_inputs(
        root: &Path,
        port: u16,
        resumed: bool,
        roster_path: &Path,
        expected_roster_sha512: &str,
        owner_key_paths: &[PathBuf],
        binding_path: &Path,
        input_paths: &[PathBuf],
        receivers: &[PinnedReceiverLaunch],
    ) -> Result<Self, String> {
        TrustedRoster::read_pinned(roster_path, expected_roster_sha512)?;
        if owner_key_paths.len() != PARTIES
            || input_paths.len() != PARTIES
            || receivers.len() != PARTIES
        {
            return Err(
                "sealed owner input launch requires seven owners, packets and receivers".into(),
            );
        }
        let binding: OwnerInputBinding = serde_json::from_slice(
            &fs::read(binding_path).map_err(|_| "public sealed input binding missing")?,
        )
        .map_err(|_| "public sealed input binding encoding")?;
        binding.validate()?;
        Self::new_inner(
            root,
            port,
            resumed,
            Some(QueryAgreementLaunch {
                roster_path: fs::canonicalize(roster_path).map_err(|_| "trusted roster path")?,
                roster_sha512: expected_roster_sha512.into(),
                owner_key_paths: owner_key_paths.to_vec(),
            }),
            Some(OwnerInputLaunch {
                binding,
                binding_path: fs::canonicalize(binding_path)
                    .map_err(|_| "public sealed binding path")?,
                input_paths: input_paths.to_vec(),
                receivers: Some(receivers.to_vec()),
            }),
        )
    }

    fn new_inner(
        root: &Path,
        port: u16,
        resumed: bool,
        query_agreement: Option<QueryAgreementLaunch>,
        owner_inputs: Option<OwnerInputLaunch>,
    ) -> Result<Self, String> {
        let root = fs::canonicalize(root).map_err(|_| "native root missing")?;
        let pin = EnginePin::verify(&root)?;
        let compiler = OfficialCompiler::from_checkout(&root).map_err(|e| e.to_string())?;
        fs::create_dir_all(root.join("Persistence")).map_err(|_| "persistence directory")?;
        fs::create_dir(root.join("cosnark-private-logs"))
            .map_err(|_| "fresh native run required")?;
        for party in 0..PARTIES {
            if root
                .join("Persistence")
                .join(format!("Transactions-P{party}.data"))
                .exists()
                != resumed
            {
                return Err("private persistence does not match declared recovery mode".into());
            }
        }
        let mut workers = Vec::new();
        for party in 0..PARTIES {
            let log = private_log(
                &root
                    .join("cosnark-private-logs")
                    .join(format!("worker-{party}.log")),
            )?;
            let mut command =
                Command::new(std::env::current_exe().map_err(|_| "current trial executable")?);
            let party_string = party.to_string();
            if let Some(launch) = &query_agreement {
                command.args([
                    if owner_inputs
                        .as_ref()
                        .is_some_and(|inputs| inputs.receivers.is_some())
                    {
                        "integration-worker-v2-sealed"
                    } else if owner_inputs.is_some() {
                        "integration-worker-v2-owned"
                    } else {
                        "integration-worker-v2"
                    },
                    root.to_str().ok_or("native path utf8")?,
                    &party_string,
                    launch
                        .roster_path
                        .to_str()
                        .ok_or("trusted query roster path utf8")?,
                    &launch.roster_sha512,
                    launch.owner_key_paths[party]
                        .to_str()
                        .ok_or("owner query key path utf8")?,
                ]);
                if let Some(inputs) = &owner_inputs {
                    command
                        .arg(&inputs.binding_path)
                        .arg(&inputs.input_paths[party]);
                    if let Some(receivers) = &inputs.receivers {
                        command
                            .arg(&receivers[party].config_path)
                            .arg(&receivers[party].config_sha256)
                            .arg(&receivers[party].recipient_key_path);
                    }
                }
            } else {
                command.args([
                    "integration-worker",
                    root.to_str().ok_or("native path utf8")?,
                    &party_string,
                ]);
            }
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::from(log))
                .spawn()
                .map_err(|_| "launch owner worker")?;
            workers.push(Worker {
                input: child.stdin.take().unwrap(),
                output: BufReader::new(child.stdout.take().unwrap()),
                child,
            });
        }
        Ok(Self {
            root,
            compiler,
            pin,
            workers,
            counter: 0,
            records: Vec::new(),
            port,
            query_roster_sha512: query_agreement.map(|launch| launch.roster_sha512),
            owner_input_binding: owner_inputs.map(|inputs| inputs.binding),
            all_public_outputs: false,
        })
    }

    pub fn initialization_source(
        &self,
        statement: &super::circuit::Statement,
        resumed: bool,
    ) -> Result<String, String> {
        match &self.owner_input_binding {
            Some(binding) => {
                binding.require_statement(statement)?;
                Ok(mpc_program::initialize_owned(
                    statement.settlement.no_fill,
                    resumed,
                ))
            }
            None => Ok(mpc_program::initialize(
                statement.settlement.no_fill,
                resumed,
            )),
        }
    }

    pub fn query_roster_sha512(&self) -> Option<&str> {
        self.query_roster_sha512.as_deref()
    }
    pub fn broadcast<T: DeserializeOwned>(&mut self, request: &Request) -> Result<Vec<T>, String> {
        for worker in &mut self.workers {
            worker.send(request)?;
        }
        self.workers.iter_mut().map(Worker::receive).collect()
    }
    pub fn roots(&mut self) -> Result<Vec<NodeRoots>, String> {
        self.broadcast(&Request::Commit)
    }

    pub fn query_with_agreement(
        &mut self,
        kind: usize,
        indices: Vec<usize>,
        final_rows: Vec<String>,
    ) -> Result<Vec<Vec<PairOpening>>, String> {
        if self.query_roster_sha512.is_none() {
            return Err("query agreement requires a pinned v2 native launch".into());
        }
        let approvals: Vec<OwnerApproval> = self.broadcast(&Request::PrepareQuery {
            kind,
            indices,
            final_rows,
        })?;
        self.broadcast(&Request::QueryWithAgreement { kind, approvals })
    }
    pub fn reject_replayed_queries(
        &mut self,
        final_rows: Vec<String>,
    ) -> Result<Vec<bool>, String> {
        let (request, expected_log) = if self.query_roster_sha512.is_some() {
            (
                Request::PrepareQuery {
                    kind: 0,
                    indices: vec![0; crate::pcs::QUERIES],
                    final_rows,
                },
                "worker query budget was consumed, prepared, or request is out of order",
            )
        } else {
            (
                Request::Query {
                    kind: 0,
                    indices: vec![0; crate::pcs::QUERIES],
                    final_rows,
                },
                "worker query budget was consumed or request is out of order",
            )
        };
        for worker in &mut self.workers {
            worker.send(&request)?;
        }
        let mut results = Vec::new();
        for (party, worker) in self.workers.iter_mut().enumerate() {
            let rejected = worker.receive::<serde_json::Value>().is_err();
            let status = worker
                .child
                .wait()
                .map_err(|_| "wait one-shot guard result")?;
            let log = fs::read_to_string(
                self.root
                    .join("cosnark-private-logs")
                    .join(format!("worker-{party}.log")),
            )
            .map_err(|_| "owner one-shot guard evidence")?;
            results.push(rejected && status.code() == Some(1) && log.contains(expected_log));
        }
        Ok(results)
    }
    pub fn run(
        &mut self,
        source: String,
        view: Hash,
        initial: bool,
        invalid: bool,
        corrupt_views: bool,
    ) -> Result<Vec<F>, String> {
        self.pin.recheck()?;
        self.counter += 1;
        let name = format!("cocodeint_{:03}", self.counter);
        let source_path = self
            .root
            .join("Programs/Source")
            .join(format!("{name}.mpc"));
        fs::write(&source_path, source.as_bytes()).map_err(|_| "write public MPC program")?;
        let started = Instant::now();
        // -F is the usable INTEGER bit length, not the prime field size.
        // The 254-bit prime is bound separately with -P at compile and runtime.
        let compile = self
            .compiler
            .command()
            .args(["-M", "-F", "64", "-P", &mpc_program::prime(), &name])
            .output()
            .map_err(|_| "launch official compiler")?;
        let log_dir = self.root.join("cosnark-private-logs");
        let mut compiler_log = private_log(&log_dir.join(format!("{name}-compiler.log")))?;
        compiler_log
            .write_all(&compile.stdout)
            .map_err(|_| "compiler log")?;
        compiler_log
            .write_all(&compile.stderr)
            .map_err(|_| "compiler log")?;
        if !compile.status.success() {
            return Err(format!(
                "official compiler failed for {name}; public-source compiler log: {}",
                log_dir.join(format!("{name}-compiler.log")).display()
            ));
        }
        let mut compiled = Vec::new();
        for folder in ["Programs/Bytecode", "Programs/Schedules"] {
            for item in
                fs::read_dir(self.root.join(folder)).map_err(|_| "compiled artifact directory")?
            {
                let item = item.map_err(|_| "compiled artifact entry")?;
                let file = item.file_name().to_string_lossy().to_string();
                if file.starts_with(&format!("{name}-")) || file == format!("{name}.sch") {
                    compiled.push((
                        format!("{folder}/{file}"),
                        public_hash(&fs::read(item.path()).map_err(|_| "compiled artifact read")?),
                    ));
                }
            }
        }
        compiled.sort();
        if compiled.len() < 2 {
            return Err("missing compiled circuit artifacts".into());
        }
        let mut view_ids = Vec::new();
        for (party, worker) in self.workers.iter_mut().enumerate() {
            let mut local = view;
            if corrupt_views && party >= 5 {
                local[0] ^= 1;
            }
            let local = hex::encode(local);
            view_ids.push(local.clone());
            worker.send(&Request::Input {
                view: local,
                initial,
                invalid,
            })?;
        }
        for (party, worker) in self.workers.iter_mut().enumerate() {
            let ack: serde_json::Value = worker.receive()?;
            if ack["party"] != party || ack["view"] != view_ids[party] {
                return Err("owner context acknowledgement".into());
            }
        }
        // An inconsistent view is deliberately allowed to reach the actual
        // native all-party check for the preregistered two-party tamper case.
        let mut children = Vec::new();
        for party in 0..PARTIES {
            let out = private_log(&log_dir.join(format!("{name}-P{party}.out")))?;
            let err = private_log(&log_dir.join(format!("{name}-P{party}.err")))?;
            let child = Command::new(self.root.join("malicious-shamir-party.x"))
                .current_dir(&self.root)
                .env(
                    "LD_LIBRARY_PATH",
                    format!("{}:/opt/pqc-openssl/lib64", self.root.display()),
                )
                .args([
                    "-N",
                    "7",
                    "-T",
                    "2",
                    "-S",
                    "128",
                    "-P",
                    &mpc_program::prime(),
                    "-lgp",
                    "254",
                    "-pn",
                    &self.port.to_string(),
                    "-IF",
                    "Player-Data/CosnarkInput",
                    &party.to_string(),
                    &name,
                ])
                .args(if self.all_public_outputs {
                    vec!["-OF", "."]
                } else {
                    vec![]
                })
                .stdout(Stdio::from(out))
                .stderr(Stdio::from(err))
                .spawn()
                .map_err(|_| "launch native MPC party")?;
            children.push(child);
        }
        let statuses = children
            .iter_mut()
            .map(|child| child.wait().map_err(|_| "wait native MPC party"))
            .collect::<Result<Vec<_>, _>>()?;
        let output = fs::read_to_string(log_dir.join(format!("{name}-P0.out")))
            .map_err(|_| "read public MPC output")?;
        let mut released = Vec::new();
        for line in output.lines() {
            if let Some(value) = line.strip_prefix("TRIAL_PUBLIC ") {
                let value = value.trim();
                let field = if let Some(abs) = value.strip_prefix('-') {
                    -F::from_str(abs).map_err(|_| "native public field encoding")?
                } else {
                    F::from_str(value).map_err(|_| "native public field encoding")?
                };
                released.push(field);
            }
        }
        let success = statuses.iter().all(|s| s.success());
        let mut rejection_reason = None;
        if !success {
            for party in 0..PARTIES {
                for extension in ["out", "err"] {
                    let text =
                        fs::read_to_string(log_dir.join(format!("{name}-P{party}.{extension}")))
                            .map_err(|_| "native rejection evidence")?;
                    for marker in ["invalid financial witness", "challenge view mismatch"] {
                        if text.contains(marker) {
                            rejection_reason = Some(marker.to_string());
                        }
                    }
                }
            }
        }
        self.records.push(RunRecord {
            program: name.clone(),
            source_sha256: public_hash(source.as_bytes()),
            compiled_artifacts: compiled,
            party_exit_codes: statuses.iter().map(|s| s.code()).collect(),
            seconds: started.elapsed().as_secs_f64(),
            released_field_count: released.len(),
            seven_context_views_equal: !corrupt_views,
            rejection_reason,
        });
        if !success {
            return Err(format!(
                "native seven-party run rejected: {name}; no private log contents released"
            ));
        }
        if initial && !output.lines().any(|l| l == "TRIAL_READY") {
            return Err("native initialization receipt missing".into());
        }
        Ok(released)
    }
}
