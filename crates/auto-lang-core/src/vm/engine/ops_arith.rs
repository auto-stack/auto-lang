//! PLAN-843 结构拆分：run_one_instruction 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl AutoVM {
    /// PLAN-843 尾递归派发段（自 run_one_instruction 连续臂切片，臂体逐字保留）。
    pub(super) fn ops_arith(
        &self,
        task: &mut AutoTask,
        op: OpCode,
        op_byte: u8,
    ) -> Result<StepResult, VMError> {
        match op {
            // Plan 073: Object field access (obj.field)
            OpCode::GET_FIELD => {
                use crate::vm::generic_registry::GenericInstanceData;
                let field_idx = self.flash.read_u32(task.ip);
                task.ip += 4;

                // Pop object ID from stack
                let receiver_nv;
                let obj_id = {
                    let nv = task.ram.pop_nv();
                    receiver_nv = nv;
                    // Plan 419: receiver 的栈上 stake 在操作完成后释放
                    // (arm 末尾;错误路径泄漏一个 stake —— 安全方向)。
                    // 不能在此立即释放:`Cell{...}.x` 临时对象会先死后用。
                    if auto_val::is_i32(nv) {
                        auto_val::decode_i32(nv) as u64
                    } else if auto_val::is_object(nv) {
                        auto_val::decode_object(nv) as u64
                    } else if auto_val::is_list(nv) {
                        // PLAN-053 P-053-1: TAG_LIST 接收者——computed
                        // helper 实参经 bridge call_vm_fn 按 encode_list
                        // 编码传入，`.length` 落此。decode_list 与
                        // decode_i32 位型同值(低 32 位即堆 id)，但显式
                        // 分派消除误报 stderr(musk 实机 571×
                        // "GET_FIELD non-i32 obj_id field=length" 噪音)。
                        auto_val::decode_list(nv) as u64
                    } else {
                        // mem 复盘修复：该臂此前每命中一行 eprintln
                        // （sys-monitor 曾 6 分钟 344MB 日志饿死 UI 线程）
                        // 且不结算槽位 stake（每次命中泄漏一份 rc 份额）。
                        // 现改为环境门控诊断 + stake 结算，与 SET_FIELD
                        // 同款纪律。
                        let _ = task.ram.take_stake_at(task.ram.sp);
                        if std::env::var_os("AUTO_DEBUG_GETFIELD").is_some() {
                            let fn_name = task
                                .call_stack
                                .last()
                                .map(|f| f.fn_name.clone().unwrap_or_default())
                                .unwrap_or_default();
                            let field_name = self
                                .strings
                                .read()
                                .unwrap()
                                .get(field_idx as usize)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            eprintln!("[GET_FIELD] non-i32 obj_id: raw={:016x} field={} fn={} bp={} ip={}", nv, field_name, fn_name, task.bp, task.ip);
                        }
                        auto_val::decode_i32(nv) as u64
                    }
                };

                // Get field name from strings pool (Plan 073: Now uses RwLock)
                let strings = self.strings.read().unwrap();
                let field_name = if let Some(field_bytes) = strings.get(field_idx as usize) {
                    String::from_utf8_lossy(field_bytes).to_string()
                } else {
                    drop(strings);
                    return Err(VMError::RuntimeError(format!(
                        "Invalid string index: {} (pool size={}, obj_id={}, ip={})",
                        field_idx,
                        self.strings.read().unwrap().len(),
                        obj_id,
                        task.ip
                    )));
                };
                drop(strings); // Release lock before potentially writing below

                // PLAN-047 T-04: VM 读臂依赖录制（field_name 可证面；
                // 非 i32 接收者的 decode 兜底 id 可能含位型碰撞——过录
                // 是保守安全方向，多失效只多一次重求值）。
                self.record_heap_read(obj_id, &field_name);

                // PLAN-055: TAG_STRING 接收者——JS 语义 `.length` = 字符数
                // （与 web a2ts `text.length` 同值）。此前 Str 接收者落
                // "non-i32 obj_id" 噪音臂 push 0——musk estimateTokens 的
                // `text.length - cjk` 恒负零系，全部思考块显示 "1 tokens"
                // （P055 探针实测 est(17 chars) = Int(1)；agentAvatarData
                // 的 hash 循环同根失效）。
                if auto_val::is_string(receiver_nv) {
                    if field_name == "length" {
                        let str_idx = auto_val::decode_string(receiver_nv) as usize;
                        let n = self
                            .strings
                            .read()
                            .unwrap()
                            .get(str_idx)
                            .map(|b| String::from_utf8_lossy(b).chars().count() as i32)
                            .unwrap_or(0);
                        task.ram.push_i32(n);
                    } else {
                        task.ram.push_i32(0);
                    }
                } else if let Some(heap_ref) = self.heap_objects.get(&obj_id) {
                    // Heap objects (4M+): Node (H3a), ObjectData (H3b),
                    // GenericInstanceData, RustStdlibObject.
                    let heap_obj = heap_ref.read().unwrap();
                    // Plan 390 §15 H3a: Node now lives in heap_objects —
                    // check it first (mold template node-field priority).
                    if let Some(node) = heap_obj.as_any().downcast_ref::<auto_val::Node>() {
                        let prop = node.get_prop(field_name.as_str());
                        let resolved: auto_val::Value = if !prop.is_nil() {
                            prop
                        } else {
                            match field_name.as_str() {
                                "id" if !node.id.is_empty() => {
                                    auto_val::Value::Str(node.id.clone())
                                }
                                "name" if !node.name.is_empty() => {
                                    auto_val::Value::Str(node.name.clone())
                                }
                                "text" if !node.text.is_empty() => {
                                    auto_val::Value::Str(node.text.clone())
                                }
                                _ => auto_val::Value::Nil,
                            }
                        };
                        if !resolved.is_nil() {
                            match resolved {
                                auto_val::Value::Str(s) => {
                                    // Plan 423 P5 续修:入池收口(原裸
                                    // strings.push 无 pool_state/dedup)。
                                    self.intern_runtime_str(task, s.as_bytes().to_vec());
                                }
                                auto_val::Value::Int(i) => task.ram.push_i32(i),
                                auto_val::Value::Uint(u) => task.ram.push_i32(u as i32),
                                auto_val::Value::Bool(b) => {
                                    task.ram.push_i32(if b { 1 } else { 0 })
                                }
                                // Plan 419: 子对象引用入栈 +1。
                                auto_val::Value::VmRef(r) => self.rc_push_id(task, r.id as u64),
                                _ => task.ram.push_i32(0),
                            }
                        } else {
                            task.ram.push_i32(0);
                        }
                    } else if let Some(obj) = heap_obj
                        .as_any()
                        .downcast_ref::<crate::vm::types::ObjectData>()
                    {
                        // Plan 390 §15 H3b: obj literals are ObjectData in
                        // heap_objects.
                        // Try multiple key formats: string, integer, boolean
                        // This handles cases like { 1: 2, 3: 4 } accessed as a.3
                        let value = if let Some(v) =
                            obj.get(&auto_val::ValueKey::Str(field_name.clone().into()))
                        {
                            Some(v.clone())
                        } else if let Ok(int_key) = field_name.parse::<i32>() {
                            obj.get(&auto_val::ValueKey::Int(int_key)).cloned()
                        } else if field_name == "true" {
                            obj.get(&auto_val::ValueKey::Bool(true)).cloned()
                        } else if field_name == "false" {
                            obj.get(&auto_val::ValueKey::Bool(false)).cloned()
                        } else {
                            None
                        };

                        if let Some(value) = value {
                            // Push field value onto stack based on type
                            match value {
                                auto_val::Value::Int(i) => task.ram.push_i32(i),
                                auto_val::Value::Uint(u) => task.ram.push_i32(u as i32),
                                auto_val::Value::Float(f) => task.ram.push_f32(f as f32),
                                auto_val::Value::Double(d) => task.ram.push_f64(d),
                                auto_val::Value::Bool(b) => {
                                    // Plan 402 §13.10: push TAG_BOOL (encode_bool),
                                    // not raw i32. Otherwise `obj.bool_field == false`
                                    // compares i32(0) vs TAG_BOOL(false) → different
                                    // nanbox tags → EQ returns false (6892: both must
                                    // be is_i32, or both is_bool; a mix falls through
                                    // to `else { false }`). This broke every bool-field
                                    // comparison in minesweeper (mine==false, revealed
                                    // ==false, flagged==false), so SET_ELEM never ran
                                    // and the board never updated.
                                    task.ram.push_nv(auto_val::encode_bool(b))
                                }
                                auto_val::Value::Char(c) => task.ram.push_i32(c as i32),
                                auto_val::Value::Str(s) => {
                                    // Push tagged string index (NaN-boxed)
                                    // Plan 423 P5 续修:入池收口(add_string
                                    // + 份额入栈;原裸 push 无计数覆盖)。
                                    self.intern_runtime_str(task, s.as_bytes().to_vec());
                                }
                                auto_val::Value::Nil => task.ram.push_nv(auto_val::encode_null()), // Plan 474 待澄清#4 读侧闭环: Nil 字段读回 null nv(原 push_i32(0),与 Plan 044 __json_object 缺键 null 不一致)
                                // Plan 073: Nested objects/arrays - push their ID
                                auto_val::Value::VmRef(vm_ref) => {
                                    // Plan 419: 子对象引用入栈 +1。
                                    self.rc_push_id(task, vm_ref.id as u64);
                                }
                                // PLAN-776 R9-02：Future 句柄字段还原
                                // TAG_FUTURE（原落 `_` placeholder 0）。
                                auto_val::Value::Future(f) => {
                                    task.ram.push_nv(auto_val::encode_future(f.id as u32));
                                }
                                _ => {
                                    // Unsupported type - push 0 as placeholder
                                    task.ram.push_i32(0);
                                }
                            }
                        } else {
                            // PLAN-053 P-053-6: 缺键读 null（对齐
                            // PLAN-044 __json_object 缺键语义与 web 轨
                            // undefined→?? 兜底惯用）——原硬报错使
                            // `msg.blocks ?? [textBlock(msg)]` 一类
                            // Option 链在普通 obj 上直接炸（musk 消息
                            // 渲染链 messageDisplayBlocks 现场）。
                            task.ram.push_nv(auto_val::encode_null());
                        }
                    } else if let Some(inst) =
                        heap_obj.as_any().downcast_ref::<GenericInstanceData>()
                    {
                        let field_idx = inst.field_names.iter().position(|n| n == &field_name);
                        if let Some(idx) = field_idx {
                            if let Some(value) = inst.get_field(idx) {
                                match value {
                                    auto_val::Value::Int(i) => {
                                        // Plan 423 P5 续修(RC 根引用缺口):
                                        // Int ≥ HEAP_ID_BASE 是裸堆 id(数组/
                                        // 列表槽约定,Plan 057 Bug 2)—— 引用
                                        // 值入栈必须 +1。此前裸 push_i32,
                                        // GET_ELEM/ARRAY_LEN 等消费弹栈释放
                                        // 时把字段持有份额吃掉归零 → 字段悬垂
                                        // → canary UAF(015 TogglePin 链式写
                                        // 实测 rc.rs:378)。
                                        if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
                                            self.rc_push_id(task, *i as u64);
                                        } else {
                                            task.ram.push_i32(*i);
                                        }
                                    }
                                    auto_val::Value::Uint(u) => task.ram.push_i32(*u as i32),
                                    auto_val::Value::Float(f) => task.ram.push_f32(*f as f32),
                                    auto_val::Value::Double(d) => task.ram.push_f64(*d),
                                    auto_val::Value::Bool(b) => {
                                        // Audit B12(a)/A4 family: push TAG_BOOL
                                        // (encode_bool), not raw i32 — `field == false`
                                        // compares against a bool nanbox and never
                                        // matched raw 0/1 (013 Init count loop read
                                        // .done on each seed and counted nothing).
                                        // Same fix as ObjectData (Plan 402 §13.10).
                                        task.ram.push_nv(auto_val::encode_bool(*b))
                                    }
                                    auto_val::Value::Char(c) => task.ram.push_i32(*c as i32),
                                    auto_val::Value::Str(s) => {
                                        // Plan 423 P5 续修:入池收口(原裸
                                        // strings.push 无 pool_state/dedup ——
                                        // 016-calendar today 参数槽悬垂根因)。
                                        self.intern_runtime_str(task, s.as_bytes().to_vec());
                                    }
                                    auto_val::Value::Nil => {
                                        task.ram.push_nv(auto_val::encode_null())
                                    } // Plan 474 待澄清#4 读侧闭环: Nil 字段读回 null nv(原 push_i32(0),与 Plan 044 __json_object 缺键 null 不一致)
                                    auto_val::Value::VmRef(vm_ref) => {
                                        // Plan 419: 子对象引用入栈 +1。
                                        self.rc_push(
                                            task,
                                            auto_val::encode_object(vm_ref.id as u32),
                                        );
                                    }
                                    // PLAN-776 R9-02：Future 句柄字段还原
                                    // TAG_FUTURE（原落 `_` placeholder 0）。
                                    auto_val::Value::Future(f) => {
                                        task.ram.push_nv(auto_val::encode_future(f.id as u32));
                                    }
                                    _ => {
                                        task.ram.push_i32(0);
                                    }
                                }
                            } else {
                                task.ram.push_i32(0);
                            }
                        } else if inst.mono_name.starts_with("__json_object") {
                            // PLAN-044: json-backed instance (axum Json/Query
                            // extractor marshalling) — missing key reads as
                            // null (Option.unwrap_or chains depend on it).
                            task.ram.push_nv(auto_val::encode_null());
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Field '{}' not found on type instance {} (fields: {:?})",
                                field_name, inst.mono_name, inst.field_names
                            )));
                        }
                    } else if let Some(rust_obj) = heap_obj
                        .as_any()
                        .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>(
                    ) {
                        // Opaque external crate type — try opaque dispatch for field access
                        let type_name = &rust_obj.type_name;
                        if type_name == "std::process::Output"
                            && (field_name == "stdout" || field_name == "stderr")
                        {
                            drop(heap_obj);
                            if let Some(obj) = self.heap_objects.get(&obj_id) {
                                let guard = obj.read().unwrap();
                                if let Some(rust_obj2) = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                                        if let Some(output) = rust_obj2.downcast_ref::<std::process::Output>() {
                                            let bytes = if field_name == "stdout" { &output.stdout } else { &output.stderr };
                                            let vec_obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("Vec<u8>", bytes.to_vec());
                                            let handle = self.insert_heap_object(vec_obj);
                                            // Plan 419: 新堆对象引用入栈 +1。
                                            self.rc_push_id(task, handle);
                                        } else {
                                            task.ram.push_i32(0);
                                        }
                                    } else {
                                        task.ram.push_i32(0);
                                    }
                            } else {
                                task.ram.push_i32(0);
                            }
                        } else {
                            let native_name =
                                crate::vm::native_catalog::lookup_opaque_dispatch_by_type(
                                    type_name,
                                    &field_name,
                                );
                            if let Some(native_name) = native_name {
                                drop(heap_obj);
                                // Plan 419: 接收者重新入栈作 self(+1;shim
                                // 侧 raw pop 不减,由 GET_FIELD 入口的 pop 减过)。
                                self.rc_push(task, auto_val::encode_object(obj_id as u32));
                                if let Some(&native_id) =
                                    crate::vm::native_registry::NATIVE_ID_MAP.get(native_name)
                                {
                                    if let Some(shim) =
                                        self.native_interface.get(native_id).cloned()
                                    {
                                        shim(task, self)?;
                                    } else {
                                        task.ram.pop_i32();
                                        task.ram.push_i32(0);
                                    }
                                } else {
                                    task.ram.pop_i32();
                                    task.ram.push_i32(0);
                                }
                            } else {
                                task.ram.push_i32(0);
                            }
                        }
                    } else if let Some(dep_obj) = heap_obj
                        .as_any()
                        .downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>(
                    ) {
                        // PLAN-591 T1: offset 直读优先——layout 命中标量字段
                        // 时按探针偏移读 cdylib 堆(乱序 repr(Rust) 布局的
                        // 真读面);String/嵌套句柄等非标量落 None → 下方的
                        // PLAN-592 T7 合成 getter 路由(clone 语义)。
                        let scalar = crate::vm::ffi::dep_methods::read_scalar_field(
                            dep_obj,
                            field_name.as_str(),
                        );
                        // PLAN-592 T7: dep crate 对象字段访问桥——`p.x` 路由到
                        // shim 包合成 getter("短类型名.字段名" 在 METHODS 表,
                        // 即 rustdoc 公共字段合成面);未命中显式报错,替代
                        // 此前的静默 push 0(p 字段名拼错拿回 0 的正确性陷阱)。
                        let short_type = dep_obj.short_type.clone();
                        let full_type = dep_obj.full_type.clone();
                        drop(heap_obj);
                        match scalar {
                            Some(crate::vm::ffi::dep_methods::ScalarFieldValue::I(i)) => {
                                self.push_i64_vm(task, i);
                            }
                            Some(crate::vm::ffi::dep_methods::ScalarFieldValue::F(v)) => {
                                task.ram.push_f64(v);
                            }
                            Some(crate::vm::ffi::dep_methods::ScalarFieldValue::B(b)) => {
                                task.ram.push_nv(auto_val::encode_bool(b));
                            }
                            None => {
                                // Plan 419: 接收者重新入栈作 self(+1;marshaller 侧
                                // raw pop 不减,与 native_catalog 字段臂同一纪律)。
                                self.rc_push(task, auto_val::encode_object(obj_id as u32));
                                let hit = crate::vm::ffi::dep_methods::dispatch(
                                    &short_type,
                                    field_name.as_str(),
                                    task,
                                    self,
                                )?;
                                if !hit {
                                    task.ram.pop_i32();
                                    return Err(VMError::RuntimeError(format!(
                                            "unknown field '{field_name}' on dep object {full_type} (无合成 getter:字段非 pub,或字段类型不在 标量/Str/Opaque 白名单)"
                                        )));
                                }
                            }
                        }
                    } else {
                        // plan-022 (auto-down): `.length` on a heap LIST
                        // that arrived through an obj-typed parameter —
                        // lists carry no named fields, so the named-field
                        // downcasts above all miss. Mirror ARRAY_LEN's
                        // element-count semantics before the 0 sentinel.
                        if field_name == "length" {
                            use crate::vm::types::ListData;
                            let len = if let Some(l) =
                                heap_obj.as_any().downcast_ref::<ListData<i32>>()
                            {
                                l.elems.len() as i32
                            } else if let Some(l) =
                                heap_obj.as_any().downcast_ref::<ListData<String>>()
                            {
                                l.elems.len() as i32
                            } else if let Some(l) =
                                heap_obj.as_any().downcast_ref::<ListData<bool>>()
                            {
                                l.elems.len() as i32
                            } else if let Some(l) = heap_obj
                                .as_any()
                                .downcast_ref::<ListData<auto_val::Value>>()
                            {
                                l.elems.len() as i32
                            } else {
                                -1
                            };
                            if len >= 0 {
                                task.ram.push_i32(len);
                            } else {
                                task.ram.push_i32(0);
                            }
                        } else {
                            task.ram.push_i32(0);
                        }
                    }
                } else {
                    // Object not found - push 0 as error sentinel
                    task.ram.push_i32(0);
                }
                // Plan 419: 操作完成,receiver 的栈上 stake 死亡。
                self.rc_release(receiver_nv);
            }
            // === Arithmetic ===
            OpCode::ADD => {
                // Plan 539 W1 (T10): py-handle operands route to the
                // Python dunder (arms stay one-liners to bound the
                // recursive frame size — see the aavm2 overflow note).
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_arith("__add__", "__radd__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    // Plan 550 T03: 算术双操作数经 null 守卫弹出（TAG_NULL
                    // 拒收，含下方字符串拼接臂——decode_i32(null) 垃圾入串
                    // 的 539 探针 2 病灶在分派前拦截）。
                    let ((a_bits, a_is_f64), (b_bits, b_is_f64)) =
                        task.ram.pop_arith_pair_non_null("+")?;
                    if a_is_f64 && b_is_f64 {
                        task.ram
                            .push_f64(f64::from_bits(a_bits) + f64::from_bits(b_bits));
                    } else if a_is_f64 || b_is_f64 {
                        // Mixed f64 + non-f64: promote both to f64
                        let a = if a_is_f64 {
                            f64::from_bits(a_bits)
                        } else {
                            nanbox_single_to_f64(a_bits)
                        };
                        let b = if b_is_f64 {
                            f64::from_bits(b_bits)
                        } else {
                            nanbox_single_to_f64(b_bits)
                        };
                        task.ram.push_f64(a + b);
                    } else if auto_val::is_f32(a_bits) && auto_val::is_f32(b_bits) {
                        task.ram
                            .push_f32(auto_val::decode_f32(a_bits) + auto_val::decode_f32(b_bits));
                    } else if auto_val::is_f32(a_bits) || auto_val::is_f32(b_bits) {
                        // Plan 576: f32×int 混算按 tag 解码（int 按值转 f32）。
                        let a = nanbox_single_to_f32(a_bits);
                        let b = nanbox_single_to_f32(b_bits);
                        task.ram.push_f32(a + b);
                    } else if auto_val::is_string(a_bits) || auto_val::is_string(b_bits) {
                        // String concatenation: decode both as strings, concatenate, push new string
                        // Plan 567 T01（P539-D1 双症状收口）: 先读后放——对齐
                        // STR_CAT 单次释放纪律。原实现有两处病灶：①release 在读
                        // 之前，左结合链的中间结果槽被 FREE+tombstone 后
                        // strings.get 归空（"got d" 病灶，池日志实证：retain"ab"
                        // 0→1 → release 1→0 → FREE → 读空 → intern"cd"）；
                        // ②add_string 前另有第二对 release 对同一操作数重复扣减，
                        // rc 从 0 回绕 u32::MAX = P053-8 freelist 幻影条目。
                        let a_str = if auto_val::is_string(a_bits) {
                            let idx = auto_val::decode_string(a_bits) as usize;
                            let strings = self.strings.read().unwrap();
                            strings.get(idx).cloned().unwrap_or_default()
                        } else {
                            Self::nv_concat_operand_display(a_bits).into_bytes()
                        };
                        let b_str = if auto_val::is_string(b_bits) {
                            let idx = auto_val::decode_string(b_bits) as usize;
                            let strings = self.strings.read().unwrap();
                            strings.get(idx).cloned().unwrap_or_default()
                        } else {
                            Self::nv_concat_operand_display(b_bits).into_bytes()
                        };
                        // Plan 419 Phase 2: 操作数 stake 随消费死亡（读完再放）。
                        self.rc_release(a_bits);
                        self.rc_release(b_bits);
                        let mut combined = a_str;
                        combined.extend_from_slice(&b_str);
                        // Plan 419 Phase 2: 同 STR_CAT(原为裸 push,dedup 漂移源)。
                        let new_idx = self.add_string(combined);
                        self.rc_push_str_idx(task, new_idx as usize);
                    } else {
                        // PLAN-026 T-03 配套: i64 参与时按 i64 值算并
                        // push_i64(纯 i32 对保持 i32 通路,既有程序零变化);
                        // 此前 i64 位模式被 decode_i32 = 垃圾(i64_probe2
                        // "mix sum=63" 实证)。
                        if auto_val::is_i64(a_bits) || auto_val::is_i64(b_bits) {
                            let a = Self::nv_int_as_i64(a_bits);
                            let b = Self::nv_int_as_i64(b_bits);
                            task.ram.push_i64(a.wrapping_add(b));
                        } else {
                            let a = auto_val::decode_i32(a_bits);
                            let b = auto_val::decode_i32(b_bits);
                            task.ram.push_i32(a.wrapping_add(b));
                        }
                    }
                }
            }
            OpCode::SUB => {
                // Plan 539 W1 (T10): py-handle operands route to the
                // Python dunder (arms stay one-liners to bound the
                // recursive frame size — see the aavm2 overflow note).
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_arith("__sub__", "__rsub__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let ((a_bits, a_is_f64), (b_bits, b_is_f64)) =
                        task.ram.pop_arith_pair_non_null("-")?;
                    if a_is_f64 && b_is_f64 {
                        task.ram
                            .push_f64(f64::from_bits(a_bits) - f64::from_bits(b_bits));
                    } else if a_is_f64 || b_is_f64 {
                        let a = if a_is_f64 {
                            f64::from_bits(a_bits)
                        } else {
                            nanbox_single_to_f64(a_bits)
                        };
                        let b = if b_is_f64 {
                            f64::from_bits(b_bits)
                        } else {
                            nanbox_single_to_f64(b_bits)
                        };
                        task.ram.push_f64(a - b);
                    } else if auto_val::is_f32(a_bits) && auto_val::is_f32(b_bits) {
                        task.ram
                            .push_f32(auto_val::decode_f32(a_bits) - auto_val::decode_f32(b_bits));
                    } else if auto_val::is_f32(a_bits) || auto_val::is_f32(b_bits) {
                        // Plan 576: f32×int 混算按 tag 解码（int 按值转 f32）。
                        let a = nanbox_single_to_f32(a_bits);
                        let b = nanbox_single_to_f32(b_bits);
                        task.ram.push_f32(a - b);
                    } else {
                        // PLAN-026 T-03 配套: i64 参与时按 i64 值算并
                        // push_i64(纯 i32 对保持 i32 通路,既有程序零变化);
                        // 此前 i64 位模式被 decode_i32 = 垃圾(i64_probe2
                        // "mix sum=63" 实证)。
                        if auto_val::is_i64(a_bits) || auto_val::is_i64(b_bits) {
                            let a = Self::nv_int_as_i64(a_bits);
                            let b = Self::nv_int_as_i64(b_bits);
                            task.ram.push_i64(a.wrapping_sub(b));
                        } else {
                            let a = auto_val::decode_i32(a_bits);
                            let b = auto_val::decode_i32(b_bits);
                            task.ram.push_i32(a.wrapping_sub(b));
                        }
                    }
                }
            }
            OpCode::MUL => {
                // Plan 539 W1 (T10): py-handle operands route to the
                // Python dunder (arms stay one-liners to bound the
                // recursive frame size — see the aavm2 overflow note).
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_arith("__mul__", "__rmul__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let ((a_bits, a_is_f64), (b_bits, b_is_f64)) =
                        task.ram.pop_arith_pair_non_null("*")?;
                    if a_is_f64 && b_is_f64 {
                        task.ram
                            .push_f64(f64::from_bits(a_bits) * f64::from_bits(b_bits));
                    } else if a_is_f64 || b_is_f64 {
                        let a = if a_is_f64 {
                            f64::from_bits(a_bits)
                        } else {
                            nanbox_single_to_f64(a_bits)
                        };
                        let b = if b_is_f64 {
                            f64::from_bits(b_bits)
                        } else {
                            nanbox_single_to_f64(b_bits)
                        };
                        task.ram.push_f64(a * b);
                    } else if auto_val::is_f32(a_bits) && auto_val::is_f32(b_bits) {
                        task.ram
                            .push_f32(auto_val::decode_f32(a_bits) * auto_val::decode_f32(b_bits));
                    } else if auto_val::is_f32(a_bits) || auto_val::is_f32(b_bits) {
                        // Plan 576: f32×int 混算按 tag 解码（int 按值转 f32）。
                        let a = nanbox_single_to_f32(a_bits);
                        let b = nanbox_single_to_f32(b_bits);
                        task.ram.push_f32(a * b);
                    } else {
                        // PLAN-026 T-03 配套: i64 参与时按 i64 值算并
                        // push_i64(纯 i32 对保持 i32 通路,既有程序零变化);
                        // 此前 i64 位模式被 decode_i32 = 垃圾(i64_probe2
                        // "mix sum=63" 实证)。
                        if auto_val::is_i64(a_bits) || auto_val::is_i64(b_bits) {
                            let a = Self::nv_int_as_i64(a_bits);
                            let b = Self::nv_int_as_i64(b_bits);
                            task.ram.push_i64(a.wrapping_mul(b));
                        } else {
                            let a = auto_val::decode_i32(a_bits);
                            let b = auto_val::decode_i32(b_bits);
                            task.ram.push_i32(a.wrapping_mul(b));
                        }
                    }
                }
            }
            OpCode::DIV => {
                // Plan 539 W1 (T10): py-handle operands route to the
                // Python dunder (arms stay one-liners to bound the
                // recursive frame size — see the aavm2 overflow note).
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_arith("__truediv__", "__rtruediv__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let ((a_bits, a_is_f64), (b_bits, b_is_f64)) =
                        task.ram.pop_arith_pair_non_null("/")?;
                    if a_is_f64 && b_is_f64 {
                        let b = f64::from_bits(b_bits);
                        if b == 0.0 {
                            return Err(VMError::DivisionByZero);
                        }
                        task.ram.push_f64(f64::from_bits(a_bits) / b);
                    } else if a_is_f64 || b_is_f64 {
                        let a = if a_is_f64 {
                            f64::from_bits(a_bits)
                        } else {
                            nanbox_single_to_f64(a_bits)
                        };
                        let b = if b_is_f64 {
                            f64::from_bits(b_bits)
                        } else {
                            nanbox_single_to_f64(b_bits)
                        };
                        if b == 0.0 {
                            return Err(VMError::DivisionByZero);
                        }
                        task.ram.push_f64(a / b);
                    } else if auto_val::is_f32(a_bits) && auto_val::is_f32(b_bits) {
                        let b = auto_val::decode_f32(b_bits);
                        if b == 0.0 {
                            return Err(VMError::DivisionByZero);
                        }
                        task.ram.push_f32(auto_val::decode_f32(a_bits) / b);
                    } else if auto_val::is_f32(a_bits) || auto_val::is_f32(b_bits) {
                        // Plan 576: f32×int 混算按 tag 解码（int 按值转 f32）。
                        let a = nanbox_single_to_f32(a_bits);
                        let b = nanbox_single_to_f32(b_bits);
                        if b == 0.0 {
                            return Err(VMError::DivisionByZero);
                        }
                        task.ram.push_f32(a / b);
                    } else {
                        let a = auto_val::decode_i32(a_bits);
                        let b = auto_val::decode_i32(b_bits);
                        if b == 0 {
                            return Err(VMError::DivisionByZero);
                        }
                        task.ram.push_i32(a.wrapping_div(b));
                    }
                }
            }

            // === Control Flow ===
            OpCode::NEG => {
                // Plan 539 W1 (T10): py-handle unary minus dunder.
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_neg(task, self)?;
                    return Ok(StepResult::Continue);
                }
                {
                    let (a_bits, a_is_f64) = task.ram.pop_arith_operand_non_null("-")?;
                    if a_is_f64 {
                        task.ram.push_f64(-f64::from_bits(a_bits));
                    } else if auto_val::is_f32(a_bits) {
                        task.ram.push_f32(-auto_val::decode_f32(a_bits));
                    } else {
                        task.ram
                            .push_i32(auto_val::decode_i32(a_bits).wrapping_neg());
                    }
                }
            }

            // Plan 073 Stage A: Floating-point arithmetic (f32)
            OpCode::ADD_F => {
                null_guard_peek_pair(task, "+")?;
                let b = pop_f32_operand(task);
                let a = pop_f32_operand(task);
                task.ram.push_f32(a + b);
                task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
            }
            OpCode::SUB_F => {
                null_guard_peek_pair(task, "-")?;
                let b = pop_f32_operand(task);
                let a = pop_f32_operand(task);
                task.ram.push_f32(a - b);
                task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
            }
            OpCode::MUL_F => {
                null_guard_peek_pair(task, "*")?;
                let b = pop_f32_operand(task);
                let a = pop_f32_operand(task);
                task.ram.push_f32(a * b);
                task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
            }
            OpCode::DIV_F => {
                null_guard_peek_pair(task, "/")?;
                let b = pop_f32_operand(task);
                let a = pop_f32_operand(task);
                if b == 0.0 {
                    return Err(VMError::DivisionByZero);
                }
                task.ram.push_f32(a / b);
                task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
            }
            OpCode::NEG_F => {
                null_guard_peek_unary(task, "-")?;
                let a = pop_f32_operand(task);
                task.ram.push_f32(-a);
                task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
            }

            // Plan 073 Stage A: Double precision arithmetic (f64)
            OpCode::ADD_D => {
                null_guard_peek_pair(task, "+")?;
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_f64(a + b);
                task.last_result_type = ResultType::Float; // Plan 403-F: mark f64 result
            }
            OpCode::SUB_D => {
                null_guard_peek_pair(task, "-")?;
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_f64(a - b);
                task.last_result_type = ResultType::Float;
            }
            OpCode::MUL_D => {
                null_guard_peek_pair(task, "*")?;
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                task.ram.push_f64(a * b);
                task.last_result_type = ResultType::Float;
            }
            OpCode::DIV_D => {
                null_guard_peek_pair(task, "/")?;
                let b = pop_f64_operand(task);
                let a = pop_f64_operand(task);
                if b == 0.0 {
                    return Err(VMError::DivisionByZero);
                }
                task.ram.push_f64(a / b);
                task.last_result_type = ResultType::Float;
            }
            OpCode::MOD => {
                // Plan 539 W1 (T10): py-handle operands route to the
                // Python dunder (arms stay one-liners to bound the
                // recursive frame size — see the aavm2 overflow note).
                #[cfg(feature = "python")]
                if crate::py_ffi::stack_has_py_handle(task, self) {
                    crate::py_ffi::py_dunder_arith("__mod__", "__rmod__", task, self)?;
                    return Ok(StepResult::Continue);
                }
                null_guard_peek_pair(task, "%")?;
                // PLAN-026 T-03 配套: 宽整取模(i32/i64 tag 归一;含
                // i64 时 push_i64,纯 i32 对保持 i32 通路)。
                let b_nv = task.ram.pop_nv();
                let a_nv = task.ram.pop_nv();
                let a = Self::nv_int_as_i64(a_nv);
                let b = Self::nv_int_as_i64(b_nv);
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                if auto_val::is_i64(a_nv) || auto_val::is_i64(b_nv) {
                    task.ram.push_i64(a % b);
                } else {
                    task.ram.push_i32((a % b) as i32);
                }
            }
            OpCode::MOD_F => {
                null_guard_peek_pair(task, "%")?;
                let b = task.ram.pop_f32();
                let a = task.ram.pop_f32();
                task.ram.push_f32(a % b);
            }
            OpCode::MOD_D => {
                null_guard_peek_pair(task, "%")?;
                let b = task.ram.pop_f64();
                let a = task.ram.pop_f64();
                task.ram.push_f64(a % b);
                task.last_result_type = ResultType::Float;
            }
            OpCode::NEG_D => {
                null_guard_peek_unary(task, "-")?;
                let a = task.ram.pop_f64();
                task.ram.push_f64(-a);
                task.last_result_type = ResultType::Float;
            }

            // 64-bit integer arithmetic (Plan 377: u64/i64 now 1 slot; heap-aware for full range)
            OpCode::ADD_U64 => {
                null_guard_peek_pair(task, "+")?;
                let b = self.pop_u64_vm(task);
                let a = self.pop_u64_vm(task);
                self.push_u64_vm(task, a.wrapping_add(b));
            }
            OpCode::SUB_U64 => {
                null_guard_peek_pair(task, "-")?;
                let b = self.pop_u64_vm(task);
                let a = self.pop_u64_vm(task);
                self.push_u64_vm(task, a.wrapping_sub(b));
            }
            OpCode::MUL_U64 => {
                null_guard_peek_pair(task, "*")?;
                let b = self.pop_u64_vm(task);
                let a = self.pop_u64_vm(task);
                self.push_u64_vm(task, a.wrapping_mul(b));
            }
            OpCode::DIV_U64 => {
                null_guard_peek_pair(task, "/")?;
                let b = self.pop_u64_vm(task);
                let a = self.pop_u64_vm(task);
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.push_u64_vm(task, a / b);
            }
            OpCode::MOD_U64 => {
                null_guard_peek_pair(task, "%")?;
                let b = self.pop_u64_vm(task);
                let a = self.pop_u64_vm(task);
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.push_u64_vm(task, a % b);
            }

            // Plan 117: Type coercion for mixed arithmetic
            OpCode::I32_TO_F32 => {
                // Plan 502 M3: tag 驱动——值已是 float-tag(如泛型 List
                // 元素经 GET_ELEM 的运行时 float)时直接透传。此前
                // pop_i32 无视标签把 f32 位模式当整数做数值转换
                // (`var x float = list[i]` 的 168.0 写入读回 ~1.12e9,
                // diagram 布局坐标整体位漂移的根因)。
                let nv = task.ram.pop_nv();
                if auto_val::is_f32(nv) {
                    task.ram.push_nv(nv);
                } else {
                    let val = auto_val::decode_i32(nv);
                    task.ram.push_f32(val as f32);
                }
                task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
            }
            OpCode::I64_TO_F64 => {
                let val = self.pop_i64_vm(task);
                task.ram.push_f64(val as f64);
                task.last_result_type = ResultType::Float;
            }
            OpCode::U64_TO_F64 => {
                let val = self.pop_u64_vm(task);
                task.ram.push_f64(val as f64);
                task.last_result_type = ResultType::Float;
            }

            OpCode::NOT => {
                let nv = task.ram.pop_nv();
                task.ram.push_nv(auto_val::encode_bool(!nv_truthy(nv)));
            }
            OpCode::CALL => {
                vm_debug!("DEBUG CALL: Stack depth before = {}", task.ram.sp);
                // Print stack before CALL
                if task.ram.sp > 0 {
                    vm_debug!("DEBUG CALL: Stack[0] = {}", task.ram.read_i32(0));
                }

                let target = self.flash.read_u32(task.ip) as usize;
                vm_debug!("DEBUG CALL: Calling function at address 0x{:04x}", target);
                task.ip += 4;

                // Plan 317: Generator short-circuit. If the callee is a
                // generator function (contains YIELD_VAL), don't execute
                // its body inline — create an Iterator::Generator and push
                // its id, just like call_fn_by_name does. This makes
                // `fn handler() ~Iter<int> { counter() }` (indirect call)
                // work the same as inline yield, so SSE detection fires on
                // the returned iter_id. The generator body runs lazily on
                // the first next() (shim_iterator_next).
                if self.is_generator_fn(target) {
                    // Plan 417-D2: parameterized generators. The callee's
                    // real arg count is in its FN_PROLOG header (byte at
                    // target+1); n_args used to come from the CALLER's
                    // current_fn_n_args (best-effort, wrong for any call
                    // with args) and the CALL-pushed args stayed on the
                    // caller stack, never reaching the generator task —
                    // parameterized generators crashed at RET (bp-n_args
                    // underflow) and read garbage params. Pop the args
                    // into stack_snapshot (declaration order); the first
                    // next() seeds them below the generator frame.
                    let real_n_args = self.flash.read_u8(target + 1) as usize;
                    let gen_state =
                        self.new_generator_state(task, target as u32, real_n_args as u8);
                    let iter_id = self.register_iterator(Iterator::Generator(gen_state));
                    task.ram.push_i32(iter_id as i32);
                    return Ok(StepResult::Continue);
                }

                // Push Return Address (IP)
                task.ram.push_i32(task.ip as i32);
                // Push Old Stack Frame (BP)
                task.ram.push_i32(task.bp as i32);

                // New BP points to the saved BP location (SP - 1)
                task.bp = task.ram.sp - 1;
                // PLAN-667 (F-01): 帧身份登记——帧实例 uid 发号。
                task.push_frame_id(task.bp);

                // Plan 199 Phase 7: Resolve function name from address
                let fn_name = self.flash.addr_to_name.get(&(target as u32)).cloned();

                // Plan 199: Push structured call frame for debugging
                // Save current function metadata for restoration on RET
                let saved_n_args = task.current_fn_n_args;
                let saved_n_locals = task.current_fn_n_locals;
                task.call_stack.push(crate::vm::task::CallFrame {
                    return_ip: task.ip,
                    old_bp: task.bp,
                    fn_name,
                    line: task.current_line,
                    old_fn_n_args: saved_n_args,
                    old_fn_n_locals: saved_n_locals,
                });

                vm_debug!(
                    "DEBUG CALL: Stack depth after setup = {}, BP = {}",
                    task.ram.sp,
                    task.bp
                );
                vm_debug!(
                    "DEBUG CALL: Stack[0] = {}, [1] = {}, [2] = {}",
                    task.ram.read_i32(0),
                    task.ram.read_i32(1),
                    task.ram.read_i32(2)
                );

                // Jump
                task.ip = target;
            }
            OpCode::CALL_SPEC => {
                // Dynamic dispatch via spec vtable
                // Reads: method_name string index (u32), arg_count (u8)
                // Stack: [..., receiver, arg0, arg1, ..., argN-1]
                let method_name_idx = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;
                let arg_count = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                // Get method name from string pool
                let method_name = self
                    .strings
                    .read()
                    .unwrap()
                    .get(method_name_idx)
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .unwrap_or_default();
                // The receiver is at stack position sp - arg_count - 1
                // (args are on top, receiver is below them)
                let sp = task.ram.sp;
                let receiver_pos = if sp >= arg_count + 1 {
                    sp - arg_count - 1
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "CALL_SPEC '{}' stack underflow: sp={} arg_count={}",
                        method_name, sp, arg_count
                    )));
                };

                let (receiver_nv, receiver_pos) = {
                    let nv = task.ram.read_nv(receiver_pos);
                    // In nanbox mode, if receiver_pos landed on a null marker
                    // (2nd slot of a 2-slot string arg), the actual receiver
                    // is one position earlier.
                    if auto_val::is_null(nv) && receiver_pos > 0 {
                        let prev_nv = task.ram.read_nv(receiver_pos - 1);
                        if auto_val::is_string(prev_nv) || auto_val::is_i32(prev_nv) {
                            (prev_nv, receiver_pos - 1)
                        } else {
                            (nv, receiver_pos)
                        }
                    } else {
                        (nv, receiver_pos)
                    }
                };
                let receiver_nv = receiver_nv;

                // Plan 446 批二 E1/E2: HTTP response-handle 访问器兜底。
                // http.get / builder .send() 返回的是 HTTP_RESPONSES
                // (stdlib 线程本地表)里的裸句柄,不是堆对象;静态类型在多语境
                // 推断坍缩后,`.status()/.body()/.header(k)` 到达 CALL_SPEC 时
                // 既无静态 native 名也无堆 tag 可分派——此前落入未定义调用
                // 的静默兜底:哨兵值(os-config E1 现场)、栈槽被吞导致后续任何
                // http 调用栈下溢崩溃(E2 现场同根)。
                // 判定收窄到"小正整数句柄且不在堆上、但命中 response 表",
                // 服务端 Response 构建链(堆对象)与普通 int 方法不受影响。
                // PLAN-776 R1-F06：receiver 双形态——client int 句柄（查表面）
                // 或 **Response 凭据对象**（快照面——read_response_payload 内
                // owner 校验分流）。对象臂 Some(0) 仅作"走 shim"标志（shim 按
                // 栈上原 receiver nv 分流，不消费该值）。
                let receiver_is_credential = auto_val::is_object(receiver_nv)
                    && self
                        .heap_objects
                        .get(&(auto_val::decode_object(receiver_nv) as u64))
                        .map(|o| {
                            o.read()
                                .map(|g| {
                                    g.type_tag()
                                        == crate::vm::heap_object::TypeTag::HttpReturnResource
                                })
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                let http_response_handle = match method_name.as_str() {
                    "status" | "status_code" | "body" | "body_bytes" | "body_to_file"
                    | "header" | "header_get"
                        if arg_count <= 2 && receiver_is_credential =>
                    {
                        Some(0i32)
                    }
                    "status" | "status_code" | "body" | "body_bytes" | "body_to_file"
                    | "header" | "header_get"
                        if arg_count <= 2 && auto_val::is_i32(receiver_nv) =>
                    {
                        let h = auto_val::decode_i32(receiver_nv);
                        let is_heap = h > 0 && self.heap_objects.contains_key(&(h as u64));
                        if h > 0
                            && !is_heap
                            && crate::vm::ffi::stdlib::lookup_http_response(h as u64).is_some()
                        {
                            Some(h)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(handle) = http_response_handle {
                    let shim_id = match (method_name.as_str(), arg_count) {
                        ("status", 0) | ("status_code", 0) => {
                            crate::vm::ffi::stdlib::NATIVE_RESPONSE_STATUS_CODE
                        }
                        ("body", 0) => crate::vm::ffi::stdlib::NATIVE_RESPONSE_BODY,
                        // plan-022 (auto-down D4): byte-faithful accessor
                        ("body_bytes", 0) => crate::vm::ffi::stdlib::NATIVE_RESPONSE_BODY_BYTES,
                        ("body_to_file", 1) => crate::vm::ffi::stdlib::NATIVE_RESPONSE_BODY_TO_FILE,
                        ("header", 1) | ("header_get", 1) => {
                            crate::vm::ffi::stdlib::NATIVE_RESPONSE_HEADER_GET
                        }
                        _ => u16::MAX,
                    };
                    if shim_id != u16::MAX {
                        if let Some(shim) = self.native_interface.get(shim_id).cloned() {
                            return shim(task, self).map(|_| StepResult::Continue);
                        }
                    }
                }

                // Plan 446 批二 E2: RequestBuilder 链的 CALL_SPEC 拦截 +
                // Yield 协议同步完成。链式 .header/.body/.timeout/.send 以
                // "RequestBuilder.X" 到达本处;若落入 exports 声明Stub/其它
                // 兜底臂,send 置位的 waiting_http_request_id 无人消费——
                // 之后任何 CALL_NAT(如 .status() 查询)撞上 stale 标记即
                // 无限 rewind+Yield(现场"send 后同作用域 http 调用崩溃/
                // 挂死"的协议级根因,轨迹实证 826K 次自旋)。
                // 处理:精确判定"接收者堆对象 tag == RequestBuilder"后直调
                // shim 支撑的 native;send 的异步语义在本路径无调度器配合,
                // 以同步 drain 完成——轮询结果就绪后触发 shim 重入(清位+
                // 推柄),保持 waiting 标记零残留。
                if matches!(
                    method_name.as_str(),
                    "header" | "body" | "timeout" | "json" | "send"
                ) && arg_count <= 2
                {
                    // PLAN-705 T-03: 段模式重入臂——yield 后 rewind 重
                    // firing 的 .send()。receiver 已在首轮被 shim 消费
                    // （堆对象已移除、栈槽已弹出），下方 receiver 探测
                    // 必然落空；`waiting_http_request_id` 即重入凭据
                    // （与 CALL_NAT 的 shim 重入协议同构）。结果已就绪
                    // → shim 重入臂消费并 Continue；仍未就绪 → 再让步
                    // park（链式等待）。
                    if method_name == "send" && task.waiting_http_request_id.is_some() {
                        if let Some(shim) = self
                            .native_interface
                            .get(crate::vm::ffi::stdlib::NATIVE_HTTP_REQUEST_BUILDER_SEND)
                            .cloned()
                        {
                            shim(task, self)?;
                            if task.waiting_http_request_id.is_some() {
                                if std::env::var_os("AUTO_DEBUG_G9").is_some() {
                                    eprintln!(
                                        "[G9] re-fire not-ready: ip={:#x} sp={} bp={}",
                                        task.ip, task.ram.sp, task.bp
                                    );
                                }
                                task.ip -= 6; // CALL_SPEC = 1 op + 4 idx + 1 count
                                return Ok(StepResult::Yield);
                            }
                            if std::env::var_os("AUTO_DEBUG_G9").is_some() {
                                eprintln!(
                                    "[G9] re-fire CONSUMED result: ip={:#x} sp={} bp={}",
                                    task.ip, task.ram.sp, task.bp
                                );
                            }
                            return Ok(StepResult::Continue);
                        }
                    }
                    let rb_heap_id = if auto_val::is_object(receiver_nv) {
                        Some(auto_val::decode_object(receiver_nv) as u64)
                    } else if auto_val::is_i32(receiver_nv) {
                        let v = auto_val::decode_i32(receiver_nv);
                        if v > 0 {
                            Some(v as u64)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let is_builder = rb_heap_id
                        .and_then(|id| self.heap_objects.get(&id))
                        .map(|o| o.read().unwrap().type_tag().name() == "RequestBuilder")
                        .unwrap_or(false);
                    if is_builder {
                        const RB_NONE: u16 = u16::MAX;
                        let shim_id = match (method_name.as_str(), arg_count) {
                            ("header", 2) => {
                                crate::vm::ffi::stdlib::NATIVE_HTTP_REQUEST_BUILDER_HEADER
                            }
                            ("body", 1) => crate::vm::ffi::stdlib::NATIVE_HTTP_REQUEST_BUILDER_BODY,
                            ("timeout", 1) => {
                                crate::vm::ffi::stdlib::NATIVE_HTTP_REQUEST_BUILDER_TIMEOUT
                            }
                            ("json", 1) => crate::vm::ffi::stdlib::NATIVE_HTTP_REQUEST_BUILDER_JSON,
                            ("send", 0) => crate::vm::ffi::stdlib::NATIVE_HTTP_REQUEST_BUILDER_SEND,
                            _ => RB_NONE,
                        };
                        if shim_id != RB_NONE {
                            if let Some(shim) = self.native_interface.get(shim_id).cloned() {
                                shim(task, self)?;
                                if shim_id
                                    == crate::vm::ffi::stdlib::NATIVE_HTTP_REQUEST_BUILDER_SEND
                                {
                                    // Yield 协议同步完成:首轮 shim 已 spawn 并置位
                                    // waiting;轮询就绪后重入(清位+推柄)。
                                    if let Some(req_id) = task.waiting_http_request_id {
                                        if task.segment_no_busy_wait {
                                            // PLAN-705 T-03: 段模式退役同步
                                            // drain——rewind 至 CALL_SPEC 指令
                                            // 起点（1 opcode + 4 name_idx +
                                            // 1 arg_count = 6 字节）并 Yield，
                                            // 段驱动 park 成 HttpRequest；恢复
                                            // 时重 firing 的 CALL_SPEC 走 shim
                                            // 重入臂消费结果（等待零占线程）。
                                            task.ip -= 6;
                                            return Ok(StepResult::Yield);
                                        }
                                        let deadline = std::time::Instant::now()
                                            + std::time::Duration::from_secs(30);
                                        // PLAN-026 T-04: 同构忙等段预算计。
                                        let mut budget_waits: u64 = 0;
                                        let budget_start = std::time::Instant::now();
                                        while !crate::vm::ffi::stdlib::async_http_result_ready(
                                            req_id,
                                        ) {
                                            if std::time::Instant::now() > deadline {
                                                task.waiting_http_request_id = None;
                                                // PLAN-027 缺陷 A:同构超时回收
                                                //(与 call_fn_by_name drain 臂同修)。
                                                crate::vm::ffi::stdlib::drop_async_result(req_id);
                                                return Err(VMError::RuntimeError(
                                                        "http request-builder send timed out (plan-446 E2)".into(),
                                                    ));
                                            }
                                            std::thread::sleep(std::time::Duration::from_millis(5));
                                            budget_waits += 1;
                                        }
                                        Self::api_budget_accumulate(
                                            budget_waits,
                                            budget_start.elapsed(),
                                        );
                                        shim(task, self)?;
                                    }
                                }
                                return Ok(StepResult::Continue);
                            }
                        }
                    }
                }

                // 2026-08-28 勘误注记: 最初版拦截缺上面的同步 drain,曾致
                // 挂死;已由带协议完成的版本取代,勿回退。

                // PLAN-044: unwrap_or 的坍缩 Option 协议——非 null 接收者
                // 即 Some(v),unwrap_or(d) = 恒等(丢默认值参);null 接收者
                // 由 None 协议臂接管(mpsc/stream 失败前 intercept)。
                if method_name == "unwrap_or" && arg_count == 1 && !auto_val::is_null(receiver_nv) {
                    let _default_arg = task.ram.pop_nv();
                    let recv = task.ram.pop_nv();
                    task.ram.push_nv(recv);
                    return Ok(StepResult::Continue);
                }

                // Plan 442 C2: `.ok()` 恒等直通 —— rust 形态 Result 语义的
                // VM-native 约定:返回 Result 的 stdlib shim(env.var 等)在
                // 生产者边界坍缩为 Option 形状(None=TAG_NULL / Some(v)=v 本身),
                // 调用方的 Rust 式 `.ok()` 链步因此是纯透传(与 codegen 侧
                // .unwrap()/.expect() 的 opaque 透明直通同构)。任何接收者形
                // 态一致透传;未知方法在别处的既有行为不变。
                if arg_count == 0 && method_name == "ok" {
                    let nv = task.ram.pop_nv();
                    task.ram.push_nv(nv);
                    return Ok(StepResult::Continue);
                }

                // 2026-08-22(方法链修复):标量接收者的内建方法兜底。
                // 链式调用 `l.len().str()` 中前一方法返回普通 i32/f64/bool
                // 标量;此前 i32>0 被一律按堆对象 id 解析(heap 无此 id →
                // <unknown:N>.str → 派发失败静默推 None —— 列表方法链
                // 串成垃圾串/None 的根因)。与 TYPE_TO_STR 同款格式化。
                if arg_count == 0 && matches!(method_name.as_str(), "str" | "to_string") {
                    let scalar_str: Option<String> = if auto_val::is_bool(receiver_nv) {
                        Some(if auto_val::decode_bool(receiver_nv) {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        })
                    } else if auto_val::is_f64(receiver_nv) {
                        Some(format!("{}", auto_val::decode_f64(receiver_nv)))
                    } else if auto_val::is_f32(receiver_nv) {
                        Some(format!("{}", auto_val::decode_f32(receiver_nv)))
                    } else if auto_val::is_i64(receiver_nv) {
                        // PLAN-026 T-03 配套: i64 标量串化(tag 8 此前
                        // 缺分支,接收者落 heap 查找路径 = 垃圾串;
                        // i64_probe "tab 2"/"sum=63" 实证)。
                        Some(format!("{}", auto_val::decode_i64(receiver_nv)))
                    } else if auto_val::is_i32(receiver_nv) {
                        let v = auto_val::decode_i32(receiver_nv);
                        if v >= 4_000_000 {
                            None
                        } else {
                            Some(format!("{}", v))
                        }
                    } else {
                        None
                    };
                    if let Some(s) = scalar_str {
                        // 栈:[receiver] → [str 结果](argc=0,receiver 即栈顶)
                        task.ram.pop_nv();
                        let idx = self.add_string(s.into_bytes());
                        self.rc_push_str_idx(task, idx as usize);
                        return Ok(StepResult::Continue);
                    }
                }

                // Look up the object's type name from all registries
                let type_name = if auto_val::is_string(receiver_nv) {
                    let str_idx = auto_val::decode_string(receiver_nv) as usize;
                    let s = self
                        .strings
                        .read()
                        .unwrap()
                        .get(str_idx)
                        .map(|b| String::from_utf8_lossy(b).to_string())
                        .unwrap_or_default();
                    // Check if this string is a type name by looking up "TypeName.method" in exports.
                    // This handles user-defined types (Settings.load, Agent.new, etc.) that
                    // the static call types list below can't cover.
                    let candidate = format!("{}.{}", s, method_name);
                    if self.flash.exports_by_name.contains_key(&candidate) {
                        s
                    } else {
                        // Fallback: check known crate type names pushed by codegen for static calls.
                        const STATIC_CALL_TYPES: &[&str] = &[
                            "Command",
                            "Stdio",
                            "Writer",
                            "Reader",
                            "ReaderBuilder",
                            "WriterBuilder",
                            "StringRecord",
                            "ThreadRng",
                            "Complex",
                            "BigInt",
                            "Normal",
                            "Rng",
                            "WalkDir",
                            "Instant",
                            "Duration",
                            "OnceCell",
                            "Regex",
                            "Url",
                            "Version",
                            "RefCell",
                            "Child",
                            "File",
                            "FileWriter",
                            "PathBuf",
                            "String",
                            "Vec",
                        ];
                        if STATIC_CALL_TYPES.contains(&s.as_str()) {
                            s
                        } else {
                            "str".to_string()
                        }
                    }
                } else if auto_val::is_i32(receiver_nv) {
                    let receiver = auto_val::decode_i32(receiver_nv);
                    if receiver > 0 {
                        let obj_key = receiver as u64;
                        if let Some(obj_lock) = self.heap_objects.get(&obj_key) {
                            let guard = obj_lock.read().unwrap();
                            if let Some(inst) = guard
                                .as_any()
                                .downcast_ref::<crate::vm::generic_registry::GenericInstanceData>(
                            ) {
                                inst.mono_name
                                    .split('_')
                                    .next()
                                    .unwrap_or(&inst.mono_name)
                                    .to_string()
                            } else if guard
                                .as_any()
                                .downcast_ref::<crate::vm::types::ObjectData>()
                                .is_some()
                            {
                                // Plan 390 §15 H3b: obj literals are ObjectData
                                // in heap_objects — keep the "HashMap" type name
                                // so .set/.get/.len dispatch to auto.hashmap.
                                "HashMap".to_string()
                            } else {
                                let tag_name = guard.type_tag().name();
                                tag_name.split('<').next().unwrap_or(&tag_name).to_string()
                            }
                        } else {
                            format!("<unknown:{}>", obj_key)
                        }
                    } else if auto_val::is_null(receiver_nv) {
                        "Option".to_string()
                    } else if receiver == -1 {
                        "Option".to_string()
                    } else {
                        format!("<invalid_i32:{}>", receiver)
                    }
                } else if auto_val::is_null(receiver_nv) {
                    "None".to_string()
                } else if auto_val::is_object(receiver_nv) {
                    let obj_key = auto_val::decode_object(receiver_nv) as u64;
                    if let Some(obj_lock) = self.heap_objects.get(&obj_key) {
                        let guard = obj_lock.read().unwrap();
                        if let Some(inst) = guard
                            .as_any()
                            .downcast_ref::<crate::vm::generic_registry::GenericInstanceData>(
                        ) {
                            inst.mono_name
                                .split('_')
                                .next()
                                .unwrap_or(&inst.mono_name)
                                .to_string()
                        } else if guard
                            .as_any()
                            .downcast_ref::<crate::vm::types::ObjectData>()
                            .is_some()
                        {
                            // Plan 390 §15 H3b: ObjectData → "HashMap" (see above).
                            "HashMap".to_string()
                        } else {
                            let tag_name = guard.type_tag().name();
                            tag_name.split('<').next().unwrap_or(&tag_name).to_string()
                        }
                    } else {
                        format!("<unknown_obj:{}>", obj_key)
                    }
                } else {
                    format!("<unknown_nv:{:016x}>", receiver_nv)
                };

                // Construct function name: TypeName.method
                let func_name = format!("{}.{}", type_name, method_name);
                if method_name == "push" || method_name == "len" {}

                // Plan 212 Phase 2.2: .unwrap() and similar on opaque handles is identity
                // (constructors already handle errors by raising VMError)
                // first_or_octet_stream: mime shim already resolves the value
                let is_identity_unwrap = matches!(
                    method_name.as_str(),
                    "unwrap" | "expect" | "first_or_octet_stream" | "first"
                );

                // Plan 249 Phase 4: Unified opaque dispatch via native_catalog
                let opaque_native_name = crate::vm::native_catalog::lookup_opaque_dispatch_by_type(
                    type_name.as_str(),
                    method_name.as_str(),
                );

                // Resolve opaque native shim
                let opaque_native_id = opaque_native_name
                    .as_ref()
                    .and_then(|name| self.native_interface.resolve(name));

                // PLAN-053 P-053-6: web 生态 Regex 静态形态——
                // `Regex.replace(text, pat, repl, flags)` / `Regex.test(text, pat)`
                // （musk forge_helpers/mention_helpers 的消息渲染链）。原仅
                // OPAQUE_DISPATCH_REGEX_METHODS 的实例方法（is_match/
                // replace_all…）可解析，静态形态落兜底报错 → 消息正文整体空。
                let regex_static_native_id =
                    if type_name == "Regex" && matches!(method_name.as_str(), "replace" | "test") {
                        self.native_interface
                            .resolve(&format!("auto.regex.{}", method_name))
                    } else {
                        None
                    };

                // Plan 240: Math method dispatch for CALL_SPEC
                // Handles chained expressions like (a-b).to_radians() where type inference fails
                const CALL_SPEC_MATH_METHODS: &[&str] = &[
                    "sin",
                    "cos",
                    "tan",
                    "sqrt",
                    "abs",
                    "floor",
                    "ceil",
                    "round",
                    "pow",
                    "powf",
                    "powi",
                    "exp",
                    "ln",
                    "log2",
                    "log10",
                    "signum",
                    "asin",
                    "acos",
                    "atan",
                    "atan2",
                    "to_radians",
                    "to_degrees",
                ];
                let math_native_id = if CALL_SPEC_MATH_METHODS.contains(&method_name.as_str()) {
                    let math_name = format!("auto.math.{}", method_name);
                    self.native_interface.resolve(&math_name)
                } else {
                    None
                };

                // Look up function address in exports first
                if is_identity_unwrap {
                    // Do nothing — receiver stays on stack, no args to pop
                } else if let Some(&addr) = self.flash.exports_by_name.get(&func_name) {
                    // Standard CALL sequence: push return address, old BP, set new BP, jump
                    task.ram.push_i32(task.ip as i32);
                    task.ram.push_i32(task.bp as i32);
                    task.bp = task.ram.sp - 1;
                    // PLAN-667 (F-01): 帧身份登记——帧实例 uid 发号。
                    task.push_frame_id(task.bp);
                    task.ip = addr as usize;
                    // Plan 417-E3: push a CallFrame exactly like regular
                    // CALL — the callee's RET pops one to restore
                    // current_fn_n_args/current_fn_n_locals. Without this
                    // frame the CALLER's frame is popped instead, and
                    // LOAD_LOCAL param addressing (which reads
                    // current_fn_n_args) degrades to the null guard —
                    // e.g. the second receiver load in
                    // `a.cmp(0) >= b.cmp(0)` pushed null.
                    let fn_name = self.flash.addr_to_name.get(&(addr as u32)).cloned();
                    let saved_n_args = task.current_fn_n_args;
                    let saved_n_locals = task.current_fn_n_locals;
                    task.call_stack.push(crate::vm::task::CallFrame {
                        return_ip: task.ip,
                        old_bp: task.bp,
                        fn_name,
                        line: task.current_line,
                        old_fn_n_args: saved_n_args,
                        old_fn_n_locals: saved_n_locals,
                    });
                } else if let Some(native_id) = regex_static_native_id {
                    // PLAN-053 P-053-6: web 生态 Regex 静态形态
                    // （Regex.replace/test）。CALL_SPEC 布局
                    // [..., recv, arg0..argN-1] → shim 按 CALL_NAT 约定
                    // 只吃实参：实参弹出暂存 → 弃接收者 → 实参回栈 →
                    // 执行（结果由 shim 压回，无额外清理）。
                    let mut args_rev = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args_rev.push(task.ram.pop_nv());
                    }
                    task.ram.pop_nv(); // receiver（"Regex" 类型名占位）
                    for nv in args_rev.into_iter().rev() {
                        task.ram.push_nv(nv);
                    }
                    if let Some(shim) = self.native_interface.get(native_id).cloned() {
                        shim(task, self)?;
                    } else {
                        return Err(VMError::MissingNative(native_id));
                    }
                } else if let Some(native_id) = opaque_native_id {
                    // Plan 212 Phase 2.2: Opaque type method routed to native shim
                    // CALL_SPEC stack: [..., receiver, arg0, arg1, ..., argN-1]
                    // Native shim expects: [..., argN-1, ..., arg0, receiver] (pop args first, then receiver)
                    // Re-push receiver on top so shim can pop in correct order
                    {
                        let recv_nv = task.ram.read_nv(receiver_pos);
                        task.ram.push_nv(recv_nv);
                    }
                    if let Some(shim) = self.native_interface.get(native_id).cloned() {
                        shim(task, self)?;
                    } else {
                        return Err(VMError::MissingNative(native_id));
                    }
                    // After shim returns, remove duplicate args+receiver from CALL_SPEC layout.
                    // Stack: [..., receiver, arg0..argN-1, receiver, return_value(s)]
                    // Need: [..., return_value(s)]
                    // In nanbox mode, string values occupy 2 slots (encode_string + encode_null).
                    {
                        use auto_val::NanoValue;
                        let top_nv: NanoValue = task.ram.pop_nv();
                        if auto_val::is_string(top_nv) {
                            for _ in 0..=arg_count {
                                task.ram.pop_nv();
                            }
                            task.ram.push_nv(top_nv);
                            task.ram.push_nv(auto_val::encode_null());
                        } else if auto_val::is_null(top_nv)
                            && task.ram.sp > 0
                            && auto_val::is_string(task.ram.read_nv(task.ram.sp - 1))
                        {
                            let str_nv = task.ram.pop_nv();
                            for _ in 0..=arg_count {
                                task.ram.pop_nv();
                            }
                            task.ram.push_nv(str_nv);
                            task.ram.push_nv(top_nv);
                        } else {
                            for _ in 0..=arg_count {
                                task.ram.pop_nv();
                            }
                            task.ram.push_nv(top_nv);
                        }
                    }
                } else if let Some(native_id) = math_native_id {
                    // Plan 240: Math method on expression result (e.g., (a-b).to_radians())
                    // CALL_SPEC stack: [..., receiver, arg0, ..., argN-1]
                    // Pop args in reverse, then pop receiver as f64, apply math, push result
                    // For unary math methods (0 args): receiver is the f64 value
                    // Plan 474: 接收者/结果必须按 NanoValue 透传。原实现
                    // read_i32/push_i32/pop_i32 是 nanbox 前 i32 栈约定的化石：
                    // 裸 f64（encode_f64=原始位）被读成低 32 位、TAG_F32 被
                    // 读成 payload 位型——`__json_object` 浮点字段 `.floor()`
                    // 返回位型垃圾（plan011④：54.16 → -515396076）。shim 的
                    // VMConvertible f64 pop 自带 TAG_F32→f64 提升与裸 f64
                    // 直读（convert.rs），此处无需任何转换。
                    if arg_count == 0 {
                        // Unary math: receiver is the float value on stack.
                        // 1. Copy the receiver nv (tag-intact) to the top
                        // 2. Shim pops it and pushes the result
                        // 3. Replace the original receiver slot with the result
                        let recv_nv = task.ram.read_nv(receiver_pos);
                        task.ram.push_nv(recv_nv);
                        if let Some(shim) = self.native_interface.get(native_id).cloned() {
                            shim(task, self)?;
                        } else {
                            return Err(VMError::MissingNative(native_id));
                        }
                        let return_nv = task.ram.pop_nv();
                        // Remove old receiver (1 slot)
                        task.ram.pop_nv();
                        task.ram.push_nv(return_nv);
                    } else {
                        // Binary+ math (e.g., powf/atan2): CALL_SPEC layout
                        // [recv, arg0..argN-1] 与 rust_fn shim 的逆序弹出
                        // 约定（宏按声明序逆序 pop：末参在栈顶）天然对齐
                        // ——shim 首参(receiver 值)在窗口底、末参(argN-1)
                        // 在栈顶，即 recv.powf(arg) 语义。原地调用即可，
                        // shim 消费整个窗口并留结果于栈顶。原实现把
                        // receiver 拷贝压顶，在逆序弹出下变成「末参收到
                        // receiver 值」的参数序倒置（54.16.powf(2.0) 算成
                        // 2.0^54.16，Plan 474 实测）。
                        if let Some(shim) = self.native_interface.get(native_id).cloned() {
                            shim(task, self)?;
                        } else {
                            return Err(VMError::MissingNative(native_id));
                        }
                    }
                } else if let Some(native_id) = self.native_interface.resolve(&func_name) {
                    // Plan 200 Task 3.3: Fallback to native registry for type.method natives
                    // (e.g., Result.Ok.map_err -> shim_result_map_err)
                    if let Some(shim) = self.native_interface.get(native_id).cloned() {
                        // PLAN-608 T03（KD-VM6①）：resolve 路径补 CALL_NAT 同款
                        // 死区结算（与 CALL_NAT 臂同构）。shim 按 CALL_NAT 约定
                        // 消费实参/接收者并回推结果，但 CALL_NAT 的
                        // rc_release_slot_range 死区不在本路径——字符串元素/实参
                        // 的暂存池份额（池不入影子，按内容结算）此前每调用孤儿
                        // +1（List<str>.push 把 dedup 共享条目永久钉死）。窗口
                        // [sp_after, sp_before)：结果落窗口底槽，被弹槽按影子
                        // （堆）/内容（池）释放，与 CALL_NAT 完全一致。
                        let sp_before_shim = task.ram.sp;
                        shim(task, self)?;
                        let sp_after_shim = task.ram.sp;
                        if sp_after_shim < sp_before_shim {
                            self.rc_release_slot_range(
                                &mut task.ram,
                                sp_after_shim,
                                sp_before_shim,
                            );
                        }
                    } else {
                        return Err(VMError::MissingNative(native_id));
                    }
                } else if type_name == "str" {
                    // Inline str type method dispatch for CALL_SPEC
                    // Stack: [..., receiver(str nanbox), arg0, ..., argN-1]
                    // PLAN-608 T02: CALL_SPEC 布局顶界。消费型臂弹毕、压结果前
                    // 以 rc_release_slot_range 结算 [sp, spec_sp_entry) 弹出窗口
                    // （堆按影子、池按内容——CALL_SPEC 路径无 CALL_NAT 死区，
                    // 此前接收者/字符串实参的暂存份额每调用孤儿 +1，KD-VM6②）。
                    let spec_sp_entry = task.ram.sp;
                    match method_name.as_str() {
                        "as_bytes" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let bytes: Vec<u8> = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| b.clone())
                                .unwrap_or_default();
                            // Wrap as RustStdlibObject for FFI consumption
                            let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
                                "Vec<u8>", bytes,
                            );
                            let handle = self.insert_heap_object(obj) as i32;
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_i32(handle));
                            }
                        }
                        // Plan 378 §10.5: 补全所有大小写别名。原表只有
                        // to_uppercase/to_lower/to_lowercase，漏了裸 upper/lower 和
                        // to_upper/to_lowercase，导致 split()/lines() 等数组元素
                        // (Expr::Index receiver，走 CALL_SPEC inline 分发) 上调用
                        // .lower()/.upper() 落入 _ => push null，返回 None。
                        "upper" | "to_upper" | "to_uppercase" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            let result = s.to_uppercase();
                            let idx = self.add_string(result.into_bytes());
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push_str_idx(task, idx as usize);
                            }
                        }
                        "lower" | "to_lower" | "to_lowercase" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            let result = s.to_lowercase();
                            let idx = self.add_string(result.into_bytes());
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push_str_idx(task, idx as usize);
                            }
                        }
                        "chars" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            use crate::vm::types::ListData;
                            let mut list: ListData<i32> = ListData::new();
                            for ch in s.chars() {
                                list.push(ch as i32);
                            }
                            let list_id = self.insert_heap_object(list);
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push(task, auto_val::encode_object(list_id as u32));
                            }
                        }
                        "graphemes" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            use crate::vm::types::ListData;
                            let mut list: ListData<i32> = ListData::new();
                            for g in s.split(|c: char| c.is_whitespace() || !c.is_alphanumeric()) {
                                if !g.is_empty() {
                                    list.push(g.chars().next().unwrap() as i32);
                                }
                            }
                            // Fallback: split by char boundaries
                            if list.len() == 0 {
                                for ch in s.chars() {
                                    list.push(ch as i32);
                                }
                            }
                            let list_id = self.insert_heap_object(list);
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push(task, auto_val::encode_object(list_id as u32));
                            }
                        }
                        "split" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            // Pop the separator argument
                            let sep_nv = task.ram.pop_nv();
                            let sep = if auto_val::is_string(sep_nv) {
                                let sep_idx = auto_val::decode_string(sep_nv) as usize;
                                self.strings
                                    .read()
                                    .unwrap()
                                    .get(sep_idx)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default()
                            } else {
                                ",".to_string()
                            };
                            let parts: Vec<&str> = s.split(&sep).collect();
                            let mut strings = self.strings.write().unwrap();
                            use crate::vm::types::ListData;
                            let mut list: ListData<i32> = ListData::new();
                            for part in &parts {
                                let idx = strings.len();
                                strings.push(part.to_string().into_bytes());
                                list.push(idx as i32);
                            }
                            drop(strings);
                            let list_id = self.insert_heap_object(list);
                            // Remove receiver from CALL_SPEC layout, push result
                            {
                                task.ram.pop_nv();
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push(task, auto_val::encode_object(list_id as u32));
                            }
                        }
                        "is_empty" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| b.is_empty())
                                .unwrap_or(true);
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(if s {
                                    auto_val::encode_i32(1)
                                } else {
                                    auto_val::encode_i32(0)
                                });
                            }
                        }
                        "starts_with" | "ends_with" | "contains" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            let arg_nv = task.ram.pop_nv();
                            let pat = if auto_val::is_string(arg_nv) {
                                let pat_idx = auto_val::decode_string(arg_nv) as usize;
                                self.strings
                                    .read()
                                    .unwrap()
                                    .get(pat_idx)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            let result = if method_name == "starts_with" {
                                s.starts_with(&pat)
                            } else if method_name == "ends_with" {
                                s.ends_with(&pat)
                            } else {
                                s.contains(&pat)
                            };
                            {
                                task.ram.pop_nv();
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(if result {
                                    auto_val::encode_i32(1)
                                } else {
                                    auto_val::encode_i32(0)
                                });
                            }
                        }
                        "len" => {
                            // PLAN-055: JS .length 语义 = 字符数（web a2ts
                            // `text.length` 同值）。此前按字节计——CJK 3×
                            // 膨胀，musk estimateTokens 的 nonCjk 基数失真。
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let len = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).chars().count() as i32)
                                .unwrap_or(0);
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_i32(len));
                            }
                        }
                        "trim" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).trim().to_string())
                                .unwrap_or_default();
                            let idx = self.add_string(s.into_bytes());
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push_str_idx(task, idx as usize);
                            }
                        }
                        // PLAN-053 P-053-6: web 生态字符串方法族（musk 消息
                        // 渲染链使用面）——此前落 _ => push null，正文链
                        // 静默退化（stripQuestionnaire 的 trimEnd、
                        // chatSearchFilter 的 includes/to_lower、
                        // estimateTokens 的 char_code_at 等）。
                        "trimEnd" | "trim_end" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).trim_end().to_string())
                                .unwrap_or_default();
                            let idx = self.add_string(s.into_bytes());
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push_str_idx(task, idx as usize);
                            }
                        }
                        "includes" => {
                            let pat_nv = if arg_count >= 1 {
                                task.ram.pop_nv()
                            } else {
                                receiver_nv
                            };
                            let pat = if auto_val::is_string(pat_nv) {
                                let i = auto_val::decode_string(pat_nv) as usize;
                                self.strings
                                    .read()
                                    .unwrap()
                                    .get(i)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            {
                                task.ram.pop_nv();
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_bool(s.contains(&pat)));
                            }
                        }
                        "indexOf" | "indexOf_str" | "lastIndexOf" | "last_index_of" => {
                            let pat_nv = if arg_count >= 1 {
                                task.ram.pop_nv()
                            } else {
                                receiver_nv
                            };
                            let pat = if auto_val::is_string(pat_nv) {
                                let i = auto_val::decode_string(pat_nv) as usize;
                                self.strings
                                    .read()
                                    .unwrap()
                                    .get(i)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            let found = if method_name == "indexOf" || method_name == "indexOf_str"
                            {
                                s.find(&pat).map(|b| b as i32).unwrap_or(-1)
                            } else {
                                s.rfind(&pat).map(|b| b as i32).unwrap_or(-1)
                            };
                            {
                                task.ram.pop_nv();
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_i32(found));
                            }
                        }
                        "substring" | "substring_str" => {
                            // JS 语义近似：(start[, end])，越界钳制，按字节
                            // 切片（musk 现场的 start/end 来自 find() 字节位）。
                            let (a, b) = if arg_count >= 2 {
                                let e = task.ram.pop_nv();
                                let s0 = task.ram.pop_nv();
                                (auto_val::decode_i32(s0), auto_val::decode_i32(e))
                            } else if arg_count == 1 {
                                (auto_val::decode_i32(task.ram.pop_nv()), i32::MAX)
                            } else {
                                (0, i32::MAX)
                            };
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            let bytes = s.as_bytes();
                            let start = (a.max(0) as usize).min(bytes.len());
                            let end = (b.max(0) as usize).min(bytes.len()).max(start);
                            // 字符边界钳制（非 UTF-8 边界时退到字符边界）。
                            let mut se = start;
                            while se < end && !s.is_char_boundary(se) {
                                se += 1;
                            }
                            let mut ee = end;
                            while ee > se && !s.is_char_boundary(ee) {
                                ee -= 1;
                            }
                            let out = String::from_utf8_lossy(&bytes[se..ee]).to_string();
                            let idx = self.add_string(out.into_bytes());
                            {
                                task.ram.pop_nv();
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push_str_idx(task, idx as usize);
                            }
                        }
                        "char_code_at" | "charCodeAt" => {
                            let i = if arg_count >= 1 {
                                auto_val::decode_i32(task.ram.pop_nv())
                            } else {
                                0
                            };
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            let code = s
                                .chars()
                                .nth(i.max(0) as usize)
                                .map(|c| c as i32)
                                .unwrap_or(-1);
                            {
                                task.ram.pop_nv();
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_i32(code));
                            }
                        }
                        "replace" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).to_string())
                                .unwrap_or_default();
                            // Pop replacement arg first (top), then pattern arg
                            let repl_nv = task.ram.pop_nv();
                            let pat_nv = task.ram.pop_nv();
                            let pat = if auto_val::is_string(pat_nv) {
                                let i = auto_val::decode_string(pat_nv) as usize;
                                self.strings
                                    .read()
                                    .unwrap()
                                    .get(i)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            let repl = if auto_val::is_string(repl_nv) {
                                let i = auto_val::decode_string(repl_nv) as usize;
                                self.strings
                                    .read()
                                    .unwrap()
                                    .get(i)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            let result = s.replace(&pat, &repl);
                            let idx = self.add_string(result.into_bytes());
                            {
                                task.ram.pop_nv();
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push_str_idx(task, idx as usize);
                            }
                        }
                        "to_string" | "to_str" | "clone" => {
                            // str.to_string() / str.to_str() / str.clone() — return self
                            // No-op: receiver stays on stack as-is
                        }
                        "to_int" | "parse_int" => {
                            // Plan 402 §13.10: handle float→int truncation too.
                            // codegen routes ALL to_int calls here regardless of
                            // receiver type (there is no float/int type_name
                            // dispatch branch). A float receiver (e.g.
                            // `(math.random() * n).to_int()`) was decoded via
                            // decode_string on a float nanbox → garbage. Decode
                            // by the receiver's actual tag.
                            let result = if auto_val::is_f64(receiver_nv) {
                                auto_val::decode_f64(receiver_nv) as i32
                            } else if auto_val::is_f32(receiver_nv) {
                                auto_val::decode_f32(receiver_nv) as i32
                            } else if auto_val::is_string(receiver_nv) {
                                let str_idx = auto_val::decode_string(receiver_nv) as usize;
                                let s = self
                                    .strings
                                    .read()
                                    .unwrap()
                                    .get(str_idx)
                                    .map(|b| String::from_utf8_lossy(b).trim().to_string())
                                    .unwrap_or_default();
                                s.parse::<i32>().unwrap_or(0)
                            } else if auto_val::is_bool(receiver_nv) {
                                if auto_val::decode_bool(receiver_nv) {
                                    1
                                } else {
                                    0
                                }
                            } else {
                                auto_val::decode_i32(receiver_nv)
                            };
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_i32(result));
                            }
                        }
                        "to_uint" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).trim().to_string())
                                .unwrap_or_default();
                            let result = s.parse::<i64>().unwrap_or(0) as i32;
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_i32(result));
                            }
                        }
                        // Plan 403: str.to_float() — parse a numeric string to f64.
                        // Needed by the calculator engine (nums stored as strings,
                        // parsed to float at arithmetic time).
                        "to_float" | "parse_float" => {
                            let str_idx = auto_val::decode_string(receiver_nv) as usize;
                            let s = self
                                .strings
                                .read()
                                .unwrap()
                                .get(str_idx)
                                .map(|b| String::from_utf8_lossy(b).trim().to_string())
                                .unwrap_or_default();
                            let result = s.parse::<f64>().unwrap_or(0.0);
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_f64(result));
                            }
                        }
                        _ => {
                            // PLAN-057 T4（等价性缺陷族⑤）：未知 str 方法兜底
                            // 配平——原只压 None 不弹 receiver+实参，栈失衡 +1
                            // 平移后续调用的参数槽。未解析静态调用
                            // （Array.isArray/JSON.stringify 等，接收者=类型名
                            // 字符串）正落本臂：case_web_builtins A/B/C 标签
                            // 乱码（4000000/None/hi）根因。
                            for _ in 0..=arg_count {
                                task.ram.pop_nv();
                            }
                            let sp_now = task.ram.sp;
                            self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                            task.ram.push_nv(auto_val::encode_null());
                        }
                    }
                } else if type_name == "List" {
                    // PLAN-608 T02: 同 str 区——消费型臂（count|len/last/
                    // 未知兜底）弹出窗口结算（接收者 list_id 的 copy-on-load
                    // 堆份额按影子释放，此前裸弹每调用孤儿 +1）。
                    let spec_sp_entry = task.ram.sp;
                    // Plan 320: receiver may be Int(heap_id) or VmRef(decode_object).
                    // Try both to get the list_id.
                    let list_id = if auto_val::is_i32(receiver_nv) {
                        auto_val::decode_i32(receiver_nv) as u64
                    } else {
                        auto_val::decode_object(receiver_nv) as u64
                    };
                    match method_name.as_str() {
                        "count" | "len" => {
                            let len = if let Some(obj) = self.heap_objects.get(&list_id) {
                                let guard = obj.read().unwrap();
                                if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<i32>>()
                                {
                                    list.len() as i32
                                } else if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    // Plan 320: ListData<Value> (struct lists)
                                    list.len() as i32
                                } else {
                                    0
                                }
                            } else {
                                0
                            };
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_i32(len));
                            }
                        }
                        // Plan 403: List.last() — peek the last element without
                        // removing it (needed by the calculator's shunting-yard
                        // engine, which inspects stack tops before popping).
                        // Returns the element re-encoded onto the stack; pushes
                        // null if the list is empty.
                        "last" => {
                            // Plan 403: List.last() — peek the last element
                            // without removing it (the calculator's shunting-yard
                            // engine inspects stack tops before popping). Handles
                            // ListData<Value>, ListData<String>, and ListData<i32>.
                            let top_nv = if let Some(obj) = self.heap_objects.get(&list_id) {
                                let guard = obj.read().unwrap();
                                if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    list.elems
                                        .last()
                                        .map(|v| match v {
                                            auto_val::Value::Int(i) => auto_val::encode_i32(*i),
                                            auto_val::Value::Double(f) => auto_val::encode_f64(*f),
                                            auto_val::Value::Float(f) => {
                                                auto_val::encode_f64(*f as f64)
                                            }
                                            auto_val::Value::Bool(b) => auto_val::encode_bool(*b),
                                            auto_val::Value::Str(s) => {
                                                let idx =
                                                    self.add_string(s.to_string().into_bytes());
                                                auto_val::encode_string(idx as u32)
                                            }
                                            auto_val::Value::String(s) => {
                                                let idx =
                                                    self.add_string(s.to_string().into_bytes());
                                                auto_val::encode_string(idx as u32)
                                            }
                                            auto_val::Value::VmRef(r) => {
                                                auto_val::encode_object(r.id as u32)
                                            }
                                            _ => auto_val::encode_null(),
                                        })
                                        .unwrap_or(auto_val::encode_null())
                                } else if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<String>>()
                                {
                                    // Plan 403: List<str> (CREATE_LIST_STR) — peek
                                    // the last String and re-encode as nanbox string.
                                    list.elems
                                        .last()
                                        .map(|s| {
                                            let idx = self.add_string(s.as_bytes().to_vec());
                                            auto_val::encode_string(idx as u32)
                                        })
                                        .unwrap_or(auto_val::encode_null())
                                } else if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<i32>>()
                                {
                                    list.elems
                                        .last()
                                        .map(|&i| auto_val::encode_i32(i))
                                        .unwrap_or(auto_val::encode_null())
                                } else {
                                    auto_val::encode_null()
                                }
                            } else {
                                auto_val::encode_null()
                            };
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                self.rc_push(task, top_nv);
                            }
                        }
                        "get" => {
                            // Pop index arg, then get element from list
                            let index = auto_val::decode_i32(task.ram.pop_nv());
                            let list_id = auto_val::decode_object(receiver_nv) as u64;
                            if let Some(obj) = self.heap_objects.get(&list_id) {
                                let guard = obj.read().unwrap();
                                if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<i32>>()
                                {
                                    if let Some(val) = list.get(index as usize) {
                                        let v = *val;
                                        if v >= 4000000 {
                                            // Heap object ID — push as object reference
                                            {
                                                task.ram.pop_nv();
                                                for _ in 1..arg_count {
                                                    task.ram.pop_nv();
                                                }
                                                self.rc_push(
                                                    task,
                                                    auto_val::encode_object(v as u32),
                                                );
                                            }
                                        } else if let Some(bytes) = self.get_string(v as u32) {
                                            let new_idx = self.add_string(bytes.to_vec());
                                            {
                                                task.ram.pop_nv();
                                                for _ in 1..arg_count {
                                                    task.ram.pop_nv();
                                                }
                                                self.rc_push_str_idx(task, new_idx as usize);
                                            }
                                        } else {
                                            {
                                                task.ram.pop_nv();
                                                for _ in 1..arg_count {
                                                    task.ram.pop_nv();
                                                }
                                                task.ram.push_nv(auto_val::encode_i32(v));
                                            }
                                        }
                                    } else {
                                        // Out of bounds — push 0 (None)
                                        {
                                            task.ram.pop_nv();
                                            for _ in 1..arg_count {
                                                task.ram.pop_nv();
                                            }
                                            task.ram.push_i32(0);
                                        }
                                    }
                                } else if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<String>>()
                                {
                                    // Plan 403: List<str> via CREATE_LIST_STR.
                                    if let Some(s) = list.get(index as usize) {
                                        let idx = self.add_string(s.clone().into_bytes());
                                        {
                                            task.ram.pop_nv();
                                            for _ in 1..arg_count {
                                                task.ram.pop_nv();
                                            }
                                            self.rc_push_str_idx(task, idx as usize);
                                        }
                                    } else {
                                        {
                                            task.ram.pop_nv();
                                            for _ in 1..arg_count {
                                                task.ram.pop_nv();
                                            }
                                            task.ram.push_i32(0);
                                        }
                                    }
                                } else if let Some(list) = guard
                                    .as_any()
                                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    // Plan 403: ListData<Value> (from CREATE_ARRAY / `[]`) —
                                    // re-encode the element by variant onto the stack.
                                    if let Some(v) = list.get(index as usize) {
                                        let nv = match v {
                                            auto_val::Value::Int(i) => auto_val::encode_i32(*i),
                                            auto_val::Value::Double(f) => auto_val::encode_f64(*f),
                                            auto_val::Value::Float(f) => {
                                                auto_val::encode_f64(*f as f64)
                                            }
                                            auto_val::Value::Bool(b) => auto_val::encode_bool(*b),
                                            auto_val::Value::Str(s) => {
                                                let idx =
                                                    self.add_string(s.to_string().into_bytes());
                                                auto_val::encode_string(idx as u32)
                                            }
                                            auto_val::Value::String(s) => {
                                                let idx =
                                                    self.add_string(s.to_string().into_bytes());
                                                auto_val::encode_string(idx as u32)
                                            }
                                            auto_val::Value::VmRef(r) => {
                                                auto_val::encode_object(r.id as u32)
                                            }
                                            _ => auto_val::encode_null(),
                                        };
                                        {
                                            task.ram.pop_nv();
                                            for _ in 1..arg_count {
                                                task.ram.pop_nv();
                                            }
                                            self.rc_push(task, nv);
                                        }
                                    } else {
                                        {
                                            task.ram.pop_nv();
                                            for _ in 1..arg_count {
                                                task.ram.pop_nv();
                                            }
                                            task.ram.push_i32(0);
                                        }
                                    }
                                }
                            }
                        }
                        "push" => {
                            // Push element to end of List (uses unified list_id above).
                            // PLAN-608 T05 定罪注记：本臂**不可达**——resolve
                            // ("List.push"→auto.list.push) 恒命中
                            // shim_list_push（604/608 实证），下方代码是历史
                            // 化石。注意：与旧注释所述相反，这里**从未有过**
                            // stake 结算/容器 retain（list.push(elem_val) 裸存）
                            // ——若 registry 覆盖回退使本臂复活，即 UAF 面
                            // （路由守护见 tests/plan608_dispatch_golden_tests.rs；
                            // 移除候选登记 KNOWN-DEBT KD-VM5）。
                            let elem_nv = task.ram.pop_nv();
                            let elem_val = if auto_val::is_i32(elem_nv) {
                                auto_val::Value::Int(auto_val::decode_i32(elem_nv))
                            } else if auto_val::is_f64(elem_nv) {
                                // Plan 403: f64 values use raw bit encoding
                                // (encode_f64), so must be checked before the
                                // else-branch Int fallback — otherwise floats
                                // pushed onto a List<float> get corrupted to
                                // garbage ints (e.g. 3.0 -> 1074266112).
                                auto_val::Value::Double(auto_val::decode_f64(elem_nv))
                            } else if auto_val::is_f32(elem_nv) {
                                auto_val::Value::Double(auto_val::decode_f64(elem_nv))
                            } else if auto_val::is_object(elem_nv) {
                                auto_val::Value::VmRef(auto_val::VmRef {
                                    id: auto_val::decode_object(elem_nv) as usize,
                                })
                            } else if auto_val::is_string(elem_nv) {
                                let idx = auto_val::decode_string(elem_nv) as usize;
                                let bytes = self
                                    .strings
                                    .read()
                                    .unwrap()
                                    .get(idx)
                                    .cloned()
                                    .unwrap_or_default();
                                auto_val::Value::Str(
                                    String::from_utf8_lossy(&bytes).to_string().into(),
                                )
                            } else if auto_val::is_bool(elem_nv) {
                                auto_val::Value::Bool(auto_val::decode_bool(elem_nv))
                            } else if auto_val::is_null(elem_nv) {
                                auto_val::Value::Nil
                            } else {
                                auto_val::Value::Int(auto_val::decode_i32(elem_nv))
                            };
                            // Plan 320: push into ListData<Value> or ListData<i32>.
                            if let Some(obj) = self.get_heap_object(list_id) {
                                let mut guard = obj.write().unwrap();
                                if let Some(list) = guard
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    list.push(elem_val);
                                } else if let Some(list) = guard
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<i32>>(
                                ) {
                                    list.push(auto_val::decode_i32(elem_nv));
                                }
                            }
                            {
                                task.ram.pop_nv();
                                for _ in 1..arg_count {
                                    task.ram.pop_nv();
                                }
                                task.ram.push_i32(0);
                            }
                        }
                        "remove" => {
                            // Remove element at index from List (arrays DashMap)
                            let arr_key = if auto_val::is_object(receiver_nv) {
                                auto_val::decode_object(receiver_nv) as u64
                            } else if auto_val::is_i32(receiver_nv) {
                                auto_val::decode_i32(receiver_nv) as u64
                            } else {
                                0u64
                            };
                            // Pop the index arg
                            let index = auto_val::decode_i32(task.ram.pop_nv()) as usize;
                            if let Some(arr_ref) = self.get_heap_object(arr_key) {
                                let mut arr = arr_ref.write().unwrap();
                                if let Some(list) = arr
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    if index < list.elems.len() {
                                        list.elems.remove(index);
                                    }
                                }
                            }
                            // Pop receiver, push 0 (void)
                            {
                                task.ram.pop_nv();
                                task.ram.push_i32(0);
                            }
                        }
                        "pop" => {
                            // Pop last element from List (arrays DashMap)
                            let arr_key = if auto_val::is_object(receiver_nv) {
                                auto_val::decode_object(receiver_nv) as u64
                            } else if auto_val::is_i32(receiver_nv) {
                                auto_val::decode_i32(receiver_nv) as u64
                            } else {
                                0u64
                            };
                            // Plan 402 §13.10: return the popped element, not a
                            // fixed 0. The old `let _ = list.elems.pop()` discarded
                            // the value and pushed 0 — so `var x = stack.pop()` in
                            // flood-fill always got 0, making the loop never
                            // terminate (stack grew until OOM → process crash).
                            let popped: Option<auto_val::Value> =
                                if let Some(arr_ref) = self.get_heap_object(arr_key) {
                                    let mut arr = arr_ref.write().unwrap();
                                    if let Some(list) = arr
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    list.elems.pop()
                                } else {
                                    None
                                }
                                } else {
                                    None
                                };
                            // Push the popped value onto the operand stack (replacing receiver)
                            {
                                task.ram.pop_nv();
                            } // pop receiver
                            match popped {
                                Some(auto_val::Value::Int(i)) => {
                                    task.ram.push_nv(auto_val::encode_i32(i))
                                }
                                Some(auto_val::Value::Bool(b)) => {
                                    task.ram.push_nv(auto_val::encode_bool(b))
                                }
                                Some(auto_val::Value::Str(s)) => {
                                    let mut strings = self.strings.write().unwrap();
                                    let idx = strings.len() as u32;
                                    strings.push(s.as_bytes().to_vec());
                                    drop(strings);
                                    self.rc_push_str_idx(task, idx as usize);
                                }
                                Some(auto_val::Value::VmRef(r)) => {
                                    self.rc_push(task, auto_val::encode_object(r.id as u32))
                                }
                                Some(other) => {
                                    // For other types, push as i32 fallback
                                    task.ram.push_nv(auto_val::encode_i32(0));
                                }
                                None => task.ram.push_nv(auto_val::encode_i32(0)),
                            }
                        }
                        "insert" => {
                            // Insert element at index in List (arrays DashMap)
                            let arr_key = if auto_val::is_object(receiver_nv) {
                                auto_val::decode_object(receiver_nv) as u64
                            } else if auto_val::is_i32(receiver_nv) {
                                auto_val::decode_i32(receiver_nv) as u64
                            } else {
                                0u64
                            };
                            // Stack: [..., receiver, index, elem]
                            let elem_nv = task.ram.pop_nv();
                            let index = auto_val::decode_i32(task.ram.pop_nv()) as usize;
                            if let Some(arr_ref) = self.get_heap_object(arr_key) {
                                let mut arr = arr_ref.write().unwrap();
                                if let Some(list) = arr
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    let pos = index.min(list.elems.len());
                                    {
                                        let value = if auto_val::is_i32(elem_nv) {
                                            auto_val::Value::Int(auto_val::decode_i32(elem_nv))
                                        } else if auto_val::is_object(elem_nv) {
                                            auto_val::Value::VmRef(auto_val::VmRef {
                                                id: auto_val::decode_object(elem_nv) as usize,
                                            })
                                        } else if auto_val::is_string(elem_nv) {
                                            let idx = auto_val::decode_string(elem_nv) as usize;
                                            let bytes = self
                                                .strings
                                                .read()
                                                .unwrap()
                                                .get(idx)
                                                .cloned()
                                                .unwrap_or_default();
                                            auto_val::Value::Str(
                                                String::from_utf8_lossy(&bytes).to_string().into(),
                                            )
                                        } else if auto_val::is_bool(elem_nv) {
                                            auto_val::Value::Bool(auto_val::decode_bool(elem_nv))
                                        } else if auto_val::is_f64(elem_nv) {
                                            auto_val::Value::Double(auto_val::decode_f64(elem_nv))
                                        } else if auto_val::is_f32(elem_nv) {
                                            auto_val::Value::Float(
                                                auto_val::decode_f32(elem_nv) as f64
                                            )
                                        } else if auto_val::is_null(elem_nv) {
                                            auto_val::Value::Nil
                                        } else {
                                            auto_val::Value::Int(auto_val::decode_i32(elem_nv))
                                        };
                                        list.elems.insert(pos, value);
                                    }
                                }
                            }
                            // Pop receiver, push 0 (void)
                            {
                                task.ram.pop_nv();
                                task.ram.push_i32(0);
                            }
                        }
                        "sort" | "dedup" | "reverse" => {
                            // In-place sort/dedup/reverse of List
                            let arr_key = if auto_val::is_object(receiver_nv) {
                                auto_val::decode_object(receiver_nv) as u64
                            } else if auto_val::is_i32(receiver_nv) {
                                auto_val::decode_i32(receiver_nv) as u64
                            } else {
                                0u64
                            };
                            if let Some(obj) = self.heap_objects.get(&arr_key) {
                                let mut guard = obj.write().unwrap();
                                if let Some(list) = guard
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    match method_name.as_str() {
                                        "sort" => {
                                            list.elems.sort_by(|a, b| match (a, b) {
                                                (
                                                    auto_val::Value::Int(x),
                                                    auto_val::Value::Int(y),
                                                ) => x.cmp(y),
                                                (
                                                    auto_val::Value::Uint(x),
                                                    auto_val::Value::Uint(y),
                                                ) => x.cmp(y),
                                                (
                                                    auto_val::Value::Float(x),
                                                    auto_val::Value::Float(y),
                                                ) => x
                                                    .partial_cmp(y)
                                                    .unwrap_or(std::cmp::Ordering::Equal),
                                                (
                                                    auto_val::Value::Double(x),
                                                    auto_val::Value::Double(y),
                                                ) => x
                                                    .partial_cmp(y)
                                                    .unwrap_or(std::cmp::Ordering::Equal),
                                                (
                                                    auto_val::Value::Bool(x),
                                                    auto_val::Value::Bool(y),
                                                ) => x.cmp(y),
                                                (
                                                    auto_val::Value::Str(x),
                                                    auto_val::Value::Str(y),
                                                ) => x.to_string().cmp(&y.to_string()),
                                                (
                                                    auto_val::Value::String(x),
                                                    auto_val::Value::String(y),
                                                ) => x.as_str().cmp(y.as_str()),
                                                _ => std::cmp::Ordering::Equal,
                                            });
                                        }
                                        "dedup" => {
                                            list.elems.dedup();
                                        }
                                        "reverse" => {
                                            list.elems.reverse();
                                        }
                                        _ => {}
                                    }
                                } else if let Some(list) = guard
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<i32>>(
                                ) {
                                    match method_name.as_str() {
                                        "sort" => {
                                            list.elems.sort();
                                        }
                                        "dedup" => {
                                            list.elems.dedup();
                                        }
                                        "reverse" => {
                                            list.elems.reverse();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            // Pop args, leave receiver on stack as return value
                            for _ in 0..arg_count {
                                task.ram.pop_nv();
                            }
                        }
                        "sort_by" | "sort_by_key" => {
                            // In-place sort with comparator — pop closure arg, use default sort for now
                            let arr_key = if auto_val::is_object(receiver_nv) {
                                auto_val::decode_object(receiver_nv) as u64
                            } else {
                                auto_val::decode_i32(receiver_nv) as u64
                            };
                            // Pop args (closure)
                            for _ in 0..arg_count {
                                task.ram.pop_nv();
                            }
                            if let Some(obj) = self.heap_objects.get(&arr_key) {
                                let mut guard = obj.write().unwrap();
                                if let Some(list) = guard
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<auto_val::Value>>()
                                {
                                    list.elems.sort_by(|a, b| match (a, b) {
                                        (auto_val::Value::Int(x), auto_val::Value::Int(y)) => {
                                            x.cmp(y)
                                        }
                                        (auto_val::Value::Uint(x), auto_val::Value::Uint(y)) => {
                                            x.cmp(y)
                                        }
                                        (auto_val::Value::Float(x), auto_val::Value::Float(y)) => {
                                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                                        }
                                        (
                                            auto_val::Value::Double(x),
                                            auto_val::Value::Double(y),
                                        ) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                        _ => std::cmp::Ordering::Equal,
                                    });
                                } else if let Some(list) = guard
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::types::ListData<i32>>(
                                ) {
                                    list.elems.sort();
                                }
                            }
                        }
                        _ => {
                            // Identity operations: return receiver unchanged, only pop args
                            if matches!(
                                method_name.as_str(),
                                "collect"
                                    | "rev"
                                    | "filter_map"
                                    | "flatten"
                                    | "into_iter"
                                    | "iter"
                                    | "iter_mut"
                                    | "par_iter"
                                    | "par_iter_mut"
                                    | "for_each"
                                    | "map"
                                    | "filter"
                                    | "find"
                                    | "any"
                                    | "all"
                                    | "reduce"
                                    | "fold"
                                    | "to_array"
                            ) {
                                // Pop args only (not receiver) — receiver stays as return value
                                for _ in 0..arg_count {
                                    task.ram.pop_nv();
                                }
                            } else {
                                // PLAN-057 T4（等价性缺陷族⑤）：未知 List
                                // 方法兜底配平（同 str 臂，原失衡 +1）。
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                let sp_now = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, sp_now, spec_sp_entry);
                                task.ram.push_nv(auto_val::encode_null());
                            }
                        }
                    }
                } else if auto_val::is_i32(receiver_nv)
                    && matches!(method_name.as_str(), "char_code_at" | "charCodeAt")
                    && !self
                        .heap_objects
                        .contains_key(&(auto_val::decode_i32(receiver_nv).max(0) as u64))
                {
                    // PLAN-055 Char（码点）接收者恒等臂 + PLAN-057 T5 重定位
                    // （等价性缺陷族④）：for-in over str 经 GET_ELEM 逐字符
                    // 产出 i32 码点，`c.char_code_at(0)` 语义等价单字符字符串
                    // 的 charCodeAt（web a2ts 同值）= 恒等。原位置在链尾，
                    // 被 `<unknown:` 接收者臂（整型字面量方法族）先行吞
                    // None——恒等臂永不命中（case_str_charcode A 红根因），
                    // 故提升到本臂之前。heap 命中者优先按对象走原链。
                    for _ in 0..=arg_count {
                        task.ram.pop_nv();
                    }
                    task.ram.push_nv(receiver_nv);
                } else if type_name.starts_with("<unknown:")
                    || type_name.starts_with("<invalid")
                    || type_name.starts_with("<unknown_nv:")
                {
                    // Integer literal methods: 0x1234.to_be_bytes(), etc.
                    // Plan 499 M3: f64/f32/bool nanbox 接收者的 type_name 是
                    // "<unknown_nv:{hex}>"(下方 6036 一带最终 else 产出),旧条件
                    // 只匹配 "<unknown:"/"<invalid" 前缀——浮点接收者的
                    // .to_int()/.to_string() 漏网直坠 CALL_SPEC 报错臂
                    // (axisPointer 索引吸附 fi.to_int() 现场)。403-F 的
                    // 按标签解码臂本就为这些接收者书写,补前缀即通。
                    let int_val = auto_val::decode_i32(receiver_nv);
                    match method_name.as_str() {
                        "to_be_bytes" | "to_le_bytes" => {
                            let be = method_name == "to_be_bytes";
                            let bytes: Vec<u8> = if be {
                                int_val.to_be_bytes().to_vec()
                            } else {
                                int_val.to_le_bytes().to_vec()
                            };
                            use crate::vm::types::ListData;
                            let mut list: ListData<i32> = ListData::new();
                            for b in bytes {
                                list.push(b as i32);
                            }
                            let list_id = self.insert_heap_object(list);
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                self.rc_push(task, auto_val::encode_object(list_id as u32));
                            }
                        }
                        "to_string" | "to_str" => {
                            // Plan 403-F Bug D: f64/f32/bool values reach here
                            // as non-i32 nanbox values (type_name
                            // "<unknown_nv:...>"). Decode by runtime tag.
                            let s = if auto_val::is_f64(receiver_nv) {
                                let f = auto_val::decode_f64(receiver_nv);
                                fmt_f64(f)
                            } else if auto_val::is_f32(receiver_nv) {
                                let f = auto_val::decode_f32(receiver_nv);
                                fmt_f64(f as f64)
                            } else if auto_val::is_bool(receiver_nv) {
                                if auto_val::decode_bool(receiver_nv) {
                                    "true".to_string()
                                } else {
                                    "false".to_string()
                                }
                            } else {
                                int_val.to_string()
                            };
                            let bytes = s.into_bytes();
                            let idx = {
                                let mut strings = self.strings.write().unwrap();
                                strings.push(bytes);
                                strings.len() - 1
                            };
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                self.rc_push_str_idx(task, idx as usize);
                            }
                        }
                        // Plan 403-F: float to_int/to_float/to_uint on
                        // f64/f32 values (which reach here as "<unknown_nv>").
                        // Without this, float.to_int() returns null and
                        // downstream integer math crashes.
                        "to_int" | "parse_int" => {
                            let v = if auto_val::is_f64(receiver_nv) {
                                auto_val::decode_f64(receiver_nv) as i32
                            } else if auto_val::is_f32(receiver_nv) {
                                auto_val::decode_f32(receiver_nv) as i32
                            } else {
                                int_val
                            };
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                task.ram.push_nv(auto_val::encode_i32(v));
                            }
                        }
                        "to_uint" => {
                            let v = if auto_val::is_f64(receiver_nv) {
                                auto_val::decode_f64(receiver_nv) as i64 as i32
                            } else if auto_val::is_f32(receiver_nv) {
                                auto_val::decode_f32(receiver_nv) as i64 as i32
                            } else {
                                int_val
                            };
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                task.ram.push_nv(auto_val::encode_i32(v));
                            }
                        }
                        "to_float" | "parse_float" => {
                            let v = if auto_val::is_f64(receiver_nv) {
                                auto_val::decode_f64(receiver_nv)
                            } else if auto_val::is_f32(receiver_nv) {
                                auto_val::decode_f32(receiver_nv) as f64
                            } else {
                                int_val as f64
                            };
                            {
                                for _ in 0..=arg_count {
                                    task.ram.pop_nv();
                                }
                                task.ram.push_nv(auto_val::encode_f64(v));
                            }
                        }
                        _ => {
                            // PLAN-057 T4（等价性缺陷族⑤）：unknown 接收者臂
                            // 兜底配平（同 str/List 臂，原失衡 +1）。
                            for _ in 0..=arg_count {
                                task.ram.pop_nv();
                            }
                            task.ram.push_nv(auto_val::encode_null());
                        }
                    }
                } else if type_name.contains("::")
                    || type_name.contains("RustStdlib")
                    || matches!(
                        type_name.as_str(),
                        "Command"
                            | "Stdio"
                            | "Writer"
                            | "Reader"
                            | "ReaderBuilder"
                            | "WriterBuilder"
                            | "StringRecord"
                            | "ThreadRng"
                            | "Complex"
                            | "BigInt"
                            | "Normal"
                            | "Rng"
                            | "WalkDir"
                            | "RefCell"
                            | "Instant"
                            | "Duration"
                            | "Child"
                            | "OnceCell"
                            | "Backtrace"
                            | "Args"
                            | "File"
                            | "FileWriter"
                            | "PathBuf"
                            | "String"
                            | "Vec"
                    )
                {
                    // Generic fallback for external crate types (csv::ReaderBuilder, etc.)
                    // Also matches bare type names from static calls (e.g., "Command" from Command.arg)
                    // Route through shim_rust_stdlib_dispatch with type_name + method injected
                    let dispatch_id: u16 = 3000; // NATIVE_RUST_STDLIB_DISPATCH
                                                 // Extract short type name: "csv::ReaderBuilder" -> "ReaderBuilder"
                    let short_type = type_name.rsplit("::").next().unwrap_or(&type_name);

                    // Detect if receiver is a type-name string from a static call
                    // Static calls have a string receiver (e.g., "WalkDir" from WalkDir.new)
                    // Instance calls have a heap handle receiver (i32 > 0 or object)
                    let receiver_is_type_string = auto_val::is_string(receiver_nv);

                    // Push method and type_name strings for the dispatch handler
                    let method_bytes = method_name.as_bytes().to_vec();
                    let type_bytes = short_type.as_bytes().to_vec();
                    let method_idx = self.add_string(method_bytes);
                    let type_idx = self.add_string(type_bytes);
                    // shim_rust_stdlib_dispatch expects: type_name(str), method(str) on top
                    // Stack is: [..., receiver, arg0..argN-1]
                    // Push type_name and method on top (1 nanbox slot each, no null marker)
                    {
                        self.rc_push_str_idx(task, type_idx as usize);
                        self.rc_push_str_idx(task, method_idx as usize);
                    }
                    if let Some(shim) = self.native_interface.get(dispatch_id).cloned() {
                        shim(task, self)?;
                    } else {
                        return Err(VMError::MissingNative(dispatch_id));
                    }
                    // Dispatch handler consumed type_name + method + args.
                    // For instance methods, receiver was consumed by handler too.
                    // For static calls, receiver type-name string is still on stack below return.
                    if receiver_is_type_string {
                        let ret_nv = task.ram.pop_nv();
                        task.ram.pop_nv(); // remove the type-name receiver leftover
                        task.ram.push_nv(ret_nv);
                    }
                } else if method_name == "is_none" || method_name == "is_some" {
                    // Plan 240: Inline is_none/is_some for any type (Option semantics)
                    // Check nanbox type tag to determine Some vs None
                    let (is_some, receiver_is_string) = {
                        let recv_nv = task.ram.read_nv(receiver_pos);
                        if auto_val::is_string(recv_nv) {
                            // String tag at top position (shouldn't happen with 2-slot string,
                            // but handle if the receiver is already the string tag)
                            (true, true)
                        } else if auto_val::is_null(recv_nv) {
                            // Could be the null marker of a 2-slot string
                            // Check the slot below for string tag
                            if receiver_pos > 0
                                && auto_val::is_string(task.ram.read_nv(receiver_pos - 1))
                            {
                                (true, true) // string value = Some
                            } else {
                                (false, false) // standalone null = None
                            }
                        } else if auto_val::is_i32(recv_nv) {
                            let val = auto_val::decode_i32(recv_nv);
                            (val >= 0, false)
                        } else {
                            (true, false) // object, bool, f64, etc. = Some
                        }
                    };
                    let result = if method_name == "is_some" {
                        if is_some {
                            1
                        } else {
                            0
                        }
                    } else {
                        if is_some {
                            0
                        } else {
                            1
                        }
                    };
                    {
                        if receiver_is_string {
                            // String receiver occupies 2 nanbox slots
                            // Pop all: null marker + args + string tag = arg_count + 2
                            for _ in 0..=(arg_count + 1) {
                                task.ram.pop_nv();
                            }
                        } else {
                            for _ in 0..=arg_count {
                                task.ram.pop_nv();
                            }
                        }
                        task.ram.push_nv(auto_val::encode_i32(result));
                    }
                } else if method_name == "to_string" || method_name == "to_str" {
                    // Inline to_string — convert to debug string representation
                    let recv_nv = task.ram.read_nv(receiver_pos);
                    let recv_val = if auto_val::is_i32(recv_nv) {
                        auto_val::decode_i32(recv_nv)
                    } else if auto_val::is_object(recv_nv) {
                        auto_val::decode_object(recv_nv) as i32
                    } else {
                        task.ram.read_i32(receiver_pos)
                    };
                    let mut converted = false;
                    if recv_val > 0 {
                        let obj_key = recv_val as u64;
                        if let Some(obj_lock) = self.heap_objects.get(&obj_key) {
                            let guard = obj_lock.read().unwrap();
                            // RustStdlibObject wrapping a String
                            if let Some(rust_obj) = guard
                                .as_any()
                                .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>(
                            ) {
                                if let Some(s) = rust_obj.downcast_ref::<String>() {
                                    let bytes = s.as_bytes().to_vec();
                                    let idx = {
                                        let mut strings = self.strings.write().unwrap();
                                        strings.push(bytes);
                                        strings.len() - 1
                                    };
                                    for _ in 0..=arg_count {
                                        task.ram.pop_i32();
                                    }
                                    {
                                        self.rc_push_str_idx(task, idx as usize);
                                    }
                                    converted = true;
                                }
                            }
                            // GenericInstanceData (user-defined struct)
                            if !converted {
                                if let Some(inst) = guard.as_any().downcast_ref::<crate::vm::generic_registry::GenericInstanceData>() {
                                        let type_name = inst.mono_name.split('_').next().unwrap_or(&inst.mono_name);
                                        let mut parts = vec![type_name.to_string()];
                                        let strings_guard = self.strings.read().unwrap();
                                        for field in &inst.fields {
                                            let s = match field {
                                                auto_val::Value::Int(i) => i.to_string(),
                                                auto_val::Value::Uint(u) => u.to_string(),
                                                auto_val::Value::Str(s) => format!("\"{}\"", s.as_str()),
                                                auto_val::Value::Bool(b) => b.to_string(),
                                                auto_val::Value::Double(d) => format!("{:.1}", d),
                                                auto_val::Value::Nil => "null".to_string(),
                                                auto_val::Value::VmRef(r) => format!("<obj:{}>", r.id),
                                                _ => "?".to_string(),
                                            };
                                            parts.push(s);
                                        }
                                        drop(strings_guard);
                                        let debug_str = if inst.fields.is_empty() {
                                            type_name.to_string()
                                        } else {
                                            format!("{}({})", type_name, parts[1..].join(", "))
                                        };
                                        let bytes = debug_str.into_bytes();
                                        let idx = {
                                            let mut strings = self.strings.write().unwrap();
                                            strings.push(bytes);
                                            strings.len() - 1
                                        };
                                        for _ in 0..=arg_count { task.ram.pop_i32(); }
                                        {
                                            self.rc_push_str_idx(task, idx as usize);
                                        }
                                        converted = true;
                                    }
                            }
                        }
                    }
                    if !converted {
                        // Primitive value — convert i32/f32/f64/bool to string
                        // Plan 403-F Bug D: added is_f32 branch (was missing,
                        // so f32 values fell through to the {:?} debug dump).
                        {
                            let s = if auto_val::is_i32(recv_nv) {
                                auto_val::decode_i32(recv_nv).to_string()
                            } else if auto_val::is_f64(recv_nv) {
                                fmt_f64(auto_val::decode_f64(recv_nv))
                            } else if auto_val::is_f32(recv_nv) {
                                fmt_f64(auto_val::decode_f32(recv_nv) as f64)
                            } else if auto_val::is_bool(recv_nv) {
                                if auto_val::decode_bool(recv_nv) {
                                    "true".to_string()
                                } else {
                                    "false".to_string()
                                }
                            } else if auto_val::is_null(recv_nv) {
                                "null".to_string()
                            } else {
                                format!("{:?}", recv_nv)
                            };
                            // Pop receiver + args
                            for _ in 0..=arg_count {
                                task.ram.pop_nv();
                            }
                            // Push string result
                            let bytes = s.into_bytes();
                            let idx = {
                                let mut strings = self.strings.write().unwrap();
                                strings.push(bytes);
                                strings.len() - 1
                            };
                            self.rc_push_str_idx(task, idx as usize);
                        }
                    }
                } else if type_name == "None" {
                    // PLAN-044: None-receiver protocol — Option 链的 VM 坍缩
                    // 语义(musk Json body 缺字段 → null 后的 unwrap_or 链):
                    //   None.unwrap_or(d) → d;None.clone()/.ok() → None 透传。
                    match method_name.as_str() {
                        "unwrap_or" if arg_count == 1 => {
                            let arg = task.ram.pop_nv();
                            task.ram.pop_nv(); // receiver (None)
                            task.ram.push_nv(arg);
                        }
                        "clone" | "ok" => {
                            // identity: receiver stays as the result
                        }
                        _ => {
                            return Err(VMError::RuntimeError(format!(
                                "CALL_SPEC: no function '{}' for type 'None'",
                                method_name
                            )));
                        }
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "CALL_SPEC: no function '{}' for type '{}'",
                        func_name, type_name
                    )));
                }
            }
            _ => return self.ops_calls(task, op, op_byte),
        }
        Ok(StepResult::Continue)
    }
}
