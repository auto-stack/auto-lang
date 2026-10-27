//! PLAN-843 结构拆分：run_one_instruction 尾递归派发段（连续臂切片，纯搬移）。
use super::*;

impl AutoVM {
    /// PLAN-843 尾递归派发段（自 run_one_instruction 连续臂切片，臂体逐字保留）。
    pub(super) fn ops_collections(&self, task: &mut AutoTask, op: OpCode, op_byte: u8) -> Result<StepResult, VMError> {
        match op {
            OpCode::SET_GENERIC_FIELD => {
                // Plan 087 Phase 2: Set field value in generic instance
                // Plan 118 Phase 7: Stack layout changed to [..., value, instance_id]
                // Code layout: [opcode, field_index:u32]
                // Stack layout: [..., value, instance_id] (value pushed first, then instance_id)
                // Stack after: [...]
                use crate::vm::generic_registry::GenericInstanceData;
                use crate::vm::heap_object::TypeTag;

                vm_debug!("DEBUG: SET_GENERIC_FIELD executing at IP={}", task.ip);

                // Read field_index from code stream (not stack!)
                let field_index = self.flash.read_u32(task.ip) as usize;
                task.ip += 4;

                // Pop instance_id (stack top)
                let instance_id = task.ram.pop_i32() as u64;

                // Pop value (below instance_id)
                let value = {
                    // Plan 377: 全值单槽。pop_arith_operand 弹单个 NanoValue 并
                    // 标识是否 f64（非 nanboxed），无需 2-slot padding 处理。
                    let (bits, is_f64) = task.ram.pop_arith_operand();
                    if is_f64 {
                        Value::Double(f64::from_bits(bits))
                    } else {
                        self.decode_tagged_nv(bits)
                    }
                };

                vm_debug!(
                    "DEBUG: SET_GENERIC_FIELD: instance_id={}, field_index={}, value={:?}",
                    instance_id,
                    field_index,
                    value
                );

                // Plan 419: 记录旧字段引用(写锁释放后级联回收)。
                let mut old_field_ref: Option<u64> = None;

                // Get instance and set field
                if let Some(obj) = self.get_heap_object(instance_id) {
                    let mut guard = obj.write().unwrap();

                    // Check if this is a GenericInstance (any variant)
                    let is_generic_instance =
                        matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                    if is_generic_instance {
                        if let Some(instance) =
                            guard.as_any_mut().downcast_mut::<GenericInstanceData>()
                        {
                            old_field_ref =
                                instance.fields.get(field_index).and_then(|v| match v {
                                    Value::VmRef(r) => Some(r.id as u64),
                                    _ => None,
                                });
                            let value_repr = format!("{:?}", value); // Capture before move
                            instance.set_field(field_index, value).map_err(|e| {
                                VMError::RuntimeError(format!("Failed to set field: {}", e))
                            })?;
                            vm_debug!(
                                "DEBUG: SET_GENERIC_FIELD: successfully set field {} to {}",
                                field_index,
                                value_repr
                            );
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: SET_GENERIC_FIELD failed to downcast GenericInstance"
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Type error: SET_GENERIC_FIELD expected GenericInstance, got {:?}",
                            guard.type_tag()
                        )));
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid instance ID: {}",
                        instance_id
                    )));
                }
                // Plan 419: 旧字段级联释放 + receiver stake 死亡
                // (value 自栈转移进字段,计数不变)。
                if let Some(old_id) = old_field_ref {
                    self.rc_release_id(old_id);
                }
                self.rc_release_id(instance_id);
            }
            OpCode::LIST_PUSH_INT => {
                // PLAN-062: 列表元素突变（同 SET_FIELD 口径）。
                // PLAN-047 T-03: bump 移至 list_id 出栈后定点归因（B 类）。
                // Plan 077 Phase 7: Optimized with inline helper
                // Stack layout: [..., list_id, value:int]
                // Pop value first (top of stack), then list_id
                let value = task.ram.pop_i32();
                let list_id = task.ram.pop_i32() as u64;
                self.bump_path(list_id, None);

                // Get list from unified registry and downcast to ListData<i32>
                use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
                use crate::vm::types::ListData;

                if let Some(obj) = self.get_heap_object(list_id) {
                    let mut guard = obj.write().unwrap();

                    // Optimized: single type check + downcast (inline)
                    if let Some(list) =
                        try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt)
                    {
                        if !list.push(value) {
                            return Err(VMError::RuntimeError(format!(
                                "List capacity exceeded (InlineInt64 limit: 64)"
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Type error: LIST_PUSH_INT expected ListInt"
                        )));
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid list ID: {}",
                        list_id
                    )));
                }
                // Plan 419: value 自栈转移进容器;receiver stake 死亡。
                self.rc_release_id(list_id);
            }
            OpCode::LIST_POP_INT => {
                // PLAN-062: 列表元素突变（同 SET_FIELD 口径）。
                // PLAN-047 T-03: bump 移至 list_id 出栈后定点归因（B 类）。
                // Plan 077 Phase 7: Optimized with inline helper
                // Stack layout: [..., list_id]
                // Pop list_id, get list, pop element, push result
                let list_id = task.ram.pop_i32() as u64;
                self.bump_path(list_id, None);

                // Get list from unified registry and downcast to ListData<i32>
                use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
                use crate::vm::types::ListData;

                if let Some(obj) = self.get_heap_object(list_id) {
                    let mut guard = obj.write().unwrap();

                    // Optimized: single type check + downcast (inline)
                    if let Some(list) =
                        try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt)
                    {
                        let value = list.pop().unwrap_or(0);
                        // Plan 419: 元素离开容器,其 stake 转移给栈(计数不变)。
                        task.ram.push_i32(value);
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Type error: LIST_POP_INT expected ListInt"
                        )));
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid list ID: {}",
                        list_id
                    )));
                }
                // Plan 419: receiver stake 死亡。
                self.rc_release_id(list_id);
            }
            OpCode::LIST_GET_INT => {
                // Plan 077 Phase 7: Optimized with inline helper
                // Stack layout: [..., list_id, index:int]
                // Pop index first (top of stack), then list_id
                let index = task.ram.pop_i32() as usize;
                let list_id = task.ram.pop_i32() as u64;
                // PLAN-047 T-04: VM 读臂依赖录制（容器内容粗粒度面；
                // 无效 id 过录=保守方向）。
                self.record_heap_read_any(list_id);

                // Get list from unified registry and downcast to ListData<i32>
                use crate::vm::heap_object::{try_downcast_checked, TypeTag};
                use crate::vm::types::ListData;

                if let Some(obj) = self.get_heap_object(list_id) {
                    let guard = obj.read().unwrap();

                    // Optimized: single type check + downcast (inline)
                    if let Some(list) =
                        try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt)
                    {
                        let value = list.get(index).copied().unwrap_or(0);
                        // Plan 419: 元素仍在容器 —— 拷贝引用入栈 +1。
                        if (value as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
                            self.rc_retain_id(value as u64);
                        }
                        task.ram.push_i32(value);
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Type error: LIST_GET_INT expected ListInt"
                        )));
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid list ID: {}",
                        list_id
                    )));
                }
                // Plan 419: receiver stake 死亡。
                self.rc_release_id(list_id);
            }
            OpCode::LIST_SET_INT => {
                // PLAN-062: 列表元素突变（同 SET_FIELD 口径）。
                // PLAN-047 T-03: bump 移至 list_id 出栈后定点归因（B 类）。
                // Plan 077 Phase 7: Optimized with inline helper
                // Stack layout: [..., list_id, index:int, value:int]
                // Pop value first, then index, then list_id
                let value = task.ram.pop_i32();
                let index = task.ram.pop_i32() as usize;
                let list_id = task.ram.pop_i32() as u64;
                self.bump_path(list_id, None);

                // Get list from unified registry and downcast to ListData<i32>
                use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
                use crate::vm::types::ListData;

                let mut old_elem_ref: Option<u64> = None;
                if let Some(obj) = self.get_heap_object(list_id) {
                    let mut guard = obj.write().unwrap();

                    // Optimized: single type check + downcast (inline)
                    if let Some(list) =
                        try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt)
                    {
                        if let Some(&old) = list.elems.get(index) {
                            if (old as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
                                old_elem_ref = Some(old as u64);
                            }
                        }
                        if !list.set(index, value) {
                            return Err(VMError::RuntimeError(format!(
                                "List index out of bounds: {}",
                                index
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Type error: LIST_SET_INT expected ListInt"
                        )));
                    }
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid list ID: {}",
                        list_id
                    )));
                }
                // Plan 419: 旧元素释放 + 新值自栈转移 + receiver 死亡。
                if let Some(old_id) = old_elem_ref {
                    self.rc_release_id(old_id);
                }
                self.rc_release_id(list_id);
            }
            // Slice: stack: container, start, end -> new_container
            OpCode::SLICE => {
                let end = task.ram.pop_i32();
                let start = task.ram.pop_i32();
                let container = task.ram.pop_i32();

                // Tagged string slice
                if container < 0 && container > -1000000 && container != -2147483648 {
                    let str_idx = (-container - 1) as usize;
                    let strings = self.strings.read().unwrap();
                    if let Some(bytes) = strings.get(str_idx) {
                        let s = String::from_utf8_lossy(bytes).to_string();
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len();
                        let s_start = if start < 0 {
                            0
                        } else {
                            (start as usize).min(len)
                        };
                        let s_end = if end < 0 {
                            len
                        } else {
                            (end as usize).min(len)
                        };
                        let sliced: String = chars[s_start..s_end].iter().collect();
                        drop(strings);
                        let new_idx = self.add_string(sliced.into_bytes());
                        self.rc_push_str_idx(task, new_idx as usize);
                    } else {
                        task.ram.push_i32(0);
                    }
                } else {
                    // Array slice
                    let arr_key = container as u64;
                    if let Some(arr_lock) = self.get_heap_object(arr_key) {
                        let arr = arr_lock.read().unwrap();
                        if let Some(list) = arr
                            .as_any()
                            .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                        {
                            let len = list.elems.len();
                            let s_start = if start < 0 {
                                0
                            } else {
                                (start as usize).min(len)
                            };
                            let s_end = if end < 0 {
                                len
                            } else {
                                (end as usize).min(len)
                            };
                            let sliced: Vec<auto_val::Value> = list.elems[s_start..s_end].to_vec();
                            drop(arr);
                            // Plan 390 §15 H3b: slice result is a new
                            // ListData<Value> in heap_objects.
                            let new_id = self.insert_heap_object(crate::vm::types::ListData {
                                elems: sliced,
                                storage: None,
                            });
                            self.rc_push(task, auto_val::encode_object(new_id as u32));
                        } else {
                            drop(arr);
                            task.ram.push_i32(0);
                        }
                    } else {
                        task.ram.push_i32(0);
                    }
                }
            }
            // Plan 200: Create tuple from stack elements
            // Stack: elem0, elem1, ..., elemN-1 -> tuple_id
            OpCode::CREATE_TUPLE => {
                use crate::vm::generic_registry::GenericInstanceData;
                let elem_count = self.flash.read_u8(task.ip);
                task.ip += 1;
                let mut fields = Vec::with_capacity(elem_count as usize);
                for _ in 0..elem_count {
                    {
                        let nv = task.ram.pop_nv();
                        let val = self.decode_tagged_nv(nv);
                        fields.push(val);
                    }
                }
                fields.reverse();
                let mut data = GenericInstanceData::new(
                    format!("tuple_{}", elem_count).into(),
                    vec![auto_val::Value::Null; fields.len()],
                );
                for (i, val) in fields.into_iter().enumerate() {
                    let _ = data.set_field(i, val);
                }
                let instance_id = self.insert_heap_object(data);
                self.rc_push(task, auto_val::encode_object(instance_id as u32));
            }
            // Plan 200: Get tuple field by index
            // Stack: tuple_id -> value (field_index from bytecode)
            OpCode::GET_TUPLE_FIELD => {
                use crate::vm::generic_registry::GenericInstanceData;
                let field_index = self.flash.read_u8(task.ip);
                task.ip += 1;
                // Plan 390 §15 H2: tuple id may be TAG_OBJECT-encoded now.
                let tuple_id = {
                    let nv = task.ram.pop_nv();
                    if auto_val::is_object(nv) {
                        auto_val::decode_object(nv) as u64
                    } else {
                        auto_val::decode_i32(nv) as u64
                    }
                };
                if let Some(lock) = self.get_heap_object(tuple_id) {
                    let guard = lock.read().unwrap();
                    if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                        if let Some(val) = instance.get_field(field_index as usize) {
                            match val {
                                auto_val::Value::Int(n) => task.ram.push_i32(*n),
                                auto_val::Value::Bool(b) => {
                                    task.ram.push_i32(if *b { 1 } else { 0 })
                                }
                                auto_val::Value::Str(s) => {
                                    let idx = self.add_string(s.as_bytes().to_vec());
                                    self.rc_push_str_idx(task, idx as usize);
                                }
                                _ => task.ram.push_i32(0),
                            }
                        } else {
                            task.ram.push_i32(0);
                        }
                    } else {
                        task.ram.push_i32(0);
                    }
                } else {
                    task.ram.push_i32(0);
                }
            }
            // Plan 073: Array element access (arr[index])
            // Plan 080: Also supports heap objects (lists like List<int>)
            // Plan 118 Phase 4: Also supports string indexing (str[index])
            OpCode::GET_ELEM => {
                // Stack: array_id/list_id/str_id, index
                // Pop index first (top of stack).
                // Plan 437: 保留原 nv tag —— record 动态字段访问 d[field_name]
                // 的 key 是 string-tagged；i32 路径行为不变（位模式解码同旧
                // pop_i32）。
                let index_nv = task.ram.pop_nv();
                // PLAN-026 T-03 配套: i64 索引下标按 i64 数值解码
                // (tag 8)。此前恒 decode_i32 位模式解码——i64 值的位模
                // 式当 i32 = 垃圾下标(evidence/026/probe i64_probe2
                // "c index=-3" 负索引泄漏实证;快照消费循环的 i64 游标
                // 全体受害)。超 i32 域夹取到负哨兵走越界路径。
                let index_i32 = if auto_val::is_i64(index_nv) {
                    auto_val::decode_i64(index_nv).clamp(i32::MIN as i64, i32::MAX as i64) as i32
                } else {
                    auto_val::decode_i32(index_nv)
                };
                // Pop array_id/list_id or str_id (tagged)
                let obj_or_str_nv = task.ram.pop_nv();

                // Plan 550 T04: null 索引对象守卫（现状：null 位模式当
                // obj_id 解码 → 堆查找落空 → 静默 push 0）。Python 风格
                // TypeError，try-catch 可捕获。
                if auto_val::is_null(obj_or_str_nv) {
                    return Err(VMError::RuntimeError(
                        "TypeError: 'NoneType' object is not subscriptable".to_string(),
                    ));
                }

                // Helper function to convert negative index to actual index
                // e.g., for array of length 3: -1 -> 2, -2 -> 1, -3 -> 0
                let normalize_index = |idx: i32, len: usize| -> Option<usize> {
                    if idx >= 0 {
                        let uidx = idx as usize;
                        if uidx < len {
                            Some(uidx)
                        } else {
                            None
                        }
                    } else {
                        // Negative index: -1 means last element, -2 means second-to-last, etc.
                        let from_end = (-idx) as usize;
                        if from_end <= len && from_end > 0 {
                            Some(len - from_end)
                        } else {
                            None
                        }
                    }
                };

                vm_debug!(
                    "DEBUG GET_ELEM: obj_or_str_bits={}, index={}",
                    { auto_val::decode_i32(obj_or_str_nv) },
                    index_i32
                );

                // Check if this is a tagged string index
                let is_string_val = auto_val::is_string(obj_or_str_nv);

                if is_string_val {
                    // This is a tagged string index - string indexing operation
                    let str_idx = auto_val::decode_string(obj_or_str_nv) as usize;
                    let strings = self.strings.read().unwrap();
                    if let Some(bytes) = strings.get(str_idx) {
                        // Get the character at the specified index
                        // Convert bytes to string and get char
                        let s = String::from_utf8_lossy(bytes);
                        let char_count = s.chars().count();
                        if let Some(normalized_idx) = normalize_index(index_i32, char_count) {
                            if let Some(ch) = s.chars().nth(normalized_idx) {
                                vm_debug!("DEBUG GET_ELEM: String[{}] = '{}'", normalized_idx, ch);
                                // Push character as i32 (Unicode code point)
                                task.ram.push_i32(ch as i32);
                            } else {
                                vm_debug!(
                                    "DEBUG GET_ELEM: String index {} out of bounds",
                                    normalized_idx
                                );
                                task.ram.push_i32(0); // Out of bounds
                            }
                        } else {
                            vm_debug!("DEBUG GET_ELEM: String index {} out of bounds", index_i32);
                            task.ram.push_i32(0); // Out of bounds
                        }
                    } else {
                        vm_debug!("DEBUG GET_ELEM: Invalid string index {}", str_idx);
                        task.ram.push_i32(0); // Invalid string index
                    }
                } else {
                    // Regular array/list access
                    let obj_id = if auto_val::is_object(obj_or_str_nv) {
                        auto_val::decode_object(obj_or_str_nv) as u64
                    } else {
                        auto_val::decode_i32(obj_or_str_nv) as u64
                    };

                    // First, try heap_objects registry (Plan 077 unified registry)
                    if let Some(obj) = self.get_heap_object(obj_id) {
                        use crate::vm::types::ListData;
                        let guard = obj.read().unwrap();
                        // PLAN-047 T-04: VM 读臂依赖录制（索引/按键读统一
                        // 粗粒度 `"*"` 面——按键 A-able v1 不展开，保守）。
                        self.record_heap_read_any(obj_id);

                        // Plan 539 W0 (DIV-PY-ITER-1): PyObjectHandle —
                        // GIL obj[index], result marshalled through the
                        // standard py return path (handles stay opaque).
                        #[cfg(feature = "python")]
                        if guard
                            .as_any()
                            .downcast_ref::<crate::py_ffi::PyObjectHandle>()
                            .is_some()
                        {
                            #[cfg(feature = "python")]
                            {
                                #[allow(unused_imports)]
                                use pyo3::prelude::*;
                                let pyh = guard
                                    .as_any()
                                    .downcast_ref::<crate::py_ffi::PyObjectHandle>()
                                    .unwrap();
                                pyo3::Python::attach(|py| -> Result<(), VMError> {
                                    let bound = pyh.obj.clone_ref(py).into_bound(py);
                                    let item = bound.get_item(index_i32).map_err(|e| {
                                        VMError::FFI(format!(
                                            "Python getitem on {} failed: {}",
                                            bound
                                                .get_type()
                                                .name()
                                                .map(|n| n.to_string())
                                                .unwrap_or_default(),
                                            e
                                        ))
                                    })?;
                                    crate::py_ffi::marshal_pyany_to_stack(&item, task, self)?;
                                    Ok(())
                                })?;
                                // Plan 419: receiver(list/数组引用)的栈上 stake 死亡。
                                self.rc_release(obj_or_str_nv);
                                return Ok(StepResult::Continue);
                            }
                        }

                        // Try List<int>
                        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                            vm_debug!(
                                "DEBUG GET_ELEM: Found List<int> with {} elems",
                                list.elems.len()
                            );
                            if let Some(normalized_idx) =
                                normalize_index(index_i32, list.elems.len())
                            {
                                let elem = list.elems[normalized_idx];
                                vm_debug!(
                                    "DEBUG GET_ELEM: Returning elem[{}]={}",
                                    normalized_idx,
                                    elem
                                );
                                // Preserve string tag encoding: negative values are string indices
                                {
                                    if elem < 0 {
                                        let str_idx = (-(elem) - 1) as u32;
                                        self.rc_push_str_idx(task, str_idx as usize);
                                    } else if elem >= 4000000 {
                                        // Heap object ID
                                        // Plan 419: 元素引用入栈 +1。
                                        self.rc_push(task, auto_val::encode_object(elem as u32));
                                    } else {
                                        task.ram.push_i32(elem);
                                    }
                                }
                            } else {
                                // Plan 550 T05: 越界翻转 0 哨兵 → IndexError。
                                return Err(index_out_of_range_error(index_i32));
                            }
                        }
                        // Try List<String>
                        else if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>()
                        {
                            vm_debug!("DEBUG GET_ELEM: Found List<String>");
                            if let Some(normalized_idx) =
                                normalize_index(index_i32, list.elems.len())
                            {
                                let elem = &list.elems[normalized_idx];
                                // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                                // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                                let str_idx = self.add_string(elem.as_bytes().to_vec());
                                self.rc_push_str_idx(task, str_idx);
                            } else {
                                // Plan 550 T05: 越界翻转 0 哨兵 → IndexError。
                                return Err(index_out_of_range_error(index_i32));
                            }
                        }
                        // Try List<bool>
                        else if let Some(list) = guard.as_any().downcast_ref::<ListData<bool>>() {
                            vm_debug!("DEBUG GET_ELEM: Found List<bool>");
                            if let Some(normalized_idx) =
                                normalize_index(index_i32, list.elems.len())
                            {
                                let elem = list.elems[normalized_idx];
                                // Plan 406: push a tagged bool so is_bool consumers
                                // (print/to_string/EQ bit-compare) behave like bool literals.
                                task.ram.push_nv(auto_val::encode_bool(elem));
                            } else {
                                // Plan 550 T05: 越界翻转 0 哨兵 → IndexError。
                                return Err(index_out_of_range_error(index_i32));
                            }
                        }
                        // Try List<Value> (generic list of Values)
                        else if let Some(list) =
                            guard.as_any().downcast_ref::<ListData<auto_val::Value>>()
                        {
                            vm_debug!("DEBUG GET_ELEM: Found List<Value>");
                            if let Some(normalized_idx) =
                                normalize_index(index_i32, list.elems.len())
                            {
                                let elem = &list.elems[normalized_idx];
                                match elem {
                                    auto_val::Value::Str(s) => {
                                        // Plan 423 P5 续修:入池收口 add_string(原裸 strings.push
                                        // 无 pool_state/dedup —— 计数覆盖后 rc=0 可释放,持有者悬垂)。
                                        let str_idx = self.add_string(s.as_bytes().to_vec());
                                        self.rc_push_str_idx(task, str_idx);
                                    }
                                    auto_val::Value::Int(i) => {
                                        task.ram.push_i32(*i);
                                    }
                                    auto_val::Value::Bool(b) => {
                                        task.ram.push_nv(auto_val::encode_bool(*b));
                                    }
                                    auto_val::Value::Float(f) => {
                                        task.ram.push_f32(*f as f32);
                                    }
                                    auto_val::Value::Double(d) => {
                                        task.ram.push_f64(*d);
                                    }
                                    // Plan 419: 元素引用入栈 +1。
                                    auto_val::Value::VmRef(r) => {
                                        self.rc_push(task, auto_val::encode_object(r.id as u32));
                                    }
                                    // PLAN-776 R8-01：Future 句柄元素还原
                                    // TAG_FUTURE 载荷（容器取出保真——
                                    // F776-R8-01 F3；原落 `_` 臂 push_i32(0)）。
                                    auto_val::Value::Future(f) => {
                                        task.ram.push_nv(auto_val::encode_future(f.id as u32));
                                    }
                                    // PLAN-053 P-053-1: 对象元素物化——
                                    // Value::Obj 原落 `_` 臂 push_i32(0)，
                                    // `messages[i].id` 读 0、守卫恒假 →
                                    // computed 产出空列表（musk
                                    // filteredMessages 消息列表恒空）。
                                    // 物化为 ObjectData 堆对象，GET_FIELD
                                    // 按名读字段。
                                    auto_val::Value::Obj(o) => {
                                        let mut od = crate::vm::types::ObjectData::new();
                                        for (k, v) in o.iter() {
                                            od.set(k.clone(), v.clone());
                                        }
                                        let id = self.insert_heap_object(od);
                                        self.rc_push_id(task, id);
                                    }
                                    auto_val::Value::Nil => {
                                        task.ram.push_i32(0);
                                    }
                                    _ => {
                                        task.ram.push_i32(0);
                                    }
                                }
                            } else {
                                // Plan 550 T05: 越界翻转 0 哨兵 → IndexError。
                                return Err(index_out_of_range_error(index_i32));
                            }
                        }
                        // Plan 437: 动态 record 字段访问 —— d[field_name]，
                        // key 为 string-tagged（charts 的 index/categories
                        // 按名取列语义）。镜像 GET_FIELD 的 ObjectData 臂
                        // （含 intern_runtime_str / rc_push 的 RC 契约）。
                        else if let Some(obj) = guard
                            .as_any()
                            .downcast_ref::<crate::vm::types::ObjectData>()
                        {
                            if auto_val::is_string(index_nv) {
                                let key_pool_idx = auto_val::decode_string(index_nv) as usize;
                                let field_name = self
                                    .strings
                                    .read()
                                    .unwrap()
                                    .get(key_pool_idx)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default();
                                let value = obj
                                    .get(&auto_val::ValueKey::Str(field_name.into()))
                                    .cloned();
                                match value {
                                    Some(auto_val::Value::Int(i)) => {
                                        task.ram.push_i32(i);
                                    }
                                    Some(auto_val::Value::Uint(u)) => {
                                        task.ram.push_i32(u as i32);
                                    }
                                    Some(auto_val::Value::Float(f)) => {
                                        task.ram.push_f32(f as f32);
                                    }
                                    Some(auto_val::Value::Double(d)) => {
                                        task.ram.push_f64(d);
                                    }
                                    Some(auto_val::Value::Bool(b)) => {
                                        task.ram.push_nv(auto_val::encode_bool(b));
                                    }
                                    Some(auto_val::Value::Char(c)) => {
                                        task.ram.push_i32(c as i32);
                                    }
                                    Some(auto_val::Value::Str(s)) => {
                                        self.intern_runtime_str(task, s.as_bytes().to_vec());
                                    }
                                    Some(auto_val::Value::VmRef(r)) => {
                                        self.rc_push(task, auto_val::encode_object(r.id as u32));
                                    }
                                    _ => {
                                        task.ram.push_i32(0);
                                    } // Nil / 缺字段
                                }
                            } else {
                                task.ram.push_i32(0);
                            }
                        }
                        // Plan 445 M3: GenericInstanceData 同语义 —— vm_bridge
                        // 把 model 字面量物化为 GenericInstanceData
                        // (eval_expr_to_value 的 Object 臂)，UI 运行时
                        // (`auto run -r vm`) 的 d[field_name] 全落此表示
                        // (024-charts 实机探针：落 Unknown → push 0)。
                        // 按名查 field_names → fields；Value 分发镜像
                        // ObjectData 臂（RC 契约一致）。
                        else if let Some(inst) = guard
                            .as_any()
                            .downcast_ref::<crate::vm::generic_registry::GenericInstanceData>(
                        ) {
                            if auto_val::is_string(index_nv) {
                                let key_pool_idx = auto_val::decode_string(index_nv) as usize;
                                let field_name = self
                                    .strings
                                    .read()
                                    .unwrap()
                                    .get(key_pool_idx)
                                    .map(|b| String::from_utf8_lossy(b).to_string())
                                    .unwrap_or_default();
                                let fidx = inst.field_names.iter().position(|n| *n == field_name);
                                let value = fidx.and_then(|i| inst.get_field(i)).cloned();
                                match value {
                                    Some(auto_val::Value::Int(i)) => {
                                        task.ram.push_i32(i);
                                    }
                                    Some(auto_val::Value::Uint(u)) => {
                                        task.ram.push_i32(u as i32);
                                    }
                                    Some(auto_val::Value::Float(f)) => {
                                        task.ram.push_f32(f as f32);
                                    }
                                    Some(auto_val::Value::Double(d)) => {
                                        task.ram.push_f64(d);
                                    }
                                    Some(auto_val::Value::Bool(b)) => {
                                        task.ram.push_nv(auto_val::encode_bool(b));
                                    }
                                    Some(auto_val::Value::Char(c)) => {
                                        task.ram.push_i32(c as i32);
                                    }
                                    Some(auto_val::Value::Str(s)) => {
                                        self.intern_runtime_str(task, s.as_bytes().to_vec());
                                    }
                                    Some(auto_val::Value::VmRef(r)) => {
                                        self.rc_push(task, auto_val::encode_object(r.id as u32));
                                    }
                                    _ => {
                                        task.ram.push_i32(0);
                                    } // Nil / 缺字段
                                }
                            } else {
                                task.ram.push_i32(0);
                            }
                        } else {
                            vm_debug!(
                                "DEBUG GET_ELEM: Unknown heap object type: {:?}",
                                guard.type_tag()
                            );
                            task.ram.push_i32(0); // Unknown heap object type
                        }
                    } else {
                        // Object not found - push 0 as error sentinel
                        // TODO: Proper error handling for invalid object IDs
                        task.ram.push_i32(0);
                    }
                } // end of else block for non-string case
                  // Plan 419: receiver(list/数组引用)的栈上 stake 死亡
                  // (字符串 tag 非堆引用,release 为 no-op)。
                self.rc_release(obj_or_str_nv);
            }
            // Plan 073: Array element assignment (arr[index] = value)
            OpCode::SET_ELEM => {
                use crate::vm::generic_registry::GenericInstanceData;
                // Stack: value, array_id, index (compiled in this order by codegen)
                // PLAN-661 T-01: keep the raw index nanbox — map bracket writes
                // (`m[k] = v`) carry a string-tagged key, mirroring GET_ELEM's
                // Plan 437/445 read arms. The list path decodes i32 as before.
                let index_nv = task.ram.pop_nv();
                let _ = task.ram.take_stake_at(task.ram.sp);
                // Pop array_id. arrays still push raw i32 (H3 will migrate),
                // but accept TAG_OBJECT too in case a heap List id reaches here.
                // Plan 419: receiver stake 在 arm 末尾释放;value 自栈转移
                // 进元素;被覆盖的旧元素为引用时释放。
                let receiver_nv;
                let array_id = {
                    let nv = task.ram.pop_nv();
                    let _ = task.ram.take_stake_at(task.ram.sp);
                    receiver_nv = nv;
                    if auto_val::is_object(nv) {
                        auto_val::decode_object(nv) as u64
                    } else {
                        auto_val::decode_i32(nv) as u64
                    }
                };
                // Pop value (bottom of stack) — tagged form. Plan 502 M3:
                // 此前 pop_i32 + Value::Int 丢弃 float/bool/str 元素标签,
                // float 位模式被当整数存入(f"84.0" 写入读回 1118306300
                // 实锤)——与 SET_FIELD 的 decode_tagged_nv 同型修复。
                let value_nv = task.ram.pop_nv();
                let _ = task.ram.take_stake_at(task.ram.sp);
                let value = self.decode_tagged_nv(value_nv);
                let mut old_elem_ref: Option<u64> = None;

                // Get array from heap_objects (Plan 390 §15 H3b: arrays are
                // ListData<Value> in the unified registry now).
                if let Some(list_ref) = self.get_heap_object(array_id) {
                    let mut guard = list_ref.write().unwrap();
                    if let Some(list) = guard
                        .as_any_mut()
                        .downcast_mut::<crate::vm::types::ListData<auto_val::Value>>()
                    {
                        let index = auto_val::decode_i32(index_nv);
                        // Check bounds
                        if index >= 0 && (index as usize) < list.elems.len() {
                            // PLAN-047 T-03: SET_ELEM 列表臂补 bump——普查
                            // 发现该写面原无全局 seq bump（PLAN-062 遗漏，
                            // memo 快路径陈旧命中窗口）；B 类定点归因。
                            self.bump_path(array_id, None);
                            // Plan 419: 记录旧元素引用,写锁释放后级联回收。
                            old_elem_ref = match &list.elems[index as usize] {
                                auto_val::Value::VmRef(r) => Some(r.id as u64),
                                _ => None,
                            };
                            list.elems[index as usize] = value;
                        } else {
                            // Plan 118: Return error for out-of-bounds assignment
                            return Err(VMError::RuntimeError(format!(
                                "Array index {} out of bounds (array length: {})",
                                index,
                                list.elems.len()
                            )));
                        }
                    }
                    // PLAN-661 T-01: map 括号写 —— 读侧 m[k] 走 GET_ELEM 的
                    // ObjectData（Plan 437）/GenericInstanceData（Plan 445 M3）
                    // 臂，写侧 SET_ELEM 此前只有 ListData 臂，map 目标落
                    // "Invalid array ID" 中止 handler（078 复现包实录）。
                    // 三臂按名写，与 GET_ELEM 读臂一一镜像。
                    else if let Some(obj) = guard
                        .as_any_mut()
                        .downcast_mut::<crate::vm::types::ObjectData>()
                    {
                        if !auto_val::is_string(index_nv) {
                            return Err(VMError::RuntimeError(
                                "TypeError: map key must be a string".to_string(),
                            ));
                        }
                        // PLAN-062: map 写算状态面突变（与 SET_FIELD 同口径）。
                        // PLAN-047 T-03: 定点归因（按键名 A-able，v1 落
                        // exact+wildcard——bump_path 通配同 bump）。
                        let key_pool_idx = auto_val::decode_string(index_nv) as usize;
                        let key_name = self
                            .strings
                            .read()
                            .unwrap()
                            .get(key_pool_idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .ok_or_else(|| {
                                VMError::RuntimeError(format!(
                                    "Invalid key string index: {}",
                                    key_pool_idx
                                ))
                            })?;
                        self.bump_path(array_id, Some(&key_name));
                        // 键语义镜像 SET_FIELD ObjectData 分支（PLAN-057）：
                        // 开放 HashMap，set=insert（JS obj.newKey = v 语义）。
                        let key = auto_val::ValueKey::Str(key_name.into());
                        old_elem_ref = obj.get(&key).and_then(|v| match v {
                            auto_val::Value::VmRef(r) => Some(r.id as u64),
                            _ => None,
                        });
                        obj.set(key, value);
                    } else if let Some(inst) =
                        guard.as_any_mut().downcast_mut::<GenericInstanceData>()
                    {
                        if !auto_val::is_string(index_nv) {
                            return Err(VMError::RuntimeError(
                                "TypeError: map key must be a string".to_string(),
                            ));
                        }
                        let key_pool_idx = auto_val::decode_string(index_nv) as usize;
                        let key_name = self
                            .strings
                            .read()
                            .unwrap()
                            .get(key_pool_idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .ok_or_else(|| {
                                VMError::RuntimeError(format!(
                                    "Invalid key string index: {}",
                                    key_pool_idx
                                ))
                            })?;
                        // PLAN-047 T-03: 定点归因（同 ObjectData 臂）。
                        self.bump_path(array_id, Some(&key_name));
                        // UI 轨 state map（StateObjectLit）表示。GET_ELEM 读臂
                        // 按 field_names 名取，故新键平行追加 fields/field_names
                        // （开键插入，镜像 PLAN-057 语义）；既有键走 set_field。
                        match inst.field_names.iter().position(|n| *n == key_name) {
                            Some(idx) => {
                                old_elem_ref = inst.fields.get(idx).and_then(|v| match v {
                                    auto_val::Value::VmRef(r) => Some(r.id as u64),
                                    _ => None,
                                });
                                inst.set_field(idx, value).map_err(VMError::RuntimeError)?;
                            }
                            None => {
                                inst.field_names.push(key_name);
                                inst.fields.push(value);
                            }
                        }
                    }
                    // auto.hashmap.new 建的 map：同 auto.hashmap.set native
                    // （NATIVE_HASHMAP_INSERT_STR）的插入语义。
                    else if let Some(map) = guard
                        .as_any_mut()
                        .downcast_mut::<crate::vm::collections::SpecializedHashMap>(
                    ) {
                        if !auto_val::is_string(index_nv) {
                            return Err(VMError::RuntimeError(
                                "TypeError: map key must be a string".to_string(),
                            ));
                        }
                        let key_pool_idx = auto_val::decode_string(index_nv) as usize;
                        let key_name = self
                            .strings
                            .read()
                            .unwrap()
                            .get(key_pool_idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .ok_or_else(|| {
                                VMError::RuntimeError(format!(
                                    "Invalid key string index: {}",
                                    key_pool_idx
                                ))
                            })?;
                        // PLAN-047 T-03: 定点归因（同上两臂）。
                        self.bump_path(array_id, Some(&key_name));
                        if let Some(auto_val::Value::VmRef(r)) = map.get(&key_name) {
                            old_elem_ref = Some(r.id as u64);
                        }
                        map.insert(key_name, value).map_err(VMError::RuntimeError)?;
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid array ID: {}",
                            array_id
                        )));
                    }
                } else {
                    // Plan 118: Return error for invalid array IDs
                    return Err(VMError::RuntimeError(format!(
                        "Invalid array ID: {}",
                        array_id
                    )));
                }
                // Plan 419: 旧元素级联释放 + receiver stake 死亡。
                if let Some(old_id) = old_elem_ref {
                    self.rc_release_id(old_id);
                }
                self.rc_release(receiver_nv);
            }
            // Plan 075: Object field assignment (obj.field = value)
            OpCode::SET_FIELD => {
                // PLAN-062: 字段写算状态面突变。PLAN-047 T-03: bump 移至
                // obj_id/field_name 解码后定点归因（A 类——exact+wildcard
                // 双 bump；错误路径此后不再多 bump，handler Err 本身置脏
                // 帧面，方向不变）。
                use crate::vm::generic_registry::GenericInstanceData;
                // Stack: value, object_id, field_name_idx (compiled in this order by codegen)
                // Pop field_name_idx first (top of stack)
                let field_idx = {
                    let nv = task.ram.pop_nv();
                    let _ = task.ram.take_stake_at(task.ram.sp);
                    if auto_val::is_string(nv) {
                        auto_val::decode_string(nv) as usize
                    } else {
                        auto_val::decode_i32(nv) as usize
                    }
                };
                // Pop object_id
                // Plan 419: receiver 的栈上 stake 在 arm 末尾释放(错误路径
                // 泄漏一个 stake —— 安全方向);value 自栈转移进字段(不变);
                // 旧字段值为 VmRef 时在写入后释放(级联回收可能触发)。
                let receiver_nv;
                let obj_id = {
                    let nv = task.ram.pop_nv();
                    let _ = task.ram.take_stake_at(task.ram.sp);
                    receiver_nv = nv;
                    if auto_val::is_i32(nv) {
                        auto_val::decode_i32(nv) as u64
                    } else if auto_val::is_object(nv) {
                        auto_val::decode_object(nv) as u64
                    } else {
                        auto_val::decode_i32(nv) as u64
                    }
                };
                // Pop value (bottom of stack)
                let value_nv = task.ram.pop_nv();
                let _ = task.ram.take_stake_at(task.ram.sp);
                let mut old_field_ref: Option<u64> = None;

                // Get field name from strings pool
                let strings = self.strings.read().unwrap();
                let field_name = if let Some(field_bytes) = strings.get(field_idx) {
                    String::from_utf8_lossy(field_bytes).to_string()
                } else {
                    return Err(VMError::RuntimeError(format!(
                        "Invalid string index: {}",
                        field_idx
                    )));
                };
                drop(strings); // Release lock before writing

                // PLAN-093 G-9 诊断：字段写去向（对象身份 vs 状态对象）。
                // 内容指纹：写前对象里的 phase/polls 现值——若 pre-park 与
                // post-resume 写面向同一对象，序列单调；若对象被换，指纹断裂。
                if std::env::var_os("AUTO_DEBUG_G9").is_some()
                    && matches!(
                        field_name.as_str(),
                        "phase" | "polls" | "r_start" | "r_source"
                    )
                {
                    let fp = self
                        .heap_objects
                        .get(&obj_id)
                        .and_then(|a| {
                            let g = a.read().unwrap();
                            g.as_any()
                                .downcast_ref::<crate::vm::generic_registry::GenericInstanceData>()
                                .map(|inst| {
                                    let f = |name: &str| -> String {
                                        inst.field_names
                                            .iter()
                                            .position(|n| n == name)
                                            .and_then(|i| inst.get_field(i))
                                            .map(|v| format!("{:?}", v))
                                            .unwrap_or_else(|| "<none>".into())
                                    };
                                    format!(
                                        "phase={} polls={} r_start={}",
                                        f("phase"),
                                        f("polls"),
                                        f("r_start")
                                    )
                                })
                        })
                        .unwrap_or_else(|| "<obj-missing>".into());
                    eprintln!(
                        "[G9] SET_FIELD {} -> obj_id={} BEFORE[{}] (ip={:#x} sp={} bp={})",
                        field_name, obj_id, fp, task.ip, task.ram.sp, task.bp
                    );
                }

                // PLAN-047 T-03: A 类定点归因（带名字段写）。
                self.bump_path(obj_id, Some(&field_name));

                // Get object from registry (Plan 390 §15 H3b: ObjectData /
                // GenericInstance / RustStdlib all in heap_objects).
                if let Some(heap_ref) = self.heap_objects.get(&obj_id) {
                    let mut heap_obj = heap_ref.write().unwrap();
                    if let Some(obj) = heap_obj
                        .as_any_mut()
                        .downcast_mut::<crate::vm::types::ObjectData>()
                    {
                        // Try multiple key formats: string, integer, boolean (same as GET_FIELD)
                        let key = if obj
                            .get(&auto_val::ValueKey::Str(field_name.clone().into()))
                            .is_some()
                        {
                            auto_val::ValueKey::Str(field_name.into())
                        } else if let Ok(int_key) = field_name.parse::<i32>() {
                            if obj.get(&auto_val::ValueKey::Int(int_key)).is_some() {
                                auto_val::ValueKey::Int(int_key)
                            } else {
                                // Plan 118: Integer field not found - return error
                                return Err(VMError::RuntimeError(format!(
                                    "Field '{}' not found on object",
                                    field_name
                                )));
                            }
                        } else if field_name == "true" {
                            if obj.get(&auto_val::ValueKey::Bool(true)).is_some() {
                                auto_val::ValueKey::Bool(true)
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Field '{}' not found on object",
                                    field_name
                                )));
                            }
                        } else if field_name == "false" {
                            if obj.get(&auto_val::ValueKey::Bool(false)).is_some() {
                                auto_val::ValueKey::Bool(false)
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Field '{}' not found on object",
                                    field_name
                                )));
                            }
                        } else {
                            // PLAN-057（等价性缺陷族①）：ObjectData 未见过的
                            // Str 键改为插入（JS `obj.newKey = v` 赋值语义；
                            // ObjectData 底层是开放 HashMap，set=insert）。
                            // Plan 118 的类型严格性仍由 Int/Bool 键格式分支
                            // （上方）与 GenericInstanceData 分支（下方）持有。
                            auto_val::ValueKey::Str(field_name.clone().into())
                        };
                        old_field_ref = obj.get(&key).and_then(|v| match v {
                            auto_val::Value::VmRef(r) => Some(r.id as u64),
                            _ => None,
                        });
                        obj.set(key, { self.decode_tagged_nv(value_nv) });
                    } else if let Some(inst) =
                        heap_obj.as_any_mut().downcast_mut::<GenericInstanceData>()
                    {
                        // PLAN-536 Phase 2 T13: PollStream 兜底链追踪面——
                        // 环境门控(AUTO_DEBUG_POLLTRACE=1),观察跨模块调用帧
                        // 内对 store 字段(合并根态 GenericInstanceData)的
                        // SET_FIELD 实际落点与值(KD P536-D2 定罪)。
                        if std::env::var("AUTO_DEBUG_POLLTRACE").is_ok()
                            && matches!(
                                field_name.as_str(),
                                "streaming" | "pre_stream_len" | "messages" | "poll_window"
                            )
                        {
                            let decoded = self.decode_tagged_nv(value_nv);
                            let vdesc = match &decoded {
                                auto_val::Value::Bool(b) => format!("bool={b}"),
                                auto_val::Value::Int(i) => format!("int={i}"),
                                auto_val::Value::Str(_) => "str".to_string(),
                                auto_val::Value::Nil => "nil".to_string(),
                                auto_val::Value::VmRef(r) => {
                                    let len = self
                                            .heap_objects
                                            .get(&(r.id as u64))
                                            .map(|h| {
                                                let g = h.read().unwrap();
                                                g.as_any()
                                                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                                                    .map(|l| l.len())
                                                    .unwrap_or(usize::MAX)
                                            })
                                            .unwrap_or(usize::MAX);
                                    format!("vmref={} len={len}", r.id)
                                }
                                other => format!("nv({:?})", std::mem::discriminant(other)),
                            };
                            eprintln!(
                                "[POLLTRACE][SET] obj={obj_id} .{field_name} = {vdesc} (ip=0x{:x})",
                                task.ip
                            );
                        }
                        let field_idx = inst.field_names.iter().position(|n| n == &field_name);
                        if let Some(idx) = field_idx {
                            old_field_ref = inst.fields.get(idx).and_then(|v| match v {
                                auto_val::Value::VmRef(r) => Some(r.id as u64),
                                _ => None,
                            });
                            inst.set_field(idx, { self.decode_tagged_nv(value_nv) })
                                .map_err(|e| VMError::RuntimeError(e))?;
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Field '{}' not found on type instance {}",
                                field_name, inst.mono_name
                            )));
                        }
                    } else if let Some(rust_obj) = heap_obj
                        .as_any()
                        .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>(
                    ) {
                        // Opaque external crate type — try field mutation
                        let type_name = rust_obj.type_name.clone();
                        drop(heap_obj); // release write lock
                                        // Handle semver field mutation
                        let mut field_set = false;
                        if type_name == "semver::Version" {
                            let new_val = auto_val::decode_i32(value_nv);
                            if let Some(obj) = self.heap_objects.get(&obj_id) {
                                let mut guard = obj.write().unwrap();
                                if let Some(rso) = guard
                                    .as_any_mut()
                                    .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>(
                                ) {
                                    if let Some(ver) =
                                        rso.downcast_mut::<std::sync::Mutex<semver::Version>>()
                                    {
                                        let mut v = ver.lock().unwrap();
                                        match field_name.as_str() {
                                            "major" => v.major = new_val as u64,
                                            "minor" => v.minor = new_val as u64,
                                            "patch" => v.patch = new_val as u64,
                                            _ => {}
                                        }
                                        field_set = true;
                                    }
                                }
                            }
                        }
                        if !field_set {
                            return Err(VMError::RuntimeError(format!(
                                "Cannot set field '{}' on opaque type {}",
                                field_name, type_name
                            )));
                        }
                        // Field set successfully — fall through to next instruction
                    } else if let Some(dep) = heap_obj
                        .as_any()
                        .downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>(
                    ) {
                        // PLAN-591 T1(V1-2): dep 对象标量字段 offset 直写——
                        // 写穿透到 cdylib 堆本体(Rust 侧方法读回可见,V1-2)。
                        // V1 纪律:单线程批处理执行;标量面仅此(String/嵌套
                        // 句柄不在写面);别名/写后 Rust 侧缓存失效语义登记
                        // KNOWN-DEBT(待澄清 #1 裁定)。写经裸指针穿透,守卫
                        // 只护句柄元数据,持锁写与释放后写等价,无重入。
                        crate::vm::ffi::dep_methods::write_scalar_field(
                            self,
                            dep,
                            field_name.as_str(),
                            value_nv,
                        )?;
                        // Field set successfully — fall through to next instruction
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid object ID: {}",
                            obj_id
                        )));
                    }
                } else {
                    // Plan 118: Return error for invalid object IDs
                    return Err(VMError::RuntimeError(format!(
                        "Invalid object ID: {}",
                        obj_id
                    )));
                }
                // Plan 419: 写锁已释放 —— 旧字段值级联释放 + receiver stake 死亡。
                if let Some(old_id) = old_field_ref {
                    self.rc_release_id(old_id);
                }
                self.rc_release(receiver_nv);
            }
            _ => return self.ops_arith(task, op, op_byte),
        }
        Ok(StepResult::Continue)
    }
}
