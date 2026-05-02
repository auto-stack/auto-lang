use auto_lang::vm::debugger::{DebugContext, DebuggerAction, DebuggerController};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Commands sent from the HTTP API to the controller.
#[derive(Debug, Clone)]
pub enum AgentDebugCommand {
    Continue,
    Step,
    StepOver,
    StepOut,
    Stop,
}

/// Serializable VM state for AI Agent consumption.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentDebugState {
    pub status: AgentDebugStatus,
    pub line: u32,
    pub ip: usize,
    pub op: String,
    pub stack: Vec<String>,
    pub call_stack: Vec<AgentCallFrameInfo>,
    pub locals: Vec<AgentLocalInfo>,
    pub args: Vec<AgentArgInfo>,
    pub registers: AgentRegisterInfo,
    pub stdout: String,
    pub stderr: String,
    pub result: Option<String>,
    /// Runtime error message if status is Error
    pub error: Option<String>,
    /// Source code lines around the current pause point for context
    pub source_context: Option<AgentSourceContext>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentSourceContext {
    pub current_line: u32,
    pub lines: Vec<AgentSourceLine>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentSourceLine {
    pub line_number: u32,
    pub text: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub enum AgentDebugStatus {
    Paused,
    Running,
    Finished,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentCallFrameInfo {
    pub fn_name: Option<String>,
    pub line: u32,
    pub return_ip: usize,
    pub bp: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentLocalInfo {
    pub index: usize,
    pub value: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentArgInfo {
    pub index: usize,
    pub value: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentRegisterInfo {
    pub ip: usize,
    pub bp: usize,
    pub sp: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum AgentMode {
    Run,
    Step,
    StepOver,
    StepOut,
}

/// Blocking debugger controller for headless AI Agent use.
///
/// Bridges VM execution (sync) with HTTP API (async) via a `tokio::sync::watch`
/// channel for state broadcast and a `std::sync::mpsc` channel for command
/// reception.
///
/// When the VM pauses, `on_pause` blocks until a command arrives from the
/// HTTP handler, then returns the corresponding `DebuggerAction`.
pub struct BlockingAgentController {
    breakpoints: Arc<Mutex<HashSet<u32>>>,
    cmd_rx: Mutex<std::sync::mpsc::Receiver<AgentDebugCommand>>,
    state_tx: tokio::sync::watch::Sender<AgentDebugState>,
    mode: Arc<Mutex<AgentMode>>,
    depth_at_pause: Arc<Mutex<usize>>,
    last_line: Arc<Mutex<u32>>,
    stop_requested: Arc<Mutex<bool>>,
    has_started: Arc<AtomicBool>,
    output_buffer: Option<Arc<std::sync::RwLock<String>>>,
    source_lines: Vec<String>,
}

impl BlockingAgentController {
    pub fn new(
        cmd_rx: std::sync::mpsc::Receiver<AgentDebugCommand>,
        state_tx: tokio::sync::watch::Sender<AgentDebugState>,
        output_buffer: Option<Arc<std::sync::RwLock<String>>>,
        source_lines: Vec<String>,
    ) -> (Self, Arc<Mutex<HashSet<u32>>>) {
        let breakpoints = Arc::new(Mutex::new(HashSet::new()));
        let controller = Self {
            breakpoints: breakpoints.clone(),
            cmd_rx: Mutex::new(cmd_rx),
            state_tx,
            mode: Arc::new(Mutex::new(AgentMode::Run)),
            depth_at_pause: Arc::new(Mutex::new(0)),
            last_line: Arc::new(Mutex::new(0)),
            stop_requested: Arc::new(Mutex::new(false)),
            has_started: Arc::new(AtomicBool::new(false)),
            output_buffer,
            source_lines,
        };
        (controller, breakpoints)
    }

    fn set_resumed_line(&self, line: u32) {
        *self.last_line.lock().unwrap() = line;
    }
}

impl DebuggerController for BlockingAgentController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        if *self.stop_requested.lock().unwrap() {
            return true;
        }

        match *self.mode.lock().unwrap() {
            AgentMode::Run => {
                if !self.has_started.swap(true, Ordering::SeqCst) {
                    return true;
                }
                let resumed_line = *self.last_line.lock().unwrap();
                if ctx.line > 0 && ctx.line == resumed_line {
                    return false;
                }
                let hit = self.breakpoints.lock().unwrap().contains(&ctx.line);
                if hit {
                    *self.last_line.lock().unwrap() = ctx.line;
                }
                hit
            }
            AgentMode::Step => {
                let mut last_line = self.last_line.lock().unwrap();
                if ctx.line == *last_line || ctx.line == 0 {
                    false
                } else {
                    *last_line = ctx.line;
                    true
                }
            }
            AgentMode::StepOver => {
                let target_depth = *self.depth_at_pause.lock().unwrap();
                let current_depth = ctx.call_stack.len();
                if current_depth > target_depth {
                    false
                } else if current_depth < target_depth {
                    true
                } else {
                    let mut last_line = self.last_line.lock().unwrap();
                    let changed = ctx.line != *last_line && ctx.line > 0;
                    if changed {
                        *last_line = ctx.line;
                    }
                    changed
                }
            }
            AgentMode::StepOut => {
                let target_depth = *self.depth_at_pause.lock().unwrap();
                ctx.call_stack.len() < target_depth
            }
        }
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        let state = build_agent_debug_state(ctx, self.output_buffer.clone(), &self.source_lines);
        // watch::Sender::send is synchronous and will notify all receivers.
        let _ = self.state_tx.send(state);

        let rx = self.cmd_rx.lock().unwrap();
        while let Ok(cmd) = rx.recv() {
            match cmd {
                AgentDebugCommand::Continue => {
                    *self.mode.lock().unwrap() = AgentMode::Run;
                    self.set_resumed_line(ctx.line);
                    return DebuggerAction::Continue;
                }
                AgentDebugCommand::Step => {
                    *self.mode.lock().unwrap() = AgentMode::Step;
                    *self.depth_at_pause.lock().unwrap() = ctx.call_stack.len();
                    *self.last_line.lock().unwrap() = ctx.line;
                    return DebuggerAction::Step;
                }
                AgentDebugCommand::StepOver => {
                    *self.mode.lock().unwrap() = AgentMode::StepOver;
                    *self.depth_at_pause.lock().unwrap() = ctx.call_stack.len();
                    *self.last_line.lock().unwrap() = ctx.line;
                    return DebuggerAction::Step;
                }
                AgentDebugCommand::StepOut => {
                    *self.mode.lock().unwrap() = AgentMode::StepOut;
                    *self.depth_at_pause.lock().unwrap() = ctx.call_stack.len();
                    return DebuggerAction::Step;
                }
                AgentDebugCommand::Stop => {
                    *self.stop_requested.lock().unwrap() = true;
                    return DebuggerAction::Quit;
                }
            }
        }

        DebuggerAction::Quit
    }
}

fn build_agent_debug_state(
    ctx: &DebugContext,
    output_buffer: Option<Arc<std::sync::RwLock<String>>>,
    source_lines: &[String],
) -> AgentDebugState {
    let stack: Vec<String> = ctx.task.ram.raw[..ctx.task.ram.sp.min(256)]
        .iter()
        .map(|v| v.to_string())
        .collect();

    let call_stack: Vec<AgentCallFrameInfo> = ctx
        .task
        .call_stack
        .iter()
        .map(|f| AgentCallFrameInfo {
            fn_name: f.fn_name.clone(),
            line: f.line,
            return_ip: f.return_ip,
            bp: f.old_bp,
        })
        .collect();

    let locals: Vec<AgentLocalInfo> = (0..ctx.task.current_fn_n_locals)
        .map(|i| {
            let val = ctx.task.ram.read_i32(ctx.task.bp + 1 + i);
            AgentLocalInfo { index: i, value: val }
        })
        .collect();

    let n_args = ctx.task.current_fn_n_args;
    let args: Vec<AgentArgInfo> = (0..n_args)
        .map(|i| {
            let offset = ctx.task.bp - n_args + i;
            let val = ctx.task.ram.read_i32(offset);
            AgentArgInfo { index: i, value: val }
        })
        .collect();

    let stdout = output_buffer
        .as_ref()
        .map(|buf| buf.read().unwrap().clone())
        .unwrap_or_default();

    AgentDebugState {
        status: AgentDebugStatus::Paused,
        line: ctx.line,
        ip: ctx.ip,
        op: ctx.current_op.to_mnemonic().to_string(),
        stack,
        call_stack,
        locals,
        args,
        registers: AgentRegisterInfo {
            ip: ctx.task.ip,
            bp: ctx.task.bp,
            sp: ctx.task.ram.sp,
        },
        stdout,
        stderr: String::new(),
        result: None,
        error: None,
        source_context: build_source_context(ctx.line, source_lines),
    }
}

fn build_source_context(line: u32, source_lines: &[String]) -> Option<AgentSourceContext> {
    if line == 0 || source_lines.is_empty() {
        return None;
    }
    let center = line as usize;
    let start = center.saturating_sub(3).max(1);
    let end = (center + 3).min(source_lines.len());
    let lines: Vec<AgentSourceLine> = (start..=end)
        .map(|ln| AgentSourceLine {
            line_number: ln as u32,
            text: source_lines.get(ln - 1).cloned().unwrap_or_default(),
            is_current: ln == center,
        })
        .collect();
    Some(AgentSourceContext {
        current_line: line,
        lines,
    })
}
