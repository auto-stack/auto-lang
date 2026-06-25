//! Plan 323 (Option B): synthesize widget handlers as REAL AutoVM functions.
//!
//! Each widget handler becomes a real `fn handler_<Name>(__state AppState, params...)`
//! compiled by the genuine VM `Codegen` (the same compiler the non-UI `run()` path
//! uses). State-field references inside the handler body (`.field` parsed as
//! `Expr::Ident(field)` or `Expr::Dot(self|., field)`) are rewritten to
//! `__state.field`, which Codegen lowers to a name-based `GET_FIELD` / `SET_FIELD`
//! against the state heap object (a `GenericInstanceData` whose id the dispatcher
//! pushes as the first argument — see `VmBridge::call_handler`).
//!
//! Imports (e.g. `build_month_grid`) + the synthesized `type AppState` + the
//! handler fns are all fed to ONE `Codegen` pass, yielding a single `Module`
//! with unified `strings` / `object_keys` / `object_types`. This dissolves the
//! cross-module table-relocation risk that a multi-module `Linker` merge would
//! introduce, and replaces the bespoke mini-compiler + AST tree-walker that
//! stalled mid-Plan-205-migration.

use std::collections::HashSet;

use crate::ast::{
    Arg, Body, Branch, Expr, Fn, FnKind, If, Member, Name, Param, Stmt, Type, TypeDecl,
    TypeDeclKind,
};
use crate::aura::{AuraWidget, LogicPayload};
use crate::vm::codegen::Codegen;
use crate::vm::loader::Module;

/// The synthesized receiver parameter name holding the widget-state heap id.
const STATE_PARAM: &str = "__state";

/// Plan 333: the exported fn that initializes imported module-level globals
/// (`var notes = ...` etc.). VmBridge runs it once before `.Init` so the
/// globals have defined values when handlers (e.g. db.all_notes → notes) read them.
pub const MODULE_INIT_FN: &str = "__module_init";

/// Error type for handler synthesis.
pub type SynthResult<T> = Result<T, String>;

/// Rewrite every widget-state reference in a list of statements to
/// `__state.<field>`, in place.
///
/// A "state reference" is either a bare `Expr::Ident(field)` whose name is one of
/// the widget's state fields (how `.field` reads parse), or a
/// `Expr::Dot(self|., field)`. Both become `Expr::Dot(Ident("__state"), field)`.
/// This transparently covers assignment LHS too, because `a = b` parses as
/// `Expr::Bina(lhs, Op::Asn, rhs)` with `lhs` an `Expr`.
pub fn rewrite_state_refs_stmts(stmts: &mut [Stmt], state_fields: &HashSet<String>) {
    for stmt in stmts.iter_mut() {
        rewrite_stmt(stmt, state_fields);
    }
}

fn rewrite_stmt(stmt: &mut Stmt, state_fields: &HashSet<String>) {
    match stmt {
        Stmt::Expr(e) => rewrite_expr(e, state_fields),
        Stmt::Store(s) => rewrite_expr(&mut s.expr, state_fields),
        Stmt::Return(e) | Stmt::Reply(e) => rewrite_expr(e, state_fields),
        Stmt::If(If { branches, else_ }) => {
            for Branch { cond, body } in branches.iter_mut() {
                rewrite_expr(cond, state_fields);
                rewrite_state_refs_stmts(&mut body.stmts, state_fields);
            }
            if let Some(else_body) = else_ {
                rewrite_state_refs_stmts(&mut else_body.stmts, state_fields);
            }
        }
        Stmt::For(f) => {
            rewrite_expr(&mut f.range, state_fields);
            if let Some(init) = f.init.as_mut() {
                rewrite_stmt(init, state_fields);
            }
            rewrite_state_refs_stmts(&mut f.body.stmts, state_fields);
        }
        Stmt::Block(b) => rewrite_state_refs_stmts(&mut b.stmts, state_fields),
        // Fn bodies inside handlers are unusual; handle defensively.
        Stmt::Fn(fn_decl) => rewrite_state_refs_stmts(&mut fn_decl.body.stmts, state_fields),
        _ => {}
    }
}

fn rewrite_expr(e: &mut Expr, state_fields: &HashSet<String>) {
    // Phase 1: decide whether THIS node is a state reference that needs replacing.
    // Compute the replacement without holding a mutable borrow into `e`, so the
    // reassignment below type-checks.
    let replacement: Option<Expr> = match e {
        Expr::Ident(name) if state_fields.contains(name.as_str()) => Some(Expr::Dot(
            Box::new(Expr::Ident(Name::from(STATE_PARAM))),
            name.clone(),
        )),
        Expr::Dot(obj, field)
            if matches!(
                obj.as_ref(),
                Expr::Ident(n) if n.as_str() == "self" || n.as_str() == "."
            ) =>
        {
            Some(Expr::Dot(
                Box::new(Expr::Ident(Name::from(STATE_PARAM))),
                field.clone(),
            ))
        }
        // A method call whose receiver is a bare state-field ident, e.g.
        // `notes.remove(1)` (parsed as Call { name: Dot(Ident("notes"), "remove") }).
        // Rewrite just the receiver to `__state.notes`, keeping the method name.
        Expr::Dot(obj, field)
            if matches!(
                obj.as_ref(),
                Expr::Ident(n) if state_fields.contains(n.as_str())
            ) =>
        {
            // Safe to unwrap: the match guard guarantees obj is an Ident.
            let state_name = match obj.as_ref() {
                Expr::Ident(n) => n.clone(),
                _ => unreachable!("guard guarantees Ident"),
            };
            Some(Expr::Dot(
                Box::new(Expr::Dot(
                    Box::new(Expr::Ident(Name::from(STATE_PARAM))),
                    state_name,
                )),
                field.clone(),
            ))
        }
        _ => None,
    };
    if let Some(new_e) = replacement {
        // `__state` is not itself a state field, and the new field slot is the
        // state-field name (now correctly qualified) — no further rewrite needed.
        *e = new_e;
        return;
    }

    // Phase 2: recurse into sub-expressions.
    match e {
        Expr::Bina(l, _, r) | Expr::NullCoalesce(l, r) => {
            rewrite_expr(l, state_fields);
            rewrite_expr(r, state_fields);
        }
        Expr::Unary(_, o) => rewrite_expr(o, state_fields),
        Expr::View(o) | Expr::Mut(o) | Expr::Move(o) | Expr::Take(o)
        | Expr::ErrorPropagate(o) | Expr::Some(o) | Expr::Ok(o) | Expr::Err(o)
        | Expr::BoxExpr(o) | Expr::ArcExpr(o) | Expr::Yield(o) => {
            rewrite_expr(o, state_fields)
        }
        Expr::Cast { expr, .. } | Expr::To { expr, .. } => rewrite_expr(expr, state_fields),
        Expr::Await { expr } | Expr::Go { expr } => rewrite_expr(expr, state_fields),
        Expr::TupleDestruct { expr, .. } => rewrite_expr(expr, state_fields),
        Expr::Index(a, i) => {
            rewrite_expr(a, state_fields);
            rewrite_expr(i, state_fields);
        }
        Expr::Array(elems) => {
            for el in elems {
                rewrite_expr(el, state_fields);
            }
        }
        Expr::Tuple(elems) => {
            for el in elems {
                rewrite_expr(el, state_fields);
            }
        }
        Expr::Object(pairs) => {
            for p in pairs {
                rewrite_expr(&mut p.value, state_fields);
            }
        }
        Expr::FStr(f) => {
            for part in &mut f.parts {
                rewrite_expr(part, state_fields);
            }
        }
        Expr::Call(c) => {
            rewrite_expr(&mut c.name, state_fields);
            for arg in &mut c.args.args {
                match arg {
                    Arg::Pos(ex) | Arg::Pair(_, ex) => rewrite_expr(ex, state_fields),
                    Arg::Name(_) => {}
                }
            }
        }
        Expr::Block(b) => rewrite_state_refs_stmts(&mut b.stmts, state_fields),
        Expr::If(If { branches, else_ }) => {
            for Branch { cond, body } in branches {
                rewrite_expr(cond, state_fields);
                rewrite_state_refs_stmts(&mut body.stmts, state_fields);
            }
            if let Some(eb) = else_ {
                rewrite_state_refs_stmts(&mut eb.stmts, state_fields);
            }
        }
        Expr::Lambda(fn_decl) => {
            rewrite_state_refs_stmts(&mut fn_decl.body.stmts, state_fields);
        }
        _ => {}
    }
}

/// Extract the bare handler name from an event pattern.
///
/// `".PrevMonth"` / `"Msg::PrevMonth"` → `"PrevMonth"`. Mirrors the private
/// helper in `vm_bridge.rs` so this module is self-contained.
pub fn handler_fn_name(pattern: &str) -> String {
    let name = pattern.trim_start_matches('.');
    let bare = name.rfind("::").map(|p| &name[p + 2..]).unwrap_or(name);
    format!("handler_{}", bare)
}

/// Look up a handler's parameter type from the widget's message definitions.
///
/// Returns `Type::StrSlice` as a permissive default when the payload type is
/// absent or unresolvable — the dispatcher pushes raw `Value`s, so the declared
/// type only influences Codegen's slot allocation, not runtime arg passing.
fn handler_param_type(widget: &AuraWidget, handler_bare: &str) -> Type {
    for msg in &widget.messages {
        if let Some(v) = msg.variants.iter().find(|v| v.name == handler_bare) {
            if let Some(ty) = &v.payload {
                return ty.clone();
            }
        }
    }
    Type::StrSlice
}

/// Synthesize the widget's state `type AppState { ... }`.
fn synthesize_state_type(widget: &AuraWidget) -> TypeDecl {
    let members: Vec<Member> = widget
        .state_vars
        .iter()
        .map(|v| {
            // `var days = []` has no declared type; default to a dynamic array so
            // `for cell in __state.days` compiles/iterates correctly.
            let ty = if matches!(v.type_info, Type::Unknown) {
                Type::List(Box::new(Type::Unknown))
            } else {
                v.type_info.clone()
            };
            Member {
                name: Name::from(v.name.as_str()),
                ty,
                value: None,
                attrs: Vec::new(),
            }
        })
        .collect();

    TypeDecl {
        name: Name::from("AppState"),
        kind: TypeDeclKind::UserType,
        parent: None,
        has: Vec::new(),
        specs: Vec::new(),
        spec_impls: Vec::new(),
        generic_params: Vec::new(),
        members,
        delegations: Vec::new(),
        methods: Vec::new(),
        attrs: Vec::new(),
        doc: None,
        is_pub: false,
    }
}

/// Synthesize a single widget handler as a real VM function statement.
fn synthesize_handler_fn(
    state_type: &TypeDecl,
    state_fields: &HashSet<String>,
    widget: &AuraWidget,
    event_pattern: &str,
    body_stmts: &[Stmt],
) -> Stmt {
    let bare = handler_fn_name(event_pattern)
        .strip_prefix("handler_")
        .map(|s| s.to_string())
        .unwrap_or_default();

    // First param is always the state receiver.
    let mut params: Vec<Param> = vec![Param::new(
        Name::from(STATE_PARAM),
        Type::User(state_type.clone()),
        None,
    )];
    // Remaining params come from the widget's handler_params map.
    if let Some(pnames) = widget.handler_params.get(event_pattern) {
        for pn in pnames {
            params.push(Param::new(
                Name::from(pn.as_str()),
                handler_param_type(widget, &bare),
                None,
            ));
        }
    }

    // Clone + rewrite the body.
    let mut stmts: Vec<Stmt> = body_stmts.to_vec();
    rewrite_state_refs_stmts(&mut stmts, state_fields);

    let body = Body {
        stmts,
        has_new_line: false,
        source_lines: Vec::new(),
    };

    Stmt::Fn(Fn::new(
        FnKind::Function,
        Name::from(handler_fn_name(event_pattern).as_str()),
        None,
        params,
        body,
        Type::Void,
    ))
}

/// Compile the widget's imports + state type + handlers into a single VM `Module`.
///
/// `import_stmts` are the `Stmt::Fn` / `Stmt::TypeDecl` / `Stmt::EnumDecl` from
/// every `use`-imported module (collected by `run_file_dynamic_ui`). They are
/// compiled on the same `Codegen` as the handlers so cross-references (e.g.
/// `build_month_grid`) resolve to in-module `CALL` targets and object/array
/// literal metadata shares one unified table.
pub fn synthesize_widget_module(
    widget: &AuraWidget,
    import_stmts: Vec<Stmt>,
) -> SynthResult<Module> {
    let state_fields: HashSet<String> = widget
        .state_vars
        .iter()
        .map(|v| v.name.clone())
        .collect();
    let state_type = synthesize_state_type(widget);

    let mut codegen = Codegen::new();

    // 0. Pre-register every imported fn's return type so forward references
    //    resolve during body compilation. Without this, an fn that calls a
    //    LATER-defined helper (e.g. build_month_grid calls day_style, declared
    //    below it in calendar_util.at) can't infer the call's return type, so
    //    Codegen's infer_object_type defaults it to NestedObject — and an Obj
    //    field whose value is that call (e.g. `style: day_style(...)`) gets
    //    stored as a VmRef instead of a String, corrupting the value.
    for stmt in &import_stmts {
        if let Stmt::Fn(f) = stmt {
            codegen
                .fn_return_types
                .insert(f.name.to_string(), f.ret.clone());
        }
    }

    // 1. Imports (functions, types, enums) — declarations + module-level
    //    stores and use statements.
    //    Order matters: compile Stmt::Use first so auto_modules are registered
    //    (so a later `db.func()` body call generates a linkable CALL reloc),
    //    then Stmt::Store (module globals like `var notes`), then Fn/Type/Ext
    //    whose bodies may reference both. Without Use/Store, imported #[api]
    //    fns that read module globals or call cross-module helpers fail with
    //    "Undefined symbol" at link time (015-notes).
    for stmt in &import_stmts {
        if let crate::ast::Stmt::Use(_) = stmt {
            if let Err(e) = codegen.compile_stmt(stmt) {
                log::warn!("handler_codegen: import use stmt failed to compile: {}", e);
            }
        }
    }
    // Plan 333: register imported module-level `var` declarations as GLOBALS
    // BEFORE compiling them, mirroring the script path (lib.rs:625-633). Without
    // this, `var notes = List<Note>.new([...])` compiles as a script-wrapper
    // LOCAL, so `db.all_notes()` (a separate fn) reads `notes` as nil →
    // "no function '.to_array' for type 'unknown_nv'". Globals live in
    // vm.globals and are visible from every function via LOAD_GLOBAL/STORE_GLOBAL.
    for stmt in &import_stmts {
        if let crate::ast::Stmt::Store(s) = stmt {
            if matches!(s.kind, crate::ast::StoreKind::Var) {
                codegen.global_vars.insert(s.name.to_string());
            }
        }
    }
    // Plan 333: wrap the module-level store initializers into a synthesized
    // `__module_init` fn rather than compiling them as bare top-level code.
    // Why: the widget VM never runs from address 0 (it only jumps to handler
    // entries via call_handler), and codegen.finish() emits no RET after the
    // top-level statements — so bare init code would fall through into the next
    // handler's bytecode. An exported __module_init fn is callable explicitly
    // (VmBridge runs it once before .Init), giving the globals defined values.
    let store_inits: Vec<Stmt> = import_stmts.iter()
        .filter_map(|s| {
            if let crate::ast::Stmt::Store(st) = s {
                // Keep only `var`/reassignment stores (declarations with an
                // initializer). These set the global to its initial value.
                Some(Stmt::Store(st.clone()))
            } else {
                None
            }
        })
        .collect();
    if !store_inits.is_empty() {
        // Plan 333: force these `var` stores to compile as STORE_GLOBAL even
        // though they're inside a fn body (see codegen Stmt::Store guard).
        codegen.force_global_store = true;
        let init_fn = Stmt::Fn(Fn::new(
            FnKind::Function,
            Name::from(MODULE_INIT_FN),
            None,
            Vec::new(),
            Body { stmts: store_inits, has_new_line: false, source_lines: Vec::new() },
            Type::Void,
        ));
        if let Err(e) = codegen.compile_stmt(&init_fn) {
            log::warn!("handler_codegen: __module_init failed to compile: {}", e);
        }
        codegen.force_global_store = false;
    }
    for stmt in &import_stmts {
        if matches!(stmt, Stmt::Fn(_) | Stmt::TypeDecl(_) | Stmt::EnumDecl(_) | Stmt::Ext(_)) {
            if let Err(e) = codegen.compile_stmt(stmt) {
                log::warn!("handler_codegen: import stmt failed to compile: {}", e);
            }
        }
    }

    // 2. State type declaration.
    if let Err(e) = codegen.compile_stmt(&Stmt::TypeDecl(state_type.clone())) {
        log::warn!("handler_codegen: AppState type failed to compile: {}", e);
    }

    // 3. Handlers + lifecycle methods (sorted for deterministic layout).
    //
    // NOTE: `.Init` / `.Destroy` are MOVED out of `widget.handlers` into
    // `widget.lifecycle` during aura extraction (see aura/extract.rs). They must
    // be synthesized as real `handler_<Name>` fns too, otherwise `Init` (which
    // populates state like `.days = build_month_grid(...)`) never runs.
    let mut all_handlers: Vec<(String, &LogicPayload)> = widget
        .handlers
        .iter()
        .map(|(p, pl)| (p.clone(), pl))
        .collect();
    for lc in &widget.lifecycle {
        all_handlers.push((lc.name.clone(), &lc.payload));
    }
    all_handlers.sort_by(|a, b| handler_fn_name(&a.0).cmp(&handler_fn_name(&b.0)));

    for (event_pattern, payload) in &all_handlers {
        let body_stmts = match payload {
            LogicPayload::AstStmts(stmts) => stmts,
            _ => continue,
        };
        let handler_fn = synthesize_handler_fn(
            &state_type,
            &state_fields,
            widget,
            event_pattern,
            body_stmts,
        );
        if let Err(e) = codegen.compile_stmt(&handler_fn) {
            log::warn!(
                "handler_codegen: handler '{}' failed to compile: {}",
                event_pattern,
                e
            );
        }
    }

    Ok(codegen.finish(widget.name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Arg, Args, Call, Name, Store, StoreKind, Type};
    use auto_val::Op;

    fn make_state_fields(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn rewrites_bare_state_ident_read() {
        let mut stmt = Stmt::Expr(Expr::Bina(
            Box::new(Expr::Ident(Name::from("count"))),
            Op::Add,
            Box::new(Expr::Int(1)),
        ));
        let fields = make_state_fields(&["count"]);
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &fields);
        // LHS should now be __state.count
        match &stmt {
            Stmt::Expr(Expr::Bina(l, _, _)) => {
                let rendered = format!("{}", l);
                assert!(rendered.contains("(name __state)"), "{}", rendered);
                assert!(rendered.contains(".count"), "{}", rendered);
            }
            other => panic!("expected rewritten Bina, got {:?}", other),
        }
    }

    #[test]
    fn rewrites_self_dot_state_in_assignment() {
        // self.count = self.count + 1  →  __state.count = __state.count + 1
        let lhs = Expr::Dot(Box::new(Expr::Ident(Name::from("self"))), Name::from("count"));
        let rhs = Expr::Bina(
            Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from("."))),
                Name::from("count"),
            )),
            Op::Add,
            Box::new(Expr::Int(1)),
        );
        let mut stmt = Stmt::Expr(Expr::Bina(Box::new(lhs), Op::Asn, Box::new(rhs)));
        let fields = make_state_fields(&["count"]);
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &fields);
        let rendered = format!("{}", stmt);
        assert!(rendered.contains("(name __state)"), "{}", rendered);
        assert!(rendered.contains(".count"), "{}", rendered);
        // self/. references must be gone
        assert!(!rendered.contains("(name self)"), "{}", rendered);
        assert!(!rendered.contains("(name .)"), "{}", rendered);
    }

    #[test]
    fn rewrites_state_field_as_method_receiver() {
        // Regression for 015-notes: `notes.remove(1)` where `notes` is a state
        // field must rewrite the receiver to `__state.notes`. Previously only
        // bare reads (`notes`) and `self.x` / `.x` forms were rewritten; a
        // method call whose receiver is a bare state-field ident was left
        // untouched, causing "Undefined variable: notes" at VM compile time.
        // notes.remove(1) parses to Call { name: Dot(Ident("notes"), "remove"), args: [1] }
        let mut stmt = Stmt::Expr(Expr::Call(Call {
            name: Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from("notes"))),
                Name::from("remove"),
            )),
            args: Args { args: vec![Arg::Pos(Expr::Int(1))] },
            ret: Type::Unknown,
            type_args: Vec::new(),
            pos: None,
        }));
        let fields = make_state_fields(&["notes"]);
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &fields);
        match &stmt {
            Stmt::Expr(Expr::Call(c)) => {
                let rendered = format!("{}", c.name);
                assert!(
                    rendered.contains("(name __state)"),
                    "receiver should be rewritten to __state.notes, got: {}",
                    rendered
                );
                assert!(
                    rendered.contains(".notes"),
                    "expected .notes in receiver, got: {}",
                    rendered
                );
            }
            other => panic!("expected Call, got {:?}", other),
        }
    }

    #[test]
    fn does_not_rewrite_local_binding_or_method_field() {
        // let n = .count + other  — n is a local, "count" is state, "other" is not.
        let mut stmt = Stmt::Store(Store {
            kind: StoreKind::Let,
            name: Name::from("n"),
            ty: Type::Unknown,
            expr: Expr::Bina(
                Box::new(Expr::Ident(Name::from("count"))),
                Op::Add,
                Box::new(Expr::Ident(Name::from("other"))),
            ),
            attrs: Vec::new(),
        });
        let fields = make_state_fields(&["count"]);
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &fields);
        match &stmt {
            Stmt::Store(s) => {
                // Binding name untouched
                assert_eq!(s.name.as_str(), "n");
                let rendered = format!("{}", s.expr);
                assert!(rendered.contains("(name __state)"), "{}", rendered);
                assert!(rendered.contains(".count"), "{}", rendered);
                assert!(rendered.contains("(name other)"), "{}", rendered);
            }
            other => panic!("expected Store, got {:?}", other),
        }
    }

    #[test]
    fn handler_fn_name_strips_dot_and_module_prefix() {
        assert_eq!(handler_fn_name(".Inc"), "handler_Inc");
        assert_eq!(handler_fn_name("Msg::PrevMonth"), "handler_PrevMonth");
        assert_eq!(handler_fn_name(".SelectDay"), "handler_SelectDay");
    }
}
