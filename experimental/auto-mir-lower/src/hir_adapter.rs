//! Checked HIR → Typed MIR lowering (PLAN-798 §5.1/§5.2).
//!
//! Consumes an `auto_ac_prototype::verify::CheckedModule` of the Scalar (or
//! Core subset) domain and produces an UNCHECKED `MirModule`; the result
//! must still pass `verify_mir` — the adapter grants nothing.
//!
//! Mapping rules (frozen by contract-decision.md):
//! - HIR locals → typed slots (params lead in signature order); one slot
//!   per `LocalId` (shadowing creates new HIR locals).
//! - HIR's structured block tree (If/Loop child blocks) flattens into
//!   explicit MIR basic blocks; scalar `for` is already HIR-expanded into
//!   Loop/If/Break by the front end, so it needs no special case here.
//! - Expressions evaluate into temp slots in SOURCE order; call arguments
//!   evaluate in `eval_args` order then map through `bindings` into
//!   parameter order — never re-ordered by the binding table.
//! - Lazy `Logical` lowers to branch-based short-circuit with a bool result
//!   slot defined in BOTH arms (verify's intersection init covers the join).
//! - `hir.test.mark_a`/`mark_b` intrinsics map to AV trace events; per the
//!   landed native contract (`test-support/marks.rs`) their i32 results are
//!   1 and 2. `true`/`false` literals arrive HIR-expanded as `0<1`/`0<0`,
//!   so HIR constants are i32-only here.
//!
//! Statement walking is iterative (explicit task stack — HIR block nesting
//! is unbounded); expression evaluation recurses with a depth ceiling well
//! above the 128-level source front door, rejecting deeper hand-made trees
//! with a controlled `mir.lower.depth` error.

use auto_ac_prototype::hir;
use auto_ac_prototype::verify::CheckedModule;
use auto_av_core::contract::SpanInfo;
use auto_av_core::mir::*;
use std::collections::BTreeMap;

/// Adapter rejection taxonomy (mirrors the core taxonomy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterError {
    Unsupported { code: String, message: String },
    Invalid { code: String, message: String },
}

impl AdapterError {
    fn uns(code: &str, message: String) -> Self {
        AdapterError::Unsupported {
            code: code.to_string(),
            message,
        }
    }
    fn inv(code: &str, message: String) -> Self {
        AdapterError::Invalid {
            code: code.to_string(),
            message,
        }
    }
    pub fn code(&self) -> String {
        match self {
            AdapterError::Unsupported { code, .. } | AdapterError::Invalid { code, .. } => {
                code.clone()
            }
        }
    }
}

/// Expression-evaluation recursion ceiling. The source front door is 128;
/// anything deeper reaching this adapter is a hand-made bundle.
const MAX_EXPR_DEPTH: u32 = 512;

/// PLAN-881: the MIR-level name of a type-bound method —
/// `m.<RecordName>.<MethodName>`, derived from the NOMINAL owner, never
/// from the def-table position (the HIR def id `d_f<fi>_<name>` is
/// fi-indexed and churns on unrelated declaration insertion/reorder).
/// Source identifiers cannot contain `.`, so the mapping is injective
/// against free functions' `d_<name>` and against every other method —
/// two same-named methods on different records, or a method sharing a
/// name with a free function, can never collide.
fn method_mir_name(
    module: &hir::Module,
    record: hir::TypeId,
    method_name: &str,
) -> Result<String, AdapterError> {
    let rec_name = module
        .types
        .get(record.0 as usize)
        .map(|t| t.id_text.strip_prefix("t_rec_").unwrap_or(&t.id_text).to_string())
        .ok_or_else(|| {
            AdapterError::inv(
                "lower.method-owner",
                format!("method `{method_name}` references a missing owner type"),
            )
        })?;
    Ok(format!("m.{rec_name}.{method_name}"))
}

/// PLAN-881: the MIR-level name of any function-or-method def — free
/// functions keep their def id (`d_<name>`); methods take the
/// owner-based [`method_mir_name`]. This is the unit/import identity the
/// whole split/link/cache chain keys on.
///
/// A method's BARE name is recovered from its fi-indexed def id
/// (`d_f<fi>_<name>` — the digits end at the first `_`, the rest is the
/// identifier, underscores included): the HIR `name` field holds the
/// emitter's internal symbol (`sm_<Rec>_<name>`), which cannot be split
/// unambiguously.
pub fn mir_name_of_def(module: &hir::Module, di: usize) -> Result<String, AdapterError> {
    match &module.defs[di].kind {
        hir::DefKind::Method { record, .. } => {
            let id = &module.defs[di].id_text;
            let rest = id.strip_prefix("d_f").ok_or_else(|| {
                AdapterError::inv(
                    "lower.method-owner",
                    format!("method def id `{id}` is not in the `d_f<fi>_<name>` shape"),
                )
            })?;
            let name = match rest.find('_') {
                Some(u) if u > 0 => &rest[u + 1..],
                _ => {
                    return Err(AdapterError::inv(
                        "lower.method-owner",
                        format!("method def id `{id}` has no `<fi>_<name>` separator"),
                    ))
                }
            };
            method_mir_name(module, *record, name)
        }
        _ => Ok(module.defs[di].id_text.clone()),
    }
}

/// Lower a checked scalar/core module to an unchecked MIR module with
/// `entry_name` selected as the entry (zero-param i32 per the frozen entry
/// contract — enforced by `verify_mir`, not here).
pub fn lower_checked(
    checked: &CheckedModule,
    entry_name: &str,
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    // PLAN-862: the entry-unit domain lowers through THIS scalar path — the
    // compute surface is identical, the additions are the Unit result type
    // and the entry-kind declaration.
    let entry_unit = match checked.domain() {
        hir::HirDomain::Scalar | hir::HirDomain::Core => false,
        hir::HirDomain::EntryUnit => true,
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the av.core-i32/1 / av.entry-unit/1 profiles",
                    other.as_str()
                ),
            ));
        }
    };
    if checked.bundle().modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {} (multi-module lowering is future work)",
                checked.bundle().modules.len()
            ),
        ));
    }
    let module = &checked.bundle().modules[0];
    let owner = OwnerScope {
        schema: if entry_unit {
            auto_av_core::contract::MIR_ENTRY_UNIT_SCHEMA.to_string()
        } else {
            auto_av_core::contract::MIR_SCHEMA.to_string()
        },
        module: module.id_text.clone(),
    };
    let mut out = if entry_unit {
        MirModule::new_entry_unit(owner)
    } else {
        MirModule::new(owner)
    };
    out.capabilities = checked.required_capabilities().to_vec();

    // Pass 1: scan defs — intern imports and fix function indices before any
    // body lowers (calls may reference later defs).
    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut fn_order: Vec<(u32, hir::TypeId, hir::BodyId, String)> = Vec::new();
    let mut next_fn_idx: u32 = 0;
    for (di, d) in module.defs.iter().enumerate() {
        match &d.kind {
            hir::DefKind::Function { ty, body, .. } => {
                def_map.insert(di as u32, CalleeRef::Func(next_fn_idx));
                next_fn_idx += 1;
                // MIR units are named by the def id (the entry contract's
                // textual id, e.g. `d_main`); the declared name is a
                // frontend-internal symbol.
                fn_order.push((di as u32, *ty, *body, d.id_text.clone()));
            }
            hir::DefKind::Method { .. } => {
                return Err(AdapterError::uns(
                    "lower.method",
                    "type-bound methods belong to the record domain, outside the av profile"
                        .to_string(),
                ));
            }
            hir::DefKind::Intrinsic { symbol, effect, ty } => {
                if symbol != "hir.test.mark_a" && symbol != "hir.test.mark_b" {
                    return Err(AdapterError::uns(
                        "lower.intrinsic",
                        format!("intrinsic `{}` is not in the av profile", symbol),
                    ));
                }
                let sig = fn_type_index(
                    *ty,
                    module,
                    &mut out,
                    false,
                    &mut std::collections::BTreeMap::new(),
                    &mut std::collections::BTreeSet::new(),
                )?;
                let idx = out.intern_import(MirImport {
                    symbol: symbol.clone(),
                    effect: match effect {
                        hir::Effect::Observable => Effect::Observable,
                        hir::Effect::Pure => Effect::Pure,
                    },
                    sig,
                    env: None,
                });
                def_map.insert(di as u32, CalleeRef::Import(idx));
            }
        }
    }

    // Pass 2: lower bodies.
    let mut entry_idx: Option<u32> = None;
    for (di, ty, body_id, name) in fn_order {
        let sig = fn_type_index(
            ty,
            module,
            &mut out,
            false,
            &mut BTreeMap::new(),
            &mut std::collections::BTreeSet::new(),
        )?;
        if name == entry_name {
            entry_idx = Some(out.funcs.len() as u32);
        }
        let mir_fn = lower_function(
            module,
            &mut out,
            di,
            body_id,
            sig,
            source_snapshot,
            &def_map,
            name.clone(),
            false,
            false,
            &BTreeMap::new(),
        )?;
        out.funcs.push(mir_fn);
    }
    let entry = entry_idx.ok_or_else(|| {
        AdapterError::inv(
            "lower.entry",
            format!("no function named `{}` in module", entry_name),
        )
    })?;
    out.entry = entry;
    // PLAN-862: the entry KIND is read off the entry function's verified
    // signature (the source frontend already rejected every ambiguous
    // form). The wire carries the kind and `verify_mir` re-checks it against
    // this same signature — a hand-edited kind is a controlled rejection.
    if entry_unit {
        let entry_sig = out
            .sig_of(out.funcs[entry as usize].sig)
            .cloned()
            .ok_or_else(|| {
                AdapterError::inv(
                    "lower.entry-sig",
                    "the entry does not reference a function type".to_string(),
                )
            })?;
        out.entry_kind = Some(match out.types.get(entry_sig.result as usize) {
            Some(auto_av_core::mir::MirType::Unit) => auto_av_core::mir::EntryKind::Unit,
            _ => auto_av_core::mir::EntryKind::Int,
        });
    }
    Ok(out)
}

/// Lower a CHECKED record-domain module to an unchecked data-profile
/// (`av.data/1`) MIR module (PLAN-820, contract-decision §7). The surface:
/// free functions (type-bound methods incl. `static fn` are explicitly
/// unsupported in this first cut), i32/bool arithmetic, nominal records
/// with named/positional construction (source-order evaluation, then the
/// binding map projects into declaration order for AGGREGATE), field-path
/// reads (rvalue copies) and writes (root-local structural rebuild), whole
/// record copies (let/assign/param/return), and free-function calls with
/// record params/results. The result must still pass `verify_mir` — the
/// adapter grants nothing; wide scalars never appear here (the record
/// source front door has no wide-scalar syntax).
pub fn lower_checked_data(
    checked: &CheckedModule,
    entry_name: &str,
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    match checked.domain() {
        hir::HirDomain::Record => {}
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the av.data/1 profile (expected record)",
                    other.as_str()
                ),
            ));
        }
    }
    if checked.bundle().modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {}",
                checked.bundle().modules.len()
            ),
        ));
    }
    let module = &checked.bundle().modules[0];
    let owner = OwnerScope {
        schema: auto_av_core::contract::MIR_DATA_SCHEMA.to_string(),
        module: module.id_text.clone(),
    };
    let mut out = MirModule::new_data(owner);
    out.capabilities = checked.required_capabilities().to_vec();

    // Pass 1: defs — intrinsics/imports and function indices first.
    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut fn_order: Vec<(u32, hir::TypeId, hir::BodyId, String)> = Vec::new();
    let mut next_fn_idx: u32 = 0;
    for (di, d) in module.defs.iter().enumerate() {
        match &d.kind {
            hir::DefKind::Function { ty, body, .. } => {
                def_map.insert(di as u32, CalleeRef::Func(next_fn_idx));
                next_fn_idx += 1;
                fn_order.push((di as u32, *ty, *body, d.id_text.clone()));
            }
            // PLAN-881: record-domain methods lower as ordinary MIR
            // functions — a view method's receiver is ALREADY the HIR
            // signature's params[0] (a read-only record view) and a
            // static method carries no receiver, so the call structure
            // needs no second runtime. The owner-based `m.<Rec>.<name>`
            // keeps unit/cache identity stable under def reordering.
            hir::DefKind::Method { ty, body, .. } => {
                def_map.insert(di as u32, CalleeRef::Func(next_fn_idx));
                next_fn_idx += 1;
                fn_order.push((di as u32, *ty, *body, mir_name_of_def(module, di)?));
            }
            hir::DefKind::Intrinsic { symbol, effect, ty } => {
                if symbol != "hir.test.mark_a" && symbol != "hir.test.mark_b" {
                    return Err(AdapterError::uns(
                        "lower.intrinsic",
                        format!("intrinsic `{}` is not in the av profile", symbol),
                    ));
                }
                let sig = fn_type_index(
                    *ty,
                    module,
                    &mut out,
                    true,
                    &mut BTreeMap::new(),
                    &mut std::collections::BTreeSet::new(),
                )?;
                let idx = out.intern_import(MirImport {
                    symbol: symbol.clone(),
                    effect: match effect {
                        hir::Effect::Observable => Effect::Observable,
                        hir::Effect::Pure => Effect::Pure,
                    },
                    sig,
                    env: None,
                });
                def_map.insert(di as u32, CalleeRef::Import(idx));
            }
        }
    }

    // Pass 2: lower bodies with the data surface.
    let mut entry_idx: Option<u32> = None;
    for (di, ty, body_id, name) in fn_order {
        let sig = fn_type_index(
            ty,
            module,
            &mut out,
            true,
            &mut BTreeMap::new(),
            &mut std::collections::BTreeSet::new(),
        )?;
        if name == entry_name {
            entry_idx = Some(out.funcs.len() as u32);
        }
        let mir_fn = lower_function(
            module,
            &mut out,
            di,
            body_id,
            sig,
            source_snapshot,
            &def_map,
            name.clone(),
            true,
            false,
            &BTreeMap::new(),
        )?;
        out.funcs.push(mir_fn);
    }
    let entry = entry_idx.ok_or_else(|| {
        AdapterError::inv(
            "lower.entry",
            format!("no function named `{}` in module", entry_name),
        )
    })?;
    out.entry = entry;
    Ok(out)
}

/// PLAN-821: lower a CHECKED text-record (or text) bundle to the
/// `av.owned/1` profile (contract-decision §7). The source path keeps its
/// 774/782 `verify.*` diagnostics at the front door — this adapter only
/// consumes the checked module; `verify_mir` re-admits everything (typed
/// fixtures share that single trust path).
/// Sentinel slot for unit-typed expression results (PLAN-864): the
/// collections surface's effect statements produce no value slot (Unit is
/// never a profile-4 operand). Consumers never read this sentinel — HIR
/// verify rejects unit values in every consuming position.
pub const NO_SLOT: SlotId = u32::MAX;

/// PLAN-864: the entry-unit surface's lowering for unit-typed bodies
/// returns a Unit-typed temp; the collections surface never materializes
/// one — `eval` statements discard. (Documented convention; no runtime
/// unit slots exist in profile 4.)
pub fn lower_checked_collections(
    checked: &CheckedModule,
    entry_name: &str,
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    match checked.domain() {
        hir::HirDomain::Collections => {}
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the av.collections/1 profile (expected collections)",
                    other.as_str()
                ),
            ));
        }
    }
    if checked.bundle().modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {}",
                checked.bundle().modules.len()
            ),
        ));
    }
    let module = &checked.bundle().modules[0];
    let owner = OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: module.id_text.clone(),
    };
    let mut out = MirModule::new_collections(owner);
    out.capabilities = checked.required_capabilities().to_vec();

    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut fn_order: Vec<(u32, hir::TypeId, hir::BodyId, String)> = Vec::new();
    let mut next_fn_idx: u32 = 0;
    for (di, d) in module.defs.iter().enumerate() {
        match &d.kind {
            hir::DefKind::Function { ty, body, .. } => {
                def_map.insert(di as u32, CalleeRef::Func(next_fn_idx));
                next_fn_idx += 1;
                fn_order.push((di as u32, *ty, *body, d.id_text.clone()));
            }
            hir::DefKind::Method { .. } => {
                return Err(AdapterError::uns(
                    "lower.method",
                    "type-bound methods are outside the av.collections/1 surface".to_string(),
                ));
            }
            hir::DefKind::Intrinsic { .. } => {
                return Err(AdapterError::uns(
                    "lower.intrinsic",
                    "intrinsics are outside the av.collections/1 surface".to_string(),
                ));
            }
        }
    }

    let mut entry_idx: Option<u32> = None;
    let entry_ids = [entry_name.to_string(), format!("d_{entry_name}")];
    for (di, ty, body_id, name) in fn_order {
        // Collections functions: zero-param i32 (the frozen entry contract);
        // ordinary signatures map through the collections type tables.
        let sig = collections_fn_sig_for_def(module, di, ty, &mut out)?;
        if entry_ids.contains(&name) {
            entry_idx = Some(out.funcs.len() as u32);
        }
        let mir_fn = lower_function(
            module,
            &mut out,
            di,
            body_id,
            sig,
            source_snapshot,
            &def_map,
            name.clone(),
            false,
            true,
            &BTreeMap::new(),
        )?;
        out.funcs.push(mir_fn);
    }
    let entry = entry_idx.ok_or_else(|| {
        AdapterError::inv(
            "lower.entry",
            format!("no function named `{}` in module", entry_name),
        )
    })?;
    out.entry = entry;
    Ok(out)
}

/// PLAN-864: lower ONE function of a checked collections bundle into a
/// self-contained collections single-function unit (the 828-style session
/// seam; callees ride `fn:<name>` imports exactly as the owned units do).
pub fn lower_function_collections_unit(
    bundle: &hir::Bundle,
    def_id_text: &str,
    capabilities: &[String],
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    match bundle.domain {
        hir::HirDomain::Collections => {}
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the av.collections/1 profile",
                    other.as_str()
                ),
            ));
        }
    }
    if bundle.modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {}",
                bundle.modules.len()
            ),
        ));
    }
    let module = &bundle.modules[0];
    let owner = OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: module.id_text.clone(),
    };
    let mut out = MirModule::collections_unit(owner);
    out.capabilities = capabilities.to_vec();

    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut target: Option<(u32, hir::TypeId, hir::BodyId)> = None;
    for (di, d) in module.defs.iter().enumerate() {
        if let hir::DefKind::Function { ty, body, .. } = &d.kind {
            if d.id_text == def_id_text {
                def_map.insert(di as u32, CalleeRef::Import(next_import_dummy()));
                target = Some((di as u32, *ty, *body));
            } else {
                let import_idx = out.imports.len() as u32;
                let sig_ty = collections_fn_sig_for_def(module, di as u32, *ty, &mut out)?;
                out.intern_import(MirImport {
                    symbol: format!("fn:{}", d.id_text),
                    effect: Effect::Pure,
                    sig: sig_ty,
                    env: None,
                });
                def_map.insert(di as u32, CalleeRef::Import(import_idx));
            }
        }
    }
    let Some((di, ty, body_id)) = target else {
        return Err(AdapterError::inv(
            "lower.entry",
            format!("no function named `{}` in module", def_id_text),
        ));
    };
    // The unit's import for the target itself resolves to func 0 after
    // linking — units lower with the target as Import(fn:<name>) and the
    // snapshot link binds it (F-828-R1-03).
    let sig = collections_fn_sig_for_def(module, di, ty, &mut out)?;
    let name = module.defs[di as usize].id_text.clone();
    let mir_fn = lower_function(
        module,
        &mut out,
        di,
        body_id,
        sig,
        source_snapshot,
        &def_map,
        name,
        false,
        true,
        &BTreeMap::new(),
    )?;
    out.funcs.push(mir_fn);
    out.entry = 0;
    Ok(out)
}

fn next_import_dummy() -> ImportIdx {
    // placeholder index replaced by the real intern position below
    u32::MAX
}

/// PLAN-864: function signature type under the collections type tables.
fn collections_fn_sig_for_def(
    m: &hir::Module,
    def_index: u32,
    ty: hir::TypeId,
    out: &mut MirModule,
) -> Result<TypeIdx, AdapterError> {
    let Some(td) = m.types.get(ty.0 as usize) else {
        return Err(AdapterError::inv(
            "lower.type-ref",
            format!("def {} signature type out of range", def_index),
        ));
    };
    let hir::TypeKind::Function(sig) = &td.kind else {
        return Err(AdapterError::inv(
            "lower.type-ref",
            format!("def {} signature is not a function type", def_index),
        ));
    };
    let mut memo = BTreeMap::new();
    let mut visiting = std::collections::BTreeSet::new();
    let mut params = Vec::with_capacity(sig.params.len());
    for p in &sig.params {
        params.push(collections_type_idx_of(
            p,
            m,
            out,
            &mut memo,
            &mut visiting,
        )?);
    }
    let result = collections_type_idx_of(&sig.result, m, out, &mut memo, &mut visiting)?;
    let sig_ty = out.intern_type(MirType::Fn(FnSig { params, result }));
    Ok(sig_ty)
}

pub fn lower_checked_owned(
    checked: &CheckedModule,
    entry_name: &str,
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    match checked.domain() {
        hir::HirDomain::TextRecord | hir::HirDomain::Text => {}
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the av.owned/1 profile (expected text-record)",
                    other.as_str()
                ),
            ));
        }
    }
    if checked.bundle().modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {}",
                checked.bundle().modules.len()
            ),
        ));
    }
    let module = &checked.bundle().modules[0];
    let owner = OwnerScope {
        schema: auto_av_core::contract::MIR_OWNED_SCHEMA.to_string(),
        module: module.id_text.clone(),
    };
    let mut out = MirModule::new_owned(owner);
    out.capabilities = checked.required_capabilities().to_vec();

    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut fn_order: Vec<(u32, hir::TypeId, hir::BodyId, String)> = Vec::new();
    let mut next_fn_idx: u32 = 0;
    for (di, d) in module.defs.iter().enumerate() {
        match &d.kind {
            hir::DefKind::Function { ty, body, .. } => {
                def_map.insert(di as u32, CalleeRef::Func(next_fn_idx));
                next_fn_idx += 1;
                fn_order.push((di as u32, *ty, *body, d.id_text.clone()));
            }
            // PLAN-881: text-record methods ride the same shared lowering
            // as av.data (owner-based `m.<Rec>.<name>` identity; the view
            // receiver is the HIR signature's params[0]).
            hir::DefKind::Method { ty, body, .. } => {
                def_map.insert(di as u32, CalleeRef::Func(next_fn_idx));
                next_fn_idx += 1;
                fn_order.push((di as u32, *ty, *body, mir_name_of_def(module, di)?));
            }
            hir::DefKind::Intrinsic { .. } => {
                return Err(AdapterError::uns(
                    "lower.intrinsic",
                    "intrinsics are outside the av.owned/1 first cut".to_string(),
                ));
            }
        }
    }

    let mut entry_idx: Option<u32> = None;
    for (di, ty, body_id, name) in fn_order {
        // R1-F08: mode-aware — borrowed record params type as RecordView.
        let sig = owned_fn_sig_for_def(module, di, ty, &mut out, &BTreeMap::new())?;
        if name == entry_name {
            entry_idx = Some(out.funcs.len() as u32);
        }
        let mir_fn = lower_function(
            module,
            &mut out,
            di,
            body_id,
            sig,
            source_snapshot,
            &def_map,
            name.clone(),
            true,
            true,
            &BTreeMap::new(),
        )?;
        out.funcs.push(mir_fn);
    }
    let entry = entry_idx.ok_or_else(|| {
        AdapterError::inv(
            "lower.entry",
            format!("no function named `{}` in module", entry_name),
        )
    })?;
    out.entry = entry;
    Ok(out)
}

/// Lower ONE function of a checked text-record bundle into a self-contained
/// OWNED single-function unit (PLAN-821 T-09 seam; the 828 snapshot link
/// resolves the `fn:<name>` imports exactly as the data units do).
pub fn lower_function_owned_unit(
    bundle: &hir::Bundle,
    def_id_text: &str,
    capabilities: &[String],
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    lower_function_owned_unit_with_modes(
        bundle,
        def_id_text,
        capabilities,
        source_snapshot,
        &BTreeMap::new(),
    )
}

/// R3-F07: [`lower_function_owned_unit`] with RETAINED param-mode facts
/// for BODY-LESS scaffold defs. A D828 single-function document binds the
/// unchanged functions as body-less scaffolds — their MUT-record param
/// modes (the t_MutRet wrapper ABI) are not expressible in the document's
/// type section, so the session supplies the retained interface facts
/// keyed by def id. A body present in the document always wins.
pub fn lower_function_owned_unit_with_modes(
    bundle: &hir::Bundle,
    def_id_text: &str,
    capabilities: &[String],
    source_snapshot: u64,
    mode_overrides: &BTreeMap<String, Vec<hir::ParamMode>>,
) -> Result<MirModule, AdapterError> {
    match bundle.domain {
        hir::HirDomain::TextRecord | hir::HirDomain::Text => {}
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the av.owned/1 profile",
                    other.as_str()
                ),
            ));
        }
    }
    if bundle.modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {}",
                bundle.modules.len()
            ),
        ));
    }
    let module = &bundle.modules[0];
    let owner = OwnerScope {
        schema: auto_av_core::contract::MIR_OWNED_SCHEMA.to_string(),
        module: module.id_text.clone(),
    };
    let mut out = MirModule::owned_unit(owner);
    out.capabilities = capabilities.to_vec();

    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut target: Option<(u32, hir::TypeId, hir::BodyId)> = None;
    for (di, d) in module.defs.iter().enumerate() {
        match &d.kind {
            hir::DefKind::Function { ty, body, .. } => {
                // R1-F08: mode-aware — the import's sig must equal the
                // callee's own emission (borrowed record params =
                // RecordView), or the snapshot link would part them.
                let sig = owned_fn_sig_for_def(module, di as u32, *ty, &mut out, mode_overrides)?;
                let imp = out.intern_import(MirImport {
                    symbol: format!("fn:{}", d.id_text),
                    effect: Effect::Pure,
                    sig,
                    env: None,
                });
                def_map.insert(di as u32, CalleeRef::Import(imp));
                if d.id_text == def_id_text {
                    target = Some((di as u32, *ty, *body));
                }
            }
            // PLAN-881: methods import under the owner-based `fn:m.<Rec>.<n>`
            // symbol (stable identity); the target is still selected by its
            // HIR def id — a transient handle the session re-resolves.
            hir::DefKind::Method { ty, body, .. } => {
                let sig = owned_fn_sig_for_def(module, di as u32, *ty, &mut out, mode_overrides)?;
                let imp = out.intern_import(MirImport {
                    symbol: format!("fn:{}", mir_name_of_def(module, di)?),
                    effect: Effect::Pure,
                    sig,
                    env: None,
                });
                def_map.insert(di as u32, CalleeRef::Import(imp));
                if d.id_text == def_id_text {
                    target = Some((di as u32, *ty, *body));
                }
            }
            hir::DefKind::Intrinsic { .. } => {
                return Err(AdapterError::uns(
                    "lower.intrinsic",
                    "intrinsics are outside the av.owned/1 first cut".to_string(),
                ));
            }
        }
    }
    let (di, ty, body_id) = target.ok_or_else(|| {
        AdapterError::inv(
            "lower.target",
            format!("no function def `{def_id_text}` in module"),
        )
    })?;
    let sig = owned_fn_sig_for_def(module, di, ty, &mut out, mode_overrides)?;
    let mir_fn = lower_function(
        module,
        &mut out,
        di,
        body_id,
        sig,
        source_snapshot,
        &def_map,
        mir_name_of_def(module, di as usize)?,
        true,
        true,
        mode_overrides,
    )?;
    out.funcs.push(mir_fn);
    out.entry = 0;
    Ok(out)
}

/// Re-verify the entry gate for an owned module: entry params scalar/char,
/// result may be owned (the verifier owns the full contract; this is the
/// driver's diagnostic seam).
pub fn owned_entry_ok(mir: &MirModule) -> bool {
    let Some(f) = mir.funcs.get(mir.entry as usize) else {
        return false;
    };
    let Some(sig) = mir.sig_of(f.sig) else {
        return false;
    };
    let ok_ty = |t: u32| -> bool {
        matches!(
            mir.types.get(t as usize),
            Some(MirType::I32)
                | Some(MirType::Bool)
                | Some(MirType::I64)
                | Some(MirType::U64)
                | Some(MirType::F32)
                | Some(MirType::F64)
                | Some(MirType::Char)
                | Some(MirType::Str)
                | Some(MirType::Record(_))
        )
    };
    sig.params.iter().all(|p| {
        matches!(
            mir.types.get(*p as usize),
            Some(MirType::I32)
                | Some(MirType::Bool)
                | Some(MirType::I64)
                | Some(MirType::U64)
                | Some(MirType::F32)
                | Some(MirType::F64)
                | Some(MirType::Char)
        )
    }) && ok_ty(sig.result)
}

/// Lower ONE function of a bound bundle directly into a self-contained
/// single-function MIR unit (PLAN-828 T-01). Cross-function callees become
/// `fn:<name>` unit imports with the callee's signature (the same shape
/// `split_units` produces from a whole-module lower); intrinsics map to
/// imports exactly as in [`lower_checked`]. The bundle's OTHER bodies are
/// never read — the caller passes a per-function bundle where only
/// `def_id_text`'s body is real — and the result must still pass
/// `verify_mir`; this adapter grants nothing.
pub fn lower_function_unit(
    bundle: &hir::Bundle,
    def_id_text: &str,
    capabilities: &[String],
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    lower_function_unit_impl(bundle, def_id_text, capabilities, source_snapshot, false)
}

/// PLAN-820 T-09: the record-domain variant — one function of a checked
/// record bundle into a self-contained DATA-profile unit (`av.data/1`,
/// AVAD codec). Cross-function callees ride the same `fn:<name>` import
/// convention; the 828 snapshot link resolves them with signature checks.
pub fn lower_function_data_unit(
    bundle: &hir::Bundle,
    def_id_text: &str,
    capabilities: &[String],
    source_snapshot: u64,
) -> Result<MirModule, AdapterError> {
    lower_function_unit_impl(bundle, def_id_text, capabilities, source_snapshot, true)
}

fn lower_function_unit_impl(
    bundle: &hir::Bundle,
    def_id_text: &str,
    capabilities: &[String],
    source_snapshot: u64,
    data: bool,
) -> Result<MirModule, AdapterError> {
    match bundle.domain {
        hir::HirDomain::Scalar | hir::HirDomain::Core if !data => {}
        hir::HirDomain::Record if data => {}
        other => {
            return Err(AdapterError::uns(
                "lower.domain",
                format!(
                    "HIR domain `{}` is outside the {} profile",
                    other.as_str(),
                    if data { "av.data/1" } else { "av.core-i32/1" }
                ),
            ));
        }
    }
    if bundle.modules.len() != 1 {
        return Err(AdapterError::uns(
            "lower.modules",
            format!(
                "expected exactly one module, found {}",
                bundle.modules.len()
            ),
        ));
    }
    let module = &bundle.modules[0];
    let owner = if data {
        OwnerScope {
            schema: auto_av_core::contract::MIR_DATA_SCHEMA.to_string(),
            module: module.id_text.clone(),
        }
    } else {
        OwnerScope {
            schema: auto_av_core::contract::MIR_SCHEMA.to_string(),
            module: module.id_text.clone(),
        }
    };
    let mut out = if data {
        MirModule::data_unit(owner)
    } else {
        MirModule::unit(owner)
    };
    out.capabilities = capabilities.to_vec();

    // Def map: the target lowers into the unit's own func 0; every other
    // function def is an unresolved `fn:<name>` import with its signature,
    // and intrinsics import as in the whole-module path.
    let mut def_map: BTreeMap<u32, CalleeRef> = BTreeMap::new();
    let mut target: Option<(u32, hir::TypeId, hir::BodyId)> = None;
    for (di, d) in module.defs.iter().enumerate() {
        match &d.kind {
            hir::DefKind::Function { ty, body, .. } => {
                if d.id_text == def_id_text {
                    // Self-recursion also rides the unit's import
                    // convention (F-828-R1-03): a unit is verifiable only
                    // when EVERY callee — itself included — is an import;
                    // the snapshot link resolves `fn:<self>` back to this
                    // unit with a signature check.
                    let self_sig = fn_type_index(
                        *ty,
                        module,
                        &mut out,
                        data,
                        &mut std::collections::BTreeMap::new(),
                        &mut std::collections::BTreeSet::new(),
                    )?;
                    let imp = out.intern_import(MirImport {
                        symbol: format!("fn:{}", d.id_text),
                        effect: Effect::Pure,
                        sig: self_sig,
                        env: None,
                    });
                    def_map.insert(di as u32, CalleeRef::Import(imp));
                    target = Some((di as u32, *ty, *body));
                } else {
                    let sig = fn_type_index(
                        *ty,
                        module,
                        &mut out,
                        data,
                        &mut std::collections::BTreeMap::new(),
                        &mut std::collections::BTreeSet::new(),
                    )?;
                    let idx = out.intern_import(MirImport {
                        symbol: format!("fn:{}", d.id_text),
                        effect: Effect::Pure,
                        sig,
                        env: None,
                    });
                    def_map.insert(di as u32, CalleeRef::Import(idx));
                }
            }
            // PLAN-881: methods are unit citizens with the owner-based
            // `fn:m.<Rec>.<name>` import identity; the target is still
            // selected by HIR def id (a transient handle).
            hir::DefKind::Method { ty, body, .. } => {
                let symbol = format!("fn:{}", mir_name_of_def(module, di)?);
                let sig = fn_type_index(
                    *ty,
                    module,
                    &mut out,
                    data,
                    &mut std::collections::BTreeMap::new(),
                    &mut std::collections::BTreeSet::new(),
                )?;
                let idx = out.intern_import(MirImport {
                    symbol,
                    effect: Effect::Pure,
                    sig,
                    env: None,
                });
                def_map.insert(di as u32, CalleeRef::Import(idx));
                if d.id_text == def_id_text {
                    target = Some((di as u32, *ty, *body));
                }
            }
            hir::DefKind::Intrinsic { symbol, effect, ty } => {
                if symbol != "hir.test.mark_a" && symbol != "hir.test.mark_b" {
                    return Err(AdapterError::uns(
                        "lower.intrinsic",
                        format!("intrinsic `{symbol}` is not in the av profile"),
                    ));
                }
                let sig = fn_type_index(
                    *ty,
                    module,
                    &mut out,
                    data,
                    &mut std::collections::BTreeMap::new(),
                    &mut std::collections::BTreeSet::new(),
                )?;
                let idx = out.intern_import(MirImport {
                    symbol: symbol.clone(),
                    effect: match effect {
                        hir::Effect::Observable => Effect::Observable,
                        hir::Effect::Pure => Effect::Pure,
                    },
                    sig,
                    env: None,
                });
                def_map.insert(di as u32, CalleeRef::Import(idx));
            }
        }
    }
    let (di, ty, body_id) = target.ok_or_else(|| {
        AdapterError::inv(
            "lower.target",
            format!("no function def `{def_id_text}` in module"),
        )
    })?;
    let sig = fn_type_index(
        ty,
        module,
        &mut out,
        data,
        &mut BTreeMap::new(),
        &mut std::collections::BTreeSet::new(),
    )?;
    let mir_fn = lower_function(
        module,
        &mut out,
        di,
        body_id,
        sig,
        source_snapshot,
        &def_map,
        mir_name_of_def(module, di as usize)?,
        data,
        false,
        &BTreeMap::new(),
    )?;
    out.funcs.push(mir_fn);
    out.entry = 0;
    Ok(out)
}

fn scalar_shape_of_use(u: &hir::TypeUse, m: &hir::Module) -> Result<ScalarShape, AdapterError> {
    let s = match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Some(ScalarShape::I32),
            hir::BuiltinType::Bool => Some(ScalarShape::Bool),
            _ => None,
        },
        hir::TypeUse::Ref(tid) => match m.types.get(tid.0 as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::I32) => Some(ScalarShape::I32),
            Some(hir::TypeKind::Bool) => Some(ScalarShape::Bool),
            _ => None,
        },
    };
    s.ok_or_else(|| {
        AdapterError::uns(
            "lower.type-use",
            format!("type use {:?} is not an i32/bool scalar", u),
        )
    })
}

fn scalar_type_idx_of(
    u: &hir::TypeUse,
    m: &hir::Module,
    out: &mut MirModule,
) -> Result<TypeIdx, AdapterError> {
    // PLAN-862: Unit is the entry-unit profile's zero-width RESULT type. It
    // is NOT a scalar shape (it has no runtime width), so it maps before the
    // scalar-shape lookup; the verifier owns its placement.
    if matches!(u, hir::TypeUse::Builtin(hir::BuiltinType::Unit)) {
        return Ok(out.intern_type(MirType::Unit));
    }
    match scalar_shape_of_use(u, m)? {
        ScalarShape::I32 => Ok(out.intern_type(MirType::I32)),
        ScalarShape::Bool => Ok(out.intern_type(MirType::Bool)),
        // Unreachable via HIR: the source front doors expose no
        // wide-scalar syntax (defense for hand-made bundles).
        _ => Err(AdapterError::uns(
            "lower.type-use",
            "wide scalars exist only in typed fixtures, not source".to_string(),
        )),
    }
}

/// Map an HIR type use to a data-profile MIR type index: the six-scalar
/// surface never appears in record-domain HIR beyond i32/bool (the source
/// front door has no wide-scalar syntax), and nominal records map with
/// their full field lists (owner = the HIR module id). Record interning
/// recurses through the HIR type table with a memo + visiting set —
/// `verify_record` guarantees acyclicity; the visiting set is defensive.
fn data_type_idx_of(
    u: &hir::TypeUse,
    m: &hir::Module,
    out: &mut MirModule,
    memo: &mut std::collections::BTreeMap<hir::TypeId, TypeIdx>,
    visiting: &mut std::collections::BTreeSet<hir::TypeId>,
) -> Result<TypeIdx, AdapterError> {
    match u {
        hir::TypeUse::Builtin(hir::BuiltinType::I32) => Ok(out.intern_type(MirType::I32)),
        hir::TypeUse::Builtin(hir::BuiltinType::Bool) => Ok(out.intern_type(MirType::Bool)),
        hir::TypeUse::Builtin(other) => Err(AdapterError::uns(
            "lower.type-use",
            format!("builtin type {:?} is outside the av.data/1 profile", other),
        )),
        hir::TypeUse::Ref(tid) => {
            if let Some(idx) = memo.get(tid) {
                return Ok(*idx);
            }
            if !visiting.insert(*tid) {
                return Err(AdapterError::inv(
                    "mir.type-cycle",
                    format!("record type id {} participates in a type cycle", tid.0),
                ));
            }
            let td = m.types.get(tid.0 as usize).ok_or_else(|| {
                AdapterError::inv("lower.type-ref", format!("type id {} out of range", tid.0))
            })?;
            let mapped = match &td.kind {
                hir::TypeKind::I32 => Ok(out.intern_type(MirType::I32)),
                hir::TypeKind::Bool => Ok(out.intern_type(MirType::Bool)),
                // Named text kinds (PLAN-770 type-table entries): `str`,
                // `Str`, `char` arrive as Ref uses in the text-record domain.
                hir::TypeKind::Str => Ok(out.intern_type(MirType::StrView)),
                hir::TypeKind::StrOwned => Ok(out.intern_type(MirType::Str)),
                hir::TypeKind::Char => Ok(out.intern_type(MirType::Char)),
                hir::TypeKind::Record(shape) => {
                    let mut fields = Vec::with_capacity(shape.fields.len());
                    let mut err = None;
                    for f in &shape.fields {
                        match data_type_idx_of(&f.ty, m, out, memo, visiting) {
                            Ok(t) => fields.push(MirFieldDef {
                                name: f.name.clone(),
                                ty: t,
                            }),
                            Err(e) => {
                                err = Some(e);
                                break;
                            }
                        }
                    }
                    err.map_or_else(
                        || {
                            Ok(out.intern_type(MirType::Record(RecordTypeRef {
                                owner_module: m.id_text.clone(),
                                name: td.id_text.clone(),
                                fields,
                            })))
                        },
                        Err,
                    )
                }
                other => Err(AdapterError::uns(
                    "lower.type-use",
                    format!("type kind {:?} is outside the av.data/1 profile", other),
                )),
            };
            visiting.remove(tid);
            let idx = mapped?;
            memo.insert(*tid, idx);
            Ok(idx)
        }
    }
}

fn fn_type_index(
    ty: hir::TypeId,
    m: &hir::Module,
    out: &mut MirModule,
    data: bool,
    memo: &mut std::collections::BTreeMap<hir::TypeId, TypeIdx>,
    visiting: &mut std::collections::BTreeSet<hir::TypeId>,
) -> Result<TypeIdx, AdapterError> {
    fn_type_index_ext(ty, m, out, data, false, memo, visiting)
}

/// R1-F08: the OWNED signature of a text-record function, carrying each
/// param's PARAM MODE: a View-mode RECORD param types as `RecordView`
/// (782 ABI-D `*const` — read-only borrow, no caller kill, externally
/// rooted field views); MoveOwn keeps the owned record; MutBorrow on a
/// record stays a CONTROLLED rejection (ABI-E `*mut` write-back is
/// beyond the frozen MIR expressiveness — pending an explicit contract
/// revision, not silently wrong semantics).
fn owned_fn_sig_for_def(
    module: &hir::Module,
    def_index: u32,
    ty: hir::TypeId,
    out: &mut MirModule,
    mode_overrides: &BTreeMap<String, Vec<hir::ParamMode>>,
) -> Result<TypeIdx, AdapterError> {
    let td = module.types.get(ty.0 as usize).ok_or_else(|| {
        AdapterError::inv("lower.type-ref", format!("type id {} out of range", ty.0))
    })?;
    let hir_sig = match &td.kind {
        hir::TypeKind::Function(sig) => sig,
        _ => {
            return Err(AdapterError::inv(
                "lower.fn-type",
                format!("type `{}` is not a function type", td.id_text),
            ))
        }
    };
    // The param locals lead the callee body's local table in signature
    // order (HIR verify invariant, same one lower_function relies on).
    // R3-F07: a BODY-LESS scaffold def (D828 single-function document)
    // carries no body table — the modes then come from the RETAINED
    // interface facts (the document's type section cannot express them).
    let def_id_text = module
        .defs
        .get(def_index as usize)
        .map(|d| d.id_text.clone())
        .unwrap_or_default();
    let param_modes: Vec<hir::ParamMode> = module
        .defs
        .get(def_index as usize)
        .and_then(|d| match &d.kind {
            // PLAN-881: methods keep their explicit param modes too — a
            // view receiver is params[0] (mode View), but a mut/move
            // EXPLICIT param must not decay to the default borrow.
            hir::DefKind::Function { body, .. } | hir::DefKind::Method { body, .. } => {
                if body.0 == u32::MAX {
                    None
                } else {
                    module.bodies.get(body.0 as usize).map(|b| {
                        b.locals
                            .iter()
                            .filter(|l| l.role == hir::LocalRole::Param)
                            .map(|l| l.param_mode)
                            .collect()
                    })
                }
            }
            _ => None,
        })
        .or_else(|| mode_overrides.get(&def_id_text).cloned())
        .unwrap_or_default();
    let mut memo = std::collections::BTreeMap::new();
    let mut visiting = std::collections::BTreeSet::new();
    let mut params = Vec::with_capacity(hir_sig.params.len());
    for (pi, p) in hir_sig.params.iter().enumerate() {
        let base = owned_type_idx_of(p, module, out, &mut memo, &mut visiting)?;
        let mode = param_modes.get(pi).copied().unwrap_or(hir::ParamMode::View);
        let mapped = match (mode, out.types.get(base as usize)) {
            (hir::ParamMode::View, Some(MirType::Record(r))) => {
                // A plain record param is a READ-ONLY borrow (782 default).
                out.intern_type(MirType::RecordView(r.clone()))
            }
            // R2-F08': a MUT record param is an exclusive write-back
            // borrow (782 ABI-E) — encoded as MOVE-IN + wrapper-result
            // write-back: the caller moves the whole local in, the callee
            // returns (source result, final mut params) in a t_MutRet
            // wrapper, the caller extracts and writes back.
            (hir::ParamMode::MutBorrow, Some(MirType::Record(_))) => base,
            _ => base,
        };
        params.push(mapped);
    }
    let result = owned_type_idx_of(&hir_sig.result, module, out, &mut memo, &mut visiting)?;
    // Any mut RECORD param wraps the result (both sides derive from this
    // builder, so caller/callee signatures agree).
    let has_mut_record = param_modes.iter().zip(hir_sig.params.iter()).any(|(m, p)| {
        *m == hir::ParamMode::MutBorrow
            && matches!(
                owned_type_idx_of(p, module, out, &mut memo, &mut visiting)
                    .ok()
                    .and_then(|i| out.types.get(i as usize).cloned()),
                Some(MirType::Record(_))
            )
    });
    if !has_mut_record {
        return Ok(out.intern_type(MirType::Fn(FnSig { params, result })));
    }
    let mut fields = vec![MirFieldDef {
        name: "f0".into(),
        ty: result,
    }];
    for (m, p) in param_modes.iter().zip(hir_sig.params.iter()) {
        if *m != hir::ParamMode::MutBorrow {
            continue;
        }
        let ty = owned_type_idx_of(p, module, out, &mut memo, &mut visiting)?;
        if matches!(out.types.get(ty as usize), Some(MirType::Record(_))) {
            fields.push(MirFieldDef {
                name: format!("wb{}", fields.len()),
                ty,
            });
        }
    }
    let wrapper = out.intern_type(MirType::Record(RecordTypeRef {
        owner_module: module.id_text.clone(),
        // PLAN-881: methods name the wrapper after their owner-based MIR
        // identity (not the fi-indexed def id) so unrelated declaration
        // shifts cannot churn the unit's type-table fingerprint.
        name: format!(
            "t_MutRet_{}",
            mir_name_of_def(module, def_index as usize)
                .map(|n| n.strip_prefix("d_").unwrap_or(&n).to_string())
                .unwrap_or_else(|_| "fn".into())
        ),
        fields,
    }));
    Ok(out.intern_type(MirType::Fn(FnSig {
        params,
        result: wrapper,
    })))
}

fn fn_type_index_ext(
    ty: hir::TypeId,
    m: &hir::Module,
    out: &mut MirModule,
    data: bool,
    owned: bool,
    memo: &mut std::collections::BTreeMap<hir::TypeId, TypeIdx>,
    visiting: &mut std::collections::BTreeSet<hir::TypeId>,
) -> Result<TypeIdx, AdapterError> {
    let td = m.types.get(ty.0 as usize).ok_or_else(|| {
        AdapterError::inv("lower.type-ref", format!("type id {} out of range", ty.0))
    })?;
    match &td.kind {
        hir::TypeKind::Function(sig) => {
            let mut params = Vec::with_capacity(sig.params.len());
            for p in &sig.params {
                params.push(if owned {
                    owned_type_idx_of(p, m, out, memo, visiting)?
                } else {
                    type_idx_of_use(p, m, out, data, memo, visiting)?
                });
            }
            let result = if owned {
                owned_type_idx_of(&sig.result, m, out, memo, visiting)?
            } else {
                type_idx_of_use(&sig.result, m, out, data, memo, visiting)?
            };
            Ok(out.intern_type(MirType::Fn(FnSig { params, result })))
        }
        _ => Err(AdapterError::inv(
            "lower.fn-type",
            format!("type `{}` is not a function type", td.id_text),
        )),
    }
}

/// Unified HIR→MIR type mapping: the scalar path (core-i32 lowering) keeps
/// the historic i32/bool-only surface; the data path additionally maps
/// nominal records with full field lists.
fn type_idx_of_use(
    u: &hir::TypeUse,
    m: &hir::Module,
    out: &mut MirModule,
    data: bool,
    memo: &mut std::collections::BTreeMap<hir::TypeId, TypeIdx>,
    visiting: &mut std::collections::BTreeSet<hir::TypeId>,
) -> Result<TypeIdx, AdapterError> {
    if data {
        data_type_idx_of(u, m, out, memo, visiting)
    } else {
        scalar_type_idx_of(u, m, out)
    }
}

/// PLAN-821 owned-profile type mapping: the 782 field surface (i32/bool/
/// char/Str/nested records; `str` views appear only as value/param/return
/// kinds, never fields) maps onto the owned MIR kinds. Owned lowering
/// never sees wide scalars from source (the text-record front door has no
/// wide-scalar syntax).
/// PLAN-864: whether `eid` is a `ListGet` expression; returns its
/// (list, index) operand ids for the fusion checks.
fn list_get_root(body: &hir::Body, eid: hir::ExprId) -> Option<hir::ExprKind> {
    match &body.exprs.get(eid.0 as usize)?.kind {
        hir::ExprKind::ListGet { list, index } => Some(hir::ExprKind::ListGet {
            list: *list,
            index: *index,
        }),
        _ => None,
    }
}

/// PLAN-864: transitive Str-reachability over the HIR type table (the
/// source-level twin of `ownership::record_reaches_str`).
fn hir_record_reaches_str(m: &hir::Module, root: hir::TypeId) -> bool {
    let mut stack = vec![root.0];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(i) = stack.pop() {
        if !seen.insert(i) {
            continue;
        }
        match m.types.get(i as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::StrOwned) | Some(hir::TypeKind::Str) => return true,
            Some(hir::TypeKind::Record(shape)) => {
                for f in &shape.fields {
                    match f.ty {
                        hir::TypeUse::Ref(t2) => stack.push(t2.0),
                        hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => return true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// PLAN-864: the collections-profile type mapping — the owned surface
/// plus List/Variant type-table entries (HIR TypeKind::List/Variant map to
/// `MirType::List(ListTypeRef)` / `MirType::Variant(VariantTypeRef)` with
/// nominal owner+name identity, exactly the 822 fixture shapes).
fn collections_type_idx_of(
    u: &hir::TypeUse,
    m: &hir::Module,
    out: &mut MirModule,
    memo: &mut std::collections::BTreeMap<hir::TypeId, TypeIdx>,
    visiting: &mut std::collections::BTreeSet<hir::TypeId>,
) -> Result<TypeIdx, AdapterError> {
    match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Ok(out.intern_type(MirType::I32)),
            hir::BuiltinType::Bool => Ok(out.intern_type(MirType::Bool)),
            hir::BuiltinType::StrOwned => Ok(out.intern_type(MirType::Str)),
            // PLAN-864: unit-typed expressions are intercepted before type
            // mapping (the effect arms return NO_SLOT); reaching here means
            // a unit value would materialize — an internal violation.
            hir::BuiltinType::Unit => Err(AdapterError::inv(
                "lower.type-use",
                "a unit type would materialize in the collections profile (effect statements discard)".to_string(),
            )),
            other => Err(AdapterError::uns(
                "lower.type-use",
                format!("builtin type {other:?} is outside the av.collections/1 source surface"),
            )),
        },
        hir::TypeUse::Ref(tid) => {
            if let Some(idx) = memo.get(tid) {
                return Ok(*idx);
            }
            if !visiting.insert(*tid) {
                return Err(AdapterError::inv(
                    "mir.type-cycle",
                    format!("type id {} participates in a type cycle", tid.0),
                ));
            }
            let td = m.types.get(tid.0 as usize).ok_or_else(|| {
                AdapterError::inv("lower.type-ref", format!("type id {} out of range", tid.0))
            })?;
            let mapped = match &td.kind {
                hir::TypeKind::I32 => Ok(out.intern_type(MirType::I32)),
                hir::TypeKind::Bool => Ok(out.intern_type(MirType::Bool)),
                hir::TypeKind::StrOwned => Ok(out.intern_type(MirType::Str)),
                hir::TypeKind::Record(shape) => {
                    let mut fields = Vec::with_capacity(shape.fields.len());
                    let mut err = None;
                    for f in &shape.fields {
                        match collections_type_idx_of(&f.ty, m, out, memo, visiting) {
                            Ok(t) => fields.push(MirFieldDef {
                                name: f.name.clone(),
                                ty: t,
                            }),
                            Err(e) => {
                                err = Some(e);
                                break;
                            }
                        }
                    }
                    err.map_or_else(
                        || {
                            Ok(out.intern_type(MirType::Record(RecordTypeRef {
                                owner_module: m.id_text.clone(),
                                name: td.id_text.clone(),
                                fields,
                            })))
                        },
                        Err,
                    )
                }
                hir::TypeKind::List { elem } => {
                    let e = collections_type_idx_of(elem, m, out, memo, visiting)?;
                    Ok(out.intern_type(MirType::List(ListTypeRef { elem: e })))
                }
                hir::TypeKind::Variant { arms } => {
                    let mut mir_arms = Vec::with_capacity(arms.len());
                    let mut err = None;
                    for a in arms {
                        match &a.payload {
                            Some(p) => match collections_type_idx_of(p, m, out, memo, visiting) {
                                Ok(t) => mir_arms.push(VariantArm {
                                    name: a.name.clone(),
                                    payload: Some(t),
                                }),
                                Err(e) => {
                                    err = Some(e);
                                    break;
                                }
                            },
                            None => mir_arms.push(VariantArm {
                                name: a.name.clone(),
                                payload: None,
                            }),
                        }
                    }
                    err.map_or_else(
                        || {
                            Ok(out.intern_type(MirType::Variant(VariantTypeRef {
                                owner_module: m.id_text.clone(),
                                name: td.id_text.clone(),
                                arms: mir_arms,
                            })))
                        },
                        Err,
                    )
                }
                other => Err(AdapterError::uns(
                    "lower.type-use",
                    format!("type kind {:?} is outside the av.collections/1 source surface", other),
                )),
            };
            visiting.remove(tid);
            let idx = mapped?;
            memo.insert(*tid, idx);
            Ok(idx)
        }
    }
}

fn owned_type_idx_of(
    u: &hir::TypeUse,
    m: &hir::Module,
    out: &mut MirModule,
    memo: &mut std::collections::BTreeMap<hir::TypeId, TypeIdx>,
    visiting: &mut std::collections::BTreeSet<hir::TypeId>,
) -> Result<TypeIdx, AdapterError> {
    match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Ok(out.intern_type(MirType::I32)),
            hir::BuiltinType::Bool => Ok(out.intern_type(MirType::Bool)),
            hir::BuiltinType::Char => Ok(out.intern_type(MirType::Char)),
            hir::BuiltinType::StrOwned => Ok(out.intern_type(MirType::Str)),
            hir::BuiltinType::StrSlice => Ok(out.intern_type(MirType::StrView)),
            // PLAN-862: Unit is a RESULT type of the entry-unit profile,
            // which this owned-profile lowering never sees (defense).
            hir::BuiltinType::Unit => Err(AdapterError::uns(
                "lower.type-use",
                "the unit result type belongs to the av.entry-unit/1 profile".to_string(),
            )),
        },
        hir::TypeUse::Ref(tid) => {
            if let Some(idx) = memo.get(tid) {
                return Ok(*idx);
            }
            if !visiting.insert(*tid) {
                return Err(AdapterError::inv(
                    "mir.type-cycle",
                    format!("record type id {} participates in a type cycle", tid.0),
                ));
            }
            let td = m.types.get(tid.0 as usize).ok_or_else(|| {
                AdapterError::inv("lower.type-ref", format!("type id {} out of range", tid.0))
            })?;
            let mapped = match &td.kind {
                hir::TypeKind::I32 => Ok(out.intern_type(MirType::I32)),
                hir::TypeKind::Bool => Ok(out.intern_type(MirType::Bool)),
                // Named text kinds (PLAN-770 type-table entries): `str`,
                // `Str`, `char` arrive as Ref uses in the text-record domain.
                hir::TypeKind::Str => Ok(out.intern_type(MirType::StrView)),
                hir::TypeKind::StrOwned => Ok(out.intern_type(MirType::Str)),
                hir::TypeKind::Char => Ok(out.intern_type(MirType::Char)),
                hir::TypeKind::Record(shape) => {
                    let mut fields = Vec::with_capacity(shape.fields.len());
                    let mut err = None;
                    for f in &shape.fields {
                        match owned_type_idx_of(&f.ty, m, out, memo, visiting) {
                            Ok(t) => fields.push(MirFieldDef {
                                name: f.name.clone(),
                                ty: t,
                            }),
                            Err(e) => {
                                err = Some(e);
                                break;
                            }
                        }
                    }
                    err.map_or_else(
                        || {
                            Ok(out.intern_type(MirType::Record(RecordTypeRef {
                                owner_module: m.id_text.clone(),
                                name: td.id_text.clone(),
                                fields,
                            })))
                        },
                        Err,
                    )
                }
                other => Err(AdapterError::uns(
                    "lower.type-use",
                    format!("type kind {:?} is outside the av.owned/1 profile", other),
                )),
            };
            visiting.remove(tid);
            let idx = mapped?;
            memo.insert(*tid, idx);
            Ok(idx)
        }
    }
}

// ---------------------------------------------------------------------------
// Function lowering
// ---------------------------------------------------------------------------

/// Where a segment hands control when it completes without its own
/// terminator: a concrete block (loop latch / inherited continuation) or
/// the ON-DEMAND after-if join (F798-R7-05).
#[derive(Clone, Copy)]
enum Exit {
    Block(BlockId),
    Join(usize),
}

/// Where a segment starts: a concrete block, or the after-if join
/// allocated on first need — a join both of whose arms terminated is
/// never created, so no unreachable unwired block remains.
#[derive(Clone, Copy)]
enum Start {
    Block(BlockId),
    JoinOnDemand(usize),
}

/// One pending HIR-block segment: lower stmts from `stmt_idx` into the
/// start block; when the segment completes without its own terminator,
/// jump to `exit` (after-if join / loop latch / inherited continuation).
struct Task {
    hir_block: hir::BlockId,
    stmt_idx: usize,
    mir_block: Start,
    exit: Option<Exit>,
}

struct FnLower<'a> {
    hir_module: &'a hir::Module,
    body: &'a hir::Body,
    def_map: &'a BTreeMap<u32, CalleeRef>,
    out: &'a mut MirModule,
    slot_of_local: Vec<SlotId>,
    /// MIR type index per BASE slot (owned lowering's transfer-vs-copy).
    slot_ty_cache: Vec<TypeIdx>,
    base_slots: u32,
    extra_slots: Vec<MirSlot>,
    t_i32: TypeIdx,
    t_bool: TypeIdx,
    /// Data-profile lowering (av.data/1): enables record type mapping and
    /// the record expression/statement arms.
    data: bool,
    /// Owned-profile lowering (av.owned/1, PLAN-821): subsumes `data` and
    /// adds the text/ownership surface (str ops, clone/move, borrow views).
    owned: bool,
    ty_memo: BTreeMap<hir::TypeId, TypeIdx>,
    ty_visiting: std::collections::BTreeSet<hir::TypeId>,
    blocks: Vec<MirBlock>,
    /// Current open MIR block cursor.
    cur: BlockId,
    /// Whether the current segment already emitted its terminator.
    terminated: bool,
    /// (header, exit) MIR blocks per HIR LoopId.
    loop_targets: Vec<(BlockId, BlockId)>,
    func_index: u32,
    /// Running flat instruction counter — emission order equals block-major
    /// order because blocks are created in order and never revisited.
    flat: u32,
    /// R2-F08' mut-record write-back: the t_MutRet wrapper type of THIS
    /// function (None = no mut record params) and the param slots to wrap.
    mut_wrap_ty: Option<TypeIdx>,
    mut_param_slots: Vec<SlotId>,
    /// R2-F08' call-site: (BorrowMut temp, borrowed local slot) pairs.
    mut_borrow_temps: Vec<(SlotId, SlotId)>,

    /// R3-F07 retained interface facts: param modes per def id for
    /// BODY-LESS scaffold defs (D828 single-function documents carry no
    /// body table for unchanged functions; the wrapper ABI needs the
    /// callee's mut-record param positions, which the document's type
    /// section cannot express). Captured where the real bodies exist.
    sig_mode_overrides: BTreeMap<String, Vec<hir::ParamMode>>,
    /// Slot index (into the FINAL slot table) one past the last slot at
    /// the current STATEMENT's start — statement-tail borrow ends (774
    /// 语句尾寿命 for temporaries) release view temps minted inside the
    /// statement once its core instruction has been emitted.
    stmt_slot_watermark: u32,
    spans: Vec<(u32, SpanInfo)>,
    /// Whether block 0 got a real (non-placeholder) terminator.
    root_wired: bool,
    /// After-if join cells (F798-R7-05): `None` until some arm falls
    /// through (or a continuation with remaining statements starts) —
    /// a both-arms-terminated `if` never materializes its join.
    pending_joins: Vec<Option<BlockId>>,
}

impl<'a> FnLower<'a> {
    /// The HIR TypeUse of an expression (declared type).
    fn expr_ty_hir(&self, eid: hir::ExprId) -> Result<hir::TypeUse, AdapterError> {
        self.body
            .exprs
            .get(eid.0 as usize)
            .map(|e| e.ty)
            .ok_or_else(|| {
                AdapterError::inv("lower.expr-ref", format!("expr {} out of range", eid.0))
            })
    }

    /// PLAN-864: whether this lowering targets the av.collections/1
    /// profile (the module profile is the flag — no signature threading).
    fn collections(&self) -> bool {
        self.out.is_collections_profile()
    }

    /// PLAN-864: HIR-level owned classification for a List element type
    /// (mirrors verify's rule): Str / record-reaching-Str / Variant.
    fn hir_elem_owned(&self, list_expr: hir::ExprId) -> Result<bool, AdapterError> {
        let ty = self.expr_ty_hir(list_expr)?;
        let Some(hir::TypeUse::Ref(tid)) = Some(ty) else {
            return Ok(false);
        };
        let Some(hir::TypeKind::List { elem }) =
            self.hir_module.types.get(tid.0 as usize).map(|t| &t.kind)
        else {
            return Ok(false);
        };
        Ok(match elem {
            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => true,
            hir::TypeUse::Builtin(_) => false,
            hir::TypeUse::Ref(e) => {
                match self.hir_module.types.get(e.0 as usize).map(|t| &t.kind) {
                    Some(hir::TypeKind::Variant { .. }) => true,
                    Some(hir::TypeKind::Record(_)) => hir_record_reaches_str(self.hir_module, *e),
                    _ => false,
                }
            }
        })
    }

    /// PLAN-864: the MIR element type of a List-typed expression.
    fn elem_vt_of(&mut self, list_e: hir::ExprId) -> Result<TypeIdx, AdapterError> {
        let Some(hir::TypeUse::Ref(tid)) = Some(self.expr_ty_hir(list_e)?) else {
            return Err(AdapterError::inv(
                "lower.list-type",
                "list receiver is not a ref type".to_string(),
            ));
        };
        let Some(hir::TypeKind::List { elem }) =
            self.hir_module.types.get(tid.0 as usize).map(|t| &t.kind)
        else {
            return Err(AdapterError::inv(
                "lower.list-type",
                "receiver is not a list".to_string(),
            ));
        };
        collections_type_idx_of(
            elem,
            self.hir_module,
            self.out,
            &mut self.ty_memo,
            &mut self.ty_visiting,
        )
    }

    /// PLAN-864: the (list, index, path) of a TextLen operand that is an
    /// owned-element view shape — a bare `xs.get(i)` (whole Str element,
    /// empty path) or `xs.get(i).f[.g]` (FieldGet rooted at a ListGet).
    /// `None` for every other receiver (the owned text path handles it).
    fn list_projection_root(
        &self,
        operand: hir::ExprId,
    ) -> Result<Option<(hir::ExprId, hir::ExprId, Vec<hir::FieldId>)>, AdapterError> {
        match &self.body.exprs[operand.0 as usize].kind {
            hir::ExprKind::ListGet { list, index } => Ok(Some((*list, *index, Vec::new()))),
            hir::ExprKind::FieldGet { base, path } => {
                match &self.body.exprs[base.0 as usize].kind {
                    hir::ExprKind::ListGet { list, index } => {
                        Ok(Some((*list, *index, path.clone())))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    fn new_block(&mut self) -> BlockId {
        self.blocks.push(MirBlock {
            instrs: vec![],
            // Placeholder; every created block is the target of exactly one
            // task and receives a real terminator before the walk ends.
            term: Terminator::Jump { target: u32::MAX },
        });
        (self.blocks.len() - 1) as BlockId
    }

    fn emit(&mut self, instr: MirInstr, span: Option<SpanInfo>) {
        self.blocks[self.cur as usize].instrs.push(instr);
        if let Some(sp) = span {
            self.spans.push((self.flat, sp));
        }
        self.flat += 1;
    }

    /// Allocate (once) the join block of an on-demand join cell.
    fn join_block(&mut self, slot: usize) -> BlockId {
        if let Some(b) = self.pending_joins[slot] {
            return b;
        }
        let b = self.new_block();
        self.pending_joins[slot] = Some(b);
        b
    }

    fn set_term(&mut self, term: Terminator) {
        self.blocks[self.cur as usize].term = term;
        self.terminated = true;
        if self.cur == 0 {
            self.root_wired = true;
        }
    }

    fn temp(&mut self, shape: ScalarShape) -> SlotId {
        let ty = match shape {
            ScalarShape::I32 => self.t_i32,
            ScalarShape::Bool => self.t_bool,
            _ => unreachable!("wide-scalar temps only exist in typed fixtures"),
        };
        self.temp_ty(ty)
    }

    /// Allocate a temp slot of an arbitrary (possibly record) MIR type.
    /// A temp of an already-interned type index (write-back unwrap).
    fn temp_ty_from_cache(&mut self, ty: TypeIdx) -> SlotId {
        let idx = self.base_slots as usize + self.extra_slots.len();
        self.extra_slots.push(MirSlot {
            kind: SlotKind::Temp,
            ty,
            name: None,
        });
        idx as SlotId
    }

    /// R3-F08: text receivers take a VIEW. An owned-Str operand (e.g. an
    /// owned Str returned from a mut call) is VIEWed first — the str ops'
    /// MIR operands are StrView by contract; a view-typed operand passes
    /// through unchanged (same dispatch as TextToBorrowed).
    fn text_view_operand(
        &mut self,
        operand: &hir::ExprId,
        depth: u32,
    ) -> Result<SlotId, AdapterError> {
        let v = self.eval_expr(*operand, depth)?;
        let src_ty = self.expr_ty(*operand)?;
        if matches!(self.out.types.get(src_ty as usize), Some(MirType::Str)) {
            let view_ty = self.out.intern_type(MirType::StrView);
            let view = self.temp_ty_from_cache(view_ty);
            let sp = None;
            self.emit(MirInstr::View { dst: view, src: v }, sp);
            Ok(view)
        } else {
            Ok(v)
        }
    }

    /// R2-F08': map a BorrowMut argument TEMP back to the borrowed LOCAL
    /// slot (the temp was created by the BorrowMut arm's MOVE).
    fn mut_borrow_target(&self, arg_temp: SlotId) -> Option<SlotId> {
        self.mut_borrow_temps
            .iter()
            .find(|(t, _)| *t == arg_temp)
            .map(|(_, l)| *l)
    }

    /// R3-F08: the callee's MUT-RECORD param positions in signature
    /// order — exactly the wrapper's wb fields, in order. The modes lead
    /// the callee body's local table (HIR verify invariant); a BODY-LESS
    /// scaffold def (D828 single-function document) consults the retained
    /// [`Self::sig_mode_overrides`]. Record-ness reads the callee's own
    /// MIR signature (a mut record param maps to `Record`, a read-only
    /// one to `RecordView`) — the same fact `owned_fn_sig_for_def`
    /// interned the wrapper from.
    fn callee_mut_record_params(
        &self,
        callee_def: hir::DefId,
        callee_sig_ty: &Option<MirType>,
    ) -> Vec<usize> {
        let mut out = Vec::new();
        let def = self.hir_module.defs.get(callee_def.0 as usize);
        let modes: Option<Vec<hir::ParamMode>> = def
            .and_then(|d| match &d.kind {
                // PLAN-881: a method callee's mut-record params wrap
                // exactly like a free function's (modes are positional
                // facts of the body's param locals; the receiver is
                // params[0] with mode View).
                hir::DefKind::Function { body, .. } | hir::DefKind::Method { body, .. } => {
                    if body.0 == u32::MAX {
                        // Scaffold sentinel: no body — retained facts.
                        None
                    } else {
                        self.hir_module.bodies.get(body.0 as usize).map(|b| {
                            b.locals
                                .iter()
                                .filter(|l| l.role == hir::LocalRole::Param)
                                .map(|l| l.param_mode)
                                .collect()
                        })
                    }
                }
                _ => None,
            })
            .or_else(|| {
                def.map(|d| d.id_text.as_str())
                    .and_then(|id| self.sig_mode_overrides.get(id).cloned())
            });
        let Some(modes) = modes else {
            return out;
        };
        let Some(MirType::Fn(sig)) = callee_sig_ty else {
            return out;
        };
        for (pi, pt) in sig.params.iter().enumerate() {
            let mode = modes.get(pi).copied().unwrap_or(hir::ParamMode::View);
            if mode != hir::ParamMode::MutBorrow {
                continue;
            }
            if matches!(self.out.types.get(*pt as usize), Some(MirType::Record(_))) {
                out.push(pi);
            }
        }
        out
    }

    fn temp_ty(&mut self, ty: TypeIdx) -> SlotId {
        let idx = self.base_slots + self.extra_slots.len() as SlotId;
        self.extra_slots.push(MirSlot {
            kind: SlotKind::Temp,
            ty,
            name: None,
        });
        idx
    }

    /// End every StrView TEMP minted since the statement watermark (774
    /// statement-tail lifetime): a method-receiver view dies with its
    /// statement, so a later field replace of the root is legal. Named
    /// view LOCALS live to scope end (782's root-hold semantics).
    fn end_stmt_temp_views(&mut self) {
        let base = self.base_slots;
        let wm = self.stmt_slot_watermark;
        let mut ends: Vec<SlotId> = Vec::new();
        for (i, slot) in self.extra_slots.iter().enumerate().rev() {
            let idx = base + i as u32;
            if idx < wm {
                break;
            }
            if matches!(self.out.types.get(slot.ty as usize), Some(MirType::StrView)) {
                ends.push(idx);
            }
        }
        for t in ends {
            self.emit(MirInstr::BorrowEnd { token: t }, None);
        }
        self.stmt_slot_watermark = base + self.extra_slots.len() as u32;
    }

    fn const_i32(&mut self, v: i32) -> u32 {
        self.out.intern_const(MirConst::I32(v))
    }

    fn const_bool(&mut self, v: bool) -> u32 {
        self.out.intern_const(MirConst::Bool(v))
    }

    /// The MIR type index of an expression's HIR type (data path maps
    /// records; the scalar path is unchanged).
    fn expr_ty(&mut self, eid: hir::ExprId) -> Result<TypeIdx, AdapterError> {
        let e = self.body.exprs.get(eid.0 as usize).ok_or_else(|| {
            AdapterError::inv("lower.expr-ref", format!("expr {} out of range", eid.0))
        })?;
        let (m, out) = (self.hir_module, &mut *self.out);
        let is_coll = out.is_collections_profile();
        if is_coll {
            collections_type_idx_of(&e.ty, m, out, &mut self.ty_memo, &mut self.ty_visiting)
        } else if self.owned {
            owned_type_idx_of(&e.ty, m, out, &mut self.ty_memo, &mut self.ty_visiting)
        } else {
            type_idx_of_use(
                &e.ty,
                m,
                out,
                self.data,
                &mut self.ty_memo,
                &mut self.ty_visiting,
            )
        }
    }

    /// The MIR type of an expression's slot kind WITHOUT allocating.
    fn type_of_expr(&self, eid: hir::ExprId) -> Option<MirType> {
        let e = self.body.exprs.get(eid.0 as usize)?;
        match &e.ty {
            hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => Some(MirType::StrView),
            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => Some(MirType::Str),
            hir::TypeUse::Builtin(hir::BuiltinType::Char) => Some(MirType::Char),
            hir::TypeUse::Builtin(hir::BuiltinType::I32) => Some(MirType::I32),
            hir::TypeUse::Builtin(hir::BuiltinType::Bool) => Some(MirType::Bool),
            _ => None,
        }
    }

    fn span(&self, eid: hir::ExprId) -> Option<SpanInfo> {
        self.body.exprs.get(eid.0 as usize).map(|e| SpanInfo {
            func: self.func_index,
            start: e.span.start as u32,
            end: e.span.end as u32,
        })
    }

    /// Evaluate one expression into a slot (existing local slot or fresh
    /// temp), emitting into the current block. Short-circuit `Logical`
    /// opens new blocks; the cursor lands on the join.
    fn eval_expr(&mut self, eid: hir::ExprId, depth: u32) -> Result<SlotId, AdapterError> {
        if depth > MAX_EXPR_DEPTH {
            return Err(AdapterError::uns(
                "mir.lower.depth",
                format!(
                    "expression nesting exceeds the lowering ceiling {}",
                    MAX_EXPR_DEPTH
                ),
            ));
        }
        // PLAN-864: unit-typed expressions (push/insert/set/clear) never
        // map their type — Unit is not a profile-4 table entry and the
        // effect arms return the NO_SLOT sentinel.
        let is_unit_result = self.collections()
            && matches!(
                self.body.exprs[eid.0 as usize].ty,
                hir::TypeUse::Builtin(hir::BuiltinType::Unit)
            );
        let ty = if is_unit_result {
            self.t_i32
        } else {
            self.expr_ty(eid)?
        };
        let sp = self.span(eid);
        let expr_kind = &self.body.exprs[eid.0 as usize].kind;
        match expr_kind {
            hir::ExprKind::ReadLocal { local } => Ok(self.slot_of_local[local.0 as usize]),
            // PLAN-862: the unit value. Zero-width: the temp slot carries the
            // Unit type and NO instruction initializes it (there is no data
            // to write). `verify_mir` treats Unit slots as initialized by
            // width and admits this slot only as the Unit entry's return
            // operand.
            hir::ExprKind::Unit => Ok(self.temp_ty(ty)),
            hir::ExprKind::Constant { value } => {
                let dst = self.temp_ty(ty);
                let c = self.const_i32(*value);
                self.emit(MirInstr::Const { dst, value: c }, sp);
                Ok(dst)
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. }
                if self.owned
                    && matches!(op, hir::BinOp::Eq | hir::BinOp::Ne)
                    && matches!(self.type_of_expr(*lhs), Some(MirType::StrView)) =>
            {
                // str eq/ne rides STR_CMP (BIN stays scalar-only).
                let l = self.eval_expr(*lhs, depth + 1)?;
                let r = self.eval_expr(*rhs, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::StrCmp {
                        eq: matches!(op, hir::BinOp::Eq),
                        dst,
                        lhs: l,
                        rhs: r,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.eval_expr(*lhs, depth + 1)?;
                let r = self.eval_expr(*rhs, depth + 1)?;
                let dst = self.temp_ty(ty);
                let mop = map_binop(*op)?;
                self.emit(
                    MirInstr::Bin {
                        op: mop,
                        dst,
                        lhs: l,
                        rhs: r,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::Unary { op, operand } => {
                let v = self.eval_expr(*operand, depth + 1)?;
                let dst = self.temp_ty(ty);
                let mop = match op {
                    hir::UnaryOp::NegI32 => UnOp::NegI32,
                    hir::UnaryOp::NotBool => UnOp::NotBool,
                };
                self.emit(
                    MirInstr::Un {
                        op: mop,
                        dst,
                        operand: v,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // Short-circuit: the result slot is defined in BOTH arms.
                let l = self.eval_expr(*lhs, depth + 1)?;
                let result = self.temp(ScalarShape::Bool);
                let rhs_b = self.new_block();
                let skip_b = self.new_block();
                let join = self.new_block();
                let (then_b, else_b) = match op {
                    hir::LogicOp::And => (rhs_b, skip_b),
                    hir::LogicOp::Or => (skip_b, rhs_b),
                };
                self.set_term(Terminator::Branch {
                    cond: l,
                    then: then_b,
                    else_: else_b,
                });
                // Skipped arm: result = the short-circuit value.
                self.cur = skip_b;
                self.terminated = false;
                let c = self.const_bool(matches!(op, hir::LogicOp::Or));
                self.emit(
                    MirInstr::Const {
                        dst: result,
                        value: c,
                    },
                    sp,
                );
                self.set_term(Terminator::Jump { target: join });
                // Taken arm: result = rhs.
                self.cur = rhs_b;
                self.terminated = false;
                let r = self.eval_expr(*rhs, depth + 1)?;
                let rsp = self.span(*rhs);
                self.emit(
                    MirInstr::Copy {
                        dst: result,
                        src: r,
                    },
                    rsp,
                );
                self.set_term(Terminator::Jump { target: join });
                self.cur = join;
                self.terminated = false;
                Ok(result)
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                // Source-order evaluation, then binding-map reorder.
                let mut arg_slots = Vec::with_capacity(eval_args.len());
                for a in eval_args {
                    arg_slots.push(self.eval_expr(*a, depth + 1)?);
                }
                let callee_ref = self.def_map.get(&callee.0).cloned().ok_or_else(|| {
                    AdapterError::inv(
                        "lower.callee",
                        format!("callee def {} out of range", callee.0),
                    )
                })?;
                if let CalleeRef::Import(imp) = callee_ref {
                    let symbol = self.out.imports[imp as usize].symbol.clone();
                    if symbol.starts_with("fn:") {
                        // Unit lowering (PLAN-828 T-04): a cross-function
                        // callee becomes a `fn:<name>` unit import call —
                        // the same shape split_units produces post-hoc; the
                        // checked linking environment resolves it at
                        // snapshot link with a signature check. Arg binding
                        // mirrors the Func path (source-order eval, formal
                        // mapping through `bindings`).
                        let sig_idx = self.out.imports[imp as usize].sig;
                        let n_params = self
                            .out
                            .sig_of(sig_idx)
                            .map(|s| s.params.len())
                            .unwrap_or(0);
                        let mut actuals: Vec<SlotId> = vec![0; n_params];
                        for b in bindings {
                            let pos = b.param as usize;
                            if pos >= n_params {
                                return Err(AdapterError::inv(
                                    "lower.binding",
                                    format!("call binding param {pos} out of range"),
                                ));
                            }
                            actuals[pos] = arg_slots[b.arg as usize];
                        }
                        let dst = self.temp_ty(ty);
                        self.emit(
                            MirInstr::Call {
                                callee: CalleeRef::Import(imp),
                                dst,
                                args: actuals,
                            },
                            sp,
                        );
                        return Ok(dst);
                    }
                    // Test marks: trace event + frozen i32 result.
                    let dst = self.temp_ty(self.t_i32);
                    let (event, value) = match symbol.as_str() {
                        "hir.test.mark_a" => (TraceEvent::MarkA, 1i32),
                        "hir.test.mark_b" => (TraceEvent::MarkB, 2),
                        other => {
                            return Err(AdapterError::uns(
                                "lower.intrinsic",
                                format!("intrinsic `{}` is not in the av profile", other),
                            ));
                        }
                    };
                    self.emit(MirInstr::Trace { event }, sp);
                    let c = self.const_i32(value);
                    self.emit(MirInstr::Const { dst, value: c }, sp);
                    return Ok(dst);
                }
                // The callee signature comes from the HIR def table —
                // forward and self references resolve before any body of
                // that function has been lowered.
                let callee_def = *callee;
                let hir_ty = self
                    .hir_module
                    .defs
                    .get(callee_def.0 as usize)
                    .and_then(|d| d.kind.fn_type())
                    .ok_or_else(|| {
                        AdapterError::inv(
                            "lower.callee-sig",
                            "callee lacks a function type".to_string(),
                        )
                    })?;
                let sig_idx = fn_type_index_ext(
                    hir_ty,
                    self.hir_module,
                    self.out,
                    self.data,
                    self.owned,
                    &mut self.ty_memo,
                    &mut self.ty_visiting,
                )?;
                let n_params = self
                    .out
                    .sig_of(sig_idx)
                    .map(|s| s.params.len())
                    .unwrap_or(0);
                let mut actuals: Vec<SlotId> = vec![0; n_params];
                for b in bindings {
                    let pos = b.param as usize;
                    if pos >= n_params {
                        return Err(AdapterError::inv(
                            "lower.binding",
                            format!("call binding param {} out of range", b.param),
                        ));
                    }
                    actuals[pos] = arg_slots[b.arg as usize];
                }
                // R2-F08': does the callee carry mut record params? Its
                // MIR sig result is then the t_MutRet wrapper.
                let callee_sig_ty = match callee_ref {
                    CalleeRef::Func(fi) => self
                        .out
                        .funcs
                        .get(fi as usize)
                        .and_then(|f| self.out.types.get(f.sig as usize).cloned()),
                    CalleeRef::Import(ii) => self
                        .out
                        .imports
                        .get(ii as usize)
                        .and_then(|imp| self.out.types.get(imp.sig as usize).cloned()),
                };
                let wrapper_fields = match callee_sig_ty.as_ref() {
                    Some(MirType::Fn(f)) => match self.out.types.get(f.result as usize) {
                        Some(MirType::Record(r)) if r.name.starts_with("t_MutRet_") => {
                            Some((f.result, r.fields.clone()))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                if let Some((wrap_ty, fields)) = wrapper_fields {
                    // The call result IS the wrapper: dst gets the WRAPPER
                    // type; then f0 = the source result, wb* write back.
                    let wrap_dst = self.temp_ty_from_cache(wrap_ty);
                    self.emit(
                        MirInstr::Call {
                            callee: callee_ref,
                            dst: wrap_dst,
                            args: actuals.clone(),
                        },
                        sp,
                    );
                    // f0: the SOURCE result. A copyable result copies out
                    // (FIELD_GET); an OWNED result (Str / resource record)
                    // is not Copy — FIELD_MOVE field 0 is the only
                    // ownership exit (R3-F08: FieldGet of a Str field only
                    // ever mints a view).
                    let f0_owned = auto_av_core::ownership::type_is_owned(&self.out, fields[0].ty);
                    let v = self.temp_ty(ty);
                    if f0_owned {
                        self.emit(
                            MirInstr::FieldMove {
                                dst: v,
                                root: wrap_dst,
                                path: vec![0],
                            },
                            sp,
                        );
                    } else {
                        self.emit(
                            MirInstr::FieldGet {
                                dst: v,
                                base: wrap_dst,
                                path: vec![0],
                            },
                            sp,
                        );
                    }
                    // wb*: one write-back field per MUT-RECORD param of the
                    // callee, in signature order (fields[1..] mirror that
                    // order exactly). Only the callee's OWN mut params
                    // consume wrapper fields — plain (read-only) record
                    // args and non-record args contribute NOTHING (R3-F08:
                    // the old loop keyed on "any record actual", mis-
                    // indexing mixed signatures and panicking past the
                    // wrapper's field count).
                    let mut wb_idx = 1usize;
                    for pos in self.callee_mut_record_params(callee_def, &callee_sig_ty) {
                        let back = self.temp_ty_from_cache(fields[wb_idx].ty);
                        self.emit(
                            MirInstr::FieldMove {
                                dst: back,
                                root: wrap_dst,
                                path: vec![wb_idx as u16],
                            },
                            sp,
                        );
                        // The write-back target: the ORIGINAL BorrowMut'd
                        // local — recover it from the HIR binding (the
                        // arg slot is the MOVE temp).
                        if let Some(target) = self.mut_borrow_target(actuals[pos]) {
                            self.emit(
                                MirInstr::Move {
                                    dst: target,
                                    src: back,
                                },
                                sp,
                            );
                        } else {
                            // Untracked target: drop the extracted record
                            // (the borrow discipline was violated upstream).
                            self.emit(MirInstr::Drop { slot: back }, sp);
                        }
                        wb_idx += 1;
                    }
                    debug_assert_eq!(
                        wb_idx,
                        fields.len(),
                        "the wrapper is fully consumed: f0 + every wb field"
                    );
                    return Ok(v);
                }
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::Call {
                        callee: callee_ref,
                        dst,
                        args: actuals,
                    },
                    sp,
                );
                Ok(dst)
            }
            // -- PLAN-821 owned text surface (av.owned/1) ------------------
            hir::ExprKind::StrLit { value } if self.owned => {
                // Destination kind follows the expression's HIR type: an
                // owning context materializes a fresh buffer; a borrowing
                // context gets the immortal static view (770).
                let dst = self.temp_ty(ty);
                let c = self
                    .out
                    .intern_const(MirConst::Str(value.as_bytes().to_vec().into_boxed_slice()));
                self.emit(MirInstr::Const { dst, value: c }, sp);
                Ok(dst)
            }
            hir::ExprKind::CharLit { scalar } if self.owned => {
                let dst = self.temp_ty(ty);
                let ch = char::from_u32(*scalar).ok_or_else(|| {
                    AdapterError::inv(
                        "lower.char-domain",
                        format!("char literal {scalar:#x} is not a Unicode scalar"),
                    )
                })?;
                let c = self.out.intern_const(MirConst::Char(ch));
                self.emit(MirInstr::Const { dst, value: c }, sp);
                Ok(dst)
            }
            hir::ExprKind::TextLen { operand } if self.owned && !self.collections() => {
                let v = self.text_view_operand(operand, depth)?;
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::StrLen { dst, view: v }, sp);
                Ok(dst)
            }
            hir::ExprKind::TextLenBytes { operand } if self.owned && !self.collections() => {
                let v = self.text_view_operand(operand, depth)?;
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::StrLenBytes { dst, view: v }, sp);
                Ok(dst)
            }
            hir::ExprKind::TextCharAt { operand, index } if self.owned && !self.collections() => {
                let v = self.text_view_operand(operand, depth)?;
                let i = self.eval_expr(*index, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::StrCharAt {
                        dst,
                        view: v,
                        index: i,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } if self.owned && !self.collections() => {
                let v = self.text_view_operand(operand, depth)?;
                let a = self.eval_expr(*start, depth + 1)?;
                let b = self.eval_expr(*end, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::StrSlice {
                        dst,
                        view: v,
                        start: a,
                        end: b,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::TextConcat { lhs, rhs } if self.owned && !self.collections() => {
                let l = self.eval_expr(*lhs, depth + 1)?;
                let r = self.eval_expr(*rhs, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::StrConcat {
                        dst,
                        lhs: l,
                        rhs: r,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::TextClone { operand } if self.owned && !self.collections() => {
                // Clone of an OWNED Str re-buffers; clone of a view is the
                // owned copy of its range (770 AP-2 over both receivers).
                let src_ty = self.expr_ty(*operand)?;
                let v = self.eval_expr(*operand, depth + 1)?;
                let dst = self.temp_ty(ty);
                match self.out.types.get(src_ty as usize) {
                    Some(MirType::StrView) => {
                        self.emit(MirInstr::StrToOwned { dst, view: v }, sp);
                    }
                    _ => {
                        self.emit(MirInstr::Clone { dst, src: v }, sp);
                    }
                }
                Ok(dst)
            }
            hir::ExprKind::TextToOwned { operand } if self.owned => {
                let v = self.eval_expr(*operand, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::StrToOwned { dst, view: v }, sp);
                Ok(dst)
            }
            hir::ExprKind::TextToBorrowed { operand } if self.owned => {
                // `.to(str)` of an owned Str slot mints a rooted VIEW; of a
                // view it is the identity borrow (a legal view share).
                let src_ty = self.expr_ty(*operand)?;
                let v = self.eval_expr(*operand, depth + 1)?;
                let dst = self.temp_ty(ty);
                match self.out.types.get(src_ty as usize) {
                    Some(MirType::Str) => {
                        self.emit(MirInstr::View { dst, src: v }, sp);
                    }
                    _ => {
                        self.emit(MirInstr::Copy { dst, src: v }, sp);
                    }
                }
                Ok(dst)
            }
            hir::ExprKind::CharToCodepoint { operand } if self.owned => {
                let v = self.eval_expr(*operand, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::CharToCodepoint { dst, ch: v }, sp);
                Ok(dst)
            }
            hir::ExprKind::CharFromCodepoint { operand } if self.owned => {
                let v = self.eval_expr(*operand, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::CharFromCodepoint { dst, code: v }, sp);
                Ok(dst)
            }
            hir::ExprKind::RecordClone { base, .. } if self.owned => {
                let v = self.eval_expr(*base, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::Clone { dst, src: v }, sp);
                Ok(dst)
            }
            hir::ExprKind::Move { inner } if self.owned => {
                // Whole-root transfer: the inner expression must be a
                // whole-local read (verify.partial-move is the source
                // gate); evaluate it to its slot and MOVE out of it.
                match &self.body.exprs[inner.0 as usize].kind {
                    hir::ExprKind::ReadLocal { local } => {
                        let src = self.slot_of_local[local.0 as usize];
                        let dst = self.temp_ty(ty);
                        self.emit(MirInstr::Move { dst, src }, sp);
                        Ok(dst)
                    }
                    _ => Err(AdapterError::uns(
                        "lower.expr",
                        ".move must wrap a whole-local read (partial moves are verify-rejected)"
                            .to_string(),
                    )),
                }
            }
            hir::ExprKind::BorrowView { base } if self.owned => {
                // R1-F08: a borrowed record argument is a PASS-THROUGH —
                // the record stays in its caller slot; the callee's
                // RecordView param signature carries the no-kill borrow
                // (782 ABI-D `*const`; the ownership analysis and the
                // runtime are both sig-driven).
                self.eval_expr(*base, depth + 1)
            }
            hir::ExprKind::BorrowMut { local } if self.owned => {
                // R2-F08': `a.mut` — the exclusive write-back borrow. The
                // whole local MOVES into a call-argument temp (exclusivity
                // by transfer; the Call arm's write-back MOVE restores the
                // caller's local from the callee's returned wrapper).
                let slot = self
                    .slot_of_local
                    .get(local.0 as usize)
                    .copied()
                    .ok_or_else(|| {
                        AdapterError::inv(
                            "lower.borrow",
                            format!("mut borrow of unknown local {}", local.0),
                        )
                    })?;
                let ty = self.slot_ty_cache[slot as usize];
                let tmp = self.temp_ty(ty);
                self.mut_borrow_temps.push((tmp, slot));
                self.emit(
                    MirInstr::Move {
                        dst: tmp,
                        src: slot,
                    },
                    sp,
                );
                Ok(tmp)
            }
            // -- PLAN-820 record surface (av.data/1) ----------------------
            hir::ExprKind::RecordCtor {
                record,
                eval_args,
                bindings,
            } => {
                if !self.data && !self.owned {
                    return Err(AdapterError::uns(
                        "lower.expr",
                        "record construction is outside the av.core-i32/1 profile".to_string(),
                    ));
                }
                // Source-order evaluation into temps (side effects as
                // written — never re-sorted by declaration order), then the
                // binding map projects into DECLARATION field order for
                // AGGREGATE (contract-decision §4/§7).
                let mut arg_slots = Vec::with_capacity(eval_args.len());
                for a in eval_args {
                    arg_slots.push(self.eval_expr(*a, depth + 1)?);
                }
                let shape = match self.hir_module.types.get(record.0 as usize) {
                    Some(hir::TypeDef {
                        kind: hir::TypeKind::Record(s),
                        ..
                    }) => s,
                    _ => {
                        return Err(AdapterError::inv(
                            "lower.record-ref",
                            format!("constructor type id {} is not a record", record.0),
                        ))
                    }
                };
                let rec_ty = if self.collections() {
                    collections_type_idx_of(
                        &hir::TypeUse::Ref(*record),
                        self.hir_module,
                        self.out,
                        &mut self.ty_memo,
                        &mut self.ty_visiting,
                    )?
                } else if self.owned {
                    owned_type_idx_of(
                        &hir::TypeUse::Ref(*record),
                        self.hir_module,
                        self.out,
                        &mut self.ty_memo,
                        &mut self.ty_visiting,
                    )?
                } else {
                    type_idx_of_use(
                        &hir::TypeUse::Ref(*record),
                        self.hir_module,
                        self.out,
                        self.data,
                        &mut self.ty_memo,
                        &mut self.ty_visiting,
                    )?
                };
                let nfields = shape.fields.len();
                if bindings.len() != nfields {
                    return Err(AdapterError::inv(
                        "lower.binding",
                        format!(
                            "constructor binds {} fields, record declares {}",
                            bindings.len(),
                            nfields
                        ),
                    ));
                }
                let mut operands: Vec<SlotId> = vec![0; nfields];
                for b in bindings {
                    if b.field as usize >= nfields {
                        return Err(AdapterError::inv(
                            "lower.binding",
                            format!("constructor field index {} out of range", b.field),
                        ));
                    }
                    if b.arg as usize >= arg_slots.len() {
                        return Err(AdapterError::inv(
                            "lower.binding",
                            format!("constructor arg index {} out of range", b.arg),
                        ));
                    }
                    operands[b.field as usize] = arg_slots[b.arg as usize];
                }
                let dst = self.temp_ty(rec_ty);
                self.emit(
                    MirInstr::Aggregate {
                        dst,
                        record_ty: rec_ty,
                        operands,
                    },
                    sp,
                );
                Ok(dst)
            }
            // -- PLAN-864 collections surface ------------------------------
            hir::ExprKind::ListNew { list } if self.collections() => {
                let ty = self.expr_ty(eid)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::ListNew {
                        dst,
                        list_ty: ty,
                        cap: 4,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::ListLen { list } if self.collections() => {
                let l = self.eval_expr(*list, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::ListLen { dst, src: l }, sp);
                Ok(dst)
            }
            hir::ExprKind::ListPush { list, value } if self.collections() => {
                let l = self.eval_expr(*list, depth + 1)?;
                let v = self.eval_expr(*value, depth + 1)?;
                self.emit(MirInstr::ListPush { root: l, value: v }, sp);
                Ok(NO_SLOT)
            }
            hir::ExprKind::ListInsert { list, index, value }
            | hir::ExprKind::ListSet { list, index, value }
                if self.collections() =>
            {
                let l = self.eval_expr(*list, depth + 1)?;
                let i = self.eval_expr(*index, depth + 1)?;
                let v = self.eval_expr(*value, depth + 1)?;
                let is_insert = matches!(
                    &self.body.exprs[eid.0 as usize].kind,
                    hir::ExprKind::ListInsert { .. }
                );
                if is_insert {
                    self.emit(
                        MirInstr::ListInsert {
                            root: l,
                            index: i,
                            value: v,
                        },
                        sp,
                    );
                } else {
                    self.emit(
                        MirInstr::ListSet {
                            root: l,
                            index: i,
                            value: v,
                        },
                        sp,
                    );
                }
                Ok(NO_SLOT)
            }
            hir::ExprKind::ListGet { list, index } if self.collections() => {
                if self.hir_elem_owned(*list)? {
                    // verify guarantees this never happens (view-escape);
                    // keep the lowering total for programmatic bundles.
                    return Err(AdapterError::inv(
                        "lower.list-get-owned",
                        "list_get over an owned-element list has no value lowering".to_string(),
                    ));
                }
                let l = self.eval_expr(*list, depth + 1)?;
                let i = self.eval_expr(*index, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::ListGet {
                        dst,
                        base: l,
                        index: i,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::ListRemove { list, index } if self.collections() => {
                let l = self.eval_expr(*list, depth + 1)?;
                let i = self.eval_expr(*index, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::ListRemove {
                        dst,
                        root: l,
                        index: i,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::ListClear { list } if self.collections() => {
                // `xs.clear()` — remove-from-tail loop (no LIST_CLEAR opcode
                // exists in profile 4 and none is added): per round, re-read
                // len; when > 0, remove(len-1) and (for owned elements)
                // drop the removed value. Frame teardown releases the empty
                // container at the exits.
                let l = self.eval_expr(*list, depth + 1)?;
                let elem_ty = match &self.body.exprs[list.0 as usize].ty {
                    hir::TypeUse::Ref(tid) => {
                        match self.hir_module.types.get(tid.0 as usize).map(|t| &t.kind) {
                            Some(hir::TypeKind::List { elem }) => Some(collections_type_idx_of(
                                elem,
                                self.hir_module,
                                self.out,
                                &mut self.ty_memo.clone(),
                                &mut self.ty_visiting.clone(),
                            )?),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                let owned_elem = self.hir_elem_owned(*list)?;
                let t_i32 = self.out.intern_type(MirType::I32);
                let t_bool = self.out.intern_type(MirType::Bool);
                let header = self.new_block();
                let body_b = self.new_block();
                let exit_b = self.new_block();
                self.set_term(Terminator::Jump { target: header });
                self.cur = header;
                self.terminated = false;
                let n = self.temp_ty(t_i32);
                self.emit(MirInstr::ListLen { dst: n, src: l }, sp);
                let zero = self.temp_ty(t_i32);
                let c0 = self.out.intern_const(MirConst::I32(0));
                self.emit(
                    MirInstr::Const {
                        dst: zero,
                        value: c0,
                    },
                    sp,
                );
                let c = self.temp_ty(t_bool);
                self.emit(
                    MirInstr::Bin {
                        op: BinOp::GtI32,
                        dst: c,
                        lhs: n,
                        rhs: zero,
                    },
                    sp,
                );
                self.set_term(Terminator::Branch {
                    cond: c,
                    then: body_b,
                    else_: exit_b,
                });
                self.cur = body_b;
                self.terminated = false;
                let one = self.temp_ty(t_i32);
                let c1 = self.out.intern_const(MirConst::I32(1));
                self.emit(
                    MirInstr::Const {
                        dst: one,
                        value: c1,
                    },
                    sp,
                );
                let idx = self.temp_ty(t_i32);
                self.emit(
                    MirInstr::Bin {
                        op: BinOp::SubI32,
                        dst: idx,
                        lhs: n,
                        rhs: one,
                    },
                    sp,
                );
                let e = self.temp_ty(elem_ty.unwrap_or(t_i32));
                self.emit(
                    MirInstr::ListRemove {
                        dst: e,
                        root: l,
                        index: idx,
                    },
                    sp,
                );
                if owned_elem {
                    self.emit(MirInstr::Drop { slot: e }, sp);
                }
                self.set_term(Terminator::Jump { target: header });
                self.cur = exit_b;
                self.terminated = false;
                Ok(NO_SLOT)
            }
            hir::ExprKind::VariantCtor {
                variant,
                disc,
                payload,
            } if self.collections() => {
                let vt = self.expr_ty(eid)?;
                let pl = match payload {
                    Some(p) => Some(self.eval_expr(*p, depth + 1)?),
                    None => None,
                };
                let dst = self.temp_ty(vt);
                self.emit(
                    MirInstr::VariantPack {
                        dst,
                        variant_ty: vt,
                        disc: *disc,
                        payload: pl,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::VariantGuardTest { root, disc } if self.collections() => {
                let r = self.eval_expr(*root, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::VariantGuard {
                        dst,
                        src: r,
                        disc: *disc,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::VariantProject { root, disc } if self.collections() => {
                let r = self.eval_expr(*root, depth + 1)?;
                let dst = self.temp_ty(ty);
                self.emit(
                    MirInstr::VariantProject {
                        dst,
                        root: r,
                        disc: *disc,
                    },
                    sp,
                );
                Ok(dst)
            }
            // TextLen over an owned-element projection: the fused view
            // sequence (contract C4) — remove → field view / element view →
            // strlen → insert back. Plain receivers keep the owned path.
            hir::ExprKind::TextLen { operand } if self.collections() => {
                let proj = self.list_projection_root(*operand)?;
                let Some((list_e, index_e, path)) = proj else {
                    // plain receiver (a local Str / literal): owned path
                    let v = self.text_view_operand(operand, depth)?;
                    let dst = self.temp_ty(ty);
                    self.emit(MirInstr::StrLen { dst, view: v }, sp);
                    return Ok(dst);
                };
                let (l, i) = (
                    self.eval_expr(list_e, depth + 1)?,
                    self.eval_expr(index_e, depth + 1)?,
                );
                let view_ty = self.out.intern_type(MirType::StrView);
                // 1. move the element OUT (single extraction)
                let evty = self.elem_vt_of(list_e)?;
                let e = self.temp_ty(evty);
                self.emit(
                    MirInstr::ListRemove {
                        dst: e,
                        root: l,
                        index: i,
                    },
                    sp,
                );
                // 2. reach the terminal Str: a whole Str element is already
                // a Str slot; a field rides the profile-4 FieldGet contract
                // (exact-type dst — the runtime handle-SHARES the field, no
                // deep copy, no ledger event; the temp share releases with
                // the frame while the buffer stays owned by the element).
                let str_slot = if path.is_empty() {
                    e
                } else {
                    let str_ty = self.out.intern_type(MirType::Str);
                    let tmp = self.temp_ty(str_ty);
                    let segs: Vec<u16> = path.iter().map(|f| f.0 as u16).collect();
                    self.emit(
                        MirInstr::FieldGet {
                            dst: tmp,
                            base: e,
                            path: segs,
                        },
                        sp,
                    );
                    tmp
                };
                // 3. view + read
                let v = self.temp_ty(view_ty);
                self.emit(
                    MirInstr::View {
                        dst: v,
                        src: str_slot,
                    },
                    sp,
                );
                let dst = self.temp_ty(ty);
                self.emit(MirInstr::StrLen { dst, view: v }, sp);
                // end the view BEFORE the restore — an insert of a value
                // under an active borrow is a mut-conflict in the lattice.
                self.emit(MirInstr::BorrowEnd { token: v }, sp);
                // 4. restore the element exactly (same buffers; len back)
                self.emit(
                    MirInstr::ListInsert {
                        root: l,
                        index: i,
                        value: e,
                    },
                    sp,
                );
                Ok(dst)
            }
            // FieldGet rooted at a ListGet over an OWNED list: the fused
            // LIST_FIELD_GET (scalar terminal — owned terminals only ride
            // TextLen, verify-guaranteed).
            hir::ExprKind::FieldGet { base, path } if self.collections() => {
                if let Some(root @ hir::ExprKind::ListGet { list, index }) =
                    list_get_root(self.body, *base)
                {
                    if self.hir_elem_owned(list)? {
                        let l = self.eval_expr(list, depth + 1)?;
                        let i = self.eval_expr(index, depth + 1)?;
                        let dst = self.temp_ty(ty);
                        let segs: Vec<u16> = path.iter().map(|f| f.0 as u16).collect();
                        self.emit(
                            MirInstr::ListFieldGet {
                                dst,
                                base: l,
                                index: i,
                                path: segs,
                            },
                            sp,
                        );
                        let _ = root;
                        return Ok(dst);
                    }
                }
                let b = self.eval_expr(*base, depth + 1)?;
                let dst = self.temp_ty(ty);
                let segs: Vec<u16> = path.iter().map(|f| f.0 as u16).collect();
                self.emit(
                    MirInstr::FieldGet {
                        dst,
                        base: b,
                        path: segs,
                    },
                    sp,
                );
                Ok(dst)
            }
            hir::ExprKind::FieldGet { base, path } => {
                if !self.data && !self.owned {
                    return Err(AdapterError::uns(
                        "lower.expr",
                        "field reads are outside the av.core-i32/1 profile".to_string(),
                    ));
                }
                let b = self.eval_expr(*base, depth + 1)?;
                let dst = self.temp_ty(ty);
                let segs: Vec<u16> = path.iter().map(|f| f.0 as u16).collect();
                self.emit(
                    MirInstr::FieldGet {
                        dst,
                        base: b,
                        path: segs,
                    },
                    sp,
                );
                Ok(dst)
            }
            // By-value record parameters/returns ride a BorrowView WRAPPER
            // in the HIR (774 ABI-D `*const` shape). In the pure-value data
            // profile this node is a transparent passthrough — evaluate the
            // base and hand the value on (records are Copy; no borrow
            // semantics exist here). Mut borrows stay outside the profile.
            hir::ExprKind::BorrowView { base } => {
                if !self.data {
                    return Err(AdapterError::uns(
                        "lower.borrow",
                        "borrow expressions are outside the av.core-i32/1 profile".to_string(),
                    ));
                }
                self.eval_expr(*base, depth + 1)
            }
            hir::ExprKind::BorrowMut { .. } => {
                return Err(AdapterError::uns(
                    "lower.borrow",
                    "mut borrows are outside the av.data/1 profile".to_string(),
                ));
            }
            other => Err(AdapterError::uns(
                "lower.expr",
                format!(
                    "expression kind `{}` is outside the av profile",
                    expr_kind_name(other)
                ),
            )),
        }
    }
}

fn map_binop(op: hir::BinOp) -> Result<BinOp, AdapterError> {
    Ok(match op {
        hir::BinOp::AddI32 => BinOp::AddI32,
        hir::BinOp::SubI32 => BinOp::SubI32,
        hir::BinOp::MulI32 => BinOp::MulI32,
        hir::BinOp::DivI32 => BinOp::DivI32,
        hir::BinOp::RemI32 => BinOp::RemI32,
        hir::BinOp::LtI32 => BinOp::LtI32,
        hir::BinOp::LeI32 => BinOp::LeI32,
        hir::BinOp::GtI32 => BinOp::GtI32,
        hir::BinOp::GeI32 => BinOp::GeI32,
        hir::BinOp::Eq => BinOp::EqI32,
        hir::BinOp::Ne => BinOp::NeI32,
    })
}

fn expr_kind_name(k: &hir::ExprKind) -> &'static str {
    use hir::ExprKind::*;
    match k {
        ReadLocal { .. } => "read-local",
        Constant { .. } => "constant",
        // PLAN-862: the unit value.
        Unit => "unit",
        Binary { .. } => "binary",
        Unary { .. } => "unary",
        Logical { .. } => "logical",
        Call { .. } => "call",
        StrLit { .. } => "str-lit",
        CharLit { .. } => "char-lit",
        TextLen { .. } => "text-len",
        TextLenBytes { .. } => "text-len-bytes",
        TextCharAt { .. } => "text-char-at",
        TextSlice { .. } => "text-slice",
        TextConcat { .. } => "text-concat",
        TextClone { .. } => "text-clone",
        TextToOwned { .. } => "text-to-owned",
        TextToBorrowed { .. } => "text-to-borrowed",
        CharToCodepoint { .. } => "char-to-codepoint",
        CharFromCodepoint { .. } => "char-from-codepoint",
        RecordCtor { .. } => "record-ctor",
        FieldGet { .. } => "field-get",
        BorrowView { .. } => "borrow-view",
        BorrowMut { .. } => "borrow-mut",
        RecordClone { .. } => "record-clone",
        Move { .. } => "move",
        // PLAN-864 collections surface
        ListNew { .. } => "list-new",
        ListLen { .. } => "list-len",
        ListPush { .. } => "list-push",
        ListInsert { .. } => "list-insert",
        ListGet { .. } => "list-get",
        ListSet { .. } => "list-set",
        ListRemove { .. } => "list-remove",
        ListClear { .. } => "list-clear",
        VariantCtor { .. } => "variant-ctor",
        VariantGuardTest { .. } => "variant-guard-test",
        VariantProject { .. } => "variant-project",
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_function(
    hir_module: &hir::Module,
    out: &mut MirModule,
    def_index: u32,
    body_id: hir::BodyId,
    sig: TypeIdx,
    source_snapshot: u64,
    def_map: &BTreeMap<u32, CalleeRef>,
    name: String,
    data: bool,
    owned: bool,
    mode_overrides: &BTreeMap<String, Vec<hir::ParamMode>>,
) -> Result<MirFunction, AdapterError> {
    let body = hir_module.bodies.get(body_id.0 as usize).ok_or_else(|| {
        AdapterError::inv("lower.body-ref", format!("body {} out of range", body_id.0))
    })?;

    let t_i32 = out.intern_type(MirType::I32);
    let t_bool = out.intern_type(MirType::Bool);

    // Base slots: params lead in signature order (HIR verify guarantees the
    // param locals lead the local table in order), then the other locals.
    let mut slots: Vec<MirSlot> = Vec::with_capacity(body.locals.len());
    let mut slot_of_local = vec![0u32; body.locals.len()];
    let mut ty_memo = BTreeMap::new();
    let mut ty_visiting = std::collections::BTreeSet::new();
    for (li, l) in body.locals.iter().enumerate() {
        let mut ty = if out.is_collections_profile() {
            collections_type_idx_of(&l.ty, hir_module, out, &mut ty_memo, &mut ty_visiting)?
        } else if owned {
            owned_type_idx_of(&l.ty, hir_module, out, &mut ty_memo, &mut ty_visiting)?
        } else {
            type_idx_of_use(&l.ty, hir_module, out, data, &mut ty_memo, &mut ty_visiting)?
        };
        if owned && l.role == hir::LocalRole::Param && l.param_mode == hir::ParamMode::View {
            // R1-F08: a plain record param is a READ-ONLY borrow — the
            // slot type mirrors the signature's RecordView.
            if let Some(MirType::Record(r)) = out.types.get(ty as usize).cloned() {
                ty = out.intern_type(MirType::RecordView(r));
            }
        }
        let idx = slots.len() as SlotId;
        slots.push(MirSlot {
            kind: if l.role == hir::LocalRole::Param {
                SlotKind::Param
            } else {
                SlotKind::Local
            },
            ty,
            name: Some(l.name.clone()),
        });
        slot_of_local[li] = idx;
    }
    let base_slots = slots.len() as u32;
    let slot_ty_cache: Vec<TypeIdx> = slots.iter().map(|s| s.ty).collect();

    // R2-F08': mut record params — plain OWNED record slots (move-in);
    // their final values wrap into the t_MutRet result at Return.
    let mut mut_param_slots: Vec<SlotId> = Vec::new();
    let mut mut_wrap_ty: Option<TypeIdx> = None;
    if owned {
        let mut has_mut = false;
        for (li, l) in body.locals.iter().enumerate() {
            if l.role != hir::LocalRole::Param || l.param_mode != hir::ParamMode::MutBorrow {
                continue;
            }
            let ty = slot_ty_cache[li];
            if matches!(out.types.get(ty as usize), Some(MirType::Record(_))) {
                has_mut = true;
                mut_param_slots.push(slot_of_local[li]);
            }
        }
        if has_mut {
            // The wrapper's field types: (src result, mut param records).
            let sig_ty = out.types.get(sig as usize).cloned();
            if let Some(MirType::Fn(f)) = sig_ty {
                // The result of the MIR sig IS the wrapper (built by
                // owned_fn_sig_for_def): f0 = src result, wb* = params.
                mut_wrap_ty = Some(f.result);
            }
        }
    }
    let mut lower = FnLower {
        hir_module,
        body,
        def_map,
        out,
        slot_of_local,
        slot_ty_cache,
        base_slots,
        extra_slots: vec![],
        t_i32,
        t_bool,
        data,
        owned,
        ty_memo,
        ty_visiting,
        blocks: vec![],
        cur: 0,
        terminated: false,
        loop_targets: vec![(0, 0); body.loops.len()],
        func_index: def_index,
        flat: 0,
        mut_wrap_ty,
        mut_param_slots,
        mut_borrow_temps: vec![],
        sig_mode_overrides: mode_overrides.clone(),
        stmt_slot_watermark: slots.len() as u32,
        spans: vec![],
        root_wired: false,
        pending_joins: vec![],
    };
    let entry_b = lower.new_block();
    debug_assert_eq!(entry_b, 0);

    let mut stack: Vec<Task> = vec![Task {
        hir_block: body.entry,
        stmt_idx: 0,
        mir_block: Start::Block(0),
        exit: None,
    }];
    while let Some(mut task) = stack.pop() {
        // F798-R7-05: an after-if continuation whose join was never
        // allocated (both arms terminated) and that has no remaining
        // statements is dead bookkeeping — skipping it leaves no
        // unreachable unwired block.
        let start_block = match task.mir_block {
            Start::Block(b) => b,
            Start::JoinOnDemand(slot) => {
                let has_stmts = body
                    .blocks
                    .get(task.hir_block.0 as usize)
                    .map(|b| task.stmt_idx < b.stmts.len())
                    .unwrap_or(false);
                if lower.pending_joins[slot].is_none() && !has_stmts {
                    continue;
                }
                lower.join_block(slot)
            }
        };
        lower.cur = start_block;
        lower.terminated = false;
        loop {
            let block = body.blocks.get(task.hir_block.0 as usize).ok_or_else(|| {
                AdapterError::inv(
                    "lower.block-ref",
                    format!("block {} out of range", task.hir_block.0),
                )
            })?;
            if task.stmt_idx >= block.stmts.len() {
                break;
            }
            let stmt = &block.stmts[task.stmt_idx];
            task.stmt_idx += 1;
            lower.stmt_slot_watermark = lower.base_slots + lower.extra_slots.len() as u32;
            match stmt {
                hir::Stmt::Let { local, value } => {
                    // PLAN-864: a variant-typed binding whose initializer is
                    // a constructor commits through VARIANT_SWITCH (the
                    // 822 slot-write transaction), never a raw Move.
                    if lower.collections() {
                        if let hir::ExprKind::VariantCtor { disc, payload, .. } =
                            &lower.body.exprs[value.0 as usize].kind
                        {
                            let dst = lower.slot_of_local[local.0 as usize];
                            let pl = match payload {
                                Some(p) => Some(lower.eval_expr(*p, 0)?),
                                None => None,
                            };
                            let disc = *disc;
                            lower.end_stmt_temp_views();
                            lower.emit(
                                MirInstr::VariantSwitch {
                                    root: dst,
                                    disc,
                                    payload: pl,
                                },
                                lower.span(*value),
                            );
                            continue;
                        }
                    }
                    let v = lower.eval_expr(*value, 0)?;
                    lower.end_stmt_temp_views();
                    let dst = lower.slot_of_local[local.0 as usize];
                    let sp = lower.span(*value);
                    if lower.owned {
                        // The binding's slot type decides: owned kinds
                        // transfer (MOVE — the temp dies), copyables copy.
                        let slot_ty = lower
                            .out
                            .types
                            .get(lower.slot_ty_cache[dst as usize] as usize)
                            .cloned();
                        let is_owned = slot_ty
                            .as_ref()
                            .map(|t| auto_av_core::ownership::type_is_owned_type_of(lower.out, t))
                            .unwrap_or(false);
                        if is_owned {
                            lower.emit(MirInstr::Move { dst, src: v }, sp);
                        } else {
                            lower.emit(MirInstr::Copy { dst, src: v }, sp);
                        }
                    } else {
                        lower.emit(MirInstr::Copy { dst, src: v }, sp);
                    }
                }
                hir::Stmt::Assign { place, value } => {
                    let p = body.places.get(place.0 as usize).ok_or_else(|| {
                        AdapterError::inv(
                            "lower.place-ref",
                            format!("place {} out of range", place.0),
                        )
                    })?;
                    match &p.kind {
                        hir::PlaceKind::Local => {
                            let dst = lower.slot_of_local[p.local.0 as usize];
                            // PLAN-864: `r = Ok(42)` — variant slot writes
                            // ride VARIANT_SWITCH (old live arm drops once,
                            // guard facts die with the switch).
                            if lower.collections() {
                                if let hir::ExprKind::VariantCtor { disc, payload, .. } =
                                    &lower.body.exprs[value.0 as usize].kind
                                {
                                    let pl = match payload {
                                        Some(pp) => Some(lower.eval_expr(*pp, 0)?),
                                        None => None,
                                    };
                                    let disc = *disc;
                                    lower.end_stmt_temp_views();
                                    lower.emit(
                                        MirInstr::VariantSwitch {
                                            root: dst,
                                            disc,
                                            payload: pl,
                                        },
                                        lower.span(*value),
                                    );
                                    continue;
                                }
                            }
                            let v = lower.eval_expr(*value, 0)?;
                            lower.end_stmt_temp_views();
                            let sp = lower.span(*value);
                            if lower.owned {
                                let slot_ty = lower
                                    .out
                                    .types
                                    .get(lower.slot_ty_cache[dst as usize] as usize)
                                    .cloned();
                                let is_owned = slot_ty
                                    .as_ref()
                                    .map(|t| {
                                        auto_av_core::ownership::type_is_owned_type_of(lower.out, t)
                                    })
                                    .unwrap_or(false);
                                if is_owned {
                                    lower.emit(MirInstr::Move { dst, src: v }, sp);
                                } else {
                                    lower.emit(MirInstr::Copy { dst, src: v }, sp);
                                }
                            } else {
                                lower.emit(MirInstr::Copy { dst, src: v }, sp);
                            }
                        }
                        hir::PlaceKind::FieldPath(path) => {
                            if !lower.data && !lower.owned {
                                return Err(AdapterError::uns(
                                    "lower.place-kind",
                                    "field-path assignment is record-domain, outside the av profile"
                                        .to_string(),
                                ));
                            }
                            // RHS FIRST (materialized in its own slot — the
                            // overlap invariant), then the structural write
                            // through the root local (contract-decision §4).
                            let v = lower.eval_expr(*value, 0)?;
                            lower.end_stmt_temp_views();
                            let root = lower.slot_of_local[p.local.0 as usize];
                            let segs: Vec<u16> = path.iter().map(|f| f.0 as u16).collect();
                            let sp = lower.span(*value);
                            lower.emit(
                                MirInstr::FieldSet {
                                    root,
                                    path: segs,
                                    value: v,
                                },
                                sp,
                            );
                        }
                    }
                }
                hir::Stmt::Return { value } => {
                    let v = lower.eval_expr(*value, 0)?;
                    lower.end_stmt_temp_views();
                    // R2-F08': a mut-record-param function returns the
                    // (source result, final mut params) wrapper — the
                    // caller unwraps and writes the params back.
                    let v = if let Some(wrap_ty) = lower.mut_wrap_ty {
                        let mut operands = vec![v];
                        for p in &lower.mut_param_slots {
                            operands.push(*p);
                        }
                        let w = lower.temp_ty(wrap_ty);
                        lower.emit(
                            MirInstr::Aggregate {
                                dst: w,
                                record_ty: wrap_ty,
                                operands,
                            },
                            None,
                        );
                        w
                    } else {
                        v
                    };
                    lower.set_term(Terminator::Return { value: v });
                    break; // HIR verify: nothing follows a terminator
                }
                hir::Stmt::If { cond, then, els } => {
                    let c = lower.eval_expr(*cond, 0)?;
                    lower.end_stmt_temp_views();
                    let then_b = lower.new_block();
                    let then_exit;
                    match els {
                        Some(else_hir) => {
                            // F798-R7-05: with both arms present the join
                            // is ON DEMAND — `if a { return } else { return }`
                            // never materializes an unreachable join block.
                            let else_b = lower.new_block();
                            let slot = lower.pending_joins.len();
                            lower.pending_joins.push(None);
                            then_exit = Some(Exit::Join(slot));
                            lower.set_term(Terminator::Branch {
                                cond: c,
                                then: then_b,
                                else_: else_b,
                            });
                            // LIFO: continuation processed last.
                            stack.push(Task {
                                hir_block: task.hir_block,
                                stmt_idx: task.stmt_idx,
                                mir_block: Start::JoinOnDemand(slot),
                                exit: task.exit,
                            });
                            stack.push(Task {
                                hir_block: *else_hir,
                                stmt_idx: 0,
                                mir_block: Start::Block(else_b),
                                exit: Some(Exit::Join(slot)),
                            });
                        }
                        None => {
                            // No else: the branch's false edge targets the
                            // join, so it is required eagerly.
                            let join = lower.new_block();
                            then_exit = Some(Exit::Block(join));
                            lower.set_term(Terminator::Branch {
                                cond: c,
                                then: then_b,
                                else_: join,
                            });
                            stack.push(Task {
                                hir_block: task.hir_block,
                                stmt_idx: task.stmt_idx,
                                mir_block: Start::Block(join),
                                exit: task.exit,
                            });
                        }
                    }
                    stack.push(Task {
                        hir_block: *then,
                        stmt_idx: 0,
                        mir_block: Start::Block(then_b),
                        exit: then_exit,
                    });
                    break;
                }
                hir::Stmt::Loop {
                    id,
                    body: loop_body,
                } => {
                    let header = lower.new_block();
                    let body_b = lower.new_block();
                    let exit = lower.new_block();
                    lower.loop_targets[id.0 as usize] = (header, exit);
                    lower.set_term(Terminator::Jump { target: header });
                    lower.cur = header;
                    lower.terminated = false;
                    lower.set_term(Terminator::Jump { target: body_b });
                    stack.push(Task {
                        hir_block: task.hir_block,
                        stmt_idx: task.stmt_idx,
                        mir_block: Start::Block(exit),
                        exit: task.exit,
                    });
                    stack.push(Task {
                        hir_block: *loop_body,
                        stmt_idx: 0,
                        mir_block: Start::Block(body_b),
                        exit: Some(Exit::Block(header)),
                    });
                    break;
                }
                hir::Stmt::Break { target } => {
                    let (_, exit) = lower.loop_targets[target.0 as usize];
                    lower.set_term(Terminator::Jump { target: exit });
                    break;
                }
                hir::Stmt::Continue { target } => {
                    let (header, _) = lower.loop_targets[target.0 as usize];
                    lower.set_term(Terminator::Jump { target: header });
                    break;
                }
                hir::Stmt::Drop { local: _ } => {
                    // Scalar locals own no resources; programmatic drops are
                    // no-ops in this profile (cleanup is compiler-scheduled).
                }
                hir::Stmt::Eval { value } => {
                    // PLAN-864: a unit-effect statement (push/insert/set/
                    // clear) — evaluate for effect; the unit result never
                    // becomes a slot (NO_SLOT convention).
                    // The result slot (a discarded copy-element remove or a
                    // unit sentinel) releases with the frame; owned
                    // transfers are verify-rejected at the source.
                    let _ = lower.eval_expr(*value, 0)?;
                    lower.end_stmt_temp_views();
                }
            }
        }
        // Segment completed without its own terminator: hand control to the
        // recorded exit (after-if join / loop latch / inherited).
        if !lower.terminated {
            match task.exit {
                Some(Exit::Block(b)) => {
                    lower.set_term(Terminator::Jump { target: b });
                }
                Some(Exit::Join(slot)) => {
                    let j = lower.join_block(slot);
                    lower.set_term(Terminator::Jump { target: j });
                }
                // exit == None (root): the placeholder stays; rejected below.
                None => {}
            }
        }
    }

    // Sanity: every block carries a real terminator; only the root may be
    // left open (missing return path — checked input cannot reach this).
    for (bi, b) in lower.blocks.iter().enumerate() {
        if matches!(b.term, Terminator::Jump { target: u32::MAX }) {
            if bi == 0 && !lower.root_wired {
                return Err(AdapterError::inv(
                    "lower.missing-return",
                    "entry block completed without a return path".to_string(),
                ));
            }
            return Err(AdapterError::inv(
                "lower.unwired-block",
                format!("block {} never received a terminator", bi),
            ));
        }
    }

    // Spans are appended in emission order == flat order.
    debug_assert!(lower.spans.windows(2).all(|w| w[0].0 < w[1].0));

    let owner = lower.out.owner.clone();
    let mut slots_final = slots;
    slots_final.append(&mut lower.extra_slots);
    Ok(MirFunction {
        owner,
        name,
        sig,
        slots: slots_final,
        blocks: lower.blocks,
        entry: 0,
        source_snapshot,
        spans: lower.spans,
        env: None,
    })
}
