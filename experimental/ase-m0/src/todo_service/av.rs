use super::storage::{Snapshot, Store};
use super::{Argument, Operation, Reply, ServiceError, Todo, MAX_TEXT_BYTES, MAX_TODOS};
use crate::registry::{Registry, RegistryApp};
use crate::source::read_sources;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::{ExecOutcome, ResourceKind, Value, TEXT_RUNTIME_CAPABILITY};
use ace_av_core::mir::{MirType, RecordTypeRef};
use ace_av_core::{execute_program_entry_args, CancelFlag, EntryArg, ExecLimits};
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{global_symbol, type_symbol, PackageModuleInput};
use ace_source_query::modules::packages::compile_package_product_auto;
use std::cell::Cell;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

pub(super) const MAX_STEPS: u64 = 1_000_000;
pub(super) const MAX_AV_WALL: Duration = Duration::from_millis(500);

pub(super) struct Owner<'a> {
    target: PackageAvTarget,
    entries: [Option<String>; Operation::ALL.len()],
    list_cell: usize,
    durable: Option<Durable<'a>>,
    pub(super) writes: Cell<u64>,
    pub(super) source_hash: String,
}

struct Durable<'a> {
    store: Store<'a>,
    next_id_cell: usize,
}

fn initialization(detail: impl ToString) -> ServiceError {
    ServiceError::Initialization(detail.to_string())
}

impl<'a> Owner<'a> {
    pub(super) fn load(
        registry: &Registry,
        app: &RegistryApp,
        path: &Path,
        store: Option<Store<'a>>,
    ) -> Result<Self, ServiceError> {
        let sources = read_sources(registry, app).map_err(initialization)?;
        let inputs: Vec<_> = sources
            .into_iter()
            .map(|source| PackageModuleInput {
                package: app.package.clone(),
                version: app.version.clone(),
                id: source.module,
                source: source.source,
                provider: None,
            })
            .collect();
        let module = format!("{}::{}", app.package, app.entry);
        let mut product =
            compile_package_product_auto(&inputs, &module, &[], FrontBudget::WORK_OWNED_64MIB)
                .map_err(|diagnostics| {
                    initialization(
                        diagnostics
                            .iter()
                            .map(|d| {
                                format!(
                                    "{}:{}..{} {}: {}",
                                    d.module,
                                    d.diagnostic.span.start,
                                    d.diagnostic.span.end,
                                    d.diagnostic.code,
                                    d.diagnostic.message
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("; "),
                    )
                })?;
        // The host's closed signature table admits borrowed text. ABC/MIR
        // verification and AV admission still enforce the actual parameter slots.
        product
            .compiled
            .checked
            .require_capability(TEXT_RUNTIME_CAPABILITY);
        let mut entries = Vec::with_capacity(Operation::ALL.len());
        for operation in Operation::ALL {
            let name = operation.name();
            if !product.bound.units[&module].exports.contains(name) {
                if matches!(operation, Operation::SetAll) {
                    entries.push(None);
                    continue;
                }
                return Err(initialization(format!("missing public operation {name}")));
            }
            let def = product
                .bound
                .origins
                .iter()
                .find(|(_, (package, unit, original))| {
                    package == &app.package && unit == &app.entry && original == name
                })
                .ok_or_else(|| initialization(format!("missing operation definition {name}")))?;
            entries.push(Some(format!("d_{}", def.0)));
        }
        let entries: [Option<String>; Operation::ALL.len()] =
            entries.try_into().expect("closed operation table");
        let list = entries[Operation::List as usize].as_deref().expect("required list operation");
        let target = PackageAvTarget::cold_with_entry(path, &product, list)
                .map_err(initialization)?;
        let todo_type = format!("t_rec_{}", type_symbol(&app.package, &app.entry, "Todo"));
        for operation in Operation::ALL {
            let Some(entry) = &entries[operation as usize] else { continue };
            let code = target
                .current
                .find_code(entry)
                .ok_or_else(|| {
                    initialization(format!("missing admitted operation {}", operation.name()))
                })?;
            let unit = code.unit().module();
            let type_at = |index: u32| &unit.types[index as usize];
            let MirType::Fn(sig) = type_at(unit.funcs[0].sig) else {
                return Err(initialization("admitted entry is not a function"));
            };
            let params: &[MirType] = match operation {
                Operation::List => &[],
                Operation::Create => &[MirType::StrView],
                Operation::Update => &[MirType::I32, MirType::StrView, MirType::Bool],
                Operation::Delete => &[MirType::I32],
                Operation::Clear | Operation::SetAll => &[MirType::Bool],
            };
            let is_todo = |ty| match type_at(ty) {
                MirType::Record(RecordTypeRef { name, fields, .. }) => {
                    name == &todo_type
                        && fields.iter().map(|f| (f.name.as_str(), type_at(f.ty))).eq([
                            ("id", &MirType::I32),
                            ("text", &MirType::Str),
                            ("done", &MirType::Bool),
                        ])
                }
                _ => false,
            };
            let valid_result = match (operation, type_at(sig.result)) {
                (Operation::List, MirType::List(list)) => is_todo(list.elem),
                (Operation::Create, _) => is_todo(sig.result),
                (Operation::Update, MirType::Variant(option)) => match option.arms.as_slice() {
                    [some, none] => {
                        some.name == "Some"
                            && some.payload.is_some_and(is_todo)
                            && none.name == "None"
                            && none.payload.is_none()
                    }
                    _ => false,
                },
                (Operation::Delete | Operation::Clear | Operation::SetAll, MirType::Bool) => true,
                _ => false,
            };
            if !sig.params.iter().map(|&ty| type_at(ty)).eq(params) || !valid_result {
                return Err(initialization(format!(
                    "signature mismatch for {}",
                    operation.name()
                )));
            }
        }
        if target
            .current
            .code()
            .iter()
            .any(|code| !code.unit().module().host_imports.is_empty())
        {
            return Err(initialization("the Todo service declares host imports"));
        }
        let state_cell = |name| {
            let global = global_symbol(&app.package, "db", name);
            target
                .current
                .globals()
                .iter()
                .position(|cell| cell.name == global)
                .ok_or_else(|| {
                    initialization(format!("the admitted Todo state cell {name} is missing"))
                })
        };
        let list_cell = state_cell("todos")?;
        let durable = store
            .map(|store| -> Result<_, ServiceError> {
                Ok(Durable {
                    store,
                    next_id_cell: state_cell("nextid")?,
                })
            })
            .transpose()?;
        let source_hash = target.source.clone();
        let owner = Self {
            target,
            entries,
            list_cell,
            durable,
            writes: Cell::new(0),
            source_hash,
        };
        if let Some(storage) = &owner.durable {
            let restore_entry = |name, params: &[MirType]| -> Result<String, ServiceError> {
                let (def, _) = product
                    .bound
                    .origins
                    .iter()
                    .find(|(_, (package, unit, original))| {
                        package == &app.package && unit == "db" && original == name
                    })
                    .ok_or_else(|| initialization(format!("missing restore entry {name}")))?;
                let entry = format!("d_{def}");
                let code = owner
                    .target
                    .current
                    .find_code(&entry)
                    .ok_or_else(|| initialization(format!("unlinked restore entry {name}")))?;
                let unit = code.unit().module();
                let signature = unit
                    .sig_of(unit.funcs[0].sig)
                    .ok_or_else(|| initialization(format!("invalid restore entry {name}")))?;
                if !signature
                    .params
                    .iter()
                    .map(|&ty| &unit.types[ty as usize])
                    .eq(params)
                    || unit.types[signature.result as usize] != MirType::Bool
                {
                    return Err(initialization(format!("signature mismatch for {name}")));
                }
                Ok(entry)
            };
            let reset = restore_entry("restore_todo_snapshot", &[MirType::I32])?;
            let row = restore_entry(
                "restore_todo_item",
                &[MirType::I32, MirType::StrView, MirType::Bool],
            )?;
            if let Some(snapshot) = storage.store.load()? {
                owner.restore(snapshot, &reset, &row)?;
            }
        }
        Ok(owner)
    }

    fn restore(&self, snapshot: Snapshot, reset: &str, row: &str) -> Result<(), ServiceError> {
        let limits = ExecLimits {
            max_steps: MAX_STEPS,
            wall: MAX_AV_WALL,
            ..ExecLimits::default()
        };
        let cancel = CancelFlag::default();
        let invoke = |entry, args: &[EntryArg<'_>]| {
            let (outcome, _) =
                execute_program_entry_args(&self.target.current, entry, &limits, &cancel, args);
            match outcome {
                ExecOutcome::Success {
                    value: Value::Bool(true),
                    ..
                } => Ok(()),
                other => Err(initialization(format!(
                    "Todo restore rejected {entry}: {other:?}"
                ))),
            }
        };
        invoke(reset, &[EntryArg::Value(Value::I32(snapshot.next_id))])?;
        for todo in snapshot.todos {
            invoke(
                row,
                &[
                    EntryArg::Value(Value::I32(todo.id)),
                    EntryArg::Text(todo.text.as_bytes().into()),
                    EntryArg::Value(Value::Bool(todo.done)),
                ],
            )?;
        }
        Ok(())
    }

    pub(super) fn initialized(&self) -> bool {
        self.target.current.initialized()
    }

    pub(super) fn call(
        &self,
        operation: Operation,
        args: &[Argument],
        cancel: &CancelFlag,
        deadline: Instant,
        max_steps: u64,
    ) -> (Result<Reply, ServiceError>, Option<u64>) {
        let Some(entry) = &self.entries[operation as usize] else {
            return (Err(ServiceError::UnknownOperation), None);
        };
        if matches!(operation, Operation::Create) && self.initialized() {
            match self.target.current.global_cells().get(self.list_cell) {
                Some(Value::List(list)) if list.len() >= MAX_TODOS => {
                    return (Err(ServiceError::Budget("todo.list-items")), None);
                }
                Some(Value::List(_)) => {}
                _ => {
                    return (
                        Err(ServiceError::Execution(
                            "Todo state has an invalid list cell".into(),
                        )),
                        None,
                    )
                }
            }
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return (
                Err(ServiceError::TimedOut {
                    may_have_committed: false,
                }),
                None,
            );
        }
        let limits = ExecLimits {
            max_steps,
            wall: remaining.min(MAX_AV_WALL),
            ..ExecLimits::default()
        };
        let args: Vec<_> = args
            .iter()
            .map(|arg| match arg {
                Argument::Text(text) => EntryArg::Text(text.as_bytes().into()),
                Argument::Int(int) => EntryArg::Value(Value::I32(*int)),
                Argument::Bool(bool) => EntryArg::Value(Value::Bool(*bool)),
            })
            .collect();
        let snapshot = &self.target.current;
        let mut seq = None;
        let dispatch = || {
            let (outcome, trace) = execute_program_entry_args(
                snapshot,
                entry,
                &limits,
                cancel,
                &args,
            );
            seq = Some(trace.execution_seq);
            let reply = match outcome {
                ExecOutcome::Success { value, .. } => decode(operation, &value)?,
                ExecOutcome::Cancelled => {
                    return Err(ServiceError::Cancelled {
                        may_have_committed: false,
                    })
                }
                ExecOutcome::Resource {
                    kind: ResourceKind::ExecutionWall,
                    ..
                } if remaining <= MAX_AV_WALL => {
                    return Err(ServiceError::TimedOut {
                        may_have_committed: false,
                    });
                }
                ExecOutcome::Resource { kind, .. } => {
                    return Err(ServiceError::Budget(kind.code()))
                }
                other => return Err(ServiceError::Execution(format!("{other:?}"))),
            };
            let cells = snapshot.global_cells();
            let Some(Value::List(list)) = cells.get(self.list_cell) else {
                return Err(ServiceError::Execution(
                    "Todo state has an invalid list cell".into(),
                ));
            };
            if list.len() > MAX_TODOS {
                return Err(ServiceError::Budget("todo.list-items"));
            }
            if let Some(durable) = &self.durable {
                if matches!(
                    &reply,
                    Reply::Created(_) | Reply::Updated(Some(_)) | Reply::Flag(true)
                ) {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(ServiceError::Cancelled {
                            may_have_committed: false,
                        });
                    }
                    if Instant::now() >= deadline {
                        return Err(ServiceError::TimedOut {
                            may_have_committed: false,
                        });
                    }
                    let Some(Value::I32(next_id)) = cells.get(durable.next_id_cell) else {
                        return Err(result_type());
                    };
                    let todos = list
                        .rc
                        .borrow()
                        .elems
                        .iter()
                        .map(decode_todo)
                        .collect::<Result<_, _>>()?;
                    durable.store.save(&Snapshot::new(*next_id, todos))?;
                    self.writes.set(self.writes.get() + 1);
                }
            }
            Ok(reply)
        };
        let result = if matches!(operation, Operation::Create) {
            snapshot.with_forked_global_list(
                self.list_cell,
                || ServiceError::Execution("unable to fork the checked Todo list".into()),
                dispatch,
            )
        } else {
            snapshot.with_global_transaction(dispatch)
        };
        (result, seq)
    }
}

fn decode_todo(value: &Value) -> Result<Todo, ServiceError> {
    let Value::Record(record) = value else {
        return Err(result_type());
    };
    let [Value::I32(id), Value::Str(text), Value::Bool(done)] = record.fields.as_slice() else {
        return Err(result_type());
    };
    if text.bytes().len() > MAX_TEXT_BYTES {
        return Err(ServiceError::Budget("todo.text-bytes"));
    }
    let text = std::str::from_utf8(text.bytes())
        .map_err(|_| result_type())?
        .to_owned();
    Ok(Todo {
        id: *id,
        text,
        done: *done,
    })
}

fn result_type() -> ServiceError {
    ServiceError::Execution("invalid checked Todo result".into())
}

fn decode(operation: Operation, value: &Value) -> Result<Reply, ServiceError> {
    match (operation, value) {
        (Operation::List, Value::List(list)) => {
            let list = list.rc.borrow();
            if list.elems.len() > MAX_TODOS {
                return Err(ServiceError::Budget("todo.list-items"));
            }
            list.elems
                .iter()
                .map(decode_todo)
                .collect::<Result<_, _>>()
                .map(Reply::List)
        }
        (Operation::Create, _) => decode_todo(value).map(Reply::Created),
        (Operation::Update, Value::Variant(option)) => {
            match (option.disc, option.payload.as_deref()) {
                (0, Some(todo)) => decode_todo(todo).map(|todo| Reply::Updated(Some(todo))),
                (1, None) => Ok(Reply::Updated(None)),
                _ => Err(result_type()),
            }
        }
        (Operation::Delete | Operation::Clear | Operation::SetAll, Value::Bool(ok)) => Ok(Reply::Flag(*ok)),
        _ => Err(result_type()),
    }
}
