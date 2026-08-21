//! TypeScript adapter for AURA handler bodies.
//!
//! Wraps the a2ts transpiler to convert AutoLang AST fragments (from `on` blocks)
//! into TypeScript code, applying UI-specific rewrites:
//! - StateRef (`.count` / `self.count`) → `count.value` ref access
//! - API function calls → `await` prefix
//! - `print()` → `console.log()`
//!
//! Everything else (control flow, types, closures, pattern matching)
//! is delegated to the a2ts transpiler for standard expressions.

use crate::ast::*;
use crate::trans::Sink;
use crate::trans::typescript::TypeScriptTrans;
use std::collections::HashSet;
use std::io::Write;

/// Context for UI-specific rewrites during TypeScript generation.
pub struct AuraTsContext {
    /// Names of reactive state variables (need `.value` in Vue).
    pub state_names: HashSet<String>,
    /// Names of component props (need `props.` prefix in script, no `.value`).
    pub prop_names: HashSet<String>,
    /// Names of template refs declared in the view (`ref: "menuEl"`).
    /// `.menuEl` maps to `menuEl.value!` (the DOM element behind the ref).
    pub ref_names: HashSet<String>,
    /// Plan 408 P10 / §7.8 缺陷 8: names of computed properties (need
    /// `.value` in Vue script, same as state — a ComputedRef is unwrapped
    /// the same way a ref is). Wired into `Expr::Dot`/`Expr::Ident` so the
    /// IIFE/multi-statement computed-body paths (ts_adapter) unwrap computed
    /// refs consistently with the single-expr path (vue.rs expr_to_js).
    pub computed_names: HashSet<String>,
    /// Known API function names (need `await` prefix).
    api_functions: Vec<String>,
    /// Plan 012 Batch A (gap 19): state/prop names whose declared type is
    /// array-ish (`T[]`). Only these receivers get the `.remove → .splice`
    /// method mapping; anything else passes through unchanged.
    typed_arrays: HashSet<String>,
    /// Plan 012 Batch A (gap 19): state/prop names whose declared type is
    /// `string`. Together with `typed_arrays`, these are the proven receivers
    /// for `.contains → .includes`.
    typed_strings: HashSet<String>,
    /// Plan 012 Batch A (gap 19): names known to be facade/plain objects
    /// (widget `use { composable: ... }` locals, e.g. `recentFilesStore`).
    /// Their methods always pass through — they are never arrays.
    facade_names: HashSet<String>,
    /// Plan 408 P12 §10.4: composable ref 字段标注——key = facade local name，
    /// value = 标注为 ref 的字段名集合。ts_adapter 的 Dot 分支对命中字段加 `.value`。
    facade_ref_fields: std::collections::HashMap<String, HashSet<String>>,
    /// Plan 053 M5/P5-6: when Some(seq_var), this context is transpiling the
    /// body of a debounced complete-handler (wrapped in setTimeout). State-ref
    /// assignments (`.suggestions = x`) get guarded with
    /// `if (<seq_var> === __completeSeq) { ... }` so a stale complete result
    /// (an older in-flight request resolving after a newer one) can't clobber
    /// the current suggestions. None = normal (non-debounced) transpilation.
    debounce_seq_var: Option<String>,
    /// Plan 012 Batch A: passthrough notes collected during transpilation.
    /// Drained by the caller into the unified codegen warning channel.
    warnings: std::cell::RefCell<Vec<String>>,
}

/// Default API function names (fallback when no dynamic list is provided)
const DEFAULT_API_FUNCTIONS: &[&str] = &[
    "listusers",
    "getuser",
    "getUser",
    "createUser",
    "updateUser",
    "deleteUser",
];

impl AuraTsContext {
    pub fn new(state_names: HashSet<String>) -> Self {
        Self {
            state_names,
            prop_names: HashSet::new(),
            ref_names: HashSet::new(),
            computed_names: HashSet::new(),
            api_functions: DEFAULT_API_FUNCTIONS.iter().map(|s| s.to_string()).collect(),
            typed_arrays: HashSet::new(),
            typed_strings: HashSet::new(),
            facade_names: HashSet::new(),
            facade_ref_fields: std::collections::HashMap::new(),
            debounce_seq_var: None,
            warnings: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn with_props(mut self, prop_names: HashSet<String>) -> Self {
        self.prop_names = prop_names;
        self
    }

    /// Plan 408 P10: set computed property names so the IIFE/multi-statement
    /// computed-body paths unwrap ComputedRef via `.value`, matching the
    /// single-expr path in vue.rs::expr_to_js.
    pub fn with_computed(mut self, computed_names: HashSet<String>) -> Self {
        self.computed_names = computed_names;
        self
    }

    /// Set template ref names (declared via `ref: "name"` in the view).
    pub fn with_refs(mut self, ref_names: HashSet<String>) -> Self {
        self.ref_names = ref_names;
        self
    }

    /// Set custom API function names (from project's api.at)
    pub fn with_api_functions(mut self, functions: Vec<String>) -> Self {
        self.api_functions = functions;
        self
    }

    /// Plan 012 Batch A (gap 19): declare which state/prop names are proven
    /// arrays / strings, so the `.remove → .splice` and `.contains →
    /// .includes` mappings apply ONLY to them. Other receivers pass through.
    pub fn with_typed_collections(
        mut self,
        arrays: HashSet<String>,
        strings: HashSet<String>,
    ) -> Self {
        self.typed_arrays = arrays;
        self.typed_strings = strings;
        self
    }

    /// Plan 012 Batch A: record a passthrough note (drained by the caller
    /// into the unified codegen warning channel).
    fn note_warning(&self, message: String) {
        self.warnings.borrow_mut().push(message);
    }

    /// Plan 012 Batch A: drain collected passthrough notes.
    pub fn take_warnings(&self) -> Vec<String> {
        self.warnings.borrow_mut().drain(..).collect()
    }

    fn is_state(&self, name: &str) -> bool {
        self.state_names.contains(name)
    }

    fn is_prop(&self, name: &str) -> bool {
        self.prop_names.contains(name)
    }

    /// Plan 408 P10: a computed property name — unwraps via `.value` like state.
    fn is_computed(&self, name: &str) -> bool {
        self.computed_names.contains(name)
    }

    fn is_ref(&self, name: &str) -> bool {
        self.ref_names.contains(name)
    }

    fn is_typed_array(&self, name: &str) -> bool {
        self.typed_arrays.contains(name)
    }

    fn is_typed_string(&self, name: &str) -> bool {
        self.typed_strings.contains(name)
    }

    /// Plan 012 Batch A (gap 19): declare facade/plain-object names (widget
    /// `use { composable: ... }` locals). Their `.remove`/`.contains` calls
    /// always pass through — never mapped to `.splice`/`.includes`.
    pub fn with_facade_names(mut self, names: HashSet<String>) -> Self {
        self.facade_names = names;
        self
    }

    /// Plan 408 P12 §10.4: composable ref 字段标注。ts_adapter 的 Dot 分支对
    /// 命中字段注入 `.value`。
    pub fn with_facade_ref_fields(mut self, fields: std::collections::HashMap<String, HashSet<String>>) -> Self {
        self.facade_ref_fields = fields;
        self
    }

    /// Plan 053 M5/P5-6: mark this context as transpiling a debounced
    /// complete-handler body. `seq_var` is the local holding the captured
    /// sequence number (e.g. `__seq`); state-ref assignments inside the body
    /// get guarded against stale complete results.
    pub fn with_debounce_seq_var(mut self, seq_var: String) -> Self {
        self.debounce_seq_var = Some(seq_var);
        self
    }

    fn is_facade(&self, name: &str) -> bool {
        self.facade_names.contains(name)
    }

    fn is_api(&self, name: &str) -> bool {
        self.api_functions.iter().any(|f| f == name)
    }
}

/// Plan 012 Batch A (gap 19): decide whether the legacy
/// `.remove → .splice(idx, 1)` / `.contains → .includes` mapping applies to a
/// method-call receiver.
///
/// The mapping was originally applied to ANY receiver — including store
/// facades and ext objects held in state refs (`.recentFilesStore.remove(x)`
/// mis-emitted as `.splice(x, 1)`), which silently broke at runtime while
/// vue-tsc stayed green. Now only receivers PROVEN to be arrays (or strings,
/// for `.contains`) are mapped; typed non-array state/props and `store.*`
/// chains pass through unchanged with a note; unknown receivers (locals,
/// loop data) keep the legacy mapping.
///
/// `Map` covers `contains` only when the receiver is proven string/array;
/// `Pass`/`PassWarn` skip the mapping.
pub enum MethodMapDecision {
    /// Apply the legacy mapping (proven array/string, or unknown receiver).
    Map,
    /// Pass the call through unchanged, silently (DOM element refs — their
    /// own `.remove()` is a real method).
    Pass,
    /// Pass the call through unchanged and record a note (typed non-array
    /// state/prop, or a `store.*` facade chain).
    PassWarn,
}

fn method_map_decision(method: &str, object: &Expr, ctx: &AuraTsContext) -> MethodMapDecision {
    // `.field` / `self.field` receiver → the widget member name.
    if let Expr::Dot(obj, field) = object {
        if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "self" || n.as_str() == ".") {
            let field = field.as_str();
            if ctx.is_typed_array(field) {
                return MethodMapDecision::Map;
            }
            if method == "contains" && ctx.is_typed_string(field) {
                return MethodMapDecision::Map;
            }
            if ctx.is_state(field) || ctx.is_prop(field) {
                // A state/prop ref with a NON-array declared type — e.g. a
                // store facade held in a model var. Its own method wins.
                return MethodMapDecision::PassWarn;
            }
            if ctx.is_ref(field) {
                return MethodMapDecision::Pass;
            }
            if ctx.is_facade(field) {
                // `use { composable: ... }` local — a facade object, never an
                // array. Its own method wins.
                return MethodMapDecision::PassWarn;
            }
            return MethodMapDecision::Map; // unknown member — legacy behavior
        }
    }
    // Root identifier of a (possibly nested) receiver chain.
    fn chain_root(object: &Expr) -> Option<&str> {
        match object {
            Expr::Ident(n) => Some(n.as_str()),
            Expr::Dot(obj, _) => chain_root(obj),
            _ => None,
        }
    }
    match chain_root(object) {
        Some("store") => MethodMapDecision::PassWarn, // store facade chain
        Some(root) if ctx.is_ref(root) => MethodMapDecision::Pass,
        Some(root) if ctx.is_facade(root) => MethodMapDecision::PassWarn,
        Some(_) => {
            if let Expr::Ident(name) = object {
                let n = name.as_str();
                if ctx.is_typed_array(n) {
                    return MethodMapDecision::Map;
                }
                if method == "contains" && ctx.is_typed_string(n) {
                    return MethodMapDecision::Map;
                }
                if ctx.is_state(n) || ctx.is_prop(n) {
                    return MethodMapDecision::PassWarn;
                }
            }
            MethodMapDecision::Map // locals / nested data — legacy behavior
        }
        None => MethodMapDecision::Map, // calls, index exprs — legacy behavior
    }
}

/// Render a receiver expression to a short string for warning messages.
fn expr_brief(expr: &Expr, ctx: &AuraTsContext) -> String {
    let mut tmp = Vec::new();
    transpile_expr(expr, ctx, &mut tmp);
    String::from_utf8(tmp).unwrap_or_else(|_| "?".to_string())
}

/// Heuristic: does this expression contain a float/double literal (or a
/// nested sub-expression that does)? Used to decide whether `/` should be
/// integer division (`Math.trunc(a/b)`) or JS float division (`a/b`).
///
/// AutoLang `/` on two ints is integer division (matches VM `DIV` opcode =
/// `wrapping_div`), but the TS adapter has no type symbol table, so this is a
/// conservative structural check: if neither operand visibly contains a float
/// literal, treat it as integer division. Variable operands of unknown type
/// fall through to float division (numerically safe, just maybe not truncated).
fn expr_looks_float(expr: &Expr) -> bool {
    match expr {
        Expr::Float(_, _) | Expr::Double(_, _) => true,
        Expr::Bina(lhs, _, rhs) => expr_looks_float(lhs) || expr_looks_float(rhs),
        Expr::Unary(_, inner) => expr_looks_float(inner),
        Expr::Block(body) => body.stmts.iter().any(|s| match s {
            Stmt::Expr(e) => expr_looks_float(e),
            Stmt::Store(s) => expr_looks_float(&s.expr),
            _ => false,
        }),
        _ => false,
    }
}

/// JS/TS operator precedence for the binops the DSL emits (higher binds
/// tighter). The parser builds the `Expr::Bina` tree honoring explicit
/// parentheses but does NOT record them, so emitters must re-derive the
/// parentheses needed to preserve the tree's grouping — otherwise
/// `(a+b)*c` silently comes out as `a + b * c` (probe 09, plan 013).
pub(crate) fn binop_js_prec(op: &auto_val::Op) -> u8 {
    use auto_val::Op;
    match op {
        Op::Or => 1,
        Op::And => 2,
        Op::Eq | Op::Neq => 3,
        Op::Lt | Op::Gt | Op::Le | Op::Ge | Op::In => 4,
        Op::Add | Op::Sub => 5,
        Op::Mul | Op::Div | Op::Mod => 6,
        _ => 0, // statement-level ops (Asn, *Eq, Dot, ...) — no paren rule
    }
}

/// Whether a `Bina` child must be parenthesized under `parent_op` to keep
/// the AST grouping. Extra parens are semantically harmless; missing ones
/// silently change semantics, so this errs on the side of wrapping.
pub(crate) fn bina_child_needs_parens(parent_op: &auto_val::Op, child: &Expr, is_right: bool) -> bool {
    use auto_val::Op;
    if let Expr::Bina(_, child_op, _) = child {
        let (p, c) = (binop_js_prec(parent_op), binop_js_prec(child_op));
        if p == 0 || c == 0 {
            return false;
        }
        if c < p {
            return true; // lower-precedence child: `(a+b)*c`, `(x||y)&&z`
        }
        if is_right && c == p {
            // All DSL binops are left-associative, so an equal-precedence
            // RIGHT child regroups (`a-(b-c)` ≠ `a-b-c`). Safe only for
            // strictly associative same-op chains and `a+(b-c)`.
            let safe = matches!(
                (parent_op, child_op),
                (Op::Add, Op::Add)
                    | (Op::Add, Op::Sub)
                    | (Op::Mul, Op::Mul)
                    | (Op::And, Op::And)
                    | (Op::Or, Op::Or)
            );
            return !safe;
        }
    }
    false
}

/// Emit a Bina operand, re-inserting parentheses when the grouping
/// requires them (see `bina_child_needs_parens`).
fn transpile_bina_child(
    child: &Expr,
    parent_op: &auto_val::Op,
    is_right: bool,
    ctx: &AuraTsContext,
    out: &mut Vec<u8>,
) {
    if bina_child_needs_parens(parent_op, child, is_right) {
        write!(out, "(").ok();
        transpile_expr(child, ctx, out);
        write!(out, ")").ok();
    } else {
        transpile_expr(child, ctx, out);
    }
}

/// Emit a method-call / field-access receiver. A binop/unary receiver must
/// keep its parens — `(a + b).toLowerCase()`, not `a + b.toLowerCase()`
/// (plan 013 follow-up; the AST does not record explicit parens).
fn transpile_receiver(expr: &Expr, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    if matches!(expr, Expr::Bina(..) | Expr::Unary(..)) {
        write!(out, "(").ok();
        transpile_expr(expr, ctx, out);
        write!(out, ")").ok();
    } else {
        transpile_expr(expr, ctx, out);
    }
}

/// Convert a snake_case identifier to camelCase (for TS/JS output).
/// e.g. `list_notes` → `listNotes`, `create_note` → `createNote`
pub fn snake_to_camel(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Transpile a list of AutoLang statements to TypeScript, with AURA rewrites.
pub fn transpile_handler_body(stmts: &[Stmt], ctx: &AuraTsContext) -> String {
    let mut out = Vec::new();
    for stmt in stmts.iter() {
        transpile_stmt(stmt, ctx, &mut out);
        // Each statement already ends with a newline (via writeln!), but
        // ensure separation even if a statement's output doesn't end with \n.
        let s = std::str::from_utf8(&out).unwrap_or("");
        if !s.ends_with('\n') {
            writeln!(out).ok();
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// Render a statement list for use as the **body of an IIFE that must produce a
/// value** (Plan 043 H1). Used by `Expr::If` IIFE generation so that
/// `if (c) { '✓' } else { '…' }` evaluates to the glyph, not `undefined`.
///
/// All leading statements render normally (via `transpile_stmt`); the **last**
/// statement, if it is a bare expression statement (`Stmt::Expr(e)`), renders as
/// `return e;` so the IIFE returns it. Non-expression trailing statements
/// (e.g. a trailing `if`) are left as-is — the IIFE then has no explicit return
/// (undefined), matching a statement-expression block.
pub fn transpile_body_as_return(stmts: &[Stmt], ctx: &AuraTsContext) -> String {
    let mut out = Vec::new();
    let n = stmts.len();
    for (i, stmt) in stmts.iter().enumerate() {
        let is_last = i + 1 == n;
        if is_last {
            if let Stmt::Expr(expr) = stmt {
                // Plan 354: NavCall → router.push(path) (same special-case as transpile_stmt).
                write!(out, "return ").ok();
                if let Expr::NavCall { path, .. } = expr {
                    write!(out, "router.push(").ok();
                    transpile_expr(path, ctx, &mut out);
                    write!(out, ")").ok();
                } else {
                    transpile_expr(expr, ctx, &mut out);
                }
                writeln!(out, ";").ok();
                continue;
            }
        }
        transpile_stmt(stmt, ctx, &mut out);
        let s = std::str::from_utf8(&out).unwrap_or("");
        if !s.ends_with('\n') {
            writeln!(out).ok();
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Statement transpilation
// ---------------------------------------------------------------------------

fn transpile_stmt(stmt: &Stmt, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match stmt {
        // Variable declarations — AURA-aware value rewriting
        Stmt::Store(store) => {
            let kw = match store.kind {
                StoreKind::Let => "let",
                StoreKind::Var => "let",
                StoreKind::Const => "const",
                _ => "let", // Shared, CVar, Field — shouldn't appear in handlers
            };
            write!(out, "{} {}", kw, store.name.as_str()).ok();
            // Plan 043 store-codegen: emit a type annotation when the parser
            // recorded a TS-builtin scalar/array type, so `var result []str = []`
            // becomes `let result: string[] = []` (valid under noImplicitAny).
            // Skip user-defined types — the TS frontend erases them to `any`,
            // so annotating with e.g. `Block` would be a `Cannot find name`.
            if let Some(ty_str) = builtin_type_annotation(&store.ty) {
                write!(out, ": {}", ty_str).ok();
            }
            write!(out, " = ").ok();
            transpile_expr(&store.expr, ctx, out);
            writeln!(out, ";").ok();
        }

        // Expression statements — AURA-aware (API calls, print, etc.)
        Stmt::Expr(expr) => {
            // Plan 053 M5/P5-6: inside a debounced complete-handler body,
            // guard state-ref assignments so a stale (older, slower) complete
            // result can't overwrite the suggestions produced by the newest
            // request. Only plain `.state = rhs` self-assignments are guarded;
            // local `let` declarations (Stmt::Store) and non-state writes pass
            // through unchanged.
            if let Some(seq_var) = &ctx.debounce_seq_var {
                if let Expr::Bina(lhs, auto_val::Op::Asn, _) = expr {
                    if assign_target_is_state_ref(lhs, ctx) {
                        write!(out, "if ({} === __completeSeq) {{ ", seq_var).ok();
                        transpile_expr(expr, ctx, out);
                        writeln!(out, "; }}").ok();
                        return;
                    }
                }
            }
            // Plan 354: NavCall in handler body → router.push(path)
            if let Expr::NavCall { path, .. } = expr {
                write!(out, "router.push(").ok();
                transpile_expr(path, ctx, out);
                write!(out, ")").ok();
            } else {
                transpile_expr(expr, ctx, out);
            }
            writeln!(out, ";").ok();
        }

        // If/else if/else — write scaffolding, AURA-rewrite all expressions
        Stmt::If(if_stmt) => {
            if let Some(first) = if_stmt.branches.first() {
                write!(out, "if (").ok();
                transpile_expr(&first.cond, ctx, out);
                write!(out, ") {{").ok();
                transpile_body(&first.body, ctx, out);
                write!(out, "}}").ok();
            }
            for branch in if_stmt.branches.iter().skip(1) {
                write!(out, " else if (").ok();
                transpile_expr(&branch.cond, ctx, out);
                write!(out, ") {{").ok();
                transpile_body(&branch.body, ctx, out);
                write!(out, "}}").ok();
            }
            if let Some(else_body) = &if_stmt.else_ {
                write!(out, " else {{").ok();
                transpile_body(else_body, ctx, out);
                write!(out, "}}").ok();
            }
        }

        // For loops
        Stmt::For(for_loop) => {
            transpile_for(for_loop, ctx, out);
        }

        // Return
        Stmt::Return(expr) => {
            // If returning nil/null in a void function, emit bare return
            match expr.as_ref() {
                Expr::Nil | Expr::Null => {
                    writeln!(out, "return;").ok();
                }
                _ => {
                    write!(out, "return ").ok();
                    transpile_expr(expr, ctx, out);
                    writeln!(out, ";").ok();
                }
            }
        }

        // Break
        Stmt::Break => {
            writeln!(out, "break;").ok();
        }

        // Plan 012 P2 (gap 4): try/catch/finally → JS try/catch/finally.
        // Bodies stay AURA-aware (state refs → .value, API calls → await) via
        // transpile_body. `catch (e)` binding is optional in JS (ES2019), so
        // a bare `catch {` is emitted when there is no param. Until this arm
        // existed, Stmt::Try fell into the a2ts fallback which had no Try
        // case and SILENTLY DROPPED the whole statement.
        Stmt::Try(t) => {
            write!(out, "try {{").ok();
            transpile_body(&t.body, ctx, out);
            write!(out, "}}").ok();
            match &t.catch_param {
                Some(p) => write!(out, " catch ({}) {{", p).ok(),
                None => write!(out, " catch {{").ok(),
            };
            transpile_body(&t.catch_body, ctx, out);
            write!(out, "}}").ok();
            if let Some(finally_body) = &t.finally_body {
                write!(out, " finally {{").ok();
                transpile_body(finally_body, ctx, out);
                write!(out, "}}").ok();
            }
            writeln!(out).ok();
        }

        // Fallback — delegate to a2ts for anything else
        _ => {
            let mut ts = TypeScriptTrans::new("fragment".into());
            let mut sink = Sink::new("fragment".into());
            let _ = ts.stmt(stmt, &mut sink);
            let _ = out.write_all(&sink.body);
        }
    }
}

fn transpile_body(body: &Body, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    for stmt in &body.stmts {
        transpile_stmt(stmt, ctx, out);
    }
}

fn transpile_for(for_loop: &For, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match &for_loop.iter {
        Iter::Cond => {
            // for condition { ... } → while (condition) { ... }
            write!(out, "while (").ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Ever => {
            // for ever { ... } → while (true) { ... }
            writeln!(out, "while (true) {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Named(iter_name) => {
            // for item in range { ... } → for (const item of range) { ... }
            write!(out, "for (const {} of ", iter_name.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Indexed(index_name, iter_name) => {
            // for i, item in range { ... } → range.forEach((item, i) => { ... })
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ".forEach(({}, {}) => {{", iter_name.as_str(), index_name.as_str()).ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}});").ok();
        }
        Iter::Call(call) => {
            // for func(args) { ... } → while (func(args)) { ... }
            write!(out, "while (").ok();
            match call.name.as_ref() {
                Expr::Dot(object, method) => {
                    transpile_receiver(object, ctx, out);
                    write!(out, ".{}(", method.as_str()).ok();
                }
                Expr::Ident(name) => {
                    write!(out, "{}(", name.as_str()).ok();
                }
                _ => {
                    write!(out, "/* complex call */(").ok();
                }
            }
            for (i, arg) in call.args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Destructured(key, val) => {
            // for (k, v) in map -> for (const [k, v] of Object.entries(map))
            write!(out, "for (const [{}, {}] of Object.entries(", key.as_str(), val.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ")) {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
    }
}

// ---------------------------------------------------------------------------
// Expression transpilation — AURA-aware with a2ts delegation
// ---------------------------------------------------------------------------

/// Public wrapper for transpile_expr (Plan 367 P2-2: store computed needs it).
pub fn transpile_expr_pub(expr: &Expr, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    transpile_expr(expr, ctx, out);
}

fn transpile_expr(expr: &Expr, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match expr {
        // === AURA-specific rewrites ===

        // StateRef: `.count` is parsed as Expr::Dot(Ident("self"), "count")
        // → `count.value` for Vue reactive refs
        // General field access: object.field → transpile object, then emit .field
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" || name.as_str() == "." {
                    let field_name = field.as_str();
                    if ctx.is_prop(field_name) {
                        // Props need `props.` prefix in script
                        write!(out, "props.{}", field_name).ok();
                    } else if ctx.is_state(field_name) || ctx.is_computed(field_name) {
                        // State ref OR computed ref → unwrap `.value` (Vue).
                        write!(out, "{}.value", field_name).ok();
                    } else if ctx.is_ref(field_name) {
                        // Template ref: `.menuEl` → `menuEl.value!` (the DOM
                        // element). Chained access (`.menuEl.scrollTop`,
                        // `.menuEl.getBoundingClientRect()`) flows through the
                        // general Dot/Call paths and lands on this element.
                        write!(out, "{}.value!", field_name).ok();
                    } else {
                        write!(out, "{}", field_name).ok();
                    }
                    return;
                }
            }
            // General field access: object.field
            if try_transpile_builtin_field(obj, field.as_str(), ctx, out) {
                return;
            }
            // Plan 408 P12 §10.4: composable facade ref 字段——当 object 解析为
            // facade local（裸 Ident 或 self.local）且 field 在 ref_fields 标注里
            // 时，注入 `.value`（composable 返回普通对象时 ref 不自动 unwrap）。
            let facade_local = match obj.as_ref() {
                Expr::Ident(local) => Some(local.as_str().to_string()),
                Expr::Dot(inner, local) if matches!(inner.as_ref(), Expr::Ident(n) if n.as_str() == "self" || n.as_str() == ".") => {
                    Some(local.as_str().to_string())
                }
                _ => None,
            };
            if let Some(local) = facade_local {
                if ctx.is_facade(&local)
                    && ctx.facade_ref_fields.get(&local)
                        .map(|fields| fields.contains(field.as_str()))
                        .unwrap_or(false)
                {
                    transpile_receiver(obj, ctx, out);
                    write!(out, ".{}.value", field.as_str()).ok();
                    return;
                }
            }
            transpile_receiver(obj, ctx, out);
            write!(out, ".{}", field.as_str()).ok();
        }

        // Identifier — check if it's a reactive state variable
        Expr::Ident(name) => {
            if ctx.is_state(name.as_str()) || ctx.is_computed(name.as_str()) {
                write!(out, "{}.value", name.as_str()).ok();
            } else if name.as_str() == "self" || name.as_str() == "." {
                // In Vue <script setup>, self/this is not needed
                // Skip output — the field access will be handled by Expr::Dot
            } else {
                write!(out, "{}", name.as_str()).ok();
            }
        }

        // String literal — escape newlines and quotes for JS single-quoted strings
        Expr::Str(s) => {
            let escaped = s
                .replace("\\", "\\\\")
                .replace("'", "\\'")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
            write!(out, "'{}'", escaped).ok();
        }

        // Function call — API detection, print, builtins, method calls
        Expr::Call(call) => {
            // Plan 412 续(toast 参数化):toast()/toast.success() 的 named 参数
            // 转 vue-sonner options 对象 —— toast('msg', position: 'top-left',
            // duration: 2000) → toast('msg', { position: 'top-left',
            // duration: 2000 })。通用路径用 get_expr() 展平参数会丢 Pair 的
            // key,故在方法调用分发前特判(与 vue.rs expr_to_js 的特判一致)。
            let is_toast_call = match call.name.as_ref() {
                Expr::Ident(n) => n.as_str() == "toast",
                Expr::Dot(obj, _) => matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "toast"),
                _ => false,
            };
            if is_toast_call {
                transpile_expr(&call.name, ctx, out);
                write!(out, "(").ok();
                let mut wrote_pos = false;
                let mut fields: Vec<u8> = Vec::new();
                for arg in &call.args.args {
                    match arg {
                        crate::ast::Arg::Pos(e) => {
                            if wrote_pos {
                                write!(out, ", ").ok();
                            }
                            transpile_expr(e, ctx, out);
                            wrote_pos = true;
                        }
                        crate::ast::Arg::Pair(k, v) => {
                            if !fields.is_empty() {
                                write!(&mut fields, ", ").ok();
                            }
                            write!(&mut fields, "{}: ", k.as_str()).ok();
                            transpile_expr(v, ctx, &mut fields);
                        }
                        _ => {}
                    }
                }
                if !fields.is_empty() {
                    if wrote_pos {
                        write!(out, ", ").ok();
                    }
                    write!(out, "{{ ").ok();
                    out.extend_from_slice(&fields);
                    write!(out, " }}").ok();
                }
                write!(out, ")").ok();
                return;
            }
            match call.name.as_ref() {
                // Method call: object.method(args)
                Expr::Dot(object, method) => {
                    // Plan 043 store-codegen: container constructor
                    // `List<T>.new([...])` / `Array<T>.new(...)` → `[...]` (or `[]`).
                    // Without this the call falls through to the generic
                    // fallback and emits `List.new([])` (undefined `List` in TS).
                    if method.as_str() == "new" {
                        let base = match object.as_ref() {
                            Expr::GenName(g) => g.as_str().split('<').next().unwrap_or(g.as_str()),
                            Expr::Ident(n) => n.as_str(),
                            _ => "",
                        };
                        if matches!(base, "List" | "Array" | "Slice" | "Map") {
                            if base == "Map" {
                                // Map has no literal; emit {} (empty object).
                                write!(out, "{{}}").ok();
                            } else if let Some(first) = call.args.args.first() {
                                transpile_expr(&first.get_expr(), ctx, out);
                            } else {
                                write!(out, "[]").ok();
                            }
                            return;
                        }
                    }
                    // D3 fix: self-method call (.MethodName()) — when object is
                    // "." or "self", this is a store sibling action call.
                    // Generate as bare MethodName() instead of .MethodName().
                    let is_self = matches!(object.as_ref(), Expr::Ident(name) if name.as_str() == "." || name.as_str() == "self");
                    if is_self {
                        // Check if it's a known builtin first
                        if try_transpile_builtin_call(object, method.as_str(), &call.args, ctx, out) {
                            return;
                        }
                        // Generate as bare function call (store sibling action)
                        write!(out, "{}(", method.as_str()).ok();
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ").ok();
                            }
                            transpile_expr(&arg.get_expr(), ctx, out);
                        }
                        write!(out, ")").ok();
                        return;
                    }
                    if try_transpile_builtin_call(object, method.as_str(), &call.args, ctx, out) {
                        return;
                    }
                    // Plan 028 F4: Regex 子集 —— 与 vue.rs expr_to_js 同规则
                    // （命名组 (?P<n>)→(?<n>)，flags 走参数；match 补 || []）。
                    if matches!(object.as_ref(), Expr::Ident(n) if n.as_str() == "Regex") {
                        let pat = call.args.args.get(1).and_then(|a| match a.get_expr() {
                            Expr::Str(s) => Some(s.as_str().to_string()),
                            _ => None,
                        });
                        if let Some(pat) = pat {
                            // flags 位置：test/match/split 第 3 参；replace 第 4 参
                            let flag_idx = if method.as_str() == "replace" { 3 } else { 2 };
                            let flags = call.args.args.get(flag_idx).and_then(|a| match a.get_expr() {
                                Expr::Str(s) => Some(s.as_str().to_string()),
                                _ => None,
                            });
                            // Plan 028 T11 对拍缺陷修复：模式嵌入 TS 字符串前必须
                            // 转义反斜杠（`\s` 直插 `'\s'` 会被 JS 求值成 `s`）。
                            let js_pat = pat
                                .replace("(?P<", "(?<")
                                .replace('\\', "\\\\")
                                .replace('\'', "\\'");
                            let re = match flags {
                                Some(f) => format!("new RegExp('{}', '{}')", js_pat, f),
                                None => format!("new RegExp('{}')", js_pat),
                            };
                            let subject: Option<Expr> = call.args.args.first().map(|a| a.get_expr());
                            let emit_subject = |out: &mut Vec<u8>| {
                                if let Some(sub) = subject.as_ref() {
                                    transpile_expr(sub, ctx, out);
                                } else {
                                    write!(out, "''").ok();
                                }
                            };
                            match method.as_str() {
                                "split" => {
                                    write!(out, "(").ok();
                                    emit_subject(out);
                                    write!(out, ").split({})", re).ok();
                                    return;
                                }
                                "match" => {
                                    write!(out, "((").ok();
                                    emit_subject(out);
                                    write!(out, ").match({}) || [])", re).ok();
                                    return;
                                }
                                "test" => {
                                    write!(out, "{}.test(", re).ok();
                                    emit_subject(out);
                                    write!(out, ")").ok();
                                    return;
                                }
                                "replace" => {
                                    write!(out, "(").ok();
                                    emit_subject(out);
                                    write!(out, ").replace({}, ", re).ok();
                                    if let Some(to) = call.args.args.get(2) {
                                        transpile_expr(&to.get_expr().clone(), ctx, out);
                                    } else {
                                        write!(out, "''").ok();
                                    }
                                    write!(out, ")").ok();
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }
                    // Plan 028 T14（F8 动态流生命周期）：`Sse.open(url, .Handler)`
                    // → 建 EventSource 并把每条（预解析）事件分发到同名 store
                    // action —— .at 侧零回调（已决③：平台层解析 JSON）。
                    // `Sse.close(.handle)` → 幂等关闭 + 置 None。
                    if matches!(object.as_ref(), Expr::Ident(n) if n.as_str() == "Sse") {
                        if method.as_str() == "open" {
                            let url_arg = call.args.args.first().map(|a| a.get_expr());
                            // 第二参数形态：.Handler（Dot(Ident("."), name)）
                            let handler_name = call.args.args.get(1).and_then(|a| match a.get_expr() {
                                Expr::Dot(obj, field) => match obj.as_ref() {
                                    Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self" => {
                                        Some(field.as_str().to_string())
                                    }
                                    _ => None,
                                },
                                Expr::Ident(n) if n.as_str().starts_with('.') => {
                                    Some(n.as_str().trim_start_matches('.').to_string())
                                }
                                _ => None,
                            });
                            if let (Some(url_arg), Some(handler)) = (url_arg, handler_name) {
                                // Plan 028 T16：可选第三参 ctx —— 注入为分发事件的
                                // `__ctx` 字段（订阅方据此定位 per-run 状态键）。
                                // onerror 分发合成 {type:'error', __ctx} 事件，让
                                // .at 侧自行决定重连/收尾（close 句柄可及）。
                                let ctx_arg = call.args.args.get(2).map(|a| a.get_expr());
                                write!(out, "(() => {{ const __es = new EventSource(").ok();
                                transpile_expr(&url_arg, ctx, out);
                                let ctx_lit = if let Some(c) = &ctx_arg {
                                    let mut buf = Vec::new();
                                    transpile_expr(c, ctx, &mut buf);
                                    String::from_utf8_lossy(&buf).to_string()
                                } else {
                                    "undefined".to_string()
                                };
                                write!(out, "); __es.onmessage = (__ev) => {{ try {{ {}(Object.assign({{ __ctx: {} }}, JSON.parse(__ev.data))); }} catch {{ }} }}; __es.onerror = () => {{ try {{ {}({{ type: 'error', __ctx: {} }}); }} catch {{ }} }}; return __es; }})()", handler, ctx_lit, handler, ctx_lit).ok();
                                return;
                            }
                        }
                        if method.as_str() == "close" {
                            let handle_arg = call.args.args.first().map(|a| a.get_expr());
                            if let Some(handle_arg) = handle_arg {
                                write!(out, "(() => {{ ").ok();
                                transpile_expr(&handle_arg, ctx, out);
                                write!(out, "?.close(); return null; }})()").ok();
                                return;
                            }
                        }
                    }
                    // Plan 028 F3: Date.format(ts, "HH:mm") → toLocaleTimeString
                    // （窄面日期 API，与 vue.rs expr_to_js 同规则）。
                    if method.as_str() == "format"
                        && matches!(object.as_ref(), Expr::Ident(n) if n.as_str() == "Date")
                    {
                        write!(out, "(new Date(").ok();
                        if let Some(first) = call.args.args.first() {
                            transpile_expr(&first.get_expr().clone(), ctx, out);
                        } else {
                            write!(out, "0").ok();
                        }
                        let pattern = call.args.args.get(1).and_then(|a| match a.get_expr() {
                            Expr::Str(s) => Some(s.as_str().to_string()),
                            _ => None,
                        }).unwrap_or_else(|| "HH:mm".to_string());
                        let mut opts = vec!["hour: '2-digit'", "minute: '2-digit'"];
                        if pattern.contains("ss") {
                            opts.push("second: '2-digit'");
                        }
                        write!(out, ").toLocaleTimeString([], {{ {} }}))", opts.join(", ")).ok();
                        return;
                    }
                    // Plan 028 F8: platform HTTP protocol — `Http.get(url)` /
                    // `Http.post(url, body)` map to awaited fetch + .json().
                    // The emitted `await` makes the enclosing action fn async
                    // (store codegen detects it in the transpiled body).
                    if matches!(object.as_ref(), Expr::Ident(n) if n.as_str() == "Http") {
                        let pos_args: Vec<crate::ast::Expr> = call.args.args.iter()
                            .map(|a| a.get_expr().clone())
                            .collect();
                        match method.as_str() {
                            "get" if pos_args.len() == 1 => {
                                write!(out, "(await (await fetch(").ok();
                                transpile_expr(&pos_args[0], ctx, out);
                                write!(out, ")).json())").ok();
                                return;
                            }
                            "post" if pos_args.len() == 2 => {
                                write!(out, "(await (await fetch(").ok();
                                transpile_expr(&pos_args[0], ctx, out);
                                write!(out, ", {{ method: 'POST', headers: {{ 'Content-Type': 'application/json' }}, body: JSON.stringify(").ok();
                                transpile_expr(&pos_args[1], ctx, out);
                                write!(out, ") }})).json())").ok();
                                return;
                            }
                            // Plan 028 T16: 无体动词（delete）与带体动词（patch/put）
                            "delete" if pos_args.len() == 1 => {
                                write!(out, "(await (await fetch(").ok();
                                transpile_expr(&pos_args[0], ctx, out);
                                write!(out, ", {{ method: 'DELETE' }})).json())").ok();
                                return;
                            }
                            "patch" | "put" if pos_args.len() == 2 => {
                                let verb = method.as_str().to_ascii_uppercase();
                                write!(out, "(await (await fetch(").ok();
                                transpile_expr(&pos_args[0], ctx, out);
                                write!(out, ", {{ method: '{}', headers: {{ 'Content-Type': 'application/json' }}, body: JSON.stringify(", verb).ok();
                                transpile_expr(&pos_args[1], ctx, out);
                                write!(out, ") }})).json())").ok();
                                return;
                            }
                            _ => {}
                        }
                    }
                    // Plan 012 Batch A (gap 19): the `.remove → .splice` /
                    // `.contains → .includes` mappings apply ONLY to receivers
                    // proven to be arrays (strings too, for `.contains`).
                    // Facades/store chains pass through unchanged.
                    if matches!(method.as_str(), "remove" | "contains") {
                        match method_map_decision(method.as_str(), object, ctx) {
                            MethodMapDecision::Map => {}
                            decision @ (MethodMapDecision::Pass | MethodMapDecision::PassWarn) => {
                                if matches!(decision, MethodMapDecision::PassWarn) {
                                    ctx.note_warning(format!(
                                        "`.{method}()` on `{}` passed through unchanged: the receiver is not a proven array, so the old `.{}` mapping no longer applies. If this IS an array, declare it with an array type; if it's a facade/ext object, its own `.{method}` method is now called as intended.",
                                        expr_brief(object, ctx),
                                        if method.as_str() == "remove" { "splice" } else { "includes" },
                                        method = method.as_str(),
                                    ));
                                }
                                transpile_receiver(object, ctx, out);
                                write!(out, ".{}(", method.as_str()).ok();
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ").ok();
                                    }
                                    transpile_expr(&arg.get_expr(), ctx, out);
                                }
                                write!(out, ")").ok();
                                return;
                            }
                        }
                    }
                    // Handle common method call conversions
                    match method.as_str() {
                        "to_int" | "parse_int" => {
                            write!(out, "parseInt(").ok();
                            transpile_receiver(object, ctx, out);
                            write!(out, ")").ok();
                            return;
                        }
                        "to_float" | "to_double" | "parse_float" => {
                            write!(out, "parseFloat(").ok();
                            transpile_receiver(object, ctx, out);
                            write!(out, ")").ok();
                            return;
                        }
                        "to_string" | "str" => {
                            write!(out, "(").ok();
                            transpile_receiver(object, ctx, out);
                            write!(out, ").toString()").ok();
                            return;
                        }
                        "len" => {
                            transpile_receiver(object, ctx, out);
                            write!(out, ".length").ok();
                            return;
                        }
                        // Plan 345 (gap N1): Auto `.contains` -> JS `.includes`
                        // (JS strings and arrays both use .includes, not .contains).
                        "contains" => {
                            transpile_receiver(object, ctx, out);
                            write!(out, ".includes(").ok();
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ").ok();
                                }
                                transpile_expr(&arg.get_expr(), ctx, out);
                            }
                            write!(out, ")").ok();
                            return;
                        }
                        "remove" => {
                            // AutoLang notes.remove(idx) → TypeScript notes.value.splice(idx, 1)
                            transpile_receiver(object, ctx, out);
                            write!(out, ".splice(").ok();
                            if let Some(first_arg) = call.args.args.first() {
                                transpile_expr(&first_arg.get_expr(), ctx, out);
                            } else {
                                write!(out, "0").ok();
                            }
                            write!(out, ", 1)").ok();
                            return;
                        }
                        // Plan 053 M1: 字符串方法映射补全（原先仅
                        // len/contains/to_string/to_int/to_float，其余走兜底
                        // `.{method}()` 原样输出，生成无效 JS）。语义对照
                        // libs/string.rs。
                        // 无参 — 大小写 + trim 方向：
                        "to_lower" | "lower" | "to_upper" | "upper" | "trim_left" | "trim_right" => {
                            let js_method = match method.as_str() {
                                "to_lower" | "lower" => "toLowerCase",
                                "to_upper" | "upper" => "toUpperCase",
                                "trim_left" => "trimStart",
                                "trim_right" => "trimEnd",
                                _ => unreachable!(),
                            };
                            transpile_receiver(object, ctx, out);
                            write!(out, ".{}()", js_method).ok();
                            return;
                        }
                        // 数组 find(lambda) 守卫：第一个实参是闭包/lambda 时
                        // 说明 receiver 是数组（Array.find 谓词语义，JS 同名），
                        // 不映射成字符串 str::find 的 indexOf —— 空 arm 落到
                        // match 之后的 pass-through，原样输出 `.find(λ)`。
                        "find" if call.args.args.first().map_or(false, |a| {
                            matches!(a.get_expr(), Expr::Closure(_) | Expr::Lambda(_))
                        }) => {}
                        // 有参 — 前后缀 / 字符 / 查找 / 子串 / 替换 / 重复：
                        // Plan 028 M1: 带闭包参数时不走字符串映射表 ——
                        // `.find(e => …)` 是 Array.find，不是 indexOf。
                        "starts_with" | "ends_with" | "char_at" | "char_code_at" | "find"
                        | "substr" | "sub" | "slice" | "replace" | "repeat"
                            if !call.args.args.iter().any(|a| matches!(a.get_expr(), Expr::Closure(_))) => {
                            let js_method = match method.as_str() {
                                "starts_with" => "startsWith",
                                "ends_with" => "endsWith",
                                // char_at 返回 1 字符 string（.at 按 Unicode char
                                // 索引；JS charAt 按 UTF-16 code unit）
                                "char_at" => "charAt",
                                // Plan 028 F3: char_code_at → charCodeAt
                                "char_code_at" => "charCodeAt",
                                // find 返回 index 或 -1
                                "find" => "indexOf",
                                // substr/sub/slice 同一 native，start..end 子串
                                "substr" | "sub" | "slice" => "substring",
                                // Rust str::replace 替换所有 → replaceAll
                                "replace" => "replaceAll",
                                "repeat" => "repeat",
                                _ => unreachable!(),
                            };
                            transpile_receiver(object, ctx, out);
                            write!(out, ".{}(", js_method).ok();
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ").ok();
                                }
                                transpile_expr(&arg.get_expr(), ctx, out);
                            }
                            write!(out, ")").ok();
                            return;
                        }
                        "is_empty" => {
                            transpile_receiver(object, ctx, out);
                            write!(out, ".length === 0").ok();
                            return;
                        }
                        // JS 无原生字符串 reverse
                        "reverse" => {
                            write!(out, "[...").ok();
                            transpile_receiver(object, ctx, out);
                            write!(out, "].reverse().join('')").ok();
                            return;
                        }
                        _ => {}
                    }
                    transpile_receiver(object, ctx, out);
                    write!(out, ".{}", method.as_str()).ok();
                    write!(out, "(").ok();
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, ")").ok();
                }
                // Regular function call
                Expr::Ident(name) => {
                    let func_name = name.as_str();
                    // API calls need `await`
                    if ctx.is_api(func_name) {
                        write!(out, "await {}", func_name).ok();
                    } else if func_name == "print" {
                        write!(out, "console.log").ok();
                    } else if ctx.is_prop(func_name) {
                        // Plan 345 (gap K2/N4): callback prop call -> props.<name>(...)
                        write!(out, "props.{}", func_name).ok();
                    } else {
                        write!(out, "{}", func_name).ok();
                    }

                    write!(out, "(").ok();
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, ")").ok();
                }
                // Fallback — delegate to a2ts for complex call names
                _ => delegate_expr(expr, ctx, out),
            }
        }

        // Plan 408 P12 §10.7: Await expression — `expr.await`.
        // Transpile the inner expression (recursing so `.path` → `props.path`
        // and state refs get `.value`), then wrap as `(await <expr>)`.
        Expr::Await { expr } => {
            write!(out, "(await ").ok();
            transpile_expr(expr, ctx, out);
            write!(out, ")").ok();
        }

        // Binary ops — AURA-aware on both sides
        Expr::Bina(lhs, op, rhs) => {
            // Handle assignment operators specially (target needs StateRef check)
            use auto_val::Op;
            match op {
                Op::Asn => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " = ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::AddEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " += ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::SubEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " -= ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::MulEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " *= ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::DivEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " /= ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::Dot => {
                    // Field access: already handled by Expr::Dot arm above,
                    // but Op::Dot can also appear in Bina. Delegate.
                    delegate_expr(expr, ctx, out);
                }
                Op::Add => {
                    // D2 fix: array concat. If RHS is an array literal,
                    // use spread syntax to avoid JS string coercion.
                    if matches!(rhs.as_ref(), Expr::Array(_)) {
                        transpile_expr(lhs, ctx, out);
                        write!(out, ".concat(").ok();
                        transpile_expr(rhs, ctx, out);
                        write!(out, ")").ok();
                    } else {
                        transpile_bina_child(lhs, op, false, ctx, out);
                        write!(out, " + ").ok();
                        transpile_bina_child(rhs, op, true, ctx, out);
                    }
                }
                Op::Div | Op::Mod => {
                    // AutoLang `/` and `%` on two ints are integer ops (matches
                    // VM DIV/MOD opcodes = wrapping_div/wrapping_rem). JS `/`
                    // and `%` are float, so when neither operand looks float,
                    // wrap in Math.trunc to get integer semantics. Float
                    // operands fall through to native JS behavior.
                    let is_int = !expr_looks_float(lhs) && !expr_looks_float(rhs);
                    let js_op = if matches!(op, Op::Mod) { " %" } else { " / " };
                    if is_int {
                        write!(out, "Math.trunc(").ok();
                        transpile_bina_child(lhs, op, false, ctx, out);
                        write!(out, "{}", js_op).ok();
                        transpile_bina_child(rhs, op, true, ctx, out);
                        write!(out, ")").ok();
                    } else {
                        transpile_bina_child(lhs, op, false, ctx, out);
                        write!(out, "{}", js_op).ok();
                        transpile_bina_child(rhs, op, true, ctx, out);
                    }
                }
                _ => {
                    // Standard binary op
                    transpile_bina_child(lhs, op, false, ctx, out);
                    write!(out, " {} ", op.op()).ok();
                    transpile_bina_child(rhs, op, true, ctx, out);
                }
            }
        }

        // Unary ops — AURA-aware on operand
        Expr::Unary(op, operand) => {
            let op_str = match op {
                auto_val::Op::Sub => "-",
                auto_val::Op::Not => "!",
                _ => "",
            };
            write!(out, "{}", op_str).ok();
            // `!` / unary `-` bind tighter than any binop — a Bina operand
            // must keep its parens (`!(a && b)`, `-(a + b)`).
            if matches!(operand.as_ref(), Expr::Bina(..) | Expr::NullCoalesce(..)) {
                write!(out, "(").ok();
                transpile_expr(operand, ctx, out);
                write!(out, ")").ok();
            } else {
                transpile_expr(operand, ctx, out);
            }
        }

        // Array literals
        Expr::Array(elems) => {
            write!(out, "[").ok();
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(elem, ctx, out);
            }
            write!(out, "]").ok();
        }

        // Object literals — recurse values through transpile_expr so builtins work
        Expr::Object(pairs) => {
            write!(out, "{{ ").ok();
            for (i, pair) in pairs.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                match &pair.key {
                    Key::NamedKey(name) => {
                        write!(out, "{}: ", name.as_str()).ok();
                    }
                    Key::IntKey(n) => {
                        write!(out, "{}: ", n).ok();
                    }
                    Key::BoolKey(b) => {
                        write!(out, "{}: ", b).ok();
                    }
                    Key::StrKey(s) => {
                        write!(out, "\"{}\": ", s).ok();
                    }
                }
                transpile_expr(&pair.value, ctx, out);
            }
            write!(out, " }}").ok();
        }

        // Null-coalescing operator ??
        Expr::NullCoalesce(lhs, rhs) => {
            // Plan 028 T9：外层括号必需 —— JS 里 `+` 优先级高于 `??`，
            // `'x' + a ?? b` 会先算加法（语义错误 + TS2869）。
            write!(out, "(").ok();
            transpile_expr(lhs, ctx, out);
            write!(out, " ?? ").ok();
            transpile_expr(rhs, ctx, out);
            write!(out, ")").ok();
        }

        // Error propagation .?
        Expr::ErrorPropagate(expr) => {
            transpile_expr(expr, ctx, out);
            write!(out, "?.").ok();
        }

        // If expression (appears when parser treats if as RHS of let)
        Expr::If(if_expr) => {
            // Convert to IIFE so it works in expression position.
            // Plan 043 H1: branch bodies must RETURN their value, else the IIFE
            // evaluates to undefined (e.g. status_glyph computed vanished).
            write!(out, "(() => {{ ").ok();
            if let Some(first) = if_expr.branches.first() {
                write!(out, "if (").ok();
                transpile_expr(&first.cond, ctx, out);
                write!(out, ") {{ ").ok();
                write!(out, "{}", transpile_body_as_return(&first.body.stmts, ctx).trim()).ok();
                write!(out, " }}").ok();
            }
            if let Some(else_body) = &if_expr.else_ {
                write!(out, " else {{ ").ok();
                write!(out, "{}", transpile_body_as_return(&else_body.stmts, ctx).trim()).ok();
                write!(out, " }}").ok();
            }
            write!(out, " }})()").ok();
        }

        // Array index access: arr[idx] → arr.value[idx.value] for Vue refs
        Expr::Index(array, index) => {
            transpile_expr(array, ctx, out);
            write!(out, "[").ok();
            transpile_expr(index, ctx, out);
            write!(out, "]").ok();
        }

        // Closure: x => expr or (a, b) => expr
        // Must use transpile_expr (not delegate) so StateRef gets .value inside closures
        // Plan 367 P2-2: add ': any' type annotations for TS strict mode.
        Expr::Closure(closure) => {
            if closure.params.len() == 1 {
                write!(out, "({}: any)", closure.params[0].name).ok();
            } else {
                write!(out, "(").ok();
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ").ok();
                    }
                    write!(out, "{}: any", param.name).ok();
                }
                write!(out, ")").ok();
            }
            write!(out, " => ").ok();
            transpile_expr(&closure.body, ctx, out);
        }

        // Block in expression position — primarily closure bodies
        // (`() => { stmts }`). Must stay Aura-aware: delegate_expr would route
        // to a2ts, which knows nothing about StateRef and would emit
        // `state = x` instead of `state.value = x`.
        Expr::Block(block) => {
            write!(out, "{{ ").ok();
            for stmt in &block.stmts {
                transpile_stmt(stmt, ctx, out);
            }
            write!(out, " }}").ok();
        }

        // Plan 043 store-codegen: struct-literal construction
        // `Type{ field: value, ... }` parses as Expr::Node. The a2ts delegate
        // emits `new Type(...)` (TS has no such types → "Cannot find name"),
        // so intercept here and emit an object literal `{ field: value }`.
        // `loop` nodes keep their special delegate behavior (`while(true)`).
        // Handler bodies only ever construct data structs (Block, BlockStatus,
        // ...), not widget view nodes — view nodes live in the template, not
        // in handler logic.
        Expr::Node(node) => {
            if node.name == "loop" {
                delegate_expr(expr, ctx, out);
            } else {
                write!(out, "{{ ").ok();
                for (i, arg) in node.args.args.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ").ok();
                    }
                    match arg {
                        Arg::Pair(key, val) => {
                            write!(out, "{}: ", key.as_str()).ok();
                            transpile_expr(val, ctx, out);
                        }
                        Arg::Pos(val) => transpile_expr(val, ctx, out),
                        Arg::Name(key) => {
                            write!(out, "{}: {}", key.as_str(), key.as_str()).ok();
                        }
                    }
                }
                write!(out, " }}").ok();
            }
        }

        // === Delegate to a2ts for everything else ===
        _ => delegate_expr(expr, ctx, out),
    }
}

/// Extract assignment target with StateRef rewriting.
fn transpile_assign_target(expr: &Expr, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match expr {
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" {
                    let field_name = field.as_str();
                    if ctx.is_state(field_name) {
                        write!(out, "{}.value", field_name).ok();
                    } else if ctx.is_ref(field_name) {
                        // `.menuEl = el` assigns the element behind the ref
                        // (rare; `.menuEl.prop = v` takes the general path).
                        write!(out, "{}.value", field_name).ok();
                    } else {
                        write!(out, "{}", field_name).ok();
                    }
                    return;
                }
            }
            // Handle nested state ref: notes[.active_id].body → notes.value[active_id.value].body
            transpile_expr(obj, ctx, out);
            write!(out, ".{}", field.as_str()).ok();
        }
        Expr::Index(arr, idx) => {
            transpile_expr(arr, ctx, out);
            write!(out, "[").ok();
            transpile_expr(idx, ctx, out);
            write!(out, "]").ok();
        }
        Expr::Ident(name) => {
            if ctx.is_state(name.as_str()) {
                write!(out, "{}.value", name.as_str()).ok();
            } else {
                write!(out, "{}", name.as_str()).ok();
            }
        }
        _ => delegate_expr(expr, ctx, out),
    }
}

/// Plan 053 M5/P5-6: is this expression a self state-ref assignment target
/// (`.field` / `self.field` where `field` is a reactive state var)? Used to
/// decide whether to wrap the assignment in a debounce seq-guard. Mirrors the
/// self-receiver check in `transpile_assign_target` (`.field` → `field.value`).
fn assign_target_is_state_ref(expr: &Expr, ctx: &AuraTsContext) -> bool {
    if let Expr::Dot(obj, field) = expr {
        if let Expr::Ident(name) = obj.as_ref() {
            let obj_str = name.as_str();
            // `.field` (placeholder self ".") and `self.field` both denote a
            // self state write; match the self-receiver recognition used by
            // collect_self_handler_calls (P5-7).
            if obj_str == "self" || obj_str == "." {
                return ctx.is_state(field.as_str());
            }
        }
    }
    false
}

/// Delegate expression to a2ts transpiler for standard transpilation.
/// Plan 043 store-codegen: return a TS type annotation string for a parsed
/// `var`/`let` type, but ONLY when it maps to a TS builtin (number, string,
/// boolean, or arrays thereof). User-defined types return None — the store
/// composable frontend erases them to `any`, and annotating with the raw name
/// (e.g. `Block`) would be a `Cannot find name` error.
fn builtin_type_annotation(ty: &Type) -> Option<String> {
    match ty {
        Type::Int | Type::I64 | Type::Byte | Type::Char
        | Type::Uint | Type::U64 | Type::USize
        | Type::Float | Type::Double => Some("number".into()),
        Type::Bool => Some("boolean".into()),
        Type::StrFixed(_) | Type::CStrLit | Type::StrSlice | Type::StrOwned => Some("string".into()),
        Type::Array(arr) => builtin_type_annotation(&arr.elem).map(|e| format!("{}[]", e)),
        Type::RuntimeArray(rta) => builtin_type_annotation(&rta.elem).map(|e| format!("{}[]", e)),
        Type::List(elem) => builtin_type_annotation(elem)
            .or_else(|| matches!(elem.as_ref(), Type::Unknown).then(|| "any".to_string()))
            .map(|e| format!("{}[]", e)),
        Type::Slice(slice) => builtin_type_annotation(&slice.elem).map(|e| format!("{}[]", e)),
        // Plan 028 M1: bare `map`/`obj`/`list` locals → any（索引与 any[]
        // 标注满足 noImplicitAny，fn 模块局部变量不再报 TS7053/7034）。
        Type::User(u) if matches!(u.name.as_str(), "map" | "obj") => Some("any".into()),
        Type::User(u) if u.name.as_str() == "list" => Some("any[]".into()),
        // Everything else (User, Enum, Spec, Unknown, Map of custom, …) → None
        _ => None,
    }
}

/// Handles: literals, arrays, objects, lambdas, closures, f-strings,
/// indexing, ranges, tag construction, etc.
fn delegate_expr(expr: &Expr, _ctx: &AuraTsContext, out: &mut Vec<u8>) {
    let mut ts = TypeScriptTrans::new("fragment".into());
    let mut sink = Sink::new("fragment".into());
    let _ = ts.expr(expr, &mut sink);
    let _ = out.write_all(&sink.body);
}

// ---------------------------------------------------------------------------
// Builtin module transpilation (Plan 235: storage, event, json, math, date)
// ---------------------------------------------------------------------------

/// Try to transpile a method call on a builtin module (e.g. `json.parse(x)`).
/// Returns true if the call was handled, false otherwise.
fn try_transpile_builtin_call(
    object: &Expr,
    method: &str,
    args: &Args,
    ctx: &AuraTsContext,
    out: &mut Vec<u8>,
) -> bool {
    // Extract the module name from the object expression
    let module = match object {
        Expr::Ident(name) => name.as_str(),
        _ => return false,
    };

    match module {
        // json.parse(x) → JSON.parse(x); json.stringify(x) → JSON.stringify(x)
        "json" => {
            let js_method = match method {
                "parse" => "parse",
                "stringify" => "stringify",
                _ => return false,
            };
            write!(out, "JSON.{}(", js_method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        // storage.get(x) → localStorage.getItem(x); storage.set(x, y) → localStorage.setItem(x, y)
        "storage" => {
            let js_method = match method {
                "get" => "getItem",
                "set" => "setItem",
                "remove" => "removeItem",
                "clear" => "clear",
                _ => return false,
            };
            write!(out, "localStorage.{}(", js_method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        // Plan 029 T17：dom 内建模块 —— 主题/快捷键/剪贴板/外链的最小 DOM 面。
        //   dom.set_dark(on)      → html.classList add/remove("dark")
        //   dom.prefers_dark()    → matchMedia('(prefers-color-scheme: dark)').matches
        //   dom.set_css_var(n, v) → documentElement.style.setProperty(n, v)
        //   dom.focus_first(sel)  → document.querySelector(sel)?.focus()
//   dom.click_first(sel)  → document.querySelector(sel)?.click()
        //   dom.open_url(url)     → window.open(url, '_blank')
        //   dom.copy_text(text)   → navigator.clipboard.writeText(text)
        "dom" => {
            match method {
                "set_dark" => {
                    write!(out, "((document.documentElement.classList)[(").ok();
                    transpile_expr(&args.args[0].get_expr(), ctx, out);
                    write!(out, ") ? 'add' : 'remove'])('dark')").ok();
                    true
                }
                "prefers_dark" => {
                    write!(
                        out,
                        "window.matchMedia('(prefers-color-scheme: dark)').matches"
                    )
                    .ok();
                    true
                }
                "set_css_var" => {
                    write!(out, "document.documentElement.style.setProperty(").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, ")").ok();
                    true
                }
                "focus_first" => {
                    write!(out, "(document.querySelector(").ok();
                    transpile_expr(&args.args[0].get_expr(), ctx, out);
                    write!(out, ") as HTMLElement | null)?.focus()").ok();
                    true
                }
                "click_first" => {
                    write!(out, "(document.querySelector(").ok();
                    transpile_expr(&args.args[0].get_expr(), ctx, out);
                    write!(out, ") as HTMLElement | null)?.click()").ok();
                    true
                }
                "open_url" => {
                    write!(out, "window.open(").ok();
                    transpile_expr(&args.args[0].get_expr(), ctx, out);
                    write!(out, ", '_blank')").ok();
                    true
                }
                "copy_text" => {
                    write!(out, "navigator.clipboard.writeText(").ok();
                    transpile_expr(&args.args[0].get_expr(), ctx, out);
                    write!(out, ")").ok();
                    true
                }
                _ => false,
            }
        }
        // event.dispatch(name) → window.dispatchEvent(new CustomEvent(name))
        // event.dispatch(name, detail) → window.dispatchEvent(new CustomEvent(name, detail))
        "event" => {
            match method {
                "dispatch" => {
                    write!(out, "window.dispatchEvent(new CustomEvent(").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, "))").ok();
                    true
                }
                "listen" => {
                    // event.listen(name, handler) → window.addEventListener(name, handler)
                    write!(out, "window.addEventListener(").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, ")").ok();
                    true
                }
                _ => false,
            }
        }
        // router.param("id") → (useRoute().params as any)["id"]
        // router.query("q") → (useRoute().query as any)["q"]
        "router" => {
            match method {
                "param" => {
                    write!(out, "(useRoute().params as any)[").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, "]").ok();
                    true
                }
                "query" => {
                    write!(out, "(useRoute().query as any)[").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, "]").ok();
                    true
                }
                _ => false,
            }
        }
        // math.random() → Math.random(); math.floor(x) → Math.floor(x)
        "math" => {
            write!(out, "Math.{}(", method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        // date.now() → Date.now()
        "date" => {
            write!(out, "Date.{}(", method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        _ => false,
    }
}

/// Try to transpile a field access on a builtin module (e.g. `route.path`).
/// Returns true if handled, false otherwise.
fn try_transpile_builtin_field(
    object: &Expr,
    field: &str,
    _ctx: &AuraTsContext,
    out: &mut Vec<u8>,
) -> bool {
    let module = match object {
        Expr::Ident(name) => name.as_str(),
        _ => return false,
    };

    match module {
        "router" => {
            match field {
                "path" => {
                    write!(out, "useRoute().path").ok();
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// API call detection (for async handler detection)
// ---------------------------------------------------------------------------

/// Check if any statement in the list contains an API function call.
pub fn stmts_contain_api_call(stmts: &[Stmt]) -> bool {
    let default_fns: Vec<String> = DEFAULT_API_FUNCTIONS.iter().map(|s| s.to_string()).collect();
    stmts_contain_api_call_with(stmts, &default_fns)
}

/// Check if any statement in the list contains an API function call (with custom function list).
pub fn stmts_contain_api_call_with(stmts: &[Stmt], api_fns: &[String]) -> bool {
    fn walk_expr(expr: &Expr, api_fns: &[String]) -> bool {
        match expr {
            Expr::Call(call) => {
                // Only simple identifier calls can be API functions;
                // method calls (Dot names) are never API calls.
                let is_api = call.get_name_text_safe()
                    .map(|name| api_fns.iter().any(|f| f == name.as_str()))
                    .unwrap_or(false);
                is_api || call.args.args.iter().any(|a| walk_expr(&a.get_expr(), api_fns))
            }
            Expr::Bina(l, _, r) => walk_expr(l, api_fns) || walk_expr(r, api_fns),
            Expr::Unary(_, e) => walk_expr(e, api_fns),
            Expr::Dot(obj, _) => walk_expr(obj, api_fns),
            Expr::Array(items) => items.iter().any(|e| walk_expr(e, api_fns)),
            // Plan 408 P12 §10.7: `.await` expressions require an async handler.
            Expr::Await { .. } => true,
            _ => false,
        }
    }

    fn check_stmts(stmts: &[Stmt], api_fns: &[String]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Expr(expr) => walk_expr(expr, api_fns),
            Stmt::Store(store) => walk_expr(&store.expr, api_fns),
            Stmt::If(if_stmt) => if_stmt
                .branches
                .iter()
                .any(|b| walk_expr(&b.cond, api_fns) || check_stmts(&b.body.stmts, api_fns)),
            // Plan 012 P2: api calls inside try/catch/finally must still mark
            // the handler async (gap 4 — save() wrapped in try).
            Stmt::Try(t) => {
                check_stmts(&t.body.stmts, api_fns)
                    || check_stmts(&t.catch_body.stmts, api_fns)
                    || t.finally_body
                        .as_ref()
                        .map(|fb| check_stmts(&fb.stmts, api_fns))
                        .unwrap_or(false)
            }
            _ => false,
        })
    }

    check_stmts(stmts, api_fns)
}

// ---------------------------------------------------------------------------
// Route access detection (for useRoute import)
// ---------------------------------------------------------------------------

/// Check if Auto statements contain route access (Plan 235)
pub fn stmts_have_route_access(stmts: &[Stmt]) -> bool {
    fn walk_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    if let Expr::Ident(name) = object.as_ref() {
                        if name.as_str() == "route" {
                            return true;
                        }
                        // router.param("id") / router.query("q") / router.path()
                        // transpile to useRoute().params/query/path (ts_adapter
                        // ~line 1097), so they need useRoute() imported.
                        if name.as_str() == "router"
                            && (method.as_str() == "param"
                                || method.as_str() == "query"
                                || method.as_str() == "path")
                        {
                            return true;
                        }
                    }
                }
                call.args.args.iter().any(|a| walk_expr(&a.get_expr()))
            }
            Expr::Dot(object, _) => {
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "route" {
                        return true;
                    }
                }
                walk_expr(object)
            }
            Expr::Bina(l, _, r) => walk_expr(l) || walk_expr(r),
            Expr::Unary(_, e) => walk_expr(e),
            Expr::Array(items) => items.iter().any(walk_expr),
            Expr::NavCall { .. } => true,
            _ => false,
        }
    }

    fn walk_stmt(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => walk_expr(expr),
            Stmt::Store(store) => walk_expr(&store.expr),
            Stmt::If(if_stmt) => if_stmt
                .branches
                .iter()
                .any(|b| walk_expr(&b.cond) || b.body.stmts.iter().any(walk_stmt)),
            // Plan 012 P2: descend into try/catch/finally bodies.
            Stmt::Try(t) => {
                t.body.stmts.iter().any(walk_stmt)
                    || t.catch_body.stmts.iter().any(walk_stmt)
                    || t.finally_body
                        .as_ref()
                        .map(|fb| fb.stmts.iter().any(walk_stmt))
                        .unwrap_or(false)
            }
            _ => false,
        }
    }

    stmts.iter().any(walk_stmt)
}

/// Check if Auto statements contain router navigation (Plan 235)
/// Detects router.push() or router.replace() calls to trigger useRouter import.
pub fn stmts_have_router_nav(stmts: &[Stmt]) -> bool {
    fn walk_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    if let Expr::Ident(name) = object.as_ref() {
                        if name.as_str() == "router" && (method.as_str() == "push" || method.as_str() == "replace") {
                            return true;
                        }
                    }
                }
                call.args.args.iter().any(|a| walk_expr(&a.get_expr()))
            }
            Expr::Bina(l, _, r) => walk_expr(l) || walk_expr(r),
            Expr::Unary(_, e) => walk_expr(e),
            Expr::Array(items) => items.iter().any(walk_expr),
            Expr::NavCall { .. } => true,
            _ => false,
        }
    }

    fn walk_stmt(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => walk_expr(expr),
            Stmt::Store(store) => walk_expr(&store.expr),
            Stmt::If(if_stmt) => if_stmt
                .branches
                .iter()
                .any(|b| walk_expr(&b.cond) || b.body.stmts.iter().any(walk_stmt)),
            // Plan 012 P2: descend into try/catch/finally bodies.
            Stmt::Try(t) => {
                t.body.stmts.iter().any(walk_stmt)
                    || t.catch_body.stmts.iter().any(walk_stmt)
                    || t.finally_body
                        .as_ref()
                        .map(|fb| fb.stmts.iter().any(walk_stmt))
                        .unwrap_or(false)
            }
            _ => false,
        }
    }

    stmts.iter().any(walk_stmt)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Op;

    fn test_ctx() -> AuraTsContext {
        AuraTsContext::new(
            ["notes".to_string(), "tags".to_string()]
                .into_iter()
                .collect(),
        )
    }

    fn self_dot(field: &str) -> Expr {
        Expr::Dot(Box::new(Expr::Ident("self".into())), field.into())
    }

    fn bare_call(name: &str) -> Expr {
        Expr::Call(Call {
            name: Box::new(Expr::Ident(name.into())),
            args: Args::new(),
            ret: Type::Unknown,
            type_args: vec![],
            generic_args: Vec::new(),
            pos: None,
        })
    }

    fn self_method_call(method: &str) -> Expr {
        Expr::Call(Call {
            name: Box::new(self_dot(method)),
            args: Args::new(),
            ret: Type::Unknown,
            type_args: vec![],
            generic_args: Vec::new(),
            pos: None,
        })
    }

    /// D2 (Plan 358): array `+` must not degrade into JS string concat.
    /// `.tags = .tags + [t]` -> `tags.value = tags.value.concat([t])`.
    #[test]
    fn d2_array_plus_uses_concat() {
        let stmt = Stmt::Expr(Expr::Bina(
            Box::new(self_dot("tags")),
            Op::Asn,
            Box::new(Expr::Bina(
                Box::new(self_dot("tags")),
                Op::Add,
                Box::new(Expr::Array(vec![Expr::Ident("t".into())])),
            )),
        ));
        let out = transpile_handler_body(&[stmt], &test_ctx());
        assert!(
            out.contains("tags.value = tags.value.concat([t])"),
            "output:\n{}",
            out
        );
    }

    /// D3 (Plan 358): a self-method call statement (`.RefreshTags()`) must be
    /// emitted as a standalone statement, not chained onto the previous
    /// statement's expression (`list_notes().RefreshTags()`).
    #[test]
    fn d3_self_method_call_is_standalone_statement() {
        let stmts = vec![
            // .notes = list_notes()
            Stmt::Expr(Expr::Bina(
                Box::new(self_dot("notes")),
                Op::Asn,
                Box::new(bare_call("list_notes")),
            )),
            // .RefreshTags()
            Stmt::Expr(self_method_call("RefreshTags")),
        ];
        let out = transpile_handler_body(&stmts, &test_ctx());
        assert!(out.contains("notes.value = list_notes();"), "output:\n{}", out);
        assert!(out.contains("RefreshTags();"), "output:\n{}", out);
        assert!(
            !out.contains("list_notes().RefreshTags()"),
            "chained call regression:\n{}",
            out
        );
    }

    // -----------------------------------------------------------------------
    // DOM escape hatch: template refs + document/window pass-through
    // -----------------------------------------------------------------------

    fn test_ctx_with_refs() -> AuraTsContext {
        AuraTsContext::new(HashSet::new()).with_refs(
            ["menuEl".to_string(), "scrollEl".to_string()]
                .into_iter()
                .collect(),
        )
    }

    fn ref_method_call(ref_name: &str, method: &str) -> Expr {
        Expr::Call(Call {
            name: Box::new(Expr::Dot(Box::new(self_dot(ref_name)), method.into())),
            args: Args::new(),
            ret: Type::Unknown,
            type_args: vec![],
            generic_args: Vec::new(),
            pos: None,
        })
    }

    /// `.menuEl.getBoundingClientRect()` → `menuEl.value!.getBoundingClientRect()`
    #[test]
    fn template_ref_method_call() {
        let stmts = vec![
            Stmt::Expr(ref_method_call("menuEl", "getBoundingClientRect")),
            Stmt::Expr(ref_method_call("menuEl", "querySelector")),
        ];
        let out = transpile_handler_body(&stmts, &test_ctx_with_refs());
        assert!(
            out.contains("menuEl.value!.getBoundingClientRect();"),
            "output:\n{}",
            out
        );
        assert!(
            out.contains("menuEl.value!.querySelector();"),
            "output:\n{}",
            out
        );
    }

    /// `.scrollEl.clientHeight` (property read) → `scrollEl.value!.clientHeight`
    #[test]
    fn template_ref_property_read() {
        let stmt = Stmt::Expr(Expr::Dot(Box::new(self_dot("scrollEl")), "clientHeight".into()));
        let out = transpile_handler_body(&[stmt], &test_ctx_with_refs());
        assert!(out.contains("scrollEl.value!.clientHeight;"), "output:\n{}", out);
    }

    /// `.scrollEl.scrollTop = .scrollEl.scrollTop + 10` (property write)
    /// → `scrollEl.value!.scrollTop = scrollEl.value!.scrollTop + 10;`
    #[test]
    fn template_ref_property_write() {
        let scroll_top = || Expr::Dot(Box::new(self_dot("scrollEl")), "scrollTop".into());
        let stmt = Stmt::Expr(Expr::Bina(
            Box::new(scroll_top()),
            Op::Asn,
            Box::new(Expr::Bina(
                Box::new(scroll_top()),
                Op::Add,
                Box::new(Expr::Int(10)),
            )),
        ));
        let out = transpile_handler_body(&[stmt], &test_ctx_with_refs());
        assert!(
            out.contains("scrollEl.value!.scrollTop = scrollEl.value!.scrollTop + 10;"),
            "output:\n{}",
            out
        );
    }

    /// `document.activeElement` / `window.innerWidth` pass through unchanged.
    #[test]
    fn document_window_passthrough() {
        let stmts = vec![
            Stmt::Expr(Expr::Dot(
                Box::new(Expr::Ident("document".into())),
                "activeElement".into(),
            )),
            Stmt::Expr(Expr::Dot(
                Box::new(Expr::Ident("window".into())),
                "innerWidth".into(),
            )),
        ];
        let out = transpile_handler_body(&stmts, &test_ctx());
        assert!(out.contains("document.activeElement;"), "output:\n{}", out);
        assert!(out.contains("window.innerWidth;"), "output:\n{}", out);
    }

    /// Block-bodied closures (`nextTick(() => { .state = x })`) must stay
    /// Aura-aware: StateRef assignments inside the body get `.value`.
    /// Regression test — previously Expr::Block fell through to a2ts, which
    /// emitted `notes = 'tick'` (no `.value`).
    #[test]
    fn block_bodied_closure_keeps_state_ref_unwrapping() {
        let body = Body {
            stmts: vec![Stmt::Expr(Expr::Bina(
                Box::new(self_dot("notes")),
                Op::Asn,
                Box::new(Expr::Str("tick".into())),
            ))],
            has_new_line: false,
            source_lines: vec![],
        };
        let closure = Expr::Closure(Closure::new(vec![], None, Expr::Block(body)));
        let call = Expr::Call(Call {
            name: Box::new(Expr::Ident("nextTick".into())),
            args: Args {
                args: vec![Arg::Pos(closure)],
            },
            ret: Type::Unknown,
            type_args: vec![],
            generic_args: Vec::new(),
            pos: None,
        });
        let out = transpile_handler_body(&[Stmt::Expr(call)], &test_ctx());
        assert!(out.contains("() =>"), "output:\n{}", out);
        assert!(
            out.contains("notes.value = 'tick'") || out.contains("notes.value = \"tick\""),
            "StateRef lost .value inside block-bodied closure:\n{}",
            out
        );
    }

    /// Plan 043 store-codegen cat-1: `List<T>.new([...])` / `Array<T>.new()`
    /// must transpile to the array argument (or `[]`), not `List.new(...)`.
    #[test]
    fn d4_container_new_transpiles_to_array_literal() {
        // List<Block>.new([])  →  []
        let call = Expr::Call(Call {
            name: Box::new(Expr::Dot(
                Box::new(Expr::GenName("List<Block>".into())),
                "new".into(),
            )),
            args: Args { args: vec![Arg::Pos(Expr::Array(vec![]))] },
            ret: Type::Unknown,
            type_args: vec![],
            generic_args: Vec::new(),
            pos: None,
        });
        let out = transpile_handler_body(&[Stmt::Expr(call)], &test_ctx());
        assert!(
            out.contains("[]") && !out.contains("List.new"),
            "List<T>.new([]) should be [], got:\n{}",
            out
        );
    }

    /// Plan 043 store-codegen cat-2: struct-literal `Type{ field: value }`
    /// (Expr::Node) must transpile to an object literal `{ field: value }`,
    /// not `new Type(...)` (TS has no such type names). Verified end-to-end
    /// via the closed-loop `auto build` on ash-gui-auto (b.status = { }).
    /// Constructing an Expr::Node directly in-unit is awkward (Node lives in
    /// a non-Default private module), so this is covered by the closed-loop
    /// + the d4_container_new test below.

    /// Plan 043 store-codegen cat-4: `var result []str = []` must emit a
    /// type annotation for the TS-builtin array type so it's valid under
    /// noImplicitAny. User-defined types must NOT be annotated.
    #[test]
    fn d4_store_array_gets_builtin_annotation() {
        use crate::ast::SliceType;
        // var result []str = []  →  let result: string[] = [];
        let store_array = Stmt::Store(crate::ast::Store {
            kind: crate::ast::StoreKind::Var,
            name: "result".into(),
            ty: Type::Slice(SliceType { elem: Box::new(Type::StrSlice) }),
            expr: Expr::Array(vec![]),
            attrs: vec![],
            is_pub: false,
        });
        let out = transpile_handler_body(&[store_array], &test_ctx());
        assert!(
            out.contains("let result: string[] ="),
            "scalar array var should get : string[] annotation, got:\n{}",
            out
        );

        // User-type var (Type::User) must NOT be annotated — TS erases it.
        let store_user_int = Stmt::Store(crate::ast::Store {
            kind: crate::ast::StoreKind::Let,
            name: "x".into(),
            ty: Type::Int, // builtin → annotated
            expr: Expr::Int(0),
            attrs: vec![],
            is_pub: false,
        });
        let out_b = transpile_handler_body(&[store_user_int], &test_ctx());
        assert!(
            out_b.contains("let x: number ="),
            "builtin int var annotated, got:\n{}",
            out_b
        );
    }

    /// Plan 012 P2 (gap 4): Stmt::Try transpiles to JS try/catch/finally
    /// (was silently dropped by the a2ts fallback), and the api-call walker
    /// descends into all three bodies so the handler is still marked async.
    #[test]
    fn p2_try_catch_finally_emitted_and_walked() {
        use crate::ast::{Body, Try};
        let try_stmt = Stmt::Try(Try {
            body: Body {
                stmts: vec![Stmt::Expr(bare_call("saveWiki"))],
                has_new_line: false,
                source_lines: vec![],
            },
            catch_param: Some("e".into()),
            catch_body: Body {
                stmts: vec![Stmt::Expr(Expr::Bina(
                    Box::new(self_dot("error")),
                    Op::Asn,
                    Box::new(Expr::Str("failed".into())),
                ))],
                has_new_line: false,
                source_lines: vec![],
            },
            finally_body: Some(Body {
                stmts: vec![Stmt::Expr(Expr::Bina(
                    Box::new(self_dot("busy")),
                    Op::Asn,
                    Box::new(Expr::Bool(false)),
                ))],
                has_new_line: false,
                source_lines: vec![],
            }),
            new_line: false,
        });

        // Emission: real JS try/catch/finally, AURA-aware bodies.
        let ctx = AuraTsContext::new(
            ["error".to_string(), "busy".to_string()].into_iter().collect(),
        )
        .with_api_functions(vec!["saveWiki".to_string()]);
        let out = transpile_handler_body(std::slice::from_ref(&try_stmt), &ctx);
        assert!(out.contains("try {"), "try emitted:\n{}", out);
        assert!(out.contains("catch (e) {"), "catch emitted:\n{}", out);
        assert!(out.contains("finally {"), "finally emitted:\n{}", out);
        assert!(out.contains("error.value = 'failed'"), "catch body aura-aware:\n{}", out);
        assert!(out.contains("busy.value = false"), "finally body aura-aware:\n{}", out);

        // Walker: api call inside try marks the handler async.
        assert!(
            stmts_contain_api_call_with(&[try_stmt], &["saveWiki".to_string()]),
            "api call inside try must be detected"
        );
    }
}
