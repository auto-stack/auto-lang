use super::controller::{
    AgentDebugCommand, AgentDebugState, AgentDebugStatus, BlockingAgentController,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A single Agent debug session.
///
/// Each session owns a dedicated OS thread running the VM.  The HTTP layer
/// communicates with the controller via:
/// - `cmd_tx`   → send commands to the VM thread
/// - `state_rx` → receive the latest VM state (via `tokio::sync::watch`)
/// - `breakpoints` → shared breakpoint set (thread-safe)
#[derive(Clone)]
pub struct AgentDebugSession {
    #[allow(dead_code)]
    pub id: String,
    pub cmd_tx: std::sync::mpsc::Sender<AgentDebugCommand>,
    pub state_rx: tokio::sync::watch::Receiver<AgentDebugState>,
    pub breakpoints: Arc<Mutex<HashSet<u32>>>,
    pub created_at: Instant,
}

impl AgentDebugSession {
    /// Spawn a new session in a dedicated OS thread.
    ///
    /// Compilation happens inside the thread (AutoVM is !Send).  The bytecode
    /// is returned via a oneshot channel **before** the VM starts running, so
    /// the HTTP handler can respond immediately.  The VM then starts and
    /// pauses at the first instruction waiting for a command.
    pub fn spawn(id: String, source: String) -> Result<(Self, Vec<serde_json::Value>), String> {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<AgentDebugCommand>();

        let initial_state = AgentDebugState {
            status: AgentDebugStatus::Running,
            line: 0,
            ip: 0,
            op: String::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            locals: Vec::new(),
            args: Vec::new(),
            registers: super::controller::AgentRegisterInfo { ip: 0, bp: 0, sp: 0 },
            stdout: String::new(),
            stderr: String::new(),
            result: None,
            error: None,
            source_context: None,
        };
        let (state_tx, state_rx) = tokio::sync::watch::channel(initial_state);

        // Oneshot channel to synchronously receive compilation result.
        let (compile_tx, compile_rx) = std::sync::mpsc::channel();

        let thread_id = id.clone();
        std::thread::spawn(move || {
            let result: Result<(Arc<Mutex<HashSet<u32>>>, Vec<serde_json::Value>), String> =
                (|| {
                    let (mut vm, output_buffer, entry_point, result_type) =
                        auto_lang::create_vm_from_source(&source)
                            .map_err(|e| format!("{:?}", e))?;

                    let disasm = auto_lang::vm::disasm::Disassembler::new(&vm.flash);
                    let lines = disasm.disassemble_range(0, vm.flash.memory.len());
                    let bytecode_json: Vec<serde_json::Value> = lines
                        .iter()
                        .map(|l| {
                            serde_json::json!({
                                "offset": l.offset,
                                "mnemonic": l.mnemonic,
                                "operands": l.operands,
                                "line": l.line,
                            })
                        })
                        .collect();

                    let source_lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
                    let state_tx2 = state_tx.clone();
                    let (controller, breakpoints) = BlockingAgentController::new(
                        cmd_rx,
                        state_tx,
                        Some(output_buffer.clone()),
                        source_lines,
                    );
                    vm.set_debugger(Box::new(controller));

                    // ── Send compile result back *before* starting VM ──
                    // This lets the HTTP handler respond immediately.
                    //
                    // NOTE: we clone breakpoints so the HTTP layer can keep
                    // a reference while we move the original into the VM
                    // thread below.
                    let _ = compile_tx.send(Ok((breakpoints.clone(), bytecode_json)));

                    // Run VM in the same thread using a local tokio runtime.
                    let rt = tokio::runtime::Runtime::new()
                        .expect("tokio runtime for agent debug");
                    rt.block_on(async move {
                        tracing::debug!("Agent debug session {}: VM starting", thread_id);
                        let task_id = vm.spawn_task(entry_point, 16384);
                        vm.run_task_loop().await;
                        tracing::debug!("Agent debug session {}: VM finished", thread_id);

                        let stdout = output_buffer.read().unwrap().clone();
                        let result = auto_lang::extract_autovm_result(
                            &vm,
                            task_id,
                            Some(result_type),
                        )
                        .await
                        .ok();

                        let (status, error) = match result {
                            Some(_) => (AgentDebugStatus::Finished, None),
                            None => {
                                // Check if task had an error
                                // Use try_lock in a spin loop because we're inside a tokio
                                // runtime and cannot call blocking_lock.
                                let task_arc = vm.tasks.get(&task_id)
                                    .map(|r| r.value().clone());
                                let last_error = if let Some(t) = task_arc {
                                    loop {
                                        if let Ok(task) = t.try_lock() {
                                            break task.last_error.clone();
                                        }
                                        std::thread::yield_now();
                                    }
                                } else { None };
                                if last_error.is_some() {
                                    (AgentDebugStatus::Error, last_error)
                                } else {
                                    (AgentDebugStatus::Finished, None)
                                }
                            }
                        };

                        let _ = state_tx2.send(AgentDebugState {
                            status,
                            line: 0,
                            ip: 0,
                            op: String::new(),
                            stack: Vec::new(),
                            call_stack: Vec::new(),
                            locals: Vec::new(),
                            args: Vec::new(),
                            registers: super::controller::AgentRegisterInfo {
                                ip: 0,
                                bp: 0,
                                sp: 0,
                            },
                            stdout,
                            stderr: String::new(),
                            result,
                            error,
                            source_context: None,
                        });
                    });

                    Ok((breakpoints, Vec::new())) // dummy, already sent above
                })();

            // If compilation failed, send error back.
            if let Err(ref e) = result {
                let _ = compile_tx.send(Err(e.clone()));
            }
        });

        let (breakpoints, bytecode_json) = compile_rx
            .recv()
            .map_err(|_| "Compile thread panicked".to_string())?
            .map_err(|e| format!("Compile error: {}", e))?;

        let session = Self {
            id,
            cmd_tx,
            state_rx,
            breakpoints,
            created_at: Instant::now(),
        };

        Ok((session, bytecode_json))
    }
}
