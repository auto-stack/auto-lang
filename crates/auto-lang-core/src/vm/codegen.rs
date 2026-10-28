use crate::ast::{Closure, Expr, Iter, ParamMode, Stmt, Type, TypeDecl};
use crate::error::SyntaxError;
use crate::error::{AutoError, AutoResult};
// use crate::val::Value; // Removed if not directly used or fix path
use crate::vm::ffi::stdlib::NATIVE_RUST_STDLIB_DISPATCH;
use crate::vm::loader::{Module, RelocEntry, RelocType};
use crate::vm::native::{
    NATIVE_ASSERT, NATIVE_ASSERT_EQ, NATIVE_ASSERT_NE, NATIVE_AUTODOWN_EDITOR_TEXT,
    NATIVE_AUTODOWN_FIND_BLOCK, NATIVE_AUTODOWN_INSERT_TEMPLATE, NATIVE_AUTODOWN_INSERT_TEXT,
    NATIVE_AUTODOWN_PARSE, NATIVE_AUTODOWN_SERIALIZE, NATIVE_AUTODOWN_TEXT,
    NATIVE_CLIPBOARD_FILES_GET, NATIVE_CLIPBOARD_FILES_SET, NATIVE_CLIPBOARD_IMAGE_GET,
    NATIVE_CLIPBOARD_IMAGE_SET, NATIVE_CLIPBOARD_SET_TEXT, NATIVE_CLIPBOARD_TEXT,
    NATIVE_CODE_EDITOR_COPY, NATIVE_CODE_EDITOR_CURSOR_COL, NATIVE_CODE_EDITOR_CURSOR_LINE,
    NATIVE_CODE_EDITOR_CUT, NATIVE_CODE_EDITOR_DELTA, NATIVE_CODE_EDITOR_EDIT,
    NATIVE_CODE_EDITOR_FIND, NATIVE_CODE_EDITOR_FOLD_HIDDEN_COUNT, NATIVE_CODE_EDITOR_FOLD_TOGGLE,
    NATIVE_CODE_EDITOR_LOAD_FILE, NATIVE_CODE_EDITOR_PASTE, NATIVE_CODE_EDITOR_REDO,
    NATIVE_CODE_EDITOR_SAVE, NATIVE_CODE_EDITOR_SCROLL_OFFSET_X,
    NATIVE_CODE_EDITOR_SCROLL_OFFSET_Y, NATIVE_CODE_EDITOR_SCROLL_TO,
    NATIVE_CODE_EDITOR_SELECTION_LEN, NATIVE_CODE_EDITOR_SELECT_ALL, NATIVE_CODE_EDITOR_SET_CURSOR,
    NATIVE_CODE_EDITOR_SET_TEXT, NATIVE_CODE_EDITOR_TEXT, NATIVE_CODE_EDITOR_UNDO,
    NATIVE_CONSOLE_CLEAR, NATIVE_CONSOLE_LINES, NATIVE_CONSOLE_LOG, NATIVE_DIALOG_OPEN,
    NATIVE_DIALOG_SAVE, NATIVE_DIFF_DIRS, NATIVE_DIFF_FILES, NATIVE_DIFF_FILES_WINDOW,
    NATIVE_DIFF_SNAPSHOTS, NATIVE_DND_START, NATIVE_FILE_BASENAME, NATIVE_PRINT_F32,
    NATIVE_PRINT_F64, NATIVE_PRINT_I32, NATIVE_PRINT_STR, NATIVE_PRINT_U64, NATIVE_PRINT_UNIFIED,
    NATIVE_RUNTIME_PANIC, NATIVE_SCROLL_ANCHOR_STATE, NATIVE_SCROLL_BY, NATIVE_SCROLL_CONTROLLER,
    NATIVE_SCROLL_STATE, NATIVE_SCROLL_TO, NATIVE_SCROLL_TO_CHILD, NATIVE_SCROLL_TO_END,
    NATIVE_SCROLL_TO_START, NATIVE_SHELL_ADD_RECENT, NATIVE_SHELL_EXIT, NATIVE_SHELL_EXPORT,
    NATIVE_SHELL_SYSTEM, NATIVE_SHELL_SYSTEM_STATUS, NATIVE_WRITE_STR,
};
use crate::vm::native_registry::BIGVM_NATIVES;
use crate::vm::opcode::OpCode;

/// Plan 317: Check whether a function body contains a `yield` expression.
/// Used to avoid misclassifying wrapper functions like
/// `fn get() ~Iter<int> { return counter() }` (no yield of their own) as
/// generators — which would short-circuit them and skip the return. Matches
/// the runtime is_generator_fn byte-scan (which looks for YIELD_VAL opcodes).
/// Uses Debug-format substring scan for simplicity (the `Yield` token is
/// unambiguous in the AST Debug output).
fn body_contains_yield(body: &crate::ast::Body) -> bool {
    format!("{:?}", body).contains("Yield")
}

// Plan 076 Phase 1: Generic type support
use crate::vm::generic::{extract_generic_instance, GenericTable};
// Plan 076 Phase 2: Monomorphization support
use crate::vm::monomorphize::{MonomorphizedModule, Monomorphizer};
// Plan 087 Phase 3: Use infer module for type inference
use crate::infer::{infer_expr, InferenceContext};
// Plan 084 Phase 3: Unified TypeStore for type management
use crate::types;
use auto_val::Op;
use miette::SourceSpan;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

// Plan 216 Phase 2: Global C-FFI runtime instance.
// Used by codegen's handle_c_import to register C functions, and by the VM
// engine to merge the shims into its NativeInterface at startup.
lazy_static::lazy_static! {
    pub static ref CFFI_GLOBAL: Mutex<crate::vm::ffi::c_ffi::CFfiRuntime> =
        Mutex::new(crate::vm::ffi::c_ffi::CFfiRuntime::new());
}

/// Debug logging macro - only prints when VM debug mode is enabled
macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if crate::is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

/// Plan 073: Type tags for object field values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Int,
    Uint,
    Float,
    Double,
    String,
    Bool,
    Char,
    Byte, // Plan 118: Byte type for hex formatting
    Void, // Plan 118 Phase 4: Void type for functions that don't return a value
    // Plan 073: Nested types for object/array fields
    NestedObject,
    Array,
}

/// Plan 193: Precise source type for .to() conversion opcode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ConvSrcType {
    I32,
    I64,
    U64,
    F32,
    F64,
    Bool,
    Str,
    Other,
}

/// Plan 073: Type information for TypeDecl
/// Stores type metadata needed for instance construction
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub _name: String,             // prefixed with _ to fix unused warning
    pub member_names: Vec<String>, // Member names in order
}

/// Plan 088 Phase 4: Parameter information for function signatures
/// Stores parameter type and mode for smart parameter passing
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub ty: Type,
    pub mode: ParamMode,
}

/// Plan 340: Metadata for an API function whose bare call should be rewritten
/// to an HTTP request in VM+VM split mode.
#[derive(Clone)]
pub struct ApiCallInfo {
    pub fn_name: String,            // 契约函数名(Plan 060 M3:宿主桥按此名注册)
    pub method: String,             // "GET" | "POST" | "PUT" | "DELETE"
    pub path: String,               // "/api/notes", "/api/notes/:id"
    pub params: Vec<String>,        // parameter names in declaration order
    pub ret_type: crate::ast::Type, // return type (for last_expr_type)
}

/// Type hint for f-string parts
enum FStrPartType {
    Int,     // i32 on stack (4 bytes)
    String,  // tagged string index (4 bytes)
    Float32, // f32 on stack (4 bytes)
    Float64, // f64 on stack (8 bytes)
    Uint64,  // u64 on stack (8 bytes, 2 slots)
}

/// Plan 194 Task 1: Map Auto types to native function name suffixes.
///
/// Used by monomorphic dispatch to resolve `m.insert("k", 42)` to
/// `HashMap.insert_int` based on the generic type parameter.
fn type_to_native_suffix(ty: &Type) -> &'static str {
    match ty {
        Type::Int | Type::I64 => "_int",
        Type::Uint | Type::U64 | Type::USize | Type::Byte => "_uint",
        Type::Float | Type::Double => "_float",
        Type::Bool => "_bool",
        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => "_str",
        _ => "",
    }
}

/// EDGE-04-B: 返回某 Type 的默认零值 Expr,用于 `Type{}` 空字面量补齐缺失字段。
/// 避免 CONSTRUCT_INSTANCE 从栈上 pop 垃圾值导致 "Invalid instance ID: 0"。
fn default_expr_for_type(ty: &Type) -> crate::ast::Expr {
    use crate::ast::Expr;
    match ty {
        Type::Int
        | Type::I64
        | Type::Uint
        | Type::U64
        | Type::USize
        | Type::Byte
        | Type::StrFixed(_)
        | Type::Char => Expr::Int(0),
        Type::Float | Type::Double => Expr::Float(0.0, "0.0".into()),
        Type::Bool => Expr::Bool(false),
        Type::StrSlice | Type::StrOwned | Type::CStrLit => Expr::Str("".into()),
        // List / 复合类型 / 用户类型:Nil 占位(handler 通常会随后赋真值)。
        _ => Expr::Nil,
    }
}

/// Codegen: Compiles AST directly to AutoVM Bytecode
pub struct Codegen {
    assembly_pending_native: bool,
    pub assembly_context: crate::stdlib_assembly::model::AssemblyContext,
    assembly_compile_depth: usize,
    assembly_native_ids: Vec<u16>,
    assembly_interface: Option<crate::vm::native::NativeInterface>,
    pub assembly_references: Vec<crate::stdlib_assembly::reference::ReferenceProof>,
    pub code: Vec<u8>,
    pub exports: HashMap<String, u32>,
    pub relocs: Vec<RelocEntry>,
    pub intrinsics: HashMap<String, u16>,
    /// String constant pool
    pub strings: Vec<Vec<u8>>,
    /// 池内容索引(add_string O(1) 去重;2026-08-22 起线性扫 O(n²) 退役)。
    /// 不变量:池只追加 —— 直推站点(绕过 add_string)只造成无害重复条目,
    /// 不会令本表失真(旧索引永有效)。
    string_dedup: HashMap<Vec<u8>, u32>,
    /// Plan 312: Collected #[api] routes (method, path, fn_name).
    /// Propagated through CompiledPackage → VirtualFlash → AutoVM for HTTP routing.
    pub api_routes: Vec<(String, String, String)>,
    /// Plan 321: Set of generator function names (return ~Iter<T> or ~Stream<T>).
    /// Call sites check this to emit CREATE_GENERATOR instead of CALL.
    pub generator_fns: std::collections::HashSet<String>,
    /// Object key pool (stores keys for object literals)
    /// Each entry is a Vec of keys for one object literal
    pub object_keys: Vec<Vec<auto_val::ValueKey>>,
    /// Plan 073: Object field types (stores type of each field value)
    pub object_types: Vec<Vec<ObjectType>>,

    /// Symbol table: Maps variable name -> local index (bp+0, bp+1, bp+2, ...)
    /// Used during compilation to emit LOAD_LOC_N and STORE_LOC_N
    pub locals: HashMap<String, usize>,

    /// Scope stack for nested scopes (functions, blocks)
    /// Each scope has its own variable namespace
    pub scope_stack: Vec<HashMap<String, usize>>,

    /// Plan 317: State fields of the task currently being compiled.
    /// Maps field name -> field_idx (for LOAD_STATE_FIELD/STORE_STATE_FIELD).
    /// Set while compiling a TaskDef's start hook + on handlers; cleared after.
    /// When set, bare identifier references to these names inside handlers
    /// resolve to actor state fields (persistent across handler invocations)
    /// instead of ordinary local variables.
    pub current_task_state_fields: HashMap<String, u8>,

    /// Plan 317: Module-level global variables (top-level `var x = ...`).
    /// Names accessible from ANY function via LOAD_GLOBAL/STORE_GLOBAL.
    /// global_inits stores (name, init_expr) so main() can run initializers
    /// at startup (the script wrapper's top-level code isn't executed because
    /// execute_autovm spawns main_entry directly).
    pub global_vars: std::collections::HashSet<String>,
    pub global_inits: Vec<(String, crate::ast::Expr)>,

    /// Plan 333: when true, `var name = expr` inside a fn body compiles to
    /// STORE_GLOBAL (instead of a local) for any name in `global_vars`. Used by
    /// the widget `__module_init` fn, which runs module-level initializers from
    /// inside a function scope (the codegen's `scope_stack.len() <= 1` guard
    /// would otherwise treat them as locals). Defaults false so normal fns
    /// still allow a `var x` shadow over a same-named global.
    pub force_global_store: bool,

    /// Plan 345: Current module name for global variable qualification.
    /// Empty = main module (no prefix). When set (e.g. "db"), global variable
    /// keys become "db.notes" instead of "notes", providing cross-module
    /// isolation in the flat vm.globals DashMap.
    pub current_module: String,

    /// Variable type tracking (Plan 080: support for instance methods on List, etc.)
    /// Maps variable name -> its type (e.g., "x" -> Type::List(Type::Int))
    /// Used to generate correct native method calls (e.g., x.push -> List.push)
    pub var_types: HashMap<String, Type>,

    /// Variable mutability tracking (Plan 080+: enforce immutability for let bindings)
    /// Maps variable name -> is_mutable (true for mut/var, false for let)
    /// Used to reject reassignments to immutable variables
    pub var_mutability: HashMap<String, bool>,

    /// Captured variables stack for nested closures (Plan 071 Phase 6.2)
    /// Each level has its own captured variable map (name -> capture index)
    /// Stack allows proper nesting: inner closures can capture from outer closures
    pub captured_vars_stack: Vec<HashMap<String, usize>>,

    /// Plan 419: 被 by-reference 捕获的局部槽位集合(CLOSURE capture_slots
    /// 机制跨作用域读写创建者帧槽)。块尾释放(pop_scope 发射 PUSH_NIL+
    /// STORE_LOCAL)跳过这些槽,避免闭包读槽被清。
    pub byref_captured_slots: std::collections::HashSet<usize>,

    /// Plan 073: Loop exit tracking for break/continue statements
    /// Each nested loop has a Vec of jump placeholders that need to be patched
    /// when the loop exits
    pub loop_exits: Vec<Vec<usize>>,

    /// Continue target tracking: stack of byte offsets to jump to on `continue`
    /// For range-based loops, this points to the increment step
    /// For iterator/collection loops, this points to the next-iteration check
    pub loop_continue_positions: Vec<usize>,

    /// Continue placeholder tracking (like loop_exits but for continue).
    /// Each nested loop has a Vec of JMP placeholders emitted by `continue`
    /// statements in the body. After the body is compiled and the continue
    /// target (increment/next-iteration) position is known, these are patched.
    /// This fixes the continue-infinite-loop bug where body-compiled continue
    /// statements jumped to offset 0 (the placeholder default) instead of the
    /// increment step.
    pub loop_continues: Vec<Vec<usize>>,

    /// Plan 073: Type registry for TypeDecl support
    /// Maps type name -> TypeInfo (member names, etc.)
    pub types: HashMap<String, TypeInfo>,

    /// Plan 076 Phase 1: Generic instantiation table
    /// Tracks all generic type instantiations (e.g., List<int>, List<string>)
    pub generics: GenericTable,

    /// Plan 087 Phase 1: Generic registry for user-defined generic types
    /// Stores generic class templates and their instantiations (e.g., Pair<int, string>)
    pub generic_registry: crate::vm::generic_registry::GenericRegistry,

    /// Plan 088 Phase 4: Function parameter information for smart parameter passing
    /// Maps function name -> Vec of parameter types and modes
    /// Used during function calls to determine whether to use value or reference passing
    pub fn_params: HashMap<String, Vec<ParamInfo>>,

    /// Plan 087 Phase 3: Function return types for .type property support
    /// Maps function name -> return type
    pub fn_return_types: HashMap<String, Type>,

    /// Plan 087 Phase 3: Current function parameter count (for correct local/param indexing)
    /// Used during compilation to distinguish parameters (before BP) from locals (after BP)
    pub current_fn_n_args: usize,

    /// Current function return type (for RET vs RET_D emission)
    pub current_fn_ret_type: Type,

    /// Plan 087 Phase 3: Starting index for function scope variables
    /// When outer scope has variables, function parameters don't start at index 0
    /// Used to correctly identify parameters: index >= fn_scope_start && index < fn_scope_start + n_args
    pub fn_scope_start: usize,

    /// Track current type's member names during method compilation
    /// Used to resolve implicit field access (bare field names → self.field)
    pub current_type_members: Option<Vec<String>>,

    /// Plan 417-E3: type-parameter names of the fn currently being compiled
    /// (`fn max_of<T has S>(...)` → ["T"]). A receiver whose static type is
    /// one of these names must dispatch methods dynamically (CALL_SPEC on the
    /// runtime heap tag) — the static `T.method` symbol doesn't exist.
    pub current_fn_type_params: Vec<String>,

    /// PLAN-019 T-06 (vm delegate): module prefix of the fn currently being
    /// compiled, when the name is module-qualified (`db.mux_resize_pane` →
    /// Some("db")). Flattened single-module synthesis (handler_codegen
    /// import_stmts) renames every imported fn to `mod.fn`, so an intra-module
    /// bare call inside such a fn body is ambiguous at module scope (api.X and
    /// db.X both define the bare name) and would otherwise stay unresolved —
    /// bare CALL reloc against dotted-only entry exports → "Undefined symbol".
    /// Call-site resolution binds bare names to the current fn's own module
    /// first (mirrors loader Plan 322/545 own-module semantics at codegen
    /// level). Unqualified fns compile with None; no behavior change elsewhere.
    pub current_fn_module: Option<String>,

    /// PLAN-733: store 域合成期（handler/computed 隐藏 fn）的 own-module
    /// 绑定域——store 源文件的模块名（collect_module_imports 登记）。合成
    /// 的 handler fn 名（`handler_<Store>_<Event>`）无点前缀，
    /// current_fn_module 按 name 推导得 None；本字段作为裸名 fn 的回退域，
    /// 使 store handler 体内**同文件**模块级 fn 的裸调用按 own-module 语义
    /// 绑定（resolve_call_symbol 步骤 2.5 同路径），不受跨模块同名裸名
    /// 歧义规则影响。仅 handler_codegen 的 per-widget 合成循环设置。
    pub store_scope_module: Option<String>,

    /// Plan 417-E3-P4: bounded type params per fn (callee name → params with
    /// their constraint lists), e.g. max_of → [(T, [Comparable])]. Populated
    /// at Stmt::Fn; consulted at call sites to reject arguments whose static
    /// type demonstrably does not implement the bound.
    pub fn_type_param_bounds: HashMap<String, Vec<(crate::ast::Name, Vec<Type>)>>,

    /// Plan 087 Phase 3: Type inference context for .type property support
    /// Uses the infer module's comprehensive type inference system
    pub infer_ctx: InferenceContext,

    /// Plan 084 Phase 3: Unified TypeStore for type declaration management
    /// Centralized storage for types, functions, specs, and generic templates
    /// Plan 123: Use RwLock for shared access with Parser
    pub type_store: Arc<RwLock<types::TypeStore>>,

    /// Plan 088 Phase 4: Jump placeholder tracking for multi-function compilation
    /// Tracks all jump_over placeholder indices to update them when FN_PROLOG is inserted
    /// When FN_PROLOG (3 bytes) is inserted, all subsequent code shifts
    /// and all jump_over placeholders after the insertion point need their indices updated
    pub jump_placeholders: Vec<usize>,

    /// Plan 118 Phase 5: Track jump targets for offset recalculation after FN_PROLOG insertion
    /// Each entry is (placeholder_idx, target_idx) where target_idx is the code position at patch time
    /// When FN_PROLOG is inserted, we need to recalculate offsets based on whether
    /// placeholder and target are before or after the insertion point
    pub jump_targets: Vec<(usize, usize)>,

    /// Plan 089: Maximum number of locals across all nested scopes
    /// Used to emit RESERVE_STACK with correct total size
    pub max_locals: usize,

    /// Plan 089: Whether to pop the result of an expression statement
    /// Used to ensure stack cleanliness for script evaluation
    pub should_pop_expr_result: bool,

    /// Plan 364 Step 5: Config-mode compilation flag.
    ///
    /// When true, `Codegen` compiles config (pac.at) code: every block-level
    /// Pair/Node/Object statement is accumulated into a container (via
    /// PUSH_ACCUM/ACCUM_PAIR/ACCUM_NODE/ACCUM_MERGE/POP_ACCUM) rather than
    /// only keeping the last value (Script semantics). This reuses Script's
    /// existing control-flow / local-var / f-string / Dot bytecode so the full
    /// Config eval power (runtime template, field access, for-over-array) is
    /// available. Defaults false — Script/TransC/TransRust paths are unchanged.
    pub config_mode: bool,

    /// Plan 375: keys that were injected as context args (dep override props)
    /// via `compile_config_program`. A top-level Pair with the same name acts as
    /// a *default* value, so it must NOT overwrite the injected local — only
    /// injected-but-absent keys fall back to the Pair's literal. Used by
    /// `compile_config_stmt_expr` to skip the `STORE_LOCAL` for these keys.
    pub config_injected_keys: std::collections::HashSet<String>,

    /// Plan 375: compile-time nesting depth of the config accumulator stack.
    /// Depth 0 = inside the root container (where top-level Pair statements
    /// like `kernel: {...}` live — these are *parameter defaults* and subject
    /// to injected-key override). Depth ≥1 = inside a nested node/dir body
    /// (where `at:` / `port:` are structural field defs, NOT parameter
    /// defaults, and must NOT be replaced by an injected value). Bumped by
    /// `compile_config_node` on entry / exit.
    pub config_accum_depth: u32,

    /// Plan 118: Track the type of the last compiled expression for result formatting
    /// Used to format output correctly (e.g., byte as hex, uint with suffix)
    pub last_expr_type: ObjectType,

    /// Plan 569 D2 (P539 根治): py-类型侧表——last_expr_type 的 py-可能伴随位。
    /// py-ffi 调用点置位；方法分派决策核优先消费（py 接收者的 `.len()` 等
    /// 改发 obj_len/obj_call 组合子，不再静态路由 str.* 把句柄 id 读成
    /// 字符串池索引）。重置纪律：compile_expr 顶 / 函数体首 / 原生调用
    /// 返回类型追踪链对照置 false（仅 py 臂置 true）。
    pub last_expr_may_py: bool,

    /// Plan 569 D2: py 可能变量名集（`var x = <py-ffi 调用>` 落盘传导）。
    /// 与 var_types 平行共存——StrFixed 谎言保留（顶层结果格式化依赖），
    /// 侧表只覆盖"路由到哪"，不影响"打印成什么"。
    pub py_typed_vars: HashSet<String>,
    /// PLAN-707 D-5: 持流句柄的变量集（`var s = Http.get_stream(..)` 登记；
    /// for-in 变量形态据此走迭代器通道）。重赋值移除（track/for 双向维护）。
    pub stream_vars: HashSet<String>,

    /// Plan 569 D2: fn 名 → 尾表达式 may_py（用户 fn 返回 py 桥值的跨函数
    /// 传导，单层流型保守正报——错报代价=obj_call 组合子 Auto 臂照常分派）。
    pub fn_may_py_returns: HashSet<String>,

    /// Track whether the last compiled expression was a native call (CALL_NAT).
    /// CALL_NAT void shims don't push return values, but CALL+RET always does.
    /// This lets Stmt::Expr emit POP only when needed.
    last_was_native_void: bool,

    /// Plan 127: Task handler registry for message routing
    /// Stores handler metadata for each task type
    pub task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry,

    /// Plan 390 §14 L3: task_name -> set of (variant_name, binding_names) for
    /// WithBindings patterns. Populated during TaskDef codegen so that a send
    /// call-site `h.send(Add(3, 5))` can recognize `Add(3,5)` as a variant
    /// constructor for the receiver's task and build a Value::Obj message.
    pub task_variants: HashMap<String, Vec<(String, Vec<String>)>>,

    /// Enum variant values: maps "EnumName.Variant" -> i32 value
    pub enum_values: HashMap<String, i32>,

    /// Plan 197 Task 13: Track the mono_name of the last constructed enum variant
    /// Set during enum variant construction codegen, consumed by the let-statement handler
    /// to record the variable's type in var_types for field access resolution.
    last_enum_variant_mono: Option<String>,

    /// Plan 203 Phase 2: Import scope for name resolution
    /// Maps local name → qualified name (e.g., "read_text" → "auto.fs.read_text")
    /// Populated from use statements with specific imports: `use auto.fs: read_text`
    pub import_scope: HashMap<String, String>,

    /// Plan 340: When true (VM+VM split mode), rewrite bare calls to `#[api]`
    /// functions into HTTP requests instead of in-process CALL relocs.
    pub api_over_http: bool,

    /// Plan 340: API function metadata for HTTP rewriting. Key = bare fn name
    /// (e.g. "list_notes"). Populated by synthesize_widget_module from the
    /// api_attrs of imported Fn declarations.
    pub api_funcs: HashMap<String, ApiCallInfo>,

    /// Known module prefixes from module-level imports (e.g., `use types`)
    /// Used to recognize qualified access patterns like `types.SectionType.Goals`
    known_module_prefixes: HashSet<String>,

    /// Plan 212b Task 3: Rust FFI function name mappings
    /// Maps local function name → (crate_name, full_path) for use.rust imports
    /// e.g., "from_str" → ("serde_json", "serde_json::from_str")
    /// Used to emit CALL_NAT for Rust FFI functions at codegen time
    rust_native_map: HashMap<String, (String, String)>,
    /// PLAN-596 T4:dep 声明 crate 名集合(Stmt::Dep 登记)——mono 实例
    /// 后缀的发放条件来源(仅 dep crate 侧的调用可加 "__i64"/"__String"
    /// 后缀;std/生成面误加会全 dispatch 链 miss)。
    declared_dep_crates: std::collections::HashSet<String>,

    /// Plan 216 Phase 2: C FFI function name → native_id mapping
    /// Populated when `use c <header.h>` is encountered, by loading the manifest
    /// and registering functions in the C-FFI runtime.
    c_ffi_functions: HashMap<String, u16>,

    /// Plan 214: Python FFI function name → (module, full_path) mapping
    /// Populated when `use.py module::{items}` is encountered.
    py_native_map: HashMap<String, (String, String)>,

    /// Plan 222: Python FFI function return types for type-aware dispatch
    /// Maps local function name → PyType (from py_ffi_types, no pyo3 dependency)
    py_return_types: HashMap<String, crate::py_ffi_types::PyType>,

    /// Plan 300: Bare Python module names (from `use.py math` without items).
    /// Used to resolve `module.method()` dot-calls at compile time.
    py_modules: std::collections::HashSet<String>,
    /// Plan 317: Auto modules imported via `use <module>` (e.g. db, api).
    /// When a method call's receiver matches one of these (e.g. db.func()),
    /// codegen generates a CALL with reloc symbol "<module>.<func>" which
    /// the linker resolves to the module's exported function (with prefix
    /// fallback).
    auto_modules: std::collections::HashSet<String>,

    /// PLAN-013 T1: names of declared `#[vm]` (body-less) functions.
    /// A `#[vm]` fn has no bytecode body — the declaration exists only as a
    /// native-shim signature, so a call to it must never shadow the native
    /// it declares (see the import_scope arm in the call resolution chain).
    /// `pub(crate)`: the compile session surfaces dep-module names to the
    /// root codegen (script path compiles modules separately).
    pub vm_fn_names: std::collections::HashSet<String>,

    /// PLAN-013 T1: qualifiers registered from FILE-module uses only
    /// (`use base64: …` → "base64"). Plan 347 shadow suppression keys on
    /// this set, not on `auto_modules` — a native-namespace use
    /// (`use auto.term: …`) must not suppress the very native it imports.
    file_modules: std::collections::HashSet<String>,

    /// Plan 212 Phase 2.2: Maps variable name → opaque crate name
    /// Tracks which variables hold opaque handles (e.g., "re" → "regex")
    /// Set when `let var = OpaqueType.new(...)` is compiled
    opaque_var_crates: HashMap<String, String>,

    /// Plan 199: Current source line for SOURCE_LINE opcode emission
    /// Avoids emitting redundant SOURCE_LINE for consecutive stmts on the same line
    current_source_line: u32,

    /// PLAN-057 T7：原始源文本（执行管线注入；None=纯 AST 编译场景，
    /// web 内建编译期门禁的 `// vm-safe-allow` 行级豁免退化为不可豁免）。
    pub source_text: Option<String>,
}

/// PLAN-671 (1): vm plain-name builtin registry (single source for the bare-call surface) — the intrinsics
/// constructor body of Codegen::new. The vue-track declaration layer (natives.d.ts) consumes the same
/// registry via [`bare_native_intrinsics`] (intersected with the actual bare-usage surface of generated files);
/// do not maintain a separate hand-copied name list.
pub fn bare_native_intrinsics() -> &'static HashMap<String, u16> {
    static MAP: std::sync::OnceLock<HashMap<String, u16>> = std::sync::OnceLock::new();
    MAP.get_or_init(build_bare_native_intrinsics)
}

fn build_bare_native_intrinsics() -> HashMap<String, u16> {
    let mut intrinsics = HashMap::new();
    // Register intrinsics - only built-in print functions
    // "print" defaults to print_str since most print calls are for strings
    intrinsics.insert("print".to_string(), NATIVE_PRINT_STR);
    intrinsics.insert("print_i32".to_string(), NATIVE_PRINT_I32);
    intrinsics.insert("print_f32".to_string(), NATIVE_PRINT_F32);
    intrinsics.insert("print_str".to_string(), NATIVE_PRINT_STR);
    intrinsics.insert("write".to_string(), NATIVE_WRITE_STR);
    intrinsics.insert("assert".to_string(), NATIVE_ASSERT);
    intrinsics.insert("assert_eq".to_string(), NATIVE_ASSERT_EQ);
    intrinsics.insert("assert_ne".to_string(), NATIVE_ASSERT_NE);
    intrinsics.insert("panic".to_string(), NATIVE_RUNTIME_PANIC);
    // Plan 442 B-support: JS web-compat alias — musk sources call the
    // browser global; route to the VM's percent-encoding native.
    intrinsics.insert(
        "encodeURIComponent".to_string(),
        crate::vm::native::NATIVE_URL_ENCODE,
    );
    // Plan 442 C2: bare `read_text(path)` — musk backend's comptime
    // `#{ read_text("../workflows/...") }` (workflow.at) compiles the
    // inner expr as a runtime call on the VM path; route to the fs
    // read native (a2r bakes the content at transpile time instead —
    // the VM-side value is cwd-relative and may differ, documented).
    if let Some(&id) = crate::vm::native_registry::NATIVE_ID_MAP.get("auto.fs.read_text") {
        intrinsics.insert("read_text".to_string(), id);
    }
    // Plan 442 C2: bare `format(fmt, args...)` — the Rust macro form
    // `format!(...)` (parser keeps the Ident, see the bang arm) routes to
    // the fmt.sprintf native ({} placeholder replacement).
    intrinsics.insert("format".to_string(), crate::vm::native::NATIVE_FMT_SPRINTF);
    // Plan 011 (MS3-B): shell-host bridge functions.
    intrinsics.insert("system".to_string(), NATIVE_SHELL_SYSTEM);
    intrinsics.insert("system_status".to_string(), NATIVE_SHELL_SYSTEM_STATUS);
    intrinsics.insert("export".to_string(), NATIVE_SHELL_EXPORT);
    intrinsics.insert("exit".to_string(), NATIVE_SHELL_EXIT);
    // PLAN-082: structured shell interop (records via shell_query,
    // print-executing commands via shell_run).
    intrinsics.insert(
        "shell_query".to_string(),
        crate::vm::native::NATIVE_SHELL_QUERY,
    );
    intrinsics.insert("shell_run".to_string(), crate::vm::native::NATIVE_SHELL_RUN);
    // Plan 413: code editor payload accessors (UI bridge).
    intrinsics.insert("code_editor_text".to_string(), NATIVE_CODE_EDITOR_TEXT);
    intrinsics.insert(
        "code_editor_cursor_line".to_string(),
        NATIVE_CODE_EDITOR_CURSOR_LINE,
    );
    intrinsics.insert(
        "code_editor_cursor_col".to_string(),
        NATIVE_CODE_EDITOR_CURSOR_COL,
    );
    intrinsics.insert(
        "code_editor_selection_len".to_string(),
        NATIVE_CODE_EDITOR_SELECTION_LEN,
    );
    intrinsics.insert("code_editor_find".to_string(), NATIVE_CODE_EDITOR_FIND);
    intrinsics.insert(
        "code_editor_set_text".to_string(),
        NATIVE_CODE_EDITOR_SET_TEXT,
    );
    // Plan 673 T-01: unified delta queue read side.
    intrinsics.insert("code_editor_delta".to_string(), NATIVE_CODE_EDITOR_DELTA);
    // Plan 673 T-02: structured write — write side.
    intrinsics.insert("code_editor_edit".to_string(), NATIVE_CODE_EDITOR_EDIT);
    intrinsics.insert(
        "code_editor_load_file".to_string(),
        NATIVE_CODE_EDITOR_LOAD_FILE,
    );
    // PLAN-701 供①/供②: save 直写 + 光标/滚动端点（9909-9913）。
    intrinsics.insert("code_editor_save".to_string(), NATIVE_CODE_EDITOR_SAVE);
    intrinsics.insert(
        "code_editor_set_cursor".to_string(),
        NATIVE_CODE_EDITOR_SET_CURSOR,
    );
    intrinsics.insert(
        "code_editor_scroll_offset_x".to_string(),
        NATIVE_CODE_EDITOR_SCROLL_OFFSET_X,
    );
    intrinsics.insert(
        "code_editor_scroll_offset_y".to_string(),
        NATIVE_CODE_EDITOR_SCROLL_OFFSET_Y,
    );
    intrinsics.insert(
        "code_editor_scroll_to".to_string(),
        NATIVE_CODE_EDITOR_SCROLL_TO,
    );
    // PLAN-701 供⑥: shell 跳转列表 Recent shim（9914）。
    intrinsics.insert("shell_add_recent".to_string(), NATIVE_SHELL_ADD_RECENT);
    // PLAN-703 供⑤: diff envelope endpoints.
    intrinsics.insert("diff_files".to_string(), NATIVE_DIFF_FILES);
    intrinsics.insert("diff_snapshots".to_string(), NATIVE_DIFF_SNAPSHOTS);
    intrinsics.insert("diff_dirs".to_string(), NATIVE_DIFF_DIRS);
    // PLAN-716 Phase 2 (供⑧): 9921 bare-name arm — Phase 1 left two gaps:
    // this intrinsics table missed the entry, AND the endpoint was booked at
    // 9920 which collides with PLAN-095 ui.focus (later catalog entry wins
    // the shim slot — VM-track calls silently dispatched to ui_focus and
    // returned Int(0)). Re-booked to 9921 in native_catalog (see 组C note);
    // ui.focus keeps 9920.
    intrinsics.insert("diff_files_window".to_string(), NATIVE_DIFF_FILES_WINDOW);
    // Plan 019 批次八: autodown 文档 natives（.at handler 可编程操作文档）。
    intrinsics.insert("autodown_parse".to_string(), NATIVE_AUTODOWN_PARSE);
    intrinsics.insert("autodown_serialize".to_string(), NATIVE_AUTODOWN_SERIALIZE);
    intrinsics.insert("autodown_text".to_string(), NATIVE_AUTODOWN_TEXT);
    intrinsics.insert(
        "autodown_find_block".to_string(),
        NATIVE_AUTODOWN_FIND_BLOCK,
    );
    intrinsics.insert(
        "autodown_insert_text".to_string(),
        NATIVE_AUTODOWN_INSERT_TEXT,
    );
    intrinsics.insert(
        "autodown_insert_template".to_string(),
        NATIVE_AUTODOWN_INSERT_TEMPLATE,
    );
    // Plan 019 批次九: 编辑壳全文回读。
    intrinsics.insert(
        "autodown_editor_text".to_string(),
        NATIVE_AUTODOWN_EDITOR_TEXT,
    );
    // Plan 413 follow-up: console natives (in-app Console panel).
    intrinsics.insert("console_log".to_string(), NATIVE_CONSOLE_LOG);
    intrinsics.insert("console_lines".to_string(), NATIVE_CONSOLE_LINES);
    intrinsics.insert("console_clear".to_string(), NATIVE_CONSOLE_CLEAR);
    // Plan 418: editor actions + clipboard + dialogs (menu/toolbar handlers).
    intrinsics.insert("code_editor_undo".to_string(), NATIVE_CODE_EDITOR_UNDO);
    intrinsics.insert("code_editor_redo".to_string(), NATIVE_CODE_EDITOR_REDO);
    intrinsics.insert(
        "code_editor_select_all".to_string(),
        NATIVE_CODE_EDITOR_SELECT_ALL,
    );
    intrinsics.insert("code_editor_cut".to_string(), NATIVE_CODE_EDITOR_CUT);
    intrinsics.insert("code_editor_copy".to_string(), NATIVE_CODE_EDITOR_COPY);
    intrinsics.insert("code_editor_paste".to_string(), NATIVE_CODE_EDITOR_PASTE);
    // Plan 428 P1: code folding natives (view state).
    intrinsics.insert(
        "code_editor_fold_toggle".to_string(),
        NATIVE_CODE_EDITOR_FOLD_TOGGLE,
    );
    intrinsics.insert(
        "code_editor_fold_hidden_count".to_string(),
        NATIVE_CODE_EDITOR_FOLD_HIDDEN_COUNT,
    );
    intrinsics.insert("clipboard_text".to_string(), NATIVE_CLIPBOARD_TEXT);
    intrinsics.insert("clipboard_set_text".to_string(), NATIVE_CLIPBOARD_SET_TEXT);
    // Plan 485: native clipboard files/images bare-name intrinsics.
    intrinsics.insert(
        "clipboard_files_get".to_string(),
        NATIVE_CLIPBOARD_FILES_GET,
    );
    intrinsics.insert(
        "clipboard_files_set".to_string(),
        NATIVE_CLIPBOARD_FILES_SET,
    );
    intrinsics.insert(
        "clipboard_image_get".to_string(),
        NATIVE_CLIPBOARD_IMAGE_GET,
    );
    intrinsics.insert(
        "clipboard_image_set".to_string(),
        NATIVE_CLIPBOARD_IMAGE_SET,
    );
    // Plan 488: OLE 拖出（DoDragDrop 调用线程内联阻塞；效果经
    // on_dnd_finished 事件回注）。
    intrinsics.insert("dnd_start".to_string(), NATIVE_DND_START);
    // PLAN-656: scroll-pane controller 原生族（bare-name intrinsics）。
    intrinsics.insert("scroll_controller".to_string(), NATIVE_SCROLL_CONTROLLER);
    intrinsics.insert("scroll_to_start".to_string(), NATIVE_SCROLL_TO_START);
    intrinsics.insert("scroll_to_end".to_string(), NATIVE_SCROLL_TO_END);
    intrinsics.insert("scroll_by".to_string(), NATIVE_SCROLL_BY);
    intrinsics.insert("scroll_to".to_string(), NATIVE_SCROLL_TO);
    intrinsics.insert("scroll_state".to_string(), NATIVE_SCROLL_STATE);
    // PLAN-781 T-01: 子内容锚点族（9922/9923，接口决策冻结）。
    intrinsics.insert("scroll_to_child".to_string(), NATIVE_SCROLL_TO_CHILD);
    intrinsics.insert(
        "scroll_anchor_state".to_string(),
        NATIVE_SCROLL_ANCHOR_STATE,
    );
    intrinsics.insert("dialog_open".to_string(), NATIVE_DIALOG_OPEN);
    intrinsics.insert("dialog_save".to_string(), NATIVE_DIALOG_SAVE);
    intrinsics.insert("file_basename".to_string(), NATIVE_FILE_BASENAME);
    intrinsics
}

impl Codegen {
    pub fn new() -> Self {
        // Initialize the global native registry
        crate::vm::native_registry::register_builtin_natives();

        // PLAN-671 (1): plain-name builtin registry moved to build_bare_native_intrinsics
        // (single source; the vue-track natives.d.ts declaration layer consumes the same registry
        // via bare_native_intrinsics(), so new plain-name builtins automatically enter the declaration candidate set).
        let intrinsics = bare_native_intrinsics().clone();

        // Register return types for native functions (used for type inference in let bindings)
        // Plan 378: actually use the built map (previously discarded as `_fn_return_types`,
        // which left fn_return_types empty for the script-run path via Codegen::new()).
        let fn_return_types = Self::build_fn_return_types();

        // Create global scope
        let locals = HashMap::new();
        let mut scope_stack = Vec::new();
        scope_stack.push(locals);

        let mut codegen = Self {
            assembly_pending_native: false,
            assembly_context: Default::default(),
            assembly_compile_depth: 0,
            assembly_native_ids: Vec::new(),
            assembly_interface: None,
            assembly_references: Vec::new(),
            code: Vec::new(),
            exports: HashMap::new(),
            api_routes: Vec::new(),
            generator_fns: std::collections::HashSet::new(),
            relocs: Vec::new(),
            intrinsics,
            strings: Vec::new(),
            string_dedup: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            locals: HashMap::new(),
            scope_stack,
            current_task_state_fields: HashMap::new(), // Plan 317: actor state fields
            global_vars: std::collections::HashSet::new(), // Plan 317: module-level vars
            global_inits: Vec::new(),                  // Plan 317: global var initializers
            force_global_store: false,                 // Plan 333: set true inside __module_init
            current_module: String::new(), // Plan 345: module name for global qualification
            var_types: HashMap::new(),     // Plan 080: variable type tracking
            var_mutability: HashMap::new(), // Plan 080+: variable mutability tracking
            captured_vars_stack: Vec::new(),
            byref_captured_slots: std::collections::HashSet::new(),
            loop_exits: Vec::new(),
            loop_continue_positions: Vec::new(),
            loop_continues: Vec::new(),
            types: HashMap::new(),
            generics: GenericTable::new(), // Plan 076 Phase 1
            generic_registry: crate::vm::generic_registry::GenericRegistry::new(), // Plan 087 Phase 1
            fn_params: HashMap::new(), // Plan 088 Phase 4: function parameter information
            fn_return_types, // Plan 087 Phase 3: function return types for .type (Plan 378: populated)
            current_fn_n_args: 0, // Plan 087 Phase 3: Initialize to 0
            current_fn_ret_type: Type::Void,
            current_fn_type_params: Vec::new(),   // Plan 417-E3
            current_fn_module: None,              // PLAN-019 T-06: own-module bare-call binding
            store_scope_module: None,             // PLAN-733: store 域合成 own-module 绑定
            fn_type_param_bounds: HashMap::new(), // Plan 417-E3-P4
            fn_scope_start: 0,                    // Plan 087 Phase 3: Initialize to 0
            infer_ctx: InferenceContext::new(),   // Plan 087 Phase 3: Type inference context
            type_store: Arc::new(RwLock::new(types::TypeStore::new())), // Plan 084 Phase 3: Unified TypeStore
            jump_placeholders: Vec::new(), // Plan 088 Phase 4: Initialize empty jump placeholder tracking
            jump_targets: Vec::new(),      // Plan 118 Phase 5: Initialize jump target tracking
            max_locals: 0,
            should_pop_expr_result: false,
            config_mode: false, // Plan 364 Step 5: Script mode by default
            config_injected_keys: std::collections::HashSet::new(), // Plan 375
            config_accum_depth: 0, // Plan 375
            last_expr_type: ObjectType::Int, // Plan 118: Default to Int
            last_was_native_void: false,
            last_expr_may_py: false,       // Plan 569 D2: py-类型侧表粘性位
            py_typed_vars: HashSet::new(), // Plan 569 D2
            stream_vars: HashSet::new(),   // PLAN-707 D-5
            fn_may_py_returns: HashSet::new(), // Plan 569 D2
            task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry::new(), // Plan 127
            task_variants: HashMap::new(), // Plan 390 §14 L3
            current_type_members: None,    // Plan 087 Phase 3: No type context initially
            enum_values: HashMap::new(),
            last_enum_variant_mono: None, // Plan 197 Task 13
            import_scope: HashMap::new(), // Plan 203 Phase 2: Import scope for name resolution
            api_over_http: false,         // Plan 340: VM+VM split mode HTTP rewriting
            api_funcs: HashMap::new(),    // Plan 340: API fn metadata
            known_module_prefixes: HashSet::new(),
            rust_native_map: HashMap::new(),
            declared_dep_crates: Default::default(), // Plan 212b Task 3: Rust FFI function mappings
            c_ffi_functions: HashMap::new(),         // Plan 216 Phase 2: C FFI function mappings
            py_native_map: HashMap::new(),           // Plan 214: Python FFI function mappings
            py_return_types: HashMap::new(),         // Plan 222: Python FFI return types
            py_modules: std::collections::HashSet::new(), // Plan 300: bare py modules
            auto_modules: std::collections::HashSet::new(), // Plan 317: Auto modules
            vm_fn_names: std::collections::HashSet::new(), // PLAN-013 T1: #[vm] decl names
            file_modules: std::collections::HashSet::new(), // PLAN-013 T1: file-module qualifiers
            opaque_var_crates: HashMap::new(),       // Plan 212 Phase 2.2: opaque var tracking
            current_source_line: 0,                  // Plan 199: Source line tracking
            source_text: None,                       // PLAN-057 T7
        };

        // Plan 197 Task 16: Register built-in Option.Some and Option.None enum variants
        codegen.register_builtin_option_variants();
        codegen
    }

    /// Plan 364 Step 5: Construct a Codegen configured for Config (pac.at) mode.
    ///
    /// Equivalent to `Codegen::new()` but with `config_mode = true`. Used by
    /// `eval_config_with_vm` / `run_with_mode(CompileMode::Config)` so the full
    /// Config eval power (runtime f-string templates, object field access,
    /// for-over-array, if/for control flow) is available — replacing the old
    /// compile-time-only `ConfigCodegen`.
    pub fn new_for_config() -> Self {
        let mut cg = Self::new();
        cg.config_mode = true;
        cg
    }

    /// Plan 364 Step 5: Compile a parsed config (pac.at) program to bytecode.
    ///
    /// Layout:
    /// ```text
    ///   FN_PROLOG(0, n_locals)
    ///   RESERVE_STACK(n_locals)
    ///   <for each (key, val) in args: LOAD_STR(val) / literal; STORE_LOCAL(slot)>
    ///   PUSH_ACCUM("root", none)        // root accumulation container
    ///   <compile each top-level stmt>
    ///   POP_ACCUM                        // leaves root Node id on stack
    ///   RET(0)
    /// ```
    /// Context variables (e.g. `port`) are injected as locals so that
    /// `if port == "win32"` resolves at runtime via LOAD_LOCAL (replacing the
    /// old compile-time `substitute_vars`). Top-level `var` declarations stay
    /// locals too and are visible to later statements / f-strings / Dot access.
    pub fn compile_config_program(
        &mut self,
        code: &crate::ast::Code,
        args: &auto_val::Obj,
    ) -> AutoResult<()> {
        // Set up a function frame so BP/locals work like a script wrapper.
        let n_locals: u8 = 32; // generous local budget for manifest vars
        self.emit(OpCode::FN_PROLOG);
        self.code.push(0); // n_args
        self.code.push(n_locals);

        self.emit(OpCode::RESERVE_STACK);
        self.code.push(n_locals);

        // Inject context args (e.g. `port`, `kernel`) as locals. Plan 375:
        // previously only `Value::Str` args were injected, so object/array/
        // numeric dep overrides (e.g. SCU001's `dep osal { kernel: kernel_config }`
        // where `kernel_config` is an Object) were silently dropped — leaving the
        // child package's defaults in effect and producing wrong include paths.
        // We now materialize every literal arg via `compile_expr` so all value
        // types reach the child pac.at's scope.
        for (key, val) in args.iter() {
            if let auto_val::ValueKey::Str(name) = key {
                if let Some(expr) = Self::value_to_expr(val) {
                    let slot = self.add_var(name.as_str());
                    self.compile_expr(&expr)?;
                    self.emit_store_loc(slot);
                    // Record this key so a later top-level Pair of the same
                    // name (the package's default) does NOT overwrite the
                    // injected override value.
                    self.config_injected_keys.insert(name.to_string());
                }
            }
        }

        // Root accumulation container.
        let root_name_idx = self.add_string("root");
        self.emit(OpCode::PUSH_ACCUM);
        self.code.extend_from_slice(&root_name_idx.to_le_bytes());
        self.code.extend_from_slice(&0xFFFFu32.to_le_bytes()); // no id(u32 化后哨兵仍是 0xFFFF 数值)

        // Compile top-level statements. Each Pair/Node/Object accumulates into
        // the root container; if/for/var use the regular Script codegen (their
        // bodies dispatch back through config_mode accumulation).
        for stmt in &code.stmts {
            self.compile_stmt(stmt)?;
        }

        // Pop the root container -> leaves root Node id on the stack.
        self.emit(OpCode::POP_ACCUM);

        // Return: RET pops the frame (the root Node id stays as the result).
        self.emit(OpCode::RET);
        self.code.push(0); // n_args
        Ok(())
    }

    /// Plan 375: Convert a runtime `auto_val::Value` literal into an `Expr`
    /// so it can be compiled via `compile_expr` and injected as a config-mode
    /// local (dep override props). Returns `None` for non-literal value kinds
    /// (functions, views, vm refs, etc.) that have no literal representation.
    fn value_to_expr(val: &auto_val::Value) -> Option<crate::ast::Expr> {
        use crate::ast::{Expr, Key, Pair};
        let expr = match val {
            auto_val::Value::Int(i) => Expr::Int(*i),
            auto_val::Value::Uint(u) => Expr::Uint(*u),
            auto_val::Value::I64(i) => Expr::I64(*i),
            auto_val::Value::U8(b) => Expr::U8(*b),
            auto_val::Value::I8(i) => Expr::I8(*i),
            auto_val::Value::Bool(b) => Expr::Bool(*b),
            auto_val::Value::Byte(b) => Expr::Byte(*b),
            auto_val::Value::Char(c) => Expr::Char(*c),
            auto_val::Value::Str(s) => Expr::Str(s.clone()),
            auto_val::Value::Float(f) => Expr::Float(*f, auto_val::AutoStr::new()),
            auto_val::Value::Double(f) => Expr::Double(*f, auto_val::AutoStr::new()),
            auto_val::Value::Nil | auto_val::Value::Null => Expr::Nil,
            auto_val::Value::Array(arr) => {
                let elems: Vec<Expr> = arr.iter().filter_map(Self::value_to_expr).collect();
                Expr::Array(elems)
            }
            auto_val::Value::Obj(obj) => {
                let pairs: Vec<Pair> = obj
                    .iter()
                    .filter_map(|(k, v)| {
                        let key = match k {
                            auto_val::ValueKey::Str(s) => Key::NamedKey(s.clone()),
                            auto_val::ValueKey::Int(i) => Key::IntKey(*i),
                            auto_val::ValueKey::Bool(b) => Key::BoolKey(*b),
                        };
                        Self::value_to_expr(v).map(|e| Pair {
                            key,
                            value: Box::new(e),
                        })
                    })
                    .collect();
                Expr::Object(pairs)
            }
            // Non-literal kinds (Node, Fn, VmRef, Instance, ...) have no literal
            // form; skip them rather than risk an incorrect injection.
            _ => return None,
        };
        Some(expr)
    }

    /// Plan 197 Task 16: Register built-in Option.Some and Option.None as enum variant templates
    fn register_builtin_option_variants(&mut self) {
        use crate::vm::generic_registry::{ClassTemplate, FieldDef};

        // Option.Some has 1 field: _0 (the wrapped value)
        let some_template = ClassTemplate::new(
            "Option.Some",
            vec![],
            vec![FieldDef::new("_0", Type::Unknown)],
            vec![],
        );
        let _ = self.generic_registry.register_template(some_template);

        // Option.None has 0 fields
        let none_template = ClassTemplate::new("Option.None", vec![], vec![], vec![]);
        let _ = self.generic_registry.register_template(none_template);
    }

    /// Register enum variants from pre-collected TypeStore data into codegen's
    /// enum_values and generic_registry. This allows cross-module enum construction
    /// (e.g. `ApiError.Http("msg")` when `ApiError` is defined in a `use errors` module).
    fn register_enums_from_data(
        &mut self,
        enums: &[(
            String,
            Vec<(
                String,
                i32,
                Option<Type>,
                Vec<Type>,
                Vec<crate::ast::EnumField>,
            )>,
        )],
    ) {
        use crate::vm::generic_registry::{ClassTemplate, FieldDef};

        for (enum_name, variants) in enums {
            for (variant_name, value, payload_type, payload_types, fields) in variants {
                let key = format!("{}.{}", enum_name, variant_name);
                self.enum_values.entry(key.clone()).or_insert(*value);

                let variant_mono = key.clone();
                if !fields.is_empty() {
                    let fdefs: Vec<FieldDef> = fields
                        .iter()
                        .map(|f| FieldDef::new(f.name.as_str(), f.field_type.clone()))
                        .collect();
                    let template = ClassTemplate::new(&variant_mono, vec![], fdefs, vec![]);
                    let _ = self.generic_registry.register_template(template);
                } else if !payload_types.is_empty() {
                    // Multi-field anonymous variant: ToolUse str str str -> fields _0, _1, _2
                    let fdefs: Vec<FieldDef> = payload_types
                        .iter()
                        .enumerate()
                        .map(|(i, t)| FieldDef::new(&format!("_{}", i), t.clone()))
                        .collect();
                    let template = ClassTemplate::new(&variant_mono, vec![], fdefs, vec![]);
                    let _ = self.generic_registry.register_template(template);
                } else if let Some(ref payload) = payload_type {
                    let fdefs = vec![FieldDef::new("_0", payload.clone())];
                    let template = ClassTemplate::new(&variant_mono, vec![], fdefs, vec![]);
                    let _ = self.generic_registry.register_template(template);
                }
                // Scalar enum variants (no fields, no payload) are NOT registered as templates.
                // They compile to plain integer constants, so has_template() returning false
                // causes the is-pattern codegen to use EQ comparison (correct for scalar enums).
            }
        }
    }

    /// Plan 084 Phase 3: Create Codegen with custom TypeStore
    /// Allows Parser and Codegen to share the same TypeStore instance
    /// Plan 123: Accept Arc<RwLock<TypeStore>> for shared access with Parser
    pub fn new_with_type_store(type_store: Arc<RwLock<types::TypeStore>>) -> Self {
        // Initialize the global native registry
        crate::vm::native_registry::register_builtin_natives();

        // Plan 198 Phase 2: Enrich native registry with return types from TypeStore
        // Only enrich once per process to avoid mutating global state across test runs
        if !crate::vm::native_registry::NATIVE_REGISTRY_ENRICHED
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            let ts = type_store.read().unwrap();
            crate::vm::native_registry::BIGVM_NATIVES
                .lock()
                .unwrap()
                .enrich_from_type_store(&ts);
            crate::vm::native_registry::NATIVE_REGISTRY_ENRICHED
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }

        // Plan 673 T-01 single-source fix: this table was a hand-synced copy
        // of build_bare_native_intrinsics (verified key+value identical,
        // 56 entries, only insertion order differed). Clone the bare table
        // instead — Codegen::new() has done the same since PLAN-671.
        let intrinsics = bare_native_intrinsics().clone();

        // Register return types for native functions (used for type inference in let bindings)
        let mut fn_return_types = Self::build_fn_return_types();
        // Plan 198 Phase 1: Enrich with TypeStore-derived return types (authoritative)
        {
            let ts = type_store.read().unwrap();
            Self::enrich_fn_return_types_from_type_store(&ts, &mut fn_return_types);
        }

        // Create global scope
        let locals = HashMap::new();
        let mut scope_stack = Vec::new();
        scope_stack.push(locals);

        let mut codegen = Self {
            assembly_pending_native: false,
            assembly_context: Default::default(),
            assembly_compile_depth: 0,
            assembly_native_ids: Vec::new(),
            assembly_interface: None,
            assembly_references: Vec::new(),
            code: Vec::new(),
            exports: HashMap::new(),
            api_routes: Vec::new(),
            generator_fns: std::collections::HashSet::new(),
            relocs: Vec::new(),
            intrinsics,
            strings: Vec::new(),
            string_dedup: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            locals: HashMap::new(),
            scope_stack,
            current_task_state_fields: HashMap::new(), // Plan 317
            global_vars: std::collections::HashSet::new(), // Plan 317
            global_inits: Vec::new(),                  // Plan 317
            force_global_store: false,                 // Plan 333
            current_module: String::new(),             // Plan 345
            var_types: HashMap::new(),
            var_mutability: HashMap::new(),
            captured_vars_stack: Vec::new(),
            byref_captured_slots: std::collections::HashSet::new(),
            loop_exits: Vec::new(),
            loop_continue_positions: Vec::new(),
            loop_continues: Vec::new(),
            types: HashMap::new(),
            generics: GenericTable::new(),
            generic_registry: crate::vm::generic_registry::GenericRegistry::new(),
            fn_params: HashMap::new(),
            fn_return_types,
            current_fn_n_args: 0,
            current_fn_ret_type: Type::Void,
            current_fn_type_params: Vec::new(),   // Plan 417-E3
            current_fn_module: None,              // PLAN-019 T-06: own-module bare-call binding
            store_scope_module: None,             // PLAN-733: store 域合成 own-module 绑定
            fn_type_param_bounds: HashMap::new(), // Plan 417-E3-P4
            fn_scope_start: 0,
            infer_ctx: InferenceContext::new(),
            type_store, // Plan 084 Phase 3: Use provided TypeStore
            jump_placeholders: Vec::new(),
            jump_targets: Vec::new(), // Plan 118 Phase 5: Initialize jump target tracking
            max_locals: 0,
            should_pop_expr_result: false,
            config_mode: false, // Plan 364 Step 5: Script mode by default
            config_injected_keys: std::collections::HashSet::new(), // Plan 375
            config_accum_depth: 0, // Plan 375
            last_expr_type: ObjectType::Int, // Plan 118: Default to Int
            last_was_native_void: false,
            last_expr_may_py: false,       // Plan 569 D2: py-类型侧表粘性位
            py_typed_vars: HashSet::new(), // Plan 569 D2
            stream_vars: HashSet::new(),   // PLAN-707 D-5
            fn_may_py_returns: HashSet::new(), // Plan 569 D2
            task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry::new(), // Plan 127
            task_variants: HashMap::new(), // Plan 390 §14 L3
            current_type_members: None,    // Plan 087 Phase 3: No type context initially
            enum_values: HashMap::new(),
            last_enum_variant_mono: None, // Plan 197 Task 13
            import_scope: HashMap::new(), // Plan 203 Phase 2: Import scope for name resolution
            api_over_http: false,         // Plan 340: VM+VM split mode HTTP rewriting
            api_funcs: HashMap::new(),    // Plan 340: API fn metadata
            known_module_prefixes: HashSet::new(),
            rust_native_map: HashMap::new(),
            declared_dep_crates: Default::default(), // Plan 212b Task 3: Rust FFI function mappings
            c_ffi_functions: HashMap::new(),         // Plan 216 Phase 2: C FFI function mappings
            py_native_map: HashMap::new(),           // Plan 214: Python FFI function mappings
            py_return_types: HashMap::new(),         // Plan 222: Python FFI return types
            py_modules: std::collections::HashSet::new(), // Plan 300: bare py modules
            auto_modules: std::collections::HashSet::new(), // Plan 317: Auto modules
            vm_fn_names: std::collections::HashSet::new(), // PLAN-013 T1: #[vm] decl names
            file_modules: std::collections::HashSet::new(), // PLAN-013 T1: file-module qualifiers
            opaque_var_crates: HashMap::new(),       // Plan 212 Phase 2.2: opaque var tracking
            current_source_line: 0,                  // Plan 199: Source line tracking
            source_text: None,                       // PLAN-057 T7
        };
        // Plan 197 Task 16: Register built-in Option.Some and Option.None enum variants
        codegen.register_builtin_option_variants();
        // Register enum variants from TypeStore (populated by use/module resolution)
        {
            let enum_data: Vec<(
                String,
                Vec<(
                    String,
                    i32,
                    Option<Type>,
                    Vec<Type>,
                    Vec<crate::ast::EnumField>,
                )>,
            )> = {
                let ts = codegen.type_store.read().unwrap();
                ts.all_enum_decls()
                    .map(|(name, decl)| {
                        let items = decl
                            .items
                            .iter()
                            .enumerate()
                            .map(|(i, item)| {
                                let value = item.scalar_value.unwrap_or(i as i32);
                                let variant_name = item.name.to_string();
                                (
                                    variant_name,
                                    value,
                                    item.payload_type.clone(),
                                    item.payload_types.clone(),
                                    item.fields.clone(),
                                )
                            })
                            .collect();
                        (name.to_string(), items)
                    })
                    .collect()
            };
            codegen.register_enums_from_data(&enum_data);
        }
        // Register user-defined types (TypeDecl) from TypeStore into generic_registry
        // so that cross-module type constructors and field access work correctly
        {
            let type_decls: Vec<(String, Vec<String>, Vec<crate::ast::Type>)> = {
                let ts = codegen.type_store.read().unwrap();
                let decls: Vec<_> = ts.all_type_decls().collect();
                decls
                    .into_iter()
                    .map(|(name, decl)| {
                        let field_names: Vec<String> =
                            decl.members.iter().map(|m| m.name.to_string()).collect();
                        let field_types: Vec<crate::ast::Type> =
                            decl.members.iter().map(|m| m.ty.clone()).collect();
                        (name.to_string(), field_names, field_types)
                    })
                    .collect()
            };
            for (type_name, field_names, field_types) in type_decls {
                if !codegen.generic_registry.has_template(&type_name) {
                    let fdefs: Vec<crate::vm::generic_registry::FieldDef> = field_names
                        .iter()
                        .zip(field_types.iter())
                        .map(|(n, t): (&String, &crate::ast::Type)| {
                            crate::vm::generic_registry::FieldDef::new(n.clone(), t.clone())
                        })
                        .collect();
                    let template = crate::vm::generic_registry::ClassTemplate::new(
                        type_name.clone(),
                        vec![],
                        fdefs,
                        vec![],
                    );
                    let _ = codegen.generic_registry.register_template(template);
                }
                // Ensure monomorphic instance exists in types map (needed for CONSTRUCT_INSTANCE)
                let _ = codegen
                    .generic_registry
                    .get_or_create_type(&type_name, vec![]);
                codegen.types.insert(
                    type_name.clone(),
                    crate::vm::codegen::TypeInfo {
                        _name: type_name.clone(),
                        member_names: field_names,
                    },
                );
            }
        }
        codegen
    }

    // Plan 076 Phase 1: Generic type tracking methods

    /// PLAN-596 T4:调用点实参齐整字面量形态 → mono 实例标签
    /// (全整型字面量 → "i64";全字符串字面量 → "String";空/混合/含非字面量 → None)。
    fn mono_literal_label(args: &[crate::ast::Arg]) -> Option<&'static str> {
        let mut all_int = true;
        let mut all_str = true;
        let mut any = false;
        for a in args {
            match a {
                crate::ast::Arg::Pos(e) => {
                    any = true;
                    match e {
                        Expr::Int(_)
                        | Expr::Uint(_)
                        | Expr::I8(_)
                        | Expr::U8(_)
                        | Expr::I64(_)
                        | Expr::U64(_)
                        | Expr::Byte(_) => all_str = false,
                        Expr::Str(_) | Expr::CStr(_) => all_int = false,
                        _ => return None,
                    }
                }
                _ => return None,
            }
        }
        if !any {
            return None;
        }
        if all_int {
            Some("i64")
        } else if all_str {
            Some("String")
        } else {
            None
        }
    }

    pub fn finish(self, name: String) -> Module {
        // Plan 348 E1: record whether this module declared any top-level
        // globals (var/const). The linker retains such modules even without
        // function exports so their STORE_GLOBAL init code runs.
        let has_globals = !self.global_vars.is_empty();
        // Plan 423 P5 加固:导出对齐闸门 —— Stmt::Fn 编译中途失败会留下
        // "导出已记录、prolog 未插"的半成品(毒化字节码),运行时从指令
        // 中间进入 → 栈失衡 → 跳出代码(InvalidOpCode 255)或在垃圾指令
        // 里跑满步数预算无界吃内存(实机 20G 内存事故)。在模块出口丢弃
        // 毒化导出:调用方得到清晰的 HandlerNotFound,而非运行时内存炸弹。
        let mut exports = self.exports;
        for n in poisoned_fn_exports(&self.code, &exports) {
            exports.remove(&n);
            eprintln!(
                "[CODEGEN] dropping poisoned export '{}' (entry not at FN_PROLOG) — its fn failed to compile; see earlier handler errors",
                n
            );
        }
        Module {
            name,
            code: self.code,
            exports,
            relocs: self.relocs,
            strings: self.strings,
            // Plan 073: Include object_keys and object_types in module
            object_keys: self.object_keys,
            object_types: self.object_types,
            has_globals,
        }
    }

    // === Helpers ===

    fn emit(&mut self, op: OpCode) {
        self.assembly_pending_native = matches!(op, OpCode::CALL_NAT | OpCode::CALL_NAT_COUNTED);
        let opcode = op as u8;
        self.code.push(opcode);
    }

    /// Public method to emit an opcode (for script setup)
    pub fn emit_op(&mut self, op: OpCode) {
        self.emit(op);
    }

    /// Public method to emit a byte (for script setup)
    pub fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    fn emit_i32(&mut self, val: i32) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    // Plan 087 Phase 2: Emit u32 value (4 bytes, little-endian)
    fn emit_u32(&mut self, val: u32) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    // Plan 087 Phase 2: Emit u16 value (2 bytes, little-endian)
    fn emit_u16(&mut self, val: u16) {
        if self.assembly_pending_native {
            self.assembly_pending_native = false;
            if !self.assembly_native_ids.contains(&val) {
                self.assembly_native_ids.push(val);
            }
        }
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    fn verify_assembly_references(&mut self) -> AutoResult<()> {
        // Verify after the outer compilation operation has released all
        // native-registry guards. Emission itself must never acquire that lock.
        let ids = std::mem::take(&mut self.assembly_native_ids);
        for id in ids {
            let is_core = {
                let registry = BIGVM_NATIVES.lock().unwrap();
                crate::vm::native_catalog::NATIVE_ID_ENTRIES
                    .iter()
                    .filter(|(_, native)| *native == id)
                    .map(|(name, _)| name.to_string())
                    .chain(
                        registry
                            .get_function_names()
                            .into_iter()
                            .filter(|name| registry.get_id(name) == Some(id)),
                    )
                    .any(|name| {
                        name.strip_prefix("auto.").is_some_and(|rest| {
                            crate::stdlib_assembly::validate::CORE_MODULES
                                .contains(&rest.split('.').next().unwrap_or(""))
                                || rest.starts_with("http_stream.")
                        })
                    })
            };
            if !is_core {
                continue;
            }
            let interface = self
                .assembly_interface
                .get_or_insert_with(crate::vm::native::NativeInterface::production);
            if let Some(proof) = interface
                .verify_core_reference(id)
                .map_err(crate::error::AutoError::Msg)?
            {
                if self.assembly_context.environment
                    == crate::stdlib_assembly::model::Environment::Browser
                {
                    return Err(crate::error::AutoError::Msg(format!(
                        "STDASSEMBLY.PROVIDER_UNSUPPORTED: {}.{} has no browser adapter",
                        proof.module, proof.symbol
                    )));
                }
                if !self
                    .assembly_references
                    .iter()
                    .any(|p| p.native_id == Some(id))
                {
                    self.assembly_references.push(proof);
                }
            }
        }
        Ok(())
    }

    /// Plan 199: Emit SOURCE_LINE opcode if line number changed.
    /// Avoids redundant emission for consecutive statements on the same line.
    pub fn emit_source_line(&mut self, line: usize) {
        let line = line as u32;
        if line > 0 && line != self.current_source_line {
            self.current_source_line = line;
            self.emit(OpCode::SOURCE_LINE);
            self.emit_u16(line as u16);
        }
    }

    // Plan 073: Emit i16 value (2 bytes, little-endian) for jump offsets
    fn emit_i16(&mut self, val: i16) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    // Plan 073 Stage A.5: Emit f32 value (4 bytes, little-endian)
    fn emit_f32(&mut self, val: f32) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    // Plan 073 Stage A.5: Emit f64 value (8 bytes, little-endian)
    fn emit_f64(&mut self, val: f64) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    // Plan 073 Stage B: Emit i64 value (8 bytes, little-endian)
    fn emit_i64(&mut self, val: i64) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    // Plan 073 Stage B: Emit u64 value (8 bytes, little-endian)
    fn emit_u64(&mut self, val: u64) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    // Plan 073: Convert AST Key to ValueKey
    fn ast_key_to_value_key(&self, key: &crate::ast::Key) -> auto_val::ValueKey {
        match key {
            crate::ast::Key::NamedKey(name) => auto_val::ValueKey::Str(name.to_string().into()),
            crate::ast::Key::IntKey(i) => auto_val::ValueKey::Int(*i),
            crate::ast::Key::BoolKey(b) => auto_val::ValueKey::Bool(*b),
            crate::ast::Key::StrKey(s) => auto_val::ValueKey::Str(s.clone()),
        }
    }

    fn emit_placeholder_i16(&mut self) -> usize {
        let idx = self.code.len();
        self.code.extend_from_slice(&0i16.to_le_bytes());
        // Plan 088 Phase 4: Track this jump placeholder for multi-function compilation
        // When FN_PROLOG is inserted later, all subsequent jump placeholders need updating
        self.jump_placeholders.push(idx);
        idx
    }

    /// Backpatch a jump instruction
    /// The `jump_instr_idx` is the index of the placeholder (offset).
    /// Offset is relative to the *end* of the jump instruction (which is usually jump_instr_idx + 2).
    /// Target is `self.code.len()`.
    /// Offset = Target - (jump_instr_idx + 2)
    fn patch_jump(&mut self, placeholder_idx: usize) {
        let target = self.code.len();
        // Jump instruction (OpCode + i16) = 3 bytes?
        // `emit_placeholder_i16` returns index of the i16, so OpCode is at -1.
        // IP advances by 3 (1 byte opcode + 2 bytes operand) -> No.
        // VM Logic check:
        // OpCode::JMP matches, reads i16.
        // engine.rs:
        //   let offset = self.flash.read_i16(self.ip);
        //   self.ip += 2;
        //   let new_ip = (self.ip as isize) + offset;
        // So offset is relative to the address *after* the JMP instruction (IP after fetching operand).
        // Address of placeholder is `placeholder_idx`.
        // Address of next instruction is `placeholder_idx + 2`.
        // So anchor = placeholder_idx + 2.

        let anchor = placeholder_idx + 2;
        let offset = (target as isize) - (anchor as isize);

        vm_debug!(
            "DEBUG patch_jump: placeholder_idx={}, target={}, anchor={}, offset={}",
            placeholder_idx,
            target,
            anchor,
            offset
        );

        // Plan 118 Phase 5: Track (placeholder, target) for offset recalculation after FN_PROLOG insertion
        // When FN_PROLOG is inserted, both placeholder and target positions may shift
        // and the offset needs to be recalculated
        self.jump_targets.push((placeholder_idx, target));

        // Check bounds
        if offset > i16::MAX as isize || offset < i16::MIN as isize {
            panic!("Jump offset too large: {}", offset);
        }

        let bytes = (offset as i16).to_le_bytes();
        self.code[placeholder_idx] = bytes[0];
        self.code[placeholder_idx + 1] = bytes[1];
    }

    /// Patch a jump placeholder to jump to a specific target position
    /// (not necessarily the current code position). Used by continue to
    /// jump to the increment step which is before the current position.
    fn patch_jump_to(&mut self, placeholder_idx: usize, target: usize) {
        let anchor = placeholder_idx + 2;
        let offset = (target as isize) - (anchor as isize);
        self.jump_targets.push((placeholder_idx, target));
        if offset > i16::MAX as isize || offset < i16::MIN as isize {
            panic!("Jump offset too large: {}", offset);
        }
        let bytes = (offset as i16).to_le_bytes();
        self.code[placeholder_idx] = bytes[0];
        self.code[placeholder_idx + 1] = bytes[1];
    }

    /// Add string constant to string pool and return its index
    /// 2026-08-22(编译期池 u32 化):返回 u16→u32。与运行期池修复同病:
    /// 应用常量超 65535 时编译期即索引回绕、静默串写。配套:所有携带
    /// 池索引的操作数字节码操作数(LDAD_STR/CREATE_NODE/PUSH_ACCUM/
    /// ACCUM_PAIR/CREATE_OBJ/GET_FIELD/CALL_SPEC/CLOSURE(名)/CAPTURE_*/
    /// LOAD|STORE_GLOBAL/CREATE_FUTURE)统一 2B→4B,engine/disasm 同步。
    pub fn add_string(&mut self, s: &str) -> u32 {
        if let Some(&idx) = self.string_dedup.get(s.as_bytes()) {
            return idx;
        }
        let idx = self.strings.len() as u32;
        let bytes = s.as_bytes().to_vec();
        self.strings.push(bytes.clone());
        self.string_dedup.insert(bytes, idx);
        idx
    }

    /// Get span from an expression for error reporting
    /// Plan 071 Phase 6.1: Helper for borrow checking errors
    fn get_expr_span(&self, _expr: &Expr) -> SourceSpan {
        // TODO: Track spans in AST nodes during parsing
        // For now, use a zero-length span at offset 0
        SourceSpan::new(0_usize.into(), 0_usize.into())
    }

    /// Register a type declaration in the type registry
    ///
    /// Plan 087 Phase 1: If the type has generic parameters, register as a generic template
    /// in the GenericRegistry. Otherwise, register as a regular type in the type registry.
    ///
    /// Plan 089: Also register type declaration in infer_ctx.type_registry
    /// This enables field type lookup in the infer module via TypeRegistry.
    pub fn register_type(&mut self, type_decl: &TypeDecl) {
        // Plan 089: Register type declaration in infer_ctx.type_registry
        // This allows infer/expr.rs to look up field types via TypeRegistry
        self.infer_ctx.register_type_decl(type_decl.clone());

        // Plan 089: Export type name for symbol resolution
        // Type names need to be exported so they can be looked up during relocation

        // Plan 087 Phase 1: Check if this is a generic type
        if !type_decl.generic_params.is_empty() {
            // Register as generic template
            self.register_generic_template(type_decl);
        } else {
            // Register as regular type
            let member_names: Vec<String> = type_decl
                .members
                .iter()
                .map(|m| m.name.to_string())
                .collect();

            let type_info = TypeInfo {
                _name: type_decl.name.to_string(),
                member_names,
            };

            self.types
                .insert(type_decl.name.to_string(), type_info.clone());

            // Plan 087 Phase 3: Also register non-generic types in generic_registry
            // This enables field access lookup for user-defined types
            self.register_generic_template(type_decl);

            // Create a ClassType for non-generic types using get_or_create_type
            // This allows get_type() to find non-generic types
            let type_args: Vec<Type> = vec![]; // Non-generic types have empty type args
            if let Ok(_class_type) = self
                .generic_registry
                .get_or_create_type(&type_decl.name.to_string(), type_args)
            {
                vm_debug!(
                    "DEBUG: Registered non-generic type '{}' in generic_registry",
                    type_decl.name
                );
            } else {
                eprintln!(
                    "Warning: Failed to create ClassType for '{}'",
                    type_decl.name
                );
            }
        }
    }

    // Plan 087 Phase 1: Register a generic type template
    ///
    /// Converts a TypeDecl with generic parameters into a ClassTemplate
    /// and registers it in the GenericRegistry.
    fn register_generic_template(&mut self, type_decl: &TypeDecl) {
        use crate::vm::generic_registry::{ClassTemplate, FieldDef, MethodInfo};

        // Convert members to FieldDef
        let fields: Vec<FieldDef> = type_decl
            .members
            .iter()
            .map(|m| FieldDef::new(m.name.to_string(), m.ty.clone()))
            .collect();

        // Convert methods to MethodInfo
        let mut methods = std::collections::HashMap::new();
        for method in &type_decl.methods {
            let method_info = MethodInfo::new(method.name.to_string(), method.clone());
            methods.insert(method.name.to_string(), method_info);
        }

        // Create ClassTemplate
        let template = ClassTemplate::new(
            type_decl.name.to_string(),
            type_decl.generic_params.clone(),
            fields,
            type_decl.methods.clone(),
        );

        // Register in GenericRegistry — skip if already registered (idempotent).
        // Child widgets receive the parent's import_stmts which re-compile the same
        // TypeDecls; the registry already has them, so this is expected, not an error.
        if let Err(e) = self.generic_registry.register_template(template) {
            if !e.contains("already registered") {
                eprintln!(
                    "Warning: Failed to register generic template '{}': {}",
                    type_decl.name, e
                );
            }
        }
    }

    /// Set a new inference context (Plan 089)
    ///
    /// Used to transfer type registry from Parser to Codegen.
    /// This ensures types registered during parsing are available for field lookup.

    /// Check if a name is a registered type (type, enum, or spec)
    ///
    /// Plan 123: Now delegates to TypeStore.is_type() which checks all type categories.
    ///
    /// Note: Plan 087 Phase 3 incorrectly added `|| self.var_types.contains_key(name)` here,
    /// which caused variables (like "l" for a List) to be treated as Types,
    /// breaking instance method calls (treated as static method calls).
    pub fn is_type(&self, name: &str) -> bool {
        self.type_store.read().unwrap().is_type(name)
    }

    /// Plan 080/118/087: record a `let/var` binding's initializer type into
    /// `var_types` so later instance-method calls on the variable can resolve
    /// the receiver's source-level type at compile time.
    ///
    /// Called ONCE at the top of the `Stmt::Store` handler, before the
    /// storage-path split. The module-level global fast path (Plan 322/327/
    /// 338) early-returns before the local-slot logic; when this tracking
    /// lived only below that return, top-level `var s = String.new()` never
    /// recorded its type, `s.push()` lost the compile-time
    /// `String.push -> auto.str.push` resolution, and the call degraded to
    /// runtime CALL_SPEC — which only knows the heap tag ("StringBuilder")
    /// and failed with `no function 'StringBuilder.push'` (dstr suite
    /// regression since Plan 338).
    fn track_store_var_type(&mut self, store: &crate::ast::Store) {
        let name_str = store.name.to_string();
        // PLAN-707 D-5: RHS 是流打开调用 → 登记持流变量（for-in 变量形态
        // 识别）；非流 RHS（含重赋值）移除标记，防误路由。
        if let Expr::Call(call) = &store.expr {
            if let Expr::Dot(_, method) = call.name.as_ref() {
                if matches!(
                    method.as_str(),
                    "get_stream" | "post_stream" | "post_stream_with_headers"
                ) {
                    self.stream_vars.insert(name_str.clone());
                } else {
                    self.stream_vars.remove(&name_str);
                }
            } else {
                self.stream_vars.remove(&name_str);
            }
        } else {
            self.stream_vars.remove(&name_str);
        }
        // Plan 080: Track variable type for instance method support
        // Plan 198 Phase 4: Replaced 120-line hardcoded if-chain with resolve_constructor_type()
        if let Expr::Call(call) = &store.expr {
            if let Expr::Dot(obj, method) = call.name.as_ref() {
                if let Expr::Ident(type_name) = obj.as_ref() {
                    if method == "new" || (type_name == "String" && method == "from") {
                        if let Some(resolved) = self.resolve_constructor_type(type_name) {
                            self.var_types.insert(name_str.clone(), resolved);
                        }
                    }
                }
            }
            // Plan 118 Phase 4: Track type instances from type constructor calls
            // Example: var duck = Duck(), var wing = Wing()
            else if let Expr::Ident(type_name) = call.name.as_ref() {
                if self.is_type(type_name)
                    && !self.rust_native_map.contains_key(type_name.as_str())
                    && !self.py_native_map.contains_key(type_name.as_str())
                {
                    // Create a TypeDecl with proper members from generic_registry
                    let type_decl = if self.generic_registry.has_template(type_name) {
                        // Create a TypeDecl from the template
                        let template = self.generic_registry.get_template(type_name).unwrap();
                        crate::ast::TypeDecl {
                            consts: Vec::new(),
                            name: crate::ast::Name::from(type_name),
                            kind: crate::ast::TypeDeclKind::UserType,
                            parent: None,
                            has: vec![],
                            specs: vec![],
                            spec_impls: vec![],
                            generic_params: vec![],
                            members: template
                                .fields
                                .iter()
                                .map(|f| crate::ast::Member {
                                    name: crate::ast::Name::from(f.name.as_str()),
                                    ty: f.field_type.clone(),
                                    value: None,
                                    attrs: Vec::new(),
                                })
                                .collect(),
                            delegations: vec![],
                            methods: vec![],
                            attrs: vec![],
                            impl_attrs: vec![],
                            doc: None,
                            is_pub: false,
                        }
                    } else if let Some(type_info) = self.get_type(type_name) {
                        // Create TypeDecl from TypeInfo (only has member names, use Unknown type)
                        crate::ast::TypeDecl {
                            consts: Vec::new(),
                            name: crate::ast::Name::from(type_name),
                            kind: crate::ast::TypeDeclKind::UserType,
                            parent: None,
                            has: vec![],
                            specs: vec![],
                            spec_impls: vec![],
                            generic_params: vec![],
                            members: type_info
                                .member_names
                                .iter()
                                .map(|name| crate::ast::Member {
                                    name: crate::ast::Name::from(name),
                                    ty: Type::Unknown,
                                    value: None,
                                    attrs: Vec::new(),
                                })
                                .collect(),
                            delegations: vec![],
                            methods: vec![],
                            attrs: vec![],
                            impl_attrs: vec![],
                            doc: None,
                            is_pub: false,
                        }
                    } else {
                        // Fallback: create minimal type decl
                        crate::ast::TypeDecl {
                            consts: Vec::new(),
                            name: crate::ast::Name::from(type_name),
                            kind: crate::ast::TypeDeclKind::UserType,
                            parent: None,
                            has: vec![],
                            specs: vec![],
                            spec_impls: vec![],
                            impl_attrs: vec![],
                            generic_params: vec![],
                            members: vec![],
                            delegations: vec![],
                            methods: vec![],
                            attrs: vec![],
                            doc: None,
                            is_pub: false,
                        }
                    };
                    self.var_types
                        .insert(name_str.clone(), Type::User(type_decl));
                    vm_debug!(
                        "DEBUG: Stored type constructor type for '{}' -> '{}' in var_types",
                        store.name,
                        type_name
                    );
                }
            }
        }
        // Plan 087 Phase 3: Track type instances from Node literals
        // Example: let c = Counter{count: 0}
        else if let Expr::Node(node) = &store.expr {
            let type_name = node.name.to_string();

            // Check if this is a user-defined generic type in GenericRegistry
            if self.generic_registry.has_template(&type_name) {
                // Get or create ClassType for this generic type
                let type_args = Vec::new(); // No explicit type args provided
                if let Ok(_class_type) = self
                    .generic_registry
                    .get_or_create_type(&type_name, type_args)
                {
                    // Create GenericInstance type to store in var_types
                    use crate::ast::GenericInstance;
                    let generic_inst = GenericInstance {
                        base_name: crate::ast::Name::from(type_name),
                        args: vec![],
                        source: None,
                    };
                    self.var_types
                        .insert(name_str.clone(), Type::GenericInstance(generic_inst));
                    vm_debug!(
                        "DEBUG: Stored generic type for '{}' in var_types",
                        store.name
                    );
                }
            } else if self.is_type(&type_name) {
                // Built-in type (List, HashMap, etc.)
                if let Some(_type_info) = self.get_type(&type_name) {
                    // Create a synthetic TypeDecl for type tracking
                    let type_decl = crate::ast::TypeDecl {
                        consts: Vec::new(),
                        name: crate::ast::Name::from(type_name),
                        kind: crate::ast::TypeDeclKind::UserType,
                        parent: None,
                        has: vec![],
                        specs: vec![],
                        spec_impls: vec![],
                        impl_attrs: vec![],
                        generic_params: vec![],
                        members: vec![],
                        delegations: vec![],
                        methods: vec![],
                        attrs: vec![],
                        doc: None,
                        is_pub: false,
                    };
                    self.var_types
                        .insert(name_str.clone(), Type::User(type_decl));
                }
            }
        }

        // Plan 087 Phase 3: Infer type from literal expressions for .type property support
        // If variable type not yet tracked (e.g., let x = 42), infer from expression
        if !self.var_types.contains_key(&name_str) {
            let ty = self.infer_expr_type(&store.expr);
            // Only store if we could infer a non-Unknown type
            if !matches!(ty, crate::ast::Type::Unknown) {
                vm_debug!(
                    "DEBUG: Inferred type for '{}' from expression: {:?}",
                    name_str,
                    ty
                );
                self.var_types.insert(name_str.clone(), ty.clone());
                // Sync with infer_ctx
                self.infer_ctx
                    .type_env
                    .insert(crate::ast::Name::from(&name_str), ty);
            }
        }
    }

    /// Get type information by name
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
        self.types.get(name)
    }

    /// Plan 128: Convert CodeGen output into CompiledPackage
    ///
    /// This is the final step of compilation - producing a "ROM cartridge"
    /// that can be loaded into the VM via VMLoader.
    ///
    /// The CompiledPackage contains all the data needed for execution:
    /// - Bytecode (linked, ready to execute)
    /// - String pool (all string literals)
    /// - Object metadata (keys and types for object literals)
    /// - Exported symbols (function entry points)
    /// - Task definitions (handler tables for message routing)
    pub fn into_compiled_package(self) -> crate::vm::loader::CompiledPackage {
        use crate::vm::loader::CompiledPackage;

        // Extract task definitions from the handler registry
        let tasks = self.task_handler_registry.export_task_definitions();

        CompiledPackage {
            bytecode: self.code,
            string_pool: self.strings,
            object_keys: self.object_keys,
            object_types: self.object_types,
            exports: self.exports,
            tasks,
            api_routes: self.api_routes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Branch, Expr, If, Name, Stmt};
    use crate::vm::opcode::OpCode;
    use auto_val::Op;

    #[test]
    fn test_codegen_expr_int() {
        let mut codegen = Codegen::new();
        codegen.compile_expr(&Expr::Int(42)).unwrap();
        assert_eq!(codegen.code[0], OpCode::CONST_I32 as u8);
        let val = i32::from_le_bytes(codegen.code[1..5].try_into().unwrap());
        assert_eq!(val, 42);
    }

    #[test]
    fn test_codegen_expr_binary() {
        let mut codegen = Codegen::new();
        let expr = Expr::Bina(Box::new(Expr::Int(1)), Op::Add, Box::new(Expr::Int(2)));
        codegen.compile_expr(&expr).unwrap();
        assert_eq!(codegen.code.len(), 11);
        assert_eq!(codegen.code[10], OpCode::ADD as u8);
    }

    #[test]
    fn test_codegen_if_stmt() {
        let mut codegen = Codegen::new();
        let stmt = Stmt::If(If {
            branches: vec![Branch {
                cond: Expr::Bool(true),
                body: Body::single_expr(Expr::Int(1)),
            }],
            else_: Some(Body::single_expr(Expr::Int(2))),
        });

        codegen.compile_stmt(&stmt).unwrap();

        let code = &codegen.code;
        // Layout since Plan 318 (PUSH_BOOL): Expr::Bool(true) compiles to
        // PUSH_BOOL + 1 operand byte (offsets 0-1), so JMP_IF_Z sits at 2
        // (operands 3-4), then-branch CONST_I32+4 at 5-9, JMP at 10
        // (operands 11-12), else-branch CONST_I32+4 at 13-17, end at 18.
        // JMP_IF_Z at 2 jumps to else start 13: 13 - (2+1+2) = 8.
        assert_eq!(code[0], OpCode::PUSH_BOOL as u8);
        assert_eq!(code[1], 1); // true
        assert_eq!(code[2], OpCode::JMP_IF_Z as u8);
        let else_offset = i16::from_le_bytes(code[3..5].try_into().unwrap());
        assert_eq!(else_offset, 8);

        // JMP at 10 jumps to end 18: 18 - (10+1+2) = 5.
        assert_eq!(code[10], OpCode::JMP as u8);
        let end_offset = i16::from_le_bytes(code[11..13].try_into().unwrap());
        assert_eq!(end_offset, 5);
    }

    #[test]
    fn test_codegen_fn() {
        let mut codegen = Codegen::new();
        // fn test_func() { return 42; }
        // AST: Fn { name: "test_func", params: [], body: [Return(42)], ret: Int }
        let fn_decl = crate::ast::Fn::new(
            crate::ast::FnKind::Function,
            "test_func".into(),
            None,
            vec![],
            Body {
                stmts: vec![Stmt::Return(Box::new(Expr::Int(42)))],
                has_new_line: false,
                source_lines: Vec::new(),
            },
            crate::ast::Type::Int,
        );
        let stmt = Stmt::Fn(fn_decl);

        codegen.compile_stmt(&stmt).unwrap();

        // Check exports
        assert!(codegen.exports.contains_key("test_func"));
        let entry_point = *codegen.exports.get("test_func").unwrap();

        // Code check
        assert_eq!(codegen.code[0], OpCode::JMP as u8);

        // JMP offset at index 1.
        let jump_offset = i16::from_le_bytes(codegen.code[1..3].try_into().unwrap());
        // Offset is relative to *end* of JMP instr (index 3).
        // Target is end of code.
        // So jump_offset = (TotalLen - 3).
        assert_eq!(codegen.code.len() as isize - 3, jump_offset as isize);

        // Entry point should be at index 3
        assert_eq!(entry_point, 3);
    }

    #[test]
    fn test_codegen_call() {
        let mut codegen = Codegen::new();
        // call foo(42)
        let call_expr = Expr::Call(crate::ast::Call {
            name: Box::new(Expr::Ident("foo".into())),
            args: crate::ast::Args {
                args: vec![crate::ast::Arg::Pos(Expr::Int(42))],
            },
            ret: crate::ast::Type::Unknown,
            type_args: vec![],
            generic_args: Vec::new(),
            pos: None,
        });

        codegen.compile_expr(&call_expr).unwrap();

        // Expected: CONST 42 (5 bytes) + CALL (1 byte) + Placeholder (4 bytes)
        assert_eq!(codegen.code[5], OpCode::CALL as u8);

        // Check Relocs
        assert_eq!(codegen.relocs.len(), 1);
        let reloc = &codegen.relocs[0];
        assert_eq!(reloc.symbol_name, "foo");
        assert_eq!(reloc.offset, 6); // Placeholder starts after CALL
        assert_eq!(reloc.reloc_type, RelocType::FuncCall);
    }

    #[test]
    fn test_codegen_closure_simple() {
        let mut codegen = Codegen::new();
        // Test: x => x + n
        // This is a closure that captures variable 'n' from outer scope
        use crate::ast::{Closure, ClosureParam};

        let closure = Closure {
            params: vec![ClosureParam::new("x".into(), None)],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::Ident("x".into())),
                Op::Add,
                Box::new(Expr::Ident("n".into())),
            )),
            is_move: false,
        };

        codegen.compile_expr(&Expr::Closure(closure)).unwrap();

        // Check that CLOSURE opcode was emitted
        // The bytecode should contain:
        // - LOAD_LOC_0 (load 'n' to capture)
        // - CLOSURE (0x90)
        // - func_addr (4 bytes, placeholder 0)
        // - capture_count (1 byte, value 1)
        // - var_name_idx (2 bytes, index of "n" in strings)

        // Find CLOSURE opcode
        let closure_pos = codegen
            .code
            .iter()
            .position(|&b| b == OpCode::CLOSURE as u8);
        assert!(closure_pos.is_some(), "CLOSURE opcode should be emitted");

        let closure_pos = closure_pos.unwrap();

        // Verify capture count (at pos + 1 + 4 = after opcode + func_addr)
        let capture_count = codegen.code[closure_pos + 5];
        assert_eq!(capture_count, 1, "Should capture 1 variable ('n')");

        // Verify string constant was added for "n"
        assert!(
            codegen.strings.iter().any(|s| s == b"n"),
            "String pool should contain 'n'"
        );
    }

    #[test]
    fn test_codegen_closure_multiple_captures() {
        let mut codegen = Codegen::new();
        // Test: x => x + a + b
        // This closure captures two variables: 'a' and 'b'
        use crate::ast::{Closure, ClosureParam};

        let closure = Closure {
            params: vec![ClosureParam::new("x".into(), None)],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::Bina(
                    Box::new(Expr::Ident("x".into())),
                    Op::Add,
                    Box::new(Expr::Ident("a".into())),
                )),
                Op::Add,
                Box::new(Expr::Ident("b".into())),
            )),
            is_move: false,
        };

        codegen.compile_expr(&Expr::Closure(closure)).unwrap();

        // Find CLOSURE opcode
        let closure_pos = codegen
            .code
            .iter()
            .position(|&b| b == OpCode::CLOSURE as u8);
        assert!(closure_pos.is_some(), "CLOSURE opcode should be emitted");

        let closure_pos = closure_pos.unwrap();

        // Verify capture count
        let capture_count = codegen.code[closure_pos + 5];
        assert_eq!(capture_count, 2, "Should capture 2 variables ('a' and 'b')");

        // Verify both strings were added
        assert!(
            codegen.strings.iter().any(|s| s == b"a"),
            "String pool should contain 'a'"
        );
        assert!(
            codegen.strings.iter().any(|s| s == b"b"),
            "String pool should contain 'b'"
        );
    }

    // Plan 073 Stage A: Type System Expansion Tests
    // These tests verify the new opcodes for float, double, and i64 support

    #[test]
    fn test_opcodes_f32_arithmetic() {
        use crate::vm::opcode::OpCode;

        // Verify f32 arithmetic opcodes exist and have correct values
        assert_eq!(OpCode::ADD_F as u8, 0x36);
        assert_eq!(OpCode::SUB_F as u8, 0x37);
        assert_eq!(OpCode::MUL_F as u8, 0x38);
        assert_eq!(OpCode::DIV_F as u8, 0x39);
        assert_eq!(OpCode::NEG_F as u8, 0x3A);
    }

    #[test]
    fn test_opcodes_f64_arithmetic() {
        use crate::vm::opcode::OpCode;

        // Verify f64 arithmetic opcodes exist and have correct values
        assert_eq!(OpCode::ADD_D as u8, 0x3B);
        assert_eq!(OpCode::SUB_D as u8, 0x3C);
        assert_eq!(OpCode::MUL_D as u8, 0x3D);
        assert_eq!(OpCode::DIV_D as u8, 0x3E);
        assert_eq!(OpCode::NEG_D as u8, 0x3F);
    }

    #[test]
    fn test_opcodes_f32_constant() {
        use crate::vm::opcode::OpCode;

        // Verify f32 constant opcode exists
        assert_eq!(OpCode::CONST_F32 as u8, 0x14);
    }

    #[test]
    fn test_opcodes_f64_constant() {
        use crate::vm::opcode::OpCode;

        // Verify f64 constant opcode exists
        assert_eq!(OpCode::CONST_F64 as u8, 0x15);
    }

    #[test]
    fn test_opcodes_i64_constant() {
        use crate::vm::opcode::OpCode;

        // Verify i64/u64 constant opcodes exist
        assert_eq!(OpCode::CONST_I64 as u8, 0x16);
        assert_eq!(OpCode::CONST_U64 as u8, 0x17);
    }

    // Plan 073 Stage A.5: Float/Double Codegen Tests

    #[test]
    fn test_codegen_float_literal() {
        let mut codegen = Codegen::new();
        codegen
            .compile_expr(&Expr::Float(3.14, "3.14".into()))
            .unwrap();

        // Should emit CONST_F32 (0x14) followed by 4 bytes
        assert_eq!(codegen.code[0], OpCode::CONST_F32 as u8);
        assert_eq!(codegen.code.len(), 5); // 1 opcode + 4 bytes for f32
    }

    #[test]
    fn test_codegen_double_literal() {
        let mut codegen = Codegen::new();
        codegen
            .compile_expr(&Expr::Double(2.718281828, "2.718281828".into()))
            .unwrap();

        // Should emit CONST_F64 (0x15) followed by 8 bytes
        assert_eq!(codegen.code[0], OpCode::CONST_F64 as u8);
        assert_eq!(codegen.code.len(), 9); // 1 opcode + 8 bytes for f64
    }

    #[test]
    fn test_codegen_float_addition() {
        let mut codegen = Codegen::new();
        // 1.5 + 2.5
        let expr = Expr::Bina(
            Box::new(Expr::Float(1.5, "1.5".into())),
            Op::Add,
            Box::new(Expr::Float(2.5, "2.5".into())),
        );

        codegen.compile_expr(&expr).unwrap();

        // Expected bytecode:
        // CONST_F32 (1 byte) + 1.5 (4 bytes) = 5 bytes
        // CONST_F32 (1 byte) + 2.5 (4 bytes) = 5 bytes
        // ADD_F (1 byte)
        // Total: 11 bytes
        assert_eq!(codegen.code.len(), 11);
        assert_eq!(codegen.code[0], OpCode::CONST_F32 as u8);
        assert_eq!(codegen.code[5], OpCode::CONST_F32 as u8);
        assert_eq!(codegen.code[10], OpCode::ADD_F as u8);
    }

    #[test]
    fn test_codegen_double_multiplication() {
        let mut codegen = Codegen::new();
        // 3.14 * 2.0
        let expr = Expr::Bina(
            Box::new(Expr::Double(3.14, "3.14".into())),
            Op::Mul,
            Box::new(Expr::Double(2.0, "2.0".into())),
        );

        codegen.compile_expr(&expr).unwrap();

        // Expected bytecode:
        // CONST_F64 (1 byte) + 3.14 (8 bytes) = 9 bytes
        // CONST_F64 (1 byte) + 2.0 (8 bytes) = 9 bytes
        // MUL_D (1 byte)
        // Total: 19 bytes
        assert_eq!(codegen.code.len(), 19);
        assert_eq!(codegen.code[0], OpCode::CONST_F64 as u8);
        assert_eq!(codegen.code[9], OpCode::CONST_F64 as u8);
        assert_eq!(codegen.code[18], OpCode::MUL_D as u8);
    }

    #[test]
    fn test_codegen_float_unary_negation() {
        let mut codegen = Codegen::new();
        // -3.14
        let expr = Expr::Unary(Op::Sub, Box::new(Expr::Float(3.14, "3.14".into())));

        codegen.compile_expr(&expr).unwrap();

        // Expected bytecode:
        // CONST_F32 (1 byte) + 3.14 (4 bytes) = 5 bytes
        // NEG_F (1 byte)
        // Total: 6 bytes
        assert_eq!(codegen.code.len(), 6);
        assert_eq!(codegen.code[0], OpCode::CONST_F32 as u8);
        assert_eq!(codegen.code[5], OpCode::NEG_F as u8);
    }

    #[test]
    fn test_codegen_double_unary_negation() {
        let mut codegen = Codegen::new();
        // -2.718
        let expr = Expr::Unary(Op::Sub, Box::new(Expr::Double(2.718, "2.718".into())));

        codegen.compile_expr(&expr).unwrap();

        // Expected bytecode:
        // CONST_F64 (1 byte) + 2.718 (8 bytes) = 9 bytes
        // NEG_D (1 byte)
        // Total: 10 bytes
        assert_eq!(codegen.code.len(), 10);
        assert_eq!(codegen.code[0], OpCode::CONST_F64 as u8);
        assert_eq!(codegen.code[9], OpCode::NEG_D as u8);
    }

    #[test]
    fn test_codegen_mixed_float_double_uses_double() {
        let mut codegen = Codegen::new();
        // 3.14 (f32) + 2.718 (f64)
        let expr = Expr::Bina(
            Box::new(Expr::Float(3.14, "3.14".into())),
            Op::Add,
            Box::new(Expr::Double(2.718, "2.718".into())),
        );

        codegen.compile_expr(&expr).unwrap();

        // Should use double precision when either operand is double
        // Layout: CONST_F32(5) + PROMOTE_F64(1) + CONST_F64(9) + ADD_D(1) = 16 bytes
        assert_eq!(codegen.code.len(), 16);
        assert_eq!(codegen.code[0], OpCode::CONST_F32 as u8);
        assert_eq!(codegen.code[5], OpCode::PROMOTE_F64 as u8);
        assert_eq!(codegen.code[6], OpCode::CONST_F64 as u8);
        assert_eq!(codegen.code[15], OpCode::ADD_D as u8);
    }

    #[test]
    fn test_codegen_variable_lookup_persists_after_code_clear() {
        use crate::ast::{Name, StoreKind, Type};

        let mut codegen = Codegen::new();

        // Compile: let x = 5
        let stmt = Stmt::Store(crate::ast::Store {
            kind: StoreKind::Let,
            attrs: vec![],
            is_pub: false,
            name: Name::from("x"),
            ty: Type::Unknown,
            expr: Expr::Int(5),
        });
        codegen.compile_stmt(&stmt).unwrap();

        // Verify variable is in scope
        assert_eq!(codegen.lookup_var("x"), Some(0));
        assert_eq!(codegen.scope_stack.last().unwrap().len(), 1);

        // Clear code (simulate REPL behavior)
        codegen.code.clear();

        // Verify variable lookup still works after clear
        assert_eq!(codegen.lookup_var("x"), Some(0));
        assert_eq!(codegen.scope_stack.last().unwrap().len(), 1);

        // Compile: x + 1
        let expr = Expr::Bina(
            Box::new(Expr::Ident(Name::from("x"))),
            Op::Add,
            Box::new(Expr::Int(1)),
        );
        codegen.compile_expr(&expr).unwrap();

        // Verify bytecode contains:
        // LOAD_LOC_0 (0x10) - load x
        // CONST_I32 1 (0x01 + 4 bytes)
        // ADD_I (0x30)
        assert_eq!(codegen.code[0], OpCode::LOAD_LOC_0 as u8);
        assert_eq!(codegen.code[1], OpCode::CONST_I32 as u8);
    }

    // Plan 121: Task/Msg statement compilation tests
    #[test]
    fn test_codegen_task_def_basic() {
        use crate::ast::TaskDef;
        use crate::token::Pos;

        let mut codegen = Codegen::new();

        // Create a basic task definition: task CounterTask { on { } }
        let pos = Pos {
            line: 1,
            at: 1,
            pos: 0,
            len: 0,
        };
        let task_def = TaskDef::new("CounterTask".into(), vec![], pos);

        let stmt = Stmt::TaskDef(task_def);
        let result = codegen.compile_stmt(&stmt);

        // Should compile successfully (no bytecode generated, just metadata)
        assert!(result.is_ok());

        // Task should be registered in types registry
        assert!(codegen.types.contains_key("CounterTask"));

        // No bytecode should be generated for task definitions
        assert_eq!(codegen.code.len(), 0);
    }

    #[test]
    fn test_codegen_task_def_singleton() {
        use crate::ast::{TaskAttr, TaskDef};
        use crate::token::Pos;

        let mut codegen = Codegen::new();

        // Create a singleton task: #[single] task SingletonTask { on { } }
        let pos = Pos {
            line: 1,
            at: 1,
            pos: 0,
            len: 0,
        };
        let task_def = TaskDef::new("SingletonTask".into(), vec![TaskAttr::Single], pos);

        let stmt = Stmt::TaskDef(task_def);
        let result = codegen.compile_stmt(&stmt);

        // Should compile successfully
        assert!(result.is_ok());

        // Task should be registered with #single marker
        let type_info = codegen.types.get("SingletonTask");
        assert!(type_info.is_some());
        assert!(type_info.unwrap()._name.contains("#single"));
    }

    #[test]
    fn test_codegen_task_def_with_state() {
        use crate::ast::TaskDef;
        use crate::token::Pos;

        let mut codegen = Codegen::new();

        // Create a task with state: task CounterTask { count mut = 0 }
        let pos = Pos {
            line: 1,
            at: 1,
            pos: 0,
            len: 0,
        };
        let mut task_def = TaskDef::new("CounterTask".into(), vec![], pos);
        task_def.add_state("count".into(), true, Expr::Int(0));

        let stmt = Stmt::TaskDef(task_def);
        let result = codegen.compile_stmt(&stmt);

        // Should compile successfully
        assert!(result.is_ok());

        // Task should be registered
        assert!(codegen.types.contains_key("CounterTask"));
    }

    // ===== Plan 378: method-call (Expr::Dot) I64/U64 recognition =====

    /// Helper: build a `recv.method()` Call expression.
    fn dot_call(recv: Expr, method: &str) -> Expr {
        use crate::ast::{Args, Call};
        Expr::Call(Call {
            name: Box::new(Expr::Dot(Box::new(recv), Name::from(method))),
            args: Args::new(),
            ret: Type::Unknown,
            type_args: vec![],
            generic_args: Vec::new(),
            pos: None,
        })
    }

    #[test]
    fn test_plan378_contains_u64_recognizes_dot_method_call() {
        // s.to_uint() should be recognised as containing a 64-bit value,
        // because str.to_uint is registered as I64 in fn_return_types.
        let mut codegen = Codegen::new();
        // Register `s` as a string variable so the type-name fallback resolves
        // str.to_uint (mirrors real codegen where `let s = "42"` records a
        // string type for `s`).
        codegen.var_types.insert("s".to_string(), Type::StrFixed(0));
        let expr = dot_call(Expr::Ident(Name::from("s")), "to_uint");
        assert!(
            codegen.contains_u64(&expr),
            "s.to_uint() should be detected as 64-bit (str.to_uint -> I64)"
        );
    }

    #[test]
    fn test_plan378_dot_method_returns_64_lookup() {
        // Direct helper check: with `s` typed as a string, the type-name
        // fallback should resolve str.to_uint -> I64.
        let mut codegen = Codegen::new();
        codegen.var_types.insert("s".to_string(), Type::StrFixed(0));
        let recv = Expr::Ident(Name::from("s"));
        let ty = codegen.lookup_dot_method_type(&recv, "to_uint");
        assert!(
            matches!(ty, Some(Type::I64)),
            "lookup_dot_method_type(s, to_uint) should resolve to I64, got {:?}",
            ty
        );
        assert!(codegen.dot_method_returns_64(&recv, "to_uint"));
    }

    #[test]
    fn test_plan378_i32_method_not_treated_as_64bit() {
        // Negative guard: str.find returns Int (1 slot) — must NOT be flagged 64-bit.
        let mut codegen = Codegen::new();
        codegen.var_types.insert("s".to_string(), Type::StrFixed(0));
        let expr = dot_call(Expr::Ident(Name::from("s")), "find");
        assert!(
            !codegen.contains_u64(&expr),
            "s.find() (Int) must not be misdetected as 64-bit"
        );
    }

    #[test]
    fn test_plan378_u64_comparison_opcode_selection() {
        // "100".to_uint() > "45".to_uint() must emit GT_U64 (not plain GT).
        // Uses string-literal receivers so no variable scope registration is
        // needed; infer_object_type resolves the receiver type to "str".
        let mut codegen = Codegen::new();
        let lhs = dot_call(Expr::Str(Name::from("100")), "to_uint");
        let rhs = dot_call(Expr::Str(Name::from("45")), "to_uint");
        let expr = Expr::Bina(Box::new(lhs), Op::Gt, Box::new(rhs));
        codegen.compile_expr(&expr).unwrap();
        assert!(
            codegen.code.contains(&(OpCode::GT_U64 as u8)),
            "\"100\".to_uint() > \"45\".to_uint() should emit GT_U64"
        );
    }
}

/// Plan 423 P5 加固:导出对齐校验 —— 函数导出地址必须落在 FN_PROLOG 字节
/// 上。返回应丢弃的毒化导出名(Stmt::Fn 编译中途失败的半成品)。任务标记
/// 导出(`<task>#start/#stop/#else`)指向非函数代码,按 `#` 豁免。
pub fn poisoned_fn_exports(
    code: &[u8],
    exports: &std::collections::HashMap<String, u32>,
) -> Vec<String> {
    exports
        .iter()
        .filter(|(n, &addr)| {
            // 非"函数入口"语义的导出族豁免:任务标记(#start/#stop/#else)、
            // async 块带外体(SPAWN_GO/.await 专用入口)与闭包体
            // (CALL_CLOSURE 协议入口)—— 三者均无 FN_PROLOG。
            let special =
                n.contains('#') || n.starts_with("async_block_body") || n.starts_with("closure_");
            !special
                && (addr as usize >= code.len() || code[addr as usize] != OpCode::FN_PROLOG as u8)
        })
        .map(|(n, _)| n.clone())
        .collect()
}

#[cfg(test)]
mod plan423_p5_tests {
    use super::*;

    #[test]
    fn poisoned_fn_exports_detects_misaligned_entries() {
        use std::collections::HashMap;
        let mut exports = HashMap::new();
        // 0xB8 = FN_PROLOG。code: [prolog, junk, prolog, junk]
        let code: Vec<u8> = vec![0xB8, 0x01, 0x00, 0x1F, 0xB8, 0x02, 0x00, 0x20];
        exports.insert("good_a".to_string(), 0); // 落在 0xB8
        exports.insert("poisoned".to_string(), 3); // 落在 0x1F(load.str)
        exports.insert("good_b".to_string(), 4); // 落在 0xB8
        exports.insert("task#start".to_string(), 3); // 任务标记豁免
        exports.insert("async_block_body_123".to_string(), 3); // async 带外体豁免(无 prolog)
        exports.insert("closure_68".to_string(), 3); // 闭包体豁免(CALL_CLOSURE 入口)
        exports.insert("oob".to_string(), 99); // 越界
        let mut dropped = poisoned_fn_exports(&code, &exports);
        dropped.sort();
        assert_eq!(dropped, vec!["oob".to_string(), "poisoned".to_string()]);
    }
}

// ---- PLAN-843 结构拆分：语法族主题模块（原单文件热点，纯搬移） ----
mod expr_async;
mod expr_calls;
mod expr_containers;
mod expr_control;
mod expr_dot_index;
mod expr_emit;
mod expr_fallback;
mod expr_ident;
mod expr_ops;

mod fn_emit;
mod stmt_emit;
mod ty_conv;
mod vars;
