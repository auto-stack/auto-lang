//! PLAN-843 结构拆分（自 engine.rs 原样搬移；算法/行为零变化）。
use super::*;

impl AutoVM {
    /// Store a VM SQLite handle under the task that opened it.
    pub(crate) fn store_sqlite_handle(
        &self,
        task_id: TaskId,
        handle: i64,
        db: a2r_std::sqlite::SqliteDb,
    ) {
        self.sqlite_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(handle, (task_id, db));
    }

    /// Clone a live SQLite connection for one shim operation.
    pub(crate) fn sqlite_handle(&self, handle: i64) -> Option<a2r_std::sqlite::SqliteDb> {
        self.sqlite_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&handle)
            .map(|(_, db)| db.clone())
    }

    /// Insert a heap-allocated object into the unified registry
    ///
    /// Returns the object ID assigned to this object.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use auto_lang::vm::types::ListData;
    /// use auto_lang::vm::engine::AutoVM;
    ///
    /// let mut list: ListData<i32> = ListData::new();
    /// list.push(1);
    /// list.push(2);
    ///
    /// let id = vm.insert_heap_object(list);
    /// ```
    pub fn insert_heap_object<T: HeapObject + Send + Sync + 'static>(&self, obj: T) -> u64 {
        let id = self.heap_object_id_gen.fetch_add(1, Ordering::Relaxed);
        if crate::vm::rc::p419_uaf_traces(id) {
            eprintln!(
                "[P419UAF] ALLOC id={} (#{}) type={:?}",
                id,
                id - crate::vm::rc::HEAP_ID_BASE,
                obj.type_tag()
            );
            if crate::vm::rc::p419_uaf_narrow() {
                eprintln!(
                    "[P419UAF] alloc site:\n{}",
                    std::backtrace::Backtrace::force_capture()
                );
            }
        }
        self.heap_objects.insert(id, Arc::new(RwLock::new(obj)));
        // Plan 419: RC 不在此建条目 —— 对象出世时尚无 owned slot,
        // 首次 rc_push/rc_retain 建条目(计数语义见 rc.rs 模块注释)。
        self.rc_created_total.fetch_add(1, Ordering::Relaxed);
        // PLAN-062: 任何堆对象出世都算状态面突变（列表/对象字面量、JSON
        // 解析中间体、桥接实参物化等）。
        self.state_mutation_seq.fetch_add(1, Ordering::Relaxed);
        id
    }

    /// PLAN-062: 状态突变序号只读访问（fire_timer 空转判定 + 测试断言）。
    pub fn state_mutation_seq(&self) -> u64 {
        self.state_mutation_seq.load(Ordering::Relaxed)
    }

    /// PLAN-047 T-03（档 C SD-09）: 归因写点定点 bump。`path = Some(字段)`
    /// = A 类带名写（**同时** bump exact 与 `"*"` 通配——通配保证
    /// materialize/容器粗粒度 dep 面不漏）；`path = None` = B 类容器/未知
    /// 字段写（只 bump 通配）。全局 [`Self::state_mutation_seq`] 在此同步
    /// bump——既有消费者（seq 快路径/fire_timer 空转判定）语义零变化。
    pub fn bump_path(&self, heap_id: u64, path: Option<&str>) {
        self.state_mutation_seq.fetch_add(1, Ordering::Relaxed);
        *self
            .path_versions
            .entry((heap_id, "*".to_string()))
            .or_insert(0) += 1;
        if let Some(p) = path {
            *self
                .path_versions
                .entry((heap_id, p.to_string()))
                .or_insert(0) += 1;
        }
    }

    /// PLAN-047 T-03: 版本读（memo 门 version_fast 判定 + 测试断言）。
    /// 无条目 = 0（未写过该 path）。
    pub fn path_version(&self, heap_id: u64, path: &str) -> u64 {
        self.path_versions
            .get(&(heap_id, path.to_string()))
            .map(|v| *v)
            .unwrap_or(0)
    }

    /// PLAN-706 r2 T-10: dep 键堆对象存活查询——`deps_unchanged` 的死对象
    /// 伪命中守卫。`path_versions` 条目随状态替换留存，而对象已从
    /// `heap_objects` 摘除 → 死键版本永不 bump，比对恒等 = 伪命中（缓存值
    /// 陈旧/悬垂）。判据：`< HEAP_ID_BASE` 的非堆 id（单测合成键）与未分配
    /// id 按存活处理（纯版本比对语义保持）；已分配（≥ base 且 id < id_gen）
    /// 而不在 `heap_objects` = 已释放（id_gen 单调不复用，dying 宽限窗内
    /// 对象仍在表）。
    pub fn heap_dep_key_alive(&self, id: u64) -> bool {
        if id < crate::vm::rc::HEAP_ID_BASE {
            return true;
        }
        if self.heap_objects.contains_key(&id) {
            return true;
        }
        id >= self.heap_object_id_gen.load(Ordering::Relaxed)
    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-04（档 C SD-08）: 依赖录制槽（VM 读臂挂钩的宿主面）
    // ─────────────────────────────────────────────────────────────────

    /// 激活录制（ui guard 持返回的 Arc；finish 时 [`Self::take_dep_recorder`]
    /// 取走收编）。先填槽后开旗（Release）。
    pub fn set_dep_recorder(
        &self,
        rec: std::sync::Arc<std::sync::Mutex<crate::vm::dep_track::RecState>>,
    ) {
        *self.dep_recorder_slot.lock().unwrap() = Some(rec);
        self.dep_rec_active
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// 取走录制状态并去激活（ui guard finish 时收编进条目）。先关旗（ext
    /// Acquire）后清槽——读臂从此零开销直落。
    pub fn take_dep_recorder(&self) -> crate::vm::dep_track::RecState {
        self.dep_rec_active
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.dep_recorder_slot
            .lock()
            .unwrap()
            .take()
            .map(|arc| arc.lock().unwrap().clone())
            .unwrap_or_default()
    }

    /// VM 读臂录制口（未激活 = 一次原子 load 直落，非 memo 零开销红线）。
    #[inline]
    pub fn record_heap_read(&self, heap_id: u64, path: &str) {
        if !self
            .dep_rec_active
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return;
        }
        if let Some(rec) = self.dep_recorder_slot.lock().unwrap().as_ref() {
            rec.lock()
                .unwrap()
                .record(crate::vm::dep_track::DepKey::field(heap_id, path));
        }
    }

    /// VM 读臂录制口（容器/结构体整体展开面，粗粒度 `"*"`）。
    #[inline]
    pub fn record_heap_read_any(&self, heap_id: u64) {
        if !self
            .dep_rec_active
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return;
        }
        if let Some(rec) = self.dep_recorder_slot.lock().unwrap().as_ref() {
            rec.lock()
                .unwrap()
                .record(crate::vm::dep_track::DepKey::any(heap_id));
        }
    }

    /// PLAN-062 T12: 槽旧值释放——堆份额按影子、字符串按内容（Plan 510
    /// 池语义稳定面）。STORE_* 覆盖赋值的统一旧值清账点。
    pub(super) fn release_slot_old_value(&self, task: &mut AutoTask, addr: usize) {
        let old_stake = task.ram.take_stake_at(addr);
        if old_stake != 0 {
            self.rc_release_id(old_stake);
        }
        let old_nv = task.ram.read_nv(addr);
        if auto_val::is_string(old_nv) {
            self.rc_release(old_nv);
        }
    }

    /// PLAN-667 (F-01): 帧相对捕获守卫——任何 capture_slots/param_abs 槽位
    /// 读写前调用。创建帧身份不符（跨任务错帧）或创建帧已返回（含同 bp
    /// 复用后的陈旧 uid）时，在读到陈旧/他人槽位**之前**以 RuntimeError
    /// 拒绝；不得静默回落 nil/旧值/副本。
    pub(super) fn ensure_capture_frame_alive(
        &self,
        task: &AutoTask,
        closure: &Closure,
        var_name: &str,
        closure_id: u32,
    ) -> Result<(), VMError> {
        let Some((creator_task, creator_uid)) = closure.creator_frame else {
            return Ok(());
        };
        if creator_task != task.id {
            return Err(VMError::RuntimeError(format!(
                "captured variable '{}' in closure {}: created in task {} but accessed from task {} — cross-task frame captures are not supported; capture by value instead",
                var_name, closure_id, creator_task, task.id
            )));
        }
        if !task.frame_alive(creator_task, creator_uid) {
            return Err(VMError::RuntimeError(format!(
                "captured variable '{}' in closure {}: creator frame has already returned — by-reference captures are only valid while the creator frame is alive; capture by value or call within the creator's lifetime",
                var_name, closure_id
            )));
        }
        Ok(())
    }

    /// Get a heap object by ID, returning a read guard
    ///
    /// Returns `None` if the object doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use auto_lang::vm::heap_object::downcast;
    /// use auto_lang::vm::types::ListData;
    ///
    /// if let Some(obj) = vm.get_heap_object(id) {
    ///     let guard = obj.read().unwrap();
    ///     if let Some(list) = downcast::<ListData<i32>>(&*guard) {
    ///         println!("Got list with {} elements", list.len());
    ///     }
    /// }
    /// ```
    pub fn get_heap_object(&self, id: u64) -> Option<Arc<RwLock<dyn HeapObject>>> {
        // Plan 419: 毒化 canary —— 命中已释放 id 即 use-after-free(debug)。
        #[cfg(debug_assertions)]
        self.rc_check_tombstone(id);
        self.heap_objects.get(&id).map(|v| v.clone())
    }

    /// Get a heap object by ID with mutable access
    ///
    /// Returns `None` if the object doesn't exist.
    pub fn get_heap_object_mut(&self, id: u64) -> Option<Arc<RwLock<dyn HeapObject>>> {
        self.heap_objects.get(&id).map(|v| v.clone())
    }

    /// Plan 390 §15 Phase H2: unified lookup that routes by id segment.
    ///
    /// Plan 390 §15 Phase H2: unified object lookup.
    ///
    /// H2 unified the *stack encoding* (everything → `encode_object`); H3a
    /// folded `nodes`, H3b folded `objects`/`arrays` into `heap_objects` — so
    /// all VM object ids resolve in the single registry and the old id-segment
    /// routing collapses to a plain `get_heap_object`.
    pub fn get_any_object(&self, id: u64) -> Option<Arc<RwLock<dyn HeapObject>>> {
        self.get_heap_object(id)
    }

    /// Plan 390 §15 H2: rehydrate the `__variant` name of a structured message
    /// held as a `Value::VmRef`. WithBindings variant constructors
    /// (`Add(3,5)`) build an `ObjectData { __variant: "Add", a:3, b:5 }` and
    /// the send shim stores only the registry id in the mailbox. The message
    /// matcher therefore sees a `VmRef`, not an inlined `Obj`, and needs this
    /// lookup to compare the variant name against the handler pattern.
    pub(super) fn vmref_variant_name(&self, id: usize) -> Option<auto_val::AutoStr> {
        let id = id as u64;
        // Plan 390 §15 H3b: ObjectData lives in heap_objects — probe + downcast
        // (no id-segment routing).
        if let Some(obj_ref) = self.get_heap_object(id) {
            let obj = obj_ref.read().unwrap();
            if let Some(od) = obj.as_any().downcast_ref::<crate::vm::types::ObjectData>() {
                if let Some(auto_val::Value::Str(v)) = od.get(&auto_val::ValueKey::Str(
                    auto_val::AutoStr::from("__variant"),
                )) {
                    return Some(v.clone());
                }
            }
        }
        None
    }

    /// Remove a heap object from the registry
    ///
    /// Returns `None` if the object doesn't exist.
    pub fn remove_heap_object(&self, id: u64) -> Option<Arc<RwLock<dyn HeapObject>>> {
        self.heap_objects.remove(&id).map(|(_, v)| v)
    }

    // ===== Plan 377 §4.3: heap-aware 64-bit push/pop =====
    //
    // virt_memory 的 push_i64/u64/pop_i64/u64 只处理 48 位内联编码（无 VM 访问），
    // 溢出或 BIGINT 会 panic。以下方法在 AutoVM 层（有 heap_objects 访问）提供完整
    // 64 位范围支持：push 溢出时 BigInt 堆装箱，pop 遇 BIGINT 时解引用堆对象。
    // engine opcode / native 调用方应使用这些方法替代 task.ram.push/pop_i64/u64。

    /// Get the number of heap objects in the registry
    pub fn heap_object_count(&self) -> usize {
        self.heap_objects.len()
    }

    /// Check if a heap object exists in the registry
    pub fn contains_heap_object(&self, id: u64) -> bool {
        self.heap_objects.contains_key(&id)
    }

    /// Clear all heap objects from the registry
    pub fn clear_heap_objects(&self) {
        self.heap_objects.clear();
    }

    // ============================================================================
    // Plan 087 Phase 2: Generic Instance Value Helper Functions
    // ============================================================================

}
