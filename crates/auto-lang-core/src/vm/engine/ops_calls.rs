//! PLAN-843 结构拆分：run_one_instruction 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl AutoVM {
    /// PLAN-843 尾递归派发段（自 run_one_instruction 连续臂切片，臂体逐字保留）。
    pub(super) fn ops_calls(
        &self,
        task: &mut AutoTask,
        op: OpCode,
        op_byte: u8,
    ) -> Result<StepResult, VMError> {
        match op {
            OpCode::CALL_NAT => {
                let native_id = self.flash.read_u16(task.ip);
                task.ip += 2;
                self.native_interface
                    .verify_core_reference(native_id)
                    .map_err(VMError::RuntimeError)?;

                // Plan 419: native 死区结算 —— shim 弹掉的参数/receiver 槽
                // 在此统一释放(shim 内 pop 均为 raw,不计数)。shim 内
                // 推栈的堆 id 必须 rc_push(+1),与本结算配平。
                let sp_before_native = task.ram.sp;
                // DEBUG: Bypass native_interface for iterator.next
                if native_id == 112 {
                    // Save IP before call — if iterator_next yields
                    // (SSE waiting), we need to retry this CALL_NAT.
                    let pre_call_ip = task.ip;
                    crate::vm::native::shim_iterator_next(task, self)?;
                    // Plan 348: if iterator_next yielded (SSE stream waiting),
                    // back up IP so the CALL_NAT re-executes on resume,
                    // and signal Yield so the scheduler can run other tasks.
                    // PLAN-707: HTTPStream 句柄惰性消费臂同款重试协议。
                    if task.waiting_sse_stream_id.is_some() || task.waiting_http_stream_id.is_some()
                    {
                        // Plan 419: yield 重试路径不结算 —— 参数仍在死区
                        // 语义之外,重试时 shim 会重新 pop。
                        task.ip = pre_call_ip - 3;
                        return Ok(StepResult::Yield);
                    }
                } else if native_id == 1202 && task.cooperative_http_sleep {
                    // PLAN-696: SSE generator stepping is driven in bounded
                    // batches by the HTTP LocalSet. Preserve Time.sleep_ms
                    // semantics by parking this generator task until its
                    // deadline instead of blocking the LocalSet thread.
                    let ms = crate::vm::native::pop_arg_i32(task).max(0) as u64;
                    task.wake_time = Some(Instant::now() + std::time::Duration::from_millis(ms));
                    task.status = TaskStatus::Waiting(format!("sleep for {}ms", ms));
                    return Ok(StepResult::Yield);
                } else if native_id == 2300 {
                    // Plan 317 Phase 1: Task.spawn -> vm-aware (register AutoVM task)
                    crate::vm::ffi::stdlib::shim_task_spawn_vm(task, self)?;
                } else if native_id == 2301 {
                    // Plan 317 Phase 1: TaskHandle.send -> vm-aware (push to pending_messages)
                    crate::vm::ffi::stdlib::shim_task_send_vm(task, self)?;
                } else if let Some(shim) = self.native_interface.get(native_id).cloned() {
                    let pre_call_ip = task.ip;
                    // Plan 567 T05：对齐 CALL_NAT_COUNTED 的 560 T11 通道
                    // 统一——py 桥 FFI 错误转 RuntimeError；**非 FFI 错误
                    // 原样传播**（`if let Err(FFI)` 形态会把 RuntimeError/
                    // MissingNative 等静默吞掉——alloc_array 负尺寸回归
                    // 实证）。
                    match shim(task, self) {
                        Err(VMError::FFI(msg)) => {
                            return Err(VMError::RuntimeError(msg));
                        }
                        Err(other) => return Err(other),
                        Ok(()) => {}
                    }
                    // Plan 349 step 7: if HTTP json native yielded (async
                    // request pending), back up IP to retry CALL_NAT.
                    // PLAN-707 T-05: 流等待同款重试协议（shim 重入臂消费
                    // waiting_http_stream_id 后重新 pull）。
                    if task.waiting_http_request_id.is_some()
                        || task.waiting_http_stream_id.is_some()
                    {
                        task.ip = pre_call_ip - 3;
                        return Ok(StepResult::Yield);
                    }
                } else {
                    return Err(VMError::MissingNative(native_id));
                }
                // Plan 419: 死区 [sp_after, sp_before) 内引用槽结算清零。
                let sp_after_native = task.ram.sp;
                if sp_after_native < sp_before_native {
                    self.rc_release_slot_range(&mut task.ram, sp_after_native, sp_before_native);
                }
            }
            // Plan 369 Task 10: Python FFI native call with explicit arg count.
            // Reads native_id:u16, arg_count:u8. The arg_count is stashed on
            // the task so the Python shim pops the ACTUAL number of args pushed
            // at this call site (count cannot be baked into the shim because
            // C builtins defeat inspect.signature and struct.pack is variadic).
            OpCode::CALL_NAT_COUNTED => {
                let native_id = self.flash.read_u16(task.ip);
                task.ip += 2;
                let arg_count = self.flash.read_u8(task.ip);
                task.ip += 1;
                task.pending_native_arg_count = arg_count;
                self.native_interface
                    .verify_core_reference(native_id)
                    .map_err(VMError::RuntimeError)?;

                // Plan 419: 同 CALL_NAT 的死区结算。
                let sp_before_native = task.ram.sp;
                if let Some(shim) = self.native_interface.get(native_id).cloned() {
                    // Plan 560 T11（§4-3）：py 桥错误统一 RuntimeError
                    // 通道（catch 拦值一致绑定——FFI 标签在载荷里保留
                    // 上下文原文；PyException 前缀精化归 P560 债）。
                    // Plan 567 T05 加固：非 FFI 错误原样传播（同 CALL_NAT
                    // 臂的静默吞错回归教训）。
                    match shim(task, self) {
                        Err(VMError::FFI(msg)) => {
                            return Err(VMError::RuntimeError(msg));
                        }
                        Err(other) => return Err(other),
                        Ok(()) => {}
                    }
                } else {
                    return Err(VMError::MissingNative(native_id));
                }
                let sp_after_native = task.ram.sp;
                if sp_after_native < sp_before_native {
                    self.rc_release_slot_range(&mut task.ram, sp_after_native, sp_before_native);
                }
            }
            OpCode::RET => {
                // Spec: RET n_args
                let n_args = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                // Check if we're in the main task (bp == 0 means no caller)
                if task.bp == 0 {
                    // Main task returning - just terminate
                    return Ok(StepResult::Terminated);
                }

                // Under nanbox: preserve NanoValue type tag for string/bool/etc.
                let result_nv = task.ram.pop_nv();
                // PLAN-062 T12: 结果份额读走——帧清扫(按影子)后随值
                // 回落调用方槽位(write_nv 清影子后重新标记)。
                let result_stake = task.ram.take_stake_at(task.ram.sp);

                let old_bp = task.ram.read_i32(task.bp) as usize;
                let ret_ip = task.ram.read_i32(task.bp - 1) as usize;

                // PLAN-667 (F-01): 帧退栈——摘除刚返回帧的身份条目
                // （创建帧死亡的闭包自此被守卫拒绝）。
                task.pop_frame_id(task.bp);

                // Plan 071 Phase 5: Restore previous closure from saved_closure_id
                task.current_closure_id = task.saved_closure_id;

                // Plan 053 后续(ash-gui VM 稳定性)+ T-03 G-8 修正:bp - n_args
                // 可为 0 / 下溢(帧不匹配 / 主任务边界 RET / 适配器派发的
                // handler 0 参入栈)。守卫必须先于减法——原实现先减后判,
                // debug 构建(bp<n_args)在守卫之前的减法处直接 panic
                // (PLAN-093 实证:VMHTTP canvas 路由整 VM 崩溃即此)。
                // 夹到最小槽位 1 保命,丢一个返回值槽位远好于崩溃;warn 便于
                // 追踪帧不匹配的真因。
                let new_sp = if task.bp >= n_args {
                    task.bp - n_args
                } else {
                    0
                };
                if new_sp == 0 {
                    eprintln!(
                        "[VM-RET] underflow guard: bp={}, n_args={} (frame mismatch?)",
                        task.bp, n_args
                    );
                }
                let new_sp = new_sp.max(1);

                // Plan 419: RET 帧扫描 —— 释放帧区间 [new_sp-1, sp) 内每个
                // 引用槽(参数/局部/临时;返回值已 pop 转移给调用方,
                // 其落入的 new_sp-1 槽旧值也随覆盖而死亡)。
                let sp_after_result_pop = task.ram.sp;
                self.rc_release_slot_range(&mut task.ram, new_sp - 1, sp_after_result_pop);

                {
                    task.ram.write_nv(new_sp - 1, result_nv);
                    task.ram.sp = new_sp;
                    task.ram.write_nv(new_sp - 1, result_nv);
                    task.ram.mark_stake_at(new_sp - 1, result_stake);
                }

                task.bp = old_bp;
                task.ip = ret_ip;

                if let Some(frame) = task.call_stack.pop() {
                    task.current_fn_n_args = frame.old_fn_n_args;
                    task.current_fn_n_locals = frame.old_fn_n_locals;
                    // Plan 432 D26 修复:num_locals 随帧恢复——此前从不恢复,
                    // 用户函数返回后 native shim 的帧几何启发式(shim_list_new
                    // 的有参判定 bp+num_locals+2)读到被调者的局部数,sp 偶然
                    // 越过错误阈值即偷弹栈值(List.new 返回被污染的初始列表,
                    // 242:S2-432;构造参数内联调用丢失即 D25-① 同机制)。
                    task.num_locals = task.current_fn_n_locals;
                }
            }
            // Plan 377 §3.3: RET_D opcode 已删除（全值单槽化后恒用 RET）。

            // === Closures (Plan 071: Direct Capture) ===
            OpCode::CLOSURE => {
                // Stack: capture_count × value -> closure_id
                // Immediate: func_addr (u32), capture_count (u8), n_args (u8)
                let func_addr = self.flash.read_u32(task.ip);
                task.ip += 4;
                let capture_count = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;
                let n_args = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                vm_debug!("DEBUG CLOSURE: func_addr={}, capture_count={}, n_args={}, ip after header={}, sp before={}", func_addr, capture_count, n_args, task.ip, task.ram.sp);

                // Pop captured values from stack and build environment
                let mut env = HashMap::new();
                let mut capture_slots = HashMap::new();
                let mut param_abs: HashMap<String, usize> = HashMap::new();
                let creator_bp = task.bp; // Plan 385: 记录创建者的 bp
                for _i in 0..capture_count {
                    // Read variable name from string table (stored in reverse order)
                    let var_name_idx = self.flash.read_u32(task.ip) as usize;
                    task.ip += 4;

                    // Plan 385: Read slot offset (u16) — 0xFFFF means "no slot" (兼容旧字节码)
                    let slot_offset = self.flash.read_u16(task.ip) as usize;
                    task.ip += 2;

                    // Pop value from stack (values pushed in order, popped in reverse)
                    // PLAN-667 (F-01): pop_nv 保留类型标签（旧 pop_i32 把
                    // 字符串/对象捕获截成垃圾整数）；弹出槽的影子随值
                    // 转入 env 持有（env 条目隐式占一份；闭包死亡属
                    // 泄漏方向——安全纪律内，见 rc.rs 头注）。
                    let value_nv = task.ram.pop_nv();
                    let _env_stake = task.ram.take_stake_at(task.ram.sp);

                    // Look up variable name from string table (Plan 073: Now uses RwLock)
                    let strings = self.strings.read().unwrap();
                    let var_name_str = if let Some(var_name) = strings.get(var_name_idx) {
                        String::from_utf8_lossy(var_name).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index for captured variable: {}",
                            var_name_idx
                        )));
                    };
                    drop(strings);
                    env.insert(var_name_str.clone(), self.decode_tagged_nv(value_nv));
                    // Plan 385/454 E5a: 记录原始栈位置用于 by-reference 捕获。
                    // 0x8000 旗标 = 函数参数域(负向寻址),此处即用创建帧
                    // 的 n_args 解析成绝对槽位;普通局部沿用 bp+1+off。
                    if slot_offset != 0xFFFF {
                        if slot_offset & 0x8000 != 0 {
                            let real = (slot_offset & 0x3FFF) as usize;
                            let cur_n = task.current_fn_n_args;
                            if real < cur_n && cur_n > 0 {
                                let abs = task.bp - (cur_n - real + 1);
                                param_abs.insert(var_name_str.clone(), abs);
                            }
                        } else {
                            capture_slots.insert(var_name_str, (creator_bp, slot_offset));
                        }
                    }
                }

                // PLAN-667 (F-01): 嵌套闭包传递继承——在闭包 outer 执行
                // 体内创建 inner 时，inner 对 outer 已捕获变量的槽位解析
                // 必须沿用 outer 的帧锚（祖父帧），而非 inner 创建帧。
                // 旧实现按 inner 编译期作用域算槽位，读的是 outer 帧的
                // 垃圾（嵌套探针得 8 非 106 的根因）。
                if let Some(outer_id) = task.current_closure_id {
                    if let Some(outer) = self.closures.get(&outer_id) {
                        for (name, entry) in capture_slots.iter_mut() {
                            if let Some(&outer_entry) = outer.capture_slots.get(name) {
                                *entry = outer_entry;
                            }
                        }
                        for (name, abs) in param_abs.iter_mut() {
                            if let Some(&outer_abs) = outer.param_abs.get(name) {
                                *abs = outer_abs;
                            }
                        }
                    }
                }

                // PLAN-667 (F-01): 帧相对捕获记录创建帧身份
                // (task_id, frame_uid)，读写前校验存亡。
                let creator_frame = if capture_slots.is_empty() && param_abs.is_empty() {
                    None
                } else {
                    Some((task.id, task.current_frame_uid()))
                };

                // Create closure
                let closure_id = self.closure_id_gen.fetch_add(1, Ordering::Relaxed);
                let closure = Closure {
                    func_addr,
                    env,
                    n_args,
                    capture_slots,
                    param_abs,
                    creator_frame,
                };

                vm_debug!(
                    "DEBUG CLOSURE: created closure_id={}, ip after names={}, sp after={}",
                    closure_id,
                    task.ip,
                    task.ram.sp
                );

                self.closures.insert(closure_id, closure);
                task.ram.push_i32(closure_id as i32);
            }
            OpCode::CAPTURE_VAR => {
                // Stack: -> value
                // Immediate: var_name_idx (u32)
                // Load variable by name from current scope and push value
                let var_name_idx = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;

                // Plan 073: Now uses RwLock for strings access
                let strings = self.strings.read().unwrap();
                let _var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                    String::from_utf8_lossy(var_name_bytes).to_string()
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid string index: {}",
                        var_name_idx
                    )));
                };
                drop(strings);

                // Look up variable in local scope (from stack frame)
                // TODO: For MVP, we'll need to implement proper scope lookup
                // For now, push placeholder value
                task.ram.push_i32(0);
            }
            OpCode::LOAD_CAPTURED => {
                // Plan 071 Phase 5: Load captured variable from current closure
                // Stack: -> value (no longer pops closure_id)
                // Immediate: var_name_idx (u32)
                let var_name_idx = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;

                // Use current_closure_id instead of popping from stack
                let closure_id = task.current_closure_id.ok_or_else(|| {
                    VMError::RuntimeError(
                        "LOAD_CAPTURED called outside of closure context".to_string(),
                    )
                })?;

                // Plan 073: Now uses RwLock for strings access
                let strings = self.strings.read().unwrap();
                let var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                    String::from_utf8_lossy(var_name_bytes).to_string()
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid string index: {}",
                        var_name_idx
                    )));
                };
                drop(strings);

                if let Some(closure) = self.closures.get(&closure_id) {
                    // Plan 454 E5a: 参数域捕获优先(绝对槽,创建帧存活期内有效)
                    if let Some(&abs_slot) = closure.param_abs.get(var_name.as_str()) {
                        // PLAN-667 (F-01): 任何槽位读写前校验创建帧身份。
                        self.ensure_capture_frame_alive(task, &closure, &var_name, closure_id)?;
                        drop(closure);
                        let nv = task.ram.read_nv(abs_slot);
                        self.rc_push(task, nv);
                        return Ok(StepResult::Continue);
                    }
                    // Plan 385: 优先通过 capture_slots 读原始栈位置（by-reference）
                    if let Some(&(creator_bp, slot_offset)) =
                        closure.capture_slots.get(var_name.as_str())
                    {
                        // PLAN-667 (F-01): 任何槽位读写前校验创建帧身份。
                        self.ensure_capture_frame_alive(task, &closure, &var_name, closure_id)?;
                        let nv = task.ram.read_nv(creator_bp + 1 + slot_offset);
                        // Plan 419: copy-on-load(+1)。
                        drop(closure);
                        self.rc_push(task, nv);
                        return Ok(StepResult::Continue);
                    }
                    // Fallback: 从 env 读值副本（by-value，兼容旧字节码）
                    if let Some(value) = closure.env.get(var_name.as_str()) {
                        // Push value to stack
                        // PLAN-667 (F-01): env 持有的引用入栈 +1 并标记
                        // 影子（旧实现裸 push_i32 无影子——POP 按影子释放
                        // 不到，循环内每次 env 加载净漏一份）。
                        match value {
                            Value::Int(i) => {
                                task.ram.push_i32(*i);
                            }
                            Value::Bool(b) => {
                                task.ram.push_nv(auto_val::encode_bool(*b));
                            }
                            Value::Float(f) => {
                                task.ram.push_nv(auto_val::encode_f64(*f as f64));
                            }
                            Value::Double(d) => {
                                task.ram.push_nv(auto_val::encode_f64(*d));
                            }
                            Value::Nil | Value::Null => {
                                task.ram.push_nv(auto_val::encode_null());
                            }
                            Value::Str(s) => {
                                let idx = self.add_string(s.as_bytes().to_vec());
                                self.rc_push_str_idx(task, idx);
                            }
                            Value::VmRef(r) => {
                                self.rc_push_id(task, r.id as u64);
                            }
                            _ => task.ram.push_i32(0),
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Captured variable '{}' not found in closure {}",
                            var_name, closure_id
                        )));
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid closure ID: {}",
                        closure_id
                    )));
                }
            }
            OpCode::STORE_CAPTURED => {
                // Plan 071 Phase 5: Store to captured variable in current closure
                // Stack: value -> (no longer pops closure_id)
                // Immediate: var_name_idx (u32)
                let var_name_idx = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;

                let value_nv = task.ram.pop_nv();
                // PLAN-667 (F-01): 弹出槽影子随值转移（旧实现不取——
                // 陈旧影子滞留死位，槽位复用后 POP 误释他人份额）。
                let in_stake = task.ram.take_stake_at(task.ram.sp);

                // Use current_closure_id instead of popping from stack
                let closure_id = task.current_closure_id.ok_or_else(|| {
                    VMError::RuntimeError(
                        "STORE_CAPTURED called outside of closure context".to_string(),
                    )
                })?;

                // Plan 073: Now uses RwLock for strings access
                let strings = self.strings.read().unwrap();
                let var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                    String::from_utf8_lossy(var_name_bytes).to_string()
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid string index: {}",
                        var_name_idx
                    )));
                };
                drop(strings);

                // Plan 454 E5a: 参数域捕获优先(绝对槽直写)
                // PLAN-667 (F-01): 旧实现此处再 pop 一次（双 pop 吃掉
                // 无关栈值）且不转影子；改为单 pop + 影子转移 + 旧槽按
                // 影子释放（内容判释放会误杀与活 id 相等的整数）。
                if let Some(closure) = self.closures.get(&closure_id) {
                    if let Some(&abs_slot) = closure.param_abs.get(var_name.as_str()) {
                        self.ensure_capture_frame_alive(task, &closure, &var_name, closure_id)?;
                        drop(closure);
                        self.release_slot_old_value(task, abs_slot);
                        task.ram.write_nv(abs_slot, value_nv);
                        task.ram.mark_stake_at(abs_slot, in_stake);
                        return Ok(StepResult::Continue);
                    }
                    // Plan 385: 优先通过 capture_slots 写原始栈位置（by-reference）
                    if let Some(&(creator_bp, slot_offset)) =
                        closure.capture_slots.get(var_name.as_str())
                    {
                        self.ensure_capture_frame_alive(task, &closure, &var_name, closure_id)?;
                        drop(closure);
                        let addr = creator_bp + 1 + slot_offset;
                        self.release_slot_old_value(task, addr);
                        task.ram.write_nv(addr, value_nv);
                        task.ram.mark_stake_at(addr, in_stake);
                        return Ok(StepResult::Continue);
                    }
                    drop(closure);
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid closure ID: {}",
                        closure_id
                    )));
                }

                // Fallback: 写 closure.env（by-value，兼容旧字节码）
                let value = self.decode_tagged_nv(value_nv);
                if let Some(mut closure) = self.closures.get_mut(&closure_id) {
                    // Plan 419: env 旧值(堆引用)释放;新值转移。
                    if let Some(Value::VmRef(old)) = closure.env.get(var_name.as_str()) {
                        self.rc_release_id(old.id as u64);
                    }
                    // 字符串 env 条目为字节拷贝（无池份额），无需释放。
                    closure.env.insert(var_name, value);
                    // 新值的影子（若为堆引用）归 env 隐式持有——计数
                    // 已在入栈时发生（copy-on-load），env 覆盖/闭包死亡
                    // 时按上述约定释放/泄漏（安全方向）。
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid closure ID: {}",
                        closure_id
                    )));
                }
            }
            OpCode::CALL_CLOSURE => {
                // Stack: closure_id, [args...] -> result
                // Immediate: arg_count (u8)
                let _arg_count = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                // Plan 550 T04: null callee 守卫。正常模式 null callee 被
                // 静态解析在编译期拦下（E0401，见 p4 探针），本守卫覆盖
                // 动态/脚本路径：现状 pop_i32 把 TAG_NULL 解码成垃圾 id，
                // 落到 "Invalid closure ID" 无类型语义错误。
                if auto_val::is_null(task.ram.peek_nv(0)) {
                    return Err(VMError::RuntimeError(
                        "TypeError: 'NoneType' object is not callable".to_string(),
                    ));
                }

                let closure_id = task.ram.pop_i32() as u32;

                if let Some(_closure) = self.closures.get(&closure_id) {
                    // Plan 071 Phase 5: Set current closure for LOAD_CAPTURED access
                    let old_closure_id = task.current_closure_id;
                    task.current_closure_id = Some(closure_id);

                    // Set current_fn_n_args for LOAD_LOCAL parameter access
                    let prev_fn_n_args = task.current_fn_n_args;
                    let prev_fn_n_locals = task.current_fn_n_locals;
                    task.current_fn_n_args = _closure.n_args;

                    // Store old closure ID in task (not on stack) to avoid breaking parameter layout
                    // The RET opcode will restore it from task.saved_closure_id
                    task.saved_closure_id = old_closure_id;

                    // Push Return Address (IP)
                    task.ram.push_i32(task.ip as i32);
                    // Push Old Stack Frame (BP)
                    task.ram.push_i32(task.bp as i32);

                    // New BP points to the saved BP location (SP - 1)
                    task.bp = task.ram.sp - 1;
                    // PLAN-667 (F-01): 帧身份登记——帧实例 uid 发号。
                    task.push_frame_id(task.bp);

                    // PLAN-624 T-03: 同 call_closure 方法 —— 闭包激活入
                    // call_stack 一帧，闭包体 RET 弹自己的帧而非外层函数
                    // 的帧（帧错位使 current_fn_n_args 跨层泄漏）。
                    task.call_stack.push(crate::vm::task::CallFrame {
                        return_ip: task.ip,
                        old_bp: task.bp,
                        fn_name: Some(format!("<closure:{}>", closure_id)),
                        line: task.current_line,
                        old_fn_n_args: prev_fn_n_args,
                        old_fn_n_locals: prev_fn_n_locals,
                    });

                    // Jump to closure function
                    task.ip = _closure.func_addr as usize;
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid closure ID: {}",
                        closure_id
                    )));
                }
            }

            // === Concurrency ===
            OpCode::SPAWN => {
                let target = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;
                let arg_count = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(task.ram.pop_i32());
                }

                let new_task_id = self.spawn_task(target, 1024);

                if let Some(new_task_arc) = self.tasks.get(&new_task_id) {
                    if let Ok(mut new_task) = new_task_arc.try_lock() {
                        // Push args in reverse order (A, B, C)
                        for arg in args.into_iter().rev() {
                            new_task.ram.push_i32(arg);
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Failed to lock spawned task {}",
                            new_task_id
                        )));
                    }
                }
                task.ram.push_i32(new_task_id as i32);
            }
            OpCode::TASK_ID => {
                task.ram.push_i32(task.id as i32);
            }
            OpCode::YIELD_TASK => {
                return Ok(StepResult::Yield);
            }
            OpCode::YIELD_VAL => {
                // Plan 321: Generator yield. The yielded value is already
                // on top of stack (compiled by codegen before YIELD_VAL).
                // Signal the generator driver that a value was yielded.
                // The task's ip now points PAST the YIELD_VAL instruction,
                // so on resume the generator continues from the next stmt.
                return Ok(StepResult::GeneratorYield);
            }
            OpCode::CREATE_GENERATOR => {
                // Plan 321: Create a generator iterator from inline operands.
                // Bytecode: CREATE_GENERATOR, func_addr:u32, n_args:u8
                // Result: pushes iterator_id:i32
                let func_addr = self.flash.read_u32(task.ip);
                task.ip += 4;
                let n_args = self.flash.read_u8(task.ip);
                task.ip += 1;

                // Plan 417-D2: the codegen pushed the generator call's
                // args onto the caller stack right before this opcode.
                // Pop them into the state (declaration order) so the
                // first next() seeds the generator frame — previously
                // they stayed on the caller stack and parameterized
                // generators underflowed at RET with garbage params.
                let gen_state = self.new_generator_state(task, func_addr, n_args);
                let iter_id = self.register_iterator(Iterator::Generator(gen_state));
                task.ram.push_i32(iter_id as i32);
            }
            OpCode::WRAP_SSE_RESOURCE => {
                // PLAN-776 R2-01/R5-01：generator 产出在**生产点**建立身份
                //（codegen 对 ~Iter/~Stream 声明 fn 内的 CREATE_GENERATOR 后
                // 插入；iterator 注册表校验后装箱 Sse 凭据）。幂等：已是
                // 凭据对象（TAG_OBJECT）则原样回推——双包装/凭据回流安全。
                let nv = task.ram.pop_nv();
                if auto_val::is_object(nv) {
                    task.ram.push_nv(nv);
                } else {
                    let iter_id = auto_val::decode_i32(nv) as u32;
                    if !self.iterators.contains_key(&iter_id) {
                        return Err(VMError::RuntimeError(format!(
                            "wrap_sse: iterator {iter_id} not registered"
                        )));
                    }
                    crate::vm::ffi::http_return_resource::push_sse_resource(task, self, iter_id)?;
                }
            }
            OpCode::SLEEP => {
                let ms = self.flash.read_u32(task.ip) as u64;
                task.ip += 4;

                // Set wake time
                task.wake_time = Some(Instant::now() + std::time::Duration::from_millis(ms));
                task.status = TaskStatus::Waiting(format!("sleep for {}ms", ms));
                return Ok(StepResult::Yield);
            }
            OpCode::JOIN => {
                let target_task_id = task.ram.pop_i32() as u64;

                // Get Arc first (must outlive the try_lock call)
                let target_task_opt: Option<Arc<Mutex<AutoTask>>> =
                    self.tasks.get(&target_task_id).map(|r| r.value().clone());

                let join_result: Option<(bool, i32)> = match &target_task_opt {
                    Some(target_task) => {
                        match target_task.try_lock() {
                            Ok(target) => {
                                if target.status == TaskStatus::Terminated {
                                    Some((true, target.ram.top().unwrap_or(0)))
                                } else {
                                    Some((false, 0))
                                }
                            }
                            Err(_) => None, // Couldn't lock
                        }
                    }
                    None => Some((true, 0)), // Task not found, return 0
                };

                match join_result {
                    Some((true, result)) => {
                        task.ram.push_i32(result);
                    }
                    Some((false, _)) | None => {
                        // Task still running or lock failed, yield and retry
                        task.ip -= 1;
                        task.ram.push_i32(target_task_id as i32);
                        return Ok(StepResult::Yield);
                    }
                }
            }
            OpCode::CHAN_NEW => {
                let id = self.channel_id_gen.fetch_add(1, Ordering::Relaxed) as u32;
                let chan = Arc::new(AutoChannel::new(id, 16));
                self.channels.insert(id, chan);
                task.ram.push_i32(id as i32);
            }
            OpCode::SEND => {
                let data = task.ram.pop_i32();
                let chan_id = task.ram.pop_i32() as u32;
                let mut success = false;
                let mut closed = false;

                if let Some(chan_ref) = self.channels.get(&chan_id) {
                    let chan = chan_ref.value().clone();
                    drop(chan_ref);
                    match chan.tx.try_send(data) {
                        Ok(_) => success = true,
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            // Channel full
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            closed = true;
                        }
                    }
                } else {
                    closed = true;
                }

                if !success && !closed {
                    // Retry later
                    task.ip -= 1;
                    task.ram.push_i32(chan_id as i32);
                    task.ram.push_i32(data);
                    return Ok(StepResult::Yield);
                }
            }
            OpCode::RECV => {
                let chan_id = task.ram.pop_i32() as u32;
                let mut success = false;
                let mut val = 0;
                let mut closed = false;
                match self.channels.get(&chan_id) {
                    Some(chan_ref) => {
                        let chan = chan_ref.value().clone();
                        drop(chan_ref);
                        // Lock rx
                        let mut rx = chan.rx.lock().unwrap();
                        match rx.try_recv() {
                            Ok(v) => {
                                val = v;
                                success = true;
                            }
                            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                                // Empty
                            }
                            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                                closed = true;
                            }
                        }
                    }
                    None => {
                        closed = true; // Invalid = closed
                        val = -1; // Error code?
                    }
                }

                if success {
                    task.ram.push_i32(val);
                } else if closed {
                    task.ram.push_i32(0); // TODO: Null/None
                } else {
                    // Empty, Retry
                    task.ip -= 1;
                    task.ram.push_i32(chan_id as i32);
                    return Ok(StepResult::Yield);
                }
            }
            OpCode::TRY_RECV => {
                let chan_id = task.ram.pop_i32() as u32;
                let mut success = false;
                let mut val = 0;
                let mut closed = false;
                match self.channels.get(&chan_id) {
                    Some(chan_ref) => {
                        let chan = chan_ref.value().clone();
                        drop(chan_ref);
                        // Lock rx
                        let mut rx = chan.rx.lock().unwrap();
                        match rx.try_recv() {
                            Ok(v) => {
                                val = v;
                                success = true;
                            }
                            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                                // Empty - return 0 without blocking
                            }
                            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                                closed = true;
                            }
                        }
                    }
                    None => {
                        closed = true; // Invalid = closed
                        val = -1; // Error code?
                    }
                }

                if success {
                    task.ram.push_i32(val);
                } else if closed {
                    task.ram.push_i32(0); // TODO: Null/None
                } else {
                    // Empty channel - return 0 immediately (non-blocking)
                    task.ram.push_i32(0);
                }
            }

            // Plan 126: SPAWN_GO - fire-and-forget spawn
            // Pops a single value from the stack (a Future created by an
            // async block `~{ ... }.go`, encoded as (future_id << 8) | 0xF0,
            // or a closure id) and spawns it as a background task.
            // Returns void (no value pushed to stack).
            // Stack layout: [..., future_or_closure:i32]
            OpCode::SPAWN_GO => {
                // Pop the single value pushed by the Expr::Go codegen.
                let value_nv = task.ram.pop_nv();

                // PLAN-776 R7-01：TAG_FUTURE 逐值来源优先（legacy 位模式
                // 兼容）；非 future 值按 closure id 处理（原语义）。
                if let Some(future_id) = Self::decode_future_handle(value_nv) {
                    let value = auto_val::decode_i32(value_nv);
                    // Look up the future to obtain its body offset and
                    // captures, then spawn a background task that starts at
                    // the body offset.
                    if let Some(future_arc) = self.futures.get(&future_id) {
                        let (body_offset, captures) = {
                            let future = future_arc.read().unwrap();
                            (future.body_offset as usize, future.captures.clone())
                        };
                        // Plan 348 Task 22: only spawn when the body offset
                        // is a real, in-range bytecode address. The new
                        // out-of-line AsyncBlock codegen always supplies a
                        // non-zero address; the guard keeps us safe if a
                        // future was constructed through some other path.
                        if body_offset != 0 && body_offset < self.flash.memory.len() {
                            let new_task_id = self.spawn_task(body_offset, 1024);
                            // Install captures: the spawned task has a fresh
                            // RAM, so any LOAD_CAPTURED the body emits must
                            // resolve via a Closure entry. We synthesize one
                            // and point the task's current_closure_id at it.
                            if !captures.is_empty() {
                                let closure_id =
                                    self.closure_id_gen.fetch_add(1, Ordering::Relaxed);
                                self.closures.insert(
                                    closure_id,
                                    Closure {
                                        func_addr: body_offset as u32,
                                        env: captures,
                                        n_args: 0,
                                        capture_slots: HashMap::new(),
                                        param_abs: HashMap::new(),
                                        creator_frame: None, // PLAN-667: 合成闭包无帧相对捕获
                                    },
                                );
                                if let Some(task_guard) = self.tasks.get(&new_task_id) {
                                    if let Ok(mut spawned) = task_guard.try_lock() {
                                        spawned.current_closure_id = Some(closure_id);
                                        spawned.saved_closure_id = Some(closure_id);
                                    }
                                }
                            }
                        }
                        // Fire-and-forget: we do not push task_id back and
                        // do not wait for the spawned task.
                    }
                } else {
                    // Otherwise treat the value as a closure id. Spawn a
                    // background task at the closure's function address so
                    // that the closure's body runs concurrently.
                    let closure_id = auto_val::decode_i32(value_nv) as u32;
                    if let Some(closure_guard) = self.closures.get(&closure_id) {
                        let func_addr = closure_guard.func_addr as usize;
                        let _new_task_id = self.spawn_task(func_addr, 1024);
                    }
                    // If neither a valid future nor closure, silently drop
                    // the value (fire-and-forget; never crash on bad input).
                }
            }

            // Plan 127: TASK_LOOP - enter task message processing loop
            // This opcode marks the start of a task's message handling loop.
            // The task will wait for messages and dispatch them to handlers.
            // Stack layout: [task_type_str_idx:i32]
            OpCode::TASK_LOOP => {
                // Get task type string index
                let task_type_idx = task.ram.pop_i32() as u16;
                let strings = self.strings.read().unwrap();
                let task_type = strings
                    .get(task_type_idx as usize)
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .unwrap_or_default();
                drop(strings);

                // Store that this task is now in message loop mode
                task.in_message_loop = true;
                task.task_type_name = Some(task_type.clone());

                // VM bug fix: #start's default initializers have run (or were
                // skipped for spawn-injected fields). Clear the lock set so
                // message handlers can freely STORE_STATE_FIELD on any field.
                task.locked_state_fields.clear();

                // Set task to waiting state (will be woken when messages arrive)
                task.status = TaskStatus::Waiting("message_loop".to_string());
                // Plan 390 §15 G2-refactor (方案 B): 记录稳定的重 park ip（TASK_LOOP
                // 之后的 RET）。handler RET 后的 RET-catch 会把 ip 重置到这里，使
                // actor 干净地等待下一条消息，不依赖 handler RET 恢复的 ip（那个 ip
                // 指向 #start 尾部 RET，执行后会推进 ip 污染下次 wake 的 saved_ip）。
                task.park_ip = Some(task.ip);

                if crate::is_vm_debug() {
                    eprintln!(
                        "[TASK_LOOP] Task {} entering message loop for type {}",
                        task.id, task_type
                    );
                }

                // Plan 317 Phase 1: yield immediately so the trailing RET
                // (bp==0 for a spawned task) doesn't terminate the task.
                // run_task_loop will re-run this task when a message arrives
                // (its ip still points at the RET after TASK_LOOP, but the
                // message wake path resets ip to the handler body_offset).
                return Ok(StepResult::Yield);
            }

            // Plan 127: HANDLE_MSG - dispatch message to matched handler
            // The message value is on the stack, handlers are looked up from metadata.
            // Stack layout: [..., msg_value:i32]
            // Pushes: handler_found:bool, handler_offset:i32 (if found)
            OpCode::HANDLE_MSG => {
                // Get message value from stack
                let msg_value = task.ram.pop_i32();

                // Get task type for handler lookup
                let task_type = task.task_type_name.clone().unwrap_or_default();
                let msg = auto_val::Value::Int(msg_value);

                // Plan 317 Phase 1: delegate to shared matcher (now also
                // covers the `else` fallback via exports).
                if let Some((body_offset, has_context)) = self.find_handler_offset(&task_type, &msg)
                {
                    task.ram.push_i32(1); // true - handler found
                    task.ram.push_i32(body_offset as i32);
                    task.current_handler_has_context = has_context;
                } else {
                    task.ram.push_i32(0); // false - no handler
                    if crate::is_vm_debug() {
                        eprintln!(
                            "[HANDLE_MSG] No handler found for message {} in task {}",
                            msg_value, task_type
                        );
                    }
                }
            }

            // Plan 127: REPLY - send reply via current MessageContext
            // Stack layout: [..., reply_value:i32]
            // Pops the reply value and sends it through the reply channel.
            OpCode::REPLY => {
                // Get reply value from stack
                let reply_value = task.ram.pop_i32();

                // Check if we have a current message context with reply capability
                if let Some(ref ctx) = task.current_msg_context {
                    // Convert i32 to Value for reply
                    let value = auto_val::Value::Int(reply_value);
                    match ctx.reply(value) {
                        Ok(()) => {
                            if crate::is_vm_debug() {
                                eprintln!("[REPLY] Sent reply value {}", reply_value);
                            }
                        }
                        Err(_e) => {
                            if crate::is_vm_debug() {
                                eprintln!("[REPLY] Failed to send reply: {}", _e);
                            }
                        }
                    }
                } else {
                    if crate::is_vm_debug() {
                        eprintln!("[REPLY] No message context available for reply");
                    }
                }
            }

            // === Local Variables ===
            //
            // Stack frame layout (Plan 080):
            //   [..., ret_ip, old_bp, local0, local1, ..., temps...]
            //                     bp
            //   Local variables are at bp+1, bp+2, ... (not bp+0!)
            //
            // For main task (bp=1):
            //   [unused, local0, local1, ..., temps...]
            //      bp-1     bp     bp+1
            //
            OpCode::LOAD_LOCAL => {
                let idx = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;
                if crate::is_vm_trace_ops() {
                    eprintln!("[VMOP] LOAD_LOCAL idx=0x{:02x}", idx);
                }

                // Plan 087 Phase 3: Check if this is a parameter (idx >= 0x80)
                if idx >= 0x80 {
                    // Parameter: decode parameter index
                    let param_idx = idx - 0x80;
                    let n_args = task.current_fn_n_args;
                    // Guard against param_idx > n_args (corrupt frame) and
                    // bp underflow. Returning Nil avoids a panic that would
                    // crash the whole UI; the caller sees an unset param.
                    if param_idx >= n_args {
                        task.ram.push_nv(auto_val::encode_null());
                    } else {
                        let offset = n_args - param_idx;
                        let actual_offset = offset + 1;
                        if actual_offset > task.bp {
                            task.ram.push_nv(auto_val::encode_null());
                        } else {
                            // Plan 419: copy-on-load —— 加载引用值 +1。
                            // PLAN-667 (F-07): 源槽份额出处门。
                            let slot = task.bp - actual_offset;
                            let nv = task.ram.read_nv(slot);
                            let ss = task.ram.stake_at(slot);
                            self.rc_push_slot(task, nv, ss);
                        }
                    }
                } else {
                    // Local variable: load from bp+1+idx
                    // Plan 419: copy-on-load。
                    // PLAN-667 (F-07): 源槽份额出处门。
                    let slot = task.bp + 1 + idx;
                    let nv = task.ram.read_nv(slot);
                    let ss = task.ram.stake_at(slot);
                    self.rc_push_slot(task, nv, ss);
                }
            }
            OpCode::STORE_LOCAL => {
                let idx = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;
                if crate::is_vm_trace_ops() {
                    eprintln!("[VMOP] STORE_LOCAL idx=0x{:02x}", idx);
                }
                let val_nv = task.ram.pop_nv();
                // PLAN-062 T12: 份额随值转移——读走弹出槽影子,写槽
                // 重新标记;旧槽值按其影子释放(内容判定废除)。
                let transferred = task.ram.take_stake_at(task.ram.sp);

                // Plan 088 Phase 4: Check if this is a parameter (idx >= 0x80)
                if idx >= 0x80 {
                    let param_idx = idx - 0x80;
                    let n_args = task.current_fn_n_args;
                    let offset = n_args - param_idx;
                    let actual_offset = offset + 1;
                    // Plan 419: 槽内旧值 -1;新值自栈转移进槽(计数不变)。
                    let addr = task.bp - actual_offset;
                    self.release_slot_old_value(task, addr);
                    task.ram.write_nv(addr, val_nv);
                    task.ram.mark_stake_at(addr, transferred);
                } else {
                    let addr = task.bp + 1 + idx;
                    // Plan 419: 覆盖赋值旧值 -1(overwrite_drop 语义)。
                    self.release_slot_old_value(task, addr);
                    task.ram.write_nv(addr, val_nv);
                    task.ram.mark_stake_at(addr, transferred);
                }
            }
            OpCode::LOAD_LOC_0 => {
                // Plan 419: copy-on-load。PLAN-667 (F-07): 源槽份额出处门。
                let slot = task.bp + 1;
                let nv = task.ram.read_nv(slot);
                let ss = task.ram.stake_at(slot);
                self.rc_push_slot(task, nv, ss);
            }
            // Plan 317: Actor state field access (absolute, bp-independent).
            // state_vars is a Vec on AutoTask; field_idx is assigned by codegen.
            OpCode::LOAD_STATE_FIELD => {
                let field_idx = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;
                let nv = task.state_vars.get(field_idx).copied().unwrap_or(0);
                // Plan 419: copy-on-load。state_vars 条目恒持有,tagged 引用
                // 正常计份;裸 i32 由 rc_push_slot 出处门处置。
                self.rc_push_slot(task, nv, u64::MAX);
            }
            _ => return self.ops_flow(task, op, op_byte),
        }
        Ok(StepResult::Continue)
    }
}
