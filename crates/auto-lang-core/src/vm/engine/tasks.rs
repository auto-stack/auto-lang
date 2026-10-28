//! PLAN-843 结构拆分（自 engine.rs 原样搬移；算法/行为零变化）。
use super::*;

impl AutoVM {
    /// Remove a task and release resources owned by its lifetime.
    pub fn remove_task(&self, task_id: TaskId) {
        // PLAN-776 R1-F04：任务终结清偿 HTTP 返回资源凭据的栈份额——
        // dispatch（含 middleware/parked/shutdown/cancel 各终态）统一经此
        // remove；Internal `~{}` body 栈与异常退出路径的残留引用在此收口
        //（只清 HttpReturnResource，通用对象栈清理是 P776-D1 既有边界）。
        // try_lock 失败（任务仍在执行——remove 本身即异常路径）跳过不阻塞。
        if let Some(t_arc) = self.tasks.get(&task_id) {
            if let Ok(t) = t_arc.try_lock() {
                crate::vm::ffi::http_return_resource::release_task_credentials(self, &t);
            }
        }
        self.tasks.remove(&task_id);
        self.sqlite_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|_, (owner, _)| *owner != task_id);
    }

    /// PLAN-779: 实例级 frame provider 注入/覆盖（宿主 adapter、外部
    /// consumer、测试装配用）。覆盖构造期的进程默认快照；只影响本 VM，
    /// 不触碰绑定身份（revision/proof 不失效）。读取面=shim 9918/9919。
    pub fn set_frame_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::FrameProvider>,
    ) {
        self.native_interface.set_frame_provider(provider);
    }

    /// PLAN-779: 本 VM 当前 frame provider（None=未捕获 0 语义）。
    pub fn frame_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::FrameProvider>> {
        self.native_interface.frame_provider()
    }

    /// Plan 317 Phase 1: Find the bytecode offset of the handler for a message.
    ///
    /// Used by both HANDLE_MSG (opcode path) and run_task_loop (message wake
    /// path). Looks up the task type's handler table, matches the message
    /// against patterns in order, and returns the matching handler's
    /// (body_offset, has_context). If no pattern matches, falls back to the
    /// task's `else` handler (exported as `"{task_type}#else"` by codegen).
    /// Returns None if no handler at all (no table, no else).
    pub fn find_handler_offset(
        &self,
        task_type: &str,
        msg: &auto_val::Value,
    ) -> Option<(u32, bool)> {
        if let Some(table) = self.task_handler_registry.get_table(task_type) {
            for handler in table.get_handlers() {
                if let Some(pattern) = table.get_pattern(handler.pattern_idx) {
                    if self.match_message_pattern_vm(msg, pattern, &table.string_pool) {
                        return Some((handler.body_offset, handler.has_context));
                    }
                }
            }
        }
        // Fallback: else handler (codegen stores it in exports as "{task_type}#else").
        let else_key = format!("{}#else", task_type);
        if let Some(&offset) = self.flash.exports_by_name.get(&else_key) {
            return Some((offset, false));
        }
        None
    }

    /// Spawn a new task starting at the given instruction pointer
    /// Returns the TaskId
    pub fn spawn_task(&self, start_ip: usize, ram_size: usize) -> TaskId {
        let id = self.id_gen.fetch_add(1, Ordering::Relaxed);
        let task = AutoTask::new(id, ram_size, start_ip);
        self.tasks.insert(id, Arc::new(Mutex::new(task)));
        id
    }

    /// Call an Auto closure from native code.
    ///
    /// # Arguments
    /// * `task` - Current task (mutable)
    /// * `closure_id` - ID of closure to call
    /// * `arg_count` - Number of arguments already on stack
    ///
    /// # Stack effect
    /// Pops `arg_count` args + closure_id from stack, pushes result.
    /// After return: result is on top of stack.
    pub fn call_closure(
        &self,
        task: &mut AutoTask,
        closure_id: u32,
        _arg_count: usize,
    ) -> Result<(), VMError> {
        // 1. Clone closure data (can't hold DashMap guard across yields)
        let closure = match self.closures.get(&closure_id) {
            Some(guard) => guard.clone(),
            None => {
                return Err(VMError::RuntimeError(format!(
                    "Invalid closure ID: {}",
                    closure_id
                )))
            }
        };

        // 2. Save current state
        let saved_ip = task.ip;
        let saved_bp = task.bp;
        let saved_closure_id = task.current_closure_id;
        let saved_fn_n_args = task.current_fn_n_args;
        let saved_fn_n_locals = task.current_fn_n_locals;
        let saved_saved_closure_id = task.saved_closure_id;

        // 3. Setup closure context (mirrors CALL_CLOSURE opcode logic)
        task.current_closure_id = Some(closure_id);
        task.current_fn_n_args = closure.n_args;
        task.saved_closure_id = saved_closure_id;

        // 4. Setup stack frame
        task.ram.push_i32(saved_ip as i32); // Return address
        task.ram.push_i32(saved_bp as i32); // Old BP
        task.bp = task.ram.sp - 1;
        // PLAN-667 (F-01): 帧身份登记——帧实例 uid 发号。
        task.push_frame_id(task.bp);

        // PLAN-624 T-03: 闭包激活必须入 call_stack 一帧 —— 闭包体末端 RET
        // 无条件弹一帧(CALL 帧协议)；此前本方法只推 ram 帧、不推 CallFrame，
        // 闭包 RET 弹走的是**外层函数**的帧(call_stack 错位一层)。后果链
        // (plan624 P2 实证)：store handler 内 `.tabs.find(λ)` → λ RET 偷走
        // store 帧 → store 自身 RET 弹空栈、不恢复 → 最深被调帧的
        // current_fn_n_args 泄漏进外层(0 参 widget handler 读 __state 时
        // 偏移越界走 NULL 守卫 → SET_FIELD "Invalid object ID" 崩)。
        // 入帧后 λ RET 弹自己的帧，外层帧原样保留，RET 恢复链闭合。
        task.call_stack.push(crate::vm::task::CallFrame {
            return_ip: saved_ip,
            old_bp: task.bp,
            fn_name: Some(format!("<closure:{}>", closure_id)),
            line: task.current_line,
            old_fn_n_args: saved_fn_n_args,
            old_fn_n_locals: saved_fn_n_locals,
        });

        // 5. Jump to closure body
        task.ip = closure.func_addr as usize;

        // 6. Execute until closure returns (BP restored to saved_bp)
        let budget = 1_000_000;
        for _ in 0..budget {
            let step = match self.run_one_instruction(task) {
                Ok(s) => s,
                Err(e) => {
                    // Plan 010 (MS3-A): try/catch interception.
                    if self.intercept_error(task, &e) {
                        continue;
                    }
                    // Restore state even on error。Err 出口上闭包帧未被 RET
                    // 弹出(RET 未执行)——手动退帧，否则外层帧账错位。
                    task.call_stack.pop();
                    task.current_closure_id = saved_closure_id;
                    task.current_fn_n_args = saved_fn_n_args;
                    task.current_fn_n_locals = saved_fn_n_locals;
                    return Err(e);
                }
            };
            match step {
                StepResult::Continue => {
                    if task.bp == saved_bp {
                        break;
                    }
                    continue;
                }
                StepResult::Terminated => {
                    // Restore state even on error(同 Err 出口退帧)
                    task.call_stack.pop();
                    task.current_closure_id = saved_closure_id;
                    task.current_fn_n_args = saved_fn_n_args;
                    task.current_fn_n_locals = saved_fn_n_locals;
                    return Err(VMError::RuntimeError(
                        "Closure execution terminated unexpectedly".into(),
                    ));
                }
                StepResult::Yield => {
                    // In call_closure context, Yield just means "continue after pause"
                    continue;
                }
                StepResult::GeneratorYield => {
                    // Plan 321: YIELD_VAL in closure context — treat like Yield
                    // (continue past the yield). Proper suspension only via
                    // Iterator::Generator driver.
                    continue;
                }
                StepResult::AwaitFuture {
                    future_id,
                    body_offset,
                } => {
                    // Handle await within closure execution(Err 同上退帧)
                    if let Err(e) = self.handle_await_future(task, future_id, body_offset) {
                        task.call_stack.pop();
                        task.current_closure_id = saved_closure_id;
                        task.current_fn_n_args = saved_fn_n_args;
                        task.current_fn_n_locals = saved_fn_n_locals;
                        return Err(e);
                    }
                }
            }
        }

        // 7. Restore non-stack state
        task.current_closure_id = saved_closure_id;
        task.current_fn_n_args = saved_fn_n_args;
        task.saved_closure_id = saved_saved_closure_id;

        Ok(())
    }

    /// PLAN-705 T-04/SD-01: 段驱动的闭包调用（`__axum:` 路由的 park 形态）。
    ///
    /// 帧建立与 [`Self::call_closure`] 逐字节同（闭包上下文/ram 返回帧/
    /// CallFrame 入栈/PLAN-667 帧身份），驱动改走
    /// [`Self::drive_handler_segment`]（10M 预算 + runaway 守卫 + try/catch
    /// 拦截 + **Yield → Parked**）：闭包内异步 HTTP/External Future 等待
    /// 不再同步自旋（旧 call_closure 对 Yield 一律 continue——等待期间
    /// 占住执行线程，budget 耗尽后静默无结果返回）。恢复走
    /// [`Self::resume_fn_by_name_segment`]（ParkedSegment.fn_name 仅用于
    /// 诊断，"<closure:id>" 形态对驱动透明）。
    pub fn call_closure_segment(
        &self,
        task: &mut AutoTask,
        closure_id: u32,
        _arg_count: usize,
    ) -> SegmentOutcome {
        // 1. Clone closure data (can't hold DashMap guard across yields)
        let closure = match self.closures.get(&closure_id) {
            Some(guard) => guard.clone(),
            None => {
                return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                    "Invalid closure ID: {}",
                    closure_id
                ))))
            }
        };

        // 2. Save current state
        let saved_ip = task.ip;
        let saved_bp = task.bp;
        let saved_closure_id = task.current_closure_id;
        let saved_fn_n_args = task.current_fn_n_args;
        let saved_fn_n_locals = task.current_fn_n_locals;
        let saved_saved_closure_id = task.saved_closure_id;

        // 3-5. Setup closure context + stack frame + jump（同 call_closure）
        task.current_closure_id = Some(closure_id);
        task.current_fn_n_args = closure.n_args;
        task.saved_closure_id = saved_closure_id;
        task.ram.push_i32(saved_ip as i32); // Return address
        task.ram.push_i32(saved_bp as i32); // Old BP
        task.bp = task.ram.sp - 1;
        task.push_frame_id(task.bp);
        task.call_stack.push(crate::vm::task::CallFrame {
            return_ip: saved_ip,
            old_bp: task.bp,
            fn_name: Some(format!("<closure:{}>", closure_id)),
            line: task.current_line,
            old_fn_n_args: saved_fn_n_args,
            old_fn_n_locals: saved_fn_n_locals,
        });
        task.ip = closure.func_addr as usize;

        // 6. Segment drive（Parked 时 task 状态原样保留供恢复）。
        // PLAN-781 T-15: 唯一生产调用方 = http_server axum closure 臂
        //（决策报告 D-2 入口矩阵）——预算档随 policy 快照。
        task.segment_no_busy_wait = true;
        let budget = match self.http_step_budget {
            Some(cap) => DriveBudget::LegacyHttp(cap),
            None => DriveBudget::Legacy,
        };
        let outcome = self.drive_handler_segment(
            task,
            &format!("<closure:{}>", closure_id),
            saved_bp,
            saved_fn_n_args,
            false,
            budget,
        );
        if matches!(outcome, SegmentOutcome::Completed(_)) {
            task.segment_no_busy_wait = false;
        }
        outcome
    }

    /// Plan 312: Call a named VM function from native code (HTTP handler dispatch).
    ///
    /// Looks up `fn_name` in `exports_by_name`, pushes a stack frame (mirroring
    /// CALL opcode), executes until RET, and leaves the result on top of stack.
    /// Arguments must be pushed onto `task.ram` BEFORE calling this method,
    /// in left-to-right order (arg0 at lower address, argN-1 on top).
    ///
    /// After return: result is on top of stack (`task.ram.pop_nv()`).
    /// Plan 423 P5:runaway 增量判定(字符串池条目数 / 堆对象数)。
    fn runaway_growth_exceeded(&self, baseline_strings: usize, baseline_heap: usize) -> bool {
        let s_now = self.strings.read().map(|s| s.len()).unwrap_or(0);
        let h_now = self.heap_objects.len();
        s_now.saturating_sub(baseline_strings) > RUNAWAY_STRINGS_GROWTH
            || h_now.saturating_sub(baseline_heap) > RUNAWAY_HEAP_GROWTH
    }

    pub fn call_fn_by_name(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        n_args: usize,
    ) -> Result<(), VMError> {
        // PLAN-702 T-05: legacy entry — busy-wait drive, reserved for NON-UI
        // callers (back_proxy / http_server / run_module_init bootstrap), whose
        // own thread may block for the wait's duration. The UI bridge
        // (vm_bridge) must dispatch via [`Self::call_fn_by_name_segment`]
        // instead: busy-waiting here freezes the iced event loop for the
        // whole wait (PLAN-083 T-01: auto-edit 切换冻结 20.9s 即此).
        match self.dispatch_fn_by_name(task, fn_name, n_args, true, DriveBudget::Legacy) {
            SegmentOutcome::Completed(res) => res,
            SegmentOutcome::Parked { .. } => Err(VMError::RuntimeError(
                "internal: segment driver parked under busy-wait mode (unreachable)".into(),
            )),
            // PLAN-711 T-11: Runnable 仅由 CpuSlice 驱动产生；legacy 同步
            // 调用图到不了这里（防御臂保持 Result 契约形状）。
            SegmentOutcome::Runnable { .. } => Err(VMError::RuntimeError(
                "internal: cpu-slice outcome escaped legacy sync driver (unreachable)".into(),
            )),
        }
    }

    /// PLAN-776 R2-01：HTTP handler dispatch 专用——generator 臂返回凭据对象。
    pub fn call_fn_by_name_http(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        n_args: usize,
    ) -> Result<(), VMError> {
        // PLAN-781 T-15: policy 快照 → LegacyHttp 档（None = Legacy 逐字节
        // 保持；决策报告 D-2 入口矩阵）。
        let budget = match self.http_step_budget {
            Some(cap) => DriveBudget::LegacyHttp(cap),
            None => DriveBudget::Legacy,
        };
        match self.dispatch_fn_by_name_opt(task, fn_name, n_args, true, budget, true) {
            SegmentOutcome::Completed(res) => res,
            SegmentOutcome::Parked { .. } => Err(VMError::RuntimeError(
                "internal: segment driver parked under busy-wait mode (unreachable)".into(),
            )),
            SegmentOutcome::Runnable { .. } => Err(VMError::RuntimeError(
                "internal: cpu-slice outcome escaped legacy sync driver (unreachable)".into(),
            )),
        }
    }

    /// PLAN-776 R2-01：segment 形态的 HTTP dispatch（parked handler）——
    /// generator 臂同样返回凭据对象。
    pub fn call_fn_by_name_segment_http(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        n_args: usize,
    ) -> SegmentOutcome {
        // 与 call_fn_by_name_segment 同参数形态（busy_wait=false），
        // wrap_generators=true（generator 臂返回凭据对象）。PLAN-781 T-15:
        // 预算档 = policy 快照（None = Legacy）。
        // PLAN-781 T-15: policy 快照 → LegacyHttp 档（None = Legacy 逐字节
        // 保持；决策报告 D-2 入口矩阵）。
        let budget = match self.http_step_budget {
            Some(cap) => DriveBudget::LegacyHttp(cap),
            None => DriveBudget::Legacy,
        };
        self.dispatch_fn_by_name_opt(task, fn_name, n_args, false, budget, true)
    }

    /// PLAN-702 T-01: segment-dispatch a named VM function (UI handler drive).
    ///
    /// Same setup contract as [`Self::call_fn_by_name`] (args pre-pushed on
    /// `task.ram` left-to-right, result on stack top on `Completed`), but a
    /// Yield with a pending wait does NOT busy-wait: the call returns
    /// [`SegmentOutcome::Parked`] immediately and the task keeps its
    /// ip/bp/stack/closure state, ready for [`Self::resume_fn_by_name_segment`]
    /// once the wait is satisfied. The iced event loop is free for the whole
    /// wait (E3/E7: 忙等冻结 UI 的事故面).
    ///
    /// Handlers that never yield complete inside this call — the step loop,
    /// budget, runaway guard and try/catch interception are byte-identical
    /// to the sync driver (兼容性论证：迁移爆炸半径 = 今天已坏的会 yield 的
    /// handler，正是缺陷面本身).
    pub fn call_fn_by_name_segment(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        n_args: usize,
    ) -> SegmentOutcome {
        // PLAN-705 T-03: 段模式标志——CALL_SPEC 的 RequestBuilder send 拦截
        // 据此跳过同步 drain（rewind+Yield → park）。Parked 时保持置位
        // （task 静置，无人驱动；resume 入口会重新置位）；Completed 复位。
        task.segment_no_busy_wait = true;
        let outcome = self.dispatch_fn_by_name(task, fn_name, n_args, false, DriveBudget::Legacy);
        if matches!(outcome, SegmentOutcome::Completed(_)) {
            task.segment_no_busy_wait = false;
        }
        outcome
    }

    /// PLAN-711 T-11 (M-02/D-2): UI 专用 CPU slice 首次派发——与
    /// [`Self::call_fn_by_name_segment`] 同一套帧建立/park 语义，但驱动
    /// 预算为 [`DriveBudget::CpuSlice`]：片耗尽返回 [`SegmentOutcome::Runnable`]
    /// （栈完整可续跑），不再有 10M 步同步长占用。首次派发与 resume 共用
    /// 同一预算策略（调用方传同一 [`CpuSliceBudget`]）。
    pub fn call_fn_by_name_cpu_slice(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        n_args: usize,
        budget: CpuSliceBudget,
    ) -> SegmentOutcome {
        task.segment_no_busy_wait = true;
        let outcome =
            self.dispatch_fn_by_name(task, fn_name, n_args, false, DriveBudget::CpuSlice(budget));
        if matches!(outcome, SegmentOutcome::Completed(_)) {
            task.segment_no_busy_wait = false;
        }
        outcome
    }

    /// PLAN-702 T-01/T-03: resume a parked handler segment once its wait is
    /// ready. Returns `Completed` on finish, or `Parked` again (still
    /// waiting / chained await). Readiness probing is the resume pump's job
    /// (vm_bridge scans `ParkedWait`); the external-future wake itself
    /// happens here.
    pub fn resume_fn_by_name_segment(
        &self,
        task: &mut AutoTask,
        seg: &ParkedSegment,
    ) -> SegmentOutcome {
        if let Some(o) = self.resume_parked_wake(task, seg) {
            return o;
        }
        // PLAN-705 T-03: 段标志复位语义同首次入口（Completed 复位，Parked
        // 保持——见 call_fn_by_name_segment 注）。
        task.segment_no_busy_wait = true;
        let outcome = self.drive_handler_segment(
            task,
            &seg.fn_name,
            seg.saved_bp,
            seg.saved_fn_n_args,
            false,
            DriveBudget::Legacy,
        );
        if matches!(outcome, SegmentOutcome::Completed(_)) {
            task.segment_no_busy_wait = false;
        }
        outcome
    }

    /// PLAN-781 T-15: HTTP 请求恢复驱动——与 [`Self::resume_fn_by_name_segment`]
    /// 同唤醒序障，预算档 = policy 快照（`DriveBudget::LegacyHttp`；None =
    /// Legacy 逐字节同）。http_server 三处 step 循环专用；UI 桥沿用既有
    /// cpu_slice / legacy resume 不经此入口。
    pub fn resume_fn_by_name_segment_http(
        &self,
        task: &mut AutoTask,
        seg: &ParkedSegment,
    ) -> SegmentOutcome {
        if let Some(o) = self.resume_parked_wake(task, seg) {
            return o;
        }
        task.segment_no_busy_wait = true;
        let budget = match self.http_step_budget {
            Some(cap) => DriveBudget::LegacyHttp(cap),
            None => DriveBudget::Legacy,
        };
        let outcome = self.drive_handler_segment(
            task,
            &seg.fn_name,
            seg.saved_bp,
            seg.saved_fn_n_args,
            false,
            budget,
        );
        if matches!(outcome, SegmentOutcome::Completed(_)) {
            task.segment_no_busy_wait = false;
        }
        outcome
    }

    /// PLAN-711 T-11 (M-02/D-2): CPU slice 档 resume——唤醒序障与 legacy
    /// resume 相同（外部 future wake / Phase B body 续驱），驱动预算换为
    /// [`DriveBudget::CpuSlice`]。首次派发与 resume 共用预算策略（D-2）。
    pub fn resume_fn_by_name_cpu_slice(
        &self,
        task: &mut AutoTask,
        seg: &ParkedSegment,
        budget: CpuSliceBudget,
    ) -> SegmentOutcome {
        if let Some(o) = self.resume_parked_wake(task, seg) {
            return o;
        }
        task.segment_no_busy_wait = true;
        let outcome = self.drive_handler_segment(
            task,
            &seg.fn_name,
            seg.saved_bp,
            seg.saved_fn_n_args,
            false,
            DriveBudget::CpuSlice(budget),
        );
        if matches!(outcome, SegmentOutcome::Completed(_)) {
            task.segment_no_busy_wait = false;
        }
        outcome
    }

    /// PLAN-711 T-11: resume 前置唤醒序障（外部 future wake + Phase B
    /// suspended body 续驱）。返回 `Some(outcome)` = 已被唤醒路径终结
    /// （仍未就绪再 park / body 出错），调用方直接上浮；`None` = 继续段驱动。
    fn resume_parked_wake(
        &self,
        task: &mut AutoTask,
        seg: &ParkedSegment,
    ) -> Option<SegmentOutcome> {
        // Wake source 6 mirror (Plan 394 Phase A): an external `~{}` future
        // reached Ready/Failed — push the await result and clear the slot so
        // the suspended body resumes at the instruction after the await.
        if let Some(fid) = task.waiting_future_id {
            let ready = self
                .futures
                .get(&fid)
                .map(|f| f.read().unwrap().state != FutureState::Pending)
                .unwrap_or(true); // future gone → wake (nil fallback)
            if !ready {
                return Some(SegmentOutcome::Parked {
                    wait: ParkedWait::Future(fid),
                    seg: seg.clone(),
                });
            }
            if let Some(future_arc) = self.futures.get(&fid) {
                let future = future_arc.read().unwrap();
                match future.state {
                    FutureState::Ready => {
                        if let Some(ref r) = future.result {
                            Self::push_value(task, r, self);
                        } else {
                            task.ram.push_nv(auto_val::encode_null());
                        }
                    }
                    _ => {
                        // Failed / missing result → null (Phase A)
                        task.ram.push_nv(auto_val::encode_null());
                    }
                }
            } else {
                task.ram.push_nv(auto_val::encode_null());
            }
            task.waiting_future_id = None;
            task.status = TaskStatus::Ready;
        }
        // Plan 394 Phase B continuation: the `~{}` body was suspended
        // mid-await (async_frames holds the continuation) — finish the body
        // first (execute_task's resume arm, adapted to the call frame), then
        // keep driving the outer handler frame below.
        if task.waiting_future_id.is_none() {
            if let Some(frame) = task.async_frames.last().copied() {
                match self.resume_suspended_body(task, frame) {
                    Ok(TaskStatus::Waiting(_)) => {
                        return Some(SegmentOutcome::Parked {
                            wait: ParkedWait::Future(task.waiting_future_id.unwrap_or(0)),
                            seg: seg.clone(),
                        });
                    }
                    Ok(_) => {}
                    Err(e) => {
                        task.current_fn_n_args = seg.saved_fn_n_args;
                        return Some(SegmentOutcome::Completed(Err(e)));
                    }
                }
            }
        }
        // Async-HTTP wait: nothing to push here — when the re-driven CALL_NAT
        // re-fires, the shim's re-entry branch consumes ASYNC_RESULTS[req_id]
        // (Plan 349 step 7 protocol; the driver loop below is identical to
        // what the busy-wait arm continued into).
        None
    }

    /// Shared first-entry path of [`Self::call_fn_by_name`],
    /// [`Self::call_fn_by_name_segment`] and [`Self::call_fn_by_name_cpu_slice`]:
    /// fn lookup, Plan 321 generator short-circuit, frame setup, then the
    /// segment drive loop.
    fn dispatch_fn_by_name(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        n_args: usize,
        allow_busy_wait: bool,
        budget: DriveBudget,
    ) -> SegmentOutcome {
        self.dispatch_fn_by_name_opt(task, fn_name, n_args, allow_busy_wait, budget, false)
    }

    /// PLAN-776 R2-01：`wrap_generators=true` 时 generator 臂在返回边界建立
    /// 身份（push Sse 凭据对象而非裸 int id）——仅供 HTTP handler dispatch
    /// 消费（marshal 资源臂按值分派）；for-in 等通用调用面保持 int 语义。
    fn dispatch_fn_by_name_opt(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        n_args: usize,
        allow_busy_wait: bool,
        budget: DriveBudget,
        wrap_generators: bool,
    ) -> SegmentOutcome {
        // 1. Look up function address
        let addr = match self.flash.exports_by_name.get(fn_name) {
            Some(a) => *a,
            None => {
                return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                    "call_fn_by_name: function '{}' not found in exports",
                    fn_name
                ))))
            }
        };

        // Plan 321: Generator detection — if the function body contains
        // YIELD_VAL (0x8D), it's a generator. Don't execute the body;
        // instead, create an Iterator::Generator and push its ID.
        // This allows #[api] handlers returning ~Iter<T>/~Stream<T> to
        // work when called via call_fn_by_name (HTTP handler dispatch).
        let is_generator = self.is_generator_fn(addr as usize);
        if is_generator {
            // Plan 417-D2: mirror the CALL-opcode short-circuit — the caller
            // pre-pushed the n_args args onto task.ram; move them into the
            // generator state so the first next() seeds the generator frame.
            let gen_state = self.new_generator_state(task, addr as u32, n_args as u8);
            let iter_id = self.register_iterator(Iterator::Generator(gen_state));
            // PLAN-776 R2-01：generator 真实 producer 在返回边界建立身份——
            // HTTP dispatch 消费面收 Sse 凭据对象（iterator 身份不靠声明或
            // 表命中猜测）；通用调用面（for-in 等）保持 int 语义。
            if wrap_generators {
                if let Err(e) =
                    crate::vm::ffi::http_return_resource::push_sse_resource(task, self, iter_id)
                {
                    return SegmentOutcome::Completed(Err(e));
                }
            } else {
                task.ram.push_i32(iter_id as i32);
            }
            return SegmentOutcome::Completed(Ok(()));
        }

        // 2. Save current state
        let saved_ip = task.ip;
        let saved_bp = task.bp;
        let saved_fn_n_args = task.current_fn_n_args;

        // 3. Setup stack frame (mirrors CALL opcode: push return addr + old BP)
        task.ram.push_i32(saved_ip as i32); // Return address
        task.ram.push_i32(saved_bp as i32); // Old BP
        task.bp = task.ram.sp - 1;
        // PLAN-667 (F-01): 帧身份登记——帧实例 uid 发号。
        task.push_frame_id(task.bp);
        task.current_fn_n_args = n_args;

        // 4. Jump to function body
        task.ip = addr as usize;

        // 5. Execute until function returns (BP restored to saved_bp)
        self.drive_handler_segment(
            task,
            fn_name,
            saved_bp,
            saved_fn_n_args,
            allow_busy_wait,
            budget,
        )
    }

    /// PLAN-702 T-01: the shared step loop behind `call_fn_by_name`
    /// (`allow_busy_wait = true`, legacy) and `call_fn_by_name_segment`
    /// (`false`, park-on-wait). The frame must already be set up; the
    /// segment completes when RET restores `task.bp` to `saved_bp`.
    fn drive_handler_segment(
        &self,
        task: &mut AutoTask,
        fn_name: &str,
        saved_bp: usize,
        saved_fn_n_args: usize,
        allow_busy_wait: bool,
        budget: DriveBudget,
    ) -> SegmentOutcome {
        let (max_steps, slice, cum_cap) = match budget {
            DriveBudget::Legacy => (10_000_000u64, None, None),
            // PLAN-781 T-15: 段帽 = policy 本身——无 park 的单段 handler（导入
            // 形态）需要单段跑满请求预算；多段由累计帽执法（后段先触累计）。
            // 默认不设 policy = 10M 段帽，逐字节不变。
            DriveBudget::LegacyHttp(cap) => (cap, None, Some(cap)),
            DriveBudget::CpuSlice(b) => (b.max_steps, Some(b), Some(b.cumulative_steps)),
        };
        // PLAN-781 T-15: LegacyHttp 的 runaway 基线跨段延续（与 CpuSlice 同
        // 理由——慢速跨段泄漏不因逐段重置基线而漏检）；纯 Legacy 档保持
        // 段独立基线（既有语义逐字节不动）。
        let baseline_carry = slice.is_some() || cum_cap.is_some();
        let slice_started = std::time::Instant::now();
        let mut steps = 0u64;
        // PLAN-711 AC-13: 段是否真正完成（RET 恢复 saved_bp）——与"预算/片
        // 耗尽"分开记账。旧实现两者共用尾部且都返回 `Completed(Ok(()))`，
        // 预算耗尽 = 静默假成功（task 停在函数中部、无结果值）。
        let mut finished = false;
        // Plan 423 P5 加固:步预算只限时长不限内存 —— 字节码错位后可在垃圾
        // 指令里无界生长字符串池/堆(实机 20G 内存事故)。每 RUNAWAY_CHECK_EVERY
        // 步核对一次增量,超阈即以明确错误中止,把内存炸弹变成可读失败。
        // (段驱动语义:每个段独立基线与预算——非 yield handler 只有首段,
        // 与同步驱动逐字节同;会 park 的 handler 每段重新计,比同步更宽。)
        // PLAN-711 T-11 (D-2): CPU slice 档的 runaway 基线跨片延续——首片
        // 初始化到 task，后续片沿用，慢速跨片泄漏不因逐片重置基线而漏检；
        // legacy 档保持段独立基线（既有语义逐字节不动）。
        let baseline_strings = if baseline_carry {
            match task.cpu_baseline_strings {
                Some(b) => b,
                None => {
                    let b = self.strings.read().map(|s| s.len()).unwrap_or(0);
                    task.cpu_baseline_strings = Some(b);
                    b
                }
            }
        } else {
            self.strings.read().map(|s| s.len()).unwrap_or(0)
        };
        let baseline_heap = if baseline_carry {
            match task.cpu_baseline_heap {
                Some(b) => b,
                None => {
                    let b = self.heap_objects.len();
                    task.cpu_baseline_heap = Some(b);
                    b
                }
            }
        } else {
            self.heap_objects.len()
        };
        while steps < max_steps {
            steps += 1;
            // PLAN-781 T-15: 请求级累计上限（LegacyHttp）——逐指令精确检查：
            // 恰好执行 cap 条指令（第 cap+1 条前拒绝），V-16 的 N-1 耗尽 /
            // N 通过边界可控。I/O park 不计步（等待期不进循环）。
            if let Some(cap) = cum_cap {
                if task.cpu_steps_total.saturating_add(steps) > cap {
                    task.current_fn_n_args = saved_fn_n_args;
                    return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                        "instruction budget exhausted in '{}' at ip=0x{:04x} (per-request cumulative budget {} steps; AUTO_HTTP_STEP_BUDGET)",
                        fn_name, task.ip, cap
                    ))));
                }
            }
            // PLAN-781 T-22（R04 修复轮）：HTTP 请求协作式取消——连续 CPU
            // 段没有 park 边界，deadline/RST/关闭终态后段会继续跑满预算并
            // 执行迟到副作用（复审反例：503 终态后 ~5.9s 仍写出 late 文
            // 件）。每 HTTP_CANCEL_PROBE_EVERY 步在安全指令边界探测当前请
            // 求 scope；已终结即中止段——与预算耗尽同 Err 形态，由 HTTP
            // 入口统一收口（迟到 500 无消费者=静默失败，不产生第二个终
            // 态）。无 scope 绑定的 CLI/UI/legacy 程序段恒 false，语义不变。
            if steps % HTTP_CANCEL_PROBE_EVERY == 0
                && crate::vm::ffi::http_server::current_request_cancelled()
            {
                task.current_fn_n_args = saved_fn_n_args;
                return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                    "request cancelled in '{}' at ip=0x{:04x} (scope terminal/deadline reached; cooperative cancel at safe instruction boundary)",
                    fn_name, task.ip
                ))));
            }
            // PLAN-711 T-11 (D-2): slice 档至多每 clock_check_every 步查一次
            // 墙钟，超 4ms 即到片末——在安全指令边界（step 之间）让出。
            if let Some(b) = slice {
                if steps % b.clock_check_every == 0 && slice_started.elapsed() >= b.max_duration {
                    steps -= 1; // 本步未执行，不计入片消耗
                    break;
                }
            }
            // PLAN-711 T-11: runaway 检查节拍跨片累计——slice 档内 steps 每片
            // 归零，若只看片内步数则 50k 节拍在 4096 步片内永不命中，堆增长
            // 护栏形同虚设；legacy 档 cpu_steps_total 恒 0，语义逐字节不变。
            let runaway_step_clock = task.cpu_steps_total + steps;
            if runaway_step_clock % RUNAWAY_CHECK_EVERY == 0
                && self.runaway_growth_exceeded(baseline_strings, baseline_heap)
            {
                let s_growth = self
                    .strings
                    .read()
                    .map(|s| s.len().saturating_sub(baseline_strings))
                    .unwrap_or(0);
                let h_growth = self.heap_objects.len().saturating_sub(baseline_heap);
                task.current_fn_n_args = saved_fn_n_args;
                return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                    "runaway execution halted in '{}' at ip=0x{:04x}: +{} strings, +{} heap objects within one call (bytecode desync or unbounded loop?)",
                    fn_name, task.ip, s_growth, h_growth
                ))));
            }
            let step = match self.run_one_instruction(task) {
                Ok(s) => s,
                Err(e) => {
                    // Plan 010 (MS3-A): try/catch interception.
                    if self.intercept_error(task, &e) {
                        continue;
                    }
                    task.current_fn_n_args = saved_fn_n_args;
                    return SegmentOutcome::Completed(Err(e));
                }
            };
            match step {
                StepResult::Continue => {
                    if task.bp == saved_bp {
                        finished = true;
                        break;
                    }
                    continue;
                }
                StepResult::Terminated => {
                    task.current_fn_n_args = saved_fn_n_args;
                    return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                        "Function '{}' execution terminated unexpectedly",
                        fn_name
                    ))));
                }
                StepResult::Yield => {
                    if !allow_busy_wait {
                        // PLAN-702 T-01: park instead of busy-waiting — the
                        // iced update returns and the resume pump re-drives
                        // the segment once the wait is satisfied.
                        if let Some(fid) = task.waiting_future_id {
                            // PLAN-781 T-15: 段终止但请求未终——本段已执行步数计入任务累计
                            //（resume 在新段继续累加，不重置）。
                            task.cpu_steps_total = task.cpu_steps_total.saturating_add(steps);
                            return SegmentOutcome::Parked {
                                wait: ParkedWait::Future(fid),
                                seg: ParkedSegment {
                                    fn_name: fn_name.to_string(),
                                    saved_bp,
                                    saved_fn_n_args,
                                },
                            };
                        }
                        if let Some(req_id) = task.waiting_http_request_id {
                            if std::env::var_os("AUTO_DEBUG_G9").is_some() {
                                eprintln!(
                                    "[G9] PARK: fn={} req={} ip={:#x} sp={} bp={} saved_bp={}",
                                    fn_name, req_id, task.ip, task.ram.sp, task.bp, saved_bp
                                );
                            }
                            // PLAN-781 T-15: 段终止但请求未终——本段已执行步数计入任务累计
                            //（resume 在新段继续累加，不重置）。
                            task.cpu_steps_total = task.cpu_steps_total.saturating_add(steps);
                            return SegmentOutcome::Parked {
                                wait: ParkedWait::HttpRequest(req_id),
                                seg: ParkedSegment {
                                    fn_name: fn_name.to_string(),
                                    saved_bp,
                                    saved_fn_n_args,
                                },
                            };
                        }
                        // PLAN-707 T-05: 外部流无数据 → park（就绪/终结后由
                        // 完成通知唤醒的泵恢复；凭证供 parked_is_ready 判定）。
                        if let Some(stream_id) = task.waiting_http_stream_id {
                            // PLAN-781 T-15: 段终止但请求未终——本段已执行步数计入任务累计
                            //（resume 在新段继续累加，不重置）。
                            task.cpu_steps_total = task.cpu_steps_total.saturating_add(steps);
                            return SegmentOutcome::Parked {
                                wait: ParkedWait::HttpStream(stream_id),
                                seg: ParkedSegment {
                                    fn_name: fn_name.to_string(),
                                    saved_bp,
                                    saved_fn_n_args,
                                },
                            };
                        }
                        // Non-waiting yield (SLEEP / SSE generator retry):
                        // sync-driver parity — keep stepping.
                        continue;
                    }
                    if let Some(req_id) = task.waiting_http_request_id {
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_secs(30);
                        // PLAN-026 T-04: 忙等段预算计(轮询次数 + 忙等时长)。
                        let mut budget_waits: u64 = 0;
                        let budget_start = std::time::Instant::now();
                        while !crate::vm::ffi::stdlib::async_http_result_ready(req_id) {
                            if std::time::Instant::now() > deadline {
                                task.waiting_http_request_id = None;
                                // PLAN-027 缺陷 A:超时放弃必须同步回收
                                // ASYNC_RESULTS 条目——worker 迟到完成的
                                // 完整响应体会永驻(每超时泄漏一个 body)。
                                crate::vm::ffi::stdlib::drop_async_result(req_id);
                                return SegmentOutcome::Completed(Err(VMError::RuntimeError(
                                    "async http request timed out in call_fn_by_name".into(),
                                )));
                            }
                            std::thread::sleep(std::time::Duration::from_millis(5));
                            budget_waits += 1;
                        }
                        Self::api_budget_accumulate(budget_waits, budget_start.elapsed());
                        steps = steps.saturating_sub(1);
                    }
                    // PLAN-707 T-05: legacy 同步 busy-wait 段的流等待臂——
                    // 既有形态比照（5ms 轮询 + 30s 超时；同步驱动的既有约束，
                    // 事件驱动路径见 segment 泵与 D-6）。
                    if let Some(stream_id) = task.waiting_http_stream_id {
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_secs(30);
                        while !crate::vm::ffi::http_stream::stream_ready(stream_id) {
                            if std::time::Instant::now() > deadline {
                                task.waiting_http_stream_id = None;
                                crate::vm::ffi::http_stream::stream_cancel(stream_id);
                                return SegmentOutcome::Completed(Err(VMError::RuntimeError(
                                    "http stream wait timed out in call_fn_by_name".into(),
                                )));
                            }
                            std::thread::sleep(std::time::Duration::from_millis(5));
                        }
                        steps = steps.saturating_sub(1);
                    }
                    continue;
                }
                StepResult::GeneratorYield => {
                    // Plan 321: YIELD_VAL in a regular call_fn_by_name context
                    // means the function is a generator. In non-generator mode
                    // (called via call_fn_by_name), we can't suspend — just
                    // continue past the yield (the yielded value is on stack,
                    // we leave it there as a side effect).
                    // NOTE: Proper generator suspension only happens via the
                    // Iterator::Generator driver (Phase 2), not call_fn_by_name.
                    continue;
                }
                StepResult::AwaitFuture {
                    future_id,
                    body_offset,
                } => {
                    if let Err(e) = self.handle_await_future(task, future_id, body_offset) {
                        task.current_fn_n_args = saved_fn_n_args;
                        return SegmentOutcome::Completed(Err(e));
                    }
                    // PLAN-702 T-01: Phase B suspension (body parked mid-await
                    // on an external future) — park instead of the sync
                    // driver's fall-through hot-continue over a mid-body ip.
                    if !allow_busy_wait && task.waiting_future_id.is_some() {
                        // PLAN-781 T-15: 段终止但请求未终——本段已执行步数计入任务累计
                        //（resume 在新段继续累加，不重置）。
                        task.cpu_steps_total = task.cpu_steps_total.saturating_add(steps);
                        return SegmentOutcome::Parked {
                            wait: ParkedWait::Future(task.waiting_future_id.unwrap_or(0)),
                            seg: ParkedSegment {
                                fn_name: fn_name.to_string(),
                                saved_bp,
                                saved_fn_n_args,
                            },
                        };
                    }
                }
            }
        }

        // PLAN-711 T-11 (AC-13): 预算/片耗尽分支——task 状态完整（ip 指向
        // 函数内、bp≠saved_bp、闭包/异常帧/返回槽原样）。
        if !finished {
            match budget {
                DriveBudget::CpuSlice(b) => {
                    // 跨片累计护栏：只累计实际执行的指令（I/O 等待不计忙时）；
                    // 达累计上限即真错误，逐片让出不得重置护栏绕过 runaway 保护。
                    task.cpu_steps_total = task.cpu_steps_total.saturating_add(steps);
                    if task.cpu_steps_total >= b.cumulative_steps {
                        task.current_fn_n_args = saved_fn_n_args;
                        return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                            "cumulative CPU budget exhausted in '{}' ({} steps across slices) — runaway handler?",
                            fn_name, task.cpu_steps_total
                        ))));
                    }
                    // 栈完整可续跑：返回 Runnable，栈/帧/闭包全部原样留给
                    // CPU 泵下一轮在预算内继续（不清栈、不重进 prologue）。
                    return SegmentOutcome::Runnable {
                        seg: ParkedSegment {
                            fn_name: fn_name.to_string(),
                            saved_bp,
                            saved_fn_n_args,
                        },
                    };
                }
                DriveBudget::LegacyHttp(cap) => {
                    // PLAN-781 T-15: 请求预算耗尽（单段形态 = 段帽即请求帽；
                    // 多段形态由循环内累计检查先触）。指名 policy 的资源拒绝
                    // （不 Runnable：HTTP 无 CPU 泵；不假成功；不落 legacy
                    // 文案——观测面可区分配置档与默认档）。累计记账在函数尾。
                    let ip = task.ip;
                    eprintln!(
                        "WARN[budget] fn='{}' ip={} per-request cap {} reached at segment end",
                        fn_name, ip, cap
                    );
                    task.current_fn_n_args = saved_fn_n_args;
                    return SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                        "instruction budget exhausted in '{}' at ip=0x{:04x} (per-request cumulative budget {} steps; AUTO_HTTP_STEP_BUDGET)",
                        fn_name, ip, cap
                    ))));
                }
                DriveBudget::Legacy => {
                    // legacy 档耗尽：保留既有 WARN 诊断（近旁 FN_PROLOGUE 扫描），
                    // 但**不再落入 `Completed(Ok(()))` 静默假成功**——真实错误
                    // 上浮（`Result` 契约形状不变；旧假成功即 AC-13 缺陷本身）。
                    let ip = task.ip;
                    let mut trace = String::new();
                    let start = ip.saturating_sub(20);
                    for i in start..(start + 30).min(self.flash.memory.len()) {
                        let b = self.flash.memory[i];
                        if b == 0x06 {
                            // RESERVE_STACK = fn prologue marker
                            trace.push_str(&format!("[FN_PROLOGUE@{}] ", i));
                        }
                    }
                    let cur_fn = task
                        .call_stack
                        .last()
                        .and_then(|f| f.fn_name.clone())
                        .unwrap_or_default();
                    let call_depth = task.call_stack.len();
                    eprintln!(
                        "WARN[budget] fn='{}' ip={} call_depth={} trace={}",
                        cur_fn, ip, call_depth, trace
                    );
                }
            }
        }

        // 6. Restore non-stack state (return value already on stack top)
        // PLAN-781 T-15: LegacyHttp 的段末统一累计（成功/段帽耗尽两路；
        // Parked 各点已内联累计；CpuSlice 臂自带累计并提前 return）。
        if cum_cap.is_some() {
            task.cpu_steps_total = task.cpu_steps_total.saturating_add(steps);
        }
        task.current_fn_n_args = saved_fn_n_args;
        // PLAN-026 T-04: handler 粒度预算报告(api.* 忙等观测护栏)。
        Self::api_budget_report(fn_name);
        // PLAN-026 needs_fix3(内存爬升分流):AUTO_VM_MEM=1 时每 500 次
        // 调用打印 VM 侧池/堆规模(strings 池字节数近似 = Σ len)——
        // 区分"VM 层泄漏(池/堆线性涨)"与"rust 层(HTTP/渲染)"。
        if std::env::var("AUTO_VM_MEM").as_deref() == Ok("1") {
            static MEM_N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let n = MEM_N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if n % 500 == 0 {
                let pool_len = self.strings.read().map(|p| p.len()).unwrap_or(0);
                let pool_bytes: usize = self
                    .strings
                    .read()
                    .map(|p| p.iter().map(|v| v.len()).sum())
                    .unwrap_or(0);
                eprintln!(
                    "[VM-MEM] calls={} strings_pool={} pool_bytes={} heap_objs={}",
                    n,
                    pool_len,
                    pool_bytes,
                    self.heap_objects.len()
                );
            }
        }
        if finished {
            SegmentOutcome::Completed(Ok(()))
        } else {
            SegmentOutcome::Completed(Err(VMError::RuntimeError(format!(
                "instruction budget exhausted in '{}' at ip=0x{:04x} (infinite loop or runaway handler; legacy budget 10M steps)",
                fn_name, task.ip
            ))))
        }
    }

    /// Plan 321: Check if a function body contains YIELD_VAL (0x8D) opcode.
    pub(super) fn is_generator_fn(&self, addr: usize) -> bool {
        // Plan 348 Task 21: Walk the bytecode instruction-by-instruction,
        // decoding each opcode's operand stream via `OpCode::operand_size` so
        // that operand bytes are never mistaken for opcodes.
        //
        // The previous implementation (Plan 347) bounded a raw byte scan at the
        // first 0x71 (RET) byte. This stopped cross-function false positives
        // but was still vulnerable to *intra-function* false positives:
        // whenever a function's operand stream (e.g. a CONST_I32 immediate, a
        // CALL target, or a SOURCE_LINE line number) happened to contain 0x8D
        // before the real RET opcode, the function was misclassified as a
        // generator. The CALL handler then short-circuited the body and
        // returned an iterator id instead of executing the function —
        // producing symptoms like "sha256 miscomputes" that were hard to
        // connect back to this scan. The fix is to never look at operand
        // bytes at all.
        //
        // A generator always yields before returning, so a real YIELD_VAL
        // cannot appear after the first genuine RET opcode. We stop at the
        // first RET found at an opcode boundary.
        //
        // For variable-length instructions (BUILD_FSTR, NEW_INSTANCE,
        // IS_VARIANT) `operand_size` returns None. NEW_INSTANCE in particular
        // reads its length from the stack rather than the bytecode, so we
        // cannot statically step past it; the safe choice is to stop scanning
        // (return false). Generators are extremely unlikely to contain these
        // opcodes before their first yield, and the previous behavior on
        // encountering them was undefined anyway.
        const YIELD_VAL: u8 = OpCode::YIELD_VAL as u8;
        const RET: u8 = OpCode::RET as u8;
        let max_scan = std::cmp::min(addr + 4096, self.flash.memory.len());
        let mut ip = addr;
        while ip < max_scan {
            let op_byte = self.flash.memory[ip];
            if op_byte == YIELD_VAL {
                return true;
            }
            if op_byte == RET {
                return false;
            }
            if !OpCode::is_valid(op_byte) {
                // Desynchronized — bail out safely.
                return false;
            }
            let op: OpCode = op_byte.into();
            let step = match op.operand_size() {
                Some(n) => n,
                None => return false, // variable-length; cannot safely step
            };
            ip = ip.saturating_add(1 + step);
        }
        false
    }

    /// Plan 317 §11 Phase 11 (Bug fix): Read a function's declared parameter
    /// count (the `n_args` byte stored right after its FN_PROLOG opcode at the
    /// function's entry address). Returns `None` if the function isn't found or
    /// the bytecodes don't start with FN_PROLOG (defensive).
    ///
    /// Used by HTTP handler dispatch (`build_handler_args`) to decide whether
    /// the handler declared an extra parameter to receive the cookies/auth
    /// metadata. Without this, the runtime can't tell a 1-param handler
    /// (`fn echo_id(id int)`) from a 2-param one that wants the metadata, and it
    /// unconditionally pushes the metadata — corrupting parameter binding (the
    /// handler's first param ends up bound to the metadata JSON instead of the
    /// path param).
    pub fn get_fn_n_args(&self, fn_name: &str) -> Option<usize> {
        let addr = *self.flash.exports_by_name.get(fn_name)? as usize;
        // The first byte at the entry point is FN_PROLOG (0xFN), followed by
        // n_args (u8), then n_locals (u8). See codegen.rs Plan 088 Phase 4:
        // "Always emit FN_PROLOG at function entry".
        let prolog_byte = self.flash.memory.get(addr).copied()?;
        if prolog_byte != OpCode::FN_PROLOG as u8 {
            return None;
        }
        self.flash.memory.get(addr + 1).copied().map(|n| n as usize)
    }

    /// Returns true if the message matches the pattern
    fn match_message_pattern_vm(
        &self,
        msg: &auto_val::Value,
        pattern: &crate::vm::task_handler::SerializedPattern,
        string_pool: &[String],
    ) -> bool {
        use crate::vm::task_handler::PatternType;
        use auto_val::Value;

        match pattern.pattern_type {
            PatternType::Literal => {
                if pattern.data.is_empty() {
                    return false;
                }
                let lit_type = pattern.data[0];
                match lit_type {
                    0x01 => {
                        // String literal
                        if pattern.data.len() < 5 {
                            return false;
                        }
                        let idx = u32::from_le_bytes([
                            pattern.data[1],
                            pattern.data[2],
                            pattern.data[3],
                            pattern.data[4],
                        ]) as usize;
                        if let Some(s) = string_pool.get(idx) {
                            matches!(msg, Value::Str(s2) if s2.as_str() == s.as_str())
                        } else {
                            false
                        }
                    }
                    0x02 => {
                        // Int literal
                        if pattern.data.len() < 9 {
                            return false;
                        }
                        let n = i64::from_le_bytes([
                            pattern.data[1],
                            pattern.data[2],
                            pattern.data[3],
                            pattern.data[4],
                            pattern.data[5],
                            pattern.data[6],
                            pattern.data[7],
                            pattern.data[8],
                        ]);
                        matches!(msg, Value::Int(i) if *i as i64 == n)
                    }
                    0x03 => {
                        // Uint literal
                        if pattern.data.len() < 9 {
                            return false;
                        }
                        let n = u64::from_le_bytes([
                            pattern.data[1],
                            pattern.data[2],
                            pattern.data[3],
                            pattern.data[4],
                            pattern.data[5],
                            pattern.data[6],
                            pattern.data[7],
                            pattern.data[8],
                        ]);
                        matches!(msg, Value::Uint(u) if *u as u64 == n)
                    }
                    0x04 => {
                        // Bool literal
                        if pattern.data.len() < 2 {
                            return false;
                        }
                        let b = pattern.data[1] != 0;
                        matches!(msg, Value::Bool(b2) if *b2 == b)
                    }
                    0x05 => {
                        // Char literal
                        if pattern.data.len() < 5 {
                            return false;
                        }
                        let c = u32::from_le_bytes([
                            pattern.data[1],
                            pattern.data[2],
                            pattern.data[3],
                            pattern.data[4],
                        ]);
                        matches!(msg, Value::Char(c2) if (*c2 as u32) == c)
                    }
                    0x06 => {
                        // Float literal (two i64 parts)
                        if pattern.data.len() < 17 {
                            return false;
                        }
                        let _integral = i64::from_le_bytes([
                            pattern.data[1],
                            pattern.data[2],
                            pattern.data[3],
                            pattern.data[4],
                            pattern.data[5],
                            pattern.data[6],
                            pattern.data[7],
                            pattern.data[8],
                        ]);
                        let _fractional = i64::from_le_bytes([
                            pattern.data[9],
                            pattern.data[10],
                            pattern.data[11],
                            pattern.data[12],
                            pattern.data[13],
                            pattern.data[14],
                            pattern.data[15],
                            pattern.data[16],
                        ]);
                        // For now, just check if it's a float type
                        matches!(msg, Value::Float(_) | Value::Double(_))
                    }
                    _ => false,
                }
            }
            PatternType::Simple => {
                // Simple variant pattern - check if message is an object with __variant field
                if pattern.data.len() < 4 {
                    return false;
                }
                let idx = u32::from_le_bytes([
                    pattern.data[0],
                    pattern.data[1],
                    pattern.data[2],
                    pattern.data[3],
                ]) as usize;
                if let Some(variant_name) = string_pool.get(idx) {
                    match msg {
                        Value::Obj(obj) => {
                            if let Some(Value::Str(v)) =
                                obj.get(auto_val::AutoStr::from("__variant"))
                            {
                                v.as_str() == variant_name.as_str()
                            } else {
                                false
                            }
                        }
                        // Plan 390 §15 H2: structured messages arrive at the
                        // matcher as Value::VmRef (send shim stores a ref, not
                        // an inlined Obj). Rehydrate just the __variant field
                        // from the objects registry to match the variant name.
                        Value::VmRef(r) => self
                            .vmref_variant_name(r.id)
                            .map_or(false, |v| v.as_str() == variant_name.as_str()),
                        Value::Str(s) => s.as_str() == variant_name.as_str(),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            PatternType::TypeBinding => {
                // Type binding pattern - check if message matches the expected type
                if pattern.data.len() < 5 {
                    return false;
                }
                let _name_idx = u32::from_le_bytes([
                    pattern.data[0],
                    pattern.data[1],
                    pattern.data[2],
                    pattern.data[3],
                ]) as usize;
                let type_tag = pattern.data[4];

                // Match based on type tag
                match type_tag {
                    0x01 => matches!(msg, Value::Int(_)),    // Int
                    0x02 => matches!(msg, Value::I64(_)),    // I64
                    0x03 => matches!(msg, Value::Uint(_)),   // Uint
                    0x04 => matches!(msg, Value::I64(_)),    // U64 -> I64 (Value doesn't have U64)
                    0x05 => matches!(msg, Value::Float(_)),  // Float
                    0x06 => matches!(msg, Value::Double(_)), // Double
                    0x07 => matches!(msg, Value::Bool(_)),   // Bool
                    0x08 => matches!(msg, Value::Char(_)),   // Char
                    0x09 => matches!(msg, Value::Str(_)),    // Str
                    0xFF => true,                            // Unknown - match anything
                    _ => false,
                }
            }
            PatternType::WithBindings => {
                // Variant with bindings pattern
                if pattern.data.len() < 5 {
                    return false;
                }
                let variant_idx = u32::from_le_bytes([
                    pattern.data[0],
                    pattern.data[1],
                    pattern.data[2],
                    pattern.data[3],
                ]) as usize;

                if let Some(variant_name) = string_pool.get(variant_idx) {
                    match msg {
                        Value::Obj(obj) => {
                            if let Some(Value::Str(v)) =
                                obj.get(auto_val::AutoStr::from("__variant"))
                            {
                                v.as_str() == variant_name.as_str()
                            } else {
                                false
                            }
                        }
                        // Plan 390 §15 H2: rehydrate __variant from the
                        // objects registry (see PatternType::Simple).
                        Value::VmRef(r) => self
                            .vmref_variant_name(r.id)
                            .map_or(false, |v| v.as_str() == variant_name.as_str()),
                        Value::Str(s) => s.as_str() == variant_name.as_str(),
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }

    /// The main async loop that schedules and runs tasks.
    pub async fn run_task_loop(&self) {
        loop {
            let mut active_count = 0;
            let mut alive_count = 0;

            // Collect tasks to iterate
            // We use a Vec of Arcs to avoid holding the map lock during execution
            let tasks: Vec<(TaskId, Arc<Mutex<AutoTask>>)> = self
                .tasks
                .iter()
                .map(|r| (*r.key(), r.value().clone()))
                .collect();

            if tasks.is_empty() {
                break; // No tasks left, exit VM
            }

            // PLAN-746 (PG-MEM-1): cooperative execution deadline. Once
            // passed, terminate every remaining task with a synthetic
            // timeout error so callers see a clean failure instead of a
            // runaway execution that never returns (and keeps allocating).
            if let Some(dl) = self.deadline {
                if std::time::Instant::now() >= dl {
                    for (_id, task_mutex) in &tasks {
                        let mut t = task_mutex.lock().await;
                        if t.status != TaskStatus::Terminated {
                            t.last_error =
                                Some("ExecutionTimeout: execution deadline exceeded".to_string());
                            t.status = TaskStatus::Terminated;
                        }
                    }
                    break;
                }
            }

            for (_id, task_mutex) in tasks {
                let mut task = task_mutex.lock().await;

                if task.status == TaskStatus::Terminated {
                    continue;
                }

                // Check if sleeping task should wake up
                if let Some(wake_time) = task.wake_time {
                    if Instant::now() >= wake_time {
                        task.wake_time = None;
                        task.status = TaskStatus::Ready;
                    } else {
                        alive_count += 1;
                        continue; // Still sleeping
                    }
                }

                // Plan 317 Phase 1: wake a message-loop task that has pending
                // messages. TASK_LOOP left it Waiting; TaskHandle.send pushed
                // to its mailbox in vm.task_mailboxes. Drain one message, find
                // its handler, set ip to the handler body_offset, and mark Ready
                // so execute_task runs it. The handler RETs; the next iteration
                // re-parks it if more messages wait, else it stays Waiting.
                if task.in_message_loop {
                    // Check the mailbox (DashMap, std Mutex — safe to lock here
                    // since we're in async run_task_loop, not a sync native).
                    let drained = if let Some(mb) = self.task_mailboxes.get(&task.id) {
                        if let Ok(mut q) = mb.lock() {
                            if !q.is_empty() {
                                Some(q.remove(0))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if crate::is_vm_debug() {
                        eprintln!(
                            "[run_task_loop] task {} in_message_loop, drained={:?}",
                            task.id,
                            drained.as_ref().map(|_| ())
                        );
                    }
                    if let Some(msg) = drained {
                        let task_type = task.task_type_name.clone().unwrap_or_default();
                        if let Some((body_offset, has_context)) =
                            self.find_handler_offset(&task_type, &msg)
                        {
                            // Plan 390 §15 G2-refactor (方案 B): 给 handler 建立真正的
                            // bp 栈帧（CALL 风格），根治 G2 方案 A（H2 的 HANDLER_LOCALS_BAND
                            // 预留固定 16 槽）的 handler locals 与 message/表达式临时值重叠
                            // 问题。深嵌套表达式或多字段 WithBindings 会穿透固定预留带。
                            //
                            // 帧布局（低 → 高）:
                            //   [ret_ip, old_bp, <locals 区 16 槽>, message, <表达式临时值>]
                            //    bp 指向 old_bp；handler locals 在 bp+1..bp+16；message 与
                            //    临时值在 locals 之上，永不重叠。
                            //
                            // ret_ip = actor park 时的 ip（#start 里 TASK_LOOP 之后的 RET）。
                            // handler RET 把 ip 恢复成 ret_ip → 执行那条 RET → bp 已恢复为
                            // old_bp(=0) → engine.rs RET 的 bp==0 短路 Terminated → 下方
                            // in_message_loop catch 把 Terminate 转回 Waiting（重 park）。
                            // 无需特殊 ret_ip 哨兵，复用 #start 尾部 RET。
                            const HANDLER_LOCALS_SLOTS: usize = 16;
                            let saved_ip = task.ip; // park 时 ip（TASK_LOOP 后的 RET）
                            let saved_bp = task.bp; // 0（parked actor）
                                                    // 1. CALL 风格建帧：push ret_ip, push old_bp, bp = sp - 1
                            task.ram.push_i32(saved_ip as i32);
                            task.ram.push_i32(saved_bp as i32);
                            task.bp = task.ram.sp - 1;
                            // PLAN-667 (F-01): 帧身份登记——帧实例 uid 发号。
                            let handler_bp = task.bp;
                            task.push_frame_id(handler_bp);
                            // 2. 预留 locals 区（bp+1 .. bp+HANDLER_LOCALS_SLOTS）
                            for _ in 0..HANDLER_LOCALS_SLOTS {
                                task.ram.push_i32(0);
                            }
                            // 3. push message（在 locals 区之上）
                            // Plan 390 §14 L3: structured messages (Value::VmRef
                            // from a WithBindings variant constructor) are pushed
                            // as object refs so the handler can GET_FIELD each
                            // binding. Scalar messages stay as i32 for backward
                            // compat with Literal/TypeBinding/Simple patterns.
                            match &msg {
                                auto_val::Value::VmRef(vmref) => {
                                    // Plan 419: 消息引用入 handler 帧 +1;邮箱
                                    // 排水侧的 stake(send 时的 retain)随之
                                    // 死亡 —— 净效果:消息 stake 转入 handler 帧。
                                    self.rc_push(
                                        &mut task,
                                        auto_val::encode_object(vmref.id as u32),
                                    );
                                    self.rc_release_id(vmref.id as u64);
                                }
                                auto_val::Value::Int(i) => {
                                    task.ram.push_i32(*i);
                                }
                                auto_val::Value::Uint(u) => {
                                    task.ram.push_i32(*u as i32);
                                }
                                auto_val::Value::Bool(b) => {
                                    task.ram.push_i32(if *b { 1 } else { 0 });
                                }
                                _ => {
                                    task.ram.push_i32(0);
                                }
                            }
                            task.ip = body_offset as usize;
                            task.current_handler_has_context = has_context;
                            task.status = TaskStatus::Ready;
                        } else if crate::is_vm_debug() {
                            eprintln!(
                                "[run_task_loop] No handler for message {:?} in task {}",
                                msg, task_type
                            );
                        }
                    }
                }

                // Plan 317 Phase 1: a Waiting message-loop task with an empty
                // mailbox is idle — it won't self-wake. Don't count it as alive,
                // so run_task_loop can exit when only such idle actors remain
                // (otherwise it busy-loops on sleep forever). If a message
                // arrives later (TaskHandle.send), the task isn't idle anymore.
                let is_idle_actor = task.in_message_loop
                    && matches!(task.status, TaskStatus::Waiting(_))
                    && !self
                        .task_mailboxes
                        .get(&task.id)
                        .map(|m| m.lock().map(|q| !q.is_empty()).unwrap_or(false))
                        .unwrap_or(false);
                if is_idle_actor {
                    continue;
                }

                // Plan 317 §11 Phase 7 (P3): a generator task suspended at a
                // YIELD_VAL (Waiting("generator_suspended")) is driven externally
                // — it only resumes when a consumer calls iter.next() (which
                // try_locks the task and runs instructions, not via run_task_loop).
                // Such a task never self-wakes, so if the consumer has broken out
                // of the for-loop and abandoned it, the task stays Waiting forever.
                // Without this exclusion, run_task_loop sees alive_count > 0 and
                // busy-loops on sleep forever — the classic "infinite generator
                // hangs the VM" symptom. Treat it like an idle actor: don't count
                // it as alive, so the VM can exit when only suspended generators
                // (or idle actors) remain.
                if matches!(task.status, TaskStatus::Waiting(ref reason) if reason == "generator_suspended")
                {
                    continue;
                }

                // Plan 348/353/349: Wake source 4 — SSE/IO stream data available.
                // Task yielded with Waiting("sse") because the async channel
                // had no data. Check if data has arrived since then. HTTP and IO
                // streams now share the unified ASYNC_STREAMS table (Plan 349
                // table consolidation); stream_id is globally unique.
                if let Some(stream_id) = task.waiting_sse_stream_id {
                    let has_data = crate::vm::ffi::stdlib::ASYNC_STREAMS
                        .lock()
                        .ok()
                        .and_then(|m| {
                            m.get(&stream_id).map(|handle| {
                                let done = handle.done.load(std::sync::atomic::Ordering::SeqCst);
                                let has_recv =
                                    handle.rx.lock().map(|rx| !rx.is_empty()).unwrap_or(false);
                                has_recv || done
                            })
                        })
                        .unwrap_or(true); // Stream gone → wake up (push -1 sentinel).
                    if has_data {
                        task.waiting_sse_stream_id = None;
                        task.status = TaskStatus::Ready;
                    } else {
                        alive_count += 1;
                        continue; // Still waiting for SSE data
                    }
                }

                // PLAN-707 T-05: Wake source 4b — HTTPStream 句柄惰性消费的
                // 等待唤醒（数据入队或终态；stream_ready 含终结与条目消失）。
                if let Some(stream_id) = task.waiting_http_stream_id {
                    if crate::vm::ffi::http_stream::stream_ready(stream_id) {
                        task.waiting_http_stream_id = None;
                        task.status = TaskStatus::Ready;
                    } else {
                        alive_count += 1;
                        continue; // Still waiting for stream data
                    }
                }

                // Plan 394 Phase A: Wake source 6 — external future ready.
                // On Ready/Failed, push the await result and clear the slot so
                // the task resumes at the instruction after AWAIT_FUTURE.
                if let Some(fid) = task.waiting_future_id {
                    let ready = self
                        .futures
                        .get(&fid)
                        .map(|f| {
                            let fv = f.read().unwrap();
                            fv.state != FutureState::Pending
                        })
                        .unwrap_or(true); // future gone → wake (nil fallback)
                    if ready {
                        if let Some(future_arc) = self.futures.get(&fid) {
                            let future = future_arc.read().unwrap();
                            match future.state {
                                FutureState::Ready => {
                                    if let Some(ref r) = future.result {
                                        Self::push_value(&mut task, r, self);
                                    } else {
                                        task.ram.push_nv(auto_val::encode_null());
                                    }
                                }
                                _ => {
                                    // Failed / missing result → null (Phase A)
                                    task.ram.push_nv(auto_val::encode_null());
                                }
                            }
                        } else {
                            task.ram.push_nv(auto_val::encode_null());
                        }
                        task.waiting_future_id = None;
                        task.status = TaskStatus::Ready;
                    } else {
                        alive_count += 1;
                        continue; // Still waiting on external future
                    }
                }

                // Plan 349 step 7 / Plan 353: Wake source 5 — async HTTP/IO completed.
                // PLAN-705 T-02: 统一 live-op 表（async_http::LIVE_OPS）——
                // 极性与旧表逐字节同（缺席/已终结 → 唤醒落错误 fallback）。
                if let Some(req_id) = task.waiting_http_request_id {
                    let ready = crate::vm::ffi::async_http::live_op_ready_or_gone(req_id);
                    if ready {
                        // NOTE: do NOT clear waiting_http_request_id here. The
                        // shim's re-entry branch needs it to look up the result
                        // in the live-op table. The shim itself clears the
                        // field once it has consumed the result. Clearing it
                        // prematurely makes the re-entry take the first-call
                        // path and pop args that are no longer on the stack
                        // (Stack Underflow). Plan 349 步骤 7.
                        task.status = TaskStatus::Ready;
                    } else {
                        alive_count += 1;
                        continue; // Still waiting for HTTP response
                    }
                }

                alive_count += 1;

                // Check if task is runnable
                if task.status != TaskStatus::Running && task.status != TaskStatus::Ready {
                    continue;
                }

                active_count += 1;
                task.status = TaskStatus::Running;

                // Run a chunk of instructions
                match self.execute_task(&mut task) {
                    Ok(new_status) => {
                        // Plan 317 Phase 1: a message-loop task that hits RET
                        // (handler finished, or start hook finished) must NOT
                        // terminate — it parks back in Waiting to receive the
                        // next message. Without this, the actor dies after its
                        // first handler returns.
                        if new_status == TaskStatus::Terminated && task.in_message_loop {
                            // Plan 390 §15 G2-refactor (方案 B): handler RET 已通过真 bp 帧
                            // 的标准 unwind 恢复 sp/bp（old_bp=0）。但 RET 恢复的 ip 指向
                            // #start 尾部 RET，那条 RET 执行时会读 n_args 字节推进 ip，
                            // 污染下次 wake 的 saved_ip。故这里把 ip 重置到 TASK_LOOP 时
                            // 记录的稳定 park_ip，使 actor 干净等待下一条消息。
                            if let Some(park) = task.park_ip {
                                task.ip = park;
                            }
                            task.status = TaskStatus::Waiting("message_loop".to_string());
                        } else {
                            task.status = new_status;
                        }
                    }
                    Err(e) => {
                        // Plan 118: Store error for proper error propagation
                        // Plan 199: Include source line number in error message
                        let line = task.current_line;
                        let ip = task.ip;
                        let error_msg = if line > 0 {
                            format!("{:?} at line {} (ip={:#06x})", e, line, ip)
                        } else {
                            format!("{:?}", e)
                        };
                        task.last_error = Some(error_msg.clone());
                        // Plan 773 F-08: keep the typed identity alongside the
                        // flattened message — the staged run API reads it as
                        // the run-stage oracle code.
                        task.last_error_code = Some(vm_error_code(&e));
                        // Plan 260: Suppress error output when running in test mode (output_buffer set)
                        // Plan 011: ExitRequested is a cooperative stop, not a failure — don't log it.
                        if self.output_buffer.is_none() && !matches!(e, VMError::ExitRequested(_)) {
                            eprintln!("Task {} Error: {}", task.id, error_msg);
                        }
                        // Plan 199: Print call stack trace on error.
                        // PLAN-081 (R4③): ExitRequested is a cooperative
                        // stop, not a failure — the same guard as the error
                        // line above applies; `exit(N≠0)` used to print a
                        // noise "Stack trace:" of <anonymous> frames.
                        if self.output_buffer.is_none()
                            && !matches!(e, VMError::ExitRequested(_))
                            && !task.call_stack.is_empty()
                        {
                            eprintln!("Stack trace:");
                            for (i, frame) in task.call_stack.iter().enumerate().rev() {
                                let name = frame.fn_name.as_deref().unwrap_or("<anonymous>");
                                eprintln!("  #{} {} at line {}", i, name, frame.line);
                            }
                        }
                        task.status = TaskStatus::Terminated;
                    }
                }
            }

            // Cleanup terminated tasks
            // This is a simplified garbage collection for MVP
            /*
            self.tasks.retain(|_, v| {
                // We need to try_lock to avoid deadlocks or blocking?
                // Since we are single-threaded loop essentially here (sequential iteration),
                // blocking_lock or try_lock is fine if no one else holds it.
                // But wait, if we are in async context, blocking_lock is bad.
                // However, we cloned the Arcs above, so we don't hold the map lock.
                // Re-acquiring lock here is okay.
                if let Ok(task) = v.try_lock() {
                    task.status != TaskStatus::Terminated
                } else {
                    true // Keep it if locked (should be rare/impossible in this simple loop)
                }
            });
            */

            if alive_count == 0 {
                break;
            }

            if active_count == 0 {
                if self.tasks.is_empty() {
                    break;
                }
                // All tasks waiting/sleeping?
                sleep(Duration::from_millis(10)).await;
            }

            // Yield to tokio runtime to let other things happen
            tokio::task::yield_now().await;
        }
    }
}
