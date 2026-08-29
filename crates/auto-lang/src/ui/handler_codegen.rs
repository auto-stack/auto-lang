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
    Arg, Body, Branch, Expr, Fn, FnKind, If, Member, Name, Param, Stmt, Try, Type, TypeDecl,
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
    // VM multi-store fix: store_name → its msg variant names, so rewrite_expr
    // can disambiguate `store.Method()` across multiple stores by matching the
    // method name to the store that declares it.
    static STORE_MSG_MAP: RefCell<HashMap<String, HashSet<String>>> = RefCell::new(HashMap::new());
    // Plan 398 §2/§3 BUG-B + BUG-C: current widget's name + message-variant
    // names, so `.SiblingHandler()` calls inside a handler (both store handlers
    // calling sibling store handlers, AND child-widget handlers calling their
    // own sibling handlers, e.g. PromptBar's `.OnCtrlD` calling `.Exit()`) get
    // rewritten to `handler_<Widget>_<Sibling>(__state, args)` instead of being
    // misread as a state-field access (`__state.Exit` → bogus `<W>_State.X`
    // symbol at link time). Set per-widget in synthesize_from_decl.
    static CURRENT_WIDGET_NAME: RefCell<String> = RefCell::new(String::new());
    static CURRENT_MSG_VARIANTS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Plan 398: set the current widget (name + msg variants) for handler rewrite.
pub fn set_current_widget(name: &str, msg_variants: HashSet<String>) {
    CURRENT_WIDGET_NAME.with(|s| *s.borrow_mut() = name.to_string());
    CURRENT_MSG_VARIANTS.with(|s| *s.borrow_mut() = msg_variants);
}

/// Plan 398: clear the current-widget context after a widget's handlers are
/// rewritten.
pub fn clear_current_widget() {
    CURRENT_WIDGET_NAME.with(|s| s.borrow_mut().clear());
    CURRENT_MSG_VARIANTS.with(|s| s.borrow_mut().clear());
}

/// Set the store context for the current synthesis pass.
pub fn set_store_context(fields: HashMap<String, Vec<String>>, names: HashMap<String, String>) {
    STORE_FIELDS.with(|s| *s.borrow_mut() = fields);
    STORE_WIDGET_NAMES.with(|s| *s.borrow_mut() = names);
}

/// Set the store msg-variant map (VM multi-store fix).
pub fn set_store_msg_map(msgs: HashMap<String, HashSet<String>>) {
    STORE_MSG_MAP.with(|s| *s.borrow_mut() = msgs);
}

thread_local! {
    // Plan 446 批二 A1: store-method 调用在多 store 工程里撞名/未声明时，
    // 禁止静默 alias 回退——错误收集在此，synthesis 收尾统一升级为 Err。
    static STORE_DISAMBIG_ERRORS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

fn record_store_disambig_error(msg: String) {
    eprintln!("[HANDLER-CODEGEN] plan-446 A1: {}", msg);
    STORE_DISAMBIG_ERRORS.with(|s| s.borrow_mut().push(msg));
}

/// Drain collected A1 disambiguation errors (called at synthesis tail).
pub fn take_store_disambig_errors() -> Vec<String> {
    STORE_DISAMBIG_ERRORS.with(|s| std::mem::take(&mut *s.borrow_mut()))
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
    // Plan 401/VM-routing: `router.push(path)` → `__state.__current_route = path`.
    // `router.push(...)` parses as Expr::Call with name Dot(Ident("router"),
    // "push"); we rewrite the whole statement into a state assignment so the
    // outlet renderer (which reads __current_route) re-renders the new page.
    // Handles both bare statement form and the legacy `nav(...)` NavCall form.
    // Plan 482: `router.back()` → `__state.__nav_back_pending = true` — the
    // post-handler hook (dynamic::consume_nav_back_pending) pops the history
    // stack and restores the previous route (browser-back equivalent).
    if let Stmt::Expr(Expr::Call(call)) = stmt {
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "router" && method.as_str() == "back" {
                    *stmt = Stmt::Expr(Expr::Bina(
                        Box::new(Expr::Dot(
                            Box::new(Expr::Ident(Name::from(STATE_PARAM))),
                            Name::from("__nav_back_pending"),
                        )),
                        auto_val::Op::Asn,
                        Box::new(Expr::Bool(true)),
                    ));
                    return;
                }
            }
        }
    }
    let nav_path = match stmt {
        Stmt::Expr(Expr::Call(call)) => {
            if let Expr::Dot(obj, method) = call.name.as_ref() {
                if let Expr::Ident(name) = obj.as_ref() {
                    if name.as_str() == "router" && method.as_str() == "push" {
                        Some(call.args.args.first().and_then(|a| match a {
                            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => Some(e.clone()),
                            crate::ast::Arg::Name(_) => None,
                        }).unwrap_or(Expr::Str(auto_val::AutoStr::from(""))))
                    } else { None }
                } else { None }
            } else { None }
        }
        Stmt::Expr(Expr::NavCall { path, .. }) => Some((**path).clone()),
        _ => None,
    };
    if let Some(mut path_expr) = nav_path {
        rewrite_expr(&mut path_expr, state_fields);
        *stmt = Stmt::Expr(Expr::Bina(
            Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from(STATE_PARAM))),
                Name::from("__current_route"),
            )),
            auto_val::Op::Asn,
            Box::new(path_expr),
        ));
        return;
    }
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
        // Plan 446 批一(auto-musk 045 现场):try/catch/finally 体同样走查。
        // 此前缺臂,体内 `.state` 读写漏成裸 self → VM 合成 handler 报
        // "Undefined variable: self"(musk 七个 try/catch 形态 HTTP handler
        // 全体毒化)。镜像 Block 的处理。
        Stmt::Try(t) => {
            rewrite_state_refs_stmts(&mut t.body.stmts, state_fields);
            rewrite_state_refs_stmts(&mut t.catch_body.stmts, state_fields);
            if let Some(fb) = t.finally_body.as_mut() {
                rewrite_state_refs_stmts(&mut fb.stmts, state_fields);
            }
        }
        _ => {}
    }
}

fn rewrite_expr(e: &mut Expr, state_fields: &HashSet<String>) {
    // Plan 448 B2: compound assignment whose LHS is (or will become) a field
    // access desugars to `lhs = lhs op rhs` BEFORE the state rewrite. VM
    // codegen's compound-assignment path only accepts Ident LHS, while plain
    // `=` already handles Dot LHS via GET_FIELD/SET_FIELD — and the Phase-1
    // rewrite below turns every state reference (`.count`, bare `count`)
    // into `__state.count`, so without this desugar any `+=` on state in a
    // VM-synthesized handler aborts the whole widget's handler synthesis
    // ("Compound assignment requires a variable on left side"). Same
    // expansion shape the toast rewrite below builds by hand (Asn + Add).
    let compound: Option<(Expr, Expr, auto_val::Op)> = match e {
        Expr::Bina(lhs, op, rhs) => {
            let compound_op = match *op {
                auto_val::Op::AddEq => Some(auto_val::Op::Add),
                auto_val::Op::SubEq => Some(auto_val::Op::Sub),
                auto_val::Op::MulEq => Some(auto_val::Op::Mul),
                auto_val::Op::DivEq => Some(auto_val::Op::Div),
                auto_val::Op::ModEq => Some(auto_val::Op::Mod),
                _ => None,
            };
            let lhs_is_field = match lhs.as_ref() {
                Expr::Dot(..) => true,
                Expr::Ident(n) => state_fields.contains(n.as_str()),
                _ => false,
            };
            match (compound_op, lhs_is_field) {
                (Some(plain), true) => {
                    Some(((**lhs).clone(), (**rhs).clone(), plain))
                }
                _ => None,
            }
        }
        _ => None,
    };
    if let Some((lhs, rhs, plain)) = compound {
        let read = lhs.clone();
        *e = Expr::Bina(
            Box::new(lhs),
            auto_val::Op::Asn,
            Box::new(Expr::Bina(Box::new(read), plain, Box::new(rhs))),
        );
        rewrite_expr(e, state_fields);
        return;
    }
    // Plan 412 续(toast VM 化):toast()/toast.success()/… 不再是 vue-only
    // escape hatch。重写为 `__state.__toast += "\x1E" + "kind\x1Fmsg\x1Fposition\x1Fduration"`
    // (追加式赋值,VM handler 可执行):同一 handler 连发多条 toast 时逐条
    // 追加、互不覆盖,update 侧按 \x1E 分隔逐条消费入队;单条场景首字符是
    // 分隔符产生一个空记录,消费时跳过。renderer 的 dynamic_view 取走该
    // state 渲染窗口级悬浮层(位置/时长支持 position + duration 参数,默认
    // bottom-right / 4000ms,与 vue-sonner 对齐)。Vue codegen 不走此重写,
    // 仍生成真实 toast() 调用。
    if let Expr::Call(call) = e {
        let is_toast = match call.name.as_ref() {
            Expr::Ident(n) => n.as_str() == "toast",
            Expr::Dot(obj, _) => matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "toast"),
            _ => false,
        };
        if is_toast {
            let kind = match call.name.as_ref() {
                Expr::Dot(_, method) => method.to_string(),
                _ => "default".to_string(),
            };
            let mut msg = String::new();
            let mut position = "bottom-right".to_string();
            let mut duration: i64 = 4000;
            for arg in &call.args.args {
                match arg {
                    crate::ast::Arg::Pos(Expr::Str(s)) => msg = s.to_string(),
                    crate::ast::Arg::Pair(k, v) => match (k.as_str(), v) {
                        ("position", Expr::Str(s)) => position = s.to_string(),
                        ("duration", Expr::Int(n)) => duration = *n as i64,
                        _ => {}
                    },
                    _ => {}
                }
            }
            let payload = format!("{}\u{1f}{}\u{1f}{}\u{1f}{}", kind, msg, position, duration);
            let toast_field = || Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from(STATE_PARAM))),
                Name::from("__toast"),
            ));
            *e = Expr::Bina(
                toast_field(),
                auto_val::Op::Asn,
                Box::new(Expr::Bina(
                    Box::new(Expr::Bina(
                        toast_field(),
                        auto_val::Op::Add,
                        Box::new(Expr::Str(auto_val::AutoStr::from("\u{1e}"))),
                    )),
                    auto_val::Op::Add,
                    Box::new(Expr::Str(auto_val::AutoStr::from(payload))),
                )),
            );
            return;
        }
    }
    // Plan 401/VM-routing: `router.param("id")` → read the captured dynamic
    // segment from the __route_params state object: `__state.__route_params.id`.
    // __route_params is populated by the outlet renderer (render_outlet) when it
    // matches the current path against a route pattern (e.g. "/book/:id" →
    // { id: "3" }). The arg is a string literal naming the segment, so we build
    // a static field access Dot(__state.__route_params, <param_name>).
    if let Expr::Call(call) = e {
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "router" && method.as_str() == "param" {
                    // The param name is the first positional arg (a str literal).
                    let param_field = call.args.args.iter().find_map(|a| match a {
                        crate::ast::Arg::Pos(Expr::Str(s)) => Some(s.clone()),
                        _ => None,
                    }).unwrap_or_default();
                    *e = Expr::Dot(
                        Box::new(Expr::Dot(
                            Box::new(Expr::Ident(Name::from(STATE_PARAM))),
                            Name::from("__route_params"),
                        )),
                        Name::from(&param_field),
                    );
                    return;
                }
            }
        }
    }
    // Plan 370 D-GAP-4 Phase 0 + VM multi-store fix: store.X rewriting.
    // store.Method(args) → handler_StoreName_Method(__state, args)
    //
    // Plan 446 批二 A1: 撞名/未声明不再静默回退 alias 表（那会把调用生成到
    // 错误的 store，运行期零诊断）。规则：
    //   - 合格化调用 `Store.Method(...)`（alias 即某 store 真名、多 store 工程）
    //     → 直接按 alias 定位，不做方法名匹配；
    //   - 泛型接收 `store.Method(...)`：方法名恰好命中唯一 store → 用之；
    //     撞名（≥2）/未声明（0）→ 记录显式错误（synthesis 收尾升级为 Err），
    //     AST 仍按旧 alias 目标改写以保持整体一致；
    //   - 单 store 工程保持旧行为（vue 轨同样容忍 msg 缺声明）。
    if let Expr::Call(call) = e {
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if let Expr::Ident(alias) = obj.as_ref() {
                let legacy_target = STORE_WIDGET_NAMES.with(|s| s.borrow().get(alias.as_str()).cloned());
                if let Some(legacy_target) = legacy_target {
                    let multi_store = STORE_MSG_MAP.with(|s| s.borrow().len() > 1);
                    let qualified_call = multi_store && alias.as_str() != "store";
                    let store_name = if qualified_call {
                        Some(legacy_target)
                    } else {
                        let matched: Vec<String> = STORE_MSG_MAP.with(|s| {
                            let map = s.borrow();
                            map.iter()
                                .filter(|(_, msgs)| msgs.contains(method.as_str()))
                                .map(|(name, _)| name.clone())
                                .collect()
                        });
                        match matched.len() {
                            1 => Some(matched[0].clone()),
                            _ => {
                                if multi_store {
                                    if matched.is_empty() {
                                        record_store_disambig_error(format!(
                                            "store method `{}` is not declared by any store's Msg/handler set (stores present: {}); declare it in exactly one store's Msg or qualify the call",
                                            method,
                                            STORE_MSG_MAP.with(|s| {
                                                let names: Vec<String> =
                                                    s.borrow().keys().cloned().collect();
                                                format!("[{}]", names.join(", "))
                                            }),
                                        ));
                                    } else {
                                        record_store_disambig_error(format!(
                                            "ambiguous store method `{}` matches stores [{}]; qualify the call as `Store.{}` or rename one",
                                            method,
                                            matched.join(", "),
                                            method,
                                        ));
                                    }
                                }
                                Some(legacy_target)
                            }
                        }
                    };
                    let store_name = store_name.unwrap_or_default();
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
                        generic_args: Vec::new(),
                        pos: None,
                    });
                    return;
                }
            }
        }
    }
    // Plan 398 §2/§3 (BUG-B + BUG-C): sibling-handler call rewriting.
    // `.Sibling(args)` where the receiver is implicit self (`.X` / `self.X`)
    // and `Sibling` is a msg variant of the CURRENT widget → rewrite to
    // `handler_<CurrentWidget>_<Sibling>(__state, args)`.
    //
    // Without this, `.Exit()` inside PromptBar's `.OnCtrlD` handler (and
    // `.RefreshGit()` inside ShellStore's `.Init`) fall through to the
    // `.field` rewrite below → `__state.Exit` → codegen emits a CALL to the
    // bogus symbol `<Widget>_State.Exit` → "Undefined symbol: <W>_State.X".
    if let Expr::Call(call) = e {
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            let is_self_receiver = matches!(
                obj.as_ref(),
                Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self"
            );
            if is_self_receiver {
                let is_msg_variant = CURRENT_MSG_VARIANTS.with(|s| s.borrow().contains(method.as_str()));
                if is_msg_variant {
                    let widget_name = CURRENT_WIDGET_NAME.with(|s| s.borrow().clone());
                    if !widget_name.is_empty() {
                        let handler_fn = format!("handler_{}_{}", widget_name, method);
                        let mut new_args = vec![crate::ast::Arg::Pos(Expr::Ident(Name::from(STATE_PARAM)))];
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
                            generic_args: Vec::new(),
                            pos: None,
                        });
                        return;
                    }
                }
            }
        }
    }
    // store.field → __state.field (store fields merged into root state)
    // Plan 423 P5 修复:先经 STORE_WIDGET_NAMES 把别名(store)解析成真名
    // (NotesStore)再查 STORE_FIELDS —— 与 store.Method() 路径同型。此前
    // 直接 contains_key(alias),而字段表按真名键控,别名永远查不中 →
    // 裸 `store` 标识符漏进 codegen("Undefined variable: store"),handler
    // 半成品毒化字节码(导出指向非 prolog → 运行时栈失衡/InvalidOpCode 255/
    // 垃圾指令跑满步数预算无界吃内存,实机 20G 内存事故的源头)。
    if let Expr::Dot(obj, _field) = e {
        if let Expr::Ident(alias) = obj.as_ref() {
            let real = STORE_WIDGET_NAMES.with(|s| s.borrow().get(alias.as_str()).cloned());
            let target = real.as_deref().unwrap_or(alias.as_str());
            let is_store = STORE_FIELDS.with(|s| s.borrow().contains_key(target));
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
                let real = STORE_WIDGET_NAMES.with(|s| s.borrow().get(store_alias.as_str()).cloned());
                let target = real.as_deref().unwrap_or(store_alias.as_str());
                let is_store = STORE_FIELDS.with(|s| s.borrow().contains_key(target));
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
    format!("handler_{}", bare_handler_name(pattern))
}

/// The bare event name of a handler pattern: strips the leading `.` and any
/// `Msg::`-style qualifier. Shared by [`handler_fn_name`] and the sibling-call
/// visibility sets below (Plan 056 blocker A).
///
/// Plan 423 P5 续修:也剥参数列表 —— `on { .SelectDay(date) -> }` 的模式带
/// `(date)`,此前名字进了导出表(`handler_App_SelectDay(date)`),而分发按
/// 无参名查找 → HandlerNotFound(016-calendar SelectDay 点击静默失败)。
pub fn bare_handler_name(pattern: &str) -> &str {
    let name = pattern.trim_start_matches('.');
    let name = name.split('(').next().unwrap_or(name);
    name.rfind("::").map(|p| &name[p + 2..]).unwrap_or(name)
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
            // Plan 043 M5 #1: payload is Vec<Type>; the handler gets a single
            // param slot, so use the first payload type.
            if let Some(ty) = v.payload.first() {
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
        consts: Vec::new(),
        attrs: Vec::new(),
        impl_attrs: Vec::new(), // Plan 364 W1
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
    //
    // Plan 340 audit: only register a BARE-name alias when the name is unique
    // across all imported #[api] fns. If two modules export the same bare name
    // (e.g. db.create_note AND api.create_note), a bare call is ambiguous, so
    // we skip the alias (last-write-wins would silently route to the wrong
    // endpoint). This mirrors the import_scope bare_counts guard below.
    // Plan 060 M3:host 分派同样需要 api_funcs 元数据(bare 名 → 端点)。
    let host_mode = crate::vm::host_bridge::has_host_calls();
    if api_over_http || host_mode {
        // Count how many imported #[api] fns define each bare name.
        let mut bare_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if f.api_attrs.is_some() {
                    if let Some(bare) = f.name.to_string().split('.').last() {
                        *bare_counts.entry(bare.to_string()).or_default() += 1;
                    }
                }
            }
        }
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if let Some(api) = &f.api_attrs {
                    let bare = f.name.to_string().split('.').last()
                        .unwrap_or(&f.name.to_string()).to_string();
                    // Skip ambiguous bare names — caller must qualify them.
                    if bare_counts.get(&bare).copied().unwrap_or(0) > 1 {
                        continue;
                    }
                    let params: Vec<String> = f.params.iter()
                        .map(|p| p.name.to_string()).collect();
                    codegen.api_funcs.insert(bare, crate::vm::codegen::ApiCallInfo {
                        fn_name: f.name.to_string(),
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
                eprintln!("[HANDLER-CODEGEN] import stmt failed to compile: {}", e);
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

        // Plan 056 blocker A (AuraWidget path): set the current-widget context
        // (msg variants + this widget's own handler/lifecycle patterns) so
        // `.Sibling()` calls inside handler bodies rewrite to
        // `handler_<W>_<Sibling>(__state, args)` — same as the decl path below.
        let mut w_msg_variants: HashSet<String> = w
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.to_string()))
            .collect();
        for (pattern, _) in &w.handlers {
            w_msg_variants.insert(bare_handler_name(pattern).to_string());
        }
        for lc in &w.lifecycle {
            w_msg_variants.insert(lc.name.clone());
        }
        set_current_widget(&w.name, w_msg_variants);

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
                eprintln!(
                    "[HANDLER-CODEGEN] {}.{} failed: {}",
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
    clear_store_context();
    clear_current_widget();
    // Plan 446 批二 A1: 消歧失败升级为 synthesis 失败 → boot 致命
    // （与 C1-3 同哲学：显式报错优于静默错路由）。
    let disambig_errors = take_store_disambig_errors();
    if !disambig_errors.is_empty() {
        return Err(format!(
            "{} store call(s) failed multi-store disambiguation (plan-446 A1):\n  {}",
            disambig_errors.len(),
            disambig_errors.join("\n  ")
        ));
    }
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
        consts: Vec::new(),
        attrs: Vec::new(),
        impl_attrs: Vec::new(), // Plan 364 W1
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
            if let Some(ty) = v.payload.first() {
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
    // Plan 340 audit: skip ambiguous bare names (see first synth site above).
    // Plan 060 M3:host 分派同样需要(bare 名 → 端点)。
    if api_over_http || crate::vm::host_bridge::has_host_calls() {
        let mut bare_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if f.api_attrs.is_some() {
                    if let Some(bare) = f.name.to_string().split('.').last() {
                        *bare_counts.entry(bare.to_string()).or_default() += 1;
                    }
                }
            }
        }
        for stmt in &import_stmts {
            if let Stmt::Fn(f) = stmt {
                if let Some(api) = &f.api_attrs {
                    let bare = f.name.to_string().split('.').last()
                        .unwrap_or(&f.name.to_string()).to_string();
                    if bare_counts.get(&bare).copied().unwrap_or(0) > 1 {
                        continue;
                    }
                    let params: Vec<String> = f.params.iter()
                        .map(|p| p.name.to_string()).collect();
                    codegen.api_funcs.insert(bare, crate::vm::codegen::ApiCallInfo {
                        fn_name: f.name.to_string(),
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
                eprintln!("[HANDLER-CODEGEN] import stmt failed to compile: {}", e);
            }
        }
    }

    // 2. Compile state types + handlers for ALL widgets (root + children),
    //    reading directly from WidgetDecl.
    let all_decls: Vec<&crate::ast::WidgetDecl> = std::iter::once(decl)
        .chain(child_decls.iter())
        .collect();

    // Plan 370 D-GAP-4 + VM multi-store fix: collect ALL stores by their real
    // name (not a hardcoded "store" key), plus each store's msg variants so
    // rewrite_expr can match `store.Method()` to the correct store by method
    // name when multiple stores share the alias "store".
    let mut store_fields_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut store_widget_names: HashMap<String, String> = HashMap::new();
    let mut store_msg_map: HashMap<String, HashSet<String>> = HashMap::new();
    let mut all_store_names: Vec<String> = Vec::new();
    for d in &all_decls {
        if d.view.is_none() {
            let fields: Vec<String> = d.model
                .as_ref()
                .map(|m| m.fields.iter().map(|f| f.name.to_string()).collect())
                .unwrap_or_default();
            let sname = d.name.to_string();
            // Plan 446 批二 A1: 消歧集合 = Msg 变体 ∪ on-block 处理器 ∪ 生命周期名。
            // 此前只收 Msg 变体——"handler 已定义但漏列 Msg 声明"（vue 容忍、
            // os-config SetSidecar 现场）会查不到，静默回退到错误 store。
            let mut msgs: HashSet<String> = d.messages
                .iter()
                .flat_map(|m| m.variants.iter().map(|v| v.name.to_string()))
                .collect();
            if let Some(on) = &d.on {
                for h in &on.handlers {
                    msgs.insert(bare_handler_name(&h.pattern).to_string());
                }
            }
            for lc in &d.lifecycle {
                msgs.insert(lc.name.clone());
            }
            all_store_names.push(sname.clone());
            store_fields_map.insert(sname.clone(), fields.clone());
            store_widget_names.insert(sname.clone(), sname.clone());
            store_msg_map.insert(sname.clone(), msgs);
            // Backward-compat: also keep the legacy "store" key pointing at
            // the first store (single-store projects still work unchanged).
            if !store_widget_names.contains_key("store") {
                store_widget_names.insert("store".to_string(), d.name.to_string());
            }
        }
    }

    // Set thread-local store context so rewrite_expr can access it.
    set_store_context(store_fields_map.clone(), store_widget_names.clone());
    set_store_msg_map(store_msg_map.clone());

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
        // Plan 398 §2/§3: set current widget (name + msg variants) so handler
        // bodies that call a sibling handler (`.Sibling()`) rewrite to
        // `handler_<Widget>_<Sibling>(__state, args)` instead of a bogus
        // `<Widget>_State.Sibling` field access.
        //
        // Plan 056 blocker A: the sibling-call target may also be an `on`-block
        // handler or a lifecycle fn — those compile to the same
        // `handler_<W>_<H>` symbols as msg variants, but were missing from the
        // visibility set, so e.g. PromptBar's `.OnInput` calling
        // `.OnInputComplete()` fell through to the field-access path and linked
        // against the nonexistent `PromptBar_State.OnInputComplete`.
        let mut d_msg_variants: HashSet<String> = d
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.to_string()))
            .collect();
        if let Some(on) = &d.on {
            for h in &on.handlers {
                d_msg_variants.insert(bare_handler_name(&h.pattern).to_string());
            }
        }
        for lc in &d.lifecycle {
            d_msg_variants.insert(lc.name.clone());
        }
        set_current_widget(d.name.to_string().as_str(), d_msg_variants);

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
                eprintln!(
                    "[HANDLER-CODEGEN] {}.{} failed: {}",
                    d.name, event_pattern, e
                );
            }
        }
    }

    let registry = std::mem::take(&mut codegen.generic_registry);
    clear_store_context();
    clear_current_widget();
    // Plan 446 批二 A1: 消歧失败升级为 synthesis 失败 → boot 致命。
    let disambig_errors = take_store_disambig_errors();
    if !disambig_errors.is_empty() {
        return Err(format!(
            "{} store call(s) failed multi-store disambiguation (plan-446 A1):\n  {}",
            disambig_errors.len(),
            disambig_errors.join("\n  ")
        ));
    }
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

    /// Plan 482: `router.back()` → `__state.__nav_back_pending = true`
    /// （post-handler 钩子消费该标记弹历史栈）。
    #[test]
    fn rewrites_router_back_to_pending_flag() {
        let mut stmt = Stmt::Expr(Expr::Call(Call {
            name: Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from("router"))),
                Name::from("back"),
            )),
            args: Args { args: vec![] },
            type_args: vec![],
            generic_args: vec![],
            pos: None,
            ret: crate::ast::Type::Void,
        }));
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &make_state_fields(&[]));
        match &stmt {
            Stmt::Expr(Expr::Bina(lhs, Op::Asn, rhs)) => {
                match lhs.as_ref() {
                    Expr::Dot(obj, field) => {
                        assert!(matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "__state"));
                        assert_eq!(field.as_str(), "__nav_back_pending");
                    }
                    other => panic!("期望 __state.__nav_back_pending 赋值,得到 {other:?}"),
                }
                assert!(matches!(rhs.as_ref(), Expr::Bool(true)));
            }
            other => panic!("期望赋值语句,得到 {other:?}"),
        }
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
    fn rewrites_state_refs_inside_try_catch() {
        // Plan 446 批一 regression (auto-musk 045 现场): handler 体内
        // try { .error = "" } catch { .configs = .configs } 的 `.x` 读写
        // 此前不被走查(rewrite_stmt 无 Try 臂),漏成裸 self → VM 合成报
        // "Undefined variable: self"。修复后 try/catch/finally 三体均重写。
        let mk_body = |stmts: Vec<Stmt>| Body {
            stmts,
            has_new_line: true,
            source_lines: Vec::new(),
        };
        let assign = |field: &str| {
            Stmt::Expr(Expr::Bina(
                Box::new(Expr::Dot(
                    Box::new(Expr::Ident(Name::from("."))),
                    Name::from(field),
                )),
                Op::Asn,
                Box::new(Expr::Dot(
                    Box::new(Expr::Ident(Name::from("self"))),
                    Name::from(field),
                )),
            ))
        };
        let mut stmt = Stmt::Try(Try {
            body: mk_body(vec![assign("error")]),
            catch_param: None,
            catch_body: mk_body(vec![assign("configs")]),
            finally_body: Some(mk_body(vec![assign("loading")])),
            new_line: true,
        });
        let fields = make_state_fields(&["error", "configs", "loading"]);
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &fields);
        match &stmt {
            Stmt::Try(t) => {
                // Stmt 的 Display 不递归,逐体断言(裸 self/. 必须消失)
                let bodies = [
                    format!("{}", t.body),
                    format!("{}", t.catch_body),
                    t.finally_body.as_ref().map(|f| format!("{}", f)).unwrap_or_default(),
                ];
                for (i, rendered) in bodies.iter().enumerate() {
                    assert!(rendered.contains("(name __state)"), "body[{i}]: {rendered}");
                    assert!(!rendered.contains("(name self)"), "body[{i}]: {rendered}");
                    assert!(!rendered.contains("(name .)"), "body[{i}]: {rendered}");
                }
                assert!(bodies[0].matches(".error").count() >= 2, "{}", bodies[0]);
                assert!(bodies[1].matches(".configs").count() >= 2, "{}", bodies[1]);
                assert!(bodies[2].matches(".loading").count() >= 2, "{}", bodies[2]);
            }
            other => panic!("expected Try, got {:?}", other),
        }
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
            generic_args: Vec::new(),
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
            is_pub: false,
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
        // Plan 423 P5 续修:带参数列表的模式必须剥 `(date)` —— 否则导出名
        // 与分发查找名错位 → HandlerNotFound。
        assert_eq!(handler_fn_name(".SelectDay(date)"), "handler_SelectDay");
        assert_eq!(handler_fn_name("Msg::Add(str)"), "handler_Add");
    }

    // ---- Plan 398 §14.1 regression tests: sibling-handler call rewriting ----
    // `.Sibling()` inside a widget's own handler must become
    // `handler_<Widget>_<Sibling>(__state, ...)` instead of falling through to
    // the `.field` path and emitting a bogus `<W>_State.Sibling` symbol.

    fn sibling_call(method: &str, args: Vec<crate::ast::Arg>) -> Stmt {
        Stmt::Expr(Expr::Call(Call {
            name: Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from("."))),
                Name::from(method),
            )),
            args: Args { args },
            ret: Type::Void,
            type_args: Vec::new(),
            generic_args: Vec::new(),
            pos: None,
        }))
    }

    #[test]
    fn rewrites_sibling_handler_call_to_handler_fn() {
        let mut stmt = sibling_call("Exit", vec![]);
        set_current_widget(
            "PromptBar",
            ["Exit", "OnCtrlD"].iter().map(|s| s.to_string()).collect(),
        );
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &HashSet::new());
        clear_current_widget();
        match &stmt {
            Stmt::Expr(Expr::Call(c)) => {
                let rendered = format!("{}", c.name);
                assert!(
                    rendered.contains("handler_PromptBar_Exit"),
                    "sibling call must become handler fn, got: {}",
                    rendered
                );
                // __state is injected as the first positional arg
                assert_eq!(c.args.args.len(), 1, "state param injected");
            }
            other => panic!("expected Call after rewrite, got {:?}", other),
        }
    }

    #[test]
    fn rewrites_sibling_handler_call_with_args() {
        let mut stmt = sibling_call("Refresh", vec![crate::ast::Arg::Pos(Expr::Int(5))]);
        set_current_widget(
            "ShellStore",
            ["Refresh", "Init"].iter().map(|s| s.to_string()).collect(),
        );
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &HashSet::new());
        clear_current_widget();
        match &stmt {
            Stmt::Expr(Expr::Call(c)) => {
                let rendered = format!("{}", c.name);
                assert!(
                    rendered.contains("handler_ShellStore_Refresh"),
                    "got: {}",
                    rendered
                );
                // __state + the original arg
                assert_eq!(c.args.args.len(), 2, "__state + original arg");
            }
            other => panic!("expected Call after rewrite, got {:?}", other),
        }
    }

    #[test]
    fn does_not_rewrite_non_msg_variant_dot_call() {
        // `.notAVariant()` — method not a msg variant of the current widget
        // and not a state field → must NOT become a handler_ call.
        let mut stmt = sibling_call("notAVariant", vec![]);
        set_current_widget("PromptBar", ["Exit"].iter().map(|s| s.to_string()).collect());
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &HashSet::new());
        clear_current_widget();
        match &stmt {
            Stmt::Expr(Expr::Call(c)) => {
                let rendered = format!("{}", c.name);
                assert!(
                    !rendered.contains("handler_"),
                    "non-variant call must stay a Dot call, got: {}",
                    rendered
                );
                assert_eq!(c.args.args.len(), 0, "no state param injected");
            }
            other => panic!("expected Call, got {:?}", other),
        }
    }

    // ---- Plan 446 批二 A1: multi-store disambiguation --------------------
    // Two failure modes reported by auto-os-config (§A1):
    //   1. method defined in store B but missing from its Msg list → the map
    //      lookup missed it and silently fell back to the FIRST-registered
    //      store (Modules), yielding "Undefined symbol: handler_Modules_X".
    //   2. same method declared in two stores → silent fallback always picked
    //      one of them. Must error with the candidate list instead.
    // Fix: the per-store handler-name set includes on-block + lifecycle names
    // (not just msg variants); qualified calls (`Collection.Init`) resolve via
    // the alias directly; ambiguous/missing generic-receiver calls record an
    // explicit error and fail synthesis.

    fn two_store_context() {
        set_store_context(
            [
                ("Modules".to_string(), Vec::<String>::new()),
                ("Collection".to_string(), Vec::<String>::new()),
            ]
            .into_iter()
            .collect(),
            [
                ("store".to_string(), "Modules".to_string()),
                ("Modules".to_string(), "Modules".to_string()),
                ("Collection".to_string(), "Collection".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        set_store_msg_map(
            [
                (
                    "Modules".to_string(),
                    ["Init", "Select"].iter().map(|s| s.to_string()).collect(),
                ),
                (
                    "Collection".to_string(),
                    ["Pick"].iter().map(|s| s.to_string()).collect(),
                ),
            ]
            .into_iter()
            .collect(),
        );
    }

    fn store_call(receiver: &str, method: &str) -> Stmt {
        Stmt::Expr(Expr::Call(Call {
            name: Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from(receiver))),
                Name::from(method),
            )),
            args: Args { args: vec![] },
            ret: Type::Void,
            type_args: Vec::new(),
            generic_args: Vec::new(),
            pos: None,
        }))
    }

    fn rewritten_target(stmt: &Stmt) -> String {
        match stmt {
            Stmt::Expr(Expr::Call(c)) => format!("{}", c.name),
            other => panic!("expected Call after rewrite, got {:?}", other),
        }
    }

    #[test]
    fn plan446_a1_qualified_store_call_resolves_by_alias_not_method_match() {
        // `Collection.Init(...)` — Init also exists on Modules but Collection's
        // own handler set contains it; the explicit alias must win outright.
        two_store_context();
        // Simulate A1 case 1: defined as an on-handler but NOT in the msg list.
        STORE_MSG_MAP.with(|s| {
            s.borrow_mut()
                .get_mut("Collection")
                .unwrap()
                .insert("Init".to_string());
        });
        let mut stmt = store_call("Collection", "Init");
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &HashSet::new());
        clear_store_context();
        assert!(
            rewritten_target(&stmt).contains("handler_Collection_Init"),
            "qualified call must target the aliased store, got: {}",
            rewritten_target(&stmt)
        );
    }

    #[test]
    fn plan446_a1_ambiguous_store_method_records_error_and_names_candidates() {
        // Generic `store.Init` where BOTH stores declare Init: previously this
        // silently picked Modules. Now records an explicit error listing both.
        two_store_context();
        STORE_MSG_MAP.with(|s| {
            s.borrow_mut().get_mut("Collection").unwrap().insert("Init".to_string());
        });
        take_store_disambig_errors(); // drain pre-existing
        let mut stmt = store_call("store", "Init");
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &HashSet::new());
        let errs = take_store_disambig_errors();
        clear_store_context();
        assert!(!errs.is_empty(), "ambiguity must be recorded, not silent");
        let joined = errs.join("; ");
        assert!(joined.contains("Init"), "error must name the method: {}", joined);
        assert!(joined.contains("Modules") && joined.contains("Collection"),
            "error must list candidates: {}", joined);
        // AST stays coherent (legacy target) so synthesis reports all sites at once.
        assert!(rewritten_target(&stmt).contains("handler_Modules_Init"));
    }

    #[test]
    fn plan446_a1_undeclared_store_method_records_error_in_multi_store_project() {
        // Method nowhere in any store's handler set: silent first-store fallback
        // is what produced link-time Undefined symbol with zero diagnostics.
        two_store_context();
        take_store_disambig_errors();
        let mut stmt = store_call("store", "SetSidecar");
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &HashSet::new());
        let errs = take_store_disambig_errors();
        clear_store_context();
        assert!(!errs.is_empty(), "undeclared method in multi-store project must be flagged");
        let joined = errs.join("; ");
        assert!(joined.contains("SetSidecar"), "{}", joined);
        assert!(joined.contains("Collection"), "must hint candidate stores: {}", joined);
    }

    #[test]
    fn plan446_a1_single_store_undeclared_method_still_falls_back_silently() {
        // Single-store compat (vue parity): no msg data required, legacy fallback
        // resolves to the only store without diagnostics.
        set_store_context(
            [("Notes".to_string(), Vec::<String>::new())].into_iter().collect(),
            [
                ("store".to_string(), "Notes".to_string()),
                ("Notes".to_string(), "Notes".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        set_store_msg_map([("Notes".to_string(), HashSet::new())].into_iter().collect());
        take_store_disambig_errors();
        let mut stmt = store_call("store", "Remove");
        rewrite_state_refs_stmts(std::slice::from_mut(&mut stmt), &HashSet::new());
        let errs = take_store_disambig_errors();
        clear_store_context();
        assert!(errs.is_empty(), "single-store fallback must stay silent, got: {:?}", errs);
        assert!(rewritten_target(&stmt).contains("handler_Notes_Remove"));
    }
}
