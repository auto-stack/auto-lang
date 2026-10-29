//! # Auto-to-Rust (a2r) Transpiler
//!
//! This module transpiles AutoLang source code to Rust, providing a native code
//! compilation path for AutoLang applications. The a2r transpiler converts AutoLang's
//! high-level syntax to idiomatic Rust code.
//!
//! ## Features
//!
//! - **Full language support**: Functions, structs, enums, closures, generics
//! - **Trait system**: AutoLang specs transpile to Rust traits
//! - **Type safety**: Preserves AutoLang's type system in Rust
//! - **Pattern matching**: AutoLang `is` expressions transpile to Rust `match`
//! - **Memory safety**: Borrow checking via AutoLang's view/mut/take semantics
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auto_lang::trans::rust::RustTrans;
//!
//! let code = r#"
//! fn main() {
//!     let x = 42
//!     print(x)
//! }
//! "#;
//!
//! let mut trans = RustTrans::new("test".into());
//! let mut sink = Sink::new(AutoStr::from("test"));
//! trans.trans(code.parse()?, &mut sink)?;
//! println!("{}", String::from_utf8(sink.done()?.to_vec())?);
//! ```
//!
//! ## Transpilation Mapping
//!
//! | AutoLang | Rust |
//! |-----------|------|
//! | `fn add(a int, b int) int` | `fn add(a: i32, b: i32) -> i32` |
//! | `let x = 42` | `let x: i32 = 42;` |
//! | `var x = 42` | `let mut x: i32 = 42;` |
//! | `(a, b) => a + b` | `|a: i32, b: i32| a + b` |
//! | `spec Flyer { fn fly() }` | `trait Flyer { fn fly(&self); }` |
//! | `type Point<T>` | `struct Point<T>` |
//! | `use auto.io: say` | `use crate::io::say;` |

use super::{escape_str, Sink, Trans};
use crate::ast::*;
use crate::database::Database;
use crate::parser::Parser;
use crate::types::TypeStore;
// Plan 091: Universe removed
use crate::{AutoError, AutoResult, Rc};
use auto_val::shared;
use auto_val::{AutoStr, Op};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::Arc;
use std::sync::RwLock;

pub enum RustEdition {
    E2021,
    E2024,
}

// Plan 387 helpers: convert an Auto task name to a Rust type / snake_case ident.
// `Counter` -> type `Counter`, spawn fn `spawn_counter`.
fn name_of(n: &crate::ast::Name) -> &str {
    n.as_str()
}

/// Convert a CamelCase task name to snake_case for the spawn helper fn name.
/// `Counter` -> `counter`, `GreeterTask` -> `greeter_task`.
fn snake_of(n: &crate::ast::Name) -> String {
    let s = n.as_str();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub struct RustTrans {
    pub assembly_modules: HashSet<String>,
    pub assembly_references: Vec<crate::stdlib_assembly::reference::ReferenceProof>,
    pub assembly_runtime: crate::stdlib_assembly::host::RustRuntime,
    indent: usize,
    uses: HashSet<AutoStr>,
    dep_crates: HashSet<AutoStr>,

    // Hybrid: Support both Universe (deprecated) and Database (new)
    // Phase 066: Migrating to Database-based architecture
    db: Option<Arc<RwLock<Database>>>, // New (Phase 066)

    // Transpiler internal state (not from Database or Universe)
    _current_fn: Option<AutoStr>,
    _current_scope: Option<crate::scope::Sid>,

    // Plan 204 Phase 3: Whether any function returns !T, requiring Err trait emission
    needs_err_trait: bool,

    // Plan 204 Phase 3: Whether current function returns !T (for Err boxing)
    current_fn_is_result: bool,

    // Inferred concrete error type for current !T function
    // If all Err(X) use the same enum E, this is Some(E) → Result<T, E>
    // Otherwise None → Result<T, Box<dyn std::error::Error>>
    current_fn_err_type: Option<AutoStr>,

    // Cache for struct field names (for positional arg mapping)
    struct_fields: HashMap<AutoStr, Vec<AutoStr>>,

    // Cache for tag type names (for tag construction detection)
    tag_types: HashSet<AutoStr>,

    // Plan 013 (B1/BUG3): Cache for locally-declared struct/type names, so
    // expression-position construction of a local type (e.g. `TierRouting{...}`)
    // is NOT spuriously qualified with an external crate prefix when a
    // `use.rust <crate>` import is present. Mirrors tag_types/known_enum_names.
    local_struct_types: HashSet<AutoStr>,

    // PLAN-009 T1 (F1, 006 S2): explicit derive traits per locally declared
    // type (struct/enum that carries `#[derive(...)]` attrs). Auto-derived
    // containers intersect their comparison-trait set with these, so a field
    // whose type explicitly derives only `Clone, Copy, Debug` no longer makes
    // the container emit `PartialEq, Eq, PartialOrd, Ord` (rustc E0369).
    explicit_derive_traits: HashMap<AutoStr, Vec<AutoStr>>,

    // PLAN-009 T1 (F1): shape info for locally AUTO-derived types (no attrs):
    // (comparison-trait mask the shape heuristics would grant, local field/
    // payload type names). Lets the restriction propagate through chains
    // (Wrapper → Inner → Grid) with order-independent memoized recursion.
    local_auto_shapes: HashMap<AutoStr, (u8, Vec<AutoStr>)>,

    // PLAN-009 T1 (F1): memo for auto_cmp_mask recursion (cycle-guarded).
    auto_cmp_mask_memo: HashMap<AutoStr, u8>,

    // PLAN-009 T1 (F1): type names whose auto-derived comparison set was
    // restricted by explicit-derive surfaces. The fix_non_ord_derives upgrade
    // pass must never re-widen them (it only sees text, so we hand it names).
    ord_restricted_names: HashSet<AutoStr>,

    // Plan 013 (B16): identifiers bound inside `is` patterns for bridge-crate
    // enum variants (e.g. `auto_val.Kid.Node(child)` binds `child: Box<Node>`).
    // When such an ident is auto-cloned at a call site, emit `(*x).clone()` so
    // the inner value is cloned, not the Box wrapper.
    bridge_pattern_bound_idents: HashSet<AutoStr>,

    // Plan 380: identifiers bound from `Some(x)` / `Ok(x)` is-patterns whose
    // scrutinee returns Option<Spec>/Result<Spec> (e.g. `is load_builtin(n) {
    // Some(prof) }` binds `prof: Box<dyn Role>`). Auto-cloning such an ident
    // at a call site is E0599 (`Box<dyn Trait>` has no Clone impl), so the
    // call-site auto-clone skips these — the value is moved instead.
    spec_bound_idents: HashSet<AutoStr>,

    // Plan 310 Phase 0.2: Cache for union type names (to rewrite construction
    // and field-access into safe accessor methods, since Rust union fields
    // require `unsafe`).
    union_types: HashSet<AutoStr>,

    // Plan 310 Phase 0.4: Ownership escape-analysis warnings.
    // Populated when a value falls back from a borrow (Tier 1) to clone (Tier 2)
    // or Rc<RefCell<T>> (Tier 3). CRITICAL: these must never be written into the
    // transpiled output (Sink), or they would corrupt .expected.rs byte diffs.
    warnings: Vec<crate::error::Warning>,

    // Plan 310 Phase 1: Per-function escape-analysis decisions.
    // Keyed by function name → EscapeMap of that function's bindings. Populated
    // by transpile_rust (after CTEE, before trans). Phase 1: populated but NOT
    // consulted — output bytes stay identical. Phase 2 wires lookups into
    // Expr::View/Mut generation.
    escape_results: HashMap<AutoStr, crate::trans::escape::EscapeMap>,

    // Plan 310 Phase 2: current function name + lexical scope depth, tracked
    // during code generation so Expr::View/Mut sites can query escape_results.
    // current_fn_name is set at fn_decl entry; current_scope_depth is reset to 0
    // at function-body entry and incremented/decremented around nested blocks.
    current_fn_name: AutoStr,
    current_scope_depth: usize,

    // Cache for struct field types: struct_name -> Vec<(field_name, field_type)>
    // Used to add .to_string() when &str is assigned to String field
    struct_field_types: HashMap<AutoStr, Vec<(AutoStr, Type)>>,
    /// Plan 399 Phase 11.4: names of loop variables that iterate a borrowed
    /// collection (`for x in &coll`). Reads `x.field` of a non-Copy type must
    /// emit `.clone()` because `x` is `&T`. Cleared per function (function-scope
    /// set; nested loops just add to it).
    borrowed_iter_vars: std::collections::HashSet<AutoStr>,
    /// Plan 032 G2-core: `Some(x)` is-arm bindings whose scrutinee is a
    /// `.get(...)` call (Map/Vec/Value lookups — Rust returns `Option<&V>`).
    /// The binding is `&V` in the emitted Rust: returning it / binding it to
    /// an owned local must emit `.clone()`. Registered per-arm (saved/restored
    /// around the arm body) at the is-arm pattern emitters.
    get_ref_bindings: std::collections::HashSet<AutoStr>,
    /// Plan 396 §2.2: `Ok(x)` is-arm bindings whose scrutinee yields a
    /// BY-VALUE iterator (fs.read_dir → std::fs::ReadDir: IntoIterator is
    /// impl'd for ReadDir only, `&ReadDir` is not an iterator → E0277).
    /// The for-loop emitter skips the `&` borrow for these names.
    by_value_iter_bindings: std::collections::HashSet<AutoStr>,
    /// Plan 427 (DIV-A2R-STRPARAM-1): `Some(x)` is-arm bindings whose scrutinee
    /// is &str-returning (strip_prefix / strip_suffix / to_str). These really
    /// are `&str` in the emitted Rust — the call-site auto-borrow must skip
    /// them (E0658 str_as_str). Registered at the is-arm emitters alongside
    /// the local_var_types entry. Plan 396 §2.4 made `is_str_slice_var` look
    /// up StrSlice in local_var_types instead, which also matched every plain
    /// `let x str` local (an owned String in Rust) and suppressed the
    /// call-site `.as_str()` → E0308 (serde_json/url/base64 a2r). The set
    /// restores §2.4's exact intended scope. Function-scoped like
    /// local_var_types (cleared at fn_decl / transpile_body_stmts entry).
    str_slice_pattern_bindings: std::collections::HashSet<AutoStr>,
    /// Plan 399 Phase 11.5: names of `let` bindings that are mutated later in
    /// the same fn body (push/insert/extend/assign). When a `let` store matches,
    /// emit `let mut` so the mutation compiles. Populated by scanning the fn
    /// body at fn_decl entry (scan_mutated_bindings).
    mutated_let_bindings: std::collections::HashSet<AutoStr>,
    /// PLAN-010 T2: idents read by statements AFTER the current one in the
    /// enclosing body(). Gates the unconstrained-List.new/None type defaults
    /// in store() — a later read can pin the real element/Option type via
    /// inference (`return Ok(messages)`), and a forced `Vec<i64>`/`Option<i64>`
    /// annotation would override it (E0308).
    later_used_locals: std::collections::HashSet<AutoStr>,
    /// PLAN-010 T2: delegation component fields per struct type
    /// (`has core WarpDrive for Engine` → `("core", WarpDrive)`). Struct
    /// literals that omit them must default-initialize the component
    /// (`Starship {}` → `Starship { core: WarpDrive {} }`; E0063 otherwise).
    struct_delegation_types: HashMap<AutoStr, Vec<(AutoStr, Type)>>,
    /// PLAN-010 T2: every `mut fn` method name seen so far (transitive set
    /// per type, unioned across types). `let x = ...; x.mutMethod()` needs
    /// `let mut x` — the scan_mutated_bindings pass consults this set.
    all_mut_method_names: std::collections::HashSet<AutoStr>,
    /// Plan 376D: Shared TypeStore from all modules (for type inference).
    /// When Some, `run_type_inference` uses this instead of building a local one.
    shared_type_store: Option<Arc<std::sync::RwLock<crate::types::TypeStore>>>,

    // Set of known enum names (for needs_enum_cast: Type::User may be an enum)
    known_enum_names: std::collections::HashSet<AutoStr>,

    // Cache for enum struct variants: (EnumName, VariantName) -> Vec<field_names>
    // Used to emit correct struct pattern syntax in match arms
    enum_struct_variants: HashMap<(AutoStr, AutoStr), Vec<AutoStr>>,

    // Cache for enum tuple variants: (EnumName, VariantName) -> arity
    // Used to emit (_) for bare tuple variant checks in match arms
    enum_tuple_variants: HashMap<(AutoStr, AutoStr), usize>,

    // Cache for enum tuple variant field types: (EnumName, VariantName) -> Vec<Type>
    // Used to add .to_string() when constructing with &str args for String fields
    enum_tuple_field_types: HashMap<(AutoStr, AutoStr), Vec<Type>>,

    // Plan 159 Phase 6B-2.2: Cache for spec declarations (for impl Trait for Type)
    spec_decls: HashMap<AutoStr, Vec<SpecMethod>>,

    // Plan 151: Global variables (top-level var declarations)
    // Tracks global variables that need Lazy<Mutex<T>> wrapper
    global_vars: HashSet<AutoStr>,
    /// PLAN-018: 全局 var 的声明类型(recv_is_list_like 全局 List 接收者
    /// 识别用——全局 List 的 .get(i)/.set(i,v) 需与局部同款索引改写)。
    global_var_types: HashMap<AutoStr, Type>,
    /// Plan 523 H5: 任一全局走了非字面量初始化（保留 once_cell Lazy 形态）
    /// ——决定产物头是否仍需 once_cell 导入。Phase 2 登记期置位。
    global_lazy_used: bool,
    /// Plan 523 W3:fn 体发射中旗标——fn 内名命中全局的 store 走全局写块
    /// 形(参考写穿语义),顶层 decl 走 static。
    in_fn_body: bool,

    // Plan 383: Top-level function names — used to recognize a bare function
    // name used as a value (function reference, e.g. `apply(handler)`) so the
    // auto-borrow layer emits a clean `handler` instead of `handler.clone()`.
    function_names: HashSet<AutoStr>,

    // Plan 167: Multi-file mode — local module names for mod declarations
    local_modules: HashSet<String>,
    /// PLAN-738 T-11：具名 stdlib 核心导入台账（裸名 → 核心模块）——
    /// `use auto.http: post_sync` 形态的裸名调用闭包依据。
    stdlib_named_imports: HashMap<String, String>,
    /// PLAN-738 T-11：wildcard 导入的核心模块集合——裸名调用按公共面
    /// 唯一归属闭包；§5.3 wildcard 导入本身不拒绝未引用 unsupported 声明。
    stdlib_wildcard_modules: HashSet<String>,
    // Multi-file mode: set of sibling module names (same directory)
    // Used to generate `use super::X;` instead of `use crate::X;`
    sibling_modules: HashSet<String>,
    // Multi-file mode: dir children of a directory module (mod.rs/mod.at)
    // use X for these should be skipped (pub mod X; already emitted)
    dir_children: HashSet<String>,
    // Whether current module is a directory module
    is_dir_module: bool,
    // Whether we're inside a pub type declaration (methods should be pub)
    inside_pub_type: bool,
    in_trait_impl: bool, // Plan 373 G2: true when emitting methods inside `impl Trait for Type`
    // Modules imported via `use X` → `use super::X::*;` in multi-file mode
    // These should NOT be used as source_crate prefix for type resolution
    glob_imported_modules: HashSet<String>,

    // Plan 232: Track current function's str-type parameter names
    // Used to add .to_string() when returning a &str param as String
    current_fn_str_params: HashSet<AutoStr>,

    // Track which function params are str (&str) type for auto-borrow at call sites
    // fn_name -> vec of booleans (true = param is str/&str, needs & at call site)
    fn_str_param_indices: HashMap<AutoStr, Vec<bool>>,
    /// Plan 514 W3: 当前正在发射的 type/ext 的传递性可变方法名集合
    /// (直接字段写/已知 Rust 变异方法为种子,经 self 接收者调用闭包)。
    /// fn_decl 的接收者 &mut 判定据此跨方法传递(expect→next 族)。
    current_type_mut_methods: Option<std::collections::HashSet<AutoStr>>,
    /// Plan 531 真缺陷②:当前正在发射的 type/ext 的名字。self.<field>
    /// 接收者的字段实型解析据(struct_field_types 查表)——514 W3 的
    /// `self.* 一律按 List 索引化` 使 Map 字段 .get 被改写成
    /// `[(key) as usize]`(str as usize 非法强转)。
    current_impl_type: Option<AutoStr>,

    // Track current function's return type for string coercion
    current_fn_ret_type: Option<Type>,

    /// Plan 417-E3: type-parameter names of the fn currently being emitted
    /// (`fn max_of<T has Comparable>(...)` → {"T"}). A `Type::User` whose name
    /// is in this set renders as the bare param name — never `impl T` (the
    /// PascalCase trait heuristic) and never a concrete-struct lookup.
    current_fn_type_params: std::collections::HashSet<AutoStr>,
    // Plan 417-E2 followup (F1): associated-type bindings of the TypeDecl
    // currently being emitted (`as Container<Item=int>` → Item → int). Rust is
    // static, so any reference to the assoc name inside the impl — method
    // signature, local `var x Item` annotation, field type — must render as
    // the bound type. Set around the type_decl() call site; empty elsewhere.
    current_assoc_bindings: HashMap<AutoStr, Type>,

    /// Plan 417-E4 调查: while emitting an if that IS the fn-body tail value,
    /// branch tail expressions omit the statement-context `;` (if_stmt's
    /// Plan 393 E3 rule). Nested non-last ifs restore statement mode.
    value_if_tail: bool,

    // Track local variable types for string concat detection in Op::Add
    local_var_types: HashMap<AutoStr, Type>,
    // PLAN-724 T-06: 当前发射是否处于 async 上下文（~T/Future 返回 fn、
    // async main、generator body、.go spawn 块）。HTTP 客户端分派据此选
    // async 面（.await）或同步桥接面；不支持位置回落同步发射并在运行期
    // 得到响亮诊断（内核同步桥接在 async 上下文 panic，不静默阻塞）。
    in_async_ctx: std::cell::Cell<bool>,
    // PLAN-724 T-06: 绑定自 HTTP 流生产者的变量（同步/异步分型）。
    // 异步集合决定 .next()/.is_done()/.close() 与 for-in 的发射形态。
    http_stream_vars: HashSet<AutoStr>,
    http_stream_async_vars: HashSet<AutoStr>,
    // PLAN-724 T-06: 绑定自 http.request(...) 的 builder 变量（.send 分派）。
    http_builder_vars: HashSet<AutoStr>,
    // Plan 447 H5: per-function count of `is` statements per identifier scrutinee.
    // `match &v` (borrow instead of move) is only emitted when the same ident is
    // matched >= 2 times in one function — narrower scope keeps a2r goldens stable
    // and preserves by-value payload moves in single-match arms.
    fn_is_scrutinee_counts: HashMap<AutoStr, usize>,
    // Plan 447 H4: monotonic counter for hoisted guard-arm scrutinee temps
    // (`__is_0`, `__is_1`, ...). Never reset — uniqueness is all we need.
    is_hoist_counter: usize,
    // Plan 447 ③-前置: set ONLY while emitting an is-arm's pattern list whose
    // scrutinee is int-typed — char literal patterns then render as code-point
    // decimals (Expr::Char). Bodies reset it (their char literals keep the
    // D37 comparison bridge).
    int_match_scrutinee: bool,
    // Track variables assigned from json.get() — need value_to_int/value_len helpers
    json_value_vars: HashSet<AutoStr>,
    /// P738-R5-02：json 值生产调用（json.parse/get/get_at/get_str/get_u64）
    /// 绑定的变量 def-use 集——receiver 方法闭包判据（不改变发射行为）。
    json_value_bindings: HashSet<AutoStr>,
    // Plan 016 Phase A A.4: when true, emit json::parse_opt instead of json::parse
    // (set by is_stmt when scrutinee is json.parse matched against Some/None).
    json_parse_as_opt: bool,
    // Track function params declared as &str (StrSlice) — safe to pass without .as_str()
    fn_param_str_slice: HashSet<AutoStr>,
    // Track current function's &mut params (merge mode context types) — skip &mut at call sites
    current_fn_mut_params: HashSet<AutoStr>,

    // Track which function params are struct/enum types (need .clone() at call sites)
    fn_struct_param_indices: HashMap<AutoStr, Vec<bool>>,
    // Full parameter types per function: fn_name -> Vec<Type>
    // Used for precise type-aware call site generation (&mut, &str, etc.)
    fn_param_types: HashMap<AutoStr, Vec<Type>>,
    fn_ret_types: HashMap<AutoStr, Type>, // Plan 373: return-type cache for .await insertion
    // In merge mode, track which params use &mut (context types like Parser, TypeEnv)
    fn_merge_mut_params: HashMap<AutoStr, Vec<bool>>,
    // C11 (Plan 018 §12 a2r-11): fn_name -> which params are `mut p T` (&mut T).
    // Call sites pass `&mut arg` instead of `arg.clone()` for these.
    fn_mut_params: HashMap<AutoStr, Vec<bool>>,
    // C11 (Plan 018 §12 a2r-11): depth of assignment-LHS emission. While >0,
    // the List `.get()` → index conversion skips `.clone()` so in-place element
    // mutation (`doc.items[i].field = v`) writes the real element, not a clone.
    assign_lhs_depth: usize,
    // Track which function params are Int type (need enum→i32 cast at call sites)
    fn_int_param_indices: HashMap<AutoStr, Vec<bool>>,
    // Track which function params are spec types (need Box::new() at call sites)
    fn_spec_param_indices: HashMap<AutoStr, Vec<bool>>,
    // Track struct→spec mapping: struct_name -> spec_name (for spec array inference)
    struct_to_spec: HashMap<AutoStr, AutoStr>,
    // Track variable→spec mapping: var_name -> spec_name
    var_spec_map: HashMap<AutoStr, AutoStr>,

    // Whether to emit #![allow(...)] pragma at file top (for full files, not test fragments)
    pub emit_allow_pragma: bool,

    // Plan 376U: When true, top-level `use module: symbol` statements render as
    // `pub use crate::module::{symbol};` (re-exports) instead of private `use`.
    // Set for crate-root files (lib.at, */mod.at) whose `use` statements are
    // public re-exports, not private imports.
    pub is_crate_root: bool,

    // Merge mode: all modules compiled into single .rs file
    // When true: skip mod X; declarations, skip use crate::X::*; / use super::X::*;
    merge_mode: bool,

    // Plan 610 ⑤: the pointer bridge kit (`auto_cabi_kit` module) is emitted
    // at most once per product, ahead of the first pointer-facing #[export]
    // wrapper.
    cabi_kit_emitted: bool,

    // Plan 610 ⑥: directory of the source file being transpiled — resolves
    // relative `use.c "<file>.json"` manifest paths (source dir first, then
    // CWD). None = CWD-only (legacy single-file entry).
    pub source_dir: Option<std::path::PathBuf>,

    // Const names seen during Phase 2.5 pre-scan (for merge mode).
    // Used to convert SCREAMING_CASE() calls to bare const references.
    const_names: HashSet<AutoStr>,

    // Plan 264: Maps module name → set of type names defined in that module.
    // Used to determine if `module.Type` should be `module::Type` in Rust.
    module_types: HashMap<String, HashSet<String>>,
    // Plan 264: Name of the module currently being transpiled.
    // Types defined in the current module don't need crate:: prefix.
    current_module_name: String,

    // Plan 270: Track whether any a2r_std symbol was actually emitted.
    // When false, skip the `use auto_lang::a2r_std::*;` import so the
    // generated Rust code can compile without depending on auto_lang.
    // Uses Cell for interior mutability (avoids borrow conflicts with &self writes).
    a2r_std_used: std::cell::Cell<bool>,

    // Plan 387: set true when any `task` definition is seen, so `fn_decl` forces
    // `#[tokio::main]` + async main for the program and emits the `drain_all`
    // epilogue. Actor programs use the multi_thread runtime (current_thread
    // deadlocked — see the comment in fn_decl); stdout behavior still matches
    // the VM's single-threaded cooperative actor scheduling (Plan 317 path B).
    program_has_actors: bool,

    // Plan 387: while compiling a task hook/handler body, set true so that bare
    // state-field identifiers are rewritten to `self.<field>`. Toggled by
    // compile_task_body; read by store()/expr() (Expr::Ident) for the rewrite.
    in_task_body: bool,

    // Plan 391 D1: when true, suppress the `as i32` cast that `.len()`/`.length()`
    // normally get under Auto's int model. `fs::Metadata::len()`, `Vec::len()`,
    // `HashMap::len()`, ... all return usize in Rust; forcing `as i32` both
    // truncates and conflicts with a wider declared type (`let sz u64 = ...`).
    // Set transiently (with save/restore) in store()/assignment while emitting
    // the RHS of a binding whose declared type is a wide integer (u64/i64/usize)
    // and the RHS is a `.len()`/`.length()` call. Read at the two len/length cast
    // sites in call(). Defaults false so all other call sites keep the cast.
    len_i32_cast_suppressed: bool,
    /// PLAN-668 R-24：str 族 `let`（含 var）以 if 表达式为初始化器时，
    /// 臂尾字符串字面量需 .to_string()（String 槽不可收 &str——017 db.at
    /// bot_reply 的 sender_name/reply_text 实案，此前仅 fn 返回位有同款
    /// 强制）。save/restore 围初始化器发射（len_i32_cast_suppressed 先例）。
    let_init_str_coercion: bool,

    // Plan 387: the set of state-field names of the task currently being compiled.
    // Populated by emit_task_impl/emit_task_handle_msg; consulted (together with
    // in_task_body) to rewrite bare `count` -> `self.count`.
    task_state_fields: std::collections::HashSet<String>,

    // Plan 387: when Some, body() emits this prologue right after the opening `{`
    // (and a matching epilogue before the closing `}`). Used to inject the
    // `let mut __rt = ...;` bootstrap and `__rt.run_to_completion().await;` drain
    // into an actor program's `fn main`.
    main_actor_prologue: Option<String>,
    main_actor_epilogue: Option<String>,

    // Plan 387 W4 + follow-up: map from a task message-variant name (e.g.
    // "Add", "Reset") to the task(s) that declare it, so `h.send(Add(5))` can be
    // rewritten to the RIGHT enum (`CounterMsg::Add(5)` vs `LedgerMsg::Add(5)`).
    // Populated by task_decl; consulted by call() to rewrite send args.
    // A flat variant→enum map silently picked the LAST task's enum when two
    // tasks declared the same variant name (cross-task conflict).
    task_variants: std::collections::HashMap<String, std::collections::HashSet<String>>,

    // Plan 387 follow-up: handle variable name → task name, recorded when a
    // `let h = Task.spawn("Counter", cap)` result is bound. Lets `h.send(...)`
    // resolve the message enum from the receiver's task, disambiguating
    // same-named variants across tasks.
    handle_task_map: std::collections::HashMap<AutoStr, String>,
}

impl RustTrans {
    pub fn new(_name: AutoStr) -> Self {
        Self {
            assembly_modules: HashSet::new(),
            assembly_references: Vec::new(),
            assembly_runtime: Default::default(),
            indent: 0,
            uses: HashSet::new(),
            dep_crates: HashSet::new(),
            db: None,
            _current_fn: None,
            _current_scope: None,
            needs_err_trait: false,
            current_fn_is_result: false,
            current_fn_err_type: None,
            struct_fields: HashMap::new(),
            struct_field_types: HashMap::new(),
            borrowed_iter_vars: std::collections::HashSet::new(),
            get_ref_bindings: std::collections::HashSet::new(),
            by_value_iter_bindings: std::collections::HashSet::new(),
            str_slice_pattern_bindings: std::collections::HashSet::new(),
            mutated_let_bindings: std::collections::HashSet::new(),
            later_used_locals: std::collections::HashSet::new(),
            struct_delegation_types: HashMap::new(),
            all_mut_method_names: std::collections::HashSet::new(),
            shared_type_store: None,
            known_enum_names: std::collections::HashSet::new(),
            tag_types: HashSet::new(),
            local_struct_types: HashSet::new(),
            explicit_derive_traits: HashMap::new(),
            local_auto_shapes: HashMap::new(),
            auto_cmp_mask_memo: HashMap::new(),
            ord_restricted_names: HashSet::new(),
            bridge_pattern_bound_idents: HashSet::new(),
            spec_bound_idents: HashSet::new(),
            union_types: HashSet::new(),
            warnings: Vec::new(),
            escape_results: HashMap::new(),
            current_fn_name: AutoStr::from(""),
            current_scope_depth: 0,
            enum_struct_variants: HashMap::new(),
            enum_tuple_variants: HashMap::new(),
            enum_tuple_field_types: HashMap::new(),
            spec_decls: HashMap::new(),
            global_vars: HashSet::new(),
            global_var_types: HashMap::new(),
            global_lazy_used: false,
            in_fn_body: false,
            function_names: HashSet::new(),
            local_modules: HashSet::new(),
            stdlib_named_imports: HashMap::new(),
            stdlib_wildcard_modules: HashSet::new(),
            sibling_modules: HashSet::new(),
            dir_children: HashSet::new(),
            is_dir_module: false,
            inside_pub_type: false,
            in_trait_impl: false,
            glob_imported_modules: HashSet::new(),
            current_fn_str_params: HashSet::new(),
            fn_str_param_indices: HashMap::new(),
            current_type_mut_methods: None,
            current_impl_type: None,
            current_fn_ret_type: None,
            current_fn_type_params: std::collections::HashSet::new(), // Plan 417-E3
            current_assoc_bindings: HashMap::new(),                   // Plan 417-E2 followup
            value_if_tail: false,
            local_var_types: HashMap::new(),
            in_async_ctx: std::cell::Cell::new(false),
            http_stream_vars: HashSet::new(),
            http_stream_async_vars: HashSet::new(),
            http_builder_vars: HashSet::new(),
            fn_is_scrutinee_counts: HashMap::new(),
            is_hoist_counter: 0,
            int_match_scrutinee: false,
            json_value_vars: HashSet::new(),
            json_value_bindings: HashSet::new(),
            json_parse_as_opt: false,
            fn_param_str_slice: HashSet::new(),
            current_fn_mut_params: HashSet::new(),
            fn_struct_param_indices: HashMap::new(),
            fn_param_types: HashMap::new(),
            fn_ret_types: HashMap::new(),
            fn_merge_mut_params: HashMap::new(),
            fn_mut_params: HashMap::new(),
            assign_lhs_depth: 0,
            fn_int_param_indices: HashMap::new(),
            struct_to_spec: HashMap::new(),
            var_spec_map: HashMap::new(),
            fn_spec_param_indices: HashMap::new(),
            emit_allow_pragma: false,
            is_crate_root: false,
            merge_mode: false,
            cabi_kit_emitted: false, // Plan 610
            source_dir: None,        // Plan 610
            const_names: HashSet::new(),
            module_types: HashMap::new(),
            current_module_name: String::new(),
            a2r_std_used: std::cell::Cell::new(false),
            program_has_actors: false,
            in_task_body: false,
            len_i32_cast_suppressed: false,
            let_init_str_coercion: false,
            task_state_fields: std::collections::HashSet::new(),
            main_actor_prologue: None,
            main_actor_epilogue: None,
            task_variants: std::collections::HashMap::new(),
            handle_task_map: std::collections::HashMap::new(),
        }
    }

    /// Create transpiler with Database (Phase 066: new API)
    pub fn with_database(db: Arc<RwLock<Database>>) -> Self {
        Self {
            assembly_modules: HashSet::new(),
            assembly_references: Vec::new(),
            assembly_runtime: Default::default(),
            indent: 0,
            uses: HashSet::new(),
            dep_crates: HashSet::new(),
            db: Some(db),
            _current_fn: None,
            _current_scope: None,
            needs_err_trait: false,
            current_fn_is_result: false,
            current_fn_err_type: None,
            struct_fields: HashMap::new(),
            struct_field_types: HashMap::new(),
            borrowed_iter_vars: std::collections::HashSet::new(),
            get_ref_bindings: std::collections::HashSet::new(),
            by_value_iter_bindings: std::collections::HashSet::new(),
            str_slice_pattern_bindings: std::collections::HashSet::new(),
            mutated_let_bindings: std::collections::HashSet::new(),
            later_used_locals: std::collections::HashSet::new(),
            struct_delegation_types: HashMap::new(),
            all_mut_method_names: std::collections::HashSet::new(),
            shared_type_store: None,
            known_enum_names: std::collections::HashSet::new(),
            tag_types: HashSet::new(),
            local_struct_types: HashSet::new(),
            explicit_derive_traits: HashMap::new(),
            local_auto_shapes: HashMap::new(),
            auto_cmp_mask_memo: HashMap::new(),
            ord_restricted_names: HashSet::new(),
            bridge_pattern_bound_idents: HashSet::new(),
            spec_bound_idents: HashSet::new(),
            union_types: HashSet::new(),
            warnings: Vec::new(),
            escape_results: HashMap::new(),
            current_fn_name: AutoStr::from(""),
            current_scope_depth: 0,
            enum_struct_variants: HashMap::new(),
            enum_tuple_variants: HashMap::new(),
            enum_tuple_field_types: HashMap::new(),
            spec_decls: HashMap::new(),
            global_vars: HashSet::new(),
            global_var_types: HashMap::new(),
            global_lazy_used: false,
            in_fn_body: false,
            function_names: HashSet::new(),
            local_modules: HashSet::new(),
            stdlib_named_imports: HashMap::new(),
            stdlib_wildcard_modules: HashSet::new(),
            sibling_modules: HashSet::new(),
            dir_children: HashSet::new(),
            is_dir_module: false,
            inside_pub_type: false,
            in_trait_impl: false,
            glob_imported_modules: HashSet::new(),
            current_fn_str_params: HashSet::new(),
            fn_str_param_indices: HashMap::new(),
            current_type_mut_methods: None,
            current_impl_type: None,
            current_fn_ret_type: None,
            current_fn_type_params: std::collections::HashSet::new(), // Plan 417-E3
            current_assoc_bindings: HashMap::new(),                   // Plan 417-E2 followup
            value_if_tail: false,
            local_var_types: HashMap::new(),
            in_async_ctx: std::cell::Cell::new(false),
            http_stream_vars: HashSet::new(),
            http_stream_async_vars: HashSet::new(),
            http_builder_vars: HashSet::new(),
            fn_is_scrutinee_counts: HashMap::new(),
            is_hoist_counter: 0,
            int_match_scrutinee: false,
            json_value_vars: HashSet::new(),
            json_value_bindings: HashSet::new(),
            json_parse_as_opt: false,
            fn_param_str_slice: HashSet::new(),
            current_fn_mut_params: HashSet::new(),
            fn_struct_param_indices: HashMap::new(),
            fn_param_types: HashMap::new(),
            fn_ret_types: HashMap::new(),
            fn_merge_mut_params: HashMap::new(),
            fn_mut_params: HashMap::new(),
            assign_lhs_depth: 0,
            fn_int_param_indices: HashMap::new(),
            struct_to_spec: HashMap::new(),
            var_spec_map: HashMap::new(),
            fn_spec_param_indices: HashMap::new(),
            emit_allow_pragma: false,
            is_crate_root: false,
            merge_mode: false,
            cabi_kit_emitted: false, // Plan 610
            source_dir: None,        // Plan 610
            const_names: HashSet::new(),
            module_types: HashMap::new(),
            current_module_name: String::new(),
            a2r_std_used: std::cell::Cell::new(false),
            program_has_actors: false,
            in_task_body: false,
            len_i32_cast_suppressed: false,
            let_init_str_coercion: false,
            task_state_fields: std::collections::HashSet::new(),
            main_actor_prologue: None,
            main_actor_epilogue: None,
            task_variants: std::collections::HashMap::new(),
            handle_task_map: std::collections::HashMap::new(),
        }
    }

    /// Plan 400 Phase 2: transpile only body statements (no signature).
    /// `params` pre-populates local_var_types. Returns indented lines.
    /// PLAN-730 T-06: 独立体转译的 async 上下文置位（api_gen 的
    /// try_transpile_body 对 `~T` 异步 handler 内联体调用——await 点合法）。
    pub fn set_async_ctx(&self, on: bool) {
        self.in_async_ctx.set(on);
    }

    /// Plan 742 R1 (F-742-R1-01): per-byte code-region mask for one emitted
    /// line — `true` = ordinary code, `false` = inside a `"…"`/`'…'` literal
    /// (escape-aware) or a `//`/`/* */` comment. Body-text post-passes may
    /// only rewrite masked-true spans: the raw scans of the first 742 round
    /// rewrote literal payloads (`"List<int>.new([])"` → `"vec![]"`,
    /// `"} return secret"` split mid-string) while generation still reported
    /// Ok. Boundary: single-line scans over emitter output — raw strings and
    /// block comments spanning lines are outside the emitted-body surface
    /// this guards.
    pub fn code_region_mask(line: &str) -> Vec<bool> {
        #[derive(Clone, Copy, PartialEq)]
        enum St {
            Code,
            Str,
            Char,
            LineComment,
            BlockComment,
        }
        let bytes = line.as_bytes();
        let mut mask = vec![false; bytes.len()];
        let mut st = St::Code;
        let mut i = 0;
        while i < bytes.len() {
            match st {
                St::Code => match bytes[i] {
                    b'"' => st = St::Str,
                    b'\'' => st = St::Char,
                    b'/' if bytes.get(i + 1) == Some(&b'/') => st = St::LineComment,
                    b'/' if bytes.get(i + 1) == Some(&b'*') => st = St::BlockComment,
                    _ => mask[i] = true,
                },
                St::Str | St::Char => {
                    if bytes[i] == b'\\' {
                        i += 1; // escape: the escaped byte stays masked-out
                    } else if bytes[i] == if st == St::Str { b'"' } else { b'\'' } {
                        st = St::Code;
                    }
                }
                St::LineComment => {}
                St::BlockComment => {
                    if bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/') {
                        i += 1;
                        st = St::Code;
                    }
                }
            }
            i += 1;
        }
        mask
    }

    /// Plan 376D: Set the shared TypeStore for cross-module type inference.
    pub fn set_shared_type_store(
        &mut self,
        store: Option<Arc<std::sync::RwLock<crate::types::TypeStore>>>,
    ) {
        self.shared_type_store = store;
    }

    /// Extract the type name from a constructor expression.
    fn extract_tag_or_ctor_type(expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Call(call) => {
                if let Expr::Ident(name) = call.name.as_ref() {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get Database reference (Phase 066)
    pub fn db(&self) -> Option<&Arc<RwLock<Database>>> {
        self.db.as_ref()
    }

    // =========================================================================
    // Plan 151: Tauri IPC Mode - Global Variable Support
    // =========================================================================

    /// Register a global variable (top-level var declaration)

    /// PLAN-018: register with declared type(全局 List 接收者识别)。
    pub fn register_global_var_typed(&mut self, name: AutoStr, ty: Type) {
        self.global_vars.insert(name.clone());
        self.global_var_types.insert(name, ty);
    }

    /// Check if a variable is a global variable
    pub fn is_global_var(&self, name: &AutoStr) -> bool {
        self.global_vars.contains(name)
    }

    /// Plan 523 H5: 全局变量字面量初始化判定——数值/布尔/字符/字符串
    /// 字面量走 std const 通道 `static X: Mutex<T> = Mutex::new(lit);`
    /// （Rust 1.63+ const Mutex::new，产物零外部依赖——once_cell 导入使
    /// 独立 rustc 产物 E0433，b42/b43 考古）。非字面量保留 Lazy 形态。
    fn global_store_is_const_init(store: &Store) -> bool {
        matches!(
            &store.expr,
            Expr::Int(_)
                | Expr::Uint(_)
                | Expr::Float(_, _)
                | Expr::Double(_, _)
                | Expr::Bool(_)
                | Expr::Char(_)
                | Expr::Str(_)
                | Expr::CStr(_)
        )
    }

    /// Get the uppercase name for a global variable static
    pub fn global_var_static_name(&self, name: &AutoStr) -> String {
        name.to_uppercase().to_string()
    }

    // =========================================================================
    // Phase 066: Unified Helper Methods (Universe or Database)
    // =========================================================================

    /// Look up metadata by name (works with Universe or Database)
    /// Phase 066: Unified helper for Database/Universe access
    fn lookup_meta(&self, name: &str) -> Option<Rc<crate::scope::Meta>> {
        // Plan 091: Use Database only
        if let Some(db) = &self.db {
            // New path: Database
            if let Ok(db) = db.try_read() {
                // Search through symbol tables for the symbol
                for (_sid, table) in db.get_all_symbol_tables() {
                    if let Some(meta) = table.symbols.get(name) {
                        return Some(meta.clone());
                    }
                }
                None
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Look up type by name (works with Universe or Database)
    /// Phase 066: Unified helper for Database/Universe access
    #[allow(dead_code)]
    fn lookup_type(&self, _type_name: &AutoStr) -> Type {
        // Plan 091: Use Database only
        if let Some(_db) = &self.db {
            // New path: Database
            // NOTE: TypeInfoStore doesn't store type kind (enum/struct/union)
            // Return Type::Unknown for now (conservative approach)
            // TODO: Enhance Database to store type metadata (enum/struct/union)
            Type::Unknown
        } else {
            Type::Unknown
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
        for _ in 0..self.indent {
            out.write(b"    ")?;
        }
        Ok(())
    }

    /// Emit a2r standard library import
    /// Uses the crate's a2r_std module instead of embedding
    fn emit_a2r_stdlib(&self, out: &mut impl Write) -> AutoResult<()> {
        writeln!(out, "// Auto-generated by a2r transpiler")?;
        if self.emit_allow_pragma {
            writeln!(out, "#![allow(dead_code, unreachable_code, unused_imports, unused_mut, unused_parens, unused_assignments, unused_variables)]")?;
        }
        writeln!(out)?;
        Ok(())
    }

    // is_enum_type() moved to unified helper methods (line 83)
    // Old implementation removed in Phase 066

    /// Plan 373: Check if a type represents an async result (Future/~Result).
    /// Plan 382 (A.1): `Type::Result` is produced ONLY by `!T` (Plan 204 —
    /// SYNCHRONOUS `Result<T, Box<dyn Error>>`); async is `~T` → GenericInstance
    /// ("Future"). Treating `!T` as async made sync functions async + inserted
    /// `.await` (regression since d269a92d). Exclude Type::Result here.
    fn type_is_async(ty: &Type) -> bool {
        matches!(ty, Type::Handle { .. })
            || matches!(ty, Type::GenericInstance(inst) if inst.base_name == "Future")
    }

    /// Plan 364 Phase 8 F1: Does this for-loop iterable produce a `~Stream<T>`
    /// (i.e. `impl futures::Stream`)? If so, the `for` loop must be rewritten to
    /// `while let Some(x) = stream.next().await` because `impl Stream` does not
    /// implement `IntoIterator`. Only bare-name function calls are checked
    /// (qualified method-call streams are rare and can be added later).
    fn iterable_is_stream(&self, range: &Expr) -> bool {
        if let Expr::Call(call) = range {
            if let Expr::Ident(fname) = call.name.as_ref() {
                if let Some(ret) = self.fn_ret_types.get(fname.as_str()) {
                    // Plan 417-D2: ~Iter<T> generator calls ride the same
                    // stream consumption path — the generator body is wrapped
                    // in async_stream::stream! (a Stream, not an Iterator), so
                    // for-in must become the `while let ... .await` form and
                    // main must go async, exactly like ~Stream.
                    return matches!(ret,
                        Type::GenericInstance(inst)
                            if matches!(inst.base_name.as_str(), "Stream" | "Iter"));
                }
            }
        }
        false
    }

    /// Plan 018 §14 W2: is this Store (`var g = <expr>`) a Mutex guard binding?
    /// Detects `var guard = self.cache.lock().unwrap()` — the pattern whose
    /// `is guard.get(k) { None -> {} }` scrutinee would otherwise keep the
    /// guard alive past a later `lock()` in the same fn (deadlock).
    fn store_is_lock_guard(store: &Store) -> bool {
        // `var` storage only (guards are locals).
        if !matches!(store.kind, crate::ast::StoreKind::Var) {
            return false;
        }
        // expr is `X.lock().unwrap()` — a Call whose callee is `X.lock()` and
        // method name is unwrap (or a direct `.lock()` for non-unwrap users).
        match &store.expr {
            Expr::Call(call) => {
                if let Expr::Dot(_, method) = call.name.as_ref() {
                    return method.as_str() == "lock" || method.as_str() == "unwrap";
                }
                false
            }
            _ => false,
        }
    }

    /// Plan 018 §14 W2: if the `is` scrutinee is `guard.method(args)` where
    /// guard is an Ident receiver, return the guard name. The scrutinee is a
    /// Call (`guard.get(k)`), whose callee is a Dot(Ident, method).
    fn is_scrutinee_receiver(is_stmt: &Is) -> Option<AutoStr> {
        match &is_stmt.target {
            Expr::Call(call) => {
                if let Expr::Dot(obj, _) = call.name.as_ref() {
                    if let Expr::Ident(name) = obj.as_ref() {
                        return Some(name.clone());
                    }
                }
                None
            }
            Expr::Dot(obj, _) => {
                if let Expr::Ident(name) = obj.as_ref() {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Plan 018 §14 W2: does any statement in the slice reference the ident
    /// (as a bare name, or as a member receiver / call arg)? Shallow scan —
    /// covers the common guard-reuse patterns (guard.get/guard.insert/guard
    /// passed as an arg).
    fn stmts_reference_ident(stmts: &[Stmt], ident: &AutoStr) -> bool {
        for stmt in stmts {
            if Self::stmt_references_ident(stmt, ident) {
                return true;
            }
        }
        false
    }

    /// PLAN-010 T2: best-effort Option/Result-ness of an expression's type.
    /// Some(true) = Option/Result; Some(false) = known non-optional type;
    /// None = unknown (callers fall back to the legacy untyped lowering).
    fn expr_type_is_option_like(&self, e: &Expr) -> Option<bool> {
        let ty = match e {
            Expr::Ident(n) => match self.local_var_types.get(n.as_str()) {
                Some(t) => t.clone(),
                None => self.infer_type_from_expr(e),
            },
            _ => self.infer_type_from_expr(e),
        };
        match ty {
            Type::Option(_) | Type::Result(_) => Some(true),
            Type::Unknown => None,
            _ => Some(false),
        }
    }

    /// Plan musk-022: does this function body contain a `yield` expression?
    fn scan_body_has_yield(body: &Body) -> bool {
        body.stmts.iter().any(Self::stmt_has_yield)
    }

    fn stmt_has_yield(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e) => Self::expr_has_yield(e),
            Stmt::Return(e) => Self::expr_has_yield(e),
            Stmt::If(iff) => {
                iff.branches
                    .iter()
                    .any(|br| Self::expr_has_yield(&br.cond) || Self::scan_body_has_yield(&br.body))
                    || iff
                        .else_
                        .as_ref()
                        .map_or(false, |b| Self::scan_body_has_yield(b))
            }
            Stmt::For(f) => Self::expr_has_yield(&f.range) || Self::scan_body_has_yield(&f.body),
            Stmt::Block(b) => Self::scan_body_has_yield(b),
            _ => false,
        }
    }

    fn expr_has_yield(expr: &Expr) -> bool {
        match expr {
            Expr::Yield(_) => true,
            Expr::View(e)
            | Expr::Mut(e)
            | Expr::Move(e)
            | Expr::Take(e)
            | Expr::Unary(_, e)
            | Expr::Some(e)
            | Expr::Ok(e)
            | Expr::Err(e)
            | Expr::ErrorPropagate(e)
            | Expr::BoxExpr(e)
            | Expr::ArcExpr(e) => Self::expr_has_yield(e),
            Expr::Bina(lhs, _, rhs) => Self::expr_has_yield(lhs) || Self::expr_has_yield(rhs),
            Expr::NullCoalesce(a, b) => Self::expr_has_yield(a) || Self::expr_has_yield(b),
            Expr::Dot(receiver, _) => Self::expr_has_yield(receiver),
            Expr::Call(call) => {
                Self::expr_has_yield(&call.name)
                    || call.args.args.iter().any(|arg| match arg {
                        crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => {
                            Self::expr_has_yield(e)
                        }
                        _ => false,
                    })
            }
            Expr::Index(t, i) => Self::expr_has_yield(t) || Self::expr_has_yield(i),
            Expr::Array(elems) | Expr::Tuple(elems) => elems.iter().any(Self::expr_has_yield),
            Expr::Await { expr: e } | Expr::Go { expr: e } => Self::expr_has_yield(e),
            _ => false,
        }
    }

    /// Plan 204 Phase 1D: Emit all statements in a loop body.
    /// Previously, only Stmt::Expr and Stmt::Store were handled, silently
    /// dropping other statement types (nested loops, if, break, return, etc.)
    fn emit_loop_body(&mut self, body: &Body, sink: &mut Sink) -> AutoResult<()> {
        for (i, stmt) in body.stmts.iter().enumerate() {
            sink.record();
            if i < body.source_lines.len() {
                sink.set_source_line(body.source_lines[i]);
            }
            self.print_indent(&mut sink.body)?;
            match stmt {
                Stmt::Expr(expr) => {
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b";\n")?;
                }
                Stmt::Store(store) => {
                    self.store(store, &mut sink.body)?;
                    sink.body.write(b";\n")?;
                }
                Stmt::EmptyLine(n) => {
                    for _ in 0..*n {
                        sink.body.write(b"\n")?;
                    }
                }
                Stmt::Break => {
                    sink.body.write(b"break;\n")?;
                }
                _ => {
                    self.stmt(stmt, sink)?;
                    sink.body.write(b"\n")?;
                }
            }
        }
        sink.record();
        Ok(())
    }

    // Is statement (pattern matching)
    /// Write match arm body inline into a generic Write (for is-as-expression).
    fn write_body_inline(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        if body.stmts.len() == 1 {
            match &body.stmts[0] {
                Stmt::Expr(expr) => {
                    self.expr(expr, out)?;
                    // Auto-coerce &str literal to String in String-returning match arms
                    if self.ret_type_needs_string_coercion()
                        && self.expr_needs_string_coercion(expr)
                    {
                        write!(out, ".to_string()")?;
                    }
                }
                Stmt::Return(ret) => {
                    self.write_return_expr(ret, out, false)?;
                }
                _ => write!(out, "{{ }}")?,
            }
        } else if body.stmts.is_empty() {
            write!(out, "{{}}")?;
        } else {
            write!(out, "{{ ")?;
            for stmt in &body.stmts {
                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, out)?;
                        write!(out, "; ")?;
                    }
                    Stmt::Return(ret) => {
                        self.write_return_expr(ret, out, true)?;
                        write!(out, " ")?;
                    }
                    Stmt::Break => write!(out, "break; ")?,
                    Stmt::Continue => write!(out, "continue; ")?,
                    _ => {}
                }
            }
            write!(out, "}}")?;
        }
        Ok(())
    }

    /// Plan 032 G2-core: is-arm body with mixed-arm normalization — when any
    /// sibling arm is a multi-stmt block, a single value-bearing Expr arm
    /// must be wrapped as `{ expr; }` (statement-position match, E0308
    /// otherwise). Value-bearing = a METHOD CALL (`m.insert(...)` returns
    /// Option, `tx.send(...)` …); println-style free-fn calls and literals
    /// are unit/common and stay inline (wrapping them only churns goldens).
    /// Return arms diverge and stay inline.
    fn write_is_arm_body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
        has_block_arm: bool,
    ) -> AutoResult<()> {
        let is_value_bearing_method_call = matches!(body.stmts.first(),
            Some(Stmt::Expr(Expr::Call(c)))
                if matches!(c.name.as_ref(),
                    Expr::Dot(_, _) | Expr::Bina(_, Op::Dot, _)));
        if has_block_arm && body.stmts.len() == 1 && is_value_bearing_method_call {
            sink.body.write(b"{ ")?;
            self.write_match_arm_body(body, sink)?;
            sink.body.write(b"; }")?;
            Ok(())
        } else {
            self.write_match_arm_body(body, sink)
        }
    }

    /// Write a match arm body: single expression inline, or block for multiple statements
    fn write_match_arm_body(&mut self, body: &Body, sink: &mut Sink) -> AutoResult<()> {
        if body.stmts.is_empty() {
            sink.body.write(b"{}")?;
        } else if body.stmts.len() == 1 {
            // Single statement: write inline
            match &body.stmts[0] {
                Stmt::Expr(expr) => {
                    self.expr(expr, &mut sink.body)?;
                    // Auto-coerce &str literal to String in String-returning match arms
                    if self.ret_type_needs_string_coercion()
                        && self.expr_needs_string_coercion(expr)
                    {
                        sink.body.write(b".to_string()")?;
                    }
                }
                Stmt::Return(ret) => {
                    self.write_return_expr(ret, &mut sink.body, false)?;
                }
                _ => {
                    // For other statement types, use a block
                    sink.body.write(b"{\n")?;
                    self.indent();
                    for stmt in &body.stmts {
                        self.print_indent(&mut sink.body)?;
                        self.stmt(stmt, sink)?;
                        if matches!(stmt, Stmt::Expr(_)) {
                            sink.body.write(b";")?;
                        }
                        sink.body.write(b"\n")?;
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
        } else {
            // Multiple statements: use a block
            sink.body.write(b"{\n")?;
            self.indent();
            for stmt in &body.stmts {
                self.print_indent(&mut sink.body)?;
                self.stmt(stmt, sink)?;
                if matches!(stmt, Stmt::Expr(_)) {
                    sink.body.write(b";")?;
                }
                sink.body.write(b"\n")?;
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}")?;
        }
        Ok(())
    }

    /// Plan 380: true when an `is`-match scrutinee is a call whose `Some(x)`
    /// binding is `&str` (strip_prefix / strip_suffix / to_str → Option<&str>).
    /// Used to record bound vars as StrSlice so call-site auto-borrows don't
    /// append `.as_str()` (E0658 str_as_str).
    /// Plan 396 §2.2: true when the is-scrutinee yields a by-value iterator
    /// (only fs.read_dir today — std::fs::ReadDir has no &IntoIterator impl).
    fn is_by_value_iter_scrutinee(target: &Expr) -> bool {
        if let Expr::Call(call) = target {
            if let Expr::Dot(obj, method) = call.name.as_ref() {
                if let Expr::Ident(m) = obj.as_ref() {
                    return m.as_str() == "fs" && method.as_str() == "read_dir";
                }
            }
        }
        false
    }

    fn is_str_returning_scrutinee(target: &Expr) -> bool {
        if let Expr::Call(call) = target {
            let m = match call.name.as_ref() {
                Expr::Dot(_, method) => Some(method.as_str()),
                Expr::Ident(n) => Some(n.as_str()),
                _ => None,
            };
            matches!(
                m,
                Some("strip_prefix") | Some("strip_suffix") | Some("to_str")
            )
        } else {
            false
        }
    }

    /// Plan 032 G2-core: the is-scrutinee is `<expr>.get(...)` — Rust's
    /// Map/Vec/Value `get` returns `Option<&V>`, so a `Some(x)` arm binds
    /// `x: &V`. Covers both method-callee AST shapes (Dot and Bina-Dot).
    /// No-arg `.get()` (e.g. slice windows) is excluded — requires ≥1 arg.
    fn is_borrowing_get_scrutinee(target: &Expr) -> bool {
        if let Expr::Call(call) = target {
            if call.args.args.is_empty() {
                return false;
            }
            let m = match call.name.as_ref() {
                Expr::Dot(_, method) => Some(method.as_str()),
                Expr::Bina(_, Op::Dot, method) => {
                    if let Expr::Ident(m) = method.as_ref() {
                        Some(m.as_str())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            matches!(m, Some("get"))
        } else {
            false
        }
    }

    /// Plan 032 G2-core: collect the `Some(x)` pattern binding names of an
    /// is-arm's pattern list into `get_ref_bindings` (call only when the
    /// scrutinee passed `is_borrowing_get_scrutinee`). Three pattern shapes:
    /// OptionPattern (the parser's canonical Some(x)/None), Expr::Some, and a
    /// Call named Some.
    fn register_get_ref_bindings(&mut self, patterns: &[Expr]) {
        for pat in patterns {
            match pat {
                Expr::OptionPattern(oc) => {
                    if let Some(b) = &oc.binding {
                        self.get_ref_bindings.insert(b.clone());
                    }
                }
                Expr::Some(inner) => {
                    if let Expr::Ident(b) = inner.as_ref() {
                        self.get_ref_bindings.insert(b.clone());
                    }
                }
                Expr::Call(c)
                    if matches!(c.name.as_ref(),
                    Expr::Ident(n) if n.as_str() == "Some") =>
                {
                    if let Some(crate::ast::Arg::Pos(Expr::Ident(b))) = c.args.args.first() {
                        self.get_ref_bindings.insert(b.clone());
                    }
                }
                _ => {}
            }
        }
    }

    /// True when the is-scrutinee resolves to `Option<Spec>` / `Result<Spec>`
    /// (or a bare `Spec`) — `Some(x)` then binds `x` to a `Box<dyn Trait>`,
    /// which has no Clone impl (call-site auto-clone would be E0599).
    fn is_spec_returning_scrutinee(&self, target: &Expr) -> bool {
        if let Expr::Call(call) = target {
            let name = match call.name.as_ref() {
                Expr::Ident(n) => Some(n.as_str().to_string()),
                Expr::Dot(_, m) => Some(m.as_str().to_string()),
                _ => None,
            };
            if let Some(n) = name {
                return self
                    .fn_ret_types
                    .get(n.as_str())
                    .map(|t| {
                        match t {
                            Type::Spec(_) => true,
                            Type::Option(inner) | Type::Result(inner) => {
                                matches!(inner.as_ref(), Type::Spec(_))
                                // Plan 380: on the single-file CLI path a
                                // cross-module spec can be typed `User(Role)`
                                // (spec_decls knows it's a spec) rather than
                                // `Spec(Role)`.
                                || (if let Type::User(usr) = inner.as_ref() {
                                    self.spec_decls.contains_key(&usr.name)
                                } else { false })
                            }
                            _ => false,
                        }
                    })
                    .unwrap_or(false);
            }
        }
        false
    }

    /// Plan 016 Phase A A.4: detect `json.parse(x)` scrutinee expression.
    fn is_json_parse_scrutinee(&self, target: &Expr) -> bool {
        if let Expr::Call(call) = target {
            matches!(call.name.as_ref(),
                Expr::Dot(obj, m) if m.as_str() == "parse"
                    && matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "json" || n.as_str() == "Json"))
                || matches!(call.name.as_ref(),
                    Expr::Bina(obj, op, m) if matches!(op, Op::Dot)
                        && matches!(m.as_ref(), Expr::Ident(mm) if mm.as_str() == "parse")
                        && matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "json" || n.as_str() == "Json"))
        } else {
            false
        }
    }

    /// Plan 610 ⑥: CTypeDesc → extern-block / Symbol-fn Rust type.
    fn c_abi_ffi_type(ty: &auto_bindgen::manifest::CTypeDesc) -> String {
        use auto_bindgen::manifest::CTypeDesc as T;
        match ty {
            T::Void => "()".to_string(),
            T::Bool => "bool".to_string(),
            T::Char | T::CStr => "*const std::os::raw::c_char".to_string(),
            T::Int => "i32".to_string(),
            T::UInt => "u32".to_string(),
            T::Long => "i64".to_string(),
            T::ULong => "u64".to_string(),
            T::Size => "usize".to_string(),
            T::Float => "f32".to_string(),
            T::Double => "f64".to_string(),
            T::Ptr => "*const std::os::raw::c_void".to_string(),
            T::PtrMut => "*mut std::os::raw::c_void".to_string(),
            T::FnPtr { .. } => "auto_bindgen_unsupported_fnptr".to_string(),
        }
    }

    /// Plan 610 ⑥: the wrapper (Auto-side) parameter type for one manifest
    /// param when it differs from the ffi type (Auto int is i64; the ABI edge
    /// is i32 with a width cast at the raw call).
    fn c_abi_wrapper_param(ty: &auto_bindgen::manifest::CTypeDesc) -> Option<&'static str> {
        use auto_bindgen::manifest::CTypeDesc as T;
        match ty {
            T::Int => Some("i64"),
            T::Float => Some("f64"),
            T::CStr => Some("&str"),
            _ => None,
        }
    }

    /// PLAN-010 T2: is this expression already an Option/Result constructor
    /// shape (Some/None/Ok/Err)? Mirrors write_return_expr's
    /// already_option_shaped predicate — such exprs must never get an
    /// additional implicit `Some(...)` tail wrap.
    fn expr_is_option_shaped(expr: &Expr) -> bool {
        match expr {
            Expr::Some(_) | Expr::None | Expr::Ok(_) => true,
            Expr::Call(c) => matches!(c.name.as_ref(),
                Expr::Ident(n) if matches!(n.as_str(), "Some" | "Ok" | "None" | "Err")),
            Expr::Ident(n) => matches!(n.as_str(), "Some" | "None" | "Ok" | "Err"),
            _ => false,
        }
    }

    fn is_returnable(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => {
                match expr {
                    // Void function calls are not returnable
                    Expr::Call(call) => {
                        if let Expr::Ident(name) = call.name.as_ref() {
                            if name == "print" || name == "println" || name == "write" {
                                return false;
                            }
                        }
                        true
                    }
                    // Nil/Null are not valid return expressions
                    Expr::Nil | Expr::Null => false,
                    // All other expressions (literals, operators, etc.) are returnable
                    _ => true,
                }
            }
            // Node (struct constructor parsed as component) is returnable
            Stmt::Node(_) => true,
            // Is (match expression) is returnable
            Stmt::Is(_) => true,
            // Plan 417-E4 调查: a tail if WITH an else carries the fn's value
            // (`if c { a } else { b }` as implicit return) — returnable.
            Stmt::If(if_) => if_.else_.is_some(),
            // Return statement already provides a value — no tail expression needed
            Stmt::Return(_) => true,
            _ => false,
        }
    }

    /// Plan 373: Detect whether a method body mutates `self` (either by direct
    /// field assignment `self.x = ...` or by calling a mutating method on a
    /// self-field like `self.vec.push(...)`). If so, the method needs `&mut self`.
    /// Plan 373: Seed known external (handwritten-Rust) struct-variant enums.
    /// These are enums whose .at port uses tuple syntax (because AutoVM can't
    /// destructure struct variants) but whose real Rust definition uses struct
    /// variants. We register the field names so construction sites emit
    /// `Type::Variant { field: val }` instead of `Type::Variant(val)`.
    fn seed_known_struct_enum_variants(&mut self) {
        let known: &[(&str, &str, &[&str])] = &[
            ("ContentBlock", "Text", &["text"]),
            ("ContentBlock", "ToolUse", &["id", "name", "input"]),
            (
                "ContentBlock",
                "ToolResult",
                &["tool_use_id", "content", "is_error"],
            ),
            // Plan 018 §Phase 3.3: auto_atom::AtomError::InvalidType is a
            // struct variant ({ expected, found }). Register it so .at
            // construction `AtomError.InvalidType("String", format!(...))`
            // emits struct syntax `AtomError::InvalidType { expected: ..., found: ... }`
            // instead of tuple syntax (E0599 no such struct variant).
            ("AtomError", "InvalidType", &["expected", "found"]),
        ];
        for (type_name, variant_name, fields) in known {
            self.enum_struct_variants.insert(
                ((*type_name).into(), (*variant_name).into()),
                fields.iter().map(|f| f.to_string().into()).collect(),
            );
            // Also register the enum name so construction sites recognize it as
            // a tag/enum type (needed in single-file mode where the enum is
            // imported from another module and not in tag_types yet).
            self.tag_types.insert((*type_name).into());
            self.known_enum_names.insert((*type_name).into());
        }
    }

    /// Plan 384 A3: Load extern function signatures from a sidecar `.at` file
    /// pointed to by the `A2R_EXTERN_SIGS` env var. The sidecar contains only
    /// `fn` declarations (no bodies) — e.g. describing an `extern_impl.rs`
    /// glue layer. We parse it and register each fn's param types (including
    /// `@T` → `Type::Reference`) into `fn_param_types` / `fn_ret_types`, so
    /// call sites can borrow owned args (`&arg`) against `&T` params. Errors
    /// are non-fatal (best-effort): a bad sidecar is ignored with a warning.
    fn load_extern_sigs(&mut self) {
        let path = match std::env::var("A2R_EXTERN_SIGS") {
            Ok(p) if !p.is_empty() => p,
            _ => return,
        };
        let code = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[a2r] warning: A2R_EXTERN_SIGS read failed ({}): {}",
                    path, e
                );
                return;
            }
        };
        let mut parser = crate::parser::Parser::from(code.as_str());
        parser.set_dest(crate::parser::CompileDest::TransRust);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[a2r] warning: A2R_EXTERN_SIGS parse failed: {}", e);
                return;
            }
        };
        for stmt in &ast.stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                let param_types: Vec<Type> = fn_decl.params.iter().map(|p| p.ty.clone()).collect();
                self.fn_param_types
                    .insert(fn_decl.name.clone(), param_types.clone());
                if let Some(parent) = &fn_decl.parent {
                    let qualified: AutoStr = format!("{}.{}", parent, fn_decl.name).into();
                    self.fn_param_types.insert(qualified, param_types);
                }
                self.fn_ret_types
                    .insert(fn_decl.name.clone(), fn_decl.ret.clone());
            }
        }
    }

    /// Only transpiles dirty fragments, caches results in Database
    pub fn trans_incremental(
        &mut self,
        session: &mut crate::compile::CompileSession,
        file_id: crate::database::FileId,
    ) -> AutoResult<std::collections::HashMap<crate::database::FragId, String>> {
        use std::collections::HashMap;

        let db = session.db();

        // Get dirty fragments for the file
        let dirty_frags = {
            let db_read = db.read().unwrap();
            let all_frags = db_read.get_fragments_by_file(file_id);
            all_frags
                .into_iter()
                .filter(|frag| db_read.is_fragment_dirty(frag))
                .collect::<Vec<_>>()
        };

        let mut results = HashMap::new();

        for frag_id in dirty_frags {
            let frag_ast = {
                let db_read = db.read().unwrap();
                db_read.get_fragment(&frag_id)
            };

            if let Some(fn_ast) = frag_ast {
                // Transpile the function
                let mut sink = Sink::new(AutoStr::from(format!("{:?}", frag_id)));
                self.fn_decl(&fn_ast, &mut sink)?;
                let output = String::from_utf8(sink.done()?.to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;

                results.insert(frag_id.clone(), output);

                // Mark as transpiled
                db.write().unwrap().mark_transpiled(&frag_id);
            }
        }

        Ok(results)
    }

    // =========================================================================
    // Post-processing: text-level fixes applied after code generation
    // Replaces the fix_transpiled.py Python script for Group B patterns
    // =========================================================================

    /// Infer variable types for a single function body and populate local_var_types.
    fn infer_fn_body(&mut self, ctx: &mut crate::infer::InferenceContext, fn_decl: &Fn) {
        use crate::infer::stmt::check_body;

        // Push scope, bind params, run check_body (without popping), extract types.
        ctx.push_scope();
        for param in &fn_decl.params {
            let ty = if !matches!(param.ty, Type::Unknown) {
                param.ty.clone()
            } else {
                Type::Unknown
            };
            ctx.bind_var(param.name.clone(), ty);
        }
        ctx.current_ret = Some(fn_decl.ret.clone());

        // Run inference on the body (may push/pop inner scopes for if/for blocks,
        // but function-level locals persist in our pushed scope).
        let _ = check_body(ctx, &fn_decl.body);

        // Extract all variable types from the function-level scope.
        if let Some(scope) = ctx.scopes.last() {
            for (name, ty) in scope.iter() {
                if !matches!(ty, Type::Unknown) {
                    self.local_var_types.insert(name.clone(), ty.clone());
                }
            }
        }

        ctx.pop_scope();
    }
}

lazy_static::lazy_static! {
    static ref RUST_REGEX_CACHE: std::sync::Mutex<std::collections::HashMap<String, regex::Regex>> =
        std::sync::Mutex::new(std::collections::HashMap::new());

    // Plan 014 Layer 3: per-process counters for post_process fix_* triggers.
    // Gated by env A2R_FIX_COUNTS=1: at the end of post_process each fix that
    // actually rewrote the output is tallied here, then printed as
    // `[fix-count] <name>=<n>` lines so a multi-file transpile run can be
    // aggregated (quantifies how much a2r still papers over vs skill-corrected
    // source, plan 014 预期效果 3).
    static ref FIX_COUNTS: std::sync::Mutex<std::collections::HashMap<String, u64>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
}

fn cached_regex(pattern: &str) -> Option<regex::Regex> {
    let mut cache = RUST_REGEX_CACHE.lock().unwrap();
    if let Some(re) = cache.get(pattern) {
        return Some(re.clone());
    }
    match regex::Regex::new(pattern) {
        Ok(re) => {
            cache.insert(pattern.to_string(), re.clone());
            Some(re)
        }
        Err(_) => None,
    }
}

/// Helper: extract capture group 1 from all regex matches, return as HashSet.
fn regex_captures(content: &str, pattern: &str) -> std::collections::HashSet<String> {
    let mut result = std::collections::HashSet::new();
    if let Some(re) = cached_regex(pattern) {
        for caps in re.captures_iter(content) {
            if let Some(m) = caps.get(1) {
                result.insert(m.as_str().to_string());
            }
        }
    }
    result
}

/// Helper: extract capture group 1 from all regex matches, return as Vec.
fn regex_captures_vec(content: &str, pattern: &str) -> Vec<String> {
    let mut result = Vec::new();
    if let Some(re) = cached_regex(pattern) {
        for caps in re.captures_iter(content) {
            if let Some(m) = caps.get(1) {
                result.push(m.as_str().to_string());
            }
        }
    }
    result
}

impl Trans for RustTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Phase 1: Emit file header with a2r standard library (includes #![allow] pragma)
        self.emit_a2r_stdlib(&mut sink.body)?;

        // Plan 373: Seed known external (handwritten-Rust) struct-variant enums.
        // These enums are declared with tuple syntax in .at (because AutoVM can't
        // destructure struct variants), but the real Rust enum they link against
        // uses struct variants. We register them here so construction sites emit
        // struct syntax `Type::Variant { field: val, ... }` instead of tuple.
        self.seed_known_struct_enum_variants();

        // Plan 384 A3: Load extern function signatures from an optional sidecar
        // .at file (env A2R_EXTERN_SIGS). The sidecar contains only `fn`
        // declarations (no bodies) describing the glue-layer stubs (e.g.
        // extern_impl.rs) so that call sites can do reference-aware injection
        // (`&arg` for `@T` params). Loaded before any emission so all call
        // sites benefit.
        self.load_extern_sigs();

        // Plan 204 Phase 3: Pre-scan for !T / Result<T,E> return types to determine Err trait need
        for stmt in &ast.stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                if matches!(fn_decl.ret, Type::Result(_))
                    || matches!(&fn_decl.ret, Type::GenericInstance(inst) if inst.base_name == "Result")
                {
                    self.needs_err_trait = true;
                    break;
                }
            }
        }

        // Plan 013 (B1/BUG3): Pre-scan for ALL locally-declared type names
        // (struct/enum/tag/union) so expression-position construction is never
        // spuriously crate-prefixed. Must run before any emission so forward
        // references (a type used before its declaration) resolve correctly.
        for stmt in &ast.stmts {
            match stmt {
                Stmt::TypeDecl(td) => {
                    self.local_struct_types.insert(td.name.clone());
                }
                Stmt::EnumDecl(ed) => {
                    self.local_struct_types.insert(ed.name.clone());
                }
                Stmt::Tag(td) => {
                    self.local_struct_types.insert(td.name.clone());
                }
                Stmt::Union(u) => {
                    self.local_struct_types.insert(u.name.clone());
                }
                _ => {}
            }
        }

        // PLAN-009 T1 (F1, 006 S2): pre-scan explicit derive surfaces of
        // locally declared types and the shape info of auto-derived ones, so
        // the auto-derive restriction at the TypeDecl/EnumDecl handlers is
        // order-independent and propagates through reference chains
        // (Wrapper → Inner → Grid). See type_cmp_mask/auto_cmp_mask below.
        for stmt in &ast.stmts {
            let (name, attrs, field_types): (AutoStr, &Vec<AutoStr>, Vec<&Type>) = match stmt {
                Stmt::TypeDecl(td) => (
                    td.name.clone(),
                    &td.attrs,
                    td.members.iter().map(|m| &m.ty).collect(),
                ),
                Stmt::EnumDecl(ed) => {
                    let mut payload_types: Vec<&Type> = Vec::new();
                    if let EnumKind::Homogeneous { payload_type } = &ed.kind {
                        payload_types.push(payload_type);
                    }
                    for item in &ed.items {
                        if let Some(pt) = &item.payload_type {
                            payload_types.push(pt);
                        }
                        for pt in &item.payload_types {
                            payload_types.push(pt);
                        }
                        for f in &item.fields {
                            payload_types.push(&f.field_type);
                        }
                    }
                    (ed.name.clone(), &ed.attrs, payload_types)
                }
                _ => continue,
            };
            let traits = Self::derive_traits_from_attrs(attrs);
            if !traits.is_empty() {
                // Explicit #[derive(...)] surface: verbatim passthrough at
                // emission; here it becomes the constraint set for containers
                // referencing this type. (Attrs WITHOUT a derive(...) attr are
                // treated as auto-shaped: manual trait impls, if any, are not
                // visible to the transpiler.)
                self.explicit_derive_traits.insert(name, traits);
            } else {
                let base = Self::shape_base_cmp_mask(&field_types);
                let mut refs: Vec<AutoStr> = Vec::new();
                for ty in &field_types {
                    Self::collect_local_type_names(ty, &self.local_struct_types, &mut refs);
                }
                self.local_auto_shapes.insert(name, (base, refs));
            }
        }

        // Pre-scan all function signatures for auto-borrow/auto-clone at call sites
        // Without this, functions declared after their callers won't have param type info
        for stmt in &ast.stmts {
            match stmt {
                Stmt::Fn(fn_decl) => {
                    let str_param_flags: Vec<bool> = fn_decl
                        .params
                        .iter()
                        .map(|p| matches!(p.ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit))
                        .collect();
                    self.fn_str_param_indices
                        .insert(fn_decl.name.clone(), str_param_flags);

                    let struct_param_flags: Vec<bool> = fn_decl
                        .params
                        .iter()
                        .map(|p| !Self::is_copy_type(&p.ty))
                        .collect();
                    self.fn_struct_param_indices
                        .insert(fn_decl.name.clone(), struct_param_flags);

                    let int_param_flags: Vec<bool> = fn_decl
                        .params
                        .iter()
                        .map(|p| matches!(p.ty, Type::Int))
                        .collect();
                    self.fn_int_param_indices
                        .insert(fn_decl.name.clone(), int_param_flags);

                    // Plan 390 §11 Phase E (D-A): spec params need Box::new() at call
                    // sites — cache the flags in the prescan so callers declared *before*
                    // this fn still auto-box. (The emit-time insert at fn_decl emission
                    // only covers fns seen before the caller.)
                    let spec_param_flags: Vec<bool> = fn_decl
                        .params
                        .iter()
                        .map(|p| matches!(p.ty, Type::Spec(_)))
                        .collect();
                    self.fn_spec_param_indices
                        .insert(fn_decl.name.clone(), spec_param_flags);

                    let param_types: Vec<Type> =
                        fn_decl.params.iter().map(|p| p.ty.clone()).collect();
                    self.fn_param_types
                        .insert(fn_decl.name.clone(), param_types);

                    // Plan 389 R2: also cache the return type in the prescan so
                    // task state fields initialized from a fn reference
                    // (`cb = noop_event`) can infer their fn-pointer type even
                    // when the fn is declared *after* the task. (Emit-time
                    // registration below only covers fns seen before the task.)
                    self.fn_ret_types
                        .insert(fn_decl.name.clone(), fn_decl.ret.clone());

                    // C11 (Plan 018 §12 a2r-11): `mut p T` params are &mut refs —
                    // call sites must pass `&mut arg` (never arg.clone()). The
                    // emit-time registration (below) only covers fns emitted
                    // *before* the caller; prescan here makes call sites to
                    // fns declared *after* their caller (e.g. a helper fn at
                    // the bottom of the file) also inject &mut.
                    let mut_param_flags: Vec<bool> = fn_decl
                        .params
                        .iter()
                        .map(|p| p.mode == crate::ast::ParamMode::Mut)
                        .collect();
                    self.fn_mut_params
                        .insert(fn_decl.name.clone(), mut_param_flags);
                }
                Stmt::SpecDecl(spec_decl) => {
                    // Plan 310 Phase 0.3: Pre-scan spec methods so that delegation
                    // `impl Spec for Type` generation (type_decl) can look them up
                    // regardless of declaration order. Without this, delegations to a
                    // spec declared *after* the type would miss the trait impl.
                    self.spec_decls
                        .insert(spec_decl.name.clone(), spec_decl.methods.clone());
                }
                Stmt::TypeDecl(type_decl) => {
                    // Also scan methods inside type declarations
                    let type_name = &type_decl.name;
                    for fn_decl in &type_decl.methods {
                        let str_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| {
                                matches!(p.ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit)
                            })
                            .collect();
                        // Use qualified key "Type.method" to avoid cross-type overwrites
                        let qualified_key: AutoStr =
                            format!("{}.{}", type_name, fn_decl.name).into();
                        self.fn_str_param_indices
                            .insert(qualified_key.clone(), str_param_flags.clone());
                        // Also store unqualified for backward compat (last one wins)
                        self.fn_str_param_indices
                            .insert(fn_decl.name.clone(), str_param_flags);

                        let struct_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| !Self::is_copy_type(&p.ty))
                            .collect();
                        self.fn_struct_param_indices
                            .insert(qualified_key.clone(), struct_param_flags.clone());
                        self.fn_struct_param_indices
                            .insert(fn_decl.name.clone(), struct_param_flags);

                        let int_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| matches!(p.ty, Type::Int))
                            .collect();
                        self.fn_int_param_indices
                            .insert(qualified_key.clone(), int_param_flags.clone());
                        self.fn_int_param_indices
                            .insert(fn_decl.name.clone(), int_param_flags);

                        // Plan 390 §11 Phase E (D-A): spec params need Box::new() at
                        // call sites — mirror the str/struct/int qualified+unqualified
                        // key pattern so `r.register(t)` (Expr::Dot) resolves via the
                        // last-segment fallback (Fix B) to "Type.method".
                        let spec_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| matches!(p.ty, Type::Spec(_)))
                            .collect();
                        self.fn_spec_param_indices
                            .insert(qualified_key.clone(), spec_param_flags.clone());
                        self.fn_spec_param_indices
                            .insert(fn_decl.name.clone(), spec_param_flags);

                        let param_types: Vec<Type> =
                            fn_decl.params.iter().map(|p| p.ty.clone()).collect();
                        self.fn_param_types
                            .insert(qualified_key.clone(), param_types.clone());
                        self.fn_param_types
                            .insert(fn_decl.name.clone(), param_types);

                        // C11: same prescan for `mut p T` flags (see Stmt::Fn above).
                        let mut_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| p.mode == crate::ast::ParamMode::Mut)
                            .collect();
                        self.fn_mut_params
                            .insert(fn_decl.name.clone(), mut_param_flags.clone());
                        self.fn_mut_params
                            .insert(qualified_key.clone(), mut_param_flags);
                    }
                }
                Stmt::Ext(ext) => {
                    // Plan 390 §11 Phase E (D-A): ext-block methods (e.g.
                    // `ext ToolRegistry { fn register(tool Tool) }`) are a separate
                    // Stmt::Ext, NOT inside Stmt::TypeDecl.methods — so the
                    // TypeDecl prescan above misses them. Mirror the same
                    // param-flag scans here (str/struct/int/spec/param_types/mut),
                    // using ext.target as the qualified-key prefix, so call sites
                    // `r.register(t)` resolve the spec-param auto-box via the
                    // last-segment fallback (Fix B).
                    let type_name = &ext.target;
                    for fn_decl in &ext.methods {
                        let qualified_key: AutoStr =
                            format!("{}.{}", type_name, fn_decl.name).into();

                        let str_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| {
                                matches!(p.ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit)
                            })
                            .collect();
                        self.fn_str_param_indices
                            .insert(qualified_key.clone(), str_param_flags.clone());
                        self.fn_str_param_indices
                            .insert(fn_decl.name.clone(), str_param_flags);

                        let struct_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| !Self::is_copy_type(&p.ty))
                            .collect();
                        self.fn_struct_param_indices
                            .insert(qualified_key.clone(), struct_param_flags.clone());
                        self.fn_struct_param_indices
                            .insert(fn_decl.name.clone(), struct_param_flags);

                        let int_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| matches!(p.ty, Type::Int))
                            .collect();
                        self.fn_int_param_indices
                            .insert(qualified_key.clone(), int_param_flags.clone());
                        self.fn_int_param_indices
                            .insert(fn_decl.name.clone(), int_param_flags);

                        let spec_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| matches!(p.ty, Type::Spec(_)))
                            .collect();
                        self.fn_spec_param_indices
                            .insert(qualified_key.clone(), spec_param_flags.clone());
                        self.fn_spec_param_indices
                            .insert(fn_decl.name.clone(), spec_param_flags);

                        let param_types: Vec<Type> =
                            fn_decl.params.iter().map(|p| p.ty.clone()).collect();
                        self.fn_param_types
                            .insert(qualified_key.clone(), param_types.clone());
                        self.fn_param_types
                            .insert(fn_decl.name.clone(), param_types);

                        let mut_param_flags: Vec<bool> = fn_decl
                            .params
                            .iter()
                            .map(|p| p.mode == crate::ast::ParamMode::Mut)
                            .collect();
                        self.fn_mut_params
                            .insert(fn_decl.name.clone(), mut_param_flags.clone());
                        self.fn_mut_params
                            .insert(qualified_key.clone(), mut_param_flags);
                    }
                }
                _ => {}
            }
        }

        // No custom Err trait — use Box<dyn std::error::Error> for !T error types

        // Plan 376 Pass 1: Pre-scan struct field types for assignment-time
        // type conversion (e.g., self.field = Some(&str) → Some(&str.to_string())).
        for stmt in &ast.stmts {
            if let Stmt::TypeDecl(td) = stmt {
                let fields: Vec<(AutoStr, Type)> = td
                    .members
                    .iter()
                    .map(|m| (m.name.clone(), m.ty.clone()))
                    .collect();
                if !fields.is_empty() {
                    self.struct_field_types.insert(td.name.clone(), fields);
                }
                // Plan 380: pre-register method return types so the trim-void
                // check (and other ret-type lookups) work regardless of
                // declaration order — e.g. `Memory.trim() void` must suppress
                // the str-trim `.to_string()` suffix even when a call to it
                // appears before its declaration (E0599 `()` Display).
                for method in &td.methods {
                    self.fn_ret_types
                        .insert(method.name.clone(), method.ret.clone());
                    let qualified: AutoStr = format!("{}.{}", td.name, method.name).into();
                    self.fn_ret_types.insert(qualified, method.ret.clone());
                }
            }
        }

        // Plan 376 Phase 2: Expression type inference pass.
        // For each function, run Auto's inference engine to determine the types
        // of local variables (including those inferred from expressions), then
        // populate local_var_types so codegen can make type-aware decisions.
        self.run_type_inference(&ast.stmts);

        // Phase 2: Split into declarations and main, preserving source line info
        self.global_lazy_used = false; // Plan 523 H5
        let mut decls: Vec<(Stmt, usize)> = Vec::new(); // (stmt, source_line)
        let mut main: Vec<(Stmt, usize)> = Vec::new(); // (stmt, source_line)

        let source_lines = ast.source_lines;
        for (i, stmt) in ast.stmts.into_iter().enumerate() {
            let line = source_lines.get(i).copied().unwrap_or(0);
            // Plan 151 / Fix: top-level let must go into main(), not module scope
            if let Stmt::Store(store) = &stmt {
                if matches!(store.kind, StoreKind::Var)
                    || matches!(store.kind, StoreKind::Shared)
                    || matches!(store.kind, StoreKind::Const)
                {
                    if matches!(store.kind, StoreKind::Var)
                        || matches!(store.kind, StoreKind::Shared)
                    {
                        self.register_global_var_typed(store.name.clone(), store.ty.clone());
                        // Plan 523 H5: 非字面量初始化的全局保留 Lazy 形态。
                        if !Self::global_store_is_const_init(store) {
                            self.global_lazy_used = true;
                        }
                    }
                    decls.push((stmt, line));
                } else {
                    // let → goes into main()
                    main.push((stmt, line));
                }
            } else if stmt.is_decl() {
                decls.push((stmt, line));
            } else {
                match stmt {
                    Stmt::For(_) => main.push((stmt, line)),
                    Stmt::If(_) => main.push((stmt, line)),
                    Stmt::Expr(_) => main.push((stmt, line)),
                    Stmt::Break => main.push((stmt, line)),
                    Stmt::Use(use_stmt) => {
                        sink.set_source_line(line);
                        self.use_stmt(&use_stmt, &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Dep(dep) => {
                        // Record dep name so crate.func() → crate::func()
                        // Use separate set to avoid blocking use.rust import generation
                        self.dep_crates.insert(dep.name.clone());
                    }
                    Stmt::Fn(fn_decl) => {
                        // Plan 383: 收集顶层函数名，供 emit_borrow 识别函数引用。
                        self.function_names.insert(fn_decl.name.clone());
                    }
                    _ => {}
                }
            }
        }

        // Plan 151: Add once_cell imports if we have global variables.
        // Plan 523 H5: 字面量初始化的全局已改 std const static Mutex——
        // 仅当存在非字面量全局（Lazy 残留）时才引入 once_cell 依赖。
        if !self.global_vars.is_empty() {
            if self.global_lazy_used {
                sink.body.write(b"use once_cell::sync::Lazy;\n")?;
            }
            sink.body.write(b"use std::sync::Mutex;\n\n")?;
        }

        // Phase 3: Generate declarations
        for (i, (decl, line)) in decls.iter().enumerate() {
            sink.record();
            sink.set_source_line(*line);
            self.stmt(decl, sink)?;
            if i < decls.len() - 1 {
                // Add blank line between declarations
                // Check if we already end with a newline
                if sink.body.ends_with(b"\n") {
                    sink.body.write(b"\n")?;
                } else {
                    sink.body.write(b"\n\n")?;
                }
            }
        }
        sink.record();

        // Phase 4: Generate main function if needed
        if !main.is_empty() {
            if !decls.is_empty() {
                // Add blank line before main
                if sink.body.ends_with(b"\n") {
                    sink.body.write(b"\n")?;
                } else {
                    sink.body.write(b"\n\n")?;
                }
            }

            // Plan 163: Check for async (await) and generate #[tokio::main] if needed.
            // Plan 364 Phase 8 F1: also treat for-over-Stream as async, since the
            // rewrite injects `.next().await` (the static has_await_refs can't see it
            // because the .await is injected at transpile time, not present in the AST).
            // Plan 387: this Phase-4 path only runs when there is NO explicit
            // `fn main()` in source (main is synthesized here). Programs WITH an
            // explicit `fn main()` go through fn_decl() instead (Stmt::Fn → Phase 3),
            // which has its own actor-main handling at the `is_main_actor` branch.
            // Actor programs normally have an explicit main (they need it to spawn
            // and send), so the actor case below is rarely hit — but kept for
            // completeness. Both paths use multi_thread (NOT current_thread):
            // current_thread deadlocks when run_to_completion().await joins an actor
            // awaiting a sender drop that only happens after main returns. See the
            // detailed rationale in fn_decl()'s is_main_actor branch.
            let is_async = {
                let refs: Vec<&Stmt> = main.iter().map(|(s, _)| s).collect();
                self.program_has_actors
                    || Self::has_await_refs(&refs)
                    || self.body_has_stream_for(&refs)
            };
            if self.program_has_actors {
                sink.body.write(b"#[tokio::main]\n")?;
                sink.body.write(b"async fn main() {\n")?;
            } else if is_async {
                sink.body.write(b"#[tokio::main]\n")?;
                sink.body.write(b"async fn main() {\n")?;
            } else {
                sink.body.write(b"fn main() {\n")?;
            }
            self.indent();

            // Plan 387 §16: no `__rt` binding — spawn helpers are parameterless
            // (track_join uses a thread-local registry). Nothing to inject here.

            for (stmt, line) in main.iter() {
                sink.record();
                sink.set_source_line(*line);
                self.print_indent(&mut sink.body)?;

                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        match stmt {
                            Stmt::Store(_) => {
                                sink.body.write(b";\n")?;
                            }
                            _ => {}
                        }
                    }
                }
            }
            sink.record();

            // Plan 387 §16: let in-flight actor messages process before exit.
            // drain_all yields so already-sent messages get processed; mailboxes
            // close naturally when TaskRefs drop at end of main.
            if self.program_has_actors {
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"a2r_std::task::drain_all().await;\n")?;
            }

            self.dedent();
            sink.body.write(b"}\n")?;
        }

        // PLAN-010 T2 (009 ledger 12_specs Arc 族): products referencing std
        // smart pointers (Arc/Mutex/Rc/RefCell/Cell) must resolve standalone —
        // inject the prelude imports once at file header. Skipped per line when
        // a use.rs import already brings the same name in (E0252).
        if !self.merge_mode {
            let text = String::from_utf8_lossy(&sink.body);
            // Text-level duplicate guard: the body may already carry explicit
            // or brace-grouped std imports emitted from use.rs statements or
            // global-var codegen (which don't register in self.uses).
            let has_text_use = |path: &str, names: &[&str]| {
                if text.contains(&format!("use {}::{{", path)) {
                    return true;
                }
                names
                    .iter()
                    .any(|n| text.contains(&format!("use {}::{};", path, n)))
            };
            let mut imports = String::new();
            if (text.contains("Arc") || text.contains("Mutex::"))
                && !has_text_use("std::sync", &["Arc", "Mutex"])
            {
                imports.push_str("#[allow(unused_imports)]\nuse std::sync::{Arc, Mutex};\n");
            }
            if text.contains("Rc::") || text.contains("Rc<") {
                if !has_text_use("std::rc", &["Rc"]) {
                    imports.push_str("#[allow(unused_imports)]\nuse std::rc::Rc;\n");
                }
            }
            if text.contains("RefCell") || text.contains("Cell::") || text.contains("Cell<") {
                if !has_text_use("std::cell", &["RefCell", "Cell"]) {
                    imports.push_str("#[allow(unused_imports)]\nuse std::cell::{RefCell, Cell};\n");
                }
            }
            if !imports.is_empty() {
                let import = imports.into_bytes();
                let body = &sink.body;
                let mut insert_pos = 0;
                for (i, line) in body.split(|&b| b == b'\n').enumerate() {
                    insert_pos += line.len() + 1;
                    if line.is_empty() && i > 0 {
                        break;
                    }
                }
                let mut new_body = Vec::with_capacity(sink.body.len() + import.len());
                new_body.extend_from_slice(&sink.body[..insert_pos]);
                new_body.extend_from_slice(&import);
                new_body.extend_from_slice(&sink.body[insert_pos..]);
                sink.body = new_body;
            }
        }

        // Add final newline only if not already ending with one
        if !sink.body.is_empty() && !sink.body.ends_with(b"\n") {
            sink.body.write(b"\n")?;
        }

        // Plan 270: Insert a2r_std import at file header if any a2r_std symbols were used.
        // Must be done AFTER all transpilation so a2r_std_used is accurate.
        // Plan 013 (B11): the import path depends on the output target. In
        // merge_mode (transpiling auto_lang's own sources) the runtime lives at
        // `auto_lang::a2r_std`. For standalone CLI output (the common case —
        // e.g. plan 013's ported crates), emit the bare `a2r_std` path so the
        // generated crate only needs `a2r-std` as a dependency, not all of
        // auto_lang.
        if !self.merge_mode && self.a2r_std_used.get() {
            // Plan 396/B3 (2026-08-20): emit the BARE `a2r_std` path again.
            // Plan 405 qualified this with `auto_lang::` claiming no standalone
            // a2r-std crate exists — but crates/a2r-std does exist and is the
            // declared dependency of external consumers (auto-ai's transpiled
            // crates path-dep it), so the qualified form broke them, and it
            // also invalidated 48 golden headers. api_gen backends (which dep
            // auto-lang instead) re-qualify the preamble in post-processing
            // (see auto-man api_gen's qualify_a2r_std).
            let import = b"// a2r Standard Library (from crate)\n#[allow(unused_imports)]\nuse a2r_std;\nuse a2r_std::*;\n\n";
            // Find the header boundary: after "#![allow]" line + blank line
            let body = &sink.body;
            let mut insert_pos = 0;
            for (i, line) in body.split(|&b| b == b'\n').enumerate() {
                insert_pos += line.len() + 1;
                if line.is_empty() && i > 0 {
                    break;
                }
            }
            let mut new_body = Vec::with_capacity(sink.body.len() + import.len());
            new_body.extend_from_slice(&sink.body[..insert_pos]);
            new_body.extend_from_slice(import);
            new_body.extend_from_slice(&sink.body[insert_pos..]);
            sink.body = new_body;
        }

        Ok(())
    }
}

/// Transpile AutoLang code to Rust
pub fn transpile_rust(name: impl Into<AutoStr>, code: &str) -> AutoResult<Sink> {
    transpile_rust_with_siblings(name, code, None)
}

/// Plan 610 ⑥: single-file transpile with a source directory — relative
/// `use.c "<file>.json"` manifest paths resolve against it first (then CWD).
pub fn transpile_rust_with_source_dir(
    source_dir: impl AsRef<std::path::Path>,
    name: impl Into<AutoStr>,
    code: &str,
) -> AutoResult<Sink> {
    transpile_rust_full(name, code, None, Some(source_dir.as_ref().to_path_buf()))
}

/// Plan 376D: Transpile with an optional sibling TypeStore.
/// When `sibling_store` is Some, it contains type declarations from ALL sibling
/// .at files in the same crate, enabling cross-module type inference.
pub fn transpile_rust_with_siblings(
    name: impl Into<AutoStr>,
    code: &str,
    sibling_store: Option<Arc<std::sync::RwLock<crate::types::TypeStore>>>,
) -> AutoResult<Sink> {
    transpile_rust_full(name, code, sibling_store, None)
}

/// Plan 610 ⑥: full single-file entry — sibling store plus an optional
/// source directory (relative use.c JSON manifests resolve against it).
pub fn transpile_rust_full(
    name: impl Into<AutoStr>,
    code: &str,
    sibling_store: Option<Arc<std::sync::RwLock<crate::types::TypeStore>>>,
    source_dir: Option<std::path::PathBuf>,
) -> AutoResult<Sink> {
    transpile_rust_full_with_options(name, code, sibling_store, source_dir, None, false)
        .map(|(sink, _)| sink)
}

/// PLAN-738 T-11：全文件转译 + 引用闭包证明与运行形态选择。生成服务的
/// back 模块（db.at/伴生 *.at）经 `qualify_a2r_std` 链接内嵌镜像——其核心
/// producer 引用按 Embedded 运行形态收集进 manifest，不再丢弃。
pub fn transpile_rust_full_with_options(
    name: impl Into<AutoStr>,
    code: &str,
    sibling_store: Option<Arc<std::sync::RwLock<crate::types::TypeStore>>>,
    source_dir: Option<std::path::PathBuf>,
    assembly_runtime: Option<crate::stdlib_assembly::host::RustRuntime>,
    take_references: bool,
) -> AutoResult<(Sink, Vec<crate::stdlib_assembly::reference::ReferenceProof>)> {
    let name = name.into();
    let _scope = shared(crate::scope_manager::ScopeManager::new());
    let mut parser = Parser::from(code);
    parser.set_dest(crate::parser::CompileDest::TransRust);
    // Plan 773 F-08: keep parse/CTEE failures TYPED — flattening them via
    // `.to_string()` here turned every emit-path syntax error into a plain
    // `Msg`, which the stage classifier must then treat as infrastructure.
    // Display text is unchanged (the old `Msg` wrapped the same string).
    let mut ast = parser.parse()?;

    // Plan 095: Run CTEE to transform compile-time constructs
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    // Plan 310 Phase 1: Run escape analysis on every top-level function body.
    let mut escape_results: HashMap<AutoStr, crate::trans::escape::EscapeMap> = HashMap::new();
    {
        use crate::trans::escape::EscapeAnalyzer;
        for stmt in &ast.stmts {
            match stmt {
                crate::ast::Stmt::Fn(func) => {
                    let map = EscapeAnalyzer::analyze_fn(func);
                    escape_results.insert(func.name.clone(), map);
                }
                crate::ast::Stmt::TypeDecl(td) => {
                    for method in &td.methods {
                        let map = EscapeAnalyzer::analyze_fn(method);
                        let key = format!("{}.{}", td.name, method.name).into();
                        escape_results.insert(key, map);
                    }
                }
                _ => {}
            }
        }
    }

    let mut out = Sink::new(name.clone());
    let mut transpiler = RustTrans::new(name);
    transpiler.escape_results = escape_results;
    transpiler.source_dir = source_dir; // Plan 610 ⑥
                                        // Plan 433 A1: pre-register fn return types for the whole file so
                                        // `let x = fn()` bindings infer their type regardless of declaration
                                        // order (see scan_call_init_bindings).
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::Fn(f) = stmt {
            transpiler
                .fn_ret_types
                .insert(f.name.clone(), f.ret.clone());
        }
    }
    // Plan 376D: Share sibling TypeStore for cross-module type inference.
    transpiler.shared_type_store = sibling_store;
    if let Some(runtime) = assembly_runtime {
        transpiler.assembly_runtime = runtime;
    }
    // Plan 013 (B1/BUG3): local-type pre-scan lives in trans() so all entry
    // points (single-file, project, CLI) benefit uniformly.
    transpiler.trans(ast, &mut out)?;

    // Apply post-processing fixes (replaces fix_transpiled.py)
    RustTrans::post_process_with(&mut out.body, &transpiler.ord_restricted_names);

    let references = if take_references {
        std::mem::take(&mut transpiler.assembly_references)
    } else {
        Vec::new()
    };
    Ok((out, references))
}

/// Plan 310 Phase 1: Run escape analysis on source and return a summary
/// (function name → number of tracked bindings). For verification that the
/// Plan musk-022 CRUD 智能扩展: transpile_fn turns a single Fn AST into Rust.
#[test]
fn test_transpile_fn_single_function() {
    use crate::parser::Parser;
    let code = r#"
pub type Note = { id: int, title: str }
pub fn create_note(title str) Note {
    let n = Note { id: 1, title: title }
    return n
}
"#;
    let mut parser = Parser::from(code);
    let ast = parser.parse().unwrap();
    // 取第一个 Stmt::Fn
    let fn_ast = ast
        .stmts
        .iter()
        .find_map(|st| match st {
            crate::ast::Stmt::Fn(f) => Some(f.clone()),
            _ => None,
        })
        .expect("no fn found");
    let mut t = RustTrans::new(AutoStr::from("test"));
    // 预注册 Note 类型（字段名 + 类型），让构造体能解析
    t.register_type("Note", vec![("id", Type::Int), ("title", Type::StrOwned)]);
    let out = t.transpile_fn(&fn_ast).unwrap();
    assert!(
        out.contains("fn create_note"),
        "transpile_fn output must contain fn: {}",
        out
    );
}

/// Plan 399 Phase 11.2: struct field `Type { title: title }` (Arg::Pair with a
/// same-name &str param) must add `.to_string()` when the field is a String type.
/// This is the form db.at actually uses (the Arg::Name shorthand `Type { title }`
/// is rarer). Verifies a2r's struct_field_types lookup drives the to_string.
#[test]
fn test_struct_pair_field_to_string() {
    use crate::parser::Parser;
    let code = r#"
pub type Note = { id: int, title: str }
pub fn make(title str) Note {
    return Note { id: 1, title: title }
}
"#;
    let mut parser = Parser::from(code);
    let ast = parser.parse().unwrap();
    let fn_ast = ast
        .stmts
        .iter()
        .find_map(|st| match st {
            crate::ast::Stmt::Fn(f) => Some(f.clone()),
            _ => None,
        })
        .expect("no fn found");
    let mut t = RustTrans::new(AutoStr::from("test"));
    t.register_type("Note", vec![("id", Type::Int), ("title", Type::StrOwned)]);
    let out = t.transpile_fn(&fn_ast).unwrap();
    assert!(
        out.contains("title: title.to_string()"),
        "Pair String field gets .to_string(): {}",
        out
    );
}

/// analysis pass actually runs in the transpile pipeline. Not used by the
/// transpiler output path.
/// PLAN-814 R3-05：跨 crate 测试支撑面（auto-lang a2r_tests 消费）——
/// 恒编译 pub（原同 crate cfg(test) pub(crate) 形）。
pub fn escape_analysis_summary(code: &str) -> std::collections::HashMap<AutoStr, usize> {
    use crate::trans::escape::EscapeAnalyzer;
    let mut parser = Parser::from(code);
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(_) => return HashMap::new(),
    };
    let mut summary = HashMap::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::Fn(func) = stmt {
            let map = EscapeAnalyzer::analyze_fn(func);
            summary.insert(func.name.clone(), map.len());
        }
    }
    summary
}
pub fn transpile_part(code: &str) -> AutoResult<AutoStr> {
    let _scope = shared(crate::scope_manager::ScopeManager::new());
    let mut parser = Parser::from(code);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new(AutoStr::from(""));
    let mut transpiler = RustTrans::new("part".into());
    transpiler.trans(ast, &mut out)?;
    let src = out.done()?.clone();
    Ok(String::from_utf8(src).unwrap().into())
}

// =============================================================================
// Plan 167: Multi-file project transpilation
// =============================================================================

/// A module discovered during project scanning
#[allow(dead_code)]
struct ProjectModule {
    /// Module name (e.g., "db", "api", "api::handlers")
    name: String,
    /// Path to the .at source file
    source_path: std::path::PathBuf,
    /// Rust output file name (e.g., "db.rs", "api/mod.rs", "api/handlers.rs")
    output_name: String,
    /// Whether this is a directory module (mod.at)
    is_dir_module: bool,
    /// Import statements from this module
    uses: Vec<crate::ast::Use>,
}

/// Plan 364 W7: Render an AST `DepStmt` as a Cargo.toml `[dependencies]` line
/// (without the trailing newline). Precedence: path > git > crates.io.
///
/// - `dep foo(path: "../foo")`             → `foo = { path = "../foo" }`
/// - `dep foo(version: "1", features: ..)` → `foo = { version = "1", features = ["a"] }`
/// - `dep foo`                              → `foo = "*"`
fn render_cargo_dep(dep: &crate::ast::DepStmt) -> String {
    // Local path dependency — `foo = { path = "..." }` (+ optional features).
    if let Some(path) = &dep.path {
        if dep.features.is_empty() {
            return format!("{} = {{ path = \"{}\" }}", dep.name, escape_cargo_str(path));
        }
        let feats = dep
            .features
            .iter()
            .map(|f| format!("\"{}\"", escape_cargo_str(f)))
            .collect::<Vec<_>>()
            .join(", ");
        return format!(
            "{} = {{ path = \"{}\", features = [{}] }}",
            dep.name,
            escape_cargo_str(path),
            feats
        );
    }
    // Git dependency — `foo = { git = "...", branch = "..." }`.
    if let Some(git) = &dep.git {
        let mut spec = format!("{} = {{ git = \"{}\"", dep.name, escape_cargo_str(git));
        if let Some(git_ref) = &dep.git_ref {
            // Heuristic: branch/tag/rev all map to Cargo's `branch`/`tag`/`rev`.
            // The parser stores whichever was given under git_ref; we emit it
            // as `branch` (the most common case). A future refinement can track
            // which keyword was used.
            spec.push_str(&format!(", branch = \"{}\"", escape_cargo_str(git_ref)));
        }
        spec.push_str(" }");
        return spec;
    }
    // crates.io — version + optional features, or wildcard.
    if let Some(version) = &dep.version {
        if dep.features.is_empty() {
            return format!("{} = \"{}\"", dep.name, escape_cargo_str(version));
        }
        let feats = dep
            .features
            .iter()
            .map(|f| format!("\"{}\"", escape_cargo_str(f)))
            .collect::<Vec<_>>()
            .join(", ");
        return format!(
            "{} = {{ version = \"{}\", features = [{}] }}",
            dep.name,
            escape_cargo_str(version),
            feats
        );
    }
    // Bare `dep foo` — no options: wildcard version.
    if dep.features.is_empty() {
        return format!("{} = \"*\"", dep.name);
    }
    let feats = dep
        .features
        .iter()
        .map(|f| format!("\"{}\"", escape_cargo_str(f)))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{} = {{ version = \"*\", features = [{}] }}",
        dep.name, feats
    )
}

/// Escape a string for safe embedding in a Cargo.toml value (escape backslash
/// and double-quote). Path separators (/) are left intact.
fn escape_cargo_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Transpile a multi-file AutoLang project to Rust
///
/// Starting from an entry file, this function:
/// 1. Parses the entry file and discovers its module dependencies
/// 2. Recursively discovers and parses all module files
/// 3. Transpiles each module into its own .rs file
/// 4. Generates mod.rs from mod.at with pub mod declarations
///
/// Returns a HashMap mapping output filename to generated Rust code.
pub fn transpile_rust_project(
    entry_file: &str,
) -> AutoResult<std::collections::HashMap<String, (Vec<u8>, Vec<super::SourceMapEntry>)>> {
    use super::MultiSink;
    use crate::ast::Stmt;

    let entry_path = std::path::Path::new(entry_file);
    let entry_dir = entry_path
        .parent()
        .ok_or_else(|| AutoError::Msg("Entry file has no parent directory".into()))?;

    // Phase 1: Discover all modules
    let mut modules = Vec::new();
    let mut visited = std::collections::HashSet::new();
    discover_modules(entry_path, entry_dir, &mut modules, &mut visited)?;

    // Phase 1.5: Pre-register all type/enum declarations into shared TypeStore
    // This allows cross-file type references (e.g., Usage{...} in json_helpers.at
    // when Usage is defined in types.at) to be resolved during parsing.
    let shared_type_store = Arc::new(RwLock::new(TypeStore::new()));
    let mut all_enum_names: HashSet<AutoStr> = HashSet::new();
    // Plan 264: module name → set of type names defined in that module.
    // Used to translate Auto's `module.Type` → Rust's `crate::module::Type`.
    let mut module_types: HashMap<String, HashSet<String>> = HashMap::new();
    {
        let mut store = shared_type_store.write().unwrap();
        for module in &modules {
            let mod_name = if module.is_dir_module {
                module
                    .source_path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            } else {
                module
                    .source_path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            };
            // Ensure module exists in module_types even if it has no type declarations.
            // This is needed so `use relay.X` gets `crate::` prefix in other modules.
            module_types.entry(mod_name.clone()).or_default();
            let source = std::fs::read_to_string(&module.source_path).map_err(|e| {
                AutoError::Msg(format!(
                    "Failed to read {}: {}",
                    module.source_path.display(),
                    e
                ))
            })?;
            for line in source.lines() {
                let trimmed = line.trim();
                let (prefix, rest) = if trimmed.starts_with("pub type ") {
                    ("pub type ", &trimmed[9..])
                } else if trimmed.starts_with("pub enum ") {
                    ("pub enum ", &trimmed[9..])
                } else if trimmed.starts_with("pub spec ") {
                    ("pub spec ", &trimmed[9..])
                } else if trimmed.starts_with("type ") {
                    ("type ", &trimmed[5..])
                } else if trimmed.starts_with("enum ") {
                    ("enum ", &trimmed[5..])
                } else if trimmed.starts_with("spec ") {
                    ("spec ", &trimmed[5..])
                } else {
                    continue;
                };
                let after_prefix = rest;
                // Extract name (first token after prefix, possibly with generics)
                let name = if let Some(angle) = after_prefix.find('<') {
                    &after_prefix[..angle]
                } else if let Some(space) = after_prefix.find(' ') {
                    &after_prefix[..space]
                } else {
                    after_prefix
                };
                if name.is_empty() {
                    continue;
                }
                // Type names must start with uppercase
                if !name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    continue;
                }
                // Plan 264: record module → type name mapping
                module_types
                    .entry(mod_name.clone())
                    .or_default()
                    .insert(name.to_string());
                if prefix.contains("type ") {
                    let decl = TypeDecl::builtin(name);
                    store.register_type_decl(&decl);
                } else if prefix.contains("enum ") {
                    let enum_decl = EnumDecl {
                        name: name.into(),
                        items: Vec::new(),
                        kind: EnumKind::Heterogeneous {
                            generic_params: Vec::new(),
                            methods: Vec::new(),
                        },
                        doc: None,
                        is_pub: prefix.starts_with("pub"),
                        attrs: Vec::new(),
                    };
                    store.register_enum_decl(enum_decl);
                    all_enum_names.insert(AutoStr::from(name));
                } else if prefix.contains("spec ") {
                    // Plan 371 (defect A): pre-register specs so cross-module /
                    // out-of-order `use`s resolve to Type::Spec (→ Box<dyn X>)
                    // instead of the Type::User placeholder. Only the name is
                    // needed for lookup_type to pick the right branch; the full
                    // SpecDecl (with methods) is filled in during Phase 2 parse.
                    let spec_decl = SpecDecl::new(name.into(), Vec::new());
                    store.register_spec_decl(&spec_decl);
                }
            }
        }
    }

    // Phase 2: Parse each module
    let mut parsed_modules = Vec::new();
    for module in &modules {
        let source = std::fs::read_to_string(&module.source_path).map_err(|e| {
            AutoError::Msg(format!(
                "Failed to read {}: {}",
                module.source_path.display(),
                e
            ))
        })?;
        let _scope = shared(crate::scope_manager::ScopeManager::new());
        let mut parser = Parser::new_with_type_store(source.as_str(), shared_type_store.clone());
        parser.set_dest(crate::parser::CompileDest::TransRust);
        parser.skip_check = true; // Plan 167: skip type checking for multi-file mode
        let ast = parser.parse().map_err(|e| {
            AutoError::Msg(format!(
                "Parse error in {}: {}",
                module.source_path.display(),
                e.to_string()
            ))
        })?;
        parsed_modules.push((module, ast));
    }

    // Phase 2.5: Pre-scan all function signatures for cross-module param-type tracking
    let mut global_fn_str_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    let mut global_fn_struct_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    let mut global_fn_int_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    let mut global_fn_param_types: std::collections::HashMap<AutoStr, Vec<Type>> =
        std::collections::HashMap::new();
    // Plan 390 §11 Phase E: spec params need Box::new() at call sites; track them
    // cross-module so `r.register(t)` (incl. ext-block methods) auto-boxes.
    let mut global_fn_spec_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();

    // Plan 390 §11 Phase E: collect spec-param flags across Fn / TypeDecl methods /
    // Ext methods (mirrors collect_fn_str_params but for spec auto-boxing). Ext-block
    // methods are a separate Stmt::Ext not covered by the TypeDecl scan, so without
    // this `ext ToolRegistry { fn register(tool Tool) }` would never auto-box.

    // Helper: collect Fn declarations from statements, including methods inside TypeDecl

    // Helper: collect non-Copy and Int param flags for cross-module clone/cast tracking
    fn collect_fn_param_types(
        stmts: &[Stmt],
        type_name: &str,
        struct_map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
        int_map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
        _merge_mut_map: Option<&mut std::collections::HashMap<AutoStr, Vec<bool>>>,
        mut param_types_map: Option<&mut std::collections::HashMap<AutoStr, Vec<Type>>>,
    ) {
        let generic_methods = [
            "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter",
            "keys", "values", "clone", "new", "update", "delete", "find", "index",
        ];
        let process_fn =
            |fn_decl: &crate::ast::Fn,
             _tname: &str,
             target_struct: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
             target_int: &mut std::collections::HashMap<AutoStr, Vec<bool>>| {
                let struct_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| {
                        !matches!(
                            p.ty,
                            Type::Int
                                | Type::Uint
                                | Type::USize
                                | Type::I64
                                | Type::U64
                                | Type::Float
                                | Type::Double
                                | Type::Bool
                                | Type::Char
                                | Type::Byte
                                | Type::StrFixed(_)
                                | Type::StrOwned
                                | Type::StrSlice
                                | Type::CStrLit
                                | Type::Void
                                | Type::Unknown
                                | Type::Slice(_)
                                | Type::Array(_)
                                | Type::List(_)
                        )
                    })
                    .collect();
                let int_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::Int))
                    .collect();
                let has_struct = struct_flags.iter().any(|&b| b);
                let has_int = int_flags.iter().any(|&b| b);
                if has_struct || has_int {
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        if has_struct {
                            target_struct.insert(fn_decl.name.clone(), struct_flags.clone());
                        }
                        if has_int {
                            target_int.insert(fn_decl.name.clone(), int_flags.clone());
                        }
                    }
                    if !type_name.is_empty() || fn_decl.parent.is_some() {
                        let parent = fn_decl
                            .parent
                            .as_ref()
                            .map(|p: &crate::ast::Name| p.to_string())
                            .unwrap_or_else(|| type_name.to_string());
                        let qualified = format!("{}.{}", parent, fn_decl.name);
                        if has_struct {
                            target_struct.insert(AutoStr::from(&qualified), struct_flags);
                        }
                        if has_int {
                            target_int.insert(AutoStr::from(&qualified), int_flags);
                        }
                    }
                }
            };
        for stmt in stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                process_fn(fn_decl, type_name, struct_map, int_map);
                if let Some(ptm) = param_types_map.as_mut() {
                    let pt: Vec<Type> = fn_decl.params.iter().map(|p| p.ty.clone()).collect();
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        ptm.insert(fn_decl.name.clone(), pt.clone());
                    }
                    if !type_name.is_empty() || fn_decl.parent.is_some() {
                        let parent = fn_decl
                            .parent
                            .as_ref()
                            .map(|p: &crate::ast::Name| p.to_string())
                            .unwrap_or_else(|| type_name.to_string());
                        let qualified = format!("{}.{}", parent, fn_decl.name);
                        ptm.insert(AutoStr::from(&qualified), pt);
                    }
                }
            }
            if let Stmt::TypeDecl(type_decl) = stmt {
                let type_name_str = type_decl.name.to_string();
                for method in &type_decl.methods {
                    // Create a temporary FnDecl-like approach by using the method directly
                    let struct_flags: Vec<bool> = method
                        .params
                        .iter()
                        .map(|p| {
                            !matches!(
                                p.ty,
                                Type::Int
                                    | Type::Uint
                                    | Type::USize
                                    | Type::I64
                                    | Type::U64
                                    | Type::Float
                                    | Type::Double
                                    | Type::Bool
                                    | Type::Char
                                    | Type::Byte
                                    | Type::StrFixed(_)
                                    | Type::StrOwned
                                    | Type::StrSlice
                                    | Type::CStrLit
                                    | Type::Void
                                    | Type::Unknown
                                    | Type::Slice(_)
                                    | Type::Array(_)
                                    | Type::List(_)
                            )
                        })
                        .collect();
                    let int_flags: Vec<bool> = method
                        .params
                        .iter()
                        .map(|p| matches!(p.ty, Type::Int))
                        .collect();
                    let has_struct = struct_flags.iter().any(|&b| b);
                    let has_int = int_flags.iter().any(|&b| b);
                    if has_struct || has_int {
                        if !generic_methods.contains(&method.name.as_str()) {
                            if has_struct {
                                struct_map.insert(method.name.clone(), struct_flags.clone());
                            }
                            if has_int {
                                int_map.insert(method.name.clone(), int_flags.clone());
                            }
                        }
                        let qualified = format!("{}.{}", type_name_str, method.name);
                        if has_struct {
                            struct_map.insert(AutoStr::from(&qualified), struct_flags);
                        }
                        if has_int {
                            int_map.insert(AutoStr::from(&qualified), int_flags);
                        }
                    }
                    if let Some(ptm) = param_types_map.as_mut() {
                        let pt: Vec<Type> = method.params.iter().map(|p| p.ty.clone()).collect();
                        if !generic_methods.contains(&method.name.as_str()) {
                            ptm.insert(method.name.clone(), pt.clone());
                        }
                        let qualified = format!("{}.{}", type_name_str, method.name);
                        ptm.insert(AutoStr::from(&qualified), pt);
                    }
                }
            }
        }
    }

    for (_module, ast) in &parsed_modules {
        collect_fn_str_params(&ast.stmts, "", &mut global_fn_str_params);
        collect_fn_spec_params(&ast.stmts, "", &mut global_fn_spec_params);
        collect_fn_param_types(
            &ast.stmts,
            "",
            &mut global_fn_struct_params,
            &mut global_fn_int_params,
            None,
            Some(&mut global_fn_param_types),
        );
    }

    // Phase 3: Transpile each module into its own Sink
    let mut multi_sink = MultiSink::new();
    // PLAN-009 T1 (F1): restricted-name accumulator across per-module
    // transpilers (see drain inside the loop).
    let mut ord_restricted_acc: HashSet<AutoStr> = HashSet::new();
    for (module, ast) in &parsed_modules {
        let sink = multi_sink.add(&module.output_name);
        sink.source_file = module
            .source_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
        let mut transpiler = RustTrans::new(AutoStr::from(&module.output_name));
        // Plan 376D: Share the global TypeStore with the transpiler for type inference.
        transpiler.shared_type_store = Some(shared_type_store.clone());
        // Only emit #![allow] for crate root (first module), not submodules
        let is_first_module = module.source_path == modules[0].source_path;
        if is_first_module {
            transpiler.emit_allow_pragma = true;
        }

        // Plan 264: Pass module_types and current module name for path qualification
        transpiler.module_types = module_types.clone();
        let cur_mod_name = if module.is_dir_module {
            module
                .source_path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        } else {
            module
                .source_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        };
        transpiler.current_module_name = cur_mod_name.clone();

        // Pre-populate tag_types with all known enum names for Err boxing detection
        transpiler.tag_types = all_enum_names.clone();
        // Plan 447 ③-P1: cross-module enum names for the return-type
        // concrete-check (p_kind -> TokenKind must not become impl TokenKind)
        for en in &all_enum_names {
            transpiler.known_enum_names.insert(en.clone());
        }

        // Pre-populate fn_str_param_indices with cross-module function signatures
        for (name, flags) in &global_fn_str_params {
            if !transpiler.fn_str_param_indices.contains_key(name) {
                transpiler
                    .fn_str_param_indices
                    .insert(name.clone(), flags.clone());
            }
        }

        // Pre-populate fn_struct_param_indices and fn_int_param_indices for cross-module clone/cast
        for (name, flags) in &global_fn_struct_params {
            if !transpiler.fn_struct_param_indices.contains_key(name) {
                transpiler
                    .fn_struct_param_indices
                    .insert(name.clone(), flags.clone());
            }
        }
        for (name, flags) in &global_fn_int_params {
            if !transpiler.fn_int_param_indices.contains_key(name) {
                transpiler
                    .fn_int_param_indices
                    .insert(name.clone(), flags.clone());
            }
        }
        // Plan 390 §11 Phase E: Pre-populate fn_spec_param_indices cross-module so
        // spec-param auto-boxing (Box::new at call sites) works across module
        // boundaries and for ext-block methods (e.g. `ext ToolRegistry { register }`).
        for (name, flags) in &global_fn_spec_params {
            if !transpiler.fn_spec_param_indices.contains_key(name) {
                transpiler
                    .fn_spec_param_indices
                    .insert(name.clone(), flags.clone());
            }
        }
        // Pre-populate fn_param_types for cross-module type-aware call site generation
        for (name, ptypes) in &global_fn_param_types {
            if !transpiler.fn_param_types.contains_key(name) {
                transpiler
                    .fn_param_types
                    .insert(name.clone(), ptypes.clone());
            }
        }
        // Plan 376 Phase 1: Pre-populate fn_ret_types cross-module for .await insertion
        for (_other_mod, other_ast) in &parsed_modules {
            for stmt in &other_ast.stmts {
                if let Stmt::Fn(fn_decl) = stmt {
                    if !transpiler.fn_ret_types.contains_key(&fn_decl.name) {
                        transpiler
                            .fn_ret_types
                            .insert(fn_decl.name.clone(), fn_decl.ret.clone());
                    }
                    if let Some(parent) = &fn_decl.parent {
                        let qualified: AutoStr = format!("{}.{}", parent, fn_decl.name).into();
                        if !transpiler.fn_ret_types.contains_key(&qualified) {
                            transpiler
                                .fn_ret_types
                                .insert(qualified, fn_decl.ret.clone());
                        }
                    }
                }
                if let Stmt::TypeDecl(td) = stmt {
                    for method in &td.methods {
                        let qualified: AutoStr = format!("{}.{}", td.name, method.name).into();
                        if !transpiler.fn_ret_types.contains_key(&qualified) {
                            transpiler
                                .fn_ret_types
                                .insert(qualified, method.ret.clone());
                        }
                    }
                }
                if let Stmt::SpecDecl(spec_decl) = stmt {
                    // Plan 380: spec methods too (e.g. builtin_roles.load_builtin
                    // returns ?Role) — needed for the spec-bound-ident detection
                    // in is-scrutinees (Option<Spec> → Box<dyn Trait>).
                    for method in &spec_decl.methods {
                        let qualified: AutoStr =
                            format!("{}.{}", spec_decl.name, method.name).into();
                        if !transpiler.fn_ret_types.contains_key(&qualified) {
                            transpiler
                                .fn_ret_types
                                .insert(qualified, method.ret.clone());
                        }
                        if !transpiler.fn_ret_types.contains_key(&method.name) {
                            transpiler
                                .fn_ret_types
                                .insert(method.name.clone(), method.ret.clone());
                        }
                    }
                }
            }
        }

        // Pre-populate struct_fields from all modules for cross-file struct construction
        // Without this, struct fields in other files fall back to field0, field1, etc.
        for (_other_mod, other_ast) in &parsed_modules {
            for stmt in &other_ast.stmts {
                if let Stmt::TypeDecl(td) = stmt {
                    if !transpiler.struct_fields.contains_key(&td.name) {
                        let field_names: Vec<AutoStr> =
                            td.members.iter().map(|m| m.name.clone()).collect();
                        if !field_names.is_empty() {
                            transpiler
                                .struct_fields
                                .insert(td.name.clone(), field_names);
                        }
                        let field_types: Vec<(AutoStr, Type)> = td
                            .members
                            .iter()
                            .map(|m| (m.name.clone(), m.ty.clone()))
                            .collect();
                        if !field_types.is_empty() {
                            transpiler
                                .struct_field_types
                                .insert(td.name.clone(), field_types);
                        }
                        // PLAN-010 T2: delegation component fields for literal
                        // default-init (E0063 missing-field debt).
                        if !td.delegations.is_empty()
                            && !transpiler.struct_delegation_types.contains_key(&td.name)
                        {
                            transpiler.struct_delegation_types.insert(
                                td.name.clone(),
                                td.delegations
                                    .iter()
                                    .map(|d| (d.member_name.clone(), d.member_type.clone()))
                                    .collect(),
                            );
                        }
                    }
                }
            }
        }

        // Plan 167: Populate local_modules for mod declarations.
        // In Rust, `mod X;` can only appear in the parent module that owns X.
        // - crate root (main.rs): can use `mod X;` for all top-level modules
        // - dir module (mod.rs): pub mod X; emitted separately below
        // - other files: must use `use crate::X;` or `use super::X;`
        // We only populate local_modules for the crate root.
        let is_entry = module.source_path == modules[0].source_path;
        if is_entry {
            for other in &modules {
                if other.source_path == module.source_path {
                    continue;
                }
                let other_name = if other.is_dir_module {
                    other
                        .source_path
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                } else {
                    other
                        .source_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                };
                transpiler.local_modules.insert(other_name);
            }
        }
        // Non-entry modules: local_modules stays empty
        // → use X will be handled by is_multi_file_bare → use crate::X;

        // Mark directory modules and populate dir_children
        if module.is_dir_module {
            transpiler.is_dir_module = true;
            let mod_dir = module.source_path.parent().unwrap();
            // Collect from discovered modules
            for other in &modules {
                if other.source_path == module.source_path || other.is_dir_module {
                    continue;
                }
                let other_dir = other.source_path.parent().unwrap();
                if other_dir == mod_dir {
                    let other_name = other
                        .source_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    transpiler.dir_children.insert(other_name);
                }
            }
            // Also scan disk for .at files not yet discovered
            if let Ok(entries) = std::fs::read_dir(mod_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        if let Some(stem) = path.file_stem() {
                            let name = stem.to_string_lossy().to_string();
                            if name != "mod" {
                                transpiler.dir_children.insert(name);
                            }
                        }
                    }
                }
            } else {
                eprintln!("[DEBUG 264] WARNING: read_dir failed for {:?}", mod_dir);
            }
        }

        // Populate sibling_modules: modules in the same directory as the current module
        // Used to generate `use super::X;` for same-directory references.
        // Exclude directory modules (mod.rs) since their same-dir files are children, not siblings.
        if !is_entry && !module.is_dir_module {
            let module_dir = module.source_path.parent().unwrap();
            for other in &modules {
                if other.source_path == module.source_path {
                    continue;
                }
                let other_dir = other.source_path.parent().unwrap();
                if other_dir == module_dir {
                    let other_name = other
                        .source_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    transpiler.sibling_modules.insert(other_name);
                }
            }
        }

        // For directory modules (mod.at), emit pub mod declarations for discovered sibling files
        // Scan the actual directory on disk to ensure all sibling .at files are included,
        // even if discover_modules didn't find them via super.X paths.
        if module.is_dir_module {
            let mod_dir = module.source_path.parent().unwrap();
            let mut submodules: Vec<String> = Vec::new();
            // First: collect from discovered modules
            for other in &modules {
                if other.source_path == module.source_path {
                    continue;
                }
                let other_dir = other.source_path.parent().unwrap();
                if other_dir == mod_dir && !other.is_dir_module {
                    if let Some(name) = other.source_path.file_stem() {
                        submodules.push(name.to_string_lossy().to_string());
                    }
                }
            }
            // Then: scan disk for any .at files not yet discovered
            if let Ok(entries) = std::fs::read_dir(mod_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        if let Some(stem) = path.file_stem() {
                            let name = stem.to_string_lossy().to_string();
                            if name != "mod" && !submodules.contains(&name) {
                                submodules.push(name);
                            }
                        }
                    }
                }
            }
            submodules.sort();
            for sub in &submodules {
                let _ = write!(sink.body, "pub mod {};\n", sub);
            }
        }

        // Plan 167b: For entry file, emit mod X; declarations before transpilation
        // For dir modules (mod.at), the effective directory is the parent of mod.at's dir
        // In merge mode, skip mod declarations — all code goes into one file
        if is_entry && !transpiler.merge_mode {
            let entry_dir = module.source_path.parent().unwrap();
            let mut mod_names: Vec<String> = Vec::new();
            for other in &modules {
                if other.source_path == module.source_path {
                    continue;
                }
                let effective_dir = if other.is_dir_module {
                    // Dir module: mod.at is in runtime/, so effective dir is auto/
                    other.source_path.parent().unwrap().parent().unwrap()
                } else {
                    // File module: file is in auto/ or auto/tools/
                    other.source_path.parent().unwrap()
                };
                if effective_dir != entry_dir {
                    continue;
                }
                let other_name = if other.is_dir_module {
                    other
                        .source_path
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                } else {
                    other
                        .source_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                };
                mod_names.push(other_name);
            }
            mod_names.sort();
            for mn in &mod_names {
                let _ = write!(sink.body, "mod {};\n", mn);
            }
        }

        transpiler.trans(ast.clone(), sink)?;
        // PLAN-009 T1 (F1): transpiler is per-module here; drain its
        // restricted-name set into the function-level accumulator so the
        // post-processing loop below sees all modules' restrictions.
        ord_restricted_acc.extend(transpiler.ord_restricted_names.drain());
    }

    // Phase 3.4: Apply post-processing to each sink's body
    for (_, sink) in &mut multi_sink.files {
        RustTrans::post_process_with(&mut sink.body, &ord_restricted_acc);
    }

    // Phase 3.5: Generate Cargo.toml
    let mut result = std::collections::HashMap::new();
    {
        let project_name = entry_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("auto_project");
        // Sanitize: replace non-alphanumeric with underscore
        let project_name = project_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .to_lowercase();

        // Plan 328: Cargo package names can't start with a digit.
        let safe_name = if project_name
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            format!("app-{}", project_name)
        } else {
            project_name.to_string()
        };
        let mut cargo_toml = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            safe_name
        );

        // Scan parsed ASTs for external Rust crate imports (.rust use kind)
        // and lang-level `dep` declarations (Plan 364 W7).
        //
        // Two sources feed the dep list:
        //   1. `use.rust foo` — emits `foo = "*"` (bare name, version inferred).
        //   2. `dep foo(...)`  — emits a structured spec: `foo = { path = ".." }`
        //      / `foo = { version = "1", features = ["a"] }` / `foo = "*"`.
        //      A `dep` statement for a crate takes precedence over a bare
        //      `use.rust` for the same crate (the structured spec wins).
        let mut deps: Vec<String> = Vec::new();
        // Plan 364 W7: structured dep specs keyed by crate name, from `dep` stmts.
        let mut dep_specs: std::collections::HashMap<&str, &crate::ast::DepStmt> =
            std::collections::HashMap::new();
        // Plan 190: Rust built-in crates are always available, don't add to Cargo.toml
        let built_in_crates = ["std", "core", "alloc", "proc_macro"];
        for (_, ast) in &parsed_modules {
            for stmt in &ast.stmts {
                match stmt {
                    Stmt::Use(u) => {
                        if matches!(u.kind, UseKind::Rust) && !u.paths.is_empty() {
                            let crate_name = u.paths[0].as_str();
                            if !deps.contains(&crate_name.to_string())
                                && !built_in_crates.contains(&crate_name)
                            {
                                deps.push(crate_name.to_string());
                            }
                        }
                    }
                    // Plan 364 W7: collect structured dep specs (path/version/features/git).
                    Stmt::Dep(dep) => {
                        dep_specs.insert(dep.name.as_str(), dep);
                        // Ensure the name is in the dep list so it gets emitted.
                        if !deps.contains(&dep.name.to_string())
                            && !built_in_crates.contains(&dep.name.as_str())
                        {
                            deps.push(dep.name.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Note: external deps from hand-written .rs files are scanned by CargoBuilder::setup()
        if !deps.is_empty() {
            cargo_toml.push_str("\n[dependencies]\n");
            for dep in &deps {
                if let Some(spec) = dep_specs.get(dep.as_str()) {
                    // Plan 364 W7: render the structured dep spec.
                    cargo_toml.push_str(&format!("{}\n", render_cargo_dep(spec)));
                } else {
                    cargo_toml.push_str(&format!("{} = \"*\"\n", dep));
                }
            }
        }

        result.insert(
            "Cargo.toml".to_string(),
            (cargo_toml.into_bytes(), Vec::new()),
        );
    }

    // Phase 4: Collect results with per-file source maps
    let files = multi_sink.done_with_source_maps();
    for (name, content, source_map) in files {
        result.insert(name, (content, source_map));
    }

    Ok(result)
}

/// Transpile a multi-file AutoLang project into a single merged Rust file.
///
/// Similar to `transpile_rust_project` but outputs one .rs file with:
/// - All module code concatenated (no mod X; declarations)
/// - Deduplicated struct/enum/use definitions
/// - merge_mode = true to skip cross-module imports
/// - post_process_merged() for additional fixes
pub fn transpile_rust_project_merged(entry_file: &str) -> AutoResult<Vec<u8>> {
    use super::Sink;
    use crate::ast::Stmt;

    let entry_path = std::path::Path::new(entry_file);

    // Phase 1: Discover all modules
    // If entry is a directory, scan all .at files in it directly.
    // If entry is a file, use the standard discover_modules mechanism.
    let mut modules = Vec::new();
    if entry_path.is_dir() {
        // Directory mode: discover all .at files in the directory
        let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(entry_path)
            .map_err(|e| {
                AutoError::Msg(format!(
                    "Cannot read directory {}: {}",
                    entry_path.display(),
                    e
                ))
            })?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "at").unwrap_or(false))
            .collect();
        // Sort by dependency order (same order as merge.sh for consistency)
        let dep_order = [
            "pos",
            "error",
            "token",
            "opcode",
            "ast",
            "lexer",
            "parser",
            "typeinfer",
            "codegen",
            "vm",
            "a2r",
            "eval",
        ];
        entries.sort_by_key(|p| {
            let name = p
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            dep_order.iter().position(|&d| d == name).unwrap_or(999)
        });
        for path in &entries {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            modules.push(ProjectModule {
                name: name.clone(),
                source_path: path.clone(),
                output_name: name,
                is_dir_module: false,
                uses: Vec::new(),
            });
        }
    } else {
        let entry_dir = entry_path
            .parent()
            .ok_or_else(|| AutoError::Msg("Entry file has no parent directory".into()))?;
        let mut visited = std::collections::HashSet::new();
        discover_modules(entry_path, entry_dir, &mut modules, &mut visited)?;
    }

    if modules.is_empty() {
        return Err(AutoError::Msg("No modules found".into()));
    }

    // Phase 1.5: Pre-register all type/enum declarations into shared TypeStore
    let shared_type_store = Arc::new(RwLock::new(TypeStore::new()));
    let mut all_enum_names: HashSet<AutoStr> = HashSet::new();
    let mut module_types: HashMap<String, HashSet<String>> = HashMap::new();
    {
        let mut store = shared_type_store.write().unwrap();
        for module in &modules {
            let mod_name = if module.is_dir_module {
                module
                    .source_path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            } else {
                module
                    .source_path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            };
            module_types.entry(mod_name.clone()).or_default();
            let source = std::fs::read_to_string(&module.source_path).map_err(|e| {
                AutoError::Msg(format!(
                    "Failed to read {}: {}",
                    module.source_path.display(),
                    e
                ))
            })?;
            for line in source.lines() {
                let trimmed = line.trim();
                let (prefix, rest) = if trimmed.starts_with("pub type ") {
                    ("pub type ", &trimmed[9..])
                } else if trimmed.starts_with("pub enum ") {
                    ("pub enum ", &trimmed[9..])
                } else if trimmed.starts_with("pub spec ") {
                    ("pub spec ", &trimmed[9..])
                } else if trimmed.starts_with("type ") {
                    ("type ", &trimmed[5..])
                } else if trimmed.starts_with("enum ") {
                    ("enum ", &trimmed[5..])
                } else if trimmed.starts_with("spec ") {
                    ("spec ", &trimmed[5..])
                } else {
                    continue;
                };
                let after_prefix = rest;
                let name = if let Some(angle) = after_prefix.find('<') {
                    &after_prefix[..angle]
                } else if let Some(space) = after_prefix.find(' ') {
                    &after_prefix[..space]
                } else {
                    after_prefix
                };
                if name.is_empty()
                    || !name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                {
                    continue;
                }
                module_types
                    .entry(mod_name.clone())
                    .or_default()
                    .insert(name.to_string());
                if prefix.contains("type ") {
                    let decl = TypeDecl::builtin(name);
                    store.register_type_decl(&decl);
                } else if prefix.contains("enum ") {
                    let enum_decl = EnumDecl {
                        name: name.into(),
                        items: Vec::new(),
                        kind: EnumKind::Heterogeneous {
                            generic_params: Vec::new(),
                            methods: Vec::new(),
                        },
                        doc: None,
                        is_pub: prefix.starts_with("pub"),
                        attrs: Vec::new(),
                    };
                    store.register_enum_decl(enum_decl);
                    all_enum_names.insert(AutoStr::from(name));
                } else if prefix.contains("spec ") {
                    // Plan 371 (defect A): pre-register specs so cross-module /
                    // out-of-order `use`s resolve to Type::Spec (→ Box<dyn X>)
                    // instead of the Type::User placeholder. Only the name is
                    // needed for lookup_type to pick the right branch; the full
                    // SpecDecl (with methods) is filled in during Phase 2 parse.
                    let spec_decl = SpecDecl::new(name.into(), Vec::new());
                    store.register_spec_decl(&spec_decl);
                }
            }
        }
    }

    // Phase 2: Parse each module
    let mut parsed_modules = Vec::new();
    for module in &modules {
        let source = std::fs::read_to_string(&module.source_path).map_err(|e| {
            AutoError::Msg(format!(
                "Failed to read {}: {}",
                module.source_path.display(),
                e
            ))
        })?;
        let _scope = shared(crate::scope_manager::ScopeManager::new());
        let mut parser = Parser::new_with_type_store(source.as_str(), shared_type_store.clone());
        parser.set_dest(crate::parser::CompileDest::TransRust);
        parser.skip_check = true;
        let ast = parser.parse().map_err(|e| {
            AutoError::Msg(format!(
                "Parse error in {}: {}",
                module.source_path.display(),
                e.to_string()
            ))
        })?;
        parsed_modules.push((module, ast));
    }

    // Phase 2.5: Pre-scan all function signatures for cross-module param-type tracking
    let mut global_fn_str_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    let mut global_fn_struct_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    let mut global_fn_int_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    let mut global_fn_param_types: std::collections::HashMap<AutoStr, Vec<Type>> =
        std::collections::HashMap::new();
    let mut global_merge_mut_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    // Plan 390 §11 Phase E: spec params need Box::new() at call sites.
    let mut global_fn_spec_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();

    // Plan 390 §11 Phase E: spec-param collector (mirrors collect_fn_str_params
    // but for spec auto-boxing). Covers Fn / TypeDecl methods / Ext methods — the
    // last is a separate Stmt::Ext not reached by the TypeDecl scan.

    fn collect_fn_param_types(
        stmts: &[Stmt],
        type_name: &str,
        struct_map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
        int_map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
        mut merge_mut_map: Option<&mut std::collections::HashMap<AutoStr, Vec<bool>>>,
        mut param_types_map: Option<&mut std::collections::HashMap<AutoStr, Vec<Type>>>,
    ) {
        let generic_methods = [
            "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter",
            "keys", "values", "clone", "new", "update", "delete", "find", "index",
        ];
        for stmt in stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                let struct_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| {
                        !matches!(
                            p.ty,
                            Type::Int
                                | Type::Uint
                                | Type::USize
                                | Type::I64
                                | Type::U64
                                | Type::Float
                                | Type::Double
                                | Type::Bool
                                | Type::Char
                                | Type::Byte
                                | Type::StrFixed(_)
                                | Type::StrOwned
                                | Type::StrSlice
                                | Type::CStrLit
                                | Type::Void
                                | Type::Unknown
                                | Type::Slice(_)
                                | Type::Array(_)
                                | Type::List(_)
                        )
                    })
                    .collect();
                let int_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::Int))
                    .collect();
                let has_struct = struct_flags.iter().any(|&b| b);
                let has_int = int_flags.iter().any(|&b| b);
                if has_struct || has_int {
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        if has_struct {
                            struct_map.insert(fn_decl.name.clone(), struct_flags.clone());
                        }
                        if has_int {
                            int_map.insert(fn_decl.name.clone(), int_flags.clone());
                        }
                    }
                    if !type_name.is_empty() || fn_decl.parent.is_some() {
                        let parent = fn_decl
                            .parent
                            .as_ref()
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| type_name.to_string());
                        let qualified = format!("{}.{}", parent, fn_decl.name);
                        if has_struct {
                            struct_map.insert(AutoStr::from(&qualified), struct_flags);
                        }
                        if has_int {
                            int_map.insert(AutoStr::from(&qualified), int_flags);
                        }
                    }
                }
                // Pre-scan merge-mut params for correct call-site handling
                if let Some(ref mut mm) = merge_mut_map {
                    let merge_flags: Vec<bool> = fn_decl
                        .params
                        .iter()
                        .map(|p| RustTrans::is_merge_mut_type(&p.ty))
                        .collect();
                    if merge_flags.iter().any(|&b| b) {
                        mm.insert(fn_decl.name.clone(), merge_flags);
                    }
                }
                // Collect full param types for type-aware call site generation
                if let Some(ref mut ptm) = param_types_map {
                    let pt: Vec<Type> = fn_decl.params.iter().map(|p| p.ty.clone()).collect();
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        ptm.insert(fn_decl.name.clone(), pt.clone());
                    }
                    if !type_name.is_empty() || fn_decl.parent.is_some() {
                        let parent = fn_decl
                            .parent
                            .as_ref()
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| type_name.to_string());
                        let qualified = format!("{}.{}", parent, fn_decl.name);
                        ptm.insert(AutoStr::from(&qualified), pt);
                    }
                }
            }
            if let Stmt::TypeDecl(type_decl) = stmt {
                let type_name_str = type_decl.name.to_string();
                for method in &type_decl.methods {
                    let struct_flags: Vec<bool> = method
                        .params
                        .iter()
                        .map(|p| {
                            !matches!(
                                p.ty,
                                Type::Int
                                    | Type::Uint
                                    | Type::USize
                                    | Type::I64
                                    | Type::U64
                                    | Type::Float
                                    | Type::Double
                                    | Type::Bool
                                    | Type::Char
                                    | Type::Byte
                                    | Type::StrFixed(_)
                                    | Type::StrOwned
                                    | Type::StrSlice
                                    | Type::CStrLit
                                    | Type::Void
                                    | Type::Unknown
                                    | Type::Slice(_)
                                    | Type::Array(_)
                                    | Type::List(_)
                            )
                        })
                        .collect();
                    let int_flags: Vec<bool> = method
                        .params
                        .iter()
                        .map(|p| matches!(p.ty, Type::Int))
                        .collect();
                    let has_struct = struct_flags.iter().any(|&b| b);
                    let has_int = int_flags.iter().any(|&b| b);
                    if has_struct || has_int {
                        if !generic_methods.contains(&method.name.as_str()) {
                            if has_struct {
                                struct_map.insert(method.name.clone(), struct_flags.clone());
                            }
                            if has_int {
                                int_map.insert(method.name.clone(), int_flags.clone());
                            }
                        }
                        let qualified = format!("{}.{}", type_name_str, method.name);
                        if has_struct {
                            struct_map.insert(AutoStr::from(&qualified), struct_flags);
                        }
                        if has_int {
                            int_map.insert(AutoStr::from(&qualified), int_flags);
                        }
                    }
                    if let Some(ref mut ptm) = param_types_map {
                        let pt: Vec<Type> = method.params.iter().map(|p| p.ty.clone()).collect();
                        if !generic_methods.contains(&method.name.as_str()) {
                            ptm.insert(method.name.clone(), pt.clone());
                        }
                        let qualified = format!("{}.{}", type_name_str, method.name);
                        ptm.insert(AutoStr::from(&qualified), pt);
                    }
                }
            }
        }
    }

    // Phase 2.5b: Collect const names for merge mode
    let mut global_const_names: HashSet<AutoStr> = HashSet::new();
    for (_module, ast) in &parsed_modules {
        for stmt in &ast.stmts {
            if let Stmt::Store(store) = stmt {
                if matches!(store.kind, crate::ast::StoreKind::Const) {
                    global_const_names.insert(store.name.clone());
                }
            }
        }
    }

    for (_module, ast) in &parsed_modules {
        collect_fn_str_params(&ast.stmts, "", &mut global_fn_str_params);
        collect_fn_spec_params(&ast.stmts, "", &mut global_fn_spec_params);
        collect_fn_param_types(
            &ast.stmts,
            "",
            &mut global_fn_struct_params,
            &mut global_fn_int_params,
            Some(&mut global_merge_mut_params),
            Some(&mut global_fn_param_types),
        );
    }

    // Plan 433 A1: cross-module C11 `mut p T` flags — a later module calling an
    // earlier module's &mut-taking fn must pass `&mut *arg`, not `arg.clone()`.
    let mut global_fn_mut_params: std::collections::HashMap<AutoStr, Vec<bool>> =
        std::collections::HashMap::new();
    for (_module, ast) in &parsed_modules {
        for stmt in &ast.stmts {
            if let Stmt::Fn(f) = stmt {
                let flags: Vec<bool> = f
                    .params
                    .iter()
                    .map(|p| p.mode == crate::ast::ParamMode::Mut)
                    .collect();
                if flags.iter().any(|&b| b) {
                    global_fn_mut_params.insert(f.name.clone(), flags);
                }
            }
        }
    }

    // Plan 433 A1: cross-module fn return types (feeds let-binding type
    // inference via scan_call_init_bindings — a `var c = cg_compile(s)` in a
    // later module must know CG before its body emits).
    let mut global_fn_ret_types: std::collections::HashMap<AutoStr, Type> =
        std::collections::HashMap::new();
    for (_module, ast) in &parsed_modules {
        for stmt in &ast.stmts {
            match stmt {
                Stmt::Fn(f) => {
                    global_fn_ret_types.insert(f.name.clone(), f.ret.clone());
                    if let Some(parent) = &f.parent {
                        let qualified: AutoStr = format!("{}.{}", parent, f.name).into();
                        global_fn_ret_types.insert(qualified, f.ret.clone());
                    }
                }
                // Plan 514 W3: type 体内方法/ext 方法同列——跨模块 infer
                // (`p.text()` 的 "P.text" qualified 键)此前只对声明模块
                // 可见(单文件 trans 预扫),方法化 lib 跨文件调用点的
                // 返回类型推断/赋值借用/实参强转全部哑火。
                Stmt::TypeDecl(td) => {
                    for m in &td.methods {
                        global_fn_ret_types.insert(m.name.clone(), m.ret.clone());
                        let qualified: AutoStr = format!("{}.{}", td.name, m.name).into();
                        global_fn_ret_types.insert(qualified, m.ret.clone());
                    }
                }
                Stmt::Ext(ext) => {
                    for m in &ext.methods {
                        global_fn_ret_types.insert(m.name.clone(), m.ret.clone());
                        let qualified: AutoStr = format!("{}.{}", ext.target, m.name).into();
                        global_fn_ret_types.insert(qualified, m.ret.clone());
                    }
                }
                _ => {}
            }
        }
    }

    // Phase 3: Transpile all modules into a single Sink with merge_mode
    let mut sink = Sink::new(AutoStr::from("merged"));
    let mut seen_structs: HashSet<String> = HashSet::new();
    let mut seen_enums: HashSet<String> = HashSet::new();
    // PLAN-009 T1 (F1): restricted-name accumulator across per-module
    // transpilers (see drain inside the loop).
    let mut ord_restricted_acc: HashSet<AutoStr> = HashSet::new();

    for (idx, (module, ast)) in parsed_modules.iter().enumerate() {
        let mut transpiler = RustTrans::new(AutoStr::from("merged"));
        transpiler.merge_mode = true;
        transpiler.shared_type_store = Some(shared_type_store.clone());
        transpiler.emit_allow_pragma = idx == 0;
        transpiler.const_names = global_const_names.clone();

        transpiler.module_types = module_types.clone();
        let cur_mod_name = if module.is_dir_module {
            module
                .source_path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        } else {
            module
                .source_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        };
        transpiler.current_module_name = cur_mod_name.clone();
        transpiler.tag_types = all_enum_names.clone();
        // Plan 447 ③-P1: cross-module enum names for the return-type
        // concrete-check (p_kind -> TokenKind must not become impl TokenKind)
        for en in &all_enum_names {
            transpiler.known_enum_names.insert(en.clone());
        }

        // Pre-populate cross-module param indices
        for (name, flags) in &global_fn_str_params {
            if !transpiler.fn_str_param_indices.contains_key(name) {
                transpiler
                    .fn_str_param_indices
                    .insert(name.clone(), flags.clone());
            }
        }
        for (name, flags) in &global_fn_struct_params {
            if !transpiler.fn_struct_param_indices.contains_key(name) {
                transpiler
                    .fn_struct_param_indices
                    .insert(name.clone(), flags.clone());
            }
        }
        for (name, flags) in &global_fn_int_params {
            if !transpiler.fn_int_param_indices.contains_key(name) {
                transpiler
                    .fn_int_param_indices
                    .insert(name.clone(), flags.clone());
            }
        }
        for (name, ptypes) in &global_fn_param_types {
            if !transpiler.fn_param_types.contains_key(name) {
                transpiler
                    .fn_param_types
                    .insert(name.clone(), ptypes.clone());
            }
        }
        for (name, flags) in &global_merge_mut_params {
            if !transpiler.fn_merge_mut_params.contains_key(name) {
                transpiler
                    .fn_merge_mut_params
                    .insert(name.clone(), flags.clone());
            }
        }
        // Plan 433 A1: cross-module fn return types (see collection above).
        for (name, ret) in &global_fn_ret_types {
            if !transpiler.fn_ret_types.contains_key(name) {
                transpiler.fn_ret_types.insert(name.clone(), ret.clone());
            }
        }
        // Plan 433 A1: cross-module C11 mut-param flags (see collection above).
        for (name, flags) in &global_fn_mut_params {
            if !transpiler.fn_mut_params.contains_key(name) {
                transpiler.fn_mut_params.insert(name.clone(), flags.clone());
            }
        }

        // Pre-populate struct_fields from all modules
        for (_other_mod, other_ast) in &parsed_modules {
            for stmt in &other_ast.stmts {
                if let Stmt::TypeDecl(td) = stmt {
                    if !transpiler.struct_fields.contains_key(&td.name) {
                        let field_names: Vec<AutoStr> =
                            td.members.iter().map(|m| m.name.clone()).collect();
                        if !field_names.is_empty() {
                            transpiler
                                .struct_fields
                                .insert(td.name.clone(), field_names);
                        }
                        let field_types: Vec<(AutoStr, Type)> = td
                            .members
                            .iter()
                            .map(|m| (m.name.clone(), m.ty.clone()))
                            .collect();
                        if !field_types.is_empty() {
                            transpiler
                                .struct_field_types
                                .insert(td.name.clone(), field_types);
                        }
                        // PLAN-010 T2: delegation component fields for literal
                        // default-init (E0063 missing-field debt).
                        if !td.delegations.is_empty()
                            && !transpiler.struct_delegation_types.contains_key(&td.name)
                        {
                            transpiler.struct_delegation_types.insert(
                                td.name.clone(),
                                td.delegations
                                    .iter()
                                    .map(|d| (d.member_name.clone(), d.member_type.clone()))
                                    .collect(),
                            );
                        }
                    }
                }
            }
        }

        // Dedup: skip struct/enum definitions already emitted by a previous module
        let mut deduped_ast = ast.clone();
        deduped_ast.stmts.retain(|stmt| match stmt {
            Stmt::TypeDecl(td) => seen_structs.insert(td.name.to_string()),
            Stmt::EnumDecl(ed) => seen_enums.insert(ed.name.to_string()),
            _ => true,
        });
        // Record what we've seen
        for stmt in &ast.stmts {
            if let Stmt::TypeDecl(td) = stmt {
                seen_structs.insert(td.name.to_string());
            }
            if let Stmt::EnumDecl(ed) = stmt {
                seen_enums.insert(ed.name.to_string());
            }
        }

        transpiler.trans(deduped_ast, &mut sink)?;
        // PLAN-009 T1 (F1): transpiler is per-module here; drain its
        // restricted-name set into the function-level accumulator.
        ord_restricted_acc.extend(transpiler.ord_restricted_names.drain());
    }

    // Phase 3.4: Apply post-processing
    RustTrans::post_process_with(&mut sink.body, &ord_restricted_acc);
    post_process_merged(&mut sink.body);
    apply_merged_regex_fixes(&mut sink.body);

    Ok(sink.body)
}

fn discover_modules(
    file_path: &std::path::Path,
    base_dir: &std::path::Path,
    modules: &mut Vec<ProjectModule>,
    visited: &mut std::collections::HashSet<String>,
) -> AutoResult<()> {
    let canonical = file_path.canonicalize().map_err(|e| {
        AutoError::Msg(format!(
            "Cannot canonicalize {}: {}",
            file_path.display(),
            e
        ))
    })?;
    let key = canonical.to_string_lossy().to_string();

    if visited.contains(&key) {
        return Ok(());
    }
    visited.insert(key);

    let file_name = file_path
        .file_stem()
        .ok_or_else(|| AutoError::Msg("File has no stem".into()))?
        .to_string_lossy()
        .to_string();

    let is_dir_module = file_name == "mod";

    // Determine output path relative to base_dir
    let rel_path = file_path
        .parent()
        .and_then(|p| p.strip_prefix(base_dir).ok())
        .unwrap_or(std::path::Path::new(""));
    let output_name = if rel_path.as_os_str().is_empty() {
        format!("{}.rs", file_name)
    } else {
        format!("{}/{}.rs", rel_path.display(), file_name)
    };

    // Read and parse the file to discover its use statements
    let source = std::fs::read_to_string(file_path)
        .map_err(|e| AutoError::Msg(format!("Failed to read {}: {}", file_path.display(), e)))?;

    let mut local_uses = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        // Plan 167: handle both "use X" and "pub use X"
        let rest = if let Some(r) = trimmed.strip_prefix("pub use ") {
            r
        } else {
            match trimmed.strip_prefix("use ") {
                Some(r) => r,
                None => continue,
            }
        };
        if !rest.starts_with("c ") && !rest.starts_with(".rust") && !rest.starts_with("auto.") {
            // Extract module name (first segment before :, *, or end)
            let module_name = rest
                .split(|c: char| c == ':' || c == ' ' || c == '*')
                .next()
                .unwrap_or("")
                .trim();
            // Skip pac/super prefixed (handled by the resolver)
            if module_name == "pac"
                || module_name == "super"
                || module_name.starts_with("pac.")
                || module_name.starts_with("super.")
            {
                continue;
            }
            local_uses.push(module_name.to_string());
        }
    }

    // Add this module
    modules.push(ProjectModule {
        name: file_name.clone(),
        source_path: file_path.to_path_buf(),
        output_name: output_name.clone(),
        is_dir_module,
        uses: Vec::new(), // populated from parsed AST later
    });

    // Recursively discover dependencies
    for dep_name in &local_uses {
        // Plan 167: Handle dotted module names (e.g., "api.handlers")
        let parts: Vec<&str> = dep_name.split('.').collect();
        if parts.len() > 1 {
            // Check if the first segment matches the current directory module name
            // e.g., in api/mod.at, "api.handlers" -> just discover "handlers"
            let dir_name = rel_path.to_str().unwrap_or("");
            if dir_name == parts[0] {
                // Self-referential dotted path: strip the directory prefix
                let rest = parts[1..].join(".");
                let rest_file = file_path.parent().unwrap().join(format!("{}.at", rest));
                let rest_dir = file_path.parent().unwrap().join(&rest).join("mod.at");
                if rest_file.exists() {
                    discover_modules(&rest_file, base_dir, modules, visited)?;
                } else if rest_dir.exists() {
                    discover_modules(&rest_dir, base_dir, modules, visited)?;
                }
                continue;
            }

            // Cross-module dotted path: resolve each segment
            // First try relative to current file, then fall back to base_dir (project root)
            let cur_dir = file_path.parent().unwrap();
            let first_file = cur_dir.join(format!("{}.at", parts[0]));
            let first_dir = cur_dir.join(&parts[0]).join("mod.at");
            let first_file_root = base_dir.join(format!("{}.at", parts[0]));
            let first_dir_root = base_dir.join(&parts[0]).join("mod.at");

            let first_path = if first_file.exists() {
                first_file.clone()
            } else if first_dir.exists() {
                first_dir.clone()
            } else if first_file_root.exists() {
                first_file_root.clone()
            } else if first_dir_root.exists() {
                first_dir_root.clone()
            } else {
                continue;
            };

            // Discover the parent module
            discover_modules(&first_path, base_dir, modules, visited)?;

            // Then discover the nested module
            let parent_dir = first_path.parent().unwrap();
            let nested_name = parts[1..].join(".");
            let nested_file = parent_dir.join(format!("{}.at", nested_name));
            let nested_dir = parent_dir.join(&nested_name).join("mod.at");
            if nested_file.exists() {
                discover_modules(&nested_file, base_dir, modules, visited)?;
            } else if nested_dir.exists() {
                discover_modules(&nested_dir, base_dir, modules, visited)?;
            }
        } else {
            let dep_file = file_path.parent().unwrap().join(format!("{}.at", dep_name));
            let dep_dir = file_path.parent().unwrap().join(dep_name).join("mod.at");
            let dep_file_root = base_dir.join(format!("{}.at", dep_name));
            let dep_dir_root = base_dir.join(dep_name).join("mod.at");

            if dep_file.exists() {
                discover_modules(&dep_file, base_dir, modules, visited)?;
            } else if dep_dir.exists() {
                discover_modules(&dep_dir, base_dir, modules, visited)?;
            } else if dep_file_root.exists() {
                discover_modules(&dep_file_root, base_dir, modules, visited)?;
            } else if dep_dir_root.exists() {
                discover_modules(&dep_dir_root, base_dir, modules, visited)?;
            }
        }
    }

    // For directory modules (mod.at), also discover all sibling .at files
    // that may not be referenced via non-super use statements.
    // E.g., relay/turn.at is only referenced via `use super.turn` in mod.at,
    // which is skipped above. Scan disk to find all submodules.
    if is_dir_module {
        if let Some(parent_dir) = file_path.parent() {
            if let Ok(entries) = std::fs::read_dir(parent_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        if let Some(stem) = path.file_stem() {
                            let name = stem.to_string_lossy().to_string();
                            if name != "mod" && !name.starts_with('.') {
                                let _ = discover_modules(&path, base_dir, modules, visited);
                            }
                        }
                    }
                    // Also discover subdirectory modules (subdir/mod.at)
                    if path.is_dir() {
                        let sub_mod = path.join("mod.at");
                        if sub_mod.exists() {
                            let _ = discover_modules(&sub_mod, base_dir, modules, visited);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// ---- PLAN-884 T-06：主题算法模块（原样迁出 rust.rs） ----
mod ast_scans;
mod merged_fixes;
mod post_fixes;
use ast_scans::*;
use call_emit::{
    emit_http_verb_call, expr_root_is_builder_var, expr_root_is_http_request,
    for_range_is_http_stream,
};
use merged_fixes::*;

// ---- PLAN-843 结构拆分：语法族主题模块（原单文件热点，纯搬移） ----
mod call_emit;
mod decl_emit;
mod expr_emit;
mod project_fixups;
mod stmt_emit;
mod ty;
