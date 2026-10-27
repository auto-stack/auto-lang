//! PLAN-843 结构拆分：compile_expr_inner 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl Codegen {
    /// PLAN-843 尾递归派发段（自 compile_expr_inner 连续臂切片，臂体逐字保留）。
    pub(super) fn expr_control(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::If(if_expr) => {
                // If expression: each branch must leave a value on the stack
                // Plan 377: 全值单槽化后，每个分支恒留 1 个栈槽。
                //
                // Plan 402 bug 3 fix: an if-*expression* is a value context — each
                // branch body (a Stmt::Block) must leave its result on the stack.
                // But Stmt::Block lets the last statement inherit the *outer*
                // `should_pop_expr_result`. When the if-expression is an assignment
                // RHS (`.field = if c { a } else { b }`), the outer context has
                // `should_pop_expr_result = true`, so the branch result gets POP'd,
                // leaving nothing for the subsequent SET_FIELD to read → it reads a
                // stale stack value. Force `should_pop_expr_result = false` here so
                // every branch keeps its result value.
                let saved_pop = self.should_pop_expr_result;
                self.should_pop_expr_result = false;
                let mut jumps_to_end = Vec::new();

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
                    // No else branch - push a single nil marker to match body stack height (1 slot)
                    self.emit(OpCode::CONST_I32);
                    self.code.extend_from_slice(&(i32::MIN + 1).to_le_bytes());
                }

                // Patch all jumps to end
                for jump in jumps_to_end {
                    self.patch_jump(jump);
                }

                self.should_pop_expr_result = saved_pop;
                // Plan 118 Phase 7: If expression produces a value
                self.last_expr_type = ObjectType::Int; // default
            }
            Expr::Closure(closure) => {
                // Plan 071: Compile closure with captured environment
                self.compile_closure(closure)?;
            }
            Expr::View(inner) | Expr::Mut(inner) | Expr::Move(inner) | Expr::Take(inner) => {
                // Plan 060/122 → Plan 419 §4.4 接线:
                // - .view(共享读):直通 —— 只读访问,RC>1 无成本。
                // - .mut(独占写):编译 inner 后发射 auto.rc.assert_unique
                //   (动态兜底:RC>1 = Shared → RuntimeError)。
                // - .move/.take:inner 为简单局部时,发射 PUSH_NIL+STORE_LOCAL
                //   清源槽(防 double-drop;源槽读回 Nil)。
                match expr {
                    Expr::Mut(_) => {
                        self.compile_expr(inner)?;
                        if let Some(&id) =
                            crate::vm::native_registry::NATIVE_ID_MAP.get("auto.rc.assert_unique")
                        {
                            self.emit(OpCode::CALL_NAT);
                            self.emit_u16(id);
                        }
                    }
                    Expr::Move(_) | Expr::Take(_) => {
                        self.compile_expr(inner)?;
                        if let Expr::Ident(name) = inner.as_ref() {
                            if let Some(slot) = self.lookup_var(&name.to_string()) {
                                self.emit(OpCode::PUSH_NIL);
                                self.emit_store_loc(slot);
                            }
                        }
                    }
                    _ => {
                        self.compile_expr(inner)?;
                    }
                }
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
            // Plan 208: Emit n_args so engine can do early return on Err/None
            Expr::ErrorPropagate(expr) => {
                // Compile expression (pushes May<T> value onto stack)
                self.compile_expr(expr)?;
                // Emit ERROR_PROPAGATE n_args (pops May<T>, pushes unwrapped value or early returns)
                self.emit(OpCode::ERROR_PROPAGATE);
                self.code.push(self.current_fn_n_args as u8);
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
                    Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => match src_precise_type {
                        ConvSrcType::F32 => OpCode::TYPE_F32_TO_STR,
                        ConvSrcType::F64 => OpCode::TYPE_F64_TO_STR,
                        ConvSrcType::I64 => OpCode::TYPE_I64_TO_STR,
                        ConvSrcType::U64 => OpCode::TYPE_U64_TO_STR,
                        ConvSrcType::Bool => OpCode::TYPE_BOOL_TO_STR,
                        ConvSrcType::Str | ConvSrcType::I32 | ConvSrcType::Other => {
                            OpCode::TYPE_TO_STR
                        }
                    },
                    Type::Int => match src_precise_type {
                        ConvSrcType::F32 => OpCode::TYPE_F32_TO_I32,
                        ConvSrcType::F64 => OpCode::TYPE_F64_TO_I32,
                        ConvSrcType::Str => OpCode::TYPE_TO_I32,
                        _ => OpCode::TYPE_TO_I32,
                    },
                    Type::I64 => match src_precise_type {
                        ConvSrcType::Str => OpCode::TYPE_STR_TO_I64,
                        _ => OpCode::TYPE_CAST_I64,
                    },
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
                    Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => ObjectType::String,
                    Type::Int | Type::I64 => ObjectType::Int,
                    Type::Float | Type::Double => ObjectType::Float,
                    Type::Uint | Type::USize | Type::U64 => ObjectType::Uint,
                    _ => ObjectType::Int,
                };
            }
            // Plan 197 Task 16: Option type constructor - Some(value)
            // For primitive types, use inline encoding (no heap allocation)
            Expr::Some(inner) => {
                // Compile inner expression (pushes value onto stack)
                self.compile_expr(inner)?;
                // The inner value is already on the stack — Some(primitive) is just the value itself
                self.last_expr_type = ObjectType::NestedObject;
                self.last_enum_variant_mono = Some("Option.Some".to_string());
            }
            // Plan 197 Task 16: Option type constructor - None
            // Push nil marker (TAG_NULL in nanbox encoding)
            Expr::None => {
                self.emit(OpCode::PUSH_NIL);

                self.last_expr_type = ObjectType::NestedObject;
                self.last_enum_variant_mono = Some("Option.None".to_string());
            }
            // Plan 120: Result type constructor - Ok(value)
            // Plan 204: emit type_tag (0=i32, 1=f64) so engine pops correctly
            Expr::Ok(inner) => {
                self.compile_expr(inner)?;
                let type_tag: u8 = match &self.current_fn_ret_type {
                    Type::Result(inner) => match inner.as_ref() {
                        Type::Float | Type::Double => 1,
                        _ => 0,
                    },
                    _ => 0,
                };
                self.emit(OpCode::CREATE_OK);
                self.code.push(type_tag);
            }
            // Plan 120: Result type constructor - Err(message)
            Expr::Err(msg) => {
                // Compile error message expression (should be a string)
                self.compile_expr(msg)?;
                // The message should be on stack as a string index
                // Emit CREATE_ERR (creates Err from string)
                self.emit(OpCode::CREATE_ERR);
            }
            _ => return self.expr_async(expr),
        }
        Ok(())
    }
}
