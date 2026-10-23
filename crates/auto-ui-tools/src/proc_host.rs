//! PLAN-815 T-05 hosted process/IO plumbing for the tool host (R1 rework).
//!
//! The migrated run paths (vue / vm / rust) keep their legacy behavior when
//! no [`RunIo`] is installed: children inherit stdio, steps `println!`.
//! When the tool host installs a [`RunIo`] for a work operation, the same
//! code paths instead:
//!
//! - spawn children through **`auto-tool-execution`** (PLAN-816 T-02's
//!   single owned-process facility): Windows Job Object / Unix process
//!   group ⇒ cancel terminates the **whole owned tree**, grandchildren
//!   included (F-815-T05-R1-01), reap-then-return semantics, and bounded
//!   joins for the stdout forwarder threads;
//! - forward child stdout to the host log stream (URL sniffing announces
//!   the launch spec) with **piped stdin owned by a cancellable writer** —
//!   `Input` bytes are queued to a dedicated writer, and the blocking
//!   write happens OFF the protocol loop, so cancel/describe stay
//!   responsive no matter how un-readingly the child treats its stdin
//!   (F-815-T05-R1-02);
//! - emit progress/launched events through host-installed hooks (the host
//!   enforces the per-op progress budget before anything reaches the
//!   control stream);
//! - invoke host-registered terminate extras (the VM main-thread iced loop
//!   exit request) on cancel/EOF (F-815-T05-R1-03 wiring).
//!
//! Protocol-agnostic by construction: the hooks are plain closures, so this
//! module never depends on `auto-tool-protocol`.

use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use auto_tool_execution::{ExecutionSpec, StdioPolicy, TreeScope};

/// Poll interval for cancel-aware waits.
const WAIT_POLL: Duration = Duration::from_millis(50);
/// Bounded grace for tree reaping and forwarder joins (cancel path).
const TEARDOWN_GRACE: Duration = Duration::from_secs(10);
/// Natural-drain window before the join forces the reader stop flag: once
/// the child (tree) is finished the pipes close and a reader ends ITSELF
/// with a natural EOF within a poll tick — preempting it with the stop
/// flag would misclassify a normal completion as a teardown failure and
/// drop trailing output (observed on fast-exiting build wrappers,
/// PLAN-815 R4 replay).
const READER_DRAIN_GRACE: Duration = Duration::from_secs(2);
/// Bounded capacity of the per-op input queue (backpressure is a named
/// rejection, never a blocked control loop).
pub const INPUT_QUEUE_DEPTH: usize = 128;

/// How a hosted run flow ended (the worker maps this onto the protocol
/// report; legacy wrappers flatten it back to `AutoResult`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowEnd {
    /// The run path returned on its own.
    Completed,
    /// A cancel (or progress-budget abort) stopped the flow; owned children
    /// were killed.
    Cancelled,
}

/// How a forwarder thread ended (F01/R3): distinguishes a natural pipe EOF
/// from a teardown-forced stop and from a real read error — all three are
/// separately named at join time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReaderEnd {
    /// Still running.
    Running,
    /// The pipe reached EOF on its own (all write ends closed).
    NaturalEof,
    /// The stop flag ended the loop before any natural EOF.
    Stopped,
    /// The read failed with an I/O error (output may be truncated).
    IoError(String),
}

/// Sentinel error text a hosted flow bails with at a cancellation
/// checkpoint.
pub const CANCELLED_ERR: &str = "operation cancelled";

/// True when the error string is the cancellation sentinel.
pub fn cancelled_err(err: &str) -> bool {
    err == CANCELLED_ERR
}

/// True when a hosted operation was cancelled (false when unhosted).
pub fn cancelled() -> bool {
    current().map(|io| io.cancelled()).unwrap_or(false)
}

/// Bail-at-checkpoint helper for hosted flows: `Err(CANCELLED_ERR)` once
/// cancelled, `Ok(())` otherwise.
pub fn check_cancel() -> Result<(), String> {
    if cancelled() {
        Err(CANCELLED_ERR.to_string())
    } else {
        Ok(())
    }
}

/// A hosted child spawn request (argv-based; the facility owns command
/// construction — no ad-hoc `Command` sites, R1-01 discipline).
#[derive(Debug, Clone, Default)]
pub struct TrackedSpawn {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(String, String)>,
    /// Keys to REMOVE from the inherited environment before `env` is
    /// applied (PLAN-815 R4-01): a per-request child context must not
    /// leak a stale same-named variable from the host process env.
    pub env_remove: Vec<String>,
    /// Windows: resolve via `cmd /C` (mirrors the legacy pkg.rs shim for
    /// .cmd/.bat package managers).
    pub windows_cmd_shim: bool,
    /// Also PIPE the child's stderr (forwarded to the host log stream +
    /// URL-sniffed) instead of inheriting it. Used by the internal VM
    /// session child: its real MCP endpoint banner lands on stderr
    /// (R2-03's accurate-URL Launched source).
    pub pipe_stderr: bool,
    /// Keep stdout for the CALLER to drain (no forwarder thread is
    /// started) — for hosted external commands whose output the run path
    /// itself parses (e.g. `adb devices`). R4-05: still an owned tree.
    pub capture_stdout: bool,
}

impl TrackedSpawn {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            cwd: None,
            env: Vec::new(),
            env_remove: Vec::new(),
            windows_cmd_shim: false,
            pipe_stderr: false,
            capture_stdout: false,
        }
    }

    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    pub fn without_env(mut self, key: &str) -> Self {
        self.env_remove.push(key.to_string());
        self
    }

    fn to_spec(&self) -> ExecutionSpec {
        let mut argv: Vec<std::ffi::OsString> = Vec::with_capacity(self.args.len() + 1);
        argv.push(self.program.as_str().into());
        argv.extend(self.args.iter().map(|a| a.as_str().into()));
        ExecutionSpec {
            argv,
            cwd: self.cwd.clone(),
            env: self
                .env
                .iter()
                .map(|(k, v)| (k.as_str().into(), v.as_str().into()))
                .collect(),
            env_remove: self.env_remove.iter().map(|k| k.as_str().into()).collect(),
            stdin: StdioPolicy::Piped,
            stdout: StdioPolicy::Piped,
            // stderr inherits the process stderr = the log stream after the
            // bin's stdio split (nothing a child writes can reach the
            // control stream and no child can consume the framed stdin),
            // unless the caller explicitly pipes it for forwarding+sniffing
            // (the internal VM session child — R2-03's URL source).
            stderr: if self.pipe_stderr {
                StdioPolicy::Piped
            } else {
                StdioPolicy::Inherit
            },
            stdin_data: None,
            deadline: None,
            tree_scope: TreeScope::ProcessTree,
            windows_cmd_shim: self.windows_cmd_shim,
            // F-816-B-R1-01：结构体加法字段（spawn_detached 的 daemon
            // 创建语义）同步本全字段构造点——owned 子进程不消费该标志。
            windows_creation_flags: 0,
        }
    }
}

/// One owned child (tree) of a hosted operation.
pub struct TrackedChild {
    proc: Mutex<Option<auto_tool_execution::OwnedProcess>>,
    stdin: Mutex<Option<auto_tool_execution::StdinPipe>>,
    /// stdout/stderr forwarder threads with their end-cause slots (F01/R3:
    /// NaturalEof / Stopped / IoError — a forced stop is NEVER reported as
    /// a natural EOF; joined with a real-finish oracle at teardown).
    readers: Mutex<Vec<(std::thread::JoinHandle<()>, Arc<Mutex<ReaderEnd>>)>>,
    /// When the spawn requested `capture_stdout`: the undrained pipe,
    /// handed to the caller (no forwarder was started for it).
    captured_stdout: Mutex<Option<std::process::ChildStdout>>,
    /// Stop flag for the input writer's cancellable writes.
    input_stop: Arc<AtomicBool>,
    /// Stop flag for the forwarder's polling read loop (F01/R2: the reader
    /// must be genuinely stoppable even when a NON-owned process holds the
    /// pipe's write end — the loop polls availability instead of blocking).
    reader_stop: Arc<AtomicBool>,
    /// Caller-assigned role (`install`, `dev-server`, `api-server`, …) —
    /// used for honest diagnostics only.
    pub label: String,
    pub id: u32,
}

impl TrackedChild {
    /// Terminate the owned tree (Windows Job / Unix process group) and
    /// unblock any writer parked on this child's stdin. Only issues the
    /// termination — reaping is the wait/cancel path.
    pub fn kill(&self) {
        if let Some(proc) = self.proc.lock().unwrap().as_mut() {
            let _ = proc.terminate();
        }
        self.input_stop.store(true, Ordering::SeqCst);
        self.reader_stop.store(true, Ordering::SeqCst);
        if let Some(stdin) = self.stdin.lock().unwrap().as_ref() {
            stdin.cancel_blocked_write();
        }
    }

    /// Close the child's stdin (drops the pipe write end — the child
    /// observes EOF). Honest lifecycle knob for bounded interactive runs.
    pub fn close_stdin(&self) {
        self.input_stop.store(true, Ordering::SeqCst);
        *self.stdin.lock().unwrap() = None;
    }

    pub fn try_wait(&self) -> std::io::Result<Option<ExitStatus>> {
        match self.proc.lock().unwrap().as_mut() {
            Some(proc) => proc.try_wait(),
            None => Ok(None),
        }
    }

    /// Terminate + bounded reap (facility `cancel_within`) + bounded reader
    /// joins. Called on the cancel path.
    fn reap(&self) -> std::io::Result<ExitStatus> {
        let status = match self.proc.lock().unwrap().as_mut() {
            Some(proc) => proc.cancel_within(TEARDOWN_GRACE),
            None => return Err(std::io::Error::other("tracked child already reaped")),
        }?;
        if let Err(reason) = self.join_readers() {
            note_teardown_failure(&reason);
            return Err(std::io::Error::other(reason));
        }
        Ok(status)
    }

    /// Bounded join of the forwarder threads with a REAL finish oracle
    /// (F01/R2). Two phases (PLAN-815 R4): first a SHORT natural-drain
    /// window — the join only runs after the child (tree) finished, and a
    /// closing pipe ends a reader on its own with a natural EOF within a
    /// poll tick; preempting it with the stop flag would misclassify a
    /// normal completion as a teardown stop and drop trailing output.
    /// Readers still running after the drain (a NON-owned process holds
    /// the write end; no EOF can ever arrive) are then SIGNALLED with the
    /// stop flag — the polling readers observe it within one tick and
    /// truly stop; a reader stopped WITHOUT a natural pipe EOF is a NAMED
    /// teardown failure (disclosed on the final report); a bound overrun
    /// is likewise named — and the thread is NOT joined past the bound
    /// (the join itself must stay bounded; an overrun thread is disclosed,
    /// not silently detached as if it had stopped, R1-01).
    fn join_readers(&self) -> Result<(), String> {
        let readers: Vec<_> = self.readers.lock().unwrap().drain(..).collect();
        let mut failures = Vec::new();
        for (handle, reached_eof) in readers {
            // Phase 1: natural drain (no stop signal).
            let drain = std::time::Instant::now() + READER_DRAIN_GRACE;
            while !handle.is_finished() {
                if std::time::Instant::now() > drain {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            // Phase 2: force the stop flag for stragglers.
            let mut overran = false;
            if !handle.is_finished() {
                self.reader_stop.store(true, Ordering::SeqCst);
                let deadline = std::time::Instant::now() + TEARDOWN_GRACE;
                while !handle.is_finished() {
                    if std::time::Instant::now() > deadline {
                        failures.push(format!(
                            "reader thread for `{}` did not stop within {TEARDOWN_GRACE:?} of teardown",
                            self.label
                        ));
                        overran = true;
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            if overran {
                // The stop flag stays set; the polling loop ends the thread
                // on its next tick. The join stays bounded — never block
                // the teardown path on a wedged reader.
                continue;
            }
            let _ = handle.join();
            if !failures.is_empty() {
                continue;
            }
            let end = reached_eof.lock().unwrap().clone();
            match end {
                ReaderEnd::NaturalEof => {}
                ReaderEnd::Running => failures.push(format!(
                    "reader for `{}` did not report an end cause",
                    self.label
                )),
                ReaderEnd::Stopped => failures.push(format!(
                    "reader for `{}` stopped by teardown without a natural pipe EOF (a process outside the owned tree may still hold the pipe)",
                    self.label
                )),
                ReaderEnd::IoError(reason) => failures.push(format!(
                    "reader for `{}` ended with an I/O error (output may be truncated): {reason}",
                    self.label
                )),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }

    /// Write bytes into the child's stdin with a cancellable write
    /// (F-815-T05-R1-02: called only from the dedicated input writer,
    /// never the protocol loop). `Ok(())` = all bytes entered the pipe;
    /// cancelled writes surface the sentinel text.
    pub fn write_input_stoppable(&self, data: &[u8]) -> std::io::Result<()> {
        use auto_tool_execution::StoppableWrite;
        let mut guard = self.stdin.lock().unwrap();
        let Some(stdin) = guard.as_mut() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "child stdin already closed",
            ));
        };
        let mut written = 0usize;
        while written < data.len() {
            let n = stdin.write_stoppable(&data[written..], &self.input_stop)?;
            if n == 0 {
                if self.input_stop.load(Ordering::SeqCst) {
                    return Err(std::io::Error::other(CANCELLED_ERR));
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "child stdin accepted zero bytes",
                ));
            }
            written += n;
        }
        Ok(())
        // No flush needed: StoppableWrite goes straight to the OS pipe
        // handle (no std-layer buffering is introduced here).
    }
}

/// Per-work-operation hosted IO state. Installed via [`install`] while the
/// operation executes (single work operation at a time — the host enforces
/// this before installing) and removed by [`uninstall`].
pub struct RunIo {
    cancelled: AtomicBool,
    children: Mutex<Vec<Arc<TrackedChild>>>,
    foreground: Mutex<Option<Arc<TrackedChild>>>,
    /// Named teardown failure (F01/R2): set when a reader (or the input
    /// writer) could not be stopped within the bound — disclosed on the
    /// final report instead of pretending a clean close.
    pub teardown_error: Mutex<Option<String>>,
    /// The input writer thread (joined with a bound at uninstall — the
    /// writer must truly stop before the operation completes).
    pub input_writer: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Per-request context for the VM session child (PLAN-815 R4-01): the
    /// EXACT env entries derived from the CURRENT request only — the
    /// `on_vm_main` spawn applies them to the child (plus removal of the
    /// internal context keys from the inherited env). Set by the run flow
    /// right before the session child spawns; a request without an
    /// external backend / theme simply carries no entries.
    pub vm_session_env: Mutex<Vec<(String, String)>>,
    /// First `VM_SESSION_FATAL: <reason>` line the vm-session child wrote
    /// to its stderr (R4-02): the structured, named failure reason for a
    /// child that exited fatally (e.g. required external backend cdylib
    /// missing) — surfaced into the final RunReport error.
    pub vm_session_fatal: Mutex<Option<String>>,
    /// `(done, total, message)` progress hook (host enforces budgets).
    pub on_progress: Mutex<Option<Box<dyn Fn(u64, Option<u64>, &str) + Send>>>,
    /// Launched-spec hook (service URL).
    pub on_launched: Mutex<Option<Box<dyn Fn(&str) + Send>>>,
    /// Child stdout chunk hook (host log stream).
    pub on_log_chunk: Mutex<Option<Box<dyn Fn(&[u8]) + Send>>>,
    /// Input enqueue hook: `(request_id, target_id, data)` — queues the
    /// blocking write onto the dedicated writer (F-815-T05-R1-02).
    /// Ok(()) = queued (receipt comes from the writer); Err(reason) =
    /// immediate named rejection.
    pub on_input_enqueue:
        Mutex<Option<Box<dyn Fn(u64, u64, Vec<u8>) -> Result<(), String> + Send>>>,
    /// VM UI entry executor (the iced run must happen on the OS main
    /// thread; the host installs a hand-off here). Unhosted callers run
    /// the entry on their own (main) thread — legacy behavior. The result
    /// type matches `auto_ui_runtime::run_file_dynamic_ui` (core
    /// `AutoResult`).
    pub on_vm_main: Mutex<
        Option<Arc<dyn Fn(&str, Option<&str>) -> auto_lang_core::AutoResult<String> + Send + Sync>>,
    >,
}

impl RunIo {
    fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            children: Mutex::new(Vec::new()),
            foreground: Mutex::new(None),
            teardown_error: Mutex::new(None),
            input_writer: Mutex::new(None),
            vm_session_env: Mutex::new(Vec::new()),
            vm_session_fatal: Mutex::new(None),
            on_progress: Mutex::new(None),
            on_launched: Mutex::new(None),
            on_log_chunk: Mutex::new(None),
            on_input_enqueue: Mutex::new(None),
            on_vm_main: Mutex::new(None),
        }
    }

    /// Cooperative cancellation flag (checkpoints + wait loops consult it).
    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Signal cancellation: kill every owned child TREE, stop in-flight
    /// input writes, and invoke the terminate extras (VM iced exit).
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let children = self.children.lock().unwrap().clone();
        for child in children.iter() {
            child.kill();
        }
    }

    fn register(&self, tracked: Arc<TrackedChild>) {
        self.children.lock().unwrap().push(Arc::clone(&tracked));
        *self.foreground.lock().unwrap() = Some(tracked);
    }

    /// The current foreground child (Input forwarding target).
    fn foreground(&self) -> Option<Arc<TrackedChild>> {
        self.foreground.lock().unwrap().clone()
    }
}

static CURRENT: OnceLock<Mutex<Option<Arc<RunIo>>>> = OnceLock::new();

fn slot() -> &'static Mutex<Option<Arc<RunIo>>> {
    CURRENT.get_or_init(|| Mutex::new(None))
}

/// Install the hosted IO for a work operation. Fails (false) when a work
/// operation is already in flight — the host answers that case with a named
/// busy error instead of racing two flows' process-global state.
pub fn install() -> bool {
    let mut guard = slot().lock().unwrap();
    if guard.is_some() {
        return false;
    }
    *guard = Some(Arc::new(RunIo::new()));
    true
}

/// Remove the hosted IO, killing any still-live owned trees (leak guard)
/// and dropping the hook slots: the closures capture strong Arcs (to this
/// RunIo and, via the host, to the shared channel state), so keeping them
/// installed after the operation ends would leak the whole cycle and the
/// tool host would never observe channel closure.
pub fn uninstall() {
    let io = slot().lock().unwrap().take();
    if let Some(io) = io {
        io.cancel();
        *io.on_progress.lock().unwrap() = None;
        *io.on_launched.lock().unwrap() = None;
        *io.on_log_chunk.lock().unwrap() = None;
        *io.on_input_enqueue.lock().unwrap() = None; // writer drains & exits
        *io.on_vm_main.lock().unwrap() = None;
        // The input writer must truly stop before the operation completes
        // (F01/R2 oracle): bounded wait; on overrun the failure is named
        // and the join itself stays bounded (R1-01 — same rule as the
        // readers; the stop signal remains set, the writer ends on its
        // next cancellable-write tick).
        let writer = io.input_writer.lock().unwrap().take();
        if let Some(handle) = writer {
            let deadline = std::time::Instant::now() + TEARDOWN_GRACE;
            let mut overran = false;
            while !handle.is_finished() {
                if std::time::Instant::now() > deadline {
                    overran = true;
                    let mut slot = io.teardown_error.lock().unwrap();
                    if slot.is_none() {
                        *slot = Some(
                            "input writer thread did not stop within the teardown bound"
                                .to_string(),
                        );
                    }
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            if !overran {
                let _ = handle.join();
            }
        }
    }
}

/// The currently installed hosted IO, when any.
pub fn current() -> Option<Arc<RunIo>> {
    slot().lock().unwrap().clone()
}

/// The host-installed VM UI main-thread executor, when a hosted operation
/// wants to run an iced entry (None when unhosted — the caller runs it on
/// its own thread, which is the OS main thread in the legacy CLI).
pub fn vm_main_hook(
) -> Option<Arc<dyn Fn(&str, Option<&str>) -> auto_lang_core::AutoResult<String> + Send + Sync>> {
    let io = current()?;
    let guard = io.on_vm_main.lock().unwrap();
    guard.clone()
}

/// Queue input bytes for the foreground child (F-815-T05-R1-02). The
/// blocking write happens on the dedicated writer; this call only enqueues
/// (bounded) and never blocks the protocol loop.
pub fn enqueue_input(request_id: u64, target_id: u64, data: Vec<u8>) -> Result<(), String> {
    let io = current().ok_or_else(|| "no work operation in flight".to_string())?;
    let guard = io.on_input_enqueue.lock().unwrap();
    match guard.as_ref() {
        Some(enqueue) => enqueue(request_id, target_id, data),
        None => Err("input channel unavailable for this operation".to_string()),
    }
}

/// The foreground child's cancellable write — called by the host's input
/// writer thread only (never the protocol loop).
pub fn write_input_stoppable(data: &[u8]) -> Result<(), String> {
    let io = current().ok_or_else(|| "no work operation in flight".to_string())?;
    match io.foreground() {
        Some(child) => child.write_input_stoppable(data).map_err(|e| e.to_string()),
        None => Err("operation has no interactive child process".to_string()),
    }
}

/// Emit a progress event for the hosted operation (no-op when unhosted or
/// no hook installed).
pub fn progress(done: u64, total: Option<u64>, message: &str) {
    let Some(io) = current() else { return };
    let guard = io.on_progress.lock().unwrap();
    if let Some(hook) = guard.as_ref() {
        hook(done, total, message);
    }
}

/// Announce a service URL (no-op when unhosted / no hook).
pub fn launched(url: &str) {
    let Some(io) = current() else { return };
    let guard = io.on_launched.lock().unwrap();
    if let Some(hook) = guard.as_ref() {
        hook(url);
    }
}

/// Forward a child stdout chunk to the host log stream (no-op unhosted).
fn log_chunk(bytes: &[u8]) {
    let Some(io) = current() else { return };
    let guard = io.on_log_chunk.lock().unwrap();
    if let Some(hook) = guard.as_ref() {
        hook(bytes);
    }
}

/// Spawn `spec` as an owned tree of the hosted operation and start its
/// stdout forwarder. The caller must have [`install`]ed a [`RunIo`].
pub fn spawn_tracked(spec: TrackedSpawn, label: &str) -> std::io::Result<Arc<TrackedChild>> {
    let io = current().expect("spawn_tracked called without installed RunIo");
    let mut proc = auto_tool_execution::spawn(&spec.to_spec())
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let pid = proc.id();
    let child_stdin = proc
        .take_stdin()
        .map_err(|e| std::io::Error::other(format!("take stdin failed: {e}")))?;
    let child_stdout = proc.take_stdout();
    let child_stderr = if spec.pipe_stderr {
        proc.take_stderr()
    } else {
        None
    };
    let tracked = Arc::new(TrackedChild {
        proc: Mutex::new(Some(proc)),
        stdin: Mutex::new(Some(child_stdin)),
        readers: Mutex::new(Vec::new()),
        captured_stdout: Mutex::new(None),
        input_stop: Arc::new(AtomicBool::new(false)),
        reader_stop: Arc::new(AtomicBool::new(false)),
        label: label.to_string(),
        id: pid,
    });
    io.register(Arc::clone(&tracked));
    if spec.capture_stdout {
        // Caller-drained stdout: hand the pipe over instead of starting a
        // forwarder (the caller drains it before/alongside wait_tracked).
        *tracked.captured_stdout.lock().unwrap() = child_stdout;
        return Ok(tracked);
    }
    if let Some(stdout) = child_stdout {
        let forward_label = label.to_string();
        let reader_stop = Arc::clone(&tracked.reader_stop);
        let end_slot = Arc::new(Mutex::new(ReaderEnd::Running));
        let slot = Arc::clone(&end_slot);
        let handle = std::thread::Builder::new()
            .name(format!("child-out:{forward_label}"))
            .spawn(move || forward_stdout(stdout, &forward_label, &reader_stop, &end_slot))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        tracked.readers.lock().unwrap().push((handle, slot));
    }
    if let Some(stderr) = child_stderr {
        let forward_label = label.to_string();
        let reader_stop = Arc::clone(&tracked.reader_stop);
        let end_slot = Arc::new(Mutex::new(ReaderEnd::Running));
        let slot = Arc::clone(&end_slot);
        let handle = std::thread::Builder::new()
            .name(format!("child-err:{forward_label}"))
            .spawn(move || forward_stderr(stderr, &forward_label, &reader_stop, &end_slot))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        tracked.readers.lock().unwrap().push((handle, slot));
    }
    Ok(tracked)
}

/// Forward a child's piped stdout to the host log stream, sniffing the
/// first announced service URL on dev-server/api-server/vm-session
/// children. F01/R2: the loop POLLS availability instead of blocking in
/// read, so the stop flag (set at teardown/cancel) always ends the thread
/// — even when a NON-owned process holds the pipe's write end and no EOF
/// can arrive. The thread is joined with a real-finish oracle at teardown.
fn forward_stdout(
    stdout: std::process::ChildStdout,
    label: &str,
    stop: &AtomicBool,
    end_slot: &Mutex<ReaderEnd>,
) {
    forward_pipe(stdout, label, stop, end_slot)
}

/// stderr forwarder: same forwarding + sniffing semantics as stdout (the
/// VM session's MCP banner is a stderr eprintln in the runtime).
fn forward_stderr(
    stderr: std::process::ChildStderr,
    label: &str,
    stop: &AtomicBool,
    end_slot: &Mutex<ReaderEnd>,
) {
    forward_pipe(stderr, label, stop, end_slot)
}

/// Marker prefix the internal VM session child writes to stderr right
/// before exiting fatally (R4-02): the first occurrence is captured into
/// the installed RunIo and surfaces as the named failure reason.
pub const VM_SESSION_FATAL_PREFIX: &str = "VM_SESSION_FATAL: ";

fn forward_pipe(
    pipe: impl std::io::Read + PipeAvailability,
    label: &str,
    stop: &AtomicBool,
    end_slot: &Mutex<ReaderEnd>,
) {
    let mut reader = std::io::BufReader::new(pipe);
    let mut announced = false;
    loop {
        if stop.load(Ordering::SeqCst) {
            *end_slot.lock().unwrap() = ReaderEnd::Stopped;
            return;
        }
        match read_available(&mut reader) {
            ReadOutcome::Chunk(bytes) if !bytes.is_empty() => {
                log_chunk(&bytes);
                if label == "vm-session" {
                    sniff_vm_session_fatal(&bytes);
                }
                if !announced
                    && (label == "dev-server" || label == "api-server" || label == "vm-session")
                {
                    if let Some(url) = sniff_url(&String::from_utf8_lossy(&bytes)) {
                        launched(&url);
                        announced = true;
                    }
                }
            }
            ReadOutcome::Chunk(_) => {
                // Zero available bytes (windows peek): brief backoff before
                // the next poll; the loop head re-checks stop each tick.
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            ReadOutcome::Eof => {
                *end_slot.lock().unwrap() = ReaderEnd::NaturalEof;
                return;
            }
            ReadOutcome::Err(reason) => {
                *end_slot.lock().unwrap() = ReaderEnd::IoError(reason);
                return;
            }
        }
    }
}

/// One poll result of [`read_available`].
enum ReadOutcome {
    Chunk(Vec<u8>),
    /// The pipe broke / all write ends closed — a natural end.
    Eof,
    /// A real read error (not EOF).
    Err(String),
}

/// Windows availability probe for the anonymous stdout pipe: zero-blocking
/// (PeekNamedPipe — synchronous pipe handles are always "signaled", so a
/// readiness wait is impossible; peeking is the cancellable shape).
#[cfg(windows)]
mod peek {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;

    extern "system" {
        fn PeekNamedPipe(
            hNamedPipe: *mut c_void,
            lpBuffer: *mut c_void,
            nBufferSize: u32,
            lpBytesRead: *mut u32,
            lpTotalBytesAvail: *mut u32,
            lpBytesLeftThisMessage: *mut u32,
        ) -> i32;
    }

    pub fn available<R: std::io::Read + AsRawHandle>(reader: &R) -> std::io::Result<u32> {
        let mut avail: u32 = 0;
        let ok = unsafe {
            PeekNamedPipe(
                reader.as_raw_handle() as *mut c_void,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut avail,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(avail)
        }
    }
}

/// Availability probe abstraction: ChildStdout/ChildStderr expose a
/// zero-blocking readiness probe on both platforms — Windows via
/// PeekNamedPipe, Unix via poll(2) with a 10ms tick (R1-01: the loop
/// never sits in a blocking read, so the stop flag is honored between
/// ticks even when a NON-owned process holds the pipe's write end).
trait PipeAvailability {
    fn available(&self) -> std::io::Result<u32>;
}

#[cfg(windows)]
impl PipeAvailability for std::process::ChildStdout {
    fn available(&self) -> std::io::Result<u32> {
        peek::available(self)
    }
}

#[cfg(windows)]
impl PipeAvailability for std::process::ChildStderr {
    fn available(&self) -> std::io::Result<u32> {
        peek::available(self)
    }
}

/// Unix readiness probe: poll(2) with a 10ms timeout — the stop flag is
/// observable every tick; POLLHUP still reports readable so a final
/// `read()` can drain leftovers and return the natural 0-byte EOF.
#[cfg(not(windows))]
mod upoll {
    use std::os::unix::io::AsRawFd;

    pub fn available<R: std::io::Read + AsRawFd>(reader: &R) -> std::io::Result<u32> {
        let mut pfd = libc::pollfd {
            fd: reader.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: single well-formed pollfd; poll does not mutate the fd.
        let rc = unsafe { libc::poll(&mut pfd, 1, 10) };
        if rc < 0 {
            let err = std::io::Error::last_os_error();
            // EINTR: transient — report "nothing available" so the loop
            // head re-checks the stop flag and polls again.
            if err.raw_os_error() == Some(libc::EINTR) {
                return Ok(0);
            }
            return Err(err);
        }
        if rc == 0 {
            return Ok(0); // timed out: nothing readable within the tick
        }
        if pfd.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
            return Err(std::io::Error::other("poll reported POLLERR/POLLNVAL"));
        }
        // POLLIN (data or EOF-signalled peer) and POLLHUP both leave the
        // subsequent read() non-blocking with a definitive answer.
        Ok(1)
    }
}

#[cfg(not(windows))]
impl PipeAvailability for std::process::ChildStdout {
    fn available(&self) -> std::io::Result<u32> {
        upoll::available(self)
    }
}

#[cfg(not(windows))]
impl PipeAvailability for std::process::ChildStderr {
    fn available(&self) -> std::io::Result<u32> {
        upoll::available(self)
    }
}

/// Read whatever is currently available (peek/poll-driven on BOTH
/// platforms, stop-flag aware between polls; buffered bytes from an
/// earlier read win over a fresh probe).
fn read_available(
    reader: &mut std::io::BufReader<impl std::io::Read + PipeAvailability>,
) -> ReadOutcome {
    use std::io::{BufRead as _, Read as _};
    // Bytes already buffered by an earlier larger read are returned first
    // (a poll/peek on the fd cannot see them).
    if !reader.buffer().is_empty() {
        let buffered = reader.buffer().to_vec();
        reader.consume(buffered.len());
        return ReadOutcome::Chunk(buffered);
    }
    let avail = match reader.get_ref().available() {
        Ok(n) => n,
        Err(e) => return classify_pipe_error(e),
    };
    if avail == 0 {
        // No data right now: yield an empty chunk so the caller's loop
        // head can re-check the stop flag and poll again.
        return ReadOutcome::Chunk(Vec::new());
    }
    let mut chunk = [0u8; 8192];
    match reader.read(&mut chunk) {
        Ok(0) => ReadOutcome::Eof,
        Ok(n) => ReadOutcome::Chunk(chunk[..n].to_vec()),
        Err(e) => classify_pipe_error(e),
    }
}

/// Distinguish a broken pipe (all write ends gone = natural EOF) from a
/// genuine read error (F01/R3's named disclosure).
fn classify_pipe_error(e: std::io::Error) -> ReadOutcome {
    match e.raw_os_error() {
        Some(109) | Some(232) => ReadOutcome::Eof, // ERROR_BROKEN_PIPE / ERROR_NO_DATA
        _ => ReadOutcome::Err(e.to_string()),
    }
}

/// Extract the first `http://127.0.0.1:PORT` / `http://localhost:PORT`
/// prefix from real child output (Vite prints `➜  Local: http://…`).
/// None when this chunk carries no URL — no fabricated launch spec.
pub fn sniff_url(text: &str) -> Option<String> {
    for prefix in ["http://127.0.0.1:", "http://localhost:"] {
        if let Some(pos) = text.find(prefix) {
            let start = pos + prefix.len();
            let digits: String = text[start..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                // Include a trailing base path when one follows; stop at
                // whitespace, quotes, or ANSI escape bytes.
                let mut url = format!("{prefix}{digits}");
                for c in text[start + digits.len()..].chars() {
                    if c.is_whitespace() || c == '"' || c == '\'' || c == '\u{1b}' {
                        break;
                    }
                    if url.len() > 256 {
                        break;
                    }
                    url.push(c);
                }
                while url.ends_with('/') {
                    url.pop();
                }
                return Some(url);
            }
        }
    }
    None
}

/// Cancel-aware wait for a tracked tree: polls `try_wait`; on cancellation
/// terminates + bounded-reaps the whole tree and joins the readers BEFORE
/// returning (the final receipt may only follow once the owned tree and
/// readers are closed — F-815-T05-R1-01).
pub fn wait_tracked(tracked: &TrackedChild) -> std::io::Result<ExitStatus> {
    loop {
        if let Some(status) = tracked.try_wait()? {
            if let Err(reason) = tracked.join_readers() {
                note_teardown_failure(&reason);
                return Err(std::io::Error::other(reason));
            }
            return Ok(status);
        }
        if cancelled() {
            return tracked.reap();
        }
        std::thread::sleep(WAIT_POLL);
    }
}

/// Set the per-request VM session child context (R4-01): EXACT entries for
/// the CURRENT request — applied to the session child's spawn; entries the
/// request does not derive are simply absent (never inherited from the
/// host process env).
pub fn set_vm_session_env(entries: Vec<(String, String)>) {
    if let Some(io) = current() {
        *io.vm_session_env.lock().unwrap() = entries;
    }
}

/// The vm-session child's named fatal reason, when it wrote one (R4-02).
pub fn vm_session_fatal() -> Option<String> {
    current()
        .and_then(|io| io.vm_session_fatal.lock().unwrap().clone())
        .filter(|s| !s.is_empty())
}

/// Capture the first `VM_SESSION_FATAL:` line from forwarded vm-session
/// stderr chunks (R4-02). Line-split tolerant: the marker may arrive in a
/// chunk that carries earlier output before it.
fn sniff_vm_session_fatal(bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    let Some(pos) = text.find(VM_SESSION_FATAL_PREFIX) else {
        return;
    };
    if let Some(io) = current() {
        let mut slot = io.vm_session_fatal.lock().unwrap();
        if slot.is_some() {
            return;
        }
        let rest = &text[pos + VM_SESSION_FATAL_PREFIX.len()..];
        let reason: String = rest
            .lines()
            .next()
            .unwrap_or("unspecified fatal error")
            .trim()
            .to_string();
        *slot = Some(reason);
    }
}

/// Hosted external command of a build/run path (R4-05: jet/ark/vscode
/// gradle/hvigor/code invocations): when a RunIo is installed the command
/// runs as an owned tree of the operation — cancel/EOF terminate the whole
/// tree before the final receipt — with stdout forwarded to the log
/// stream. Legacy CLI (no RunIo) keeps the frozen direct
/// `Command::status` behavior.
pub fn run_external_status(
    program: &std::path::Path,
    args: &[&str],
    cwd: &std::path::Path,
    label: &str,
) -> std::io::Result<ExitStatus> {
    if current().is_some() {
        let spec = TrackedSpawn::new(
            program.to_string_lossy().to_string(),
            args.iter().map(|a| a.to_string()).collect(),
        )
        .with_cwd(cwd);
        let tracked = spawn_tracked(spec, label)?;
        return wait_tracked(&tracked);
    }
    std::process::Command::new(program)
        .args(args)
        .current_dir(cwd)
        .status()
}

/// Owned capture probe with the full R6 consumer contract (PLAN-815
/// T-06 R1）：natural EOF + exit 0 → `Ok(Some(complete_stdout))`（空输出
/// 是合法完整结果）；取消 → `Err(CANCELLED_ERR)`；非自然捕获结束
/// （BoundOverrun/Truncated/IoError/无取消 Stopped）→ 先 kill+有界收尸
/// 再 `Err(具名)`；非零退出/等待失败/spawn 失败同具名。部分输出永不
/// 冒充完整探测结果。
pub fn run_capture_probe(
    program: &std::path::Path,
    args: &[&str],
    cwd: &std::path::Path,
    label: &str,
) -> Result<Option<String>, String> {
    let spec = TrackedSpawn {
        capture_stdout: true,
        ..TrackedSpawn::new(
            program.to_string_lossy().to_string(),
            args.iter().map(|a| a.to_string()).collect(),
        )
        .with_cwd(cwd)
    };
    let tracked =
        spawn_tracked(spec, label).map_err(|e| format!("{label} probe failed to start: {e}"))?;
    let (stdout, end) = drain_captured_stdout(&tracked);
    if cancelled() {
        return Err(CANCELLED_ERR.to_string());
    }
    if !end.is_natural_eof() {
        tracked.kill();
        let teardown = wait_tracked(&tracked);
        return Err(format!(
            "{label} probe did not complete ({end:?}; owned tree teardown {teardown:?})"
        ));
    }
    match wait_tracked(&tracked) {
        Ok(status) if status.success() => Ok(Some(String::from_utf8_lossy(&stdout).into_owned())),
        Ok(status) => Err(format!("{label} probe exited with {status}")),
        Err(reason) => Err(format!("{label} probe wait failed: {reason}")),
    }
}

impl TrackedChild {
    /// Take the caller-drained stdout pipe (spawn with `capture_stdout`).
    pub fn take_captured_stdout(&self) -> Option<std::process::ChildStdout> {
        self.captured_stdout.lock().unwrap().take()
    }
}

/// How a caller-drained capture ended (PLAN-815 R5 /
/// F-815-T05-R4-05-P1): EVERY actual reader — the forwarder threads AND
/// the caller-drained capture pipes — must respond to the operation's
/// cancel/EOF (truly stop, or end with a named bounded failure); captured
/// bytes are never collected unbounded and an interrupted/error drain is
/// never presented as complete output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainEnd {
    /// The pipe reached EOF on its own (complete output).
    Eof,
    /// The stop flag / cancellation ended the drain (output may be
    /// partial — a non-owned holder may still hold the pipe).
    Stopped,
    /// The collection cap was hit (output truncated; the pipe is dropped).
    Truncated,
    /// The read failed (output may be partial).
    IoError(String),
    /// The drain did not end within the hard bound (named failure — the
    /// stop flag remains set, the pipe is dropped).
    BoundOverrun,
}

impl DrainEnd {
    /// True only for the natural, complete end (PLAN-815 R6 /
    /// F-815-T05-R4-05-P2): every other classification is an INCOMPLETE
    /// capture that consumers must surface as a named failure — never
    /// silently downgrade to "no data" semantics.
    pub fn is_natural_eof(&self) -> bool {
        matches!(self, DrainEnd::Eof)
    }
}

/// Hard bound for a caller-drained capture: same shape as the forwarder
/// teardown bound — no reader, drained or forwarded, may block an
/// operation's teardown indefinitely.
const DRAIN_HARD_BOUND: Duration = TEARDOWN_GRACE;
/// Collection cap for captured output (`adb devices`-style probes are
/// tiny; unbounded collection is a leak vector on a chatty child).
const DRAIN_MAX_BYTES: usize = 1024 * 1024;

/// Drain a captured stdout with the same stop-aware polling read the
/// forwarders use, PLUS the operation's cancel/stop flags and a hard
/// bound: the loop checks `reader_stop` (set by kill/teardown) and
/// `cancelled()` (cancel AND host-EOF both set it) every tick, so even a
/// NON-owned process holding the pipe cannot pin the drain past the
/// operation's lifecycle (PLAN-815 R5 / F-815-T05-R4-05-P1 — the R4
/// version looped on empty chunks with no stop check at all).
pub fn drain_captured_stdout(tracked: &TrackedChild) -> (Vec<u8>, DrainEnd) {
    let Some(stdout) = tracked.take_captured_stdout() else {
        // No pipe was captured (spawn without capture_stdout or already
        // drained): nothing observable, nothing collected.
        return (Vec::new(), DrainEnd::Eof);
    };
    let mut reader = std::io::BufReader::new(stdout);
    let mut collected = Vec::new();
    let deadline = std::time::Instant::now() + DRAIN_HARD_BOUND;
    loop {
        if tracked.reader_stop.load(Ordering::SeqCst) || cancelled() {
            return (collected, DrainEnd::Stopped);
        }
        if std::time::Instant::now() > deadline {
            return (collected, DrainEnd::BoundOverrun);
        }
        match read_available(&mut reader) {
            ReadOutcome::Chunk(bytes) => {
                if bytes.is_empty() {
                    // Nothing available this tick: back off (never busy
                    // spin) — the loop head re-checks the stop flags.
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                collected.extend_from_slice(&bytes);
                if collected.len() >= DRAIN_MAX_BYTES {
                    return (collected, DrainEnd::Truncated);
                }
            }
            ReadOutcome::Eof => return (collected, DrainEnd::Eof),
            ReadOutcome::Err(reason) => return (collected, DrainEnd::IoError(reason)),
        }
    }
}

/// Record a named teardown failure on the installed RunIo (if any) —
/// disclosed on the final report instead of pretending a clean close.
fn note_teardown_failure(reason: &str) {
    if let Some(io) = current() {
        let mut slot = io.teardown_error.lock().unwrap();
        if slot.is_none() {
            *slot = Some(reason.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The RunIo slot is process-global; tests that touch it must not
    /// interleave under a single-process libtest run.
    static SLOT_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn sniff_url_finds_vite_local_line() {
        let line = "  ➜  Local:   http://127.0.0.1:5173/\n";
        assert_eq!(sniff_url(line).as_deref(), Some("http://127.0.0.1:5173"));
    }

    #[test]
    fn sniff_url_finds_localhost_and_stops_at_ansi() {
        let line = "ready on http://localhost:4173 \u{1b}[32mok\u{1b}[0m";
        assert_eq!(sniff_url(line).as_deref(), Some("http://localhost:4173"));
    }

    #[test]
    fn sniff_url_none_without_url_or_bare_digits() {
        assert_eq!(sniff_url("vite v5.0.0 dev server starting"), None);
        assert_eq!(sniff_url("http://127.0.0.1:"), None);
    }

    #[test]
    fn install_is_single_flight_and_uninstall_clears() {
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        assert!(!install(), "second work operation must not install");
        assert!(current().is_some());
        assert!(check_cancel().is_ok());
        current().unwrap().cancel();
        assert!(cancelled());
        assert_eq!(check_cancel().unwrap_err(), CANCELLED_ERR);
        assert!(cancelled_err(CANCELLED_ERR));
        uninstall();
        assert!(current().is_none());
        assert!(!cancelled(), "unhosted is never cancelled");
    }

    /// Real owned-tree lifecycle (F-815-T05-R1-01 core): a REAL tree with
    /// grandchildren (cmd → cmd → ping) is terminated and reaped as a
    /// whole before the final return, and the readers join; a real echo
    /// child proves cancellable input forwarding end to end.
    #[test]
    fn real_tree_cancel_input_forward_and_reader_join() {
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        let io = current().unwrap();
        let logged: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let launched_urls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        {
            let logged = Arc::clone(&logged);
            *io.on_log_chunk.lock().unwrap() = Some(Box::new(move |bytes: &[u8]| {
                logged.lock().unwrap().extend_from_slice(bytes);
            }));
        }
        {
            let urls = Arc::clone(&launched_urls);
            *io.on_launched.lock().unwrap() = Some(Box::new(move |url: &str| {
                urls.lock().unwrap().push(url.to_string());
            }));
        }

        // A real interactive child forwards lines containing "x" from its
        // stdin to its stdout (the cancellable write is this module's
        // half of F-815-T05-R1-02). Cross-platform (PLAN-815 T-07 已披露
        // 可移植缺口收口): Windows `findstr x`, Unix `grep x`.
        #[cfg(windows)]
        let spec = TrackedSpawn {
            windows_cmd_shim: true,
            ..TrackedSpawn::new("findstr", vec!["x".to_string()])
        };
        #[cfg(not(windows))]
        let spec = TrackedSpawn::new("grep", vec!["x".to_string()]);
        let tracked = spawn_tracked(spec, "dev-server").expect("spawn real child");
        assert_eq!(tracked.id, tracked.id);
        // Direct foreground write (writer-thread decoupling is host-level).
        assert!(tracked.write_input_stoppable(b"hello x line\n").is_ok());
        assert!(tracked.write_input_stoppable(b"plain line\n").is_ok());
        tracked.close_stdin(); // EOF so the filter terminates on its own

        let status = wait_tracked(&tracked).expect("wait real child");
        assert!(status.success(), "filter should exit 0, got {status:?}");

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if logged.lock().unwrap().contains(&b'\n') {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!("forwarder did not deliver child output");
            }
            std::thread::sleep(WAIT_POLL);
        }
        let logged_vec = logged.lock().unwrap().clone();
        let logged = String::from_utf8_lossy(&logged_vec);
        assert!(logged.contains("hello x line"), "forwarded: {logged}");
        assert!(launched_urls.lock().unwrap().is_empty());
        assert!(tracked.readers.lock().unwrap().is_empty(), "readers joined");

        // Cancel path: a REAL owned tree (grandchildren included) is
        // terminated and reaped as a whole before return. Cross-platform:
        // Windows nested cmd ping, Unix sh with a backgrounded sleeper.
        #[cfg(windows)]
        let spec = TrackedSpawn {
            windows_cmd_shim: true,
            ..TrackedSpawn::new(
                "cmd",
                vec!["/C".to_string(), "cmd /C ping -n 30 127.0.0.1".to_string()],
            )
        };
        #[cfg(not(windows))]
        let spec = TrackedSpawn::new(
            "sh",
            vec!["-c".to_string(), "sleep 30 & exec sleep 30".to_string()],
        );
        let tracked = spawn_tracked(spec, "api-server").expect("spawn blocking tree");
        std::thread::sleep(Duration::from_millis(700)); // let the grandchild spawn
        io.cancel();
        let start = std::time::Instant::now();
        let _ = wait_tracked(&tracked);
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "cancel must reap the owned tree promptly"
        );
        // Reaped + readers joined — direct observables for both halves.
        assert!(tracked.readers.lock().unwrap().is_empty());
        uninstall();
    }

    /// PLAN-815 R4-01: the VM session child context is per-request EXACT —
    /// the second request's entries REPLACE the first's (a request without
    /// an external backend/theme carries none; nothing accumulates).
    #[test]
    fn vm_session_env_is_replaced_per_request() {
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        let io = current().unwrap();
        assert!(
            io.vm_session_env.lock().unwrap().is_empty(),
            "fresh RunIo starts with no session context"
        );
        set_vm_session_env(vec![(
            "AUTO_UI_TOOL_VM_BACK_ROOT".to_string(),
            "D:/does/not/matter".to_string(),
        )]);
        assert_eq!(
            io.vm_session_env.lock().unwrap().as_slice(),
            [(
                "AUTO_UI_TOOL_VM_BACK_ROOT".to_string(),
                "D:/does/not/matter".to_string()
            )]
        );
        // Second request (no backend, no theme): the exact-empty set — the
        // spawn applies env_remove for the internal keys and adds NOTHING.
        set_vm_session_env(Vec::new());
        assert!(
            io.vm_session_env.lock().unwrap().is_empty(),
            "a None-request must replace, not retain, the previous context"
        );
        assert!(vm_session_fatal().is_none(), "no fatal before any child");
        uninstall();
    }

    /// PLAN-815 R4-02: the `VM_SESSION_FATAL:` marker on forwarded
    /// vm-session stderr chunks is captured (first line wins).
    #[test]
    fn vm_session_fatal_marker_is_captured_once() {
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        sniff_vm_session_fatal(
            b"some log line\nVM_SESSION_FATAL: external backend load failed: dll missing\nmore\n",
        );
        assert_eq!(
            vm_session_fatal().as_deref(),
            Some("external backend load failed: dll missing")
        );
        // A later marker does not overwrite the first.
        sniff_vm_session_fatal(b"VM_SESSION_FATAL: second failure\n");
        assert_eq!(
            vm_session_fatal().as_deref(),
            Some("external backend load failed: dll missing")
        );
        uninstall();
    }

    /// PLAN-815 R5（F-815-T05-R4-05-P1）：caller-drained 捕获管道同样应答
    /// 本操作的取消——取消后排水在停止标志处立即返回（有界、非忙等），
    /// 自然 EOF 正控保持完整输出语义。跨平台：Unix 用 sh 等价命令。
    #[test]
    fn captured_drain_answers_cancel_and_natural_eof() {
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        let io = current().unwrap();

        // Natural-EOF positive: `echo` closes its end — the drain collects
        // the output and classifies Eof.
        #[cfg(windows)]
        let echo_spec = TrackedSpawn {
            windows_cmd_shim: true,
            capture_stdout: true,
            ..TrackedSpawn::new("cmd", vec!["/C".to_string(), "echo drain-ok".to_string()])
        };
        #[cfg(not(windows))]
        let echo_spec = TrackedSpawn {
            capture_stdout: true,
            ..TrackedSpawn::new("sh", vec!["-c".to_string(), "echo drain-ok".to_string()])
        };
        let tracked = spawn_tracked(echo_spec, "capture-eof").expect("spawn echo");
        let (out, end) = drain_captured_stdout(&tracked);
        let _ = wait_tracked(&tracked);
        assert!(matches!(end, DrainEnd::Eof), "end: {end:?}");
        assert!(String::from_utf8_lossy(&out).contains("drain-ok"));

        // Cancel positive: a long-running tree with a captured stdout —
        // cancel (kill sets reader_stop) must end the drain promptly
        // instead of looping on empty polls (the R4 bug shape).
        #[cfg(windows)]
        let sleeper_spec = TrackedSpawn {
            windows_cmd_shim: true,
            capture_stdout: true,
            ..TrackedSpawn::new(
                "cmd",
                vec!["/C".to_string(), "ping -n 30 127.0.0.1".to_string()],
            )
        };
        #[cfg(not(windows))]
        let sleeper_spec = TrackedSpawn {
            capture_stdout: true,
            ..TrackedSpawn::new("sh", vec!["-c".to_string(), "sleep 30".to_string()])
        };
        let tracked = spawn_tracked(sleeper_spec, "capture-cancel").expect("spawn sleeper");
        std::thread::sleep(Duration::from_millis(500));
        io.cancel();
        let started = std::time::Instant::now();
        let (_out, end) = drain_captured_stdout(&tracked);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "drain must answer cancel promptly (took {:?}, end {end:?})",
            started.elapsed()
        );
        assert!(
            matches!(end, DrainEnd::Stopped | DrainEnd::Eof),
            "cancelled drain ends Stopped (flag) or Eof (tree died): {end:?}"
        );
        let _ = wait_tracked(&tracked);
        uninstall();
    }

    /// PLAN-815 R6（F-815-T05-R4-05-P2）代表对照：Truncated——子进程输出
    /// 超过 1MiB 收集上限，drain 以 Truncated 结束（部分输出，不冒充
    /// 完整）；随后树终止+有界收尸（消费者契约见 jet adb 臂）。
    #[test]
    fn captured_drain_truncates_at_collection_cap() {
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        // A real child that writes ~1.2 MiB to stdout then exits 0.
        let spill = tempfile::tempdir().expect("tempdir");
        let big = spill.path().join("big.txt");
        let chunk = "0123456789abcdef".repeat(64); // 1 KiB per line-ish
        let mut content = String::with_capacity(1024 * 1200);
        for _ in 0..1200 {
            content.push_str(&chunk);
            content.push('\n');
        }
        std::fs::write(&big, content).unwrap();
        #[cfg(windows)]
        let spec = TrackedSpawn {
            windows_cmd_shim: true,
            capture_stdout: true,
            ..TrackedSpawn::new(
                "cmd",
                vec!["/C".to_string(), format!("type {}", big.display())],
            )
        };
        #[cfg(not(windows))]
        let spec = TrackedSpawn {
            capture_stdout: true,
            ..TrackedSpawn::new("cat", vec![big.display().to_string()])
        };
        let tracked = spawn_tracked(spec, "capture-cap").expect("spawn writer");
        let started = std::time::Instant::now();
        let (out, end) = drain_captured_stdout(&tracked);
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "a fast writer must hit the cap quickly"
        );
        assert_eq!(end, DrainEnd::Truncated, "cap hit: {end:?}");
        assert!(
            out.len() >= DRAIN_MAX_BYTES,
            "collected up to the cap: {}",
            out.len()
        );
        // The abandoned tree is torn down and reaped with a bound.
        tracked.kill();
        let _ = wait_tracked(&tracked);
        uninstall();
    }

    /// PLAN-815 R6（F-815-T05-R4-05-P2）代表对照：BoundOverrun——一个
    /// 静默子进程持有捕获管道（无数据、不退出），drain 在硬界限处具名
    /// 返回（本测试约 10s，归重串行档）。
    #[test]
    #[ignore = "slow: ~10s hard-bound wait; serial evidence run via --ignored"]
    fn captured_drain_overruns_hard_bound_when_silent_holder_keeps_pipe() {
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        #[cfg(windows)]
        let spec = TrackedSpawn {
            capture_stdout: true,
            ..TrackedSpawn::new(
                "powershell",
                vec![
                    "-NoProfile".to_string(),
                    "-Command".to_string(),
                    "Start-Sleep -Seconds 60".to_string(),
                ],
            )
        };
        #[cfg(not(windows))]
        let spec = TrackedSpawn {
            capture_stdout: true,
            ..TrackedSpawn::new("sh", vec!["-c".to_string(), "sleep 60".to_string()])
        };
        let tracked = spawn_tracked(spec, "capture-bound").expect("spawn silent holder");
        let started = std::time::Instant::now();
        let (out, end) = drain_captured_stdout(&tracked);
        let elapsed = started.elapsed();
        assert_eq!(end, DrainEnd::BoundOverrun, "hard bound fired: {end:?}");
        assert!(
            elapsed >= DRAIN_HARD_BOUND && elapsed < DRAIN_HARD_BOUND + Duration::from_secs(5),
            "bound fires at the limit, not later: {elapsed:?}"
        );
        assert!(out.is_empty(), "silent holder produced no output");
        // The abandoned tree is torn down and reaped with a bound.
        tracked.kill();
        let _ = wait_tracked(&tracked);
        uninstall();
    }

    /// R1-01 non-Windows arm: a NON-owned process holds the pipe's write
    /// end (no data, no EOF can ever arrive) — the poll-based reader still
    /// TRULY stops on the stop flag within the bound (no unbounded
    /// read_until, no unbounded join) and the end cause is the named
    /// `Stopped` disclosure. A natural-EOF positive control keeps the
    /// three-way classification honest. Runs on the declared supported
    /// non-Windows platforms (Linux CI / review environments).
    #[cfg(not(windows))]
    #[test]
    fn unix_reader_truly_stops_when_non_owned_process_holds_pipe() {
        use std::io::Write as _;
        use std::process::{Command, Stdio};
        let _guard = SLOT_LOCK.lock().unwrap();
        assert!(install());
        let _ = current().unwrap();

        // Held pipe: `sh -c 'sleep 30 & exit 0'` — the direct child exits
        // immediately, the backgrounded sleep (outside any owned tree of
        // ours) inherits the piped stdout write end and holds it.
        let mut holder = Command::new("sh")
            .arg("-c")
            .arg("sleep 30 & exit 0")
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn sh holder");
        let held_stdout = holder.stdout.take().expect("piped stdout");
        let _ = holder.wait();
        let stop = Arc::new(AtomicBool::new(false));
        let end_slot = Arc::new(Mutex::new(ReaderEnd::Running));
        let slot_clone = Arc::clone(&end_slot);
        let stop_clone = Arc::clone(&stop);
        let handle = std::thread::Builder::new()
            .name("held-reader-unix".into())
            .spawn(move || {
                forward_pipe(held_stdout, "held", &stop_clone, &slot_clone);
            })
            .expect("spawn reader");

        std::thread::sleep(Duration::from_millis(300)); // reader is parked in poll
        let started = std::time::Instant::now();
        stop.store(true, Ordering::SeqCst);
        while !handle.is_finished() {
            assert!(
                std::time::Instant::now() - started < Duration::from_secs(5),
                "poll-based reader must stop on the flag, not block in read"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = handle.join();
        assert_eq!(*end_slot.lock().unwrap(), ReaderEnd::Stopped);

        // Natural-EOF positive: an echo child closes its end → NaturalEof.
        let mut echo = Command::new("sh")
            .arg("-c")
            .arg("echo held-reader-eof")
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn echo");
        let echo_stdout = echo.stdout.take().expect("piped stdout");
        let _ = echo.wait();
        let stop = Arc::new(AtomicBool::new(false));
        let end_slot = Arc::new(Mutex::new(ReaderEnd::Running));
        let slot_clone = Arc::clone(&end_slot);
        let stop_clone = Arc::clone(&stop);
        let handle = std::thread::Builder::new()
            .name("eof-reader-unix".into())
            .spawn(move || {
                forward_pipe(echo_stdout, "held", &stop_clone, &slot_clone);
            })
            .expect("spawn reader");
        let _ = handle.join();
        assert_eq!(*end_slot.lock().unwrap(), ReaderEnd::NaturalEof);

        uninstall();
    }
}
