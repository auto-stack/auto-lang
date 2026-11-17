//! PLAN-971 B (T-02): the reference SOURCE-LEVEL debug session over the 960
//! packages lane.
//!
//! One session compiles one project generation, opens its own scratch store
//! and executes the entry chain with the AV's real stepping boundary
//! (`execute_program_steps_after_init` / `resume_program`): every stop is a
//! REAL instruction boundary of that chain, and the reported span comes from
//! the producer's own per-instruction span table of the ORIGINAL module
//! source (never a legacy AST position). The stack and locals are the live
//! AV frames; nothing is re-executed speculatively and no legacy engine is
//! consulted.
//!
//! Breakpoints are matched against the producer's bound declaration table
//! (symbol + module, taken from the same binder facts the check face
//! publishes); a `span: None` breakpoint matches exactly the function's
//! first instruction. Commands are continue / step / step-over / step-out.
//! Closing the session drops the paused chain, the generation and the
//! scratch store (stop → reclaim).

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

use std::time::{Duration, Instant};

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::av::{
    execute_program_steps_after_init, resume_program, CancelFlag, DebugFrame as AvDebugFrame,
    DebugPosition, PausedRun, SteppedOutcome,
};
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::PackageModuleInput;
use ace_source_query::modules::packages::compile_package_product;

use auto_lang_core::{
    AceBreakpoint, AceDebugCommand, AceDebugFrame, AceDebugLocal, AceDebugRequest, AceDebugSession,
    AceDebugSnapshot, AceDebugStartError, AceDebugStop, AceModuleSpec, AceProjectOutcome, AceSpan,
};

use crate::project::{generation_limits, outcome_from_exec, ScratchStore};

/// Frozen default wall budget of one debug session.
const DEFAULT_DEBUG_WALL: Duration = Duration::from_secs(30);

/// `symbol` → `(package, module, original name)` from the binder.
type Origins = BTreeMap<String, (String, String, String)>;

/// Start one debug session (called by the project host's `start_debug`).
pub(crate) fn start(
    request: &AceDebugRequest<'_>,
) -> Result<Box<dyn AceDebugSession>, AceDebugStartError> {
    let cancel = CancelFlag::new(
        request
            .cancel
            .is_some_and(|flag| flag.load(Ordering::Relaxed)),
    );
    let inputs: Vec<PackageModuleInput> = request.modules.iter().map(module_input).collect();
    let product = match compile_package_product(
        &inputs,
        request.entry_module,
        request.capabilities,
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(product) => product,
        Err(diags) => {
            let diagnostic = diags.first().map(|d| auto_lang_core::AceDiagnostic {
                module: d.module.clone(),
                stage: d.diagnostic.stage.as_str().to_string(),
                code: d.diagnostic.code.clone(),
                message: d.diagnostic.message.clone(),
                span: Some(AceSpan {
                    start: d.diagnostic.span.start as u64,
                    end: d.diagnostic.span.end as u64,
                }),
            });
            return Err(AceDebugStartError {
                code: "ace.debug/project-diagnostic".to_string(),
                message: diagnostic
                    .as_ref()
                    .map(|d| d.message.clone())
                    .unwrap_or_else(|| "the project did not compile".to_string()),
                diagnostic,
            });
        }
    };
    let scratch = ScratchStore::new().map_err(|e| AceDebugStartError {
        code: "ace.debug/scratch-open".to_string(),
        message: format!("scratch store open failed: {e}"),
        diagnostic: None,
    })?;
    let target = PackageAvTarget::cold_with_entry(&scratch.path, &product, request.entry_def_id)
        .map_err(|e| AceDebugStartError {
            code: "ace.debug/link".to_string(),
            message: format!("entry link failed: {e}"),
            diagnostic: None,
        })?;
    let deadline = Instant::now() + request.wall.unwrap_or(DEFAULT_DEBUG_WALL);
    // Pause BEFORE the entry's first instruction (budget 0): module
    // initializers have already run for this generation, so the debug chain
    // starts over initialized module cells.
    let paused = match execute_program_steps_after_init(
        &target.current,
        &generation_limits(deadline),
        &cancel,
        0,
    ) {
        SteppedOutcome::Paused(paused) => paused,
        SteppedOutcome::Finished(out, _) => {
            let (code, message) = match out {
                ace_av_core::ExecOutcome::Cancelled => (
                    "ace.debug/cancelled",
                    "the request cancellation flag was set before the entry stopped".to_string(),
                ),
                other => (
                    "ace.debug/entry-unavailable",
                    format!("the entry produced no pausable chain: {other:?}"),
                ),
            };
            return Err(AceDebugStartError {
                code: code.to_string(),
                message,
                diagnostic: None,
            });
        }
    };
    let origins: Origins = product
        .bound
        .origins
        .iter()
        .map(|(symbol, (pkg, module, original))| {
            (
                symbol.clone(),
                (pkg.clone(), module.clone(), original.clone()),
            )
        })
        .collect();
    // The MIR span table is in the FLAT package-program coordinate space;
    // each module's segment start turns a span back into the module-relative
    // (original source) offset every tool-facing fact uses.
    let mut segment_starts: BTreeMap<String, usize> = BTreeMap::new();
    for (pkg, module, _) in product.bound.origins.values() {
        let key = format!("{pkg}::{module}");
        if let Some(start) = product.bound.segment_start(&key) {
            segment_starts.insert(key, start);
        }
    }
    let session = ProjectDebugSession {
        target: Some(target),
        scratch: Some(scratch),
        origins,
        segment_starts,
        deadline,
        cancel,
        breakpoints: request.breakpoints.to_vec(),
        state: Some(paused),
        stop: AceDebugStop::Entry,
        steps: 0,
        outcome: None,
        message: "stopped at the entry's first instruction".to_string(),
    };
    Ok(Box::new(session))
}

fn module_input(spec: &AceModuleSpec<'_>) -> PackageModuleInput {
    PackageModuleInput {
        package: spec.package.to_string(),
        version: spec.version.to_string(),
        id: spec.id.to_string(),
        source: spec.source.to_string(),
        provider: None,
    }
}

fn bare_symbol(name: &str) -> &str {
    name.strip_prefix("d_").unwrap_or(name)
}

/// The live session (owns the paused chain, the generation and the scratch
/// store; closing reclaims them).
struct ProjectDebugSession {
    target: Option<PackageAvTarget>,
    scratch: Option<ScratchStore>,
    origins: Origins,
    /// `pkg::mod` → start offset of that module's segment in the flat
    /// package-program text (MIR spans are flat; tools see module-relative).
    segment_starts: BTreeMap<String, usize>,
    deadline: Instant,
    cancel: CancelFlag,
    breakpoints: Vec<AceBreakpoint>,
    state: Option<PausedRun>,
    stop: AceDebugStop,
    steps: u64,
    outcome: Option<AceProjectOutcome>,
    message: String,
}

impl ProjectDebugSession {
    fn current_position(&self) -> Option<DebugPosition> {
        self.state
            .as_ref()
            .and_then(|paused| paused.debug_position())
    }

    /// Translate an executor position to the producer's source identity:
    /// `(module = pkg::mod, source-level function name)`.
    fn translate(&self, position: &DebugPosition) -> (String, String) {
        let bare = bare_symbol(&position.function);
        match self.origins.get(bare) {
            Some((pkg, module, original)) => (format!("{pkg}::{module}"), original.clone()),
            None => (position.module.clone(), bare.to_string()),
        }
    }

    /// A flat MIR span turned into a module-relative original-source span.
    fn original_span(&self, module: &str, span: ace_av_core::contract::SpanInfo) -> AceSpan {
        let base = self.segment_starts.get(module).copied().unwrap_or(0) as u64;
        AceSpan {
            start: (span.start as u64).saturating_sub(base),
            end: (span.end as u64).saturating_sub(base),
        }
    }

    fn translate_frame(&self, frame: AvDebugFrame) -> AceDebugFrame {
        let raw_function = frame.function.clone();
        let bare = bare_symbol(&frame.function);
        let (module, function) = match self.origins.get(bare) {
            Some((pkg, module, original)) => (format!("{pkg}::{module}"), original.clone()),
            None => (frame.module.clone(), bare.to_string()),
        };
        // Slot names come from the SAME lowering generation the code was
        // admitted from (the ABC wire carries none).
        let slot_names = self
            .target
            .as_ref()
            .and_then(|target| target.slot_names.get(&raw_function));
        let locals: Vec<AceDebugLocal> = frame
            .locals
            .into_iter()
            .filter_map(|local| {
                let name = local.name.or_else(|| {
                    slot_names.and_then(|names| names.get(local.slot as usize).cloned().flatten())
                })?;
                Some(AceDebugLocal {
                    slot: local.slot,
                    name,
                    value: local.value,
                })
            })
            .collect();
        AceDebugFrame {
            span: frame.span.map(|s| self.original_span(&module, s)),
            module,
            function,
            locals,
        }
    }

    fn snapshot_now(&self) -> AceDebugSnapshot {
        let frames = match &self.state {
            Some(paused) => paused
                .debug_snapshot()
                .frames
                .into_iter()
                .map(|f| self.translate_frame(f))
                .collect(),
            None => Vec::new(),
        };
        AceDebugSnapshot {
            stop: self.stop,
            frames,
            steps: self.steps,
            outcome: self.outcome.clone(),
            message: self.message.clone(),
        }
    }

    fn depth(&self) -> usize {
        self.state
            .as_ref()
            .map(|paused| paused.frame_depth())
            .unwrap_or(0)
    }

    /// Does the CURRENT paused position match breakpoint `index`?
    fn breakpoint_matches(&self, index: usize, position: &DebugPosition) -> bool {
        let Some(bp) = self.breakpoints.get(index) else {
            return false;
        };
        let bare = bare_symbol(&position.function);
        let (module, function) = self.translate(position);
        let symbol_ok = bp.symbol == bare || bp.symbol == function;
        if !symbol_ok || bp.module != module {
            return false;
        }
        match bp.span {
            // Function-entry breakpoint: exactly the first instruction.
            None => position.at_function_entry,
            // Span breakpoint: the instruction's original span intersects
            // the requested span (both module-relative).
            Some(span) => position.span.is_some_and(|s| {
                let mapped = self.original_span(&module, s);
                mapped.start < span.end && span.start < mapped.end
            }),
        }
    }

    fn first_breakpoint_at(&self, position: &DebugPosition) -> Option<usize> {
        (0..self.breakpoints.len()).find(|index| self.breakpoint_matches(*index, position))
    }

    /// Finish the session with a terminal outcome (drops the paused chain).
    fn finish_with(
        &mut self,
        outcome: AceProjectOutcome,
        message: impl Into<String>,
    ) -> AceDebugSnapshot {
        self.state = None;
        self.outcome = Some(outcome);
        self.stop = AceDebugStop::Finished;
        self.message = message.into();
        self.snapshot_now()
    }

    /// Execute exactly one instruction. `Some(snapshot)` means the session
    /// ENDED (finished / cancelled / wall); `None` means another real
    /// instruction boundary is available for inspection.
    fn step_once(&mut self) -> Option<AceDebugSnapshot> {
        if self.state.is_none() {
            return Some(self.snapshot_now());
        }
        if self.cancel.load(Ordering::Relaxed) {
            return Some(self.finish_with(AceProjectOutcome::Cancelled, "cancelled"));
        }
        if Instant::now() >= self.deadline {
            return Some(self.finish_with(
                AceProjectOutcome::ResourceDenied {
                    kind: "wall".to_string(),
                    message: "debug session wall budget exhausted".to_string(),
                },
                "wall budget exhausted",
            ));
        }
        let paused = self.state.take().expect("state checked above");
        match resume_program(paused, &self.cancel, Some(1)) {
            SteppedOutcome::Paused(next) => {
                self.steps = next.step_count();
                self.state = Some(next);
                None
            }
            SteppedOutcome::Finished(out, _) => {
                Some(self.finish_with(outcome_from_exec(out), "run finished"))
            }
        }
    }

    fn resume_continue(&mut self) -> AceDebugSnapshot {
        loop {
            if let Some(snapshot) = self.step_once() {
                return snapshot;
            }
            let Some(position) = self.current_position() else {
                return self.snapshot_now();
            };
            if let Some(index) = self.first_breakpoint_at(&position) {
                self.stop = AceDebugStop::Breakpoint { index };
                self.message = format!("stopped on breakpoint #{index}");
                return self.snapshot_now();
            }
        }
    }

    fn resume_step_over(&mut self) -> AceDebugSnapshot {
        let target_depth = self.depth();
        let start = self
            .current_position()
            .map(|p| (p.function.clone(), p.span.map(|s| (s.start, s.end))));
        loop {
            if let Some(snapshot) = self.step_once() {
                return snapshot;
            }
            let depth = self.depth();
            if depth < target_depth {
                self.stop = AceDebugStop::StepOver;
                self.message = "stepped over into the caller frame".to_string();
                return self.snapshot_now();
            }
            if depth == target_depth {
                let now = self
                    .current_position()
                    .map(|p| (p.function.clone(), p.span.map(|s| (s.start, s.end))));
                if now != start {
                    self.stop = AceDebugStop::StepOver;
                    self.message = "stepped over to the next source position".to_string();
                    return self.snapshot_now();
                }
            }
        }
    }

    fn resume_step_out(&mut self) -> AceDebugSnapshot {
        let target_depth = self.depth();
        if target_depth == 0 {
            return self.resume_step_over();
        }
        loop {
            if let Some(snapshot) = self.step_once() {
                return snapshot;
            }
            if self.depth() < target_depth {
                self.stop = AceDebugStop::StepOut;
                self.message = "stepped out to the caller frame".to_string();
                return self.snapshot_now();
            }
        }
    }
}

impl AceDebugSession for ProjectDebugSession {
    fn snapshot(&self) -> AceDebugSnapshot {
        self.snapshot_now()
    }

    fn resume(&mut self, command: AceDebugCommand) -> AceDebugSnapshot {
        if self.stop == AceDebugStop::Finished {
            return self.snapshot_now();
        }
        match command {
            AceDebugCommand::Continue => self.resume_continue(),
            AceDebugCommand::Step => {
                if let Some(snapshot) = self.step_once() {
                    return snapshot;
                }
                self.stop = AceDebugStop::Step;
                self.message = "stepped one instruction".to_string();
                self.snapshot_now()
            }
            AceDebugCommand::StepOver => self.resume_step_over(),
            AceDebugCommand::StepOut => self.resume_step_out(),
        }
    }

    fn close(&mut self) {
        self.state = None;
        self.target = None;
        self.scratch = None;
        if self.stop != AceDebugStop::Finished {
            self.stop = AceDebugStop::Finished;
            self.message = "session closed".to_string();
        }
    }

    fn set_breakpoints(&mut self, breakpoints: &[AceBreakpoint]) {
        self.breakpoints = breakpoints.to_vec();
    }

    fn cancel(&mut self) {
        // The session owns its chain's flag; the start request's flag is only
        // the initial state (a live UI needs a handle after the start call).
        self.cancel.store(true, Ordering::SeqCst);
    }
}
