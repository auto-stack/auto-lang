//! Reusable AV entries for the root of a checked widget package.
//!
//! The entire authored model stays in a verified, generation-local cell. Only
//! scalar event payloads cross the AV entry boundary. The compiler's existing
//! model initializer, transitions, view and timer predicates are called by
//! checked MIR adapters, so events never become source or replay instructions.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ace_av_core::contract::{ExecOutcome, Value, TEXT_RUNTIME_CAPABILITY};
use ace_av_core::function_code::FunctionCode;
use ace_av_core::mir::{
    CalleeRef, FnSig, MirBlock, MirFunction, MirGlobal, MirInstr, MirModule, MirSlot, MirType,
    SlotKind, Terminator,
};
use ace_av_core::mir_verify::MirVerifyError;
pub use ace_av_core::program_snapshot::HostInput;
use ace_av_core::program_snapshot::{PreparedGlobalUpdate, ProgramSnapshot};
use ace_av_core::span_map::SpanSegments;
use ace_av_core::{execute_program_entry_args, EntryArg};
use ace_core::source::age_package::{
    AgePackageCompilation, AgePackageDescription, AgeWidgetRefs, AGE_PACKAGE_HIDDEN_PREFIX,
    FILE_LIST_BINDING, WIDGET_FOCUS_FIELD,
};

use super::{live_view, CancelFlag, ComponentFieldValue, ExecLimits, WidgetViewItem};
use crate::component_event_sessions::field_value_to_arg;

mod children;
mod watch;
use children::{Children, ChildrenProgram};

/// Verified code is shared by all mounts. Each session owns its model and other cells.
pub struct WidgetProgram {
    mir: MirModule,
    codes: Vec<Arc<FunctionCode>>,
    init: String,
    view: String,
    timers: Vec<Option<String>>,
    handlers: BTreeMap<String, String>,
    context_handler: Option<String>,
    directory_reply: Option<HostEntry>,
    child_views: BTreeMap<(String, String), String>,
    children: Option<ChildrenProgram>,
    fields: Vec<(String, String)>,
    model_cell: usize,
    outbound: bool,
    focus_field: Option<usize>,
}

pub struct WidgetSession {
    program: Arc<WidgetProgram>,
    snapshot: ProgramSnapshot,
    children: RefCell<Children>,
    context_token: Option<String>,
    viewport_width: Cell<i32>,
}

fn fresh_context_token() -> String {
    static NEXT_MOUNT: AtomicU64 = AtomicU64::new(1);
    format!("av-{}", NEXT_MOUNT.fetch_add(1, Ordering::Relaxed))
}

/// A focus request from the current mounted component, relative to its view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidgetFocusRequest {
    pub component: Vec<usize>,
    pub target: String,
}

pub struct PreparedWidgetUpdate<'a, T> {
    global: PreparedGlobalUpdate<'a, T>,
    children: &'a RefCell<Children>,
    candidate: Children,
}

impl<T> PreparedWidgetUpdate<'_, T> {
    pub fn commit(self) -> T {
        self.children.replace(self.candidate);
        self.global.commit()
    }
}

struct HostEntry {
    message: String,
    entry: String,
    cell: u32,
}

impl WidgetProgram {
    /// Compile a second-stage AV program from the checked interactive package.
    /// The entry widget must own a model and have no required host inputs.
    pub fn prepare(compiled: &AgePackageCompilation, entry: &str) -> Result<Arc<Self>, String> {
        let (index, root) = compiled
            .description
            .components
            .iter()
            .enumerate()
            .find(|(_, node)| node.module == entry)
            .ok_or("checked package entry has no widget")?;
        let runtime = compiled
            .widget_runtime
            .iter()
            .find(|item| item.node == index)
            .ok_or("entry has no checked widget runtime")?;
        let resolve = |name: &str| {
            compiled
                .symbol_of_in(entry, name)
                .map(|symbol| format!("d_{symbol}"))
                .ok_or_else(|| format!("missing checked widget runtime entry {name:?}"))
        };
        let init_def = resolve(
            runtime
                .initialize
                .as_deref()
                .ok_or("entry widget has no model")?,
        )?;
        let mut mir = ace_lowering::lower_checked_age_package(&compiled.checked, &init_def, 0x981)
            .map_err(|error| format!("lower widget runtime: {error:?}"))?;
        let init_target = find_function(&mir, &init_def)?;
        let init_signature = mir
            .sig_of(mir.funcs[init_target as usize].sig)
            .ok_or("the checked widget initializer has no signature")?;
        if !init_signature.params.is_empty() {
            return Err(
                "ace.desktop.widget-runtime-inputs: widget initializer requires arguments".into(),
            );
        }
        let model = init_signature.result;
        if !matches!(mir.types.get(model as usize), Some(MirType::Record(_))) {
            return Err("the checked widget initializer does not return a model record".into());
        }
        let model_cell = mir.globals.len();
        mir.globals.push(MirGlobal {
            name: format!("{AGE_PACKAGE_HIDDEN_PREFIX}session_model"),
            ty: model,
            mutable: true,
        });
        if !mir.has_text_entry_capability() {
            mir.capabilities.push(TEXT_RUNTIME_CAPABILITY.into());
        }
        let init = add_entry(&mut mir, init_target, model_cell as u32, Entry::Initialize)?;
        mir.entry = find_function(&mir, &init)?;
        let view_target = find_function(&mir, &resolve(&runtime.view)?)?;
        let view = add_entry(&mut mir, view_target, model_cell as u32, Entry::Observe)?;
        let mut child_views = BTreeMap::new();
        for child in compiled
            .widget_runtime
            .iter()
            .filter(|child| child.node != index && child.initialize.is_none())
        {
            let node = &compiled.description.components[child.node];
            let symbol = compiled
                .symbol_of_in(&node.module, &child.view)
                .ok_or_else(|| format!("missing checked view for {}", node.name))?;
            let entry = format!("d_{symbol}");
            let target = find_function(&mir, &entry)?;
            if mir
                .sig_of(mir.funcs[target as usize].sig)
                .is_some_and(|sig| sig.params.is_empty())
            {
                child_views.insert((node.module.clone(), node.name.clone()), entry);
            }
        }
        let mut timers = Vec::new();
        for timer in &runtime.timers {
            timers.push(
                timer
                    .condition
                    .as_deref()
                    .map(|condition| {
                        let target = find_function(&mir, &resolve(condition)?)?;
                        add_entry(&mut mir, target, model_cell as u32, Entry::Observe)
                    })
                    .transpose()?,
            );
        }
        let mut handlers = BTreeMap::new();
        let directory_message = root
            .members
            .iter()
            .find(|row| row.role == "bind" && row.name == FILE_LIST_BINDING)
            .and_then(|row| row.text.as_deref());
        let mut directory_reply = None;
        for handler in &runtime.handlers {
            let target = find_function(&mir, &resolve(&handler.function)?)?;
            let host_cell = if Some(handler.message.as_str()) == directory_message {
                Some(add_directory_cell(&mut mir, target)?)
            } else {
                None
            };
            let name = add_entry(
                &mut mir,
                target,
                model_cell as u32,
                Entry::Transition(host_cell),
            )?;
            if let Some(cell) = host_cell {
                directory_reply = Some(HostEntry {
                    message: handler.message.clone(),
                    entry: name,
                    cell,
                });
            } else {
                handlers.insert(handler.message.clone(), name);
            }
        }
        let children = compiled.widget_runtime.iter().any(|runtime| {
            runtime.outbound || runtime.focus || runtime.context_handler.is_some() || !runtime.watches.is_empty()
        })
            .then(|| ChildrenProgram::prepare(compiled, &mut mir, index, model_cell as u32))
            .transpose()?;
        let mut entries = handlers
            .values()
            .chain(timers.iter().flatten())
            .chain(std::iter::once(&view))
            .chain(child_views.values())
            .chain(directory_reply.iter().map(|reply| &reply.entry))
            .map(|name| find_function(&mir, name))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(children) = &children {
            entries.extend_from_slice(children.entries());
        }
        live_view::retain_closures(&mut mir, &entries)?;
        ace_av_core::mir_verify::verify_mir(&mir).map_err(|error| {
            if let MirVerifyError::Invalid { span: Some(span), .. } = &error {
                // Spans keep their original function coordinates when the
                // runtime's reachable function table is compacted.
                if let Some(function) = mir.funcs.iter().find(|function| {
                    function.spans.iter().any(|(_, candidate)| candidate == span)
                }) {
                    return format!("verify widget runtime MIR in `{}`: {error:?}", function.name);
                }
            }
            format!("verify widget runtime MIR: {error:?}")
        })?;
        let wire = crate::wire::WireCodec::LegacyAbc;
        let codes = crate::cache::split_units(&mir)?
            .into_iter()
            .map(|unit| {
                let name = &unit.funcs[0].name;
                let bytes = wire
                    .encode(&unit)
                    .map_err(|error| format!("encode `{name}`: {error:?}"))?;
                let loaded = wire
                    .admit_bytes(&bytes, &unit.owner)
                    .map_err(|error| format!("admit `{name}`: {error:?}"))?;
                FunctionCode::admit_unit(loaded, 0).map(Arc::new)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Arc::new(Self {
            mir,
            codes,
            init,
            view,
            timers,
            handlers,
            context_handler: runtime.context_handler.clone(),
            directory_reply,
            child_views,
            children,
            fields: root.fields.clone(),
            model_cell,
            outbound: runtime.outbound,
            focus_field: runtime.focus.then_some(root.fields.len() + usize::from(runtime.outbound)),
        }))
    }

    pub fn open(
        self: &Arc<Self>,
        generation: u64,
        limits: &ExecLimits,
    ) -> Result<WidgetSession, String> {
        self.open_with_viewport(generation, 0, limits)
    }

    pub fn open_with_viewport(
        self: &Arc<Self>,
        generation: u64,
        viewport_width: i32,
        limits: &ExecLimits,
    ) -> Result<WidgetSession, String> {
        let inits = self
            .mir
            .init
            .iter()
            .map(|&index| self.mir.funcs[index as usize].name.clone())
            .collect::<Vec<_>>();
        let host_inputs = self
            .directory_reply
            .iter()
            .map(|reply| reply.cell)
            .collect::<Vec<_>>();
        let snapshot = ProgramSnapshot::link_with_host_inputs(
            self.codes.clone(),
            &self.init,
            generation,
            vec![SpanSegments::default(); self.codes.len()],
            &self.mir,
            &inits,
            &host_inputs,
        )?;
        let session = WidgetSession {
            program: Arc::clone(self),
            snapshot,
            children: RefCell::new(Children::default()),
            context_token: self.context_handler.as_ref().map(|_| fresh_context_token()),
            viewport_width: Cell::new(viewport_width.max(0)),
        };
        session.execute(&self.init, &[], limits)?;
        if self.handlers.contains_key("Init") {
            session.dispatch("Init", &[], limits)?;
        } else {
            session.apply_root_context(limits)?;
        }
        Ok(session)
    }
}

impl WidgetSession {
    pub fn set_viewport_width(&self, width: i32) {
        self.viewport_width.set(width.max(0));
    }

    fn apply_context(&self, entry: &str, token: &str, limits: &ExecLimits) -> Result<(), String> {
        let args = [
            ComponentFieldValue::Str(token.to_owned()),
            ComponentFieldValue::Int(self.viewport_width.get()),
        ].iter().map(field_value_to_arg).collect::<Vec<_>>();
        self.execute(entry, &args, limits)?;
        Ok(())
    }

    fn apply_root_context(&self, limits: &ExecLimits) -> Result<(), String> {
        if let Some((handler, token)) = self.program.context_handler.as_ref().zip(self.context_token.as_deref()) {
            self.apply_context(&self.program.handlers[handler], token, limits)?;
        }
        Ok(())
    }

    /// Commit an event and its host replies only if the host accepts the outcome.
    pub fn transaction<T>(
        &self,
        operation: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let previous = self.children.borrow().clone();
        let result = self.snapshot.with_global_transaction(operation);
        if result.is_err() {
            self.children.replace(previous);
        }
        result
    }

    /// Evaluate a host reply and capture the candidate state before a file write.
    pub fn preview_update<T>(
        &self,
        operation: impl FnOnce() -> Result<T, String>,
    ) -> Result<PreparedWidgetUpdate<'_, T>, String> {
        let previous = self.children.borrow().clone();
        let global = self.snapshot.preview_global_update(operation);
        let candidate = self.children.replace(previous);
        global.map(|global| PreparedWidgetUpdate { global, children: &self.children, candidate })
    }

    fn execute(
        &self,
        name: &str,
        args: &[EntryArg<'_>],
        limits: &ExecLimits,
    ) -> Result<Value, String> {
        let (outcome, _) =
            execute_program_entry_args(&self.snapshot, name, limits, &CancelFlag::new(false), args);
        match outcome {
            ExecOutcome::Success { value, .. } => Ok(value),
            ExecOutcome::Infra(error) if error.starts_with("entry expects ")
                || error.starts_with("entry argument #") =>
            {
                Err(format!("source.age.component-event: {error}"))
            }
            other => Err(format!("widget entry `{name}` did not succeed: {other:?}")),
        }
    }

    /// Dispatch a single event. A trap, payload mismatch or limit refusal keeps the prior model.
    pub fn dispatch(
        &self,
        message: &str,
        payload: &[ComponentFieldValue],
        limits: &ExecLimits,
    ) -> Result<(), String> {
        self.transaction(|| {
            let focus = self.transition_root(message, payload, limits)?;
            self.children.borrow_mut().request_root_focus(focus);
            Ok(())
        })
    }

    fn transition_root(
        &self,
        message: &str,
        payload: &[ComponentFieldValue],
        limits: &ExecLimits,
    ) -> Result<Vec<String>, String> {
        if self.program.context_handler.as_deref() == Some(message) {
            return Err("ace.desktop.widget-context: only the host can deliver widget context".into());
        }
        let name = self
            .program
            .handlers
            .get(message)
            .ok_or_else(|| format!("source.age.component-event: unknown widget message {message:?}"))?;
        let args = payload.iter().map(field_value_to_arg).collect::<Vec<_>>();
        self.apply_root_context(limits)?;
        self.execute(name, &args, limits)?;
        self.focus_targets(self.program.model_cell as u32, self.program.focus_field)
    }

    fn focus_targets(&self, model: u32, field: Option<usize>) -> Result<Vec<String>, String> {
        let Some(field) = field else { return Ok(Vec::new()) };
        let cells = self.snapshot.global_cells();
        let Some(Value::Record(record)) = cells.get(model as usize) else {
            return Err("ace.desktop.widget-focus: missing checked model".into());
        };
        let Some(Value::List(queue)) = record.fields.get(field) else {
            return Err(format!("ace.desktop.widget-focus: invalid {WIDGET_FOCUS_FIELD}"));
        };
        let targets = queue.rc.borrow().elems.iter()
            .map(|value| live_view::text(value).map(str::to_owned)).collect();
        targets
    }

    /// Take the focus requests resolved against the latest composed package view.
    pub fn take_focus_requests(&self) -> Vec<WidgetFocusRequest> {
        self.children.borrow_mut().take_focus_requests()
    }

    /// Deliver a child event from the current rendered widget path.
    pub fn dispatch_child(
        &self,
        position: &[usize],
        message: &str,
        payload: &[ComponentFieldValue],
        limits: &ExecLimits,
    ) -> Result<(), String> {
        let program = self.program.children.as_ref().ok_or("widget has no live child transitions")?;
        self.transaction(|| program.dispatch(self, position, message, payload, &mut self.children.borrow_mut(), limits))
    }

    /// Evaluate the authored root tree against the current model. Child sites remain in the tree.
    pub fn view(&self, limits: &ExecLimits) -> Result<Vec<WidgetViewItem>, String> {
        self.snapshot.with_global_transaction(|| {
            live_view::decode_view(self.execute(&self.program.view, &[], limits)?)
        })
    }

    /// Compose the current root view with checked input-free child views.
    pub fn package_view(
        &self,
        description: &AgePackageDescription,
        widget_refs: &AgeWidgetRefs,
        limits: &ExecLimits,
    ) -> Result<Vec<WidgetViewItem>, String> {
        self.transaction(|| {
            if let Some(program) = &self.program.children {
                let (view, next) = program.compose(self, &self.children.borrow(), limits)?;
                self.children.replace(next);
                return Ok(view);
            }
            live_view::compose_with(description, widget_refs, self.view(limits)?, |site, _| {
                let entry = self
                    .program
                    .child_views
                    .get(&(site.module.clone(), site.node.clone()))
                    .ok_or_else(|| format!("missing checked live view for {}", site.node))?;
                live_view::decode_view(self.execute(entry, &[], limits)?)
            })
        })
    }

    pub fn directory_reply(
        &self,
        message: &str,
        value: HostInput,
        limits: &ExecLimits,
    ) -> Result<(), String> {
        let reply = self
            .program
            .directory_reply
            .as_ref()
            .filter(|reply| reply.message == message)
            .ok_or("widget has no matching checked directory reply")?;
        self.transaction(|| {
            self.snapshot
                .with_host_input(&reply.entry, reply.cell, value, limits, || {
                    self.execute(&reply.entry, &[], limits)?;
                    let focus = self.focus_targets(self.program.model_cell as u32, self.program.focus_field)?;
                    self.children.borrow_mut().request_root_focus(focus);
                    Ok(())
                })
        })
    }

    pub fn timer_enabled(&self, index: usize, limits: &ExecLimits) -> Result<bool, String> {
        let condition = self
            .program
            .timers
            .get(index)
            .ok_or("unknown widget runtime timer")?;
        match condition {
            None => Ok(true),
            Some(name) => {
                self.snapshot
                    .with_global_transaction(|| match self.execute(name, &[], limits)? {
                        Value::Bool(enabled) => Ok(enabled),
                        _ => Err("widget timer did not return bool".into()),
                    })
            }
        }
    }

    pub fn scalar_states(&self) -> Result<Vec<(String, ComponentFieldValue)>, String> {
        let cells = self.snapshot.global_cells();
        let Some(Value::Record(model)) = cells.get(self.program.model_cell) else {
            return Err("widget model cell is not initialized".into());
        };
        if self.program.fields.len() + usize::from(self.program.outbound)
            + usize::from(self.program.focus_field.is_some()) != model.fields.len()
        {
            return Err("widget model cell does not match its checked field layout".into());
        }
        self.program
            .fields
            .iter()
            .zip(&model.fields)
            .filter_map(|((name, ty), value)| {
                let value = match (ty.as_str(), value) {
                    ("int" | "i32", Value::I32(value)) => ComponentFieldValue::Int(*value),
                    ("bool", Value::Bool(value)) => ComponentFieldValue::Bool(*value),
                    ("f32" | "float", Value::F32(value)) => ComponentFieldValue::F32(*value),
                    ("str" | "Str", Value::Str(value)) => {
                        match std::str::from_utf8(value.bytes()) {
                            Ok(text) => ComponentFieldValue::Str(text.to_owned()),
                            Err(error) => {
                                return Some(Err(format!(
                                    "widget model `{name}` is not UTF-8: {error}"
                                )))
                            }
                        }
                    }
                    ("int" | "i32" | "bool" | "f32" | "float" | "str" | "Str", _) => {
                        return Some(Err(format!(
                            "widget model `{name}` does not have its checked scalar type"
                        )));
                    }
                    _ => return None,
                };
                Some(Ok((name.clone(), value)))
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
enum Entry {
    Initialize,
    Transition(Option<u32>),
    Observe,
}

fn add_directory_cell(mir: &mut MirModule, target: u32) -> Result<u32, String> {
    let sig = mir
        .sig_of(mir.funcs[target as usize].sig)
        .ok_or("missing directory reply signature")?;
    let [_, payload] = sig.params.as_slice() else {
        return Err("widget directory reply must receive exactly one record".into());
    };
    let ty = match mir.types.get(*payload as usize) {
        Some(MirType::Record(_)) => *payload,
        Some(MirType::RecordView(layout)) => mir.intern_type(MirType::Record(layout.clone())),
        _ => return Err("widget directory reply must receive a record".into()),
    };
    let cell = mir.globals.len() as u32;
    mir.globals.push(MirGlobal {
        name: format!("{AGE_PACKAGE_HIDDEN_PREFIX}directory_reply"),
        ty,
        mutable: false,
    });
    Ok(cell)
}

fn find_function(mir: &MirModule, name: &str) -> Result<u32, String> {
    mir.funcs
        .iter()
        .position(|function| function.name == name)
        .map(|index| index as u32)
        .ok_or_else(|| format!("missing lowered widget entry {name:?}"))
}

fn add_entry(mir: &mut MirModule, target: u32, cell: u32, kind: Entry) -> Result<String, String> {
    let function = &mir.funcs[target as usize];
    let signature = mir
        .sig_of(function.sig)
        .ok_or("missing checked widget signature")?
        .clone();
    let model = mir.globals[cell as usize].ty;
    let receives_model = signature.params.first().is_some_and(|&ty| {
        ty == model
            || matches!(
                (mir.types.get(model as usize), mir.types.get(ty as usize)),
                (Some(MirType::Record(owned)), Some(MirType::RecordView(view))) if owned == view
            )
    });
    let (inputs, output) = match kind {
        Entry::Initialize if signature.params.is_empty() && signature.result == model => (vec![], Some(cell)),
        Entry::Transition(None) if receives_model && signature.result == model => {
            (vec![cell], Some(cell))
        }
        Entry::Transition(Some(input))
            if receives_model && signature.result == model && signature.params.len() == 2 =>
        {
            (vec![cell, input], Some(cell))
        }
        Entry::Observe if receives_model && signature.params.len() == 1 => (vec![cell], None),
        _ => {
            return Err(format!(
                "invalid checked widget {kind:?} signature in {}",
                function.name
            ))
        }
    };
    add_cell_call(mir, target, &inputs, output)
}

fn add_cell_call(
    mir: &mut MirModule,
    target: u32,
    inputs: &[u32],
    output: Option<u32>,
) -> Result<String, String> {
    let function = &mir.funcs[target as usize];
    let signature = mir.sig_of(function.sig).ok_or("missing checked widget signature")?.clone();
    let args = signature.params.get(inputs.len()..).ok_or("missing checked widget inputs")?;
    let owner = function.owner.clone();
    let source_snapshot = function.source_snapshot;
    if output.is_some_and(|cell| mir.globals[cell as usize].ty != signature.result) {
        return Err(format!("checked widget result does not match its cell in {}", function.name));
    }
    let name = format!(
        "d_{AGE_PACKAGE_HIDDEN_PREFIX}session_entry_{}",
        mir.funcs.len()
    );
    let params = args
        .iter()
        .map(|&ty| {
            if matches!(mir.types.get(ty as usize), Some(MirType::Str)) {
                mir.intern_type(MirType::StrView)
            } else {
                ty
            }
        })
        .collect::<Vec<_>>();
    let mut slots = params
        .iter()
        .map(|ty| MirSlot {
            kind: SlotKind::Param,
            ty: *ty,
            name: None,
        })
        .collect::<Vec<_>>();
    let mut instrs = Vec::new();
    let mut operands = Vec::new();
    for &global in inputs {
        let dst = slots.len() as u32;
        slots.push(MirSlot {
            kind: SlotKind::Temp,
            ty: mir.globals[global as usize].ty,
            name: None,
        });
        instrs.push(MirInstr::GlobalGet { dst, global });
        operands.push(dst);
    }
    for (slot, (&external, &internal)) in params.iter().zip(args).enumerate() {
        if internal == external {
            operands.push(slot as u32);
        } else {
            let dst = slots.len() as u32;
            slots.push(MirSlot {
                kind: SlotKind::Temp,
                ty: internal,
                name: None,
            });
            instrs.push(MirInstr::StrToOwned {
                dst,
                view: slot as u32,
            });
            operands.push(dst);
        }
    }
    let dst = slots.len() as u32;
    slots.push(MirSlot {
        kind: SlotKind::Temp,
        ty: signature.result,
        name: None,
    });
    instrs.push(MirInstr::Call {
        dst,
        callee: CalleeRef::Func(target),
        args: operands,
    });
    let (result, returned) = if let Some(global) = output {
        let result = mir.intern_type(MirType::I32);
        let ack = slots.len() as u32;
        slots.push(MirSlot {
            kind: SlotKind::Temp,
            ty: result,
            name: None,
        });
        instrs.push(MirInstr::GlobalSet {
            dst: ack,
            global,
            src: dst,
        });
        (result, ack)
    } else {
        (signature.result, dst)
    };
    let sig = mir.intern_type(MirType::Fn(FnSig { params, result }));
    mir.funcs.push(MirFunction {
        owner,
        name: name.clone(),
        sig,
        slots,
        entry: 0,
        source_snapshot,
        spans: Vec::new(),
        env: None,
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: returned },
        }],
    });
    Ok(name)
}

fn add_scalar_setter(mir: &mut MirModule, cell: u32) -> Result<(String, &'static str), String> {
    let ty = mir.globals[cell as usize].ty;
    let (param, spelling) = match mir.types[ty as usize] {
        MirType::Str => (mir.intern_type(MirType::StrView), "str"),
        MirType::I32 => (ty, "i32"),
        MirType::Bool => (ty, "bool"),
        _ => return Err("ace.desktop.widget-runtime-inputs: scoped property requires a scalar".into()),
    };
    let name = format!("d_{AGE_PACKAGE_HIDDEN_PREFIX}session_entry_{}", mir.funcs.len());
    let mut slots = vec![MirSlot { kind: SlotKind::Param, ty: param, name: None }];
    let mut instrs = Vec::new();
    let source = if ty == param {
        0
    } else {
        slots.push(MirSlot { kind: SlotKind::Temp, ty, name: None });
        instrs.push(MirInstr::StrToOwned { dst: 1, view: 0 });
        1
    };
    let result = mir.intern_type(MirType::I32);
    let output = slots.len() as u32;
    slots.push(MirSlot { kind: SlotKind::Temp, ty: result, name: None });
    instrs.push(MirInstr::GlobalSet { dst: output, global: cell, src: source });
    let sig = mir.intern_type(MirType::Fn(FnSig { params: vec![param], result }));
    mir.funcs.push(MirFunction {
        owner: mir.owner.clone(),
        name: name.clone(),
        sig,
        slots,
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
        blocks: vec![MirBlock { instrs, term: Terminator::Return { value: output } }],
    });
    Ok((name, spelling))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::age_h0::LegacyWidgetAlias;
    use ace_core::source::age_package::compile_age_package_widget_runtime;
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

    #[test]
    fn live_widget_sessions_keep_current_models_and_refuse_invalid_transitions() {
        let source = r#"type Row { name str }
type DirEntry {
    size u64
    mtime i64
}
type DirListing {
    ok bool
    entries List<DirEntry>
}
widget Browser {
    msg { Init, Tick, Pick(str), Divide(int), DirectoryListed(DirListing) }
    bind { "file:list" -> .DirectoryListed }
    model {
        var count int = 0
        var selected str = ""
        var items List<Row> = [Row { name: "资料" }]
    }
    on {
        .Init -> { .count += 1 }
        .Tick -> { .count += 1 }
        .Pick(name str) -> { .selected = name }
        .Divide(divisor int) -> { .count = .count / divisor }
        .DirectoryListed(listing DirListing) -> {
            if listing.ok && listing.entries.len() > 0 {
                if listing.entries[0].size == 18446744073709551615u64 && listing.entries[0].mtime == (-9223372036854775807i64 - 1i64) {
                    .selected = "wide fields retained"
                }
            }
        }
    }
    timer { Tick (every_ms: 250, when: .count < 2) }
    view {
        text f"count ${.count} ${.selected}" {}
        for row in .items { text row.name {} }
    }
}
"#;
        let input = PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: source.into(),
            provider: None,
        };
        let compiled = compile_age_package_widget_runtime(
            &[input],
            "app::app",
            LegacyWidgetAlias::Accept,
            &[PACKAGES_CAPABILITY.into()],
        )
        .expect("checked widget package");
        let program =
            WidgetProgram::prepare(&compiled, "app::app").expect("admit verified widget entries");
        let limits = ExecLimits::default();
        let first = program.open(1, &limits).expect("initialize first mount");
        let second = program.open(2, &limits).expect("initialize second mount");
        assert!(first.timer_enabled(0, &limits).unwrap());
        let states = first.scalar_states().unwrap();
        assert!(first
            .dispatch("Pick", &[ComponentFieldValue::Bool(true)], &limits)
            .is_err());
        assert!(first
            .dispatch("Divide", &[ComponentFieldValue::Int(0)], &limits)
            .is_err());
        assert_eq!(first.scalar_states().unwrap(), states);
        let reply = |size| {
            HostInput::Record(vec![
                (
                    "entries".into(),
                    HostInput::List(vec![HostInput::Record(vec![
                        ("size".into(), size),
                        ("mtime".into(), HostInput::I64(i64::MIN)),
                    ])]),
                ),
                ("ok".into(), HostInput::Bool(true)),
            ])
        };
        assert!(first.dispatch("DirectoryListed", &[], &limits).is_err());
        assert!(first
            .directory_reply("DirectoryListed", reply(HostInput::I64(-1)), &limits)
            .unwrap_err()
            .contains("checked field type"));
        let small = ExecLimits {
            max_slot_bytes: 1,
            ..Default::default()
        };
        assert!(first
            .directory_reply("DirectoryListed", reply(HostInput::U64(u64::MAX)), &small)
            .unwrap_err()
            .contains("size budget"));
        assert_eq!(first.scalar_states().unwrap(), states);
        first
            .directory_reply("DirectoryListed", reply(HostInput::U64(u64::MAX)), &limits)
            .unwrap();
        assert!(first.scalar_states().unwrap().contains(&(
            "selected".into(),
            ComponentFieldValue::Str("wide fields retained".into()),
        )));
        for _ in 0..128 {
            first.dispatch("Tick", &[], &limits).unwrap();
        }
        first
            .dispatch("Pick", &[ComponentFieldValue::Str("目录".into())], &limits)
            .unwrap();
        assert!(!first.timer_enabled(0, &limits).unwrap());
        assert!(second.timer_enabled(0, &limits).unwrap());
        assert_eq!(second.scalar_states().unwrap(), states);
        let view = format!("{:?}", first.view(&limits).unwrap());
        assert!(view.contains("count 129 目录"), "{view}");
        assert!(view.contains("资料"), "{view}");
    }
}
