use crate::ast::{Closure, Expr, Iter, ParamMode, Stmt, Type, TypeDecl};
use crate::error::SyntaxError;
use crate::error::{AutoError, AutoResult};
// use crate::val::Value; // Removed if not directly used or fix path
use crate::vm::loader::{Module, RelocEntry, RelocType};
use crate::vm::ffi::stdlib::NATIVE_RUST_STDLIB_DISPATCH;
use crate::vm::native::{NATIVE_ASSERT, NATIVE_ASSERT_EQ, NATIVE_ASSERT_NE, NATIVE_PRINT_F32, NATIVE_PRINT_I32, NATIVE_PRINT_STR};
use crate::vm::native_registry::BIGVM_NATIVES;
use crate::vm::opcode::OpCode;
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
use std::sync::{Arc, RwLock};

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

/// Type hint for f-string parts
enum FStrPartType {
    Int,     // i32 on stack (4 bytes)
    String,  // tagged string index (4 bytes)
    Float32, // f32 on stack (4 bytes)
    Float64, // f64 on stack (8 bytes)
    Uint64,  // u64 on stack (8 bytes, 2 slots)
}

/// Codegen: Compiles AST directly to AutoVM Bytecode
pub struct Codegen {
    pub code: Vec<u8>,
    pub exports: HashMap<String, u32>,
    pub relocs: Vec<RelocEntry>,
    pub intrinsics: HashMap<String, u16>,
    /// String constant pool
    pub strings: Vec<Vec<u8>>,
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

    /// Plan 073: Loop exit tracking for break/continue statements
    /// Each nested loop has a Vec of jump placeholders that need to be patched
    /// when the loop exits
    pub loop_exits: Vec<Vec<usize>>,

    /// Continue target tracking: stack of byte offsets to jump to on `continue`
    /// For range-based loops, this points to the increment step
    /// For iterator/collection loops, this points to the next-iteration check
    pub loop_continue_positions: Vec<usize>,

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

    /// Plan 118: Track the type of the last compiled expression for result formatting
    /// Used to format output correctly (e.g., byte as hex, uint with suffix)
    pub last_expr_type: ObjectType,

    /// Plan 127: Task handler registry for message routing
    /// Stores handler metadata for each task type
    pub task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry,

    /// Enum variant values: maps "EnumName.Variant" -> i32 value
    pub enum_values: HashMap<String, i32>,
}

impl Codegen {
    pub fn new() -> Self {
        // Initialize the global native registry
        crate::vm::native_registry::register_builtin_natives();

        let mut intrinsics = HashMap::new();
        // Register intrinsics - only built-in print functions
        // "print" defaults to print_str since most print calls are for strings
        intrinsics.insert("print".to_string(), NATIVE_PRINT_STR);
        intrinsics.insert("print_i32".to_string(), NATIVE_PRINT_I32);
        intrinsics.insert("print_f32".to_string(), NATIVE_PRINT_F32);
        intrinsics.insert("print_str".to_string(), NATIVE_PRINT_STR);
        intrinsics.insert("assert".to_string(), NATIVE_ASSERT);
        intrinsics.insert("assert_eq".to_string(), NATIVE_ASSERT_EQ);
        intrinsics.insert("assert_ne".to_string(), NATIVE_ASSERT_NE);

        // Register return types for native functions (used for type inference in let bindings)
        let fn_return_types = Self::build_fn_return_types();

        // Create global scope
        let locals = HashMap::new();
        let mut scope_stack = Vec::new();
        scope_stack.push(locals);

        Self {
            code: Vec::new(),
            exports: HashMap::new(),
            relocs: Vec::new(),
            intrinsics,
            strings: Vec::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            locals: HashMap::new(),
            scope_stack,
            var_types: HashMap::new(), // Plan 080: variable type tracking
            var_mutability: HashMap::new(), // Plan 080+: variable mutability tracking
            captured_vars_stack: Vec::new(),
            loop_exits: Vec::new(),
            loop_continue_positions: Vec::new(),
            types: HashMap::new(),
            generics: GenericTable::new(), // Plan 076 Phase 1
            generic_registry: crate::vm::generic_registry::GenericRegistry::new(), // Plan 087 Phase 1
            fn_params: HashMap::new(), // Plan 088 Phase 4: function parameter information
            fn_return_types: HashMap::new(), // Plan 087 Phase 3: function return types for .type
            current_fn_n_args: 0,      // Plan 087 Phase 3: Initialize to 0
            current_fn_ret_type: Type::Void,
            fn_scope_start: 0,         // Plan 087 Phase 3: Initialize to 0
            infer_ctx: InferenceContext::new(), // Plan 087 Phase 3: Type inference context
            type_store: Arc::new(RwLock::new(types::TypeStore::new())), // Plan 084 Phase 3: Unified TypeStore
            jump_placeholders: Vec::new(), // Plan 088 Phase 4: Initialize empty jump placeholder tracking
            jump_targets: Vec::new(),      // Plan 118 Phase 5: Initialize jump target tracking
            max_locals: 0,
            should_pop_expr_result: false,
            last_expr_type: ObjectType::Int, // Plan 118: Default to Int
            task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry::new(), // Plan 127
            current_type_members: None, // Plan 087 Phase 3: No type context initially
            enum_values: HashMap::new(),
        }
    }

    /// Plan 084 Phase 3: Create Codegen with custom TypeStore
    /// Allows Parser and Codegen to share the same TypeStore instance
    /// Plan 123: Accept Arc<RwLock<TypeStore>> for shared access with Parser
    pub fn new_with_type_store(type_store: Arc<RwLock<types::TypeStore>>) -> Self {
        // Initialize the global native registry
        crate::vm::native_registry::register_builtin_natives();

        let mut intrinsics = HashMap::new();
        // Register intrinsics - only built-in print functions
        // "print" defaults to print_str since most print calls are for strings
        intrinsics.insert("print".to_string(), NATIVE_PRINT_STR);
        intrinsics.insert("print_i32".to_string(), NATIVE_PRINT_I32);
        intrinsics.insert("print_f32".to_string(), NATIVE_PRINT_F32);
        intrinsics.insert("print_str".to_string(), NATIVE_PRINT_STR);
        intrinsics.insert("assert".to_string(), NATIVE_ASSERT);
        intrinsics.insert("assert_eq".to_string(), NATIVE_ASSERT_EQ);
        intrinsics.insert("assert_ne".to_string(), NATIVE_ASSERT_NE);

        // Register return types for native functions (used for type inference in let bindings)
        let fn_return_types = Self::build_fn_return_types();

        // Create global scope
        let locals = HashMap::new();
        let mut scope_stack = Vec::new();
        scope_stack.push(locals);

        Self {
            code: Vec::new(),
            exports: HashMap::new(),
            relocs: Vec::new(),
            intrinsics,
            strings: Vec::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            locals: HashMap::new(),
            scope_stack,
            var_types: HashMap::new(),
            var_mutability: HashMap::new(),
            captured_vars_stack: Vec::new(),
            loop_exits: Vec::new(),
            loop_continue_positions: Vec::new(),
            types: HashMap::new(),
            generics: GenericTable::new(),
            generic_registry: crate::vm::generic_registry::GenericRegistry::new(),
            fn_params: HashMap::new(),
            fn_return_types,
            current_fn_n_args: 0,
            current_fn_ret_type: Type::Void,
            fn_scope_start: 0,
            infer_ctx: InferenceContext::new(),
            type_store, // Plan 084 Phase 3: Use provided TypeStore
            jump_placeholders: Vec::new(),
            jump_targets: Vec::new(),      // Plan 118 Phase 5: Initialize jump target tracking
            max_locals: 0,
            should_pop_expr_result: false,
            last_expr_type: ObjectType::Int, // Plan 118: Default to Int
            task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry::new(), // Plan 127
            current_type_members: None, // Plan 087 Phase 3: No type context initially
            enum_values: HashMap::new(),
        }
    }

    // Plan 076 Phase 1: Generic type tracking methods

    /// Track a generic type instantiation during compilation
    /// Returns the monomorphic name for this instantiation
    ///
    /// Example:
    /// ```ignore
    /// let list_type = Type::List(Box::new(Type::Int));
    /// let mono_name = codegen.track_generic(&list_type);
    /// assert_eq!(mono_name, "List_int");
    /// ```
    pub fn track_generic(&mut self, ty: &crate::ast::Type) -> Option<String> {
        if let Some(instance) = extract_generic_instance(ty) {
            let mono_name = self.generics.register(instance);
            Some(mono_name)
        } else {
            None
        }
    }

    /// Check if a type is a generic instantiation and get its monomorphic name
    pub fn get_monomorphic_name(&self, ty: &crate::ast::Type) -> Option<String> {
        if let Some(instance) = extract_generic_instance(ty) {
            Some(instance.monomorphic_name())
        } else {
            None
        }
    }

    /// Get all tracked generic instantiations
    pub fn get_generic_instantiations(&self) -> Vec<crate::vm::generic::GenericInstance> {
        self.generics.all().into_iter().cloned().collect()
    }

    /// Get all List instantiations (e.g., List<int>, List<string>)
    pub fn get_list_instantiations(&self) -> Vec<crate::vm::generic::GenericInstance> {
        self.generics
            .list_instantiations()
            .into_iter()
            .cloned()
            .collect()
    }

    // Plan 076 Phase 2: Monomorphization methods

    /// Perform monomorphization pass on all tracked generics
    /// Generates specialized bytecode for each generic instantiation
    pub fn monomorphize(&mut self) -> Vec<MonomorphizedModule> {
        let mut monomorphizer = Monomorphizer::new();

        // Transfer all tracked generics to the monomorphizer
        for instance in self.generics.all() {
            monomorphizer.register_generic(instance.clone());
        }

        // Generate specialized bytecode
        monomorphizer.monomorphize()
    }

    /// Check if a monomorphic module exists for a given name
    pub fn has_monomorphic_module(&self, name: &str) -> bool {
        self.generics.contains(name)
    }

    /// Get monomorphic name for a type if it's a generic instantiation
    pub fn get_monomorphic_name_checked(&self, ty: &crate::ast::Type) -> Option<String> {
        if let Some(instance) = extract_generic_instance(ty) {
            Some(instance.monomorphic_name())
        } else {
            None
        }
    }

    pub fn compile_stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                // Plan 089: Evaluate and discard result if this is not the last expression
                // of a block or script. This keeps the stack clean for subsequent ops.
                // Plan 118 Phase 7: Don't pop if expression is void (no value on stack)
                if self.should_pop_expr_result && self.last_expr_type != ObjectType::Void {
                    if matches!(self.last_expr_type, ObjectType::Double | ObjectType::Uint) {
                        self.emit(OpCode::POP_N);
                        self.code.push(2);
                    } else {
                        self.emit(OpCode::POP);
                    }
                }
            }
            Stmt::Block(body) => {
                // Plan 118 Phase 5: Blocks create a new scope for variable shadowing
                // Variables declared inside the block are scoped to the block
                self.push_scope();

                let n = body.stmts.len();
                for (i, s) in body.stmts.iter().enumerate() {
                    let is_last = i == n - 1;
                    let old_pop = self.should_pop_expr_result;
                    // Plan 118 Phase 5: For the last statement in a block, we should NOT pop
                    // the result because it's the block's return value.
                    // For non-last statements, we should pop to prevent stack growth.
                    if is_last {
                        self.should_pop_expr_result = false;
                    } else {
                        self.should_pop_expr_result = true;
                    }
                    self.compile_stmt(s)?;
                    self.should_pop_expr_result = old_pop;
                }

                self.pop_scope();
            }
            Stmt::If(if_stmt) => {
                let mut jumps_to_end = Vec::new();

                for branch in &if_stmt.branches {
                    // Cond
                    self.compile_expr(&branch.cond)?;

                    // JMP_IF_Z to Next Branch (or Else/End)
                    self.emit(OpCode::JMP_IF_Z);
                    let jump_to_next = self.emit_placeholder_i16();

                    // Body
                    self.compile_stmt(&Stmt::Block(branch.body.clone()))?;

                    // If True, JMP to End (skip other branches/else)
                    // Optimization: We could skip this for the very last block, but keeping it uniform is safer/easier.
                    self.emit(OpCode::JMP);
                    let jump_to_end = self.emit_placeholder_i16();
                    jumps_to_end.push(jump_to_end);

                    // Patch JMP_IF_Z to point here (Start of Next Branch)
                    self.patch_jump(jump_to_next);
                }

                if let Some(else_body) = &if_stmt.else_ {
                    self.compile_stmt(&Stmt::Block(else_body.clone()))?;
                }

                // Patch all "JMP to End" to point here
                for jump in jumps_to_end {
                    self.patch_jump(jump);
                }
            }
            Stmt::Fn(fn_decl) => {
                // Reset last_expr_type for each function to avoid stale type from previous compilation
                self.last_expr_type = ObjectType::Void;

                // #[vm] functions are implemented by native Rust shims, not VM bytecode.
                // If the native registry doesn't have a matching entry, the codegen will
                // emit a regular CALL to this function. Generate a runtime error stub
                // so the user gets a clear message instead of silent wrong behavior.
                if matches!(fn_decl.kind, crate::ast::FnKind::VmFunction) {
                    let fn_name_str = fn_decl.name.to_string();
                    vm_debug!("DEBUG: Compiling #[vm] stub for '{}' — will panic at runtime if native not found",
                        fn_name_str
                    );

                    // Export the function so linker can resolve it
                    let entry_point = self.code.len() as u32;
                    self.exports.insert(fn_name_str.clone(), entry_point);

                    // FN_PROLOG with correct arg count
                    let n_args = fn_decl.params.len() as u8;
                    self.emit(OpCode::FN_PROLOG);
                    self.code.push(n_args);
                    self.code.push(0); // n_locals = 0

                    // Push error message as string constant
                    let err_msg = format!(
                        "Runtime error: #[vm] function '{}' has no native implementation. \
                        Check that the native registry has a matching entry (e.g., \"str.{}\" or \"Str.{}\").",
                        fn_name_str,
                        fn_name_str.split('.').last().unwrap_or(&fn_name_str),
                        fn_name_str.split('.').last().unwrap_or(&fn_name_str)
                    );
                    let msg_idx = self.strings.len() as u16;
                    self.strings.push(err_msg.as_bytes().to_vec());
                    self.emit(OpCode::LOAD_STR);
                    self.code.extend_from_slice(&msg_idx.to_le_bytes());

                    // Call NATIVE_RUNTIME_PANIC — pops the message string and returns VMError
                    self.emit(OpCode::CALL_NAT);
                    self.code.extend_from_slice(
                        &crate::vm::native::NATIVE_RUNTIME_PANIC.to_le_bytes()
                    );

                    // RET — unreachable but needed for well-formed bytecode
                    let n_args_i16 = n_args as i16;
                    self.emit(OpCode::RET);
                    self.code.extend_from_slice(&n_args_i16.to_le_bytes());

                    // Record return type
                    self.fn_return_types.insert(fn_name_str.clone(), fn_decl.ret.clone());

                    return Ok(());
                }

                // 1. Jump over function body (so it's not executed during definition flow)
                self.emit(OpCode::JMP);
                let jump_over = self.emit_placeholder_i16();

                // 2. Record function entry point (export)
                // Entry point is HERE (after JMP instruction)
                let entry_point = self.code.len() as u32;
                vm_debug!("DEBUG: Exporting function '{}' at address {:#04x}",
                    fn_decl.name, entry_point
                );
                self.exports.insert(fn_decl.name.to_string(), entry_point);

                // 3. Push new scope for function locals
                self.push_scope();

                // 4. Compile function parameters
                // Plan 088 Phase 4: Store parameter types and modes for smart parameter passing
                let param_infos: Vec<ParamInfo> = fn_decl
                    .params
                    .iter()
                    .map(|param| ParamInfo {
                        ty: param.ty.clone(),
                        mode: param.mode,
                    })
                    .collect();

                // Store parameter information in fn_params map
                self.fn_params
                    .insert(fn_decl.name.to_string(), param_infos.clone());

                // Plan 087 Phase 3: Store function return type for .type property support
                self.fn_return_types
                    .insert(fn_decl.name.to_string(), fn_decl.ret.clone());

                // Plan 087 Phase 3: Set current function parameter count
                self.current_fn_n_args = fn_decl.params.len();
                self.current_fn_ret_type = fn_decl.ret.clone();

                // Plan 087 Phase 3: Record starting index for function scope
                // This is needed because outer scope variables affect parameter indices
                // Parameters will have indices: fn_scope_start, fn_scope_start+1, ...
                self.fn_scope_start = self.scope_stack.iter().map(|s| s.len()).sum();

                // Save and reset max_locals for this function
                let old_max_locals = self.max_locals;
                self.max_locals = 0;

                // Add parameters to scope
                // Plan 087 Phase 3: Record self parameter type for field access
                for param in &fn_decl.params {
                    self.add_var(&param.name);

                    // Register parameter type for method resolution (e.g., s.upper())
                    if !matches!(param.ty, Type::Unknown) {
                        self.var_types.insert(param.name.to_string(), param.ty.clone());
                    }

                    // Check if this is a 'self' parameter in a method
                    if param.name.to_string() == "self" {
                        // Extract type name from method name (e.g., "Counter.get" → "Counter")
                        if let Some(dot_pos) = fn_decl.name.to_string().find('.') {
                            let type_name = fn_decl.name.to_string()[..dot_pos].to_string();
                            vm_debug!("DEBUG: Recording self parameter type: {}", type_name);
                            // Create a synthetic TypeDecl for type tracking
                            if let Some(_type_info) = self.get_type(&type_name) {
                                let type_decl = crate::ast::TypeDecl {
                                    name: crate::ast::Name::from(type_name),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert("self".to_string(), Type::User(type_decl));
                            }
                        }
                    }
                }

                // 5. Compile body FIRST to count locals
                self.compile_stmt(&Stmt::Block(fn_decl.body.clone()))?;

                // Plan 118 Phase 7: Update function return type based on body inference
                // If parser defaulted to Void but body has implicit return, update the type
                // This allows proper void detection for calls like: fn hi(s str) { print(s); }; hi("hello")
                if matches!(fn_decl.ret, Type::Void) {
                    // Check if body actually returns a value (has implicit return)
                    if self.last_expr_type != ObjectType::Void {
                        // Body has implicit return - mark as non-void
                        // Use Unknown to indicate "has value but type unknown"
                        self.fn_return_types.insert(fn_decl.name.to_string(), Type::Unknown);
                    }
                } else {
                    // Function has explicit return type — record it for callers
                    self.fn_return_types.insert(fn_decl.name.to_string(), fn_decl.ret.clone());
                }

                // 6. Get number of locals and INSERT stack reservation at function entry
                let n_args = fn_decl.params.len();
                // Use max_locals to account for nested scopes correctly
                let n_locals = if self.max_locals > n_args {
                    self.max_locals - n_args
                } else {
                    0
                };

                // Plan 088 Phase 4: Always emit FN_PROLOG at function entry
                // This provides function metadata for dynamic parameter counting
                // IMPORTANT: Adjust exports FIRST (before inserting FN_PROLOG and RESERVE_STACK)
                // All function addresses > entry_point (after current function) will shift after insertion
                // NOTE: Current function (at entry_point) should NOT be adjusted!
                let mut adjusted_exports = std::collections::HashMap::new();
                for (name, &addr) in &self.exports {
                    if addr > entry_point {
                        // Note: > not >=
                        let shift = if n_locals > 0 { 5 } else { 3 }; // FN_PROLOG (3 bytes) + optional RESERVE_STACK (2 bytes)
                        adjusted_exports.insert(name.clone(), addr + shift);
                    }
                }
                // Apply the adjustments
                for (name, new_addr) in adjusted_exports {
                    self.exports.insert(name, new_addr);
                }

                // IMPORTANT: Adjust reloc offsets too!
                // Relocations that target positions >= entry_point will have their placeholder
                // positions shifted after insertion.
                let shift = if n_locals > 0 { 5 } else { 3 };
                for reloc in &mut self.relocs {
                    if reloc.offset >= entry_point {
                        reloc.offset += shift;
                    }
                }

                // Plan 088 Phase 4: Adjust jump placeholder indices BEFORE insertion!
                // Jump placeholders AFTER entry_point need to be shifted
                // Jump placeholders BEFORE or AT entry_point are NOT affected
                // (e.g., current function's jump_over at entry_point-2 stays at same position)
                // This MUST happen BEFORE code.insert() so patch_jump uses correct indices
                for placeholder_idx in &mut self.jump_placeholders {
                    if *placeholder_idx > entry_point as usize {
                        *placeholder_idx += shift as usize;
                    }
                }

                // Insert FN_PROLOG at entry_point (before function body)
                // This is 3 bytes: 1 byte opcode + 1 byte n_args + 1 byte n_locals
                vm_debug!("DEBUG: Emitting FN_PROLOG at address {}, n_args={}, n_locals={}",
                    entry_point, n_args, n_locals
                );
                self.code
                    .insert(entry_point as usize, OpCode::FN_PROLOG as u8);
                self.code.insert(entry_point as usize + 1, n_args as u8);
                self.code.insert(entry_point as usize + 2, n_locals as u8);

                // Insert RESERVE_STACK after FN_PROLOG (if needed)
                if n_locals > 0 {
                    // Insert RESERVE_STACK at entry_point + 3 (after FN_PROLOG)
                    // This is 2 bytes: 1 byte opcode + 1 byte operand
                    self.code
                        .insert(entry_point as usize + 3, OpCode::RESERVE_STACK as u8);
                    self.code.insert(entry_point as usize + 4, n_locals as u8);
                }

                // Plan 118 Phase 5: Recalculate jump offsets after FN_PROLOG insertion
                // After inserting FN_PROLOG (+ RESERVE_STACK if any), all jump targets have shifted.
                // We need to recalculate all jump offsets for jumps that were patched before this insertion.
                //
                // For each (placeholder_idx, target_idx) pair in jump_targets:
                // - Both placeholder and target may have shifted if they were after entry_point
                // - The offset needs to be recalculated using the new positions
                //
                // Cases:
                // 1. placeholder > entry_point, target > entry_point: Both shifted by `shift` bytes
                //    New offset = (target + shift) - (placeholder + shift + 2) = target - placeholder - 2
                //    Same as before! No change needed.
                // 2. placeholder <= entry_point, target > entry_point: Only target shifted
                //    New offset = (target + shift) - (placeholder + 2) = old_offset + shift
                // 3. placeholder > entry_point, target <= entry_point: Only placeholder shifted
                //    This shouldn't happen for forward jumps (target is always after placeholder)
                // 4. placeholder <= entry_point, target <= entry_point: Neither shifted
                //    No change needed.
                //
                // So we only need to fix case 2: jumps that START BEFORE entry_point and END AFTER entry_point
                let _shift_amount = shift as isize;
                for (old_placeholder, old_target) in &self.jump_targets {
                    // Calculate new positions
                    let new_placeholder = if *old_placeholder > entry_point as usize {
                        old_placeholder + shift as usize
                    } else {
                        *old_placeholder
                    };
                    let new_target = if *old_target > entry_point as usize {
                        old_target + shift as usize
                    } else {
                        *old_target
                    };

                    // Check if this jump crosses the insertion point
                    // (placeholder before or at entry_point, target after entry_point)
                    if *old_placeholder <= entry_point as usize && *old_target > entry_point as usize {
                        // Recalculate offset with shifted target
                        let new_anchor = new_placeholder + 2;
                        let new_offset = (new_target as isize) - (new_anchor as isize);

                        vm_debug!("DEBUG: Recalculating jump at {} (was {}): old_target={}, new_target={}, new_offset={}",
                            new_placeholder, old_placeholder, old_target, new_target, new_offset
                        );

                        if new_offset > i16::MAX as isize || new_offset < i16::MIN as isize {
                            panic!("Jump offset too large after recalculation: {}", new_offset);
                        }

                        let bytes = (new_offset as i16).to_le_bytes();
                        self.code[new_placeholder] = bytes[0];
                        self.code[new_placeholder + 1] = bytes[1];
                    }
                }

                // Restore max_locals
                self.max_locals = old_max_locals;

                // 7. Emit RET (or RET_D for 2-slot return types) at end of body
                let n_args = fn_decl.params.len() as u8;
                let ret_is_two_slot = matches!(self.current_fn_ret_type,
                    Type::Double | Type::U64 | Type::I64 | Type::USize);
                if ret_is_two_slot {
                    if matches!(self.current_fn_ret_type, Type::Double)
                        && !matches!(self.last_expr_type, ObjectType::Double)
                        && matches!(self.last_expr_type, ObjectType::Float) {
                        self.emit(OpCode::PROMOTE_F64);
                    }
                    self.emit(OpCode::RET_D);
                    self.code.push(n_args);
                } else {
                    self.emit(OpCode::RET);
                    self.code.push(n_args);
                }

                // 8. Pop function scope
                self.pop_scope();

                // Plan 087 Phase 3: Reset current function parameter count and scope start
                self.current_fn_n_args = 0;
                self.current_fn_ret_type = Type::Void;
                self.fn_scope_start = 0;

                // 9. Patch jump to skip body
                self.patch_jump(jump_over);
            }
            Stmt::Store(store) => {
                // Variable declaration: let/mut/var name = expr
                //
                // Immutability checking:
                // - let x = 5: creates immutable binding
                // - mut x = 5: creates mutable binding
                // - var x = 5: creates mutable binding
                // - x = 7: reassignment (error if x was declared with let)

                let name_str = store.name.to_string();
                let scope = self
                    .scope_stack
                    .last_mut()
                    .expect("Scope stack should never be empty");

                // Plan 091: Check if this is a new declaration or reassignment
                // New declaration (let/var) allows shadowing
                // Only assignment expressions should check immutability
                let is_new_declaration = matches!(
                    store.kind,
                    crate::ast::StoreKind::Let
                        | crate::ast::StoreKind::Var
                        | crate::ast::StoreKind::Const
                        | crate::ast::StoreKind::CVar
                        | crate::ast::StoreKind::Shared
                );

                if !is_new_declaration && scope.contains_key(&name_str) {
                    // This is a reassignment (not a new declaration) - check if variable is immutable
                    if let Some(&is_mutable) = self.var_mutability.get(&name_str) {
                        if !is_mutable {
                            // Variable was declared with 'let' (immutable) - reject reassignment
                            return Err(crate::error::AutoError::Msg(format!(
                                "Cannot reassign to immutable variable '{}' (declared with 'let')",
                                name_str
                            )));
                        }
                        // Variable is mutable - allow reassignment
                    }
                }

                if is_new_declaration || !scope.contains_key(&name_str) {
                    // First-time declaration - track mutability based on StoreKind
                    let is_mutable = matches!(
                        store.kind,
                        crate::ast::StoreKind::Var | crate::ast::StoreKind::CVar | crate::ast::StoreKind::Shared
                    );
                    self.var_mutability.insert(name_str.clone(), is_mutable);

                    // Plan 087 Phase 1: Handle generic type instantiations
                    // If the variable has an explicit generic type annotation (e.g., let p: Pair<int, string>),
                    // register the instantiation in the GenericRegistry
                    if let Type::GenericInstance(ref inst) = store.ty {
                        // Extract type arguments from GenericInstance
                        let type_args: Vec<Type> = inst.args.clone();

                        // Register or get the ClassType from GenericRegistry
                        if let Ok(_class_type) = self
                            .generic_registry
                            .get_or_create_type(&inst.base_name.to_string(), type_args)
                        {
                            // Store the complete type in var_types
                            self.var_types.insert(name_str.clone(), store.ty.clone());
                        } else {
                            eprintln!(
                                "Warning: Failed to register generic instance '{}'",
                                inst.base_name
                            );
                        }
                    } else if !matches!(store.ty, Type::Unknown) {
                        // Plan 118: Store the explicit type annotation for proper output formatting
                        self.var_types.insert(name_str.clone(), store.ty.clone());
                    } else {
                        // Plan 118 Phase 4: Infer type from expression when annotation is Unknown
                        // Plan 118 Phase 7: Add closure type inference
                        let inferred_type = match &store.expr {
                            Expr::Str(s) => Type::Str(s.len()),
                            Expr::CStr(_) => Type::CStr,
                            Expr::Char(_) => Type::Char,
                            Expr::Int(_) => Type::Int,
                            Expr::I8(_) => Type::Int,  // I8 maps to Int
                            Expr::I64(_) => Type::I64,
                            Expr::Uint(_) => Type::Uint,
                            Expr::U8(_) => Type::Int,  // U8 maps to Int (result is plain integer)
                            Expr::U64(_) => Type::U64,
                            Expr::Byte(_) => Type::Byte,
                            Expr::Float(_, _) => Type::Float,
                            Expr::Double(_, _) => Type::Double,
                            Expr::Bool(_) => Type::Bool,
                            // Plan 118 Phase 7: Closure type inference
                            // Infer fn(params) return_type for closure expressions
                            Expr::Closure(closure) => {
                                let param_types: Vec<Type> = closure.params.iter()
                                    .map(|p| p.ty.clone().unwrap_or(Type::Unknown))
                                    .collect();
                                // Infer return type from body expression
                                let ret_type = self.infer_expr_type(&closure.body);
                                Type::Fn(param_types, Box::new(ret_type))
                            }
                            // Plan 158: Infer type from function call
                                Expr::Call(call) => {
                                    if let Expr::Ident(fn_name) = call.name.as_ref() {
                                        self.fn_return_types.get(fn_name.as_ref())
                                            .cloned()
                                            .unwrap_or(Type::Unknown)
                                    } else if let Expr::Dot(obj, method) = call.name.as_ref() {
                                        // Instance method: try to infer from var_types + method return type
                                        if let Expr::Ident(obj_name) = obj.as_ref() {
                                            let full_name = format!("{}.{}", obj_name, method);
                                            if let Some(ty) = self.fn_return_types.get(&full_name) {
                                                ty.clone()
                                            } else if let Some(type_name) = self.infer_type_from_var(obj_name.as_ref()) {
                                                let type_method = format!("{}.{}", type_name, method);
                                                self.fn_return_types.get(&type_method)
                                                    .cloned()
                                                    .unwrap_or(Type::Unknown)
                                            } else {
                                                Type::Unknown
                                            }
                                        } else {
                                            // Plan 197 Task 3: Chained method call (e.g., Point.origin().to_str())
                                            // Resolve the type of the inner expression via fn_return_types
                                            let fn_name = format!("{}.{}", self.expr_to_name(obj.as_ref()), method.as_ref());
                                            if let Some(ty) = self.fn_return_types.get(&fn_name) {
                                                ty.clone()
                                            } else if let Some(type_name) = self.infer_user_type_name(obj.as_ref()) {
                                                let type_method = format!("{}.{}", type_name, method.as_ref());
                                                self.fn_return_types.get(&type_method)
                                                    .cloned()
                                                    .unwrap_or(Type::Unknown)
                                            } else {
                                                Type::Unknown
                                            }
                                        }
                                    } else {
                                        Type::Unknown
                                    }
                                }
                                _ => store.ty.clone(),
                        };
                        self.var_types.insert(name_str.clone(), inferred_type);
                    }
                }

                // Plan 052: Handle runtime array allocation (var arr [N]int)
                // If the type is RuntimeArray or static Array AND there's no initializer expression,
                // allocate the array. If there IS an initializer, compile it normally (CREATE_ARRAY).
                if let Type::RuntimeArray(ref rta) = store.ty {
                    if matches!(&store.expr, Expr::Nil | Expr::Int(0)) {
                        // No initializer - allocate zeroed array
                        self.compile_expr(&rta.size_expr)?;
                        self.emit(OpCode::CALL_NAT);
                        self.code.extend_from_slice(&190u16.to_le_bytes());
                    } else {
                        // Has initializer - compile the expression
                        self.compile_expr(&store.expr)?;
                    }
                } else if let Type::Array(ref arr) = store.ty {
                    if matches!(&store.expr, Expr::Nil | Expr::Int(0)) {
                        // No initializer - allocate zeroed array of static size
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(arr.len as i32);
                        self.emit(OpCode::CALL_NAT);
                        self.code.extend_from_slice(&190u16.to_le_bytes());
                    } else {
                        // Has initializer (e.g., [1, 2, 3]) - compile the expression
                        self.compile_expr(&store.expr)?;
                    }
                } else {
                    // Compile the RHS expression (pushes result on stack)
                    self.compile_expr(&store.expr)?;

                    // Infer 2-slot type from last_expr_type when store.ty is Unknown or narrower
                    // Double expression result overrides Float/Unknown store type
                    // Uint expression result overrides Int/Unknown store type
                    if matches!(self.last_expr_type, ObjectType::Double) {
                        if matches!(store.ty, Type::Unknown) || matches!(store.ty, Type::Float) {
                            self.var_types.insert(name_str.clone(), Type::Double);
                        }
                    } else if matches!(self.last_expr_type, ObjectType::Uint) {
                        if matches!(store.ty, Type::Unknown) || matches!(store.ty, Type::Int) {
                            self.var_types.insert(name_str.clone(), Type::U64);
                        }
                    }
                }

                // Promote i32 to u64 if variable type is u64/i64 but expression is not 64-bit
                let stored_type = self.var_types.get(&name_str).cloned();
                if matches!(stored_type, Some(Type::U64 | Type::I64))
                    && !self.contains_u64(&store.expr)
                {
                    self.emit(OpCode::TYPE_CAST_U64);
                } else if matches!(stored_type, Some(Type::Double))
                    && self.last_expr_type != ObjectType::Double
                {
                    match self.last_expr_type {
                        ObjectType::Float => {
                            self.emit(OpCode::PROMOTE_F64);
                        }
                        ObjectType::Int | ObjectType::Byte | ObjectType::Bool => {
                            self.emit(OpCode::I32_TO_F32);
                            self.emit(OpCode::PROMOTE_F64);
                        }
                        ObjectType::Uint => {
                            self.emit(OpCode::U64_TO_F64);
                        }
                        _ => {
                            self.emit(OpCode::PROMOTE_F64);
                        }
                    }
                }

                // Plan 080: Track variable type for instance method support
                // If the expression is a call like List.new(), track that the variable has type List
                if let Expr::Call(call) = &store.expr {
                    if let Expr::Dot(obj, method) = call.name.as_ref() {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            // Check if this is a known type (List, HashMap, etc.)
                            if type_name == "List" && method == "new" {
                                // Variable is being assigned a List
                                self.var_types.insert(
                                    store.name.to_string(),
                                    Type::List(Box::new(Type::Int)),
                                );
                            }
                            // Plan 086: Add collection type constructors
                            else if type_name == "HashMap" && method == "new" {
                                // HashMap<String, i32> - simplified version
                                // Create a synthetic TypeDecl to represent HashMap
                                let type_decl = crate::ast::TypeDecl {
                                    name: crate::ast::Name::from("HashMap"),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert(store.name.to_string(), Type::User(type_decl));
                            } else if type_name == "HashSet" && method == "new" {
                                let type_decl = crate::ast::TypeDecl {
                                    name: crate::ast::Name::from("HashSet"),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert(store.name.to_string(), Type::User(type_decl));
                            } else if type_name == "VecDeque" && method == "new" {
                                let type_decl = crate::ast::TypeDecl {
                                    name: crate::ast::Name::from("VecDeque"),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert(store.name.to_string(), Type::User(type_decl));
                            } else if type_name == "BTreeMap" && method == "new" {
                                let type_decl = crate::ast::TypeDecl {
                                    name: crate::ast::Name::from("BTreeMap"),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert(store.name.to_string(), Type::User(type_decl));
                            } else if type_name == "StringBuilder" && method == "new" {
                                let type_decl = crate::ast::TypeDecl {
                                    name: crate::ast::Name::from("StringBuilder"),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert(store.name.to_string(), Type::User(type_decl));
                            } else if type_name == "String" && (method == "new" || method == "from") {
                                let type_decl = crate::ast::TypeDecl {
                                    name: crate::ast::Name::from("String"),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: vec![],
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                };
                                self.var_types
                                    .insert(store.name.to_string(), Type::User(type_decl));
                            }
                            // Plan 087 Phase 3: Track user-defined type instances
                            // Example: let c = Counter.new()
                            else if self.is_type(type_name) {
                                // Get type info and create a synthetic TypeDecl
                                if let Some(_type_info) = self.get_type(type_name) {
                                    let type_decl = crate::ast::TypeDecl {
                                        name: crate::ast::Name::from(type_name),
                                        kind: crate::ast::TypeDeclKind::UserType,
                                        parent: None,
                                        has: vec![],
                                        specs: vec![],
                                        spec_impls: vec![],
                                        generic_params: vec![],
                                        members: vec![],
                                        delegations: vec![],
                                        methods: vec![],
                                        attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                    };
                                    self.var_types
                                        .insert(store.name.to_string(), Type::User(type_decl));
                                }
                            }
                        }
                    }
                    // Plan 118 Phase 4: Track type instances from type constructor calls
                    // Example: var duck = Duck(), var wing = Wing()
                    else if let Expr::Ident(type_name) = call.name.as_ref() {
                        if self.is_type(type_name) {
                            // Create a TypeDecl with proper members from generic_registry
                            let type_decl = if self.generic_registry.has_template(type_name) {
                                // Create a TypeDecl from the template
                                let template = self.generic_registry.get_template(type_name).unwrap();
                                crate::ast::TypeDecl {
                                    name: crate::ast::Name::from(type_name),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: template.fields.iter().map(|f| crate::ast::Member {
                                        name: crate::ast::Name::from(f.name.as_str()),
                                        ty: f.field_type.clone(),
                                        value: None,
                                        attrs: Vec::new(),
                                    }).collect(),
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                }
                            } else if let Some(type_info) = self.get_type(type_name) {
                                // Create TypeDecl from TypeInfo (only has member names, use Unknown type)
                                crate::ast::TypeDecl {
                                    name: crate::ast::Name::from(type_name),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
                                    generic_params: vec![],
                                    members: type_info.member_names.iter().map(|name| crate::ast::Member {
                                        name: crate::ast::Name::from(name.as_str()),
                                        ty: Type::Unknown,
                                        value: None,
                                        attrs: Vec::new(),
                                    }).collect(),
                                    delegations: vec![],
                                    methods: vec![],
                                    attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                                }
                            } else {
                                // Fallback: create minimal type decl
                                crate::ast::TypeDecl {
                                    name: crate::ast::Name::from(type_name),
                                    kind: crate::ast::TypeDeclKind::UserType,
                                    parent: None,
                                    has: vec![],
                                    specs: vec![],
                                    spec_impls: vec![],
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
                                .insert(store.name.to_string(), Type::User(type_decl));
                            vm_debug!("DEBUG: Stored type constructor type for '{}' -> '{}' in var_types",
                                store.name, type_name
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
                            self.var_types.insert(
                                store.name.to_string(),
                                Type::GenericInstance(generic_inst),
                            );
                            vm_debug!("DEBUG: Stored generic type for '{}' in var_types",
                                store.name
                            );
                        }
                    } else if self.is_type(&type_name) {
                        // Built-in type (List, HashMap, etc.)
                        if let Some(_type_info) = self.get_type(&type_name) {
                            // Create a synthetic TypeDecl for type tracking
                            let type_decl = crate::ast::TypeDecl {
                                name: crate::ast::Name::from(type_name),
                                kind: crate::ast::TypeDeclKind::UserType,
                                parent: None,
                                has: vec![],
                                specs: vec![],
                                spec_impls: vec![],
                                generic_params: vec![],
                                members: vec![],
                                delegations: vec![],
                                methods: vec![],
                                attrs: vec![],
                                    doc: None,
                                    is_pub: false,
                            };
                            self.var_types
                                .insert(store.name.to_string(), Type::User(type_decl));
                        }
                    }
                }

                // Plan 087 Phase 3: Infer type from literal expressions for .type property support
                // If variable type not yet tracked (e.g., let x = 42), infer from expression
                if !self.var_types.contains_key(&name_str) {
                    let ty = self.infer_expr_type(&store.expr);
                    // Only store if we could infer a non-Unknown type
                    if !matches!(ty, crate::ast::Type::Unknown) {
                        vm_debug!("DEBUG: Inferred type for '{}' from expression: {:?}",
                            name_str, ty
                        );
                        self.var_types.insert(name_str.clone(), ty.clone());
                        // Sync with infer_ctx
                        self.infer_ctx
                            .type_env
                            .insert(crate::ast::Name::from(&name_str), ty);
                    }
                }

                // Add variable to symbol table and get its index.
                //
                // Key insight: StoreKind::Var is used for BOTH `var x = 5` (new declaration)
                // and `x = 5` (reassignment). There is no separate Assign kind.
                //
                // Look up the variable across ALL scopes first:
                //   - If found: this is a reassignment or same-name re-declaration.
                //     For StoreKind::Let, always shadow (create new slot at inner scope).
                //     For StoreKind::Var/CVar, reuse the existing slot (don't shadow outer scope).
                //   - If not found: always create a new slot via add_var.
                //
                // This fixes the bug where `__out__ = __out__ + ...` inside a for-loop body
                // would create a new `__out__` slot in the inner scope instead of updating
                // the outer scope's slot.
                let var_index = if let Some(existing_index) = self.lookup_var(&name_str) {
                    match store.kind {
                        crate::ast::StoreKind::Let
                        | crate::ast::StoreKind::Const
                        | crate::ast::StoreKind::Shared => {
                            // `let x = ...` always creates a new slot, even if x exists in outer scope
                            self.add_var(&store.name)
                        }
                        crate::ast::StoreKind::Var | crate::ast::StoreKind::CVar => {
                            // `var x = ...` or `x = ...` (reassignment) reuses the existing slot
                            // from any scope to avoid accidental inner-scope shadowing
                            existing_index
                        }
                        crate::ast::StoreKind::Field => {
                            // Struct field: create new slot
                            self.add_var(&store.name)
                        }
                    }
                } else {
                    // Variable not found anywhere: always create new slot
                    self.add_var(&store.name)
                };

                // Store the value into the local variable
                let stored_type = self.var_types.get(&name_str).cloned();
                let is_two_slot = matches!(stored_type, Some(Type::U64 | Type::I64 | Type::Double))
                    || matches!(self.last_expr_type, ObjectType::Double | ObjectType::Uint);
                if is_two_slot {
                    // u64/i64 on stack: [low, high] (high on top)
                    // pop high first → var_index+1, then pop low → var_index
                    self.emit_store_loc(var_index + 1);
                    self.emit_store_loc(var_index);
                } else {
                    self.emit_store_loc(var_index);
                }

                // Plan 080: DON'T load the value back to stack
                // This avoids overlapping variable storage and stack
                // REPL will display the value from the expression result on stack
            }
            Stmt::Return(expr) => {
                self.compile_expr(expr)?;
                let n_args = self.current_fn_n_args as u8;
                let ret_is_two_slot = matches!(self.current_fn_ret_type,
                    Type::Double | Type::U64 | Type::I64 | Type::USize);
                if ret_is_two_slot {
                    // Promote 1-slot value to 2-slot if needed
                    if matches!(self.current_fn_ret_type, Type::Double) {
                        if !matches!(self.last_expr_type, ObjectType::Double) {
                            if matches!(self.last_expr_type, ObjectType::Float) {
                                self.emit(OpCode::PROMOTE_F64);
                            }
                            // For other types (int, etc.), I64_TO_F64 will be used by the caller
                        }
                    }
                    self.emit(OpCode::RET_D);
                    self.code.push(n_args);
                } else {
                    self.emit(OpCode::RET);
                    self.code.push(n_args);
                }
            }
            // Plan 124 Phase 2.3: reply statement for ask/reply RPC
            // reply expr -> compile expr, then send to oneshot channel
            Stmt::Reply(expr) => {
                // Phase 2.3: Simplified implementation
                // Full implementation would:
                // 1. Look up the reply channel from the current message context
                // 2. Compile the expression
                // 3. Send the value through the channel
                // For now, just compile the expression and leave it on stack
                self.compile_expr(expr)?;
                // TODO: Implement actual channel send when oneshot channels are ready
            }
            // Plan 073: TypeDecl support - register type metadata
            Stmt::TypeDecl(type_decl) => {
                // Register the type in the type registry
                self.register_type(type_decl);

                // Plan 087 Phase 3: Compile type methods as standalone functions
                // Method naming: TypeName.method_name (e.g., Counter.increment)
                // self becomes the first parameter
                let type_name = type_decl.name.to_string();
                let member_names: Vec<String> = type_decl.members.iter().map(|m| m.name.to_string()).collect();
                for method in &type_decl.methods {
                    // Create mangled method name: TypeName.method_name
                    let mangled_name = format!("{}.{}", type_name, method.name);

                    // Clone and modify the method for standalone compilation
                    let mut method_fn = method.clone();
                    method_fn.name = crate::ast::Name::from(mangled_name.as_str());
                    method_fn.parent = Some(crate::ast::Name::from(type_name.as_str()));

                    // For instance methods (non-static), inject 'self' as first parameter
                    if !method.is_static {
                        let has_self = method_fn.params.first().map(|p| p.name.to_string() == "self").unwrap_or(false);
                        if !has_self {
                            method_fn.params.insert(0, crate::ast::Param {
                                name: crate::ast::Name::from("self"),
                                ty: Type::User(type_decl.clone()),
                                default: None,
                                mode: crate::ast::ParamMode::View,
                            });
                        }
                    }

                    // Store member names for implicit field access resolution
                    self.current_type_members = Some(member_names.clone());

                    // Compile as a standalone function
                    self.compile_stmt(&Stmt::Fn(method_fn))?;

                    self.current_type_members = None;
                }
            }
            Stmt::Ext(ext_block) => {
                // Compile ext methods as standalone functions (same pattern as TypeDecl methods)
                let type_name = ext_block.target.to_string();

                // Look up the TypeDecl for self parameter typing
                let self_type = self.infer_ctx.lookup_type_decl(&ext_block.target)
                    .map(|td| Type::User(td))
                    .unwrap_or(Type::Unknown);

                // Store member names for implicit .field resolution
                let member_names: Vec<String> = self.get_type(&type_name)
                    .map(|ti| ti.member_names.clone())
                    .unwrap_or_default();

                for method in &ext_block.methods {
                    let mangled_name = format!("{}.{}", type_name, method.name);
                    let mut method_fn = method.clone();
                    method_fn.name = crate::ast::Name::from(mangled_name.as_str());
                    method_fn.parent = Some(crate::ast::Name::from(type_name.as_str()));

                    if !method.is_static {
                        let has_self = method_fn.params.first().map(|p| p.name.to_string() == "self").unwrap_or(false);
                        if !has_self {
                            method_fn.params.insert(0, crate::ast::Param {
                                name: crate::ast::Name::from("self"),
                                ty: self_type.clone(),
                                default: None,
                                mode: crate::ast::ParamMode::View,
                            });
                        }
                    }

                    // Set member names for implicit .field access resolution
                    self.current_type_members = Some(member_names.clone());
                    self.compile_stmt(&Stmt::Fn(method_fn))?;

                    // Also register short name (e.g., "upper") as export alias
                    // so `use auto.str: upper` can resolve via linker
                    let mangled_entry = self.exports.get(&mangled_name).copied();
                    if let Some(addr) = mangled_entry {
                        self.exports.insert(method.name.to_string(), addr);
                    }

                    self.current_type_members = None;
                }
            }
            Stmt::EnumDecl(enum_decl) => {
                // Register enum variant values for Cover(TagCover) compilation
                let enum_name = enum_decl.name.to_string();
                for (i, item) in enum_decl.items.iter().enumerate() {
                    let value = item.scalar_value.unwrap_or(i as i32);
                    let key = format!("{}.{}", enum_name, item.name);
                    self.enum_values.insert(key, value);
                }
            }
            Stmt::SpecDecl(_spec_decl) => {
                // Plan 073 Phase 8.6: Spec declaration support
                // Spec declarations (traits) don't generate bytecode at compile time
                // They register method signatures for type checking and constraint validation
                // TODO: Register spec in type registry for future use
                // For now, specs are metadata-only and used during type checking
            }
            // Plan 073: For statement support
            Stmt::For(for_stmt) => {
                vm_debug!("DEBUG FOR: Compiling for loop");
                // Push new loop exit tracking
                self.loop_exits.push(Vec::new());
                self.loop_continue_positions.push(0); // placeholder, will be set per variant

                // Handle range-based for loops: for x in start..end { ... }
                // Only support simple range iteration for now
                match &for_stmt.iter {
                    Iter::Named(var_name) => {
                        // Check if range is a Range expression (for x in 0..10)
                        if let Expr::Range(range) = &for_stmt.range {
                            vm_debug!("DEBUG FOR: Range-based loop, start={:?}, end={:?}, eq={}",
                                range.start, range.end, range.eq
                            );
                            // Compile start expression and initialize loop variable
                            self.compile_expr(&range.start)?;
                            vm_debug!("DEBUG FOR: After start expr, code len = {}",
                                self.code.len()
                            );

                            // Store to loop variable
                            let var_str = var_name.to_string();
                            vm_debug!("DEBUG FOR: Loop var = {}", var_str);
                            self.push_scope(); // New scope for loop variable
                                               // Calculate total index across all scopes
                            let var_index = self.add_var(&var_str);
                            vm_debug!("DEBUG FOR: var_index = {}", var_index);
                            self.emit_store_loc(var_index);
                            vm_debug!("DEBUG FOR: After store_loc, code len = {}", self.code.len());

                            // Loop start label
                            let loop_start = self.code.len() as i16;
                            vm_debug!("DEBUG FOR: Loop start at {}", loop_start);

                            // Load loop variable
                            self.emit_load_loc(var_index);

                            // Compile end expression
                            self.compile_expr(&range.end)?;

                            // Compare: if range.eq is true, use LE (<=), else use LT (<)
                            if range.eq {
                                self.emit(OpCode::LE); // Inclusive range: start..=end
                            } else {
                                self.emit(OpCode::LT); // Exclusive range: start..end
                            }

                            // JMP_IF_Z to end (exit loop if condition false)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Compile loop body
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                            // Continue target: increment loop variable
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(var_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(var_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len() as i16;
                            // Offset is from IP after reading the offset (current_pos + 2)
                            self.emit_i16(loop_start - current_pos - 2);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                        } else if let Expr::Call(_call) = &for_stmt.range {
                            // Plan 073: Iterator-based for loop: for x in list.iter() { ... }
                            // Compile the iterator call to get the iterator object
                            self.compile_expr(&for_stmt.range)?;

                            // Store iterator in a local variable
                            self.push_scope(); // New scope for loop variable and iterator
                            let iter_index = self.add_var("_iterator");
                            self.emit_store_loc(iter_index);

                            // Loop start label
                            let loop_start = self.code.len() as i16;

                            // Call iter.next() to get next element
                            self.emit_load_loc(iter_index); // Load iterator

                            // Emit CALL_NAT for Iterator.next
                            // Look up the native function ID
                            let native_id = if let Some(id) =
                                BIGVM_NATIVES.lock().unwrap().get_id("Iterator.next")
                            {
                                id
                            } else {
                                self.loop_exits.pop();
                                return Err(AutoError::Msg(
                                    "Iterator.next native function not found".to_string(),
                                ));
                            };
                            self.emit(OpCode::CALL_NAT);
                            self.code.extend_from_slice(&native_id.to_le_bytes());

                            // Check if result is nil (end of iteration)
                            // Nil is represented as -1 in our VM
                            // DUP the result so we can both check for nil AND store to loop variable
                            self.emit(OpCode::DUP);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(-1);
                            self.emit(OpCode::EQ);
                            self.emit(OpCode::JMP_IF_NZ);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Store the element to the loop variable
                            let var_str = var_name.to_string();
                            self.var_types.insert(var_str.clone(), Type::Int);
                            let var_index = self.add_var(&var_str);
                            self.emit_store_loc(var_index);

                            // Continue target: loop_start (re-check iterator.next())
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = loop_start as usize;
                            }

                            // Compile loop body
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len() as i16;
                            // Offset is from IP after reading the offset (current_pos + 2)
                            self.emit_i16(loop_start - current_pos - 2);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                        } else if let Expr::Ident(_) = &for_stmt.range {
                            // Plan 089: Array-based for loop: for x in array_var { ... }
                            // Load the array variable
                            self.compile_expr(&for_stmt.range)?;

                            // DUP to keep array reference for GET_ELEM later
                            self.emit(OpCode::DUP);

                            // Get array length (consumes the duped array_id)
                            self.emit(OpCode::ARRAY_LEN);

                            // Initialize loop counter to 0
                            // Stack now: [array_id, length]
                            self.emit(OpCode::CONST_0);
                            // Stack now: [array_id, length, 0]

                            // Store to loop variable
                            let var_str = var_name.to_string();
                            self.push_scope(); // New scope for loop variable

                            // Store counter (0) - stack top
                            let counter_index = self.add_var("_counter");
                            self.emit_store_loc(counter_index);
                            // Stack now: [array_id, length]

                            // Store length
                            let len_index = self.add_var("_array_len");
                            self.emit_store_loc(len_index);
                            // Stack now: [array_id]

                            // Store array reference for GET_ELEM
                            let array_ref_index = self.add_var("_array_ref");
                            self.emit_store_loc(array_ref_index);
                            // Stack now: []

                            // Create the actual loop variable slot (will be overwritten each iteration)
                            let var_index = self.add_var(&var_str);

                            // Loop start label
                            let loop_start = self.code.len() as i16;

                            // Load counter for comparison
                            self.emit_load_loc(counter_index);

                            // Compare with length
                            self.emit_load_loc(len_index);
                            self.emit(OpCode::LT);

                            // JMP_IF_Z to end (exit loop if counter >= length)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Load array reference
                            self.emit_load_loc(array_ref_index);

                            // Load current counter as index
                            self.emit_load_loc(counter_index);

                            // Get element at index (GET_ELEM: array_id, index -> value)
                            self.emit(OpCode::GET_ELEM);

                            // Store element to loop variable
                            self.emit_store_loc(var_index);

                            // Compile loop body
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                            // Continue target: increment counter
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(counter_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(counter_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len() as i16;
                            self.emit_i16(loop_start - current_pos - 2);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                        } else {
                            // For now, only support range, iterator, and array identifier expressions
                            self.loop_exits.pop();
                            self.loop_continue_positions.pop();
                            return Err(AutoError::Msg("For loops with non-range/non-iterator/non-array expressions not supported yet".to_string()));
                        }
                    }
                    Iter::Indexed(index_name, iter_name) => {
                        // Plan 073: Indexed iteration: for i, x in 0..10 { ... }
                        // Check if range is a Range expression
                        if let Expr::Range(range) = &for_stmt.range {
                            // Compile start expression and initialize loop variables
                            self.compile_expr(&range.start)?;

                            // Store to both index and value variables
                            let index_str = index_name.to_string();
                            let iter_str = iter_name.to_string();
                            self.push_scope(); // New scope for loop variables

                            // Store to index variable
                            let index_index = self.add_var(&index_str);
                            self.emit_store_loc(index_index);

                            // Store same value to iter variable
                            let iter_index = self.add_var(&iter_str);
                            self.emit_store_loc(iter_index);

                            // Loop start label
                            let loop_start = self.code.len() as i16;

                            // Load index variable for comparison
                            self.emit_load_loc(index_index);

                            // Compile end expression
                            self.compile_expr(&range.end)?;

                            // Compare: if range.eq is true, use LE (<=), else use LT (<)
                            if range.eq {
                                self.emit(OpCode::LE); // Inclusive range: start..=end
                            } else {
                                self.emit(OpCode::LT); // Exclusive range: start..end
                            }

                            // JMP_IF_Z to end (exit loop if condition false)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Compile loop body
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                            // Continue target: increment both loop variables
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(index_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(index_index);

                            // Update iter variable to match index
                            self.emit_load_loc(index_index);
                            self.emit_store_loc(iter_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len() as i16;
                            // Offset is from IP after reading the offset (current_pos + 2)
                            self.emit_i16(loop_start - current_pos - 2);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                        } else if let Expr::Ident(_) = &for_stmt.range {
                            // Plan 089: Indexed array iteration: for i, x in array_var { ... }
                            let index_str = index_name.to_string();
                            let iter_str = iter_name.to_string();
                            self.push_scope(); // New scope for loop variables

                            // Load the array variable
                            self.compile_expr(&for_stmt.range)?;
                            // Stack: [array_id]

                            // DUP to keep array reference
                            self.emit(OpCode::DUP);
                            // Stack: [array_id, array_id]

                            // Get array length (consumes one array_id)
                            self.emit(OpCode::ARRAY_LEN);
                            // Stack: [array_id, length]

                            // Initialize loop counter to 0
                            self.emit(OpCode::CONST_0);
                            // Stack: [array_id, length, 0]

                            // Store counter (0)
                            let counter_index = self.add_var("_counter");
                            self.emit_store_loc(counter_index);
                            // Stack: [array_id, length]

                            // Store length
                            let len_index = self.add_var("_array_len");
                            self.emit_store_loc(len_index);
                            // Stack: [array_id]

                            // Store array reference
                            let array_ref_index = self.add_var("_array_ref");
                            self.emit_store_loc(array_ref_index);
                            // Stack: []

                            // Store index variable
                            let index_var_index = self.add_var(&index_str);

                            // Store iter variable
                            let iter_var_index = self.add_var(&iter_str);

                            // Loop start label
                            let loop_start = self.code.len() as i16;

                            // Load counter
                            self.emit_load_loc(counter_index);

                            // Compare with length
                            self.emit_load_loc(len_index);
                            self.emit(OpCode::LT);

                            // JMP_IF_Z to end (exit loop if counter >= length)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Store current index to index variable
                            self.emit_load_loc(counter_index);
                            self.emit_store_loc(index_var_index);

                            // Load array reference
                            self.emit_load_loc(array_ref_index);

                            // Load current index
                            self.emit_load_loc(counter_index);

                            // Get element at index (GET_ELEM: array_id, index -> value)
                            self.emit(OpCode::GET_ELEM);

                            // Store to iter variable
                            self.emit_store_loc(iter_var_index);

                            // Compile loop body
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                            // Continue target: increment counter
                            let continue_pos = self.code.len();
                            if let Some(pos) = self.loop_continue_positions.last_mut() {
                                *pos = continue_pos;
                            }
                            self.emit_load_loc(counter_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(counter_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len() as i16;
                            self.emit_i16(loop_start - current_pos - 2);

                            // This is the loop exit point - patch all break jumps here
                            let _loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                            self.loop_continue_positions.pop();
                        } else {
                            // For now, only support range and array identifier expressions
                            self.loop_exits.pop();
                            self.loop_continue_positions.pop();
                            return Err(AutoError::Msg(
                                "Indexed for loops with non-range/non-array expressions not supported yet"
                                    .to_string(),
                            ));
                        }
                    }
                    Iter::Cond => {
                        // Conditional for loop: for condition { ... } (like while)
                        // Loop start label
                        let loop_start = self.code.len();

                        // Set continue position: loop_start (re-evaluate condition)
                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = loop_start;
                        }

                        // Compile condition
                        self.compile_expr(&for_stmt.range)?;

                        // JMP_IF_Z to end (exit loop if condition false)
                        self.emit(OpCode::JMP_IF_Z);
                        let jump_to_end = self.emit_placeholder_i16();

                        // Compile loop body
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len() as i16;
                        self.emit_i16(loop_start as i16 - current_pos - 2);

                        // This is the loop exit point - patch all break jumps here
                        let _loop_exit = self.code.len();

                        // Patch exit jump (for loop condition)
                        self.patch_jump(jump_to_end);

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                        self.loop_continue_positions.pop();
                    }
                    Iter::Ever => {
                        // Infinite loop: for ever { ... }
                        let loop_start = self.code.len();

                        // Set continue position: loop_start
                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = loop_start;
                        }

                        // Compile loop body
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len() as i16;
                        self.emit_i16(loop_start as i16 - current_pos - 2);

                        // This is the loop exit point - patch all break jumps here
                        let _loop_exit = self.code.len();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                        self.loop_continue_positions.pop();
                    }
                    Iter::Call(call) => {
                        // Plan 073: Iterator-based for loop: for x in list.iter() { ... }
                        // Compile the iterator call to get the iterator object
                        self.compile_expr(&Expr::Call(call.clone()))?;

                        // Store iterator in a local variable
                        self.push_scope(); // New scope for loop variable and iterator
                        let iter_index = self.add_var("_iterator");
                        self.emit_store_loc(iter_index);

                        // Loop start label
                        let loop_start = self.code.len() as i16;

                        // Call iter.next() to get next element
                        self.emit_load_loc(iter_index); // Load iterator

                        // Call next() method - this is a method call on the iterator
                        // For AutoVM, we need to call this as a native function
                        // iterator.next() should be compiled as a method call
                        // For now, we'll emit a CALL_NAT instruction for "Iterator.next"

                        // Get the variable name from the call
                        // The call should be like: list.iter() or iterator.next()
                        // For for x in list.iter(), the variable name should be extracted from context
                        // Since we don't have easy access to the variable name here, we'll use a placeholder
                        // The user should have: for x in list.iter()

                        // Emit CALL_NAT for Iterator.next
                        self.emit(OpCode::CALL_NAT);
                        // TODO: Get the native function ID for Iterator.next
                        // For now, use a placeholder ID
                        self.code.extend_from_slice(&0u32.to_le_bytes()); // Placeholder for native function ID

                        // Check if result is nil (end of iteration)
                        // Nil is represented as -1 in our VM
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(-1);
                        self.emit(OpCode::EQ);
                        self.emit(OpCode::JMP_IF_Z);
                        let jump_to_end = self.emit_placeholder_i16();

                        // Store the element to the loop variable
                        // Get variable name from context - for now, use "x" as default
                        let var_str = "x"; // TODO: Extract actual variable name from AST
                        let var_index = self.add_var(var_str);
                        self.emit_store_loc(var_index);

                        // Continue target: loop_start (re-check iterator.next())
                        if let Some(pos) = self.loop_continue_positions.last_mut() {
                            *pos = loop_start as usize;
                        }

                        // Compile loop body
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len() as i16;
                        self.emit_i16(loop_start - current_pos - 2);

                        // This is the loop exit point - patch all break jumps here
                        let _loop_exit = self.code.len();

                        // Patch exit jump (for loop condition)
                        self.patch_jump(jump_to_end);

                        // Pop loop scope
                        self.pop_scope();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                        self.loop_continue_positions.pop();
                    }
                }
            }
            Stmt::Break => {
                // Plan 073: Break statement support
                // Check if we're inside a loop
                if self.loop_exits.last().is_some() {
                    // Emit JMP instruction
                    self.emit(OpCode::JMP);
                    // Add placeholder to current loop's exit list
                    let exit_placeholder = self.emit_placeholder_i16();
                    self.loop_exits.last_mut().unwrap().push(exit_placeholder);
                } else {
                    return Err(AutoError::Msg(
                        "Break statement outside of loop".to_string(),
                    ));
                }
            }
            Stmt::Continue => {
                // Continue statement: jump to loop continue target
                if let Some(&continue_pos) = self.loop_continue_positions.last() {
                    self.emit(OpCode::JMP);
                    let current_pos = self.code.len() as i16;
                    let offset = continue_pos as i16 - current_pos - 2;
                    self.emit_i16(offset);
                } else {
                    return Err(AutoError::Msg(
                        "Continue statement outside of loop".to_string(),
                    ));
                }
            }
            // Plan 073: Is pattern matching statement
            Stmt::Is(is_stmt) => {
                // Evaluate target expression once and keep on stack
                self.compile_expr(&is_stmt.target)?;

                let mut end_jumps = Vec::new();

                // Process each branch
                for branch in &is_stmt.branches {
                    match branch {
                        crate::ast::IsBranch::EqBranch(patterns, body) => {
                            // Plan 120: Check for Option/Result pattern matching
                            // Use first pattern for special pattern types (Option/Result/Cover)
                            let pattern = &patterns[0];
                            match pattern {
                                // Legacy: None as expression (for backward compatibility)
                                crate::ast::Expr::None => {
                                    // Duplicate target for comparison
                                    self.emit(OpCode::DUP);
                                    // Check if value is None (-1)
                                    self.emit(OpCode::IS_NIL);
                                }
                                // Plan 120: OptionPattern - Some(x) or None in is statement
                                crate::ast::Expr::OptionPattern(opt_cover) => {
                                    match opt_cover.variant {
                                        crate::ast::OptionVariant::Some => {
                                            // Duplicate target for checking
                                            self.emit(OpCode::DUP);
                                            // Check if value is Some (not None)
                                            self.emit(OpCode::IS_SOME);

                                            // Jump to next branch if not matched
                                            self.emit(OpCode::JMP_IF_Z);
                                            let jump_to_next = self.emit_placeholder_i16();
                                            // If we have a binding, extract the value and store it
                                            if let Some(binding) = &opt_cover.binding {
                                                // The target is still on stack (from DUP)
                                                // Unwrap the Some value
                                                self.emit(OpCode::UNWRAP_SOME);
                                                // Store in local variable
                                                let var_idx = self.add_var(binding.as_str());
                                                self.emit(OpCode::STORE_LOCAL);
                                                self.emit_u16(var_idx as u16);
                                            } else {
                                                // Pop the duplicated target
                                                self.emit(OpCode::POP);
                                            }

                                            // Compile branch body
                                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                                            // Jump to end of is statement
                                            self.emit(OpCode::JMP);
                                            let jump_to_end = self.emit_placeholder_i16();
                                            end_jumps.push(jump_to_end);

                                            // Patch jump to next branch
                                            self.patch_jump(jump_to_next);
                                            continue; // Skip the default handling
                                        }
                                        crate::ast::OptionVariant::None => {
                                            // Duplicate target for comparison
                                            self.emit(OpCode::DUP);
                                            // Check if value is None (-1)
                                            self.emit(OpCode::IS_NIL);
                                        }
                                    }
                                }
                                // Plan 120: ResultPattern - Ok(x) or Err(e) in is statement
                                crate::ast::Expr::ResultPattern(res_cover) => {
                                    match res_cover.variant {
                                        crate::ast::ResultVariant::Ok => {
                                            // Duplicate target for checking
                                            self.emit(OpCode::DUP);
                                            // Check if value is Ok
                                            self.emit(OpCode::IS_OK);

                                            // Jump to next branch if not matched
                                            self.emit(OpCode::JMP_IF_Z);
                                            let jump_to_next = self.emit_placeholder_i16();
                                            // If we have a binding, extract the value and store it
                                            if let Some(binding) = &res_cover.binding {
                                                // The target is still on stack (from DUP)
                                                // Unwrap the Ok value
                                                self.emit(OpCode::UNWRAP_OK);
                                                // Store in local variable
                                                let var_idx = self.add_var(binding.as_str());
                                                self.emit(OpCode::STORE_LOCAL);
                                                self.emit_u16(var_idx as u16);
                                            } else {
                                                // Pop the duplicated target
                                                self.emit(OpCode::POP);
                                            }

                                            // Compile branch body
                                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                                            // Jump to end of is statement
                                            self.emit(OpCode::JMP);
                                            let jump_to_end = self.emit_placeholder_i16();
                                            end_jumps.push(jump_to_end);

                                            // Patch jump to next branch
                                            self.patch_jump(jump_to_next);
                                            continue; // Skip the default handling
                                        }
                                        crate::ast::ResultVariant::Err => {
                                            // Duplicate target for checking
                                            self.emit(OpCode::DUP);
                                            // Check if value is Err (not Ok)
                                            // IS_OK returns 1 if Ok, 0 if Err
                                            self.emit(OpCode::IS_OK);
                                            // Invert: 0 = Err (match), 1 = Ok (no match)
                                            self.emit(OpCode::CONST_I32);
                                            self.emit_i32(1);
                                            self.emit(OpCode::XOR);

                                            // Jump to next branch if not matched
                                            self.emit(OpCode::JMP_IF_Z);
                                            let jump_to_next = self.emit_placeholder_i16();
                                            // If we have a binding, extract the error and store it
                                            if let Some(binding) = &res_cover.binding {
                                                // The target is still on stack (from DUP)
                                                // Unwrap the Err value (error message)
                                                self.emit(OpCode::UNWRAP_ERR);
                                                // Store in local variable
                                                let var_idx = self.add_var(binding.as_str());
                                                self.emit(OpCode::STORE_LOCAL);
                                                self.emit_u16(var_idx as u16);
                                            } else {
                                                // Pop the duplicated target
                                                self.emit(OpCode::POP);
                                            }

                                            // Compile branch body
                                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                                            // Jump to end of is statement
                                            self.emit(OpCode::JMP);
                                            let jump_to_end = self.emit_placeholder_i16();
                                            end_jumps.push(jump_to_end);

                                            // Patch jump to next branch
                                            self.patch_jump(jump_to_next);
                                            continue; // Skip the default handling
                                        }
                                    }
                                }
                                // Legacy: Some/Ok/Err as expressions (for backward compatibility)
                                crate::ast::Expr::Some(inner) => {
                                    // Duplicate target for checking
                                    self.emit(OpCode::DUP);
                                    // Check if value is Some (not None)
                                    self.emit(OpCode::IS_SOME);

                                    let _ = inner; // Suppress unused warning
                                }
                                crate::ast::Expr::Ok(inner) => {
                                    // Duplicate target for checking
                                    self.emit(OpCode::DUP);
                                    // Check if value is Ok (not Err)
                                    self.emit(OpCode::IS_OK);

                                    let _ = inner; // Suppress unused warning
                                }
                                crate::ast::Expr::Err(msg) => {
                                    // Duplicate target for checking
                                    self.emit(OpCode::DUP);
                                    // Check if value is Err (not Ok)
                                    self.emit(OpCode::IS_OK);
                                    self.emit(OpCode::CONST_I32);
                                    self.emit_i32(1);
                                    self.emit(OpCode::XOR);

                                    let _ = msg; // Suppress unused warning
                                }
                                _ => {
                                    // Standard equality comparison for patterns
                                    // For multi-pattern (OR): compare each, OR results together
                                    if patterns.len() == 1 {
                                        // Single pattern: existing behavior
                                        self.emit(OpCode::DUP);
                                        self.compile_expr(pattern)?;
                                        self.emit(OpCode::EQ);
                                    } else {
                                        // Multi-pattern: save target, compare each with short-circuit OR
                                        // If any pattern matches, jump to matched label; otherwise fall through to next branch
                                        let target_slot = self.add_var("_is_target");
                                        self.emit(OpCode::STORE_LOCAL);
                                        self.emit_u16(target_slot as u16);

                                        let mut match_jumps = Vec::new();

                                        // First pattern
                                        self.emit(OpCode::LOAD_LOCAL);
                                        self.emit_u16(target_slot as u16);
                                        self.compile_expr(&patterns[0])?;
                                        self.emit(OpCode::EQ);
                                        self.emit(OpCode::JMP_IF_NZ);
                                        match_jumps.push(self.emit_placeholder_i16());

                                        // Subsequent patterns: if previous didn't match, try this one
                                        for pat in &patterns[1..] {
                                            self.emit(OpCode::LOAD_LOCAL);
                                            self.emit_u16(target_slot as u16);
                                            self.compile_expr(pat)?;
                                            self.emit(OpCode::EQ);
                                            self.emit(OpCode::JMP_IF_NZ);
                                            match_jumps.push(self.emit_placeholder_i16());
                                        }

                                        // No pattern matched — restore target and jump to next branch
                                        self.emit(OpCode::LOAD_LOCAL);
                                        self.emit_u16(target_slot as u16);
                                        self.emit(OpCode::JMP);
                                        let jump_to_next = self.emit_placeholder_i16();

                                        // === Matched label ===
                                        // Patch all JMP_IF_NZ to jump here
                                        let matched_pos = self.code.len();
                                        for j in &match_jumps {
                                            let anchor = *j + 2;
                                            let offset = (matched_pos as isize) - (anchor as isize);
                                            let bytes = (offset as i16).to_le_bytes();
                                            self.code[*j] = bytes[0];
                                            self.code[*j + 1] = bytes[1];
                                        }

                                        // Restore target on stack for body execution
                                        self.emit(OpCode::LOAD_LOCAL);
                                        self.emit_u16(target_slot as u16);

                                        // Compile branch body
                                        self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                                        // Jump to end of is statement
                                        self.emit(OpCode::JMP);
                                        let jump_to_end = self.emit_placeholder_i16();
                                        end_jumps.push(jump_to_end);

                                        // Patch jump to next branch (the fall-through JMP)
                                        self.patch_jump(jump_to_next);

                                        continue; // Skip the default handling below
                                    }
                                }
                            }

                            // Jump to next branch if not matched
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();

                            // Compile branch body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end of is statement
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);

                            // Patch jump to next branch
                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::IfBranch(condition, body) => {
                            // Plan 073: Evaluate condition expression
                            self.compile_expr(condition)?;

                            // Jump to next branch if condition is false (zero)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();

                            // Compile branch body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end of is statement
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);

                            // Patch jump to next branch
                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::ElseBranch(body) => {
                            // This is the default case - just compile body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end (in case there are more branches after else)
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            end_jumps.push(jump_to_end);
                        }
                    }
                }

                // Pop the target value from stack
                self.emit(OpCode::POP);

                // Patch all jump_to_end placeholders
                for jump_to_end in end_jumps {
                    self.patch_jump(jump_to_end);
                }
            }
            // Plan 121/127: Task/Msg support - compile task definition with handlers
            Stmt::TaskDef(task_def) => {
                // Register task type in the local types registry for lookup
                let task_name = task_def.name.to_string();

                // Store task metadata in local types HashMap
                self.types.insert(
                    task_name.clone(),
                    TypeInfo {
                        _name: if task_def.is_single() {
                            format!("{}#single", task_name)
                        } else {
                            task_name.clone()
                        },
                        member_names: Vec::new(),
                    },
                );

                // Plan 127: Create handler table for this task type
                let mut handler_table = crate::vm::task_handler::TaskHandlerTable::new(task_name.clone());

                // Compile lifecycle hooks if present
                // Start hook
                if let Some(ref start_hook) = task_def.start_hook {
                    let start_offset = self.code.len() as u32;
                    // Compile the hook body
                    self.push_scope();
                    for stmt in &start_hook.body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                    self.pop_scope();
                    self.emit(OpCode::RET);
                    // Store start hook offset (will be registered with TaskRegistry)
                    self.exports.insert(format!("{}#start", task_name), start_offset);
                }

                // Stop hook
                if let Some(ref stop_hook) = task_def.stop_hook {
                    let stop_offset = self.code.len() as u32;
                    // Compile the hook body
                    self.push_scope();
                    for stmt in &stop_hook.body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                    self.pop_scope();
                    self.emit(OpCode::RET);
                    // Store stop hook offset
                    self.exports.insert(format!("{}#stop", task_name), stop_offset);
                }

                // Plan 127: Compile each message handler in the on block
                let on_block = &task_def.on_block;
                let has_context = on_block.context_param.is_some();

                for (pattern, _guard, body) in &on_block.handlers {
                    // Record handler bytecode offset
                    let handler_offset = self.code.len() as u32;

                    // Add handler entry to table (pattern will be serialized)
                    #[allow(unused_variables)]
                    let pattern_idx = handler_table.add_handler(pattern, handler_offset, has_context);

                    // Compile handler body
                    // The handler receives message value on stack
                    // If has_context, also receives context id
                    for stmt in &body.stmts {
                        self.compile_stmt(stmt)?;
                    }

                    // Handler must return - emit RET
                    self.emit(OpCode::RET);

                    if crate::is_vm_debug() {
                        eprintln!("[TaskDef] Compiled handler {} for task {} at offset {}",
                            pattern_idx, task_name, handler_offset);
                    }
                }

                // Compile else handler if present
                if let Some(ref else_body) = on_block.else_handler {
                    let else_offset = self.code.len() as u32;
                    // Add else as a special handler with pattern_idx = 0xFFFFFFFF
                    // (handled specially at runtime)
                    for stmt in &else_body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                    self.emit(OpCode::RET);
                    self.exports.insert(format!("{}#else", task_name), else_offset);
                }

                // Register handler table
                self.task_handler_registry.register(handler_table);

                // Emit TASK_LOOP opcode if task has handlers
                if !on_block.handlers.is_empty() {
                    // Push task type name string index
                    let task_name_bytes = task_name.as_bytes().to_vec();
                    let task_name_idx = self.strings.len() as u16;
                    self.strings.push(task_name_bytes);

                    self.emit(OpCode::CONST_I32);
                    self.emit_i32(task_name_idx as i32);
                    self.emit(OpCode::TASK_LOOP);
                }

                // Note: Lifecycle hooks and message handlers are now compiled to bytecode.
                // The FFI shim_task_spawn function creates a TaskInstance and
                // registers it in the TaskRegistry.
            }
            Stmt::Node(node) => {
                // Stmt::Node wraps an Expr::Node — compile the expression
                self.compile_expr(&Expr::Node(node.clone()))?;
                if self.should_pop_expr_result && self.last_expr_type != ObjectType::Void {
                    self.emit(OpCode::POP);
                }
            }
            _ => {
                // TODO: Implement other statements
            }
        }
        Ok(())
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::Int(i) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*i);
            }
            Expr::Bool(b) => {
                self.last_expr_type = ObjectType::Bool;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(if *b { 1 } else { 0 });
            }
            // Plan 073 Stage A.5: Float literal support
            // Plan 118: Track type for output formatting
            Expr::Float(f, _) => {
                self.last_expr_type = ObjectType::Float;
                self.emit(OpCode::CONST_F32);
                self.emit_f32(*f as f32);
            }
            // Plan 073 Stage A.5: Double literal support
            Expr::Double(d, _) => {
                self.last_expr_type = ObjectType::Double;
                self.emit(OpCode::CONST_F64);
                self.emit_f64(*d);
            }
            // Plan 073 Stage B: I64 literal support
            Expr::I64(i) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I64);
                self.emit_i64(*i);
            }
            // Plan 073 Stage B: U64 literal support
            Expr::U64(u) => {
                self.emit(OpCode::CONST_U64);
                self.emit_u64(*u);
            }
            // Plan 073 Stage B: Uint literal support (use CONST_I32)
            // Plan 118: Track type for output formatting
            Expr::Uint(u) => {
                self.last_expr_type = ObjectType::Uint;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*u as i32);
            }
            // Plan 073 Stage B: I8 literal support (use CONST_I32)
            Expr::I8(i) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*i as i32);
            }
            // Plan 073 Stage B: U8 literal support (use CONST_I32)
            Expr::U8(u) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*u as i32);
            }
            // Plan 073 Stage B: Byte literal support (use CONST_I32)
            // Plan 118: Track type for output formatting (hex display)
            Expr::Byte(_b) => {
                self.last_expr_type = ObjectType::Byte;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*_b as i32);
            }
            // Plan 073 Stage B: Char literal support (use CONST_I32 for UTF-32 codepoint)
            Expr::Char(c) => {
                self.last_expr_type = ObjectType::Char;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*c as i32);
            }
            // Plan 073 Stage B: CStr literal support (use LOAD_STR like regular strings)
            Expr::CStr(s) => {
                // Add C string to constant pool and emit LOAD_STR <index>
                let bytes = s.as_bytes().to_vec();
                let idx = self.strings.len() as u16;
                self.strings.push(bytes);
                self.emit(OpCode::LOAD_STR);
                self.code.extend_from_slice(&idx.to_le_bytes());
            }
            // Plan 073: Object literal support {key: val, ...}
            Expr::Object(pairs) => {
                // Evaluate each value expression (pushes values onto stack)
                for pair in pairs {
                    self.compile_expr(&pair.value)?;
                }

                // Store keys in the object_keys pool
                let keys: Vec<auto_val::ValueKey> = pairs
                    .iter()
                    .map(|pair| self.ast_key_to_value_key(&pair.key))
                    .collect();
                let key_index = self.object_keys.len() as u16;

                // Plan 073: Track field types for runtime conversion
                let types: Vec<ObjectType> = pairs
                    .iter()
                    .map(|pair| self.infer_object_type(&pair.value))
                    .collect();
                self.object_types.push(types.clone());

                self.object_keys.push(keys);

                // Emit CREATE_OBJ with key_index and field count
                let field_count = pairs.len() as u8;
                self.emit(OpCode::CREATE_OBJ);
                self.code.extend_from_slice(&key_index.to_le_bytes());
                self.code.push(field_count);
            }
            // Plan 073: Array literal support [elem1, elem2, ...]
            Expr::Array(elems) => {
                // Evaluate each element expression (pushes values onto stack)
                for elem in elems {
                    self.compile_expr(elem)?;
                }

                // Emit CREATE_ARRAY with element count
                let elem_count = elems.len() as u8;
                self.emit(OpCode::CREATE_ARRAY);
                self.code.push(elem_count);
            }
            // Plan 073: Range expression support (0..10, 0..=10)
            Expr::Range(range) => {
                // Compile start expression (pushes onto stack)
                self.compile_expr(&range.start)?;

                // Compile end expression (pushes onto stack)
                self.compile_expr(&range.end)?;

                // Emit CREATE_RANGE or CREATE_RANGE_EQ based on range.eq
                if range.eq {
                    self.emit(OpCode::CREATE_RANGE_EQ); // Inclusive range: 0..=10
                } else {
                    self.emit(OpCode::CREATE_RANGE); // Exclusive range: 0..10
                }
            }
            // Plan 073: F-string support (f"hello $name")
            Expr::FStr(fstr) => {
                // Determine type tag for each part: 0=i32, 1=string, 2=f64, 3=f32
                let type_tags: Vec<u8> = fstr.parts.iter().map(|part| {
                    match self.expr_type_hint(part) {
                        FStrPartType::Int => 0,
                        FStrPartType::String => 1,
                        FStrPartType::Float64 => 2,
                        FStrPartType::Float32 => 3,
                        FStrPartType::Uint64 => 4,
                    }
                }).collect();

                // Compile each part expression (pushes values onto stack)
                for part in &fstr.parts {
                    self.compile_expr(part)?;
                }

                // Emit BUILD_FSTR with part count and type tags
                let part_count = fstr.parts.len() as u8;
                self.emit(OpCode::BUILD_FSTR);
                self.code.push(part_count);
                for &tag in &type_tags {
                    self.code.push(tag);
                }
                self.last_expr_type = ObjectType::String;
            }
            // Plan 073: Node support (for type instances like Point(10, 20))
            Expr::Node(node) => {
                // Plan 087 Phase 3: Check if this is a user-defined type instance
                let type_name = node.name.to_string();

                // Check if this is a type registered in generic_registry
                let is_registered_type = self.generic_registry.has_template(&type_name);

                if is_registered_type {
                    // Plan 087 Phase 2/3: Generic type or user-defined type instance
                    // Generate: [CONST_I32, length, NEW_INSTANCE, name_bytes..., CONSTRUCT_INSTANCE]
                    // VM: pop length, read name from code (after NEW_INSTANCE), push instance_id
                    // Note: NEW_INSTANCE does NOT push instance_id - VM will push it when executing the instruction
                    vm_debug!("DEBUG: Compiling type instance: {}", type_name);

                    // Get ClassType to determine mono_name and field count
                    let type_args = Vec::new(); // Non-generic types have empty type args
                    if let Ok(class_type) = self
                        .generic_registry
                        .get_or_create_type(&type_name, type_args)
                    {
                        let field_count = class_type.template.fields.len();
                        let mono_name = class_type.mono_name.clone();
                        vm_debug!("DEBUG: mono_name = '{}' ({} bytes)",
                            mono_name,
                            mono_name.len()
                        );
                        vm_debug!("DEBUG: mono_name bytes = {:?}", mono_name.as_bytes());
                        let name_bytes = mono_name.as_bytes();
                        let name_len = name_bytes.len();

                        // Plan 087 Phase 3: Generate bytecode in correct order
                        // CONSTRUCT_INSTANCE expects: [..., field_count, value1, value2, ..., instance_id]
                        // So we need to push field_count FIRST, then values, then instance_id

                        // Collect all field values from both args and body
                        let mut field_values = Vec::new();

                        // 1. Collect from args (if any, e.g., Counter(count: 42))
                        for arg in &node.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    field_values.push(expr.clone());
                                }
                                crate::ast::Arg::Pair(_, expr) => {
                                    field_values.push(expr.clone());
                                }
                                crate::ast::Arg::Name(_) => {
                                    // Name-only arg - treat as nil value
                                    field_values.push(crate::ast::Expr::Nil);
                                }
                            }
                        }

                        // 2. Collect from body stmts (for Counter{count: 42} syntax)
                        // The body contains Stmt::Expr(Expr::Pair(...)) for each field
                        for stmt in &node.body.stmts {
                            if let crate::ast::Stmt::Expr(expr) = stmt {
                                if let crate::ast::Expr::Pair(pair) = expr {
                                    // Extract the value from the pair
                                    field_values.push(pair.value.as_ref().clone());
                                }
                            }
                        }

                        vm_debug!("DEBUG codegen: field_count = {}, collected {} field values from args ({}), body ({})",
                            field_count,
                            field_values.len(),
                            node.args.args.len(),
                            node.body.stmts.len()
                        );

                        // Plan 087 Phase 3: Generate bytecode in correct order
                        // CONSTRUCT_INSTANCE pops: instance_id, then field_count, then field_count values
                        // So the stack should be: [..., value1, value2, ..., valueN, instance_id, field_count]

                        // 1. Compile each field value expression (pushes values onto stack)
                        // Stack: ..., value1, value2, ..., valueN
                        for (i, value_expr) in field_values.iter().enumerate() {
                            vm_debug!("DEBUG codegen: Compiling field value {}", i);
                            self.compile_expr(value_expr)?;
                            vm_debug!("DEBUG codegen: code.len() = 0x{:04x} after field value {}",
                                self.code.len(),
                                i
                            );
                        }

                        // 2. Push mono_name length onto stack (for NEW_INSTANCE to pop)
                        self.emit(OpCode::CONST_I32);
                        self.emit_u32(name_len as u32);

                        // 3. Emit NEW_INSTANCE instruction
                        // VM will pop length, read name_bytes from code, push instance_id
                        // Stack after: [..., value1, value2, ..., valueN, instance_id]
                        self.emit(OpCode::NEW_INSTANCE);

                        // 4. Emit mono_name bytes directly into code (AFTER NEW_INSTANCE instruction)
                        for &byte in name_bytes {
                            self.code.push(byte);
                        }

                        // 5. Push field_count (for CONSTRUCT_INSTANCE)
                        // Stack: [..., value1, value2, ..., valueN, instance_id, field_count]
                        self.emit(OpCode::CONST_I32);
                        self.emit_u32(field_count as u32);

                        // 6. Emit CONSTRUCT_INSTANCE
                        // Stack layout: [..., value1, value2, ..., valueN, instance_id, field_count]
                        self.emit(OpCode::CONSTRUCT_INSTANCE);
                        vm_debug!("DEBUG codegen: code.len() = 0x{:04x} after CONSTRUCT_INSTANCE",
                            self.code.len()
                        );
                    } else {
                        eprintln!(
                            "Warning: Failed to get/create ClassType for '{}'",
                            type_name
                        );
                        // Fallback to CREATE_OBJ (regular object)
                        let member_names = ["count".to_string()]; // Fallback

                        // Compile args
                        let arg_count = node.num_args as u8;
                        for arg in &node.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Pair(_, expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Name(_) => {
                                    self.emit(OpCode::CONST_0);
                                }
                            }
                        }

                        // Create object keys
                        let keys: Vec<auto_val::ValueKey> = member_names
                            .iter()
                            .take(arg_count as usize)
                            .map(|name| auto_val::ValueKey::Str(name.clone().into()))
                            .collect();
                        let key_index = self.object_keys.len() as u16;
                        self.object_keys.push(keys);

                        // Emit CREATE_OBJ
                        let field_count = arg_count.min(member_names.len() as u8);
                        self.emit(OpCode::CREATE_OBJ);
                        self.code.extend_from_slice(&key_index.to_le_bytes());
                        self.code.push(field_count);
                    }
                } else if let Some(type_info) = self.get_type(&type_name) {
                    // This is a type instance! Generate object creation instead of node
                    // Example: Point(10, 20) -> object with x: 10, y: 20

                    // Clone type_info to avoid holding immutable borrow
                    let member_names = type_info.member_names.clone();

                    // Compile each argument expression (pushes values onto stack)
                    let arg_count = node.num_args as u8;
                    for arg in &node.args.args {
                        match arg {
                            crate::ast::Arg::Pos(expr) => {
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Pair(_key, expr) => {
                                // For named args, compile the value
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Name(_) => {
                                // Name-only arg (placeholder for future)
                            }
                        }
                    }

                    // Create object keys using type member names
                    // Positional args map to type members in order
                    let keys: Vec<auto_val::ValueKey> = member_names
                        .iter()
                        .take(arg_count as usize) // Only take as many as we have args
                        .map(|name| auto_val::ValueKey::Str(name.clone().into()))
                        .collect();

                    // Register keys in object_keys pool
                    let key_index = self.object_keys.len() as u16;
                    self.object_keys.push(keys);

                    // Plan 087: Infer field types from node args
                    // For Node instances like Point{x: 1, y: 2}, infer types from args
                    let types: Vec<ObjectType> = node
                        .args
                        .args
                        .iter()
                        .take(arg_count as usize)
                        .map(|arg| {
                            match arg {
                                crate::ast::Arg::Pos(expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Pair(_, expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Name(_) => {
                                    ObjectType::Int // Default to Int
                                }
                            }
                        })
                        .collect();

                    // Register types in object_types pool
                    self.object_types.push(types);

                    // Emit CREATE_OBJ instead of CREATE_NODE
                    let field_count = arg_count.min(member_names.len() as u8);
                    self.emit(OpCode::CREATE_OBJ);
                    self.code.extend_from_slice(&key_index.to_le_bytes());
                    self.code.push(field_count);
                } else {
                    // Not a type - create generic Node
                    // Compile node name as string
                    let name_bytes = node.name.as_bytes().to_vec();
                    let name_idx = self.strings.len() as u16;
                    self.strings.push(name_bytes);

                    // Compile each argument expression (pushes values onto stack)
                    let arg_count = node.num_args as u8;
                    for arg in &node.args.args {
                        match arg {
                            crate::ast::Arg::Pos(expr) => {
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Pair(_key, expr) => {
                                // For named args, compile the value
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Name(_) => {
                                // Name-only arg (placeholder for future)
                            }
                        }
                    }

                    // Compile node body into props object (Plan 073)
                    let mut keys = Vec::new();
                    let mut types = Vec::new();
                    let mut prop_count = 0;

                    for stmt in &node.body.stmts {
                        if let crate::ast::Stmt::Expr(expr) = stmt {
                            if let crate::ast::Expr::Pair(pair) = expr {
                                let key_str = match &pair.key {
                                    crate::ast::Key::NamedKey(name) => name.to_string(),
                                    crate::ast::Key::StrKey(s) => s.to_string(),
                                    _ => format!("_prop{}", prop_count),
                                };
                                keys.push(auto_val::ValueKey::Str(key_str.into()));
                                
                                self.compile_expr(&pair.value)?;
                                types.push(self.infer_object_type(&pair.value));
                                prop_count += 1;
                            }
                        } else if let crate::ast::Stmt::Store(store) = stmt {
                            let key_str = store.name.to_string();
                            keys.push(auto_val::ValueKey::Str(key_str.into()));
                            
                            self.compile_expr(&store.expr)?;
                            types.push(self.infer_object_type(&store.expr));
                            prop_count += 1;
                        }
                    }

                    if prop_count > 0 {
                        let key_index = self.object_keys.len() as u16;
                        self.object_keys.push(keys);
                        self.object_types.push(types);

                        self.emit(OpCode::CREATE_OBJ);
                        self.code.extend_from_slice(&key_index.to_le_bytes());
                        self.code.push(prop_count as u8);
                    } else {
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(-1); // props_id
                    }

                    // For now, use -1 for kids_id
                    self.emit(OpCode::CONST_I32);
                    self.emit_i32(-1); // kids_id

                    // Compile node.id (if present) as string index, or 0xFFFF if absent
                    let id_idx = if !node.id.is_empty() {
                        let id_bytes = node.id.as_bytes().to_vec();
                        let idx = self.strings.len() as u16;
                        self.strings.push(id_bytes);
                        idx
                    } else {
                        0xFFFF // sentinel: no id
                    };

                    // Emit CREATE_NODE with name index, arg count, and id index
                    self.emit(OpCode::CREATE_NODE);
                    self.code.extend_from_slice(&name_idx.to_le_bytes());
                    self.code.push(arg_count);
                    self.code.extend_from_slice(&id_idx.to_le_bytes());
                }
            }
            Expr::Str(s) => {
                // Intern string literal: reuse existing index for identical strings
                // so that == compares identical tags
                let idx = self.add_string(s);
                self.emit(OpCode::LOAD_STR);
                self.code.extend_from_slice(&idx.to_le_bytes());
                // Plan 118 Phase 7: Track expression type for proper result formatting
                self.last_expr_type = ObjectType::String;
            }
            Expr::Ident(name) => {
                let name_str = name.to_string();
                vm_debug!("DEBUG: Compiling Ident: {}", name_str);

                // Plan 118: Check variable type for result formatting
                if let Some(var_type) = self.var_types.get(&name_str) {
                    self.last_expr_type = match var_type {
                        Type::Str(_) | Type::StrSlice | Type::String => ObjectType::String,
                        Type::Byte => ObjectType::Byte,
                        Type::Uint | Type::U64 => ObjectType::Uint,
                        Type::Float => ObjectType::Float,
                        Type::Double => ObjectType::Double,
                        _ => ObjectType::Int,
                    };
                }

                // Check if this is a captured variable (Plan 071)
                if let Some(_capture_index) = self.current_captured_vars().get(&name_str) {
                    // Variable is captured - emit LOAD_CAPTURED
                    vm_debug!("DEBUG: Variable {} is captured", name_str);
                    self.emit_load_captured(&name_str);
                } else if let Some(var_index) = self.lookup_var(&name_str) {
                    // Variable found in local scope - load it
                    self.emit_load_loc(var_index);
                    if matches!(self.var_types.get(&name_str), Some(Type::U64 | Type::I64 | Type::Double)) {
                        self.emit_load_loc(var_index + 1);
                    }
                } else {
                    // Plan 127: Check if this is an enum variant (e.g., Red from enum Color)
                    let enum_variant_value = self.type_store.read().unwrap()
                        .find_enum_variant_by_name(&name_str)
                        .map(|(_, v)| v);
                    if let Some(value) = enum_variant_value {
                        vm_debug!("DEBUG: Variable {} resolved as enum variant with value {}", name_str, value);
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(value);
                        self.last_expr_type = ObjectType::Int;
                    } else {
                        // Plan 087 Phase 3: Check implicit field access in methods
                        // If we're inside a type method, bare field names should resolve to self.field
                        if let Some(ref members) = self.current_type_members {
                            if members.contains(&name_str) {
                                if let Some(self_index) = self.lookup_var("self") {
                                    vm_debug!("DEBUG: Variable {} resolved as implicit self.{} access", name_str, name_str);
                                    self.emit_load_loc(self_index);
                                    // Determine type name from var_types["self"]
                                    let type_name = self.var_types.get("self")
                                        .and_then(|t| if let Type::User(td) = t { Some(td.name.to_string()) } else { None });
                                    if let Some(ref tn) = type_name {
                                        if self.generic_registry.has_template(tn) {
                                            if let Ok(class_type) = self.generic_registry.get_or_create_type(tn, vec![]) {
                                                if let Some(field_idx) = class_type.template.field_index(&name_str) {
                                                    self.emit(OpCode::GET_GENERIC_FIELD);
                                                    self.code.extend_from_slice(&(field_idx as u32).to_le_bytes());
                                                    self.last_expr_type = ObjectType::Int;
                                                    return Ok(());
                                                }
                                            }
                                        }
                                    }
                                    // Fallback: use GET_FIELD (name-based)
                                    let field_bytes = name_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u16;
                                    self.strings.push(field_bytes);
                                    self.emit(OpCode::GET_FIELD);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                    return Ok(());
                                }
                            }
                        }
                        vm_debug!("DEBUG: Variable {} NOT FOUND!", name_str);
                        // Plan 080: Variable not found - return proper error
                        // Even with skip_check=true in parser, we catch undefined variables here
                        return Err(AutoError::Msg(format!("Undefined variable: {}", name_str)));
                    }
                }
            }
            // Plan 073: Dot expression field access (obj.field)
            // Plan 087 Phase 2: Support generic instance field access
            Expr::Dot(obj, field) => {
                // Plan 123: Check if this is enum variant access (e.g., Color.Red)
                if let Expr::Ident(type_name) = obj.as_ref() {
                    // Extract value from type_store to release the borrow before emitting
                    let variant_value = self.type_store.read().unwrap()
                        .get_enum_variant_value(type_name.as_ref(), field.as_ref());
                    if let Some(value) = variant_value {
                        // Enum variant access - emit the variant value as integer
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(value);
                        return Ok(());
                    }
                }

                // Check if this is the .type property - returns type name as string
                if field.as_str() == "type" {
                    // Get the type of the object expression using infer module
                    let ty = self.infer_expr_type(obj);
                    // Get type name as string
                    let type_name = ty.unique_name();
                    // Add to string pool
                    let type_bytes = type_name.to_string().into_bytes();
                    let str_idx = self.strings.len() as u16;
                    self.strings.push(type_bytes);
                    // Emit LOAD_STR instruction
                    self.emit(OpCode::LOAD_STR);
                    self.code.extend_from_slice(&str_idx.to_le_bytes());
                    self.last_expr_type = ObjectType::String;
                    vm_debug!("DEBUG: .type property: obj={:?}, type_name={}",
                        obj, type_name
                    );
                    return Ok(());
                }

                // Plan 087 Phase 3: Check if this is field access on a user-defined type instance
                let is_user_type_instance = if let Expr::Ident(var_name) = obj.as_ref() {
                    // Look up variable type
                    if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                        matches!(var_type, Type::User(_))
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Check if obj is a generic instance variable
                let is_generic_instance = if let Expr::Ident(var_name) = obj.as_ref() {
                    // Look up variable type
                    if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                        matches!(var_type, Type::GenericInstance(_))
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_user_type_instance || is_generic_instance {
                    vm_debug!("DEBUG: Compiling field access: obj={:?}, field={}",
                        obj, field
                    );
                    // Plan 087 Phase 2/3: Generic instance or user-defined type field access
                    // Compile object expression (pushes instance_id onto stack)
                    self.compile_expr(obj)?;

                    // Get field index from type registry
                    if let Expr::Ident(var_name) = obj.as_ref() {
                        if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                            let type_name = match var_type {
                                Type::User(type_decl) => type_decl.name.to_string(),
                                Type::GenericInstance(inst) => {
                                    // Generate mono_name from base_name and args
                                    self.generic_registry
                                        .get_template(&inst.base_name.to_string())
                                        .map(|t| t.mono_name_from_args(&inst.args))
                                        .unwrap_or_else(|| format!("{}_unknown", inst.base_name))
                                }
                                _ => var_name.to_string(),
                            };

                            vm_debug!("DEBUG: Looking up type '{}' for field '{}'",
                                type_name, field
                            );
                            // Get ClassType to find field index
                            if let Some(class_type) = self.generic_registry.get_type(&type_name) {
                                let field_str = field.to_string();
                                if let Some(field_index) = class_type.field_index(&field_str) {
                                    vm_debug!("DEBUG: Field '{}' index = {}", field, field_index);
                                    // Emit GET_GENERIC_FIELD with field index
                                    self.emit(OpCode::GET_GENERIC_FIELD);
                                    self.emit_u32(field_index as u32);

                                    // Plan 118 Phase 7: Set last_expr_type based on field type
                                    // This is crucial for nested field access (obj.inner.x)
                                    if let Some(field_type) = class_type.field_type(&field_str) {
                                        self.last_expr_type = match field_type {
                                            Type::User(_) | Type::GenericInstance(_) => ObjectType::NestedObject,
                                            Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => ObjectType::String,
                                            Type::Char => ObjectType::Char,
                                            Type::Int | Type::I64 => ObjectType::Int,
                                            Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                                            Type::Byte => ObjectType::Byte,
                                            Type::Float => ObjectType::Float,
                                            Type::Double => ObjectType::Double,
                                            Type::Bool => ObjectType::Bool,
                                            Type::Array(_) | Type::RuntimeArray(_) => ObjectType::Array,
                                            _ => ObjectType::Int,
                                        };
                                        vm_debug!("DEBUG: Field '{}' type = {:?}, last_expr_type = {:?}",
                                            field, field_type, self.last_expr_type);
                                    }
                                } else {
                                    eprintln!(
                                        "Warning: Field '{}' not found in type '{}'",
                                        field, type_name
                                    );
                                    // Fallback: emit placeholder
                                    self.emit(OpCode::GET_GENERIC_FIELD);
                                    self.emit_u32(0);
                                }
                            } else {
                                eprintln!("Warning: Type '{}' not found in registry", type_name);
                                // Fallback to regular field access
                                self.emit(OpCode::GET_FIELD);
                                let field_str = field.to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u16;
                                self.strings.push(field_bytes);
                                self.emit_u16(field_idx);
                            }
                        } else {
                            // Fallback to regular field access
                            self.emit(OpCode::GET_FIELD);
                            let field_str = field.to_string();
                            let field_bytes = field_str.as_bytes().to_vec();
                            let field_idx = self.strings.len() as u16;
                            self.strings.push(field_bytes);
                            self.emit_u16(field_idx);
                        }
                    }
                } else {
                    // Regular field access (Plan 073)
                    // Or nested field access on user type (Plan 118 Phase 7)
                    // Compile object expression (should push object_id onto stack)
                    self.compile_expr(obj)?;

                    // Plan 118 Phase 7: Check if the result is a heap object (VmRef)
                    // If last_expr_type is NestedObject, we should use GET_GENERIC_FIELD
                    let is_heap_object = self.last_expr_type == ObjectType::NestedObject;

                    if is_heap_object {
                        // For heap objects (user type instances), use GET_GENERIC_FIELD
                        // Need to get type info from infer_expr_type
                        let obj_expr_type = self.infer_expr_type(obj);
                        let type_name = match &obj_expr_type {
                            Type::User(type_decl) => type_decl.name.to_string(),
                            Type::GenericInstance(inst) => {
                                self.generic_registry
                                    .get_template(&inst.base_name.to_string())
                                    .map(|t| t.mono_name_from_args(&inst.args))
                                    .unwrap_or_else(|| format!("{}_unknown", inst.base_name))
                            }
                            _ => "Unknown".to_string(),
                        };

                        if let Some(class_type) = self.generic_registry.get_type(&type_name) {
                            let field_str = field.to_string();
                            if let Some(field_index) = class_type.field_index(&field_str) {
                                self.emit(OpCode::GET_GENERIC_FIELD);
                                self.emit_u32(field_index as u32);

                                // Set last_expr_type based on field type
                                if let Some(field_type) = class_type.field_type(&field_str) {
                                    self.last_expr_type = match field_type {
                                        Type::User(_) | Type::GenericInstance(_) => ObjectType::NestedObject,
                                        Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => ObjectType::String,
                                        Type::Char => ObjectType::Char,
                                        Type::Int | Type::I64 => ObjectType::Int,
                                        Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                                        Type::Byte => ObjectType::Byte,
                                        Type::Float => ObjectType::Float,
                                        Type::Double => ObjectType::Double,
                                        Type::Bool => ObjectType::Bool,
                                        Type::Array(_) | Type::RuntimeArray(_) => ObjectType::Array,
                                        _ => ObjectType::Int,
                                    };
                                }
                            } else {
                                eprintln!("Warning: Field '{}' not found in type '{}' (nested access)",
                                    field, type_name);
                                self.emit(OpCode::GET_GENERIC_FIELD);
                                self.emit_u32(0);
                            }
                        } else {
                            // Type not in registry, fall back to GET_FIELD
                            let field_str = field.to_string();
                            let field_bytes = field_str.as_bytes().to_vec();
                            let field_idx = self.strings.len() as u16;
                            self.strings.push(field_bytes);
                            self.emit(OpCode::GET_FIELD);
                            self.code.extend_from_slice(&field_idx.to_le_bytes());
                        }
                    } else {
                        // Add field name to string pool and emit GET_FIELD <field_idx>
                        let field_str = field.to_string();
                        let field_bytes = field_str.as_bytes().to_vec();
                        let field_idx = self.strings.len() as u16;
                        self.strings.push(field_bytes);

                        self.emit(OpCode::GET_FIELD);
                        self.code.extend_from_slice(&field_idx.to_le_bytes());
                    }
                }
            }
            // Plan 073: Array indexing (arr[index])
            // Plan 118 Phase 4: Also supports string indexing (str[index] -> char)
            Expr::Index(arr, idx) => {
                // Compile array/string expression (should push array_id or tagged_str_idx onto stack)
                self.compile_expr(arr)?;
                // Compile index expression (should push index onto stack)
                self.compile_expr(idx)?;
                // Emit GET_ELEM (pops array_id/str_idx and index, pushes element value)
                self.emit(OpCode::GET_ELEM);

                // Plan 118 Phase 4: Set last_expr_type based on array element type or string char
                // Check if indexing a string literal
                if let Expr::Str(_) = arr.as_ref() {
                    // String indexing returns a character
                    self.last_expr_type = ObjectType::Char;
                } else {
                    // For arrays, try to infer element type
                    let arr_type = self.infer_object_type(arr);
                    match arr_type {
                        ObjectType::Array => {
                            // Could enhance this to track array element types
                            // For now, default to Int
                            self.last_expr_type = ObjectType::Int;
                        }
                        ObjectType::String => {
                            // String indexing returns char
                            self.last_expr_type = ObjectType::Char;
                        }
                        _ => {
                            self.last_expr_type = ObjectType::Int;
                        }
                    }
                }
            }
            Expr::Bina(lhs, op, rhs) => {
                // Assignment is special: compile RHS first, then store to LHS
                if matches!(
                    op,
                    Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq | Op::ModEq
                ) {
                    // For a += b, compile: LOAD_LOC(a), LOAD_CONST(b), ADD, STORE_LOC(a)
                    // IMPORTANT: Order matters! Stack must have [a, b] so DIV computes a/b (not b/a)
                    // Since binary ops pop b then a, we need a pushed before b
                    if let Expr::Ident(name) = lhs.as_ref() {
                        let name_str = name.to_string();

                        // Check if this is a captured variable (Plan 071)
                        if self.current_captured_vars().contains_key(&name_str) {
                            // Variable is captured - we need different handling
                            return Err(crate::error::AutoError::Msg(
                                "Compound assignment to captured variables not yet supported in AutoVM".to_string()
                            ));
                        } else if let Some(var_index) = self.lookup_var(&name_str) {
                            // Variable found in local scope
                            // Load variable FIRST (for correct operand order)
                            self.emit_load_loc(var_index);

                            // Compile RHS (value to add/sub/mul/div/mod)
                            self.compile_expr(rhs)?;

                            // Perform operation
                            self.emit(match op {
                                Op::AddEq => OpCode::ADD,
                                Op::SubEq => OpCode::SUB,
                                Op::MulEq => OpCode::MUL,
                                Op::DivEq => OpCode::DIV,
                                Op::ModEq => OpCode::MOD,
                                _ => OpCode::NOP,
                            });

                            // Plan 080: Duplicate value on stack because THIS IS AN EXPRESSION
                            // The parent (Stmt::Expr) or outer expression expects 1 value result!
                            self.emit(OpCode::DUP);
                            // Store result back to variable
                            self.emit_store_loc(var_index);
                        } else {
                            // Variable not found - error
                            return Err(crate::error::AutoError::Msg(format!(
                                "Undefined variable '{}' in compound assignment",
                                name_str
                            )));
                        }
                    } else {
                        // LHS is not an identifier - error for compound assignment
                        return Err(crate::error::AutoError::Msg(
                            "Compound assignment requires a variable on left side".to_string(),
                        ));
                    }
                } else if *op == Op::Asn {
                    // Compile RHS (value to store)
                    self.compile_expr(rhs)?;

                    // Check if LHS is an identifier (variable assignment)
                    if let Expr::Ident(name) = lhs.as_ref() {
                        let name_str = name.to_string();

                        // Coerce RHS to match LHS type if needed
                        let asn_stored_type = self.var_types.get(&name_str).cloned();
                        if matches!(asn_stored_type, Some(Type::U64 | Type::I64))
                            && !self.contains_u64(rhs.as_ref())
                        {
                            self.emit(OpCode::TYPE_CAST_U64);
                        } else if matches!(asn_stored_type, Some(Type::Double))
                            && self.last_expr_type != ObjectType::Double
                        {
                            if matches!(self.last_expr_type, ObjectType::Float) {
                                self.emit(OpCode::PROMOTE_F64);
                            }
                        }

                        // Check if this is a captured variable (Plan 071)
                        if self.current_captured_vars().contains_key(&name_str) {
                            // Variable is captured - emit STORE_CAPTURED
                            self.emit(OpCode::DUP); // Keep value for expression result
                            self.emit_store_captured(&name_str);
                        } else if let Some(var_index) = self.lookup_var(&name_str) {
                            // Variable found in local scope - check mutability
                            // Plan 118: Check if variable is immutable (declared with 'let')
                            if let Some(&is_mutable) = self.var_mutability.get(&name_str) {
                                if !is_mutable {
                                    return Err(crate::error::AutoError::Msg(format!(
                                        "Cannot reassign to immutable variable '{}' (declared with 'let')",
                                        name_str
                                    )));
                                }
                            }
                            // Variable is mutable - store value to it
                            let asn_is_two_slot = matches!(asn_stored_type, Some(Type::U64 | Type::I64 | Type::Double))
                                || matches!(self.last_expr_type, ObjectType::Double | ObjectType::Uint);
                            if asn_is_two_slot {
                                // u64/i64 on stack: [low, high] (high on top)
                                // Store high→var_index+1, then low→var_index
                                self.emit_store_loc(var_index + 1);
                                self.emit_store_loc(var_index);
                                // Reload for expression result
                                self.emit_load_loc(var_index);
                                self.emit_load_loc(var_index + 1);
                            } else {
                                self.emit(OpCode::DUP); // Keep value for expression result
                                self.emit_store_loc(var_index);
                            }
                        } else {
                            // Variable not found - this is an error
                            // For now, emit STORE_LOC_0 as a fallback
                            // TODO: Proper error handling for undefined variables
                            self.emit(OpCode::DUP); // Keep value for expression result
                            self.emit(OpCode::STORE_LOC_0);
                        }
                    } else if let Expr::Index(array, index) = lhs.as_ref() {
                        // Array element assignment: arr[index] = value
                        // Stack has: value (from RHS compilation above)
                        // Need to compile: array, index, then emit SET_ELEM
                        // Compile array expression
                        self.compile_expr(array)?;
                        // Compile index expression
                        self.compile_expr(index)?;
                        // Now stack has: value, array_id, index (need to reorder)
                        // SET_ELEM expects: array_id, index, value
                        // So we need to swap: value, array_id, index -> array_id, index, value
                        // For now, let's use a simpler approach:
                        // 1. Compile array (push array_id)
                        // 2. Compile index (push index)
                        // 3. Emit SET_ELEM (pops array_id, index, value from stack)
                        // But the value is already on stack from RHS!
                        // So we need: SWAP to get array_id to top, then SWAP again...
                        // Actually, let's reorder: compile array, index, RHS
                        // But we already compiled RHS...

                        // Simpler: emit SWAP instructions to reorder
                        // Current stack: value (top), need: array, index, value (bottom)
                        // After compiling array and index: value, array, index (top)
                        // We want: array, index, value
                        // So: SWAP (value, array -> array, value), then rotate...
                        // This is getting complex. Let's use a simpler approach:
                        // Just swap value to bottom after compiling array and index

                        // Actually, the stack after RHS is: [value]
                        // After compiling array: [value, array_id]
                        // After compiling index: [value, array_id, index]
                        // SET_ELEM wants: [array_id, index, value]
                        // So we need to rotate: SWAP index<->value -> [value, array_id, index]
                        // No wait, SWAP swaps top two: [value, array_id, index] -> [value, array_id, index] (no change if we swap index and value?)
                        // Let me think again... SWAP swaps top 2 elements
                        // [value, array_id, index] -> SWAP -> [value, array_id, index]?? No...
                        // SWAP swaps top two: index and array_id -> [value, index, array_id]
                        // Then SWAP again: [value, index, array_id] -> [index, value, array_id]
                        // This is confusing. Let's just emit the opcodes in the right order.

                        // Better approach: compile RHS last
                        // But that's a big refactor...

                        // Simplest fix: Add more swap opcodes or just handle it
                        // For now, let's use: we have [value, array, index] and want [array, index, value]
                        // Rotate top 3: [value, array, index] -> [array, index, value]
                        // This needs a ROTATE opcode or we can use SWAP twice:
                        // SWAP: [value, array, index] -> [value, index, array] (swaps index<->array)
                        // No wait, SWAP swaps top TWO elements, not the bottom two

                        // Let me re-read SWAP implementation...
                        // Looking at the code, I think SWAP swaps sp[-1] and sp[-2]
                        // So [value, array, index] with index at sp-1, array at sp-2
                        // SWAP -> [value, index, array]

                        // OK so the approach is:
                        // [value, array, index] -> SWAP -> [value, index, array]
                        // Then SWAP value and index? No, SWAP only swaps top 2...

                        // Let me just use a practical approach:
                        // 1. POP value to temp
                        // 2. Compile array, index
                        // 3. PUSH value back
                        // 4. SET_ELEM
                        // But we don't have a POP_TEMP opcode...

                        // Actually, the simplest is:
                        // Just compile in order: array, index, value (RHS)
                        // But the code already compiled RHS first...

                        // For now, let's use the existing stack:
                        // [value, array, index]
                        // We need [array, index, value]
                        // Solution: Change the codegen to compile array, index, RHS in that order
                        // But that's a bigger change...

                        // Quick fix: Accept the current order and change SET_ELEM to expect [value, array, index]
                        self.emit(OpCode::SET_ELEM); // Expects: value, array_id, index
                        // SET_ELEM doesn't push a return value - mark as void to prevent
                        // Stmt::Expr from emitting a POP that would corrupt the stack
                        self.last_expr_type = ObjectType::Void;
                    } else if let Expr::Dot(obj, field) = lhs.as_ref() {
                        // Plan 075: Field assignment: obj.field = value
                        // Plan 087 Phase 2: Support generic instance field assignment
                        // Stack has: value (from RHS compilation above)

                        // Check if obj is a generic instance variable
                        let is_generic_instance = if let Expr::Ident(var_name) = obj.as_ref() {
                            // Look up variable type
                            if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                                matches!(var_type, Type::GenericInstance(_))
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_generic_instance {
                            // Plan 087 Phase 2: Generic instance field assignment
                            // Compile object expression (pushes instance_id onto stack)
                            self.compile_expr(obj)?;
                            // Now stack has: value, instance_id

                            // Get field index from generic registry
                            if let Expr::Ident(var_name) = obj.as_ref() {
                                if let Some(Type::GenericInstance(ref inst)) =
                                    self.var_types.get(var_name.as_ref())
                                {
                                    // Generate mono_name from base_name and args
                                    let mono_name = self
                                        .generic_registry
                                        .get_template(&inst.base_name.to_string())
                                        .map(|t| t.mono_name_from_args(&inst.args))
                                        .unwrap_or_else(|| format!("{}_unknown", inst.base_name));

                                    // Get ClassType to find field index
                                    if let Some(class_type) =
                                        self.generic_registry.get_type(&mono_name)
                                    {
                                        let field_str = field.to_string();
                                        if let Some(field_index) =
                                            class_type.field_index(&field_str)
                                        {
                                            // Emit SET_GENERIC_FIELD: code layout [opcode, field_index:u32]
                                            self.emit(OpCode::SET_GENERIC_FIELD);
                                            self.emit_u32(field_index as u32);
                                        } else {
                                            eprintln!("Warning: Field '{}' not found in generic type '{}' (assignment)",
                                                field, inst.base_name);
                                            // Fallback: emit placeholder
                                            self.emit(OpCode::SET_GENERIC_FIELD);
                                            self.emit_u32(0);
                                        }
                                    } else {
                                        eprintln!("Warning: Generic type '{}' not found in registry (assignment)", mono_name);
                                        // Fallback to regular field access
                                        let field_str = field.to_string();
                                        let field_bytes = field_str.as_bytes().to_vec();
                                        let field_idx = self.strings.len() as u16;
                                        self.strings.push(field_bytes);
                                        self.emit(OpCode::LOAD_STR);
                                        self.code.extend_from_slice(&field_idx.to_le_bytes());
                                        self.emit(OpCode::SET_FIELD);
                                    }
                                } else {
                                    // Fallback to regular field access
                                    let field_str = field.to_string();
                                    let field_bytes = field_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u16;
                                    self.strings.push(field_bytes);
                                    self.emit(OpCode::LOAD_STR);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                    self.emit(OpCode::SET_FIELD);
                                }
                            } else {
                                // Fallback to regular field access
                                let field_str = field.to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u16;
                                self.strings.push(field_bytes);
                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&field_idx.to_le_bytes());
                                self.emit(OpCode::SET_FIELD);
                            }
                        } else {
                            // Regular field assignment (Plan 075)
                            // Or nested field assignment on user type (Plan 118 Phase 7)
                            // Compile object expression
                            self.compile_expr(obj)?;

                            // Check if the result is a heap object (nested user type)
                            let obj_expr_type = self.infer_expr_type(obj);
                            vm_debug!("DEBUG ASSIGN: obj={:?}, field={}, obj_expr_type={:?}", obj, field, obj_expr_type);
                            let is_user_type = matches!(obj_expr_type, Type::User(_) | Type::GenericInstance(_));
                            let is_heap_object = is_user_type || self.last_expr_type == ObjectType::NestedObject;
                            vm_debug!("DEBUG ASSIGN: is_user_type={}, is_heap_object={}, last_expr_type={:?}",
                                is_user_type, is_heap_object, self.last_expr_type);

                            if is_heap_object || is_user_type {
                                // Get type name from inferred type
                                let type_name = match &obj_expr_type {
                                    Type::User(type_decl) => type_decl.name.to_string(),
                                    Type::GenericInstance(inst) => {
                                        self.generic_registry
                                            .get_template(&inst.base_name.to_string())
                                            .map(|t| t.mono_name_from_args(&inst.args))
                                            .unwrap_or_else(|| format!("{}_unknown", inst.base_name))
                                    }
                                    _ => {
                                        // Fallback: try var_types for Ident
                                        if let Expr::Ident(var_name) = obj.as_ref() {
                                            if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                                                match var_type {
                                                    Type::User(type_decl) => type_decl.name.to_string(),
                                                    Type::GenericInstance(inst) => {
                                                        self.generic_registry
                                                            .get_template(&inst.base_name.to_string())
                                                            .map(|t| t.mono_name_from_args(&inst.args))
                                                            .unwrap_or_else(|| format!("{}_unknown", inst.base_name))
                                                    }
                                                    _ => "Unknown".to_string(),
                                                }
                                            } else {
                                                "Unknown".to_string()
                                            }
                                        } else {
                                            "Unknown".to_string()
                                        }
                                    }
                                };

                                if let Some(class_type) = self.generic_registry.get_type(&type_name) {
                                    let field_str = field.to_string();
                                    if let Some(field_index) = class_type.field_index(&field_str) {
                                        // Stack: [value, instance_id]
                                        // SET_GENERIC_FIELD code layout: [opcode, field_index:u32]
                                        vm_debug!("DEBUG: Emitting SET_GENERIC_FIELD for field '{}' with index {} at code position {}",
                                            field_str, field_index, self.code.len());
                                        self.emit(OpCode::SET_GENERIC_FIELD);
                                        self.emit_u32(field_index as u32);
                                        vm_debug!("DEBUG: After emit, code position = {}", self.code.len());
                                    } else {
                                        eprintln!("Warning: Field '{}' not found in type '{}' (nested assignment)",
                                            field, type_name);
                                        self.emit(OpCode::SET_GENERIC_FIELD);
                                        self.emit_u32(0);
                                    }
                                } else {
                                    // Type not in registry, fall back to SET_FIELD
                                    let field_str = field.to_string();
                                    let field_bytes = field_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u16;
                                    self.strings.push(field_bytes);
                                    self.emit(OpCode::LOAD_STR);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                    self.emit(OpCode::SET_FIELD);
                                }
                            } else {
                                // Now stack has: value, object_id
                                // Load field name
                                let field_str = field.to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u16;
                                self.strings.push(field_bytes);

                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&field_idx.to_le_bytes());
                                // Emit SET_FIELD: expects value, object_id, field_name_idx
                                self.emit(OpCode::SET_FIELD);
                            }
                        }
                    } else {
                        unimplemented!("Assignment to complex LHS not supported yet");
                    }
                } else {
                    // Plan 073 Stage A.5: Check if this is a float/double operation
                    let mut is_float = self.is_float_operation(lhs, rhs);
                    let mut is_double = self.is_double_operation(lhs, rhs);
                    let is_string = self.is_string_operation(lhs, rhs);
                    let is_u64 = self.is_u64_operation(lhs, rhs);

                    // Mixed u64 + float arithmetic: promote to double (f64 can hold all u64 values)
                    if is_u64 && is_float && !is_double {
                        is_double = true;
                        is_float = false;
                    }

                    // Normal binary operation: compile both operands, then apply operator
                    // Plan 117: Emit type coercion for mixed int/float arithmetic
                    self.compile_expr(lhs)?;
                    if is_float && !is_double && self.needs_float_coercion(lhs) {
                        self.emit(OpCode::I32_TO_F32);
                    } else if is_double && self.needs_double_coercion(lhs) {
                        if matches!(lhs.as_ref(), Expr::Float(_, _)) {
                            self.emit(OpCode::PROMOTE_F64);
                        } else if self.is_u64_expr(lhs) {
                            self.emit(OpCode::U64_TO_F64);
                        } else {
                            self.emit(OpCode::I64_TO_F64);
                        }
                    } else if is_u64 && !self.contains_u64(lhs) {
                        // Promote i32 operand to u64 for u64 arithmetic
                        self.emit(OpCode::TYPE_CAST_U64);
                    }

                    self.compile_expr(rhs)?;
                    if is_float && !is_double && self.needs_float_coercion(rhs) {
                        self.emit(OpCode::I32_TO_F32);
                    } else if is_double && self.needs_double_coercion(rhs) {
                        if matches!(rhs.as_ref(), Expr::Float(_, _)) {
                            self.emit(OpCode::PROMOTE_F64);
                        } else if self.is_u64_expr(rhs) {
                            self.emit(OpCode::U64_TO_F64);
                        } else {
                            self.emit(OpCode::I64_TO_F64);
                        }
                    } else if is_u64 && !self.contains_u64(rhs) {
                        // Promote i32 operand to u64 for u64 arithmetic
                        self.emit(OpCode::TYPE_CAST_U64);
                    }

                    // For arithmetic operations, use float/double opcodes if operands are floats
                    match op {
                        Op::Add => {
                            if is_string {
                                self.emit(OpCode::STR_CAT);
                            } else if is_double {
                                self.emit(OpCode::ADD_D);
                            } else if is_float {
                                self.emit(OpCode::ADD_F);
                            } else if is_u64 {
                                self.emit(OpCode::ADD_U64);
                            } else {
                                self.emit(OpCode::ADD);
                            }
                        }
                        Op::Sub => {
                            if is_double {
                                self.emit(OpCode::SUB_D);
                            } else if is_float {
                                self.emit(OpCode::SUB_F);
                            } else if is_u64 {
                                self.emit(OpCode::SUB_U64);
                            } else {
                                self.emit(OpCode::SUB);
                            }
                        }
                        Op::Mul => {
                            if is_double {
                                self.emit(OpCode::MUL_D);
                            } else if is_float {
                                self.emit(OpCode::MUL_F);
                            } else if is_u64 {
                                self.emit(OpCode::MUL_U64);
                            } else {
                                self.emit(OpCode::MUL);
                            }
                        }
                        Op::Div => {
                            if is_double {
                                self.emit(OpCode::DIV_D);
                            } else if is_float {
                                self.emit(OpCode::DIV_F);
                            } else if is_u64 {
                                self.emit(OpCode::DIV_U64);
                            } else {
                                self.emit(OpCode::DIV);
                            }
                        }
                        Op::Mod => {
                            if is_double {
                                self.emit(OpCode::MOD_D);
                            } else if is_float {
                                self.emit(OpCode::MOD_F);
                            } else if is_u64 {
                                self.emit(OpCode::MOD_U64);
                            } else {
                                self.emit(OpCode::MOD);
                            }
                        }
                        Op::Eq => {
                            if is_double { self.emit(OpCode::EQ_D); }
                            else { self.emit(OpCode::EQ); }
                        }
                        Op::Neq => {
                            if is_double { self.emit(OpCode::NE_D); }
                            else { self.emit(OpCode::NE); }
                        }
                        Op::Lt => {
                            if is_double { self.emit(OpCode::LT_D); }
                            else { self.emit(OpCode::LT); }
                        }
                        Op::Le => {
                            if is_double { self.emit(OpCode::LE_D); }
                            else { self.emit(OpCode::LE); }
                        }
                        Op::Gt => {
                            if is_double { self.emit(OpCode::GT_D); }
                            else { self.emit(OpCode::GT); }
                        }
                        Op::Ge => {
                            if is_double { self.emit(OpCode::GE_D); }
                            else { self.emit(OpCode::GE); }
                        }
                        Op::And => self.emit(OpCode::AND),
                        Op::Or => self.emit(OpCode::OR),
                        Op::Not => self.emit(OpCode::NOT),
                        _ => {
                            // Other ops (Bang, DotView, etc.) shouldn't appear in binary expressions
                        }
                    }

                    // Plan 118 Phase 4: Track result type for binary operations
                    // For arithmetic ops, result type matches operand type
                    // For comparison ops, result is always bool (Int)
                    let is_comparison = matches!(op, Op::Eq | Op::Neq | Op::Lt | Op::Le | Op::Gt | Op::Ge);
                    if !is_comparison {
                        // Check operand types to determine result type
                        if is_double {
                            self.last_expr_type = ObjectType::Double;
                        } else if is_float {
                            self.last_expr_type = ObjectType::Float;
                        } else if is_u64 {
                            self.last_expr_type = ObjectType::Uint;
                        } else {
                            // For integer types, check if operands are Uint/Byte/U8/I8
                            // by looking at the expression types
                            let lhs_type = self.infer_object_type(lhs);
                            let rhs_type = self.infer_object_type(rhs);
                            if lhs_type == ObjectType::Uint || rhs_type == ObjectType::Uint {
                                self.last_expr_type = ObjectType::Uint;
                            } else if lhs_type == ObjectType::Byte || rhs_type == ObjectType::Byte {
                                self.last_expr_type = ObjectType::Byte;
                            } else {
                                self.last_expr_type = ObjectType::Int;
                            }
                        }
                    } else {
                        // Comparison results are always bool (Int for now)
                        self.last_expr_type = ObjectType::Int;
                    }
                }
            }
            Expr::Unary(op, rhs) => {
                // Plan 073 Stage A.5: Check if this is a float/double operation
                let is_float = matches!(rhs.as_ref(), Expr::Float(_, _));
                let is_double = matches!(rhs.as_ref(), Expr::Double(_, _));

                // Compile the operand first
                self.compile_expr(rhs)?;

                // Emit the appropriate unary opcode
                match op {
                    Op::Sub => {
                        if is_double {
                            self.emit(OpCode::NEG_D);
                        } else if is_float {
                            self.emit(OpCode::NEG_F);
                        } else {
                            self.emit(OpCode::NEG);
                        }
                    }
                    Op::Not => self.emit(OpCode::NOT),
                    _ => unimplemented!("Unary Op {:?}", op),
                }
            }
            Expr::Call(call) => {
                // Plan 087 Phase 2: Check if this is a generic constructor call (e.g., Pair.new(1, "a"))
                // IMPORTANT: Skip inline construction if the type has a user-defined new() method
                let is_generic_constructor = if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if method == "new" {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            // Check if a user-defined TypeName.new method exists
                            let mangled = format!("{}.new", type_name.as_ref());
                            if self.exports.contains_key(&mangled) {
                                false // User defined their own new() — use regular CALL
                            } else {
                                self.generic_registry.has_template(type_name.as_ref())
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_generic_constructor {
                    // Plan 087 Phase 2: Generic constructor call
                    // Generate: NEW_INSTANCE + CONSTRUCT_INSTANCE

                    if let Expr::Dot(obj, _method) = call.name.as_ref() {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            // Get or create ClassType to determine mono_name and field count
                            let type_args = Vec::new(); // For Phase 2, use empty type args (no inference)
                            if let Ok(class_type) = self
                                .generic_registry
                                .get_or_create_type(type_name.as_ref(), type_args)
                            {
                                let field_count = class_type.template.fields.len();

                                // Compile arguments (push values onto stack)
                                // Stack: ..., arg1, arg2, ..., argN
                                if !call.args.args.is_empty() {
                                    for arg in &call.args.args {
                                        match arg {
                                            crate::ast::Arg::Pos(expr) => {
                                                self.compile_expr(expr)?;
                                            }
                                            crate::ast::Arg::Pair(_key, expr) => {
                                                // Named argument: compile value only
                                                self.compile_expr(expr)?;
                                            }
                                            crate::ast::Arg::Name(name) => {
                                                // Named argument without value - treat as string
                                                self.emit(OpCode::LOAD_STR);
                                                let s_bytes = name.to_string().as_bytes().to_vec();
                                                let s_idx = self.strings.len() as u16;
                                                self.strings.push(s_bytes);
                                                self.code.extend_from_slice(&s_idx.to_le_bytes());
                                            }
                                        }
                                    }
                                }

                                // Emit NEW_INSTANCE instruction
                                // Push mono_name length
                                let mono_name = class_type.mono_name.clone();
                                let name_bytes = mono_name.as_bytes();
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(name_bytes.len() as i32);

                                // Emit NEW_INSTANCE opcode first
                                self.emit(OpCode::NEW_INSTANCE);

                                // Then emit mono_name bytes directly into code (after opcode)
                                for &byte in name_bytes {
                                    self.code.push(byte);
                                }

                                // Emit CONSTRUCT_INSTANCE
                                // Stack layout should be: ..., instance_id, field_count, arg1, ..., argN
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(field_count as i32);
                                self.emit(OpCode::CONSTRUCT_INSTANCE);

                                return Ok(());
                            } else {
                                eprintln!(
                                    "Warning: Failed to get/create generic type '{}'",
                                    type_name
                                );
                                // Fallback to regular call
                            }
                        }
                    }

                    // Fallback to regular call if something went wrong
                }

                // Plan 118 Phase 2: Check if this is a type constructor call (e.g., Inner(x: 10))
                // If the call name is a registered type, treat it as a type instance creation
                if let Expr::Ident(type_name) = call.name.as_ref() {
                    let type_name_str = type_name.to_string();

                    // Check if this type is registered
                    if self.generic_registry.has_template(&type_name_str) || self.get_type(&type_name_str).is_some() {
                        // This is a type constructor call - compile as type instance
                        vm_debug!("DEBUG: Compiling type constructor call for '{}'", type_name_str);

                        // Get type info
                        let member_names = if self.generic_registry.has_template(&type_name_str) {
                            let type_args = Vec::new();
                            if let Ok(class_type) = self.generic_registry.get_or_create_type(&type_name_str, type_args) {
                                class_type.template.fields.iter().map(|f| f.name.clone()).collect()
                            } else {
                                Vec::new()
                            }
                        } else if let Some(type_info) = self.get_type(&type_name_str) {
                            type_info.member_names.clone()
                        } else {
                            Vec::new()
                        };

                        // Compile arguments (push values onto stack)
                        let arg_count = call.args.args.len() as u8;
                        for arg in &call.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Pair(_key, expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Name(_) => {
                                    // Name-only arg - placeholder
                                }
                            }
                        }

                        // Create object keys using type member names
                        let keys: Vec<auto_val::ValueKey> = member_names
                            .iter()
                            .take(arg_count as usize)
                            .map(|name| auto_val::ValueKey::Str(name.clone().into()))
                            .collect();

                        let _key_index = self.object_keys.len() as u16;
                        self.object_keys.push(keys);

                        // Infer field types from args
                        let types: Vec<ObjectType> = call.args.args
                            .iter()
                            .take(arg_count as usize)
                            .map(|arg| match arg {
                                crate::ast::Arg::Pos(expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Pair(_, expr) => self.infer_object_type(expr),
                                crate::ast::Arg::Name(_) => ObjectType::Int,
                            })
                            .collect();
                        self.object_types.push(types);

                        // Plan 118 Phase 7: Use NEW_INSTANCE + CONSTRUCT_INSTANCE for user types
                        // This ensures objects are stored in heap_objects (4000000+) instead of objects (1000000+)
                        let field_count = arg_count.min(member_names.len() as u8);

                        // NEW_INSTANCE expects:
                        // - Stack: mono_name_len (i32)
                        // - Flash: mono_name_bytes
                        // After execution: instance_id pushed to stack

                        let mono_name_bytes = type_name_str.as_bytes().to_vec();

                        // Push mono_name length to stack
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(mono_name_bytes.len() as i32);

                        // Emit NEW_INSTANCE opcode
                        self.emit(OpCode::NEW_INSTANCE);

                        // Emit mono_name bytes directly to flash (code stream)
                        for byte in &mono_name_bytes {
                            self.code.push(*byte);
                        }

                        // Stack now has: [..., field_value1, ..., field_valueN, instance_id]
                        // CONSTRUCT_INSTANCE expects:
                        // - Stack: field_count, instance_id, field_value1, ..., field_valueN
                        // So we need to: push field_count, then CONSTRUCT_INSTANCE
                        self.emit(OpCode::CONST_I32);
                        self.emit_i32(field_count as i32);
                        self.emit(OpCode::CONSTRUCT_INSTANCE);

                        // Track variable type for this instance
                        self.last_expr_type = ObjectType::NestedObject;

                        return Ok(());
                    }
                }

                // Regular function/method call (existing code)
                // Extract function name and determine if it's a method call
                // Plan 073: Support both static methods (Type.method) and instance methods (obj.method)
                let func_name = match call.name.as_ref() {
                    Expr::Ident(name) => Some(name.to_string()),
                    Expr::Dot(obj, method) => {
                        // Method call: Type.method (static) or obj.method (instance)
                        // Plan 087 Phase 3: Support generic instance method calls
                        match obj.as_ref() {
                            Expr::Ident(obj_name) => {
                                // Check if it's a static method call (Type.method with capital T)
                                if self.is_type_name_heuristic(obj_name) || self.is_type(obj_name) {
                                    // Plan 127: Special handling for TaskType.spawn() and TaskType.send()
                                    // These should use the generic Task.spawn/Task.send native functions
                                    if method.as_str() == "spawn" && self.types.contains_key(obj_name.as_ref()) {
                                        // Check if this is a task type
                                        let type_info = self.types.get(obj_name.as_ref());
                                        if let Some(info) = type_info {
                                            if info._name.contains("#single") || info._name == obj_name.as_ref() {
                                                // This is a task type - use Task.spawn
                                                vm_debug!("DEBUG: Task spawn detected: {}.spawn() -> Task.spawn", obj_name);
                                                Some("Task.spawn".to_string())
                                            } else {
                                                Some(format!("{}.{}", obj_name, method))
                                            }
                                        } else {
                                            Some(format!("{}.{}", obj_name, method))
                                        }
                                    } else if method.as_str() == "send" && self.types.contains_key(obj_name.as_ref()) {
                                        // Singleton task send: TaskType.send(msg)
                                        let type_info = self.types.get(obj_name.as_ref());
                                        if let Some(info) = type_info {
                                            if info._name.contains("#single") {
                                                // This is a singleton task - use Task.send
                                                vm_debug!("DEBUG: Singleton task send detected: {}.send() -> Task.send", obj_name);
                                                Some("Task.send".to_string())
                                            } else {
                                                Some(format!("{}.{}", obj_name, method))
                                            }
                                        } else {
                                            Some(format!("{}.{}", obj_name, method))
                                        }
                                    } else {
                                        // Static method call: Type.method
                                        Some(format!("{}.{}", obj_name, method))
                                    }
                                } else {
                                    // Instance method call: obj.method
                                    // Plan 087 Phase 3: Check if obj is a generic instance
                                    let func_name = if let Some(ty) =
                                        self.var_types.get(obj_name.as_ref())
                                    {
                                        if let Type::GenericInstance(inst) = ty {
                                            // Generate monomorphic method name for generic instance
                                            // Example: p.get_key() where p: Pair<int, string>
                                            //          → "Pair_int_str.get_key"
                                            let mono_name = self
                                                .generic_registry
                                                .get_template(&inst.base_name.to_string())
                                                .map(|t| t.mono_name_from_args(&inst.args))
                                                .unwrap_or_else(|| {
                                                    format!("{}_unknown", inst.base_name)
                                                });

                                            Some(format!("{}.{}", mono_name, method))
                                        } else {
                                            // Not a generic instance, use regular inference
                                            vm_debug!("DEBUG: Instance method call: obj={}, method={}, var_types={:?}", obj_name, method, self.var_types);
                                            if let Some(type_name) =
                                                self.infer_type_from_var(obj_name.as_ref())
                                            {
                                                vm_debug!("DEBUG: Inferred type name: {}",
                                                    type_name
                                                );
                                                Some(format!("{}.{}", type_name, method))
                                            } else {
                                                // Plan 127: Handle TaskHandle.send() when type is Unknown
                                                // If the method is "send", assume it's a task handle
                                                if method.as_str() == "send" {
                                                    vm_debug!("DEBUG: Assuming TaskHandle.send for unknown type variable {}", obj_name);
                                                    Some("TaskHandle.send".to_string())
                                                } else {
                                                    vm_debug!("DEBUG: Failed to infer type for {}",
                                                        obj_name
                                                    );
                                                    Some(format!("{}.{}", obj_name, method))
                                                }
                                            }
                                        }
                                    } else {
                                        // No type info, use regular inference
                                        vm_debug!("DEBUG: No type info for obj={}, var_types empty",
                                            obj_name
                                        );
                                        if let Some(type_name) =
                                            self.infer_type_from_var(obj_name.as_ref())
                                        {
                                            vm_debug!("DEBUG: Inferred type name: {}", type_name);
                                            Some(format!("{}.{}", type_name, method))
                                        } else {
                                            // Plan 127: Handle TaskHandle.send() when type is unknown
                                            // If the variable name suggests it's a handle (e.g., "handle")
                                            // and the method is "send", use TaskHandle.send
                                            if method.as_str() == "send" {
                                                vm_debug!("DEBUG: Assuming TaskHandle.send for unknown type variable {}", obj_name);
                                                Some("TaskHandle.send".to_string())
                                            } else {
                                                vm_debug!("DEBUG: Failed to infer type for {}",
                                                    obj_name
                                                );
                                                Some(format!("{}.{}", obj_name, method))
                                            }
                                        }
                                    };

                                    func_name
                                }
                            }
                            _ => {
                                // Complex expression (e.g., arr[0].push, foo().method)
                                // Or literal expressions (e.g., 1.str(), "hello".upper())
                                // Plan 118 Phase 4: Handle literal method calls
                                let inferred_type = self.infer_object_type(obj.as_ref());
                                let type_name: String = match inferred_type {
                                    ObjectType::Int | ObjectType::Byte => "int".to_string(),
                                    ObjectType::Uint => "uint".to_string(),
                                    ObjectType::Float | ObjectType::Double => "float".to_string(),
                                    ObjectType::String => "str".to_string(),
                                    ObjectType::Char => "char".to_string(),
                                    ObjectType::Bool => "bool".to_string(),
                                    _ => {
                                        // Plan 197 Task 3: Try to resolve user-defined type name
                                        // from fn_return_types for chained method calls
                                        self.infer_user_type_name(obj.as_ref())
                                            .unwrap_or_else(|| "Unknown".to_string())
                                    }
                                };
                                let native_name = format!("{}.{}", type_name, method);

                                // Check if this native exists
                                if BIGVM_NATIVES.lock().unwrap().get_id(&native_name).is_some() {
                                    Some(native_name)
                                } else if self.exports.contains_key(&format!("{}.{}", type_name, method.as_ref())) {
                                    // Plan 197 Task 3: User-defined method on chained result
                                    Some(format!("{}.{}", type_name, method.as_ref()))
                                } else if method.as_str() == "len" {
                                    // Fallback for len() - most common
                                    Some("str.len".to_string())
                                } else {
                                    Some(format!("Unknown_{}", method))
                                }
                            }
                        }
                    }
                    _ => None,
                };

                // Check if it's a native function (either intrinsic or BIGVM_NATIVE)
                let native_id = if let Some(name) = &func_name {
                    // Check intrinsics first (print, etc.)
                    if let Some(&id) = self.intrinsics.get(name) {
                        Some(id)
                    }
                    // Then check BIGVM_NATIVES (List methods, etc.)
                    else if let Some(id) = BIGVM_NATIVES.lock().unwrap().get_id(name) {
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Plan 118 Phase 7: Check if calling a closure variable
                // If call.name is Ident and the variable has Fn type, use CALL_CLOSURE
                let is_closure_call = if let Expr::Ident(name) = call.name.as_ref() {
                    let name_str = name.to_string();
                    if let Some(var_type) = self.var_types.get(&name_str) {
                        matches!(var_type, Type::Fn(_, _))
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_closure_call {
                    // Closure variable call: load closure_id, then use CALL_CLOSURE
                    // Stack layout for CALL_CLOSURE: [..., arg1, arg2, closure_id]
                    // (closure_id should be on top of stack when CALL_CLOSURE executes)
                    if let Expr::Ident(name) = call.name.as_ref() {
                        let name_str = name.to_string();
                        vm_debug!("DEBUG: Closure variable call: {}", name_str);

                        // Compile arguments FIRST (push them to stack)
                        for arg in &call.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Pair(_, expr) => {
                                    self.compile_expr(expr)?;
                                }
                                crate::ast::Arg::Name(name) => {
                                    self.compile_expr(&Expr::Ident(name.clone()))?;
                                }
                            }
                        }

                        // Load the closure_id from the variable LAST (so it's on top)
                        if let Some(var_index) = self.lookup_var(&name_str) {
                            self.emit_load_loc(var_index);
                        } else {
                            return Err(AutoError::Msg(format!(
                                "Undefined closure variable: {}",
                                name_str
                            )));
                        }

                        // Emit CALL_CLOSURE with arg_count
                        self.emit(OpCode::CALL_CLOSURE);
                        self.code.push(call.args.args.len() as u8);

                        // Skip the rest of the function call logic
                        return Ok(());
                    }
                }

                if let Some(id) = native_id {
                    // Native function call
                    // For instance methods, compile receiver (self) FIRST, then arguments
                    // This ensures stack order: [self, arg1, arg2, ...]
                    vm_debug!("DEBUG: Native function call: func_name={:?}, native_id={}",
                        func_name, id
                    );
                    if let Expr::Dot(obj, _method) = call.name.as_ref() {
                        // Check if it's a static method call (Type.method with capital T)
                        let is_static_method = match obj.as_ref() {
                            Expr::Ident(obj_name) => {
                                self.is_type_name_heuristic(obj_name) || self.is_type(obj_name)
                            }
                            _ => false,
                        };
                        vm_debug!("DEBUG: is_static_method={}", is_static_method);

                        // Compile receiver for instance methods
                        if !is_static_method {
                            vm_debug!("DEBUG: Compiling receiver for instance method");
                            // Check if this method needs 'id' field extraction
                            if let Some(ref method_name) = func_name {
                                vm_debug!("DEBUG: method_name={}, needs_id_extraction={}",
                                    method_name,
                                    self.needs_id_extraction(method_name)
                                );
                                if self.needs_id_extraction(method_name) {
                                    // Compile object expression
                                    self.compile_expr(obj)?;

                                    // Extract 'id' field using GET_FIELD
                                    let field_str = "id".to_string();
                                    let field_bytes = field_str.as_bytes().to_vec();
                                    let field_idx = self.strings.len() as u16;
                                    self.strings.push(field_bytes);

                                    self.emit(OpCode::GET_FIELD);
                                    self.code.extend_from_slice(&field_idx.to_le_bytes());
                                } else {
                                    // Compile full instance (for user-defined types)
                                    vm_debug!("DEBUG: Compiling object expr (no id extraction)");
                                    self.compile_expr(obj)?;
                                }
                            } else {
                                self.compile_expr(obj)?;
                            }
                        }
                    }

                    // Plan 127: Special handling for Task.spawn - inject task_type and capacity
                    // The rust_fn macro pops args in REVERSE order:
                    // shim_task_spawn(task_type: String, capacity: i32)
                    // Pop order: capacity first (from top), then task_type (from next)
                    // Stack layout needed: [task_type, capacity] where capacity is on top
                    // Push order: task_type first (bottom), capacity second (top)
                    if func_name.as_deref() == Some("Task.spawn") {
                        // Get the task type name from the original Dot expression
                        if let Expr::Dot(obj, _) = call.name.as_ref() {
                            if let Expr::Ident(task_type) = obj.as_ref() {
                                // Push task_type as string FIRST (goes to bottom of stack)
                                let task_type_str = task_type.to_string();
                                let task_type_bytes = task_type_str.as_bytes().to_vec();
                                let str_idx = self.strings.len() as u16;
                                self.strings.push(task_type_bytes);
                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&str_idx.to_le_bytes());

                                // Push default capacity (64) as i32 SECOND (goes to top of stack)
                                self.emit(OpCode::CONST_I32);
                                self.emit_i32(64);

                                vm_debug!("DEBUG: Injected task_type='{}' capacity=64 for Task.spawn", task_type_str);
                            }
                        }
                    }

                    // Plan 127: Special handling for Task.send - inject task_type for singleton tasks
                    if func_name.as_deref() == Some("Task.send") {
                        // Get the task type name from the original Dot expression
                        if let Expr::Dot(obj, _) = call.name.as_ref() {
                            if let Expr::Ident(task_type) = obj.as_ref() {
                                // Push task_type as string (first arg)
                                let task_type_str = task_type.to_string();
                                let task_type_bytes = task_type_str.as_bytes().to_vec();
                                let str_idx = self.strings.len() as u16;
                                self.strings.push(task_type_bytes);
                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&str_idx.to_le_bytes());
                                vm_debug!("DEBUG: Injected task_type='{}' for Task.send", task_type_str);
                            }
                        }
                    }

                    // Compile arguments (left-to-right)
                    // Plan 088 Phase 4: Smart parameter passing for native functions
                    if !call.args.is_empty() {
                        let func_name_for_params =
                            func_name.as_ref().map(|s| s.as_str()).unwrap_or("");
                        let is_assert_msg = func_name_for_params == "assert_eq"
                            || func_name_for_params == "assert_ne";
                        let max_args = if is_assert_msg && call.args.args.len() > 2 {
                            2 // Skip the 3rd arg (message string) for assert_eq/ne
                        } else {
                            call.args.args.len()
                        };
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i >= max_args {
                                break; // Skip extra args (e.g., assert_eq message)
                            }
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    // Use smart parameter passing if we have function info
                                    if !func_name_for_params.is_empty()
                                        && self.fn_params.contains_key(func_name_for_params)
                                    {
                                        self.compile_call_arg(expr, func_name_for_params, i)?;
                                    } else {
                                        // No parameter info, compile normally
                                        self.compile_expr(expr)?;
                                    }
                                }
                                _ => {
                                    unimplemented!("Named arguments not supported in AutoVM yet")
                                }
                            }
                        }
                    }

                    // Plan 192: Inject implicit type_name and method for Rust stdlib dispatch
                    // Push AFTER user args so type_name/method are on top of stack.
                    // Handler pops method first (top), then type_name (next).
                    if id == NATIVE_RUST_STDLIB_DISPATCH {
                        if let Expr::Dot(obj, method_name) = call.name.as_ref() {
                            if let Expr::Ident(type_name_ident) = obj.as_ref() {
                                let type_str = type_name_ident.to_string();
                                let type_bytes = type_str.as_bytes().to_vec();
                                let type_idx = self.strings.len() as u16;
                                self.strings.push(type_bytes);
                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&type_idx.to_le_bytes());

                                let method_str = method_name.to_string();
                                let method_bytes = method_str.as_bytes().to_vec();
                                let method_idx = self.strings.len() as u16;
                                self.strings.push(method_bytes);
                                self.emit(OpCode::LOAD_STR);
                                self.code.extend_from_slice(&method_idx.to_le_bytes());
                            }
                        }
                    }

                    // Plan 178: Select correct print intrinsic based on argument type
                    // print() defaults to NATIVE_PRINT_STR, but if the argument is
                    // a numeric expression, use NATIVE_PRINT_I32 or NATIVE_PRINT_F32 instead.
                    // This fixes negative integer printing (e.g., print(-1) would otherwise
                    // be misinterpreted as a tagged string index).
                    let resolved_id = if id == NATIVE_PRINT_STR {
                        match self.last_expr_type {
                            ObjectType::Int | ObjectType::Byte | ObjectType::Uint
                            | ObjectType::Bool | ObjectType::Char => NATIVE_PRINT_I32,
                            ObjectType::Float | ObjectType::Double => NATIVE_PRINT_F32,
                            _ => id, // keep PRINT_STR for String, Void, etc.
                        }
                    } else {
                        id
                    };

                    self.emit(OpCode::CALL_NAT);
                    self.code.extend_from_slice(&resolved_id.to_le_bytes());

                    // Track return type for type-aware dispatch (e.g., print choosing STR vs I32)
                    if let Some(ref name) = func_name {
                        if name.starts_with("print") || name == "say" || name.starts_with("assert") {
                            self.last_expr_type = ObjectType::Void;
                        } else if name.ends_with(".to_hex") || name.ends_with(".to_str")
                            || name.ends_with(".str") || name == "int_str" {
                            self.last_expr_type = ObjectType::String;
                        } else if let Some(ret_ty) = self.fn_return_types.get(name) {
                            self.last_expr_type = match ret_ty {
                                Type::Void => ObjectType::Void,
                                Type::Float => ObjectType::Float,
                                Type::Double => ObjectType::Double,
                                Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => ObjectType::String,
                                Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                                Type::Byte => ObjectType::Byte,
                                Type::Bool => ObjectType::Bool,
                                Type::Int | Type::I64 => ObjectType::Int,
                                _ => ObjectType::NestedObject,
                            };
                        }
                    }

                    return Ok(()).into();
                }

                // Normal Function Call (user-defined)
                // Plan 087 Phase 3 + Plan 088 Phase 4: Instance method receiver as first argument
                let mut is_instance_method_call = false;

                if let Expr::Dot(obj, _method) = call.name.as_ref() {
                    // Check if it's a static method call (Type.method with capital T)
                    let is_static_method = match obj.as_ref() {
                        Expr::Ident(obj_name) => {
                            self.is_type_name_heuristic(obj_name) || self.is_type(obj_name)
                        }
                        _ => false,
                    };

                    // For instance methods, treat receiver as first argument
                    if !is_static_method {
                        is_instance_method_call = true;

                        // Plan 088 Phase 4: Compile receiver as first argument (index 0)
                        if let Some(ref method_name) = func_name {
                            vm_debug!("DEBUG: Compiling instance method call: receiver is arg 0 for '{}'",
                                method_name
                            );

                            // Check if this method needs 'id' field extraction
                            if self.needs_id_extraction(method_name) {
                                // Compile object expression
                                self.compile_expr(obj)?;

                                // Extract 'id' field using GET_FIELD
                                let field_str = "id".to_string();
                                let field_bytes = field_str.as_bytes().to_vec();
                                let field_idx = self.strings.len() as u16;
                                self.strings.push(field_bytes);

                                self.emit(OpCode::GET_FIELD);
                                self.code.extend_from_slice(&field_idx.to_le_bytes());
                            } else {
                                // Plan 088 Phase 4: Smart parameter passing for receiver
                                // Use compile_call_arg to support Copy/View/Mut/Take modes

                                // Receiver (arg 0) is always an object ID (i32 value),
                                // so always use direct compile_expr instead of smart param passing
                                // (LOAD_REF would push var_index instead of the actual object ID)
                                self.compile_expr(obj)?;
                            }
                        }
                    } else {
                        // Static method - no receiver
                        is_instance_method_call = false;
                    }
                }

                // Compile Arguments (pushes them to stack)
                // Plan 088 Phase 4: Smart parameter passing based on type and mode
                let call_display = format!("{:?}", call.name);
                vm_debug!("DEBUG: ===== Compiling call: {} =====", call_display);
                vm_debug!("DEBUG: Before compiling args, code.len()={:04x} ({})",
                    self.code.len(),
                    self.code.len()
                );

                // For instance methods, receiver is arg 0, so other args start from index 1
                let arg_offset = if is_instance_method_call { 1 } else { 0 };

                if !call.args.is_empty() {
                    let func_name_for_params = func_name.as_ref().map(|s| s.as_str()).unwrap_or("");
                    for (i, arg) in call.args.args.iter().enumerate() {
                        match arg {
                            crate::ast::Arg::Pos(expr) => {
                                let param_index = i + arg_offset;
                                if !func_name_for_params.is_empty()
                                    && self.fn_params.contains_key(func_name_for_params)
                                {
                                    vm_debug!("DEBUG:   Arg {}: smart param passing for '{}'",
                                        param_index, func_name_for_params
                                    );
                                    self.compile_call_arg(expr, func_name_for_params, param_index)?;
                                } else {
                                    vm_debug!("DEBUG:   Arg {}: normal compile", param_index);
                                    self.compile_expr(expr)?;
                                }
                            }
                            crate::ast::Arg::Pair(_, expr) => {
                                // Named argument (e.g., add(a: 12, b: 2))
                                // Extract the value expression and compile it like a positional arg
                                let param_index = i + arg_offset;
                                if !func_name_for_params.is_empty()
                                    && self.fn_params.contains_key(func_name_for_params)
                                {
                                    vm_debug!("DEBUG:   Named arg {}: smart param passing for '{}'",
                                        param_index, func_name_for_params
                                    );
                                    self.compile_call_arg(expr, func_name_for_params, param_index)?;
                                } else {
                                    vm_debug!("DEBUG:   Named arg {}: normal compile", param_index);
                                    self.compile_expr(expr)?;
                                }
                            }
                            crate::ast::Arg::Name(name) => {
                                // Name-only argument (e.g., add(a) where a is both name and value)
                                // Convert to expression by wrapping in Ident
                                let param_index = i + arg_offset;
                                if !func_name_for_params.is_empty()
                                    && self.fn_params.contains_key(func_name_for_params)
                                {
                                    vm_debug!("DEBUG:   Named arg {}: smart param passing for '{}'",
                                        param_index, func_name_for_params
                                    );
                                    self.compile_call_arg(
                                        &Expr::Ident(name.clone()),
                                        func_name_for_params,
                                        param_index,
                                    )?;
                                } else {
                                    vm_debug!("DEBUG:   Named arg {}: normal compile", param_index);
                                    self.compile_expr(&Expr::Ident(name.clone()))?;
                                }
                            }
                        }
                    }
                }
                // 2. Emit CALL opcode
                self.emit(OpCode::CALL);

                // 3. Emit Placeholder for Address (u32)
                let placeholder_idx = self.code.len();
                self.code.extend_from_slice(&0u32.to_le_bytes());

                // 4. Create Relocation Entry
                let reloc_name = func_name.unwrap_or_else(|| match call.name.as_ref() {
                    Expr::Ident(name) => name.to_string(),
                    _ => unimplemented!("Dynamic call (computed function name) not supported yet"),
                });

                vm_debug!("DEBUG: Creating reloc for function '{}' at offset 0x{:04x}",
                    reloc_name, placeholder_idx
                );
                vm_debug!("DEBUG: Available exports: {:?}",
                    self.exports.keys().collect::<Vec<_>>()
                );

                self.relocs.push(RelocEntry {
                    offset: placeholder_idx as u32,
                    symbol_name: reloc_name.clone(),
                    reloc_type: RelocType::FuncCall,
                    source_pos: call.pos,
                });

                // Plan 118 Phase 4: Function return type inference
                // After function body compilation, fn_return_types is updated with actual return type:
                // - Type::Void: truly void (body ends with void call or no expression)
                // - Type::Unknown: has implicit return value (body ends with non-void expression)
                // - Other: explicit return type
                if let Some(ret_ty) = self.fn_return_types.get(&reloc_name) {
                    self.last_expr_type = match ret_ty {
                        Type::Void => ObjectType::Void,
                        Type::Float => ObjectType::Float,
                        Type::Double => ObjectType::Double,
                        Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => ObjectType::String,
                        Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                        Type::Byte => ObjectType::Byte,
                        Type::Bool => ObjectType::Bool,
                        Type::Int | Type::I64 => ObjectType::Int,
                        _ => ObjectType::NestedObject,
                    };
                }
            }
            Expr::If(if_expr) => {
                // If expression: each branch must leave a value on the stack
                let mut jumps_to_end = Vec::new();
                let mut body_is_two_slot = false;

                for branch in &if_expr.branches {
                    // Compile condition
                    self.compile_expr(&branch.cond)?;

                    // JMP_IF_Z to next branch
                    self.emit(OpCode::JMP_IF_Z);
                    let jump_to_next = self.emit_placeholder_i16();

                    // Compile body (should push result)
                    // Body is a Block, compile all statements
                    // Plan 118 Phase 5: Use compile_stmt on Block to handle should_pop_expr_result correctly
                    // The last expression in the block should be left on stack
                    let body_block = Stmt::Block(branch.body.clone());
                    self.compile_stmt(&body_block)?;

                    // Check if body left a 2-slot result by inspecting the last statement
                    // (reassignment of u64/f64/double vars leaves 2 slots from reload)
                    if let Some(last_stmt) = branch.body.stmts.last() {
                        if let Stmt::Expr(expr) = last_stmt {
                            if let Expr::Bina(lhs, op, _) = expr {
                                if *op == Op::Asn {
                                    if let Expr::Ident(name) = lhs.as_ref() {
                                        if let Some(ty) = self.var_types.get(name.as_str()) {
                                            if matches!(ty, Type::U64 | Type::I64 | Type::Double) {
                                                body_is_two_slot = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Jump to end
                    self.emit(OpCode::JMP);
                    let jump_to_end = self.emit_placeholder_i16();
                    jumps_to_end.push(jump_to_end);

                    // Patch jump to next branch
                    self.patch_jump(jump_to_next);
                }

                // Else branch (if any)
                if let Some(else_body) = &if_expr.else_ {
                    // Plan 118 Phase 5: Use compile_stmt on Block to handle should_pop_expr_result correctly
                    let else_block = Stmt::Block(else_body.clone());
                    self.compile_stmt(&else_block)?;
                } else {
                    // No else branch - push nil marker(s) to match body stack height
                    if body_is_two_slot {
                        // Body leaves 2 slots, push 2 nil markers
                        self.emit(OpCode::CONST_I32);
                        self.code.extend_from_slice(&(i32::MIN + 1).to_le_bytes());
                        self.emit(OpCode::CONST_I32);
                        self.code.extend_from_slice(&(i32::MIN + 1).to_le_bytes());
                    } else {
                        self.emit(OpCode::CONST_I32);
                        self.code.extend_from_slice(&(i32::MIN + 1).to_le_bytes());
                    }
                }

                // Patch all jumps to end
                for jump in jumps_to_end {
                    self.patch_jump(jump);
                }

                // Plan 118 Phase 7: If expression produces a value
                self.last_expr_type = ObjectType::Int; // default
                if body_is_two_slot {
                    self.last_expr_type = ObjectType::Double; // 2-slot result
                }
            }
            Expr::Closure(closure) => {
                // Plan 071: Compile closure with captured environment
                self.compile_closure(closure)?;
            }
            Expr::View(inner) | Expr::Mut(inner) | Expr::Move(inner) | Expr::Take(inner) => {
                // Plan 060/122: Ownership operators (.view, .mut, .move, .take)
                // For MVP, just compile the inner expression
                // TODO: In future, implement proper borrow checking and ownership semantics
                self.compile_expr(inner)?;
            }
            // Plan 073: May<T> null coalesce operator: left ?? right
            Expr::NullCoalesce(left, right) => {
                // Compile left expression (pushes May<T> value onto stack)
                self.compile_expr(left)?;
                // Compile right expression (pushes default value onto stack)
                self.compile_expr(right)?;
                // Emit NULL_COALESCE (pops May<T> and default, pushes unwrapped value or default)
                self.emit(OpCode::NULL_COALESCE);
            }
            // Plan 073: May<T> error propagate operator: expression.?
            Expr::ErrorPropagate(expr) => {
                // Compile expression (pushes May<T> value onto stack)
                self.compile_expr(expr)?;
                // Emit ERROR_PROPAGATE (pops May<T>, pushes unwrapped value or early returns)
                self.emit(OpCode::ERROR_PROPAGATE);
            }
            // Plan 162: Type cast: expr.as(Type) — runtime type conversion
            Expr::Cast { expr, target_type } => {
                // Compile inner expression
                self.compile_expr(expr)?;
                // Emit appropriate cast opcode based on target type
                match target_type {
                    Type::Int => self.emit(OpCode::TYPE_CAST_I32),
                    Type::Uint => self.emit(OpCode::TYPE_CAST_U32),
                    Type::I64 => self.emit(OpCode::TYPE_CAST_I64),
                    Type::U64 => self.emit(OpCode::TYPE_CAST_U64),
                    Type::Float => self.emit(OpCode::TYPE_CAST_F64),
                    Type::Double => {
                        // i32 -> f32 -> f64 (2 slots)
                        self.emit(OpCode::TYPE_CAST_F64);
                        self.emit(OpCode::PROMOTE_F64);
                    }
                    Type::Ptr(_) => self.emit(OpCode::TYPE_CAST_PTR),
                    _ => {
                        // For unknown/unsupported types, just leave the value as-is
                        return Ok(());
                    }
                };
                self.last_expr_type = match target_type {
                    Type::Int | Type::I64 | Type::Ptr(_) => ObjectType::Int,
                    Type::Uint | Type::USize | Type::U64 => ObjectType::Uint,
                    Type::Float => ObjectType::Float,
                    Type::Double => ObjectType::Double,
                    Type::Byte => ObjectType::Byte,
                    Type::Bool => ObjectType::Bool,
                    _ => ObjectType::Int,
                };
            }
            // Plan 162/193: Explicit type conversion: expr.to(Type)
            Expr::To { expr, target_type } => {
                // Compile inner expression
                self.compile_expr(expr)?;
                let src_type = self.last_expr_type;
                // Determine precise source Type for opcode selection
                let src_precise_type = self.infer_expr_type_for_conv(expr.as_ref(), src_type);
                // Emit appropriate conversion opcode based on source + target type
                let opcode = match target_type {
                    Type::Str(_) | Type::String => {
                        match src_precise_type {
                            ConvSrcType::F32 => OpCode::TYPE_F32_TO_STR,
                            ConvSrcType::F64 => OpCode::TYPE_F64_TO_STR,
                            ConvSrcType::I64 => OpCode::TYPE_I64_TO_STR,
                            ConvSrcType::U64 => OpCode::TYPE_U64_TO_STR,
                            ConvSrcType::Bool => OpCode::TYPE_BOOL_TO_STR,
                            ConvSrcType::Str | ConvSrcType::I32 | ConvSrcType::Other => OpCode::TYPE_TO_STR,
                        }
                    }
                    Type::Int => {
                        match src_precise_type {
                            ConvSrcType::F32 => OpCode::TYPE_F32_TO_I32,
                            ConvSrcType::F64 => OpCode::TYPE_F64_TO_I32,
                            ConvSrcType::Str => OpCode::TYPE_TO_I32,
                            _ => OpCode::TYPE_TO_I32,
                        }
                    }
                    Type::I64 => {
                        match src_precise_type {
                            ConvSrcType::Str => OpCode::TYPE_STR_TO_I64,
                            _ => OpCode::TYPE_CAST_I64,
                        }
                    }
                    Type::Float | Type::Double => OpCode::TYPE_TO_F64,
                    // For numeric-to-numeric, reuse cast opcodes (no allocation needed)
                    Type::Uint => OpCode::TYPE_CAST_U32,
                    Type::U64 => OpCode::TYPE_CAST_U64,
                    _ => {
                        // For unknown/unsupported types, just leave the value as-is
                        return Ok(());
                    }
                };
                self.emit(opcode);
                self.last_expr_type = match target_type {
                    Type::Str(_) | Type::String | Type::StrSlice => ObjectType::String,
                    Type::Int | Type::I64 => ObjectType::Int,
                    Type::Float | Type::Double => ObjectType::Float,
                    Type::Uint | Type::USize | Type::U64 => ObjectType::Uint,
                    _ => ObjectType::Int,
                };
            }
            // Plan 120: Option type constructor - Some(value)
            Expr::Some(inner) => {
                // Compile inner expression (pushes value onto stack)
                self.compile_expr(inner)?;
                // Emit CREATE_SOME (wraps value in Some)
                self.emit(OpCode::CREATE_SOME);
            }
            // Plan 120: Option type constructor - None
            Expr::None => {
                // Emit CREATE_NONE (pushes None onto stack)
                self.emit(OpCode::CREATE_NONE);
            }
            // Plan 120: Result type constructor - Ok(value)
            Expr::Ok(inner) => {
                // Compile inner expression (pushes value onto stack)
                self.compile_expr(inner)?;
                // Emit CREATE_OK (wraps value in Ok)
                self.emit(OpCode::CREATE_OK);
            }
            // Plan 120: Result type constructor - Err(message)
            Expr::Err(msg) => {
                // Compile error message expression (should be a string)
                self.compile_expr(msg)?;
                // The message should be on stack as a string index
                // Emit CREATE_ERR (creates Err from string)
                self.emit(OpCode::CREATE_ERR);
            }
            // Plan 124: Async block - ~{ stmts }
            Expr::AsyncBlock { body, return_type: _ } => {
                // Create a Future value wrapping the async block body
                // The body will be executed when .await is called
                // For now, we store the body's code offset and compile it inline
                // In Phase 2.1, we use a simplified approach:
                // The async block immediately returns a Future that wraps the body
                // When .await is called, the body is executed

                // Store the current code position as the body's start
                let _body_offset = self.code.len() as u32;

                // Compile the body statements
                for stmt in &body.stmts {
                    self.compile_stmt(stmt)?;
                }

                // Emit CREATE_FUTURE with body offset
                // Note: In a full implementation, the body would be compiled separately
                // For Phase 2.1, we use a placeholder approach
                self.emit(OpCode::CREATE_FUTURE);
                self.code.extend_from_slice(&0u32.to_le_bytes()); // placeholder offset
            }
            // Plan 124: Await expression - expr.await
            Expr::Await { expr } => {
                // Compile the inner expression (should evaluate to a Future)
                self.compile_expr(expr)?;
                // Emit AWAIT_FUTURE to wait for the future's completion
                self.emit(OpCode::AWAIT_FUTURE);
            }
            // Plan 126: Go expression - expr.go (spawn background task)
            // Fire-and-forget semantics: spawn the future and discard the result
            Expr::Go { expr } => {
                // Compile the inner expression (should evaluate to a Future)
                self.compile_expr(expr)?;
                // Emit SPAWN_GO to spawn the future in background
                // SPAWN_GO pops the Future, spawns it, and pushes void
                self.emit(OpCode::SPAWN_GO);
            }
            Expr::Pair(pair) => {
                // Handle Pair as a single-element object for config syntax like: name: "value"
                // This is equivalent to Object {key: value}
                self.compile_expr(&pair.value)?;

                // Store key in the object_keys pool
                let key = self.ast_key_to_value_key(&pair.key);
                let key_index = self.object_keys.len() as u16;
                self.object_keys.push(vec![key.clone()]);

                // Track field type
                let ty = self.infer_object_type(&pair.value);
                self.object_types.push(vec![ty]);

                // Emit CREATE_OBJ with key_index and field count (1)
                self.emit(OpCode::CREATE_OBJ);
                self.code.extend_from_slice(&key_index.to_le_bytes());
                self.code.push(1); // field_count = 1
            }
            // Plan 095: Compile-time expression #{ expr }
            // For now, compile the inner expression normally
            // TODO: In full implementation, evaluate at compile time and substitute the result
            Expr::Comptime(hash_brace) => {
                // Compile the inner expression
                // The result will be on the stack
                self.compile_expr(&hash_brace.expr)?;
            }
            // Hold expression: bind value to name, evaluate body
            // For MVP: compile path, compile body statements
            Expr::Hold(hold) => {
                // Compile the path expression (the value to hold)
                self.compile_expr(&hold.path)?;
                // Compile the body statements
                for stmt in &hold.body.stmts {
                    self.compile_stmt(stmt)?;
                }
            }
            // Nil expression: push nil marker
            Expr::Nil => {
                // Use special nil marker value (i32::MIN + 1 = -2147483647)
                self.emit(OpCode::CONST_I32);
                self.emit_i32(-2147483647);
            }
            // Null literal: push -1 (VM representation of null, distinct from nil)
            Expr::Null => {
                self.emit(OpCode::CONST_I32);
                self.emit_i32(-1);
            }
            // Cover expression (enum variant pattern like Color.Red)
            Expr::Cover(crate::ast::Cover::Tag(tag_cover)) => {
                let key = format!("{}.{}", tag_cover.kind, tag_cover.tag);
                if let Some(&value) = self.enum_values.get(&key) {
                    self.emit(OpCode::CONST_I32);
                    self.emit_i32(value);
                } else {
                    return Err(AutoError::Msg(format!("Unknown enum variant: {}", key)));
                }
            }
            _ => {
                unimplemented!("Expression {:?}", expr);
            }
        }
        Ok(())
    }

    pub fn finish(self, name: String) -> Module {
        Module {
            name,
            code: self.code,
            exports: self.exports,
            relocs: self.relocs,
            strings: self.strings,
            // Plan 073: Include object_keys and object_types in module
            object_keys: self.object_keys,
            object_types: self.object_types,
        }
    }

    // === Helpers ===

    fn emit(&mut self, op: OpCode) {
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
        self.code.extend_from_slice(&val.to_le_bytes());
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

    // Plan 087 Phase 3: Infer expression type using the infer module
    // Returns the inferred type
    fn infer_expr_type(&mut self, expr: &Expr) -> crate::ast::Type {
        // Handle Dot expressions for field access type inference (e.g., a.x.type)
        if let Expr::Dot(obj, field) = expr {
            // First infer the type of the object (e.g., 'a' in 'a.x')
            let obj_ty = self.infer_expr_type(obj);

            // Look up field type from type template
            let field_name = field.as_ref();

            match &obj_ty {
                // User-defined types (type A { x int })
                Type::User(type_decl) => {
                    if let Some(member) = type_decl
                        .members
                        .iter()
                        .find(|m| m.name.as_ref() == field_name)
                    {
                        return member.ty.clone();
                    }
                }
                // Generic instances (Point<int>, List<int>, etc.)
                Type::GenericInstance(inst) => {
                    if let Some(template) = self
                        .generic_registry
                        .get_template(&inst.base_name.to_string())
                    {
                        if let Some(field_def) =
                            template.fields.iter().find(|f| f.name == field_name)
                        {
                            // Substitute type parameters with actual types
                            let generic_params: Vec<crate::ast::Name> = template
                                .generic_params
                                .iter()
                                .filter_map(|p| match p {
                                    crate::ast::GenericParam::Type(tp) => Some(tp.name.clone()),
                                    crate::ast::GenericParam::Const(_) => None,
                                })
                                .collect();
                            return field_def.field_type.substitute(&generic_params, &inst.args);
                        }
                    }
                }
                _ => {}
            }
            // Fall through to default inference if field type not found
        }

        // Check if this is a function call and we know the return type
        if let Expr::Call(call) = expr {
            // Extract function name from call expression
            let func_name = match call.name.as_ref() {
                Expr::Ident(name) => Some(name.to_string()),
                _ => None,
            };

            if let Some(name) = func_name {
                if let Some(ret_ty) = self.fn_return_types.get(&name) {
                    // We know the return type from fn_return_types
                    // Sync with infer_ctx for future lookups
                    let key = crate::ast::Name::from(&name);
                    if !self.infer_ctx.type_env.contains_key(&key) {
                        self.infer_ctx.type_env.insert(key, ret_ty.clone());
                    }
                    return ret_ty.clone();
                }
            }
        }

        // Ensure infer_ctx type_env is synced with var_types
        for (name, ty) in &self.var_types {
            let key = crate::ast::Name::from(name.as_str());
            if !self.infer_ctx.type_env.contains_key(&key) {
                self.infer_ctx.type_env.insert(key, ty.clone());
            }
        }
        // Use the infer module's comprehensive type inference
        infer_expr(&mut self.infer_ctx, expr)
    }

    // Plan 073 Stage A.5: Check if we should use float/double arithmetic
    // Returns true if either operand is a float/double or contains one recursively
    fn is_float_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        // Check if either operand is a float/double literal or contains one recursively
        self.contains_float(lhs) || self.contains_float(rhs)
    }

    // Plan 117: Recursively check if expression contains float/double literals or variables
    fn contains_float(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Float(_, _) | Expr::Double(_, _) => true,
            Expr::Ident(name) => {
                self.var_types
                    .get(name.as_ref())
                    .map(|t| matches!(t, Type::Float | Type::Double))
                    .unwrap_or(false)
            }
            Expr::Bina(lhs, _, rhs) => {
                self.contains_float(lhs) || self.contains_float(rhs)
            }
            Expr::Unary(_, inner) => self.contains_float(inner),
            Expr::Block(body) => {
                body.stmts.iter().any(|s| self.stmt_contains_float(s))
            }
            _ => false,
        }
    }

    // Plan 117: Check if a statement contains float expressions
    fn stmt_contains_float(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e) => self.contains_float(e),
            Stmt::Store(s) => self.contains_float(&s.expr),
            _ => false,
        }
    }

    // Plan 073 Stage A.5: Check if we should use double precision (f64) vs float (f32)
    fn is_double_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        self.contains_double(lhs) || self.contains_double(rhs)
    }

    // Plan 117: Recursively check if expression contains double literals
    fn contains_double(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Double(_, _) => true,
            Expr::Ident(name) => {
                self.var_types
                    .get(name.as_ref())
                    .map(|t| matches!(t, Type::Double))
                    .unwrap_or(false)
            }
            Expr::Bina(lhs, _, rhs) => {
                self.contains_double(lhs) || self.contains_double(rhs)
            }
            Expr::Unary(_, inner) => self.contains_double(inner),
            Expr::Block(body) => {
                body.stmts.iter().any(|s| self.stmt_contains_double(s))
            }
            _ => false,
        }
    }

    // Plan 117: Check if a statement contains double expressions
    fn stmt_contains_double(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e) => self.contains_double(e),
            Stmt::Store(s) => self.contains_double(&s.expr),
            _ => false,
        }
    }

    // Plan 117: Check if expression is an integer type that needs coercion to float
    fn needs_float_coercion(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Int(_) | Expr::I8(_) | Expr::Byte(_) | Expr::U8(_) => true,
            Expr::Ident(name) => {
                // Check variable type from type inference
                self.var_types
                    .get(name.as_ref())
                    .map(|t| matches!(t, Type::Int | Type::Byte))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    // Plan 117: Check if expression is an i64/u64 type that needs coercion to f64
    fn needs_double_coercion(&self, expr: &Expr) -> bool {
        match expr {
            Expr::I64(_) | Expr::U64(_) | Expr::Int(_) | Expr::I8(_) | Expr::U8(_) | Expr::Byte(_) | Expr::Uint(_) => true,
            Expr::Float(_, _) => true, // f32 needs promotion to f64
            Expr::Ident(name) => {
                self.var_types
                    .get(name.as_ref())
                    .map(|t| matches!(t, Type::I64 | Type::U64 | Type::Int | Type::Float | Type::Uint | Type::USize))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    // Check if expression is specifically u64 (vs i64) for choosing the right coercion opcode
    fn is_u64_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::U64(_) => true,
            Expr::Ident(name) => self
                .var_types
                .get(name.as_ref())
                .map(|t| matches!(t, Type::U64))
                .unwrap_or(false),
            _ => false,
        }
    }

    // Check if this is a string operation (either operand is a string type)
    // Used to emit STR_CAT instead of ADD for string concatenation
    fn is_string_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        self.is_string_expr(lhs) || self.is_string_expr(rhs)
    }

    fn is_u64_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        self.contains_u64(lhs) || self.contains_u64(rhs)
    }

    fn contains_u64(&self, expr: &Expr) -> bool {
        match expr {
            Expr::U64(_) | Expr::I64(_) => true,
            Expr::Cast { target_type, .. } => matches!(target_type,
                Type::U64 | Type::I64 | Type::USize | Type::Uint),
            Expr::Ident(name) => self.var_types.get(name.as_ref())
                .map(|t| matches!(t, Type::U64 | Type::I64)).unwrap_or(false),
            Expr::Call(call) => {
                if let Expr::Ident(fn_name) = call.name.as_ref() {
                    self.fn_return_types.get(fn_name.as_ref())
                        .map(|t| matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            Expr::Bina(lhs, _, rhs) => self.contains_u64(lhs) || self.contains_u64(rhs),
            Expr::Unary(_, inner) => self.contains_u64(inner),
            _ => false,
        }
    }

    // Type hint for f-string parts to guide BUILD_FSTR value popping
    // Reflects the ACTUAL stack representation, not the logical type
    fn expr_type_hint(&self, expr: &Expr) -> FStrPartType {
        match expr {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => FStrPartType::String,
            // Float compiles to CONST_F32 (4 bytes, 1 slot)
            Expr::Float(_, _) => FStrPartType::Float32,
            // Double compiles to CONST_F64 (8 bytes, 2 slots)
            Expr::Double(_, _) => FStrPartType::Float64,
            // u64/i64 literals compile to 2-slot values
            Expr::U64(_) | Expr::I64(_) => FStrPartType::Uint64,
            Expr::Ident(name) => {
                let name_str = name.to_string();
                if let Some(ty) = self.var_types.get(&name_str) {
                    match ty {
                        Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => FStrPartType::String,
                        Type::Float => FStrPartType::Float32,
                        Type::Double => FStrPartType::Float64,
                        Type::U64 | Type::I64 | Type::Uint | Type::USize => FStrPartType::Uint64,
                        // All other locals are stored as i32 (1 slot)
                        _ => FStrPartType::Int,
                    }
                } else {
                    FStrPartType::Int
                }
            }
            Expr::Bina(lhs, _, rhs) => {
                // Check if binary result is a u64 or f64 operation (2 slots)
                if self.is_u64_operation(lhs, rhs) {
                    FStrPartType::Uint64
                } else if self.is_double_operation(lhs, rhs) {
                    FStrPartType::Float64
                } else if self.is_float_operation(lhs, rhs) {
                    FStrPartType::Float32
                } else {
                    FStrPartType::Int
                }
            }
            Expr::Call(call) => {
                // Check fn_return_types for the called function
                if let Expr::Ident(fn_name) = call.name.as_ref() {
                    if let Some(ret_ty) = self.fn_return_types.get(fn_name.as_ref()) {
                        match ret_ty {
                            Type::Double => FStrPartType::Float64,
                            Type::U64 | Type::I64 | Type::USize | Type::Uint => FStrPartType::Uint64,
                            Type::Float => FStrPartType::Float32,
                            Type::Str(_) | Type::String => FStrPartType::String,
                            _ => FStrPartType::Int,
                        }
                    } else {
                        FStrPartType::Int
                    }
                } else {
                    FStrPartType::Int
                }
            }
            _ => FStrPartType::Int,
        }
    }

    fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => true,
            Expr::Ident(name) => {
                // Check inferred type for the variable
                let ty = self.infer_ctx.type_env.get(name);
                matches!(ty, Some(Type::Str(_)) | Some(Type::String))
            }
            _ => false,
        }
    }

    // Plan 073: Convert expression to ObjectType for object field tracking
    pub(crate) fn infer_object_type(&self, expr: &Expr) -> ObjectType {
        match expr {
            Expr::Float(_, _) => ObjectType::Float,
            Expr::Double(_, _) => ObjectType::Double,
            Expr::Int(_) | Expr::I8(_) | Expr::I64(_) => ObjectType::Int,
            Expr::Uint(_) | Expr::U64(_) => ObjectType::Uint,
            Expr::U8(_) => ObjectType::Int,  // U8 arithmetic returns plain int
            Expr::Byte(_) => ObjectType::Byte, // Plan 118: Byte has its own type for hex formatting
            Expr::Str(_) | Expr::CStr(_) => ObjectType::String,
            Expr::Char(_) => ObjectType::Char,
            Expr::Bool(_) => ObjectType::Bool,
            // Plan 197 Task 3: Method chaining — resolve return type from fn_return_types
            Expr::Call(call) => {
                // Try to resolve return type from fn_return_types
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    let fn_name = format!("{}.{}", self.expr_to_name(obj.as_ref()), method.as_ref());
                    if let Some(ret_ty) = self.fn_return_types.get(&fn_name) {
                        self.type_to_object_type(ret_ty)
                    } else {
                        ObjectType::NestedObject
                    }
                } else if let Expr::Ident(name) = call.name.as_ref() {
                    if let Some(ret_ty) = self.fn_return_types.get(name.as_ref()) {
                        self.type_to_object_type(ret_ty)
                    } else {
                        ObjectType::NestedObject
                    }
                } else {
                    ObjectType::NestedObject
                }
            }
            // Plan 073: Nested object, node, pair and array types
            Expr::Object(_) | Expr::Node(_) | Expr::Bina(_, _, _) | Expr::If(_) | Expr::Lambda(_) | Expr::Closure(_) | Expr::Pair(_) => ObjectType::NestedObject,
            Expr::Array(_) => ObjectType::Array,
            // Plan 118 Phase 4: Check variable types for identifier expressions
            Expr::Ident(name) => {
                let name_str = name.to_string();
                if let Some(var_type) = self.var_types.get(&name_str) {
                    match var_type {
                        Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => ObjectType::String,
                        Type::Char => ObjectType::Char,
                        Type::Int | Type::I64 => ObjectType::Int,
                        Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                        Type::Byte => ObjectType::Byte,
                        Type::Float => ObjectType::Float,
                        Type::Double => ObjectType::Double,
                        Type::Bool => ObjectType::Bool,
                        Type::Array(_) | Type::RuntimeArray(_) => ObjectType::Array,
                        _ => ObjectType::Int,
                    }
                } else {
                    ObjectType::Int
                }
            }
            // For other expressions, default to Int
            Expr::Cast { target_type, .. } => {
                match target_type {
                    Type::Int | Type::I64 => ObjectType::Int,
                    Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                    Type::Float => ObjectType::Float,
                    Type::Double => ObjectType::Double,
                    Type::Byte => ObjectType::Byte,
                    Type::Bool => ObjectType::Bool,
                    Type::Str(_) | Type::String => ObjectType::String,
                    Type::Char => ObjectType::Char,
                    _ => ObjectType::Int,
                }
            }
            _ => ObjectType::Int,
        }
    }

    /// Plan 197 Task 3: Try to infer a user-defined type name from an expression
    /// by looking up its return type in fn_return_types.
    /// Returns the type name (e.g., "Point") if found, or None.
    fn infer_user_type_name(&self, expr: &Expr) -> Option<String> {
        if let Expr::Call(call) = expr {
            let fn_name = if let Expr::Dot(obj, method) = call.name.as_ref() {
                format!("{}.{}", self.expr_to_name(obj.as_ref()), method.as_ref())
            } else if let Expr::Ident(name) = call.name.as_ref() {
                name.to_string()
            } else {
                return None;
            };
            if let Some(ret_ty) = self.fn_return_types.get(&fn_name) {
                match ret_ty {
                    Type::User(td) => return Some(td.name.to_string()),
                    Type::Enum(ed) => return Some(ed.borrow().name.to_string()),
                    _ => {}
                }
            }
        }
        None
    }

    /// Plan 197 Task 3: Convert an expression to its dot-separated name for fn_return_types lookup
    fn expr_to_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(name) => name.to_string(),
            Expr::Dot(obj, method) => format!("{}.{}", self.expr_to_name(obj), method),
            _ => "Unknown".to_string(),
        }
    }

    /// Plan 197 Task 3: Map a Type to ObjectType for object field tracking
    fn type_to_object_type(&self, ty: &Type) -> ObjectType {
        match ty {
            Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => ObjectType::String,
            Type::Char => ObjectType::Char,
            Type::Int | Type::I64 => ObjectType::Int,
            Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
            Type::Byte => ObjectType::Byte,
            Type::Float => ObjectType::Float,
            Type::Double => ObjectType::Double,
            Type::Bool => ObjectType::Bool,
            Type::Array(_) | Type::RuntimeArray(_) => ObjectType::Array,
            _ => ObjectType::NestedObject,
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

        vm_debug!("DEBUG patch_jump: placeholder_idx={}, target={}, anchor={}, offset={}",
            placeholder_idx, target, anchor, offset
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

    // === Symbol Table Helpers ===

    /// Look up variable in symbol table (checks all scopes from innermost to outermost)
    fn lookup_var(&self, name: &str) -> Option<usize> {
        // Check innermost scope first (current function/block)
        for scope in self.scope_stack.iter().rev() {
            if let Some(&index) = scope.get(name) {
                return Some(index);
            }
        }

        // No variable found
        None
    }

    /// Add variable to current scope and return its index
    fn add_var(&mut self, name: &str) -> usize {
        // Calculate next available slot offset (accounts for 2-slot variables)
        let mut next_offset: usize = 0;
        for scope in &self.scope_stack {
            for (var_name, &existing_index) in scope {
                let sc = if matches!(self.var_types.get(var_name), Some(Type::U64 | Type::I64 | Type::Double)) { 2 } else { 1 };
                next_offset = next_offset.max(existing_index + sc);
            }
        }

        // Check if this variable is u64/i64 (occupies two slots)
        let is_64bit = matches!(self.var_types.get(name), Some(Type::U64 | Type::I64 | Type::Double));
        let slot_count = if is_64bit { 2 } else { 1 };

        // Update max_locals to reflect the high-water mark of variables (including parameters)
        self.max_locals = self.max_locals.max(next_offset + slot_count);

        let scope = self
            .scope_stack
            .last_mut()
            .expect("Scope stack should never be empty");
        scope.insert(name.to_string(), next_offset);
        // Reserve the second slot for u64/i64 variables
        if is_64bit {
            scope.insert(format!("__{}_high", name), next_offset + 1);
        }
        next_offset
    }

    /// Push a new scope (for function entry, blocks, etc.)
    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Pop the current scope
    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// Emit STORE_LOCAL for a given local index
    /// Plan 087 Phase 3: Distinguishes between parameters (before BP) and locals (after BP)
    /// Parameters: fn_scope_start <= index < fn_scope_start + n_args
    /// Locals: otherwise
    /// Uses dedicated opcodes for locals 0-1 for performance
    fn emit_store_loc(&mut self, index: usize) {
        let n_args = self.current_fn_n_args;
        let fn_scope_start = self.fn_scope_start;

        // Check if this is a parameter: index must be within [fn_scope_start, fn_scope_start + n_args)
        if index >= fn_scope_start && index < fn_scope_start + n_args {
            // This is a parameter, stored before BP
            // Calculate relative parameter index (0, 1, 2, ...)
            let param_index = index - fn_scope_start;

            // Store parameter using STORE_LOCAL with negative offset logic
            self.emit(OpCode::STORE_LOCAL);
            // Encode as negative offset (0x80..0xFF means parameter)
            let encoded_index = 0x80 + param_index as u8;
            self.code.push(encoded_index);
        } else {
            // This is a local variable, stored after BP
            // Local index is relative to the position after all parameters
            let local_index = index - fn_scope_start - n_args;

            match local_index {
                0 => self.emit(OpCode::STORE_LOC_0),
                1 => self.emit(OpCode::STORE_LOC_1),
                _ => {
                    self.emit(OpCode::STORE_LOCAL);
                    self.code.push(local_index as u8);
                }
            }
        }
    }

    /// Emit LOAD_LOCAL for a given local index
    /// Plan 087 Phase 3: Distinguish between parameters (before BP) and locals (after BP)
    /// Parameters: fn_scope_start <= index < fn_scope_start + n_args, stored at BP - n_args + (index - fn_scope_start)
    /// Locals: otherwise, stored at BP + 1 + (index - fn_scope_start - n_args)
    /// Uses dedicated opcodes for locals 0-2 for performance
    fn emit_load_loc(&mut self, index: usize) {
        vm_debug!("DEBUG: emit_load_loc called with index={}, n_args={}, fn_scope_start={}",
            index, self.current_fn_n_args, self.fn_scope_start
        );

        let n_args = self.current_fn_n_args;
        let fn_scope_start = self.fn_scope_start;

        // Check if this is a parameter: index must be within [fn_scope_start, fn_scope_start + n_args)
        if index >= fn_scope_start && index < fn_scope_start + n_args {
            // This is a parameter, stored before BP
            // Calculate relative parameter index (0, 1, 2, ...)
            let param_index = index - fn_scope_start;

            // Stack layout: [..., args(0), args(1), ..., return_addr, old_bp, locals...]
            //                        ^- BP-n_args     ^- BP-1    ^- BP
            // Parameter i is at BP - n_args + i
            let offset = (n_args - param_index) as i32; // Positive offset going backwards from BP
            vm_debug!("DEBUG: Loading parameter {} (absolute index {}) at BP-{}",
                param_index, index, offset
            );

            // Load parameter using LOAD_LOCAL with negative offset logic
            // For now, use LOAD_LOCAL with special encoding
            self.emit(OpCode::LOAD_LOCAL);
            // Encode as negative offset (0x80..0xFF means parameter)
            let encoded_index = 0x80 + param_index as u8; // 0x80 means param 0, 0x81 means param 1, etc.
            self.code.push(encoded_index);
            vm_debug!("DEBUG: Emitting LOAD_LOCAL with encoded parameter index 0x{:02x}",
                encoded_index
            );
        } else {
            // This is a local variable, stored after BP
            // Local index is relative to the position after all parameters
            let local_index = index - fn_scope_start - n_args;
            vm_debug!("DEBUG: Loading local variable {} (absolute index {}) at BP+1+{}",
                local_index, index, local_index
            );

            match local_index {
                0 => {
                    vm_debug!("DEBUG: Emitting LOAD_LOC_0 (opcode 0x22)");
                    self.emit(OpCode::LOAD_LOC_0);
                }
                1 => {
                    vm_debug!("DEBUG: Emitting LOAD_LOC_1");
                    self.emit(OpCode::LOAD_LOC_1);
                }
                2 => {
                    vm_debug!("DEBUG: Emitting LOAD_LOC_2");
                    self.emit(OpCode::LOAD_LOC_2);
                }
                _ => {
                    self.emit(OpCode::LOAD_LOCAL);
                    self.code.push(local_index as u8);
                    vm_debug!("DEBUG: Emitting LOAD_LOCAL with local index {}",
                        local_index
                    );
                }
            }
        }
    }

    /// Plan 088 Phase 4: Emit LOAD_REF for immutable reference
    fn emit_load_ref(&mut self, index: usize) {
        self.emit(OpCode::LOAD_REF);
        self.code.extend_from_slice(&(index as u32).to_le_bytes());
    }

    /// Plan 088 Phase 4: Emit STORE_REF for immutable reference
    #[allow(dead_code)]
    fn emit_store_ref(&mut self, index: usize) {
        self.emit(OpCode::STORE_REF);
        self.code.extend_from_slice(&(index as u32).to_le_bytes());
    }

    /// Plan 088 Phase 4: Emit LOAD_MUT_REF for mutable reference
    fn emit_load_mut_ref(&mut self, index: usize) {
        vm_debug!("DEBUG: emit_load_mut_ref called with index={}", index);
        self.emit(OpCode::LOAD_MUT_REF);
        let bytes = (index as u32).to_le_bytes();
        vm_debug!("DEBUG: emit_load_mut_ref bytes: {:02x} {:02x} {:02x} {:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );
        self.code.extend_from_slice(&bytes);
    }

    /// Plan 088 Phase 4: Emit STORE_MUT_REF for mutable reference
    #[allow(dead_code)]
    fn emit_store_mut_ref(&mut self, index: usize) {
        self.emit(OpCode::STORE_MUT_REF);
        self.code.extend_from_slice(&(index as u32).to_le_bytes());
    }

    /// Emit LOAD_CAPTURED for a captured variable by name (Plan 071)
    fn emit_load_captured(&mut self, var_name: &str) {
        let var_idx = self.add_string(var_name);
        self.emit(OpCode::LOAD_CAPTURED);
        self.code.extend_from_slice(&var_idx.to_le_bytes());
    }

    /// Emit STORE_CAPTURED for a captured variable by name (Plan 071)
    fn emit_store_captured(&mut self, var_name: &str) {
        let var_idx = self.add_string(var_name);
        self.emit(OpCode::STORE_CAPTURED);
        self.code.extend_from_slice(&var_idx.to_le_bytes());
    }

    // === Plan 088 Phase 4: Smart Parameter Passing ===

    /// Get parameter information for a function at a specific index
    /// Returns (type, mode) if found, None otherwise
    fn get_param_info(&self, func_name: &str, param_index: usize) -> Option<(Type, ParamMode)> {
        if let Some(params) = self.fn_params.get(func_name) {
            if param_index < params.len() {
                let param = &params[param_index];
                return Some((param.ty.clone(), param.mode));
            }
        }
        None
    }

    /// Compile a single argument for a function call with smart parameter passing
    /// This implements the Plan 088 ABO-01 strategy: "Semantic View, Implementation Copy"
    fn compile_call_arg(
        &mut self,
        arg: &Expr,
        func_name: &str,
        param_index: usize,
    ) -> AutoResult<()> {
        // Get target parameter info (type and mode)
        let param_info = self.get_param_info(func_name, param_index);

        if let Some((param_ty, param_mode)) = param_info {
            // We have parameter info, use smart parameter passing
            match arg {
                Expr::Ident(var_name) => {
                    // Argument is a variable/identifier
                    // === BUG FIX === Use lookup_var() instead of self.locals.get()
                    // to properly search the scope stack
                    if let Some(var_index) = self.lookup_var(var_name.as_ref()) {
                        // Local variable: choose loading strategy based on type and mode
                        match param_mode {
                            ParamMode::View => {
                                // View mode: immutable reference
                                // TODO: Once LOAD_REF/STORE_REF are fully supported on the callee side,
                                // restore the reference passing optimization for large objects.
                                // Currently all parameters are passed by value (the instance ID / heap object ID).
                                self.emit_load_loc(var_index);
                            }
                            ParamMode::Mut => {
                                // Mut mode: mutable reference
                                // TODO: Once LOAD_MUT_REF is fully supported on the callee side,
                                // restore the reference passing optimization for large objects.
                                // Currently all parameters are passed by value (the instance ID / heap object ID).
                                self.emit_load_loc(var_index);
                            }
                            ParamMode::Copy => {
                                // Copy mode: explicit value passing
                                if param_ty.is_optimized_by_value() {
                                    // Small object: direct value passing (LOAD_LOC)
                                    self.emit_load_loc(var_index);
                                } else {
                                    // Large object + Copy: needs clone
                                    // For now, use LOAD_LOC (TODO: implement clone in future)
                                    self.emit_load_loc(var_index);
                                }
                            }
                            #[allow(deprecated)]
                            ParamMode::Take => {
                                // Take mode: move semantics (value passing)
                                // DEPRECATED: Use Move instead
                                self.emit_load_loc(var_index);
                            }
                            ParamMode::Move => {
                                // Move mode: ownership transfer (value passing)
                                self.emit_load_loc(var_index);
                            }
                        }
                        return Ok(());
                    }
                }
                _ => {
                    // Argument is a complex expression (constant, operation, etc.)
                    // Just compile it normally
                }
            }
        }

        // Fallback: compile argument as expression
        self.compile_expr(arg)
    }

    // === Closure Support (Plan 071) ===

    /// Get the current captured_vars map (top of stack)
    /// Plan 071 Phase 6.2: Helper for accessing captured variables
    fn current_captured_vars(&self) -> &HashMap<String, usize> {
        self.captured_vars_stack.last().unwrap_or_else(|| {
            // If stack is empty, return empty map (not in a closure)
            static EMPTY_MAP: std::sync::OnceLock<std::collections::HashMap<String, usize>> =
                std::sync::OnceLock::new();
            EMPTY_MAP.get_or_init(|| HashMap::new())
        })
    }

    /// Get mutable reference to current captured_vars map (top of stack)
    /// Plan 071 Phase 6.2: Helper for modifying captured variables
    #[allow(dead_code)]
    fn current_captured_vars_mut(&mut self) -> &mut HashMap<String, usize> {
        if self.captured_vars_stack.is_empty() {
            // If stack is empty, push a new map
            self.captured_vars_stack.push(HashMap::new());
        }
        self.captured_vars_stack.last_mut().unwrap()
    }

    /// Push a new captured_vars level (for compiling a closure body)
    /// Plan 071 Phase 6.2: Support nested closures
    fn push_captured_vars(&mut self, vars: HashMap<String, usize>) {
        self.captured_vars_stack.push(vars);
    }

    /// Pop the current captured_vars level (after compiling a closure body)
    /// Plan 071 Phase 6.2: Support nested closures
    fn pop_captured_vars(&mut self) -> Option<HashMap<String, usize>> {
        if self.captured_vars_stack.is_empty() {
            None
        } else {
            self.captured_vars_stack.pop()
        }
    }

    /// Find free variables in an expression (variables that should be captured)
    /// Excludes: parameters and locally-defined variables
    fn find_free_vars(&self, expr: &Expr, params: &HashSet<String>) -> Vec<String> {
        let mut free_vars = HashSet::new();
        self.collect_free_vars(expr, params, &mut free_vars);
        free_vars.into_iter().collect()
    }

    /// Recursively collect free variables from an expression
    fn collect_free_vars(
        &self,
        expr: &Expr,
        exclude: &HashSet<String>,
        free_vars: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Ident(name) => {
                let name_str = name.to_string();
                // Only collect if not in exclude list (parameters/locals)
                if !exclude.contains(&name_str) {
                    free_vars.insert(name_str);
                }
            }
            Expr::Bina(lhs, _op, rhs) => {
                self.collect_free_vars(lhs, exclude, free_vars);
                self.collect_free_vars(rhs, exclude, free_vars);
            }
            Expr::Unary(_op, rhs) => {
                self.collect_free_vars(rhs, exclude, free_vars);
            }
            Expr::Call(call) => {
                self.collect_free_vars(&call.name, exclude, free_vars);
                for arg in &call.args.args {
                    if let crate::ast::Arg::Pos(expr) = arg {
                        self.collect_free_vars(expr, exclude, free_vars);
                    }
                }
            }
            Expr::Array(elems) => {
                for elem in elems {
                    self.collect_free_vars(elem, exclude, free_vars);
                }
            }
            Expr::Block(body) => {
                // For block expressions, exclude local variables defined in the block
                let inner_exclude = exclude.clone();
                for stmt in &body.stmts {
                    if let Stmt::Expr(e) = stmt {
                        self.collect_free_vars(e, &inner_exclude, free_vars);
                    } else if let Stmt::Return(e) = stmt {
                        self.collect_free_vars(e, &inner_exclude, free_vars);
                    }
                    // TODO: Exclude local variable definitions from inner_exclude
                }
            }
            Expr::If(if_expr) => {
                for branch in &if_expr.branches {
                    self.collect_free_vars(&branch.cond, exclude, free_vars);
                    for stmt in &branch.body.stmts {
                        if let Stmt::Expr(e) = stmt {
                            self.collect_free_vars(e, exclude, free_vars);
                        }
                    }
                }
            }
            Expr::Closure(inner_closure) => {
                // For nested closures, process inner body with updated excludes
                let mut inner_exclude = exclude.clone();
                for p in &inner_closure.params {
                    inner_exclude.insert(p.name.to_string());
                }
                self.collect_free_vars(&inner_closure.body, &inner_exclude, free_vars);
            }
            // Dot expressions - check object (e.g., x in x.view)
            Expr::View(inner) | Expr::Mut(inner) | Expr::Move(inner) | Expr::Take(inner) => {
                self.collect_free_vars(inner, exclude, free_vars);
            }
            Expr::Dot(obj, _method) => {
                self.collect_free_vars(obj, exclude, free_vars);
            }
            // Index expressions
            Expr::Index(arr, idx) => {
                self.collect_free_vars(arr, exclude, free_vars);
                self.collect_free_vars(idx, exclude, free_vars);
            }
            // Primitives - no identifiers to collect
            Expr::Int(_)
            | Expr::Float(_, _)
            | Expr::Str(_)
            | Expr::Bool(_)
            | Expr::Nil
            | Expr::Byte(_) => {}
            // Other expressions - add more cases as needed
            _ => {}
        }
    }

    /// Add string constant to string pool and return its index
    pub fn add_string(&mut self, s: &str) -> u16 {
        // Check if string already exists
        for (idx, existing) in self.strings.iter().enumerate() {
            if existing == s.as_bytes() {
                return idx as u16;
            }
        }

        // Add new string
        let idx = self.strings.len();
        self.strings.push(s.as_bytes().to_vec());
        idx as u16
    }

    /// Get span from an expression for error reporting
    /// Plan 071 Phase 6.1: Helper for borrow checking errors
    fn get_expr_span(&self, _expr: &Expr) -> SourceSpan {
        // TODO: Track spans in AST nodes during parsing
        // For now, use a zero-length span at offset 0
        SourceSpan::new(0_usize.into(), 0_usize.into())
    }

    /// Check if a variable is captured with unsafe borrowing (.view or .mut)
    /// Returns the expression that uses unsafe borrowing, if found
    /// Plan 071 Phase 6.1: Borrow Checking Integration
    fn check_unsafe_capture<'a>(&'a self, var_name: &str, expr: &'a Expr) -> Option<&'a Expr> {
        match expr {
            // Direct unsafe borrow: x.view or x.mut
            Expr::View(inner) => {
                // Check if this is borrowing the target variable
                if let Expr::Ident(name) = inner.as_ref() {
                    if name.to_string() == var_name {
                        return Some(expr); // Found unsafe capture
                    }
                }
                // Recursively check inner expression
                self.check_unsafe_capture(var_name, inner)
            }
            Expr::Mut(inner) => {
                // Check if this is borrowing the target variable
                if let Expr::Ident(name) = inner.as_ref() {
                    if name.to_string() == var_name {
                        return Some(expr); // Found unsafe capture
                    }
                }
                // Recursively check inner expression
                self.check_unsafe_capture(var_name, inner)
            }

            // Binary expressions - check both sides
            Expr::Bina(lhs, _op, rhs) => {
                // First check left side
                if let Some(found) = self.check_unsafe_capture(var_name, lhs) {
                    return Some(found);
                }
                // Then check right side
                self.check_unsafe_capture(var_name, rhs)
            }

            // Unary expressions - check operand
            Expr::Unary(_op, rhs) => self.check_unsafe_capture(var_name, rhs),

            // Function calls - check arguments
            Expr::Call(call) => {
                // Check function name
                if let Some(found) = self.check_unsafe_capture(var_name, &call.name) {
                    return Some(found);
                }
                // Check arguments
                for arg in &call.args.args {
                    if let crate::ast::Arg::Pos(arg_expr) = arg {
                        if let Some(found) = self.check_unsafe_capture(var_name, arg_expr) {
                            return Some(found);
                        }
                    }
                }
                None
            }

            // Arrays - check elements
            Expr::Array(elems) => {
                for elem in elems {
                    if let Some(found) = self.check_unsafe_capture(var_name, elem) {
                        return Some(found);
                    }
                }
                None
            }

            // Block expressions - check statements
            Expr::Block(body) => {
                for stmt in &body.stmts {
                    match stmt {
                        Stmt::Expr(e) => {
                            if let Some(found) = self.check_unsafe_capture(var_name, e) {
                                return Some(found);
                            }
                        }
                        Stmt::Return(e) => {
                            if let Some(found) = self.check_unsafe_capture(var_name, e) {
                                return Some(found);
                            }
                        }
                        // Other statements don't contain expressions we care about
                        _ => {}
                    }
                }
                None
            }

            // Closure expressions - recurse into closure body
            Expr::Closure(inner_closure) => {
                self.check_unsafe_capture(var_name, &inner_closure.body)
            }

            // If expressions - check branches
            Expr::If(if_expr) => {
                // Check all branches
                for branch in &if_expr.branches {
                    // Check condition
                    if let Some(found) = self.check_unsafe_capture(var_name, &branch.cond) {
                        return Some(found);
                    }
                    // Check branch body
                    if let Some(found) = self.check_unsafe_capture_in_body(var_name, &branch.body) {
                        return Some(found);
                    }
                }
                // Check else branch
                if let Some(else_body) = &if_expr.else_ {
                    self.check_unsafe_capture_in_body(var_name, else_body)
                } else {
                    None
                }
            }

            // Index expressions - check both parts
            Expr::Index(arr, idx) => {
                if let Some(found) = self.check_unsafe_capture(var_name, arr) {
                    return Some(found);
                }
                self.check_unsafe_capture(var_name, idx)
            }

            // Dot expressions - check both object and method
            Expr::Dot(obj, _method) => {
                // Check object (e.g., x in x.view)
                if let Expr::Ident(name) = obj.as_ref() {
                    if name.to_string() == var_name {
                        // This is a direct reference to the variable
                        // Check if this dot expression is part of a borrow
                        // We need to look at the outer context to determine this
                        // For now, we'll be conservative and check the method name
                        return None; // Safe - just accessing a field
                    }
                }
                self.check_unsafe_capture(var_name, obj)
            }

            // Identifiers - direct references are safe (copy semantics)
            // Only .view/.mut are unsafe, not direct variable access
            Expr::Ident(_name) => None,

            // Other expressions - no unsafe borrowing possible
            _ => None,
        }
    }

    /// Check for unsafe captures in a Body (block)
    /// Plan 071 Phase 6.1: Helper for checking if/branch bodies
    fn check_unsafe_capture_in_body<'a>(
        &'a self,
        var_name: &str,
        body: &'a crate::ast::Body,
    ) -> Option<&'a Expr> {
        for stmt in &body.stmts {
            if let Stmt::Expr(expr) = stmt {
                if let Some(found) = self.check_unsafe_capture(var_name, expr) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Compile closure expression (Plan 071)
    pub fn compile_closure(&mut self, closure: &Closure) -> AutoResult<()> {
        // Step 1: Find free variables to capture
        let param_names: HashSet<String> =
            closure.params.iter().map(|p| p.name.to_string()).collect();
        let free_vars = self.find_free_vars(&closure.body, &param_names);

        // Plan 071 Phase 6.1: Borrow Checking - Check for unsafe captures
        // Block .view/.mut in closure capture to prevent dangling references
        for var_name in &free_vars {
            if let Some(unsafe_expr) = self.check_unsafe_capture(var_name, &closure.body) {
                // Found unsafe capture - emit compiler error
                let span = self.get_expr_span(unsafe_expr);
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Cannot capture borrowed value '{0}' in closure. \
                        Closures may outlive their parent scope, causing dangling references. \
                        Use .take to transfer ownership, or remove .view/.mut. \
                        Note: Default capture semantics copy the value, which is safe.",
                        var_name
                    ),
                    span,
                }
                .into());
            }
        }

        // Step 2: For each captured variable, emit code to load its current value
        // For MVP, we emit LOAD_LOCAL for each captured variable
        // TODO: In Phase 3.6, emit proper variable loading based on scope
        for var_name in &free_vars {
            // Try to look up the variable in current scope
            if let Some(var_index) = self.lookup_var(var_name) {
                self.emit_load_loc(var_index);
            } else {
                // Variable not found - emit 0 as fallback
                self.emit(OpCode::CONST_0);
            }
        }

        // Step 3: Emit CLOSURE opcode at current position (Plan 071 Phase 6.2)
        // Save position for reloc entry
        let closure_opcode_offset = self.code.len();

        // Create unique symbol name for this closure
        let closure_symbol = format!("closure_{}", closure_opcode_offset);

        // Emit CLOSURE opcode with placeholder address
        self.emit(OpCode::CLOSURE);
        let func_addr_offset = self.code.len() as u32; // Position where func_addr will be
        self.code.extend_from_slice(&(0u32).to_le_bytes()); // Placeholder - will be filled later
        self.code.push(free_vars.len() as u8); // capture_count
        self.code.push(closure.params.len() as u8); // n_args (for CALL_CLOSURE)

        // Emit variable name indices for each captured variable
        for var_name in &free_vars {
            let var_idx = self.add_string(var_name);
            self.code.extend_from_slice(&var_idx.to_le_bytes());
        }

        // Step 3.5: Emit JMP to skip closure body during normal execution
        // After CLOSURE opcode, we need to jump over the closure body
        // JMP offset will be patched later after we know body size
        self.emit(OpCode::JMP);
        let jmp_offset_pos = self.code.len();
        self.code.extend_from_slice(&(0i16).to_le_bytes()); // Placeholder - will be filled after body

        // Step 4: Compile closure body as separate function (Plan 071 Phase 6.2)
        // Closure body is compiled AFTER the CLOSURE opcode (at the end of current code)

        // Create captured variable map for this closure
        let mut new_captured_vars = HashMap::new();
        for (idx, var_name) in free_vars.iter().enumerate() {
            new_captured_vars.insert(var_name.clone(), idx);
        }

        // Push new captured_vars level for nested closures
        self.push_captured_vars(new_captured_vars);

        // Compile closure body at the END of current code
        let func_addr = self.code.len() as u32;

        // Save old current_fn_n_args and set new value for closure
        let old_fn_n_args = self.current_fn_n_args;
        let old_fn_scope_start = self.fn_scope_start;
        self.current_fn_n_args = closure.params.len();

        // Enter new scope for closure parameters
        self.push_scope();

        // Record starting index for closure scope (for correct parameter indexing)
        self.fn_scope_start = self.scope_stack.iter().map(|s| s.len()).sum();

        for param in &closure.params {
            self.add_var(&param.name);
        }

        // Compile closure body expression
        self.compile_expr(&closure.body)?;

        // Emit RET for closure
        self.emit(OpCode::RET);
        self.code.push(closure.params.len() as u8);

        // Exit closure scope
        self.pop_scope();

        // Restore old current_fn_n_args and fn_scope_start
        self.current_fn_n_args = old_fn_n_args;
        self.fn_scope_start = old_fn_scope_start;

        // Pop captured_vars (restore outer closure's captured vars)
        self.pop_captured_vars();

        // Step 4.5: Back-fill JMP offset to skip closure body during normal execution
        let body_end_addr = self.code.len() as u32;
        let jmp_offset = ((body_end_addr as i32) - (jmp_offset_pos as i32 + 2)) as i16; // +2 for the i16 offset itself
        let jmp_bytes = jmp_offset.to_le_bytes();
        for (i, byte) in jmp_bytes.iter().enumerate() {
            self.code[jmp_offset_pos as usize + i] = *byte;
        }

        // Step 5: Back-fill the func_addr in the CLOSURE opcode
        // Now we know the actual function address, so we can fill it in
        let func_addr_bytes = func_addr.to_le_bytes();
        for (i, byte) in func_addr_bytes.iter().enumerate() {
            let idx = func_addr_offset as usize + i;
            self.code[idx] = *byte;
        }

        // Step 6: Create reloc entry for this closure (Plan 071 Phase 6.2)
        self.relocs.push(crate::vm::loader::RelocEntry {
            offset: func_addr_offset,
            symbol_name: closure_symbol.clone(),
            reloc_type: crate::vm::loader::RelocType::FuncCall,
            source_pos: None,
        });

        // Export the closure function address
        self.exports.insert(closure_symbol, func_addr);

        Ok(())
    }

    // ========== Plan 073: Instance Method Call Support ==========

    /// Check if a name is likely a type name (capitalized first letter)
    /// This is a heuristic following Rust/AutoLang naming conventions:
    /// - Type names: Capitalized (List, Point, MyType)
    /// - Variables/Functions: lowercase (list, foo, my_var)
    fn is_type_name_heuristic(&self, name: &str) -> bool {
        name.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    /// Infer type name from a variable name (Plan 080: with explicit type tracking)
    ///
    /// First checks the var_types map (tracked during let declarations)
    /// Plan 193: Infer precise source type for .to() conversion opcode selection.
    /// Uses var_types for identifiers, AST node type for literals.
    fn infer_expr_type_for_conv(&self, expr: &Expr, runtime_type: ObjectType) -> ConvSrcType {
        match expr {
            Expr::I64(_) => ConvSrcType::I64,
            Expr::U64(_) => ConvSrcType::U64,
            Expr::Float(_, _) => ConvSrcType::F32,
            Expr::Double(_, _) => ConvSrcType::F64,
            Expr::Bool(_) => ConvSrcType::Bool,
            Expr::Str(_) | Expr::FStr(_) => ConvSrcType::Str,
            Expr::Ident(name) => {
                if let Some(ty) = self.var_types.get(name.as_str()) {
                    match ty {
                        Type::I64 => ConvSrcType::I64,
                        Type::U64 => ConvSrcType::U64,
                        Type::Float => ConvSrcType::F32,
                        Type::Double => ConvSrcType::F64,
                        Type::Bool => ConvSrcType::Bool,
                        Type::Str(_) | Type::String | Type::StrSlice => ConvSrcType::Str,
                        _ => ConvSrcType::I32,
                    }
                } else {
                    match runtime_type {
                        ObjectType::Float => ConvSrcType::F32,
                        ObjectType::Double => ConvSrcType::F64,
                        ObjectType::Uint => ConvSrcType::U64,
                        ObjectType::Bool => ConvSrcType::Bool,
                        ObjectType::String => ConvSrcType::Str,
                        _ => ConvSrcType::I32,
                    }
                }
            }
            Expr::Dot(_, _) => {
                match runtime_type {
                    ObjectType::Float => ConvSrcType::F32,
                    ObjectType::Double => ConvSrcType::F64,
                    ObjectType::Uint => ConvSrcType::U64,
                    ObjectType::Bool => ConvSrcType::Bool,
                    ObjectType::String => ConvSrcType::Str,
                    _ => ConvSrcType::I32,
                }
            }
            _ => {
                match runtime_type {
                    ObjectType::Float => ConvSrcType::F32,
                    ObjectType::Double => ConvSrcType::F64,
                    ObjectType::Uint => ConvSrcType::U64,
                    ObjectType::Bool => ConvSrcType::Bool,
                    ObjectType::String => ConvSrcType::Str,
                    _ => ConvSrcType::I32,
                }
            }
        }
    }

    /// Falls back to a heuristic based on common naming patterns
    ///
    /// For standard library types, we can map variable names to types:
    /// - "list", "arr" -> "List"
    /// - "str", "s" -> "String"
    /// Build fn_return_types from the native registry + intrinsic extras.
    ///
    /// The native registry (`BIGVM_NATIVES`) carries return types for registered
    /// native functions.  We import those here and add a handful of intrinsics
    /// that don't go through the registry (e.g. `int.to_str`, `List.join`).
    fn build_fn_return_types() -> HashMap<String, Type> {
        use crate::vm::native_registry::NativeRetType;

        let mut map = HashMap::new();

        // 1. Bulk-import from the native registry
        let registry = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
        for (name, ret) in registry.get_all_return_types() {
            let ty = match ret {
                NativeRetType::Void => Type::Void,
                NativeRetType::Int => Type::Int,
                NativeRetType::Float => Type::Float,
                NativeRetType::Bool => Type::Bool,
                NativeRetType::String => Type::String,
                NativeRetType::I64 => Type::I64,
                NativeRetType::List => Type::Unknown, // list types need more context
            };
            map.insert(name.clone(), ty);
        }
        drop(registry);

        // 2. Intrinsics that aren't in the registry but codegen needs type info for
        map.insert("int.to_str".to_string(), Type::String);
        map.insert("int_str".to_string(), Type::String);
        map.insert("uint.to_hex".to_string(), Type::String);
        map.insert("List.join".to_string(), Type::String);

        map
    }

    /// - "map", "dict" -> "Map"
    /// This is a fallback for when type information is not available
    fn infer_type_from_var(&self, var_name: &str) -> Option<String> {
        // Plan 080: First check if we have explicit type information from var_types
        if let Some(ty) = self.var_types.get(var_name) {
            // Return the base type name (without generic parameters for now)
            match ty {
                Type::Int | Type::I64 => Some("int".to_string()),
                Type::Uint | Type::U64 | Type::Byte | Type::USize => Some("uint".to_string()),
                Type::Float | Type::Double => Some("float".to_string()),
                Type::Bool => Some("bool".to_string()),
                Type::Char => Some("char".to_string()),
                Type::Str(_) | Type::String | Type::StrSlice => Some("str".to_string()),
                Type::CStr => Some("str".to_string()),
                Type::Array(_) => Some("Array".to_string()),
                Type::List(_) => Some("List".to_string()),
                Type::Map(_, _) => Some("Map".to_string()),  // Plan 160
                Type::User(type_decl) => Some(type_decl.name.to_string()),
                Type::GenericInstance(inst) => Some(inst.base_name.to_string()),
                _ => None,
            }
        } else {
            // Fallback: heuristic based on variable naming
            match var_name {
                "list" | "arr" | "array" | "vec" => Some("List".to_string()),
                "str" | "string" => Some("str".to_string()),
                "map" | "dict" | "hashmap" => Some("HashMap".to_string()),
                "set" => Some("HashSet".to_string()),
                "opt" | "option" => Some("Option".to_string()),
                "file" => Some("File".to_string()),
                "deque" => Some("VecDeque".to_string()),
                "bmap" | "treemap" => Some("BTreeMap".to_string()),
                _ => None,
            }
        }
    }

    /// Check if a method requires extracting the 'id' field from the instance
    ///
    /// Plan 077 Phase 5: With unified heap registry, heap objects (List, HashMap, etc.)
    /// are now referenced directly by their ID (u64) instead of being wrapped in
    /// Value::Instance with an 'id' field. So we NO LONGER need to extract 'id' for these types.
    ///
    /// This function now only returns true for legacy types that still use Value::Instance.
    ///
    /// Legacy examples (when this returns true):
    /// - `Iterator.next` → extract `id` field (iterators still use old format)
    ///
    /// Examples (now returns false, no extraction needed):
    /// - `List.push` → NO extraction needed, use list_id directly
    /// - `List.len` → NO extraction needed, use list_id directly
    /// - `List.iter` → NO extraction needed, use list_id directly
    fn needs_id_extraction(&self, method_name: &str) -> bool {
        // Plan 077 Phase 5: With unified heap registry, List/HashMap/HashSet don't need id extraction
        // They are now stored as heap objects with direct IDs

        // Only iterators still use the old Value::Instance format with id field
        if method_name.starts_with("Iterator.") {
            return matches!(
                method_name,
                "Iterator.next"
                    | "Iterator.map"
                    | "Iterator.filter"
                    | "Iterator.collect"
                    | "Iterator.reduce"
                    | "Iterator.find"
            );
        }

        // All other types now use direct heap object IDs - no extraction needed
        false
    }

    // ========== Plan 073: Type Registry Helper Methods ==========

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
                vm_debug!("DEBUG: Registered non-generic type '{}' in generic_registry",
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

        // Register in GenericRegistry
        if let Err(e) = self.generic_registry.register_template(template) {
            eprintln!(
                "Warning: Failed to register generic template '{}': {}",
                type_decl.name, e
            );
        }
    }

    /// Set a new inference context (Plan 089)
    ///
    /// Used to transfer type registry from Parser to Codegen.
    /// This ensures types registered during parsing are available for field lookup.
    pub fn set_infer_ctx(&mut self, infer_ctx: InferenceContext) {
        self.infer_ctx = infer_ctx;
    }

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Branch, Expr, If, Stmt};
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
        // JMP_IF_Z at 5 should jump to 16. Offset 8.
        assert_eq!(code[5], OpCode::JMP_IF_Z as u8);
        let else_offset = i16::from_le_bytes(code[6..8].try_into().unwrap());
        assert_eq!(else_offset, 8);

        // JMP at 13 should jump to 21. Offset 5.
        // Wait, why 5?
        // 13 (JMP) + 1 + 2 = 16.
        // End is at 21. 21 - 16 = 5. Correct.
        assert_eq!(code[13], OpCode::JMP as u8);
        let end_offset = i16::from_le_bytes(code[14..16].try_into().unwrap());
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
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
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
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
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
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut task_def = TaskDef::new("CounterTask".into(), vec![], pos);
        task_def.add_state("count".into(), true, Expr::Int(0));

        let stmt = Stmt::TaskDef(task_def);
        let result = codegen.compile_stmt(&stmt);

        // Should compile successfully
        assert!(result.is_ok());

        // Task should be registered
        assert!(codegen.types.contains_key("CounterTask"));
    }
}
