//! PLAN-843 结构拆分（自 engine.rs 原样搬移；算法/行为零变化）。
use super::*;

impl AutoVM {
    /// Plan 394 Phase A: allocate an External future for a native async source.
    pub fn register_external_future(&self, owner_task: TaskId) -> u32 {
        let future_id = self.future_id_gen.fetch_add(1, Ordering::SeqCst);
        let future = FutureValue {
            body_offset: 0,
            state: FutureState::Pending,
            result: None,
            owner_task_id: owner_task,
            captures: HashMap::new(),
            kind: FutureKind::External,
        };
        self.futures
            .insert(future_id, Arc::new(RwLock::new(future)));
        future_id
    }

    /// Plan 394 Phase A: resolve an External future from a worker thread.
    /// Ok(v) → Ready with result; Err(_) → Failed (await yields null).
    pub fn resolve_external_future(&self, future_id: u32, result: Result<auto_val::Value, String>) {
        if let Some(future_arc) = self.futures.get(&future_id) {
            let mut future = future_arc.write().unwrap();
            match result {
                Ok(v) => {
                    future.result = Some(v);
                    future.state = FutureState::Ready;
                }
                Err(_) => {
                    future.state = FutureState::Failed;
                }
            }
        }
    }

    /// Plan 394: encode future id onto the VM stack (`(id << 8) | 0xF0`).
    pub fn encode_future_bits(future_id: u32) -> i32 {
        ((future_id as i32) << 8) | 0xF0
    }

    /// PLAN-776 R7-01：Future 句柄解码——TAG_FUTURE（逐值来源）优先；
    /// i32 `0xF0` 位模式为 legacy 兼容臂（句柄经 `Value::Int` 载体往返
    /// ——容器元素/闭包捕获 env——标签退化为位形态，功能保留）。
    pub fn decode_future_handle(nv: auto_val::NanoValue) -> Option<u32> {
        if auto_val::is_future(nv) {
            Some(auto_val::decode_future(nv))
        } else if auto_val::is_i32(nv) {
            let bits = auto_val::decode_i32(nv);
            if (bits & 0xFF) == 0xF0 {
                Some((bits >> 8) as u32)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Plan 442 A5: schedule a one-shot timer. Non-blocking — the callback
    /// fires on the render loop's timer tick (16ms granularity, matching the
    /// auto-down SchedulerTimer batch cadence). Returns the timer id handle.
    pub fn set_timer(&self, callback: TimerCallback, delay_ms: u64) -> u32 {
        let id = self.timer_id_gen.fetch_add(1, Ordering::Relaxed);
        self.timers.insert(
            id,
            TimerEntry {
                deadline: std::time::Instant::now()
                    + std::time::Duration::from_millis(delay_ms.max(1)),
                callback,
            },
        );
        id
    }

    /// Plan 442 A5: cancel a one-shot timer. Returns whether a pending
    /// timer was actually removed.
    pub fn clear_timer(&self, id: u32) -> bool {
        self.timers.remove(&id).is_some()
    }

    /// Plan 442 A5: remove and return the timers whose deadline has passed.
    /// One-shot semantics: entries are taken out of the registry when they
    /// fire (a timer that stays would keep the render tick subscribed
    /// forever and could re-fire on every poll).
    pub fn due_timers(&self) -> Vec<(u32, TimerCallback)> {
        let now = std::time::Instant::now();
        let due_ids: Vec<u32> = self
            .timers
            .iter()
            .filter(|e| e.value().deadline <= now)
            .map(|e| *e.key())
            .collect();
        let mut out = Vec::with_capacity(due_ids.len());
        for id in due_ids {
            if let Some((_, entry)) = self.timers.remove(&id) {
                out.push((id, entry.callback));
            }
        }
        out
    }

    /// Plan 442 A5: whether any one-shot timer is pending (render loop uses
    /// this to decide whether to subscribe to the timer tick at all).
    pub fn has_pending_timers(&self) -> bool {
        !self.timers.is_empty()
    }

    /// Plan 224: Handle AWAIT_FUTURE for a pending future.
    /// Executes the async body bytecode via recursive execute_single_frame.
    pub fn handle_await_future(
        &self,
        task: &mut AutoTask,
        future_id: u32,
        body_offset: u32,
    ) -> Result<(), VMError> {
        const MAX_RECURSION_DEPTH: u32 = 64;
        // Plan 348 Task 22: if the future carries captures, install a synthetic
        // closure so LOAD_CAPTURED/STORE_CAPTURED inside the body can resolve.
        // The closure_id is saved/restored across the body execution below so
        // the caller's closure context is untouched on return.
        let saved_closure_id = task.current_closure_id;
        let saved_saved_closure_id = task.saved_closure_id;
        let installed_closure_id = self.install_future_captures(future_id, body_offset);
        if let Some(cid) = installed_closure_id {
            task.current_closure_id = Some(cid);
            // Keep saved_closure_id in sync so RET inside the body restores to
            // the synthetic closure (allowing multiple LOAD_CAPTURED ops).
            task.saved_closure_id = Some(cid);
        }
        let result = self.execute_future_body(task, future_id, body_offset, 0, MAX_RECURSION_DEPTH);
        // Phase B: body suspended mid-await — keep synthetic closure so resume
        // can still LOAD_CAPTURED; caller treats waiting_future_id as Waiting.
        if task.waiting_future_id.is_some() {
            return Ok(());
        }
        task.current_closure_id = saved_closure_id;
        task.saved_closure_id = saved_saved_closure_id;
        result
    }

    /// Plan 348 Task 22: register a synthetic Closure for a future's captures.
    /// Returns the closure_id (so the caller can wire it into the task's
    /// current_closure_id), or None if the future has no captures.
    fn install_future_captures(&self, future_id: u32, body_offset: u32) -> Option<u32> {
        let captures = {
            let future_arc = self.futures.get(&future_id)?;
            let future = future_arc.read().unwrap();
            if future.captures.is_empty() {
                return None;
            }
            future.captures.clone()
        };
        let closure_id = self.closure_id_gen.fetch_add(1, Ordering::Relaxed);
        self.closures.insert(
            closure_id,
            Closure {
                func_addr: body_offset,
                env: captures,
                n_args: 0,
                capture_slots: HashMap::new(),
                param_abs: HashMap::new(),
                creator_frame: None, // PLAN-667: 合成闭包无帧相对捕获
            },
        );
        Some(closure_id)
    }

    /// Recursively execute a future's body bytecode.
    fn execute_future_body(
        &self,
        task: &mut AutoTask,
        future_id: u32,
        body_offset: u32,
        depth: u32,
        max_depth: u32,
    ) -> Result<(), VMError> {
        if depth >= max_depth {
            // Set future to Failed on recursion limit
            if let Some(future_arc) = self.futures.get(&future_id) {
                future_arc.write().unwrap().state = FutureState::Failed;
            }
            return Err(VMError::RuntimeError(format!(
                "Future recursion depth exceeded ({})",
                max_depth
            )));
        }

        // Save current IP so we can restore after body execution
        let saved_ip = task.ip;
        task.ip = body_offset as usize;

        // Execute body until Return/AwaitFuture/Error
        const BODY_BUDGET: u32 = 10_000;
        let mut result_value = auto_val::Value::Int(0);

        loop {
            match self.execute_single_frame(task, BODY_BUDGET) {
                FrameResult::Return => {
                    // Body completed — read result from stack top.
                    // PLAN-050 T9: pop_nv+decode_tagged_nv（与 RET 的 write_nv
                    // 对齐）——此前 pop_i32+decode_tagged_value 把 nanobox
                    // 标签位当 raw i32 重解码，return None 的 null nv 被拍平
                    // 成 Int(0)/Int(-1)，`.x = f().await` 后 `.x != None` 恒真。
                    // PLAN-776 R5-01：bp==0（dispatch 直驱立即体——handler 已
                    // RET、任务栈归 base，RET bp==0 短路不弹帧）时值就在
                    // 栈顶；原 `sp > bp + 1` 只覆盖帧内（await/spawn）形态，
                    // base 形态漏读 → 终值恒 Int(0)。
                    let has_result = if task.bp == 0 {
                        task.ram.sp > task.bp
                    } else {
                        task.ram.sp > task.bp + 1
                    };
                    if has_result {
                        let raw = task.ram.pop_nv();
                        // PLAN-776 R1-F04：body 栈槽份额终结（与 RET 指令的
                        // 影子转移语义对齐）——result 是 VmRef（无份额），
                        // push_value 回调用者栈时另行建份额（Plan 419 咽喉
                        // 点）；此前只 pop 不释放，每 Internal body 的对象
                        // 终值悬空一份（dying 宽限窗内的 retain 复活使窗口
                        // 期安全）。
                        if auto_val::is_object(raw) || auto_val::is_list(raw) {
                            self.rc_release(raw);
                        }
                        result_value = self.decode_tagged_nv(raw);
                    }
                    break;
                }
                FrameResult::AwaitFuture {
                    future_id: inner_id,
                    body_offset: inner_offset,
                } => {
                    // Recursive await: execute inner future body
                    self.execute_future_body(task, inner_id, inner_offset, depth + 1, max_depth)?;
                    // Plan 394 Phase B: nested body suspended on external await
                    if task.waiting_future_id.is_some() {
                        return Ok(());
                    }
                    // Inner future result was pushed onto stack, continue body execution
                }
                FrameResult::BudgetExhausted => {
                    // Continue executing body
                    continue;
                }
                FrameResult::Yielded => {
                    // Plan 394 Phase B: external await inside `~{}` body —
                    // save continuation, leave IP mid-body, do NOT restore
                    // outer saved_ip. Wake source 6 pushes the result; resume
                    // continues from async_frames.
                    if let Some(fid) = task.waiting_future_id {
                        task.async_frames.push(crate::vm::task::AsyncFrame {
                            resume_ip: task.ip,
                            resume_bp: task.bp,
                            future_id: fid,
                            outer_future_id: future_id,
                            outer_saved_ip: saved_ip,
                        });
                        return Ok(());
                    }
                    continue;
                }
                FrameResult::Error(e) => {
                    if let Some(future_arc) = self.futures.get(&future_id) {
                        future_arc.write().unwrap().state = FutureState::Failed;
                    }
                    task.ip = saved_ip;
                    return Err(e);
                }
                FrameResult::Continue => unreachable!(),
            }
        }

        // Restore IP
        task.ip = saved_ip;

        // Update future state
        if let Some(future_arc) = self.futures.get(&future_id) {
            let mut future = future_arc.write().unwrap();
            future.state = FutureState::Ready;
            future.result = Some(result_value.clone());
        }

        // Push result onto caller's stack
        Self::push_value(task, &result_value, self);

        Ok(())
    }

    /// Plan 394 Phase B: continue a `~{}` body after an external await resume.
    /// The wake path has already pushed the await result; IP is mid-body.
    pub(super) fn resume_suspended_body(
        &self,
        task: &mut AutoTask,
        frame: crate::vm::task::AsyncFrame,
    ) -> Result<TaskStatus, VMError> {
        const BODY_BUDGET: u32 = 10_000;
        let mut result_value = auto_val::Value::Int(0);
        loop {
            match self.execute_single_frame(task, BODY_BUDGET) {
                FrameResult::Return => {
                    if task.ram.sp > task.bp + 1 {
                        let raw = task.ram.pop_nv();
                        // PLAN-776 R1-F04：与 execute_future_body 同款——body
                        // 栈槽份额终结（VmRef 无份额，push_value 另建）。
                        if auto_val::is_object(raw) || auto_val::is_list(raw) {
                            self.rc_release(raw);
                        }
                        result_value = self.decode_tagged_nv(raw);
                    }
                    // Complete the outer `~{}` future
                    if let Some(future_arc) = self.futures.get(&frame.outer_future_id) {
                        let mut future = future_arc.write().unwrap();
                        future.state = FutureState::Ready;
                        future.result = Some(result_value.clone());
                    }
                    // Drop all continuation frames for this body; restore outer IP
                    let mut restore_ip = frame.outer_saved_ip;
                    // Pop every frame belonging to this body (sequential external
                    // awaits each push one; leaving orphans re-enters resume).
                    while let Some(f) = task.async_frames.pop() {
                        if f.outer_future_id == frame.outer_future_id {
                            restore_ip = f.outer_saved_ip;
                        } else {
                            task.async_frames.push(f);
                            break;
                        }
                    }
                    task.ip = restore_ip;
                    Self::push_value(task, &result_value, self);
                    return Ok(TaskStatus::Ready);
                }
                FrameResult::AwaitFuture {
                    future_id: inner_id,
                    body_offset: inner_offset,
                } => {
                    self.handle_await_future(task, inner_id, inner_offset)?;
                    if task.waiting_future_id.is_some() {
                        return Ok(TaskStatus::Waiting("future".into()));
                    }
                }
                FrameResult::BudgetExhausted | FrameResult::Yielded => {
                    if let Some(fid) = task.waiting_future_id {
                        task.async_frames.push(crate::vm::task::AsyncFrame {
                            resume_ip: task.ip,
                            resume_bp: task.bp,
                            future_id: fid,
                            outer_future_id: frame.outer_future_id,
                            outer_saved_ip: frame.outer_saved_ip,
                        });
                        return Ok(TaskStatus::Waiting("future".into()));
                    }
                    continue;
                }
                FrameResult::Error(e) => {
                    if let Some(future_arc) = self.futures.get(&frame.outer_future_id) {
                        future_arc.write().unwrap().state = FutureState::Failed;
                    }
                    task.async_frames
                        .retain(|f| f.outer_future_id != frame.outer_future_id);
                    task.ip = frame.outer_saved_ip;
                    return Err(e);
                }
                FrameResult::Continue => unreachable!(),
            }
        }
    }

}
