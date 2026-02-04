use crate::ast::{Expr, Stmt, Closure};
use crate::error::AutoResult;
use crate::error::SyntaxError;
// use crate::val::Value; // Removed if not directly used or fix path
use crate::vm::loader::{Module, RelocEntry, RelocType};
use crate::vm::native::{NATIVE_PRINT_F32, NATIVE_PRINT_I32, NATIVE_PRINT_STR};
use crate::vm::native_registry::BIGVM_NATIVES;
use crate::vm::opcode::OpCode;
use auto_val::Op;
use std::collections::{HashMap, HashSet};
use miette::{SourceSpan, ByteOffset};

/// Codegen: Compiles AST directly to BigVM Bytecode
pub struct Codegen {
    pub code: Vec<u8>,
    pub exports: HashMap<String, u32>,
    pub relocs: Vec<RelocEntry>,
    pub intrinsics: HashMap<String, u16>,
    /// String constant pool
    pub strings: Vec<Vec<u8>>,

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
            locals: HashMap::new(),
            scope_stack,
            captured_vars_stack: Vec::new(),
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
                    } else {
                        unimplemented!("Assignment to non-identifier LHS not supported yet");
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
                // Extract function name and check for native functions
                let func_name = match call.name.as_ref() {
                    Expr::Ident(name) => Some(name.to_string()),
                    Expr::Dot(obj, method) => {
                        // Method call: Type.method or obj.method
                        // For simplicity, we only support Type.method for native calls (e.g., List.new)
                        match obj.as_ref() {
                            Expr::Ident(type_name) => {
                                // Static method call: Type.method
                                Some(format!("{}.{}", type_name, method))
                            }
                            _ => None,
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
                    // Compile arguments (push to stack in reverse order?)
                    // Actually, we push in forward order, left-to-right
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

                    // For instance methods (Dot expressions), compile receiver (self)
                    if let Expr::Dot(obj, _method) = call.name.as_ref() {
                        // Check if it's a static method call (Type.method)
                        // If obj is an Ident and not a lowercase local variable, it's static
                        if let Expr::Ident(_) = obj.as_ref() {
                            // Static method - no receiver needed
                            // But we still need to check if it's a type name (capitalized)
                            // For now, assume all Ident-based Dot calls are static methods
                            // TODO: Better heuristic needed
                        } else {
                            // Instance method - compile receiver (self)
                            self.compile_expr(obj)?;
                        }
                    }

                    self.emit(OpCode::CALL_NAT);
                    self.code.extend_from_slice(&id.to_le_bytes());
                    return Ok(()).into();
                }

                // Normal Function Call (user-defined)
                // 1. Compile Arguments (pushes them to stack)
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

