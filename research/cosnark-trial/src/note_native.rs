//! Native note prover and private owner workers. Public program generation and
//! challenge progression are repeated by every owner; an untrusted coordinator
//! cannot request an arbitrary linear functional or an alternate query view.
use crate::{
    algebra::{fold_table, pack, small_polynomial, unpack},
    field::{decode, domain, F},
    integration::{
        native::{Native, RunRecord},
        query_agreement::{self, DurableQueryBudget, OwnerApproval, OwnerIdentity, TrustedRoster},
    },
    note_circuit::Circuit,
    note_mpc,
    note_piop::{self, OpeningContext, Prefix, Relation, Statement},
    recursive_pcs::{self, Proof as OpeningProof},
    transcript::{BatchOpening, Merkle, Transcript},
};
use ark_ff::{BigInteger, FftField, Field, PrimeField, UniformRand};
use ark_poly::EvaluationDomain;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    str::FromStr,
};
use zeroize::Zeroizing;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    pub statement: Statement,
    pub circuit: Circuit,
    pub roster_path: PathBuf,
    pub owner_key_paths: Vec<PathBuf>,
    pub private_input_paths: Vec<PathBuf>,
    pub private_input_sha512: Vec<String>,
}

/// Each file contains ONE owner's share vector, never the seven vectors or a
/// plaintext witness. The trusted launcher pins its digest independently.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateInput {
    pub party: usize,
    pub application_statement_sha512: String,
    pub circuit_sha512: String,
    pub session_sha512: String,
    pub shares: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
enum Request {
    Status,
    Prepare,
    Input {
        view: String,
        initial: bool,
        invalid: bool,
    },
    Finish,
    Commit,
    BindRoots {
        roots: Vec<Vec<String>>,
    },
    FoldRoots {
        roots: Vec<String>,
    },
    PrepareQuery {
        rows: Vec<String>,
    },
    Queries {
        approvals: Vec<OwnerApproval>,
    },
    Complete {
        proof: OpeningProof,
    },
    Shutdown,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
enum Phase {
    Initialize,
    AwaitRoots,
    InitialSums,
    ConstraintRound,
    Endpoints,
    PcsMask,
    PcsSumcheck,
    AwaitFoldRoots,
    AwaitTerminal,
    AwaitApprovals,
    AwaitCompleted,
    Done,
}

#[derive(Serialize, Deserialize)]
struct Plan {
    source: Option<String>,
    sha256: String,
    view: String,
    initial: bool,
}

#[derive(Serialize, Deserialize)]
struct Status {
    phase: Phase,
    next_root: Option<String>,
    terminal_row: Option<String>,
    prefix: Option<Prefix>,
    opening: Option<OpeningProof>,
    prior_mask: Option<OpeningProof>,
    transcript_sha512: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct OwnerQueries {
    original: BatchOpening,
    mask: BatchOpening,
    folded: Vec<BatchOpening>,
}

struct LocalOracle {
    coefficients: Vec<F>,
    tree: Merkle,
}

struct Session {
    context: OpeningContext,
    t: Transcript,
    weights: Vec<F>,
    claim: F,
    aggregation: F,
    folds: Vec<F>,
    current: Vec<F>,
    layers: Vec<Merkle>,
    proof: OpeningProof,
    own_terminal: Option<String>,
    seeds: Vec<usize>,
    query_digest: Option<String>,
    budget: Option<DurableQueryBudget>,
}

struct Owner {
    root: PathBuf,
    party: usize,
    launch: Launch,
    roster: TrustedRoster,
    identity: OwnerIdentity,
    phase: Phase,
    prepared: Option<Plan>,
    running: bool,
    counter: usize,
    oracles: Vec<LocalOracle>,
    prefix: Option<Prefix>,
    t: Option<Transcript>,
    tau: Vec<F>,
    point: Vec<F>,
    batch: F,
    claim: F,
    session: Option<Session>,
    prior_mask: Option<OpeningProof>,
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn read_launch(path: &Path, pin: &str) -> Result<Launch, String> {
    let bytes = fs::read(path).map_err(|_| "note public launch missing")?;
    if hex::encode(Sha512::digest(&bytes)) != pin {
        return Err("note trusted launch pin mismatch".into());
    }
    let launch: Launch = serde_json::from_slice(&bytes).map_err(|_| "note launch encoding")?;
    launch.statement.validate(&launch.circuit)?;
    if launch.owner_key_paths.len() != 7
        || launch.private_input_paths.len() != 7
        || launch.private_input_sha512.len() != 7
    {
        return Err("note launch requires seven owner boundaries".into());
    }
    Ok(launch)
}

fn encode_coset(coefficients: &[F], layer: usize) -> Result<Vec<F>, String> {
    let mut shift = F::GENERATOR;
    for _ in 0..layer {
        shift.square_in_place();
    }
    Ok(domain(coefficients.len() * 4)
        .map_err(str::to_owned)?
        .get_coset(shift)
        .ok_or("note oracle coset")?
        .fft(coefficients))
}

impl Owner {
    fn status(&self) -> Status {
        Status {
            phase: self.phase,
            next_root: self
                .session
                .as_ref()
                .and_then(|s| s.layers.last().map(Merkle::root)),
            terminal_row: self.session.as_ref().and_then(|s| s.own_terminal.clone()),
            prefix: self.prefix.clone(),
            opening: self.session.as_ref().map(|s| s.proof.clone()),
            prior_mask: self.prior_mask.clone(),
            transcript_sha512: self.session.as_ref().map(|s| hex::encode(s.t.view())),
        }
    }
    fn program(&self) -> Result<(String, Transcript), String> {
        let relation = Relation::new(&self.launch.circuit)?;
        let (source, t) = match self.phase {
            Phase::Initialize => (
                note_mpc::initialize(&self.launch.circuit)?,
                Transcript::new(
                    &serde_json::to_vec(&self.launch.statement)
                        .map_err(|_| "note initial context")?,
                ),
            ),
            Phase::InitialSums => (
                note_mpc::initial_sums(&relation, &self.tau)?,
                self.t.clone().ok_or("note constraint cursor")?,
            ),
            Phase::ConstraintRound => (
                note_mpc::constraint_round(&relation, &self.tau, &self.point, self.batch)?,
                self.t.clone().ok_or("note constraint cursor")?,
            ),
            Phase::Endpoints => (
                note_mpc::endpoints(&relation, &self.point)?,
                self.t.clone().ok_or("note constraint cursor")?,
            ),
            Phase::PcsMask => {
                let s = self.session.as_ref().ok_or("note opening session")?;
                (
                    note_mpc::linear_openings(
                        &relation,
                        s.context.kind,
                        std::slice::from_ref(&s.weights),
                        true,
                        F::from(0u64),
                    )?,
                    s.t.clone(),
                )
            }
            Phase::PcsSumcheck => {
                let s = self.session.as_ref().ok_or("note opening session")?;
                (
                    note_mpc::linear_openings(
                        &relation,
                        s.context.kind,
                        &note_mpc::sumcheck_functionals(&s.weights, &s.folds)?,
                        false,
                        s.aggregation,
                    )?,
                    s.t.clone(),
                )
            }
            _ => return Err("native note program out of order".into()),
        };
        Ok((source, t))
    }
    fn prepare(&mut self) -> Result<Plan, String> {
        if self.prepared.is_some() || self.running {
            return Err("native note program already prepared".into());
        }
        let (source, t) = self.program()?;
        let plan = Plan {
            sha256: sha256(source.as_bytes()),
            source: None,
            view: hex::encode(t.view()),
            initial: self.phase == Phase::Initialize,
        };
        self.prepared = Some(Plan {
            source: None,
            sha256: plan.sha256.clone(),
            view: plan.view.clone(),
            initial: plan.initial,
        });
        Ok(Plan {
            source: (self.party == 0).then_some(source),
            ..plan
        })
    }
    fn write_input(
        &mut self,
        view: &str,
        initial: bool,
        invalid: bool,
    ) -> Result<serde_json::Value, String> {
        let plan = self
            .prepared
            .as_ref()
            .ok_or("owner note program not prepared")?;
        if self.running || invalid || plan.view != view || plan.initial != initial {
            return Err("owner note program or view mismatch".into());
        }
        let name = format!("cocodeint_{:03}", self.counter + 1);
        let source = fs::read(
            self.root
                .join("Programs/Source")
                .join(format!("{name}.mpc")),
        )
        .map_err(|_| "owner public native source missing")?;
        if sha256(&source) != plan.sha256 {
            return Err("owner rejected unapproved native program".into());
        }
        let path = self
            .root
            .join("Player-Data")
            .join(format!("CosnarkInput-P{}-0", self.party));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .map_err(|_| "owner private note input file")?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .map_err(|_| "owner input permissions")?;
        let view = hex::decode(view).map_err(|_| "note context digest")?;
        if view.len() != 64 {
            return Err("note context digest shape".into());
        }
        for chunk in view.chunks_exact(8) {
            writeln!(file, "{}", u64::from_le_bytes(chunk.try_into().unwrap()))
                .map_err(|_| "note public context input")?;
        }
        if initial {
            let input_path = &self.launch.private_input_paths[self.party];
            let metadata =
                fs::metadata(input_path).map_err(|_| "owner private note input missing")?;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err("owner note input file must be private".into());
            }
            let bytes = Zeroizing::new(fs::read(input_path).map_err(|_| "owner note input read")?);
            if hex::encode(Sha512::digest(&*bytes)) != self.launch.private_input_sha512[self.party]
            {
                return Err("owner private note input digest mismatch".into());
            }
            let mut input: PrivateInput =
                serde_json::from_slice(&bytes).map_err(|_| "owner note input encoding")?;
            if input.party != self.party
                || input.application_statement_sha512
                    != self.launch.statement.application_statement_sha512
                || input.circuit_sha512 != self.launch.statement.circuit_sha512
                || input.session_sha512 != self.launch.statement.session_sha512
            {
                return Err("owner note input binding mismatch".into());
            }
            let shares = unpack(&input.shares, self.launch.circuit.inputs.len())?;
            use zeroize::Zeroize;
            input.shares.zeroize();
            for share in shares {
                writeln!(file, "{share}").map_err(|_| "owner note share input")?;
            }
            for _ in 0..note_mpc::Layout::for_circuit(&self.launch.circuit)?.entropy_fields() {
                writeln!(file, "{}", F::rand(&mut OsRng))
                    .map_err(|_| "owner private note entropy")?;
            }
        }
        file.sync_all().map_err(|_| "owner note input sync")?;
        self.running = true;
        self.counter += 1;
        Ok(serde_json::json!({"party":self.party,"view":plan.view}))
    }
    fn finish(&mut self) -> Result<(), String> {
        if !self.running || self.prepared.is_none() {
            return Err("owner native note finish out of order".into());
        }
        let path = self
            .root
            .join("cosnark-private-logs")
            .join(format!("cocodeint_{:03}-P{}.out", self.counter, self.party));
        let log = fs::read_to_string(path).map_err(|_| "owner native public output")?;
        let mut values = Vec::new();
        for line in log.lines() {
            if let Some(raw) = line.strip_prefix("TRIAL_PUBLIC ") {
                let raw = raw.trim();
                values.push(if let Some(abs) = raw.strip_prefix('-') {
                    -F::from_str(abs).map_err(|_| "owner public native field")?
                } else {
                    F::from_str(raw).map_err(|_| "owner public native field")?
                });
            }
        }
        let count = match self.phase {
            Phase::Initialize => 0,
            Phase::InitialSums => 2,
            Phase::ConstraintRound | Phase::Endpoints => 4,
            Phase::PcsMask => 1,
            Phase::PcsSumcheck => 3,
            _ => return Err("owner native completion phase".into()),
        };
        if values.len() != count
            || (self.phase == Phase::Initialize && !log.lines().any(|l| l == "TRIAL_READY"))
        {
            return Err("owner native output shape or receipt".into());
        }
        match self.phase {
            Phase::Initialize => self.phase = Phase::AwaitRoots,
            Phase::InitialSums => {
                let t = self.t.as_mut().ok_or("note transcript missing")?;
                self.prefix
                    .as_mut()
                    .ok_or("note prefix missing")?
                    .initial_mask_sums = pack(&values);
                t.absorb_fields(b"initial-mask-sums", &values);
                let pad = t.field(b"constraint-padding");
                self.tau.push(pad);
                self.batch = t.field(b"sumcheck-mask-batch");
                self.claim = pad * values[0] + self.batch * values[1];
                self.phase = Phase::ConstraintRound;
            }
            Phase::ConstraintRound => {
                if values[0] + values[1] != self.claim {
                    return Err("owner native note sumcheck identity".into());
                }
                let t = self.t.as_mut().ok_or("note transcript missing")?;
                t.absorb_fields(b"constraint-h", &values);
                let r = t.field(b"constraint-round");
                self.claim = small_polynomial(&values, r);
                self.point.push(r);
                self.prefix
                    .as_mut()
                    .ok_or("note prefix missing")?
                    .sumcheck_rounds
                    .push(pack(&values));
                if self.point.len() == Relation::new(&self.launch.circuit)?.rounds {
                    self.phase = Phase::Endpoints;
                }
            }
            Phase::Endpoints => {
                self.prefix
                    .as_mut()
                    .ok_or("note prefix missing")?
                    .endpoint_claims = pack(&values);
                self.begin_opening()?;
            }
            Phase::PcsMask => {
                let s = self.session.as_mut().ok_or("note PCS session")?;
                s.proof.mask_claim = pack(&values);
                s.t.absorb_fields(b"pcs-mask-claim", &values);
                s.aggregation = s.t.field(b"pcs-aggregation");
                if s.aggregation == F::from(0u64) {
                    return Err("zero note PCS aggregation requires fresh session".into());
                }
                s.claim += s.aggregation * values[0];
                let kind = s.context.kind;
                s.current = self.oracles[kind]
                    .coefficients
                    .iter()
                    .zip(&self.oracles[kind + 1].coefficients)
                    .map(|(a, b)| *a + s.aggregation * b)
                    .collect();
                self.phase = Phase::PcsSumcheck;
            }
            Phase::PcsSumcheck => {
                let s = self.session.as_mut().ok_or("note PCS session")?;
                if values[0] + values[1] != s.claim {
                    return Err("owner recursive native sumcheck identity".into());
                }
                let round = s.folds.len();
                s.t.absorb(b"pcs-round", &(round as u64).to_le_bytes());
                s.t.absorb_fields(b"pcs-h", &values);
                let r = s.t.field(b"pcs-fold");
                s.claim = small_polynomial(&values, r);
                s.weights = fold_table(&s.weights, r);
                s.current = fold_table(&s.current, r);
                s.folds.push(r);
                s.proof.rounds.push(recursive_pcs::Round {
                    sumcheck: pack(&values),
                    next_roots: vec![],
                });
                let code = encode_coset(&s.current, round + 1)?;
                if round + 1 == s.context.parameters.rounds() {
                    s.own_terminal = Some(pack(&code));
                    self.phase = Phase::AwaitTerminal;
                } else {
                    s.layers.push(Merkle::new(
                        recursive_pcs::oracle_domain(s.context.kind, self.party, round + 1),
                        code,
                    ));
                    self.phase = Phase::AwaitFoldRoots;
                }
            }
            _ => return Err("owner native note unexpected finish".into()),
        }
        self.prepared = None;
        self.running = false;
        Ok(())
    }
    fn commit(&mut self) -> Result<Vec<String>, String> {
        if self.phase != Phase::AwaitRoots || !self.oracles.is_empty() {
            return Err("note owner commit out of order".into());
        }
        let relation = Relation::new(&self.launch.circuit)?;
        let saved = qomm_mpc::persistence::read(
            self.root
                .join("Persistence")
                .join(format!("Transactions-P{}.data", self.party)),
            self.party,
        )
        .map_err(|_| "owner note native persistence")?;
        if saved.shares.len() != relation.state_size()
            || saved.element_bytes != 32
            || saved.prime.to_bytes_be() != F::MODULUS.to_bytes_be()
        {
            return Err("owner note persistence field or layout".into());
        }
        let factor = if saved.montgomery {
            F::from(2u64).pow([256u64]).inverse().unwrap()
        } else {
            F::from(1u64)
        };
        for kind in 0..4 {
            let p = if kind < 2 {
                relation.witness
            } else {
                relation.mask
            };
            let offset = relation.oracle_offset(kind)?;
            let row = saved.shares[offset..offset + p.padded]
                .iter()
                .map(|v| {
                    let bytes = v.to_bytes_le(32).map_err(|_| "owner note share width")?;
                    Ok(
                        decode(bytes.try_into().unwrap()).ok_or("owner note share encoding")?
                            * factor,
                    )
                })
                .collect::<Result<Vec<F>, String>>()?;
            let coefficients = domain(p.padded).map_err(str::to_owned)?.ifft(&row);
            let tree = Merkle::new(
                recursive_pcs::oracle_domain(kind, self.party, 0),
                encode_coset(&coefficients, 0)?,
            );
            self.oracles.push(LocalOracle { coefficients, tree });
        }
        Ok(self.oracles.iter().map(|o| o.tree.root()).collect())
    }
    fn bind_roots(&mut self, roots: Vec<Vec<String>>) -> Result<(), String> {
        if self.phase != Phase::AwaitRoots
            || self.oracles.len() != 4
            || roots.len() != 4
            || roots.iter().any(|r| r.len() != 7)
            || roots
                .iter()
                .enumerate()
                .any(|(kind, row)| row[self.party] != self.oracles[kind].tree.root())
        {
            return Err("owner note root binding".into());
        }
        let mut t = note_piop::start(&self.launch.statement, &self.launch.circuit, &roots)?;
        self.tau = t.fields(
            b"constraint-weight",
            Relation::new(&self.launch.circuit)?.rounds - 1,
        );
        self.t = Some(t);
        self.prefix = Some(Prefix {
            statement: self.launch.statement.clone(),
            roots,
            initial_mask_sums: String::new(),
            sumcheck_rounds: vec![],
            endpoint_claims: String::new(),
        });
        self.phase = Phase::InitialSums;
        Ok(())
    }
    fn begin_opening(&mut self) -> Result<(), String> {
        let prefix = self.prefix.as_ref().ok_or("note prefix missing")?;
        let context = note_piop::opening_context(
            &self.launch.statement,
            &self.launch.circuit,
            prefix,
            self.prior_mask.as_ref(),
        )?;
        let mut t = context.transcript.clone();
        recursive_pcs::start(
            &mut t,
            context.parameters,
            context.kind,
            &prefix.roots[context.kind],
            &prefix.roots[context.kind + 1],
            context.claim,
        )?;
        recursive_pcs::bind_weights(&mut t, &context.weights);
        let weights = recursive_pcs::coefficient_weights(context.parameters, &context.weights)?;
        let claim = context.claim;
        self.session = Some(Session {
            context,
            t,
            weights,
            claim,
            aggregation: F::from(0u64),
            folds: vec![],
            current: vec![],
            layers: vec![],
            proof: OpeningProof {
                mask_claim: String::new(),
                rounds: vec![],
                terminal_rows: vec![],
                queries: recursive_pcs::Queries {
                    original: vec![],
                    mask: vec![],
                    folded: vec![],
                },
            },
            own_terminal: None,
            seeds: vec![],
            query_digest: None,
            budget: None,
        });
        self.phase = Phase::PcsMask;
        Ok(())
    }
    fn fold_roots(&mut self, roots: Vec<String>) -> Result<(), String> {
        if self.phase != Phase::AwaitFoldRoots {
            return Err("note fold roots out of order".into());
        }
        let s = self.session.as_mut().ok_or("note PCS session")?;
        if roots.len() != 7
            || Some(&roots[self.party]) != s.layers.last().map(Merkle::root).as_ref()
        {
            return Err("owner note folded root mismatch".into());
        }
        recursive_pcs::absorb_roots(&mut s.t, &roots)?;
        s.proof
            .rounds
            .last_mut()
            .ok_or("note fold step missing")?
            .next_roots = roots;
        self.phase = Phase::PcsSumcheck;
        Ok(())
    }
    fn prepare_query(&mut self, rows: Vec<String>) -> Result<OwnerApproval, String> {
        if self.phase != Phase::AwaitTerminal {
            return Err("note query preparation out of order".into());
        }
        let s = self.session.as_mut().ok_or("note PCS session")?;
        if rows.len() != 7 || Some(&rows[self.party]) != s.own_terminal.as_ref() {
            return Err("owner note terminal row mismatch".into());
        }
        recursive_pcs::terminal(&mut s.t, &rows)?;
        s.proof.terminal_rows = rows;
        s.seeds = (0..recursive_pcs::QUERIES)
            .map(|_| s.t.query(s.context.parameters.code(0) / 2))
            .collect();
        let body = serde_json::to_vec(&(
            "ZKFMI:NOTE:ALL-OWNER-QUERY:v1",
            &self.launch.statement,
            s.context.kind,
            s.context.parameters,
            hex::encode(s.t.view()),
            &s.seeds,
        ))
        .map_err(|_| "note query agreement encoding")?;
        let digest = hex::encode(Sha512::digest(&body));
        s.budget = Some(DurableQueryBudget::reserve_note(
            self.identity.state_root(),
            self.party,
            &self.launch.statement.session_sha512,
            &self.launch.statement.query_roster_sha512,
            s.context.kind,
            &digest,
        )?);
        let approval = self.identity.approve(&digest)?;
        s.query_digest = Some(digest);
        self.phase = Phase::AwaitApprovals;
        Ok(approval)
    }
    fn queries(&mut self, approvals: Vec<OwnerApproval>) -> Result<OwnerQueries, String> {
        if self.phase != Phase::AwaitApprovals {
            return Err("note query lifetime budget or order".into());
        }
        let s = self.session.as_mut().ok_or("note PCS session")?;
        query_agreement::verify_all_approvals(
            &self.roster,
            &self.launch.statement.query_roster_sha512,
            s.query_digest.as_ref().ok_or("note query digest missing")?,
            &approvals,
        )?;
        s.budget
            .as_ref()
            .ok_or("note query budget missing")?
            .consume(&approvals)?;
        let p = s.context.parameters;
        let indices = recursive_pcs::query_indices(p, 0, &s.seeds);
        let original = self.oracles[s.context.kind].tree.open_many(&indices)?;
        let mask = self.oracles[s.context.kind + 1].tree.open_many(&indices)?;
        let folded = s
            .layers
            .iter()
            .enumerate()
            .map(|(i, tree)| tree.open_many(&recursive_pcs::query_indices(p, i + 1, &s.seeds)))
            .collect::<Result<Vec<_>, _>>()?;
        self.phase = Phase::AwaitCompleted;
        Ok(OwnerQueries {
            original,
            mask,
            folded,
        })
    }
    fn complete(&mut self, proof: OpeningProof) -> Result<(), String> {
        if self.phase != Phase::AwaitCompleted {
            return Err("note opening completion out of order".into());
        }
        let s = self.session.as_mut().ok_or("note PCS session")?;
        let prefix = self.prefix.as_ref().ok_or("note prefix missing")?;
        let mut checked = s.context.transcript.clone();
        recursive_pcs::verify(
            &mut checked,
            s.context.parameters,
            s.context.kind,
            &prefix.roots[s.context.kind],
            &prefix.roots[s.context.kind + 1],
            &s.context.weights,
            s.context.claim,
            &proof,
        )?;
        if checked.view() != s.t.view() {
            return Err("owner completed note opening transcript mismatch".into());
        }
        if s.context.kind == 2 {
            self.prior_mask = Some(proof);
            self.begin_opening()?;
        } else {
            s.proof = proof;
            self.phase = Phase::Done;
        }
        Ok(())
    }
}

pub fn serve(root: &Path, party: usize, config: &Path, pin: &str) -> Result<(), String> {
    if party >= 7 {
        return Err("note party index".into());
    }
    let launch = read_launch(config, pin)?;
    let roster =
        TrustedRoster::read_pinned(&launch.roster_path, &launch.statement.query_roster_sha512)?;
    let identity = OwnerIdentity::load(
        &launch.owner_key_paths[party],
        party,
        &roster,
        &launch.statement.query_roster_sha512,
    )?;
    let mut owner = Owner {
        root: root.to_path_buf(),
        party,
        launch,
        roster,
        identity,
        phase: Phase::Initialize,
        prepared: None,
        running: false,
        counter: 0,
        oracles: vec![],
        prefix: None,
        t: None,
        tau: vec![],
        point: vec![],
        batch: F::from(0u64),
        claim: F::from(0u64),
        session: None,
        prior_mask: None,
    };
    for line in BufReader::new(std::io::stdin()).lines() {
        let line = line.map_err(|_| "note worker request input")?;
        let request: Request =
            serde_json::from_str(&line).map_err(|_| "note worker request encoding")?;
        let reply = match request {
            Request::Status => serde_json::to_value(owner.status()),
            Request::Prepare => serde_json::to_value(owner.prepare()?),
            Request::Input {
                view,
                initial,
                invalid,
            } => Ok(owner.write_input(&view, initial, invalid)?),
            Request::Finish => {
                owner.finish()?;
                serde_json::to_value(owner.status())
            }
            Request::Commit => serde_json::to_value(owner.commit()?),
            Request::BindRoots { roots } => {
                owner.bind_roots(roots)?;
                serde_json::to_value(owner.status())
            }
            Request::FoldRoots { roots } => {
                owner.fold_roots(roots)?;
                serde_json::to_value(owner.status())
            }
            Request::PrepareQuery { rows } => serde_json::to_value(owner.prepare_query(rows)?),
            Request::Queries { approvals } => serde_json::to_value(owner.queries(approvals)?),
            Request::Complete { proof } => {
                owner.complete(proof)?;
                serde_json::to_value(owner.status())
            }
            Request::Shutdown => break,
        }
        .map_err(|_| "note worker public response encoding")?;
        let mut out = std::io::stdout().lock();
        serde_json::to_writer(&mut out, &reply).map_err(|_| "note worker public output")?;
        writeln!(out)
            .and_then(|_| out.flush())
            .map_err(|_| "note worker response flush")?;
    }
    Ok(())
}

/// Concrete seven-process native proof path. No scalar interpolation or
/// private oracle loading occurs in this coordinator function.
pub fn prove(
    root: &Path,
    port: u16,
    config: &Path,
    pin: &str,
) -> Result<(note_piop::Proof, Vec<RunRecord>), String> {
    let launch = read_launch(config, pin)?;
    let mut native = Native::new_note_workers(root, port, config, pin)?;
    loop {
        let states: Vec<Status> = native.note_broadcast(&Request::Status)?;
        let first = states.first().ok_or("note worker roster empty")?;
        if states.len() != 7 || states.iter().any(|s| s.phase != first.phase) {
            return Err("note owner phase disagreement".into());
        }
        match first.phase {
            Phase::Initialize
            | Phase::InitialSums
            | Phase::ConstraintRound
            | Phase::Endpoints
            | Phase::PcsMask
            | Phase::PcsSumcheck => {
                let plans: Vec<Plan> = native.note_broadcast(&Request::Prepare)?;
                let first = plans.first().ok_or("note program roster empty")?;
                if plans.len() != 7
                    || plans.iter().any(|p| {
                        p.sha256 != first.sha256
                            || p.view != first.view
                            || p.initial != first.initial
                    })
                {
                    return Err("seven note owners disagree on native program".into());
                }
                let source = first
                    .source
                    .as_ref()
                    .ok_or("owner public native source missing")?;
                if sha256(source.as_bytes()) != first.sha256 {
                    return Err("note public native source digest".into());
                }
                let view: [u8; 64] = hex::decode(&first.view)
                    .map_err(|_| "note native context")?
                    .try_into()
                    .map_err(|_| "note native context shape")?;
                native.run(source.clone(), view, first.initial, false, false)?;
                let _: Vec<Status> = native.note_broadcast(&Request::Finish)?;
            }
            Phase::AwaitRoots => {
                let roots: Vec<Vec<String>> = native.note_broadcast(&Request::Commit)?;
                if roots.len() != 7 || roots.iter().any(|r| r.len() != 4) {
                    return Err("note owner commitment roster".into());
                }
                let roots = (0..4)
                    .map(|kind| roots.iter().map(|p| p[kind].clone()).collect())
                    .collect();
                let _: Vec<Status> = native.note_broadcast(&Request::BindRoots { roots })?;
            }
            Phase::AwaitFoldRoots => {
                let roots = states
                    .iter()
                    .map(|s| s.next_root.clone().ok_or("note folded owner root"))
                    .collect::<Result<Vec<_>, _>>()?;
                let _: Vec<Status> = native.note_broadcast(&Request::FoldRoots { roots })?;
            }
            Phase::AwaitTerminal => {
                let rows = states
                    .iter()
                    .map(|s| s.terminal_row.clone().ok_or("note terminal owner row"))
                    .collect::<Result<Vec<_>, _>>()?;
                let approvals: Vec<OwnerApproval> =
                    native.note_broadcast(&Request::PrepareQuery { rows })?;
                let statuses: Vec<Status> = native.note_broadcast(&Request::Status)?;
                let mut proof = statuses[0]
                    .opening
                    .clone()
                    .ok_or("note opening public messages missing")?;
                let queries: Vec<OwnerQueries> =
                    native.note_broadcast(&Request::Queries { approvals })?;
                if queries.len() != 7
                    || queries
                        .iter()
                        .any(|q| q.folded.len() + 1 != proof.rounds.len())
                {
                    return Err("note owner query response shape".into());
                }
                proof.queries = recursive_pcs::Queries {
                    original: queries.iter().map(|q| q.original.clone()).collect(),
                    mask: queries.iter().map(|q| q.mask.clone()).collect(),
                    folded: (0..proof.rounds.len() - 1)
                        .map(|i| queries.iter().map(|q| q.folded[i].clone()).collect())
                        .collect(),
                };
                let _: Vec<Status> = native.note_broadcast(&Request::Complete { proof })?;
            }
            Phase::Done => {
                let proof = note_piop::Proof {
                    prefix: first.prefix.clone().ok_or("note final prefix")?,
                    mask_opening: first.prior_mask.clone().ok_or("note final mask opening")?,
                    witness_opening: first.opening.clone().ok_or("note final witness opening")?,
                    transcript_sha512: first
                        .transcript_sha512
                        .clone()
                        .ok_or("note final transcript")?,
                };
                note_piop::verify(&launch.statement, &launch.circuit, &proof)?;
                return Ok((proof, std::mem::take(&mut native.records)));
            }
            _ => return Err("unexpected note coordinator phase".into()),
        }
    }
}

pub fn prove_to_directory(
    root: &Path,
    port: u16,
    config: &Path,
    pin: &str,
    output: &Path,
) -> Result<(), String> {
    fs::create_dir(output).map_err(|_| "note proof output must be new")?;
    let (proof, records) = prove(root, port, config, pin)?;
    let bytes = serde_json::to_vec(&proof).map_err(|_| "note public proof encoding")?;
    let records =
        serde_json::to_vec_pretty(&records).map_err(|_| "note native records encoding")?;
    for (name, body) in [("proof.json", bytes), ("native.json", records)] {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(name))
            .map_err(|_| "note proof output creation")?;
        file.write_all(&body)
            .and_then(|_| file.sync_all())
            .map_err(|_| "note proof output persistence")?;
    }
    Ok(())
}
