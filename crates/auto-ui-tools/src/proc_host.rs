//! PLAN-815 T-05 hosted process/IO plumbing for the tool host.
//!
//! The migrated run paths (vue / vm / rust) keep their legacy behavior when
//! no [`RunIo`] is installed: children inherit stdio, steps `println!`.
//! When the tool host installs a [`RunIo`] for a work operation, the same
//! code paths instead:
//!
//! - spawn children with **piped stdout** (forwarded chunk-wise to the host
//!   log stream), **piped stdin** (the `Input` protocol message forwards
//!   bytes here — the Vite dev server genuinely consumes `q` + Enter), and
//!   inherited stderr (the log stream after the bin's stdio split);
//! - register every owned child so `cancel` kills them and the wait loops
//!   observe cancellation;
//! - emit progress/launched events through host-installed hooks (the host
//!   enforces the per-op progress budget before anything reaches the
//!   control stream);
//! - announce the dev-server URL sniffed from real child stdout.
//!
//! Protocol-agnostic by construction: the hooks are plain closures, so this
//! module never depends on `auto-tool-protocol`.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// Poll interval for cancel-aware waits.
const WAIT_POLL: std::time::Duration = std::time::Duration::from_millis(50);

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

/// One owned child of a hosted operation.
pub struct TrackedChild {
    child: Mutex<Option<Child>>,
    stdin: Mutex<Option<ChildStdin>>,
    /// Caller-assigned role (`install`, `dev-server`, `api-server`, …) —
    /// used for honest diagnostics only.
    pub label: String,
    pub id: u32,
}

impl TrackedChild {
    /// Kill the child if it is still live (idempotent).
    pub fn kill(&self) {
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = child.kill();
        }
    }

    pub fn try_wait(&self) -> std::io::Result<Option<ExitStatus>> {
        match self.child.lock().unwrap().as_mut() {
            Some(child) => child.try_wait(),
            None => Ok(None),
        }
    }

    /// Write bytes into the child's stdin (the `Input` channel target).
    pub fn write_input(&self, data: &[u8]) -> std::io::Result<()> {
        let mut guard = self.stdin.lock().unwrap();
        match guard.as_mut() {
            Some(stdin) => stdin.write_all(data).and_then(|_| stdin.flush()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "child stdin already closed",
            )),
        }
    }

    #[cfg(test)]
    fn take_stdin(&self) -> Option<ChildStdin> {
        self.stdin.lock().unwrap().take()
    }
}

/// Per-work-operation hosted IO state. Installed via [`install`] while the
/// operation executes (single work operation at a time — the host enforces
/// this before installing) and removed by [`uninstall`].
pub struct RunIo {
    cancelled: AtomicBool,
    children: Mutex<Vec<Arc<TrackedChild>>>,
    foreground: Mutex<Option<Arc<TrackedChild>>>,
    /// `(done, total, message)` progress hook (host enforces budgets).
    pub on_progress: Mutex<Option<Box<dyn Fn(u64, Option<u64>, &str) + Send>>>,
    /// Launched-spec hook (service URL).
    pub on_launched: Mutex<Option<Box<dyn Fn(&str) + Send>>>,
    /// Child stdout chunk hook (host log stream).
    pub on_log_chunk: Mutex<Option<Box<dyn Fn(&[u8]) + Send>>>,
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
            on_progress: Mutex::new(None),
            on_launched: Mutex::new(None),
            on_log_chunk: Mutex::new(None),
            on_vm_main: Mutex::new(None),
        }
    }

    /// Cooperative cancellation flag (checkpoints + wait loops consult it).
    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Signal cancellation and kill every owned child process.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let children = self.children.lock().unwrap();
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

/// Remove the hosted IO, killing any still-live owned children (leak guard
/// — the honest answer for flows that lost their own cleanup path) and
/// dropping the hook slots: the progress/launched/log closures capture
/// strong Arcs (to this RunIo and, via the host, to the shared channel
/// state), so keeping them installed after the operation ends would leak
/// the whole cycle and the tool host would never observe channel closure.
pub fn uninstall() {
    let io = slot().lock().unwrap().take();
    if let Some(io) = io {
        io.cancel();
        *io.on_progress.lock().unwrap() = None;
        *io.on_launched.lock().unwrap() = None;
        *io.on_log_chunk.lock().unwrap() = None;
        *io.on_vm_main.lock().unwrap() = None;
    }
}

/// The currently installed hosted IO, when any.
pub fn current() -> Option<Arc<RunIo>> {
    slot().lock().unwrap().clone()
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

/// Spawn `command` as an owned child of the hosted operation and start its
/// stdout forwarder. The caller must have [`install`]ed a [`RunIo`].
pub fn spawn_tracked(mut command: Command, label: &str) -> std::io::Result<Arc<TrackedChild>> {
    let io = current().expect("spawn_tracked called without installed RunIo");
    // Hosted stdio policy: piped stdout (forwarded to the log stream),
    // piped stdin (Input channel), inherited stderr (log stream after the
    // bin's stdio split). Nothing a child writes can reach the control
    // stream, and no child can consume the framed protocol stdin.
    command.stdout(Stdio::piped()).stdin(Stdio::piped());
    let mut child = command.spawn()?;
    let pid = child.id();
    let child_stdin = child.stdin.take();
    let child_stdout = child.stdout.take();
    let tracked = Arc::new(TrackedChild {
        child: Mutex::new(Some(child)),
        stdin: Mutex::new(child_stdin),
        label: label.to_string(),
        id: pid,
    });
    io.register(Arc::clone(&tracked));
    if let Some(stdout) = child_stdout {
        let forward_label = label.to_string();
        std::thread::Builder::new()
            .name(format!("child-out:{forward_label}"))
            .spawn(move || forward_stdout(stdout, &forward_label))
            .ok();
    }
    Ok(tracked)
}

/// Forward a child's piped stdout to the host log stream, sniffing the
/// first announced service URL on dev-server children. The thread exits on
/// EOF — i.e. when the child dies — which is the reader-exit half of the
/// cancel contract.
fn forward_stdout(stdout: impl Read, label: &str) {
    let mut reader = BufReader::new(stdout);
    let mut announced = false;
    let mut buf = Vec::with_capacity(8192);
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        log_chunk(&buf);
        if !announced && (label == "dev-server" || label == "api-server") {
            if let Some(url) = sniff_url(&String::from_utf8_lossy(&buf)) {
                launched(&url);
                announced = true;
            }
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

/// Cancel-aware wait for a tracked child: polls `try_wait`, killing the
/// child once cancellation is signalled. Returns the exit status.
pub fn wait_tracked(tracked: &TrackedChild) -> std::io::Result<ExitStatus> {
    loop {
        if let Some(status) = tracked.try_wait()? {
            return Ok(status);
        }
        if cancelled() {
            tracked.kill();
        }
        std::thread::sleep(WAIT_POLL);
    }
}

/// Forward `data` to the foreground child's stdin. Errors when no hosted
/// operation or no foreground child with a live stdin is present — the host
/// turns that into the honest named protocol error.
pub fn write_input(data: &[u8]) -> Result<(), String> {
    let io = current().ok_or_else(|| "no work operation in flight".to_string())?;
    match io.foreground() {
        Some(child) => child
            .write_input(data)
            .map_err(|e| format!("foreground child stdin unavailable: {e}")),
        None => Err("operation has no interactive child process".to_string()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The RunIo slot is process-global; the two tests that touch it must
    /// not interleave under a single-process libtest run.
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

    /// Real child forwarding + Input + cancel: a real OS child echoes
    /// matching stdin lines to stdout; the hosted forwarder delivers its
    /// output to the log hook; cancel kills a blocking child promptly.
    /// Real processes only — no fakes.
    #[test]
    fn real_child_input_forward_and_cancel() {
        let _guard = SLOT_LOCK.lock().unwrap();
        use std::io::Write as _;
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
        // containing "x" from its stdin to its stdout.
        let mut command = Command::new("cmd");
        command.args(["/C", "findstr", "x"]);
        let tracked = spawn_tracked(command, "dev-server").expect("spawn real child");

        // Input channel forwards real bytes into the real child's stdin.
        {
            let mut stdin = tracked.take_stdin().expect("child stdin");
            stdin
                .write_all(b"hello x line\nplain line\n")
                .expect("write child stdin");
        } // EOF so findstr terminates on its own

        let status = wait_tracked(&tracked).expect("wait real child");
        assert!(status.success(), "findstr should exit 0, got {status:?}");

        // The forwarder thread drains the pipe asynchronously.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
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

        // Cancel path: a second real child that blocks ~30s unless killed.
        let mut command = Command::new("cmd");
        command.args(["/C", "ping", "-n", "30", "127.0.0.1"]);
        let tracked = spawn_tracked(command, "api-server").expect("spawn blocking child");
        io.cancel();
        let start = std::time::Instant::now();
        let _ = wait_tracked(&tracked);
        assert!(
            start.elapsed() < std::time::Duration::from_secs(10),
            "cancel must kill the owned child promptly"
        );
        uninstall();
    }
}
