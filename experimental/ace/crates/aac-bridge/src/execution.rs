use crate::protocol::{hash_bytes, CtFacts, Object, SourceSpan};
use crate::{Domains, Failure, FailureKind, Mode, Outcome, Request, Value, RESPONSE_CAP};
use ace_av_core::abc::{decode_abc, AbcDecodeError};
use ace_av_core::abc_verify::AbcAdmissionError;
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::TEXT_RUNTIME_CAPABILITY;
use ace_av_core::mir::{MirGlobal, MirModule, MirType, OwnerScope};
use ace_av_core::{ExecOutcome, MirVerifyError, Value as AvValue};
use ace_native::{emit, MnError};
use std::time::{Duration, Instant};

pub(crate) fn run(
    request: Request,
    bytes: &[u8],
    started: Instant,
) -> Result<(Outcome, Vec<u8>), Failure> {
    if !cfg!(all(windows, target_arch = "x86_64", target_env = "msvc"))
        || request.domains != Domains::windows(request.mode)
    {
        return Err(Failure::admission(
            "bridge.domains",
            "build, host, target, execution domain or ABI is outside the Windows x64 service contract",
        ));
    }
    let budget = remaining(&request, started)?;
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
    if request.capabilities != module.capabilities
        || module
            .capabilities
            .iter()
            .any(|cap| cap != TEXT_RUNTIME_CAPABILITY)
        || !module.host_imports.is_empty()
    {
        return Err(Failure::admission(
            "bridge.capability",
            "service profile grants only guest-owned text and no host imports",
        ));
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
    let owner = OwnerScope {
        schema: request.mir_schema.clone(),
        module: request.owner.clone(),
    };
    let loaded = ace_av_core::verify_abc(decoded, &owner).map_err(abc_admission_failure)?;
    if !loaded.executable() {
        return Err(Failure::admission(
            "bridge.closure",
            "a complete closure with resolved functions is required",
        ));
    }
    // The service owns execution only. Hostless baseline accepts a closure
    // without program-owned state; those cells need an explicit runtime lane.
    if !loaded.module().globals.is_empty() || !loaded.module().init.is_empty() {
        return Err(Failure::admission(
            "bridge.program-state",
            "the selected execution lane has no program state provider",
        ));
    }
    match request.mode {
        Mode::NativeObjects => native_objects(loaded.module()),
        Mode::CompileTime => ct(&loaded, budget.min(remaining(&request, started)?)),
    }
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

fn native_objects(module: &MirModule) -> Result<(Outcome, Vec<u8>), Failure> {
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
    for unit in &units {
        emit::admit_unit(unit).map_err(native_failure)?;
    }
    let mut payload = Vec::new();
    let mut objects = Vec::with_capacity(units.len());
    for unit in &units {
        let object = emit::emit_unit_coff(unit).map_err(native_failure)?;
        objects.push(append_object(object, &mut payload)?);
    }
    let wrapper = append_object(
        emit::UnitObject {
            symbol: ace_native::config::START_SYMBOL.into(),
            imports: Vec::new(),
            bytes: emit::emit_wrapper_coff(&entry).map_err(native_failure)?,
        },
        &mut payload,
    )?;
    let text_runtime = module
        .capabilities
        .iter()
        .any(|cap| cap == TEXT_RUNTIME_CAPABILITY);
    Ok((
        Outcome::Objects {
            entry,
            units: objects,
            wrapper,
            text_runtime,
        },
        payload,
    ))
}

fn ct(loaded: &ace_av_core::LoadedModule, wall: Duration) -> Result<(Outcome, Vec<u8>), Failure> {
    let limits = ExecLimits {
        wall: wall.min(ExecLimits::default().wall),
        ..Default::default()
    };
    let (outcome, trace) = ace_av_core::execute(loaded, &limits, &CancelFlag::default());
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
                read_set: Vec::new(), // No filesystem capability is granted in this execution profile.
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
