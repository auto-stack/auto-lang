//! `probe` adapter executor (PLAN-796 T-06 + R4 repairs): the
//! parent-side half of the restricted probe/run seam. auto-lang NEVER
//! links the experimental semantic stack — this module spawns the
//! `sema-probe` worker (env override `SEMA_CASE_WORKER`, else located
//! from the repo layout and built release once per process) speaking
//! `semaworker/1` and JUDGES its structured step reports against the
//! case oracle. The worker's word is never trusted.
//!
//! R4 repairs wired here:
//! * F-796-R4-02 — typed outcome admission BEFORE any oracle judging:
//!   `ok` requires zero diagnostics; `diags` must match a declared
//!   error completion (stage/code/message); `resource`/`cancelled`/
//!   `infra` are infrastructure errors, never language verdicts.
//! * F-796-R4-03 — full lifecycle: the parent wall covers response AND
//!   exit/reap (a worker sleeping after its response is killed at the
//!   deadline, not passed); stderr overflow kills immediately; reads
//!   are bounded by the 4MiB IPC metadata cap; the worker build has a
//!   deadline and capped output.
//! * F-796-R4-04 — the child's EXIT STATUS is admitted (success
//!   required), worker/core versions are matched against the mirrored
//!   expectations, and the worker exe CONTENT hash feeds the report's
//!   toolchain identity (and through it, stale adjudication).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, Completion, MessageMatcher, ProbeStep};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{ArtifactRef, NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;
use serde::Deserialize;

use super::avm::Verdict;

/// Frozen caps: the 4MiB IPC metadata cap (+ framing slack) bounds the
/// response read; stderr drain has its own 1MiB cap and KILLS the tree
/// the moment it overflows (never a silent partial drain).
const READ_CAP: usize = (4 << 20) + (64 << 10);
const STDERR_CAP: usize = 1 << 20;
/// Request METADATA cap (§5.4: request/response metadata ≤4 MiB). The
/// pre-spawn check refuses an oversized request before any pipe write;
/// the worker enforces the same bound while reading.
const REQUEST_CAP: usize = 4 << 20;
/// Bounded drain grace after the child exits (the containment job has
/// already reaped descendants, so this is a backstop, not a wait).
const READER_GRACE: Duration = Duration::from_secs(2);
/// Per-SESSION parent wall: spawn + write + response + exit + reap.
const SESSION_WALL: Duration = Duration::from_secs(60);
/// Bounded build of the worker from source.
const BUILD_WALL: Duration = Duration::from_secs(300);
/// Mirrored worker identity expectations (F-796-R4-04): the versions
/// the real worker reports. Drift (a worker bump without a mirror
/// update) fails admission loudly — these are admission constants,
/// not documentation.
pub const EXPECTED_WORKER_VERSION: &str = "ac-sema-probe/0.1.0";
pub const EXPECTED_SEMA_CORE_VERSION: &str = "ac-sema-core/0.1.0";

/// Locate (and on first use build) the sema worker binary. Env override
/// first (trusted host configuration binds the tool identity); otherwise
/// find `experimental/ac-semantics` from this crate's source root and
/// build it release (bounded — see BUILD_WALL).
pub fn worker_binary() -> Result<PathBuf, String> {
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(locate_worker).clone()
}

fn locate_worker() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("SEMA_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "SEMA_CASE_WORKER points at a missing file: {}",
            env_path
        ));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "sema-probe worker not located: no experimental/ac-semantics above the crate and no SEMA_CASE_WORKER override".to_string(),
            );
        };
        let m = d
            .join("experimental")
            .join("ac-semantics")
            .join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    let mut child = Command::new("cargo")
        .args(["build", "--release", "--workspace", "--manifest-path"])
        .arg(&manifest)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn cargo for sema worker build: {}", e))?;
    let started = Instant::now();
    let mut err_pipe = child.stderr.take();
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(p) = err_pipe.as_mut() {
            let mut chunk = [0u8; 16384];
            loop {
                match p.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if buf.len() + n > STDERR_CAP {
                            break;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }
                }
            }
        }
        buf
    });
    let status = loop {
        if started.elapsed() > BUILD_WALL {
            let _ = child.kill();
            let _ = child.wait();
            return Err("sema worker build exceeded the bounded build wall".to_string());
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(format!("poll worker build: {e}")),
        }
    };
    let stderr_tail = err_reader.join().unwrap_or_default();
    if !status.success() {
        let tail = String::from_utf8_lossy(&stderr_tail);
        let tail: String = tail
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!("sema worker build failed:\n{}", tail));
    }
    let exe = manifest
        .parent()
        .unwrap()
        .join("target")
        .join("release")
        .join("sema-probe.exe");
    if exe.is_file() {
        Ok(exe)
    } else {
        Err(format!(
            "sema worker build produced no binary at {}",
            exe.display()
        ))
    }
}

/// Current sema toolchain identity (F-796-R4-04): the axes stale
/// adjudication binds probe rows to — the worker exe CONTENT hash and
/// the mirrored versions.
pub fn current_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    if let Ok(sha) = worker_exe_sha256() {
        m.insert("sema-worker-exe-sha256".to_string(), sha);
    }
    m.insert(
        "sema-worker-version".to_string(),
        EXPECTED_WORKER_VERSION.to_string(),
    );
    // F-796-R6-04: these two axes ARE part of the probe row's
    // consumption path (the report carries them); the current identity
    // must describe them too, or an unchanged report could never be
    // FRESH.
    m.insert("sema-protocol".to_string(), "semaworker/1".to_string());
    m.insert("recipe".to_string(), "sema/query-v1".to_string());
    m
}

/// The report's probe toolchain axes (F-796-R6-04): the SAME set the
/// stale checker declares applicable for probe rows.
fn probe_toolchain_axes(exe_sha: &str) -> Vec<(String, String)> {
    vec![
        ("sema-protocol".into(), "semaworker/1".into()),
        ("recipe".into(), "sema/query-v1".into()),
        ("sema-worker-exe-sha256".into(), exe_sha.to_string()),
        (
            "sema-worker-version".into(),
            EXPECTED_WORKER_VERSION.to_string(),
        ),
    ]
}

/// Content identity of the worker tool actually spawned (F-796-R4-04):
/// reported into the check's toolchain axes; stale adjudication binds
/// to it, so swapping the exe flips the row to STALE.
fn worker_exe_sha256() -> Result<String, String> {
    let exe = worker_binary()?;
    let bytes = std::fs::read(&exe).map_err(|e| format!("read worker exe: {e}"))?;
    Ok(hex(&sha256(&bytes)))
}

// ---- typed response mirror (admission; drift caught by suites) --------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerResponse {
    schema: String,
    #[serde(default)]
    nonce: String,
    #[serde(default)]
    request_hash: String,
    #[serde(default)]
    ok: Option<OkOut>,
    #[serde(default)]
    error: Option<WorkerError>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerError {
    code: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OkOut {
    recipe: String,
    steps: Vec<StepOut>,
    restart_pending: bool,
    worker_version: String,
    sema_core_version: String,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct StepOut {
    op: String,
    outcome: String,
    cache: String,
    compute_calls: u32,
    #[serde(default)]
    bindings: Vec<BindingOut>,
    #[serde(default)]
    diagnostics: Vec<DiagMirror>,
    #[serde(default)]
    stages: Vec<StageSpanMirror>,
    /// F-796-R6-05: the worker's realized session meter (proof the SAME
    /// meter ran the flow, never `None`).
    #[serde(default)]
    meter: MeterMirror,
}

#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct MeterMirror {
    #[serde(default)]
    scratch_peak_bytes: u64,
    #[serde(default)]
    hot_bytes: u64,
    #[serde(default)]
    rejections: u32,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct BindingOut {
    name: String,
    ty: String,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct DiagMirror {
    code: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct StageSpanMirror {
    name: String,
    wall_ns: u128,
}

/// TEST-ONLY fault injection (F-796-R4-03/R4-04 process-level
/// controls): a mutex-guarded set of CHILD environment variables (and
/// an optional session-wall override) applied only to spawned worker
/// processes while a test holds the lock. Cases can never reach this —
/// it is not part of the contract decode.
#[doc(hidden)]
pub struct TestFaults {
    pub child_envs: Vec<(String, String)>,
    pub session_wall_ms: Option<u64>,
}

#[doc(hidden)]
pub static TEST_FAULTS: std::sync::RwLock<Option<TestFaults>> = std::sync::RwLock::new(None);

fn active_faults() -> Option<(Vec<(String, String)>, Option<u64>)> {
    TEST_FAULTS.read().ok().and_then(|g| {
        g.as_ref()
            .map(|f| (f.child_envs.clone(), f.session_wall_ms))
    })
}

/// Windows Job-Object containment (the PLAN-788 controlled-execution
/// pattern, completed here — F-796-R6-03): the worker starts SUSPENDED,
/// is assigned to a kill-on-close job BEFORE its first instruction, and
/// is resumed; every descendant it spawns inherits the job. Closing the
/// job (after the child exits, or on a deadline kill) reaps the WHOLE
/// tree, so a descendant that inherited stdout/stderr can never wedge
/// the reader joins past the parent wall.
#[cfg(windows)]
pub(crate) mod containment {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;

    type Handle = *mut core::ffi::c_void;
    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const JOB_OBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION: i32 = 8;
    const TH32CS_SNAPTHREAD: u32 = 0x0000_0004;
    const THREAD_SUSPEND_RESUME: u32 = 0x0002;
    pub const CREATE_SUSPENDED: u32 = 0x0000_0004;

    #[repr(C)]
    struct JobBasicLimit {
        per_process_user_time: i64,
        per_job_user_time: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    struct JobExtendedLimit {
        basic: JobBasicLimit,
        io_info: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    /// Class 8 (JOBOBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION) — the
    /// reliable way to read a job's I/O transfer counters (the class-9
    /// extended-limit query fills the peak-memory fields but its io
    /// block reads zero on current Windows builds).
    #[repr(C)]
    struct JobBasicAccounting {
        total_user_time: i64,
        total_kernel_time: i64,
        this_period_total_user_time: i64,
        this_period_total_kernel_time: i64,
        total_page_fault_count: i32,
        total_processes: u32,
        active_processes: u32,
        total_terminated_processes: u32,
    }
    #[repr(C)]
    struct JobBasicAndIoAccounting {
        basic: JobBasicAccounting,
        io_info: IoCounters,
    }

    #[repr(C)]
    struct ThreadEntry {
        dw_size: u32,
        cnt_usage: u32,
        thread_id: u32,
        owner_process_id: u32,
        base_pri: i32,
        delta_pri: i32,
        flags: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateJobObjectW(attrs: *const core::ffi::c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(
            job: Handle,
            class: i32,
            info: *const core::ffi::c_void,
            len: u32,
        ) -> i32;
        fn QueryInformationJobObject(
            job: Handle,
            class: i32,
            info: *mut core::ffi::c_void,
            len: u32,
            return_len: *mut u32,
        ) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn CloseHandle(hObject: isize) -> i32;
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> Handle;
        fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn OpenThread(access: u32, inherit: i32, thread_id: u32) -> Handle;
        fn ResumeThread(thread: Handle) -> u32;
    }

    /// Post-exit resource facts of a contained worker tree, read from
    /// the Job object (F-797-R4-10 R8): the REAL process peak memory
    /// and the accumulated I/O transfer bytes — the per-request
    /// resource attribution the formal turnaround samples publish.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct JobStats {
        pub peak_process_memory_used: u64,
        pub peak_job_memory_used: u64,
        pub read_transfer_bytes: u64,
        pub write_transfer_bytes: u64,
    }

    pub struct JobGuard(Handle);

    impl JobGuard {
        /// Assign the (still suspended) child to a kill-on-close job.
        pub fn for_child(child: &Child) -> Option<JobGuard> {
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return None;
                }
                let mut info: JobExtendedLimit = std::mem::zeroed();
                info.basic.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if SetInformationJobObject(
                    job,
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JobExtendedLimit>() as u32,
                ) == 0
                {
                    CloseHandle(job as isize);
                    return None;
                }
                if AssignProcessToJobObject(job, child.as_raw_handle() as Handle) == 0 {
                    CloseHandle(job as isize);
                    return None;
                }
                Some(JobGuard(job))
            }
        }

        /// Read the job's peak-memory and I/O counters. Must be called
        /// while the job handle is still open (before the guard drops).
        pub fn stats(&self) -> Option<JobStats> {
            unsafe {
                let mut info: JobExtendedLimit = std::mem::zeroed();
                if QueryInformationJobObject(
                    self.0,
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                    &mut info as *mut _ as *mut core::ffi::c_void,
                    std::mem::size_of::<JobExtendedLimit>() as u32,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return None;
                }
                let mut io: JobBasicAndIoAccounting = std::mem::zeroed();
                if QueryInformationJobObject(
                    self.0,
                    JOB_OBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION,
                    &mut io as *mut _ as *mut core::ffi::c_void,
                    std::mem::size_of::<JobBasicAndIoAccounting>() as u32,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return None;
                }
                Some(JobStats {
                    peak_process_memory_used: info.peak_process_memory_used as u64,
                    peak_job_memory_used: info.peak_job_memory_used as u64,
                    read_transfer_bytes: io.io_info.read_transfer_count,
                    write_transfer_bytes: io.io_info.write_transfer_count,
                })
            }
        }
    }

    impl Drop for JobGuard {
        fn drop(&mut self) {
            // KILL_ON_JOB_CLOSE: dropping the last handle reaps every
            // process the job still holds (the whole worker tree).
            unsafe {
                CloseHandle(self.0 as isize);
            }
        }
    }

    /// Resume the suspended child's main thread (exactly one thread).
    pub fn resume_suspended_process(child: &Child) -> Result<(), &'static str> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot as isize == -1 {
                return Err("thread snapshot failed while resuming the worker");
            }
            let mut entry: ThreadEntry = std::mem::zeroed();
            entry.dw_size = std::mem::size_of::<ThreadEntry>() as u32;
            if Thread32First(snapshot, &mut entry) == 0 {
                CloseHandle(snapshot as isize);
                return Err("thread enumeration failed while resuming the worker");
            }
            let mut resumed = false;
            loop {
                if entry.owner_process_id == child.id() {
                    let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.thread_id);
                    if thread.is_null() {
                        CloseHandle(snapshot as isize);
                        return Err("failed to open the worker's main thread");
                    }
                    let prev = ResumeThread(thread);
                    CloseHandle(thread as isize);
                    if prev == u32::MAX {
                        CloseHandle(snapshot as isize);
                        return Err("failed to resume the worker's main thread");
                    }
                    resumed = true;
                }
                if Thread32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
            CloseHandle(snapshot as isize);
            if resumed {
                Ok(())
            } else {
                Err("worker main thread not found in the snapshot")
            }
        }
    }
}

#[cfg(not(windows))]
pub(crate) mod containment {
    use std::process::Child;
    pub const CREATE_SUSPENDED: u32 = 0;
    #[derive(Clone, Copy, Debug, Default)]
    pub struct JobStats {
        pub peak_process_memory_used: u64,
        pub peak_job_memory_used: u64,
        pub read_transfer_bytes: u64,
        pub write_transfer_bytes: u64,
    }
    pub struct JobGuard;
    impl JobGuard {
        pub fn for_child(_child: &Child) -> Option<JobGuard> {
            None
        }
        pub fn stats(&self) -> Option<JobStats> {
            None
        }
    }
    pub fn resume_suspended_process(_child: &Child) -> Result<(), &'static str> {
        Ok(())
    }
}

/// One bounded single-shot worker session with a FULL lifecycle
/// (F-796-R4-03 + F-796-R6-03): threaded request write; bounded stdout
/// read; stderr drained concurrently with an overflow KILL; the deadline
/// covers spawn→response→exit→reap; the child is Job-contained so a
/// descendant inheriting the pipes dies with it and the bounded reader
/// joins can never outlive the wall.
fn call_worker(request_json: &str, deadline: Duration) -> Result<WorkerResponse, String> {
    if request_json.len() > REQUEST_CAP {
        return Err(format!(
            "sema worker request exceeds the {} byte metadata cap ({} bytes)",
            REQUEST_CAP,
            request_json.len()
        ));
    }
    let exe = worker_binary()?;
    let started = Instant::now();
    let mut cmd = Command::new(&exe);
    cmd.arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let faults = active_faults();
    let wall = match &faults {
        Some((_, Some(ms))) => Duration::from_millis(*ms),
        _ => deadline,
    };
    if let Some((envs, _)) = &faults {
        for (k, v) in envs {
            cmd.env(k, v);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn sema worker ({}): {}", exe.display(), e))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained sema worker tree".to_string(),
            );
        }
    };
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("containment: {e}"));
    }
    let (write_tx, write_rx) = std::sync::mpsc::channel::<()>();
    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        let req = request_json.to_string();
        std::thread::spawn(move || {
            let _ = writeln!(stdin, r#"{{"op":"begin","schema":"semaworker/1"}}"#);
            let _ = writeln!(stdin, r#"{{"op":"request","request":{}}}"#, req);
            let _ = writeln!(stdin, r#"{{"op":"end"}}"#);
            let _ = stdin.flush();
            let _ = write_tx.send(());
        });
    }
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    let (err_tx, err_rx) = std::sync::mpsc::channel::<(bool, Vec<u8>)>();
    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let _out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        let mut done = false;
        while !done {
            match stdout_pipe.read(&mut chunk) {
                Ok(0) => done = true,
                Ok(n) => {
                    if buf.len() + n > READ_CAP {
                        let _ = tx.send(Err(buf.len() + n));
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.ends_with(b"\n") {
                        done = true;
                    }
                }
                Err(_) => done = true,
            }
        }
        let _ = tx.send(Ok(buf));
    });
    let _err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 16384];
        let mut overflow = false;
        loop {
            match stderr_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > STDERR_CAP {
                        overflow = true;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = err_tx.send((overflow, buf));
    });

    // deadline loop: response AND exit, killing on wall or stderr
    // overflow; a child exit closes the job (descendants reaped) BEFORE
    // the bounded drains, so inherited pipes cannot outlive the wall
    let mut received: Option<Result<Vec<u8>, usize>> = None;
    let mut stderr_overflow = false;
    let mut stderr_result: Option<(bool, Vec<u8>)> = None;
    let mut timed_out = false;
    let mut exit_status: Option<std::process::ExitStatus> = None;
    loop {
        if started.elapsed() > wall {
            timed_out = true;
            break;
        }
        if stderr_overflow {
            break;
        }
        if received.is_none() {
            match rx.try_recv() {
                Ok(v) => received = Some(v),
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => received = Some(Err(0)),
            }
        }
        if stderr_result.is_none() {
            match err_rx.try_recv() {
                Ok(v) => {
                    stderr_overflow = v.0;
                    stderr_result = Some(v);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
            }
        }
        if exit_status.is_none() {
            match child.try_wait() {
                Ok(st @ Some(_)) => exit_status = st,
                Ok(None) => {}
                Err(e) => return Err(format!("poll sema worker: {e}")),
            }
        }
        if exit_status.is_some() {
            // child gone: closing the job kills any descendant still
            // holding the inherited pipes, then the drains are bounded
            drop(job.take());
            if received.is_none() {
                received = match rx.recv_timeout(READER_GRACE) {
                    Ok(v) => Some(v),
                    Err(_) => Some(Err(0)),
                };
            }
            if stderr_result.is_none() {
                stderr_result = err_rx.recv_timeout(READER_GRACE).ok();
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    if timed_out || stderr_overflow {
        let _ = child.kill();
        drop(job.take());
        let _ = child.wait();
        let _ = write_rx.recv_timeout(READER_GRACE);
        let _ = rx.recv_timeout(READER_GRACE);
        let _ = err_rx.recv_timeout(READER_GRACE);
        return Err(if timed_out {
            "sema worker exceeded the parent wall (response-or-exit deadline)".to_string()
        } else {
            format!(
                "sema worker stderr exceeded the {}-byte cap — tree killed",
                STDERR_CAP
            )
        });
    }
    // normal exit path: the job is already dropped (descendants reaped)
    drop(job.take());
    if write_rx.recv_timeout(READER_GRACE).is_err() {
        return Err("sema worker request writer did not finish within the drain grace".into());
    }
    let stderr_tail = match stderr_result {
        Some((_, buf)) => buf,
        None => match err_rx.recv_timeout(READER_GRACE) {
            Ok((_, buf)) => buf,
            Err(_) => {
                return Err(
                    "sema worker stderr reader did not finish within the drain grace".into(),
                )
            }
        },
    };
    let exit = exit_status.ok_or("sema worker exit status not observed")?;
    let bytes = received
        .ok_or("sema worker closed without a response")?
        .map_err(|n| format!("sema worker response exceeded the read cap ({n} bytes)"))?;
    // F-796-R4-04: the child's exit status is ADMITTED — a worker that
    // answers correctly and then exits nonzero is a failure.
    if !exit.success() {
        return Err(format!(
            "sema worker exited with {} after responding",
            exit.code().unwrap_or(-1)
        ));
    }
    if !stderr_tail.is_empty() {
        let tail = String::from_utf8_lossy(&stderr_tail);
        let tail: String = tail
            .lines()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | ");
        return Err(format!("sema worker wrote to stderr: {}", tail));
    }
    let line = String::from_utf8_lossy(&bytes);
    let line = line.lines().next().unwrap_or("");
    serde_json::from_str(line).map_err(|e| format!("sema worker response malformed: {e}"))
}

// ---- the check ---------------------------------------------------------

/// Map a sema diagnostic code to the contract's probe stage set
/// (resolve for name-resolution failures, typecheck otherwise).
fn stage_of(code: &str) -> &'static str {
    match code {
        // F-796-R6-07: typed source-language parse diagnostics keep
        // their own stage instead of being folded into typecheck
        "source.syntax" | "source.lex" => "parse",
        "unknown-name" | "ambiguous-candidates" => "resolve",
        _ => "typecheck",
    }
}

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    // recipe×source admission: sema/query-v1 consumes `.at` sources
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "sema/query-v1 consumes .at sources; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    let sema = check
        .options
        .probe_sema
        .as_ref()
        .ok_or("probe check without a sema binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if check.options.probe_recipe.as_deref() != Some("sema/query-v1") {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    let store_root = artifact_dir.join("store");
    std::fs::create_dir_all(&store_root).map_err(|e| format!("probe store root: {}", e))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("probe artifact dir: {}", e))?;

    let mut source = input_bytes.clone();
    let mut sessions: Vec<Vec<&ProbeStep>> = Vec::new();
    let mut current: Vec<&ProbeStep> = Vec::new();
    for st in steps.iter() {
        let is_restart = matches!(st, ProbeStep::Restart);
        current.push(st);
        if is_restart {
            sessions.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        sessions.push(current);
    }

    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut reported_query_steps: Vec<StepOut> = Vec::new();
    let mut stage_spans: Vec<(String, u64)> = Vec::new();
    // session walls kept at µs resolution (ms is derived at emit); the
    // three segment vectors carry the consecutive prep/worker/admit
    // spans that tile each wall (F-796-R10-01)
    let mut session_walls_us: Vec<u64> = Vec::new();
    let mut session_prep_us: Vec<u64> = Vec::new();
    let mut session_worker_us: Vec<u64> = Vec::new();
    let mut session_admit_us: Vec<u64> = Vec::new();
    let mut infra_detail: Option<String> = None;
    let exe_sha = worker_exe_sha256()?;

    for (si, session) in sessions.iter().enumerate() {
        // F-796-R10-01: the session wall covers the COMPLETE edit-request
        // window — request preparation (steps/nonce/base64/JSON/hash), the
        // worker call, AND parent-side admission until the response's
        // current result is available to the parent. The prep/worker/admit
        // marks below are consecutive, so their segments tile the window
        // (prep+worker+admit ≈ wall) and no stage can silently escape it.
        let session_started = Instant::now();
        let steps_json: Vec<serde_json::Value> = session
            .iter()
            .map(|st| match st {
                ProbeStep::Query => serde_json::json!({"op": "query"}),
                ProbeStep::EditSource { from, to } => {
                    serde_json::json!({"op": "edit-source", "from": from, "to": to})
                }
                ProbeStep::Restart => serde_json::json!({"op": "restart"}),
                // Decode restricts `run` to the ac-av recipe; the sema
                // adapter never receives it (defensive explicit arm).
                ProbeStep::Run => serde_json::json!({"op": "run"}),
            })
            .collect();
        let nonce = hex(&sha256(
            format!("{}|{:?}|{}", std::process::id(), started, si).as_bytes(),
        ));
        let mut request = serde_json::json!({
            "recipe": "sema/query-v1",
            "nonce": nonce,
            "source_b64": b64(&source),
            "package": sema.package,
            "module": sema.module,
            "fn_name": sema.fn_name,
            "store_root": store_root.display().to_string().replace('/', "\\"),
            "steps": steps_json,
        });
        let request_hash = hex(&sha256(request.to_string().as_bytes()));
        request["request_hash"] = serde_json::json!(request_hash);
        let prep_us = session_started.elapsed().as_micros() as u64;

        let worker_started = Instant::now();
        let resp_result = call_worker(&request.to_string(), SESSION_WALL);
        let worker_us = worker_started.elapsed().as_micros() as u64;

        let admit_started = Instant::now();
        'admit: {
            let resp = match resp_result {
                Ok(r) => r,
                Err(e) => {
                    // infrastructure lifecycle failure — an ERROR outcome,
                    // never a language verdict (F-796-R4-02/03)
                    infra_detail = Some(e);
                    break 'admit;
                }
            };
            if resp.schema != "semaworker/1" {
                infra_detail = Some(format!("sema worker schema mismatch: {}", resp.schema));
                break 'admit;
            }
            if resp.nonce != nonce || resp.request_hash != request_hash {
                infra_detail = Some("sema worker nonce/request_hash echo mismatch".to_string());
                break 'admit;
            }
            let ok = match resp.ok.clone() {
                Some(ok) => ok,
                None => {
                    let e = resp.error.unwrap_or(WorkerError {
                        code: "missing".into(),
                        message: "neither ok nor error".into(),
                    });
                    infra_detail = Some(format!("sema worker error {}: {}", e.code, e.message));
                    break 'admit;
                }
            };
            if ok.recipe != "sema/query-v1" {
                infra_detail = Some(format!("sema worker recipe mismatch: {}", ok.recipe));
                break 'admit;
            }
            // F-796-R4-04: version admission against the mirrored
            // expectations — a wrong worker build fails loudly
            if ok.worker_version != EXPECTED_WORKER_VERSION {
                infra_detail = Some(format!(
                    "sema worker version `{}` does not match the admitted `{}`",
                    ok.worker_version, EXPECTED_WORKER_VERSION
                ));
                break 'admit;
            }
            if ok.sema_core_version != EXPECTED_SEMA_CORE_VERSION {
                infra_detail = Some(format!(
                    "sema core version `{}` does not match the admitted `{}`",
                    ok.sema_core_version, EXPECTED_SEMA_CORE_VERSION
                ));
                break 'admit;
            }
            for st in session.iter() {
                if let ProbeStep::EditSource { from, to } = st {
                    source = replace_once(&source, from, to)?;
                }
            }
            if matches!(session.last(), Some(ProbeStep::Restart)) && !ok.restart_pending {
                infra_detail = Some("restart step did not end the session as declared".to_string());
                break 'admit;
            }
            for s in ok.steps.iter().filter(|s| s.op == "query") {
                for sp in &s.stages {
                    stage_spans.push((sp.name.clone(), (sp.wall_ns / 1_000_000) as u64));
                }
            }
            reported_query_steps.extend(ok.steps.iter().filter(|s| s.op == "query").cloned());
        }
        // the wall ends when the response has passed parent-side
        // admission and its current result is available (F-796-R10-01)
        let admit_us = admit_started.elapsed().as_micros() as u64;
        session_walls_us.push(session_started.elapsed().as_micros() as u64);
        session_prep_us.push(prep_us);
        session_worker_us.push(worker_us);
        session_admit_us.push(admit_us);
        if infra_detail.is_some() {
            break;
        }
    }

    // ---- infrastructure short-circuit ------------------------------------
    if let Some(detail) = infra_detail {
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![],
            detail: Some(detail),
            artifacts: vec![],
            native: Some(NativeRecord {
                exit_code: None,
                stage_spans,
                cache_mode: Some("sema-store".into()),
                cache_hits: vec![],
                counters: vec![
                    ("query_steps".into(), reported_query_steps.len() as u32),
                    (
                        "compute_calls".into(),
                        reported_query_steps.iter().map(|s| s.compute_calls).sum(),
                    ),
                    // F-796-R6-05: the worker's REAL session meter is
                    // reported, so a `meter=None` regression is visible
                    // on the reported path itself
                    (
                        "meter_scratch_peak_bytes".into(),
                        reported_query_steps
                            .iter()
                            .map(|s| s.meter.scratch_peak_bytes)
                            .max()
                            .unwrap_or(0) as u32,
                    ),
                    (
                        "meter_rejections".into(),
                        reported_query_steps
                            .iter()
                            .map(|s| s.meter.rejections)
                            .sum(),
                    ),
                ]
                .into_iter()
                .chain(session_wall_counters(
                    &session_walls_us,
                    &session_prep_us,
                    &session_worker_us,
                    &session_admit_us,
                ))
                .collect::<Vec<(String, u32)>>(),
                toolchain: probe_toolchain_axes(&exe_sha),
                gaps: vec![],
            }),
        });
    }

    // ---- parent-side judgement (F-796-R4-02: outcome FIRST) -------------
    let expects = oracle.probe.as_deref().unwrap_or(&[]);
    if reported_query_steps.len() != expects.len() {
        mismatches.push(Mismatch {
            observation: "probe.steps".into(),
            kind: "text",
            expected: format!("{} query steps", expects.len()),
            actual: format!("{} query steps", reported_query_steps.len()),
            first_diff: None,
        });
    }
    for (i, (got, want)) in reported_query_steps.iter().zip(expects.iter()).enumerate() {
        // 1) typed completion admission
        match (&oracle.completion, got.outcome.as_str()) {
            (Completion::Success, "ok") => {
                if !got.diagnostics.is_empty() {
                    mismatches.push(Mismatch {
                        observation: format!("probe[{i}].outcome"),
                        kind: "text",
                        expected: "ok with zero diagnostics".into(),
                        actual: format!(
                            "ok carrying {} diagnostic(s) [{}]",
                            got.diagnostics.len(),
                            got.diagnostics
                                .first()
                                .map(|d| d.code.as_str())
                                .unwrap_or("?")
                        ),
                        first_diff: None,
                    });
                }
            }
            (
                Completion::Error {
                    stage,
                    code,
                    message,
                },
                "diags",
            ) => {
                let want_stage = stage.as_str();
                let matched = got.diagnostics.iter().any(|d| {
                    stage_of(&d.code) == want_stage
                        && code.as_deref().map(|c| c == d.code).unwrap_or(true)
                        && match message {
                            Some(MessageMatcher::Exact(m)) => &d.message == m,
                            Some(MessageMatcher::Contains(m)) => d.message.contains(m.as_str()),
                            None => true,
                        }
                });
                if !matched {
                    mismatches.push(Mismatch {
                        observation: format!("probe[{i}].diagnostics"),
                        kind: "message",
                        expected: format!(
                            "a {}-stage diagnostic matching code={:?} message={:?}",
                            want_stage,
                            code.as_deref().unwrap_or("-"),
                            message
                        ),
                        actual: got
                            .diagnostics
                            .iter()
                            .map(|d| format!("{}: {}", stage_of(&d.code), d.message))
                            .collect::<Vec<_>>()
                            .join(" | "),
                        first_diff: None,
                    });
                }
            }
            (Completion::Success, other) => {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].outcome"),
                    kind: "text",
                    expected: "success".into(),
                    actual: format!("{other} (cache={})", got.cache),
                    first_diff: None,
                });
            }
            (Completion::Error { .. }, other) => {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].outcome"),
                    kind: "text",
                    expected: "a declared error completion".into(),
                    actual: format!("{other} with zero matching diagnostics"),
                    first_diff: None,
                });
            }
        }
        // 2) structured oracles (coarse cache vocabulary: published-* →
        // published; hit/reprojected verbatim)
        if let Some(want_cache) = &want.cache {
            let coarse = match got.cache.as_str() {
                "published-miss" | "published-stale" => "published".to_string(),
                other => other.to_string(),
            };
            if coarse != *want_cache {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].cache"),
                    kind: "text",
                    expected: want_cache.clone(),
                    actual: got.cache.clone(),
                    first_diff: None,
                });
            }
        }
        if let Some(want_calls) = want.compute_calls {
            if got.compute_calls != want_calls {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].compute_calls"),
                    kind: "text",
                    expected: want_calls.to_string(),
                    actual: got.compute_calls.to_string(),
                    first_diff: None,
                });
            }
        }
        for stage in &want.stages_include {
            if !got.stages.iter().any(|s| &s.name == stage) {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].stages"),
                    kind: "text",
                    expected: format!("includes {stage}"),
                    actual: got
                        .stages
                        .iter()
                        .map(|s| s.name.clone())
                        .collect::<Vec<_>>()
                        .join(","),
                    first_diff: None,
                });
            }
        }
        if let (Some(bytes), Some(_)) = (&want.bindings_bytes, &want.bindings_sha256) {
            if let Err(diff) = compare_bindings(bytes, &got.bindings) {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].bindings"),
                    kind: "text",
                    expected: diff.expected,
                    actual: diff.actual,
                    first_diff: None,
                });
            }
        }
    }

    // persist the structured receipt (audit trail under the artifact dir)
    let receipt = serde_json::json!({
        "recipe": "sema/query-v1",
        "query_steps": reported_query_steps,
        "store_root": store_root,
        "toolchain": {
            "sema-protocol": "semaworker/1",
            "recipe": "sema/query-v1",
            "sema-worker-exe-sha256": exe_sha,
            "sema-worker-version": EXPECTED_WORKER_VERSION,
        },
    });
    let receipt_path = artifact_dir.join("probe-receipt.json");
    std::fs::write(
        &receipt_path,
        serde_json::to_string_pretty(&receipt).unwrap(),
    )
    .map_err(|e| format!("receipt write: {e}"))?;
    let receipt_sha = hex(&sha256(&std::fs::read(&receipt_path).unwrap_or_default()));

    let artifacts = vec![ArtifactRef {
        logical_path: "probe-receipt.json".into(),
        stored_path: receipt_path.display().to_string(),
        sha256: receipt_sha,
        bytes: None,
    }];
    let native = NativeRecord {
        exit_code: None,
        stage_spans,
        cache_mode: Some("sema-store".into()),
        cache_hits: reported_query_steps
            .iter()
            .map(|s| (s.cache.clone(), s.cache.clone()))
            .collect(),
        counters: vec![
            ("query_steps".into(), reported_query_steps.len() as u32),
            (
                "compute_calls".into(),
                reported_query_steps.iter().map(|s| s.compute_calls).sum(),
            ),
            (
                "meter_scratch_peak_bytes".into(),
                reported_query_steps
                    .iter()
                    .map(|s| s.meter.scratch_peak_bytes)
                    .max()
                    .unwrap_or(0) as u32,
            ),
            (
                "meter_rejections".into(),
                reported_query_steps
                    .iter()
                    .map(|s| s.meter.rejections)
                    .sum(),
            ),
        ]
        .into_iter()
        .chain(session_wall_counters(
            &session_walls_us,
            &session_prep_us,
            &session_worker_us,
            &session_admit_us,
        ))
        .collect::<Vec<(String, u32)>>(),
        toolchain: probe_toolchain_axes(&exe_sha),
        gaps: vec![],
    };
    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "sema/query-v1: {} query step(s) judged; receipt probe-receipt.json",
            reported_query_steps.len()
        )),
        artifacts,
        native: Some(native),
    })
}

/// Session wall + consecutive segment counters (F-796-R10-01): the
/// wall is emitted at ms (gate resolution) AND µs, together with the
/// prep/worker/admit segments that tile it — `session_wall_us_i ==
/// session_prep_us_i + session_worker_us_i + session_admit_us_i` up to
/// µs truncation, so a window that starts after request preparation or
/// ends before parent-side admission is detectable from the report.
fn session_wall_counters<'a>(
    walls_us: &'a [u64],
    prep_us: &'a [u64],
    worker_us: &'a [u64],
    admit_us: &'a [u64],
) -> impl Iterator<Item = (String, u32)> + 'a {
    let walls = walls_us.iter().enumerate().flat_map(|(i, us)| {
        [
            (format!("session_wall_ms_{i}"), (*us / 1000) as u32),
            (format!("session_wall_us_{i}"), *us as u32),
        ]
    });
    let segments = [
        ("prep", prep_us),
        ("worker", worker_us),
        ("admit", admit_us),
    ]
    .into_iter()
    .flat_map(|(name, seg)| {
        seg.iter()
            .enumerate()
            .map(move |(i, us)| (format!("session_{name}_us_{i}"), *us as u32))
    });
    walls.chain(segments)
}

fn replace_once(src: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(src).map_err(|e| format!("source not utf-8: {e}"))?;
    let hits = s.matches(from).count();
    if hits != 1 {
        return Err(format!(
            "edit-source literal matched {hits} times (expected exactly once)"
        ));
    }
    Ok(s.replacen(from, to, 1).into_bytes())
}

struct Diff {
    expected: String,
    actual: String,
}

/// Bindings oracle: JSON {"bindings":[{name,ty},…]} — order-sensitive
/// list compare (program order is the observable).
fn compare_bindings(oracle_bytes: &[u8], got: &[BindingOut]) -> Result<(), Diff> {
    #[derive(Deserialize)]
    struct Oracle {
        bindings: Vec<OracleBinding>,
    }
    #[derive(Deserialize)]
    struct OracleBinding {
        name: String,
        ty: String,
    }
    let want: Oracle = serde_json::from_slice(oracle_bytes).map_err(|e| Diff {
        expected: format!("valid bindings oracle json: {e}"),
        actual: String::new(),
    })?;
    let want_pairs: Vec<(String, String)> =
        want.bindings.into_iter().map(|b| (b.name, b.ty)).collect();
    let got_pairs: Vec<(String, String)> =
        got.iter().map(|b| (b.name.clone(), b.ty.clone())).collect();
    if want_pairs != got_pairs {
        return Err(Diff {
            expected: format!("{want_pairs:?}"),
            actual: format!("{got_pairs:?}"),
        });
    }
    Ok(())
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
