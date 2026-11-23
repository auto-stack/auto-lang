//! PLAN-976 T-03: the UI-neutral fragment interface (`emit_stmt_fragment` /
//! `emit_expr_fragment`, frozen in plan §5 I.2).
//!
//! The consumer (PLAN-980 replacing the four legacy `ts_adapter.rs` /
//! `ark_adapter.rs` delegation sites) supplies:
//!
//! * a `SemanticView` + the `FnFacts` of the body the fragment lives in
//!   (produced by the SAME checked frontend — no second parser, no legacy
//!   `TypeScriptTrans` callback);
//! * the fragment locator: `(block, stmt index)` for a statement, an
//!   `ExprId` (see [`innermost_expr_in_span`]) for an expression;
//! * the explicit program context (scalar enum set, struct set, unit variant
//!   set) the host declares around the fragment.
//!
//! The interface returns the emitted statement/expression text, the runtime
//! helper names the text references (host imports them from `a2x_rt.mjs` or
//! renders the seed helpers), and the local-name map so the host can align
//! its own bindings with the deterministic `p<i>_*` / `l<i>_*` / `env.e<i>_*`
//! names. Function-level `try/finally` cleanup is the HOST's duty (the
//! fragment emits only the semantics of its own stmt/expr); reactive state,
//! captures and host-variable substitution stay in the UI adapter — this
//! interface emits general Auto semantics only (the frozen boundary).

use ace_a2x_core::{A2xReject, FnFacts, SemanticView, Ty};
use ace_core::hir;

use crate::lower::{lower_single_fn_in_host, prepare_body_cx, LowerFamily};
use crate::target::ScriptLang;

/// The explicit program context the host declares around the fragment
/// (scalar enum set, struct set, unit variant set — plan §5 I.2).
#[derive(Clone, Debug, Default)]
pub struct FragmentProgramContext {
    pub scalar_enums: Vec<String>,
    pub structs: Vec<String>,
    pub unit_variants: Vec<String>,
}

impl FragmentProgramContext {
    /// Derive the context from the view itself (the names the checked module
    /// declares). Hosts that splice into a program with MORE declarations
    /// pass an explicit context instead.
    pub fn from_view(view: &SemanticView<'_>) -> Self {
        let mut ctx = FragmentProgramContext::default();
        for r in view.records() {
            ctx.structs.push(r.name.to_string());
        }
        for v in view.variants() {
            ctx.scalar_enums.push(v.name.to_string());
            for arm in &v.arms {
                if arm.payload.is_none() {
                    ctx.unit_variants.push(arm.name.to_string());
                }
            }
        }
        ctx
    }

    fn all_names(&self) -> Vec<&str> {
        self.scalar_enums
            .iter()
            .chain(self.structs.iter())
            .chain(self.unit_variants.iter())
            .map(|s| s.as_str())
            .collect()
    }
}

/// One local the fragment text may reference (host alignment map).
#[derive(Clone, Debug)]
pub struct FragmentLocal {
    /// Index into the body's locals table (stable locator).
    pub index: u32,
    /// The emitted name (`p<i>_*` params, `env.e<i>_*` captures, `l<i>_*`
    /// bindings) the fragment text actually uses.
    pub emitted_name: String,
    /// The source name of the local (for host-side binding alignment).
    pub source_name: String,
    /// The resolved type of the local.
    pub ty: Ty,
    /// `param` / `capture` / `binding`.
    pub role: &'static str,
}

/// The result of one fragment emission.
#[derive(Clone, Debug)]
pub struct FragmentOut {
    /// The emitted fragment: for a statement, its lines (each a complete
    /// statement line); for an expression, the helper lines followed by the
    /// expression text itself as the LAST line (splice as `const v = <expr>;`
    /// or equivalent).
    pub text: Vec<String>,
    /// The expression text when this is an expression fragment (empty for
    /// statement fragments).
    pub expr: Option<String>,
    /// Runtime helper names referenced by the text (`a2x_*`; seed helpers
    /// like record clone/free included). The host must provide every one.
    pub helpers: Vec<String>,
    /// The locals the fragment's body context defines (name map).
    pub locals: Vec<FragmentLocal>,
}

/// Extract every `a2x_*` identifier from the emitted text, in first-use order
/// (the host's import/render list; seed helpers match the `a2x_` prefix too).
fn collect_helpers(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |id: &str| {
        if id.starts_with("a2x_") && !out.iter().any(|h| h == id) {
            out.push(id.to_string());
        }
    };
    for line in lines {
        let mut cur = String::new();
        for ch in line.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                cur.push(ch);
            } else {
                if !cur.is_empty() {
                    push(&cur);
                    cur.clear();
                }
            }
        }
        if !cur.is_empty() {
            push(&cur);
        }
    }
    out
}

/// Validate that the program-level names the fragment's body references are
/// covered: a record/variant used by the body must exist in the view (the
/// type authority) AND be named in the host context — a host context that
/// omits a construct the fragment emits helpers for is a named rejection,
/// never a silent splice that will fail at runtime.
fn validate_context(
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    ctx: &FragmentProgramContext,
) -> Result<(), A2xReject> {
    let names = ctx.all_names();
    // Every record/variant type this body's locals carry must be declared in
    // the host context (by name): a host that omits a construct the fragment
    // emits helpers for is a named rejection, never a silent splice.
    for l in &facts.body.locals {
        let Ok(ty) = view.resolve_use_ty(l.ty) else {
            continue;
        };
        let declared = match ty {
            Ty::Record(idx) => view.record(idx).map(|r| r.name.to_string()),
            Ty::Variant(idx) => view.variant(idx).map(|v| v.name.to_string()),
            _ => continue,
        };
        if let Some(name) = declared {
            if !names.contains(&name.as_str()) {
                return Err(A2xReject::new(
                    "a2ts.fragment-context",
                    format!(
                        "fragment references `{name}` which the host program context does not declare (scalar_enums/structs/unit_variants)"
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Emit ONE statement of a checked body as host-splicable script text.
///
/// `block`/`idx` locate the statement (`block` is the `BlockId` number,
/// `idx` its position in that block's statement list).
pub fn emit_stmt_fragment(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    block: u32,
    idx: usize,
    ctx: &FragmentProgramContext,
) -> Result<FragmentOut, A2xReject> {
    // Foreign/scheduler rejections apply to fragments exactly as to whole
    // functions (named, never silent).
    validate_view_admission(view)?;
    validate_context(view, facts, ctx)?;
    let family = LowerFamily::for_profile(view.profile);
    let stmts = facts
        .body
        .blocks
        .get(block as usize)
        .ok_or_else(|| A2xReject::new("a2ts.fragment", "block index out of range"))?
        .stmts
        .clone();
    let stmt = stmts
        .get(idx)
        .ok_or_else(|| A2xReject::new("a2ts.fragment", "statement index out of range"))?;
    let mut bc = prepare_body_cx(lang, view, facts, family, false)?;
    bc.cx.lower_stmt(block, idx, stmt)?;
    let locals = fragment_locals(&bc, view);
    let mut lines = std::mem::take(&mut bc.cx.out);
    let helpers = collect_helpers(&lines);
    lines = lines
        .into_iter()
        .map(|l| l.trim_start().to_string())
        .collect();
    Ok(FragmentOut {
        text: lines,
        expr: None,
        helpers,
        locals,
    })
}

/// Emit a whole checked function body, including local declarations and
/// ownership cleanup. The host supplies the enclosing function signature.
pub fn emit_function_body_fragment(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    ctx: &FragmentProgramContext,
) -> Result<FragmentOut, A2xReject> {
    emit_function_body_in_host(lang, view, facts, ctx, false)
}

/// Emit for the checked Vue widget host. A body marked by
/// `SemanticView::age_http_suspends` requires a generator function wrapper.
pub fn emit_widget_function_body_fragment(
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    ctx: &FragmentProgramContext,
) -> Result<FragmentOut, A2xReject> {
    emit_function_body_in_host(ScriptLang::JavaScript, view, facts, ctx, true)
}

fn emit_function_body_in_host(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    ctx: &FragmentProgramContext,
    widget_http: bool,
) -> Result<FragmentOut, A2xReject> {
    validate_view_admission(view)?;
    validate_context(view, facts, ctx)?;
    let family = LowerFamily::for_profile(view.profile);
    let text = lower_single_fn_in_host(lang, view, facts, family, widget_http)?.body;
    let names = prepare_body_cx(lang, view, facts, family, false)?;
    Ok(FragmentOut {
        helpers: collect_helpers(&text),
        locals: fragment_locals(&names, view),
        expr: None,
        text,
    })
}

/// Emit ONE expression of a checked body as host-splicable script text.
///
/// `expr` is the `ExprId` (see [`innermost_expr_in_span`]); the returned
/// `expr` string is the expression text and `text` carries any helper lines
/// emitted before it.
pub fn emit_expr_fragment(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    expr: hir::ExprId,
    ctx: &FragmentProgramContext,
) -> Result<FragmentOut, A2xReject> {
    validate_view_admission(view)?;
    validate_context(view, facts, ctx)?;
    if facts.body.exprs.get(expr.0 as usize).is_none() {
        return Err(A2xReject::new(
            "a2ts.fragment",
            "expression index out of range",
        ));
    }
    let family = LowerFamily::for_profile(view.profile);
    let mut bc = prepare_body_cx(lang, view, facts, family, false)?;
    let value = bc.cx.lower_expr(expr)?;
    let locals = fragment_locals(&bc, view);
    let mut lines = std::mem::take(&mut bc.cx.out);
    let mut helpers = collect_helpers(&lines);
    for h in collect_helpers(std::slice::from_ref(&value)) {
        if !helpers.iter().any(|x| *x == h) {
            helpers.push(h);
        }
    }
    lines = lines
        .into_iter()
        .map(|l| l.trim_start().to_string())
        .collect();
    Ok(FragmentOut {
        text: lines,
        expr: Some(value),
        helpers,
        locals,
    })
}

/// The innermost expression whose span covers `range` (host locator: the UI
/// template knows the expression's source byte range, not HIR ids).
pub fn innermost_expr_in_span(
    facts: &FnFacts<'_>,
    range: std::ops::Range<usize>,
) -> Option<hir::ExprId> {
    let mut best: Option<(usize, hir::ExprId)> = None;
    for (i, e) in facts.body.exprs.iter().enumerate() {
        let s = e.span.start;
        let en = e.span.end;
        if s <= range.start && range.end <= en {
            let width = en.saturating_sub(s);
            match best {
                Some((w, _)) if w <= width => {}
                _ => best = Some((width, hir::ExprId(i as u32))),
            }
        }
    }
    best.map(|(_, id)| id)
}

fn fragment_locals<'v, 'a>(
    bc: &crate::lower::BodyCx<'v, 'a>,
    view: &'v SemanticView<'a>,
) -> Vec<FragmentLocal> {
    let body = bc.facts().body;
    let mut out = Vec::new();
    for (i, l) in body.locals.iter().enumerate() {
        let role = match l.role {
            hir::LocalRole::Param => "param",
            hir::LocalRole::Capture => "capture",
            _ => "binding",
        };
        if let Some(name) = bc.cx.local_names.get(&(i as u32)).cloned() {
            out.push(FragmentLocal {
                index: i as u32,
                emitted_name: name,
                source_name: l.name.clone(),
                ty: view.resolve_use_ty(l.ty).unwrap_or_else(|_| Ty::I32),
                role,
            });
        }
    }
    out
}

/// The whole-function view admission rejections (foreign / scheduler), shared
/// with the provider path — fragments must reject the SAME surfaces by name.
fn validate_view_admission(view: &SemanticView<'_>) -> Result<(), A2xReject> {
    if let Some(decl) = view.foreign_declarations().next() {
        return Err(A2xReject::new(
            "a2ts.foreign-unsupported",
            format!(
                "foreign native declaration `{}` is outside the TypeScript/JavaScript target domain",
                decl.name
            ),
        ));
    }
    Ok(())
}
