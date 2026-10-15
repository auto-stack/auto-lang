//! Native lowering for the core-i32 profile (PLAN-741, Windows x86-64).
//!
//! Consumes a `CheckedModule` (verifier output only) and lowers the module
//! that owns the selected entry to a Windows x64 COFF object via Cranelift.
//! The object exports every module function plus a generic `ac_start`
//! wrapper that calls the zero-parameter i32 entry and forwards its result
//! to `ExitProcess`. Arithmetic declared `overflow: trap` takes an explicit
//! error exit (`ExitProcess(70)`) — exit code 70 is the test-profile trap
//! code, not a language-wide exception ABI.
//!
//! Boundaries (plan §5.3):
//! - target is the Windows x64 calling convention; no production Auto ABI is
//!   frozen here,
//! - checked HIR carries no registers/SSA; locals map to Cranelift variables
//!   and the structured block tree maps to Cranelift blocks,
//! - intrinsics become imports of their declared symbol (e.g.
//!   `hir.test.mark_a`); the document's `requires` capabilities must have
//!   been provided to the build before lowering (`capabilities_check`).

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::hir::{self, DefId, ExprId, LoopId};
use crate::verify::CheckedModule;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::FuncRef;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature, Value};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::{BTreeMap, BTreeSet};
use target_lexicon::Triple;

const WINCALL: CallConv = CallConv::WindowsFastcall;
/// Exit code used by the test profile's overflow trap path (plan §5.3).
pub const TRAP_EXIT_CODE: i32 = 70;
/// Symbol name of the generated startup wrapper.
pub const START_SYMBOL: &str = "ac_start";

/// Fail the build when a document requires a capability the caller did not
/// explicitly provide. Providing a capability is the only way to link a
/// module that needs it; it is never inferred.
pub fn capabilities_check(
    checked: &CheckedModule,
    provided: &[String],
) -> Result<(), Vec<Diagnostic>> {
    let mut diags = Vec::new();
    // r2 QA-02: the DERIVED runtime need gates even without a declaration
    // (verify_text already rejects undeclared usage; this is the build-side
    // backstop).
    if checked.requires_runtime_text()
        && !provided
            .iter()
            .any(|p| p == crate::text_resources::RUNTIME_TEXT_CAPABILITY)
    {
        diags.push(Diagnostic::new(
            Stage::Capability,
            "capability.missing",
            format!(
                "document requires capability `{}`, which this build does not provide",
                crate::text_resources::RUNTIME_TEXT_CAPABILITY
            ),
            Span { start: 0, end: 0 },
        ));
    }
    for req in checked.required_capabilities() {
        if !provided.iter().any(|p| p == req) {
            diags.push(Diagnostic::new(
                Stage::Capability,
                "capability.missing",
                format!(
                    "document requires capability `{}`, which this build does not provide",
                    req
                ),
                Span { start: 0, end: 0 },
            ));
        }
    }
    if diags.is_empty() {
        Ok(())
    } else {
        Err(diags)
    }
}

/// The lowered entry: a zero-parameter, i32-returning function def selected
/// by its textual id.
#[derive(Clone, Debug)]
pub struct EntrySelection {
    pub module: usize,
    pub def: DefId,
    /// Symbol exported for the entry function (its declared name).
    pub symbol: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Vt {
    I32,
    Bool,
    Function,
    /// Nominal record type (PLAN-771; 774 D-05): lowered as a pointer
    /// (ABI-D `*const` view / ABI-E `*mut` mut+out) with value storage in
    /// stack slots — never as a by-value aggregate parameter or return.
    Record(hir::TypeId),
}

/// Total projection of a type use (review P767-R2 hardening): an
/// out-of-range `TypeUse::Ref` is `None` — a controlled backend error —
/// instead of an indexing panic. Verify's module type-bounds gate makes
/// this unreachable for Checked modules; this keeps the backend total for
/// any future caller.
fn value_of_type_use(u: &hir::TypeUse, m: &hir::Module) -> Option<Vt> {
    match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Some(Vt::I32),
            hir::BuiltinType::Bool => Some(Vt::Bool),
            // Text values flow through the text lowering path (PLAN-770
            // T-05); the scalar signature projection stays i32/bool-only
            // and reports None — a controlled backend rejection.
            hir::BuiltinType::StrSlice | hir::BuiltinType::StrOwned | hir::BuiltinType::Char => {
                None
            }
        },
        hir::TypeUse::Ref(tid) => match &m.types.get(tid.0 as usize)?.kind {
            hir::TypeKind::I32 => Some(Vt::I32),
            hir::TypeKind::Bool => Some(Vt::Bool),
            hir::TypeKind::Function(_) => Some(Vt::Function),
            hir::TypeKind::Str | hir::TypeKind::StrOwned | hir::TypeKind::Char => None,
            hir::TypeKind::Record(_) => Some(Vt::Record(*tid)),
        },
    }
}

/// Record byte sizes by type index (PLAN-771 §3 / 774 D-05): declaration
/// order, every field 4-aligned, size = sum of field sizes (scalar fields
/// are 4 bytes; nested fields contribute their own size). Computed with an
/// explicit work stack and checked adds — the verifier guarantees acyclicity.
fn record_layouts(m: &hir::Module) -> BTreeMap<u32, u64> {
    let n = m.types.len();
    let mut size: Vec<Option<u64>> = vec![None; n];
    let is_record = |i: usize| {
        matches!(
            m.types.get(i).map(|t| &t.kind),
            Some(hir::TypeKind::Record(_))
        )
    };
    for start in 0..n {
        if !is_record(start) || size[start].is_some() {
            continue;
        }
        let mut stack: Vec<usize> = vec![start];
        let mut on_path = vec![false; n];
        while let Some(&node) = stack.last() {
            let done = match &m.types[node].kind {
                hir::TypeKind::Record(shape) => {
                    let mut pending = false;
                    for f in &shape.fields {
                        if let hir::TypeUse::Ref(t) = &f.ty {
                            let fi = t.0 as usize;
                            if is_record(fi) && size[fi].is_none() {
                                if !on_path[fi] {
                                    on_path[node] = true;
                                    stack.push(fi);
                                    pending = true;
                                    break;
                                }
                            }
                        }
                    }
                    if pending {
                        continue;
                    }
                    let mut total: u64 = 0;
                    for f in &shape.fields {
                        let fsz = match &f.ty {
                            hir::TypeUse::Builtin(_) => 4u64,
                            hir::TypeUse::Ref(t) => {
                                let fi = t.0 as usize;
                                if is_record(fi) {
                                    size[fi].unwrap_or(4)
                                } else {
                                    4
                                }
                            }
                        };
                        total = total.checked_add(fsz).unwrap_or(u32::MAX as u64);
                    }
                    size[node] = Some(total);
                    true
                }
                _ => true,
            };
            if done {
                on_path[node] = false;
                stack.pop();
            }
        }
    }
    let mut out = BTreeMap::new();
    for (i, sz) in size.into_iter().enumerate() {
        if let Some(sz) = sz {
            out.insert(i as u32, sz);
        }
    }
    out
}

/// Whether a type use names a RECORD type (refs to i32/bool typedefs are
/// scalars — the source adapter emits field types through the `t_i32`/
/// `t_bool` typedefs).
fn type_is_record(m: &hir::Module, u: &hir::TypeUse) -> bool {
    match u {
        hir::TypeUse::Builtin(_) => false,
        hir::TypeUse::Ref(t) => matches!(
            m.types.get(t.0 as usize).map(|x| &x.kind),
            Some(hir::TypeKind::Record(_))
        ),
    }
}

/// Record-typed locals whose storage an expression subtree may read or hand
/// to a callee, indexed by expr id (F-771-R1-01). Copy elision forwards an
/// assignment destination into RHS evaluation; that is only sound when the
/// destination's root local does not appear here — otherwise the RHS could
/// observe (or a callee could interleave with) writes the elision performs
/// into storage that is still being read. `BorrowMut` counts too: a callee
/// writing through a mut argument touches the same storage.
fn record_mention_sets(body: &hir::Body, local_is_record: &[bool]) -> Vec<BTreeSet<u32>> {
    let mut sets: Vec<BTreeSet<u32>> = vec![BTreeSet::new(); body.exprs.len()];
    enum Step {
        Enter(hir::ExprId),
        Combine(hir::ExprId),
    }
    let mut roots: Vec<hir::ExprId> = Vec::new();
    for bl in &body.blocks {
        for s in &bl.stmts {
            match s {
                hir::Stmt::Let { value, .. } | hir::Stmt::Return { value } => roots.push(*value),
                hir::Stmt::If { cond, .. } => roots.push(*cond),
                hir::Stmt::Assign { value, .. } => roots.push(*value),
                hir::Stmt::Loop { .. }
                | hir::Stmt::Break { .. }
                | hir::Stmt::Continue { .. }
                | hir::Stmt::Drop { .. } => {}
            }
        }
    }
    for root in roots {
        // single ownership (verifier) makes the forest acyclic; the walk is
        // iterative so deep expressions cannot grow the emitter's stack
        let mut stack: Vec<Step> = vec![Step::Enter(root)];
        while let Some(step) = stack.pop() {
            match step {
                Step::Enter(id) => {
                    let Some(e) = body.exprs.get(id.0 as usize) else {
                        continue;
                    };
                    let mut children: Vec<hir::ExprId> = Vec::new();
                    match &e.kind {
                        hir::ExprKind::ReadLocal { local } => {
                            if local_is_record.get(local.0 as usize).copied() == Some(true) {
                                sets[id.0 as usize].insert(local.0);
                            }
                        }
                        hir::ExprKind::BorrowMut { local } => {
                            sets[id.0 as usize].insert(local.0);
                        }
                        hir::ExprKind::Constant { .. } => {}
                        hir::ExprKind::Binary { lhs, rhs, .. }
                        | hir::ExprKind::Logical { lhs, rhs, .. } => {
                            children.push(*lhs);
                            children.push(*rhs);
                        }
                        hir::ExprKind::Unary { operand, .. } => children.push(*operand),
                        hir::ExprKind::Call { eval_args, .. }
                        | hir::ExprKind::RecordCtor { eval_args, .. } => {
                            children.extend(eval_args.iter().copied());
                        }
                        hir::ExprKind::FieldGet { base, .. }
                        | hir::ExprKind::BorrowView { base } => {
                            children.push(*base);
                        }
                        // text kinds never appear in a record module (the
                        // verifier's profile defense); totality only
                        _ => {}
                    }
                    if !children.is_empty() {
                        stack.push(Step::Combine(id));
                        for c in children.into_iter().rev() {
                            stack.push(Step::Enter(c));
                        }
                    }
                }
                Step::Combine(id) => {
                    let mut union: BTreeSet<u32> = sets[id.0 as usize].clone();
                    let mut children: Vec<hir::ExprId> = Vec::new();
                    if let Some(e) = body.exprs.get(id.0 as usize) {
                        match &e.kind {
                            hir::ExprKind::Binary { lhs, rhs, .. }
                            | hir::ExprKind::Logical { lhs, rhs, .. } => {
                                children.push(*lhs);
                                children.push(*rhs);
                            }
                            hir::ExprKind::Unary { operand, .. } => children.push(*operand),
                            hir::ExprKind::Call { eval_args, .. }
                            | hir::ExprKind::RecordCtor { eval_args, .. } => {
                                children.extend(eval_args.iter().copied());
                            }
                            hir::ExprKind::FieldGet { base, .. }
                            | hir::ExprKind::BorrowView { base } => {
                                children.push(*base);
                            }
                            _ => {}
                        }
                    }
                    for c in children {
                        if let Some(s) = sets.get(c.0 as usize) {
                            union.extend(s.iter().copied());
                        }
                    }
                    sets[id.0 as usize] = union;
                }
            }
        }
    }
    sets
}

/// Record-typed `ReadLocal` expressions bound to record-valued constructor
/// fields, by expr id (F-771-R2-01). A record rvalue operand must be
/// CAPTURED at its own source-order evaluation point: deferring the field
/// copy to the ctor's combine step lets a later sibling argument (e.g.
/// `bump(q.mut)` in `P { a: q, b: bump(q.mut) }`) mutate the source storage
/// first. Every other record-producing operand already materializes at its
/// evaluation position (FieldGet copies, Calls own their out slot, nested
/// ctors own their temp); borrow expressions never reach ctor position
/// (verifier rule), so only plain local reads need the eager set.
fn record_eager_reads(body: &hir::Body, m: &hir::Module) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    for e in &body.exprs {
        if let hir::ExprKind::RecordCtor {
            record,
            eval_args,
            bindings,
        } = &e.kind
        {
            let shape = match m.types.get(record.0 as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::Record(sh)) => sh,
                _ => continue,
            };
            for b in bindings {
                let (Some(field), Some(arg)) = (
                    shape.fields.get(b.field as usize),
                    eval_args.get(b.arg as usize),
                ) else {
                    continue;
                };
                if type_is_record(m, &field.ty)
                    && matches!(
                        body.exprs.get(arg.0 as usize).map(|x| &x.kind),
                        Some(hir::ExprKind::ReadLocal { .. })
                    )
                {
                    out.insert(arg.0);
                }
            }
        }
    }
    out
}

/// Walk a field path from a root record type (PLAN-771): returns the byte
/// offset of the final field and, when that field is itself a record, its
/// type index (pointer storage); `None` in the second slot marks a scalar.
fn field_path_offset(
    m: &hir::Module,
    layouts: &BTreeMap<u32, u64>,
    root: u32,
    path: &[hir::FieldId],
) -> Option<(u64, Option<u32>)> {
    let field_size = |ty: &hir::TypeUse| -> Option<u64> {
        match ty {
            hir::TypeUse::Builtin(_) => Some(4),
            hir::TypeUse::Ref(t) => layouts.get(&t.0).copied().or(Some(4)),
        }
    };
    let mut off: u64 = 0;
    let mut cur = root;
    for (i, seg) in path.iter().enumerate() {
        let shape = match &m.types.get(cur as usize)?.kind {
            hir::TypeKind::Record(sh) => sh,
            _ => return None,
        };
        for f in shape.fields.iter().take(seg.0 as usize) {
            off = off.checked_add(field_size(&f.ty)?)?;
        }
        let field = shape.fields.get(seg.0 as usize)?;
        if type_is_record(m, &field.ty) {
            let hir::TypeUse::Ref(t) = &field.ty else {
                unreachable!("checked by type_is_record")
            };
            if i + 1 == path.len() {
                return Some((off, Some(t.0)));
            }
            cur = t.0;
        } else {
            // scalar field (builtin word or i32/bool typedef): must be last
            if i + 1 != path.len() {
                return None;
            }
            return Some((off, None));
        }
    }
    None
}

fn signature_of(m: &hir::Module, ty: hir::TypeId) -> Option<(Vec<Vt>, Vt)> {
    match &m.types.get(ty.0 as usize)?.kind {
        hir::TypeKind::Function(sig) => {
            let mut params = Vec::with_capacity(sig.params.len());
            for p in &sig.params {
                params.push(value_of_type_use(p, m)?);
            }
            Some((params, value_of_type_use(&sig.result, m)?))
        }
        _ => None,
    }
}

/// Locate the build entry by its textual def id. The entry must be a function
/// (not an intrinsic) with zero parameters returning i32 (plan §5.3).
pub fn find_entry(
    checked: &CheckedModule,
    entry_id_text: &str,
) -> Result<EntrySelection, Vec<Diagnostic>> {
    let bundle = checked.bundle();
    let mut found = Vec::new();
    for (mi, m) in bundle.modules.iter().enumerate() {
        for (di, d) in m.defs.iter().enumerate() {
            if d.id_text == entry_id_text {
                found.push((mi, di, d));
            }
        }
    }
    match found.len() {
        0 => Err(vec![Diagnostic::new(
            Stage::Backend,
            "entry.not-found",
            format!(
                "no declaration with id `{}` in this document",
                entry_id_text
            ),
            Span { start: 0, end: 0 },
        )]),
        1 => {
            let (mi, di, d) = &found[0];
            let m = &bundle.modules[*mi];
            let hir::DefKind::Function { name, ty, .. } = &d.kind else {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "entry.not-function",
                    format!(
                        "entry `{}` must be a function, not an intrinsic",
                        entry_id_text
                    ),
                    d.span,
                )]);
            };
            let Some((params, result)) = signature_of(m, *ty) else {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "entry.signature",
                    format!("entry `{}` does not have a function type", entry_id_text),
                    d.span,
                )]);
            };
            if !params.is_empty() || result != Vt::I32 {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "entry.signature",
                    format!(
                        "entry `{}` must be a zero-parameter function returning i32 (found {} params, result {:?})",
                        entry_id_text,
                        params.len(),
                        result
                    ),
                    d.span,
                )]);
            }
            Ok(EntrySelection {
                module: *mi,
                def: DefId(*di as u32),
                symbol: name.clone(),
            })
        }
        n => Err(vec![Diagnostic::new(
            Stage::Backend,
            "entry.ambiguous",
            format!(
                "declaration id `{}` occurs in {} modules; ids are module-local",
                entry_id_text, n
            ),
            Span { start: 0, end: 0 },
        )]),
    }
}

/// Lower the entry's module to a COFF object. The object defines every
/// function of the module and the generic `ac_start` wrapper. Text-domain
/// modules (PLAN-770) dispatch to the dedicated text lowerer below; the
/// scalar path is untouched.
pub fn lower_object(
    checked: &CheckedModule,
    sel: &EntrySelection,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    if matches!(
        checked.domain(),
        hir::HirDomain::Text | hir::HirDomain::TextRecord
    ) {
        return lower_text_object(checked, sel);
    }
    let module = &checked.bundle().modules[sel.module];

    // Symbol table: function name collisions are a backend error (readable
    // names are not identities elsewhere, but a COFF symbol table cannot
    // hold two entries with the same name).
    let mut seen_names: BTreeMap<&str, DefId> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        let decl_name = match &d.kind {
            hir::DefKind::Function { name, .. } | hir::DefKind::Method { name, .. } => {
                Some(name.as_str())
            }
            hir::DefKind::Intrinsic { .. } => None,
        };
        if let Some(name) = decl_name {
            if seen_names.insert(name, DefId(di as u32)).is_some() {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "backend.duplicate-symbol",
                    format!("function name `{}` occurs more than once in module `{}`; the native symbol table cannot represent both", name, module.id_text),
                    d.span,
                )]);
            }
        }
    }

    // --- set up the object module ---
    let triple = Triple::host();
    let mut flag_builder = cranelift_codegen::settings::builder();
    flag_builder.set("opt_level", "none").expect("opt_level");
    flag_builder.set("is_pic", "false").expect("is_pic");
    let flags = cranelift_codegen::settings::Flags::new(flag_builder);
    let isa = match cranelift_codegen::isa::lookup(triple) {
        Ok(b) => match b.finish(flags) {
            Ok(isa) => isa,
            Err(e) => return Err(vec![backend_diag(format!("isa finish failed: {}", e))]),
        },
        Err(e) => return Err(vec![backend_diag(format!("host isa unavailable: {}", e))]),
    };
    let obj_builder = match ObjectBuilder::new(
        isa,
        module.path.as_deref().unwrap_or("ac_module"),
        default_libcall_names(),
    ) {
        Ok(b) => b,
        Err(e) => return Err(vec![backend_diag(format!("object builder failed: {}", e))]),
    };
    let mut om: ObjectModule = ObjectModule::new(obj_builder);

    // ExitProcess import (startup wrapper + trap path).
    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = match om.declare_function("ExitProcess", Linkage::Import, &exit_sig) {
        Ok(id) => id,
        Err(e) => return Err(vec![backend_diag(format!("declare ExitProcess: {}", e))]),
    };

    // Pre-declare every def (functions exported, intrinsics imported by
    // symbol) so recursive DefRef calls resolve without body duplication.
    let mut mod_func_ids: BTreeMap<u32, cranelift_module::FuncId> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        let (sym, linkage, ty) = match &d.kind {
            hir::DefKind::Function { name, ty, .. } => (name.as_str(), Linkage::Export, *ty),
            // methods export under their internal symbol (`sm_...` from the
            // source adapter / author-given name for hand-written HIR)
            hir::DefKind::Method { name, ty, .. } => (name.as_str(), Linkage::Export, *ty),
            hir::DefKind::Intrinsic { symbol, ty, .. } => (symbol.as_str(), Linkage::Import, *ty),
        };
        let Some((params, result)) = signature_of(module, ty) else {
            return Err(vec![backend_diag(format!(
                "declaration `{}` has no function type",
                d.id_text
            ))]);
        };
        // ABI-E out slot applies to the DECLARED signature too (call sites
        // append the out pointer; the define site below mirrors this)
        let layouts0 = record_layouts(module);
        let mut machine_params0 = params.clone();
        if let Vt::Record(tid) = result {
            if layouts0.contains_key(&tid.0) {
                machine_params0.push(Vt::Record(tid));
            }
        }
        let sig = cranelift_sig(&machine_params0, result);
        let fid = match om.declare_function(sym, linkage, &sig) {
            Ok(id) => id,
            Err(e) => return Err(vec![backend_diag(format!("declare {}: {}", sym, e))]),
        };
        mod_func_ids.insert(di as u32, fid);
    }

    // Define each function body.
    for (di, d) in module.defs.iter().enumerate() {
        let (ty, body) = match &d.kind {
            hir::DefKind::Function { ty, body, .. } | hir::DefKind::Method { ty, body, .. } => {
                (*ty, *body)
            }
            hir::DefKind::Intrinsic { .. } => continue,
        };
        let Some((params, result)) = signature_of(module, ty) else {
            continue;
        };
        let body = &module.bodies[body.0 as usize];
        let layouts = record_layouts(module);
        // ABI-E: a record result becomes a trailing out pointer param.
        let mut machine_params = params.clone();
        if let Vt::Record(tid) = result {
            if layouts.contains_key(&tid.0) {
                machine_params.push(Vt::Record(tid)); // lowers to I64
            }
        }
        let mut ctx = om.make_context();
        ctx.func.signature = cranelift_sig(&machine_params, result);
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let entry_block = fb.create_block();
            fb.append_block_params_for_function_params(entry_block);
            fb.switch_to_block(entry_block);
            fb.seal_block(entry_block);

            let mut fnrefs: BTreeMap<u32, FuncRef> = BTreeMap::new();
            for (dj, _) in module.defs.iter().enumerate() {
                let fid = mod_func_ids[&(dj as u32)];
                fnrefs.insert(dj as u32, om.declare_func_in_func(fid, &mut fb.func));
            }
            let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);

            // Locals → Cranelift variables. Record-typed locals hold an
            // I64 pointer to their storage (774 D-05 ABI-D/E); scalars are
            // i32 values.
            let layouts = record_layouts(module);
            let is_rec_vt =
                |u: &hir::TypeUse| matches!(value_of_type_use(u, module), Some(Vt::Record(_)));
            let local_is_record: Vec<bool> = body.locals.iter().map(|l| is_rec_vt(&l.ty)).collect();
            let mentions = record_mention_sets(body, &local_is_record);
            let mut vars: BTreeMap<u32, Variable> = BTreeMap::new();
            for (li, rec) in local_is_record.iter().enumerate() {
                let v = if *rec {
                    fb.declare_var(types::I64)
                } else {
                    fb.declare_var(types::I32)
                };
                vars.insert(li as u32, v);
            }
            // ABI-E out-slot: a record result becomes a trailing pointer
            // param; the machine return is void.
            let sig_out = match &module.types[ty.0 as usize].kind {
                hir::TypeKind::Function(sig) => {
                    matches!(value_of_type_use(&sig.result, module), Some(Vt::Record(_)))
                }
                _ => unreachable!("signature checked above"),
            };
            // Params start defined: entry block params are the signature
            // values, in signature order (the ABI-E out slot, when present,
            // is the LAST machine param and belongs to no body local).
            let out_param_value = if sig_out {
                let n = fb.block_params(entry_block).len();
                Some(fb.block_params(entry_block)[n - 1])
            } else {
                None
            };
            let result_bytes = if sig_out {
                match &module.types[ty.0 as usize].kind {
                    hir::TypeKind::Function(sig) => match &sig.result {
                        hir::TypeUse::Ref(t) => layouts.get(&t.0).copied(),
                        _ => None,
                    },
                    _ => None,
                }
            } else {
                None
            };
            for (li, l) in body.locals.iter().enumerate() {
                if l.role == hir::LocalRole::Param {
                    let idx = l.index.expect("verifier: param has index") as usize;
                    let n_block_params = fb.block_params(entry_block).len();
                    if idx >= n_block_params {
                        return Err(vec![backend_diag(format!(
                            "body `{}`: param index {} out of range for function `{}` with {} signature params",
                            body.id_text, idx, d.id_text, n_block_params
                        ))]);
                    }
                    let var = vars[&(li as u32)];
                    let pv = fb.block_params(entry_block)[idx];
                    fb.def_var(var, pv);
                }
            }

            let mut fx = FuncEmitter {
                fb,
                body,
                module,
                layouts,
                local_is_record,
                mentions,
                eager_reads: record_eager_reads(body, module),
                out_param: out_param_value,
                result_bytes,
                dests: BTreeMap::new(),
                elided: std::collections::BTreeSet::new(),
                vars,
                fnrefs,
                exit_ref,
                trap_block: None,
                loops: Vec::new(),
            };
            fx.emit_block(body.entry);
            // Termination: the entry block ended in `return` (verifier).
            let trap = fx.ensure_trap_block();
            fx.fb.switch_to_block(trap);
            fx.fb.seal_block(trap);
            let code = fx.fb.ins().iconst(types::I32, TRAP_EXIT_CODE as i64);
            let call = fx.fb.ins().call(fx.exit_ref, &[code]);
            let _ = call;
            fx.fb
                .ins()
                .trap(cranelift_codegen::ir::TrapCode::INTEGER_OVERFLOW); // after ExitProcess; never reached
            fx.fb.finalize();
        }
        if let Err(e) = om.define_function(mod_func_ids[&(di as u32)], &mut ctx) {
            return Err(vec![backend_diag(format!("define {}: {}", d.id_text, e))]);
        }
    }

    // Generic startup wrapper: call the entry, ExitProcess(result).
    {
        let start_sig = Signature {
            call_conv: WINCALL,
            params: vec![],
            returns: vec![],
        };
        let start_id = match om.declare_function(START_SYMBOL, Linkage::Export, &start_sig) {
            Ok(id) => id,
            Err(e) => {
                return Err(vec![backend_diag(format!(
                    "declare {}: {}",
                    START_SYMBOL, e
                ))])
            }
        };
        let mut ctx = om.make_context();
        ctx.func.signature = start_sig;
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let blk = fb.create_block();
            fb.switch_to_block(blk);
            fb.seal_block(blk);
            let entry_ref = om.declare_func_in_func(mod_func_ids[&sel.def.0], &mut fb.func);
            let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);
            let call = fb.ins().call(entry_ref, &[]);
            let result = fb.inst_results(call)[0];
            fb.ins().call(exit_ref, &[result]);
            fb.ins().return_(&[]);
            fb.finalize();
        }
        if let Err(e) = om.define_function(start_id, &mut ctx) {
            return Err(vec![backend_diag(format!(
                "define {}: {}",
                START_SYMBOL, e
            ))]);
        }
    }

    match om.finish().emit() {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(vec![backend_diag(format!("emit object: {}", e))]),
    }
}

fn cranelift_sig(params: &[Vt], result: Vt) -> Signature {
    let t = |v: &Vt| {
        AbiParam::new(match v {
            Vt::Bool | Vt::I32 => types::I32, // bool is an i32 0/1 internally
            Vt::Function => types::I32,       // unreachable per verifier
            // PLAN-771 (774 D-05): records cross as pointers (ABI-D/E).
            Vt::Record(_) => types::I64,
        })
    };
    Signature {
        call_conv: WINCALL,
        params: params.iter().map(&t).collect(),
        returns: if matches!(result, Vt::I32 | Vt::Bool) {
            vec![t(&result)]
        } else {
            vec![]
        },
    }
}

fn backend_diag(msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        Stage::Backend,
        "backend.lowering",
        msg,
        Span { start: 0, end: 0 },
    )
}

/// A copy-elision destination: the storage an expression writes into
/// directly, plus the local whose storage that is (F-771-R1-01). `root` is
/// `None` for freshly allocated slots (unreachable from any other
/// expression) and for the ABI-E out param — the out param is disjoint
/// from every argument storage by the call-site forwarding rule, so
/// writing it during a callee's evaluation never clobbers a live input.
#[derive(Clone, Copy)]
struct Dest {
    ptr: Value,
    root: Option<u32>,
}

/// Per-function emission state.
struct FuncEmitter<'c> {
    fb: FunctionBuilder<'c>,
    body: &'c hir::Body,
    module: &'c hir::Module,
    /// Record byte sizes by type index (PLAN-771).
    layouts: BTreeMap<u32, u64>,
    /// Machine type of each body local by index: `false` = i32 value,
    /// `true` = record (an I64 pointer to storage).
    local_is_record: Vec<bool>,
    /// The out-slot block param for record-returning functions.
    out_param: Option<Value>,
    /// The record result's byte size (when `out_param` is set).
    result_bytes: Option<u64>,
    /// Copy-elision destinations (PLAN-771 V-09): the storage pointer a
    /// record-producing expression should write into directly, instead of
    /// allocating a fresh temp. Keyed by expr id; each expr has exactly one
    /// consuming position (verifier), so the entry is consumed on use.
    dests: BTreeMap<u32, Dest>,
    /// Record-typed locals touched by each expression subtree (per expr
    /// id); the insert-side disjointness gate for `dests` (F-771-R1-01).
    mentions: Vec<BTreeSet<u32>>,
    /// Record-typed `ReadLocal` exprs bound to record ctor fields
    /// (F-771-R2-01): value operands that must snapshot at their own
    /// evaluation position instead of passing a live storage pointer.
    eager_reads: BTreeSet<u32>,
    /// Expr ids that WROTE into their destination directly (their parent
    /// store step must not re-copy them).
    elided: std::collections::BTreeSet<u32>,
    vars: BTreeMap<u32, Variable>,
    fnrefs: BTreeMap<u32, FuncRef>,
    exit_ref: FuncRef,
    trap_block: Option<cranelift_codegen::ir::Block>,
    /// HIR loop nesting: (LoopId, header block, after block).
    loops: Vec<(
        LoopId,
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    )>,
}

impl<'c> FuncEmitter<'c> {
    /// Fresh stack slot of `bytes` bytes (8-aligned) and its address.
    fn new_slot(&mut self, bytes: u64) -> Value {
        use cranelift_codegen::ir::StackSlotData;
        let slot = self.fb.create_sized_stack_slot(StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            bytes.max(4) as u32,
            3,
        ));
        self.fb.ins().stack_addr(types::I64, slot, 0)
    }

    /// Inline byte copy `dst <- src` (records are 4-multiples; 8-byte words
    /// then a 4-byte tail). Self-contained — no libc dependency (774 D-05
    /// internal ABI; copies are value semantics, never aliasing).
    fn emit_copy(&mut self, dst: Value, src: Value, bytes: u64) {
        let flags = cranelift_codegen::ir::MemFlags::trusted();
        let mut off = 0u64;
        while off + 8 <= bytes {
            let v = self.fb.ins().load(types::I64, flags, src, off as i32);
            self.fb.ins().store(flags, v, dst, off as i32);
            off += 8;
        }
        if off + 4 <= bytes {
            let v = self.fb.ins().load(types::I32, flags, src, off as i32);
            self.fb.ins().store(flags, v, dst, off as i32);
        }
    }

    /// Whether def `di`'s function type returns a record (and so takes the
    /// ABI-E out slot at call sites).
    fn def_result_is_record(&self, di: u32) -> Option<u32> {
        let d = self.module.defs.get(di as usize)?;
        let ty = match &d.kind {
            hir::DefKind::Function { ty, .. }
            | hir::DefKind::Method { ty, .. }
            | hir::DefKind::Intrinsic { ty, .. } => *ty,
        };
        match &self.module.types.get(ty.0 as usize)?.kind {
            hir::TypeKind::Function(sig) => match &sig.result {
                hir::TypeUse::Ref(t) => {
                    if matches!(
                        self.module.types.get(t.0 as usize).map(|x| &x.kind),
                        Some(hir::TypeKind::Record(_))
                    ) {
                        Some(t.0)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn ensure_trap_block(&mut self) -> cranelift_codegen::ir::Block {
        *self
            .trap_block
            .get_or_insert_with(|| self.fb.create_block())
    }

    /// Whether a destination rooted at `root` may be forwarded into the
    /// evaluation of `eid` (F-771-R1-01): the subtree must not touch the
    /// root local's storage, or an interleaved read (in this expression or
    /// inside a callee reading a borrowed argument) would observe the
    /// destination writes. `None` roots — fresh slots, the ABI-E out
    /// param — never alias reachable storage and are always safe.
    fn forward_safe(&self, root: Option<u32>, eid: hir::ExprId) -> bool {
        match root {
            None => true,
            Some(local) => !self
                .mentions
                .get(eid.0 as usize)
                .is_some_and(|touched| touched.contains(&local)),
        }
    }

    /// Emit one block with an explicit frame stack (R6: block nesting is
    /// unlimited, so no recursion). `terminated` is per-frame; a child
    /// frame's role (then-arm / else-arm / loop-body) is carried on the
    /// frame itself and applied when it completes, so the parent resumes
    /// with the right block switched to and the right `terminated` — the
    /// same creation/seal/switch order as the previous recursive form.
    fn emit_block(&mut self, entry: hir::BlockId) {
        struct EFrame {
            stmts: Vec<hir::Stmt>,
            idx: usize,
            terminated: bool,
            role: ERole,
        }
        enum ERole {
            /// The function's entry block.
            Entry,
            /// Then-arm of an if that has an else.
            ThenWithElse {
                else_id: hir::BlockId,
                else_b: cranelift_codegen::ir::Block,
                end_b: cranelift_codegen::ir::Block,
            },
            /// Then-arm of an if without else.
            ThenNoElse { end_b: cranelift_codegen::ir::Block },
            /// The else-arm of an if.
            ElseArm {
                end_b: cranelift_codegen::ir::Block,
                then_terminated: bool,
            },
            /// The body of a loop.
            LoopBody {
                body_b: cranelift_codegen::ir::Block,
                after: cranelift_codegen::ir::Block,
            },
        }
        let mut frames: Vec<EFrame> = vec![EFrame {
            stmts: self.body.blocks[entry.0 as usize].stmts.clone(),
            idx: 0,
            terminated: false,
            role: ERole::Entry,
        }];
        while let Some(frame) = frames.last_mut() {
            let Some(stmt) = frame.stmts.get(frame.idx).cloned() else {
                let frame = frames.pop().expect("emit frame");
                match frame.role {
                    ERole::Entry => {}
                    ERole::ThenWithElse {
                        else_id,
                        else_b,
                        end_b,
                    } => {
                        let then_terminated = frame.terminated;
                        if !then_terminated {
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.switch_to_block(else_b);
                        frames.push(EFrame {
                            stmts: self.body.blocks[else_id.0 as usize].stmts.clone(),
                            idx: 0,
                            terminated: false,
                            role: ERole::ElseArm {
                                end_b,
                                then_terminated,
                            },
                        });
                    }
                    ERole::ThenNoElse { end_b } => {
                        let t = frame.terminated;
                        if !t {
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.seal_block(end_b);
                        self.fb.switch_to_block(end_b);
                        frames.last_mut().expect("emit frame").terminated = false;
                    }
                    ERole::ElseArm {
                        end_b,
                        then_terminated,
                    } => {
                        let else_terminated = frame.terminated;
                        if !else_terminated {
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.seal_block(end_b);
                        if then_terminated && else_terminated {
                            // both branches transferred; the verifier
                            // guarantees no executable statements follow
                            frames.last_mut().expect("emit frame").terminated = true;
                        } else {
                            self.fb.switch_to_block(end_b);
                            frames.last_mut().expect("emit frame").terminated = false;
                        }
                    }
                    ERole::LoopBody { body_b, after } => {
                        // Fall-through of the loop body repeats the loop.
                        let t = frame.terminated;
                        if !t {
                            self.fb.ins().jump(body_b, &[]);
                        }
                        self.fb.seal_block(body_b);
                        self.fb.switch_to_block(after);
                        self.fb.seal_block(after);
                        self.loops.pop();
                        frames.last_mut().expect("emit frame").terminated = false;
                    }
                }
                continue;
            };
            frame.idx += 1;
            match stmt {
                hir::Stmt::Let { local, value } => {
                    if self.local_is_record.get(local.0 as usize).copied() == Some(true) {
                        // value semantics: the binding owns a fresh copy —
                        // a record-producing initializer writes straight
                        // into the binding's slot (copy elision; the slot
                        // is fresh, so no aliasing is possible)
                        let ty_idx = match self.body.locals[local.0 as usize].ty {
                            hir::TypeUse::Ref(t) => t.0,
                            _ => 0,
                        };
                        let bytes = self.layouts.get(&ty_idx).copied().unwrap_or(4);
                        let slot = self.new_slot(bytes);
                        self.dests.insert(
                            value.0,
                            Dest {
                                ptr: slot,
                                root: None,
                            },
                        );
                        let v = self.eval_expr(value);
                        self.emit_copy(slot, v, bytes);
                        let var = self.vars[&local.0];
                        self.fb.def_var(var, slot);
                    } else {
                        let v = self.eval_expr(value);
                        let var = self.vars[&local.0];
                        self.fb.def_var(var, v);
                    }
                }
                hir::Stmt::Return { value } => {
                    if let (Some(out), Some(bytes)) = (self.out_param, self.result_bytes) {
                        // record result: write into the ABI-E out slot
                        // (elided straight through ctor/call results),
                        // void machine return. The out param is disjoint
                        // from every argument storage by the call-site
                        // forwarding rule (see Step::Call below).
                        self.dests.insert(
                            value.0,
                            Dest {
                                ptr: out,
                                root: None,
                            },
                        );
                        let v = self.eval_expr(value);
                        self.emit_copy(out, v, bytes);
                        self.fb.ins().return_(&[]);
                    } else {
                        let v = self.eval_expr(value);
                        self.fb.ins().return_(&[v]);
                    }
                    frame.terminated = true;
                }
                hir::Stmt::If { cond, then, els } => {
                    let c = self.eval_expr(cond);
                    let then_b = self.fb.create_block();
                    match els {
                        Some(e) => {
                            // Join block created eagerly; it is always sealed
                            // (cranelift allows zero-predecessor sealing) and
                            // only entered when a branch fell through.
                            let end_b = self.fb.create_block();
                            let else_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], else_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.seal_block(else_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                stmts: self.body.blocks[then.0 as usize].stmts.clone(),
                                idx: 0,
                                terminated: false,
                                role: ERole::ThenWithElse {
                                    else_id: e,
                                    else_b,
                                    end_b,
                                },
                            });
                        }
                        None => {
                            // Without else, the false edge of the branch IS
                            // the join block, so it always has a predecessor.
                            let end_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], end_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                stmts: self.body.blocks[then.0 as usize].stmts.clone(),
                                idx: 0,
                                terminated: false,
                                role: ERole::ThenNoElse { end_b },
                            });
                        }
                    }
                }
                hir::Stmt::Loop { id, body } => {
                    // This profile's `loop` has no condition header: the body
                    // repeats until it breaks; `continue` jumps straight back
                    // to the body. `after` receives the break edges.
                    let after = self.fb.create_block();
                    let body_b = self.fb.create_block();
                    self.fb.ins().jump(body_b, &[]);
                    self.loops.push((id, body_b, after));
                    self.fb.switch_to_block(body_b);
                    frames.push(EFrame {
                        stmts: self.body.blocks[body.0 as usize].stmts.clone(),
                        idx: 0,
                        terminated: false,
                        role: ERole::LoopBody { body_b, after },
                    });
                }
                hir::Stmt::Assign { place, value } => {
                    let p = &self.body.places[place.0 as usize];
                    match &p.kind {
                        hir::PlaceKind::Local => {
                            if self.local_is_record.get(p.local.0 as usize).copied() == Some(true) {
                                // whole-record assign = value copy into the
                                // local's own slot (elided for ctors — but
                                // only when the RHS does not read storage
                                // rooted at the same local: F-771-R1-01,
                                // `p = P { a: p.b, b: p.a }` /
                                // `p = swap(p)` must materialize the RHS
                                // first and commit afterwards)
                                let dst = self.fb.use_var(self.vars[&p.local.0]);
                                let ty_idx = match self.body.locals[p.local.0 as usize].ty {
                                    hir::TypeUse::Ref(t) => t.0,
                                    _ => 0,
                                };
                                let bytes = self.layouts.get(&ty_idx).copied().unwrap_or(4);
                                if self.forward_safe(Some(p.local.0), value) {
                                    self.dests.insert(
                                        value.0,
                                        Dest {
                                            ptr: dst,
                                            root: Some(p.local.0),
                                        },
                                    );
                                }
                                let v = self.eval_expr(value);
                                self.emit_copy(dst, v, bytes);
                            } else {
                                let v = self.eval_expr(value);
                                let var = self.vars[&p.local.0];
                                self.fb.def_var(var, v);
                            }
                        }
                        hir::PlaceKind::FieldPath(path) => {
                            // address into the root local's storage
                            let base = self.fb.use_var(self.vars[&p.local.0]);
                            let root_idx = match self.body.locals[p.local.0 as usize].ty {
                                hir::TypeUse::Ref(t) => t.0,
                                _ => 0,
                            };
                            let Some((off, field_rec)) =
                                field_path_offset(self.module, &self.layouts, root_idx, path)
                            else {
                                unreachable!("verifier: field path resolves");
                            };
                            let addr = self.fb.ins().iadd_imm(base, off as i64);
                            match field_rec {
                                None => {
                                    let v = self.eval_expr(value);
                                    self.fb.ins().store(
                                        cranelift_codegen::ir::MemFlags::trusted(),
                                        v,
                                        addr,
                                        0,
                                    );
                                }
                                Some(rt) => {
                                    let bytes = self.layouts.get(&rt).copied().unwrap_or(4);
                                    // F-771-R1-01: forward the field address
                                    // only when the RHS does not touch the
                                    // root local (`p.a = P { a: p.a.b, ... }`
                                    // materializes first, then commits)
                                    if self.forward_safe(Some(p.local.0), value) {
                                        self.dests.insert(
                                            value.0,
                                            Dest {
                                                ptr: addr,
                                                root: Some(p.local.0),
                                            },
                                        );
                                    }
                                    let v = self.eval_expr(value);
                                    self.emit_copy(addr, v, bytes);
                                }
                            }
                        }
                    }
                }
                hir::Stmt::Break { target } => {
                    let (_, _, after) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(after, &[]);
                    frame.terminated = true;
                }
                hir::Stmt::Continue { target } => {
                    let (_, header, _) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(header, &[]);
                    frame.terminated = true;
                }
                // Text-profile statements cannot reach the scalar lowering:
                // the verifier's profile gate rejects them outside the text
                // domain, and the text lowering path (PLAN-770 T-05) owns
                // their emission.
                hir::Stmt::Drop { .. } => {
                    unreachable!("verifier: drop statements exist only in the text profile")
                }
            }
        }
    }

    fn eval_expr(&mut self, eid: ExprId) -> Value {
        // Iterative post-order emission (PLAN-745 R3-QA-02): operator
        // chains within one statement are unbounded, so the emitter must
        // not recurse. Ordering is exactly the recursive form's: lhs fully
        // emitted, then rhs, then the node's own instruction (the explicit
        // stack yields operands left-to-right before the Exit step).
        // PLAN-767: `Logical` nodes add two steps — after the lhs value is
        // ready the machine opens a conditional rhs block + merge (branch on
        // the lhs under the operator's polarity), emits the ENTIRE rhs
        // subtree inside that block only, then merges through a Cranelift
        // variable. The rhs's potentially trapping code is never on an
        // unconditional path.
        enum Step {
            Enter(ExprId),
            Binary {
                dest: ExprId,
                op: hir::BinOp,
                lhs: ExprId,
                rhs: ExprId,
            },
            Unary {
                dest: ExprId,
                op: hir::UnaryOp,
                operand: ExprId,
            },
            /// lhs value ready: define the result variable with it, branch
            /// on it (and: rhs|merge, or: merge|rhs), switch to the rhs
            /// block. `merge`/`var` were created at Enter time.
            LogicalLhs {
                op: hir::LogicOp,
                lhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
            },
            /// rhs value ready: define the result variable with it, jump to
            /// the merge, seal it, switch to it, read the variable.
            LogicalRhs {
                dest: ExprId,
                rhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
            },
            Call(ExprId),
            /// Record-construction combiner: all field values evaluated in
            /// source order; write them into a fresh temp (PLAN-771).
            CtorOut(ExprId),
            /// Field-read combiner: base pointer ready; load/copy.
            FieldOut(ExprId),
            /// View-borrow combiner: base pointer ready; pass through.
            BorrowOut(ExprId),
        }
        enum Val {
            V(Value),
        }
        // value slots per expr id (an expr is emitted exactly once — the
        // verifier guarantees single ownership)
        let mut slots: std::collections::BTreeMap<u32, Val> = std::collections::BTreeMap::new();
        let mut stack: Vec<Step> = vec![Step::Enter(eid)];
        while let Some(step) = stack.pop() {
            match step {
                Step::Enter(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    match &e.kind {
                        hir::ExprKind::Constant { value } => {
                            let v = self.fb.ins().iconst(types::I32, *value as i64);
                            slots.insert(id.0, Val::V(v));
                        }
                        hir::ExprKind::ReadLocal { local } => {
                            let var = self.vars[&local.0];
                            let ptr = self.fb.use_var(var);
                            // A record local read is a VALUE operand when a
                            // destination is registered for it or it feeds a
                            // record ctor field: capture the bytes NOW, at
                            // this operand's source-order evaluation point —
                            // later sibling expressions (a `.mut` call arg)
                            // may mutate the source storage before the ctor
                            // combines its fields (F-771-R2-01). Without a
                            // value context the pointer passes through (the
                            // base of a view/mut borrow keeps borrow
                            // semantics).
                            let is_value =
                                self.dests.get(&id.0).is_some() || self.eager_reads.contains(&id.0);
                            let v = if is_value {
                                let ty_idx = match self.body.locals[local.0 as usize].ty {
                                    hir::TypeUse::Ref(t) => t.0,
                                    _ => 0,
                                };
                                let bytes = self.layouts.get(&ty_idx).copied().unwrap_or(4);
                                let used_dest = self.dests.remove(&id.0).map(|d| d.ptr);
                                let dst = match used_dest {
                                    // value written straight into the
                                    // registered destination — the parent's
                                    // store step must not re-copy it
                                    Some(d) => {
                                        self.elided.insert(id.0);
                                        d
                                    }
                                    // fresh snapshot: the ctor's combine
                                    // step still copies it into its field
                                    None => self.new_slot(bytes),
                                };
                                self.emit_copy(dst, ptr, bytes);
                                dst
                            } else {
                                ptr
                            };
                            slots.insert(id.0, Val::V(v));
                        }
                        hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                            stack.push(Step::Binary {
                                dest: id,
                                op: *op,
                                lhs: *lhs,
                                rhs: *rhs,
                            });
                            stack.push(Step::Enter(*rhs));
                            stack.push(Step::Enter(*lhs));
                        }
                        hir::ExprKind::Unary { op, operand } => {
                            stack.push(Step::Unary {
                                dest: id,
                                op: *op,
                                operand: *operand,
                            });
                            stack.push(Step::Enter(*operand));
                        }
                        hir::ExprKind::Logical { op, lhs, rhs } => {
                            let var = self.fb.declare_var(types::I32);
                            let merge = self.fb.create_block();
                            stack.push(Step::LogicalRhs {
                                dest: id,
                                rhs: *rhs,
                                var,
                                merge,
                            });
                            stack.push(Step::Enter(*rhs));
                            stack.push(Step::LogicalLhs {
                                op: *op,
                                lhs: *lhs,
                                var,
                                merge,
                            });
                            stack.push(Step::Enter(*lhs));
                        }
                        hir::ExprKind::Call { eval_args, .. } => {
                            stack.push(Step::Call(id));
                            for a in eval_args.iter().rev() {
                                stack.push(Step::Enter(*a));
                            }
                        }
                        // Text expressions cannot reach the scalar lowering
                        // (verifier profile gate); the text path (PLAN-770
                        // T-05) owns their emission. The PLAN-782
                        // combination kinds are likewise text-record only
                        // (its merged lowering lands in 782 T-05).
                        hir::ExprKind::StrLit { .. }
                        | hir::ExprKind::CharLit { .. }
                        | hir::ExprKind::TextLen { .. }
                        | hir::ExprKind::TextLenBytes { .. }
                        | hir::ExprKind::TextCharAt { .. }
                        | hir::ExprKind::TextSlice { .. }
                        | hir::ExprKind::TextConcat { .. }
                        | hir::ExprKind::TextClone { .. }
                        | hir::ExprKind::TextToOwned { .. }
                        | hir::ExprKind::TextToBorrowed { .. }
                        | hir::ExprKind::CharToCodepoint { .. }
                        | hir::ExprKind::CharFromCodepoint { .. } => {
                            unreachable!(
                                "verifier: text expressions exist only in the text profile"
                            );
                        }
                        hir::ExprKind::RecordClone { .. } | hir::ExprKind::Move { .. } => {
                            unreachable!(
                                "verifier: record_clone/move expressions exist only in the text-record profile"
                            );
                        }
                        hir::ExprKind::RecordCtor {
                            record,
                            eval_args,
                            bindings,
                        } => {
                            // source evaluation order (eval_args order);
                            // record-typed field args whose field offset is
                            // statically known thread the destination down
                            // (copy elision — no temp for nested ctors) —
                            // gated per child on disjointness from the
                            // destination's root local (F-771-R1-01)
                            let my_dest: Option<(Value, Option<u32>)> =
                                self.dests.get(&id.0).map(|d| (d.ptr, d.root));
                            if let Some((dst, dst_root)) = my_dest {
                                let shape = match &self.module.types[record.0 as usize].kind {
                                    hir::TypeKind::Record(sh) => sh,
                                    _ => unreachable!("ctor expr names a record"),
                                };
                                for b in bindings {
                                    if let Some(field) = shape.fields.get(b.field as usize) {
                                        if type_is_record(self.module, &field.ty) {
                                            let mut off: u64 = 0;
                                            for prior in shape.fields.iter().take(b.field as usize)
                                            {
                                                off += match &prior.ty {
                                                    hir::TypeUse::Builtin(_) => 4,
                                                    hir::TypeUse::Ref(t) => {
                                                        self.layouts.get(&t.0).copied().unwrap_or(4)
                                                    }
                                                };
                                            }
                                            if let Some(arg) = eval_args.get(b.arg as usize) {
                                                // only record-PRODUCING
                                                // children can write into
                                                // the destination — a
                                                // plain value (read, load)
                                                // still needs its copy;
                                                // a ReadLocal bound to a
                                                // record field captures its
                                                // value at its own
                                                // evaluation position, so
                                                // it counts as producing
                                                // (F-771-R2-01)
                                                let produces = matches!(
                                                    &self.body.exprs[arg.0 as usize].kind,
                                                    hir::ExprKind::RecordCtor { .. }
                                                        | hir::ExprKind::FieldGet { .. }
                                                        | hir::ExprKind::ReadLocal { .. }
                                                ) || self
                                                    .def_result_is_record(
                                                        match &self.body.exprs[arg.0 as usize].kind
                                                        {
                                                            hir::ExprKind::Call {
                                                                callee, ..
                                                            } => callee.0,
                                                            _ => u32::MAX,
                                                        },
                                                    )
                                                    .is_some();
                                                if produces && self.forward_safe(dst_root, *arg) {
                                                    let child_dst =
                                                        self.fb.ins().iadd_imm(dst, off as i64);
                                                    self.dests.insert(
                                                        arg.0,
                                                        Dest {
                                                            ptr: child_dst,
                                                            root: dst_root,
                                                        },
                                                    );
                                                    self.elided.insert(arg.0);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            stack.push(Step::CtorOut(id));
                            for a in eval_args.iter().rev() {
                                stack.push(Step::Enter(*a));
                            }
                        }
                        hir::ExprKind::FieldGet { base, .. } => {
                            stack.push(Step::FieldOut(id));
                            stack.push(Step::Enter(*base));
                        }
                        hir::ExprKind::BorrowView { base } => {
                            stack.push(Step::BorrowOut(id));
                            stack.push(Step::Enter(*base));
                        }
                        hir::ExprKind::BorrowMut { local } => {
                            // leaf: the local's storage pointer
                            let var = self.vars[&local.0];
                            let pv = self.fb.use_var(var);
                            slots.insert(id.0, Val::V(pv));
                        } // PLAN-771: RecordCtor/FieldGet/BorrowView carry
                          // children and are handled above; BorrowMut is a
                          // leaf handled above too — nothing falls through.
                    }
                }
                Step::Binary { dest, op, lhs, rhs } => {
                    let a = match slots.remove(&lhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("lhs slot filled"),
                    };
                    let b = match slots.remove(&rhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("rhs slot filled"),
                    };
                    let v = match op {
                        hir::BinOp::AddI32 => self.checked_add(a, b),
                        hir::BinOp::SubI32 => self.checked_sub(a, b),
                        hir::BinOp::MulI32 => self.checked_mul(a, b),
                        hir::BinOp::DivI32 => self.guarded_sdiv(a, b),
                        hir::BinOp::RemI32 => self.guarded_srem(a, b),
                        hir::BinOp::LtI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedLessThan, a, b)
                        }
                        hir::BinOp::LeI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedLessThanOrEqual, a, b)
                        }
                        hir::BinOp::GtI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedGreaterThan, a, b)
                        }
                        hir::BinOp::GeI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedGreaterThanOrEqual, a, b)
                        }
                        hir::BinOp::Eq => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::Equal, a, b)
                        }
                        hir::BinOp::Ne => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::NotEqual, a, b)
                        }
                    };
                    slots.insert(dest.0, Val::V(v));
                }
                Step::CtorOut(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::RecordCtor {
                        record, bindings, ..
                    } = &e.kind
                    else {
                        unreachable!("ctor step on ctor expr")
                    };
                    let bytes = self.layouts.get(&record.0).copied().unwrap_or(4);
                    let temp = self
                        .dests
                        .remove(&id.0)
                        .map(|d| d.ptr)
                        .unwrap_or_else(|| self.new_slot(bytes));
                    let mut field_tys: Vec<hir::TypeUse> = Vec::new();
                    if let Some(hir::TypeKind::Record(sh)) =
                        self.module.types.get(record.0 as usize).map(|t| &t.kind)
                    {
                        field_tys.extend(sh.fields.iter().map(|f| f.ty));
                    }
                    // offsets from prior field sizes (declaration order)
                    let mut offs: Vec<u64> = Vec::with_capacity(field_tys.len());
                    let mut acc: u64 = 0;
                    for ft in &field_tys {
                        offs.push(acc);
                        acc = acc
                            .checked_add(match ft {
                                hir::TypeUse::Builtin(_) => 4,
                                hir::TypeUse::Ref(t) => {
                                    self.layouts.get(&t.0).copied().unwrap_or(4)
                                }
                            })
                            .unwrap_or(u32::MAX as u64);
                    }
                    let eval_ids: Vec<u32> = match &e.kind {
                        hir::ExprKind::RecordCtor { eval_args, .. } => {
                            eval_args.iter().map(|a| a.0).collect()
                        }
                        _ => unreachable!("ctor step on ctor expr"),
                    };
                    for b in bindings {
                        let slot_key = eval_ids
                            .get(b.arg as usize)
                            .copied()
                            .expect("ctor arg index in range");
                        let Val::V(av) = slots.remove(&slot_key).expect("ctor arg slot filled");
                        // an elided child ctor already wrote into
                        // temp+off directly — no re-copy
                        if self.elided.remove(&slot_key) {
                            continue;
                        }
                        let off = offs.get(b.field as usize).copied().unwrap_or(0);
                        let ft = field_tys.get(b.field as usize).copied();
                        if ft.map(|t| type_is_record(self.module, &t)) == Some(true) {
                            let hir::TypeUse::Ref(rt) = ft.unwrap() else {
                                unreachable!()
                            };
                            let dst = self.fb.ins().iadd_imm(temp, off as i64);
                            let bytes = self.layouts.get(&rt.0).copied().unwrap_or(4);
                            self.emit_copy(dst, av, bytes);
                        } else {
                            self.fb.ins().store(
                                cranelift_codegen::ir::MemFlags::trusted(),
                                av,
                                temp,
                                off as i32,
                            );
                        }
                    }
                    slots.insert(id.0, Val::V(temp));
                }
                Step::FieldOut(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::FieldGet { base, path } = &e.kind else {
                        unreachable!("field step on field expr")
                    };
                    let Val::V(bv) = slots.remove(&base.0).expect("field base slot");
                    let root = match self.body.exprs[base.0 as usize].ty {
                        hir::TypeUse::Ref(t) => t.0,
                        _ => 0,
                    };
                    let Some((off, field_rec)) =
                        field_path_offset(self.module, &self.layouts, root, path)
                    else {
                        unreachable!("verifier: field path resolves");
                    };
                    match field_rec {
                        None => {
                            let v = self.fb.ins().load(
                                types::I32,
                                cranelift_codegen::ir::MemFlags::trusted(),
                                bv,
                                off as i32,
                            );
                            slots.insert(id.0, Val::V(v));
                        }
                        Some(rt) => {
                            // record field read = a VALUE copy (rvalue) into
                            // the elision destination when one is set
                            let bytes = self.layouts.get(&rt).copied().unwrap_or(4);
                            let temp = self
                                .dests
                                .remove(&id.0)
                                .map(|d| d.ptr)
                                .unwrap_or_else(|| self.new_slot(bytes));
                            let src = self.fb.ins().iadd_imm(bv, off as i64);
                            self.emit_copy(temp, src, bytes);
                            slots.insert(id.0, Val::V(temp));
                        }
                    }
                }
                Step::BorrowOut(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::BorrowView { base } = &e.kind else {
                        unreachable!("borrow step on borrow expr")
                    };
                    let Val::V(bv) = slots.remove(&base.0).expect("borrow base slot");
                    // a view borrow passes the storage pointer through
                    slots.insert(id.0, Val::V(bv));
                }
                Step::Unary { dest, op, operand } => {
                    let a = match slots.remove(&operand.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("operand slot filled"),
                    };
                    let v = match op {
                        hir::UnaryOp::NegI32 => self.checked_neg(a),
                        hir::UnaryOp::NotBool => {
                            // bools are the canonical i32 0/1 here; `!b = b^1`
                            let one = self.fb.ins().iconst(types::I32, 1);
                            self.fb.ins().bxor(a, one)
                        }
                    };
                    slots.insert(dest.0, Val::V(v));
                }
                Step::LogicalLhs {
                    op,
                    lhs,
                    var,
                    merge,
                } => {
                    let l = match slots.remove(&lhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("logical lhs slot filled"),
                    };
                    // The lhs is evaluated exactly once, here.
                    self.fb.def_var(var, l);
                    let rhs_b = self.fb.create_block();
                    match op {
                        hir::LogicOp::And => {
                            // && evaluates the rhs only when the lhs was true
                            self.fb.ins().brif(l, rhs_b, &[], merge, &[]);
                        }
                        hir::LogicOp::Or => {
                            // || evaluates the rhs only when the lhs was false
                            self.fb.ins().brif(l, merge, &[], rhs_b, &[]);
                        }
                    }
                    self.fb.seal_block(rhs_b);
                    self.fb.switch_to_block(rhs_b);
                }
                Step::LogicalRhs {
                    dest,
                    rhs,
                    var,
                    merge,
                } => {
                    let r = match slots.remove(&rhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("logical rhs slot filled"),
                    };
                    self.fb.def_var(var, r);
                    self.fb.ins().jump(merge, &[]);
                    // Both predecessors (lhs branch + rhs jump) are known.
                    self.fb.seal_block(merge);
                    self.fb.switch_to_block(merge);
                    let v = self.fb.use_var(var);
                    slots.insert(dest.0, Val::V(v));
                }
                Step::Call(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::Call {
                        callee,
                        eval_args,
                        bindings,
                    } = &e.kind
                    else {
                        unreachable!("call step on call expr")
                    };
                    let mut vals = Vec::with_capacity(eval_args.len());
                    for a in eval_args {
                        match slots.remove(&a.0) {
                            Some(Val::V(v)) => vals.push(v),
                            _ => unreachable!("arg slot filled"),
                        }
                    }
                    let mut args = Vec::with_capacity(vals.len());
                    for i in 0..vals.len() {
                        let b = bindings
                            .iter()
                            .find(|b| b.param == i as u32)
                            .expect("verifier: full param binding");
                        args.push(vals[b.arg as usize]);
                    }
                    // ABI-E: a record-returning callee gets a fresh out
                    // slot appended; the call's result is that slot's
                    // pointer (774 D-05 — no aggregate return in RAX).
                    // A forwarded destination (rooted at local L) reached
                    // this point only after the insert-side disjointness
                    // gate proved no argument storage touches L — the
                    // callee may therefore write the out slot progressively
                    // while reading its arguments (F-771-R1-01).
                    let callee_rec_result = self.def_result_is_record(callee.0);
                    let out_temp = callee_rec_result.map(|tid| {
                        let bytes = self.layouts.get(&tid).copied().unwrap_or(4);
                        self.dests
                            .remove(&id.0)
                            .map(|d| d.ptr)
                            .unwrap_or_else(|| self.new_slot(bytes))
                    });
                    if let Some(t) = out_temp {
                        args.push(t);
                    }
                    let fnref = self.fnrefs[&callee.0];
                    let call = self.fb.ins().call(fnref, &args);
                    let n_results = self.fb.inst_results(call).len();
                    let v = if let Some(t) = out_temp {
                        t
                    } else if n_results == 0 {
                        // defensive: this profile's calls all return i32
                        self.fb.ins().iconst(types::I32, 0)
                    } else {
                        self.fb.inst_results(call)[0]
                    };
                    slots.insert(id.0, Val::V(v));
                }
            }
        }
        match slots.remove(&eid.0) {
            Some(Val::V(v)) => v,
            _ => unreachable!("root slot filled"),
        }
    }

    /// Signed compare producing the profile's bool representation (i32 0/1).
    /// Cranelift `icmp` yields i8; every bool value that flows into a
    /// variable, branch, call argument or return must be widened to i32
    /// first (QA-01: an unwidened icmp result panicked the frontend on
    /// `let` into an i32 bool local and failed the verifier on bool returns).
    fn icmp_bool(
        &mut self,
        cc: cranelift_codegen::ir::condcodes::IntCC,
        a: Value,
        b: Value,
    ) -> Value {
        let c = self.fb.ins().icmp(cc, a, b);
        self.fb.ins().uextend(types::I32, c)
    }

    fn checked_add(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let sum = self.fb.ins().iadd(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        // overflow iff (b >= 0 && a > max - b) || (b < 0 && a < min - b)
        let room_pos = self.fb.ins().isub(max, b);
        let b_nonneg = self.icmp_bool(IntCC::SignedGreaterThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonneg, a_gt_room);
        let room_neg = self.fb.ins().isub(min, b);
        let b_neg = self.icmp_bool(IntCC::SignedLessThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_neg, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        sum
    }

    /// Emit `brif(ovf -> trap, else continue)`: the trap block ends in
    /// ExitProcess(70), the fall-through block carries on with the wrapping
    /// result (which is only reachable when no overflow occurred).
    fn branch_to_trap_on(&mut self, ovf: Value) {
        let trap = self.ensure_trap_block();
        let cont = self.fb.create_block();
        self.fb.ins().brif(ovf, trap, &[], cont, &[]);
        self.fb.seal_block(cont);
        self.fb.switch_to_block(cont);
    }

    fn checked_mul(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        // Widen to i64, multiply, compare against the sign-extended i32.
        let a64 = self.fb.ins().sextend(types::I64, a);
        let b64 = self.fb.ins().sextend(types::I64, b);
        let r64 = self.fb.ins().imul(a64, b64);
        let r32 = self.fb.ins().ireduce(types::I32, r64);
        let back = self.fb.ins().sextend(types::I64, r32);
        let ovf = self.icmp_bool(IntCC::NotEqual, r64, back);
        self.branch_to_trap_on(ovf);
        r32
    }

    /// `a - b` with a pre-target overflow guard. Subtracting a positive
    /// underflows iff `a < min + b`; subtracting a non-positive overflows
    /// iff `a > max + b`. Both room terms stay inside the i32 domain for
    /// their guard's polarity, so the guard arithmetic itself cannot wrap.
    fn checked_sub(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let diff = self.fb.ins().isub(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let room_pos = self.fb.ins().iadd(max, b);
        let b_nonpos = self.icmp_bool(IntCC::SignedLessThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonpos, a_gt_room);
        let room_neg = self.fb.ins().iadd(min, b);
        let b_pos = self.icmp_bool(IntCC::SignedGreaterThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_pos, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        diff
    }

    /// `-a`; MIN is the only value whose negation leaves the i32 domain.
    fn checked_neg(&mut self, a: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let is_min = self.icmp_bool(IntCC::Equal, a, min);
        self.branch_to_trap_on(is_min);
        self.fb.ins().ineg(a)
    }

    /// `a / b` with explicit guards BEFORE the target instruction (plan
    /// §5.3): the divisor is sanitized to 1 for the trapping polarities so
    /// the hardware `idiv` never sees a zero divisor or the MIN/-1
    /// quotient overflow, then the exact result is selected back and the
    /// profile's fixed trap rules (b == 0, MIN / -1) branch to the trap70
    /// path — no OS hardware exception can escape.
    fn guarded_sdiv(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().sdiv(a, safe);
        // correct quotient for b == -1 is -a (INEG wraps only on MIN, whose
        // path traps below before the wrapped value can be observed)
        let neg = self.fb.ins().ineg(a);
        let q = self.fb.ins().select(bm1, neg, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        q
    }

    /// `a % b` with the same pre-target guards. `x % -1 == 0` in exact
    /// arithmetic, but this profile fixes MIN % -1 as trap70 too (plan
    /// §5.2), so the sanitized remainder is selected to 0 only for the
    /// non-MIN `b == -1` runs.
    fn guarded_srem(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().srem(a, safe);
        let r = self.fb.ins().select(bm1, zero, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        r
    }
}

// ---------------------------------------------------------------------------
// Text-domain lowering (PLAN-770 T-05)
//
// A separate lowerer for `text-i32-draft` modules: locals carry (ptr, len)
// pairs, text expressions become calls into the frozen `ac_rt_*` runtime ABI
// (the static library linked via `--support-lib` plus its contract sidecar),
// and cleanup follows the verifier's TextResourcePlan verbatim — the backend
// performs no ownership analysis of its own. The scalar path above is
// behaviorally untouched (zero-regression contract).
// ---------------------------------------------------------------------------

/// Language-facing machine type of the text domain (parallel to `Vt`).
/// The combination surface (PLAN-782) adds records: an inline aggregate
/// addressed by pointer (stack slot for locals, out-slot for results) -
/// view/mut/move-in parameters and results all pass one I64 pointer
/// (ABI-D/E; the responsibility differences live in the verifier's plan,
/// not the machine shape).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Vt2 {
    I32,
    Bool,
    Char,
    /// Borrowed slice: (I64 ptr, I32 len) representation.
    StrBorrow,
    /// Owned buffer: (I64 ptr, I32 len); freed per the resource plan.
    StrOwn,
    /// Record type id: aggregate by pointer.
    Record(u32),
}

fn value_of_type_use_text(u: &hir::TypeUse, m: &hir::Module) -> Option<Vt2> {
    match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Some(Vt2::I32),
            hir::BuiltinType::Bool => Some(Vt2::Bool),
            hir::BuiltinType::Char => Some(Vt2::Char),
            hir::BuiltinType::StrSlice => Some(Vt2::StrBorrow),
            hir::BuiltinType::StrOwned => Some(Vt2::StrOwn),
        },
        hir::TypeUse::Ref(tid) => match &m.types.get(tid.0 as usize)?.kind {
            hir::TypeKind::I32 => Some(Vt2::I32),
            hir::TypeKind::Bool => Some(Vt2::Bool),
            hir::TypeKind::Char => Some(Vt2::Char),
            hir::TypeKind::Str => Some(Vt2::StrBorrow),
            hir::TypeKind::StrOwned => Some(Vt2::StrOwn),
            hir::TypeKind::Function(_) => None,
            // PLAN-782: records exist on the combination surface; a text
            // module never reaches here (profile defense)
            hir::TypeKind::Record(_) => Some(Vt2::Record(tid.0)),
        },
    }
}

fn signature_of_text(m: &hir::Module, ty: hir::TypeId) -> Option<(Vec<Vt2>, Vt2)> {
    match &m.types.get(ty.0 as usize)?.kind {
        hir::TypeKind::Function(sig) => {
            let mut params = Vec::with_capacity(sig.params.len());
            for p in &sig.params {
                params.push(value_of_type_use_text(p, m)?);
            }
            Some((params, value_of_type_use_text(&sig.result, m)?))
        }
        _ => None,
    }
}

/// Machine signature expansion (774 D-05 frozen shapes):
/// - str/Str parameters become (I64 ptr, I32 len) double parameters;
/// - a Str result returns the pointer and takes one TRAILING `*mut i32`
///   out-length parameter (ABI-C);
/// - a str result returns both values (ptr, len);
/// - char/i32/bool are single I32.
fn cranelift_sig_text(params: &[Vt2], result: Vt2) -> Signature {
    let mut ps = Vec::new();
    for p in params {
        match p {
            Vt2::StrBorrow | Vt2::StrOwn => {
                ps.push(AbiParam::new(types::I64));
                ps.push(AbiParam::new(types::I32));
            }
            // PLAN-782 ABI-D/E: one pointer for view/mut/move-in records
            Vt2::Record(_) => ps.push(AbiParam::new(types::I64)),
            _ => ps.push(AbiParam::new(types::I32)),
        }
    }
    let rets = match result {
        Vt2::StrOwn => {
            ps.push(AbiParam::new(types::I64)); // trailing out-length pointer
            vec![AbiParam::new(types::I64)]
        }
        Vt2::StrBorrow => vec![AbiParam::new(types::I64), AbiParam::new(types::I32)],
        // PLAN-782 ABI-E: record results go through a trailing `*mut`
        // out-slot written by the callee; the pointer returns in RAX
        Vt2::Record(_) => {
            ps.push(AbiParam::new(types::I64)); // trailing out-aggregate pointer
            vec![AbiParam::new(types::I64)]
        }
        _ => vec![AbiParam::new(types::I32)],
    };
    Signature {
        call_conv: WINCALL,
        params: ps,
        returns: rets,
    }
}

/// PLAN-782 section 4.5 layout: fields in declaration order, each at its
/// own alignment (`Str` = 12/8 with {ptr u64, len u32}; scalars/char 4/4;
/// nested records take their own size/align); `align(R) = max(field
/// aligns)`, `size = round_up(tail, align)` (checked u64, capped at
/// u32::MAX). A record whose closure holds no Str field keeps the 771
/// 4-alignment; a resource record is 8-aligned. Frozen reference values:
/// `Tok{text Str, kind, line, at, len int}` size 32, offsets
/// 0/12/16/20/24; `{k int, text Str}` size 24, text@8.
use std::rc::Rc;

#[derive(Clone)]
struct TrLayout {
    size: u64,
    align: u64,
    field_off: Vec<u64>,
}

fn tr_record_layouts(m: &hir::Module, resource: &[bool]) -> BTreeMap<u32, TrLayout> {
    let n = m.types.len();
    let mut done: Vec<Option<Rc<TrLayout>>> = vec![None; n];
    let shape_of = |i: usize| match m.types.get(i).map(|t| &t.kind) {
        Some(hir::TypeKind::Record(sh)) => Some(sh),
        _ => None,
    };
    fn field_sa(
        m: &hir::Module,
        done: &[Option<std::rc::Rc<TrLayout>>],
        f_ty: &hir::TypeUse,
    ) -> Option<(u64, u64)> {
        match f_ty {
            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => Some((12, 8)),
            hir::TypeUse::Builtin(_) => Some((4, 4)),
            hir::TypeUse::Ref(t) => match m.types.get(t.0 as usize).map(|x| &x.kind) {
                Some(hir::TypeKind::StrOwned) => Some((12, 8)),
                Some(hir::TypeKind::Str) => Some((12, 8)), // rejected earlier; totality
                Some(hir::TypeKind::Record(_)) => {
                    done[t.0 as usize].as_ref().map(|l| (l.size, l.align))
                }
                _ => Some((4, 4)),
            },
        }
    }
    for start in 0..n {
        if shape_of(start).is_none() || done[start].is_some() {
            continue;
        }
        let mut stack: Vec<usize> = vec![start];
        let mut on_path = vec![false; n];
        while let Some(&node) = stack.last() {
            let Some(shape) = shape_of(node) else {
                stack.pop();
                continue;
            };
            let mut pending = false;
            for f in &shape.fields {
                if let hir::TypeUse::Ref(t) = &f.ty {
                    let fi = t.0 as usize;
                    if shape_of(fi).is_some() && done[fi].is_none() && !on_path[fi] {
                        on_path[node] = true;
                        stack.push(fi);
                        pending = true;
                        break;
                    }
                }
            }
            if pending {
                continue;
            }
            let mut total: u64 = 0;
            let mut align: u64 = 1;
            let mut offs = Vec::with_capacity(shape.fields.len());
            for f in &shape.fields {
                let (fsz, fal) = field_sa(m, &done, &f.ty).unwrap_or((4, 4));
                align = align.max(fal);
                let off = total.next_multiple_of(fal);
                offs.push(off);
                total = off.checked_add(fsz).unwrap_or(u32::MAX as u64);
            }
            let size = total.next_multiple_of(align.max(1)).min(u32::MAX as u64);
            done[node] = Some(Rc::new(TrLayout {
                size,
                align: align.max(1),
                field_off: offs,
            }));
            on_path[node] = false;
            stack.pop();
        }
    }
    let _ = resource; // alignment derives from the field kinds themselves
    let mut out = BTreeMap::new();
    for (i, l) in done.into_iter().enumerate() {
        if let Some(l) = l {
            out.insert(i as u32, (*l).clone());
        }
    }
    out
}

/// Machine signature of a language-facing runtime symbol (the frozen 774
/// ABI table the shim exports; shapes mirror `runtime/text` SHIM_SIGS).
fn runtime_lang_sig(name: &str) -> Option<Signature> {
    let (params, rets): (&[types::Type], &[types::Type]) = match name {
        "ac_rt_own_free" => (&[types::I64], &[]),
        "ac_rt_own_clone" => (&[types::I64, types::I32, types::I64], &[types::I64]),
        "ac_rt_own_concat" => (
            &[types::I64, types::I32, types::I64, types::I32, types::I64],
            &[types::I64],
        ),
        "ac_rt_own_from_lit" => (&[types::I64, types::I32, types::I64], &[types::I64]),
        "ac_rt_text_scalars" => (&[types::I64, types::I32], &[types::I32]),
        "ac_rt_text_bytes" => (&[types::I64, types::I32], &[types::I32]),
        "ac_rt_char_at" => (&[types::I64, types::I32, types::I32], &[types::I32]),
        "ac_rt_text_slice" => (
            &[types::I64, types::I32, types::I32, types::I32, types::I64],
            &[types::I64],
        ),
        "ac_rt_text_eq" | "ac_rt_text_ne" => (
            &[types::I64, types::I32, types::I64, types::I32],
            &[types::I32],
        ),
        "ac_rt_char_from_codepoint" | "ac_rt_char_to_codepoint" => (&[types::I32], &[types::I32]),
        _ => return None,
    };
    Some(Signature {
        call_conv: WINCALL,
        params: params.iter().map(|t| AbiParam::new(*t)).collect(),
        returns: rets.iter().map(|t| AbiParam::new(*t)).collect(),
    })
}

/// Runtime symbols this body's lowering can reference (first-use demand set;
/// over-declaration is impossible to link wrong, under-declaration is an
/// internal panic, so the scan is conservative).
fn scan_body_rt_needs(
    module: &hir::Module,
    body: &hir::Body,
    plan: &crate::text_resources::BodyResourcePlan,
) -> std::collections::BTreeSet<&'static str> {
    use std::collections::BTreeSet;
    let mut needs: BTreeSet<&'static str> = BTreeSet::new();
    let is_text_ty = |e: &hir::Expr| {
        matches!(
            value_of_type_use_text(&e.ty, module),
            Some(Vt2::StrBorrow | Vt2::StrOwn)
        )
    };
    for e in &body.exprs {
        match &e.kind {
            hir::ExprKind::TextLen { .. } => {
                needs.insert("ac_rt_text_scalars");
            }
            hir::ExprKind::TextLenBytes { .. } => {
                needs.insert("ac_rt_text_bytes");
            }
            hir::ExprKind::TextCharAt { .. } => {
                needs.insert("ac_rt_char_at");
            }
            hir::ExprKind::TextSlice { .. } => {
                needs.insert("ac_rt_text_slice");
            }
            hir::ExprKind::TextConcat { .. } => {
                needs.insert("ac_rt_own_concat");
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::TextClone { .. } | hir::ExprKind::TextToOwned { .. } => {
                needs.insert("ac_rt_own_clone");
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::CharToCodepoint { .. } => {
                needs.insert("ac_rt_char_to_codepoint");
            }
            hir::ExprKind::CharFromCodepoint { .. } => {
                needs.insert("ac_rt_char_from_codepoint");
            }
            hir::ExprKind::StrLit { .. }
                if value_of_type_use_text(&e.ty, module) == Some(Vt2::StrOwn) =>
            {
                needs.insert("ac_rt_own_from_lit");
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::Binary {
                op: hir::BinOp::Eq | hir::BinOp::Ne,
                lhs,
                ..
            } if is_text_ty(&body.exprs[lhs.0 as usize]) => {
                if matches!(op_ref(e), hir::BinOp::Eq) {
                    needs.insert("ac_rt_text_eq");
                } else {
                    needs.insert("ac_rt_text_ne");
                }
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::Call { callee, .. } => {
                if let Some(hir::DefKind::Function { ty, .. }) =
                    module.defs.get(callee.0 as usize).map(|d| &d.kind)
                {
                    if let Some((_, Vt2::StrOwn)) = signature_of_text(module, *ty) {
                        needs.insert("ac_rt_own_free");
                    }
                }
            }
            _ => {}
        }
    }
    if !plan.owned.is_empty() || !plan.exit_frees.is_empty() {
        needs.insert("ac_rt_own_free");
    }
    // PLAN-782: record-clone lowers one ac_rt_own_clone per Str field and
    // the buffer/replace schedules lower ac_rt_own_free calls
    for e in &body.exprs {
        if matches!(e.kind, hir::ExprKind::RecordClone { .. }) {
            needs.insert("ac_rt_own_clone");
            needs.insert("ac_rt_own_free");
        }
    }
    if !plan.buffer_frees.is_empty() || !plan.replace_frees.is_empty() {
        needs.insert("ac_rt_own_free");
    }
    needs
}

/// Borrow helper for the match arm above (the op is already matched).
fn op_ref(e: &hir::Expr) -> &hir::BinOp {
    match &e.kind {
        hir::ExprKind::Binary { op, .. } => op,
        _ => unreachable!("op_ref on binary expr"),
    }
}

/// Lower a text-domain module (PLAN-770): same object/linking frame as the
/// scalar path (every def defined, `ac_start` wrapper, trap70 tail), with
/// text machine signatures, runtime calls and plan-driven cleanup.
fn lower_text_object(
    checked: &CheckedModule,
    sel: &EntrySelection,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let module = &checked.bundle().modules[sel.module];

    // Symbol table: same uniqueness rule as the scalar path.
    let mut seen_names: BTreeMap<&str, DefId> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        if let hir::DefKind::Function { name, .. } = &d.kind {
            if seen_names.insert(name.as_str(), DefId(di as u32)).is_some() {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "backend.duplicate-symbol",
                    format!(
                        "function name `{}` occurs more than once in module `{}`; the native symbol table cannot represent both",
                        name, module.id_text
                    ),
                    d.span,
                )]);
            }
        }
    }

    let triple = Triple::host();
    let mut flag_builder = cranelift_codegen::settings::builder();
    flag_builder.set("opt_level", "none").expect("opt_level");
    flag_builder.set("is_pic", "false").expect("is_pic");
    // r3 R3-03: statement-tail temporaries can keep thousands of live
    // pointers -> frames beyond one page. enable_probestack defaults to
    // FALSE in cranelift 0.126; without probes a >4KB frame jumps the
    // Windows stack guard page and dies with 0xC0000005. The x64 INLINE
    // strategy probes by unrolled touches + a loop — no external symbol.
    flag_builder
        .set("enable_probestack", "true")
        .expect("enable_probestack");
    flag_builder
        .set("probestack_strategy", "inline")
        .expect("probestack_strategy");
    let flags = cranelift_codegen::settings::Flags::new(flag_builder);
    let isa = match cranelift_codegen::isa::lookup(triple) {
        Ok(b) => match b.finish(flags) {
            Ok(isa) => isa,
            Err(e) => return Err(vec![backend_diag(format!("isa finish failed: {}", e))]),
        },
        Err(e) => return Err(vec![backend_diag(format!("host isa unavailable: {}", e))]),
    };
    let obj_builder = match ObjectBuilder::new(
        isa,
        module.path.as_deref().unwrap_or("ac_text_module"),
        default_libcall_names(),
    ) {
        Ok(b) => b,
        Err(e) => return Err(vec![backend_diag(format!("object builder failed: {}", e))]),
    };
    let mut om: ObjectModule = ObjectModule::new(obj_builder);

    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = match om.declare_function("ExitProcess", Linkage::Import, &exit_sig) {
        Ok(id) => id,
        Err(e) => return Err(vec![backend_diag(format!("declare ExitProcess: {}", e))]),
    };

    // Pre-declare every def with its TEXT machine signature.
    let mut mod_func_ids: BTreeMap<u32, cranelift_module::FuncId> = BTreeMap::new();
    let mut def_sigs: BTreeMap<u32, (Vec<Vt2>, Vt2)> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        let (sym, linkage, ty) = match &d.kind {
            hir::DefKind::Function { name, ty, .. } | hir::DefKind::Method { name, ty, .. } => {
                (name.as_str(), Linkage::Export, *ty)
            }
            hir::DefKind::Intrinsic { symbol, ty, .. } => (symbol.as_str(), Linkage::Import, *ty),
        };
        let Some((params, result)) = signature_of_text(module, ty) else {
            return Err(vec![backend_diag(format!(
                "declaration `{}` has no function type",
                d.id_text
            ))]);
        };
        let sig = cranelift_sig_text(&params, result);
        let fid = match om.declare_function(sym, linkage, &sig) {
            Ok(id) => id,
            Err(e) => return Err(vec![backend_diag(format!("declare {}: {}", sym, e))]),
        };
        mod_func_ids.insert(di as u32, fid);
        def_sigs.insert(di as u32, (params, result));
    }

    // Runtime symbol demand set: union over all bodies (declared once).
    let Some(text_plan) = checked.text_resources() else {
        return Err(vec![backend_diag(
            "text module without a verifier resource plan",
        )]);
    };
    let mut rt_names: std::collections::BTreeSet<&'static str> = std::collections::BTreeSet::new();
    let empty_plan = crate::text_resources::BodyResourcePlan::default();
    for (bi, body) in module.bodies.iter().enumerate() {
        let plan = text_plan
            .bodies
            .get(&(sel.module, bi as u32))
            .unwrap_or(&empty_plan);
        rt_names.extend(scan_body_rt_needs(module, body, plan));
    }
    let mut rt_ids: BTreeMap<&'static str, cranelift_module::FuncId> = BTreeMap::new();
    for name in rt_names {
        let sig = runtime_lang_sig(name)
            .ok_or_else(|| vec![backend_diag(format!("unknown runtime symbol `{}`", name))])?;
        let fid = match om.declare_function(name, Linkage::Import, &sig) {
            Ok(id) => id,
            Err(e) => return Err(vec![backend_diag(format!("declare {}: {}", name, e))]),
        };
        rt_ids.insert(name, fid);
    }

    // Define each function body (PLAN-782: record METHODS included - the
    // combination surface carries type-bound methods whose receiver is
    // parameter 0 of the same lowering).
    for (di, d) in module.defs.iter().enumerate() {
        let body_id = match &d.kind {
            hir::DefKind::Function { body, .. } | hir::DefKind::Method { body, .. } => *body,
            hir::DefKind::Intrinsic { .. } => continue,
        };
        let (params_vt, result_vt) = def_sigs[&(di as u32)].clone();
        let body = &module.bodies[body_id.0 as usize];
        let empty = crate::text_resources::BodyResourcePlan::default();
        let plan = text_plan
            .bodies
            .get(&(sel.module, body_id.0))
            .unwrap_or(&empty);

        // Per-body string literals: declare + define read-only data now so
        // the emitter only takes addresses.
        let mut lit_defs: BTreeMap<u32, cranelift_module::DataId> = BTreeMap::new();
        for (ei, e) in body.exprs.iter().enumerate() {
            if let hir::ExprKind::StrLit { value } = &e.kind {
                let name = format!("ac_lit_d{}_e{}", di, ei);
                let id = match om.declare_data(&name, Linkage::Local, false, false) {
                    Ok(id) => id,
                    Err(err) => {
                        return Err(vec![backend_diag(format!(
                            "declare literal data {}: {}",
                            name, err
                        ))])
                    }
                };
                let mut desc = cranelift_module::DataDescription::new();
                desc.define(value.as_bytes().to_vec().into_boxed_slice());
                desc.set_align(1);
                if let Err(err) = om.define_data(id, &desc) {
                    return Err(vec![backend_diag(format!(
                        "define literal data {}: {}",
                        name, err
                    ))]);
                }
                lit_defs.insert(ei as u32, id);
            }
        }

        let mut ctx = om.make_context();
        ctx.func.signature = cranelift_sig_text(&params_vt, result_vt);
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let entry_block = fb.create_block();
            fb.append_block_params_for_function_params(entry_block);
            fb.switch_to_block(entry_block);
            fb.seal_block(entry_block);

            let mut fnrefs: BTreeMap<u32, FuncRef> = BTreeMap::new();
            for (dj, _) in module.defs.iter().enumerate() {
                let fid = mod_func_ids[&(dj as u32)];
                fnrefs.insert(dj as u32, om.declare_func_in_func(fid, &mut fb.func));
            }
            let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);
            let mut rt_refs: BTreeMap<&'static str, FuncRef> = BTreeMap::new();
            for (name, fid) in &rt_ids {
                rt_refs.insert(name, om.declare_func_in_func(*fid, &mut fb.func));
            }
            let mut lit_gvs: BTreeMap<u32, cranelift_codegen::ir::GlobalValue> = BTreeMap::new();
            for (ei, id) in &lit_defs {
                lit_gvs.insert(*ei, om.declare_data_in_func(*id, &mut fb.func));
            }

            // Locals → machine slots: text locals get a (ptr, len) variable
            // pair, scalar locals a single I32; record locals (PLAN-782)
            // get one aggregate stack slot, record params one pointer var.
            let mut svars: BTreeMap<u32, Variable> = BTreeMap::new();
            let mut tvars: BTreeMap<u32, (Variable, Variable)> = BTreeMap::new();
            let mut rslots: BTreeMap<u32, cranelift_codegen::ir::StackSlot> = BTreeMap::new();
            let mut rvars: BTreeMap<u32, Variable> = BTreeMap::new();
            let mut rtids: BTreeMap<u32, u32> = BTreeMap::new();
            let tr_layouts = if checked.domain() == hir::HirDomain::TextRecord {
                tr_record_layouts(module, &[])
            } else {
                BTreeMap::new()
            };
            for (li, l) in body.locals.iter().enumerate() {
                match value_of_type_use_text(&l.ty, module) {
                    Some(Vt2::StrBorrow) | Some(Vt2::StrOwn) => {
                        let p = fb.declare_var(types::I64);
                        let n = fb.declare_var(types::I32);
                        tvars.insert(li as u32, (p, n));
                    }
                    Some(Vt2::Record(tid)) => {
                        rtids.insert(li as u32, tid);
                        let lay = tr_layouts.get(&tid).cloned().unwrap_or(TrLayout {
                            size: 4,
                            align: 4,
                            field_off: Vec::new(),
                        });
                        let align_log2 = if lay.align >= 8 { 3 } else { 2 };
                        let slot =
                            fb.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                                lay.size.max(1) as u32,
                                align_log2,
                            ));
                        // a param's storage is the CALLER's slot: only the
                        // incoming pointer is kept (rslots stays
                        // binding-only so address resolution never reads
                        // the callee's own uninitialized slot)
                        if l.role == hir::LocalRole::Param {
                            let v = fb.declare_var(types::I64);
                            rvars.insert(li as u32, v);
                        } else {
                            rslots.insert(li as u32, slot);
                        }
                    }
                    _ => {
                        let v = fb.declare_var(types::I32);
                        svars.insert(li as u32, v);
                    }
                }
            }

            // Params start defined, in SIGNATURE order: a text param feeds
            // two consecutive block params (ptr, len), a scalar one.
            let bps: Vec<Value> = fb.block_params(entry_block).to_vec();
            let mut cursor = 0usize;
            for (pi, pvt) in params_vt.iter().enumerate() {
                let Some(li) = body
                    .locals
                    .iter()
                    .position(|l| l.role == hir::LocalRole::Param && l.index == Some(pi as u32))
                else {
                    return Err(vec![backend_diag(format!(
                        "body `{}`: parameter {} has no local",
                        body.id_text, pi
                    ))]);
                };
                match pvt {
                    Vt2::StrBorrow | Vt2::StrOwn => {
                        let (p, n) = tvars[&(li as u32)];
                        if cursor + 1 >= bps.len() {
                            return Err(vec![backend_diag(format!(
                                "body `{}`: expanded parameter slots exceed the machine signature",
                                body.id_text
                            ))]);
                        }
                        fb.def_var(p, bps[cursor]);
                        fb.def_var(n, bps[cursor + 1]);
                        cursor += 2;
                    }
                    // PLAN-782 ABI-D/E: one aggregate pointer param
                    Vt2::Record(_) => {
                        let var = rvars[&(li as u32)];
                        if cursor >= bps.len() {
                            return Err(vec![backend_diag(format!(
                                "body `{}`: expanded parameter slots exceed the machine signature",
                                body.id_text
                            ))]);
                        }
                        fb.def_var(var, bps[cursor]);
                        cursor += 1;
                    }
                    _ => {
                        let var = svars[&(li as u32)];
                        if cursor >= bps.len() {
                            return Err(vec![backend_diag(format!(
                                "body `{}`: expanded parameter slots exceed the machine signature",
                                body.id_text
                            ))]);
                        }
                        fb.def_var(var, bps[cursor]);
                        cursor += 1;
                    }
                }
            }
            // StrOwn result: the trailing machine param is the out-length
            // pointer, kept in a variable for the return statements.
            // A record result (PLAN-782 ABI-E) keeps the trailing
            // out-AGGREGATE pointer the same way.
            let out_len_var = if result_vt == Vt2::StrOwn {
                let v = fb.declare_var(types::I64);
                fb.def_var(v, *bps.last().expect("StrOwn sig has the out param"));
                Some(v)
            } else if let Vt2::Record(_) = result_vt {
                let v = fb.declare_var(types::I64);
                fb.def_var(v, *bps.last().expect("record sig has the out param"));
                Some(v)
            } else {
                None
            };

            let mut fx = TextEmitter {
                fb,
                module,
                body,
                svars,
                tvars,
                fnrefs,
                exit_ref,
                trap_block: None,
                loops: Vec::new(),
                def_sigs: &def_sigs,
                result_vt,
                out_len_var,
                plan,
                rt_refs,
                lit_gvs,
                abic_slot: None,
                pending_temps: Vec::new(),
                layouts: tr_layouts,
                rslots,
                rvars,
                rtids,
                pending_aggs: Vec::new(),
            };
            fx.emit_block(body.entry);
            let trap = fx.ensure_trap_block();
            fx.fb.switch_to_block(trap);
            fx.fb.seal_block(trap);
            let code = fx.fb.ins().iconst(types::I32, TRAP_EXIT_CODE as i64);
            fx.fb.ins().call(fx.exit_ref, &[code]);
            fx.fb
                .ins()
                .trap(cranelift_codegen::ir::TrapCode::INTEGER_OVERFLOW);
            fx.fb.finalize();
        }
        if let Err(e) = om.define_function(mod_func_ids[&(di as u32)], &mut ctx) {
            return Err(vec![backend_diag(format!("define {}: {}", d.id_text, e))]);
        }
    }

    // Generic startup wrapper: identical shape to the scalar path (the
    // entry itself is a zero-parameter i32 function in every profile).
    {
        let start_sig = Signature {
            call_conv: WINCALL,
            params: vec![],
            returns: vec![],
        };
        let start_id = match om.declare_function(START_SYMBOL, Linkage::Export, &start_sig) {
            Ok(id) => id,
            Err(e) => {
                return Err(vec![backend_diag(format!(
                    "declare {}: {}",
                    START_SYMBOL, e
                ))])
            }
        };
        let mut ctx = om.make_context();
        ctx.func.signature = start_sig;
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let blk = fb.create_block();
            fb.switch_to_block(blk);
            fb.seal_block(blk);
            let entry_ref = om.declare_func_in_func(mod_func_ids[&sel.def.0], &mut fb.func);
            let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);
            let call = fb.ins().call(entry_ref, &[]);
            let result = fb.inst_results(call)[0];
            fb.ins().call(exit_ref, &[result]);
            fb.ins().return_(&[]);
            fb.finalize();
        }
        if let Err(e) = om.define_function(start_id, &mut ctx) {
            return Err(vec![backend_diag(format!(
                "define {}: {}",
                START_SYMBOL, e
            ))]);
        }
    }

    match om.finish().emit() {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(vec![backend_diag(format!("emit object: {}", e))]),
    }
}

/// One lowered value in the text domain: a scalar (i32/bool/char on the
/// I32 channel) or a text (ptr, len) pair. Owned temporaries live to the
/// END OF THEIR STATEMENT (plan §4.4): every allocation point pushes its
/// pointer onto the emitter's pending list and the statement's closing
/// step frees them LIFO — views of temporaries therefore stay valid for
/// the whole statement, and the root value that flows into an owning
/// `let`/owned `return` transfers OUT of the pending list first.
#[derive(Clone)]
enum TVal {
    I(Value),
    T {
        ptr: Value,
        len: Value,
    },
    /// PLAN-782: a record value - the address of its inline aggregate
    /// (a local's stack slot, a temporary slot, a caller-supplied
    /// out-slot or an incoming parameter pointer).
    R {
        addr: Value,
        tid: u32,
    },
}

/// Per-function emission state for the text domain.
struct TextEmitter<'c> {
    fb: FunctionBuilder<'c>,
    module: &'c hir::Module,
    body: &'c hir::Body,
    svars: BTreeMap<u32, Variable>,
    tvars: BTreeMap<u32, (Variable, Variable)>,
    fnrefs: BTreeMap<u32, FuncRef>,
    exit_ref: FuncRef,
    trap_block: Option<cranelift_codegen::ir::Block>,
    loops: Vec<(
        LoopId,
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    )>,
    def_sigs: &'c BTreeMap<u32, (Vec<Vt2>, Vt2)>,
    result_vt: Vt2,
    out_len_var: Option<Variable>,
    plan: &'c crate::text_resources::BodyResourcePlan,
    rt_refs: BTreeMap<&'static str, FuncRef>,
    lit_gvs: BTreeMap<u32, cranelift_codegen::ir::GlobalValue>,
    /// The shared ABI-C out-length stack slot (see `abic_out_slot`).
    abic_slot: Option<cranelift_codegen::ir::StackSlot>,
    /// Owned temporaries of the statement being emitted, in production
    /// order (freed LIFO at the statement's end). A `merged` entry exists
    /// only on the executed path of a short-circuit right side (r4 R4-02):
    /// the join passes its pointer as a block param (null on the skipped
    /// edge), and the tail free is guarded — no allocation, no free there.
    pending_temps: Vec<PendingTemp>,
    // -- PLAN-782 combination state (empty on a pure text module) --
    /// record layouts (section 4.5); empty when the module has no records
    layouts: BTreeMap<u32, TrLayout>,
    /// record locals' aggregate stack slots
    rslots: BTreeMap<u32, cranelift_codegen::ir::StackSlot>,
    /// record PARAMS: the incoming aggregate pointer saved in a variable
    rvars: BTreeMap<u32, Variable>,
    /// record-type ids of the rslots/rvars entries
    rtids: BTreeMap<u32, u32>,
    /// pending aggregate TEMPORARIES of the statement (ctor/clone/call
    /// results not yet received by an owning position): their Str-field
    /// buffers free at the statement tail, LIFO like text temps
    pending_aggs: Vec<AggTemp>,
}

/// One pending aggregate temporary (PLAN-782).
#[derive(Clone, Copy)]
struct AggTemp {
    slot: cranelift_codegen::ir::StackSlot,
    tid: u32,
    /// the expr id that produced the value (un-pend key for transfers)
    src: u32,
}

/// One pending owned temporary of the current statement.
#[derive(Clone, Copy)]
struct PendingTemp {
    ptr: Value,
    /// short-circuit RHS temp: the value is a join param that is null on
    /// the skipped path — free it only when non-null (r4 R4-02)
    merged: bool,
}

impl<'c> TextEmitter<'c> {
    fn ensure_trap_block(&mut self) -> cranelift_codegen::ir::Block {
        *self
            .trap_block
            .get_or_insert_with(|| self.fb.create_block())
    }

    /// Call a pre-declared runtime symbol; returns the first result value.
    fn call_rt(&mut self, name: &'static str, args: &[Value]) -> Option<Value> {
        let &f = self
            .rt_refs
            .get(name)
            .unwrap_or_else(|| panic!("internal: runtime symbol `{name}` not declared"));
        let inst = self.fb.ins().call(f, args);
        self.fb.inst_results(inst).first().copied()
    }

    /// Free every pending owned temporary of the statement, LIFO — the
    /// statement's FULL tail (774 D-04 / design §4.4: every later effect
    /// in the statement, e.g. a trace mark, precedes the FREE). Called
    /// after each statement's expression evaluation, before the
    /// statement's structural emission.
    ///
    /// `merged` entries (short-circuit RHS temps, r4 R4-02) are join
    /// params that are NULL on the skipped path: each gets a null-guarded
    /// diamond, so the skipped path executes no free while the executed
    /// path frees exactly once — still at the statement tail.
    fn release_stmt_temps(&mut self) {
        use cranelift_codegen::ir::condcodes::IntCC;
        while let Some(t) = self.pending_temps.pop() {
            if !t.merged {
                self.call_rt("ac_rt_own_free", &[t.ptr]);
                continue;
            }
            let is_null = self.fb.ins().icmp_imm(IntCC::Equal, t.ptr, 0);
            let free_b = self.fb.create_block();
            let cont = self.fb.create_block();
            self.fb.ins().brif(is_null, cont, &[], free_b, &[]);
            self.fb.seal_block(free_b);
            self.fb.switch_to_block(free_b);
            self.call_rt("ac_rt_own_free", &[t.ptr]);
            self.fb.ins().jump(cont, &[]);
            self.fb.seal_block(cont);
            self.fb.switch_to_block(cont);
        }
        // PLAN-782: unreceived aggregate temporaries free their Str-field
        // buffers at the same statement tail, LIFO (the aggregate storage
        // itself is a stack slot - nothing to free there)
        while let Some(agg) = self.pending_aggs.pop() {
            let addr = self.fb.ins().stack_addr(types::I64, agg.slot, 0);
            let mut paths = self.agg_str_paths(agg.tid);
            paths.reverse();
            self.free_agg_buffers(addr, agg.tid, &paths);
        }
    }

    /// Transfer a root value's own buffer out of the pending list (the
    /// value escapes into an owning `let` local or an owned `return`).
    fn transfer_root(&mut self, v: &TVal) {
        if let TVal::T { ptr, .. } = v {
            if let Some(pos) = self.pending_temps.iter().rposition(|p| p.ptr == *ptr) {
                self.pending_temps.remove(pos);
            }
        }
    }

    /// ABI-C allocation shape: trailing `*mut i32` out-length slot, pointer
    /// in RAX. A negative length is the runtime's failure channel (RT-FAIL
    /// already traced) — branch to the trap70 exit.
    ///
    /// The out-length slot is SHARED per function: every use loads the
    /// length before the next call, so lifetimes never overlap — and the
    /// frame stays constant-size no matter how many allocation points a
    /// statement chain contains (a fresh slot per call grew the frame past
    /// the stack guard page at chain depth 4096, V-09).
    fn abic_out_slot(&mut self) -> (cranelift_codegen::ir::StackSlot, Value) {
        let slot = *self.abic_slot.get_or_insert_with(|| {
            self.fb
                .create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                    cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                    4,
                    2,
                ))
        });
        let outp = self.fb.ins().stack_addr(types::I64, slot, 0);
        (slot, outp)
    }

    fn abic_call(&mut self, sym: &'static str, args: &[Value]) -> (Value, Value) {
        let (slot, outp) = self.abic_out_slot();
        let mut full = args.to_vec();
        full.push(outp);
        let ptr = self
            .call_rt(sym, &full)
            .unwrap_or_else(|| panic!("internal: `{sym}` must return the pointer"));
        let len = self.fb.ins().stack_load(types::I32, slot, 0);
        self.negative_trap(len);
        self.pending_temps.push(PendingTemp { ptr, merged: false });
        (ptr, len)
    }

    /// `v < 0` branches to the trap70 block (runtime failure channel).
    fn negative_trap(&mut self, v: Value) {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let neg = self.icmp_bool(IntCC::SignedLessThan, v, zero);
        self.branch_to_trap_on(neg);
    }

    /// Emit the resource plan's free schedule at one exit position.
    fn emit_exit_frees(&mut self, bid: u32, idx: Option<usize>) {
        if let Some(frees) = self.plan.exit_frees.get(&(bid, idx)) {
            for &l in frees {
                let (p, _) = self.tvars[&l];
                let ptr = self.fb.use_var(p);
                self.call_rt("ac_rt_own_free", &[ptr]);
            }
        }
        // PLAN-782: the unified buffer schedule - plain owned-Str roots
        // (empty path, already freed by the loop above in the text list)
        // and one own_free per Str field buffer of the live record roots
        // (probe P3 order)
        if let Some(bufs) = self.plan.buffer_frees.get(&(bid, idx)) {
            let flags = cranelift_codegen::ir::MemFlags::trusted();
            for b in bufs {
                if let Some(&(p, _)) = self.tvars.get(&b.root) {
                    // a plain owned-Str root: its free already ran in the
                    // exit_frees loop above (both lists carry it) - skip
                    let _ = p;
                    continue;
                }
                let addr = self.root_addr(b.root);
                let off = self.path_offset(self.rtids[&b.root], &b.path) as i32;
                let ptr = self.fb.ins().load(types::I64, flags, addr, off);
                self.call_rt("ac_rt_own_free", &[ptr]);
            }
        }
    }

    fn def_local(&mut self, id: u32, v: TVal) {
        match v {
            TVal::I(i) => {
                let var = self.svars[&id];
                self.fb.def_var(var, i);
            }
            TVal::T { ptr, len, .. } => {
                let (p, n) = self.tvars[&id];
                self.fb.def_var(p, ptr);
                self.fb.def_var(n, len);
            }
            // record locals live in their aggregate slot - the let/assign
            // arms copy the value into it before this is reached
            TVal::R { .. } => {}
        }
    }

    fn expr_is_owned(&self, id: ExprId) -> bool {
        value_of_type_use_text(&self.body.exprs[id.0 as usize].ty, self.module) == Some(Vt2::StrOwn)
    }

    /// Text-domain block emission: the scalar frame machine plus resource
    /// cleanup at terminator positions and natural block ends.
    fn emit_block(&mut self, entry: hir::BlockId) {
        struct EFrame {
            bid: u32,
            stmts: Vec<hir::Stmt>,
            idx: usize,
            terminated: bool,
            role: ERole,
        }
        enum ERole {
            Entry,
            ThenWithElse {
                else_id: hir::BlockId,
                else_b: cranelift_codegen::ir::Block,
                end_b: cranelift_codegen::ir::Block,
            },
            ThenNoElse {
                end_b: cranelift_codegen::ir::Block,
            },
            ElseArm {
                end_b: cranelift_codegen::ir::Block,
                then_terminated: bool,
            },
            LoopBody {
                body_b: cranelift_codegen::ir::Block,
                after: cranelift_codegen::ir::Block,
            },
        }
        let mut frames: Vec<EFrame> = vec![EFrame {
            bid: entry.0,
            stmts: self.body.blocks[entry.0 as usize].stmts.clone(),
            idx: 0,
            terminated: false,
            role: ERole::Entry,
        }];
        while let Some(frame) = frames.last_mut() {
            let Some(stmt) = frame.stmts.get(frame.idx).cloned() else {
                let frame = frames.pop().expect("emit frame");
                match frame.role {
                    ERole::Entry => {
                        if !frame.terminated {
                            self.emit_exit_frees(frame.bid, None);
                        }
                    }
                    ERole::ThenWithElse {
                        else_id,
                        else_b,
                        end_b,
                    } => {
                        let then_terminated = frame.terminated;
                        if !then_terminated {
                            self.emit_exit_frees(frame.bid, None);
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.switch_to_block(else_b);
                        frames.push(EFrame {
                            bid: else_id.0,
                            stmts: self.body.blocks[else_id.0 as usize].stmts.clone(),
                            idx: 0,
                            terminated: false,
                            role: ERole::ElseArm {
                                end_b,
                                then_terminated,
                            },
                        });
                    }
                    ERole::ThenNoElse { end_b } => {
                        let t = frame.terminated;
                        if !t {
                            self.emit_exit_frees(frame.bid, None);
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.seal_block(end_b);
                        self.fb.switch_to_block(end_b);
                        frames.last_mut().expect("emit frame").terminated = false;
                    }
                    ERole::ElseArm {
                        end_b,
                        then_terminated,
                    } => {
                        let else_terminated = frame.terminated;
                        if !else_terminated {
                            self.emit_exit_frees(frame.bid, None);
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.seal_block(end_b);
                        if then_terminated && else_terminated {
                            frames.last_mut().expect("emit frame").terminated = true;
                        } else {
                            self.fb.switch_to_block(end_b);
                            frames.last_mut().expect("emit frame").terminated = false;
                        }
                    }
                    ERole::LoopBody { body_b, after } => {
                        let t = frame.terminated;
                        if !t {
                            self.emit_exit_frees(frame.bid, None);
                            self.fb.ins().jump(body_b, &[]);
                        }
                        self.fb.seal_block(body_b);
                        self.fb.switch_to_block(after);
                        self.fb.seal_block(after);
                        self.loops.pop();
                        frames.last_mut().expect("emit frame").terminated = false;
                    }
                }
                continue;
            };
            frame.idx += 1;
            let stmt_idx = frame.idx - 1;
            let bid = frame.bid;
            match stmt {
                hir::Stmt::Let { local, value } => {
                    let v = self.eval_text(value);
                    // an owned Str binding TRANSFERS the root temporary out
                    // of the pending list (the local owns it now); the
                    // statement's OTHER temporaries die at its tail
                    if self.tvars.contains_key(&local.0) {
                        self.transfer_root(&v);
                    }
                    if let (Some(&slot), TVal::R { addr, tid }) = (self.rslots.get(&local.0), &v) {
                        // PLAN-782: the binding copies the aggregate into
                        // the local's slot (bitwise - buffers transfer)
                        let dst = self.fb.ins().stack_addr(types::I64, slot, 0);
                        self.copy_aggregate(dst, *addr, *tid);
                        self.unpend_agg(value.0);
                    } else {
                        self.def_local(local.0, v);
                    }
                    self.release_stmt_temps();
                }
                hir::Stmt::Return { value } => {
                    // Value first (an owned Str result's root transfers
                    // out), then the statement's temporaries (the FULL
                    // statement tail — later effects precede the frees),
                    // then the plan's exit frees, then the transfer.
                    let v = self.eval_text(value);
                    if self.result_vt == Vt2::StrOwn {
                        self.transfer_root(&v);
                    }
                    if let (Vt2::Record(_), TVal::R { .. }) = (self.result_vt, &v) {
                        // the returned aggregate transfers out of the
                        // pending list (the receiver owns its buffers)
                        self.unpend_agg(value.0);
                    }
                    self.release_stmt_temps();
                    self.emit_exit_frees(bid, Some(stmt_idx));
                    match (self.result_vt, v) {
                        (Vt2::Record(tid), TVal::R { addr, .. }) => {
                            // ABI-E: write the aggregate through the
                            // out-pointer, return the pointer
                            let var = self
                                .out_len_var
                                .expect("record result keeps the out-pointer variable");
                            let outp = self.fb.use_var(var);
                            self.copy_aggregate(outp, addr, tid);
                            self.fb.ins().return_(&[outp]);
                        }
                        (Vt2::StrOwn, TVal::T { ptr, len, .. }) => {
                            // ABI-C return: len through the out channel.
                            let var = self
                                .out_len_var
                                .expect("StrOwn result keeps the out-length variable");
                            let outp = self.fb.use_var(var);
                            self.fb.ins().store(
                                cranelift_codegen::ir::MemFlags::new(),
                                len,
                                outp,
                                0,
                            );
                            self.fb.ins().return_(&[ptr]);
                        }
                        (Vt2::StrBorrow, TVal::T { ptr, len, .. }) => {
                            self.fb.ins().return_(&[ptr, len]);
                        }
                        (_, TVal::I(v)) => {
                            self.fb.ins().return_(&[v]);
                        }
                        // the verifier types the return value as the result
                        _ => unreachable!("verifier: return value matches the result type"),
                    }
                    frame.terminated = true;
                }
                hir::Stmt::If { cond, then, els } => {
                    let c = self.eval_text(cond);
                    let c = match c {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: if condition is a bool"),
                    };
                    // the condition's temporaries die with the statement
                    // head, before the branch structure
                    self.release_stmt_temps();
                    let then_b = self.fb.create_block();
                    match els {
                        Some(e) => {
                            let end_b = self.fb.create_block();
                            let else_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], else_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.seal_block(else_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                bid: then.0,
                                stmts: self.body.blocks[then.0 as usize].stmts.clone(),
                                idx: 0,
                                terminated: false,
                                role: ERole::ThenWithElse {
                                    else_id: e,
                                    else_b,
                                    end_b,
                                },
                            });
                        }
                        None => {
                            let end_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], end_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                bid: then.0,
                                stmts: self.body.blocks[then.0 as usize].stmts.clone(),
                                idx: 0,
                                terminated: false,
                                role: ERole::ThenNoElse { end_b },
                            });
                        }
                    }
                }
                hir::Stmt::Loop { id, body } => {
                    let after = self.fb.create_block();
                    let body_b = self.fb.create_block();
                    self.fb.ins().jump(body_b, &[]);
                    self.loops.push((id, body_b, after));
                    self.fb.switch_to_block(body_b);
                    frames.push(EFrame {
                        bid: body.0,
                        stmts: self.body.blocks[body.0 as usize].stmts.clone(),
                        idx: 0,
                        terminated: false,
                        role: ERole::LoopBody { body_b, after },
                    });
                }
                hir::Stmt::Assign { place, value } => {
                    let v = self.eval_text(value);
                    let p = &self.body.places[place.0 as usize];
                    let path: Vec<u32> = match &p.kind {
                        hir::PlaceKind::FieldPath(fp) => fp.iter().map(|f| f.0).collect(),
                        _ => Vec::new(),
                    };
                    if self.rslots.contains_key(&p.local.0) || self.rvars.contains_key(&p.local.0) {
                        // PLAN-782 P4: replacement on a record root. The
                        // RHS is complete; free the OLD buffers the plan
                        // scheduled, then commit.
                        self.emit_replace_frees(bid, stmt_idx);
                        let root = self.root_addr(p.local.0);
                        if path.is_empty() {
                            if let TVal::R { addr, tid } = &v {
                                self.copy_aggregate(root, *addr, *tid);
                                self.unpend_agg(value.0);
                            }
                        } else {
                            let tid = self.rtids[&p.local.0];
                            let off = self.path_offset(tid, &path) as i32;
                            let flags = cranelift_codegen::ir::MemFlags::trusted();
                            match &v {
                                TVal::T { ptr, len } => {
                                    // Str-field replacement: the buffer the
                                    // RHS produced moves into the field
                                    self.transfer_root(&TVal::T {
                                        ptr: *ptr,
                                        len: *len,
                                    });
                                    self.fb.ins().store(flags, *ptr, root, off);
                                    self.fb.ins().store(flags, *len, root, off + 8);
                                }
                                TVal::I(iv) => {
                                    self.fb.ins().store(flags, *iv, root, off);
                                }
                                TVal::R { addr, tid: ftid } => {
                                    let faddr = self.fb.ins().iadd_imm(root, off as i64);
                                    self.copy_aggregate(faddr, *addr, *ftid);
                                    self.unpend_agg(value.0);
                                }
                            }
                        }
                    } else {
                        self.def_local(p.local.0, v);
                    }
                    self.release_stmt_temps();
                }
                hir::Stmt::Break { target } => {
                    self.emit_exit_frees(bid, Some(stmt_idx));
                    let (_, _, after) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(after, &[]);
                    frame.terminated = true;
                }
                hir::Stmt::Continue { target } => {
                    self.emit_exit_frees(bid, Some(stmt_idx));
                    let (_, header, _) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(header, &[]);
                    frame.terminated = true;
                }
                hir::Stmt::Drop { local: _ } => {
                    // the plan maps this exact statement position to the
                    // single free of the dropped owned local
                    self.emit_exit_frees(bid, Some(stmt_idx));
                }
            }
        }
    }

    // ==================== PLAN-782 combination emission ====================

    /// Address of a root local's aggregate (slot for bindings, saved
    /// pointer for parameters).
    fn root_addr(&mut self, local: u32) -> Value {
        if let Some(&slot) = self.rslots.get(&local) {
            self.fb.ins().stack_addr(types::I64, slot, 0)
        } else {
            let v = self.rvars[&local];
            self.fb.use_var(v)
        }
    }

    /// Byte offset of a field path within a record type (declaration
    /// indices through nested records).
    fn path_offset(&self, tid: u32, path: &[u32]) -> u64 {
        let mut off = 0u64;
        let mut cur = tid;
        for fi in path {
            let lay = self
                .layouts
                .get(&cur)
                .unwrap_or_else(|| panic!("internal: layout of type {cur}"));
            off = off
                .checked_add(lay.field_off.get(*fi as usize).copied().unwrap_or(0))
                .unwrap_or(0);
            // advance to the field's record type for the next segment
            if let hir::TypeKind::Record(sh) = &self.module.types[cur as usize].kind {
                if let Some(f) = sh.fields.get(*fi as usize) {
                    if let hir::TypeUse::Ref(t) = &f.ty {
                        if matches!(
                            self.module.types.get(t.0 as usize).map(|x| &x.kind),
                            Some(hir::TypeKind::Record(_))
                        ) {
                            cur = t.0;
                            continue;
                        }
                    }
                }
            }
            break;
        }
        off
    }

    /// Str-field paths of a record type, declaration order (a record with
    /// no Str field transitively yields nothing by definition).
    fn agg_str_paths(&self, tid: u32) -> Vec<Vec<u32>> {
        let mut out = Vec::new();
        let mut stack = vec![(tid, Vec::new())];
        while let Some((t, path)) = stack.pop() {
            let Some(hir::TypeKind::Record(sh)) =
                self.module.types.get(t as usize).map(|x| &x.kind)
            else {
                continue;
            };
            for (fi, f) in sh.fields.iter().enumerate().rev() {
                let mut p = path.clone();
                p.push(fi as u32);
                let is_str = match &f.ty {
                    hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => true,
                    hir::TypeUse::Ref(tr) => matches!(
                        self.module.types.get(tr.0 as usize).map(|x| &x.kind),
                        Some(hir::TypeKind::StrOwned)
                    ),
                    _ => false,
                };
                let is_rec = match &f.ty {
                    hir::TypeUse::Ref(tr) => matches!(
                        self.module.types.get(tr.0 as usize).map(|x| &x.kind),
                        Some(hir::TypeKind::Record(_))
                    ),
                    _ => false,
                };
                if is_str {
                    out.push(p);
                } else if is_rec {
                    if let hir::TypeUse::Ref(tr) = &f.ty {
                        stack.push((tr.0, p));
                    }
                }
            }
        }
        // reverse the per-level stack order into declaration order
        out.sort_unstable();
        out
    }

    /// Bitwise aggregate copy (move semantics: buffers transfer with the
    /// bits; a copyable record is the same operation).
    fn copy_aggregate(&mut self, dst: Value, src: Value, tid: u32) {
        let (size, align) = {
            let lay = self
                .layouts
                .get(&tid)
                .unwrap_or_else(|| panic!("internal: layout of type {tid}"));
            (lay.size, lay.align)
        };
        let chunk = if size % 8 == 0 && align >= 8 {
            8u64
        } else {
            4u64
        };
        let flags = cranelift_codegen::ir::MemFlags::trusted();
        let mut off = 0u64;
        while off < size {
            if chunk == 8 {
                let v = self.fb.ins().load(types::I64, flags, src, off as i32);
                self.fb.ins().store(flags, v, dst, off as i32);
            } else {
                let v = self.fb.ins().load(types::I32, flags, src, off as i32);
                self.fb.ins().store(flags, v, dst, off as i32);
            }
            off += chunk;
        }
    }

    /// Free every Str-field buffer of an aggregate (per path, in the
    /// caller's given order).
    fn free_agg_buffers(&mut self, addr: Value, tid: u32, paths: &[Vec<u32>]) {
        let flags = cranelift_codegen::ir::MemFlags::trusted();
        for path in paths {
            let off = self.path_offset(tid, path) as i32;
            let ptr = self.fb.ins().load(types::I64, flags, addr, off);
            self.call_rt("ac_rt_own_free", &[ptr]);
        }
    }

    /// Un-pend the aggregate temp produced by `src` (ownership transferred
    /// into a let/assign/move-arg/owned return).
    fn unpend_agg(&mut self, src: u32) {
        if let Some(pos) = self.pending_aggs.iter().rposition(|a| a.src == src) {
            self.pending_aggs.remove(pos);
        }
    }

    /// Fresh aggregate temp slot for `tid`.
    fn agg_temp_slot(&mut self, tid: u32) -> cranelift_codegen::ir::StackSlot {
        let lay = self.layouts.get(&tid).cloned().unwrap_or(TrLayout {
            size: 4,
            align: 4,
            field_off: Vec::new(),
        });
        let align_log2 = if lay.align >= 8 { 3 } else { 2 };
        self.fb
            .create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                lay.size.max(1) as u32,
                align_log2,
            ))
    }

    /// Emit the plan's replacement frees for one assign statement (P4:
    /// after the RHS materialized, before the commit).
    fn emit_replace_frees(&mut self, bid: u32, idx: usize) {
        if let Some(bufs) = self.plan.replace_frees.get(&(bid, idx)) {
            for b in bufs {
                let addr = self.root_addr(b.root);
                let off = self.path_offset(self.rtids[&b.root], &b.path) as i32;
                let ptr = self.fb.ins().load(
                    types::I64,
                    cranelift_codegen::ir::MemFlags::trusted(),
                    addr,
                    off,
                );
                self.call_rt("ac_rt_own_free", &[ptr]);
            }
        }
    }

    /// Iterative post-order text expression emission (the scalar spine
    /// machine extended with the text node reductions). Owned temporaries
    /// are pushed onto the emitter's pending list and freed LIFO at the
    /// END of their full statement (plan §4.4); a root that flows into an
    /// owning let/owned return transfers out of the pending list first.
    fn eval_text(&mut self, eid: ExprId) -> TVal {
        enum TStep {
            Enter(ExprId),
            // PLAN-782 combination reductions
            Ctor(ExprId),
            Field(ExprId),
            RClone(ExprId),
            MoveP(ExprId),
            BorrowV(ExprId),
            Binary {
                dest: ExprId,
                op: hir::BinOp,
                lhs: ExprId,
                rhs: ExprId,
            },
            Unary {
                dest: ExprId,
                op: hir::UnaryOp,
                operand: ExprId,
            },
            LogicalLhs {
                dest: ExprId,
                op: hir::LogicOp,
                lhs: ExprId,
                rhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
            },
            LogicalRhs {
                dest: ExprId,
                rhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
                /// pending_temps length at RHS entry (r4 R4-02): entries
                /// beyond this were allocated inside the RHS region only
                temps_mark: usize,
            },
            Call(ExprId),
            Len {
                dest: ExprId,
                operand: ExprId,
            },
            LenBytes {
                dest: ExprId,
                operand: ExprId,
            },
            CharAt {
                dest: ExprId,
                operand: ExprId,
                index: ExprId,
            },
            Slice {
                dest: ExprId,
                operand: ExprId,
                start: ExprId,
                end: ExprId,
            },
            Concat {
                dest: ExprId,
                lhs: ExprId,
                rhs: ExprId,
            },
            Clone {
                dest: ExprId,
                operand: ExprId,
            },
            ToBorrowed {
                dest: ExprId,
                operand: ExprId,
            },
            Codepoint {
                dest: ExprId,
                operand: ExprId,
                from: bool,
            },
        }
        let mut slots: BTreeMap<u32, TVal> = BTreeMap::new();
        let mut stack: Vec<TStep> = vec![TStep::Enter(eid)];
        while let Some(step) = stack.pop() {
            match step {
                TStep::Enter(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    match &e.kind {
                        hir::ExprKind::Constant { value } => {
                            let v = self.fb.ins().iconst(types::I32, *value as i64);
                            slots.insert(id.0, TVal::I(v));
                        }
                        hir::ExprKind::ReadLocal { local } => {
                            if let Some(&(p, n)) = self.tvars.get(&local.0) {
                                let ptr = self.fb.use_var(p);
                                let len = self.fb.use_var(n);
                                slots.insert(id.0, TVal::T { ptr, len });
                            } else if let Some(&slot) = self.rslots.get(&local.0) {
                                // PLAN-782: a record local's value is the
                                // address of its aggregate slot
                                let tid = self.rtids[&local.0];
                                let addr = self.fb.ins().stack_addr(types::I64, slot, 0);
                                slots.insert(id.0, TVal::R { addr, tid });
                            } else if let Some(&v) = self.rvars.get(&local.0) {
                                let tid = self.rtids[&local.0];
                                let addr = self.fb.use_var(v);
                                slots.insert(id.0, TVal::R { addr, tid });
                            } else {
                                let var = self.svars[&local.0];
                                slots.insert(id.0, TVal::I(self.fb.use_var(var)));
                            }
                        }
                        hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                            stack.push(TStep::Binary {
                                dest: id,
                                op: *op,
                                lhs: *lhs,
                                rhs: *rhs,
                            });
                            stack.push(TStep::Enter(*rhs));
                            stack.push(TStep::Enter(*lhs));
                        }
                        hir::ExprKind::Unary { op, operand } => {
                            stack.push(TStep::Unary {
                                dest: id,
                                op: *op,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::Logical { op, lhs, rhs } => {
                            let var = self.fb.declare_var(types::I32);
                            let merge = self.fb.create_block();
                            // r5 R5-01: the RHS-temp boundary is frozen by
                            // the LogicalLhs step AFTER the lhs (including
                            // any logicals nested inside it) has fully
                            // evaluated — lhs temps execute on EVERY path
                            // and must keep their real pointers; only the
                            // rhs-region temps ride the join as null-on-
                            // skip params.
                            stack.push(TStep::LogicalLhs {
                                dest: id,
                                op: *op,
                                lhs: *lhs,
                                rhs: *rhs,
                                var,
                                merge,
                            });
                            stack.push(TStep::Enter(*lhs));
                        }
                        hir::ExprKind::Call { eval_args, .. } => {
                            stack.push(TStep::Call(id));
                            for a in eval_args.iter().rev() {
                                stack.push(TStep::Enter(*a));
                            }
                        }
                        hir::ExprKind::StrLit { value } => {
                            let gv = self.lit_gvs[&id.0];
                            let addr = self.fb.ins().global_value(types::I64, gv);
                            let len = self.fb.ins().iconst(types::I32, value.len() as i64);
                            if self.expr_is_owned(id) {
                                // owning context: promote the static bytes
                                // into a fresh owned buffer (AP-4)
                                let (ptr, len) = self.abic_call("ac_rt_own_from_lit", &[addr, len]);
                                slots.insert(id.0, TVal::T { ptr, len });
                            } else {
                                slots.insert(id.0, TVal::T { ptr: addr, len });
                            }
                        }
                        hir::ExprKind::CharLit { scalar } => {
                            let v = self.fb.ins().iconst(types::I32, *scalar as i64);
                            slots.insert(id.0, TVal::I(v));
                        }
                        hir::ExprKind::TextLen { operand } => {
                            stack.push(TStep::Len {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextLenBytes { operand } => {
                            stack.push(TStep::LenBytes {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextCharAt { operand, index } => {
                            stack.push(TStep::CharAt {
                                dest: id,
                                operand: *operand,
                                index: *index,
                            });
                            stack.push(TStep::Enter(*index));
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextSlice {
                            operand,
                            start,
                            end,
                        } => {
                            stack.push(TStep::Slice {
                                dest: id,
                                operand: *operand,
                                start: *start,
                                end: *end,
                            });
                            stack.push(TStep::Enter(*end));
                            stack.push(TStep::Enter(*start));
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextConcat { lhs, rhs } => {
                            stack.push(TStep::Concat {
                                dest: id,
                                lhs: *lhs,
                                rhs: *rhs,
                            });
                            stack.push(TStep::Enter(*rhs));
                            stack.push(TStep::Enter(*lhs));
                        }
                        hir::ExprKind::TextClone { operand }
                        | hir::ExprKind::TextToOwned { operand } => {
                            stack.push(TStep::Clone {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextToBorrowed { operand } => {
                            stack.push(TStep::ToBorrowed {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::CharToCodepoint { operand } => {
                            stack.push(TStep::Codepoint {
                                dest: id,
                                operand: *operand,
                                from: false,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::CharFromCodepoint { operand } => {
                            stack.push(TStep::Codepoint {
                                dest: id,
                                operand: *operand,
                                from: true,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        // PLAN-782 combination kinds (never in a pure text
                        // module - profile defense)
                        hir::ExprKind::RecordCtor { eval_args, .. } => {
                            stack.push(TStep::Ctor(id));
                            for a in eval_args.iter().rev() {
                                stack.push(TStep::Enter(*a));
                            }
                        }
                        hir::ExprKind::FieldGet { base, .. } => {
                            stack.push(TStep::Field(id));
                            stack.push(TStep::Enter(*base));
                        }
                        hir::ExprKind::RecordClone { base, .. } => {
                            stack.push(TStep::RClone(id));
                            stack.push(TStep::Enter(*base));
                        }
                        hir::ExprKind::Move { inner } => {
                            stack.push(TStep::MoveP(id));
                            stack.push(TStep::Enter(*inner));
                        }
                        hir::ExprKind::BorrowView { base } => {
                            stack.push(TStep::BorrowV(id));
                            stack.push(TStep::Enter(*base));
                        }
                        hir::ExprKind::BorrowMut { local } => {
                            let tid = self.rtids[&local.0];
                            let addr = if let Some(&slot) = self.rslots.get(&local.0) {
                                self.fb.ins().stack_addr(types::I64, slot, 0)
                            } else {
                                let v = self.rvars[&local.0];
                                self.fb.use_var(v)
                            };
                            slots.insert(id.0, TVal::R { addr, tid });
                        }
                    }
                }
                TStep::Ctor(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::RecordCtor {
                        record,
                        eval_args,
                        bindings,
                    } = &e.kind
                    else {
                        unreachable!("ctor step on ctor expr")
                    };
                    let tid = record.0;
                    let mut arg_vals: Vec<Option<TVal>> =
                        eval_args.iter().map(|a| slots.remove(&a.0)).collect();
                    let slot = self.agg_temp_slot(tid);
                    let addr = self.fb.ins().stack_addr(types::I64, slot, 0);
                    let flags = cranelift_codegen::ir::MemFlags::trusted();
                    let lay = self.layouts[&tid].clone();
                    for b in bindings {
                        let off = lay.field_off.get(b.field as usize).copied().unwrap_or(0) as i32;
                        let v = arg_vals[b.arg as usize]
                            .take()
                            .expect("ctor arg slot filled");
                        match v {
                            TVal::I(v) => {
                                self.fb.ins().store(flags, v, addr, off);
                            }
                            TVal::T { ptr, len } => {
                                // the aggregate takes over the buffer
                                self.transfer_root(&TVal::T { ptr, len });
                                self.fb.ins().store(flags, ptr, addr, off);
                                self.fb.ins().store(flags, len, addr, off + 8);
                            }
                            TVal::R {
                                addr: src,
                                tid: ftid,
                            } => {
                                // nested record field: bitwise copy from the
                                // source aggregate into this field; a temp
                                // source transfers its buffers into the ctor
                                let faddr = self.fb.ins().iadd_imm(addr, off as i64);
                                self.copy_aggregate(faddr, src, ftid);
                                self.unpend_agg(eval_args[b.arg as usize].0);
                            }
                        }
                    }
                    self.pending_aggs.push(AggTemp {
                        slot,
                        tid,
                        src: id.0,
                    });
                    slots.insert(id.0, TVal::R { addr, tid });
                }
                TStep::Field(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::FieldGet { base, path } = &e.kind else {
                        unreachable!("field step on field expr")
                    };
                    let bv = slots.remove(&base.0).expect("field base slot filled");
                    let TVal::R { addr, tid } = bv else {
                        unreachable!("verifier: field access base is a record")
                    };
                    let path: Vec<u32> = path.iter().map(|f| f.0).collect();
                    let off = self.path_offset(tid, &path) as i64;
                    let faddr = self.fb.ins().iadd_imm(addr, off);
                    let flags = cranelift_codegen::ir::MemFlags::trusted();
                    match value_of_type_use_text(&e.ty, self.module) {
                        // a Str-field read spelled either way (verify's
                        // 782 compatibility accepts `Str` and the retagged
                        // `str`) is a borrowing view — a read never copies
                        // the owner, so both spellings load (ptr, len).
                        Some(Vt2::StrBorrow | Vt2::StrOwn) => {
                            let ptr = self.fb.ins().load(types::I64, flags, faddr, 0);
                            let len = self.fb.ins().load(types::I32, flags, faddr, 8);
                            slots.insert(id.0, TVal::T { ptr, len });
                        }
                        Some(Vt2::Record(t)) => {
                            slots.insert(
                                id.0,
                                TVal::R {
                                    addr: faddr,
                                    tid: t,
                                },
                            );
                        }
                        _ => {
                            let v = self.fb.ins().load(types::I32, flags, faddr, 0);
                            slots.insert(id.0, TVal::I(v));
                        }
                    }
                }
                TStep::RClone(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::RecordClone { record, base } = &e.kind else {
                        unreachable!("clone step on clone expr")
                    };
                    let tid = record.0;
                    let bv = slots.remove(&base.0).expect("clone base slot filled");
                    let TVal::R { addr, .. } = bv else {
                        unreachable!("verifier: clone base is a record")
                    };
                    let slot = self.agg_temp_slot(tid);
                    let taddr = self.fb.ins().stack_addr(types::I64, slot, 0);
                    // scalars (and copyable nesting) copy bitwise; every Str
                    // field re-clones through the runtime (design 5)
                    self.copy_aggregate(taddr, addr, tid);
                    let flags = cranelift_codegen::ir::MemFlags::trusted();
                    for path in self.agg_str_paths(tid) {
                        let off = self.path_offset(tid, &path) as i32;
                        let ptr = self.fb.ins().load(types::I64, flags, addr, off);
                        let len = self.fb.ins().load(types::I32, flags, addr, off + 8);
                        let (cptr, clen) = self.abic_call("ac_rt_own_clone", &[ptr, len]);
                        // the cloned buffer belongs to the AGGREGATE temp
                        // now, not to the text temporary list
                        self.transfer_root(&TVal::T {
                            ptr: cptr,
                            len: clen,
                        });
                        self.fb.ins().store(flags, cptr, taddr, off);
                        self.fb.ins().store(flags, clen, taddr, off + 8);
                    }
                    self.pending_aggs.push(AggTemp {
                        slot,
                        tid,
                        src: id.0,
                    });
                    slots.insert(id.0, TVal::R { addr: taddr, tid });
                }
                TStep::MoveP(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::Move { inner } = &e.kind else {
                        unreachable!("move step on move expr")
                    };
                    let v = slots.remove(&inner.0).expect("move inner slot filled");
                    match v {
                        TVal::T { ptr, len } => {
                            // a moved text temp belongs to the receiver
                            self.transfer_root(&TVal::T { ptr, len });
                            slots.insert(id.0, TVal::T { ptr, len });
                        }
                        TVal::R { addr, tid } => {
                            self.unpend_agg(inner.0);
                            slots.insert(id.0, TVal::R { addr, tid });
                        }
                        TVal::I(i) => {
                            slots.insert(id.0, TVal::I(i));
                        }
                    }
                }
                TStep::BorrowV(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::BorrowView { base } = &e.kind else {
                        unreachable!("borrow step on borrow expr")
                    };
                    let v = slots.remove(&base.0).expect("borrow base slot filled");
                    slots.insert(id.0, v);
                }
                TStep::Binary { dest, op, lhs, rhs } => {
                    let a = slots.remove(&lhs.0).expect("lhs slot filled");
                    let b = slots.remove(&rhs.0).expect("rhs slot filled");
                    let v = match (op, a, b) {
                        (
                            hir::BinOp::Eq | hir::BinOp::Ne,
                            TVal::T { ptr: ap, len: al },
                            TVal::T { ptr: bp, len: bl },
                        ) => {
                            // byte-exact UTF-8 value equality (D-02); the
                            // operands' temporaries live to statement end
                            let sym = if matches!(op, hir::BinOp::Eq) {
                                "ac_rt_text_eq"
                            } else {
                                "ac_rt_text_ne"
                            };
                            let r = self
                                .call_rt(sym, &[ap, al, bp, bl])
                                .expect("eq returns i32");
                            self.negative_trap(r);
                            r
                        }
                        (_, TVal::I(a), TVal::I(b)) => match op {
                            hir::BinOp::AddI32 => self.checked_add(a, b),
                            hir::BinOp::SubI32 => self.checked_sub(a, b),
                            hir::BinOp::MulI32 => self.checked_mul(a, b),
                            hir::BinOp::DivI32 => self.guarded_sdiv(a, b),
                            hir::BinOp::RemI32 => self.guarded_srem(a, b),
                            hir::BinOp::LtI32 => {
                                use cranelift_codegen::ir::condcodes::IntCC;
                                self.icmp_bool(IntCC::SignedLessThan, a, b)
                            }
                            hir::BinOp::LeI32 => {
                                use cranelift_codegen::ir::condcodes::IntCC;
                                self.icmp_bool(IntCC::SignedLessThanOrEqual, a, b)
                            }
                            hir::BinOp::GtI32 => {
                                use cranelift_codegen::ir::condcodes::IntCC;
                                self.icmp_bool(IntCC::SignedGreaterThan, a, b)
                            }
                            hir::BinOp::GeI32 => {
                                use cranelift_codegen::ir::condcodes::IntCC;
                                self.icmp_bool(IntCC::SignedGreaterThanOrEqual, a, b)
                            }
                            hir::BinOp::Eq => {
                                use cranelift_codegen::ir::condcodes::IntCC;
                                self.icmp_bool(IntCC::Equal, a, b)
                            }
                            hir::BinOp::Ne => {
                                use cranelift_codegen::ir::condcodes::IntCC;
                                self.icmp_bool(IntCC::NotEqual, a, b)
                            }
                        },
                        // text operands only reach Eq/Ne (concat is its own
                        // node; typing rejects every other mixed form)
                        _ => unreachable!("verifier: binary operand kinds match the op"),
                    };
                    slots.insert(dest.0, TVal::I(v));
                }
                TStep::Unary { dest, op, operand } => {
                    let a = match slots.remove(&operand.0).expect("operand slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: unary operand is scalar"),
                    };
                    let v = match op {
                        hir::UnaryOp::NegI32 => self.checked_neg(a),
                        hir::UnaryOp::NotBool => {
                            let one = self.fb.ins().iconst(types::I32, 1);
                            self.fb.ins().bxor(a, one)
                        }
                    };
                    slots.insert(dest.0, TVal::I(v));
                }
                TStep::LogicalLhs {
                    dest,
                    op,
                    lhs,
                    rhs,
                    var,
                    merge,
                } => {
                    let l = match slots.remove(&lhs.0).expect("logical lhs slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: logical operands are bool"),
                    };
                    self.fb.def_var(var, l);
                    let rhs_b = self.fb.create_block();
                    match op {
                        hir::LogicOp::And => {
                            self.fb.ins().brif(l, rhs_b, &[], merge, &[]);
                        }
                        hir::LogicOp::Or => {
                            self.fb.ins().brif(l, merge, &[], rhs_b, &[]);
                        }
                    }
                    self.fb.seal_block(rhs_b);
                    self.fb.switch_to_block(rhs_b);
                    // r5 R5-01: freeze the RHS temp boundary HERE — the
                    // lhs (including logicals nested inside it) has fully
                    // evaluated, so pending_temps below this mark execute
                    // on every path and keep their real (unconditionally
                    // freed) pointers; only the rhs region rides the join
                    // as null-on-skip params.
                    let temps_mark = self.pending_temps.len();
                    stack.push(TStep::LogicalRhs {
                        dest,
                        rhs,
                        var,
                        merge,
                        temps_mark,
                    });
                    stack.push(TStep::Enter(rhs));
                }
                TStep::LogicalRhs {
                    dest,
                    rhs,
                    var,
                    merge,
                    temps_mark,
                } => {
                    let r = match slots.remove(&rhs.0).expect("logical rhs slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: logical operands are bool"),
                    };
                    self.fb.def_var(var, r);
                    // r4 R4-02: owned temporaries allocated inside the RHS
                    // region exist ONLY on the executed path. The statement
                    // tail still frees them (774 D-04: later statement
                    // effects precede the FREE), so their pointers are
                    // carried through the join as block params — the
                    // skipped edge forwards null, and the tail free is
                    // null-guarded. Without this the tail free used an SSA
                    // value the skip predecessor never defined (backend
                    // verifier error on every RHS-owned-temp short
                    // circuit).
                    let rhs_temps = self.pending_temps.split_off(temps_mark);
                    let mut phis: Vec<Value> = Vec::new();
                    let cont = if rhs_temps.is_empty() {
                        self.fb.ins().jump(merge, &[]);
                        self.fb.seal_block(merge);
                        merge
                    } else {
                        let cont = self.fb.create_block();
                        for _ in &rhs_temps {
                            phis.push(self.fb.append_block_param(cont, types::I64));
                        }
                        let args: Vec<cranelift_codegen::ir::BlockArg> =
                            rhs_temps.iter().map(|t| t.ptr.into()).collect();
                        self.fb.ins().jump(cont, &args);
                        // the skip edge (the lhs branch straight into
                        // `merge`) forwards null for every RHS temp; both
                        // edges exist before the join is sealed
                        self.fb.switch_to_block(merge);
                        let null = self.fb.ins().iconst(types::I64, 0);
                        let nulls: Vec<cranelift_codegen::ir::BlockArg> =
                            rhs_temps.iter().map(|_| null.into()).collect();
                        self.fb.ins().jump(cont, &nulls);
                        self.fb.seal_block(cont);
                        self.fb.seal_block(merge);
                        cont
                    };
                    self.fb.switch_to_block(cont);
                    for p in phis {
                        self.pending_temps.push(PendingTemp {
                            ptr: p,
                            merged: true,
                        });
                    }
                    let v = self.fb.use_var(var);
                    slots.insert(dest.0, TVal::I(v));
                }
                TStep::Call(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::Call {
                        callee,
                        eval_args,
                        bindings,
                    } = &e.kind
                    else {
                        unreachable!("call step on call expr")
                    };
                    let mut arg_vals: Vec<Option<TVal>> =
                        eval_args.iter().map(|a| slots.remove(&a.0)).collect();
                    let (params_vt, result_vt) = &self.def_sigs[&callee.0];
                    let mut margs: Vec<Value> = Vec::with_capacity(params_vt.len() * 2);
                    for (i, pvt) in params_vt.iter().enumerate() {
                        let b = bindings
                            .iter()
                            .find(|b| b.param == i as u32)
                            .expect("verifier: full param binding");
                        let v = arg_vals[b.arg as usize].take().expect("arg slot filled");
                        match (pvt, v) {
                            // `.move` transfers the local (never a
                            // temp); every text argument's TEMPORARY stays
                            // pending until the statement's end — a
                            // borrowed-returning callee may hand back a
                            // view of that same buffer, and the full-stmt
                            // lifetime covers its consumers (r2 QA-05).
                            (Vt2::StrBorrow | Vt2::StrOwn, TVal::T { ptr, len }) => {
                                margs.push(ptr);
                                margs.push(len);
                            }
                            (Vt2::Record(_), TVal::R { addr, .. }) => {
                                // ABI-D/E: one aggregate pointer argument
                                margs.push(addr);
                            }
                            (_, TVal::I(v)) => margs.push(v),
                            _ => unreachable!("verifier: argument matches the parameter type"),
                        }
                    }
                    let fnref = self.fnrefs[&callee.0];
                    let result = match result_vt {
                        Vt2::StrOwn => {
                            // ABI-C: trailing out-length slot, ptr in RAX
                            let (slot, outp) = self.abic_out_slot();
                            margs.push(outp);
                            let call = self.fb.ins().call(fnref, &margs);
                            let ptr = self.fb.inst_results(call)[0];
                            let len = self.fb.ins().stack_load(types::I32, slot, 0);
                            self.negative_trap(len);
                            self.pending_temps.push(PendingTemp { ptr, merged: false });
                            TVal::T { ptr, len }
                        }
                        Vt2::StrBorrow => {
                            let call = self.fb.ins().call(fnref, &margs);
                            let rs = self.fb.inst_results(call);
                            TVal::T {
                                ptr: rs[0],
                                len: rs[1],
                            }
                        }
                        Vt2::Record(tid) => {
                            // ABI-E: caller-provided out-slot; the callee
                            // writes the aggregate and returns the pointer
                            let slot = self.agg_temp_slot(*tid);
                            let outp = self.fb.ins().stack_addr(types::I64, slot, 0);
                            margs.push(outp);
                            self.fb.ins().call(fnref, &margs);
                            let addr = self.fb.ins().stack_addr(types::I64, slot, 0);
                            self.pending_aggs.push(AggTemp {
                                slot,
                                tid: *tid,
                                src: id.0,
                            });
                            TVal::R { addr, tid: *tid }
                        }
                        _ => {
                            let call = self.fb.ins().call(fnref, &margs);
                            let n = self.fb.inst_results(call).len();
                            let v = if n == 0 {
                                self.fb.ins().iconst(types::I32, 0)
                            } else {
                                self.fb.inst_results(call)[0]
                            };
                            TVal::I(v)
                        }
                    };
                    slots.insert(id.0, result);
                }
                TStep::Len { dest, operand } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let (ptr, len) = text_parts(&o);
                    let r = self
                        .call_rt("ac_rt_text_scalars", &[ptr, len])
                        .expect("scalars returns i32");
                    self.negative_trap(r);
                    slots.insert(dest.0, TVal::I(r));
                }
                TStep::LenBytes { dest, operand } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let (ptr, len) = text_parts(&o);
                    let r = self
                        .call_rt("ac_rt_text_bytes", &[ptr, len])
                        .expect("bytes returns i32");
                    self.negative_trap(r);
                    slots.insert(dest.0, TVal::I(r));
                }
                TStep::CharAt {
                    dest,
                    operand,
                    index,
                } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let i = match slots.remove(&index.0).expect("index slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: char_at index is i32"),
                    };
                    let (ptr, len) = text_parts(&o);
                    let r = self
                        .call_rt("ac_rt_char_at", &[ptr, len, i])
                        .expect("char_at returns i32");
                    self.negative_trap(r);
                    slots.insert(dest.0, TVal::I(r));
                }
                TStep::Slice {
                    dest,
                    operand,
                    start,
                    end,
                } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let s = match slots.remove(&start.0).expect("start slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: slice start is i32"),
                    };
                    let e = match slots.remove(&end.0).expect("end slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: slice end is i32"),
                    };
                    let (ptr, len) = text_parts(&o);
                    let (slot, outp) = self.abic_out_slot();
                    let sptr = self
                        .call_rt("ac_rt_text_slice", &[ptr, len, s, e, outp])
                        .expect("slice returns the view pointer");
                    let slen = self.fb.ins().stack_load(types::I32, slot, 0);
                    self.negative_trap(slen);
                    // a borrowed view; the operand's temporary (if any)
                    // lives to the statement's end, after this view's
                    // consumers
                    slots.insert(
                        dest.0,
                        TVal::T {
                            ptr: sptr,
                            len: slen,
                        },
                    );
                }
                TStep::Concat { dest, lhs, rhs } => {
                    let a = slots.remove(&lhs.0).expect("lhs slot filled");
                    let b = slots.remove(&rhs.0).expect("rhs slot filled");
                    let (ap, al) = text_parts(&a);
                    let (bp, bl) = text_parts(&b);
                    let (ptr, len) = self.abic_call("ac_rt_own_concat", &[ap, al, bp, bl]);
                    slots.insert(dest.0, TVal::T { ptr, len });
                }
                TStep::Clone { dest, operand } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let (ptr, len) = text_parts(&o);
                    let (nptr, nlen) = self.abic_call("ac_rt_own_clone", &[ptr, len]);
                    slots.insert(
                        dest.0,
                        TVal::T {
                            ptr: nptr,
                            len: nlen,
                        },
                    );
                }
                TStep::ToBorrowed { dest, operand } => {
                    // a pure view: no instruction, no transfer
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    slots.insert(dest.0, o);
                }
                TStep::Codepoint {
                    dest,
                    operand,
                    from,
                } => {
                    let o = match slots.remove(&operand.0).expect("operand slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: codepoint operand is scalar"),
                    };
                    let sym = if from {
                        "ac_rt_char_from_codepoint"
                    } else {
                        "ac_rt_char_to_codepoint"
                    };
                    let r = self.call_rt(sym, &[o]).expect("codepoint returns i32");
                    self.negative_trap(r);
                    slots.insert(dest.0, TVal::I(r));
                }
            }
        }
        match slots.remove(&eid.0) {
            Some(v) => v,
            None => unreachable!("root slot filled"),
        }
    }

    /// Signed compare producing the profile's bool representation (i32 0/1).
    fn icmp_bool(
        &mut self,
        cc: cranelift_codegen::ir::condcodes::IntCC,
        a: Value,
        b: Value,
    ) -> Value {
        let c = self.fb.ins().icmp(cc, a, b);
        self.fb.ins().uextend(types::I32, c)
    }

    fn checked_add(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let sum = self.fb.ins().iadd(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let room_pos = self.fb.ins().isub(max, b);
        let b_nonneg = self.icmp_bool(IntCC::SignedGreaterThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonneg, a_gt_room);
        let room_neg = self.fb.ins().isub(min, b);
        let b_neg = self.icmp_bool(IntCC::SignedLessThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_neg, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        sum
    }

    fn branch_to_trap_on(&mut self, ovf: Value) {
        let trap = self.ensure_trap_block();
        let cont = self.fb.create_block();
        self.fb.ins().brif(ovf, trap, &[], cont, &[]);
        self.fb.seal_block(cont);
        self.fb.switch_to_block(cont);
    }

    fn checked_mul(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let a64 = self.fb.ins().sextend(types::I64, a);
        let b64 = self.fb.ins().sextend(types::I64, b);
        let r64 = self.fb.ins().imul(a64, b64);
        let r32 = self.fb.ins().ireduce(types::I32, r64);
        let back = self.fb.ins().sextend(types::I64, r32);
        let ovf = self.icmp_bool(IntCC::NotEqual, r64, back);
        self.branch_to_trap_on(ovf);
        r32
    }

    fn checked_sub(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let diff = self.fb.ins().isub(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let room_pos = self.fb.ins().iadd(max, b);
        let b_nonpos = self.icmp_bool(IntCC::SignedLessThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonpos, a_gt_room);
        let room_neg = self.fb.ins().iadd(min, b);
        let b_pos = self.icmp_bool(IntCC::SignedGreaterThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_pos, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        diff
    }

    fn checked_neg(&mut self, a: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let is_min = self.icmp_bool(IntCC::Equal, a, min);
        self.branch_to_trap_on(is_min);
        self.fb.ins().ineg(a)
    }

    fn guarded_sdiv(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().sdiv(a, safe);
        let neg = self.fb.ins().ineg(a);
        let q = self.fb.ins().select(bm1, neg, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        q
    }

    fn guarded_srem(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().srem(a, safe);
        let r = self.fb.ins().select(bm1, zero, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        r
    }
}

/// Borrow the (ptr, len) parts of a text value (panics on scalars — the
/// verifier types every text operation's operands as text).
fn text_parts(v: &TVal) -> (Value, Value) {
    // (record arms never reach the text helpers; the emitter loads field
    // views into TVal::T directly)
    match v {
        TVal::T { ptr, len, .. } => (*ptr, *len),
        TVal::I(_) | TVal::R { .. } => unreachable!("verifier: text operand is a text value"),
    }
}
