//! PLAN-815 T-05: the threaded tool host.
//!
//! Architecture (M815-B):
//!
//! - a **protocol loop thread** owns the framed stdin, answers the bounded
//!   A-stage operations inline (describe/validate — semantics unchanged),
//!   and dispatches the B-stage work operations (`generate_ui`, `run_ui`)
//!   onto worker threads;
//! - the **OS main thread** becomes the UI executor: the in-process VM UI
//!   (iced) must run there, so `run_ui` hands the entry off through
//!   [`MainJob`] and waits for the session result;
//! - one **work operation in flight at a time** (process-global run paths
//!   and env channels make this the honest concurrency contract; a second
//!   concurrent work op gets a named busy error before any side effect);
//! - `run_ui` is cancellable (owned children are killed; wait loops and
//!   step boundaries observe the flag) and interactive (the `Input`
//!   protocol message forwards bytes to the foreground child's stdin —
//!   Vite genuinely quits on `q` + Enter);
//! - child stdout is forwarded to the host **log** stream; the control
//!   stream stays framed JSON by construction;
//! - the per-op progress budget is enforced host-side before anything
//!   reaches the control stream (overflow emits `BudgetExceeded` and
//!   aborts the operation like a cancel).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use auto_tool_protocol::budget::Budgets;
use auto_tool_protocol::message::{
    ClientBody, ClientEnvelope, ServerBody, ServerEnvelope, ToolEvent, WireRequest,
};
use auto_tool_protocol::{
    Capability, LifecyclePolicy, ToolDescriptor, ToolError, EXIT_CODE_FATAL_STREAM,
    EXIT_CODE_INTERNAL, EXIT_CODE_OK, PROTOCOL_VERSION,
};

use crate::proc_host;
use crate::validate;

/// Renders the `run_ui` operation serves in this build (T-05 scope; the
/// full profile matrix lands with T-06/T-07 — other renders get a named
/// rejection before any side effect).
pub const RUN_UI_SERVED_PROFILES: &[&str] = &["vue", "vm", "rust"];

/// Renders the `generate_ui` operation serves in this build.
pub const GENERATE_UI_SERVED_PROFILES: &[&str] = &["vue"];

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
            Capability::non_interactive("generate_ui", false),
            Capability {
                operation: "run_ui".to_string(),
                cancellable: true,
                interactive: true,
            },
        ],
        lifecycle: LifecyclePolicy::contract(),
    }
}

/// A job for the OS main thread (iced requirement).
enum MainJob {
    RunVm {
        code: String,
        path: Option<String>,
        done: mpsc::Sender<auto_lang_core::AutoResult<String>>,
    },
}

struct InFlight {
    id: u64,
    operation: &'static str,
    io: Arc<proc_host::RunIo>,
    /// Last announced service URL (also mirrored into the final report).
    launched: Arc<Mutex<Option<String>>>,
}

#[derive(Default)]
struct Registry {
    served: HashMap<u64, &'static str>,
    in_flight: Option<InFlight>,
}

enum Lookup {
    InFlight(
        &'static str,
        Arc<proc_host::RunIo>,
        Arc<Mutex<Option<String>>>,
    ),
    Served(&'static str),
    Unknown,
}

impl Registry {
    fn lookup(&self, id: u64) -> Lookup {
        if let Some(flight) = &self.in_flight {
            if flight.id == id {
                return Lookup::InFlight(
                    flight.operation,
                    Arc::clone(&flight.io),
                    Arc::clone(&flight.launched),
                );
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
    main_tx: Mutex<mpsc::Sender<MainJob>>,
    exit_code: Arc<AtomicI32>,
}

impl<W: Write, L: Write> HostShared<W, L> {
    fn log_line(&self, message: &str) {
        let _ = writeln!(self.log.lock().unwrap(), "{message}");
    }

    /// Serialize and write one envelope on the control stream. Frame-level
    /// write failures are fatal (internal) — the caller decides the exit.
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
/// - exit codes follow the frozen contract (0 EOF / 1 internal / 2 stream).
pub fn run_tool<R, W, L>(input: R, control: W, log: L, max_frame_bytes: usize) -> i32
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    L: Write + Send + 'static,
{
    let (main_tx, main_rx) = mpsc::channel::<MainJob>();
    let exit_code = Arc::new(AtomicI32::new(EXIT_CODE_OK));
    let host = Arc::new(HostShared {
        control: Mutex::new(control),
        log: Mutex::new(log),
        max_frame_bytes,
        ops: Mutex::new(Registry::default()),
        main_tx: Mutex::new(main_tx),
        exit_code: Arc::clone(&exit_code),
    });

    let loop_host = Arc::clone(&host);
    let loop_thread = std::thread::Builder::new()
        .name("i-tool-loop".into())
        .spawn(move || {
            let code = serve_loop(input, Arc::clone(&loop_host));
            // EOF/fatal shutdown: cancel any in-flight work (kills owned
            // children) so workers finish; their final envelopes may still
            // write before the process goes away.
            if let Some(flight) = loop_host.ops.lock().unwrap().in_flight.take() {
                flight.io.cancel();
                // Break the hook Arc cycle (see proc_host::uninstall) so
                // EOF shutdown cannot leak the shared channel state.
                let io = flight.io;
                *io.on_progress.lock().unwrap() = None;
                *io.on_launched.lock().unwrap() = None;
                *io.on_log_chunk.lock().unwrap() = None;
                *io.on_vm_main.lock().unwrap() = None;
            }
            loop_host.exit_code.store(code, Ordering::SeqCst);
        })
        .expect("spawn protocol loop thread");

    // The main frame must NOT hold the last Arc: the recv loop below ends
    // when every HostShared handle (loop thread + workers, each carrying a
    // main_tx clone) drops — keeping one alive here would deadlock at EOF.
    drop(host);

    // The OS main thread is the UI executor (iced constraint).
    for job in main_rx {
        match job {
            MainJob::RunVm { code, path, done } => {
                let result = auto_ui_runtime::run_file_dynamic_ui(&code, path.as_deref());
                let _ = done.send(result);
            }
        }
    }
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
            Ok(None) => {
                return EXIT_CODE_OK;
            }
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
                    host.exit_code.store(e, Ordering::SeqCst);
                    return e;
                }
                continue;
            }
        };
        host.log_line(&format!("accepted request id={id} op={}", body.operation()));

        let response = match body {
            ClientBody::Describe => {
                // Bounded inline ops are recorded as served so later
                // cancel/input target them honestly (NotCancellable /
                // already-completed) instead of "unknown target".
                host.ops.lock().unwrap().served.insert(id, "describe");
                ServerEnvelope {
                    id,
                    body: ServerBody::DescribeOk {
                        descriptor: tool_descriptor(),
                    },
                }
            }
            ClientBody::ValidateProject { request } => {
                let report = validate::validate_request(&request.0);
                host.ops
                    .lock()
                    .unwrap()
                    .served
                    .insert(id, "validate_project");
                ServerEnvelope {
                    id,
                    body: ServerBody::ValidateOk { report },
                }
            }
            ClientBody::GenerateUi { request } => {
                match start_work_op(&host, id, "generate_ui") {
                    Ok(worker) => {
                        worker.spawn_generate(request);
                        continue; // response comes from the worker
                    }
                    Err(error) => ServerEnvelope::error(id, error),
                }
            }
            ClientBody::RunUi { request } => match start_work_op(&host, id, "run_ui") {
                Ok(worker) => {
                    worker.spawn_run(request);
                    continue; // response comes from the worker
                }
                Err(error) => ServerEnvelope::error(id, error),
            },
            ClientBody::Input { target_id, data } => handle_input(&host, id, target_id, data),
            ClientBody::Cancel { target_id } => handle_cancel(&host, id, target_id),
        };
        if let Err(e) = host.respond(&response) {
            host.exit_code.store(e, Ordering::SeqCst);
            return e;
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

    // Progress hook with host-side budget enforcement.
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
                    let _ = host.respond(&ServerEnvelope::error(
                        id,
                        ToolError::budget_exceeded("max_progress_bytes_per_op", limit, observed),
                    ));
                    // Abort like a cancel: children die, flow checkpoints
                    // return Cancelled.
                    hook_io.cancel();
                }
                return;
            }
            let _ = host.respond(&envelope);
        });
        *io.on_progress.lock().unwrap() = Some(progress_hook);
    }

    // Launched hook: record the URL and announce it on the control stream.
    {
        let host = Arc::clone(host);
        let launched = Arc::clone(&launched);
        let io = Arc::clone(&io);
        *io.on_launched.lock().unwrap() = Some(Box::new(move |url| {
            *launched.lock().unwrap() = Some(url.to_string());
            let _ = host.emit_event(
                id,
                ToolEvent::Launched {
                    url: url.to_string(),
                },
            );
            let _ = io; // keep the io alive for the hook lifetime
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

    // VM main-thread hand-off hook (run_ui only reaches it).
    {
        let main_tx = host.main_tx.lock().unwrap().clone();
        *io.on_vm_main.lock().unwrap() = Some(Arc::new(move |code: &str, path: Option<&str>| {
            let (done_tx, done_rx) = mpsc::channel();
            main_tx
                .send(MainJob::RunVm {
                    code: code.to_string(),
                    path: path.map(str::to_string),
                    done: done_tx,
                })
                .map_err(|_| {
                    auto_lang_core::error::AutoError::Msg(
                        "main-thread UI executor unavailable".to_string(),
                    )
                })?;
            done_rx
                .recv()
                .map_err(|_| {
                    auto_lang_core::error::AutoError::Msg(
                        "main-thread UI executor dropped the job".to_string(),
                    )
                })
                .and_then(|inner| inner)
        }));
    }

    {
        let mut ops = host.ops.lock().unwrap();
        ops.served.insert(id, operation);
        ops.in_flight = Some(InFlight {
            id,
            operation,
            io: Arc::clone(&io),
            launched: Arc::clone(&launched),
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
    fn spawn_generate(self, request: WireRequest) {
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
                let result = crate::tool_ops::generate_report(&request.0);
                // Uninstall first: leftover children die, hooks go quiet.
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
                let _ = host.respond(&response);
                finish_op(&host, id);
            })
            .expect("spawn work thread");
    }

    fn spawn_run(self, request: WireRequest) {
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
                let result = crate::tool_ops::run_report(&request.0);
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
                let _ = host.respond(&response);
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

fn handle_input<W: Write, L: Write>(
    host: &Arc<HostShared<W, L>>,
    id: u64,
    target_id: u64,
    data: Vec<u8>,
) -> ServerEnvelope {
    match host.ops.lock().unwrap().lookup(target_id) {
        Lookup::InFlight(operation, _io, _launched) => {
            if operation != "run_ui" {
                return ServerEnvelope::error(
                    id,
                    ToolError::invalid_field(
                        "data",
                        format!("operation '{operation}' has no interactive child process"),
                    ),
                );
            }
            match proc_host::write_input(&data) {
                Ok(()) => ServerEnvelope {
                    id,
                    body: ServerBody::InputAck {
                        target_id,
                        bytes: data.len() as u64,
                    },
                },
                Err(reason) => ServerEnvelope::error(id, ToolError::invalid_field("data", reason)),
            }
        }
        Lookup::Served(_) => ServerEnvelope::error(
            id,
            ToolError::invalid_field(
                "target_id",
                format!("operation {target_id} already completed"),
            ),
        ),
        Lookup::Unknown => ServerEnvelope::error(
            id,
            ToolError::invalid_field(
                "target_id",
                format!("no in-flight or completed operation with id {target_id}"),
            ),
        ),
    }
}

fn handle_cancel<W: Write, L: Write>(
    host: &Arc<HostShared<W, L>>,
    id: u64,
    target_id: u64,
) -> ServerEnvelope {
    match host.ops.lock().unwrap().lookup(target_id) {
        Lookup::InFlight(operation, io, _launched) => {
            if operation == "run_ui" {
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
