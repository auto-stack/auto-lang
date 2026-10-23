//! PLAN-815 T-05: the threaded tool host (R1 rework).
//!
//! Architecture (M815-B):
//!
//! - a **protocol loop thread** owns the framed stdin, answers the bounded
//!   A-stage operations inline (describe/validate — semantics unchanged),
//!   and dispatches the B-stage work operations (`generate_ui`, `run_ui`)
//!   onto worker threads;
//! - the **OS main thread** becomes the UI executor: the in-process VM UI
//!   (iced) must run there, so `run_ui` hands the entry off through
//!   [`MainJob`] and waits for the session result; cancel/EOF terminate
//!   the iced loop via the runtime's host-exit request
//!   (F-815-T05-R1-03), so the session returns and the worker finishes;
//! - one **work operation in flight at a time** (process-global run paths
//!   and env channels make this the honest concurrency contract; a second
//!   concurrent work op gets a named busy error before any side effect);
//! - `run_ui` is cancellable (owned child TREES are terminated and reaped
//!   before the final receipt, F-815-T05-R1-01) and interactive (the
//!   `Input` protocol message queues bytes for the foreground child's
//!   stdin; the blocking write runs on a dedicated writer thread, so the
//!   protocol loop stays responsive to cancel/describe/EOF no matter how
//!   un-readingly the child behaves, F-815-T05-R1-02);
//! - child stdout is forwarded to the host **log** stream; the control
//!   stream stays framed JSON by construction;
//! - the per-op progress budget is enforced host-side before anything
//!   reaches the control stream (overflow emits `BudgetExceeded` and
//!   aborts the operation like a cancel);
//! - control-write failures anywhere (loop, workers, hooks, input writer)
//!   record the internal exit code, and a later clean EOF cannot override
//!   a recorded non-zero status (F-815-T05-R1-04).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use auto_tool_protocol::budget::Budgets;
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, ToolEvent, WireRequest,
};
use auto_tool_protocol::{
    op_options, Capability, LifecyclePolicy, ToolDescriptor, ToolError, EXIT_CODE_FATAL_STREAM,
    EXIT_CODE_INTERNAL, EXIT_CODE_OK, PROTOCOL_VERSION,
};

use crate::proc_host;
use crate::validate;

/// Renders the `run_ui` operation serves in this build (T-05 scope; the
/// full profile matrix lands with T-06/T-07 — other renders get a named
/// rejection before any side effect).
pub const RUN_UI_SERVED_PROFILES: &[&str] = &["vue", "vm", "rust", "jet", "ark", "tauri", "vscode"];

/// Renders the `generate_ui` operation serves in this build (T-06: full
/// matrix — every generator closure lives in this crate since T-04).
pub const GENERATE_UI_SERVED_PROFILES: &[&str] = &["vue", "rust", "jet", "ark", "tauri", "vscode"];

/// Renders the `build_ui` operation serves in this build (T-06). `vm` has
/// no separate build step (interpreted) — builds serve the compiled
/// frontends only.
pub const BUILD_UI_SERVED_PROFILES: &[&str] = &["vue", "rust", "jet", "ark", "vscode"];

/// The B-stage capability list: bounded A operations plus the real
/// execution operations. `run_ui` is honestly cancellable and interactive;
/// generation is bounded and non-cancellable.
pub fn tool_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        tool_name: crate::TOOL_NAME.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        protocol_version: PROTOCOL_VERSION,
        capabilities: vec![
            Capability::non_interactive("describe", false),
            Capability::non_interactive("validate_project", false),
            // F-816-T05-R2-04：generate_ui/build_ui/run_ui 广播各自兑现
            // 的 op 级执行选项（op_options 单源）——消费方据此对旧 v1
            // 工具（无此广播）携带选项的请求在副作用前具名拒绝。
            Capability::with_options(
                "generate_ui",
                false,
                false,
                &[
                    op_options::OUTPUT,
                    op_options::PROJECT,
                    op_options::DELIVERY_ROOT,
                    op_options::STRICT,
                ],
            ),
            Capability::with_options("build_ui", true, false, &[op_options::STRICT]),
            Capability::with_options("run_ui", true, true, &[op_options::ARGS]),
        ],
        lifecycle: LifecyclePolicy::contract(),
    }
}

struct InFlight {
    id: u64,
    operation: &'static str,
    io: Arc<proc_host::RunIo>,
}

#[derive(Default)]
struct Registry {
    served: HashMap<u64, &'static str>,
    in_flight: Option<InFlight>,
}

enum Lookup {
    InFlight(&'static str, Arc<proc_host::RunIo>),
    Served(&'static str),
    Unknown,
}

impl Registry {
    fn lookup(&self, id: u64) -> Lookup {
        if let Some(flight) = &self.in_flight {
            if flight.id == id {
                return Lookup::InFlight(flight.operation, Arc::clone(&flight.io));
            }
        }
        match self.served.get(&id) {
            Some(op) => Lookup::Served(op),
            None => Lookup::Unknown,
        }
    }
}

struct HostShared<W: Write, L: Write> {
    control: Mutex<W>,
    log: Mutex<L>,
    max_frame_bytes: usize,
    ops: Mutex<Registry>,
    /// Shutdown signal: held by every HostShared Arc (loop thread +
    /// workers); the main run_tool frame waits on its receiver until every
    /// holder drops (T-05 R2: the VM session itself runs in an isolated
    /// child process — no main-thread executor jobs remain). Never sent
    /// to — the Sender's lifetime IS the signal.
    #[allow(dead_code)]
    shutdown_tx: Mutex<mpsc::Sender<()>>,
    exit_code: Arc<AtomicI32>,
}

impl<W: Write, L: Write> HostShared<W, L> {
    fn log_line(&self, message: &str) {
        let _ = writeln!(self.log.lock().unwrap(), "{message}");
    }

    /// Record an exit status (F-815-T05-R1-04): **non-zero wins** — a clean
    /// EOF (0) recorded after a worker/hook control-write failure must not
    /// downgrade the process exit to success.
    fn record_exit(&self, code: i32) {
        if code != EXIT_CODE_OK {
            self.exit_code.store(code, Ordering::SeqCst);
        }
    }

    /// Serialize and write one envelope on the control stream. Frame-level
    /// write failures are fatal (internal) and are recorded so the final
    /// exit code reflects them even when the loop later ends on clean EOF.
    fn respond(&self, envelope: &ServerEnvelope) -> Result<(), i32> {
        let json = serde_json::to_value(envelope).map_err(|e| {
            self.log_line(&format!("fatal: response serialization failed: {e}"));
            EXIT_CODE_INTERNAL
        })?;
        let mut control = self.control.lock().unwrap();
        auto_tool_protocol::framing::write_frame(&mut *control, &json, self.max_frame_bytes)
            .map_err(|e| {
                self.log_line(&format!("fatal: control stream write failed: {e}"));
                EXIT_CODE_INTERNAL
            })
    }

    /// Respond and treat fatal failures as host-fatal (F04, R2): the
    /// control stream is broken — record the non-zero code AND terminate
    /// the process immediately (a client that keeps stdin open must not
    /// keep a dead-on-paper host alive).
    fn respond_recorded(&self, envelope: &ServerEnvelope) {
        if let Err(code) = self.respond(envelope) {
            self.record_exit(code);
            self.fatal_shutdown(code);
        }
    }

    /// Host-fatal shutdown (F04): cancel in-flight work, record, exit —
    /// no further serving is possible once the control stream rejected a
    /// write.
    fn fatal_shutdown(&self, code: i32) {
        if let Some(flight) = self.ops.lock().unwrap().in_flight.take() {
            flight.io.cancel();
        }
        eprintln!("auto-ui-tool exiting with code {code}");
        std::process::exit(code);
    }

    /// Enqueue an event envelope for an in-flight operation's id.
    fn emit_event(&self, id: u64, event: ToolEvent) -> Result<(), i32> {
        self.respond(&ServerEnvelope {
            id,
            body: ServerBody::Event { event },
        })
    }
}

/// Run the tool host over framed stdio until EOF (PLAN-815 T-05 threaded
/// form; the caller-visible contract — signatures, exit codes, A-stage
/// synchronous semantics — is unchanged from M815-A).
///
/// - control stream (`control`) carries framed JSON only;
/// - `log` receives free-form human diagnostics and forwarded child output;
/// - exit codes follow the frozen contract (0 EOF / 1 internal / 2 stream),
///   with any recorded non-zero status taking precedence over a later
///   clean EOF (F-815-T05-R1-04).
pub fn run_tool<R, W, L>(input: R, control: W, log: L, max_frame_bytes: usize) -> i32
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    L: Write + Send + 'static,
{
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
    let exit_code = Arc::new(AtomicI32::new(EXIT_CODE_OK));
    let host = Arc::new(HostShared {
        control: Mutex::new(control),
        log: Mutex::new(log),
        max_frame_bytes,
        ops: Mutex::new(Registry::default()),
        shutdown_tx: Mutex::new(shutdown_tx),
        exit_code: Arc::clone(&exit_code),
    });

    let loop_host = Arc::clone(&host);
    let loop_thread = std::thread::Builder::new()
        .name("i-tool-loop".into())
        .spawn(move || {
            let code = serve_loop(input, Arc::clone(&loop_host));
            // EOF/fatal shutdown: cancel any in-flight work (terminates
            // owned trees + requests the VM iced exit) so workers finish;
            // their final envelopes may still write before the process
            // goes away.
            if let Some(flight) = loop_host.ops.lock().unwrap().in_flight.take() {
                flight.io.cancel();
            }
            // A clean EOF only sticks when nothing recorded a failure.
            loop_host.record_exit(code);
        })
        .expect("spawn protocol loop thread");

    // The main frame must NOT hold the last Arc: the recv loop below ends
    // when every HostShared handle (loop thread + workers, each carrying a
    // main_tx clone) drops — keeping one alive here would deadlock at EOF.
    drop(host);

    // Shutdown signal: ends when every HostShared holder (loop thread +
    // workers) drops. The VM session itself runs in an isolated
    // `--internal-vm-exec` child process (T-05 R2 / R2-01) — iced/winit's
    // one-event-loop-per-process constraint is satisfied by construction.
    while shutdown_rx.recv().is_ok() {}
    // Channel closed: loop thread and all workers finished.
    let code = exit_code.load(Ordering::SeqCst);
    let _ = loop_thread.join();
    code
}

fn serve_loop<R: Read, W: Write + Send + 'static, L: Write + Send + 'static>(
    mut input: R,
    host: Arc<HostShared<W, L>>,
) -> i32 {
    loop {
        let frame = match auto_tool_protocol::framing::read_frame(&mut input, host.max_frame_bytes)
        {
            Ok(Some(frame)) => frame,
            Ok(None) => return EXIT_CODE_OK,
            Err(e) => {
                host.log_line(&format!("fatal: stream violation: {e}"));
                return EXIT_CODE_FATAL_STREAM;
            }
        };
        let (id, body) = match classify_and_parse(&frame, &host) {
            Ok(parsed) => parsed,
            Err(error) => {
                let id = frame.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                if let Err(e) = host.respond(&ServerEnvelope::error(id, error)) {
                    host.record_exit(e);
                    return e;
                }
                continue;
            }
        };
        host.log_line(&format!("accepted request id={id} op={}", body.operation()));

        // Input may defer its receipt to the writer thread (F02): None
        // means queued — the writer responds with InputAck/Error later.
        let deferred_response: Option<ServerEnvelope> = match body {
            ClientBody::Describe => {
                // Bounded inline ops are recorded as served so later
                // cancel/input target them honestly (NotCancellable /
                // already-completed) instead of "unknown target".
                host.ops.lock().unwrap().served.insert(id, "describe");
                Some(ServerEnvelope {
                    id,
                    body: ServerBody::DescribeOk {
                        descriptor: tool_descriptor(),
                    },
                })
            }
            ClientBody::ValidateProject { request } => {
                let report = validate::validate_request(&request.0);
                host.ops
                    .lock()
                    .unwrap()
                    .served
                    .insert(id, "validate_project");
                Some(ServerEnvelope {
                    id,
                    body: ServerBody::ValidateOk { report },
                })
            }
            ClientBody::GenerateUi {
                request,
                output,
                project,
                delivery_root,
                strict,
            } => {
                match start_work_op(&host, id, "generate_ui") {
                    Ok(worker) => {
                        worker.spawn_generate(request, output, project, delivery_root, strict);
                        None // response comes from the worker
                    }
                    Err(error) => Some(ServerEnvelope::error(id, error)),
                }
            }
            ClientBody::RunUi { request, args } => match start_work_op(&host, id, "run_ui") {
                Ok(worker) => {
                    worker.spawn_run(request, args);
                    None // response comes from the worker
                }
                Err(error) => Some(ServerEnvelope::error(id, error)),
            },
            ClientBody::BuildUi { request, strict } => match start_work_op(&host, id, "build_ui") {
                Ok(worker) => {
                    worker.spawn_build(request, strict);
                    None // response comes from the worker
                }
                Err(error) => Some(ServerEnvelope::error(id, error)),
            },
            ClientBody::Input { target_id, data } => handle_input(&host, id, target_id, data),
            ClientBody::Cancel { target_id } => Some(handle_cancel(&host, id, target_id)),
        };
        if let Some(response) = deferred_response {
            if let Err(e) = host.respond(&response) {
                host.record_exit(e);
                return e;
            }
        }
    }
}

/// Pre-classify a raw frame into precise error kinds before the typed parse
/// (unchanged A-stage discipline, extended to the new operations): missing
/// envelope fields, unknown protocol version, then full serde parse
/// (residual failures map to `invalid-field`).
fn classify_and_parse(
    frame: &serde_json::Value,
    host: &HostShared<impl Write, impl Write>,
) -> Result<(u64, ClientBody), ToolError> {
    let object = frame
        .as_object()
        .ok_or_else(|| ToolError::invalid_field("envelope", "frame must be a JSON object"))?;
    for field in ["protocol_version", "id", "body"] {
        if !object.contains_key(field) {
            return Err(ToolError::missing_field(field));
        }
    }
    let version = frame["protocol_version"]
        .as_u64()
        .ok_or_else(|| ToolError::invalid_field("protocol_version", "must be a number"))?;
    if version != PROTOCOL_VERSION as u64 {
        host.log_line(&format!(
            "rejected request: protocol version {version} (server: {PROTOCOL_VERSION})"
        ));
        return Err(ToolError::unknown_protocol_version(
            version,
            PROTOCOL_VERSION,
        ));
    }
    let body = frame["body"]
        .as_object()
        .ok_or_else(|| ToolError::invalid_field("body", "must be an object"))?;
    if !body.contains_key("op") {
        return Err(ToolError::missing_field("op"));
    }
    let op = frame["body"]["op"]
        .as_str()
        .ok_or_else(|| ToolError::invalid_field("op", "must be a string"))?;
    // Version/capability rejection BEFORE any work or side effect.
    // `cancel`/`input` are protocol-level control messages, not work
    // operations: they always parse and are answered per their target —
    // the work operations are gated by the capability list.
    if op != "cancel" && op != "input" && !tool_descriptor().offers(op) {
        return Err(ToolError::unknown_operation(op));
    }
    let envelope: ClientEnvelope = serde_json::from_value(frame.clone()).map_err(|e| {
        ToolError::invalid_field(
            "envelope",
            format!("payload does not match the protocol: {e}"),
        )
    })?;
    Ok((envelope.id, envelope.body))
}

struct WorkStarter<W: Write, L: Write> {
    host: Arc<HostShared<W, L>>,
    id: u64,
    operation: &'static str,
    launched: Arc<Mutex<Option<String>>>,
}

/// Install the single-flight hosted IO and register the operation. Errors
/// (busy) are answered by the caller BEFORE any side effect.
fn start_work_op<W: Write + Send + 'static, L: Write + Send + 'static>(
    host: &Arc<HostShared<W, L>>,
    id: u64,
    operation: &'static str,
) -> Result<WorkStarter<W, L>, ToolError> {
    if !proc_host::install() {
        return Err(ToolError::invalid_field(
            "request",
            "another work operation is already in flight; cancel it or wait for completion",
        ));
    }
    let io = proc_host::current().expect("RunIo just installed");
    let launched = Arc::new(Mutex::new(None));

    // Progress hook with host-side budget enforcement; a control-write
    // failure here records the internal exit and cancels the op (F04).
    {
        let host = Arc::clone(host);
        let budget = Arc::new(AtomicU64::new(0));
        let overflow = Arc::new(AtomicBool::new(false));
        let hook_io = Arc::clone(&io);
        let progress_hook = Box::new(move |done: u64, total: Option<u64>, message: &str| {
            let event = ToolEvent::Progress {
                message: message.to_string(),
                done,
                total,
            };
            let envelope = ServerEnvelope {
                id,
                body: ServerBody::Event { event },
            };
            let bytes = serde_json::to_vec(&envelope)
                .map(|v| v.len() as u64)
                .unwrap_or(u64::MAX);
            let observed = budget.fetch_add(bytes, Ordering::SeqCst) + bytes;
            let limit = Budgets::contract().max_progress_bytes_per_op as u64;
            if observed > limit {
                if !overflow.swap(true, Ordering::SeqCst) {
                    if let Err(code) = host.respond(&ServerEnvelope::error(
                        id,
                        ToolError::budget_exceeded("max_progress_bytes_per_op", limit, observed),
                    )) {
                        host.record_exit(code);
                        host.fatal_shutdown(code);
                    }
                    // Abort like a cancel: trees die, flow checkpoints
                    // return Cancelled.
                    hook_io.cancel();
                }
                return;
            }
            if let Err(code) = host.respond(&envelope) {
                host.record_exit(code);
                hook_io.cancel();
                host.fatal_shutdown(code);
            }
        });
        *io.on_progress.lock().unwrap() = Some(progress_hook);
    }

    // Launched hook: record the URL and announce it on the control stream.
    {
        let host = Arc::clone(host);
        let launched = Arc::clone(&launched);
        *io.on_launched.lock().unwrap() = Some(Box::new(move |url| {
            *launched.lock().unwrap() = Some(url.to_string());
            if let Err(code) = host.emit_event(
                id,
                ToolEvent::Launched {
                    url: url.to_string(),
                },
            ) {
                host.record_exit(code);
                host.fatal_shutdown(code);
            }
        }));
    }

    // Log hook: forwarded child stdout lands on the host log stream only.
    {
        let host = Arc::clone(host);
        *io.on_log_chunk.lock().unwrap() = Some(Box::new(move |bytes: &[u8]| {
            let mut log = host.log.lock().unwrap();
            let _ = log.write_all(bytes);
            let _ = log.flush();
        }));
    }

    // Input enqueue hook (F-815-T05-R1-02): a bounded queue feeds a
    // dedicated writer thread — the protocol loop never blocks on a
    // non-reading child's stdin, and the InputAck/Error receipt comes
    // from the writer after the real (cancellable) write.
    {
        let (input_tx, input_rx) =
            mpsc::sync_channel::<(u64, u64, Vec<u8>)>(proc_host::INPUT_QUEUE_DEPTH);
        {
            let enqueue_io = Arc::clone(&io);
            *io.on_input_enqueue.lock().unwrap() = Some(Box::new(
                move |request_id: u64, target_id: u64, data: Vec<u8>| {
                    // Keep the enqueue honest about the current foreground.
                    if enqueue_io.cancelled() {
                        return Err("operation is being cancelled".to_string());
                    }
                    input_tx
                        .try_send((request_id, target_id, data))
                        .map_err(|_| {
                            format!(
                                "input queue full ({} pending writes); retry after the child drains",
                                proc_host::INPUT_QUEUE_DEPTH
                            )
                        })
                },
            ));
        }
        let writer_host = Arc::clone(host);
        let writer_handle = std::thread::Builder::new()
            .name(format!("work-input:{id}"))
            .spawn(move || {
                for (request_id, target_id, data) in input_rx {
                    let response = match proc_host::write_input_stoppable(&data) {
                        Ok(()) => ServerEnvelope {
                            id: request_id,
                            body: ServerBody::InputAck {
                                target_id,
                                bytes: data.len() as u64,
                            },
                        },
                        Err(reason) => ServerEnvelope::error(
                            request_id,
                            ToolError::invalid_field("data", reason),
                        ),
                    };
                    if let Err(code) = writer_host.respond(&response) {
                        writer_host.record_exit(code);
                        writer_host.fatal_shutdown(code);
                    }
                }
            })
            .expect("spawn input writer thread");
        *io.input_writer.lock().unwrap() = Some(writer_handle);
    }

    // VM session hook (R2-01): every VM run executes in an ISOLATED child
    // process of this very binary (`--internal-vm-exec`) — iced/winit
    // allows one event loop per process, so process isolation is what
    // makes serial VM sessions (cancel → run again → …) work without
    // RecreationAttempt. The child is an owned tracked tree: cancel/EOF
    // kills it, its stdout forwards to the log stream (URL sniffing
    // announces the session's MCP endpoint), and its stdin is fed by the
    // `Input` channel (`q` quits the session for real).
    {
        *io.on_vm_main.lock().unwrap() = Some(Arc::new(move |code: &str, path: Option<&str>| {
            let _ = code; // the child reads the entry from disk itself
            let entry = path
                .map(str::to_string)
                .unwrap_or_else(|| "src/front/app.at".to_string());
            let exe = std::env::current_exe().map_err(|e| {
                auto_lang_core::error::AutoError::Msg(format!(
                    "vm session exe resolution failed: {e}"
                ))
            })?;
            let mut spec = crate::proc_host::TrackedSpawn::new(
                exe.to_string_lossy(),
                vec!["--internal-vm-exec".to_string(), entry],
            );
            // T-05 R4（F-815-T05-R4-01）：会话子进程只获得**本请求**的
            // 上下文——内部上下文键先从继承环境移除，再应用当前请求
            // 衍生的条目（外部后端 None → 本地、主题 Some → None、失败
            // 后新会话均不沿用上一请求）。
            spec = spec
                .without_env("AUTO_UI_TOOL_VM_BACK_ROOT")
                .without_env("AUTO_UI_TOOL_VM_THEME_DECL");
            if let Some(io) = crate::proc_host::current() {
                for (key, value) in io.vm_session_env.lock().unwrap().iter() {
                    spec = spec.with_env(key, value);
                }
            }
            // Pipe the child's stderr too: the session's real MCP endpoint
            // banner is a stderr eprintln in the runtime — forwarding and
            // sniffing it is R2-03's accurate-URL source.
            spec.pipe_stderr = true;
            let tracked = crate::proc_host::spawn_tracked(spec, "vm-session").map_err(|e| {
                auto_lang_core::error::AutoError::Msg(format!("vm session spawn failed: {e}"))
            })?;
            let status = crate::proc_host::wait_tracked(&tracked).map_err(|e| {
                auto_lang_core::error::AutoError::Msg(format!("vm session wait failed: {e}"))
            })?;
            if status.success() {
                Ok("vm session closed".to_string())
            } else {
                // R4-02: a required external backend load failure (or any
                // other child-fatal) is named — the child writes a
                // `VM_SESSION_FATAL:` line its stderr forwarder captured.
                let reason = crate::proc_host::vm_session_fatal();
                Err(auto_lang_core::error::AutoError::Msg(match reason {
                    Some(reason) => format!("vm session failed: {reason}"),
                    None => format!("vm session exited with {status}"),
                }))
            }
        }));
    }

    {
        let mut ops = host.ops.lock().unwrap();
        ops.served.insert(id, operation);
        ops.in_flight = Some(InFlight {
            id,
            operation,
            io: Arc::clone(&io),
        });
    }

    Ok(WorkStarter {
        host: Arc::clone(host),
        id,
        operation,
        launched,
    })
}

impl<W: Write + Send + 'static, L: Write + Send + 'static> WorkStarter<W, L> {
    fn spawn_generate(
        self,
        request: WireRequest,
        output: Option<std::path::PathBuf>,
        project: Option<bool>,
        delivery_root: Option<std::path::PathBuf>,
        strict: Option<bool>,
    ) {
        let WorkStarter {
            host,
            id,
            operation,
            launched: _,
        } = self;
        std::thread::Builder::new()
            .name(format!("work:{operation}"))
            .spawn(move || {
                let started = std::time::Instant::now();
                let options = crate::tool_ops::GenerateOptions {
                    output,
                    project,
                    delivery_root,
                    strict,
                };
                let result = crate::tool_ops::generate_report_with(&request.0, &options);
                // Uninstall first: leftover trees die, hooks go quiet
                // (which also ends the input writer).
                proc_host::uninstall();
                let response = match result {
                    Ok(mut report) => {
                        report.duration_ms = started.elapsed().as_millis() as u64;
                        ServerEnvelope {
                            id,
                            body: ServerBody::GenerateOk { report },
                        }
                    }
                    Err(error) => ServerEnvelope::error(id, error),
                };
                host.respond_recorded(&response);
                finish_op(&host, id);
            })
            .expect("spawn work thread");
    }

    fn spawn_build(self, request: WireRequest, strict: Option<bool>) {
        let WorkStarter {
            host,
            id,
            operation,
            launched: _,
        } = self;
        std::thread::Builder::new()
            .name(format!("work:{operation}"))
            .spawn(move || {
                let started = std::time::Instant::now();
                let result = crate::tool_ops::build_report_with(&request.0, strict);
                proc_host::uninstall();
                let response = match result {
                    Ok(mut report) => {
                        report.duration_ms = started.elapsed().as_millis() as u64;
                        ServerEnvelope {
                            id,
                            body: ServerBody::BuildOk { report },
                        }
                    }
                    Err(error) => ServerEnvelope::error(id, error),
                };
                host.respond_recorded(&response);
                finish_op(&host, id);
            })
            .expect("spawn work thread");
    }

    fn spawn_run(self, request: WireRequest, args: Vec<String>) {
        let WorkStarter {
            host,
            id,
            operation,
            launched,
        } = self;
        std::thread::Builder::new()
            .name(format!("work:{operation}"))
            .spawn(move || {
                let started = std::time::Instant::now();
                let result = crate::tool_ops::run_report_with(&request.0, args);
                proc_host::uninstall();
                let response = match result {
                    Ok(mut report) => {
                        report.duration_ms = started.elapsed().as_millis() as u64;
                        report.launch = launched.lock().unwrap().clone();
                        ServerEnvelope {
                            id,
                            body: ServerBody::RunOk { report },
                        }
                    }
                    Err(error) => ServerEnvelope::error(id, error),
                };
                host.respond_recorded(&response);
                finish_op(&host, id);
            })
            .expect("spawn work thread");
    }
}

/// Move the operation from in-flight to served (completed).
fn finish_op<W: Write, L: Write>(host: &Arc<HostShared<W, L>>, id: u64) {
    let mut ops = host.ops.lock().unwrap();
    if ops.in_flight.as_ref().map(|f| f.id == id).unwrap_or(false) {
        ops.in_flight = None;
    }
}

/// Queue input bytes for the target's foreground child (F02). None = the
/// writer thread will deliver the receipt; Some = immediate named error.
fn handle_input<W: Write, L: Write>(
    host: &Arc<HostShared<W, L>>,
    id: u64,
    target_id: u64,
    data: Vec<u8>,
) -> Option<ServerEnvelope> {
    match host.ops.lock().unwrap().lookup(target_id) {
        Lookup::InFlight(operation, _io) => {
            if operation != "run_ui" && operation != "build_ui" {
                return Some(ServerEnvelope::error(
                    id,
                    ToolError::invalid_field(
                        "data",
                        format!("operation '{operation}' has no interactive child process"),
                    ),
                ));
            }
            match proc_host::enqueue_input(id, target_id, data) {
                Ok(()) => None, // receipt comes from the writer thread
                Err(reason) => Some(ServerEnvelope::error(
                    id,
                    ToolError::invalid_field("data", reason),
                )),
            }
        }
        Lookup::Served(_) => Some(ServerEnvelope::error(
            id,
            ToolError::invalid_field(
                "target_id",
                format!("operation {target_id} already completed"),
            ),
        )),
        Lookup::Unknown => Some(ServerEnvelope::error(
            id,
            ToolError::invalid_field(
                "target_id",
                format!("no in-flight or completed operation with id {target_id}"),
            ),
        )),
    }
}

fn handle_cancel<W: Write, L: Write>(
    host: &Arc<HostShared<W, L>>,
    id: u64,
    target_id: u64,
) -> ServerEnvelope {
    match host.ops.lock().unwrap().lookup(target_id) {
        Lookup::InFlight(operation, io) => {
            if operation == "run_ui" || operation == "build_ui" {
                // Terminates owned trees, stops in-flight input writes,
                // and requests the VM iced exit (F01/F03).
                io.cancel();
                ServerEnvelope {
                    id,
                    body: ServerBody::Cancelled { target_id },
                }
            } else {
                // Bounded operation: honest per-operation answer.
                ServerEnvelope::error(id, ToolError::not_cancellable(operation))
            }
        }
        Lookup::Served(op) => ServerEnvelope::error(id, ToolError::not_cancellable(op)),
        Lookup::Unknown => ServerEnvelope::error(
            id,
            ToolError::invalid_field(
                "target_id",
                format!("no in-flight or completed operation with id {target_id}"),
            ),
        ),
    }
}
