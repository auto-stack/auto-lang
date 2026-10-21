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
    /// Windows: resolve via `cmd /C` (mirrors the legacy pkg.rs shim for
    /// .cmd/.bat package managers).
    pub windows_cmd_shim: bool,
    /// Also PIPE the child's stderr (forwarded to the host log stream +
    /// URL-sniffed) instead of inheriting it. Used by the internal VM
    /// session child: its real MCP endpoint banner lands on stderr
    /// (R2-03's accurate-URL Launched source).
    pub pipe_stderr: bool,
}

impl TrackedSpawn {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            cwd: None,
            env: Vec::new(),
            windows_cmd_shim: false,
            pipe_stderr: false,
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
        }
    }
}

/// One owned child (tree) of a hosted operation.
pub struct TrackedChild {
    proc: Mutex<Option<auto_tool_execution::OwnedProcess>>,
    stdin: Mutex<Option<auto_tool_execution::StdinPipe>>,
    /// stdout/stderr forwarder threads with their natural-EOF flags
    /// (joined with a real-finish oracle at teardown).
    readers: Mutex<Vec<(std::thread::JoinHandle<()>, Arc<AtomicBool>)>>,
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
    /// (F01/R2): teardown SIGNALS the stop flag first — the polling
    /// readers observe it within one 10ms tick and truly stop even when a
    /// NON-owned process holds the pipe's write end and no EOF can ever
    /// arrive. A reader that stopped WITHOUT a natural pipe EOF is a NAMED
    /// teardown failure (disclosed on the final report); a bound overrun is
    /// likewise named, never silently detached.
    fn join_readers(&self) -> Result<(), String> {
        self.reader_stop.store(true, Ordering::SeqCst);
        let readers: Vec<_> = self.readers.lock().unwrap().drain(..).collect();
        let mut failures = Vec::new();
        for (handle, reached_eof) in readers {
            let deadline = std::time::Instant::now() + TEARDOWN_GRACE;
            while !handle.is_finished() {
                if std::time::Instant::now() > deadline {
                    failures.push(format!(
                        "reader thread for `{}` did not stop within {TEARDOWN_GRACE:?} of teardown",
                        self.label
                    ));
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let _ = handle.join();
            if !failures.is_empty() {
                continue;
            }
            if !reached_eof.load(Ordering::SeqCst) {
                failures.push(format!(
                    "reader for `{}` stopped by teardown without a natural pipe EOF (a process outside the owned tree may still hold the pipe)",
                    self.label
                ));
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
        // (F01/R2 oracle): bounded join, named failure on timeout.
        let writer = io.input_writer.lock().unwrap().take();
        if let Some(handle) = writer {
            let deadline = std::time::Instant::now() + TEARDOWN_GRACE;
            while !handle.is_finished() {
                if std::time::Instant::now() > deadline {
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
            let _ = handle.join();
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
        input_stop: Arc::new(AtomicBool::new(false)),
        reader_stop: Arc::new(AtomicBool::new(false)),
        label: label.to_string(),
        id: pid,
    });
    io.register(Arc::clone(&tracked));
    if let Some(stdout) = child_stdout {
        let forward_label = label.to_string();
        let reader_stop = Arc::clone(&tracked.reader_stop);
        let reached_eof = Arc::new(AtomicBool::new(false));
        let eof_flag = Arc::clone(&reached_eof);
        let handle = std::thread::Builder::new()
            .name(format!("child-out:{forward_label}"))
            .spawn(move || forward_stdout(stdout, &forward_label, &reader_stop, &reached_eof))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        tracked.readers.lock().unwrap().push((handle, eof_flag));
    }
    if let Some(stderr) = child_stderr {
        let forward_label = label.to_string();
        let reader_stop = Arc::clone(&tracked.reader_stop);
        let reached_eof = Arc::new(AtomicBool::new(false));
        let eof_flag = Arc::clone(&reached_eof);
        let handle = std::thread::Builder::new()
            .name(format!("child-err:{forward_label}"))
            .spawn(move || forward_stderr(stderr, &forward_label, &reader_stop, &reached_eof))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        tracked.readers.lock().unwrap().push((handle, eof_flag));
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
    reached_eof: &AtomicBool,
) {
    forward_pipe(stdout, label, stop, reached_eof)
}

/// stderr forwarder: same forwarding + sniffing semantics as stdout (the
/// VM session's MCP banner is a stderr eprintln in the runtime).
fn forward_stderr(
    stderr: std::process::ChildStderr,
    label: &str,
    stop: &AtomicBool,
    reached_eof: &AtomicBool,
) {
    forward_pipe(stderr, label, stop, reached_eof)
}

fn forward_pipe(
    pipe: impl std::io::Read + PipeAvailability,
    label: &str,
    stop: &AtomicBool,
    reached_eof: &AtomicBool,
) {
    let mut reader = std::io::BufReader::new(pipe);
    let mut announced = false;
    loop {
        if stop.load(Ordering::SeqCst) {
            return; // teardown signalled — not a natural EOF
        }
        let chunk = read_available(&mut reader, stop);
        match chunk {
            Some(bytes) if !bytes.is_empty() => {
                log_chunk(&bytes);
                if !announced
                    && (label == "dev-server" || label == "api-server" || label == "vm-session")
                {
                    if let Some(url) = sniff_url(&String::from_utf8_lossy(&bytes)) {
                        launched(&url);
                        announced = true;
                    }
                }
            }
            Some(_) => continue,
            None => {
                // Natural end (EOF / broken pipe) without the stop flag.
                reached_eof.store(true, Ordering::SeqCst);
                return;
            }
        }
    }
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

/// Availability probe abstraction: ChildStdout/ChildStder both expose
/// AsRawHandle on Windows; the generic fallback is the bounded line read.
trait PipeAvailability {
    fn available(&self) -> std::io::Result<u32>;
}

#[cfg(not(windows))]
impl<T: std::io::Read> PipeAvailability for T {
    fn available(&self) -> std::io::Result<u32> {
        // Non-Windows fallback: no peek — the bounded line read drives the
        // loop and teardown relies on the bound + named failure disclosure.
        Err(std::io::Error::other("availability probe unsupported here"))
    }
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

/// Read whatever is currently available (Windows: peek-driven, stop-flag
/// aware between polls; Unix: line read — the stop path then relies on the
/// teardown bound + named failure disclosure).
fn read_available(
    reader: &mut std::io::BufReader<impl std::io::Read + PipeAvailability>,
    stop: &AtomicBool,
) -> Option<Vec<u8>> {
    #[cfg(windows)]
    {
        use std::io::Read as _;
        loop {
            if stop.load(Ordering::SeqCst) {
                return None;
            }
            let avail = match reader.get_ref().available() {
                Ok(n) => n,
                Err(_) => return None, // broken pipe = EOF after tree death
            };
            if avail == 0 {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            let mut chunk = vec![0u8; avail as usize];
            let mut filled = 0usize;
            while filled < chunk.len() {
                match reader.read(&mut chunk[filled..]) {
                    Ok(0) => return Some(chunk[..filled].to_vec()),
                    Ok(n) => filled += n,
                    Err(_) => return Some(chunk[..filled].to_vec()),
                }
            }
            return Some(chunk);
        }
    }
    #[cfg(not(windows))]
    {
        use std::io::BufRead;
        let mut buf = Vec::with_capacity(8192);
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => None,
            Ok(_) => Some(buf),
            Err(_) => None,
        }
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

        // A real interactive child: `cmd /C findstr x` forwards lines
        // containing "x" from its stdin to its stdout (the cancellable
        // write is this module's half of F-815-T05-R1-02).
        let spec = TrackedSpawn {
            windows_cmd_shim: true,
            ..TrackedSpawn::new("findstr", vec!["x".to_string()])
        };
        let tracked = spawn_tracked(spec, "dev-server").expect("spawn real child");
        assert_eq!(tracked.id, tracked.id);
        // Direct foreground write (writer-thread decoupling is host-level).
        assert!(tracked.write_input_stoppable(b"hello x line\n").is_ok());
        assert!(tracked.write_input_stoppable(b"plain line\n").is_ok());
        tracked.close_stdin(); // EOF so findstr terminates on its own

        let status = wait_tracked(&tracked).expect("wait real child");
        assert!(status.success(), "findstr should exit 0, got {status:?}");

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
        // terminated and reaped as a whole before return.
        let spec = TrackedSpawn {
            windows_cmd_shim: true,
            ..TrackedSpawn::new(
                "cmd",
                vec!["/C".to_string(), "cmd /C ping -n 30 127.0.0.1".to_string()],
            )
        };
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
}
