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
use auto_val::{shared, Shared};
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

    edition: RustEdition,

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
            edition: RustEdition::E2021,
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
            edition: RustEdition::E2021,
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

    #[deprecated(note = "Use with_database() instead (Phase 066)")]
    pub fn set_scope(&mut self, _scope: Shared<crate::scope_manager::ScopeManager>) {
        // Plan 091: scope removed, no-op
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

    pub fn set_edition(&mut self, edition: RustEdition) {
        self.edition = edition;
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
    pub fn register_global_var(&mut self, name: AutoStr) {
        self.global_vars.insert(name);
    }

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

    // **Phase 6: Generic Programming**
    // Type alias declaration
    fn type_alias_decl(&mut self, type_alias: &TypeAlias, sink: &mut Sink) -> AutoResult<()> {
        // Generate type alias: type List<T> = List<T, Heap>;
        // In Rust: type List<T> = List<T, Heap>;
        write!(sink.body, "type {}", type_alias.name)?;

        // Type parameters
        if !type_alias.params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in type_alias.params.iter().enumerate() {
                write!(sink.body, "{}", param)?;
                if i < type_alias.params.len() - 1 {
                    write!(sink.body, ", ")?;
                }
            }
            write!(sink.body, ">")?;
        }

        // For the target type, if it's a GenericInstance with Unknown args,
        // we need to use the type parameter names instead of "Unknown"
        if let Type::GenericInstance(inst) = &type_alias.target {
            write!(sink.body, " = {}<", inst.base_name)?;
            // Use type parameters if available, otherwise use Unknown count
            let args: Vec<String> = if !type_alias.params.is_empty() {
                type_alias.params.iter().map(|p| p.to_string()).collect()
            } else {
                inst.args
                    .iter()
                    .map(|t| match t {
                        Type::Unknown => "_".to_string(),
                        _ => self.rust_type_name(t),
                    })
                    .collect()
            };
            write!(sink.body, "{}>;", args.join(", "))?;
        } else {
            write!(sink.body, " = {};", self.rust_type_name(&type_alias.target))?;
        }
        sink.body.write(b"\n")?;

        Ok(())
    }

    /// Convert a Heterogeneous EnumDecl to a Tag for reusing tag code generation.
    #[allow(dead_code)]
    fn enum_decl_to_tag(enum_decl: &EnumDecl) -> Tag {
        let fields: Vec<TagField> = enum_decl
            .items
            .iter()
            .map(|item| TagField {
                name: item.name.clone().into(),
                ty: item.payload_type.clone().unwrap_or(Type::Void),
            })
            .collect();
        let (generic_params, methods) = match &enum_decl.kind {
            EnumKind::Heterogeneous {
                generic_params,
                methods,
            } => (generic_params.clone(), methods.clone()),
            _ => (vec![], vec![]),
        };
        Tag {
            name: enum_decl.name.clone().into(),
            generic_params,
            fields,
            methods,
        }
    }

    // **Phase 1.2: Union Types (test: 013_union)**
    fn union_decl(&mut self, union: &Union, sink: &mut Sink) -> AutoResult<()> {
        // Cache union type name so construction and field-access sites can be
        // rewritten to safe accessor methods (Plan 310 Phase 0.2).
        // In Rust, union field access/construct is unsafe; we wrap it.
        self.union_types.insert(union.name.clone());

        // Generate union definition
        writeln!(sink.body, "union {} {{", union.name)?;
        self.indent();

        for field in &union.fields {
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "{}: {},",
                field.name,
                self.rust_type_name(&field.ty)
            )?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;

        // Plan 310 Phase 0.2: Generate safe accessor methods for each field.
        // Union field construction (`Union { f: v }`) and read (`u.f`) require
        // `unsafe` in Rust. We expose `new_<f>(v)` constructors and `<f>()`
        // readers so generated code stays in safe Rust.
        write!(sink.body, "impl {} {{", union.name)?;
        sink.body.write(b"\n")?;
        self.indent();
        for field in &union.fields {
            let fname = field.name.as_str();
            let fty = self.rust_type_name(&field.ty);
            // Constructor: fn new_<f>(v: T) -> Self { unsafe { Self { f: v } } }
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "pub fn new_{}(value: {}) -> Self {{ unsafe {{ Self {{ {}: value }} }} }}",
                fname, fty, fname
            )?;
            // Reader: fn <f>(&self) -> T { unsafe { self.f } }
            // For non-Copy field types (e.g. String) reading is unsafe-by-copy;
            // we only emit readers for Copy-like field types to avoid footguns.
            if Self::is_copy_type(&field.ty) {
                self.print_indent(&mut sink.body)?;
                writeln!(
                    sink.body,
                    "pub fn {}(&self) -> {} {{ unsafe {{ self.{} }} }}",
                    fname, fty, fname
                )?;
            }
        }
        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;

        Ok(())
    }

    // **Phase 1.3: Tag Types (test: 014_tag)**
    fn tag_decl(&mut self, tag: &Tag, sink: &mut Sink) -> AutoResult<()> {
        // Cache tag type name for tag construction detection
        self.tag_types.insert(tag.name.clone());

        // Generate enum definition for tag
        // AutoLang tags are algebraic data types that map to Rust enums
        write!(sink.body, "enum {}", tag.name)?;

        // Add generic parameters if present
        if !tag.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in tag.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => {
                        write!(sink.body, "{}", tp.name)?;
                        // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                        if !tp.constraint.is_empty() {
                            write!(sink.body, ": ")?;
                            for (ci, ct) in tp.constraint.iter().enumerate() {
                                if ci > 0 {
                                    write!(sink.body, " + ")?;
                                }
                                write!(sink.body, "{}", self.rust_bound_name(ct))?;
                            }
                        }
                    }
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        writeln!(sink.body, " {{")?;
        self.indent();

        for field in &tag.fields {
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "{}({}),",
                field.name,
                self.rust_type_name(&field.ty)
            )?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;
        sink.body.write(b"\n")?;

        // TODO: Generate impl block for tag methods (if any)
        for method in &tag.methods {
            // Tag methods will be added here
            let _ = method;
        }

        Ok(())
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

    fn method_mutates_self(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_mutates_self(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_mutates_self(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_mutates_self(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::method_mutates_self(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::method_mutates_self(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(els) = &if_stmt.else_ {
                        if Self::method_mutates_self(&els.stmts) {
                            return true;
                        }
                    }
                }
                Stmt::For(for_stmt) => {
                    if Self::method_mutates_self(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, b) => b,
                            crate::ast::IsBranch::IfBranch(_, b) => b,
                            crate::ast::IsBranch::ElseBranch(b) => b,
                        };
                        if Self::method_mutates_self(&body.stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if an expression mutates self (assignment to self.field, or
    /// mutating method call on self.field like push/insert).
    fn expr_mutates_self(expr: &Expr) -> bool {
        match expr {
            // self.field = value  (Op::Asn, AddEq, SubEq, etc.)
            Expr::Bina(lhs, op, _) => {
                if matches!(
                    op,
                    auto_val::Op::Asn
                        | auto_val::Op::AddEq
                        | auto_val::Op::SubEq
                        | auto_val::Op::MulEq
                        | auto_val::Op::DivEq
                        | auto_val::Op::ModEq
                ) {
                    // Plan 514 W3-16: 索引位赋值 self.scopes[i] = v 的根
                    // 也是 self——Index 基座为 self 点链同列(vpush 实证)。
                    if Self::is_self_dot(lhs) {
                        return true;
                    }
                    if let Expr::Index(base, _) = lhs.as_ref() {
                        if Self::is_self_dot(base) {
                            return true;
                        }
                    }
                }
                false
            }
            // self.field.push(...) / self.field.insert(...) / etc.
            Expr::Call(call) => {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    let mut_methods = [
                        "push", "pop", "insert", "remove", "clear", "next", "extend", "truncate",
                        "retain", "sort", "sort_by", "reverse", "dedup", "swap", "splice", "drain",
                        "append", "resize",
                        // Plan 514 W3-16: .set(i, v) 在发射期转索引赋值——
                        // 同为接收者变异(vpush 的 self.scopes.set 实证)。
                        "set",
                    ];
                    // Plan 514 W3: 裸 `self.method()`(obj 即 Ident self)与
                    // 字段中介形 `self.field.method()` 同列——此前只匹配后者,
                    // 方法体内 `.next()`/`self.next()` 语句漏判 → 接收者误发
                    // &self(E0596)。
                    let obj_is_self = Self::is_self_dot(obj)
                        || matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "self");
                    if mut_methods.contains(&method.as_str()) && obj_is_self {
                        return true;
                    }
                    // Also check nested: self.inner.push(...)
                    if mut_methods.contains(&method.as_str()) {
                        if let Expr::Dot(inner_obj, _) = obj.as_ref() {
                            if Self::is_self_dot(inner_obj) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Plan 514 W3: 表达式内是否存在对 targets 集合中方法的 self 接收者调用
    /// (`.m(...)`/`self.m(...)`——前导点解析为 Dot(Ident(\"self\"), m))。
    fn expr_calls_self_method(expr: &Expr, targets: &std::collections::HashSet<AutoStr>) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Dot(obj, m) = call.name.as_ref() {
                    if targets.contains(m.as_str())
                        && matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "self")
                    {
                        return true;
                    }
                    if Self::expr_calls_self_method(obj, targets) {
                        return true;
                    }
                }
                call.args.args.iter().any(|a| {
                    matches!(a,
                    Arg::Pos(e) if Self::expr_calls_self_method(e, targets))
                })
            }
            Expr::Dot(o, _) => Self::expr_calls_self_method(o, targets),
            Expr::Bina(a, _, b) | Expr::Index(a, b) | Expr::NullCoalesce(a, b) => {
                Self::expr_calls_self_method(a, targets) || Self::expr_calls_self_method(b, targets)
            }
            Expr::Unary(_, e)
            | Expr::ErrorPropagate(e)
            | Expr::Some(e)
            | Expr::Ok(e)
            | Expr::Err(e)
            | Expr::BoxExpr(e)
            | Expr::ArcExpr(e) => Self::expr_calls_self_method(e, targets),
            Expr::Array(items) | Expr::Tuple(items) => items
                .iter()
                .any(|e| Self::expr_calls_self_method(e, targets)),
            Expr::Object(pairs) => pairs
                .iter()
                .any(|pr| Self::expr_calls_self_method(&pr.value, targets)),
            _ => false,
        }
    }

    /// Plan 514 W3: 语句体内是否存在对 targets 中方法的 self 接收者调用。
    fn body_calls_self_method(
        stmts: &[Stmt],
        targets: &std::collections::HashSet<AutoStr>,
    ) -> bool {
        for stmt in stmts {
            let hit = match stmt {
                Stmt::Expr(e) => Self::expr_calls_self_method(e, targets),
                Stmt::Store(st) => Self::expr_calls_self_method(&st.expr, targets),
                Stmt::Return(e) => Self::expr_calls_self_method(e, targets),
                Stmt::If(i) => {
                    i.branches.iter().any(|b| {
                        Self::expr_calls_self_method(&b.cond, targets)
                            || Self::body_calls_self_method(&b.body.stmts, targets)
                    }) || i
                        .else_
                        .as_ref()
                        .map_or(false, |e| Self::body_calls_self_method(&e.stmts, targets))
                }
                Stmt::For(f) => {
                    Self::expr_calls_self_method(&f.range, targets)
                        || Self::body_calls_self_method(&f.body.stmts, targets)
                }
                Stmt::Block(b) => Self::body_calls_self_method(&b.stmts, targets),
                Stmt::Is(is) => is.branches.iter().any(|b| {
                    let body = match b {
                        crate::ast::IsBranch::EqBranch(_, bd) => bd,
                        crate::ast::IsBranch::IfBranch(_, bd) => bd,
                        crate::ast::IsBranch::ElseBranch(bd) => bd,
                    };
                    Self::body_calls_self_method(&body.stmts, targets)
                }),
                Stmt::Try(t) => {
                    Self::body_calls_self_method(&t.body.stmts, targets)
                        || Self::body_calls_self_method(&t.catch_body.stmts, targets)
                        || t.finally_body
                            .as_ref()
                            .map_or(false, |f| Self::body_calls_self_method(&f.stmts, targets))
                }
                _ => false,
            };
            if hit {
                return true;
            }
        }
        false
    }

    /// Plan 514 W3: type/ext 方法集的传递性可变方法名——直接判定(is_mut/
    /// method_mutates_self)为种子,经 self 接收者调用闭包扩张(方法体内
    /// 只调 `.fail(...)` 等用户可变方法的方法,接收者同样需要 &mut)。
    fn compute_type_mut_methods(methods: &[&Fn]) -> std::collections::HashSet<AutoStr> {
        let mut set: std::collections::HashSet<AutoStr> = methods
            .iter()
            .filter(|m| m.is_mut || Self::method_mutates_self(&m.body.stmts))
            .map(|m| m.name.clone())
            .collect();
        loop {
            let mut grew = false;
            for m in methods {
                if set.contains(m.name.as_str()) {
                    continue;
                }
                if Self::body_calls_self_method(&m.body.stmts, &set) {
                    set.insert(m.name.clone());
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        set
    }

    /// Incremental transpilation (Phase 066)
    /// Plan 163: Check if statements contain any await expression
    fn has_await(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_await(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::has_await(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::has_await(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::has_await(&else_body.stmts) {
                            return true;
                        }
                    }
                }
                // Plan 364 Phase 8 F1: recurse into for-loop bodies (previously
                // fell into `_ => {}`, so an .await inside a for-loop didn't
                // trigger #[tokio::main] async fn main).
                Stmt::For(for_stmt) => {
                    if Self::has_await(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Like has_await but takes a slice of references (for use with split stmts)
    fn has_await_refs(stmts: &[&Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_await(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    let refs: Vec<&Stmt> = body.stmts.iter().collect();
                    if Self::has_await_refs(&refs) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        let refs: Vec<&Stmt> = branch.body.stmts.iter().collect();
                        if Self::has_await_refs(&refs) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        let refs: Vec<&Stmt> = else_body.stmts.iter().collect();
                        if Self::has_await_refs(&refs) {
                            return true;
                        }
                    }
                }
                // Plan 364 Phase 8 F1: recurse into for-loop bodies.
                Stmt::For(for_stmt) => {
                    let refs: Vec<&Stmt> = for_stmt.body.stmts.iter().collect();
                    if Self::has_await_refs(&refs) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 240: Check if statements contain ErrorPropagate (`.?` operator)
    fn has_error_propagate(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_error_propagate(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_error_propagate(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_error_propagate(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::has_error_propagate(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::has_error_propagate(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::has_error_propagate(&else_body.stmts) {
                            return true;
                        }
                    }
                }
                Stmt::For(for_stmt) => {
                    if Self::has_error_propagate(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        if Self::has_error_propagate(&body.stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 347: Detect whether a function body returns an explicit
    /// `Ok(...)` / `Err(...)` value. The Auto source for some library
    /// functions (e.g. `fn decode(input str) { ... return Ok(...) }`) declares
    /// no return type but actually produces a `Result`. The transpiler must
    /// infer a `Result` return type in that case, otherwise Rust rejects the
    /// `return Ok(...)` against the implicit `()` return type. Recurses into
    /// nested blocks/if/for like `has_error_propagate`.
    fn body_returns_result(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Return(expr) => {
                    if matches!(&**expr, Expr::Ok(_) | Expr::Err(_)) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::body_returns_result(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::body_returns_result(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::body_returns_result(&else_body.stmts) {
                            return true;
                        }
                    }
                }
                Stmt::For(for_stmt) => {
                    if Self::body_returns_result(&for_stmt.body.stmts) {
                        return true;
                    }
                }
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        if Self::body_returns_result(&body.stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 347 (url): Infer the Ok payload type for an un-annotated function
    /// whose body returns `Ok(...)` / `Err(...)`. Previously the inferred
    /// Result was always `Result<String, String>`, which miscompiles when the
    /// `Ok` payload is a struct (e.g. `fn parse(...) { return Ok(Url { ... }) }`
    /// would be typed `Result<String, String>`). This scans the body's
    /// `return Ok(X)` / tail `Ok(X)` expressions and returns:
    ///   * `Type::User(TypeDecl{ name, .. })` when X is a struct construction
    ///     (`Expr::Node { name }`),
    ///   * `Type::StrOwned` (the original default) otherwise.
    /// The first struct-typed Ok payload wins; mixed payloads fall back to
    /// `StrOwned` to stay safe.
    fn infer_result_ok_type(&self, stmts: &[Stmt]) -> Type {
        let mut found = None;
        self.scan_result_ok_type(stmts, &mut found);
        found.unwrap_or(Type::StrOwned)
    }

    fn scan_result_ok_type(&self, stmts: &[Stmt], found: &mut Option<Type>) {
        if found.is_some() {
            return;
        }
        for stmt in stmts {
            match stmt {
                Stmt::Return(expr) => {
                    if let Expr::Ok(inner) = &**expr {
                        self.classify_ok_payload(inner, found);
                    }
                }
                // A bare `Ok(...)` expression as the last statement (tail expr).
                Stmt::Expr(e) => {
                    if let Expr::Ok(inner) = e {
                        self.classify_ok_payload(inner, found);
                    }
                }
                Stmt::Block(body) => self.scan_result_ok_type(&body.stmts, found),
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        self.scan_result_ok_type(&branch.body.stmts, found);
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        self.scan_result_ok_type(&else_body.stmts, found);
                    }
                }
                Stmt::For(for_stmt) => self.scan_result_ok_type(&for_stmt.body.stmts, found),
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        self.scan_result_ok_type(&body.stmts, found);
                    }
                }
                _ => {}
            }
            if found.is_some() {
                return;
            }
        }
    }

    /// Classify a single `Ok(...)` payload expression into a Rust type.
    /// Struct construction (`Url { ... }`) parses as `Expr::Node { name }`.
    fn classify_ok_payload(&self, inner: &Expr, found: &mut Option<Type>) {
        if found.is_some() {
            return;
        }
        match inner {
            Expr::Node(node) => {
                // Struct construction: `Url { scheme: ..., ... }`.
                let ty = Type::User(crate::ast::TypeDecl {
                    consts: Vec::new(),
                    name: node.name.clone(),
                    kind: crate::ast::TypeDeclKind::UserType,
                    parent: None,
                    has: Vec::new(),
                    specs: Vec::new(),
                    spec_impls: Vec::new(),
                    generic_params: Vec::new(),
                    members: Vec::new(),
                    delegations: Vec::new(),
                    methods: Vec::new(),
                    attrs: Vec::new(),
                    impl_attrs: vec![],
                    doc: None,
                    is_pub: false,
                });
                *found = Some(ty);
            }
            Expr::Str(_) | Expr::CStr(_) => {
                // String payload -> keep the historical default.
                *found = Some(Type::StrOwned);
            }
            _ => {
                // Anything else (idents, calls, ...): leave unset so other
                // payloads can still refine; falls back to StrOwned at the end.
            }
        }
    }

    /// Plan 347: Collect the names of identifiers that are used as the
    /// scrutinee of an `is`/`match` expression whose branches pattern-match on
    /// `Ok(...)` or `Err(...)`. Such identifiers must be `Result` values. This
    /// lets the transpiler infer a `Result` type for untyped function
    /// parameters (which otherwise default to `i32`) that are matched this way,
    /// e.g. `fn ok_value(r) { is r { Ok(v) -> ... } }`.
    fn result_pattern_idents(stmts: &[Stmt]) -> std::collections::HashSet<String> {
        let mut out = std::collections::HashSet::new();
        Self::collect_result_pattern_idents(stmts, &mut out);
        out
    }

    fn collect_result_pattern_idents(stmts: &[Stmt], out: &mut std::collections::HashSet<String>) {
        for stmt in stmts {
            match stmt {
                Stmt::Is(is_stmt) => {
                    let has_ok_err = is_stmt.branches.iter().any(|branch| {
                        if let crate::ast::IsBranch::EqBranch(patterns, _) = branch {
                            patterns.iter().any(|p| matches!(p, Expr::ResultPattern(_)))
                        } else {
                            false
                        }
                    });
                    if has_ok_err {
                        if let Expr::Ident(name) = &is_stmt.target {
                            out.insert(name.to_string());
                        }
                    }
                    // Recurse into branch bodies.
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        Self::collect_result_pattern_idents(&body.stmts, out);
                    }
                }
                Stmt::Block(body) => {
                    Self::collect_result_pattern_idents(&body.stmts, out);
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        Self::collect_result_pattern_idents(&branch.body.stmts, out);
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        Self::collect_result_pattern_idents(&else_body.stmts, out);
                    }
                }
                Stmt::For(for_stmt) => {
                    Self::collect_result_pattern_idents(&for_stmt.body.stmts, out);
                }
                _ => {}
            }
        }
    }

    /// Plan 240: Check if an expression contains ErrorPropagate (`.?` operator)
    fn expr_has_error_propagate(expr: &Expr) -> bool {
        match expr {
            Expr::ErrorPropagate(_) => true,
            Expr::Call(call) => {
                if Self::expr_has_error_propagate(call.name.as_ref()) {
                    return true;
                }
                for arg in &call.args.args {
                    match arg {
                        Arg::Pos(e) | Arg::Pair(_, e) => {
                            if Self::expr_has_error_propagate(e) {
                                return true;
                            }
                        }
                        Arg::Name(_) => {}
                    }
                }
                false
            }
            Expr::Block(body) => Self::has_error_propagate(&body.stmts),
            Expr::Bina(left, _, right) => {
                Self::expr_has_error_propagate(left) || Self::expr_has_error_propagate(right)
            }
            Expr::Unary(_, e) => Self::expr_has_error_propagate(e),
            Expr::Dot(obj, _) => Self::expr_has_error_propagate(obj),
            Expr::Index(arr, idx) => {
                Self::expr_has_error_propagate(arr) || Self::expr_has_error_propagate(idx)
            }
            Expr::View(e) | Expr::Mut(e) | Expr::Move(e) | Expr::Take(e) => {
                Self::expr_has_error_propagate(e)
            }
            Expr::FStr(fstr) => fstr.parts.iter().any(|p| Self::expr_has_error_propagate(p)),
            Expr::Array(arr) => arr.iter().any(|e| Self::expr_has_error_propagate(e)),
            _ => false,
        }
    }

    /// Plan 220 Task 4: Check if an expression needs an `as usize` cast
    /// when used as a slice/array index in Rust.
    ///
    /// Integer literals do NOT need a cast -- Rust infers the correct type
    /// automatically in index position (e.g., `arr[0]` just works).
    /// Non-trivial expressions (variables, binary ops, calls) may be u32/i32
    /// and need explicit `as usize` for Rust indexing and range bounds.
    fn needs_usize_cast(expr: &Expr) -> bool {
        match expr {
            // Integer literals: Rust infers correct type in index position
            Expr::Int(_)
            | Expr::Uint(_)
            | Expr::I8(_)
            | Expr::U8(_)
            | Expr::I64(_)
            | Expr::U64(_)
            | Expr::Byte(_) => false,
            // Range: bounds are handled individually, not the range itself
            Expr::Range(_) => false,
            // Non-integer literals: not used as indices
            Expr::Bool(_) | Expr::Nil | Expr::Null => false,
            // Variables, binary ops, calls, dot access, etc. may be u32/i32
            _ => true,
        }
    }

    /// PLAN-596 T-07 (DIV-DEP-8): a use.rs-imported dep TYPE — PascalCase
    /// name present as a use leaf, never declared locally. Local decls
    /// (struct/tag/enum/union) always win, keeping Debug-dependent存量语料
    /// on the `{:?}` path (AC-04 isolation arm).
    fn is_dep_type_name(&self, name: &str) -> bool {
        // PascalCase required: SCREAMING_CASE use.rs imports are consts
        // (DIV-DEP-13), never types — `STANDARD.encode(..)` is a const-value
        // method call, not a ctor chain.
        if !name.chars().next().map_or(false, |c| c.is_uppercase())
            || !name.chars().any(|c| c.is_ascii_lowercase())
        {
            return false;
        }
        if self.local_struct_types.contains(name)
            || self.struct_fields.contains_key(name)
            || self.struct_field_types.contains_key(name)
            || self.tag_types.contains(name)
            || self.known_enum_names.contains(name)
            || self.union_types.contains(name)
        {
            return false;
        }
        self.uses.iter().any(|u| {
            let u = u.as_str();
            u == name || u.ends_with(&format!("::{}", name)) || u.contains(&format!("{{{}}}", name))
        })
    }

    /// PLAN-596 T-07 (DIV-DEP-13): a SCREAMING_CASE use.rs import (STANDARD,
    /// NIL, OP_XXX) is a const VALUE receiver, not a type — `CONST.method(..)`
    /// must lower to a dot-call (trait-method consts like base64's
    /// `STANDARD.encode` are E0224 as `CONST::method`). Rust types are
    /// PascalCase; local declarations take precedence.
    fn is_dep_constant_name(&self, name: &str) -> bool {
        let has_upper = name.chars().any(|c| c.is_ascii_uppercase());
        let no_lower = !name.chars().any(|c| c.is_ascii_lowercase());
        has_upper
            && no_lower
            && !self.local_struct_types.contains(name)
            && !self.tag_types.contains(name)
            && !self.known_enum_names.contains(name)
            && !self.union_types.contains(name)
            && self.uses.iter().any(|u| {
                let u = u.as_str();
                u == name
                    || u.ends_with(&format!("::{}", name))
                    || u.contains(&format!("{{{}}}", name))
            })
    }

    /// PLAN-596 T-07 (DIV-DEP-8): does this expression denote a rust-typed
    /// VALUE from a dep crate (Display-stringified, three-way parity) rather
    /// than an Auto type (Debug fallback)? Ident receivers resolve through
    /// local_var_types (fed by dep_ctor_type at store time); inline ctor
    /// chains (`Url.parse(..).unwrap()`) walk the chain directly.
    fn receiver_is_dep_rust_value(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => matches!(self.local_var_types.get(name),
                Some(Type::User(td)) if self.is_dep_type_name(td.name.as_str())),
            Expr::Call(call) => match call.name.as_ref() {
                Expr::Dot(obj, m) if m.as_str() == "unwrap" => self.receiver_is_dep_rust_value(obj),
                Expr::Dot(obj, _) => {
                    matches!(obj.as_ref(), Expr::Ident(n) if self.is_dep_type_name(n))
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// PLAN-596 T-07: dep ctor chain detection for store-time typing —
    /// `let v = Version.parse(..).unwrap()` / `let u = Url.parse(..)` yield
    /// the dep type name, so the binding gets a User type and downstream
    /// `.to(str)`/print resolve Display instead of the Unknown fallback.
    fn dep_ctor_type(&self, expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Call(call) => match call.name.as_ref() {
                Expr::Dot(obj, m) if m.as_str() == "unwrap" => self.dep_ctor_type(obj),
                Expr::Dot(obj, _) => match obj.as_ref() {
                    Expr::Ident(n) if self.is_dep_type_name(n) => Some(n.clone()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    /// Check if an expression likely produces a Debug-only type (no Display impl).
    /// Detects patterns like `.elapsed()`, `Instant::now()`, and variables named
    /// duration/elapsed/instant.
    fn needs_debug_format(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                // Only use name heuristics for types strongly associated with non-Display:
                // Duration, Instant, DirEntry, etc. Common variable names like "count",
                // "value", "avg" are often primitives (i32, f64) that implement Display.
                let lower = name.as_str().to_lowercase();
                if lower.contains("duration")
                    || lower.contains("elapsed")
                    || lower.contains("instant")
                    || lower.contains("dir_entry")
                {
                    return true;
                }
                // Check local_var_types for non-Display types
                if let Some(ty) = self.local_var_types.get(name) {
                    return matches!(
                        ty,
                        Type::List(_)
                            | Type::Map(_, _)
                            | Type::Array(_)
                            | Type::RuntimeArray(_)
                            | Type::Slice(_)
                            | Type::Option(_)
                            | Type::Result(_)
                            | Type::Tuple(_)
                            | Type::Tag(_)
                            | Type::Enum(_)
                    );
                }
                false
            }
            Expr::Dot(obj, method) => method == "elapsed" || self.needs_debug_format(obj),
            Expr::Bina(lhs, op, rhs) => {
                if matches!(op, Op::Dot) {
                    // Check for expr.elapsed()
                    if let Expr::Ident(m) = rhs.as_ref() {
                        if m.as_str() == "elapsed" {
                            return true;
                        }
                    }
                    self.needs_debug_format(lhs)
                } else {
                    self.needs_debug_format(lhs) || self.needs_debug_format(rhs)
                }
            }
            Expr::Call(call) => self.needs_debug_format(&call.name),
            Expr::ErrorPropagate(inner) => self.needs_debug_format(inner),
            _ => false,
        }
    }

    fn is_self_dot(expr: &Expr) -> bool {
        matches!(expr, Expr::Dot(obj, _) if matches!(obj.as_ref(), Expr::Ident(name) if name == "self"))
    }

    /// Plan 013 (B16): collect bare-identifier names used as variant-pattern
    /// arguments (e.g. the `child` in `Kid.Node(child)`), so a later auto-clone
    /// at a call site can deref a Box<T> before cloning. Recurses into nested
    /// calls so multi-field variants are covered too.
    fn collect_pattern_idents(expr: &Expr, out: &mut HashSet<AutoStr>) {
        match expr {
            Expr::Call(call) => {
                for arg in &call.args.args {
                    if let Arg::Pos(e) = arg {
                        Self::collect_pattern_idents(e, out);
                    }
                }
            }
            Expr::Ident(name) => {
                out.insert(name.clone());
            }
            // Plan 013 (B16): `Type.Variant(binding)` patterns parse as
            // Cover(Tag(TagCover { bindings })). Collect those bindings — they
            // bind to Box<T> for bridge-crate variants like Kid.Node(child).
            Expr::Cover(cover) => {
                if let crate::ast::Cover::Tag(tc) = cover {
                    for b in &tc.bindings {
                        if b != "_" {
                            out.insert(b.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Plan 163: Check if an expression contains await
    fn expr_has_await(expr: &Expr) -> bool {
        match expr {
            Expr::Await { .. } => true,
            Expr::Call(call) => {
                if Self::expr_has_await(call.name.as_ref()) {
                    return true;
                }
                for arg in &call.args.args {
                    match arg {
                        Arg::Pos(e) | Arg::Pair(_, e) => {
                            if Self::expr_has_await(e) {
                                return true;
                            }
                        }
                        Arg::Name(_) => {}
                    }
                }
                false
            }
            Expr::Block(body) => Self::has_await(&body.stmts),
            Expr::Bina(left, _, right) => Self::expr_has_await(left) || Self::expr_has_await(right),
            Expr::Unary(_, expr) => Self::expr_has_await(expr),
            Expr::Dot(obj, _) => Self::expr_has_await(obj),
            Expr::Index(arr, idx) => Self::expr_has_await(arr) || Self::expr_has_await(idx),
            Expr::View(e) | Expr::Mut(e) | Expr::Move(e) | Expr::Take(e) => Self::expr_has_await(e),
            Expr::AsyncBlock { body, .. } => Self::has_await(&body.stmts),
            Expr::Cast { expr, .. } | Expr::To { expr, .. } => Self::expr_has_await(expr),
            Expr::NullCoalesce(l, r) => Self::expr_has_await(l) || Self::expr_has_await(r),
            Expr::ErrorPropagate(e) => Self::expr_has_await(e),
            _ => false,
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

    /// Apply all post-processing fixes to generated Rust source.
    /// Called after trans() on the final output.
    /// Plan 014 Layer 3: run a post_process fix, tallying whether it rewrote
    /// the output (for A2R_FIX_COUNTS instrumentation).
    fn fix_counted(content: &mut String, name: &str, f: fn(&mut String)) {
        let before = content.len();
        f(content);
        if content.len() != before {
            if let Ok(mut m) = FIX_COUNTS.lock() {
                *m.entry(name.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// PLAN-009 T1 (F1): names whose auto-derived comparison set was
    /// restricted; callers thread this into `post_process_with`. (Read-only
    /// accessor; the field itself stays private.)
    pub fn ord_restricted_names(&self) -> &HashSet<AutoStr> {
        &self.ord_restricted_names
    }

    pub fn post_process(output: &mut Vec<u8>) {
        Self::post_process_with(output, &HashSet::new());
    }

    /// Remove `use self::X;` lines that duplicate `pub mod X;` declarations.
    fn remove_duplicate_module_uses(content: &mut String) {
        use std::collections::HashSet;
        let pub_mods: HashSet<String> = regex_captures(content, r"pub mod (\w+);");

        if pub_mods.is_empty() {
            return;
        }

        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut new_lines = Vec::new();
        let mut removed = 0;
        for line in &lines {
            let stripped = line.trim();
            let mut skip = false;
            for mod_name in &pub_mods {
                if stripped == format!("use self::{};", mod_name) {
                    skip = true;
                    removed += 1;
                    break;
                }
            }
            if !skip {
                new_lines.push(line.clone());
            }
        }
        if removed > 0 {
            *content = new_lines.join("\n");
        }
    }

    /// Remove `use` statements that import symbols already defined locally.
    fn remove_duplicate_imports(content: &mut String) {
        // Find locally defined symbols (fn, struct, enum, trait, const, static, type names)
        let local_syms: Vec<String> = regex_captures_vec(
            content,
            r"\b(?:pub\s+)?(?:fn|struct|enum|trait|const|static|type)\s+(\w+)",
        );

        if local_syms.is_empty() {
            return;
        }

        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut new_lines = Vec::new();
        let mut removed = 0;
        for line in &lines {
            let stripped = line.trim();
            if stripped.starts_with("use ") && stripped.ends_with(';') {
                // Extract symbol from use path
                let path = &stripped[4..stripped.len() - 1]; // strip "use " and ";"
                let last_part = path.rsplit("::").next().unwrap_or(path);
                // Handle braced imports: use crate::module::{A, B};
                if last_part.starts_with('{') && last_part.ends_with('}') {
                    let inner = &last_part[1..last_part.len() - 1];
                    let syms: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                    let any_local = syms.iter().any(|s| local_syms.contains(&s.to_string()));
                    if any_local {
                        removed += 1;
                        continue;
                    }
                } else if local_syms.contains(&last_part.to_string()) && path.contains("::") {
                    removed += 1;
                    continue;
                }
            }
            new_lines.push(line.clone());
        }
        if removed > 0 {
            *content = new_lines.join("\n");
        }
    }

    /// Fix numeric literal .as_str() — numbers should never have .as_str()
    /// E.g., 0.as_str() → 0, 100000.as_str() → 100000
    fn fix_numeric_get_as_str(content: &mut String) {
        // Remove .as_str() after any numeric literal (standalone digits)
        // Use \b to avoid matching trailing digits in identifiers like body_str2.as_str()
        if let Some(re) = cached_regex(r"\b(\d+)\.as_str\(\)") {
            let new_content = re.replace_all(content.as_str(), "$1").to_string();
            *content = new_content;
        }
    }

    /// Fix `match expr.get(X) { Some(binding) => { ...` by adding `.cloned()`
    /// to convert `Option<&T>` to `Option<T>` when the binding is used as a value.
    fn fix_get_cloned_for_match(content: &mut String) {
        // Pattern: self.field.get(X) { Some(var) => { let ... = var; → add .cloned()
        // Also: self.field.get(X) { Some(var) -> { var.field → add .cloned()
        let fields_needing_cloned = ["sessions", "run", "checkpoint"];
        for field in &fields_needing_cloned {
            let pattern = format!(r"self\.{}\.get\(([^)]+)\) \{{", regex::escape(field));
            if let Some(re) = cached_regex(&pattern) {
                let new_content = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        format!(
                            "self.{}.get({}).cloned() {{",
                            field,
                            caps.get(1).unwrap().as_str()
                        )
                    })
                    .to_string();
                if new_content != *content {
                    *content = new_content;
                }
            }
        }
        // Also fix: return self.field.get(X); → return self.field.get(X).cloned();
        // HashMap::get returns Option<&T>, but Auto expects Option<T> for return types
        if let Some(re) = cached_regex(r"return self\.(\w+)\.get\(([^)]+)\);") {
            let map_fields = [
                "sessions",
                "run",
                "checkpoint",
                "pages",
                "wiki_dirs",
                "project_locks",
                "professions",
                "souls",
                "agents",
            ];
            let new_content = re
                .replace_all(content, |caps: &regex::Captures| {
                    let field = caps.get(1).unwrap().as_str();
                    let key = caps.get(2).unwrap().as_str();
                    if map_fields.contains(&field) {
                        format!("return self.{}.get({}).cloned();", field, key)
                    } else {
                        caps.get(0).unwrap().as_str().to_string()
                    }
                })
                .to_string();
            if new_content != *content {
                *content = new_content;
            }
        }
    }

    /// Fix Vec/HashMap .insert() where first argument needs to be usize.
    /// Only handles variables with known integer type annotations (u32, i32).
    fn fix_insert_usize(content: &mut String) {
        use std::collections::HashSet;
        let mut int_names: HashSet<String> = HashSet::new();
        for ty in &["u32", "i32"] {
            let pat = format!(r"let\s+(?:mut\s+)?(\w+)\s*:\s*{}\s*=", ty);
            if let Some(re) = cached_regex(&pat) {
                for caps in re.captures_iter(content) {
                    int_names.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            let pat = format!(r"pub\s+(\w+)\s*:\s*{}\s*,", ty);
            if let Some(re) = cached_regex(&pat) {
                for caps in re.captures_iter(content) {
                    int_names.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
        }
        // Also collect vars assigned from known u32-returning functions
        if let Some(re) = cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*=\s*self\.ensure_tool_call\(") {
            for caps in re.captures_iter(content) {
                int_names.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }
        if int_names.is_empty() {
            return;
        }

        for name in &int_names {
            let pattern = format!(r"\.insert\(\s*{}\s*(,)", regex::escape(name));
            if let Some(re) = cached_regex(&pattern) {
                let n = name.clone();
                let new = re
                    .replace_all(content.as_str(), move |caps: &regex::Captures| {
                        let comma = caps.get(1).unwrap().as_str();
                        format!(".insert({} as usize{}", n, comma)
                    })
                    .to_string();
                *content = new;
            }
        }
    }

    /// Fix derive macros on structs containing `dyn Trait` fields — `Box<dyn X>`
    /// (spec params) or `Arc<dyn X>` (from `Arc<Spec>` type annotations, Plan
    /// 390 §15.11 L2 转正). `dyn Trait` doesn't implement Clone/PartialEq/Eq/
    /// PartialOrd/Ord (unless the wrapper provides them):
    ///   - `Box<dyn T>`: Clone requires T: Clone → unsafe; Debug requires T: Debug → unsafe.
    ///   - `Arc<dyn T>`: `Arc<T>` is unconditionally Clone for T: ?Sized → Clone SAFE;
    ///                    Debug/PartialEq/Eq/Ord still require T: those → unsafe.
    /// So Box<dyn> fields → allow(dead_code) (all derives unsafe); Arc<dyn>
    /// fields keep Clone but strip the rest. If the user explicitly supplied a
    /// derive omitting the unsafe traits, leave it untouched (Plan 376 override).
    fn fix_dyn_trait_derives(content: &mut String) {
        if let Some(re) = cached_regex(
            r"(?s)(#\[derive\(([^)]*)\)\]\npub struct (\w+) \{[^}]*?(?:Box<dyn|Arc<dyn))",
        ) {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let full = caps.get(0).unwrap().as_str();
                    let derives = caps.get(2).unwrap().as_str();
                    let derive_list: Vec<&str> = derives.split(',').map(|s| s.trim()).collect();
                    // Plan 376: respect a user-supplied derive that already omits the
                    // unsafe traits (e.g. just "Debug", or "Clone, Debug" when the
                    // dyn-Trait is wrapped in an Arc — which IS Clone). Only strip
                    // PartialEq/Eq/PartialOrd/Ord from the AUTO-generated derive.
                    // Plan 376V: Clone AND Debug are unsafe for Box<dyn Trait>:
                    //   - Box<T>: Clone requires T: Clone (dyn Trait never is)
                    //   - Box<dyn Trait>: Debug requires Trait: Debug (specs don't bound it)
                    // So when a struct has a bare Box<dyn> field, replace the whole
                    // derive with #[allow(dead_code)] (matches the hand-written version).
                    // Plan 390 §15.11 (L2): `Arc<dyn T>` keeps Clone — Arc<T> is
                    // unconditionally Clone for T: ?Sized; only Debug/PartialEq/Eq/
                    // Ord remain unsafe (they require T: those).
                    let has_arc_dyn = full.contains("Arc<dyn");
                    let unsafe_traits: &[&str] = if has_arc_dyn {
                        &["PartialEq", "Eq", "PartialOrd", "Ord", "Debug"]
                    } else {
                        &["PartialEq", "Eq", "PartialOrd", "Ord", "Clone", "Debug"]
                    };
                    let needs_fix = derive_list.iter().any(|d| unsafe_traits.contains(d));
                    if !needs_fix {
                        return full.to_string();
                    }
                    // If ALL derives are unsafe (typical for Box<dyn>), use allow(dead_code).
                    let any_safe = derive_list.iter().any(|d| !unsafe_traits.contains(d));
                    if !any_safe {
                        return full
                            .replace(&format!("#[derive({})]", derives), "#[allow(dead_code)]");
                    }
                    // Keep only the safe traits (Clone, Debug, Copy, Default, ...).
                    let kept: Vec<&&str> = derive_list
                        .iter()
                        .filter(|d| !unsafe_traits.contains(d))
                        .collect();
                    if kept.is_empty() {
                        full.replace(&format!("#[derive({})]", derives), "#[derive(Debug)]")
                    } else {
                        let new_derives: Vec<&str> = kept.into_iter().copied().collect();
                        full.replace(
                            &format!("#[derive({})]", derives),
                            &format!("#[derive({})]", new_derives.join(", ")),
                        )
                    }
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix `return None;` in void (unit-return) functions → `return;`.
    /// Auto's `return` in void functions is parsed as `Return(Nil)` → transpiled as `return None;`
    /// but Rust void functions need plain `return;`.
    fn fix_void_return_none(content: &mut String) {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::with_capacity(lines.len());
        let mut in_void_fn = false;
        let mut brace_depth: i32 = 0;
        let mut fn_brace_depth: i32 = 0;

        for line in &lines {
            let trimmed = line.trim();

            // Track function declarations without return type (void).
            // Plan 384: `-> ()` (explicit unit) is also void.
            let is_void_ret =
                !trimmed.contains("->") || trimmed.contains("-> ()") || trimmed.contains("->()");
            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && is_void_ret
            {
                in_void_fn = true;
                fn_brace_depth = brace_depth;
            }

            // Track braces
            for ch in trimmed.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }

            // If we've exited the void function's scope, reset
            if in_void_fn && brace_depth <= fn_brace_depth && trimmed.contains('}') {
                in_void_fn = false;
            }

            // Replace return None; with return; in void functions
            if in_void_fn && trimmed == "return None;" {
                result.push(line.replacen("return None;", "return;", 1));
            } else {
                result.push(line.to_string());
            }
        }

        let new_content = result.join("\n");
        if new_content != *content {
            *content = new_content;
        }
    }

    /// Fix enum == "str" comparisons.
    fn fix_enum_str_comparisons(content: &mut String) {
        let enum_fields = [
            "section_type",
            "status",
            "phase",
            "kind",
            "role",
            "stop_reason",
            "source_type",
            "provider",
            "decision",
        ];
        for field in &enum_fields {
            let eq_pat = format!(".{}\\s*==\\s*\"", field);
            if let Some(re) = cached_regex(&eq_pat) {
                let old_eq = format!(".{field} ==");
                let new_eq = format!(".{field}.to_string() ==");
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        caps.get(0).unwrap().as_str().replace(&old_eq, &new_eq)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
            let ne_pat = format!(".{}\\s*!=\\s*\"", field);
            if let Some(re) = cached_regex(&ne_pat) {
                let old_ne = format!(".{field} !=");
                let new_ne = format!(".{field}.to_string() !=");
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        caps.get(0).unwrap().as_str().replace(&old_ne, &new_ne)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
        }
    }

    /// Fix tuple.get_N() -> tuple.N (Rust tuple indexing)
    fn fix_tuple_get_n(content: &mut String) {
        let mut count = 0;
        for n in 0..=9 {
            let pattern = format!(".get_{}()", n);
            let replacement = format!(".{}", n);
            let reduced = content.replace(&pattern, &replacement);
            if reduced != *content {
                count += 1;
                *content = reduced;
            }
        }
        let _ = count;
    }

    /// Fix map.get(key).as_str() → map.get(key).map(|s| s.as_str()).unwrap_or("")
    /// HashMap::get returns Option<&String>, but Auto treats get() as returning the value directly.
    fn fix_map_get_as_str(content: &mut String) {
        // Step 1: Replace `let VAR = EXPR.get(KEY);` with
        //         `let VAR = EXPR.get(KEY).cloned().unwrap_or_default();`
        //         ONLY for bootstrap compiler env/state variables (env.*, params.*, state.*)
        //         NOT for use.rust HashMap (those should keep native Option<&V> semantics)
        if let Some(re) = cached_regex(r"let\s+(\w+)\s*=\s*(\w+\.get\([^)]+\));") {
            let mut replacements = Vec::new();
            for caps in re.captures_iter(content) {
                let var = caps.get(1).unwrap().as_str();
                let get_expr = caps.get(2).unwrap().as_str();
                // Only apply to known bootstrap env variables: env.*, params.*, headers.*, state.*
                let is_bootstrap_env = get_expr.starts_with("env.")
                    || get_expr.starts_with("params.")
                    || get_expr.starts_with("headers.")
                    || get_expr.starts_with("state.");
                if is_bootstrap_env
                    && (get_expr.contains(".get(\"") || get_expr.contains(".get(\""))
                {
                    replacements.push((var.to_string(), get_expr.to_string()));
                }
            }
            for (var, get_expr) in &replacements {
                let old = format!("let {} = {};", var, get_expr);
                let new = format!("let {} = {}.cloned().unwrap_or_default();", var, get_expr);
                let replaced = content.replace(&old, &new);
                if replaced != *content {
                    *content = replaced;
                }
            }
        }

        // Step 2: Replace EXPR.get(KEY).as_str() inline patterns
        // Pattern: var.get("key").as_str() → var.get("key").map(|s| s.as_str()).unwrap_or("")
        if let Some(re) = cached_regex(r#"(\w+\.get\("[^"]+"\))\.as_str\(\)"#) {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let get_expr = caps.get(1).unwrap().as_str();
                    format!("{}.map(|s| s.as_str()).unwrap_or(\"\")", get_expr)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix integer.as_str() → integer.to_string().as_str()
    /// i32/u32 don't have .as_str(), but Auto's str() conversion maps to .as_str().
    fn fix_int_as_str(content: &mut String) {
        // Track which variables are assigned from integer-returning expressions
        // Pattern: let VAR = ... as i32; or let VAR: u32 = ...;
        let mut int_vars = std::collections::HashSet::new();
        if let Some(re) = cached_regex(r"let\s+(\w+)\s*:\s*(u32|i32|usize)\s*=") {
            for caps in re.captures_iter(content) {
                int_vars.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }
        // Also track: let VAR = expr as i32/u32/usize;
        if let Some(re) = cached_regex(r"let\s+(\w+)\s*=\s*[^;]+\s+as\s+(u32|i32|usize)\s*;") {
            for caps in re.captures_iter(content) {
                int_vars.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }
        // Also track: let VAR: u32/i32;
        if let Some(re) = cached_regex(r"let\s+mut\s+(\w+)\s*:\s*(u32|i32|usize)\s*;") {
            for caps in re.captures_iter(content) {
                int_vars.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }

        if int_vars.is_empty() {
            return;
        }

        // Replace VAR.as_str() with format!("{}", VAR).as_str() for integer vars
        for var in &int_vars {
            let pattern = format!("{}.as_str()", var);
            let replacement = format!("format!(\"{{}}\", {}).as_str()", var);
            let new = content.replace(&pattern, &replacement);
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix str.split(X).len() → str.split(X).count()
    /// and str.split(X).get(i) → str.split(X).nth(i)
    /// Rust's Split is an iterator, not a Vec.
    fn fix_split_methods(content: &mut String) {
        // Pattern: VAR.split(X).len() → VAR.split(X).count()
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\.len\(\)") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let arg = caps.get(1).unwrap().as_str();
                    format!(".split({}).count()", arg)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Pattern: VAR.split(X).get(N) → VAR.split(X).nth(N)
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\.get\(([^)]+)\)") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let split_arg = caps.get(1).unwrap().as_str();
                    let get_arg = caps.get(2).unwrap().as_str();
                    format!(".split({}).nth({})", split_arg, get_arg)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Pattern: VAR.split(X)[N] → VAR.split(X).nth(N).unwrap()
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\[(\d+)\]") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let split_arg = caps.get(1).unwrap().as_str();
                    let idx = caps.get(2).unwrap().as_str();
                    format!(".split({}).nth({}).unwrap()", split_arg, idx)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Pattern: VAR.split(X)[VAR2 as usize] → VAR.split(X).nth(VAR2 as usize).unwrap()
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\[(\w+ as usize)\]") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let split_arg = caps.get(1).unwrap().as_str();
                    let idx = caps.get(2).unwrap().as_str();
                    format!(".split({}).nth({}).unwrap()", split_arg, idx)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix common String/&str mismatch patterns.
    fn fix_string_str_mismatches(content: &mut String) {
        // 1. Remove .to_string().as_str() → .as_str()
        let reduced = content.replace(".to_string().as_str()", ".as_str()");
        if reduced != *content {
            *content = reduced;
        }
        // 2. Remove .clone().as_str() → .as_str()
        let reduced = content.replace(".clone().as_str()", ".as_str()");
        if reduced != *content {
            *content = reduced;
        }
        // 3. Remove duplicate .to_string().to_string() → .to_string()
        let reduced = content.replace(".to_string().to_string()", ".to_string()");
        if reduced != *content {
            *content = reduced;
        }
    }

    /// Fix HashMap.keys() used as indexable collection.
    /// Auto: `var keys = map.keys()` returns List<str>, supports keys[i] and keys.len()
    /// Rust: keys() returns an iterator — need to collect into Vec first.
    /// Pattern: `let mut? var = expr.keys()` → `let mut? var: Vec<_> = expr.keys().cloned().collect()`
    fn fix_map_keys_indexing(content: &mut String) {
        // Find all .keys() assignments and check if they're used with indexing or .len()
        if let Some(re) = cached_regex(r"(?m)^(\s+let (?:mut )?)(\w+) = (.+?)\.keys\(\)") {
            let captures: Vec<(usize, String, String, String)> = re
                .captures_iter(content.as_str())
                .filter_map(|caps| {
                    let full = caps.get(0)?;
                    let indent = caps.get(1)?.as_str().to_string();
                    let var = caps.get(2)?.as_str().to_string();
                    let expr = caps.get(3)?.as_str().to_string();
                    Some((full.start(), indent, var, expr))
                })
                .collect();

            // Check which vars are used with indexing [i] or .len()
            for (_pos, indent, var, expr) in captures.iter().rev() {
                // Check if var is used with indexing or .len()
                let idx_pat = format!("{}[", var);
                let len_pat = format!("{}.len()", var);
                let needs_fix = content.contains(&idx_pat) || content.contains(&len_pat);
                if !needs_fix {
                    continue;
                }

                let old_line = format!("{}{} = {}.keys();", indent, var, expr);
                let new_line = format!(
                    "{}{}: Vec<_> = {}.keys().cloned().collect();",
                    indent, var, expr
                );
                *content = content.replace(&old_line, &new_line);

                // After converting to Vec, fix map.get(var[i].clone()) → map.get(&var[i])
                // and map.insert(var[i].clone(), ...) → map.insert(var[i].clone(), ...)
                let get_clone_pat =
                    format!(r"\.get\({}\[([^\]]+)\]\s*\.clone\(\)\)", regex::escape(var));
                if let Some(get_re) = cached_regex(&get_clone_pat) {
                    let new = get_re
                        .replace_all(content.as_str(), |caps: &regex::Captures| {
                            format!(".get(&{}[{}])", var, caps.get(1).unwrap().as_str())
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
            }
        }
    }

    /// Fix Option<String>.unwrap_or("") → Option<String>.unwrap_or_default()
    /// Auto: Option<str>.unwrap_or("") works because "" is str
    /// Rust: Option<String>.unwrap_or("") fails because "" is &str not String
    fn fix_option_unwrap_or_empty(content: &mut String) {
        // Pattern: .unwrap_or("") → .unwrap_or_default()
        // This handles Option<String>.unwrap_or("") → unwrap_or_default()
        if let Some(re) = cached_regex(r#"\.unwrap_or\(""\)"#) {
            let new = re
                .replace_all(content.as_str(), ".unwrap_or_default()")
                .to_string();
            if new != *content {
                *content = new;
            }
        }
        // Pattern: .unwrap_or(vec![]) → .unwrap_or_default()
        if let Some(re) = cached_regex(r"\.unwrap_or\(vec!\[\]\)") {
            let new = re
                .replace_all(content.as_str(), ".unwrap_or_default()")
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix String passed where &_ is expected.
    /// Uses pattern-based matching instead of variable name tracking.
    fn fix_string_to_ref(_content: &mut String) {
        // DON'T blindly add & to all .get(var) — this causes E0277 when var is &str
        // Instead, only fix specific known patterns
        // For now, this is a no-op to avoid regressions
    }

    /// Fix &str assigned to String fields and pushed to Vec<String>.
    /// Pattern 1: `self.field = str_param` where field is String → add .to_string()
    /// Pattern 2: `vec.push(str_param)` where vec is Vec<String> → add .to_string()
    /// Pattern 3: `map.insert(&str_key, ...)` → `map.insert(key.to_string(), ...)`
    fn fix_str_to_string_assignments(content: &mut String) {
        // Line-by-line approach: scan for patterns where &str is used where String is needed.
        // This avoids OOM from repeated regex replacements on the entire file.

        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::with_capacity(content.len());

        // Plan 514 W3-15: &str 参数按**当前函数**作用域收集——原全文件
        // 名字池让跨函数同名局部误伤(短名单 d/e/g/... 撞 loop_exit 的
        // i64 局部 d,`self.loop_depth = d;` 被追加 .to_string() E0308)。
        // 发射产物签名单行,行进时遇 fn 头行重置即可。
        let sig_re = cached_regex(r#"fn \w+\([^)]*\)"#).unwrap();
        let param_re = cached_regex(r#"fn \w+\([^)]*(\w+):\s*&str"#).unwrap();
        let mut str_params = std::collections::HashSet::new();

        for line in &lines {
            let mut new_line = line.to_string();
            if sig_re.is_match(line) {
                str_params.clear();
                for caps in param_re.captures_iter(line) {
                    if let Some(m) = caps.get(1) {
                        str_params.insert(m.as_str().to_string());
                    }
                }
            }

            // Pattern: .push(param) where param is &str → .push(param.to_string())
            for param in &str_params {
                let push_target = format!(".push({})", param);
                let push_replacement = format!(".push({}.to_string())", param);
                if new_line.contains(&push_target) && !new_line.contains(&push_replacement) {
                    new_line = new_line.replace(&push_target, &push_replacement);
                }

                // Pattern: self.field = param; → self.field = param.to_string();
                let assign_target = format!("= {};", param);
                let assign_replacement = format!("= {}.to_string();", param);
                if new_line.contains(&assign_target) && !new_line.contains(&assign_replacement) {
                    // Only apply for self.field or var.field assignments
                    if new_line.contains("self.")
                        || new_line.contains("page.")
                        || new_line.contains("s.")
                    {
                        new_line = new_line.replace(&assign_target, &assign_replacement);
                    }
                }
            }

            result.push_str(&new_line);
            result.push('\n');
        }

        // Remove trailing newline if original didn't have one
        if !content.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        if result != *content {
            *content = result;
        }
    }

    /// Plan 373: lower `s.substring(lo, hi)` to Rust slicing.
    /// Auto str has `.substring(lo, hi)`; Rust str has no such method.
    /// `expr.substring(0, cap)` → `&expr[0..cap as usize]`
    /// `expr.substring(lo, end)` → `&expr[lo as usize..end as usize]`
    /// Both args are integer-typed (i32/u32) in the transpiled output, so cast
    /// to usize for byte slicing. (Byte slicing is adequate — Auto sources use
    /// substring only for truncation/summaries, never on multi-byte boundaries
    /// that would split a char.)
    fn fix_substring_method(content: &mut String) {
        // `IDENT.substring(a, b)` — IDENT is a simple receiver (str value).
        // Plan 380: add a leading `&` ONLY when the slice isn't chained —
        // `let x = s[lo..hi]` would bind an unsized `str` (E0277), while
        // `s[lo..hi].to_string()` works without `&` (and `&s[..].to_string()`
        // would become `&String`, E0308).
        if let Some(re) = cached_regex(r"(\w+)\.substring\(([^,)]+),\s*([^)]+)\)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let m0 = caps.get(0).unwrap();
                    let after_is_dot = content.as_bytes().get(m0.end()) == Some(&b'.');
                    let recv = caps.get(1).unwrap().as_str();
                    let lo = caps.get(2).unwrap().as_str().trim();
                    let hi = caps.get(3).unwrap().as_str().trim();
                    let inner = format!("{}[{} as usize..{} as usize]", recv, lo, hi);
                    if after_is_dot {
                        inner
                    } else {
                        format!("&{}", inner)
                    }
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 373: lower Auto-VM numeric conversion methods to Rust casts.
    /// `expr.to_float()` → `(expr as f64)`
    /// `expr.to_uint()`  → `(expr as u32)`
    /// `expr.to_int()`   → `(expr as i64)`
    /// (Rust ints/floats have no `.to_float()`/`.to_uint()` methods.)
    pub(crate) fn fix_numeric_conversion_methods(content: &mut String) {
        // Match a receiver that is either an identifier or a method chain we can
        // wrap in parens. Keep it conservative: `IDENT.method_chain().to_float()`.
        for (method, cast) in [("to_float", "f64"), ("to_uint", "u32"), ("to_int", "i32")] {
            let pat = format!(r"([\w.()]+)\.{}\(\)", method);
            if let Some(re) = cached_regex(&pat) {
                let mthd = method;
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let recv = caps.get(1).unwrap().as_str();
                        // Avoid double-wrapping if already parenthesized.
                        if recv.starts_with('(') && recv.ends_with(')') {
                            format!("{} as {})", &recv[..recv.len() - 1], cast)
                        } else {
                            format!("({} as {})", recv, cast)
                        }
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
                let _ = mthd;
            }
        }
    }

    /// Plan 373: drop residual `Err(Box::new(...))` wrapping.
    /// a2r sometimes wraps error payloads in `Box::new(...)` even when the
    /// enclosing `Result`'s error type is a plain enum/String (not Box<...>).
    /// `Err(Box::new(X))` → `Err(X)`
    fn fix_residual_error_box(content: &mut String) {
        let needle = "Err(Box::new(";
        let mut out = String::with_capacity(content.len());
        let mut rest: &str = content;
        loop {
            match rest.find(needle) {
                None => {
                    out.push_str(rest);
                    break;
                }
                Some(pos) => {
                    out.push_str(&rest[..pos]);
                    let after = &rest[pos + needle.len()..];
                    // Find the close paren that balances the Box::new( open,
                    // counting nesting (payload may itself contain parens, e.g.
                    // nested format!(...) from `+` concat). The old regex only
                    // handled ONE paren level — nested format! stayed Box::new
                    // and broke `Result<_, String>` fns (E0308 Box<String>).
                    let mut depth = 1i32;
                    let mut close: Option<usize> = None;
                    for (i, ch) in after.char_indices() {
                        match ch {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    close = Some(i);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    // `Err(Box::new(` opens BOTH Err( and Box::new( — the Err's
                    // own close paren immediately follows the Box::new close.
                    // Consume it too: `Err(Box::new(X))` → `Err(X)`.
                    match close {
                        Some(i) => {
                            out.push_str("Err(");
                            out.push_str(&after[..i]);
                            out.push_str(")");
                            let skip = if after[i + 1..].starts_with(')') {
                                i + 2
                            } else {
                                i + 1
                            };
                            rest = &after[skip..];
                        }
                        None => {
                            // Unbalanced — leave the text untouched.
                            out.push_str(needle);
                            rest = after;
                        }
                    }
                }
            }
        }
        if out != *content {
            *content = out;
        }
    }

    const D_PE: u8 = 1; // PartialEq
    const D_EQ: u8 = 2; // Eq
    const D_PO: u8 = 4; // PartialOrd
    const D_OR: u8 = 8; // Ord
    const D_FULL: u8 = Self::D_PE | Self::D_EQ | Self::D_PO | Self::D_OR;

    /// Trait names listed in a type's explicit `#[derive(...)]` attrs.
    /// Attrs without a derive(...) contribute nothing (empty Vec).
    fn derive_traits_from_attrs(attrs: &[AutoStr]) -> Vec<AutoStr> {
        let mut out: Vec<AutoStr> = Vec::new();
        for attr in attrs {
            let a = attr.as_str().trim();
            let Some(inner) = a.strip_prefix("derive(") else {
                continue;
            };
            let Some(inner) = inner.strip_suffix(')') else {
                continue;
            };
            for t in inner.split(',') {
                let t = t.trim();
                if t.is_empty() {
                    continue;
                }
                // Path-qualified traits (std::fmt::Debug) count by last segment.
                let name = t.rsplit("::").next().unwrap_or(t);
                if !out.iter().any(|e| e.as_str() == name) {
                    out.push(name.into());
                }
            }
        }
        out
    }

    fn mask_from_traits(traits: &[AutoStr]) -> u8 {
        let mut mask = 0;
        for t in traits {
            match t.as_str() {
                "PartialEq" => mask |= Self::D_PE,
                "Eq" => mask |= Self::D_EQ,
                "PartialOrd" => mask |= Self::D_PO,
                "Ord" => mask |= Self::D_OR,
                _ => {}
            }
        }
        mask
    }

    /// Comparison-trait mask the shape heuristics (TaskRef/dyn/float/map/
    /// enum-field downgrades — same branches as the TypeDecl handler) would
    /// grant a type with these field/payload types, before explicit-derive
    /// intersection.
    fn shape_base_cmp_mask(field_types: &[&Type]) -> u8 {
        let has_taskref = field_types.iter().any(|ty| Self::type_is_taskref(ty));
        let has_dyn = field_types.iter().any(|ty| Self::type_contains_dyn(ty));
        let has_eq_unsafe = field_types.iter().any(|ty| {
            Self::type_has_float(ty) || Self::type_has_map(ty) || Self::type_contains_enum(ty)
        });
        if has_taskref || has_dyn {
            0
        } else if has_eq_unsafe {
            Self::D_PE
        } else {
            Self::D_FULL
        }
    }

    /// Names of locally declared types reachable inside `ty` (through
    /// List/Option/Result/Reference/Map/GenericInstance wrappers).
    fn collect_local_type_names(ty: &Type, locals: &HashSet<AutoStr>, out: &mut Vec<AutoStr>) {
        match ty {
            Type::User(td) => {
                let n = td.name.clone();
                if locals.contains(&n) && !out.contains(&n) {
                    out.push(n);
                }
            }
            Type::GenericInstance(inst) => {
                for arg in &inst.args {
                    Self::collect_local_type_names(arg, locals, out);
                }
            }
            Type::List(inner)
            | Type::Option(inner)
            | Type::Result(inner)
            | Type::Reference(inner) => {
                Self::collect_local_type_names(inner, locals, out);
            }
            Type::Map(k, v) => {
                Self::collect_local_type_names(k, locals, out);
                Self::collect_local_type_names(v, locals, out);
            }
            _ => {}
        }
    }

    /// Constraint mask imposed by a referenced type name: its explicit
    /// derive surface if declared with attrs, otherwise its (recursively
    /// computed) auto-derived set; unknown/external types impose none.
    fn type_cmp_mask(&mut self, name: &str) -> u8 {
        if let Some(traits) = self.explicit_derive_traits.get(name) {
            return Self::mask_from_traits(traits);
        }
        if self.local_auto_shapes.contains_key(name) {
            return self.auto_cmp_mask(name);
        }
        Self::D_FULL
    }

    /// Memoized, cycle-guarded recursion: the comparison mask an auto-derived
    /// local type ends up with (shape base ∩ referenced types' masks).
    fn auto_cmp_mask(&mut self, name: &str) -> u8 {
        if let Some(m) = self.auto_cmp_mask_memo.get(name) {
            return *m;
        }
        // Cycle guard: assume unconstrained until proven otherwise.
        self.auto_cmp_mask_memo.insert(name.into(), Self::D_FULL);
        let mut mask = Self::D_FULL;
        if let Some((base, refs)) = self
            .local_auto_shapes
            .get(name)
            .map(|(b, r)| (*b, r.clone()))
        {
            mask = base;
            for r in refs {
                mask &= self.type_cmp_mask(&r);
            }
        }
        self.auto_cmp_mask_memo.insert(name.into(), mask);
        mask
    }

    /// Intersect the candidate mask with the constraints of every local type
    /// referenced inside `field_types` (struct members / enum payloads).
    fn field_cmp_mask(&mut self, field_types: &[&Type]) -> u8 {
        let mut mask = Self::D_FULL;
        let mut names: Vec<AutoStr> = Vec::new();
        for ty in field_types {
            Self::collect_local_type_names(ty, &self.local_struct_types, &mut names);
        }
        for n in names {
            mask &= self.type_cmp_mask(&n);
        }
        mask
    }

    /// Rebuild a `#[derive(...)]` attribute keeping only the comparison
    /// traits `mask` allows (other traits and canonical order preserved).
    fn filter_cmp_traits(derive_attr: &str, mask: u8) -> String {
        let inner = derive_attr
            .trim_start_matches("#[derive(")
            .trim_end_matches(")]");
        let mut out = String::from("#[derive(");
        let mut first = true;
        for t in inner.split(',') {
            let t = t.trim();
            let keep = match t {
                "PartialEq" => mask & Self::D_PE != 0,
                "Eq" => mask & Self::D_EQ != 0,
                "PartialOrd" => mask & Self::D_PO != 0,
                "Ord" => mask & Self::D_OR != 0,
                _ => true,
            };
            if keep {
                if !first {
                    out.push_str(", ");
                }
                out.push_str(t);
                first = false;
            }
        }
        out.push_str(")]");
        out
    }

    fn type_has_float(ty: &Type) -> bool {
        match ty {
            Type::Float | Type::Double => true,
            Type::List(inner) | Type::Result(inner) | Type::Option(inner) => {
                Self::type_has_float(inner)
            }
            _ => false,
        }
    }

    fn type_has_map(ty: &Type) -> bool {
        matches!(ty, Type::Map(_, _))
            || matches!(ty, Type::Rust(source) if {
                let name = source.short_name();
                name.starts_with("HashMap") || name.starts_with("BTreeMap")
            })
    }

    fn type_contains_enum(ty: &Type) -> bool {
        match ty {
            Type::Tag(_) | Type::Enum(_) => true,
            // Type::User with empty members is a generic type param (T), not a concrete type
            Type::User(td) if !td.members.is_empty() || !td.generic_params.is_empty() => true,
            Type::GenericInstance(inst) => {
                inst.args.iter().any(|arg| Self::type_contains_enum(arg))
            }
            Type::List(inner) | Type::Result(inner) | Type::Option(inner) => {
                Self::type_contains_enum(inner)
            }
            _ => false,
        }
    }

    fn type_contains_dyn(ty: &Type) -> bool {
        match ty {
            Type::User(td) => td.name.starts_with("dyn "),
            Type::GenericInstance(inst) => inst.args.iter().any(|arg| Self::type_contains_dyn(arg)),
            Type::List(inner)
            | Type::Result(inner)
            | Type::Option(inner)
            | Type::Reference(inner) => Self::type_contains_dyn(inner),
            _ => false,
        }
    }

    fn type_is_taskref(ty: &Type) -> bool {
        matches!(ty, Type::GenericInstance(inst) if inst.base_name == "TaskRef")
    }

    /// Plan 373 G2: fix string-literal args to enum variants that expect String.
    /// e.g. `ToolError::Args("msg")` → `ToolError::Args("msg".to_string())`
    /// Pattern: `Variant("...")` where Variant is Args, Exec, LoopDetected,
    /// Config, Failed, or other common error variant names.
    fn fix_string_literal_enum_args(content: &mut String) {
        for variant in &["Args", "Exec", "Config", "LoopDetected", "Failed"] {
            let pat = format!(r#"{}\("([^"]*)"\)"#, variant);
            if let Some(re) = cached_regex(&pat) {
                let var = *variant;
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let msg = caps.get(1).unwrap().as_str();
                        format!("{}(\"{}\".to_string())", var, msg)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
        }
    }

    /// Plan 373 G2: add missing use imports for common crate types referenced
    /// in trait impl blocks. Single-file transpile doesn't know about the
    /// assembled crate's module layout.
    fn fix_missing_trait_impl_uses(content: &mut String) {
        for (type_name, use_stmt) in [
            ("JsonValue", "use crate::wire::JsonValue;"),
            ("ToolError", "use crate::error::ToolError;"),
            ("AgentError", "use crate::error::AgentError;"),
        ] {
            // Skip if already imported via `use crate::wire::JsonValue;` or
            // via `use wire: ..., JsonValue, ...` (Auto import syntax).
            //
            // Plan 018: the normal `use error: AgentError` translation emits the
            // brace form `use crate::error::{AgentError};`, which a plain
            // `contains(use_stmt)` (no braces) misses — causing a duplicate
            // injection and E0252. Derive and also check the brace form.
            let brace_form = use_stmt
                .strip_suffix(";")
                .map(|s| {
                    if let Some(pos) = s.rfind("::") {
                        format!("{}::{{{}}};", &s[..pos], &s[pos + 2..])
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_else(|| use_stmt.to_string());
            let already_via_rust = content.contains(use_stmt) || content.contains(&brace_form);
            let already_via_auto = content.contains(&format!(": {}", type_name))
                || content.contains(&format!(", {}", type_name))
                || content.contains(&format!("{} ,", type_name));
            if content.contains(type_name) && !already_via_rust && !already_via_auto {
                if let Some(pos) = content.find("use a2r_std::*;") {
                    content.insert_str(pos + "use a2r_std::*;".len(), &format!("\n{}", use_stmt));
                }
            }
        }
    }

    /// Plan 376 Pass 4: Fix `for x in self.field` → `for x in &self.field`
    /// when the enclosing method is `&self` (not `&mut self`). Without this,
    /// iterating a Vec field of `&self` causes E0507 (cannot move out of self.field).
    fn fix_for_in_self_field_borrow(content: &mut String) {
        let re = cached_regex(r"for\s+(\w+)\s+in\s+self\.(\w+)\s*\{");
        if let Some(re) = re {
            let mut new_content = content.clone();
            let mut offset = 0;
            for caps in re.captures_iter(content.as_str()) {
                let full = caps.get(0).unwrap();
                let var = caps.get(1).unwrap().as_str();
                let field = caps.get(2).unwrap().as_str();
                let before = &content[..full.start()];
                let fn_line = before.rfind("fn ").map(|pos| &content[pos..full.start()]);
                let is_mut = fn_line.map_or(false, |line| line.contains("&mut self"));
                if !is_mut {
                    let old = format!("for {} in self.{} {{", var, field);
                    let new = format!("for {} in &self.{} {{", var, field);
                    let pos = new_content[offset..].find(&old);
                    if let Some(p) = pos {
                        let abs = offset + p;
                        new_content.replace_range(abs..abs + old.len(), &new);
                        offset = abs + new.len();
                    }
                }
            }
            if new_content != *content {
                *content = new_content;
            }
        }
    }

    /// Plan 376 Pass 2: Fix `.get(key).field` → `.get(key).unwrap().field`
    /// (HashMap.get returns Option, not the value directly).
    fn fix_option_get_field_access(content: &mut String) {
        let safe_methods = [
            "is_some",
            "is_none",
            "unwrap",
            "unwrap_or",
            "unwrap_or_default",
            "map",
            "and_then",
            "unwrap_or_else",
            "as_ref",
            "as_deref",
            "copied",
            "cloned",
            "ok",
            "err",
            "iter",
            "into_iter",
            "as_mut",
            // Plan 610 ⑥: Option has expect too — injecting .unwrap() before
            // it broke libloading Symbol resolution (`lib().get(b"..").expect(..)`).
            "expect",
        ];
        if let Some(re) = cached_regex(r"\.get\(([^)]+)\)\.(\w+)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let key = caps.get(1).unwrap().as_str();
                    let field = caps.get(2).unwrap().as_str();
                    if safe_methods.contains(&field) {
                        return format!(".get({}).{}", key, field);
                    }
                    format!(".get({}).unwrap().{}", key, field)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376 Pass 2b: Fix a2r_std::fs function return type mismatches.
    /// read_to_string returns String (not Result); wrap in Ok() so match works.
    fn fix_a2r_std_fs_result_patterns(content: &mut String) {
        // Plan 376W: use non-greedy .*? to handle nested parens in args
        // (e.g. path.to_str().unwrap() contains ')' which broke the old [^)]*).
        let re = cached_regex(
            r"match\s+(a2r_std::fs::(?:read_to_string|write|read_dir|exists|is_dir)\(.*?\))\s*\{",
        );
        if let Some(re) = re {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let call = caps.get(1).unwrap().as_str();
                    // Plan 380: the bridged read_to_string returns String (not
                    // Result), so we fake a Result with Ok(...) for the match.
                    // Annotate the error type (the Err arm is dead) — bare
                    // `Ok(x)` can't infer E (E0282).
                    format!("match Ok::<String, std::io::Error>({}) {{", call)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376V: Box spec-trait constructors in functions returning Box<dyn Trait>.
    /// `Some(Assistant())` → `Some(Box::new(Assistant()))` when the enclosing
    /// function returns Option<Box<dyn Role>> (or similar spec-trait). Auto's
    /// `has Role` spec means a concrete `Assistant` value must be boxed to
    /// satisfy `Box<dyn Role>`. Without this, `Some(Assistant {})` fails E0308
    /// (expected Box<dyn Role>, found Assistant).
    fn fix_spec_trait_boxing(content: &mut String) {
        // Find functions returning Option<Box<dyn <Trait>>> and box their
        // Some(Constructor()) returns. The signature pattern:
        //   fn name(...) -> Option<Box<dyn Trait>> {
        // Match the trait name so we only box inside the right functions.
        if let Some(re) =
            cached_regex(r"(?ms)(fn \w+\([^)]*\)[^{]*->\s*Option<Box<dyn (\w+)>>\s*\{)(.*?)(^\})")
        {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let header = caps.get(1).unwrap().as_str();
                    let body = caps.get(3).unwrap().as_str();
                    let close = caps.get(4).unwrap().as_str();
                    // Wrap Some(PascalCase {}) → Some(Box::new(PascalCase {}))
                    // (escaped \) — the unescaped form panicked with "unopened
                    // group" and silently dropped the builtin_roles module).
                    let body_re = regex::Regex::new(r"Some\((\w+)\s*\{\s*\}\)").unwrap();
                    let new_body = body_re.replace_all(body, "Some(Box::new($1 {}))");
                    format!("{}{}{}", header, new_body, close)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376V: Convert tuple indexing pair[0]/pair[1] → pair.0/pair.1.
    /// Auto uses [] for both list and tuple access, but Rust tuples need .N.
    fn fix_tuple_index(content: &mut String) {
        if let Some(re) = cached_regex(r"\bpair\[(\d+)\]") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    format!("pair.{}", caps.get(1).unwrap().as_str())
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

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
    fn collect_fn_spec_params(
        stmts: &[Stmt],
        type_name: &str,
        map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
    ) {
        let generic_methods = [
            "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter",
            "keys", "values", "clone", "new", "update", "delete", "find", "index",
        ];
        let scan_fn =
            |fn_decl: &Fn,
             parent: &str,
             map: &mut std::collections::HashMap<AutoStr, Vec<bool>>| {
                let spec_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::Spec(_)))
                    .collect();
                if !spec_flags.is_empty() {
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        map.insert(fn_decl.name.clone(), spec_flags.clone());
                    }
                    if !parent.is_empty() || fn_decl.parent.is_some() {
                        let p = fn_decl
                            .parent
                            .as_ref()
                            .map(|x| x.to_string())
                            .unwrap_or_else(|| parent.to_string());
                        let qualified = format!("{}.{}", p, fn_decl.name);
                        map.insert(AutoStr::from(qualified), spec_flags);
                    }
                }
            };
        for stmt in stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                scan_fn(fn_decl, type_name, map);
            }
            if let Stmt::TypeDecl(type_decl) = stmt {
                for method in &type_decl.methods {
                    scan_fn(method, &type_decl.name.to_string(), map);
                }
            }
            if let Stmt::Ext(ext) = stmt {
                for method in &ext.methods {
                    scan_fn(method, &ext.target.to_string(), map);
                }
            }
        }
    }

    // Helper: collect Fn declarations from statements, including methods inside TypeDecl
    fn collect_fn_str_params(
        stmts: &[Stmt],
        type_name: &str,
        map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
    ) {
        // Generic method names that should never be stored as bare-name keys
        // to avoid false positive .as_str() on unrelated calls
        let generic_methods = [
            "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter",
            "keys", "values", "clone", "new", "update", "delete", "find", "index",
        ];
        for stmt in stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                let str_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                    .collect();
                if !str_flags.is_empty() {
                    // Only store bare name for non-generic method names
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        map.insert(fn_decl.name.clone(), str_flags.clone());
                    }
                    // Always store qualified key "TypeName.method_name" for methods
                    if !type_name.is_empty() || fn_decl.parent.is_some() {
                        let parent = fn_decl
                            .parent
                            .as_ref()
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| type_name.to_string());
                        let qualified = format!("{}.{}", parent, fn_decl.name);
                        map.insert(AutoStr::from(qualified), str_flags);
                    }
                }
            }
            // Also scan inside type declarations for methods
            if let Stmt::TypeDecl(type_decl) = stmt {
                let type_name_str = type_decl.name.to_string();
                for method in &type_decl.methods {
                    let str_flags: Vec<bool> = method
                        .params
                        .iter()
                        .map(|p| {
                            matches!(p.ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_))
                        })
                        .collect();
                    if !str_flags.is_empty() {
                        if !generic_methods.contains(&method.name.as_str()) {
                            map.insert(method.name.clone(), str_flags.clone());
                        }
                        let qualified = format!("{}.{}", type_name_str, method.name);
                        map.insert(AutoStr::from(qualified), str_flags);
                    }
                }
            }
        }
    }

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

    fn collect_fn_str_params(
        stmts: &[Stmt],
        type_name: &str,
        map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
    ) {
        let generic_methods = [
            "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter",
            "keys", "values", "clone", "new", "update", "delete", "find", "index",
        ];
        for stmt in stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                let str_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                    .collect();
                if !str_flags.is_empty() {
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        map.insert(fn_decl.name.clone(), str_flags.clone());
                    }
                    if !type_name.is_empty() || fn_decl.parent.is_some() {
                        let parent = fn_decl
                            .parent
                            .as_ref()
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| type_name.to_string());
                        let qualified = format!("{}.{}", parent, fn_decl.name);
                        map.insert(AutoStr::from(qualified), str_flags);
                    }
                }
            }
            if let Stmt::TypeDecl(type_decl) = stmt {
                let type_name_str = type_decl.name.to_string();
                for method in &type_decl.methods {
                    let str_flags: Vec<bool> = method
                        .params
                        .iter()
                        .map(|p| {
                            matches!(p.ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_))
                        })
                        .collect();
                    if !str_flags.is_empty() {
                        if !generic_methods.contains(&method.name.as_str()) {
                            map.insert(method.name.clone(), str_flags.clone());
                        }
                        let qualified = format!("{}.{}", type_name_str, method.name);
                        map.insert(AutoStr::from(qualified), str_flags);
                    }
                }
            }
        }
    }

    // Plan 390 §11 Phase E: spec-param collector (mirrors collect_fn_str_params
    // but for spec auto-boxing). Covers Fn / TypeDecl methods / Ext methods — the
    // last is a separate Stmt::Ext not reached by the TypeDecl scan.
    fn collect_fn_spec_params(
        stmts: &[Stmt],
        type_name: &str,
        map: &mut std::collections::HashMap<AutoStr, Vec<bool>>,
    ) {
        let generic_methods = [
            "get", "set", "insert", "push", "remove", "contains", "len", "is_empty", "iter",
            "keys", "values", "clone", "new", "update", "delete", "find", "index",
        ];
        let scan_fn =
            |fn_decl: &Fn,
             parent: &str,
             map: &mut std::collections::HashMap<AutoStr, Vec<bool>>| {
                let spec_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::Spec(_)))
                    .collect();
                if !spec_flags.is_empty() {
                    if !generic_methods.contains(&fn_decl.name.as_str()) {
                        map.insert(fn_decl.name.clone(), spec_flags.clone());
                    }
                    if !parent.is_empty() || fn_decl.parent.is_some() {
                        let p = fn_decl
                            .parent
                            .as_ref()
                            .map(|x| x.to_string())
                            .unwrap_or_else(|| parent.to_string());
                        let qualified = format!("{}.{}", p, fn_decl.name);
                        map.insert(AutoStr::from(qualified), spec_flags);
                    }
                }
            };
        for stmt in stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                scan_fn(fn_decl, type_name, map);
            }
            if let Stmt::TypeDecl(type_decl) = stmt {
                for method in &type_decl.methods {
                    scan_fn(method, &type_decl.name.to_string(), map);
                }
            }
            if let Stmt::Ext(ext) = stmt {
                for method in &ext.methods {
                    scan_fn(method, &ext.target.to_string(), map);
                }
            }
        }
    }

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

/// Post-processing passes specific to merged mode output.
/// These handle cross-file issues that arise when concatenating modules.
fn post_process_merged(body: &mut Vec<u8>) {
    let content = String::from_utf8(std::mem::take(body)).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Track seen definitions for deduplication
    // Note: struct/enum dedup is handled at AST level in transpile_rust_project_merged
    let mut seen_allow = false;
    let mut seen_uses: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_top_level_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut brace_depth: i32 = 0;

    let mut result = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Skip duplicate #![allow(...)] pragmas
        if trimmed.starts_with("#![allow(") {
            if seen_allow {
                i += 1;
                continue;
            }
            seen_allow = true;
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Skip duplicate use statements (any form)
        if trimmed.starts_with("use ")
            || trimmed.starts_with("#[allow(unused_imports)]")
                && i + 1 < lines.len()
                && lines[i + 1].trim().starts_with("use ")
        {
            let use_line = if trimmed.starts_with("#[allow") {
                // Skip the #[allow(unused_imports)] annotation line too
                let actual_use = lines[i + 1].trim();
                if seen_uses.contains(actual_use) {
                    i += 2;
                    continue; // skip both annotation and use
                }
                seen_uses.insert(actual_use.to_string());
                result.push_str(line);
                result.push('\n');
                i += 1;
                continue;
            } else {
                trimmed
            };
            if seen_uses.contains(use_line) {
                i += 1;
                continue;
            }
            seen_uses.insert(use_line.to_string());
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Skip duplicate const definitions (OP_*, BOOL_*, NATIVE_*, etc.)
        if trimmed.starts_with("const ") && trimmed.ends_with(';') {
            // Extract const name: "const OP_POP: i32 = 1;" → "OP_POP"
            if let Some(name) = trimmed.strip_prefix("const ") {
                if let Some(colon_pos) = name.find(':') {
                    let const_name = &name[..colon_pos];
                    if seen_top_level_names.contains(const_name) {
                        i += 1;
                        continue;
                    }
                    seen_top_level_names.insert(const_name.to_string());
                }
            }
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Skip duplicate fn definitions (OP_*, BOOL_*, NATIVE_*, etc.)
        // Only at top level (brace_depth == 0), not inside impl blocks
        if trimmed.starts_with("fn ") && brace_depth == 0 {
            // Extract fn name: "fn OP_POP() -> i32 {" → "OP_POP"
            if let Some(rest) = trimmed.strip_prefix("fn ") {
                if let Some(paren_pos) = rest.find('(') {
                    let fn_name = &rest[..paren_pos];
                    if seen_top_level_names.contains(fn_name) {
                        // Skip the entire function body and update brace_depth
                        for ch in trimmed.chars() {
                            match ch {
                                '{' => brace_depth += 1,
                                '}' => brace_depth -= 1,
                                _ => {}
                            }
                        }
                        while i + 1 < lines.len() && brace_depth > 0 {
                            i += 1;
                            for ch in lines[i].chars() {
                                match ch {
                                    '{' => brace_depth += 1,
                                    '}' => brace_depth -= 1,
                                    _ => {}
                                }
                            }
                        }
                        i += 1;
                        continue;
                    }
                    seen_top_level_names.insert(fn_name.to_string());
                }
            }
            result.push_str(line);
            result.push('\n');
            for ch in trimmed.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }
            i += 1;
            continue;
        }

        // Skip duplicate type aliases: "type X = ..."
        if trimmed.starts_with("type ") && trimmed.contains('=') {
            if let Some(rest) = trimmed.strip_prefix("type ") {
                let name = rest
                    .split(|c: char| c == '=' || c == '<' || c == ' ')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    if seen_top_level_names.contains(name) {
                        i += 1;
                        continue;
                    }
                    seen_top_level_names.insert(name.to_string());
                }
            }
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Keep all other lines
        result.push_str(line);
        result.push('\n');
        // Track brace depth for fn dedup scope detection
        for ch in trimmed.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }
        i += 1;
    }

    *body = result.into_bytes();
}

/// Apply regex-based fixes to merged output, mirroring the Python post-processing scripts.
/// Only deterministic, pattern-based fixes are applied here. Fragile flow-sensitive fixes
/// (borrow2, clone, push_clone, move_after_field) have been removed — they require AST-level
/// analysis that text-based regex processing cannot do reliably.
fn apply_merged_regex_fixes(body: &mut Vec<u8>) {
    let mut content = String::from_utf8(std::mem::take(body)).unwrap();

    // === fix_cross_file.py ===
    // int_to_str(kind) -> int_to_str(kind as i64): partially at AST level (needs_enum_cast)
    // Still needed as fallback for cases where local_var_types has User(NodeKind) instead of Enum
    content = content.replace("int_to_str(kind)", "int_to_str(kind as i64)");
    // String + String: (output + int_to_str(val)) -> (output + &int_to_str(val))
    content = content.replace("(output + int_to_str(val))", "(output + &int_to_str(val))");
    // prefix + a2r_expr(...) -> prefix + &a2r_expr(...)
    let re = cached_regex(r"prefix \+ (a2r_expr\([^)]+\))").unwrap();
    content = re.replace_all(&content, "prefix + &$1").to_string();
    // return left == right; -> return if left == right { 1 } else { 0 };
    for op in &["==", "!=", "<", ">", "<=", ">="] {
        let old = format!("return left {} right;", op);
        let new = format!("return if left {} right {{ 1 }} else {{ 0 }};", op);
        content = content.replace(&old, &new);
    }
    // tenv clone at cross-file call sites: no longer needed in merge mode
    // since TypeEnv is now &mut TypeEnv (auto-reborrow handles multiple calls)
    // node.name partial move fix
    content = content.replace(
        "let mut callee_name = node.name;",
        "let mut callee_name = node.name.clone();",
    );
    // str_to_int arithmetic fix
    let re = cached_regex(r#"result = format!\("\{\}\{\}", result \* 10, ch - 48\)"#).unwrap();
    content = re
        .replace_all(&content, "result = result * 10 + (ch - 48)")
        .to_string();
    // Allow overflowing literals
    if !content.contains("#![allow(overflowing_literals)]") {
        content = format!("#![allow(overflowing_literals)]\n{}", content);
    }

    // === fix_misc.py ===
    // nil_node() in match arms: remove trailing semicolon
    content = content.replace(
        "=> { p.pos = p.pos + 1; nil_node(); }",
        "=> { p.pos = p.pos + 1; nil_node() }",
    );
    // Option.drop() -> Option.take(): now handled at AST level (method name mapping)
    // Fix int_to_str(X).cloned().unwrap_or_default()
    let re = cached_regex(r"int_to_str\(([^)]+)\)\.cloned\(\)\.unwrap_or_default\(\)").unwrap();
    content = re.replace_all(&content, "int_to_str($1)").to_string();
    // NodeKind derives: Copy from AST level, Eq/Ord for ASTNode derive compatibility
    content = content.replace(
        "#[derive(Clone, Debug, PartialEq)]\nenum NodeKind",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]\nenum NodeKind",
    );
    // else_if.value partial move fix
    content = content.replace(
        "else_str = else_if.value;\n            else_body.push(else_if)",
        "else_str = else_if.value.clone();\n            else_body.push(else_if)",
    );
    // fn_defs type: NOW HANDLED AT SOURCE LEVEL — eval.at uses Map<str, ASTNode>
    // But .to_string() on insert args is still generated by a2r for non-primitive Map values.
    // Fix: (stmt).to_string() -> stmt.clone() for ASTNode-typed map inserts
    content = content.replace(
        "env.fn_defs.insert(stmt.name.to_string(), (stmt).to_string())",
        "env.fn_defs.insert(stmt.name.to_string(), stmt.clone())",
    );
    content = content.replace(
        "env.fn_defs.insert(node.name.to_string(), (node).to_string())",
        "env.fn_defs.insert(node.name.to_string(), node.clone())",
    );

    // === fix_param_vec.py ===
    // Add NodeKind::Param variant
    content = content.replace("MoveExpr = 33,\n}", "MoveExpr = 33,\n    Param = 34,\n}");
    // Add to Display impl
    content = content.replace(
        "NodeKind::NilNode => write!(f, \"NilNode\")",
        "NodeKind::Param => write!(f, \"Param\"),\n            NodeKind::NilNode => write!(f, \"NilNode\")",
    );
    // Add to from_str match
    content = content.replace(
        "\"NilNode\" | \"nilnode\" => NodeKind::NilNode",
        "\"Param\" | \"param\" => NodeKind::Param,\n            \"NilNode\" | \"nilnode\" => NodeKind::NilNode",
    );
    // Convert push(Param { name: x, type_name: y }) to push(ASTNode { ... })
    let re = cached_regex(r#"\.push\(Param \{ name: ([^,]+), type_name: ([^}]+) \}\)"#).unwrap();
    content = re.replace_all(&content, ".push(ASTNode { kind: NodeKind::Param, name: $1, type_name: $2, value: \"\".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: \"\".to_string(), params: empty_list(), cond: empty_list(), else_body: empty_list() })").to_string();

    // === fix_return_types.py ===
    content = content.replace(
        "fn tokenize_list(mut source: &str) {",
        "fn tokenize_list(mut source: &str) -> Vec<Token> {",
    );
    content = content.replace(
        "fn lex_fstr_backtick(mut source: &str, mut pos: i32) {",
        "fn lex_fstr_backtick(mut source: &str, mut pos: i32) -> Vec<Token> {",
    );
    content = content.replace(
        "fn lex_fstr_f(mut source: &str, mut pos: i32) {",
        "fn lex_fstr_f(mut source: &str, mut pos: i32) -> Vec<Token> {",
    );
    // Plan 434: v1 bootstrap self-test main 追加已清退(433 KNOWN-DEBT 条目)
    // ——v1 lib 仅存于 lib-legacy 封存与 99_bootstrap 的 #[ignore] 测试,
    // 不再为 merge 产物注入自测 main;v2 lib 无 main 语义保持。

    // === fix_contains_key.py: now handled at AST level (contains_rust logic + cross-module struct_field_types) ===

    // === fix_vec_get.py: AST level covers most cases, regex catches remaining 2 edge cases ===
    let re = cached_regex(r"p\.tokens\.get\(([^)]+)\)").unwrap();
    content = re
        .replace_all(&content, "p.tokens[$1 as usize].clone()")
        .to_string();
    let re = cached_regex(r"\bcode\.get\(([^)]+)\)").unwrap();
    content = re
        .replace_all(&content, "code[$1 as usize].clone()")
        .to_string();

    // === fix_usize_insert.py ===
    // .insert(arith_expr, -> .insert((arith_expr) as usize,
    let re = cached_regex(r"\.insert\(([^,]+),").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            let idx = caps[1].trim().to_string();
            if idx.contains("as usize")
                || idx.starts_with('"')
                || idx.contains(".to_string()")
                || idx.starts_with('&')
            {
                caps[0].to_string()
            } else if idx.chars().any(|c| "+-*/%".contains(c)) {
                format!(".insert(({}) as usize,", idx)
            } else {
                caps[0].to_string()
            }
        })
        .to_string();

    // === fix_hashmap_get.py ===
    // Replace .get(expr) with .get(&expr).cloned().unwrap_or_default() for HashMap types.
    // Only applies to known HashMap field names to avoid corrupting Vec .get() calls.
    let hashmap_fields = [
        "struct_fields",
        "fn_param_types",
        "fn_defs",
        "globals",
        "type_aliases",
        "scopes",
        "strings",
        "state",
        "env",
    ];
    for field in &hashmap_fields {
        // env.field.get(X) pattern
        let pat = cached_regex(&format!(r"env\.{}\.get\(([^)]+)\)", regex::escape(field))).unwrap();
        content = pat
            .replace_all(&content, |caps: &regex::Captures| {
                let arg = caps[1].trim();
                let key_expr = if arg.starts_with('"') || arg.starts_with("c\"") {
                    arg.to_string()
                } else if arg.contains("format!") || arg.contains("to_string") {
                    format!("&{}", arg)
                } else {
                    format!("&*{}", arg)
                };
                format!(
                    "env.{}.get({}).cloned().unwrap_or_default()",
                    field, key_expr
                )
            })
            .to_string();
        // bare field.get(X) pattern (when field is a local variable)
        let pat = cached_regex(&format!(r"\b{}\.get\(([^)]+)\)", regex::escape(field))).unwrap();
        content = pat
            .replace_all(&content, |caps: &regex::Captures| {
                let arg = caps[1].trim();
                // Skip Vec-style .get() with 'as usize' index
                if arg.contains("as usize") {
                    return caps[0].to_string();
                }
                let key_expr = if arg.starts_with('"') || arg.starts_with("c\"") {
                    arg.to_string()
                } else if arg.contains("format!") || arg.contains("to_string") {
                    format!("&{}", arg)
                } else {
                    format!("&*{}", arg)
                };
                format!("{}.get({}).cloned().unwrap_or_default()", field, key_expr)
            })
            .to_string();
    }

    // === fix_misc: void functions return 0 -> return ===
    // AST level handles top-level return 0 in void functions, but if-block returns need regex
    for fn_name in &[
        "codegen_expr",
        "codegen_stmt",
        "type_infer_stmts",
        "codegen_call",
        "codegen_binop",
        "codegen_unary",
        "a2r_transpile",
    ] {
        let fn_pattern = format!("fn {}(", fn_name);
        if let Some(pos) = content.find(&fn_pattern) {
            if let Some(brace_pos) = content[pos..].find('{') {
                let abs_brace = pos + brace_pos;
                let mut depth = 1i32;
                let mut end = abs_brace + 1;
                let bytes = content.as_bytes();
                while end < bytes.len() && depth > 0 {
                    match bytes[end] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    end += 1;
                }
                let body = &content[abs_brace + 1..end - 1];
                let fixed_body = body.replace("return 0;", "return;");
                if body != fixed_body {
                    content = format!(
                        "{}{}{}{}",
                        &content[..abs_brace + 1],
                        fixed_body,
                        &content[end - 1..],
                        ""
                    );
                }
            }
        }
    }

    // === OP_XXX {} -> OP_XXX() now handled at AST level (is_screaming_case check) ===

    // === crate:: path fixes now handled at AST level in qualify_type_name ===
    // (merge_mode skips crate:: prefix generation)

    // === env.scopes type: NOW HANDLED AT SOURCE LEVEL — eval.at uses List<Map<str, str>> ===

    // === use auto_lang::a2r_std now handled at AST level (merge_mode skips emit) ===

    // === CONST_NAME() -> CONST_NAME now handled at AST level (is_screaming_case check) ===

    // === .to_string().cloned().unwrap_or_default() -> .to_string() (E0599: String not iterator) ===
    content = content.replace(".to_string().cloned().unwrap_or_default()", ".to_string()");
    // Also fix .cloned().unwrap_or_default() on format!() results
    let re = cached_regex(r#"format!\([^)]*\)\.cloned\(\)\.unwrap_or_default\(\)"#).unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            caps[0]
                .trim_end_matches(".cloned().unwrap_or_default()")
                .to_string()
        })
        .to_string();

    // === &&expr -> &expr (E0277: double reference to String) ===
    // state.get(&&"key".to_string()) -> state.get(&"key".to_string())
    // state.get(&&format!(...)) -> state.get(&format!(...))
    // state.get(&&nkey.to_string()) -> state.get(&nkey.to_string())
    // Plan 525 W1 宿主顺修:盲替换 `&&"`→`&"` 会误吃以 && 结尾的字符串
    // 字面量(lib 的 lexer/parser/a2r 各有 "&&" 字面量,aavm2_bin merge
    // 产物被改成 "&";g26 四路 p2≠p4 实证)——改为捕获前导字符,仅当
    // 前导非 `"`(即不是 "…&&" 字面量收尾)才替换
    {
        let re_amp = cached_regex(r#"([^"])&&""#).unwrap();
        content = re_amp.replace_all(&content, "$1&\"").to_string();
    }
    content = content.replace("&&format!", "&format!");
    // Fix &&var.to_string() patterns
    for var in &["nkey", "ekey", "vkey", "name", "key", "skey"] {
        content = content.replace(&format!("&&{}.", var), &format!("&{}.", var));
    }
    // int_to_str(x).cloned().unwrap_or_default() already fixed above, but check again
    let re = cached_regex(r"int_to_str\(([^)]+)\)\.cloned\(\)\.unwrap_or_default\(\)").unwrap();
    content = re.replace_all(&content, "int_to_str($1)").to_string();

    // === Fix Display trait missing fmt method (E0046): now handled at AST level ===
    // a2r generates Display impl with fmt method for all-unit heterogeneous enums
    // content = content.replace(...)

    // === Fix return; in non-void function: no longer needed ===
    // AST level now correctly emits "return 0;" for non-void functions, "return;" for void functions

    // === Fix double .cloned().unwrap_or_default() (E0599: String is not iterator) ===
    // Pattern: .cloned().unwrap_or_default().cloned().unwrap_or_default()
    while content.contains(".cloned().unwrap_or_default().cloned().unwrap_or_default()") {
        content = content.replace(
            ".cloned().unwrap_or_default().cloned().unwrap_or_default()",
            ".cloned().unwrap_or_default()",
        );
    }

    // === Fix &*&* double dereference ===
    content = content.replace("&*&*", "&*");

    // === a2r_std::str_substr -> inline str_substr (E0433) ===
    // Add str_substr function definition before fn main() and replace a2r_std:: prefix
    content = content.replace("a2r_std::str_substr", "str_substr");
    let str_substr_fn = r#"
fn str_substr<S: AsRef<str>>(s: S, start: i32, end: i32) -> String {
    let s = s.as_ref();
    if start < 0 || end <= start || start as usize >= s.len() {
        return String::new();
    }
    let start_usize = start as usize;
    let end_usize = std::cmp::min(end as usize, s.len());
    s[start_usize..end_usize].to_string()
}
"#;
    if let Some(pos) = content.find("\nfn main() {") {
        content = format!("{}{}{}", &content[..pos], str_substr_fn, &content[pos..]);
    }

    // === Fix known &str param functions called with String args (E0308) ===
    // === Fix .get(...).as_str() (E0599: no method as_str on Option) ===
    // .get(X).as_str() -> .get(X).cloned().unwrap_or_default()
    let re = cached_regex(r"\.get\(([^)]+)\)\.as_str\(\)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!(".get({}).cloned().unwrap_or_default()", &caps[1])
        })
        .to_string();

    // === Fix .to_string() after format!() (unnecessary, format! returns String) ===
    // This causes "expected &str, found String" in some contexts
    // Actually, keep it — it's harmless. The real E0308 issue is String where &str expected.

    // === Fix env.globals.get and similar — add & before key (E0308) ===
    // env.globals.get("__last_str__") -> env.globals.get("__last_str__")
    // Already handled by fix_hashmap_get, but some patterns may have been missed.

    // === Fix String where &str expected: specific known patterns ===
    // node.name passed to &str params: need &*node.name or node.name.as_str()
    // Pattern: (node.name) where function expects &str
    // This is too broad for regex. The real fix is AST-level.

    // === Fix .push(var) where var is ASTNode and used later (E0382) ===
    // NOW HANDLED AT AST LEVEL: is_copy_type() check in method call emission.
    // .push(ident) automatically gets .clone() for non-Copy type identifiers.
    // Keeping fix_push_move() as fallback for edge cases.

    // === Fix tokens move in parser_new ===
    // fn parser_new(mut tokens: Vec<Token>) -> Parser { ... tokens ... }
    // tokens is moved into Parser.tokens, but later code uses tokens.len()
    // Fix: use tokens.len() before the move, or clone
    content = content.replace(
        "fn parser_new(mut tokens: Vec<Token>) -> Parser",
        "fn parser_new(mut tokens: Vec<Token>) -> Parser",
    ); // placeholder — actual fix needs AST-level changes

    // === Fix path move: NOW HANDLED AT AST LEVEL ===
    // store() auto-appends .clone() when assigning from non-Copy struct field (e.g., node.name).

    // === Fix nil_node(); in match arms -> nil_node() (E0308: returns () instead of ASTNode) ===
    // Pattern: TokenKind::Break => { p.pos = ...; nil_node(); }
    // Should be: TokenKind::Break => { p.pos = ...; nil_node() }
    content = content.replace(
        "TokenKind::Break => {\n            p.pos = p.pos + 1;\n            nil_node();\n        }",
        "TokenKind::Break => {\n            p.pos = p.pos + 1;\n            nil_node()\n        }",
    );
    content = content.replace(
        "TokenKind::Continue => {\n            p.pos = p.pos + 1;\n            nil_node();\n        }",
        "TokenKind::Continue => {\n            p.pos = p.pos + 1;\n            nil_node()\n        }",
    );

    // === Fix parser_new tokens move (E0382) ===
    // Parser { tokens: tokens, pos: 0, token_count: (tokens.len() as i64) }
    // tokens moved, then tokens.len() used -> swap order or clone
    content = content.replace(
        "Parser { tokens: tokens, pos: 0, token_count: (tokens.len() as i64) }",
        "Parser { pos: 0, token_count: (tokens.len() as i64), tokens: tokens }",
    );

    // === Fix ASTNode: Default not satisfied (E0277) ===
    // env.fn_defs.get(...).cloned().unwrap_or_default() needs ASTNode: Default
    // Already have the long unwrap_or(ASTNode { ... }) replacement, but another instance exists
    content = content.replace(
        "env.fn_defs.get(&*fn_name).cloned().unwrap_or_default()",
        "env.fn_defs.get(&*fn_name).cloned().unwrap_or(ASTNode { kind: NodeKind::NilNode, value: \"\".to_string(), name: \"\".to_string(), children: empty_list(), left: empty_list(), right: empty_list(), op: \"\".to_string(), params: empty_list(), type_name: \"\".to_string(), cond: empty_list(), else_body: empty_list() })",
    );

    // === Fix path = node.name move (E0382) ===
    // Need .clone() since path is used later
    content = content.replace(
        "let mut path: String = node.name;",
        "let mut path: String = node.name.clone();",
    );

    // === Fix state.get(&format!(...)).as_str() still remaining (E0599) ===
    // These specific patterns weren't caught by the general regex
    content = content.replace(
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).as_str()",
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).cloned().unwrap_or_default()",
    );
    content = content.replace(
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).as_str()",
        "state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).cloned().unwrap_or_default()",
    );

    // === Fix state.get(X).to_string() where it returns Option (E0599) ===
    // state.get(&format!(...)).to_string() on Option
    // Line 4504: state.insert(format!(...).to_string(), (s).to_string())
    // The second arg (s).to_string() is wrong - s is already String? Or s is from state.get()?
    // Let me check the specific pattern.

    // === Fix state.get(X) -> need .cloned().unwrap_or_default() for String result (E0308) ===
    // bvm_pop_str_key: return state.get(X) -> return state.get(X).cloned().unwrap_or_default()
    // Already handled by general regex, but some patterns with specific args may have been missed.
    // Fix specific patterns:
    content = content.replace(
        "return state.get(&format!(\"{}{}\", \"s\", int_to_str(sp)));",
        "return state.get(&format!(\"{}{}\", \"s\", int_to_str(sp))).cloned().unwrap_or_default();",
    );
    // ret_str_key = state.get(X) -> state.get(X).cloned().unwrap_or_default()
    content = content.replace(
        "ret_str_key = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp)));",
        "ret_str_key = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp))).cloned().unwrap_or_default();",
    );

    // === Fix bvm_push_str expects &str but gets String (E0308) ===
    // bvm_push_str(state.clone(), String) -> bvm_push_str(state.clone(), &*String)
    // or bvm_push_str(state.clone(), result.as_str())
    content = content.replace(
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).cloned().unwrap_or_default())",
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(sp - 1))).cloned().unwrap_or_default().as_str())",
    );
    content = content.replace(
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).cloned().unwrap_or_default())",
        "bvm_push_str(state.clone(), state.get(&format!(\"{}{}\", \"s\", int_to_str(abs_idx))).cloned().unwrap_or_default().as_str())",
    );

    // === Fix (s).to_string() where s is Option (E0599) ===
    // state.insert(format!(...), (s).to_string()) where s = state.get(...)
    // The s variable holds an Option from state.get(). Need to unwrap.
    // Actually s is assigned earlier as: let mut s = state.get(...)
    // Let me check the specific context.
    content = content.replace(
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), (s).to_string());",
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), s.cloned().unwrap_or_default());",
    );

    // === Fix node.name partial move in eval (E0382) ===
    // let mut callee_name: String = node.name; then node.clone() later
    // node.name moves out of node, then node.clone() fails
    content = content.replace(
        "let mut callee_name: String = node.name;",
        "let mut callee_name: String = node.name.clone();",
    );

    // === Fix path move into str_substr (E0382) ===
    // str_substr(path, 0, 5) -> str_substr(&path, 0, 5) to avoid moving path
    content = content.replace("str_substr(path, 0, 5)", "str_substr(&path, 0, 5)");
    content = content.replace(
        "str_substr(path, 5, (path.len() as i64))",
        "str_substr(&path, 5, (path.len() as i64))",
    );
    content = content.replace(
        "a2r_path_to_rust(str_substr(&path, 5, (path.len() as i64)).as_str())",
        "a2r_path_to_rust(&str_substr(&path, 5, (path.len() as i64)))",
    );

    // === Fix state borrow conflict (E0502) ===
    // let s = state.get(X); ... state.insert(Y, Z);
    // s borrows state immutably, then insert borrows mutably
    // Fix: clone the result of get() to release the borrow
    content = content.replace(
        "let mut s = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp)));",
        "let mut s = state.get(&format!(\"{}{}\", \"s\", int_to_str(sp))).cloned().unwrap_or_default();",
    );
    // Fix: s is now String (not Option), so s.cloned().unwrap_or_default() -> s
    content = content.replace(
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), s.cloned().unwrap_or_default());",
        "state.insert(format!(\"{}{}\", \"s\", int_to_str(abs_idx)).to_string(), s);",
    );
    // callee = node.name; then callee is used as &str -> callee = node.name.clone()
    // But callee might already have been handled. Check specific cases.
    content = content.replace(
        "let mut path: String = node.name;\n    let mut rest = \"\".to_string();",
        "let mut path: String = node.name.clone();\n    let mut rest = \"\".to_string();",
    );

    // === Final cleanup passes (run after all other transforms) ===
    // Fix triple &&& -> &
    content = content.replace("&&&", "&");
    // Fix remaining && -> & (double ref)
    while content.contains("&&") {
        let before = content.len();
        // Only replace && that are before expressions, not logical AND
        // Safe patterns: &&"  &&{  &&var.  &&*  &&format!
        // Plan 525 W1 宿主顺修:同前——字面量收尾保护(前导非 `"`)
        {
            let re_amp2 = cached_regex(r#"([^"])&&""#).unwrap();
            content = re_amp2.replace_all(&content, "$1&\"").to_string();
        }
        content = content.replace("&&format!", "&format!");
        content = content.replace("&&*", "&*");
        for var in &["nkey", "ekey", "vkey", "name", "key", "skey", "fn_name"] {
            content = content.replace(&format!("&&{}.", var), &format!("&{}.", var));
        }
        if content.len() == before {
            break;
        } // no more replacements
    }
    // Fix .to_string().cloned().unwrap_or_default() -> .to_string()
    content = content.replace(".to_string().cloned().unwrap_or_default()", ".to_string()");
    // Fix double .cloned().unwrap_or_default()
    while content.contains(".cloned().unwrap_or_default().cloned().unwrap_or_default()") {
        content = content.replace(
            ".cloned().unwrap_or_default().cloned().unwrap_or_default()",
            ".cloned().unwrap_or_default()",
        );
    }
    // Fix .cloned().unwrap_or_default().unwrap() (unwrap on String)
    content = content.replace(
        ".cloned().unwrap_or_default().unwrap()",
        ".cloned().unwrap_or_default()",
    );
    // Fix .cloned().unwrap_or_default().as_str() (as_str on String)
    // Actually .as_str() on String is fine. But on Option it's not.
    // The .get().as_str() pattern was already fixed above.

    // Fix state.get(X) where X has nested .cloned().unwrap_or_default() inside get arg
    // Pattern: .get(&"str".to_string().cloned().unwrap_or_default())
    // Should be: .get(&"str".to_string())
    let re =
        cached_regex(r#"\.get\((&[^)]+?)\.to_string\(\)\.cloned\(\)\.unwrap_or_default\(\)\)"#)
            .unwrap();
    content = re.replace_all(&content, ".get($1.to_string())").to_string();

    // === Fix .get(X).as_str() where .get returns Option (E0599) ===
    let re = cached_regex(r"state\.get\(([^)]+)\)\.as_str\(\)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("state.get({}).cloned().unwrap_or_default()", &caps[1])
        })
        .to_string();

    // AST-level fn_str_param_indices + .as_str() auto-borrow covers:
    // contains_key(callee), codegen_lookup_elem(vn2), block_node(body_str2),
    // a2r_struct_init(callee), a2r_path_to_rust(path), env.fn_defs.contains_key(callee_name),
    // eval_fn_call(callee_name), codegen_extract_var_name(callee), type_is_cmp_op(op), etc.

    // === E0308: return state.get(X) -> return state.get(X).cloned().unwrap_or_default() ===
    let re = cached_regex(r"return state\.get\((&[^)]+)\);").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            let arg = &caps[1];
            if arg.contains(".cloned()") {
                caps[0].to_string()
            } else {
                format!("return state.get({}).cloned().unwrap_or_default();", arg)
            }
        })
        .to_string();
    let re = cached_regex(r"= state\.get\((&[^)]+)\);").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            let arg = &caps[1];
            if arg.contains(".cloned()") {
                caps[0].to_string()
            } else {
                format!("= state.get({}).cloned().unwrap_or_default();", arg)
            }
        })
        .to_string();

    // === E0308: TokenKind::Break/Continue match arms returning nil_node() instead of Token ===
    // The match is in a function returning Token. nil_node() returns ASTNode, not Token.
    // Need to create a proper Token. This is a parser issue.
    // Pattern: TokenKind::Break => { p.pos = ...; nil_node() }
    // Should be: TokenKind::Break => { p.pos = ...; Token { kind: ..., pos: ..., text: ... } }
    // Too complex for regex — will need AST-level fix. Leave for now.

    // === E0308: entry-point functions pass owned context types to &mut params ===
    // ✅ Now handled at AST level via fn_param_types + is_merge_mut_type()
    // No regex needed — call sites auto-insert &mut for context-type params.

    // === E0499/E0502: double mutable borrows of &mut env in eval functions ===
    // Pattern: some_fn(env, ..., eval_get_last_str(env)...)
    // Fix: extract inner call to a temp variable before the outer call.
    {
        // Helper: extract inner env call to temp variable to avoid double borrow
        let extract_env_tmp = |content: &str, pattern: &str, tmpl: &str| -> String {
            let re = cached_regex(pattern).unwrap();
            re.replace_all(content, tmpl).to_string()
        };

        // eval_bind_str(env, X, eval_get_last_str(env).as_str())
        content = extract_env_tmp(
            &content,
            r#"eval_bind_str\(env, ([^,]+), eval_get_last_str\(env\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
            eval_bind_str(env, $1, __tmp.as_str())"#,
        );

        // eval_set_last_str(env, eval_str_cat(X, eval_get_last_str(env).as_str()).as_str())
        content = extract_env_tmp(
            &content,
            r#"eval_set_last_str\(env, eval_str_cat\(([^,]+), eval_get_last_str\(env\)\.as_str\(\)\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
                eval_set_last_str(env, eval_str_cat($1, __tmp.as_str()).as_str())"#,
        );

        // eval_set_last_str(env, eval_lookup_str_var(env, X).as_str())
        content = extract_env_tmp(
            &content,
            r#"eval_set_last_str\(env, eval_lookup_str_var\(env, (.+?)\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_lookup_str_var(env, $1);
            eval_set_last_str(env, __tmp.as_str())"#,
        );

        // env.globals.insert(X, (eval_get_last_type(env)).to_string())
        content = extract_env_tmp(
            &content,
            r#"env\.globals\.insert\(([^,]+), \(eval_get_last_type\(env\)\)\.to_string\(\)\)"#,
            r#"let __tmp = eval_get_last_type(env);
            env.globals.insert($1, (__tmp).to_string())"#,
        );

        // env.globals.insert(X, (eval_get_last_str(env)).to_string())
        content = extract_env_tmp(
            &content,
            r#"env\.globals\.insert\(([^,]+), \(eval_get_last_str\(env\)\)\.to_string\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
                env.globals.insert($1, (__tmp).to_string())"#,
        );

        // env.output = eval_str_cat(env.output.as_str(), eval_str_cat(eval_get_last_str(env).as_str(), "\n").as_str())
        content = extract_env_tmp(
            &content,
            r#"env\.output = eval_str_cat\(env\.output\.as_str\(\), eval_str_cat\(eval_get_last_str\(env\)\.as_str\(\), "\\n"\)\.as_str\(\)\)"#,
            r#"let __tmp = eval_get_last_str(env);
            env.output = eval_str_cat(env.output.as_str(), eval_str_cat(__tmp.as_str(), "\n").as_str())"#,
        );
    }

    // === Fix partial move: var = struct.field where struct used after (E0382) ===
    // Pattern: else_str = else_if.value; ... else_if.clone()
    // The field access moves the String, then clone() tries to borrow the whole struct
    // Fix: add .clone() to the field access
    content = content.replace(
        "else_str = else_if.value;",
        "else_str = else_if.value.clone();",
    );

    // Ensure trailing newline
    if !content.ends_with('\n') {
        content.push('\n');
    }

    *body = content.into_bytes();
}

/// Split a comma-separated argument string respecting nested parens/brackets.
#[allow(dead_code)]
fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut arg_start = 0;
    let mut depth = 0i32;
    for (k, ch) in s.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                args.push(s[arg_start..k].to_string());
                arg_start = k + 1;
            }
            _ => {}
        }
    }
    args.push(s[arg_start..].to_string());
    args
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

/// PLAN-724 T-06：HTTP 普通动词/流生产者的统一发射——async 上下文发
/// `a2r_std::http::<verb>_async(args).await`（内核 async 面），同步上下文发
/// `a2r_std::http::<verb>(args)`（同步桥接面）。实参按 str 形发射。
fn emit_http_verb_call(
    tr: &mut RustTrans,
    out: &mut impl std::io::Write,
    verb: &str,
    call: &crate::ast::Call,
) -> AutoResult<()> {
    if tr.in_async_ctx.get() {
        write!(out, "a2r_std::http::{}_async(", verb)?;
        for (i, arg) in call.args.args.iter().enumerate() {
            if i > 0 {
                write!(out, ", ")?;
            }
            if let crate::ast::Arg::Pos(expr) = arg {
                tr.expr_as_str(expr, out)?;
            }
        }
        write!(out, ").await")?;
    } else {
        write!(out, "a2r_std::http::{}(", verb)?;
        for (i, arg) in call.args.args.iter().enumerate() {
            if i > 0 {
                write!(out, ", ")?;
            }
            if let crate::ast::Arg::Pos(expr) = arg {
                tr.expr_as_str(expr, out)?;
            }
        }
        write!(out, ")")?;
    }
    Ok(())
}

/// PLAN-724 T-06：链根是否 builder（builder 变量或 http.request(...)）——
/// `let b2 = b.header(..)` 形态的重绑定保持 builder 分型。
fn expr_root_is_builder_var(tr: &RustTrans, e: &Expr) -> bool {
    let mut cur = e;
    loop {
        match cur {
            Expr::Dot(base, _) => cur = base.as_ref(),
            Expr::Ident(name) => return tr.http_builder_vars.contains(name),
            Expr::Call(call) => {
                return match call.name.as_ref() {
                    Expr::Dot(obj, m) => {
                        matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "http")
                            && m.as_str() == "request"
                    }
                    Expr::Ident(n) => n.as_str() == "request",
                    _ => false,
                };
            }
            _ => return false,
        }
    }
}

/// PLAN-724 T-06：表达式根是否 http.request(...)（链式 builder 的 send 分派）。
fn expr_root_is_http_request(e: &Expr) -> bool {
    let mut cur = e;
    loop {
        match cur {
            Expr::Dot(base, _) => cur = base.as_ref(),
            Expr::Call(call) => {
                return match call.name.as_ref() {
                    Expr::Dot(obj, m) => {
                        matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "http")
                            && m.as_str() == "request"
                    }
                    Expr::Ident(n) => n.as_str() == "request",
                    _ => false,
                };
            }
            _ => return false,
        }
    }
}

/// PLAN-724 T-06：for-in 的可迭代是否 HTTP 流消费面（`s.iter()` /
/// `stream_iter(s)` / 直接流生产调用）。
fn for_range_is_http_stream(tr: &RustTrans, range: &Expr) -> Option<bool> {
    // Some(is_async) = 命中流消费面并给出 async 分型。
    if let Expr::Call(call) = range {
        match call.name.as_ref() {
            Expr::Dot(obj, m) if m.as_str() == "iter" => {
                if let Expr::Ident(name) = obj.as_ref() {
                    if tr.http_stream_async_vars.contains(name) {
                        return Some(true);
                    }
                    if tr.http_stream_vars.contains(name) {
                        return Some(false);
                    }
                }
                None
            }
            Expr::Ident(fname) if fname.as_str() == "stream_iter" => {
                if let Some(crate::ast::Arg::Pos(Expr::Ident(name))) = call.args.args.first() {
                    if tr.http_stream_async_vars.contains(name) {
                        return Some(true);
                    }
                    if tr.http_stream_vars.contains(name) {
                        return Some(false);
                    }
                }
                None
            }
            Expr::Dot(obj, m)
                if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "http")
                    && matches!(
                        m.as_str(),
                        "get_stream" | "post_stream" | "post_stream_with_headers"
                    ) =>
            {
                Some(tr.in_async_ctx.get())
            }
            _ => None,
        }
    } else {
        None
    }
}

// ---- PLAN-843 结构拆分：语法族主题模块（原单文件热点，纯搬移） ----
mod call_emit;
mod decl_emit;
mod expr_emit;
mod project_fixups;
mod stmt_emit;
mod ty;
