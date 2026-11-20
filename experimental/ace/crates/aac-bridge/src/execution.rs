use crate::foreign::Provider;
use crate::protocol::{hash_bytes, CtFacts, Object, SourceSpan};
use crate::ForeignPermit;
use crate::{
    Domains, Failure, FailureKind, Mode, NativeRuntime, Outcome, Request, Value, RESPONSE_CAP,
};
use ace_av_core::abc::{decode_abc, AbcDecodeError};
use ace_av_core::abc_verify::AbcAdmissionError;
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::TEXT_RUNTIME_CAPABILITY;
use ace_av_core::mir::{CalleeRef, MirGlobal, MirInstr, MirModule, MirType, OwnerScope};
use ace_av_core::{ExecOutcome, MirVerifyError, Value as AvValue};
use ace_native::{emit, MnError};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(crate) fn run(
    request: Request,
    bytes: &[u8],
    started: Instant,
    permit: Option<&ForeignPermit>,
) -> Result<(Outcome, Vec<u8>), Failure> {
    if !cfg!(all(windows, target_arch = "x86_64", target_env = "msvc"))
        || request.domains != Domains::windows(request.mode)
    {
        return Err(Failure::admission(
            "bridge.domains",
            "build, host, target, execution domain or ABI is outside the Windows x64 service contract",
        ));
    }
    remaining(&request, started)?;
    let mut decoded = decode_abc(bytes).map_err(abc_decode_failure)?;
    let module = &mut decoded.module;
    let expected_schema = match module.profile.as_str() {
        ace_av_core::contract::AV_PROFILE => ace_av_core::contract::MIR_SCHEMA,
        ace_av_core::contract::AV_DATA_PROFILE => ace_av_core::contract::MIR_DATA_SCHEMA,
        ace_av_core::contract::AV_OWNED_PROFILE => ace_av_core::contract::MIR_OWNED_SCHEMA,
        ace_av_core::contract::AV_COLLECTIONS_PROFILE => {
            ace_av_core::contract::MIR_COLLECTIONS_SCHEMA
        }
        ace_av_core::contract::AV_CLOSURES_PROFILE => ace_av_core::contract::MIR_CLOSURES_SCHEMA,
        ace_av_core::contract::AV_DYNAMIC_PROFILE => ace_av_core::contract::MIR_DYNAMIC_SCHEMA,
        ace_av_core::contract::AV_ENTRY_UNIT_PROFILE => {
            ace_av_core::contract::MIR_ENTRY_UNIT_SCHEMA
        }
        ace_av_core::contract::AV_HOST_CONSOLE_PROFILE => {
            ace_av_core::contract::MIR_HOST_CONSOLE_SCHEMA
        }
        _ => {
            return Err(Failure::admission(
                "bridge.mir-profile",
                "unsupported ABC family",
            ))
        }
    };
    if request.mir_schema != expected_schema || request.owner.is_empty() {
        return Err(Failure::admission(
            "bridge.mir-identity",
            "logical MIR schema or owner differs from the request's admitted profile",
        ));
    }
    if request.capabilities != module.capabilities {
        return Err(Failure::admission(
            "bridge.capability",
            "request capabilities differ from the typed closure",
        ));
    }
    let foreign = match request.external_provider {
        Some(hash) if matches!(request.mode, Mode::CompileTime | Mode::NativeObjects) => {
            Some(permit.filter(|permit| permit.hash == hash).ok_or_else(|| {
                Failure::admission(
                    "bridge.foreign-authority",
                    "no matching host-owned provider binding is installed",
                )
            })?)
        }
        Some(_) => {
            return Err(Failure::admission(
                "bridge.foreign-domain",
                "foreign provider binding requires the native-object or CT execution domain",
            ))
        }
        None => None,
    };
    if request.mode != Mode::NativeScheduler
        && (!module.host_imports.is_empty()
            || module.capabilities.iter().any(|cap| {
                !pure_capability(cap)
                    && !(foreign.is_some()
                        && matches!(
                            cap.as_str(),
                            ace_foreign_contract::manual::SOURCE_CAP
                                | ace_foreign_contract::resource::RESOURCE_CAP
                        ))
            }))
    {
        return Err(Failure::admission(
            "bridge.capability",
            "the selected service rejects undeclared or ambient host imports",
        ));
    }
    if module.is_unit {
        return Err(Failure::admission(
            "bridge.closure",
            "service requires a complete closure, not a detached unit",
        ));
    }
    bind_program_state(&request, module)?;
    let owner = OwnerScope {
        schema: request.mir_schema.clone(),
        module: request.owner.clone(),
    };
    if let (Mode::NativeObjects, Some(permit)) = (request.mode, foreign) {
        if !module.globals.is_empty() || !module.init.is_empty() {
            return Err(Failure::admission(
                "bridge.foreign-program-state",
                "native foreign emission requires a stateless closure",
            ));
        }
        let authority = permit.admit(module, false)?;
        let checked =
            ace_av_core::abc_verify::verify_abc_native_foreign(decoded, &owner, &authority)
                .map_err(abc_admission_failure)?;
        let _provider = permit.verified_image()?;
        remaining(&request, started)?;
        return native_objects(checked.module(), Mode::NativeObjects, Some(&authority));
    }
    let (loaded, provider) = if let Some(permit) = foreign {
        if !module.globals.is_empty() || !module.init.is_empty() {
            return Err(Failure::admission(
                "bridge.foreign-program-state",
                "foreign CT runs in one checked stateless closure",
            ));
        }
        permit.admit(module, true)?;
        module.owner = owner.clone();
        for function in &mut module.funcs {
            function.owner = owner.clone();
        }
        ace_av_core::verify_mir(module)
            .map_err(|error| abc_admission_failure(AbcAdmissionError::Verify(error)))?;
        remaining(&request, started)?;
        let provider = permit.load(module)?;
        let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &owner, &provider.stubs)
            .map_err(abc_admission_failure)?;
        (loaded, Some(provider))
    } else {
        (
            ace_av_core::verify_abc(decoded, &owner).map_err(abc_admission_failure)?,
            None,
        )
    };
    if !loaded.executable() {
        return Err(Failure::admission(
            "bridge.closure",
            "a complete closure with resolved functions is required",
        ));
    }
    let budget = remaining(&request, started)?;
    #[cfg(test)]
    EXECUTION_ENTRIES.with(|entries| entries.set(entries.get() + 1));
    match request.mode {
        Mode::NativeObjects | Mode::NativeCollections => {
            native_objects(loaded.module(), request.mode, None)
        }
        Mode::NativeScheduler => native_scheduler(loaded.module()),
        Mode::CompileTime => ct(&loaded, budget, provider),
    }
}

fn pure_capability(capability: &str) -> bool {
    matches!(
        capability,
        TEXT_RUNTIME_CAPABILITY
            | "av.aggregates/1"
            | ace_av_core::contract::GLOBAL_TAKE_CAPABILITY
            | ace_av_core::contract::LANGUAGE_VALUES_CAPABILITY
    )
}

fn bind_program_state(request: &Request, module: &mut MirModule) -> Result<(), Failure> {
    let mut names = BTreeSet::new();
    for global in &request.globals {
        if global.name.is_empty()
            || !names.insert(&global.name)
            || module.types.get(global.ty as usize).is_none()
        {
            return Err(Failure::admission(
                "bridge.global-declaration",
                "invalid name or type in the typed global table",
            ));
        }
    }
    let mut initializers = BTreeSet::new();
    for initializer in &request.initializers {
        let signature = module
            .funcs
            .get(*initializer as usize)
            .and_then(|function| module.sig_of(function.sig));
        if *initializer == module.entry
            || !initializers.insert(*initializer)
            || signature.is_none_or(|signature| !signature.params.is_empty())
        {
            return Err(Failure::admission(
                "bridge.initializer",
                "initializer must be a distinct zero-argument function in the closure",
            ));
        }
    }
    for (function_index, function) in module.funcs.iter().enumerate() {
        for instruction in function.blocks.iter().flat_map(|block| &block.instrs) {
            match instruction {
                MirInstr::Call {
                    callee: CalleeRef::Func(target),
                    ..
                }
                | MirInstr::CreateClosure {
                    callee: CalleeRef::Func(target),
                    ..
                }
                | MirInstr::SchedSpawn { callee: target, .. }
                | MirInstr::SchedGo { callee: target, .. }
                    if initializers.contains(target) =>
                {
                    return Err(Failure::admission(
                        "bridge.initializer-reference",
                        format!("{} refers to a startup-only initializer", function.name),
                    ));
                }
                _ => {}
            }
            let (global, may_write) = match instruction {
                MirInstr::GlobalGet { global, .. } => (*global, false),
                MirInstr::GlobalTake { global, .. } => (*global, true),
                MirInstr::GlobalSet { dst, global, .. } => {
                    let result = function
                        .slots
                        .get(*dst as usize)
                        .and_then(|slot| module.types.get(slot.ty as usize));
                    if !matches!(result, Some(MirType::I32)) {
                        return Err(Failure::admission(
                            "global.result-type",
                            format!("{} requires an i32 global store result", function.name),
                        ));
                    }
                    (*global, true)
                }
                _ => continue,
            };
            let decl = request.globals.get(global as usize).ok_or_else(|| {
                Failure::admission(
                    "global.reference",
                    format!("{} references absent cell #{global}", function.name),
                )
            })?;
            if may_write && !decl.mutable && !initializers.contains(&(function_index as u32)) {
                return Err(Failure::admission(
                    "global.immutable",
                    format!("{} writes immutable cell {}", function.name, decl.name),
                ));
            }
        }
    }
    module.globals = request
        .globals
        .iter()
        .map(|global| MirGlobal {
            name: global.name.clone(),
            ty: global.ty,
            mutable: global.mutable,
        })
        .collect();
    module.init.clone_from(&request.initializers);
    Ok(())
}

fn remaining(request: &Request, started: Instant) -> Result<Duration, Failure> {
    let max = request.mode.budget().min(Duration::from_millis(u64::from(
        request.options.remaining_ms,
    )));
    let left = max.saturating_sub(started.elapsed());
    if left.is_zero() {
        Err(Failure::resource(
            "bridge.deadline",
            "the request has no remaining wall budget",
        ))
    } else {
        Ok(left)
    }
}

fn abc_decode_failure(error: AbcDecodeError) -> Failure {
    match error {
        AbcDecodeError::Resource { kind, message } => Failure::resource(kind.code(), message),
        error => Failure::admission(error.code(), format!("{error:?}")),
    }
}

fn abc_admission_failure(error: AbcAdmissionError) -> Failure {
    match error {
        AbcAdmissionError::Verify(MirVerifyError::Resource { kind, message }) => {
            Failure::resource(kind.code(), message)
        }
        AbcAdmissionError::Verify(MirVerifyError::Invalid {
            code,
            message,
            span,
        })
        | AbcAdmissionError::Verify(MirVerifyError::Unsupported {
            code,
            message,
            span,
        }) => Failure {
            kind: FailureKind::Admission,
            code,
            message,
            span: span.map(SourceSpan::from),
        },
        AbcAdmissionError::Decode(error) => abc_decode_failure(error),
        AbcAdmissionError::Identity(message) => Failure::admission("bridge.abc-identity", message),
    }
}

fn native_failure(error: MnError) -> Failure {
    let kind = match &error {
        MnError::Unsupported(..) | MnError::Admit(..) => FailureKind::Admission,
        MnError::Resource(..) => FailureKind::Resource,
        MnError::Source(..) | MnError::Backend(..) | MnError::Infra(..) => FailureKind::Tool,
    };
    Failure::new(kind, error.code(), error.message())
}

fn append_object(object: emit::UnitObject, payload: &mut Vec<u8>) -> Result<Object, Failure> {
    let len = object.bytes.len();
    if len > ace_native::config::MAX_COFF_BYTES
        || payload
            .len()
            .checked_add(len)
            .is_none_or(|len| len > RESPONSE_CAP)
    {
        return Err(Failure::resource(
            "bridge.object-bytes",
            "machine object output exceeds the existing native budget",
        ));
    }
    let hash = hash_bytes(&object.bytes);
    payload.extend_from_slice(&object.bytes);
    Ok(Object {
        symbol: object.symbol,
        imports: object.imports,
        len: len as u32,
        hash,
    })
}

fn native_objects(
    module: &MirModule,
    mode: Mode,
    foreign: Option<&ace_foreign_contract::manual::AdmittedForeign>,
) -> Result<(Outcome, Vec<u8>), Failure> {
    let collections = mode == Mode::NativeCollections;
    let stateful = !module.globals.is_empty() || !module.init.is_empty();
    if stateful && !collections {
        emit::admit_program_startup(module).map_err(native_failure)?;
    }
    let entry = &module.funcs[module.entry as usize];
    let sig = module.sig_of(entry.sig).expect("verified entry signature");
    if !sig.params.is_empty()
        || !matches!(
            module.types[sig.result as usize],
            MirType::I32 | MirType::Bool | MirType::Char
        )
    {
        return Err(Failure::admission(
            "bridge.native-entry",
            "startup wrapper requires a zero-argument i32-family entry",
        ));
    }
    let entry = ace_native::symbols::function_symbol(&module.owner, &entry.name);
    let units = ace_av::cache::split_units(module)
        .map_err(|message| Failure::admission("bridge.native-units", message))?;
    if foreign.is_none() {
        for unit in &units {
            (if collections {
                ace_native::collections_lower::admit_unit(unit)
            } else {
                emit::admit_unit(unit)
            })
            .map_err(native_failure)?;
        }
    }
    let mut payload = Vec::new();
    let mut objects = Vec::with_capacity(units.len());
    for unit in &units {
        let object = if let Some(foreign) = foreign {
            emit::emit_foreign_unit_coff(unit, foreign)
        } else if collections {
            ace_native::collections_lower::emit_unit_coff(unit)
        } else if stateful {
            emit::emit_program_unit_coff(unit)
        } else {
            emit::emit_unit_coff(unit)
        }
        .map_err(native_failure)?;
        objects.push(append_object(object, &mut payload)?);
    }
    let wrapper = append_object(
        emit::UnitObject {
            symbol: ace_native::config::START_SYMBOL.into(),
            imports: Vec::new(),
            bytes: if collections && stateful {
                ace_native::collections_lower::emit_program_wrapper_coff(module)
            } else if collections {
                ace_native::collections_lower::emit_wrapper_coff(&entry, &Default::default())
            } else if stateful {
                emit::emit_program_wrapper_coff(module)
            } else {
                emit::emit_wrapper_coff(&entry)
            }
            .map_err(native_failure)?,
        },
        &mut payload,
    )?;
    let runtime = if collections {
        NativeRuntime::Collections
    } else if module
        .capabilities
        .iter()
        .any(|cap| cap == TEXT_RUNTIME_CAPABILITY)
    {
        NativeRuntime::Text
    } else {
        NativeRuntime::None
    };
    Ok((
        Outcome::Objects {
            entry,
            units: objects,
            wrapper,
            runtime,
        },
        payload,
    ))
}

fn native_scheduler(module: &MirModule) -> Result<(Outcome, Vec<u8>), Failure> {
    if !module.globals.is_empty() || !module.init.is_empty() {
        return Err(Failure::admission(
            "bridge.scheduler-program-state",
            "scheduler ABI needs a program-owned cell provider",
        ));
    }
    let entry = &module.funcs[module.entry as usize].name;
    let program =
        ace_native::sched::emit_sched_service_objects(module, entry).map_err(native_failure)?;
    let mut payload = Vec::new();
    let units = program
        .units
        .into_iter()
        .map(|unit| append_object(unit, &mut payload))
        .collect::<Result<Vec<_>, _>>()?;
    let wrapper = append_object(
        emit::UnitObject {
            symbol: ace_native::config::START_SYMBOL.into(),
            imports: Vec::new(),
            bytes: program.wrapper,
        },
        &mut payload,
    )?;
    Ok((
        Outcome::Objects {
            entry: program.entry_symbol,
            units,
            wrapper,
            runtime: NativeRuntime::Scheduler,
        },
        payload,
    ))
}

fn ct_program(
    module: &MirModule,
    limits: ExecLimits,
) -> Result<(ExecOutcome, ace_av_core::ExecTrace), Failure> {
    let started = Instant::now();
    let units = ace_av::cache::split_units(module)
        .map_err(|message| Failure::admission("bridge.ct-units", message))?;
    let mut code = Vec::with_capacity(units.len());
    for unit in units {
        if started.elapsed() >= limits.wall {
            return Err(Failure::resource(
                "bridge.deadline",
                "CT function admission exhausted the execution budget",
            ));
        }
        let wire = ace_av_core::encode_abc(&unit).map_err(abc_decode_failure)?;
        let mut decoded = decode_abc(&wire.to_flat()).map_err(abc_decode_failure)?;
        decoded.module.globals = unit.globals;
        let loaded =
            ace_av_core::verify_abc(decoded, &module.owner).map_err(abc_admission_failure)?;
        let function = ace_av_core::FunctionCode::admit_unit(loaded, 0)
            .map_err(|message| Failure::admission("bridge.ct-function", message))?;
        code.push(Arc::new(function));
    }
    let init_names = module
        .init
        .iter()
        .map(|&index| module.funcs[index as usize].name.clone())
        .collect::<Vec<_>>();
    let span_maps = vec![Default::default(); code.len()];
    let entry = &module.funcs[module.entry as usize].name;
    let program = ace_av_core::ProgramSnapshot::link_with_checked_globals(
        code,
        entry,
        0,
        span_maps,
        module,
        &init_names,
    )
    .map_err(|message| Failure::admission("bridge.ct-program", message))?;
    let wall = limits.wall.saturating_sub(started.elapsed());
    if wall.is_zero() {
        return Err(Failure::resource(
            "bridge.deadline",
            "CT program linking exhausted the execution budget",
        ));
    }
    Ok(ace_av_core::execute_program(
        &program,
        &ExecLimits { wall, ..limits },
        &CancelFlag::default(),
    ))
}

fn ct(
    loaded: &ace_av_core::LoadedModule,
    wall: Duration,
    mut provider: Option<Provider>,
) -> Result<(Outcome, Vec<u8>), Failure> {
    let limits = ExecLimits {
        wall: wall.min(ExecLimits::default().wall),
        ..Default::default()
    };
    let module = loaded.module();
    let (outcome, trace) = if let Some(provider) = provider.as_mut() {
        ace_av_core::av::execute_foreign(
            loaded,
            &limits,
            &CancelFlag::default(),
            &mut provider.stubs,
        )
        .map_err(|error| {
            Failure::admission(
                "bridge.foreign-execution",
                format!("{}: {}", error.code, error.message),
            )
        })?
    } else if module.globals.is_empty() && module.init.is_empty() {
        ace_av_core::execute(loaded, &limits, &CancelFlag::default())
    } else {
        ct_program(module, limits)?
    };
    match outcome {
        ExecOutcome::Success { value, steps } => {
            let (value, bytes) = portable(value)?;
            let live_text = trace.arena.as_ref().map_or(0, |arena| arena.live()) as u64;
            if live_text != 0 || trace.frames_remaining != 0 {
                return Err(Failure::tool(
                    "bridge.ct-ownership",
                    "AV left live resources after the result was copied",
                ));
            }
            let facts = CtFacts {
                steps,
                peak_slot_bytes: trace.peak_slot_bytes,
                root_frames: trace.root_frame_commits,
                live_text,
                foreign_calls: provider
                    .as_ref()
                    .map_or(0, |provider| provider.stubs.callee_count()),
                read_set: provider
                    .into_iter()
                    .map(|provider| provider.identity)
                    .collect(),
            };
            Ok((Outcome::Value { value, facts }, bytes))
        }
        ExecOutcome::Trap { kind, span } => Err(Failure {
            kind: FailureKind::Language,
            code: kind.code().into(),
            message: kind.code().into(),
            span: span.map(SourceSpan::from),
        }),
        ExecOutcome::Error { message, span } => Err(Failure {
            kind: FailureKind::Language,
            code: "bridge.ct-error".into(),
            message,
            span: span.map(SourceSpan::from),
        }),
        ExecOutcome::Resource { kind, message } => Err(Failure::resource(kind.code(), message)),
        ExecOutcome::Cancelled => Err(Failure::new(
            FailureKind::Cancelled,
            "bridge.ct-cancelled",
            "AV cancelled",
        )),
        ExecOutcome::Infra(message) => Err(Failure::tool("bridge.ct-infrastructure", message)),
        ExecOutcome::Exit { .. } => Err(Failure::tool(
            "bridge.ct-exit",
            "AV attempted exit without a process capability",
        )),
    }
}

fn portable(value: AvValue) -> Result<(Value, Vec<u8>), Failure> {
    let value = match value {
        AvValue::I32(v) => Value::I32(v),
        AvValue::I64(v) => Value::I64(v),
        AvValue::U64(v) => Value::U64(v),
        AvValue::I8(v) => Value::I8(v),
        AvValue::U8(v) => Value::U8(v),
        AvValue::U32(v) => Value::U32(v),
        AvValue::F32(v) => Value::F32Bits(v.to_bits()),
        AvValue::F64(v) => Value::F64Bits(v.to_bits()),
        AvValue::Bool(v) => Value::Bool(v),
        AvValue::Char(v) => Value::Char(v),
        AvValue::Nil => Value::Nil,
        AvValue::Unit => Value::Unit,
        AvValue::Str(v) => {
            let bytes = v.bytes();
            if bytes.len() > RESPONSE_CAP {
                return Err(Failure::resource(
                    "bridge.ct-value-size",
                    "text result exceeds the output budget",
                ));
            }
            return Ok((
                Value::Text {
                    len: bytes.len() as u32,
                },
                bytes.to_vec(),
            ));
        }
        _ => {
            return Err(Failure::admission(
                "bridge.ct-value-type",
                format!(
                    "{} is not a portable compile-time result",
                    value.type_name()
                ),
            ))
        }
    };
    Ok((value, Vec::new()))
}

#[cfg(test)]
std::thread_local! {
    pub(crate) static EXECUTION_ENTRIES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}
