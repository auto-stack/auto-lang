use crate::ast::{Expr, Stmt};
use crate::error::AutoResult;
// use crate::val::Value; // Removed if not directly used or fix path
use crate::vm::loader::{Module, RelocEntry, RelocType};
use crate::vm::native::{NATIVE_PRINT_F32, NATIVE_PRINT_I32, NATIVE_PRINT_STR};
use crate::vm::native_registry::BIGVM_NATIVES;
use crate::vm::opcode::OpCode;
use auto_val::Op;
use std::collections::HashMap;

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
            Expr::Str(s) => {
                // Add string to constant pool and emit LOAD_STR <index>
                let bytes = s.as_bytes().to_vec();
                let idx = self.strings.len() as u16;
                self.strings.push(bytes);
                self.emit(OpCode::LOAD_STR);
                self.code.extend_from_slice(&idx.to_le_bytes());
            }
            Expr::Ident(name) => {
                // Look up variable in symbol table
                if let Some(var_index) = self.lookup_var(name) {
                    // Variable found - load it
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
                        // Look up variable in symbol table
                        if let Some(var_index) = self.lookup_var(name) {
                            // Variable found - store value to it
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
                    // Normal binary operation: compile both operands, then apply operator
                    self.compile_expr(lhs)?;
                    self.compile_expr(rhs)?;
                    match op {
                        Op::Add => self.emit(OpCode::ADD),
                        Op::Sub => self.emit(OpCode::SUB),
                        Op::Mul => self.emit(OpCode::MUL),
                        Op::Div => self.emit(OpCode::DIV),
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
                // Compile the operand first
                self.compile_expr(rhs)?;

                // Emit the appropriate unary opcode
                match op {
                    Op::Sub => self.emit(OpCode::NEG),
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
}
