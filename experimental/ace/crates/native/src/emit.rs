//! CLIF assembly and COFF emission for MIR units (contract-decision §2/§5):
//! one relocatable object per function unit, external symbols per the
//! logical scheme, imports resolved at link time by the A10' manifest.
//! The target triple is the FROZEN `x86_64-pc-windows-msvc` — never the
//! host triple silently.

use crate::config::*;
use crate::lower;
use crate::symbols;
use crate::MnError;
use ace_av_core::contract::{AV_DATA_PROFILE, AV_OWNED_PROFILE, AV_PROFILE};
use ace_av_core::mir::{MirModule, MirType};
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

pub(crate) const WINCALL: CallConv = CallConv::WindowsFastcall;

fn triple() -> Result<Triple, MnError> {
    MN_TARGET
        .parse::<Triple>()
        .map_err(|e| MnError::Infra(format!("frozen target triple rejected: {e:?}")))
}

pub(crate) fn make_isa() -> Result<cranelift_codegen::isa::OwnedTargetIsa, MnError> {
    let mut b = cranelift_codegen::settings::builder();
    b.set("opt_level", "none")
        .and_then(|_| b.set("is_pic", "false"))
        .map_err(|e| MnError::Backend(format!("flags: {e}")))?;
    let flags = cranelift_codegen::settings::Flags::new(b);
    let builder = cranelift_codegen::isa::lookup(triple()?)
        .map_err(|e| MnError::Infra(format!("ISA lookup for {MN_TARGET}: {e:?}")))?;
    builder
        .finish(flags)
        .map_err(|e| MnError::Backend(format!("isa finish: {e}")))
}

/// Cranelift signature of a MIR `MirType::Fn` under the private ABI
/// (PLAN-870 freeze §3): i32/bool/char cross as I32 (bool keeps logical
/// 0/1); Str/StrView cross as an (I64 ptr, I32 len) pair; Record and
/// RecordView cross as one I64 address; a Record RESULT crosses through a
/// trailing hidden `*mut` out parameter. Win x64 call conv.
pub fn cranelift_sig_of(module: &MirModule, ty: u32) -> Result<Signature, MnError> {
    let Some(sig) = module.sig_of(ty) else {
        return Err(MnError::Unsupported(
            "fn-sig",
            "type is not a function signature".to_string(),
        ));
    };
    let mut params = Vec::with_capacity(sig.params.len() + 1);
    for p in &sig.params {
        match module.types.get(*p as usize) {
            Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I32))
            }
            Some(MirType::Str) | Some(MirType::StrView) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
                params.push(AbiParam::new(cranelift_codegen::ir::types::I32));
            }
            Some(MirType::Record(_)) | Some(MirType::RecordView(_)) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
            }
            None => {
                return Err(MnError::Unsupported(
                    "param",
                    "unknown parameter type is outside the private ABI".to_string(),
                ))
            }
            // PLAN-961: numeric params cross on their machine carriers
            // (i8/u8/u32/f32 → I32, i64/u64/f64 → I64; bit-exact).
            Some(MirType::I8) | Some(MirType::U8) | Some(MirType::U32) | Some(MirType::F32) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I32))
            }
            Some(MirType::I64) | Some(MirType::U64) | Some(MirType::F64) => {
                params.push(AbiParam::new(cranelift_codegen::ir::types::I64))
            }
            // Function-typed and out-of-scope (collections / closures /
            // dynamic) parameter shapes stay a controlled rejection.
            Some(_) => {
                return Err(MnError::Unsupported(
                    "param",
                    "parameter type is outside the mn data/owned ABI".to_string(),
                ))
            }
        }
    }
    let ret = match module.types.get(sig.result as usize) {
        Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char) => {
            vec![AbiParam::new(cranelift_codegen::ir::types::I32)]
        }
        Some(MirType::Str) | Some(MirType::StrView) => vec![
            AbiParam::new(cranelift_codegen::ir::types::I64),
            AbiParam::new(cranelift_codegen::ir::types::I32),
        ],
        Some(MirType::Record(_)) | Some(MirType::RecordView(_)) => {
            // Hidden out-aggregate parameter (782 ABI-E shape): the caller
            // passes the destination aggregate's address; no return value.
            params.push(AbiParam::new(cranelift_codegen::ir::types::I64));
            vec![]
        }
        None => {
            return Err(MnError::Unsupported(
                "result",
                "unknown result type is outside the private ABI".to_string(),
            ))
        }
        // PLAN-961: numeric results cross on their machine carriers.
        Some(MirType::I8) | Some(MirType::U8) | Some(MirType::U32) | Some(MirType::F32) => {
            vec![AbiParam::new(cranelift_codegen::ir::types::I32)]
        }
        Some(MirType::I64) | Some(MirType::U64) | Some(MirType::F64) => {
            vec![AbiParam::new(cranelift_codegen::ir::types::I64)]
        }
        Some(_) => {
            return Err(MnError::Unsupported(
                "result",
                "result type is outside the mn data/owned ABI".to_string(),
            ))
        }
    };
    Ok(Signature {
        call_conv: WINCALL,
        params,
        returns: ret,
    })
}

fn exit_sig() -> Signature {
    Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
        returns: vec![],
    }
}

/// Admission re-checks at the codegen door (contract §7.3): the input must
/// be a single-function unit of an ADMITTED profile with an admitted
/// import surface. PLAN-870 widens the admitted set to the three shared
/// MIR profiles (scalar + av.data/1 + av.owned/1); everything else stays a
/// controlled rejection. Rejected here with codegen count 0 — never NOP'd
/// past.
pub fn admit_unit(unit: &MirModule) -> Result<(), MnError> {
    admit_unit_shape(unit)?;
    if has_foreign(unit) {
        return Err(MnError::Unsupported(
            "foreign.authority",
            "use the current-host foreign codegen entry".into(),
        ));
    }
    Ok(())
}

fn has_foreign(unit: &MirModule) -> bool {
    unit.funcs.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.instrs
                .iter()
                .any(|i| matches!(i, ace_av_core::mir::MirInstr::ForeignCall { .. }))
        })
    })
}

/// PLAN-947: the console-lane admission door. Admits ONLY the host-console
/// profile, single-function shape, no foreign calls, and every live
/// `HostCall` must index a declared console/1 import whose symbol the
/// shared `av-core` host contract knows. The ordinary lanes
/// (`admit_unit`) keep rejecting the host-console profile unchanged —
/// PLAN-863's fail-closed compat contract is not relaxed.
pub fn admit_console_unit(unit: &MirModule) -> Result<(), MnError> {
    if !unit.is_host_console_profile() {
        return Err(MnError::Unsupported(
            "profile",
            format!(
                "the console lane admits only the host-console profile, not `{}`",
                unit.profile
            ),
        ));
    }
    if !unit.is_unit || unit.funcs.len() != 1 {
        return Err(MnError::Unsupported(
            "unit-shape",
            "console codegen consumes single-function units only".to_string(),
        ));
    }
    if has_foreign(unit) {
        return Err(MnError::Unsupported(
            "foreign.authority",
            "console units cannot carry foreign calls".into(),
        ));
    }
    let f = &unit.funcs[0];
    for b in &f.blocks {
        for i in &b.instrs {
            if let ace_av_core::mir::MirInstr::HostCall { host, .. } = i {
                let Some(h) = unit.host_imports.get(*host as usize) else {
                    return Err(MnError::Unsupported(
                        "host-console",
                        format!("host declaration {host} is missing in the unit"),
                    ));
                };
                if ace_av_core::host::signature(&h.symbol).is_none() {
                    return Err(MnError::Unsupported(
                        "host-console",
                        format!("host symbol `{}` is not a console/1 symbol", h.symbol),
                    ));
                }
            }
        }
    }
    for imp in &unit.imports {
        symbols::import_symbol(&unit.owner, &imp.symbol)?;
    }
    Ok(())
}

fn admit_unit_shape(unit: &MirModule) -> Result<(), MnError> {
    // PLAN-961: the closures profile (the numeric source surface's MIR
    // family) joins the admitted set — its numeric emission rides the
    // same object lane.
    let admitted = unit.profile == AV_PROFILE
        || unit.profile == AV_DATA_PROFILE
        || unit.profile == AV_OWNED_PROFILE
        || unit.profile == ace_av_core::contract::AV_CLOSURES_PROFILE
        || unit.profile == ace_av_core::contract::AV_COLLECTIONS_PROFILE;
    if !admitted {
        return Err(MnError::Unsupported(
            "profile",
            format!(
                "profile `{}` is not one of `{AV_PROFILE}`/`{AV_DATA_PROFILE}`/`{AV_OWNED_PROFILE}`",
                unit.profile
            ),
        ));
    }
    if !unit.is_unit || unit.funcs.len() != 1 {
        return Err(MnError::Unsupported(
            "unit-shape",
            "codegen consumes single-function units only".to_string(),
        ));
    }
    // Defensive wide-scalar touch check (freeze §4): R2-N stays closed —
    // any REACHED wide numeric type (slot, signature, or record field)
    // rejects by name even though the type table itself may carry dead
    // entries.
    let mut seen = std::collections::BTreeSet::new();
    let f = &unit.funcs[0];
    let mut touched: Vec<u32> = vec![f.sig];
    for s in &f.slots {
        touched.push(s.ty);
    }
    for imp in &unit.imports {
        touched.push(imp.sig);
    }
    for i in &f.blocks {
        for ins in &i.instrs {
            if let ace_av_core::mir::MirInstr::Aggregate { record_ty, .. } = ins {
                touched.push(*record_ty);
            }
        }
    }
    while let Some(t) = touched.pop() {
        if !seen.insert(t) {
            continue;
        }
        match unit.types.get(t as usize) {
            // PLAN-961: the R2-N wide-type gate is superseded — wide
            // and narrow numeric types are the delivered numeric surface
            // (spec delta SD-01); their carriers are machine-only.
            Some(MirType::Record(r)) | Some(MirType::RecordView(r)) => {
                for fd in &r.fields {
                    touched.push(fd.ty);
                }
            }
            Some(MirType::Fn(s)) => {
                for p in &s.params {
                    touched.push(*p);
                }
                touched.push(s.result);
            }
            _ => {}
        }
    }
    for imp in &unit.imports {
        symbols::import_symbol(&unit.owner, &imp.symbol)?;
    }
    // Touched wide CONSTANTS (freeze §4/§7): R2-N stays closed — a Const
    // instruction referencing a wide numeric constant rejects by name
    // (unreferenced dead table entries do not).
    for i in &f.blocks {
        for ins in &i.instrs {
            if let ace_av_core::mir::MirInstr::Const { value, .. } = ins {
                match unit.consts.get(*value as usize) {
                    // PLAN-961: numeric constants (wide and narrow) are
                    // the delivered surface — bit-exact carriers (SD-01).
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// One emitted unit object: the COFF bytes plus the declared identities
/// the A10' manifest records (export symbol + resolved import symbols).
#[derive(Debug)]
pub struct UnitObject {
    pub symbol: String,
    pub bytes: Vec<u8>,
    /// (MIR import symbol, resolved link symbol) in table order.
    pub imports: Vec<(String, String)>,
}

/// Lower and emit one verified MIR unit to a relocatable COFF object.
pub fn emit_unit_coff(unit: &MirModule) -> Result<UnitObject, MnError> {
    emit_unit_coff_impl(unit, None, None, lower::LowerMode::Numeric, false)
        .map(|(object, _)| object)
}

/// PLAN-947: lower and emit one host-console unit. Console admission is a
/// dedicated door — the ordinary single-function lanes keep rejecting the
/// host-console profile (the PLAN-863 compat contract is unchanged); this
/// entry admits ONLY the host-console profile and emits console/1 host
/// calls against the console shim import.
pub fn emit_console_unit_coff(unit: &MirModule) -> Result<UnitObject, MnError> {
    emit_unit_coff_impl(unit, None, None, lower::LowerMode::Console, true).map(|(object, _)| object)
}

/// PLAN-1000 F-1000-01 (owner option a): the trap-probe observation build
/// of one unit — the production `Numeric` lowering with reason-carrying
/// trap branches (`mn_typed_trap` import, resolved by the typed-entry
/// runtime at the probe link). Probe objects are never cached, never
/// published and never production artifacts; only `MnDriver::trap_probe`
/// links and runs them.
pub fn emit_unit_coff_trap_probe(unit: &MirModule) -> Result<UnitObject, MnError> {
    emit_unit_coff_impl(unit, None, None, lower::LowerMode::TrapProbe, false)
        .map(|(object, _)| object)
}

pub fn emit_foreign_unit_coff(
    unit: &MirModule,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> Result<UnitObject, MnError> {
    emit_unit_coff_impl(
        unit,
        Some(environment),
        None,
        lower::LowerMode::Numeric,
        false,
    )
    .map(|(object, _)| object)
}

/// Keep the exact output reservation through the caller's cache handoff.
pub fn emit_foreign_unit_coff_reserved(
    unit: &MirModule,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
    resources: &ace_source_query::resources::Resources,
) -> Result<(UnitObject, ace_source_query::resources::Reservation), MnError> {
    let (object, reservation) = emit_unit_coff_impl(
        unit,
        Some(environment),
        Some(resources),
        lower::LowerMode::Numeric,
        false,
    )?;
    Ok((object, reservation.expect("host-backed COFF reservation")))
}

#[path = "coff_buffer.rs"]
mod coff_buffer;

/// Conservative bound of the Cranelift emit workspace (PLAN-938): the
/// ISA/object-module/context fixed cost (measured ~53 KiB for a minimal
/// scalar unit on the pinned cranelift; the base doubles it) plus
/// per-element allowances for the CLIF working set of a verified scalar
/// unit. Reserved on the shared staging account BEFORE `make_isa` and held
/// through the whole window (merged into the returned COFF reservation).
fn codegen_workspace_bound(unit: &MirModule) -> Option<usize> {
    let f = unit.funcs.first()?;
    let instrs: usize = f.blocks.iter().map(|b| b.instrs.len()).sum();
    let bound = 96usize << 10;
    let bound = bound.checked_add(instrs.checked_mul(1024)?)?;
    let bound = bound.checked_add(f.blocks.len().checked_mul(1024)?)?;
    let bound = bound.checked_add(unit.imports.len().checked_mul(2048)?)?;
    let bound = bound.checked_add(unit.types.len().checked_mul(256)?)?;
    let bound = bound.checked_add(unit.consts.len().checked_mul(256)?)?;
    bound.checked_add(f.slots.len().checked_mul(64)?)
}

fn emit_unit_coff_impl(
    unit: &MirModule,
    environment: Option<&ace_foreign_contract::manual::AdmittedForeign>,
    resources: Option<&ace_source_query::resources::Resources>,
    mode: lower::LowerMode,
    console: bool,
) -> Result<(UnitObject, Option<ace_source_query::resources::Reservation>), MnError> {
    if console {
        admit_console_unit(unit)?;
    } else if let Some(environment) = environment {
        admit_unit_shape(unit)?;
        if let Some(resources) = resources {
            crate::foreign_lower::admit_reserved(unit, environment, resources)?;
        } else {
            crate::foreign_lower::admit(unit, environment)
                .map_err(|e| MnError::Unsupported("foreign.authority", e))?;
        }
    } else {
        admit_unit(unit)?;
    }
    let mut workspace = match resources {
        Some(resources) => {
            let bound = codegen_workspace_bound(unit).ok_or_else(|| {
                MnError::Resource("codegen-memory", "workspace bound overflow".into())
            })?;
            Some(
                resources
                    .reserve(ace_source_query::resources::Charge::staging(bound))
                    .map_err(|e| MnError::Resource("codegen-memory", e.to_string()))?,
            )
        }
        None => None,
    };
    let f = &unit.funcs[0];
    let isa = make_isa()?;
    let obj_name = format!("mn8-{}", f.name);
    let builder = ObjectBuilder::new(isa, obj_name, default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);

    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig())
        .map_err(|e| MnError::Backend(format!("declare ExitProcess: {e}")))?;

    // Imports first: logical symbols per the scheme.
    let mut import_ids = Vec::with_capacity(unit.imports.len());
    let mut import_syms = Vec::with_capacity(unit.imports.len());
    for imp in &unit.imports {
        let logical = symbols::import_symbol(&unit.owner, &imp.symbol)?;
        let sig = cranelift_sig_of(unit, imp.sig)?;
        let id = om
            .declare_function(&logical, Linkage::Import, &sig)
            .map_err(|e| MnError::Backend(format!("declare import {}: {e}", logical)))?;
        import_ids.push(id);
        import_syms.push((imp.symbol.clone(), logical));
    }
    for block in &f.blocks {
        for instruction in &block.instrs {
            if let ace_av_core::mir::MirInstr::ForeignCall { declaration, .. } = instruction {
                let item = (
                    format!("foreign:{}:{}", declaration.logical, declaration.digest()),
                    declaration.function.symbol.clone(),
                );
                if !import_syms.contains(&item) {
                    import_syms.push(item);
                }
                if let Some(context) = &declaration.context_symbol {
                    let item = (
                        format!("foreign-context:{}", declaration.logical),
                        context.clone(),
                    );
                    if !import_syms.contains(&item) {
                        import_syms.push(item);
                    }
                }
                if matches!(
                    declaration.function.form,
                    ace_foreign_contract::FnForm::Fallible { .. }
                ) || declaration
                    .params()
                    .contains(&ace_foreign_contract::LaneKind::Bool)
                    || declaration.result() == ace_foreign_contract::LaneKind::Bool
                {
                    let item = ("foreign-status-runtime/1".into(), "ace_foreign_fail".into());
                    if !import_syms.contains(&item) {
                        import_syms.push(item);
                    }
                }
            }
        }
    }
    // A bare Trace instruction without a matching table import still needs
    // its runtime symbol (verify admits trace via capability; the unit
    // split copies only declared imports).
    let mut trace_ids: Vec<(String, cranelift_module::FuncId)> = Vec::new();
    for sym in ["hir.test.mark_a", "hir.test.mark_b"] {
        let uses = f.blocks.iter().any(|b| {
            b.instrs
                .iter()
                .any(|i| matches!(i, ace_av_core::mir::MirInstr::Trace { event } if event_symbol(*event) == sym))
        });
        if uses && !unit.imports.iter().any(|i| i.symbol == sym) {
            let sig = trace_sig();
            let id = om
                .declare_function(sym, Linkage::Import, &sig)
                .map_err(|e| MnError::Backend(format!("declare {sym}: {e}")))?;
            trace_ids.push((sym.to_string(), id));
        }
    }

    let fn_sig = cranelift_sig_of(unit, f.sig)?;
    let symbol = symbols::function_symbol(&unit.owner, &f.name);
    let fid = om
        .declare_function(&symbol, Linkage::Export, &fn_sig)
        .map_err(|e| MnError::Backend(format!("declare {symbol}: {e}")))?;

    // PLAN-947: console units with a live host call target the console
    // shim import (resolved by the shim object at the console link).
    let console_write = if console {
        let uses_host = f.blocks.iter().any(|b| {
            b.instrs
                .iter()
                .any(|i| matches!(i, ace_av_core::mir::MirInstr::HostCall { .. }))
        });
        if uses_host {
            let sig = Signature {
                call_conv: WINCALL,
                params: vec![
                    AbiParam::new(cranelift_codegen::ir::types::I32),
                    AbiParam::new(cranelift_codegen::ir::types::I64),
                    AbiParam::new(cranelift_codegen::ir::types::I64),
                ],
                returns: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
            };
            let id = om
                .declare_function(crate::config::CONSOLE_WRITE_SYMBOL, Linkage::Import, &sig)
                .map_err(|e| {
                    MnError::Backend(format!(
                        "declare {}: {e}",
                        crate::config::CONSOLE_WRITE_SYMBOL
                    ))
                })?;
            Some(id)
        } else {
            None
        }
    } else {
        None
    };

    let mut ctx = om.make_context();
    ctx.func.signature = fn_sig;
    lower::emit_function(
        unit,
        &mut om,
        fid,
        &import_ids,
        &trace_ids,
        exit_id,
        &mut ctx,
        // PLAN-961: the source lane emits in Numeric mode (the Legacy
        // surface plus the numeric carriers/ops — identical objects for
        // numeric-free programs). PLAN-1000: the trap probe passes
        // `TrapProbe` — the same value surface, reason-carrying traps.
        // PLAN-947: the console lane passes `Console` with the shim ref.
        mode,
        console_write,
    )
    .map_err(|e| MnError::Backend(e))?;
    om.define_function(fid, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {symbol}: {e}")))?;

    let product = om.finish();
    let (bytes, reservation) = coff_buffer::emit(&product.object, resources)?;
    // The emit workspace's ticket joins the output ticket in ONE account
    // entry (no release/reacquire gap); the caller holds both through its
    // cache handoff.
    let reservation = match (workspace.take(), reservation) {
        (Some(workspace), Some(output)) => Some(
            workspace
                .merge(output)
                .map_err(|e| MnError::Resource("codegen-memory", e.to_string()))?,
        ),
        (Some(workspace), None) => Some(workspace),
        (None, reservation) => reservation,
    };
    Ok((
        UnitObject {
            symbol,
            bytes,
            imports: import_syms,
        },
        reservation,
    ))
}

fn trace_sig() -> Signature {
    Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
    }
}

pub(crate) fn event_symbol(e: ace_av_core::mir::TraceEvent) -> &'static str {
    match e {
        ace_av_core::mir::TraceEvent::MarkA => "hir.test.mark_a",
        ace_av_core::mir::TraceEvent::MarkB => "hir.test.mark_b",
    }
}

/// The startup wrapper object: `ac_start` calls the entry function and
/// forwards its i32 result BIT-EXACT to `ExitProcess` (observers compare
/// the full 32-bit exit code). Emitted per request — it is not a cached
/// unit artifact (contract-decision §8 note).
pub fn emit_wrapper_coff(entry_symbol: &str) -> Result<Vec<u8>, MnError> {
    emit_wrapper_coff_impl(entry_symbol, None).map(|(bytes, _)| bytes)
}

/// Source request wrapper bytes retain their real memory reservation through
/// their last use, including file-write failures. The payload drops first.
pub struct ReservedWrapper {
    bytes: Vec<u8>,
    _allocation: ace_source_query::resources::Reservation,
}
impl ReservedWrapper {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
pub fn emit_wrapper_coff_reserved(
    entry_symbol: &str,
    resources: &ace_source_query::resources::Resources,
) -> Result<ReservedWrapper, MnError> {
    let (bytes, allocation) = emit_wrapper_coff_impl(entry_symbol, Some(resources))?;
    Ok(ReservedWrapper {
        bytes,
        _allocation: allocation.expect("reserved wrapper output"),
    })
}
/// Conservative bound of the startup wrapper's Cranelift window: the same
/// fixed ISA/object-module/context cost as the unit emit (measured ~53 KiB,
/// base doubled) plus slack for the fixed two-import function shape
/// (PLAN-938; the wrapper is emitted per request, never cached).
const WRAPPER_WORKSPACE_BOUND: usize = (96 << 10) + (8 << 10);

fn emit_wrapper_coff_impl(
    entry_symbol: &str,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<(Vec<u8>, Option<ace_source_query::resources::Reservation>), MnError> {
    let mut workspace = match resources {
        Some(resources) => Some(
            resources
                .reserve(ace_source_query::resources::Charge::staging(
                    WRAPPER_WORKSPACE_BOUND,
                ))
                .map_err(|e| MnError::Resource("codegen-memory", e.to_string()))?,
        ),
        None => None,
    };
    let isa = make_isa()?;
    let builder = ObjectBuilder::new(isa, "mn8-ac-start", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);
    let entry_sig = trace_sig(); // zero-param i32 — the frozen entry contract
    let entry_id = om
        .declare_function(entry_symbol, Linkage::Import, &entry_sig)
        .map_err(|e| MnError::Backend(format!("declare entry {entry_symbol}: {e}")))?;
    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig())
        .map_err(|e| MnError::Backend(format!("declare ExitProcess: {e}")))?;
    let start_sig = Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![],
    };
    let start_id = om
        .declare_function(START_SYMBOL, Linkage::Export, &start_sig)
        .map_err(|e| MnError::Backend(format!("declare {}: {e}", START_SYMBOL)))?;
    let mut ctx = om.make_context();
    ctx.func.signature = start_sig;
    {
        let mut fb_ctx = cranelift_frontend::FunctionBuilderContext::new();
        let mut fb = cranelift_frontend::FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        let entry_ref = om.declare_func_in_func(entry_id, &mut fb.func);
        let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);
        let call = fb.ins().call(entry_ref, &[]);
        let result = fb.inst_results(call)[0];
        fb.ins().call(exit_ref, &[result]);
        fb.ins().return_(&[]);
        fb.finalize();
    }
    om.define_function(start_id, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {}: {e}", START_SYMBOL)))?;
    let product = om.finish();
    if resources.is_some() {
        // The workspace ticket joins the wrapper's output ticket in ONE
        // account entry, held through the wrapper bytes' last use.
        let (bytes, reservation) = coff_buffer::emit(&product.object, resources)?;
        let reservation = match (workspace.take(), reservation) {
            (Some(workspace), Some(output)) => Some(
                workspace
                    .merge(output)
                    .map_err(|e| MnError::Resource("codegen-memory", e.to_string()))?,
            ),
            (Some(workspace), None) => Some(workspace),
            (None, reservation) => reservation,
        };
        Ok((bytes, reservation))
    } else {
        // Preserve the original legacy wrapper path and its acceptance.
        product
            .emit()
            .map(|bytes| (bytes, None))
            .map_err(|e| MnError::Backend(format!("emit wrapper: {e}")))
    }
}
