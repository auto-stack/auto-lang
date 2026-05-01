use auto_lang::vm::debugger::{DebugContext, DebuggerAction, DebuggerController};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Commands sent from the WebSocket session to the controller
#[derive(Debug, Clone)]
pub enum DebugCommand {
    Continue,
    Step,
    StepOver,
    StepOut,
    Stop,
}

/// Serializable VM state sent to the frontend when paused
#[derive(Debug, Clone, serde::Serialize)]
pub struct DebugState {
    pub status: DebugStatus,
    pub line: u32,
    pub ip: usize,
    pub op: String,
    pub stack: Vec<String>,
    pub call_stack: Vec<CallFrameInfo>,
    pub locals: Vec<LocalInfo>,
    pub args: Vec<ArgInfo>,
    pub registers: RegisterInfo,
    pub stdout: String,
    pub stderr: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum DebugStatus {
    Paused,
    #[allow(dead_code)]
    Running,
    #[allow(dead_code)]
    Finished,
    #[allow(dead_code)]
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CallFrameInfo {
    pub fn_name: Option<String>,
    pub line: u32,
    pub return_ip: usize,
    pub bp: usize,
    pub n_args: usize,
    pub n_locals: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalInfo {
    pub index: usize,
    pub value: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArgInfo {
    pub index: usize,
    pub value: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegisterInfo {
    pub ip: usize,
    pub bp: usize,
    pub sp: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum DebugMode {
    Run,
    Step,
    StepOver,
    StepOut,
}

/// Network-aware debugger controller for the Playground.
/// Bridges VM execution (sync) with WebSocket frontend (async) via channels.
pub struct PlaygroundController {
    breakpoints: Arc<Mutex<HashSet<u32>>>,
    cmd_rx: Mutex<std::sync::mpsc::Receiver<DebugCommand>>,
    state_tx: tokio::sync::mpsc::Sender<DebugState>,
    mode: Arc<Mutex<DebugMode>>,
    depth_at_pause: Arc<Mutex<usize>>,
    last_line: Arc<Mutex<u32>>,
    stop_requested: Arc<Mutex<bool>>,
    has_started: Arc<AtomicBool>,
    output_buffer: Option<Arc<std::sync::RwLock<String>>>,
}

impl PlaygroundController {
    pub fn new(
        cmd_rx: std::sync::mpsc::Receiver<DebugCommand>,
        state_tx: tokio::sync::mpsc::Sender<DebugState>,
        output_buffer: Option<Arc<std::sync::RwLock<String>>>,
    ) -> (Self, Arc<Mutex<HashSet<u32>>>) {
        let breakpoints = Arc::new(Mutex::new(HashSet::new()));
        let controller = Self {
            breakpoints: breakpoints.clone(),
            cmd_rx: Mutex::new(cmd_rx),
            state_tx,
            mode: Arc::new(Mutex::new(DebugMode::Run)),
            depth_at_pause: Arc::new(Mutex::new(0)),
            last_line: Arc::new(Mutex::new(0)),
            stop_requested: Arc::new(Mutex::new(false)),
            has_started: Arc::new(AtomicBool::new(false)),
            output_buffer,
        };
        (controller, breakpoints)
    }

    /// Update last_line to the current line — used when resuming to avoid
    /// re-hitting the same breakpoint immediately after Continue.
    fn set_resumed_line(&self, line: u32) {
        *self.last_line.lock().unwrap() = line;
    }
}

impl DebuggerController for PlaygroundController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        if *self.stop_requested.lock().unwrap() {
            return true;
        }

        match *self.mode.lock().unwrap() {
            DebugMode::Run => {
                if !self.has_started.swap(true, Ordering::SeqCst) {
                    // First instruction: pause to let frontend set breakpoints
                    return true;
                }
                // In Run mode, pause only at breakpoints.
                // Skip the line we just resumed from so that Continue doesn't
                // immediately re-pause on the same breakpoint.
                let resumed_line = *self.last_line.lock().unwrap();
                if ctx.line > 0 && ctx.line == resumed_line {
                    return false;
                }
                let hit = self.breakpoints.lock().unwrap().contains(&ctx.line);
                if hit {
                    // Remember this line so that subsequent instructions on
                    // the same line don't re-trigger the breakpoint.
                    *self.last_line.lock().unwrap() = ctx.line;
                }
                hit
            }
            DebugMode::Step => {
                // Step Into: pause when source line changes (enters new line or new function)
                let mut last_line = self.last_line.lock().unwrap();
                if ctx.line == *last_line || ctx.line == 0 {
                    false
                } else {
                    *last_line = ctx.line;
                    true
                }
            }
            DebugMode::StepOver => {
                let target_depth = *self.depth_at_pause.lock().unwrap();
                let current_depth = ctx.call_stack.len();
                if current_depth > target_depth {
                    // Inside a function call — keep running
                    false
                } else if current_depth < target_depth {
                    // Returned from function — pause
                    true
                } else {
                    // Same depth — pause when source line changes
                    let mut last_line = self.last_line.lock().unwrap();
                    let changed = ctx.line != *last_line && ctx.line > 0;
                    if changed {
                        *last_line = ctx.line;
                    }
                    changed
                }
            }
            DebugMode::StepOut => {
                let target_depth = *self.depth_at_pause.lock().unwrap();
                ctx.call_stack.len() < target_depth
            }
        }
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        // Build and send state to frontend
        let state = build_debug_state(ctx, self.output_buffer.clone());
        let _ = self.state_tx.try_send(state);

        // Wait for next command from frontend
        let rx = self.cmd_rx.lock().unwrap();
        while let Ok(cmd) = rx.recv() {
            match cmd {
                DebugCommand::Continue => {
                    *self.mode.lock().unwrap() = DebugMode::Run;
                    self.set_resumed_line(ctx.line);
                    return DebuggerAction::Continue;
                }
                DebugCommand::Step => {
                    *self.mode.lock().unwrap() = DebugMode::Step;
                    *self.depth_at_pause.lock().unwrap() = ctx.call_stack.len();
                    *self.last_line.lock().unwrap() = ctx.line;
                    return DebuggerAction::Step;
                }
                DebugCommand::StepOver => {
                    *self.mode.lock().unwrap() = DebugMode::StepOver;
                    *self.depth_at_pause.lock().unwrap() = ctx.call_stack.len();
                    *self.last_line.lock().unwrap() = ctx.line;
                    return DebuggerAction::Step;
                }
                DebugCommand::StepOut => {
                    *self.mode.lock().unwrap() = DebugMode::StepOut;
                    *self.depth_at_pause.lock().unwrap() = ctx.call_stack.len();
                    return DebuggerAction::Step;
                }
                DebugCommand::Stop => {
                    *self.stop_requested.lock().unwrap() = true;
                    return DebuggerAction::Quit;
                }
            }
        }

        // Channel closed — stop
        DebuggerAction::Quit
    }
}

fn build_debug_state(
    ctx: &DebugContext,
    output_buffer: Option<Arc<std::sync::RwLock<String>>>,
) -> DebugState {
    let stack: Vec<String> = ctx.task.ram.raw[..ctx.task.ram.sp.min(256)]
        .iter()
        .map(|v| v.to_string())
        .collect();

    let call_stack: Vec<CallFrameInfo> = ctx
        .task
        .call_stack
        .iter()
        .map(|f| CallFrameInfo {
            fn_name: f.fn_name.clone(),
            line: f.line,
            return_ip: f.return_ip,
            bp: f.old_bp,
            n_args: 0,
            n_locals: 0,
        })
        .collect();

    let locals: Vec<LocalInfo> = (0..ctx.task.current_fn_n_locals)
        .map(|i| {
            let val = ctx.task.ram.read_i32(ctx.task.bp + 1 + i);
            LocalInfo { index: i, value: val }
        })
        .collect();

    let n_args = ctx.task.current_fn_n_args;
    let args: Vec<ArgInfo> = (0..n_args)
        .map(|i| {
            // args are at bp - n_args - 1 + i
            // (accounting for return_addr at bp-1 and old_bp at bp)
            let offset = ctx.task.bp - n_args + i;
            let val = ctx.task.ram.read_i32(offset);
            ArgInfo { index: i, value: val }
        })
        .collect();

    let stdout = output_buffer
        .as_ref()
        .map(|buf| buf.read().unwrap().clone())
        .unwrap_or_default();

    DebugState {
        status: DebugStatus::Paused,
        line: ctx.line,
        ip: ctx.ip,
        op: ctx.current_op.to_mnemonic().to_string(),
        stack,
        call_stack,
        locals,
        args,
        registers: RegisterInfo {
            ip: ctx.task.ip,
            bp: ctx.task.bp,
            sp: ctx.task.ram.sp,
        },
        stdout,
        stderr: String::new(),
        result: None,
    }
}
