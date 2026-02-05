use crate::ast::{Expr, Stmt, Closure, Iter, TypeDecl};
use crate::error::{AutoResult, AutoError};
use crate::error::SyntaxError;
// use crate::val::Value; // Removed if not directly used or fix path
use crate::vm::loader::{Module, RelocEntry, RelocType};
use crate::vm::native::{NATIVE_PRINT_F32, NATIVE_PRINT_I32, NATIVE_PRINT_STR};
use crate::vm::native_registry::BIGVM_NATIVES;
use crate::vm::opcode::OpCode;
// Plan 076 Phase 1: Generic type support
use crate::vm::generic::{GenericTable, extract_generic_instance};
// Plan 076 Phase 2: Monomorphization support
use crate::vm::monomorphize::{Monomorphizer, MonomorphizedModule};
use auto_val::Op;
use std::collections::{HashMap, HashSet};
use miette::SourceSpan;

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
    // Plan 073: Nested types for object/array fields
    NestedObject,
    Array,
}

/// Plan 073: Type information for TypeDecl
/// Stores type metadata needed for instance construction
#[derive(Debug, Clone)]
struct TypeInfo {
    pub name: String,
    pub member_names: Vec<String>,  // Member names in order
}

/// Codegen: Compiles AST directly to BigVM Bytecode
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

    /// Captured variables stack for nested closures (Plan 071 Phase 6.2)
    /// Each level has its own captured variable map (name -> capture index)
    /// Stack allows proper nesting: inner closures can capture from outer closures
    pub captured_vars_stack: Vec<HashMap<String, usize>>,

    /// Plan 073: Loop exit tracking for break/continue statements
    /// Each nested loop has a Vec of jump placeholders that need to be patched
    /// when the loop exits
    pub loop_exits: Vec<Vec<usize>>,

    /// Plan 073: Type registry for TypeDecl support
    /// Maps type name -> TypeInfo (member names, etc.)
    pub types: HashMap<String, TypeInfo>,

    /// Plan 076 Phase 1: Generic instantiation table
    /// Tracks all generic type instantiations (e.g., List<int>, List<string>)
    pub generics: GenericTable,
}

impl Codegen {
    pub fn new() -> Self {
        // Initialize the global native registry
        crate::vm::native_registry::register_builtin_natives();

        let mut intrinsics = HashMap::new();
        // Register intrinsics
        intrinsics.insert("print".to_string(), NATIVE_PRINT_I32);
        intrinsics.insert("print_i32".to_string(), NATIVE_PRINT_I32);
        intrinsics.insert("print_f32".to_string(), NATIVE_PRINT_F32);
        intrinsics.insert("print_str".to_string(), NATIVE_PRINT_STR);

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
            captured_vars_stack: Vec::new(),
            loop_exits: Vec::new(),
            types: HashMap::new(),
            generics: GenericTable::new(), // Plan 076 Phase 1
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
        self.generics.list_instantiations().into_iter().cloned().collect()
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
                // Statement referencing an expression usually pops the result?
                // In stack machine, if expr pushes value, we might want to pop it if it's a stmt?
                // For now, let's assume expressions are side-effect only or return value is ignored.
                // But wait, `if` stmt logic might depend on this.
                // Standard: ExprStmt usually implies "Evaluate and Discard result".
                // We'll add POP if needed later. For now, just compile.
            }
            Stmt::Block(body) => {
                // Enter new scope? (Locals not implemented yet in this phase)
                for s in &body.stmts {
                    self.compile_stmt(s)?;
                }
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
                // 1. Jump over function body (so it's not executed during definition flow)
                self.emit(OpCode::JMP);
                let jump_over = self.emit_placeholder_i16();

                // 2. Record function entry point (export)
                // Entry point is HERE (after JMP instruction)
                let entry_point = self.code.len() as u32;
                self.exports.insert(fn_decl.name.to_string(), entry_point);

                // 3. Push new scope for function locals
                self.push_scope();

                // 4. Compile body
                self.compile_stmt(&Stmt::Block(fn_decl.body.clone()))?;

                // 5. Get number of locals and emit stack reservation at function entry
                let n_locals = self.scope_stack.last().unwrap().len();

                // Emit stack reservation at FUNCTION START (right after entry point)
                // This ensures sp starts at n_locals, preventing stack from overwriting locals
                if n_locals > 0 {
                    // Insert CONST_0 opcodes at entry_point to reserve stack space
                    // Each CONST_0 is 5 bytes (1 byte opcode + 4 bytes i32)
                    for _ in 0..n_locals {
                        self.code.insert(entry_point as usize, OpCode::CONST_0 as u8);
                        self.code.insert(entry_point as usize + 1, 0u8);
                        self.code.insert(entry_point as usize + 2, 0u8);
                        self.code.insert(entry_point as usize + 3, 0u8);
                        self.code.insert(entry_point as usize + 4, 0u8);
                    }
                }

                // 6. Emit RET at end of body
                let n_args = fn_decl.params.len() as u8;
                self.emit(OpCode::RET);
                self.code.push(n_args);

                // 7. Pop function scope
                self.pop_scope();

                // 8. Patch jump to skip body
                self.patch_jump(jump_over);
            }
            Stmt::Store(store) => {
                // Variable declaration: let/mut/var name = expr
                // Compile the RHS expression (pushes result on stack)
                self.compile_expr(&store.expr)?;

                // Add variable to symbol table and get its index
                let var_index = self.add_var(&store.name);

                // Store the value into the local variable
                self.emit_store_loc(var_index);
            }
            Stmt::Return(expr) => {
                // Compile expression to leave result on stack
                self.compile_expr(expr)?;
                // FIXME: We need to know `n_args` here to emit correct RET.
                // Codegen struct doesn't track "current function context" yet.
                // TODO: Add `current_fn_args_count` to Codegen state.
                // For now, hardcode 0 or implement context tracking.
                // This is a limitation. I will mark TODO.
                // Assuming 0 for now might break things if used inside args func.
                // WORKAROUND: For this iteration, I'll allow simple returns, but `RET` instruction REQUIRES n_args.
                // I'll emit RET 0 and file a task to fix context.
                self.emit(OpCode::RET);
                self.code.push(0); // TODO: Fix this
            }
            // Plan 073: TypeDecl support - register type metadata
            Stmt::TypeDecl(type_decl) => {
                // Register the type in the type registry
                self.register_type(type_decl);
                // Type declarations don't generate any bytecode at compile time
                // They just register metadata for use in instance construction
            }
            Stmt::EnumDecl(enum_decl) => {
                // Plan 073 Phase 8.6: Enum declaration support
                // Enum declarations don't generate bytecode at compile time
                // They register metadata for use in pattern matching and type checking
                // TODO: Register enum in type registry for future use
                // For now, enums are handled by the Tag system (Plan 073 Phase 8.3.7)
            }
            Stmt::SpecDecl(spec_decl) => {
                // Plan 073 Phase 8.6: Spec declaration support
                // Spec declarations (traits) don't generate bytecode at compile time
                // They register method signatures for type checking and constraint validation
                // TODO: Register spec in type registry for future use
                // For now, specs are metadata-only and used during type checking
            }
            // Plan 073: For statement support
            Stmt::For(for_stmt) => {
                // Push new loop exit tracking
                self.loop_exits.push(Vec::new());

                // Handle range-based for loops: for x in start..end { ... }
                // Only support simple range iteration for now
                match &for_stmt.iter {
                    Iter::Named(var_name) => {
                        // Check if range is a Range expression (for x in 0..10)
                        if let Expr::Range(range) = &for_stmt.range {
                            // Compile start expression and initialize loop variable
                            self.compile_expr(&range.start)?;

                            // Store to loop variable
                            let var_str = var_name.to_string();
                            self.push_scope(); // New scope for loop variable
                            let var_index = self.scope_stack.last_mut().unwrap().len();
                            self.scope_stack.last_mut().unwrap().insert(var_str.clone(), var_index);
                            self.emit_store_loc(var_index);

                            // Loop start label
                            let loop_start = self.code.len() as i16;

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

                            // Increment loop variable
                            self.emit_load_loc(var_index);
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(1);
                            self.emit(OpCode::ADD);
                            self.emit_store_loc(var_index);

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len() as i16;
                            self.emit_i16(loop_start - current_pos);

                            // This is the loop exit point - patch all break jumps here
                            let loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                        } else if let Expr::Call(call) = &for_stmt.range {
                            // Plan 073: Iterator-based for loop: for x in list.iter() { ... }
                            // Compile the iterator call to get the iterator object
                            self.compile_expr(&for_stmt.range)?;

                            // Store iterator in a local variable
                            self.push_scope(); // New scope for loop variable and iterator
                            let iter_index = self.scope_stack.last_mut().unwrap().len();
                            self.scope_stack.last_mut().unwrap().insert("_iterator".to_string(), iter_index);
                            self.emit_store_loc(iter_index);

                            // Loop start label
                            let loop_start = self.code.len() as i16;

                            // Call iter.next() to get next element
                            self.emit_load_loc(iter_index); // Load iterator

                            // Emit CALL_NAT for Iterator.next
                            // Look up the native function ID
                            let native_id = if let Some(id) = BIGVM_NATIVES.lock().unwrap().get_id("Iterator.next") {
                                id
                            } else {
                                self.loop_exits.pop();
                                return Err(AutoError::Msg("Iterator.next native function not found".to_string()));
                            };
                            self.emit(OpCode::CALL_NAT);
                            self.code.extend_from_slice(&native_id.to_le_bytes());

                            // Check if result is nil (end of iteration)
                            // Nil is represented as -1 in our VM
                            self.emit(OpCode::CONST_I32);
                            self.emit_i32(-1);
                            self.emit(OpCode::EQ);
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_end = self.emit_placeholder_i16();

                            // Store the element to the loop variable
                            let var_str = var_name.to_string();
                            let var_index = self.scope_stack.last_mut().unwrap().len();
                            self.scope_stack.last_mut().unwrap().insert(var_str.clone(), var_index);
                            self.emit_store_loc(var_index);

                            // Compile loop body
                            self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                            // JMP back to loop start
                            self.emit(OpCode::JMP);
                            let current_pos = self.code.len() as i16;
                            self.emit_i16(loop_start - current_pos);

                            // This is the loop exit point - patch all break jumps here
                            let loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                        } else {
                            // For now, only support range and iterator expressions
                            self.loop_exits.pop();
                            return Err(AutoError::Msg("For loops with non-range/non-iterator expressions not supported yet".to_string()));
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
                            let index_index = self.scope_stack.last_mut().unwrap().len();
                            self.scope_stack.last_mut().unwrap().insert(index_str.clone(), index_index);
                            self.emit_store_loc(index_index);

                            // Store same value to iter variable
                            let iter_index = self.scope_stack.last_mut().unwrap().len();
                            self.scope_stack.last_mut().unwrap().insert(iter_str.clone(), iter_index);
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

                            // Increment both loop variables
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
                            self.emit_i16(loop_start - current_pos);

                            // This is the loop exit point - patch all break jumps here
                            let loop_exit = self.code.len();

                            // Patch exit jump (for loop condition)
                            self.patch_jump(jump_to_end);

                            // Pop loop scope
                            self.pop_scope();

                            // Patch all break statements
                            let exits = self.loop_exits.pop().unwrap();
                            for exit_placeholder in exits {
                                self.patch_jump(exit_placeholder);
                            }
                        } else {
                            // For now, only support range expressions
                            self.loop_exits.pop();
                            return Err(AutoError::Msg("Indexed for loops with non-range expressions not supported yet".to_string()));
                        }
                    }
                    Iter::Cond => {
                        // Conditional for loop: for condition { ... } (like while)
                        // Loop start label
                        let loop_start = self.code.len() as i16;

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
                        self.emit_i16(loop_start - current_pos);

                        // This is the loop exit point - patch all break jumps here
                        let loop_exit = self.code.len();

                        // Patch exit jump (for loop condition)
                        self.patch_jump(jump_to_end);

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                    }
                    Iter::Ever => {
                        // Infinite loop: for ever { ... }
                        let loop_start = self.code.len() as i16;

                        // Compile loop body
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len() as i16;
                        self.emit_i16(loop_start - current_pos);

                        // This is the loop exit point - patch all break jumps here
                        let loop_exit = self.code.len();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
                    }
                    Iter::Call(call) => {
                        // Plan 073: Iterator-based for loop: for x in list.iter() { ... }
                        // Compile the iterator call to get the iterator object
                        self.compile_expr(&Expr::Call(call.clone()))?;

                        // Store iterator in a local variable
                        self.push_scope(); // New scope for loop variable and iterator
                        let iter_index = self.scope_stack.last_mut().unwrap().len();
                        self.scope_stack.last_mut().unwrap().insert("_iterator".to_string(), iter_index);
                        self.emit_store_loc(iter_index);

                        // Loop start label
                        let loop_start = self.code.len() as i16;

                        // Call iter.next() to get next element
                        self.emit_load_loc(iter_index); // Load iterator

                        // Call next() method - this is a method call on the iterator
                        // For BigVM, we need to call this as a native function
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
                        let var_index = self.scope_stack.last_mut().unwrap().len();
                        self.scope_stack.last_mut().unwrap().insert(var_str.to_string(), var_index);
                        self.emit_store_loc(var_index);

                        // Compile loop body
                        self.compile_stmt(&Stmt::Block(for_stmt.body.clone()))?;

                        // JMP back to loop start
                        self.emit(OpCode::JMP);
                        let current_pos = self.code.len() as i16;
                        self.emit_i16(loop_start - current_pos);

                        // This is the loop exit point - patch all break jumps here
                        let loop_exit = self.code.len();

                        // Patch exit jump (for loop condition)
                        self.patch_jump(jump_to_end);

                        // Pop loop scope
                        self.pop_scope();

                        // Patch all break statements
                        let exits = self.loop_exits.pop().unwrap();
                        for exit_placeholder in exits {
                            self.patch_jump(exit_placeholder);
                        }
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
                    return Err(AutoError::Msg("Break statement outside of loop".to_string()));
                }
            }
            // Plan 073: Is pattern matching statement
            Stmt::Is(is_stmt) => {
                // Evaluate target expression once and keep on stack
                self.compile_expr(&is_stmt.target)?;

                let mut branch_jumps = Vec::new();

                // Process each branch
                for branch in &is_stmt.branches {
                    match branch {
                        crate::ast::IsBranch::EqBranch(pattern, body) => {
                            // Duplicate target for comparison
                            self.emit(OpCode::DUP);

                            // Evaluate pattern expression
                            self.compile_expr(pattern)?;

                            // Compare target with pattern
                            self.emit(OpCode::EQ);

                            // Jump to next branch if not equal
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();
                            branch_jumps.push(jump_to_next);

                            // Compile branch body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end of is statement
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            branch_jumps.push(jump_to_end);

                            // Patch jump to next branch
                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::IfBranch(condition, body) => {
                            // Plan 073: Evaluate condition expression
                            self.compile_expr(condition)?;

                            // Jump to next branch if condition is false (zero)
                            self.emit(OpCode::JMP_IF_Z);
                            let jump_to_next = self.emit_placeholder_i16();
                            branch_jumps.push(jump_to_next);

                            // Compile branch body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end of is statement
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            branch_jumps.push(jump_to_end);

                            // Patch jump to next branch
                            self.patch_jump(jump_to_next);
                        }
                        crate::ast::IsBranch::ElseBranch(body) => {
                            // This is the default case - just compile body
                            self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;

                            // Jump to end (in case there are more branches after else)
                            self.emit(OpCode::JMP);
                            let jump_to_end = self.emit_placeholder_i16();
                            branch_jumps.push(jump_to_end);
                        }
                    }
                }

                // Pop the target value from stack
                self.emit(OpCode::POP);

                // Patch all jump_to_end placeholders
                for jump_to_end in branch_jumps {
                    self.patch_jump(jump_to_end);
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
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*i);
            }
            Expr::Bool(b) => {
                self.emit(OpCode::CONST_I32);
                self.emit_i32(if *b { 1 } else { 0 });
            }
            // Plan 073 Stage A.5: Float literal support
            Expr::Float(f, _) => {
                self.emit(OpCode::CONST_F32);
                self.emit_f32(*f as f32);
            }
            // Plan 073 Stage A.5: Double literal support
            Expr::Double(d, _) => {
                self.emit(OpCode::CONST_F64);
                self.emit_f64(*d);
            }
            // Plan 073 Stage B: I64 literal support
            Expr::I64(i) => {
                self.emit(OpCode::CONST_I64);
                self.emit_i64(*i);
            }
            // Plan 073 Stage B: U64 literal support
            Expr::U64(u) => {
                self.emit(OpCode::CONST_U64);
                self.emit_u64(*u);
            }
            // Plan 073 Stage B: Uint literal support (use CONST_I32)
            Expr::Uint(u) => {
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*u as i32);
            }
            // Plan 073 Stage B: I8 literal support (use CONST_I32)
            Expr::I8(i) => {
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*i as i32);
            }
            // Plan 073 Stage B: U8 literal support (use CONST_I32)
            Expr::U8(u) => {
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*u as i32);
            }
            // Plan 073 Stage B: Byte literal support (use CONST_I32)
            Expr::Byte(b) => {
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*b as i32);
            }
            // Plan 073 Stage B: Char literal support (use CONST_I32 for UTF-32 codepoint)
            Expr::Char(c) => {
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
                let keys: Vec<auto_val::ValueKey> = pairs.iter().map(|pair| {
                    self.ast_key_to_value_key(&pair.key)
                }).collect();
                let key_index = self.object_keys.len() as u16;

                // Plan 073: Track field types for runtime conversion
                let types: Vec<ObjectType> = pairs.iter().map(|pair| {
                    self.infer_object_type(&pair.value)
                }).collect();
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
                // Compile each part expression (pushes values onto stack)
                for part in &fstr.parts {
                    self.compile_expr(part)?;
                }

                // Emit BUILD_FSTR with part count
                let part_count = fstr.parts.len() as u8;
                self.emit(OpCode::BUILD_FSTR);
                self.code.push(part_count);
            }
            // Plan 073: Node support (for type instances like Point(10, 20))
            Expr::Node(node) => {
                // Check if this is a type instance
                let type_name = node.name.to_string();

                if let Some(type_info) = self.get_type(&type_name) {
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
                            crate::ast::Arg::Pair(key, expr) => {
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
                        .take(arg_count as usize)  // Only take as many as we have args
                        .map(|name| auto_val::ValueKey::Str(name.clone().into()))
                        .collect();

                    // Register keys in object_keys pool
                    let key_index = self.object_keys.len() as u16;
                    self.object_keys.push(keys);

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
                            crate::ast::Arg::Pair(key, expr) => {
                                // For named args, compile the value
                                self.compile_expr(expr)?;
                            }
                            crate::ast::Arg::Name(_) => {
                                // Name-only arg (placeholder for future)
                            }
                        }
                    }

                    // Emit CREATE_NODE with name index and arg count
                    self.emit(OpCode::CREATE_NODE);
                    self.code.extend_from_slice(&name_idx.to_le_bytes());
                    self.code.push(arg_count);
                }
            }
            Expr::Str(s) => {
                // Add string to constant pool and emit LOAD_STR <index>
                let bytes = s.as_bytes().to_vec();
                let idx = self.strings.len() as u16;
                self.strings.push(bytes);
                self.emit(OpCode::LOAD_STR);
                self.code.extend_from_slice(&idx.to_le_bytes());
            }
            Expr::Ident(name) => {
                let name_str = name.to_string();

                // Check if this is a captured variable (Plan 071)
                if let Some(_capture_index) = self.current_captured_vars().get(&name_str) {
                    // Variable is captured - emit LOAD_CAPTURED
                    self.emit_load_captured(&name_str);
                } else if let Some(var_index) = self.lookup_var(&name_str) {
                    // Variable found in local scope - load it
                    self.emit_load_loc(var_index);
                } else {
                    // Variable not found - this is an error
                    // For now, emit LOAD_LOC_0 as a fallback (will be fixed later)
                    // TODO: Proper error handling for undefined variables
                    self.emit(OpCode::LOAD_LOC_0);
                }
            }
            // Plan 073: Dot expression field access (obj.field)
            Expr::Dot(obj, field) => {
                // Compile object expression (should push object_id onto stack)
                self.compile_expr(obj)?;

                // Add field name to string pool and emit GET_FIELD <field_idx>
                let field_str = field.to_string();
                let field_bytes = field_str.as_bytes().to_vec();
                let field_idx = self.strings.len() as u16;
                self.strings.push(field_bytes);

                self.emit(OpCode::GET_FIELD);
                self.code.extend_from_slice(&field_idx.to_le_bytes());
            }
            // Plan 073: Array indexing (arr[index])
            Expr::Index(arr, idx) => {
                // Compile array expression (should push array_id onto stack)
                self.compile_expr(arr)?;
                // Compile index expression (should push index onto stack)
                self.compile_expr(idx)?;
                // Emit GET_ELEM (pops array_id and index, pushes element value)
                self.emit(OpCode::GET_ELEM);
            }
            Expr::Bina(lhs, op, rhs) => {
                // Assignment is special: compile RHS first, then store to LHS
                if *op == Op::Asn {
                    // Compile RHS (value to store)
                    self.compile_expr(rhs)?;

                    // Check if LHS is an identifier (variable assignment)
                    if let Expr::Ident(name) = lhs.as_ref() {
                        let name_str = name.to_string();

                        // Check if this is a captured variable (Plan 071)
                        if self.current_captured_vars().contains_key(&name_str) {
                            // Variable is captured - emit STORE_CAPTURED
                            self.emit_store_captured(&name_str);
                        } else if let Some(var_index) = self.lookup_var(&name_str) {
                            // Variable found in local scope - store value to it
                            self.emit_store_loc(var_index);
                        } else {
                            // Variable not found - this is an error
                            // For now, emit STORE_LOC_0 as a fallback
                            // TODO: Proper error handling for undefined variables
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
                        self.emit(OpCode::SET_ELEM);  // Expects: value, array_id, index
                    } else if let Expr::Dot(obj, field) = lhs.as_ref() {
                        // Plan 075: Field assignment: obj.field = value
                        // Stack has: value (from RHS compilation above)
                        // Compile object expression
                        self.compile_expr(obj)?;
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
                    } else {
                        unimplemented!("Assignment to complex LHS not supported yet");
                    }
                } else {
                    // Plan 073 Stage A.5: Check if this is a float/double operation
                    let is_float = self.is_float_operation(lhs, rhs);
                    let is_double = self.is_double_operation(lhs, rhs);

                    // Normal binary operation: compile both operands, then apply operator
                    self.compile_expr(lhs)?;
                    self.compile_expr(rhs)?;

                    // For arithmetic operations, use float/double opcodes if operands are floats
                    match op {
                        Op::Add => {
                            if is_double {
                                self.emit(OpCode::ADD_D);
                            } else if is_float {
                                self.emit(OpCode::ADD_F);
                            } else {
                                self.emit(OpCode::ADD);
                            }
                        }
                        Op::Sub => {
                            if is_double {
                                self.emit(OpCode::SUB_D);
                            } else if is_float {
                                self.emit(OpCode::SUB_F);
                            } else {
                                self.emit(OpCode::SUB);
                            }
                        }
                        Op::Mul => {
                            if is_double {
                                self.emit(OpCode::MUL_D);
                            } else if is_float {
                                self.emit(OpCode::MUL_F);
                            } else {
                                self.emit(OpCode::MUL);
                            }
                        }
                        Op::Div => {
                            if is_double {
                                self.emit(OpCode::DIV_D);
                            } else if is_float {
                                self.emit(OpCode::DIV_F);
                            } else {
                                self.emit(OpCode::DIV);
                            }
                        }
                        // Comparison operators currently use integer opcodes for all types
                        // TODO: Add float/double comparison opcodes if needed
                        Op::Eq => self.emit(OpCode::EQ),
                        Op::Neq => self.emit(OpCode::NE),
                        Op::Lt => self.emit(OpCode::LT),
                        Op::Le => self.emit(OpCode::LE),
                        Op::Gt => self.emit(OpCode::GT),
                        Op::Ge => self.emit(OpCode::GE),
                        _ => unimplemented!("Binary Op {:?}", op),
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
                // Extract function name and determine if it's a method call
                // Plan 073: Support both static methods (Type.method) and instance methods (obj.method)
                let mut func_name = match call.name.as_ref() {
                    Expr::Ident(name) => Some(name.to_string()),
                    Expr::Dot(obj, method) => {
                        // Method call: Type.method (static) or obj.method (instance)
                        match obj.as_ref() {
                            Expr::Ident(obj_name) => {
                                // Check if it's a static method call (Type.method with capital T)
                                if self.is_type_name_heuristic(obj_name) || self.is_type(obj_name) {
                                    // Static method call: Type.method
                                    Some(format!("{}.{}", obj_name, method))
                                } else {
                                    // Instance method call: obj.method
                                    // Infer type from variable name and generate: TypeName.method
                                    if let Some(type_name) = self.infer_type_from_var(obj_name) {
                                        Some(format!("{}.{}", type_name, method))
                                    } else {
                                        // Fallback: generate obj.method (may fail at link time)
                                        Some(format!("{}.{}", obj_name, method))
                                    }
                                }
                            }
                            _ => {
                                // Complex expression (e.g., arr[0].push, foo().method)
                                // We cannot determine the type at compile time without type inference
                                // For now, generate a generic name that may fail at link time
                                Some(format!("Unknown_{}", method))
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

                if let Some(id) = native_id {
                    // Native function call
                    // For instance methods, compile receiver (self) FIRST, then arguments
                    // This ensures stack order: [self, arg1, arg2, ...]
                    if let Expr::Dot(obj, _method) = call.name.as_ref() {
                        // Check if it's a static method call (Type.method with capital T)
                        let is_static_method = match obj.as_ref() {
                            Expr::Ident(obj_name) => {
                                self.is_type_name_heuristic(obj_name) || self.is_type(obj_name)
                            }
                            _ => false,
                        };

                        // Compile receiver for instance methods
                        if !is_static_method {
                            // Check if this method needs 'id' field extraction
                            if let Some(ref method_name) = func_name {
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
                                    self.compile_expr(obj)?;
                                }
                            } else {
                                self.compile_expr(obj)?;
                            }
                        }
                    }

                    // Compile arguments (left-to-right)
                    if !call.args.is_empty() {
                        for arg in &call.args.args {
                            match arg {
                                crate::ast::Arg::Pos(expr) => {
                                    self.compile_expr(expr)?;
                                }
                                _ => {
                                    unimplemented!("Named arguments not supported in BigVM yet")
                                }
                            }
                        }
                    }

                    self.emit(OpCode::CALL_NAT);
                    self.code.extend_from_slice(&id.to_le_bytes());
                    return Ok(()).into();
                }

                // Normal Function Call (user-defined)
                // Plan 073: For instance methods, compile receiver (self) FIRST, then arguments
                if let Expr::Dot(obj, _method) = call.name.as_ref() {
                    // Check if it's a static method call (Type.method with capital T)
                    let is_static_method = match obj.as_ref() {
                        Expr::Ident(obj_name) => {
                            self.is_type_name_heuristic(obj_name) || self.is_type(obj_name)
                        }
                        _ => false,
                    };

                    // Compile receiver for instance methods
                    if !is_static_method {
                        // Check if this method needs 'id' field extraction
                        if let Some(ref method_name) = func_name {
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
                                self.compile_expr(obj)?;
                            }
                        } else {
                            self.compile_expr(obj)?;
                        }
                    }
                }

                // Compile Arguments (pushes them to stack)
                if !call.args.is_empty() {
                    for arg in &call.args.args {
                        match arg {
                            crate::ast::Arg::Pos(expr) => {
                                self.compile_expr(expr)?;
                            }
                            _ => unimplemented!("Named arguments not supported in BigVM yet"),
                        }
                    }
                }

                // 2. Emit CALL opcode
                self.emit(OpCode::CALL);

                // 3. Emit Placeholder for Address (u32)
                let placeholder_idx = self.code.len();
                self.code.extend_from_slice(&0u32.to_le_bytes());

                // 4. Create Relocation Entry
                let reloc_name = func_name.unwrap_or_else(|| {
                    match call.name.as_ref() {
                        Expr::Ident(name) => name.to_string(),
                        _ => unimplemented!("Dynamic call (computed function name) not supported yet"),
                    }
                });

                self.relocs.push(RelocEntry {
                    offset: placeholder_idx as u32,
                    symbol_name: reloc_name,
                    reloc_type: RelocType::FuncCall,
                });
            }
            Expr::If(if_expr) => {
                // If expression: each branch must leave a value on the stack
                let mut jumps_to_end = Vec::new();

                for branch in &if_expr.branches {
                    // Compile condition
                    self.compile_expr(&branch.cond)?;

                    // JMP_IF_Z to next branch
                    self.emit(OpCode::JMP_IF_Z);
                    let jump_to_next = self.emit_placeholder_i16();

                    // Compile body (should push result)
                    // Body is a Block, compile all statements
                    for stmt in &branch.body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                    // The last expression in the block should be left on stack
                    // For simplicity, we assume the last statement leaves a value

                    // Jump to end
                    self.emit(OpCode::JMP);
                    let jump_to_end = self.emit_placeholder_i16();
                    jumps_to_end.push(jump_to_end);

                    // Patch jump to next branch
                    self.patch_jump(jump_to_next);
                }

                // Else branch (if any)
                if let Some(else_body) = &if_expr.else_ {
                    for stmt in &else_body.stmts {
                        self.compile_stmt(stmt)?;
                    }
                }

                // Patch all jumps to end
                for jump in jumps_to_end {
                    self.patch_jump(jump);
                }
            }
            Expr::Closure(closure) => {
                // Plan 071: Compile closure with captured environment
                self.compile_closure(closure)?;
            }
            Expr::View(inner) | Expr::Mut(inner) | Expr::Take(inner) => {
                // Plan 060: Ownership operators (.view, .mut, .take)
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
        }
    }

    // === Helpers ===

    fn emit(&mut self, op: OpCode) {
        self.code.push(op as u8);
    }

    fn emit_i32(&mut self, val: i32) {
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
            crate::ast::Key::NamedKey(name) => {
                auto_val::ValueKey::Str(name.to_string().into())
            }
            crate::ast::Key::IntKey(i) => {
                auto_val::ValueKey::Int(*i)
            }
            crate::ast::Key::BoolKey(b) => {
                auto_val::ValueKey::Bool(*b)
            }
            crate::ast::Key::StrKey(s) => {
                auto_val::ValueKey::Str(s.clone())
            }
        }
    }

    // Plan 073 Stage A.5: Check if expression is a float/double type
    // Returns: Some(Type) if the type is known, None otherwise
    fn infer_expr_type(&self, expr: &Expr) -> Option<crate::ast::Type> {
        match expr {
            // Literals with known types
            Expr::Float(_, _) => Some(crate::ast::Type::Float),
            Expr::Double(_, _) => Some(crate::ast::Type::Double),
            Expr::Int(_) => Some(crate::ast::Type::Int),
            Expr::I64(_) => Some(crate::ast::Type::I64),
            Expr::U64(_) => Some(crate::ast::Type::U64),
            Expr::Uint(_) => Some(crate::ast::Type::Uint),
            Expr::I8(_) => Some(crate::ast::Type::Int),
            Expr::U8(_) => Some(crate::ast::Type::Uint),
            Expr::Byte(_) => Some(crate::ast::Type::Byte),
            Expr::Char(_) => Some(crate::ast::Type::Char),
            Expr::Str(_) => Some(crate::ast::Type::Str(0)),
            Expr::CStr(_) => Some(crate::ast::Type::CStr),
            Expr::Bool(_) => Some(crate::ast::Type::Bool),
            // For now, we can't infer types from identifiers or complex expressions
            // This would require full type inference integration
            _ => None,
        }
    }

    // Plan 073 Stage A.5: Check if we should use float/double arithmetic
    // Returns true if either operand is a float/double
    fn is_float_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        // Check if either operand is a float/double literal
        matches!(lhs, Expr::Float(_, _) | Expr::Double(_, _))
            || matches!(rhs, Expr::Float(_, _) | Expr::Double(_, _))
    }

    // Plan 073 Stage A.5: Check if we should use double precision (f64) vs float (f32)
    fn is_double_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        // If either operand is double, use double precision
        matches!(lhs, Expr::Double(_, _)) || matches!(rhs, Expr::Double(_, _))
    }

    // Plan 073: Convert expression to ObjectType for object field tracking
    fn infer_object_type(&self, expr: &Expr) -> ObjectType {
        match expr {
            Expr::Float(_, _) => ObjectType::Float,
            Expr::Double(_, _) => ObjectType::Double,
            Expr::Int(_) | Expr::I8(_) | Expr::I64(_) => ObjectType::Int,
            Expr::Uint(_) | Expr::U8(_) | Expr::U64(_) | Expr::Byte(_) => ObjectType::Uint,
            Expr::Str(_) | Expr::CStr(_) => ObjectType::String,
            Expr::Char(_) => ObjectType::Char,
            Expr::Bool(_) => ObjectType::Bool,
            // Plan 073: Nested object and array types
            Expr::Object(_) => ObjectType::NestedObject,
            Expr::Array(_) => ObjectType::Array,
            // For complex expressions, default to Int (will be refined later with full type inference)
            _ => ObjectType::Int,
        }
    }

    fn emit_placeholder_i16(&mut self) -> usize {
        let idx = self.code.len();
        self.code.extend_from_slice(&0i16.to_le_bytes());
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
        let scope = self.scope_stack.last_mut().expect("Scope stack should never be empty");
        let index = scope.len();
        scope.insert(name.to_string(), index);
        index
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
    /// Uses dedicated opcodes for locals 0-1 for performance
    fn emit_store_loc(&mut self, index: usize) {
        match index {
            0 => self.emit(OpCode::STORE_LOC_0),
            1 => self.emit(OpCode::STORE_LOC_1),
            _ => {
                self.emit(OpCode::STORE_LOCAL);
                self.code.push(index as u8);
            }
        }
    }

    /// Emit LOAD_LOCAL for a given local index
    /// Uses dedicated opcodes for locals 0-2 for performance
    fn emit_load_loc(&mut self, index: usize) {
        match index {
            0 => self.emit(OpCode::LOAD_LOC_0),
            1 => self.emit(OpCode::LOAD_LOC_1),
            2 => self.emit(OpCode::LOAD_LOC_2),
            _ => {
                self.emit(OpCode::LOAD_LOCAL);
                self.code.push(index as u8);
            }
        }
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

    // === Closure Support (Plan 071) ===

    /// Get the current captured_vars map (top of stack)
    /// Plan 071 Phase 6.2: Helper for accessing captured variables
    fn current_captured_vars(&self) -> &HashMap<String, usize> {
        self.captured_vars_stack.last()
            .unwrap_or_else(|| {
                // If stack is empty, return empty map (not in a closure)
                static EMPTY_MAP: std::sync::OnceLock<std::collections::HashMap<String, usize>> = std::sync::OnceLock::new();
                EMPTY_MAP.get_or_init(|| HashMap::new())
            })
    }

    /// Get mutable reference to current captured_vars map (top of stack)
    /// Plan 071 Phase 6.2: Helper for modifying captured variables
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
    fn collect_free_vars(&self, expr: &Expr, exclude: &HashSet<String>, free_vars: &mut HashSet<String>) {
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
                let mut inner_exclude = exclude.clone();
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
            Expr::View(inner) | Expr::Mut(inner) | Expr::Take(inner) => {
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
            Expr::Int(_) | Expr::Float(_, _) | Expr::Str(_) | Expr::Bool(_) | Expr::Nil | Expr::Byte(_) => {}
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
            Expr::Unary(_op, rhs) => {
                self.check_unsafe_capture(var_name, rhs)
            }

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
    fn check_unsafe_capture_in_body<'a>(&'a self, var_name: &str, body: &'a crate::ast::Body) -> Option<&'a Expr> {
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
        let param_names: HashSet<String> = closure.params.iter()
            .map(|p| p.name.to_string())
            .collect();
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
                }.into());
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

        // Emit variable name indices for each captured variable
        for var_name in &free_vars {
            let var_idx = self.add_string(var_name);
            self.code.extend_from_slice(&var_idx.to_le_bytes());
        }

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

        // Enter new scope for closure parameters
        self.push_scope();
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

        // Pop captured_vars (restore outer closure's captured vars)
        self.pop_captured_vars();

        // Step 5: Back-fill the func_addr in the CLOSURE opcode
        // Now we know the actual function address, so we can fill it in
        let func_addr_bytes = func_addr.to_le_bytes();
        for (i, byte) in func_addr_bytes.iter().enumerate() {
            self.code[func_addr_offset as usize + i] = *byte;
        }

        // Step 6: Create reloc entry for this closure (Plan 071 Phase 6.2)
        self.relocs.push(crate::vm::loader::RelocEntry {
            offset: func_addr_offset,
            symbol_name: closure_symbol.clone(),
            reloc_type: crate::vm::loader::RelocType::FuncCall,
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

    /// Infer type name from a variable name (heuristic)
    /// For standard library types, we can map variable names to types:
    /// - "list", "arr" -> "List"
    /// - "str", "s" -> "String"
    /// - "map", "dict" -> "Map"
    /// This is a fallback for when type information is not available
    fn infer_type_from_var(&self, var_name: &str) -> Option<String> {
        match var_name {
            "list" | "arr" | "array" | "vec" => Some("List".to_string()),
            "str" | "string" | "s" => Some("String".to_string()),
            "map" | "dict" | " hashmap" => Some("Map".to_string()),
            "set" => Some("Set".to_string()),
            "opt" | "option" => Some("Option".to_string()),
            "file" => Some("File".to_string()),
            _ => None,
        }
    }

    /// Check if a method requires extracting the 'id' field from the instance
    /// instead of passing the full instance.
    ///
    /// This is needed for built-in types (List, HashMap, etc.) that use shim functions
    /// expecting raw IDs (u64) rather than full Value::Instance objects.
    ///
    /// Examples:
    /// - `List.push` → extract `id` field
    /// - `List.len` → extract `id` field
    /// - `HashMap.insert` → extract `id` field
    /// - `ListIter.next` → extract `id` field (iterator methods)
    fn needs_id_extraction(&self, method_name: &str) -> bool {
        // List methods that use shim
        if method_name.starts_with("List.") {
            return matches!(
                method_name,
                "List.push" | "List.pop" | "List.len" | "List.is_empty"
                    | "List.clear" | "List.get" | "List.set" | "List.insert"
                    | "List.remove" | "List.drop" | "List.capacity"
            );
        }

        // HashMap methods that use shim
        if method_name.starts_with("HashMap.") {
            return matches!(
                method_name,
                "HashMap.insert_str" | "HashMap.insert_int"
                    | "HashMap.get_str" | "HashMap.get_int"
                    | "HashMap.contains" | "HashMap.remove"
                    | "HashMap.size" | "HashMap.clear" | "HashMap.drop"
            );
        }

        // HashSet methods that use shim
        if method_name.starts_with("HashSet.") {
            return matches!(
                method_name,
                "HashSet.insert" | "HashSet.contains"
                    | "HashSet.remove" | "HashSet.size"
                    | "HashSet.clear" | "HashSet.drop"
            );
        }

        // StringBuilder methods that use shim
        if method_name.starts_with("StringBuilder.") {
            return matches!(
                method_name,
                "StringBuilder.append" | "StringBuilder.append_char"
                    | "StringBuilder.append_int" | "StringBuilder.build"
                    | "StringBuilder.clear" | "StringBuilder.len"
                    | "StringBuilder.drop"
            );
        }

        // Heap/InlineInt64 storage methods
        if method_name.starts_with("Heap.") || method_name.starts_with("InlineInt64.") {
            return matches!(
                method_name,
                "Heap.data" | "Heap.capacity" | "Heap.try_grow" | "Heap.drop"
                    | "InlineInt64.data" | "InlineInt64.capacity"
                    | "InlineInt64.try_grow" | "InlineInt64.drop"
            );
        }

        // Iterator methods (ListIter, MapIter, FilterIter)
        if method_name.starts_with("ListIter.") || method_name.starts_with("MapIter.")
            || method_name.starts_with("FilterIter.")
        {
            return matches!(
                method_name,
                "ListIter.next" | "ListIter.map" | "ListIter.filter"
                    | "ListIter.reduce" | "ListIter.count" | "ListIter.for_each"
                    | "ListIter.collect" | "ListIter.any" | "ListIter.all"
                    | "ListIter.find" | "MapIter.next" | "MapIter.filter"
                    | "MapIter.reduce" | "MapIter.count" | "MapIter.for_each"
                    | "MapIter.collect" | "MapIter.any" | "MapIter.all"
                    | "MapIter.find" | "FilterIter.next" | "FilterIter.map"
                    | "FilterIter.reduce" | "FilterIter.count" | "FilterIter.for_each"
                    | "FilterIter.collect" | "FilterIter.any" | "FilterIter.all"
                    | "FilterIter.find"
            );
        }

        false
    }

    // ========== Plan 073: Type Registry Helper Methods ==========

    /// Register a type declaration in the type registry
    pub fn register_type(&mut self, type_decl: &TypeDecl) {
        let member_names: Vec<String> = type_decl.members
            .iter()
            .map(|m| m.name.to_string())
            .collect();

        let type_info = TypeInfo {
            name: type_decl.name.to_string(),
            member_names,
        };

        self.types.insert(type_decl.name.to_string(), type_info);
    }

    /// Check if a name is a registered type
    pub fn is_type(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Get type information by name
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
        self.types.get(name)
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
        use crate::ast::{Closure, ClosureParam, Body};

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
        let closure_pos = codegen.code.iter().position(|&b| b == OpCode::CLOSURE as u8);
        assert!(closure_pos.is_some(), "CLOSURE opcode should be emitted");

        let closure_pos = closure_pos.unwrap();

        // Verify capture count (at pos + 1 + 4 = after opcode + func_addr)
        let capture_count = codegen.code[closure_pos + 5];
        assert_eq!(capture_count, 1, "Should capture 1 variable ('n')");

        // Verify string constant was added for "n"
        assert!(codegen.strings.iter().any(|s| s == b"n"), "String pool should contain 'n'");
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
        let closure_pos = codegen.code.iter().position(|&b| b == OpCode::CLOSURE as u8);
        assert!(closure_pos.is_some(), "CLOSURE opcode should be emitted");

        let closure_pos = closure_pos.unwrap();

        // Verify capture count
        let capture_count = codegen.code[closure_pos + 5];
        assert_eq!(capture_count, 2, "Should capture 2 variables ('a' and 'b')");

        // Verify both strings were added
        assert!(codegen.strings.iter().any(|s| s == b"a"), "String pool should contain 'a'");
        assert!(codegen.strings.iter().any(|s| s == b"b"), "String pool should contain 'b'");
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
        codegen.compile_expr(&Expr::Float(3.14, "3.14".into())).unwrap();

        // Should emit CONST_F32 (0x14) followed by 4 bytes
        assert_eq!(codegen.code[0], OpCode::CONST_F32 as u8);
        assert_eq!(codegen.code.len(), 5); // 1 opcode + 4 bytes for f32
    }

    #[test]
    fn test_codegen_double_literal() {
        let mut codegen = Codegen::new();
        codegen.compile_expr(&Expr::Double(2.718281828, "2.718281828".into())).unwrap();

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
        assert_eq!(codegen.code[0], OpCode::CONST_F32 as u8);
        assert_eq!(codegen.code[5], OpCode::CONST_F64 as u8);
        assert_eq!(codegen.code[14], OpCode::ADD_D as u8);
    }
}

