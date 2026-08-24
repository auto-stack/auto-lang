//! Plan 442 A3: web-ecosystem ext imports (`use.web` / widget `use {}` blocks)
//! on the VM render target.
//!
//! ## Problem (musk KNOWN-DEBT 028 ③ "ext link")
//!
//! The VM-render loader ignored `Stmt::UseWeb` entirely: the use scanner
//! produced garbage module strings for `use.web` lines, module resolution
//! failed, and the failure was a silent skip. Pure-Auto symbols imported via
//! `use.web platformInjectStyles from "…/platform.at"` therefore never got a
//! definition, and any handler calling them left an unresolved CALL reloc —
//! surfacing as a hard `LinkError: Undefined symbol` that killed the whole
//! VmBridge init (one missing web-platform helper takes down the entire app).
//!
//! ## Design
//!
//! 1. `.at` ext sources load for the VM target through the port-adapter
//!    chain (`X.at` → `X.vm.at` → `X.web.at`, mirroring auto-man's
//!    `resolve_at_adapter` gating): pure-Auto adapter fns become real module
//!    symbols.
//! 2. Remaining ext symbols (TS/npm sources, or adapters that in turn import
//!    `.ts`) get **platform stubs** by default: no-op Auto fns synthesized
//!    with the arity observed at their call sites (the VM's RET unwinds with
//!    `bp - n_args`, so an arity mismatch corrupts the caller frame — the
//!    arity must be exact). `AUTO_VM_EXT_STUBS=0` restores the strict
//!    hard-link-error behavior for debugging.
//!
//! Stubs are only ever created for symbols that appear in an ext import
//! clause — ordinary unresolved symbols still fail the link, so real errors
//! stay loud.

use crate::ast::{Body, Expr, Stmt};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Whether ext-import platform stubs are enabled. Default ON (web-platform
/// helpers are legitimately absent on a desktop VM target and must not kill
/// the app); `AUTO_VM_EXT_STUBS=0` opts into strict link errors.
pub(crate) fn ext_stubs_enabled() -> bool {
    std::env::var("AUTO_VM_EXT_STUBS").map_or(true, |v| v != "0")
}

/// One ext import entry relevant to the VM loader: (symbols, source path).
#[derive(Debug, Clone)]
pub(crate) struct ExtImportRef {
    pub symbols: Vec<String>,
    pub path: String,
}

/// Collect ext imports from an AST's top-level `use.web` statements.
pub(crate) fn collect_useweb_imports(stmts: &[Stmt]) -> Vec<ExtImportRef> {
    let mut out = Vec::new();
    for stmt in stmts {
        if let Stmt::UseWeb(imports) = stmt {
            for imp in imports {
                out.push(ExtImportRef {
                    symbols: imp.symbols.iter().map(|s| s.to_string()).collect(),
                    path: imp.path.to_string(),
                });
            }
        }
    }
    out
}

/// Collect ext imports from widget-level `use { ... }` blocks.
pub(crate) fn collect_widget_ext_imports(decls: &[crate::ast::ui::WidgetDecl]) -> Vec<ExtImportRef> {
    let mut out = Vec::new();
    for decl in decls {
        for imp in &decl.ext_imports {
            out.push(ExtImportRef {
                symbols: imp.symbols.iter().map(|s| s.to_string()).collect(),
                path: imp.path.to_string(),
            });
        }
    }
    out
}

/// Resolve a `use.web` source path (project-root relative by convention) to
/// a local file: probe the base dir and up to two ancestors (base dir →
/// project root for layouts like `<root>/src/front`), then the cwd.
fn resolve_ext_source(base_dir: &Path, path: &str) -> Option<PathBuf> {
    let p = Path::new(path);
    if p.is_absolute() {
        return if p.is_file() { Some(p.to_path_buf()) } else { None };
    }
    let mut dir = Some(base_dir);
    for _ in 0..3 {
        let Some(d) = dir else { break };
        let cand = d.join(path);
        if cand.is_file() {
            return Some(cand);
        }
        dir = d.parent();
    }
    std::env::current_dir()
        .ok()
        .map(|c| c.join(path))
        .filter(|c| c.is_file())
}

/// Resolve the port-adapter chain for a `.at` ext source on the VM target:
/// `X.vm.at` first, then the web adapter `X.web.at`, then `X.at` itself.
/// Returns the first existing candidate (mirrors auto-man's
/// `resolve_at_adapter` target gating, with web as the fallback since it is
/// the only adapter family that exists before Plan 442 Phase B lands).
pub(crate) fn resolve_vm_at_adapter(resolved: &Path) -> Option<PathBuf> {
    let stem = resolved.file_stem()?.to_str()?;
    for cand in [
        resolved.with_file_name(format!("{}.vm.at", stem)),
        resolved.with_file_name(format!("{}.web.at", stem)),
        resolved.to_path_buf(),
    ] {
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Load `.at` ext sources through the VM adapter chain; TS/npm sources are
/// skipped (their symbols become stub candidates). Returns the adapter files
/// loaded plus every ext import found inside them (for recursive stubbing).
pub(crate) fn load_at_ext_imports(
    base_dir: &Path,
    imports: &[ExtImportRef],
    load_module: &mut dyn FnMut(&Path),
) -> (Vec<PathBuf>, Vec<ExtImportRef>) {
    let mut loaded = Vec::new();
    let mut nested = Vec::new();
    for imp in imports {
        if !imp.path.ends_with(".at") {
            continue; // TS/npm source — stub candidates only
        }
        let Some(resolved) = resolve_ext_source(base_dir, &imp.path) else {
            log::warn!(
                "ext import: source file for {} not found (path {:?})",
                imp.symbols.join(", "),
                imp.path
            );
            continue;
        };
        let adapter = resolve_vm_at_adapter(&resolved).unwrap_or(resolved);
        // Collect nested use.web from the adapter so ITS TS deps stub too.
        if let Ok(code) = std::fs::read_to_string(&adapter) {
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            if let Ok(ast) = parser.parse() {
                nested.extend(collect_useweb_imports(&ast.stmts));
            }
        }
        load_module(&adapter);
        loaded.push(adapter);
    }
    (loaded, nested)
}

/// Record call arities for the target symbols while walking statements.
/// Missing variants in the walker only risk under-counting a call site;
/// ext symbols called with args in handler bodies are the coverage target.
pub(crate) fn scan_call_arities(
    stmts: &[Stmt],
    targets: &HashSet<String>,
    out: &mut HashMap<String, usize>,
) {
    scan_stmts(stmts, targets, out);
}

/// Scan a widget decl's executable bodies (on-handlers, lifecycle, setup,
/// computed) for ext call sites.
pub(crate) fn scan_decl_arities(
    decls: &[crate::ast::ui::WidgetDecl],
    targets: &HashSet<String>,
    out: &mut HashMap<String, usize>,
) {
    if targets.is_empty() {
        return;
    }
    for decl in decls {
        if let Some(ref on) = decl.on {
            for handler in &on.handlers {
                scan_body(&handler.body, targets, out);
            }
        }
        for lc in &decl.lifecycle {
            scan_stmts(&lc.body, targets, out);
        }
        if let Some(ref setup) = decl.setup {
            scan_body(&setup.body, targets, out);
        }
        if let Some(ref computed) = decl.computed {
            for c in &computed.properties {
                scan_expr(&c.expr, targets, out);
            }
        }
    }
}

fn note_call(call: &crate::ast::Call, targets: &HashSet<String>, out: &mut HashMap<String, usize>) {
    if let Some(name) = call.get_name_text_safe() {
        let name = name.to_string();
        if targets.contains(&name) {
            let arity = call.args.args.len();
            out.entry(name).and_modify(|n| *n = (*n).max(arity)).or_insert(arity);
        }
    }
}

fn scan_body(body: &Body, targets: &HashSet<String>, out: &mut HashMap<String, usize>) {
    scan_stmts(&body.stmts, targets, out);
}

fn scan_stmts(stmts: &[Stmt], targets: &HashSet<String>, out: &mut HashMap<String, usize>) {
    for stmt in stmts {
        scan_stmt(stmt, targets, out);
    }
}

fn scan_stmt(stmt: &Stmt, targets: &HashSet<String>, out: &mut HashMap<String, usize>) {
    match stmt {
        Stmt::Expr(e) => scan_expr(e, targets, out),
        Stmt::Store(s) => scan_expr(&s.expr, targets, out),
        Stmt::Return(e) | Stmt::Reply(e) => scan_expr(e, targets, out),
        Stmt::If(i) => {
            for branch in &i.branches {
                scan_expr(&branch.cond, targets, out);
                scan_body(&branch.body, targets, out);
            }
            if let Some(ref else_body) = i.else_ {
                scan_body(else_body, targets, out);
            }
        }
        Stmt::For(f) => {
            scan_expr(&f.range, targets, out);
            if let Some(ref init) = f.init {
                scan_stmt(init, targets, out);
            }
            scan_body(&f.body, targets, out);
        }
        Stmt::Try(t) => {
            scan_body(&t.body, targets, out);
            scan_body(&t.catch_body, targets, out);
            if let Some(ref fb) = t.finally_body {
                scan_body(fb, targets, out);
            }
        }
        Stmt::Block(b) => scan_body(b, targets, out),
        Stmt::Fn(f) => scan_body(&f.body, targets, out),
        Stmt::WidgetDecl(d) => {
            let decls = std::slice::from_ref(d);
            scan_decl_arities(decls, targets, out);
        }
        Stmt::StoreDecl(s) => {
            if let Some(ref on) = s.on {
                for handler in &on.handlers {
                    scan_body(&handler.body, targets, out);
                }
            }
            if let Some(ref computed) = s.computed {
                for c in &computed.properties {
                    scan_expr(&c.expr, targets, out);
                }
            }
        }
        _ => {}
    }
}

fn scan_expr(e: &Expr, targets: &HashSet<String>, out: &mut HashMap<String, usize>) {
    match e {
        Expr::Call(call) => {
            note_call(call, targets, out);
            for arg in &call.args.args {
                scan_expr(&arg.get_expr(), targets, out);
            }
            scan_expr(&call.name, targets, out);
        }
        Expr::Unary(_, a) => scan_expr(a, targets, out),
        Expr::Bina(a, _, b) => {
            scan_expr(a, targets, out);
            scan_expr(b, targets, out);
        }
        Expr::Dot(a, _) => scan_expr(a, targets, out),
        Expr::Index(a, b) => {
            scan_expr(a, targets, out);
            scan_expr(b, targets, out);
        }
        Expr::Array(elems) => {
            for el in elems {
                scan_expr(el, targets, out);
            }
        }
        Expr::Object(pairs) => {
            for p in pairs {
                scan_expr(&p.value, targets, out);
            }
        }
        Expr::Pair(p) => scan_expr(&p.value, targets, out),
        Expr::Block(b) => scan_body(b, targets, out),
        Expr::Some(a) | Expr::Ok(a) | Expr::Err(a) => scan_expr(a, targets, out),
        Expr::View(a) | Expr::Mut(a) | Expr::Move(a) | Expr::Take(a) | Expr::ErrorPropagate(a)
        | Expr::BoxExpr(a) | Expr::ArcExpr(a) | Expr::Yield(a) => scan_expr(a, targets, out),
        Expr::Await { expr } => scan_expr(expr, targets, out),
        Expr::NullCoalesce(a, b) => {
            scan_expr(a, targets, out);
            scan_expr(b, targets, out);
        }
        Expr::Tuple(elems) => {
            for el in elems {
                scan_expr(el, targets, out);
            }
        }
        Expr::TupleDestruct { expr, .. } => scan_expr(expr, targets, out),
        Expr::Cast { expr, .. } | Expr::To { expr, .. } => scan_expr(expr, targets, out),
        Expr::If(i) => {
            for branch in &i.branches {
                scan_expr(&branch.cond, targets, out);
                scan_body(&branch.body, targets, out);
            }
            if let Some(ref else_body) = i.else_ {
                scan_body(else_body, targets, out);
            }
        }
        _ => {}
    }
}

/// Synthesize a no-op platform stub fn for an ext symbol. Param types are
/// `Unknown` (the VM is dynamically typed; the declared type is not enforced
/// at the call boundary) and the arity comes from `scan_call_arities`.
pub(crate) fn synthesize_stub_fn(name: &str, arity: usize) -> Stmt {
    use crate::ast::{Fn, FnKind, Name as FnName, Param};
    let params = (0..arity)
        .map(|i| Param {
            name: FnName::from(format!("__stub_arg_{}", i)),
            ty: crate::ast::Type::Unknown,
            default: None,
            mode: Default::default(),
            destructure: None,
        })
        .collect();
    let f = Fn {
        kind: FnKind::Function,
        name: FnName::from(name),
        parent: None,
        params,
        body: Body::new(),
        ret: crate::ast::Type::Void,
        ret_name: None,
        is_static: false,
        is_pub: true,
        is_mut: false,
        is_test: false,
        type_params: vec![],
        const_params: Vec::new(),
        doc: Some(
            format!(
                "Plan 442 A3 platform stub: ext import `{}` has no VM-target implementation (web-ecosystem source); no-op on this platform.",
                name
            )
            .into(),
        ),
        span: None,
        api_attrs: None,
        attrs: Vec::new(),
    };
    Stmt::Fn(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_mode_defaults_on_and_env_overrides() {
        // Default: enabled (absent env). Note: env-var tests are
        // process-global; only assert the non-mutating default here.
        std::env::remove_var("AUTO_VM_EXT_STUBS");
        assert!(ext_stubs_enabled());
    }

    #[test]
    fn stub_fn_shape() {
        match synthesize_stub_fn("platformInjectStyles", 0) {
            Stmt::Fn(f) => {
                assert_eq!(f.name.as_str(), "platformInjectStyles");
                assert_eq!(f.params.len(), 0);
            }
            other => panic!("expected Fn, got {:?}", other),
        }
        match synthesize_stub_fn("resize", 2) {
            Stmt::Fn(f) => assert_eq!(f.params.len(), 2),
            other => panic!("expected Fn, got {:?}", other),
        }
    }
}
