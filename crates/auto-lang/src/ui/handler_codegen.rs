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

use std::collections::{HashSet, HashMap};
use std::cell::RefCell;

use crate::ast::{
    Arg, Body, Branch, Expr, Fn, FnKind, If, Member, Name, Param, Stmt, Type, TypeDecl,
    TypeDeclKind,
};
use crate::aura::{AuraWidget, LogicPayload};
use crate::vm::codegen::Codegen;
use crate::vm::loader::Module;

// Plan 370 D-GAP-4: thread-local store context for rewrite.
// Set during synthesize_from_decl when processing root widget handlers.
thread_local! {
    static STORE_FIELDS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new());
    static STORE_WIDGET_NAMES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Set the store context for the current synthesis pass.
pub fn set_store_context(fields: HashMap<String, Vec<String>>, names: HashMap<String, String>) {
    STORE_FIELDS.with(|s| *s.borrow_mut() = fields);
    STORE_WIDGET_NAMES.with(|s| *s.borrow_mut() = names);
}

/// Clear the store context after synthesis.
pub fn clear_store_context() {
    STORE_FIELDS.with(|s| s.borrow_mut().clear());
    STORE_WIDGET_NAMES.with(|s| s.borrow_mut().clear());
}

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
    // Plan 370 D-GAP-4 Phase 0: store.X rewriting.
    // store.Method(args) → handler_StoreName_Method(__state, args)
    if let Expr::Call(call) = e {
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if let Expr::Ident(alias) = obj.as_ref() {
                let has_store = STORE_WIDGET_NAMES.with(|s| s.borrow().contains_key(alias.as_str()));
                if has_store {
                    let store_name = STORE_WIDGET_NAMES.with(|s| s.borrow().get(alias.as_str()).cloned()).unwrap_or_default();
                    let handler_fn = format!("handler_{}_{}", store_name, method);
                    let mut new_args = vec![crate::ast::Arg::Pos(Expr::Ident(Name::from(STATE_PARAM)))];
                    // Clone and rewrite each original arg before adding
                    for arg in &call.args.args {
                        let mut cloned = arg.clone();
                        match &mut cloned {
                            crate::ast::Arg::Pos(ex) | crate::ast::Arg::Pair(_, ex) => {
                                rewrite_expr(ex, state_fields);
                            }
                            crate::ast::Arg::Name(_) => {}
                        }
                        new_args.push(cloned);
                    }
                    *e = Expr::Call(crate::ast::Call {
                        name: Box::new(Expr::Ident(Name::from(handler_fn))),
                        args: crate::ast::Args { args: new_args },
                        ret: Type::Void,
                        type_args: Vec::new(),
                        pos: None,
                    });
                    return;
                }
            }
        }
    }
    // store.field → __state.field (store fields merged into root state)
    if let Expr::Dot(obj, _field) = e {
        if let Expr::Ident(alias) = obj.as_ref() {
            let is_store = STORE_FIELDS.with(|s| s.borrow().contains_key(alias.as_str()));
            if is_store {
                *obj = Box::new(Expr::Ident(Name::from(STATE_PARAM)));
                return;
            }
        }
    }
    // .store.field → __state.field (self.store.X in view bindings or handler body)
    // This is Dot(Dot(Ident("."), "store"), "field") → Dot(Ident("__state"), "field")
    if let Expr::Dot(inner, field) = e {
        if let Expr::Dot(obj, store_alias) = inner.as_ref() {
            if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self") {
                let is_store = STORE_FIELDS.with(|s| s.borrow().contains_key(store_alias.as_str()));
                if is_store {
                    *e = Expr::Dot(
                        Box::new(Expr::Ident(Name::from(STATE_PARAM))),
                        field.clone(),
                    );
                    return;
                }
            }
        }
    }

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
        // Plan 318: recurse into Dot's object so nested `self` refs rewrite.
        // E.g. `.note.title` = Dot(Dot(Ident("self"), "note"), "title"): the top
        // Dot doesn't match the Phase-1 self/state-field patterns (its object is a
        // Dot, not an Ident), so without recursing, the inner `self` survives and
        // codegen reports "Undefined variable: self".
        Expr::Dot(obj, _) => {
            rewrite_expr(obj, state_fields);
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

/// Plan 320: namespaced handler fn name: `handler_<WidgetName>_<EventName>`.
/// E.g. `handler_App_SelectNote`, `handler_EditorPanel_Edit`.
pub fn namespaced_handler_fn_name(widget_name: &str, pattern: &str) -> String {
    let full = handler_fn_name(pattern); // "handler_<Event>"
    let bare = full.strip_prefix("handler_").unwrap_or(&full);
    format!("handler_{}_{}", widget_name, bare)
}

/// Plan 320: state type name per widget: `<WidgetName>_State`.
pub fn state_type_name(widget_name: &str) -> String {
    format!("{}_State", widget_name)
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

/// Synthesize the widget's state `type <WidgetName>_State { ... }`.
/// Plan 320: state type is namespaced by widget name so multiple widgets'
/// state types coexist in one module.
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
        name: Name::from(state_type_name(&widget.name).as_str()),
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
/// Plan 320: `widget_name` is used to namespace the handler fn name
/// (handler_<WidgetName>_<EventName>) so multiple widgets coexist in one module.
fn synthesize_handler_fn(
    widget_name: &str,
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
        Name::from(namespaced_handler_fn_name(widget_name, event_pattern).as_str()),
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
    child_widgets: &[AuraWidget],
    import_stmts: Vec<Stmt>,
    import_aliases: &std::collections::HashMap<String, String>,
    api_over_http: bool,
) -> SynthResult<(Module, crate::vm::generic_registry::GenericRegistry)> {
    // PR-3: 此函数只依赖 widget 的【逻辑部分】（state_vars/handlers/lifecycle/
    // messages/handler_params）。view_tree/span_map 等视图部分在本函数中不使用。
    // 辅助函数（synthesize_state_type 等）暂仍接受 &AuraWidget 以避免大面积签名
    // 变更；后续 PR-3b 去中转时改为直读 WidgetDecl。
    let _logic = widget.logic();
    let mut codegen = Codegen::new();
    codegen.api_over_http = api_over_http;

    // Plan 339 Phase 4: populate import_scope directly from use_scanner data.
    // This maps bare function names to their module-qualified exports so
    // `delete_note(...)` resolves to `api.delete_note` in the exports table.
    for (bare, qualified) in import_aliases {
        codegen.import_scope.insert(bare.clone(), qualified.clone());
    }

    // Plan 340: build api_funcs metadata from imported Fn declarations that
    // carry #[api(method,path)] attrs. Used by Expr::Call to rewrite bare API
    // calls into HTTP requests when api_over_http is set.
    if api_over_http {
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if let Some(api) = &f.api_attrs {
                    let bare = f.name.to_string().split('.').last()
                        .unwrap_or(&f.name.to_string()).to_string();
                    let params: Vec<String> = f.params.iter()
                        .map(|p| p.name.to_string()).collect();
                    codegen.api_funcs.insert(bare, crate::vm::codegen::ApiCallInfo {
                        method: api.method.clone(),
                        path: api.path.clone(),
                        params,
                        ret_type: f.ret.clone(),
                    });
                }
            }
        }
    }

    // Plan 339 Phase 6b: intra-module bare calls. After flattening, every
    // imported fn is module-qualified (e.g. calendar_util.day_style), but a
    // sibling's body still calls it by its bare name (day_style). For forward
    // references the export isn't populated yet at the call site, so the
    // unique bare-name → qualified fallback in resolved_func/resolve_call_symbol
    // can't fire. Pre-register every flattened fn's bare name → qualified name
    // here. When two modules define the same bare name (db.create_note AND
    // api.create_note) this is ambiguous — last-write-wins is wrong, so we
    // only register a bare alias when the name is unique across all modules.
    {
        // Count how many modules define each bare name.
        let mut bare_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if let Some(bare) = f.name.to_string().split('.').last() {
                    *bare_counts.entry(bare.to_string()).or_insert(0) += 1;
                }
            }
        }
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                let qualified = f.name.to_string();
                if let Some(bare) = qualified.split('.').last() {
                    // Only auto-alias unique bare names; ambiguous ones must
                    // come through the explicit `use` import_aliases.
                    if bare_counts.get(bare) == Some(&1) {
                        codegen
                            .import_scope
                            .entry(bare.to_string())
                            .or_insert(qualified);
                    }
                }
            }
        }
    }

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
    //    Order matters (Plan 318): Use → TypeDecl/EnumDecl → Store(__module_init)
    //    → Fn/Ext.
    //    - Use first: registers auto_modules so `db.func()` calls generate
    //      linkable CALL relocs.
    //    - TypeDecl/EnumDecl BEFORE __module_init: a `var notes = List<Note>.new`
    //      initializer references the `Note` type. codegen's Expr::Node branch
    //      checks `generic_registry.has_template("Note")`; if Note isn't
    //      registered yet, it falls back to CREATE_NODE (node_id 3M) instead of
    //      CONSTRUCT_INSTANCE/CREATE_OBJ (object id 1M) — corrupting the element
    //      id (Plan 318). Registering types first fixes that.
    //    - Store wrapped in __module_init (Plan 333): runs module-level globals
    //      before Init.
    //    - Fn/Ext last: their bodies reference globals and types.
    for stmt in &import_stmts {
        if let crate::ast::Stmt::Use(_) = stmt {
            if let Err(e) = codegen.compile_stmt(stmt) {
                log::warn!("handler_codegen: import use stmt failed to compile: {}", e);
            }
        }
    }
    for stmt in &import_stmts {
        if matches!(stmt, Stmt::TypeDecl(_) | Stmt::EnumDecl(_)) {
            if let Err(e) = codegen.compile_stmt(stmt) {
                log::warn!("handler_codegen: import type/enum decl failed to compile: {}", e);
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

    // 2. Compile state types + handlers for ALL widgets (root + children).
    //    Plan 320: single VM — all widgets' state types and handlers are compiled
    //    into one module. Handler fns are namespaced: handler_<Widget>_<Event>.
    //    State types are namespaced: <Widget>_State.
    //
    //    The root widget is compiled first, then children. Each widget's state
    //    type uses its own field set; handlers reference their own state type.
    let all_widgets: Vec<&AuraWidget> = std::iter::once(widget)
        .chain(child_widgets.iter())
        .collect();

    for w in &all_widgets {
        let w_state_fields: HashSet<String> = w
            .state_vars
            .iter()
            .map(|v| v.name.clone())
            .collect();
        let w_state_type = synthesize_state_type(w);

        // State type declaration.
        if let Err(e) = codegen.compile_stmt(&Stmt::TypeDecl(w_state_type.clone())) {
            log::warn!("handler_codegen: {} state type failed: {}", w.name, e);
        }

        // Handlers + lifecycle (sorted for deterministic layout).
        let mut w_handlers: Vec<(String, &LogicPayload)> = w
            .handlers
            .iter()
            .map(|(p, pl)| (p.clone(), pl))
            .collect();
        for lc in &w.lifecycle {
            w_handlers.push((lc.name.clone(), &lc.payload));
        }
        w_handlers.sort_by(|a, b| handler_fn_name(&a.0).cmp(&handler_fn_name(&b.0)));

        for (event_pattern, payload) in &w_handlers {
            let body_stmts = match payload {
                LogicPayload::AstStmts(stmts) => stmts,
                _ => continue,
            };
            let handler_fn = synthesize_handler_fn(
                &w.name,
                &w_state_type,
                &w_state_fields,
                w,
                event_pattern,
                body_stmts,
            );
            if let Err(e) = codegen.compile_stmt(&handler_fn) {
                log::warn!(
                    "handler_codegen: {}.{} failed: {}",
                    w.name, event_pattern, e
                );
            }
        }
    }

    // Plan 318: return the codegen's populated generic_registry along with the
    // module. CONSTRUCT_INSTANCE (runtime) reads field_names from the VM's
    // generic_registry; if the widget VM doesn't inherit the registry that
    // compiled the types (Note), field_names fall back to "_unknown" and struct
    // field access (note.title) fails. new_with_imports loads this into the VM.
    let registry = std::mem::take(&mut codegen.generic_registry);
    Ok((codegen.finish(widget.name.clone()), registry))
}

// ============================================================================
// PR-3b Step 1: Decl-based synthesis (reads &WidgetDecl, bypasses AuraWidget)
// ============================================================================

/// Detect a `.Tick` handler and extract its `interval` (in ms) from the model.
///
/// Mirrors `extract_widget_from_decl` (extract.rs:763-781): if a `.Tick` handler
/// exists, look for a model field named "interval" with an integer initial value.
/// Defaults to 1000ms when the field is absent or not an int. Returns `None`
/// when there is no `.Tick` handler.
pub fn extract_tick_interval_from_decl(decl: &crate::ast::WidgetDecl) -> Option<u32> {
    let has_tick = decl
        .on
        .as_ref()
        .map(|on| on.handlers.iter().any(|h| h.pattern == ".Tick"))
        .unwrap_or(false);
    if !has_tick {
        return None;
    }
    let interval_val = decl
        .model
        .as_ref()
        .and_then(|m| m.fields.iter().find(|f| f.name.as_str() == "interval"))
        .and_then(|f| {
            if let Expr::Int(n) = &f.init {
                Some(*n as u32)
            } else {
                None
            }
        })
        .or(Some(1000));
    interval_val
}

/// Synthesize the widget state `type <WidgetName>_State { ... }` from a WidgetDecl.
///
/// PR-3b: reads `decl.model.fields` directly instead of going through
/// `AuraWidget.state_vars`. When `tick_interval` is `Some`, the "interval"
/// field is skipped (it's consumed by the tick scheduler, not a ref() state).
fn synthesize_state_type_from_decl(
    decl: &crate::ast::WidgetDecl,
    tick_interval: Option<u32>,
) -> TypeDecl {
    let members: Vec<Member> = decl
        .model
        .as_ref()
        .map(|m| {
            m.fields
                .iter()
                .filter(|f| {
                    // Skip "interval" when tick scheduling is active for this widget.
                    !(tick_interval.is_some() && f.name.as_str() == "interval")
                })
                .map(|f| {
                    // `var days = []` has no declared type (Type::Unknown); default
                    // to a dynamic array so `for cell in __state.days` iterates.
                    let ty = if matches!(f.ty, Type::Unknown) {
                        Type::List(Box::new(Type::Unknown))
                    } else {
                        f.ty.clone()
                    };
                    Member {
                        name: f.name.clone(),
                        ty,
                        value: None,
                        attrs: Vec::new(),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    TypeDecl {
        name: Name::from(state_type_name(decl.name.as_str()).as_str()),
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

/// Look up a handler's parameter type from a WidgetDecl's message definitions.
///
/// PR-3b: reads `decl.messages` directly. Returns `Type::StrSlice` as a
/// permissive default when the payload type is absent/unresolvable (same
/// behavior as the AuraWidget `handler_param_type`).
fn handler_param_type_from_decl(decl: &crate::ast::WidgetDecl, handler_bare: &str) -> Type {
    for msg in &decl.messages {
        if let Some(v) = msg.variants.iter().find(|v| v.name.as_str() == handler_bare) {
            if let Some(ty) = &v.payload {
                return ty.clone();
            }
        }
    }
    Type::StrSlice
}

/// Synthesize a single widget handler as a real VM function statement (decl-based).
///
/// PR-3b: reads handler params from `decl.on` and the param type from
/// `handler_param_type_from_decl`. Otherwise identical to `synthesize_handler_fn`.
fn synthesize_handler_fn_from_decl(
    decl: &crate::ast::WidgetDecl,
    widget_name: &str,
    state_type: &TypeDecl,
    state_fields: &HashSet<String>,
    event_pattern: &str,
    body_stmts: &[Stmt],
) -> Stmt {
    synthesize_handler_fn_from_decl_with_store(
        decl, widget_name, "", state_type, state_fields, event_pattern, body_stmts,
        &HashMap::new(), &HashMap::new(),
    )
}

/// Plan 370 D-GAP-4: strip callback prop calls from child widget handler bodies.
/// `on_delete()`, `on_tags_changed()`, etc. are routed by the renderer
/// (DynamicMessage → parent handler), not by the VM. Removing them from the
/// compiled body prevents linker errors for undefined symbols.
fn strip_callback_calls(stmts: &mut Vec<Stmt>) {
    for stmt in stmts.iter_mut() {
        strip_callback_calls_stmt(stmt);
    }
    stmts.retain(|stmt| !is_noop_callback_call(stmt));
}

fn is_noop_callback_call(stmt: &Stmt) -> bool {
    if let Stmt::Expr(Expr::Call(call)) = stmt {
        if let Expr::Ident(name) = call.name.as_ref() {
            return name.as_str().starts_with("on_");
        }
    }
    false
}

fn strip_callback_calls_stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::If(If { branches, else_ }) => {
            for Branch { body, .. } in branches.iter_mut() {
                strip_callback_calls(&mut body.stmts);
            }
            if let Some(eb) = else_ {
                strip_callback_calls(&mut eb.stmts);
            }
        }
        Stmt::For(f) => {
            strip_callback_calls(&mut f.body.stmts);
        }
        Stmt::Block(b) => {
            strip_callback_calls(&mut b.stmts);
        }
        Stmt::Fn(fn_decl) => {
            strip_callback_calls(&mut fn_decl.body.stmts);
        }
        Stmt::Expr(e) => {
            strip_callback_calls_expr(e);
        }
        _ => {}
    }
}

fn strip_callback_calls_expr(e: &mut Expr) {
    match e {
        Expr::Block(b) => strip_callback_calls(&mut b.stmts),
        Expr::If(If { branches, else_ }) => {
            for Branch { body, .. } in branches {
                strip_callback_calls(&mut body.stmts);
            }
            if let Some(eb) = else_ {
                strip_callback_calls(&mut eb.stmts);
            }
        }
        _ => {}
    }
}

/// Plan 370 D-GAP-4: synthesize handler with store field rewriting support.
fn synthesize_handler_fn_from_decl_with_store(
    decl: &crate::ast::WidgetDecl,
    widget_name: &str,
    root_widget_name: &str,
    state_type: &TypeDecl,
    state_fields: &HashSet<String>,
    event_pattern: &str,
    body_stmts: &[Stmt],
    store_fields: &HashMap<String, Vec<String>>,
    store_widget_names: &HashMap<String, String>,
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
    // Remaining params come from the matching on handler's params list.
    if let Some(pnames) = decl
        .on
        .as_ref()
        .and_then(|on| on.handlers.iter().find(|h| h.pattern == event_pattern))
        .map(|h| h.params.clone())
    {
        for pn in pnames {
            params.push(Param::new(
                Name::from(pn.as_str()),
                handler_param_type_from_decl(decl, &bare),
                None,
            ));
        }
    }

    // Clone + rewrite the body (Plan 370 D-GAP-4: store context is in thread_local).
    let mut stmts: Vec<Stmt> = body_stmts.to_vec();
    rewrite_state_refs_stmts(&mut stmts, state_fields);

    // Plan 370 D-GAP-4: for child widgets, strip callback prop calls (on_delete,
    // on_tags_changed, etc.) — they're routed by the renderer (DynamicMessage),
    // not the VM. Replacing them with no-op prevents linker errors.
    if widget_name != root_widget_name {
        strip_callback_calls(&mut stmts);
    }

    let body = Body {
        stmts,
        has_new_line: false,
        source_lines: Vec::new(),
    };

    Stmt::Fn(Fn::new(
        FnKind::Function,
        Name::from(namespaced_handler_fn_name(widget_name, event_pattern).as_str()),
        None,
        params,
        body,
        Type::Void,
    ))
}

/// Compile a WidgetDecl (root + children) into a single VM `Module` WITHOUT
/// going through the AuraWidget intermediate representation.
///
/// PR-3b Step 1: the VM-bypass entry point. The import/setup half is identical
/// to `synthesize_widget_module` (the code is AuraWidget-agnostic); the
/// per-widget loop reads the `WidgetDecl` directly via the `_from_decl`
/// helpers. The lifecycle merge (`.Init`/`.Destroy` pulled out of the `on`
/// handlers map) and the `interval` filtering for `.Tick` are replicated
/// here so the synthesized module matches the AuraWidget path exactly.
pub fn synthesize_from_decl(
    decl: &crate::ast::WidgetDecl,
    child_decls: &[crate::ast::WidgetDecl],
    import_stmts: Vec<Stmt>,
    import_aliases: &std::collections::HashMap<String, String>,
    api_over_http: bool,
) -> SynthResult<(Module, crate::vm::generic_registry::GenericRegistry)> {
    let mut codegen = Codegen::new();
    codegen.api_over_http = api_over_http;

    // Plan 339 Phase 4: populate import_scope directly from use_scanner data.
    for (bare, qualified) in import_aliases {
        codegen.import_scope.insert(bare.clone(), qualified.clone());
    }

    // Plan 340: build api_funcs metadata from imported Fn declarations that
    // carry #[api(method,path)] attrs.
    if api_over_http {
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if let Some(api) = &f.api_attrs {
                    let bare = f.name.to_string().split('.').last()
                        .unwrap_or(&f.name.to_string()).to_string();
                    let params: Vec<String> = f.params.iter()
                        .map(|p| p.name.to_string()).collect();
                    codegen.api_funcs.insert(bare, crate::vm::codegen::ApiCallInfo {
                        method: api.method.clone(),
                        path: api.path.clone(),
                        params,
                        ret_type: f.ret.clone(),
                    });
                }
            }
        }
    }

    // Plan 339 Phase 6b: pre-register unique bare-name → qualified aliases for
    // intra-module forward references.
    {
        let mut bare_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if let Some(bare) = f.name.to_string().split('.').last() {
                    *bare_counts.entry(bare.to_string()).or_insert(0) += 1;
                }
            }
        }
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                let qualified = f.name.to_string();
                if let Some(bare) = qualified.split('.').last() {
                    if bare_counts.get(bare) == Some(&1) {
                        codegen
                            .import_scope
                            .entry(bare.to_string())
                            .or_insert(qualified);
                    }
                }
            }
        }
    }

    // 0. Pre-register every imported fn's return type for forward references.
    for stmt in &import_stmts {
        if let Stmt::Fn(f) = stmt {
            codegen
                .fn_return_types
                .insert(f.name.to_string(), f.ret.clone());
        }
    }

    // 1. Imports — same ordering as synthesize_widget_module.
    for stmt in &import_stmts {
        if let crate::ast::Stmt::Use(_) = stmt {
            if let Err(e) = codegen.compile_stmt(stmt) {
                log::warn!("handler_codegen: import use stmt failed to compile: {}", e);
            }
        }
    }
    for stmt in &import_stmts {
        if matches!(stmt, Stmt::TypeDecl(_) | Stmt::EnumDecl(_)) {
            if let Err(e) = codegen.compile_stmt(stmt) {
                log::warn!("handler_codegen: import type/enum decl failed to compile: {}", e);
            }
        }
    }
    for stmt in &import_stmts {
        if let crate::ast::Stmt::Store(s) = stmt {
            if matches!(s.kind, crate::ast::StoreKind::Var) {
                codegen.global_vars.insert(s.name.to_string());
            }
        }
    }
    let store_inits: Vec<Stmt> = import_stmts.iter()
        .filter_map(|s| {
            if let crate::ast::Stmt::Store(st) = s {
                Some(Stmt::Store(st.clone()))
            } else {
                None
            }
        })
        .collect();
    if !store_inits.is_empty() {
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

    // 2. Compile state types + handlers for ALL widgets (root + children),
    //    reading directly from WidgetDecl.
    let all_decls: Vec<&crate::ast::WidgetDecl> = std::iter::once(decl)
        .chain(child_decls.iter())
        .collect();

    // Plan 370 D-GAP-4: build store alias → field names map AND alias → widget name map.
    let mut store_fields_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut store_widget_names: HashMap<String, String> = HashMap::new();
    for d in &all_decls {
        if d.view.is_none() {
            let fields: Vec<String> = d.model
                .as_ref()
                .map(|m| m.fields.iter().map(|f| f.name.to_string()).collect())
                .unwrap_or_default();
            if !fields.is_empty() {
                store_fields_map.insert("store".to_string(), fields.clone());
                store_widget_names.insert("store".to_string(), d.name.to_string());
            }
        }
    }

    // Set thread-local store context so rewrite_expr can access it.
    set_store_context(store_fields_map.clone(), store_widget_names.clone());

    for d in &all_decls {
        let d_tick = extract_tick_interval_from_decl(d);
        let mut d_state_fields: HashSet<String> = {
            let mut s: HashSet<String> = d
                .model
                .as_ref()
                .map(|m| m.fields.iter().map(|f| f.name.to_string()).collect())
                .unwrap_or_default();
            if d_tick.is_some() {
                s.retain(|n| n != "interval");
            }
            s
        };
        // Plan 370 D-GAP-4: merge store fields into root widget's state_fields
        if d.name == decl.name {
            for (_, fields) in &store_fields_map {
                for f in fields {
                    d_state_fields.insert(f.clone());
                }
            }
        }
        let d_state_type = synthesize_state_type_from_decl(d, d_tick);

        if let Err(e) = codegen.compile_stmt(&Stmt::TypeDecl(d_state_type.clone())) {
            log::warn!("handler_codegen: {} state type failed: {}", d.name, e);
        }

        let mut d_handlers: Vec<(String, Vec<Stmt>)> = Vec::new();
        if let Some(on) = &d.on {
            for h in &on.handlers {
                d_handlers.push((h.pattern.clone(), h.body.stmts.clone()));
            }
        }
        for lc in &d.lifecycle {
            d_handlers.push((lc.name.clone(), lc.body.clone()));
        }
        d_handlers.sort_by(|a, b| handler_fn_name(&a.0).cmp(&handler_fn_name(&b.0)));

        for (event_pattern, body_stmts) in &d_handlers {
            let handler_fn = synthesize_handler_fn_from_decl_with_store(
                d,
                &d.name.to_string(),
                &decl.name.to_string(),
                &d_state_type,
                &d_state_fields,
                event_pattern,
                body_stmts,
                &store_fields_map,
                &store_widget_names,
            );
            // Plan 370 D-GAP-4: for child widget handlers that call callback props
            // (on_delete, on_tags_changed, etc.), we need to ensure those symbols
            // exist in the VM module. Since we can't easily route them to the parent
            // handler, create stub functions for any on_* callback that the body references.
            // This prevents linker errors; the actual routing happens at the renderer level
            // (iced event → DynamicMessage → parent handler).
            if let Err(e) = codegen.compile_stmt(&handler_fn) {
                log::warn!(
                    "handler_codegen: {}.{} failed: {}",
                    d.name, event_pattern, e
                );
            }
        }
    }

    let registry = std::mem::take(&mut codegen.generic_registry);
    clear_store_context();
    Ok((codegen.finish(decl.name.to_string()), registry))
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
