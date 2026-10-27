//! PLAN-843 结构拆分（自 codegen.rs 原样搬移；算法/行为零变化）。
use super::*;

impl Codegen {
    /// Look up variable in symbol table (checks all scopes from innermost to outermost)
    pub(super) fn lookup_var(&self, name: &str) -> Option<usize> {
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
    pub(super) fn add_var(&mut self, name: &str) -> usize {
        // Plan 377: 全值单槽化后，每个局部变量恒占 1 个栈槽（u64/i64/f64 亦然）。
        // 不再为 64 位变量预留 `__name_high` 第二槽。
        let mut next_offset: usize = 0;
        for scope in &self.scope_stack {
            for (_var_name, &existing_index) in scope {
                next_offset = next_offset.max(existing_index + 1);
            }
        }

        // Update max_locals to reflect the high-water mark of variables (including parameters)
        self.max_locals = self.max_locals.max(next_offset + 1);

        let scope = self
            .scope_stack
            .last_mut()
            .expect("Scope stack should never be empty");
        scope.insert(name.to_string(), next_offset);
        next_offset
    }

    /// Push a new scope (for function entry, blocks, etc.)
    pub(super) fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Pop the current scope
    ///
    /// Plan 419: 深于函数体的作用域(块/循环体)在弹栈时发射槽位释放
    /// (PUSH_NIL + STORE_LOCAL:旧值 -1、槽置 Nil)。跳过 by-ref 捕获槽;
    /// 函数体顶层作用域由 RET 帧扫描兜底,不在此发射(防双重释放);
    /// break/continue/return 跳过释放点 → 泄漏(安全方向)。
    pub(super) fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            if self.scope_stack.len() > 2 {
                if let Some(scope) = self.scope_stack.last() {
                    let slots: Vec<usize> = scope
                        .values()
                        .copied()
                        .filter(|idx| !self.byref_captured_slots.contains(idx))
                        .collect();
                    for idx in slots {
                        // emit_store_loc 需要 fn 上下文(参数 vs 局部判定);
                        // config 模式无 fn 上下文时退化为 STORE_LOCAL 直发。
                        self.emit(OpCode::PUSH_NIL);
                        self.emit_store_loc(idx);
                    }
                }
            }
            self.scope_stack.pop();
        }
    }

    /// Emit STORE_LOCAL for a given local index
    /// Plan 087 Phase 3: Distinguishes between parameters (before BP) and locals (after BP)
    /// Parameters: fn_scope_start <= index < fn_scope_start + n_args
    /// Locals: otherwise
    /// Uses dedicated opcodes for locals 0-1 for performance
    pub(super) fn emit_store_loc(&mut self, index: usize) {
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
            let local_index = index.saturating_sub(fn_scope_start).saturating_sub(n_args);

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
    /// Plan 317: Emit LOAD_GLOBAL for a module-level variable.
    /// Plan 345: Qualify a global variable name with the current module prefix.
    /// When current_module is "db", "notes" becomes "db.notes".
    fn qualified_global_name(&self, name: &str) -> String {
        if self.current_module.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", self.current_module, name)
        }
    }

    pub(super) fn emit_global_load(&mut self, name: &str) {
        let qname = self.qualified_global_name(name);
        // Plan 348 C1: intern the qualified name via add_string so the same
        // global name always resolves to a single, deduplicated pool entry.
        // The previous direct `self.strings.push(...)` could push duplicate
        // copies of the same name at different indices, which after linker
        // pool merging left the STORE and LOAD referencing inconsistent
        // entries. Interning guarantees both sides agree.
        let idx = self.add_string(&qname);
        self.emit(OpCode::LOAD_GLOBAL);
        self.emit_u32(idx);
    }

    /// Plan 348 E1: Emit LOAD_GLOBAL for an already-qualified name (e.g.
    /// "testlib.MAX" resolved from `import_scope`). Unlike emit_global_load,
    /// this does NOT prepend `self.current_module` — the caller supplies the
    /// exact name under which the global was stored in `vm.globals`.
    pub(super) fn emit_global_load_qualified(&mut self, qname: &str) {
        let idx = self.add_string(qname);
        self.emit(OpCode::LOAD_GLOBAL);
        self.emit_u32(idx);
    }

    /// Plan 317: Emit STORE_GLOBAL for a module-level variable.
    pub(super) fn emit_global_store(&mut self, name: &str) {
        let qname = self.qualified_global_name(name);
        // Plan 348 C1: see emit_global_load — intern via add_string for dedup.
        let idx = self.add_string(&qname);
        self.emit(OpCode::STORE_GLOBAL);
        self.emit_u32(idx);
    }

    pub(super) fn emit_load_loc(&mut self, index: usize) {
        vm_debug!(
            "DEBUG: emit_load_loc called with index={}, n_args={}, fn_scope_start={}",
            index,
            self.current_fn_n_args,
            self.fn_scope_start
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
            vm_debug!(
                "DEBUG: Loading parameter {} (absolute index {}) at BP-{}",
                param_index,
                index,
                offset
            );

            // Load parameter using LOAD_LOCAL with negative offset logic
            // For now, use LOAD_LOCAL with special encoding
            self.emit(OpCode::LOAD_LOCAL);
            // Encode as negative offset (0x80..0xFF means parameter)
            let encoded_index = 0x80 + param_index as u8; // 0x80 means param 0, 0x81 means param 1, etc.
            self.code.push(encoded_index);
            vm_debug!(
                "DEBUG: Emitting LOAD_LOCAL with encoded parameter index 0x{:02x}",
                encoded_index
            );
        } else {
            // This is a local variable, stored after BP
            // Local index is relative to the position after all parameters
            let local_index = index.saturating_sub(fn_scope_start).saturating_sub(n_args);
            vm_debug!(
                "DEBUG: Loading local variable {} (absolute index {}) at BP+1+{}",
                local_index,
                index,
                local_index
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
                    vm_debug!(
                        "DEBUG: Emitting LOAD_LOCAL with local index {}",
                        local_index
                    );
                }
            }
        }
    }

    /// Plan 088 Phase 4: Emit LOAD_REF for immutable reference
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn emit_load_mut_ref(&mut self, index: usize) {
        vm_debug!("DEBUG: emit_load_mut_ref called with index={}", index);
        self.emit(OpCode::LOAD_MUT_REF);
        let bytes = (index as u32).to_le_bytes();
        vm_debug!(
            "DEBUG: emit_load_mut_ref bytes: {:02x} {:02x} {:02x} {:02x}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3]
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
    pub(super) fn emit_load_captured(&mut self, var_name: &str) {
        let var_idx = self.add_string(var_name);
        self.emit(OpCode::LOAD_CAPTURED);
        self.code.extend_from_slice(&var_idx.to_le_bytes());
    }

    /// Emit STORE_CAPTURED for a captured variable by name (Plan 071)
    pub(super) fn emit_store_captured(&mut self, var_name: &str) {
        let var_idx = self.add_string(var_name);
        self.emit(OpCode::STORE_CAPTURED);
        self.code.extend_from_slice(&var_idx.to_le_bytes());
    }

    // === Plan 088 Phase 4: Smart Parameter Passing ===
}
