//! Explicit test-only scalar grouping. Requests, admission, comparison and
//! per-case receipts still use the ordinary parent adapter, once per item.
use super::*;
use std::io::BufRead;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};

const SCHEMA: &str = "avworker-test-batch/1";
const FRAME_CAP: usize = 4 << 20;

pub mod combined;
pub mod pilot;

pub struct Item<'a> {
    pub case: &'a ResolvedCase,
    pub check: &'a CheckSpec,
    pub oracle: &'a ResolvedOracle,
    pub project_dir: &'a Path,
    pub artifact_dir: &'a Path,
    pub run_dir: &'a Path,
}

pub struct ItemResult {
    pub id: String,
    pub status: Status,
    pub verdict: Option<Verdict>,
    pub detail: Option<String>,
    pub wall_ms: f64,
}

pub struct GroupResult {
    pub group: String,
    pub items: Vec<ItemResult>,
    pub error: Option<String>,
    pub worker_starts: usize,
    pub worker_pid: u32,
    pub wall_ms: f64,
    pub stderr: String,
    pub cleanup_complete: bool,
    pub worker_exe_sha256: String,
}

fn id(item: &Item<'_>) -> String {
    format!("{}/{}", item.case.spec.id, item.check.id)
}

pub fn run_group(items: &[Item<'_>], budget: Duration) -> Result<GroupResult, String> {
    run_group_with_fault(items, budget, None)
}

/// Counter-controls are scoped to the spawned worker, never process-global env.
#[doc(hidden)]
pub fn run_group_with_fault(
    items: &[Item<'_>],
    budget: Duration,
    fault: Option<&str>,
) -> Result<GroupResult, String> {
    let started = Instant::now();
    let mut parent_clock = parent_observations::Clock::new();
    if items.is_empty() || items.len() > 10 || budget.is_zero() || budget > SESSION_WALL {
        return Err("batch requires 1..10 items and group wall in (0,60s]".into());
    }
    let ids: Vec<_> = items.iter().map(id).collect();
    if ids.iter().collect::<std::collections::BTreeSet<_>>().len() != ids.len() {
        return Err("duplicate batch item identity".into());
    }
    for item in items {
        if Instant::now() >= started + budget {
            return Err("batch deadline includes whitelist preparation".into());
        }
        let av = item
            .check
            .options
            .probe_av
            .as_ref()
            .ok_or("missing AV binding")?;
        if item.check.options.probe_recipe.as_deref() != Some(AV_RECIPE)
            || av.profile.is_some()
            || av.session.is_some()
            || !av.caps.is_empty()
            || av.entry != "d_main"
            || !matches!(item.oracle.completion, Completion::Success)
            || !matches!(item.check.expect.completion, Completion::Success)
            || item.oracle.exit.is_none()
            || item.check.skip.is_some()
            || item.case.spec.status_skip.is_some()
            || !matches!(
                item.check.options.probe_steps.as_deref(),
                Some([auto_test_case::contract::ProbeStep::Run])
            )
        {
            return Err(format!(
                "{} is outside the batch scalar whitelist",
                id(item)
            ));
        }
        let input = item
            .case
            .files
            .first()
            .ok_or("batch entry source missing")?;
        let allowlist: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../auto-lang/test/cases/governance-888/batch-allowlist.json"
        )))
        .map_err(|e| format!("batch allowlist: {e}"))?;
        let source_sha = hex(&sha256(&input.bytes));
        let allowed = item.case.files.len() == 1
            && allowlist["entries"]
                .as_array()
                .ok_or("empty batch allowlist")?
                .iter()
                .any(|row| {
                    row["case"] == item.case.spec.id
                        && row["check"] == item.check.id
                        && row["entry_sha256"] == source_sha
                });
        if !allowed && fault != Some("no-read-after-ready") {
            return Err(format!(
                "{} source is not in the reviewed 15-case batch allowlist",
                id(item)
            ));
        }
    }
    if Instant::now() >= started + budget {
        return Err("batch deadline includes whitelist preparation".into());
    }
    let group = hex(&sha256(
        format!("{}|{:?}|{:?}", std::process::id(), started, ids).as_bytes(),
    ));
    parent_clock.lap("prepare_ms");
    let mut worker = Transport::start(&group, started + budget, fault)?;
    let pid = worker.child.id();
    let mut results = Vec::new();
    let mut error = {
        let _transport = parent_observations::Phase::start("transport_ms");
        worker.initialize(&ids).err()
    };
    for (index, item) in items.iter().enumerate() {
        if error.is_some() {
            results.push(ItemResult {
                id: ids[index].clone(),
                status: Status::NotRun,
                verdict: None,
                detail: Some("batch stopped by preceding infrastructure failure".into()),
                wall_ms: 0.0,
            });
            continue;
        }
        let item_started = Instant::now();
        let mut item_deadline = started + budget;
        let leased_worker_sha = worker.exe_sha.clone();
        let judged = run_check_with_transport(
            item.case,
            item.check,
            item.oracle,
            item.project_dir,
            item.artifact_dir,
            item.run_dir,
            RecipeFamily::Core,
            |request, wall| {
                item_deadline = (item_started + wall).min(started + budget);
                worker.request(index, &ids[index], request, item_deadline)
            },
            Some(&leased_worker_sha),
        );
        let mut verdict = match judged {
            Ok(v) => v,
            Err(e) => Verdict::error(e),
        };
        if Instant::now() >= item_deadline {
            verdict =
                Verdict::error("group deadline includes parent preparation/admission/comparison");
        }
        if verdict.status == Status::Error {
            // The ordinary adapter keeps the precise admission/transport
            // rejection in its worker mismatch; preserve it in the group
            // diagnostic rather than flattening every fault to "infra failure".
            error = verdict
                .mismatches
                .iter()
                .find(|m| m.observation == "worker" && m.kind == "infra")
                .map(|m| m.actual.clone())
                .or_else(|| verdict.detail.clone())
                .or(Some("item infrastructure failure".into()));
        }
        results.push(ItemResult {
            id: ids[index].clone(),
            status: verdict.status,
            detail: verdict.detail.clone(),
            verdict: Some(verdict),
            wall_ms: item_started.elapsed().as_secs_f64() * 1000.0,
        });
        // An assertion FAIL is safe: preserve it and continue the next item.
    }
    if error.is_none() {
        let _transport = parent_observations::Phase::start("transport_ms");
        if let Err(e) = worker.finish(items.len()) {
            error = Some(e);
        }
    }
    let cleanup = {
        let _cleanup = parent_observations::Phase::start("cleanup_ms");
        worker.shutdown()
    };
    let cleanup_complete = cleanup.is_ok();
    if let Err(e) = cleanup {
        error = Some(format!("{}; cleanup: {e}", error.unwrap_or_default()));
    }
    if worker.overflow.load(Ordering::SeqCst) {
        error.get_or_insert("batch stderr cap exceeded".into());
    }
    if Instant::now() >= started + budget {
        error
            .get_or_insert("batch absolute deadline includes parent comparison and cleanup".into());
    }
    let stderr = String::from_utf8_lossy(&worker.stderr.lock().unwrap_or_else(|e| e.into_inner()))
        .into_owned();
    Ok(GroupResult {
        group,
        items: results,
        error,
        worker_starts: 1,
        worker_pid: pid,
        wall_ms: started.elapsed().as_secs_f64() * 1000.0,
        stderr,
        cleanup_complete,
        worker_exe_sha256: worker.exe_sha.clone(),
    })
}

struct Transport {
    child: std::process::Child,
    job: Option<containment::JobGuard>,
    group: String,
    deadline: Instant,
    writer: Option<mpsc::SyncSender<Option<Vec<u8>>>>,
    written: mpsc::Receiver<Result<(), String>>,
    frames: Option<mpsc::Receiver<Result<Option<serde_json::Value>, String>>>,
    threads: Vec<std::thread::JoinHandle<()>>,
    overflow: Arc<AtomicBool>,
    stderr: Arc<Mutex<Vec<u8>>>,
    case_stores: Vec<PathBuf>,
    exe: PathBuf,
    exe_sha: String,
    lease: Option<std::fs::File>,
}

impl Transport {
    fn start(group: &str, deadline: Instant, fault: Option<&str>) -> Result<Self, String> {
        Self::start_command(group, deadline, fault, "test-batch-serve")
    }

    fn start_command(
        group: &str,
        deadline: Instant,
        fault: Option<&str>,
        subcommand: &str,
    ) -> Result<Self, String> {
        let _transport = parent_observations::Phase::start("transport_ms");
        if !cfg!(windows) {
            return Err(
                "test batch requires implemented process-tree containment on this platform".into(),
            );
        }
        // Test grouping's absolute wall cannot hide an implicit release build.
        // Ordinary independent run_check keeps its existing build behaviour.
        let exe = std::env::var_os("AC_AV_CASE_WORKER")
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .ok_or("batch/combined requires an explicitly prebuilt AC_AV_CASE_WORKER")?;
        let lease = lease_executable(&exe)?;
        let exe_sha = worker_file_sha256(&exe)?;
        let mut command = Command::new(&exe);
        if let Some(fault) = fault {
            command.env("AV888_BATCH_FAULT", fault);
        }
        command
            .arg(subcommand)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(containment::CREATE_SUSPENDED);
        }
        let mut child = command.spawn().map_err(|e| format!("batch spawn: {e}"))?;
        parent_observations::spawned(child.id());
        let Some(job) = containment::JobGuard::for_child(&child) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err("batch containment unavailable".into());
        };
        if let Err(e) = containment::resume_suspended_process(&child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e.into());
        }
        let mut input = child.stdin.take().ok_or("batch stdin unavailable")?;
        let output = child.stdout.take().ok_or("batch stdout unavailable")?;
        let mut errors = child.stderr.take().ok_or("batch stderr unavailable")?;
        let (tx, rx) = mpsc::sync_channel::<Option<Vec<u8>>>(1);
        let (ack_tx, written) = mpsc::channel();
        let writer = std::thread::spawn(move || {
            while let Ok(Some(bytes)) = rx.recv() {
                let result = input
                    .write_all(&bytes)
                    .and_then(|_| input.flush())
                    .map_err(|e| e.to_string());
                let failed = result.is_err();
                if ack_tx.send(result).is_err() || failed {
                    break;
                }
            }
        });
        let (out_tx, frames) = mpsc::sync_channel(2);
        let reader = std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(output);
            let mut total = 0;
            loop {
                let frame = read_response(&mut reader, &mut total);
                let terminal = matches!(frame, Err(_) | Ok(None));
                if out_tx.send(frame).is_err() || terminal {
                    break;
                }
            }
        });
        let overflow = Arc::new(AtomicBool::new(false));
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let (flag, saved) = (overflow.clone(), stderr.clone());
        let err_reader = std::thread::spawn(move || {
            let mut bytes = [0u8; 4096];
            while let Ok(n) = errors.read(&mut bytes) {
                if n == 0 {
                    break;
                }
                let mut saved = saved.lock().unwrap_or_else(|e| e.into_inner());
                if saved.len() + n > STDERR_CAP {
                    flag.store(true, Ordering::SeqCst);
                    break;
                }
                saved.extend_from_slice(&bytes[..n]);
            }
        });
        Ok(Self {
            child,
            job: Some(job),
            group: group.into(),
            deadline,
            writer: Some(tx),
            written,
            frames: Some(frames),
            threads: vec![writer, reader, err_reader],
            overflow,
            stderr,
            case_stores: Vec::new(),
            exe,
            exe_sha,
            lease: Some(lease),
        })
    }

    fn initialize(&mut self, ids: &[String]) -> Result<(), String> {
        self.send(
            serde_json::json!({"kind":"init","schema":SCHEMA,"group":self.group,"items":ids}),
            self.deadline,
        )?;
        let ready = self
            .frame(self.deadline)?
            .ok_or("batch worker exited without readiness (feature/capability missing)")?;
        if ready["schema"] != SCHEMA
            || ready["kind"] != "ready"
            || ready["group"] != self.group
            || ready["items"] != serde_json::json!(ids)
            || ready["capabilities"]["worker_version"] != EXPECTED_AV_WORKER_VERSION
            || ready["capabilities"]["max_items"] != 10
            || ready["capabilities"]["frame_cap"] != FRAME_CAP
            || ready["capabilities"]["profiles"] != serde_json::json!(["scalar"])
            || ready["capabilities"]["schema"] != SCHEMA
            || ready["capabilities"]["expected_in_worker"] != false
        {
            return Err("batch readiness/capability/manifest mismatch".into());
        }
        Ok(())
    }

    fn check(&self, deadline: Instant) -> Result<(), String> {
        if Instant::now() >= self.deadline.min(deadline) {
            return Err("batch absolute deadline".into());
        }
        if self.overflow.load(Ordering::SeqCst) {
            return Err("batch stderr cap exceeded".into());
        }
        Ok(())
    }

    fn send(&mut self, frame: serde_json::Value, deadline: Instant) -> Result<(), String> {
        self.check(deadline)?;
        let mut bytes = serde_json::to_vec(&frame).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        if bytes.len() > FRAME_CAP {
            return Err("batch request frame cap exceeded".into());
        }
        self.writer
            .as_ref()
            .ok_or("batch writer closed")?
            .try_send(Some(bytes))
            .map_err(|e| format!("batch writer queue: {e}"))?;
        loop {
            self.check(deadline)?;
            match self.written.recv_timeout(Duration::from_millis(2)) {
                Ok(r) => return r,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(e) => return Err(format!("batch writer: {e}")),
            }
        }
    }

    fn frame(&mut self, deadline: Instant) -> Result<Option<serde_json::Value>, String> {
        loop {
            self.check(deadline)?;
            match self
                .frames
                .as_ref()
                .ok_or("batch reader closed")?
                .recv_timeout(Duration::from_millis(2))
            {
                Ok(r) => return r,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(e) => return Err(format!("batch reader: {e}")),
            }
        }
    }

    fn request(
        &mut self,
        index: usize,
        id: &str,
        request: &str,
        deadline: Instant,
    ) -> Result<WorkerResponse, String> {
        self.check_tool()?;
        let request: serde_json::Value =
            serde_json::from_str(request).map_err(|e| e.to_string())?;
        let nonce = request["nonce"].as_str().ok_or("request nonce missing")?;
        let store_root = request["store_root"]
            .as_str()
            .ok_or("request store root missing")?;
        if nonce.len() != 64 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("request nonce must bind the private case store".into());
        }
        self.case_stores
            .push(Path::new(store_root).join(format!("case-{}", &nonce[..16])));
        self.send(
            serde_json::json!({"kind":"item","schema":SCHEMA,"group":self.group,
            "index":index,"id":id,"request":request}),
            deadline,
        )?;
        let frame = self
            .frame(deadline)?
            .ok_or("batch worker EOF before item actual")?;
        if frame["schema"] != SCHEMA
            || frame["kind"] != "actual"
            || frame["group"] != self.group
            || frame["index"] != index
            || frame["id"] != id
        {
            return Err("batch actual identity/order mismatch".into());
        }
        serde_json::from_value(frame["response"].clone())
            .map_err(|e| format!("batch actual shape: {e}"))
    }

    fn finish(&mut self, count: usize) -> Result<(), String> {
        self.check_tool()?;
        self.send(
            serde_json::json!({"kind":"finish","schema":SCHEMA,"group":self.group,"count":count}),
            self.deadline,
        )?;
        // Dropping the channel closes stdin after the already-acked finish.
        self.writer.take();
        let done = self.frame(self.deadline)?.ok_or("missing batch done")?;
        if done["kind"] != "done"
            || done["schema"] != SCHEMA
            || done["group"] != self.group
            || done["count"] != count
            || done["worker_starts"] != 1
            || done["case_calls"] != count
        {
            return Err("batch done count/identity mismatch".into());
        }
        if self.frame(self.deadline)?.is_some() {
            return Err("extra batch output after done".into());
        }
        loop {
            self.check(self.deadline)?;
            if let Some(status) = self.child.try_wait().map_err(|e| e.to_string())? {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!("batch nonzero exit: {status}"))
                };
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn shutdown(&mut self) -> Result<(), String> {
        let _ = self.child.kill();
        self.job.take();
        let _ = self.child.wait();
        self.writer.take();
        self.frames.take();
        let grace = Instant::now() + READER_GRACE;
        while self.threads.iter().any(|t| !t.is_finished()) && Instant::now() < grace {
            std::thread::sleep(Duration::from_millis(1));
        }
        let mut error = None;
        for thread in self.threads.drain(..) {
            if thread.is_finished() {
                if thread.join().is_err() {
                    error = Some("batch I/O thread panicked".into());
                }
            } else {
                error = Some("batch I/O cleanup deadline exceeded".into());
            }
        }
        for store in &self.case_stores {
            if store.exists() {
                if let Err(e) = std::fs::remove_dir_all(store) {
                    error = Some(format!("batch case-store cleanup: {e}"));
                }
            }
        }
        self.lease.take();
        error.map_or(Ok(()), Err)
    }

    fn check_tool(&self) -> Result<(), String> {
        let lease = self
            .lease
            .as_ref()
            .ok_or("batch executable lease no longer held")?;
        if !lease.metadata().map_err(|e| e.to_string())?.is_file() || !self.exe.is_file() {
            return Err("batch executable lease target is not a file".into());
        }
        Ok(())
    }
}

#[cfg(windows)]
fn lease_executable(path: &Path) -> Result<std::fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    // FILE_SHARE_READ: later readers/execution are permitted; write access and
    // rename/delete are denied for this same file for the entire group.
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(path)
        .map_err(|e| format!("immutable batch tool lease: {e}"))
}

#[cfg(not(windows))]
fn lease_executable(_path: &Path) -> Result<std::fs::File, String> {
    Err("batch immutable file lease is not implemented on this platform".into())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn tool_digest_reuse_requires_a_real_write_and_replace_exclusion_lease() {
        let root = std::env::temp_dir().join(format!("plan888-tool-lease-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("tool.bin");
        std::fs::write(&path, b"original executable identity").unwrap();
        let before = worker_file_sha256(&path).unwrap();
        let lease = lease_executable(&path).unwrap();
        assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err());
        assert!(std::fs::rename(&path, root.join("replaced.bin")).is_err());
        assert!(std::fs::remove_file(&path).is_err());
        assert_eq!(worker_file_sha256(&path).unwrap(), before);
        drop(lease);
        std::fs::write(&path, b"replacement executable identity").unwrap();
        assert_ne!(worker_file_sha256(&path).unwrap(), before);
        std::fs::remove_dir_all(root).unwrap();
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn read_response(
    reader: &mut impl BufRead,
    total: &mut usize,
) -> Result<Option<serde_json::Value>, String> {
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|e| e.to_string())?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err("truncated batch response".into())
            };
        }
        let n = available
            .iter()
            .position(|&b| b == b'\n')
            .map_or(available.len(), |n| n + 1);
        if bytes.len() + n > FRAME_CAP || *total + n > 10 * FRAME_CAP {
            return Err("batch response metadata cap".into());
        }
        let end = available[n - 1] == b'\n';
        bytes.extend_from_slice(&available[..n]);
        *total += n;
        reader.consume(n);
        if end {
            break;
        }
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| format!("invalid batch response: {e}"))
}
