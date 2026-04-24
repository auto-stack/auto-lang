//! Plan 199 Phase 4: Debugger controller trait and implementations
//!
//! Provides the DebuggerController trait for VM debugging hooks,
//! with NoOpController (normal execution), AgentController (AI Agent),
//! and ReplController (interactive human debugging).

use crate::vm::task::{AutoTask, CallFrame};
use crate::vm::opcode::OpCode;
use std::collections::HashSet;

/// Snapshot of VM state at a pause point
pub struct DebugContext<'a> {
    pub task: &'a AutoTask,
    pub current_op: OpCode,
    pub ip: usize,
    pub line: u32,
    pub call_stack: &'a [CallFrame],
}

/// Action for the VM to take after a pause
#[derive(Debug, Clone, PartialEq)]
pub enum DebuggerAction {
    Continue,
    Step,
    StepOver,
    StepOut,
    Quit,
}

/// Breakpoint definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Breakpoint {
    AtIp(usize),
    AtLine(u32),
    AtFunction(String),
}

/// Controller trait — implements debugging behavior
pub trait DebuggerController: Send {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool;
    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction;
}

/// No-op controller for normal execution (zero overhead when should_pause returns false)
pub struct NoOpController;

impl DebuggerController for NoOpController {
    fn should_pause(&mut self, _ctx: &DebugContext) -> bool {
        false
    }
    fn on_pause(&mut self, _ctx: &DebugContext) -> DebuggerAction {
        DebuggerAction::Continue
    }
}

/// Debug mode for AgentController
#[derive(Debug, Clone, PartialEq)]
pub enum DebugMode {
    Run,
    Step,
    StepLine,
}

/// Programmatic debugger controller for AI Agent use
pub struct AgentController {
    pub breakpoints: HashSet<Breakpoint>,
    pub mode: DebugMode,
    pub paused_state: Option<PausedState>,
    last_line: u32,
}

/// Captured state when paused
#[derive(Debug, Clone)]
pub struct PausedState {
    pub ip: usize,
    pub line: u32,
    pub op: String,
    pub call_depth: usize,
    pub call_stack: Vec<CallFrame>,
}

impl AgentController {
    pub fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            mode: DebugMode::Run,
            paused_state: None,
            last_line: 0,
        }
    }

    pub fn add_breakpoint(&mut self, bp: Breakpoint) {
        self.breakpoints.insert(bp);
    }

    pub fn set_mode(&mut self, mode: DebugMode) {
        self.mode = mode;
    }
}

impl DebuggerController for AgentController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        match self.mode {
            DebugMode::Step => true,
            DebugMode::StepLine => {
                let should = ctx.line != self.last_line && ctx.line > 0;
                if should {
                    self.last_line = ctx.line;
                }
                should
            }
            DebugMode::Run => {
                self.breakpoints.iter().any(|bp| match bp {
                    Breakpoint::AtIp(ip) => *ip == ctx.ip,
                    Breakpoint::AtLine(line) => *line == ctx.line,
                    Breakpoint::AtFunction(name) => {
                        ctx.call_stack
                            .last()
                            .and_then(|f| f.fn_name.as_ref())
                            .map(|n| n == name)
                            .unwrap_or(false)
                    }
                })
            }
        }
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        self.paused_state = Some(PausedState {
            ip: ctx.ip,
            line: ctx.line,
            op: format!("{:?}", ctx.current_op),
            call_depth: ctx.call_stack.len(),
            call_stack: ctx.call_stack.to_vec(),
        });
        DebuggerAction::Continue
    }
}

/// Interactive REPL debugger for human use
pub struct ReplController {
    breakpoints: HashSet<Breakpoint>,
    step_mode: bool,
    last_line: u32,
}

impl ReplController {
    pub fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            step_mode: false,
            last_line: 0,
        }
    }
}

impl DebuggerController for ReplController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        if self.step_mode {
            return true;
        }

        // Pause on line change when there are breakpoints
        if ctx.line != self.last_line && ctx.line > 0 {
            self.last_line = ctx.line;
        }

        self.breakpoints.iter().any(|bp| match bp {
            Breakpoint::AtLine(line) => *line == ctx.line,
            Breakpoint::AtIp(ip) => *ip == ctx.ip,
            Breakpoint::AtFunction(name) => {
                ctx.call_stack
                    .last()
                    .and_then(|f| f.fn_name.as_ref())
                    .map(|n| n == name)
                    .unwrap_or(false)
            }
        })
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        let line = if ctx.line > 0 {
            format!("line {}", ctx.line)
        } else {
            format!("ip={}", ctx.ip)
        };
        println!("--- Paused at {} | op={:?} ---", line, ctx.current_op);

        let sp = ctx.task.ram.sp;
        if sp > 0 {
            let show = std::cmp::min(sp, 5);
            print!("  Stack[{}]: ", show);
            for i in (sp - show)..sp {
                print!("{} ", ctx.task.ram.read_i32(i));
            }
            println!();
        }

        let mut input = String::new();
        loop {
            print!("(auto-dbg) ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            input.clear();
            if std::io::stdin().read_line(&mut input).is_err() {
                return DebuggerAction::Quit;
            }
            let cmd = input.trim();
            match cmd {
                "c" | "continue" => {
                    self.step_mode = false;
                    return DebuggerAction::Continue;
                }
                "s" | "step" => {
                    self.step_mode = true;
                    return DebuggerAction::Step;
                }
                "q" | "quit" => return DebuggerAction::Quit,
                "stack" => {
                    println!("Call stack:");
                    for (i, frame) in ctx.call_stack.iter().enumerate().rev() {
                        let name = frame.fn_name.as_deref().unwrap_or("<anonymous>");
                        println!("  #{} {} at line {}", i, name, frame.line);
                    }
                }
                "locals" => {
                    let bp = ctx.task.bp;
                    let n = ctx.task.current_fn_n_locals;
                    println!("Locals ({}):", n);
                    for i in 0..n {
                        let val = ctx.task.ram.read_i32(bp + 1 + i);
                        println!("  [{}] = {}", i, val);
                    }
                }
                _ => {
                    println!("Commands: c(ontinue), s(tep), q(uit), stack, locals");
                }
            }
        }
    }
}
