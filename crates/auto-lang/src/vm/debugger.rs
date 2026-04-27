//! Plan 199 Phase 4: Debugger controller trait and implementations
//!
//! Provides the DebuggerController trait for VM debugging hooks,
//! with NoOpController (normal execution), AgentController (AI Agent),
//! and GdbController (interactive human debugging with GDB-like commands).

use crate::vm::task::{AutoTask, CallFrame};
use crate::vm::opcode::OpCode;
use crate::vm::virt_memory::VirtualFlash;
use std::collections::HashSet;
use colored::Colorize;

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

// =========================================================================
// GdbController — GDB-style interactive debugger
// =========================================================================

/// Step mode for controlling pause behavior
#[derive(Debug, Clone, PartialEq)]
enum StepMode {
    None,
    StepInto,
    StepOver,
    StepOut,
    UntilLine(u32),
}

/// GDB-style interactive debugger controller
pub struct GdbController {
    breakpoints: Vec<Breakpoint>,
    step_mode: StepMode,
    last_line: u32,
    call_depth_at_step: usize,
    source_lines: Vec<String>,
    flash: VirtualFlash,
    started: bool,
    /// Plan 199 Phase 7: Function name -> address mapping for enhanced break syntax
    exports_by_name: std::collections::HashMap<String, u32>,
    /// Line we last paused at — used to skip same-line hits on continue
    paused_line: u32,
}

impl GdbController {
    pub fn new(source_lines: Vec<String>, flash_bytes: Vec<u8>, exports_by_name: std::collections::HashMap<String, u32>) -> Self {
        Self {
            breakpoints: Vec::new(),
            step_mode: StepMode::None,
            last_line: 0,
            call_depth_at_step: 0,
            source_lines,
            flash: VirtualFlash::from_vec(flash_bytes),
            started: false,
            exports_by_name,
            paused_line: 0,
        }
    }

    fn print_source_context(&self, line: u32, context: usize) {
        if line == 0 || self.source_lines.is_empty() {
            return;
        }
        let center = line as usize;
        let start = center.saturating_sub(context);
        let end = (center + context + 1).min(self.source_lines.len());
        for i in start..end {
            let ln = i + 1;
            let is_current = ln == center;
            if is_current {
                println!("{} {} | {}", ">".green().bold(), format!("{:>4}", ln).yellow().bold(), self.source_lines[i].white());
            } else {
                println!("  {} | {}", format!("{:>4}", ln).dimmed(), self.source_lines[i].dimmed());
            }
        }
    }

    fn print_disassembly(&self, ip: usize, count: usize) {
        let start = ip.saturating_sub(20);
        let end = (ip + count * 10).min(self.flash.memory.len());
        if start >= end {
            println!("No bytecode to disassemble.");
            return;
        }
        let disasm = crate::vm::disasm::Disassembler::new(&self.flash);
        let lines = disasm.disassemble_range(start, end);
        for dl in &lines {
            let is_current = dl.offset == ip;
            let line_info = match dl.line {
                Some(l) => format!("; line {}", l).dimmed().to_string(),
                None => String::new(),
            };
            if is_current {
                println!("{} {:04x}  {:<12} {} {}",
                    ">".green().bold(),
                    dl.offset,
                    dl.mnemonic.white().bold(),
                    dl.operands,
                    line_info
                );
            } else {
                println!("  {}  {:<12} {} {}",
                    format!("{:04x}", dl.offset).dimmed(),
                    dl.mnemonic,
                    dl.operands,
                    line_info
                );
            }
        }
    }

    /// Scan bytecode from function entry to find the first SOURCE_LINE opcode
    fn find_function_start_line(&self, addr: usize) -> u32 {
        let source_line_opcode = OpCode::SOURCE_LINE as u8;
        let fn_prolog_opcode = OpCode::FN_PROLOG as u8;
        let mut ip = addr;
        let end = self.flash.memory.len().min(ip + 100);
        while ip < end {
            let byte = self.flash.read_u8(ip);
            if byte == source_line_opcode && ip + 2 < end {
                return self.flash.read_u16(ip + 1) as u32;
            }
            if byte == fn_prolog_opcode {
                ip += 3; // FN_PROLOG: opcode + n_args + n_locals
            } else {
                ip += 1;
            }
        }
        0
    }

    /// Get a truncated source line for display
    fn source_preview(&self, line: u32) -> String {
        if line == 0 { return String::new(); }
        self.source_lines.get(line as usize - 1)
            .map(|s| {
                let trimmed = s.trim();
                if trimmed.len() > 50 {
                    format!("{}...", &trimmed[..50])
                } else {
                    trimmed.to_string()
                }
            })
            .unwrap_or_default()
    }

    /// Print a breakpoint confirmation with source preview
    fn print_bp_confirmed(&self, idx: usize, detail: &str, line: u32) {
        let preview = self.source_preview(line);
        if preview.is_empty() {
            println!("{} {} {}", "Breakpoint".green(), idx.to_string().cyan(), detail);
        } else {
            println!("{} {} {}:", "Breakpoint".green(), idx.to_string().cyan(), detail);
            println!("  {} {}", format!("{}:", line).dimmed(), preview);
        }
    }

    fn show_help(&self) {
        println!("GDB-like commands:");
        println!("  run (r)                Start / continue execution");
        println!("  continue (c)           Continue to next breakpoint");
        println!("  step (s)               Step into (one instruction)");
        println!("  next (n)               Step over (next source line)");
        println!("  finish (fin)           Run until current function returns");
        println!("  until <line> (u)       Run until source line");
        println!("  break <line|fn|fn/N> (b) Set breakpoint (line, function, or function+offset)");
        println!("  delete <n> (d)         Delete breakpoint #n");
        println!("  info breakpoints (i b) List breakpoints");
        println!("  info stack (i s)       Show call stack (backtrace)");
        println!("  info locals (i l)      Show local variables");
        println!("  info registers (i r)   Show IP/BP/SP registers");
        println!("  list (l)               Show source code context");
        println!("  disassemble (disas)    Disassemble nearby bytecode");
        println!("  print <slot> (p)       Print local variable by slot index");
        println!("  quit (q)               Exit debugger");
        println!("  help (h)               Show this help");
    }
}

impl DebuggerController for GdbController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        // Before first run, always pause (user hasn't typed 'run' yet)
        if !self.started {
            return true;
        }

        match self.step_mode {
            StepMode::None => {
                // Once we leave the paused line (to any other line, even 0), clear it
                if self.paused_line > 0 && ctx.line != self.paused_line {
                    self.paused_line = 0;
                }
                // Only pause at breakpoints, but skip AtLine if same as paused_line
                self.breakpoints.iter().any(|bp| match bp {
                    Breakpoint::AtIp(ip) => *ip == ctx.ip,
                    Breakpoint::AtLine(line) => {
                        *line == ctx.line && ctx.line != self.paused_line
                    }
                    Breakpoint::AtFunction(name) => {
                        if self.paused_line > 0 {
                            return false;
                        }
                        ctx.call_stack
                            .last()
                            .and_then(|f| f.fn_name.as_ref())
                            .map(|n| n == name)
                            .unwrap_or(false)
                    }
                })
            }
            StepMode::StepInto => true,
            StepMode::StepOver => {
                let line_changed = ctx.line != self.last_line && ctx.line > 0;
                if line_changed {
                    self.last_line = ctx.line;
                }
                line_changed
            }
            StepMode::StepOut => {
                ctx.call_stack.len() < self.call_depth_at_step
            }
            StepMode::UntilLine(target) => {
                if ctx.line == target {
                    self.step_mode = StepMode::None;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        // Record paused line to skip same-line re-hits on continue
        self.paused_line = ctx.line;

        // Show current position
        if ctx.line > 0 {
            let source_line = self.source_lines.get(ctx.line as usize - 1)
                .map(|s| s.as_str())
                .unwrap_or("");
            println!("\n{} {} {} {} {} {}",
                "---".dimmed(),
                format!("line {}", ctx.line).yellow().bold(),
                "|".dimmed(),
                format!("ip={:04x}", ctx.ip).blue(),
                "|".dimmed(),
                format!("{:?}", ctx.current_op).magenta(),
            );
            println!("  {}", source_line);
        } else {
            println!("\n{} {} {} {}",
                "---".dimmed(),
                format!("ip={:04x}", ctx.ip).blue(),
                "|".dimmed(),
                format!("{:?}", ctx.current_op).magenta(),
            );
        }

        // REPL loop
        let mut input = String::new();
        loop {
            print!("{} ", "(auto-dbg)".green().bold());
            std::io::Write::flush(&mut std::io::stdout()).ok();
            input.clear();
            if std::io::stdin().read_line(&mut input).is_err() {
                return DebuggerAction::Quit;
            }
            let raw = input.trim();
            if raw.is_empty() {
                continue;
            }
            let parts: Vec<&str> = raw.splitn(2, ' ').collect();
            let cmd = parts[0];
            let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

            match cmd {
                "run" | "r" => {
                    self.started = true;
                    self.step_mode = StepMode::None;
                    return DebuggerAction::Continue;
                }
                "continue" | "c" => {
                    self.started = true;
                    self.step_mode = StepMode::None;
                    return DebuggerAction::Continue;
                }
                "step" | "s" => {
                    self.started = true;
                    self.step_mode = StepMode::StepInto;
                    return DebuggerAction::Step;
                }
                "next" | "n" => {
                    self.started = true;
                    self.last_line = ctx.line;
                    self.step_mode = StepMode::StepOver;
                    return DebuggerAction::Step;
                }
                "finish" | "fin" => {
                    self.started = true;
                    self.call_depth_at_step = ctx.call_stack.len();
                    self.step_mode = StepMode::StepOut;
                    return DebuggerAction::Step;
                }
                "until" | "u" => {
                    if let Ok(line) = arg.parse::<u32>() {
                        self.started = true;
                        self.step_mode = StepMode::UntilLine(line);
                        return DebuggerAction::Step;
                    } else {
                        println!("Usage: until <line_number>");
                    }
                }
                "break" | "b" => {
                    if arg.is_empty() {
                        println!("Usage: break <line | function | function/N | file:line>");
                        continue;
                    }
                    let idx = self.breakpoints.len();

                    // 1. Pure number → line breakpoint
                    if let Ok(line) = arg.parse::<u32>() {
                        self.breakpoints.push(Breakpoint::AtLine(line));
                        self.print_bp_confirmed(idx, &format!("at line {}", line), line);
                        continue;
                    }

                    // 2. Contains colon → file:line or file:fn/N (multi-file, not yet supported)
                    if arg.contains(':') {
                        println!("{} multi-file breakpoints not yet supported.", "Error:".red().bold());
                        println!("  Use: b <line> or b <function> or b <function/N>");
                        continue;
                    }

                    // 3. Contains slash → function/line_offset
                    if let Some(slash_pos) = arg.find('/') {
                        let fn_name = &arg[..slash_pos];
                        let offset_str = &arg[slash_pos + 1..];
                        let offset: u32 = match offset_str.parse() {
                            Ok(n) => n,
                            Err(_) => {
                                println!("{} invalid line offset '{}'", "Error:".red().bold(), offset_str);
                                continue;
                            }
                        };
                        if let Some(&addr) = self.exports_by_name.get(fn_name) {
                            let start_line = self.find_function_start_line(addr as usize);
                            if start_line == 0 {
                                println!("{} could not determine start line for function '{}'", "Error:".red().bold(), fn_name);
                                continue;
                            }
                            let target_line = start_line + offset;
                            self.breakpoints.push(Breakpoint::AtLine(target_line));
                            self.print_bp_confirmed(idx, &format!("at line {} ({} + {})", target_line, fn_name, offset), target_line);
                        } else {
                            println!("{} function '{}' not found.", "Error:".red().bold(), fn_name);
                            let names: Vec<&String> = self.exports_by_name.keys().collect();
                            if !names.is_empty() {
                                println!("  {} {}", "Available:".dimmed(), names.iter().map(|s| s.cyan().to_string()).collect::<Vec<_>>().join(", "));
                            }
                        }
                        continue;
                    }

                    // 4. Plain function name → AtFunction
                    if self.exports_by_name.contains_key(arg) {
                        self.breakpoints.push(Breakpoint::AtFunction(arg.to_string()));
                        let start_line = self.exports_by_name.get(arg)
                            .and_then(|&addr| {
                                let sl = self.find_function_start_line(addr as usize);
                                if sl > 0 { Some(sl) } else { None }
                            })
                            .unwrap_or(0);
                        self.print_bp_confirmed(idx, &format!("at function {}", arg.cyan()), start_line);
                    } else {
                        println!("{} function '{}' not found.", "Error:".red().bold(), arg);
                        let names: Vec<&String> = self.exports_by_name.keys().collect();
                        if !names.is_empty() {
                            println!("  {} {}", "Available:".dimmed(), names.iter().map(|s| s.cyan().to_string()).collect::<Vec<_>>().join(", "));
                        }
                    }
                }
                "delete" | "d" => {
                    if let Ok(idx) = arg.parse::<usize>() {
                        if idx < self.breakpoints.len() {
                            self.breakpoints.remove(idx);
                            println!("{} breakpoint #{}", "Deleted".yellow(), idx);
                        } else {
                            println!("{} breakpoint #{}", "No".red(), idx);
                        }
                    } else {
                        println!("Usage: delete <breakpoint_number>");
                    }
                }
                "info" | "i" => {
                    match arg {
                        "breakpoints" | "b" => {
                            if self.breakpoints.is_empty() {
                                println!("No breakpoints.");
                            } else {
                                for (i, bp) in self.breakpoints.iter().enumerate() {
                                    match bp {
                                        Breakpoint::AtLine(line) => {
                                            let preview = self.source_preview(*line);
                                            println!("  {} at {} {}",
                                                format!("#{}", i).cyan(),
                                                format!("line {}", line).yellow(),
                                                if preview.is_empty() { String::new() } else { format!(": {}", preview.dimmed()) }
                                            );
                                        }
                                        Breakpoint::AtIp(ip) => {
                                            println!("  {} at {} {:04x}", format!("#{}", i).cyan(), "ip".blue(), ip);
                                        }
                                        Breakpoint::AtFunction(name) => {
                                            println!("  {} at {} {}", format!("#{}", i).cyan(), "function".blue(), name.cyan());
                                        }
                                    }
                                }
                            }
                        }
                        "stack" | "s" => {
                            if ctx.call_stack.is_empty() {
                                println!("Call stack: {}", "<top level>".dimmed());
                            } else {
                                println!("{}", "Call stack:".bold());
                                for (i, frame) in ctx.call_stack.iter().enumerate().rev() {
                                    let name = frame.fn_name.as_deref().unwrap_or("<anonymous>");
                                    println!("  {} {} {} {}",
                                        format!("#{}", i).cyan(),
                                        name.green(),
                                        "at line".dimmed(),
                                        frame.line.to_string().yellow(),
                                    );
                                }
                            }
                        }
                        "locals" | "l" => {
                            let bp = ctx.task.bp;
                            let n = ctx.task.current_fn_n_locals;
                            println!("{} ({} slots from bp+1):", "Locals".bold(), n);
                            for i in 0..n {
                                let val = ctx.task.ram.read_i32(bp + 1 + i);
                                println!("  {} = {}", format!("[{}]", i).cyan(), val.to_string().yellow());
                            }
                        }
                        "registers" | "r" => {
                            println!("  {} = {} ({})", "IP ".blue().bold(), format!("{:04x}", ctx.task.ip).yellow(), ctx.task.ip);
                            println!("  {} = {} ({})", "BP ".blue().bold(), format!("{:04x}", ctx.task.bp).yellow(), ctx.task.bp);
                            println!("  {} = {} ({})", "SP ".blue().bold(), format!("{:04x}", ctx.task.ram.sp).yellow(), ctx.task.ram.sp);
                            println!("  {} = {}", "Line".blue().bold(), ctx.task.current_line.to_string().yellow());
                        }
                        _ => {
                            println!("Usage: info <breakpoints|stack|locals|registers>");
                        }
                    }
                }
                "list" | "l" => {
                    self.print_source_context(ctx.line, 5);
                }
                "disassemble" | "disas" => {
                    self.print_disassembly(ctx.ip, 10);
                }
                "print" | "p" => {
                    if let Ok(slot) = arg.parse::<usize>() {
                        let bp = ctx.task.bp;
                        let val = ctx.task.ram.read_i32(bp + 1 + slot);
                        println!("{} = {}", format!("local[{}]", slot).cyan(), val.to_string().yellow());
                    } else {
                        println!("Usage: print <slot_index>");
                    }
                }
                "quit" | "q" => {
                    println!("{}", "Exiting debugger.".yellow());
                    return DebuggerAction::Quit;
                }
                "help" | "h" => {
                    self.show_help();
                }
                _ => {
                    println!("{}: {}. Type {} for commands.", "Unknown command".red(), cmd, "'help'".cyan());
                }
            }
        }
    }
}
