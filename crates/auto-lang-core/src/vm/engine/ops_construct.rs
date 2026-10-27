//! PLAN-843 结构拆分：run_one_instruction 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl AutoVM {
    /// PLAN-843 尾递归派发段（自 run_one_instruction 连续臂切片，臂体逐字保留）。
    pub(super) fn ops_construct(
        &self,
        task: &mut AutoTask,
        op: OpCode,
        op_byte: u8,
    ) -> Result<StepResult, VMError> {
        match op {
            OpCode::PUSH_NIL => {
                task.ram.push_nv(auto_val::encode_null());
            }
            // Plan 318: push a real bool (nanbox tag=3), not Int(0/1). Without
            // this, `.editing = false` stores Value::Int(0) and the view
            // builder's `.editing == false` comparison fails ("0" != "false"),
            // hiding conditional blocks (note.title/body). byte: 0=false, 1=true.
            OpCode::PUSH_BOOL => {
                let b = self.flash.read_u8(task.ip);
                task.ip += 1;
                task.ram.push_nv(auto_val::encode_bool(b != 0));
            }
            // Plan 010 (MS3-A): push a try/catch handler frame.
            // Operand: i16 relative offset from the end of this instruction
            // to the catch handler. Resolved to an absolute IP and recorded.
            OpCode::PUSH_HANDLER => {
                let rel = i16::from_le_bytes([
                    self.flash.read_u8(task.ip),
                    self.flash.read_u8(task.ip + 1),
                ]);
                task.ip += 2;
                let catch_pc = task.ip.wrapping_add(rel as usize);
                task.handler_stack.push(crate::vm::task::HandlerFrame {
                    catch_pc,
                    bp: task.bp,
                    sp: task.ram.sp,
                });
            }
            // Plan 010 (MS3-A): pop the handler frame on normal try-exit.
            OpCode::POP_HANDLER => {
                task.handler_stack.pop();
            }
            OpCode::TYPE_CAST_F64 => {
                // Always pop i32 and push f32 (1 slot → 1 slot)
                let v = task.ram.pop_i32();
                task.ram.push_f32(v as f32);
                task.last_result_type = ResultType::Float;
            }
            OpCode::TYPE_CAST_PTR => {
                // Pointer cast — no-op at runtime (same bits)
            }
            // Plan 162: Explicit type conversion (.to) opcodes
            // Plan 197 Task 10: Struct instances formatted as Type { field: val, ... }
            OpCode::TYPE_TO_STR => {
                {
                    let nv = task.ram.pop_nv();
                    if auto_val::is_string(nv) {
                        task.ram.push_nv(nv);
                    } else if auto_val::is_object(nv)
                        || (auto_val::is_i32(nv) && auto_val::decode_i32(nv) >= 4000000)
                    {
                        use crate::vm::generic_registry::GenericInstanceData;
                        use crate::vm::heap_object::TypeTag;
                        let obj_id = if auto_val::is_object(nv) {
                            auto_val::decode_object(nv) as u64
                        } else {
                            auto_val::decode_i32(nv) as u64
                        };
                        let value_bits = obj_id as i32;
                        let string_value = match self.get_heap_object(obj_id) {
                            Some(obj) => {
                                let guard = obj.read().unwrap();
                                // Check for RustStdlibObject (e.g. semver::Version, Duration, etc.)
                                if let Some(rust_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                                    crate::vm::native::format_rust_stdlib_obj(rust_obj)
                                } else if let Some(dep_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>() {
                                    // PLAN-591 D2(DIV-DEP-8 .to(str) VM 半边):
                                    // dep 对象 → shim 包 Display 合成 to_string;
                                    // 无 Display 面维持 <heap:N> 占位。注意 a2r 轨
                                    // .to(str) 仍发 Debug(无 dep 元数据,P592-D3),
                                    // 该形态不入三轨语料。
                                    let short_type = dep_obj.short_type.clone();
                                    drop(guard);
                                    crate::vm::ffi::dep_methods::display_string(task, self, obj_id, &short_type)?
                                        .unwrap_or_else(|| format!("<heap:{}>", value_bits))
                                } else if let TypeTag::GenericInstance(_) = guard.type_tag() {
                                    if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                                        let type_name = self.generic_registry
                                            .get_type(&inst.mono_name)
                                            .map(|ct| ct.base_name().to_string())
                                            .unwrap_or_else(|| inst.mono_name.clone());
                                        let field_strs: Vec<String> = inst.field_names.iter()
                                            .zip(inst.fields.iter())
                                            .map(|(name, val)| {
                                                let val_str = match val {
                                                    Value::Int(i) => i.to_string(),
                                                    Value::Uint(u) => u.to_string(),
                                                    Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                                                    Value::Float(f) => f.to_string(),
                                                    Value::Double(d) => d.to_string(),
                                                    Value::Char(c) => format!("'{}'", c),
                                                    Value::Str(s) => format!("\"{}\"", s.as_str()),
                                                    Value::VmRef(r) => format!("<heap:{}>", r.id),
                                                    Value::Nil => "nil".to_string(),
                                                    _ => format!("{:?}", val),
                                                };
                                                format!("{}: {}", name, val_str)
                                            })
                                            .collect();
                                        format!("{} {{ {} }}", type_name, field_strs.join(", "))
                                    } else { format!("<heap:{}>", value_bits) }
                                } else { format!("<heap:{}>", value_bits) }
                            }
                            None => format!("<heap:{}>", value_bits),
                        };
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_bool(nv) {
                        let b = auto_val::decode_bool(nv);
                        let string_value = if b { "true" } else { "false" }.to_string();
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_f64(nv) {
                        let d = auto_val::decode_f64(nv);
                        let string_value = format!("{}", d);
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_f32(nv) {
                        let f = auto_val::decode_f32(nv);
                        let string_value = format!("{}", f);
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_bool(nv) {
                        let b = auto_val::decode_bool(nv);
                        let string_value = if b { "true" } else { "false" };
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.as_bytes().to_vec());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_null(nv) {
                        // Plan 550 T07: null → "None"（a2py str(None) 三方
                        // parity）。print shim（native.rs shim_print_i32）
                        // 已然输出 "None"，本臂补齐 TYPE_TO_STR 同型——
                        // 原落 i32 兜底臂把位模式解码成 -2147483647
                        // （p7 探针病灶）。
                        // Plan 423 P5 续修:入池收口 add_string。
                        let str_idx = self.add_string(b"None".to_vec());
                        self.rc_push_str_idx(task, str_idx);
                    } else {
                        let value_bits = auto_val::decode_i32(nv);
                        let string_value = format!("{}", value_bits);
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    }
                }
            }
            OpCode::TYPE_TO_I32 => {
                // Plan 539 W0 (DIV-PY-FLOAT-1): py-FFI returns now carry
                // real f64/f32 NanoValues — convert by truncation instead
                // of letting pop_tagged misdecode the payload bits as i32.
                let nv = task.ram.pop_nv();
                if auto_val::is_string(nv) {
                    let idx = auto_val::decode_string(nv);
                    let strings = self.strings.read().unwrap();
                    let parsed = strings
                        .get(idx as usize)
                        .and_then(|b| String::from_utf8_lossy(b).trim().parse::<i32>().ok())
                        .unwrap_or(0);
                    drop(strings);
                    // Plan 510 G3:解析已完成(内容拷贝于 parsed),份额配平释放。
                    self.pool_release(idx as usize);
                    task.ram.push_i32(parsed);
                } else if auto_val::is_f64(nv) {
                    task.ram.push_i32(auto_val::decode_f64(nv) as i32);
                } else if auto_val::is_f32(nv) {
                    task.ram.push_i32(auto_val::decode_f32(nv) as i32);
                } else if auto_val::is_bool(nv) {
                    task.ram
                        .push_i32(if auto_val::decode_bool(nv) { 1 } else { 0 });
                } else if auto_val::is_null(nv) {
                    // Plan 550 T06: null 静默 -1 臂翻案 → TypeError
                    // （原臂为 539 T05 兼容自加；TYPE_TO_I32 仅由显式
                    // .to(int) 发射，内部路径无依赖——T06 前置排查结论）。
                    return Err(crate::vm::virt_memory::null_to_type_error("int"));
                } else {
                    task.ram.push_i32(auto_val::decode_i32(nv));
                }
                task.last_result_type = ResultType::Int;
            }
            OpCode::TYPE_TO_F64 => {
                // Plan 539 W0 (DIV-PY-FLOAT-1): same tag-aware dispatch —
                // an f64 on the stack is already the answer; f32 widens.
                let nv = task.ram.pop_nv();
                if auto_val::is_string(nv) {
                    let idx = auto_val::decode_string(nv);
                    let strings = self.strings.read().unwrap();
                    let parsed = strings
                        .get(idx as usize)
                        .and_then(|b| String::from_utf8_lossy(b).trim().parse::<f32>().ok())
                        .unwrap_or(0.0);
                    drop(strings);
                    // Plan 510 G3:解析已完成(内容拷贝于 parsed),份额配平释放。
                    self.pool_release(idx as usize);
                    task.ram.push_f32(parsed);
                } else if auto_val::is_f64(nv) {
                    task.ram.push_f64(auto_val::decode_f64(nv));
                } else if auto_val::is_f32(nv) {
                    task.ram.push_f32(auto_val::decode_f32(nv));
                } else if auto_val::is_bool(nv) {
                    task.ram
                        .push_f32(if auto_val::decode_bool(nv) { 1.0 } else { 0.0 });
                } else if auto_val::is_null(nv) {
                    // Plan 550 T06: null 静默 -1.0 臂翻案 → TypeError。
                    return Err(crate::vm::virt_memory::null_to_type_error("float"));
                } else {
                    task.ram.push_f32(auto_val::decode_i32(nv) as f32);
                }
                task.last_result_type = ResultType::Float;
            }
            // Plan 193: f64 -> String
            OpCode::TYPE_F64_TO_STR => {
                let val = task.ram.pop_f64();
                let string_value = format!("{}", val);
                // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                let str_idx = self.add_string(string_value.into_bytes());
                self.rc_push_str_idx(task, str_idx);
            }
            // Plan 193: i64 -> String
            OpCode::TYPE_I64_TO_STR => {
                let val = self.pop_i64_vm(task);
                let string_value = format!("{}", val);
                // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                let str_idx = self.add_string(string_value.into_bytes());
                self.rc_push_str_idx(task, str_idx);
            }
            // Plan 193: u64 -> String (hex)
            OpCode::TYPE_U64_TO_STR => {
                let val = self.pop_u64_vm(task);
                let string_value = format!("{:08x}", val);
                // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                let str_idx = self.add_string(string_value.into_bytes());
                self.rc_push_str_idx(task, str_idx);
            }
            // Plan 193: bool -> String
            OpCode::TYPE_BOOL_TO_STR => {
                let nv = task.ram.pop_nv();
                let is_true = nv_truthy(nv);
                let string_value = if is_true { "true" } else { "false" };
                // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                let str_idx = self.add_string(string_value.as_bytes().to_vec());
                self.rc_push_str_idx(task, str_idx);
            }
            // Plan 193: f64 -> i32 (truncate)
            OpCode::TYPE_F64_TO_I32 => {
                let val = task.ram.pop_f64();
                task.ram.push_i32(val as i32);
                task.last_result_type = ResultType::Int;
            }
            // Plan 193: String -> i64
            OpCode::TYPE_STR_TO_I64 => {
                match pop_tagged(&mut task.ram) {
                    StackTag::Str(idx) => {
                        let strings = self.strings.read().unwrap();
                        let parsed = strings
                            .get(idx as usize)
                            .and_then(|b| String::from_utf8_lossy(b).trim().parse::<i64>().ok())
                            .unwrap_or(0i64);
                        drop(strings);
                        // Plan 510 G3:解析已完成(内容拷贝于 parsed),份额配平释放。
                        self.pool_release(idx as usize);
                        self.push_i64_vm(task, parsed);
                    }
                    StackTag::Int(v) => {
                        self.push_i64_vm(task, v as i64);
                    }
                }
            }
            // Plan 193: f32 -> String
            OpCode::TYPE_F32_TO_STR => {
                let val = task.ram.pop_f32();
                let string_value = format!("{}", val);
                // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                let str_idx = self.add_string(string_value.into_bytes());
                self.rc_push_str_idx(task, str_idx);
            }
            // Plan 193: f32 -> i32 (truncate)
            OpCode::TYPE_F32_TO_I32 => {
                let val = task.ram.pop_f32();
                task.ram.push_i32(val as i32);
                task.last_result_type = ResultType::Int;
            }
            // Plan 075: Convert any value to string
            // Plan 197 Task 10: Struct instances formatted as Type { field: val, ... }
            OpCode::TO_STR => {
                {
                    let nv = task.ram.pop_nv();
                    if auto_val::is_string(nv) {
                        task.ram.push_nv(nv);
                    } else if auto_val::is_object(nv) {
                        use crate::vm::generic_registry::GenericInstanceData;
                        use crate::vm::heap_object::TypeTag;
                        let obj_id = auto_val::decode_object(nv) as u64;
                        let value_bits = auto_val::decode_object(nv) as i32;
                        let string_value = match self.get_heap_object(obj_id) {
                            Some(obj) => {
                                let guard = obj.read().unwrap();
                                if let TypeTag::GenericInstance(_) = guard.type_tag() {
                                    if let Some(inst) =
                                        guard.as_any().downcast_ref::<GenericInstanceData>()
                                    {
                                        let type_name = self
                                            .generic_registry
                                            .get_type(&inst.mono_name)
                                            .map(|ct| ct.base_name().to_string())
                                            .unwrap_or_else(|| inst.mono_name.clone());
                                        let field_strs: Vec<String> = inst
                                            .field_names
                                            .iter()
                                            .zip(inst.fields.iter())
                                            .map(|(name, val)| {
                                                let val_str = match val {
                                                    Value::Int(i) => i.to_string(),
                                                    Value::Uint(u) => u.to_string(),
                                                    Value::Bool(b) => {
                                                        if *b {
                                                            "true".to_string()
                                                        } else {
                                                            "false".to_string()
                                                        }
                                                    }
                                                    Value::Float(f) => f.to_string(),
                                                    Value::Double(d) => d.to_string(),
                                                    Value::Char(c) => format!("'{}'", c),
                                                    Value::Str(s) => format!("\"{}\"", s.as_str()),
                                                    Value::VmRef(r) => format!("<heap:{}>", r.id),
                                                    Value::Nil => "nil".to_string(),
                                                    _ => format!("{:?}", val),
                                                };
                                                format!("{}: {}", name, val_str)
                                            })
                                            .collect();
                                        format!("{} {{ {} }}", type_name, field_strs.join(", "))
                                    } else {
                                        format!("<heap:{}>", value_bits)
                                    }
                                } else {
                                    format!("<heap:{}>", value_bits)
                                }
                            }
                            None => format!("<heap:{}>", value_bits),
                        };
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_bool(nv) {
                        let b = auto_val::decode_bool(nv);
                        let string_value = if b { "true" } else { "false" }.to_string();
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_f64(nv) {
                        let d = auto_val::decode_f64(nv);
                        let string_value = format!("{}", d);
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else if auto_val::is_f32(nv) {
                        let f = auto_val::decode_f32(nv);
                        let string_value = format!("{}", f);
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    } else {
                        let value_bits = auto_val::decode_i32(nv);
                        let string_value = format!("{}", value_bits);
                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                        let str_idx = self.add_string(string_value.into_bytes());
                        self.rc_push_str_idx(task, str_idx);
                    }
                }
            }
            // Plan 075: Check if value is nil
            OpCode::IS_NIL => {
                let nv = task.ram.pop_nv();
                let is_nil = if auto_val::is_null(nv) {
                    1
                } else if auto_val::is_i32(nv) && auto_val::decode_i32(nv) == -1 {
                    1
                } else {
                    0
                };
                task.ram.push_i32(is_nil);
            }
            // Plan 075: Concatenate two strings
            OpCode::STR_CAT => {
                {
                    let right_nv = task.ram.pop_nv();
                    let left_nv = task.ram.pop_nv();
                    // Plan 550 T03: null 拼接守卫（539 探针 2 病灶：
                    // "a" + null → 垃圾数字入串）。字符串形态的 `+` 在
                    // codegen 静态路由到本臂，守卫消息与算术族同格式
                    // （op 报 '+'，null 方报 'NoneType'，另一方报 'str'）。
                    if auto_val::is_null(left_nv) || auto_val::is_null(right_nv) {
                        return Err(null_binop_type_error(
                            "+",
                            (left_nv, !auto_val::is_nanboxed(left_nv)),
                            (right_nv, !auto_val::is_nanboxed(right_nv)),
                        ));
                    }
                    // Plan 419 Phase 2: 操作数 stake 在物化完成后释放(先读后放)。
                    let strings = self.strings.read().unwrap();
                    let left_str = if auto_val::is_string(left_nv) {
                        let idx = auto_val::decode_string(left_nv) as usize;
                        strings
                            .get(idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .unwrap_or_default()
                    } else if auto_val::is_object(left_nv) {
                        // Plan 390 §15 H3b: heap ref rendered as its id
                        // (explicit, not the low-32 decode coincidence).
                        auto_val::decode_object(left_nv).to_string()
                    } else {
                        Self::nv_concat_operand_display(left_nv)
                    };
                    let right_str = if auto_val::is_string(right_nv) {
                        let idx = auto_val::decode_string(right_nv) as usize;
                        strings
                            .get(idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .unwrap_or_default()
                    } else if auto_val::is_object(right_nv) {
                        auto_val::decode_object(right_nv).to_string()
                    } else {
                        Self::nv_concat_operand_display(right_nv)
                    };
                    drop(strings);
                    let result = format!("{}{}", left_str, right_str);
                    self.rc_release(left_nv);
                    self.rc_release(right_nv);
                    // Plan 419 Phase 2: 走 add_string(dedup + freelist 复用)
                    // + 池计数入栈。
                    let result_idx = self.add_string(result.into_bytes());
                    self.rc_push_str_idx(task, result_idx as usize);
                }
            }
            // Plan 120: Option type constructor - Some(value)
            OpCode::CREATE_SOME => {
                // Value is already on stack, just tag it as Some
                // We use a special encoding: Some values are positive, None is -1
                // The value is already on stack, no change needed for now
                // This opcode is a marker for type tracking
                // TODO: Implement proper Option<T> type tracking in VM
            }
            // Plan 120: Option type constructor - None
            OpCode::CREATE_NONE => {
                // Push None onto stack (represented as -1)
                task.ram.push_i32(-1);
            }
            // Plan 120: Result type constructor - Ok(value)
            // Plan 208: Wrap value in a Result.Ok heap object
            // Plan 204: type_tag operand (0=i32, 1=f64) for multi-type support
            OpCode::CREATE_OK => {
                use crate::vm::generic_registry::GenericInstanceData;
                let type_tag = self.flash.read_u8(task.ip);
                task.ip += 1;
                let val = if type_tag == 1 {
                    auto_val::Value::Double(task.ram.pop_f64())
                } else {
                    {
                        let nv = task.ram.pop_nv();
                        if auto_val::is_string(nv) {
                            let idx = auto_val::decode_string(nv) as usize;
                            let s = self
                                .get_string(idx as u32)
                                .map(|b| String::from_utf8_lossy(&b).to_string())
                                .unwrap_or_default();
                            auto_val::Value::Str(auto_val::AutoStr::from(s))
                        } else if auto_val::is_object(nv) {
                            auto_val::Value::VmRef(auto_val::VmRef {
                                id: auto_val::decode_object(nv) as usize,
                            })
                        } else {
                            auto_val::Value::Int(auto_val::decode_i32(nv))
                        }
                    }
                };
                let instance = GenericInstanceData::new("Result.Ok".to_string(), vec![val]);
                let instance_id = self.insert_heap_object(instance);
                self.rc_push(task, auto_val::encode_object(instance_id as u32));
            }
            // Plan 120: Result type constructor - Err(message)
            // Plan 208: Wrap error value in a Result.Err heap object
            OpCode::CREATE_ERR => {
                use crate::vm::generic_registry::GenericInstanceData;
                let err_val = task.ram.pop_i32();
                let instance = GenericInstanceData::new(
                    "Result.Err".to_string(),
                    vec![auto_val::Value::Int(err_val)],
                );
                let instance_id = self.insert_heap_object(instance);
                self.rc_push(task, auto_val::encode_object(instance_id as u32));
            }
            // Plan 120: Check if Option is Some
            OpCode::IS_SOME => {
                let nv = task.ram.pop_nv();
                let is_some = !(auto_val::is_null(nv)
                    || (auto_val::is_i32(nv) && auto_val::decode_i32(nv) == -1));
                task.ram.push_nv(auto_val::encode_bool(is_some));
            }
            // Plan 120: Check if Result is Ok
            // Plan 208: Check heap object mono_name instead of sign
            OpCode::IS_OK => {
                use crate::vm::generic_registry::GenericInstanceData;
                // Plan 390 §15 H3b: Result refs are TAG_OBJECT (encode_object).
                // Decode explicitly instead of relying on the low-32-bit
                // coincidence of decode_i32 on a tagged object payload.
                let nv = task.ram.pop_nv();
                let value = if auto_val::is_object(nv) {
                    auto_val::decode_object(nv) as i64
                } else if auto_val::is_i32(nv) {
                    auto_val::decode_i32(nv) as i64
                } else {
                    0
                };
                let is_ok = if value > 0 {
                    if let Some(obj) = self.get_heap_object(value as u64) {
                        let guard = obj.read().unwrap();
                        if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                            inst.mono_name == "Result.Ok"
                        } else {
                            // Legacy: plain positive value = Ok
                            true
                        }
                    } else {
                        // Legacy: plain positive value = Ok
                        true
                    }
                } else {
                    false
                };
                // VM boolean convention: i32::MIN = true, i32::MIN+1 = false
                task.ram.push_nv(auto_val::encode_bool(is_ok));
            }
            // Plan 120: Unwrap Option (panic if None)
            OpCode::UNWRAP_SOME => {
                let nv = task.ram.pop_nv();
                if auto_val::is_null(nv) || (auto_val::is_i32(nv) && auto_val::decode_i32(nv) == -1)
                {
                    return Err(VMError::RuntimeError("called unwrap on None".to_string()));
                }
                task.ram.push_nv(nv);
            }
            // Plan 120: Unwrap Result (panic if Err)
            // Plan 208: Extract field[0] from Result.Ok heap object
            OpCode::UNWRAP_OK => {
                use crate::vm::generic_registry::GenericInstanceData;
                // Plan 390 §15 H3b: Result refs are TAG_OBJECT — decode
                // explicitly instead of the low-32 coincidence.
                let nv = task.ram.pop_nv();
                let value = if auto_val::is_object(nv) {
                    auto_val::decode_object(nv) as i64
                } else if auto_val::is_i32(nv) {
                    auto_val::decode_i32(nv) as i64
                } else {
                    0
                };
                if value <= 0 {
                    return Err(VMError::RuntimeError("called unwrap on Err".to_string()));
                }
                if let Some(obj) = self.get_heap_object(value as u64) {
                    let guard = obj.read().unwrap();
                    if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                        if inst.mono_name == "Result.Ok" {
                            if let Some(field) = inst.fields.first() {
                                Self::push_value(task, field, self);
                                return Ok(StepResult::Continue);
                            }
                        }
                    }
                }
                // Legacy fallback: plain positive value
                task.ram.push_i32(value as i32);
            }
            // Plan 120: Unwrap Result error (panic if Ok)
            // Plan 208: Extract field[0] from Result.Err heap object
            OpCode::UNWRAP_ERR => {
                use crate::vm::generic_registry::GenericInstanceData;
                // Plan 390 §15 H3b: Result refs are TAG_OBJECT — decode
                // explicitly instead of the low-32 coincidence.
                let nv = task.ram.pop_nv();
                let value = if auto_val::is_object(nv) {
                    auto_val::decode_object(nv) as i64
                } else if auto_val::is_i32(nv) {
                    auto_val::decode_i32(nv) as i64
                } else {
                    0
                };
                if value <= 0 {
                    return Err(VMError::RuntimeError(
                        "called unwrap_err on non-heap value".to_string(),
                    ));
                }
                if let Some(obj) = self.get_heap_object(value as u64) {
                    let guard = obj.read().unwrap();
                    if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                        if inst.mono_name == "Result.Err" {
                            if let Some(field) = inst.fields.first() {
                                Self::push_value(task, field, self);
                                return Ok(StepResult::Continue);
                            }
                        }
                    }
                }
                task.ram.push_i32(value as i32);
            }
            // Plan 076 Phase 3 & 4: Generic List opcodes with storage strategies
            OpCode::CREATE_LIST_INT => {
                // Plan 077 Phase 5: Create List<int> in unified registry
                use crate::vm::types::ListData;
                let list_data: ListData<i32> = ListData::new(); // Heap storage (default)
                let list_id = self.insert_heap_object(list_data);

                // Push list ID onto stack
                self.rc_push(task, auto_val::encode_object(list_id as u32));
            }
            OpCode::CREATE_LIST_STR => {
                // Plan 077 Phase 5: Create List<String> in unified registry
                use crate::vm::types::ListData;
                let list_data: ListData<String> = ListData::new(); // Heap storage (default)
                let list_id = self.insert_heap_object(list_data);

                // Push list ID onto stack
                self.rc_push(task, auto_val::encode_object(list_id as u32));
            }
            OpCode::CREATE_LIST_BOOL => {
                // Plan 077 Phase 5: Create List<bool> in unified registry
                use crate::vm::types::ListData;
                let list_data: ListData<bool> = ListData::new(); // Heap storage (default)
                let list_id = self.insert_heap_object(list_data);

                // Push list ID onto stack
                self.rc_push(task, auto_val::encode_object(list_id as u32));
            }
            // Plan 076 Phase 4: InlineInt64 storage variants
            OpCode::CREATE_LIST_INT_INLINE => {
                // Plan 077 Phase 5: Create List<int> with InlineInt64 storage in unified registry
                use crate::vm::types::{ListData, ListStorage};
                let mut list_data: ListData<i32> = ListData::new();
                list_data.storage = Some(ListStorage::InlineInt64);
                let list_id = self.insert_heap_object(list_data);

                // Push list ID onto stack
                self.rc_push(task, auto_val::encode_object(list_id as u32));
            }
            OpCode::CREATE_LIST_STR_INLINE => {
                // Plan 077 Phase 5: Create List<String> with InlineInt64 storage in unified registry
                use crate::vm::types::{ListData, ListStorage};
                let mut list_data: ListData<String> = ListData::new();
                list_data.storage = Some(ListStorage::InlineInt64);
                let list_id = self.insert_heap_object(list_data);

                // Push list ID onto stack
                self.rc_push(task, auto_val::encode_object(list_id as u32));
            }
            OpCode::CREATE_LIST_BOOL_INLINE => {
                // Plan 077 Phase 5: Create List<bool> with InlineInt64 storage in unified registry
                use crate::vm::types::{ListData, ListStorage};
                let mut list_data: ListData<bool> = ListData::new();
                list_data.storage = Some(ListStorage::InlineInt64);
                let list_id = self.insert_heap_object(list_data);

                // Push list ID onto stack
                self.rc_push(task, auto_val::encode_object(list_id as u32));
            }

            // === Plan 087 Phase 2: Generic Instance Support ===
            OpCode::NEW_INSTANCE => {
                // Plan 087 Phase 2: Create a new generic instance (type-erased storage)
                // Stack layout: [..., mono_name_len]
                // Code layout: [opcode, mono_name_bytes...]
                // Stack after: [..., instance_id]
                use crate::vm::generic_registry::GenericInstanceData;

                vm_debug!(
                    "DEBUG NEW_INSTANCE: Stack depth before pop = {}",
                    task.ram.sp
                );

                // Read mono_name length from stack
                let name_len = task.ram.pop_i32() as usize;
                vm_debug!("DEBUG NEW_INSTANCE: Popped name_len = {}", name_len);

                // Read mono_name bytes from flash memory and convert to String
                // Note: task.ip already points to the first byte after the opcode (advanced by main loop)
                let mut name_bytes = vec![0u8; name_len];
                for i in 0..name_len {
                    let byte_addr = task.ip.wrapping_add(i);
                    name_bytes[i] = self.flash.read_u8(byte_addr);
                }

                // Advance IP past the name bytes
                task.ip = task.ip.wrapping_add(name_len);

                let mono_name = String::from_utf8(name_bytes).map_err(|e| {
                    VMError::RuntimeError(format!("Invalid UTF-8 in mono_name: {}", e))
                })?;
                vm_debug!("DEBUG NEW_INSTANCE: mono_name = '{}'", mono_name);

                // Create instance with no fields (uninitialized)
                let instance = GenericInstanceData::new(mono_name, vec![]);
                let instance_id = self.insert_heap_object(instance);

                // Push instance ID onto stack
                vm_debug!("DEBUG NEW_INSTANCE: Pushing instance_id = {}", instance_id);
                self.rc_push(task, auto_val::encode_object(instance_id as u32));
            }
            OpCode::CONSTRUCT_INSTANCE => {
                // Plan 087 Phase 2: Populate fields of a generic instance
                // Stack layout: [..., value1, value2, ..., valueN, instance_id, field_count]
                // Stack after: [..., instance_id]  (instance_id left on stack for variable assignment)
                use crate::vm::generic_registry::GenericInstanceData;
                use crate::vm::heap_object::TypeTag;

                vm_debug!(
                    "DEBUG CONSTRUCT_INSTANCE: Stack depth before pop = {}",
                    task.ram.sp
                );

                // Pop field_count (top of stack)
                let field_count = task.ram.pop_i32() as usize;
                vm_debug!(
                    "DEBUG CONSTRUCT_INSTANCE: Popped field_count = {}",
                    field_count
                );

                // Pop instance_id (next on stack)
                // Plan 390 §15 H2: NEW_INSTANCE now pushes the id as a
                // TAG_OBJECT-encoded value, so decode either tag form (the
                // i32 path is retained for any legacy producer/inline test).
                // PLAN-062 T12: 取走弹出槽份额(随值转移到回推栈顶)。
                // PLAN-604 T03: 必须在 pop field_count 之后、pop instance_id
                // 之前取 **sp-1** 槽——此刻实例恰在栈顶(sp-1)(NEW_INSTANCE
                // rc_push 的份额所在;实测 sp_after_count=9、实例@8)。原实现
                // 在两次 pop 之后取 sp,读到的是末字段槽(影子恒 0);
                // 首版修复取 sp(=field_count 旧槽,亦 0)——仪器化实测后钉定
                // sp-1。份额失明 → struct 字面量经 push 每实例永久滞留
                // (025-sys-monitor 55-147MB/min 复盘主凶,探针 +100 obj/拍)。
                let instance_stake = task.ram.take_stake_at(task.ram.sp - 1);
                let instance_id = {
                    let nv = task.ram.pop_nv();
                    if auto_val::is_object(nv) {
                        auto_val::decode_object(nv) as u64
                    } else {
                        auto_val::decode_i32(nv) as u64
                    }
                };
                vm_debug!(
                    "DEBUG CONSTRUCT_INSTANCE: Popped instance_id = {}",
                    instance_id
                );

                // Peek mono_name from the instance to look up field types
                let mono_name = if let Some(obj) = self.get_heap_object(instance_id) {
                    let guard = obj.read().unwrap();
                    guard
                        .as_any()
                        .downcast_ref::<GenericInstanceData>()
                        .map(|inst| inst.mono_name.clone())
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                // Pop values from stack (in reverse order)
                // Use field types from generic registry to correctly interpret stack values
                let field_types: Vec<crate::ast::Type> = self
                    .generic_registry
                    .get_type(&mono_name)
                    .map(|ct| {
                        ct.template
                            .fields
                            .iter()
                            .map(|f| f.field_type.clone())
                            .collect()
                    })
                    .unwrap_or_else(|| vec![]);
                let mut field_values = Vec::with_capacity(field_count);
                for i in 0..field_count {
                    vm_debug!(
                        "DEBUG CONSTRUCT_INSTANCE: Popping value {}/{}, stack depth = {}",
                        i + 1,
                        field_count,
                        task.ram.sp
                    );
                    // Fields are popped in reverse order; look up type from the end
                    let type_idx = field_count.saturating_sub(1 + i);
                    let field_type = field_types.get(type_idx);

                    let value = match field_type {
                        Some(crate::ast::Type::Float) => {
                            let val_f32 = task.ram.pop_f32();
                            vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popped float = {}", val_f32);
                            Value::Float(val_f32 as f64)
                        }
                        Some(crate::ast::Type::Double) => {
                            let val_f64 = task.ram.pop_f64();
                            vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popped double = {}", val_f64);
                            Value::Double(val_f64)
                        }
                        _ => {
                            // Pop value — in nanbox mode, preserve type information
                            {
                                let nv = task.ram.pop_nv();
                                if auto_val::is_string(nv) {
                                    let idx = auto_val::decode_string(nv) as usize;
                                    let strings_guard = self.strings.read().unwrap();
                                    if idx < strings_guard.len() {
                                        let s = String::from_utf8_lossy(&strings_guard[idx])
                                            .to_string();
                                        drop(strings_guard);
                                        // Plan 419 Phase 2: 实例字段持拷贝,池 stake 死亡。
                                        self.pool_release(idx);
                                        Value::Str(auto_val::AutoStr::from(s))
                                    } else {
                                        drop(strings_guard);
                                        // Plan 454 A3(447 2.4):字符串 tag 指向
                                        // 越界池索引曾是"解成 i32 继续跑"的静默
                                        // 回退——nanbox 重读出的必然是乱值,编码
                                        // 漂移应显式失败而非产出垃圾字段。
                                        return Err(VMError::RuntimeError(format!(
                                                "CONSTRUCT_INSTANCE: string payload pool index {} out of bounds (pool len {})",
                                                idx,
                                                self.strings.read().unwrap().len()
                                            )));
                                    }
                                } else if auto_val::is_object(nv) {
                                    Value::VmRef(auto_val::VmRef {
                                        id: auto_val::decode_object(nv) as usize,
                                    })
                                } else if auto_val::is_null(nv) {
                                    Value::Nil
                                } else if auto_val::is_bool(nv) {
                                    Value::Bool(auto_val::decode_bool(nv))
                                } else if auto_val::is_f64(nv) {
                                    Value::Double(auto_val::decode_f64(nv))
                                } else if auto_val::is_f32(nv) {
                                    Value::Float(auto_val::decode_f32(nv) as f64)
                                } else if auto_val::is_future(nv) {
                                    // PLAN-776 R9-02：Future 句柄字段**保真映射**
                                    //（原落 Int 兜臂把 fid 存成 Int(1)——F776-R9-02）。
                                    Value::Future(Box::new(auto_val::FutureData {
                                        id: auto_val::decode_future(nv) as usize,
                                        state: auto_val::FutureState::Pending,
                                        result: None,
                                    }))
                                } else {
                                    Value::Int(auto_val::decode_i32(nv))
                                }
                            }
                        }
                    };
                    field_values.push(value);
                }
                field_values.reverse(); // Reverse to get correct order

                vm_debug!(
                    "DEBUG CONSTRUCT_INSTANCE: Field values (reversed): {:?}",
                    field_values
                );

                // Get instance and populate fields
                if let Some(obj) = self.get_heap_object(instance_id) {
                    let mut guard = obj.write().unwrap();

                    // Check if this is a GenericInstance by checking the type tag
                    let is_generic_instance =
                        matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                    if is_generic_instance {
                        // Use as_any_mut for downcasting (works without exact TypeTag match)
                        if let Some(instance) =
                            guard.as_any_mut().downcast_mut::<GenericInstanceData>()
                        {
                            let field_count = field_values.len();
                            instance.fields = field_values;

                            // Plan 197 Task 9: Populate field_names from generic registry
                            let field_names = self
                                .generic_registry
                                .get_type(&instance.mono_name)
                                .map(|ct| {
                                    ct.template.fields.iter().map(|f| f.name.clone()).collect()
                                })
                                .unwrap_or_else(|| vec!["_unknown".to_string(); field_count]);
                            instance.field_names = field_names;

                            vm_debug!(
                                "DEBUG CONSTRUCT_INSTANCE: Successfully populated {} fields",
                                field_count
                            );
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: Failed to downcast GenericInstance"
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Type error: CONSTRUCT_INSTANCE expected GenericInstance, got {:?}",
                            guard.type_tag()
                        )));
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid instance ID: {}",
                        instance_id
                    )));
                }

                // Push instance_id back onto stack for variable assignment
                // Stack layout after: [..., instance_id]
                // Plan 419: 自栈 pop 转移回栈(不 +1 —— NEW_INSTANCE 的
                // stake 一路随行;字段值的 stakes 已随 field_values 转移进实例)。
                vm_debug!(
                    "DEBUG CONSTRUCT_INSTANCE: Pushing instance_id back to stack: {}",
                    instance_id
                );
                task.ram
                    .push_nv(auto_val::encode_object(instance_id as u32));
                // PLAN-062 T12: 份额随值转移回栈顶。
                task.ram.mark_top_stake(instance_stake);
                vm_debug!(
                    "DEBUG CONSTRUCT_INSTANCE: Stack depth after = {}",
                    task.ram.sp
                );
            }
            OpCode::IS_VARIANT => {
                // Check if a value matches an expected variant name
                // Code layout: [opcode, name_len:u16, name_bytes...]
                // Stack layout: [..., value]
                // Stack after: [..., bool] (value consumed, bool pushed)
                use crate::vm::generic_registry::GenericInstanceData;

                let name_len = self.flash.read_u16(task.ip) as usize;
                task.ip += 2;
                let mut name_bytes = vec![0u8; name_len];
                for i in 0..name_len {
                    name_bytes[i] = self.flash.read_u8(task.ip);
                    task.ip += 1;
                }
                let expected_name = String::from_utf8_lossy(&name_bytes).to_string();

                {
                    let nv = task.ram.pop_nv();
                    let obj_id = if auto_val::is_object(nv) {
                        Some(auto_val::decode_object(nv) as u64)
                    } else if auto_val::is_i32(nv) {
                        let v = auto_val::decode_i32(nv);
                        if v >= 4000000 {
                            Some(v as u64)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let result = if auto_val::is_null(nv) {
                        expected_name == "Option.None"
                    } else if let Some(id) = obj_id {
                        if let Some(obj) = self.get_heap_object(id) {
                            let guard = obj.read().unwrap();
                            if let Some(instance) =
                                guard.as_any().downcast_ref::<GenericInstanceData>()
                            {
                                instance.mono_name == expected_name
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        // Option 原始载荷编码(CREATE_SOME 不包对象):
                        // int/str/bool/f64 载荷按 Some 命中是设计行为。
                        // Plan 454 A2 复核(447 2.4):曾尝试对"用户变体名
                        // ×非对象标量"显式报错,被 p03_enum_payload 实证
                        // 否决——单元变体(Val.VN 等)以裸判别值标量合法
                        // 流入级联 payload 模式测试,静默 false 是级联
                        // 语义的一部分,运行时无类型元数据不可与编码漂移
                        // 区分。维持宽松,false 为终态。
                        expected_name == "Option.Some"
                    };
                    task.ram.push_nv(auto_val::encode_bool(result));
                    // Plan 419: 被消费的 receiver 引用 stake 死亡。
                    self.rc_release(nv);
                }
            }
            OpCode::GET_GENERIC_FIELD => {
                // Get field value from a generic instance or primitive Option.Some value
                // Code layout: [opcode, field_index:u32]
                // Stack layout: [..., value]
                // Stack after: [..., field_value]
                use crate::vm::generic_registry::GenericInstanceData;
                use crate::vm::heap_object::TypeTag;

                let field_index = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;

                {
                    let nv = task.ram.pop_nv();
                    // Determine object ID: if it's a proper TAG_OBJECT, decode it;
                    // if it's TAG_I32 with a large value, it might be an object ID stored as i32
                    let obj_id = if auto_val::is_object(nv) {
                        Some(auto_val::decode_object(nv) as u64)
                    } else if auto_val::is_i32(nv) {
                        let v = auto_val::decode_i32(nv);
                        if v >= 4000000 {
                            Some(v as u64)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(id) = obj_id {
                        // Plan 419 §9.4 ②:访问点打点——tag 来源直接判
                        // H0(TAG_I32≥4M 启发式误判)vs H1/H2(真悬垂)。
                        if crate::vm::rc::p419_uaf_traces(id) {
                            let via = if auto_val::is_object(nv) {
                                "TAG_OBJECT"
                            } else if auto_val::is_i32(nv) {
                                "TAG_I32-heur"
                            } else {
                                "other"
                            };
                            // 裸表探测(绕过 canary):log 先行,随后真正的
                            // get_heap_object 若命中 tombstone 即 panic。
                            let live = self.heap_objects.get(&id).is_some();
                            // tombstones 表只在 debug_assertions 下存在
                            // (字段级 cfg);release 下该探测退化为
                            // live-only(否则 release 编译 E0609)。
                            #[cfg(debug_assertions)]
                            let tomb = self.tombstones.get(&id).is_some();
                            #[cfg(not(debug_assertions))]
                            let tomb = false;
                            let fn_name = task
                                .call_stack
                                .last()
                                .and_then(|f| f.fn_name.clone())
                                .unwrap_or_else(|| "<root>".to_string());
                            eprintln!(
                                    "[P419UAF] ACCESS GET_GENERIC_FIELD id={} via={} field={} live={} tomb={} fn='{}' ip=0x{:x}",
                                    id, via, field_index, live, tomb, fn_name, task.ip
                                );
                        }
                        if let Some(obj) = self.get_heap_object(id) {
                            let guard = obj.read().unwrap();
                            let is_generic_instance =
                                matches!(guard.type_tag(), TypeTag::GenericInstance(_));
                            if is_generic_instance {
                                if let Some(instance) =
                                    guard.as_any().downcast_ref::<GenericInstanceData>()
                                {
                                    if let Some(value) = instance.get_field(field_index) {
                                        // PLAN-047 T-04: VM 读臂依赖录制
                                        // （字段名可证面；未激活零开销）。
                                        if let Some(name) = instance.field_names.get(field_index) {
                                            self.record_heap_read(id, name);
                                        }
                                        Self::push_value(task, value, self);
                                    } else {
                                        return Err(VMError::RuntimeError(format!(
                                            "Field index {} out of bounds",
                                            field_index
                                        )));
                                    }
                                } else {
                                    return Err(VMError::RuntimeError(
                                        "GET_GENERIC_FIELD: failed to downcast".to_string(),
                                    ));
                                }
                            } else if field_index == 0 {
                                self.rc_push(task, nv); // Plan 419: 透传 +1(与臂尾释放配对成转移)
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Field index {} out of bounds for non-generic object",
                                    field_index
                                )));
                            }
                        } else if field_index == 0 {
                            self.rc_push(task, nv); // Plan 419: 透传 +1(与臂尾释放配对成转移)
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Field index {} out of bounds",
                                field_index
                            )));
                        }
                    } else if field_index == 0 {
                        // Primitive value — field 0 is the value itself
                        self.rc_push(task, nv); // Plan 419: 透传 +1(与臂尾释放配对成转移)
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Field index {} out of bounds for primitive",
                            field_index
                        )));
                    }
                    // Plan 419: 被消费的 receiver 引用 stake 死亡
                    // (push_value 已对字段 VmRef +1;原样回推路径为转移)。
                    self.rc_release(nv);
                }
            }
            _ => return self.ops_collections(task, op, op_byte),
        }
        Ok(StepResult::Continue)
    }
}
