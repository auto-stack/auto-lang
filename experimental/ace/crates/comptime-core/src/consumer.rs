//! Consumer bidirectional binding (contract-decision §5): the let-anchor
//! scheme. Each `CteBindingV1` names (owner fn, binding name); the
//! semantic leg substitutes an `InputExpr` literal in the projected
//! `SemaInput` and re-infers, the HIR leg replaces the Let's initializer
//! expr with a `Constant` / `0 < n` bool subtree and re-verifies. Both
//! legs must derive the declared expected type.

use ace_comptime_contract::{CteAdmissionError, CteBindingV1, CteCode, CteExpectType, CteValue};
use ace_core::hir::{self, Bundle, DefKind, Expr, ExprId, ExprKind, LocalId, Stmt, TypeUse};
use ace_core::source::{compile_parts_with, SourceProfile};
use ace_core::verify::{self, CheckedModule};
use ace_semantics::{InferOutcome, InferSession, InputExpr, InputExprKind, InputStmt, SemaInput};
use std::collections::BTreeMap;

/// A binding resolved against one concrete `SemaInput`.
#[derive(Clone, Debug)]
pub struct ResolvedBinding {
    pub binding: CteBindingV1,
}

// ---------------------------------------------------------------------------
// statement-path addressing over the 796 input model (iterative walks)
// ---------------------------------------------------------------------------

fn stmt_at_mut<'a>(
    stmts: &'a mut [InputStmt],
    want: usize,
    path: &mut usize,
) -> Option<&'a mut InputStmt> {
    for s in stmts.iter_mut() {
        let here = *path;
        *path += 1;
        if here == want {
            return Some(s);
        }
        match s {
            InputStmt::If { then, els, .. } => {
                if let Some(x) = stmt_at_mut(&mut then.stmts, want, path) {
                    return Some(x);
                }
                if let Some(e) = els {
                    if let Some(x) = stmt_at_mut(&mut e.stmts, want, path) {
                        return Some(x);
                    }
                }
            }
            InputStmt::While { body, .. } | InputStmt::Loop { body, .. } => {
                if let Some(x) = stmt_at_mut(&mut body.stmts, want, path) {
                    return Some(x);
                }
            }
            _ => {}
        }
    }
    None
}

/// (fn idx, statement path) of the unique Let anchor, validating the
/// placeholder literal when `check_placeholder` is set.
fn locate_anchor(
    input: &SemaInput,
    owner: &str,
    binding_name: &str,
    placeholder: Option<&CteValue>,
) -> Result<(usize, usize), CteAdmissionError> {
    let fidx = input
        .fns
        .iter()
        .position(|f| f.name == owner)
        .ok_or_else(|| {
            CteAdmissionError::new(
                CteCode::BindingUnknownOwner,
                format!("binding owner fn `{owner}` is not in the projected input"),
            )
        })?;
    // One pre-order walk collects every matching anchor: (path, literal).
    fn collect<'a>(
        stmts: &'a [InputStmt],
        path: &mut usize,
        binding_name: &str,
        out: &mut Vec<(usize, Option<CteValue>)>,
    ) {
        for s in stmts {
            let here = *path;
            *path += 1;
            if let InputStmt::Let { name, value, .. } = s {
                if name == binding_name {
                    out.push((here, lit_of(value)));
                }
            }
            match s {
                InputStmt::If { then, els, .. } => {
                    collect(&then.stmts, path, binding_name, out);
                    if let Some(e) = els {
                        collect(&e.stmts, path, binding_name, out);
                    }
                }
                InputStmt::While { body, .. } | InputStmt::Loop { body, .. } => {
                    collect(&body.stmts, path, binding_name, out);
                }
                _ => {}
            }
        }
    }
    let mut anchors: Vec<(usize, Option<CteValue>)> = Vec::new();
    let mut p = 0usize;
    collect(
        &input.fns[fidx].body.stmts,
        &mut p,
        binding_name,
        &mut anchors,
    );
    if anchors.is_empty() {
        return Err(CteAdmissionError::new(
            CteCode::BindingAmbiguous,
            format!("binding `{binding_name}` has no let anchor in `{owner}`"),
        ));
    }
    if anchors.len() > 1 {
        return Err(CteAdmissionError::new(
            CteCode::BindingAmbiguous,
            format!(
                "binding `{binding_name}` matches {} let anchors in `{owner}` (shadowing is not an address)",
                anchors.len()
            ),
        ));
    }
    let (path, found) = anchors[0].clone();
    if let Some(want) = placeholder {
        let found = found.ok_or_else(|| {
            CteAdmissionError::new(
                CteCode::BindingPlaceholderMismatch,
                format!(
                    "binding `{binding_name}` in `{owner}` is not initialized by a literal (placeholder contract)"
                ),
            )
        })?;
        if &found != want {
            return Err(CteAdmissionError::new(
                CteCode::BindingPlaceholderMismatch,
                format!(
                    "binding `{binding_name}` source placeholder {found:?} != declared {want:?} (source/binding drift)"
                ),
            ));
        }
    }
    Ok((fidx, path))
}

fn lit_of(e: &InputExpr) -> Option<CteValue> {
    match &e.kind {
        InputExprKind::IntLit(v) => Some(CteValue::I32(*v)),
        InputExprKind::BoolLit(v) => Some(CteValue::Bool(*v)),
        _ => None,
    }
}

/// Resolve the whole binding table against the projected input:
/// duplicate ct_refs, unknown owners, ambiguous anchors and placeholder
/// drift are all controlled errors BEFORE any value flows.
pub fn resolve_bindings(
    input: &SemaInput,
    bindings: &[CteBindingV1],
) -> Result<Vec<ResolvedBinding>, CteAdmissionError> {
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for b in bindings {
        if !seen.insert(b.ct_ref.as_str()) {
            return Err(CteAdmissionError::new(
                CteCode::BindingDuplicate,
                format!("ct_ref `{}` is bound twice", b.ct_ref),
            ));
        }
        locate_anchor(input, &b.owner_fn, &b.binding_name, Some(&b.placeholder))?;
        out.push(ResolvedBinding { binding: b.clone() });
    }
    Ok(out)
}

/// Semantic leg: substitute the CT values as `InputExpr` literals at the
/// anchors and re-infer each owner. Returns (ct_ref, inferred type name)
/// pairs — "i32"/"bool" on success.
pub fn substitute_and_infer(
    input: &mut SemaInput,
    resolved: &[ResolvedBinding],
    values: &BTreeMap<String, CteValue>,
) -> Result<Vec<(String, String)>, CteAdmissionError> {
    for r in resolved {
        let Some(value) = values.get(&r.binding.ct_ref) else {
            return Err(CteAdmissionError::new(
                CteCode::Infra("missing-value".into()),
                format!(
                    "internal: substitution without a value for `{}`",
                    r.binding.ct_ref
                ),
            ));
        };
        let (fidx, path) =
            locate_anchor(input, &r.binding.owner_fn, &r.binding.binding_name, None)?;
        let mut fns = std::mem::take(&mut input.fns);
        let stmt = stmt_at_mut(&mut fns[fidx].body.stmts, path, &mut 0).ok_or_else(|| {
            CteAdmissionError::new(CteCode::Infra("anchor".into()), "anchor path lost")
        })?;
        if let InputStmt::Let { value: v, .. } = stmt {
            *v = match value {
                CteValue::I32(x) => InputExpr::int(*x),
                CteValue::Bool(x) => InputExpr::boolean(*x),
            };
        }
        input.fns = fns;
    }
    let owners: Vec<String> = resolved
        .iter()
        .map(|r| r.binding.owner_fn.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut derived: Vec<(String, String)> = Vec::new();
    for owner in owners {
        let mut session = InferSession::new(input);
        match session.infer_function(&owner) {
            InferOutcome::Ok(summary) => {
                for r in resolved.iter().filter(|r| r.binding.owner_fn == owner) {
                    let ty = summary
                        .facts
                        .bindings
                        .iter()
                        .find(|(n, _)| n == &r.binding.binding_name)
                        .map(|(_, t)| t.clone())
                        .ok_or_else(|| {
                            CteAdmissionError::new(
                                CteCode::BindingAmbiguous,
                                format!(
                                    "infer produced no fact for `{}` in `{}`",
                                    r.binding.binding_name, owner
                                ),
                            )
                        })?;
                    let ty_name = match ty {
                        ace_semantics_contract::TypeDesc::I32 => "i32".to_string(),
                        ace_semantics_contract::TypeDesc::Bool => "bool".to_string(),
                        other => format!("non-scalar:{other:?}"),
                    };
                    derived.push((r.binding.ct_ref.clone(), ty_name));
                }
            }
            InferOutcome::Diags(summary) => {
                let first = summary
                    .facts
                    .diagnostics
                    .first()
                    .map(|d| format!("{} {}", d.code, d.message))
                    .unwrap_or_else(|| "diags".into());
                return Err(CteAdmissionError::new(
                    CteCode::BindingTypeMismatch,
                    format!("infer after substitution reported diagnostics: {first}"),
                ));
            }
            InferOutcome::ResourceExhausted(_) => {
                return Err(CteAdmissionError::new(
                    CteCode::Resource("infer".into()),
                    "semantic inference budget exhausted",
                ));
            }
            InferOutcome::Cancelled => {
                return Err(CteAdmissionError::new(
                    CteCode::Cancelled,
                    "infer cancelled",
                ));
            }
        }
    }
    Ok(derived)
}

// ---------------------------------------------------------------------------
// HIR leg
// ---------------------------------------------------------------------------

/// (module idx, body idx, local, old value expr) of the unique anchor.
#[derive(Clone, Copy)]
struct HirAnchor {
    module: usize,
    body: usize,
    _local: LocalId,
    expr: ExprId,
}

fn locate_hir_anchor(
    bundle: &Bundle,
    owner: &str,
    binding: &str,
) -> Result<HirAnchor, CteAdmissionError> {
    for (mi, m) in bundle.modules.iter().enumerate() {
        for d in m.defs.iter() {
            // The Atom emitter gives defs `d_`-prefixed id_texts
            // (`d_main`); `Function.name` is an `sf_` frontend-internal
            // symbol. The binding table addresses the source-level name,
            // so match the id_text convention (`d_` + name).
            if d.id_text != format!("d_{owner}") {
                continue;
            }
            let DefKind::Function { body, .. } = &d.kind else {
                continue;
            };
            let body_idx = body.0 as usize;
            let b = &m.bodies[body_idx];
            let locals: Vec<LocalId> = b
                .locals
                .iter()
                .enumerate()
                .filter(|(_, l)| l.name == binding)
                .map(|(i, _)| LocalId(i as u32))
                .collect();
            if locals.len() != 1 {
                return Err(CteAdmissionError::new(
                    CteCode::BindingAmbiguous,
                    format!(
                        "binding `{binding}` resolves to {} locals in HIR `{owner}`",
                        locals.len()
                    ),
                ));
            }
            let mut hits: Vec<ExprId> = Vec::new();
            for blk in &b.blocks {
                for s in &blk.stmts {
                    if let Stmt::Let { local: l, value } = s {
                        if *l == locals[0] {
                            hits.push(*value);
                        }
                    }
                }
            }
            if hits.len() != 1 {
                return Err(CteAdmissionError::new(
                    CteCode::BindingAmbiguous,
                    format!(
                        "binding `{binding}` has {} defining lets in HIR `{owner}`",
                        hits.len()
                    ),
                ));
            }
            return Ok(HirAnchor {
                module: mi,
                body: body_idx,
                _local: locals[0],
                expr: hits[0],
            });
        }
    }
    Err(CteAdmissionError::new(
        CteCode::BindingUnknownOwner,
        format!("owner fn `{owner}` not found in the consumer HIR"),
    ))
}

/// HIR leg: compile the source, replace every bound anchor's initializer
/// with the constant subtree (i32 → `Constant{v}`; bool → the adapter's
/// deterministic `0 < 1`/`0 < 0` encoding), then RE-VERIFY the whole
/// bundle. Returns the checked module plus the backfilled constant
/// rendering per ct_ref (observable proof for V-01).
pub fn backfill_and_verify(
    source: &str,
    module_path: &str,
    resolved: &[ResolvedBinding],
    values: &BTreeMap<String, CteValue>,
    profile: ace_comptime_contract::CteSourceProfile,
    entry: &str,
) -> Result<(CheckedModule, BTreeMap<String, String>), CteAdmissionError> {
    let ac_profile = match profile {
        ace_comptime_contract::CteSourceProfile::Core => SourceProfile::Core,
        ace_comptime_contract::CteSourceProfile::Record => SourceProfile::Record,
    };
    // Fast path (R3-01/R3-02): the consumer HIR leg verifies the OWNER
    // functions — the anchors live there, and helper bodies are not part
    // of the consumer deliverable. Compiling only the owner source
    // avoids the whole-source Atom-emit frontend on 256KiB baselines
    // (~0.5s and a ~400MiB transient PER REQUEST). A reduced module that
    // does not compile/verify (e.g. an owner calling a dropped helper)
    // falls back to the FULL source — never a silent scope reduction.
    let owners: Vec<String> = resolved
        .iter()
        .map(|r| r.binding.owner_fn.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if let Some(reduced) = extract_owner_functions(source, &owners, entry) {
        if let Ok(parts) = compile_parts_with(&reduced, module_path, &[], ac_profile) {
            if let Ok(out) = patch_and_verify(parts.bundle, resolved, values, profile) {
                return Ok(out);
            }
        }
    }
    let parts = compile_parts_with(source, module_path, &[], ac_profile).map_err(|ds| {
        let d = ds.first().cloned();
        // A consumer-side source failure is a LANGUAGE diagnostic
        // (parse/resolve stage) — never infra, never silently green.
        CteAdmissionError::new(
            CteCode::ConsumerFrontend,
            format!(
                "consumer frontend: {}",
                d.map(|x| format!("{} {}", x.code, x.message))
                    .unwrap_or_else(|| "failed".into())
            ),
        )
    })?;
    patch_and_verify(parts.bundle, resolved, values, profile)
}

/// Splice the CT constants into one compiled bundle's anchors and
/// re-verify it.
fn patch_and_verify(
    mut bundle: Bundle,
    resolved: &[ResolvedBinding],
    values: &BTreeMap<String, CteValue>,
    profile: ace_comptime_contract::CteSourceProfile,
) -> Result<(CheckedModule, BTreeMap<String, String>), CteAdmissionError> {
    let mut proof: BTreeMap<String, String> = BTreeMap::new();

    for r in resolved {
        let Some(value) = values.get(&r.binding.ct_ref) else {
            return Err(CteAdmissionError::new(
                CteCode::Infra("missing-value".into()),
                format!(
                    "internal: backfill without a value for `{}`",
                    r.binding.ct_ref
                ),
            ));
        };
        let anchor = locate_hir_anchor(&bundle, &r.binding.owner_fn, &r.binding.binding_name)?;
        let m = &mut bundle.modules[anchor.module];
        let b = &mut m.bodies[anchor.body];
        let span = b.exprs[anchor.expr.0 as usize].span;
        // Replace the placeholder expr IN PLACE (the verifier's
        // single-owning-reference graph must not see an orphaned old
        // node). An i32 placeholder is a single Constant; a bool
        // placeholder is the adapter's own `0 < n` Binary — its operand
        // slots are REUSED so nothing is orphaned.
        let old = &b.exprs[anchor.expr.0 as usize];
        let render = match (&old.kind, value) {
            (ExprKind::Constant { .. }, CteValue::I32(v)) => {
                let v = *v;
                b.exprs[anchor.expr.0 as usize] = Expr {
                    id_text: format!("cte817:const:{v}"),
                    ty: TypeUse::Builtin(hir::BuiltinType::I32),
                    kind: ExprKind::Constant { value: v },
                    span,
                };
                format!("{v}")
            }
            (
                ExprKind::Binary {
                    op: hir::BinOp::LtI32,
                    lhs,
                    rhs,
                    ..
                },
                CteValue::Bool(v),
            ) => {
                let (lhs, rhs) = (*lhs, *rhs);
                let new_rhs = if *v { 1 } else { 0 };
                // Operands stay plain Constants (0 and 1|0) — reused in
                // place, still owned by the untouched Binary node.
                b.exprs[lhs.0 as usize].kind = ExprKind::Constant { value: 0 };
                b.exprs[lhs.0 as usize].id_text = "cte817:bool:0".into();
                b.exprs[rhs.0 as usize].kind = ExprKind::Constant { value: new_rhs };
                b.exprs[rhs.0 as usize].id_text = format!("cte817:bool:{new_rhs}");
                format!("0<{new_rhs}")
            }
            (other, v) => {
                return Err(CteAdmissionError::new(
                    CteCode::BindingPlaceholderMismatch,
                    format!(
                        "placeholder expr shape {other:?} cannot host a {}/{:?} value",
                        v.type_name(),
                        v
                    ),
                ));
            }
        };
        proof.insert(r.binding.ct_ref.clone(), render);
    }

    let verify_result = match profile {
        ace_comptime_contract::CteSourceProfile::Core => verify::verify(bundle),
        ace_comptime_contract::CteSourceProfile::Record => verify::verify_record(bundle),
    };
    let checked = verify_result.map_err(|ds| {
        let d = ds.first().cloned();
        CteAdmissionError::new(
            CteCode::ConsumerFrontend,
            format!(
                "backfilled HIR re-verify failed: {}",
                d.map(|x| format!("{} {}", x.code, x.message))
                    .unwrap_or_else(|| "failed".into())
            ),
        )
    })?;
    Ok((checked, proof))
}

/// Extract a REDUCED program source keeping OWNER ∪ ENTRY ∪ the call
/// closure linking them (F-817-R4-04): the native observable runs
/// `entry`, so an owner that is not the entry must never delete the
/// entry's call chain. The reduced set only ever ADDS functions to the
/// owner set — never drops a verified dependency.
fn extract_owner_functions(source: &str, owners: &[String], entry: &str) -> Option<String> {
    if owners.is_empty() {
        return None;
    }
    let mut keep: std::collections::BTreeSet<String> = owners.iter().cloned().collect();
    keep.insert(entry.to_string());
    // Worklist closure over `callee(` call sites found inside kept
    // functions' text (conservative: identifier + '(' anywhere in the
    // body; declarations inside comments cannot exist in this grammar).
    let mut frontier: Vec<String> = keep.iter().cloned().collect();
    while let Some(f) = frontier.pop() {
        let Some(text) = extract_fn_text(source, &f) else {
            // A wanted function that does not extract cleanly (odd
            // formatting): refuse the fast path entirely.
            return None;
        };
        for callee in all_fn_names(source) {
            if callee != f && text.contains(&format!("{callee}(")) && keep.insert(callee.clone()) {
                frontier.push(callee);
            }
        }
    }
    // Emit in SOURCE ORDER (stable, readable diagnostics).
    let order: Vec<(usize, String)> = all_fn_names_with_pos(source)
        .into_iter()
        .filter(|(_, n)| keep.contains(n.as_str()))
        .collect();
    if order.is_empty() {
        return None;
    }
    let mut out = String::new();
    for (_, name) in order {
        let text = extract_fn_text(source, name.as_str())?;
        out.push_str(&text);
        out.push('\n');
    }
    Some(out)
}

/// One function's full text (from `fn NAME(` to its column-0 closing
/// brace, inclusive). `None` when it cannot be found cleanly.
fn extract_fn_text(source: &str, name: &str) -> Option<String> {
    let needle = format!("fn {name}(");
    let mut from = 0usize;
    while let Some(idx) = source[from..].find(&needle) {
        let start = from + idx;
        let line_start = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        if source[line_start..start].trim().is_empty() {
            if let Some(rel) = source[start..].find("\n}") {
                return Some(source[start..start + rel + 2].to_string());
            }
        }
        from = start + needle.len();
    }
    None
}

/// All top-level function names in declaration order.
fn all_fn_names(source: &str) -> Vec<String> {
    all_fn_names_with_pos(source)
        .into_iter()
        .map(|(_, n)| n)
        .collect()
}

/// All top-level function names with their byte positions.
fn all_fn_names_with_pos(source: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut rest = source;
    let mut base = 0usize;
    while let Some(idx) = rest.find("fn ") {
        let at = base + idx;
        let after = at + 3;
        let name: String = source[after..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() && source[after..].starts_with(&name) {
            out.push((at, name));
        }
        let skip = idx + 3;
        rest = &rest[skip..];
        base += skip;
    }
    out
}

/// Splice the CT constants into one compiled bundle's anchors and

/// Extract the named functions from an Auto source into a reduced
/// program source. `None` when any owner cannot be extracted cleanly.
///
/// The reduced set is OWNER ∪ ENTRY ∪ the entry's transitive CALL
/// closure (F-817-R4-04): the native observable runs `entry`, and an
/// owner that is NOT the entry (or an owner the entry calls through)
/// must not silently delete the entry's call chain — the reduced module

/// Expected-type comparison used by the driver.
pub fn type_matches(expect: CteExpectType, derived: &str) -> bool {
    match expect {
        CteExpectType::I32 => derived == "i32",
        CteExpectType::Bool => derived == "bool",
    }
}
