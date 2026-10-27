//! PLAN-843 结构拆分：run_one_instruction 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl AutoVM {
    /// PLAN-843 尾递归派发段（自 run_one_instruction 连续臂切片，臂体逐字保留）。
    pub(super) fn ops_flow(&self, task: &mut AutoTask, op: OpCode, op_byte: u8) -> Result<StepResult, VMError> {
        match op {
            OpCode::STORE_STATE_FIELD => {
                let field_idx = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;
                let val_nv = task.ram.pop_nv();
                // VM bug fix: skip the default initializer for fields set at
                // spawn via init args (shim_task_spawn_vm locked them). The
                // lock set is cleared at TASK_LOOP, so handler writes (which
                // run after TASK_LOOP) are unaffected.
                if !task.locked_state_fields.contains(&(field_idx as u8)) {
                    // Grow state_vars if needed (safety; normally pre-sized at spawn).
                    if field_idx >= task.state_vars.len() {
                        task.state_vars.resize(field_idx + 1, 0);
                    }
                    // PLAN-062 T12: 弹出槽影子随值转移;state_vars 条目
                    // 恒持一份(无影子的裸堆引用防御性补持)。
                    let transferred = task.ram.take_stake_at(task.ram.sp);
                    let _ = transferred;
                    // PLAN-667 (F-07): 防御性补持收紧到 tagged 引用
                    // (TAG_OBJECT/LIST/BIGINT)——裸 i32≥4M 与真整数不可
                    // 按值区分,补持会让真整数冒领活对象份额。裸 id 生产
                    // 者已全部迁移 rc_push_id(带影子),无份额裸 i32 即真整数。
                    let is_tagged_ref = auto_val::is_object(val_nv)
                        || auto_val::is_list(val_nv)
                        || auto_val::is_bigint(val_nv);
                    if is_tagged_ref {
                        if let Some(id) = crate::vm::rc::heap_ref_id(val_nv) {
                            self.rc_retain_id(id);
                        }
                    }
                    // Plan 419: 旧 state 字段值 -1;新值转移进槽。
                    self.rc_release(task.state_vars[field_idx]);
                    task.state_vars[field_idx] = val_nv;
                } else {
                    // Plan 419: 被锁定跳过写入时,弹出的值按死亡处理。
                    self.rc_release(val_nv);
                }
            }
            // Plan 317: Global variable access (module-level var).
            // name_idx: u32 indexes the string pool.
            OpCode::LOAD_GLOBAL => {
                let name_idx = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;
                let name = self
                    .strings
                    .read()
                    .unwrap()
                    .get(name_idx)
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .unwrap_or_default();
                let nv = self.globals.get(&name).map(|v| *v).unwrap_or(0);
                // Plan 419: copy-on-load(全局表项保留自己的 stake)。
                // PLAN-667 (F-07): 全局表条目持有份额(写入时已立),按
                // tagged 判定走正常 copy-on-load;rc_push_slot 对 tagged
                // 引用恒计份。
                self.rc_push_slot(task, nv, u64::MAX);
            }
            OpCode::STORE_GLOBAL => {
                let name_idx = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;
                let name = self
                    .strings
                    .read()
                    .unwrap()
                    .get(name_idx)
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .unwrap_or_default();
                let nv = task.ram.pop_nv();
                // PLAN-062 T12: 读走弹出槽影子(转移语义);无份额的裸堆
                // 引用入全局表前防御性补一份(全局表条目恒持有——保守
                // 方向:多持至多延后回收,不持有即悬垂)。
                let transferred = task.ram.take_stake_at(task.ram.sp);
                let _ = transferred;
                // PLAN-667 (F-07): 同 STORE_STATE_FIELD——防御性补持收紧
                // 到 tagged 引用,真整数不再冒领活对象份额。
                let is_tagged_ref =
                    auto_val::is_object(nv) || auto_val::is_list(nv) || auto_val::is_bigint(nv);
                if is_tagged_ref {
                    if let Some(id) = crate::vm::rc::heap_ref_id(nv) {
                        self.rc_retain_id(id);
                    }
                }
                // Plan 419: 旧全局值 -1;新值自栈转移进全局表。
                if let Some(old) = self.globals.get(&name) {
                    let old_nv = *old;
                    drop(old);
                    self.rc_release(old_nv);
                }
                self.globals.insert(name, nv);
            }
            OpCode::LOAD_LOC_1 => {
                // Plan 419: copy-on-load。PLAN-667 (F-07): 源槽份额出处门。
                let slot = task.bp + 2;
                let nv = task.ram.read_nv(slot);
                let ss = task.ram.stake_at(slot);
                self.rc_push_slot(task, nv, ss);
            }
            OpCode::LOAD_LOC_2 => {
                // Plan 419: copy-on-load。
                self.rc_push(task, task.ram.read_nv(task.bp + 3));
            }
            OpCode::STORE_LOC_0 => {
                // Plan 419: 覆盖旧值 -1;新值转移。PLAN-062 T12: 影子随值
                // 转移,旧槽按影子释放。
                let v = task.ram.pop_nv();
                let transferred = task.ram.take_stake_at(task.ram.sp);
                let addr = task.bp + 1;
                self.release_slot_old_value(task, addr);
                task.ram.write_nv(addr, v);
                task.ram.mark_stake_at(addr, transferred);
            }
            OpCode::STORE_LOC_1 => {
                // Plan 419: 覆盖旧值 -1;新值转移。PLAN-062 T12: 影子随值
                // 转移,旧槽按影子释放。
                let v = task.ram.pop_nv();
                let transferred = task.ram.take_stake_at(task.ram.sp);
                let addr = task.bp + 2;
                self.release_slot_old_value(task, addr);
                task.ram.write_nv(addr, v);
                task.ram.mark_stake_at(addr, transferred);
            }

            // === Stack ===
            OpCode::DROP => {
                // Plan 419: DROP 兑现 RAII 承诺 —— 弹出并释放 owned value
                // (引用值计数 -1,归零则真回收);槽位清零。
                // PLAN-062 T12: 堆份额按影子释放;字符串沿内容释放。
                let nv = task.ram.pop_nv();
                let stake = task.ram.take_stake_at(task.ram.sp);
                if stake != 0 {
                    self.rc_release_id(stake);
                }
                if auto_val::is_string(nv) {
                    self.rc_release(nv);
                }
                task.ram.raw_nv[task.ram.sp] = 0;
            }
            // Plan 088 Phase 4: Function Prologue
            OpCode::FN_PROLOG => {
                // Read function metadata
                let n_args = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;
                let n_locals = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                vm_debug!("DEBUG FN_PROLOG: n_args={}, n_locals={}", n_args, n_locals);

                // Save function metadata in task for use by LOAD_LOCAL/STORE_LOCAL
                task.current_fn_n_args = n_args;
                task.current_fn_n_locals = n_locals;
            }
            OpCode::RESERVE_STACK => {
                // Reserve stack space for n_locals to prevent stack from overwriting locals
                let n_locals = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                // Push n_locals+1 zeros to reserve space for local variables + 1 extra slot
                // The extra slot ensures SP starts beyond all local variable addresses
                for _ in 0..n_locals + 1 {
                    task.ram.push_i32(0);
                }

                vm_debug!("RESERVE_STACK: n_locals={} sp={}", n_locals, task.ram.sp);

                // Track num_locals for native shims
                task.num_locals = n_locals;
            }

            // === Comparison ===
            OpCode::EQ => {
                // Plan 539 W1 (T10): py-handle comparison dunder.
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_cmp("__eq__", "", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let b_nv = task.ram.pop_nv();
                    let a_nv = task.ram.pop_nv();
                    // Plan 419: 比较消费操作数 —— stake 在比较完成后释放(先读后放)。
                    // PLAN-053 P-053-2: nil 家族等值 —— 一侧 tag-null
                    // (encode_null)、另一侧旧编码 i32(-1) 或 i32(i32::MIN+1)
                    // 时视为相等。新代码字面量已全归一 PUSH_NIL，此处兜底
                    // 存量载荷(musk KV 持久化的旧 Int(-1) 字段、生成器
                    // CREATE_NONE 返回值)，否则 `x != None` 守卫对旧数据
                    // 恒真(gate 卡常显)。
                    let a_is_null_family = Self::nv_is_null_family(a_nv);
                    let b_is_null_family = Self::nv_is_null_family(b_nv);
                    let result = if a_nv == b_nv {
                        true
                    } else if a_is_null_family && b_is_null_family {
                        true
                    } else if auto_val::is_object(a_nv) && auto_val::is_object(b_nv) {
                        let a = auto_val::decode_object(a_nv) as i32;
                        let b = auto_val::decode_object(b_nv) as i32;
                        self.struct_eq(a, b)
                    } else if auto_val::is_string(a_nv) && auto_val::is_string(b_nv) {
                        let a_idx = auto_val::decode_string(a_nv);
                        let b_idx = auto_val::decode_string(b_nv);
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => sa == sb,
                            _ => false,
                        }
                    } else if auto_val::is_i32(a_nv) && auto_val::is_i32(b_nv) {
                        let a_val = auto_val::decode_i32(a_nv);
                        let b_val = auto_val::decode_i32(b_nv);
                        if a_val >= 4000000 && b_val >= 4000000 {
                            self.struct_eq(a_val, b_val)
                        } else {
                            Self::bool_eq(a_val, b_val)
                        }
                    } else if auto_val::is_f64(a_nv) && auto_val::is_f64(b_nv) {
                        auto_val::decode_f64(a_nv) == auto_val::decode_f64(b_nv)
                    } else if auto_val::is_f32(a_nv) && auto_val::is_f32(b_nv) {
                        auto_val::decode_f32(a_nv) == auto_val::decode_f32(b_nv)
                    } else if Self::nv_is_numeric(a_nv) && Self::nv_is_numeric(b_nv) {
                        // Plan 539 W2 (T19): mixed numeric compare (f64 ==
                        // i32 literal) — py_float results against int
                        // literals previously fell to the false catch-all.
                        Self::nv_as_f64(a_nv) == Self::nv_as_f64(b_nv)
                    } else {
                        false
                    };
                    self.rc_release(a_nv);
                    self.rc_release(b_nv);
                    task.ram.push_nv(auto_val::encode_bool(result));
                }
            }
            OpCode::NE => {
                // Plan 539 W1 (T10): py-handle comparison dunder.
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_cmp("__ne__", "", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let b_nv = task.ram.pop_nv();
                    let a_nv = task.ram.pop_nv();
                    // Plan 419: 比较消费操作数 —— stake 在比较完成后释放(先读后放)。
                    // PLAN-053 P-053-2: nil 家族等值(与 EQ 对称,见其注释)。
                    let a_is_null_family = Self::nv_is_null_family(a_nv);
                    let b_is_null_family = Self::nv_is_null_family(b_nv);
                    let result = if a_nv == b_nv {
                        false
                    } else if a_is_null_family && b_is_null_family {
                        false
                    } else if auto_val::is_object(a_nv) && auto_val::is_object(b_nv) {
                        let a = auto_val::decode_object(a_nv) as i32;
                        let b = auto_val::decode_object(b_nv) as i32;
                        !self.struct_eq(a, b)
                    } else if auto_val::is_string(a_nv) && auto_val::is_string(b_nv) {
                        let a_idx = auto_val::decode_string(a_nv);
                        let b_idx = auto_val::decode_string(b_nv);
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => sa != sb,
                            _ => true,
                        }
                    } else if auto_val::is_i32(a_nv) && auto_val::is_i32(b_nv) {
                        let a_val = auto_val::decode_i32(a_nv);
                        let b_val = auto_val::decode_i32(b_nv);
                        // Both values >= 4000000 are heap object IDs stored as i32 — use struct_eq
                        if a_val >= 4000000 && b_val >= 4000000 {
                            !self.struct_eq(a_val, b_val)
                        } else {
                            !Self::bool_eq(a_val, b_val)
                        }
                    } else if Self::nv_is_numeric(a_nv) && Self::nv_is_numeric(b_nv) {
                        // Plan 539 W2 (T19): mixed numeric compare — see EQ.
                        Self::nv_as_f64(a_nv) != Self::nv_as_f64(b_nv)
                    } else {
                        true
                    };
                    self.rc_release(a_nv);
                    self.rc_release(b_nv);
                    task.ram.push_nv(auto_val::encode_bool(result));
                }
            }
            OpCode::LT => {
                // Plan 539 W1 (T10): py-handle comparison dunder.
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_cmp("__lt__", "__gt__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let b_nv = task.ram.pop_nv();
                    let a_nv = task.ram.pop_nv();
                    let result = if auto_val::is_string(a_nv) && auto_val::is_string(b_nv) {
                        let a_idx = auto_val::decode_string(a_nv);
                        let b_idx = auto_val::decode_string(b_nv);
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => {
                                let sa = String::from_utf8_lossy(&sa);
                                let sb = String::from_utf8_lossy(&sb);
                                sa < sb
                            }
                            _ => false,
                        }
                    } else if Self::nv_pair_is_float_numeric(a_nv, b_nv) {
                        // Plan 576: float×(int|float) 按值比较——原 else 臂
                        // decode_i32 把 f32/raw-f64 位型当整数（043 比较守卫假）。
                        Self::nv_as_f64(a_nv) < Self::nv_as_f64(b_nv)
                    } else {
                        // Plan 390 §15 H3b: heap refs are TAG_OBJECT — decode
                        // explicitly (same id value as the old low-32 trick).
                        // PLAN-026 T-03 配套: 宽整兜底(object ref 旧路
                        // 保留;i32/i64 tag 归一按值;纯 i32 语义等价)。
                        let a = if auto_val::is_object(a_nv) {
                            auto_val::decode_object(a_nv) as i64
                        } else {
                            Self::nv_int_as_i64(a_nv)
                        };
                        let b = if auto_val::is_object(b_nv) {
                            auto_val::decode_object(b_nv) as i64
                        } else {
                            Self::nv_int_as_i64(b_nv)
                        };
                        a < b
                    };
                    task.ram.push_nv(auto_val::encode_bool(result));
                }
            }
            OpCode::GT => {
                // Plan 539 W1 (T10): py-handle comparison dunder.
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_cmp("__gt__", "__lt__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let b_nv = task.ram.pop_nv();
                    let a_nv = task.ram.pop_nv();
                    let a_is_obj = auto_val::is_object(a_nv);
                    let b_is_obj = auto_val::is_object(b_nv);
                    let result = if a_is_obj && b_is_obj {
                        let a_id = auto_val::decode_object(a_nv) as u64;
                        let b_id = auto_val::decode_object(b_nv) as u64;
                        if let Some(cmp_result) = self.compare_opaque_objects(a_id, b_id) {
                            cmp_result
                        } else {
                            a_id > b_id
                        }
                    } else if auto_val::is_i32(a_nv) && auto_val::is_i32(b_nv) {
                        let a = auto_val::decode_i32(a_nv);
                        let b = auto_val::decode_i32(b_nv);
                        // Opaque object comparison: check if both are heap objects
                        if a > 0 && b > 0 {
                            let cmp = self.compare_opaque_objects(a as u64, b as u64);
                            if let Some(cmp_result) = cmp {
                                cmp_result
                            } else {
                                a > b
                            }
                        } else if auto_val::is_string(a_nv) || auto_val::is_string(b_nv) {
                            // shouldn't reach here for i32 check, but be safe
                            a > b
                        } else {
                            a > b
                        }
                    } else if auto_val::is_string(a_nv) && auto_val::is_string(b_nv) {
                        let a_idx = auto_val::decode_string(a_nv);
                        let b_idx = auto_val::decode_string(b_nv);
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => {
                                let sa = String::from_utf8_lossy(&sa);
                                let sb = String::from_utf8_lossy(&sb);
                                sa > sb
                            }
                            _ => false,
                        }
                    } else if Self::nv_pair_is_float_numeric(a_nv, b_nv) {
                        // Plan 576: float×(int|float) 按值比较——GT fallback
                        // 注记承认的 decode_i32 位型误读（f64 落 GT 时取低
                        // 32 位、f32 取 payload 位型）就此收口。
                        Self::nv_as_f64(a_nv) > Self::nv_as_f64(b_nv)
                    } else {
                        // Fallback (covers f64 values that land in GT instead
                        // of GT_D when type inference misses the double type).
                        // PLAN-026 T-03 配套: 宽整兜底(i32/i64 tag 归一按值;
                        // 纯 i32 语义等价)。
                        let a = Self::nv_int_as_i64(a_nv);
                        let b = Self::nv_int_as_i64(b_nv);
                        a > b
                    };
                    task.ram.push_nv(auto_val::encode_bool(result));
                }
            }
            OpCode::LE => {
                // Plan 539 W1 (T10): py-handle comparison dunder.
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_cmp("__le__", "__ge__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let b_nv = task.ram.pop_nv();
                    let a_nv = task.ram.pop_nv();
                    let result = if auto_val::is_string(a_nv) && auto_val::is_string(b_nv) {
                        let a_idx = auto_val::decode_string(a_nv);
                        let b_idx = auto_val::decode_string(b_nv);
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => {
                                let sa = String::from_utf8_lossy(&sa);
                                let sb = String::from_utf8_lossy(&sb);
                                sa <= sb
                            }
                            _ => true,
                        }
                    } else if Self::nv_pair_is_float_numeric(a_nv, b_nv) {
                        // Plan 576: float×(int|float) 按值比较（同 LT 注记）。
                        Self::nv_as_f64(a_nv) <= Self::nv_as_f64(b_nv)
                    } else {
                        // PLAN-026 T-03 配套: 宽整兜底(i32/i64 tag
                        // 归一按值;纯 i32 语义等价)。
                        let a = Self::nv_int_as_i64(a_nv);
                        let b = Self::nv_int_as_i64(b_nv);
                        a <= b
                    };
                    task.ram.push_nv(auto_val::encode_bool(result));
                }
            }
            OpCode::GE => {
                // Plan 539 W1 (T10): py-handle comparison dunder.
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_cmp("__ge__", "__le__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let b_nv = task.ram.pop_nv();
                    let a_nv = task.ram.pop_nv();
                    let result = if auto_val::is_string(a_nv) && auto_val::is_string(b_nv) {
                        let a_idx = auto_val::decode_string(a_nv);
                        let b_idx = auto_val::decode_string(b_nv);
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => {
                                let sa = String::from_utf8_lossy(&sa);
                                let sb = String::from_utf8_lossy(&sb);
                                sa >= sb
                            }
                            _ => true,
                        }
                    } else if Self::nv_pair_is_float_numeric(a_nv, b_nv) {
                        // Plan 576: float×(int|float) 按值比较（同 LT 注记）。
                        Self::nv_as_f64(a_nv) >= Self::nv_as_f64(b_nv)
                    } else {
                        // PLAN-026 T-03 配套: 宽整兜底(i32/i64 tag
                        // 归一按值;纯 i32 语义等价)。
                        let a = Self::nv_int_as_i64(a_nv);
                        let b = Self::nv_int_as_i64(b_nv);
                        a >= b
                    };
                    task.ram.push_nv(auto_val::encode_bool(result));
                }
            }

            // f64 comparison opcodes (Plan 377: each pops 1+1 slot, pushes 1 bool)
            OpCode::EQ_D => {
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_nv(auto_val::encode_bool(a == b));
            }
            OpCode::NE_D => {
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_nv(auto_val::encode_bool(a != b));
            }
            OpCode::LT_D => {
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_nv(auto_val::encode_bool(a < b));
            }
            OpCode::GT_D => {
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_nv(auto_val::encode_bool(a > b));
            }
            OpCode::LE_D => {
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_nv(auto_val::encode_bool(a <= b));
            }
            OpCode::GE_D => {
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_nv(auto_val::encode_bool(a >= b));
            }

            // Plan 378/377: u64/i64 comparison (each pops 1+1 slot, pushes 1 bool)
            OpCode::EQ_U64 => {
                let b = self.pop_i64_vm(task);
                let a = self.pop_i64_vm(task);
                task.ram.push_nv(auto_val::encode_bool(a == b));
            }
            OpCode::NE_U64 => {
                let b = self.pop_i64_vm(task);
                let a = self.pop_i64_vm(task);
                task.ram.push_nv(auto_val::encode_bool(a != b));
            }
            OpCode::LT_U64 => {
                let b = self.pop_i64_vm(task);
                let a = self.pop_i64_vm(task);
                task.ram.push_nv(auto_val::encode_bool(a < b));
            }
            OpCode::GT_U64 => {
                let b = self.pop_i64_vm(task);
                let a = self.pop_i64_vm(task);
                task.ram.push_nv(auto_val::encode_bool(a > b));
            }
            OpCode::LE_U64 => {
                let b = self.pop_i64_vm(task);
                let a = self.pop_i64_vm(task);
                task.ram.push_nv(auto_val::encode_bool(a <= b));
            }
            OpCode::GE_U64 => {
                let b = self.pop_i64_vm(task);
                let a = self.pop_i64_vm(task);
                task.ram.push_nv(auto_val::encode_bool(a >= b));
            }

            // Plan 403-F: f32 comparison (each pops 1+1 slot, pushes 1 bool).
            // Mirrors EQ_D/NE_D/LT_D/GT_D/LE_D/GE_D but for TAG_F32-encoded
            // floats. Use pop_f32/push_nv(encode_bool) so operands are decoded
            // from the f32 nanbox tag rather than mis-read as i32.
            OpCode::EQ_F => {
                let b = task.ram.pop_f32();
                let a = task.ram.pop_f32();
                task.ram.push_nv(auto_val::encode_bool(a == b));
            }
            OpCode::NE_F => {
                let b = task.ram.pop_f32();
                let a = task.ram.pop_f32();
                task.ram.push_nv(auto_val::encode_bool(a != b));
            }
            OpCode::LT_F => {
                let b = task.ram.pop_f32();
                let a = task.ram.pop_f32();
                task.ram.push_nv(auto_val::encode_bool(a < b));
            }
            OpCode::GT_F => {
                let b = task.ram.pop_f32();
                let a = task.ram.pop_f32();
                task.ram.push_nv(auto_val::encode_bool(a > b));
            }
            OpCode::LE_F => {
                let b = task.ram.pop_f32();
                let a = task.ram.pop_f32();
                task.ram.push_nv(auto_val::encode_bool(a <= b));
            }
            OpCode::GE_F => {
                let b = task.ram.pop_f32();
                let a = task.ram.pop_f32();
                task.ram.push_nv(auto_val::encode_bool(a >= b));
            }

            // === Logical ===
            OpCode::AND => {
                // Plan 406: tag-first truthiness (was raw i32 magic compare)
                let b = task.ram.pop_nv();
                let a = task.ram.pop_nv();
                task.ram
                    .push_nv(auto_val::encode_bool(nv_truthy(a) && nv_truthy(b)));
            }
            OpCode::OR => {
                // Plan 406: tag-first truthiness (was raw i32 magic compare)
                let b = task.ram.pop_nv();
                let a = task.ram.pop_nv();
                task.ram
                    .push_nv(auto_val::encode_bool(nv_truthy(a) || nv_truthy(b)));
            }
            OpCode::XOR => {
                let b = task.ram.pop_i32();
                let a = task.ram.pop_i32();
                task.ram.push_i32(a ^ b);
            }

            // === Control Flow ===
            OpCode::JMP => {
                let offset = self.flash.read_i16(task.ip) as isize;
                task.ip += 2;

                let new_ip = (task.ip as isize) + offset;

                if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                    return Err(jump_oob_error(
                        task,
                        offset,
                        new_ip,
                        self.flash.memory.len(),
                    ));
                }

                task.ip = new_ip as usize;
            }
            OpCode::JMP_FAR => {
                let offset = self.flash.read_i32(task.ip) as isize;
                task.ip += 4;

                let new_ip = (task.ip as isize) + offset;

                if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                    return Err(jump_oob_error(
                        task,
                        offset,
                        new_ip,
                        self.flash.memory.len(),
                    ));
                }

                task.ip = new_ip as usize;
            }
            OpCode::JMP_IF_Z => {
                let offset = self.flash.read_i16(task.ip) as isize;
                task.ip += 2;

                // Plan 406: decode the condition by its nanbox tag instead
                // of comparing raw i32 magic values. Tagged bools (Plan 091)
                // are authoritative; 0 stays falsy for i32 values pushed by
                // legacy paths.
                let cond_nv = task.ram.pop_nv();
                if !nv_truthy(cond_nv) {
                    let new_ip = (task.ip as isize) + offset;
                    if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                        return Err(jump_oob_error(
                            task,
                            offset,
                            new_ip,
                            self.flash.memory.len(),
                        ));
                    }
                    task.ip = new_ip as usize;
                }
            }
            OpCode::JMP_IF_NZ => {
                let offset = self.flash.read_i16(task.ip) as isize;
                task.ip += 2;

                // Plan 406: see JMP_IF_Z — tag-first truthiness.
                let cond_nv = task.ram.pop_nv();
                if nv_truthy(cond_nv) {
                    let new_ip = (task.ip as isize) + offset;
                    if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                        return Err(jump_oob_error(
                            task,
                            offset,
                            new_ip,
                            self.flash.memory.len(),
                        ));
                    }
                    task.ip = new_ip as usize;
                }
            }

            // === Debug ===
            OpCode::SOURCE_LINE => {
                // Plan 199: Record current source line for debugging
                let line = self.flash.read_u16(task.ip);
                task.ip += 2;
                task.current_line = line as u32;
            }
            OpCode::HALT => {
                return Ok(StepResult::Terminated);
            }

            // === Plan 088 Phase 5: Reference Passing Instructions ===
            // Note: For Phase 5, references are implemented as var_index on the stack
            // LOAD_REF/LOAD_MUT_REF push the var_index, STORE_REF/STORE_MUT_REF use it
            OpCode::LOAD_REF => {
                // Plan 088 Phase 5: Load immutable reference
                // Format: var_index: u32
                let var_index = self.flash.read_u32(task.ip);
                task.ip += 4;

                // Push var_index onto stack as the "reference"
                // This will be used by subsequent STORE_REF or other operations
                task.ram.push_i32(var_index as i32);

                vm_debug!("DEBUG: LOAD_REF: var_index={}, bp={}", var_index, task.bp);
            }
            OpCode::STORE_REF => {
                // Plan 088 Phase 5: Store through immutable reference
                // Format: var_index: u32
                let var_index = self.flash.read_u32(task.ip);
                task.ip += 4;

                // Pop the value to store
                let val = task.ram.pop_i32();

                // Store to bp+1+var_index (same as LOAD_LOCAL logic)
                task.ram.write_i32(task.bp + 1 + var_index as usize, val);

                vm_debug!(
                    "DEBUG: STORE_REF: var_index={}, val={}, bp={}",
                    var_index,
                    val,
                    task.bp
                );
            }
            OpCode::LOAD_MUT_REF => {
                // Plan 088 Phase 5: Load mutable reference
                // Format: var_index: u32
                let var_index = self.flash.read_u32(task.ip);
                task.ip += 4;

                // Push var_index onto stack as the "mutable reference"
                task.ram.push_i32(var_index as i32);
            }
            OpCode::STORE_MUT_REF => {
                // Plan 088 Phase 5: Store through mutable reference
                // Format: var_index: u32
                let var_index = self.flash.read_u32(task.ip);
                task.ip += 4;

                // Pop the value to store
                let val = task.ram.pop_i32();

                // Store to bp+1+var_index (same as STORE_LOCAL logic)
                task.ram.write_i32(task.bp + 1 + var_index as usize, val);

                vm_debug!(
                    "DEBUG: STORE_MUT_REF: var_index={}, val={}, bp={}",
                    var_index,
                    val,
                    task.bp
                );
            }

            // === Plan 124: Async/Future/Await Instructions ===
            OpCode::CREATE_FUTURE => {
                // Create a Future value from an async block body.
                // Immediates: body_offset: u32, capture_count: u8,
                //             capture_name_idx: u16 (× capture_count)
                // Stack (popped in reverse): capture_count values pushed by
                // the AsyncBlock codegen (in declaration order).
                let body_offset = self.flash.read_u32(task.ip);
                task.ip += 4;
                let capture_count = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                // Read the capture name indices and pop the matching values.
                // The codegen pushes captures in declaration order; we read
                // names in the same order and pop in reverse, mirroring the
                // CLOSURE opcode's convention.
                let mut capture_names: Vec<String> = Vec::with_capacity(capture_count);
                let strings = self.strings.read().unwrap();
                for _ in 0..capture_count {
                    let name_idx = self.flash.read_u32(task.ip) as usize;
                    task.ip += 4;
                    let name = strings
                        .get(name_idx)
                        .map(|b| String::from_utf8_lossy(b).to_string())
                        .unwrap_or_default();
                    capture_names.push(name);
                }
                drop(strings);

                let mut captures: HashMap<String, auto_val::Value> = HashMap::new();
                for name in capture_names.into_iter().rev() {
                    let raw = task.ram.pop_i32();
                    captures.insert(name, auto_val::Value::Int(raw));
                }

                // Allocate a new future ID from VM's registry
                let future_id = self.future_id_gen.fetch_add(1, Ordering::SeqCst);

                // Create the future value with pending state
                let future = FutureValue {
                    body_offset,
                    state: FutureState::Pending,
                    result: None,
                    owner_task_id: task.id,
                    captures,
                    kind: FutureKind::Internal,
                };

                // Store in VM's future registry
                self.futures
                    .insert(future_id, Arc::new(RwLock::new(future)));

                // PLAN-776 R7-01：Future 句柄走 TAG_FUTURE **逐值来源**载荷
                //（普通 i32 永不携带 Future 身份——`(fid<<8)|0xF0` 位模式
                // 撞值即数据；legacy 位形态仅作 Value 往返降级兼容）。
                task.ram.push_nv(auto_val::encode_future(future_id));

                vm_debug!(
                    "DEBUG: CREATE_FUTURE: id={}, body_offset={}",
                    future_id,
                    body_offset
                );
            }
            OpCode::AWAIT_FUTURE => {
                // Wait for future completion (blocking)
                // Stack: [..., future_bits]
                // Returns: value when ready
                // PLAN-050 T9: pop_nv/push_nv 全程保标签——此前 pop_i32 把
                // await 的同步值（含 return None 的 null nv）按 raw i32
                // 重推（identity 路径），null 被拍平成 Int(-2147483647)。
                let future_nv = task.ram.pop_nv();

                // PLAN-776 R7-01：TAG_FUTURE 逐值来源优先；0xF0 位模式为
                // legacy 兼容臂（非 future 值原样回推——下方 else）。
                if let Some(future_id) = Self::decode_future_handle(future_nv) {
                    // Look up the future in the registry
                    if let Some(future_arc) = self.futures.get(&future_id) {
                        let future = future_arc.write().unwrap();

                        match future.state {
                            FutureState::Ready => {
                                // Future is ready - return the result
                                vm_debug!("DEBUG: AWAIT_FUTURE: id={} is ready", future_id);
                                let result = future.result.clone();
                                drop(future); // Release lock before push_value
                                if let Some(ref r) = result {
                                    Self::push_value(task, r, self);
                                } else {
                                    task.ram.push_nv(auto_val::encode_null()); // No result = nil
                                }
                            }
                            FutureState::Failed => {
                                // Future failed - return nil
                                vm_debug!("DEBUG: AWAIT_FUTURE: id={} failed", future_id);
                                drop(future);
                                task.ram.push_nv(auto_val::encode_null());
                            }
                            FutureState::Pending => {
                                // Plan 394: External futures suspend the task (wake source 6)
                                // instead of inlining a body. future_bits already popped; the
                                // wake path pushes the result. Do NOT pop again on resume.
                                if future.kind == FutureKind::External {
                                    vm_debug!(
                                        "DEBUG: AWAIT_FUTURE: id={} external pending, suspend",
                                        future_id
                                    );
                                    drop(future);
                                    task.waiting_future_id = Some(future_id);
                                    task.status = TaskStatus::Waiting("future".into());
                                    return Ok(StepResult::Yield);
                                }
                                // Plan 224: Return AwaitFuture signal for frame-level handling
                                vm_debug!("DEBUG: AWAIT_FUTURE: id={} is pending, returning AwaitFuture signal", future_id);
                                let body_offset = future.body_offset;
                                drop(future);
                                return Ok(StepResult::AwaitFuture {
                                    future_id,
                                    body_offset,
                                });
                            }
                        }
                    } else {
                        // Future not found - return nil
                        vm_debug!(
                            "DEBUG: AWAIT_FUTURE: id={} not found in registry",
                            future_id
                        );
                        task.ram.push_nv(auto_val::encode_null());
                    }
                } else {
                    // Not a future - push back as-is (identity, tag-preserving)
                    task.ram.push_nv(future_nv);
                }
            }
            OpCode::POLL_FUTURE => {
                // Non-blocking poll for future state
                // Stack: [..., future_handle]
                // Returns: (is_ready: bool, value_or_nil)
                let future_nv = task.ram.pop_nv();

                if let Some(future_id) = Self::decode_future_handle(future_nv) {
                    // Look up the future in the registry
                    if let Some(future_arc) = self.futures.get(&future_id) {
                        let future = future_arc.read().unwrap();

                        match future.state {
                            FutureState::Ready => {
                                // Push is_ready = 1
                                task.ram.push_i32(1);

                                // Push the result value
                                let result = future.result.clone();
                                drop(future); // Release lock before push_value
                                if let Some(ref r) = result {
                                    Self::push_value(task, r, self);
                                } else {
                                    task.ram.push_i32(0);
                                }
                            }
                            FutureState::Failed => {
                                // Push is_ready = 1 (failed is also "complete")
                                drop(future);
                                task.ram.push_i32(1);
                                // Push nil for failed
                                task.ram.push_i32(0);
                            }
                            FutureState::Pending => {
                                // Push is_ready = 0
                                drop(future);
                                task.ram.push_i32(0);
                                // Push nil (no value yet)
                                task.ram.push_i32(0);
                            }
                        }
                    } else {
                        // Future not found - return not ready
                        task.ram.push_i32(0);
                        task.ram.push_i32(0);
                    }
                } else {
                    // Not a future - return not ready
                    task.ram.push_i32(0);
                    task.ram.push_i32(0);
                }
            }

            _ => {
                // Unimplemented opcodes for Phase 1. Include the ip and
                // current fn name — mid-instruction landings (mispatched
                // jumps / call-resume offsets) otherwise give a bare 255
                // with no way to locate the site (Plan 423 P5).
                let fn_name = task
                    .call_stack
                    .last()
                    .and_then(|f| f.fn_name.clone())
                    .unwrap_or_else(|| "<root>".to_string());
                return Err(VMError::RuntimeError(format!(
                    "InvalidOpCode(0x{:02x}) at ip=0x{:04x} in {}",
                    op_byte, task.ip, fn_name
                )));
            }
        }
        Ok(StepResult::Continue)
    }
}
