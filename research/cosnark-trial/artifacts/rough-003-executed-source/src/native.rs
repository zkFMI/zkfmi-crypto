//! Native seven-party process orchestration. Secret input generation and raw
//! persistence loading live only in the dedicated per-party child workers.
use crate::{circuit::PARTIES,field::F,mpc_program,party::Request,pcs::NodeRoots,transcript::Hash};
use qomm_mpc::{compiler::OfficialCompiler,engine_policy::EnginePin};
use serde::{de::DeserializeOwned,Serialize};
use sha2::{Digest,Sha256};
use std::{fs::{self,File,OpenOptions},io::{BufRead,BufReader,Write},os::unix::fs::OpenOptionsExt,path::{Path,PathBuf},process::{Child,ChildStdin,ChildStdout,Command,Stdio},str::FromStr,time::Instant};

pub struct Worker { child:Child,input:ChildStdin,output:BufReader<ChildStdout> }
impl Worker {
    fn send(&mut self,request:&Request)->Result<(),String> {
        serde_json::to_writer(&mut self.input,request).map_err(|_|"worker request write")?;
        writeln!(self.input).map_err(|_|"worker request newline")?; self.input.flush().map_err(|_|"worker flush".to_string())
    }
    fn receive<T:DeserializeOwned>(&mut self)->Result<T,String> {
        let mut answer=String::new();
        if self.output.read_line(&mut answer).map_err(|_|"worker response read")?==0 { return Err("private worker exited; see owner-local error log".into()); }
        serde_json::from_str(&answer).map_err(|_|"worker public response shape".to_string())
    }
}
impl Drop for Worker {
    fn drop(&mut self) { let _=self.send(&Request::Shutdown); let _=self.child.wait(); }
}

#[derive(Serialize,Clone)]
pub struct RunRecord {
    pub program:String,pub source_sha256:String,pub compiled_artifacts:Vec<(String,String)>,
    pub party_exit_codes:Vec<Option<i32>>,pub seconds:f64,pub released_field_count:usize,
    pub seven_context_views_equal:bool,
    pub rejection_reason:Option<String>,
}

pub struct Native {
    root:PathBuf,compiler:OfficialCompiler,pin:EnginePin,workers:Vec<Worker>,
    counter:usize,pub records:Vec<RunRecord>,port:u16,
}
fn public_hash(bytes:&[u8])->String { hex::encode(Sha256::digest(bytes)) }
fn private_log(path:&Path)->Result<File,String> {
    OpenOptions::new().create_new(true).write(true).mode(0o600).open(path).map_err(|_|"unique native log file".to_string())
}

impl Native {
    pub fn new(root:&Path,port:u16)->Result<Self,String> {
        let root=fs::canonicalize(root).map_err(|_|"native root missing")?;
        let pin=EnginePin::verify(&root)?;
        let compiler=OfficialCompiler::from_checkout(&root).map_err(|e|e.to_string())?;
        fs::create_dir_all(root.join("Persistence")).map_err(|_|"persistence directory")?;
        fs::create_dir(root.join("cosnark-private-logs")).map_err(|_|"fresh native run required")?;
        for party in 0..PARTIES {
            if root.join("Persistence").join(format!("Transactions-P{party}.data")).exists() {
                return Err("refusing existing private persistence; use fresh isolated engine".into());
            }
        }
        let mut workers=Vec::new();
        for party in 0..PARTIES {
            let log=private_log(&root.join("cosnark-private-logs").join(format!("worker-{party}.log")))?;
            let mut child=Command::new(std::env::current_exe().map_err(|_|"current trial executable")?)
                .args(["worker",root.to_str().ok_or("native path utf8")?,&party.to_string()])
                .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::from(log)).spawn().map_err(|_|"launch owner worker")?;
            workers.push(Worker { input:child.stdin.take().unwrap(),output:BufReader::new(child.stdout.take().unwrap()),child });
        }
        Ok(Self{root,compiler,pin,workers,counter:0,records:Vec::new(),port})
    }
    pub fn broadcast<T:DeserializeOwned>(&mut self,request:&Request)->Result<Vec<T>,String> {
        for worker in &mut self.workers { worker.send(request)?; }
        self.workers.iter_mut().map(Worker::receive).collect()
    }
    pub fn roots(&mut self)->Result<Vec<NodeRoots>,String> { self.broadcast(&Request::Commit) }
    pub fn run(&mut self,source:String,view:Hash,initial:bool,invalid:bool,corrupt_views:bool)->Result<Vec<F>,String> {
        self.pin.recheck()?;
        self.counter+=1;
        let name=format!("cocode002_{:03}",self.counter);
        let source_path=self.root.join("Programs/Source").join(format!("{name}.mpc"));
        fs::write(&source_path,source.as_bytes()).map_err(|_|"write public MPC program")?;
        let started=Instant::now();
        // -F is the usable INTEGER bit length, not the prime field size.
        // The 254-bit prime is bound separately with -P at compile and runtime.
        let compile=self.compiler.command().args(["-F","64","-P",&mpc_program::prime(),&name]).output().map_err(|_|"launch official compiler")?;
        let log_dir=self.root.join("cosnark-private-logs");
        let mut compiler_log=private_log(&log_dir.join(format!("{name}-compiler.log")))?;
        compiler_log.write_all(&compile.stdout).map_err(|_|"compiler log")?;
        compiler_log.write_all(&compile.stderr).map_err(|_|"compiler log")?;
        if !compile.status.success() { return Err(format!("official compiler failed for {name}; public-source compiler log: {}",log_dir.join(format!("{name}-compiler.log")).display())); }
        let mut compiled=Vec::new();
        for folder in ["Programs/Bytecode","Programs/Schedules"] {
            for item in fs::read_dir(self.root.join(folder)).map_err(|_|"compiled artifact directory")? {
                let item=item.map_err(|_|"compiled artifact entry")?;
                let file=item.file_name().to_string_lossy().to_string();
                if file.starts_with(&format!("{name}-")) || file==format!("{name}.sch") {
                    compiled.push((format!("{folder}/{file}"),public_hash(&fs::read(item.path()).map_err(|_|"compiled artifact read")?)));
                }
            }
        }
        compiled.sort(); if compiled.len()<2 { return Err("missing compiled circuit artifacts".into()); }
        let mut view_ids=Vec::new();
        for (party,worker) in self.workers.iter_mut().enumerate() {
            let mut local=view;
            if corrupt_views && party>=5 { local[0]^=1; }
            let local=hex::encode(local); view_ids.push(local.clone());
            worker.send(&Request::Input{view:local,initial,invalid})?;
        }
        for (party,worker) in self.workers.iter_mut().enumerate() {
            let ack:serde_json::Value=worker.receive()?;
            if ack["party"]!=party || ack["view"]!=view_ids[party] { return Err("owner context acknowledgement".into()); }
        }
        // An inconsistent view is deliberately allowed to reach the actual
        // native all-party check for the preregistered two-party tamper case.
        let mut children=Vec::new();
        for party in 0..PARTIES {
            let out=private_log(&log_dir.join(format!("{name}-P{party}.out")))?;
            let err=private_log(&log_dir.join(format!("{name}-P{party}.err")))?;
            let child=Command::new(self.root.join("malicious-shamir-party.x"))
                .current_dir(&self.root)
                .args(["-N","7","-T","2","-S","128","-P",&mpc_program::prime(),"-lgp","254","-pn",&self.port.to_string(),"-IF","Player-Data/CosnarkInput",&party.to_string(),&name])
                .stdout(Stdio::from(out)).stderr(Stdio::from(err)).spawn().map_err(|_|"launch native MPC party")?;
            children.push(child);
        }
        let statuses=children.iter_mut().map(|child|child.wait().map_err(|_|"wait native MPC party")).collect::<Result<Vec<_>,_>>()?;
        let output=fs::read_to_string(log_dir.join(format!("{name}-P0.out"))).map_err(|_|"read public MPC output")?;
        let mut released=Vec::new();
        for line in output.lines() {
            if let Some(value)=line.strip_prefix("TRIAL_PUBLIC ") {
                let value=value.trim();
                let field=if let Some(abs)=value.strip_prefix('-') { -F::from_str(abs).map_err(|_|"native public field encoding")? }
                    else { F::from_str(value).map_err(|_|"native public field encoding")? };
                released.push(field);
            }
        }
        let success=statuses.iter().all(|s|s.success());
        let mut rejection_reason=None;
        if !success {
            for party in 0..PARTIES {
                for extension in ["out","err"] {
                    let text=fs::read_to_string(log_dir.join(format!("{name}-P{party}.{extension}"))).map_err(|_|"native rejection evidence")?;
                    for marker in ["invalid financial witness","challenge view mismatch"] {
                        if text.contains(marker) { rejection_reason=Some(marker.to_string()); }
                    }
                }
            }
        }
        self.records.push(RunRecord{program:name.clone(),source_sha256:public_hash(source.as_bytes()),compiled_artifacts:compiled,
            party_exit_codes:statuses.iter().map(|s|s.code()).collect(),seconds:started.elapsed().as_secs_f64(),released_field_count:released.len(),
            seven_context_views_equal:!corrupt_views,rejection_reason});
        if !success { return Err(format!("native seven-party run rejected: {name}; no private log contents released")); }
        if initial && !output.lines().any(|l|l=="TRIAL_READY") { return Err("native initialization receipt missing".into()); }
        Ok(released)
    }
}
