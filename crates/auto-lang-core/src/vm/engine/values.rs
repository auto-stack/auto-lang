//! PLAN-843 结构拆分（自 engine.rs 原样搬移；算法/行为零变化）。
use super::*;

impl AutoVM {
    /// Decode a tagged value from the VM stack.
    /// Negative values encode string indices (tag = -(idx+1)), positive values are plain ints.
    fn decode_tagged_value(&self, raw: i32) -> auto_val::Value {
        if raw < 0 {
            let str_idx = (-raw - 1) as usize;
            let strings = self.strings.read().unwrap();
            if let Some(bytes) = strings.get(str_idx) {
                return auto_val::Value::Str(String::from_utf8_lossy(bytes).to_string().into());
            }
        }
        auto_val::Value::Int(raw)
    }

    /// Decode a tagged NanoValue into a Value (nanbox mode).
    pub(super) fn decode_tagged_nv(&self, nv: auto_val::NanoValue) -> auto_val::Value {
        if auto_val::is_string(nv) {
            let str_idx = auto_val::decode_string(nv) as usize;
            let strings = self.strings.read().unwrap();
            if let Some(bytes) = strings.get(str_idx) {
                let copy = auto_val::Value::Str(String::from_utf8_lossy(bytes).to_string().into());
                drop(strings);
                // Plan 419 Phase 2: 字段持有字节拷贝 —— 池引用的 stake 随转移死亡。
                self.pool_release(str_idx);
                return copy;
            }
        } else if auto_val::is_future(nv) {
            // PLAN-776 R8-01：Future 句柄入 Value 载体（容器/捕获 env/异步
            // 终值）——**保真映射** `Value::Future(FutureData{id: fid})`（句柄
            // 视图，state 占位 Pending；真实状态在 vm.futures）。R7 的降级
            // Int(bits) 使 marshal 无法区分真实 Future 与同值普通数字
            //（F776-R8-01 F2/F3），已废弃。
            return auto_val::Value::Future(Box::new(auto_val::FutureData {
                id: auto_val::decode_future(nv) as usize,
                state: auto_val::FutureState::Pending,
                result: None,
            }));
        } else if auto_val::is_i32(nv) {
            // Plan 390 §15 H3b: all heap refs are TAG_OBJECT now (objects/
            // arrays/nodes produce encode_object), so a raw i32 is always a
            // plain scalar Int — the old `>= 4000000 → VmRef` heuristic is gone.
            let i = auto_val::decode_i32(nv);
            return auto_val::Value::Int(i);
        } else if auto_val::is_bool(nv) {
            return auto_val::Value::Bool(auto_val::decode_bool(nv));
        } else if auto_val::is_f32(nv) {
            return auto_val::Value::Float(auto_val::decode_f32(nv) as f64);
        } else if auto_val::is_f64(nv) {
            return auto_val::Value::Double(auto_val::decode_f64(nv));
        } else if auto_val::is_null(nv) {
            // Plan 474 待澄清#4: null nv 原落 _ => Value::Int(0) 兜底——
            // SET_FIELD 存 null 值字段失真（json null / __json_object 缺键
            // 读回的 encode_null 再写字段会变成 Int(0)）。归一到 VM 侧
            // null 约定 Value::Nil（json_to_vm_value 的 Null 臂同款）。
            return auto_val::Value::Nil;
        } else if auto_val::is_object(nv) {
            return auto_val::Value::VmRef(auto_val::VmRef {
                id: auto_val::decode_object(nv) as usize,
            });
        } else if auto_val::is_list(nv) {
            return auto_val::Value::VmRef(auto_val::VmRef {
                id: auto_val::decode_list(nv) as usize,
            });
        } else if auto_val::is_null(nv) {
            // PLAN-050 T9: tag-null 字段赋值此前落兜底 Int(0)——`x = f()` 而 f
            // return None 时 `.x != None` 恒真、`.x.field` 落空（musk
            // WorkspaceSelector rail 底部裸 "${currentName}" 根因）。
            return auto_val::Value::Nil;
        }
        auto_val::Value::Int(0)
    }

    /// Plan 375: Inject an `auto_val::Value` into this VM, returning the
    /// `NanoValue` that represents it on the data stack / in `globals`.
    ///
    /// Scalars are encoded directly. Compound values (Array / Obj / Node) are
    /// recursively registered into the VM heap registries (`arrays` / `objects`
    /// / `nodes`) and represented by their tagged id — mirroring the reverse
    /// `extract_value_from_vm`. This lets mold templates read globals set via
    /// `AutoInterpreter::set_global` / `merge_atom` (e.g. the IAR `.eww`
    /// template's `@ for app in apps { ... }`).
    pub fn inject_value(&mut self, value: &auto_val::Value) -> auto_val::NanoValue {
        use auto_val::Value;
        use std::sync::atomic::Ordering;
        match value {
            // --- scalars ---
            Value::Int(i) => auto_val::encode_i32(*i),
            Value::Uint(u) => auto_val::encode_i32(*u as i32),
            Value::I64(i) => auto_val::encode_i32(*i as i32),
            Value::USize(u) => auto_val::encode_i32(*u as i32),
            Value::Bool(b) => auto_val::encode_bool(*b),
            Value::Nil | Value::Null => auto_val::encode_null(),
            Value::Float(f) => auto_val::encode_f64(*f),
            Value::Double(d) => auto_val::encode_f64(*d),
            Value::Byte(b) => auto_val::encode_i32(*b as i32),
            Value::Char(c) => auto_val::encode_i32(*c as i32),
            Value::Str(s) => {
                // Intern into the string pool and return its tagged index.
                let idx = self.add_string(s.as_bytes().to_vec());
                auto_val::encode_string(idx as u32)
            }
            Value::VmRef(r) => auto_val::encode_object(r.id as u32),
            // PLAN-776 R8-01：Future 句柄 Value 载体还原为 TAG_FUTURE 载荷
            //（容器/捕获往返保真；普通 Int 恒数据）。
            Value::Future(f) => auto_val::encode_future(f.id as u32),

            // --- Array: register into heap_objects as ListData<Value> (H3b) ---
            Value::Array(arr) => {
                let mut elems: Vec<auto_val::Value> = Vec::with_capacity(arr.values.len());
                for v in &arr.values {
                    elems.push(self.materialize_value(v));
                }
                let id = self.insert_heap_object(crate::vm::types::ListData {
                    elems,
                    storage: None,
                });
                // Plan 390 §15 H3b: array ids are TAG_OBJECT-encoded (same as
                // CREATE_ARRAY now) — consumers distinguish via is_object.
                auto_val::encode_object(id as u32)
            }

            // --- Obj: register into heap_objects as ObjectData (H3b) ---
            Value::Obj(obj) => {
                let mut data = crate::vm::types::ObjectData::new();
                for (key, val) in obj.iter() {
                    data.fields.insert(key.clone(), self.materialize_value(val));
                }
                // Plan 390 §15 H3b: objects live in heap_objects (ObjectData
                // impl HeapObject, H1). Stack encoding stays TAG_OBJECT so
                // consumers distinguish it from scalar i32 via is_object.
                let id = self.insert_heap_object(data);
                auto_val::encode_object(id as u32)
            }

            // --- Node: register into heap_objects (Plan 390 §15 H3a) ---
            Value::Node(node) => {
                let mut cloned = (**node).clone();
                // Recurse into props so nested VmRefs point at real heap entries.
                let props = cloned.props_clone();
                for (key, val) in props.iter() {
                    cloned.set_prop(key.clone(), self.materialize_value(val));
                }
                let id = self.insert_heap_object(cloned);
                auto_val::encode_object(id as u32)
            }

            _ => auto_val::encode_null(),
        }
    }

    /// Plan 375: helper used by `inject_value`. For compound element values it
    /// injects them into the registries and records the resulting id as a
    /// `Value::VmRef` (so the array/object stores carry heap references, just
    /// like values produced at runtime). Scalars are returned as-is.
    fn materialize_value(&mut self, val: &auto_val::Value) -> auto_val::Value {
        match val {
            auto_val::Value::Array(_) | auto_val::Value::Obj(_) | auto_val::Value::Node(_) => {
                let nv = self.inject_value(val);
                // Plan 390 §15 H2: Obj encodes its id as TAG_OBJECT while
                // Node has migrated to encode_object as well (H3a); Array
                // still uses raw i32 (its storage migration is H3b).
                // Decode whichever tag inject_value produced — the low-32-bit
                // payload is the registry id in both forms.
                let id = if auto_val::is_object(nv) {
                    auto_val::decode_object(nv) as usize
                } else {
                    auto_val::decode_i32(nv) as usize
                };
                auto_val::Value::VmRef(auto_val::VmRef { id })
            }
            auto_val::Value::VmRef(r) => auto_val::Value::VmRef(r.clone()),
            other => other.clone(),
        }
    }

    /// 压入 i64：48 位内联，否则 BigInt 堆装箱（完整 64 位范围）。
    #[inline(always)]
    pub fn push_i64_vm(&self, task: &mut AutoTask, val: i64) {
        task.ram
            .push_nv(crate::vm::ffi::encode_i64_with_heap(self, val));
    }

    /// 压入 u64：48 位内联，否则 BigInt 堆装箱（完整 64 位范围）。
    #[inline(always)]
    pub fn push_u64_vm(&self, task: &mut AutoTask, val: u64) {
        task.ram
            .push_nv(crate::vm::ffi::encode_u64_with_heap(self, val));
    }

    /// 弹出 i64：TAG_I64/U64 内联解码，TAG_BIGINT 解引用堆对象（完整 64 位范围）。
    #[inline(always)]
    pub fn pop_i64_vm(&self, task: &mut AutoTask) -> i64 {
        let nv = task.ram.pop_nv();
        crate::vm::ffi::decode_i64_full(self, nv)
    }

    /// 弹出 u64：TAG_U64/I64 内联解码，TAG_BIGINT 解引用堆对象（完整 64 位范围）。
    #[inline(always)]
    pub fn pop_u64_vm(&self, task: &mut AutoTask) -> u64 {
        let nv = task.ram.pop_nv();
        crate::vm::ffi::decode_u64_full(self, nv)
    }

    /// Push a Value onto the stack based on its type
    ///
    /// For Phase 2, supports: Int, Uint, Float, Double, Bool, Char, Nil, Str
    pub(super) fn push_value(task: &mut AutoTask, value: &Value, vm: &AutoVM) {
        let ram = &mut task.ram;
        match value {
            Value::Int(i) => {
                ram.push_nv(auto_val::encode_i32(*i));
            }
            Value::Uint(u) => {
                ram.push_nv(auto_val::encode_i32(*u as i32));
            }
            Value::Float(f) => ram.push_f32(*f as f32),
            Value::Double(d) => ram.push_f64(*d),
            Value::Bool(b) => {
                ram.push_nv(auto_val::encode_bool(*b));
            }
            Value::Char(c) => {
                ram.push_nv(auto_val::encode_i32(*c as i32));
            }
            Value::Nil => {
                ram.push_nv(auto_val::encode_null());
            }
            Value::Str(s) => {
                // Store string in constant pool and push its tagged index
                // String indices are stored as -(index+1) to distinguish from integers
                // 2026-08-22(侧栏串写根因):Value::Str 字段每次被读取都会走到
                // 这里 —— 直推无去重曾令池随每次视图重建膨胀(81 命令 × name/
                // description,每敲一键全量重建),越过 u16 上限后索引回绕互串。
                // 改走 add_string(内容去重):重复内容零增长。
                // 2026-08-26(plan447-E2):入池条目 rc=0,freelist 复用注释明言
                // "由 push 侧 +1 建立 stake"——原直推漏 retain,GET_GENERIC_FIELD
                // 读运行期字符串载荷后任意一次 pop/槽释放即超额释放(canary;
                // 此前全库载荷均为 pinned 常量故未暴露)。统一走 rc_push_str_idx。
                let idx = vm.add_string(s.as_bytes().to_vec());
                vm.rc_push_str_idx(task, idx);
            }
            Value::VmRef(vmref) => {
                // Plan 419: 堆引用入栈 +1(咽喉点)。PLAN-062 T12: 记影子。
                vm.rc_retain_id(vmref.id as u64);
                ram.push_nv(auto_val::encode_object(vmref.id as u32));
                ram.mark_top_stake(vmref.id as u64);
            }
            // PLAN-776 R9-02：Future 句柄还原 TAG_FUTURE（原落 WARNING+0）。
            Value::Future(f) => {
                ram.push_nv(auto_val::encode_future(f.id as u32));
            }
            _ => {
                eprintln!("WARNING: push_value unsupported type: {:?}", value);
                ram.push_nv(auto_val::encode_i32(0));
            }
        }
    }

    /// Plan 364 Step 5: Pop the top of the data stack as an `auto_val::Value`,
    /// resolving object/node references into the actual Obj/Node.
    ///
    /// Used by the Config accumulation opcodes (ACCUM_PAIR/ACCUM_NODE/
    /// ACCUM_MERGE). Mirrors the resolution in `extract_value_from_vm`
    /// (lib.rs): negative bits = tagged string index, positive bits = object
    /// id / node id (distinguished by registry membership) or plain int.
    ///
    /// Note: unlike `extract_value_from_vm` this does NOT recurse into nested
    /// refs (no `visited` cycle guard) — accumulation only inspects the top
    /// value; its nested VmRefs stay as-is in the Obj/Node clone and are
    /// resolved later by the result materializer.
    pub(super) fn pop_auto_value(&self, task: &mut AutoTask) -> auto_val::Value {
        use auto_val::{
            decode_bool, decode_f32, decode_f64, decode_i32, decode_list, decode_object,
            decode_string, is_bool, is_f32, is_f64, is_list, is_null, is_object, is_string,
        };
        let nv = task.ram.pop_nv();
        // Non-NaN-boxed values are raw f64.
        if is_f64(nv) {
            return auto_val::Value::Float(decode_f64(nv));
        }
        if is_string(nv) {
            let str_idx = decode_string(nv) as usize;
            let strings = self.strings.read().unwrap();
            return if let Some(bytes) = strings.get(str_idx) {
                auto_val::Value::Str(String::from_utf8_lossy(bytes).to_string().into())
            } else {
                auto_val::Value::Nil
            };
        }
        if is_bool(nv) {
            return auto_val::Value::Bool(decode_bool(nv));
        }
        if is_null(nv) {
            return auto_val::Value::Nil;
        }
        if is_f32(nv) {
            return auto_val::Value::Float(decode_f32(nv) as f64);
        }
        // Object reference: ObjectData/Node/ListData all live in heap_objects
        // (Plan 390 §15 H3b) — probe + downcast instead of registry-membership
        // checks.
        if is_object(nv) {
            let id = decode_object(nv) as u64;
            if let Some(obj_ref) = self.get_heap_object(id) {
                let guard = obj_ref.read().unwrap();
                if let Some(od) = guard
                    .as_any()
                    .downcast_ref::<crate::vm::types::ObjectData>()
                {
                    let mut out = auto_val::Obj::new();
                    for (k, v) in od.fields.iter() {
                        out.set(k.clone(), v.clone());
                    }
                    return auto_val::Value::Obj(Box::new(out));
                }
                if let Some(list) = guard
                    .as_any()
                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                {
                    // CREATE_ARRAY now encodes arrays as TAG_OBJECT (H3b).
                    let mut out = auto_val::Array::new();
                    for v in list.elems.iter() {
                        out.push(self.resolve_array_elem(v));
                    }
                    return auto_val::Value::Array(out);
                }
                if let Some(node) = guard.as_any().downcast_ref::<auto_val::Node>() {
                    // ACCUM_NODE pops a nested node through this path.
                    return auto_val::Value::node((*node).clone());
                }
            }
            return auto_val::Value::Nil;
        }
        // List reference: arrays/lists are stored in heap_objects (H3b).
        if is_list(nv) {
            let id = decode_list(nv) as u64;
            if let Some(list_ref) = self.get_heap_object(id) {
                let elems = list_ref.read().unwrap();
                if let Some(list) = elems
                    .as_any()
                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                {
                    let mut out = auto_val::Array::new();
                    for v in list.elems.iter() {
                        out.push(self.resolve_array_elem(v));
                    }
                    return auto_val::Value::Array(out);
                }
            }
            return auto_val::Value::Nil;
        }
        // i32-tagged payload. CREATE_NODE / POP_ACCUM push node ids via
        // push_i32 (also i32-tagged), so a positive payload may be a node /
        // object / array id — distinguish by registry membership before
        // treating it as a literal Int.
        let bits = decode_i32(nv);
        if bits < 0 {
            // Legacy negative-tagged string index: -(index+1).
            let str_idx = (-bits - 1) as usize;
            let strings = self.strings.read().unwrap();
            return if let Some(bytes) = strings.get(str_idx) {
                auto_val::Value::Str(String::from_utf8_lossy(bytes).to_string().into())
            } else {
                auto_val::Value::Nil
            };
        }
        let id = bits as u64;
        // Plan 390 §15 H3b: ObjectData/Node live in heap_objects — single probe.
        if let Some(obj_ref) = self.get_heap_object(id) {
            let guard = obj_ref.read().unwrap();
            if let Some(od) = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ObjectData>()
            {
                let mut out = auto_val::Obj::new();
                for (k, v) in od.fields.iter() {
                    out.set(k.clone(), v.clone());
                }
                return auto_val::Value::Obj(Box::new(out));
            }
            if let Some(node) = guard.as_any().downcast_ref::<auto_val::Node>() {
                // Deep-clone the Node (props, args, AND kids). Sub-nodes are
                // accumulated into `kids` via ACCUM_NODE (Plan 364 Step 5), so
                // they must survive this pop to be queryable via `nodes(name)`.
                return auto_val::Value::node((*node).clone());
            }
        }
        if let Some(list_ref) = self.get_heap_object(id) {
            let elems = list_ref.read().unwrap();
            if let Some(list) = elems
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
            {
                let mut out = auto_val::Array::new();
                for v in list.elems.iter() {
                    out.push(self.resolve_array_elem(v));
                }
                return auto_val::Value::Array(out);
            }
        }
        auto_val::Value::Int(bits)
    }

    /// Plan 364 Step 5: Resolve a stored array element into a proper
    /// `auto_val::Value`. The Script VM stores string elements inside arrays
    /// as negative `Value::Int` (the neg_tag string-index convention shared
    /// with GET_ELEM), so we convert those back to `Value::Str` here. Other
    /// element types pass through unchanged.
    fn resolve_array_elem(&self, v: &auto_val::Value) -> auto_val::Value {
        if let auto_val::Value::Int(i) = v {
            if *i < 0 {
                let str_idx = (-(*i) - 1) as usize;
                let strings = self.strings.read().unwrap();
                if let Some(bytes) = strings.get(str_idx) {
                    return auto_val::Value::Str(String::from_utf8_lossy(bytes).to_string().into());
                }
            }
        }
        v.clone()
    }

    /// Pop a basic value from the stack as i32
    ///
    /// For Phase 2, assumes the value is a basic type (int, bool, char, nil)
    #[allow(dead_code)]
    fn pop_value_as_int(ram: &mut VirtualRAM) -> i32 {
        ram.pop_i32()
    }

    /// Pop a float value from the stack as f32
    #[allow(dead_code)]
    fn pop_value_as_float(ram: &mut VirtualRAM) -> f32 {
        ram.pop_f32()
    }

    /// Pop a double value from the stack as f64
    #[allow(dead_code)]
    fn pop_value_as_double(ram: &mut VirtualRAM) -> f64 {
        ram.pop_f64()
    }

    /// Plan 197 Task 7: Structural equality for heap objects (struct instances)
    ///
    /// Compares two heap objects by their structural content rather than by ID.
    /// Both operands are expected to be >= 4000000 (heap object IDs).
    /// Normalize boolean comparison: literal 1/0 vs comparison sentinel i32::MIN/i32::MIN+1.
    pub(super) fn bool_eq(a: i32, b: i32) -> bool {
        if a == b {
            return true;
        }
        let a_bool = match a {
            1 | -2147483648 => Some(true),
            0 | -2147483647 => Some(false),
            _ => None,
        };
        let b_bool = match b {
            1 | -2147483648 => Some(true),
            0 | -2147483647 => Some(false),
            _ => None,
        };
        match (a_bool, b_bool) {
            (Some(ab), Some(bb)) => ab == bb,
            _ => false,
        }
    }

    pub(super) fn nv_int_as_i64(nv: auto_val::NanoValue) -> i64 {
        if auto_val::is_i64(nv) {
            auto_val::decode_i64(nv)
        } else {
            auto_val::decode_i32(nv) as i64
        }
    }

    /// PLAN-749 T-02：ADD 串拼接臂的非串操作数显示。此前一律
    /// `decode_i32(bits).to_string()`——TAG_BOOL 低 32 位 0x80000000 →
    /// `"-2147483648"`（musk PLAN-100 E2E `[084-probe] ... ok=-2147483648`
    /// 实锤），TAG_I64/U64 截断为低 32 位同族垃圾。bool/宽整型按值显示；
    /// f64/f32 到不了串臂（ADD 前置数值臂已拦截）；obj/list 维持既有
    /// 堆 id 数值形态（display 语义变更不在本修复面）。
    pub(super) fn nv_concat_operand_display(nv: auto_val::NanoValue) -> String {
        if auto_val::is_bool(nv) {
            if auto_val::decode_bool(nv) {
                "true".to_string()
            } else {
                "false".to_string()
            }
        } else if auto_val::is_i64(nv) {
            auto_val::decode_i64(nv).to_string()
        } else if auto_val::is_u64(nv) {
            auto_val::decode_u64(nv).to_string()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    }

    pub(super) fn nv_is_numeric(nv: auto_val::NanoValue) -> bool {
        // PLAN-591 T9:补 is_i64——dep 方法返回的宽整型走 push_i64_vm
        // (TAG_I64),此前 EQ/数值谓词恒 false(uuid 勘测 get_version_num
        // == 4 实勘暴露);GT/序比较走独立臂未受影响。
        auto_val::is_f64(nv) || auto_val::is_f32(nv) || auto_val::is_i32(nv) || auto_val::is_i64(nv)
    }

    /// Plan 539 W2 (T19): widen a numeric nv to f64 for comparison.
    pub(super) fn nv_as_f64(nv: auto_val::NanoValue) -> f64 {
        if auto_val::is_f64(nv) {
            auto_val::decode_f64(nv)
        } else if auto_val::is_f32(nv) {
            auto_val::decode_f32(nv) as f64
        } else if auto_val::is_i64(nv) {
            auto_val::decode_i64(nv) as f64
        } else {
            auto_val::decode_i32(nv) as f64
        }
    }

    /// Plan 576: 有序比较（LT/GT/LE/GE）的"至少一侧 float"路由条件——
    /// 双侧数值且一侧 f32/f64 时走 nv_as_f64 按值比较；纯 int 对与
    /// object/string 走各自既有臂不变。
    pub(super) fn nv_pair_is_float_numeric(a: auto_val::NanoValue, b: auto_val::NanoValue) -> bool {
        Self::nv_is_numeric(a)
            && Self::nv_is_numeric(b)
            && (auto_val::is_f32(a)
                || auto_val::is_f32(b)
                || auto_val::is_f64(a)
                || auto_val::is_f64(b))
    }

    pub(super) fn nv_is_null_family(nv: auto_val::NanoValue) -> bool {
        if auto_val::is_null(nv) {
            return true;
        }
        if auto_val::is_i32(nv) {
            let v = auto_val::decode_i32(nv);
            return v == -1 || v == -2147483647;
        }
        false
    }

    /// Compare two opaque heap objects by value.
    /// Returns Some(true/false) if comparison is supported, None to fall back to integer comparison.
    pub(super) fn compare_opaque_objects(&self, a_id: u64, b_id: u64) -> Option<bool> {
        use crate::vm::ffi::rust_stdlib::RustStdlibObject;
        let a_obj = self.get_heap_object(a_id)?;
        let b_obj = self.get_heap_object(b_id)?;
        let a_guard = a_obj.read().ok()?;
        let b_guard = b_obj.read().ok()?;
        let a_rso = a_guard.as_any().downcast_ref::<RustStdlibObject>()?;
        let b_rso = b_guard.as_any().downcast_ref::<RustStdlibObject>()?;
        // semver::Version comparison
        if a_rso.type_name == "semver::Version" && b_rso.type_name == "semver::Version" {
            let a_ver = a_rso.downcast_ref::<std::sync::Mutex<semver::Version>>()?;
            let b_ver = b_rso.downcast_ref::<std::sync::Mutex<semver::Version>>()?;
            Some(*a_ver.lock().unwrap() > *b_ver.lock().unwrap())
        } else {
            None
        }
    }

    pub fn struct_eq(&self, a: i32, b: i32) -> bool {
        use crate::vm::generic_registry::GenericInstanceData;
        use crate::vm::heap_object::TypeTag;

        let id_a = a as u64;
        let id_b = b as u64;

        // Quick pointer equality check
        if id_a == id_b {
            return true;
        }

        // Look up both instances
        let obj_a = match self.get_heap_object(id_a) {
            Some(obj) => obj,
            None => return false,
        };
        let obj_b = match self.get_heap_object(id_b) {
            Some(obj) => obj,
            None => return false,
        };

        let guard_a = obj_a.read().unwrap();
        let guard_b = obj_b.read().unwrap();

        // Check both are GenericInstance
        if !matches!(guard_a.type_tag(), TypeTag::GenericInstance(_)) {
            return false;
        }
        if !matches!(guard_b.type_tag(), TypeTag::GenericInstance(_)) {
            return false;
        }

        let inst_a = match guard_a.as_any().downcast_ref::<GenericInstanceData>() {
            Some(inst) => inst,
            None => return false,
        };
        let inst_b = match guard_b.as_any().downcast_ref::<GenericInstanceData>() {
            Some(inst) => inst,
            None => return false,
        };

        // Different types are never equal
        if inst_a.mono_name != inst_b.mono_name {
            return false;
        }

        // Must have same number of fields
        if inst_a.fields.len() != inst_b.fields.len() {
            return false;
        }

        // Compare each field pairwise
        for (fa, fb) in inst_a.fields.iter().zip(inst_b.fields.iter()) {
            if !self.values_equal(fa, fb) {
                return false;
            }
        }

        true
    }

    /// Plan 197 Task 16: Check if a heap object is an Option.None instance
    pub fn is_option_none(&self, instance_id: u64) -> bool {
        use crate::vm::generic_registry::GenericInstanceData;
        if let Some(obj) = self.get_heap_object(instance_id) {
            let guard = obj.read().unwrap();
            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                return instance.mono_name == "Option.None";
            }
        }
        false
    }

    /// Plan 197 Task 16: Check if a heap object is an Option.Some instance
    pub fn is_option_some(&self, instance_id: u64) -> bool {
        use crate::vm::generic_registry::GenericInstanceData;
        if let Some(obj) = self.get_heap_object(instance_id) {
            let guard = obj.read().unwrap();
            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                return instance.mono_name == "Option.Some";
            }
        }
        false
    }

    /// Plan 197 Task 16: Get the inner value from an Option.Some instance
    pub fn get_option_inner(&self, instance_id: u64) -> Option<Value> {
        use crate::vm::generic_registry::GenericInstanceData;
        if let Some(obj) = self.get_heap_object(instance_id) {
            let guard = obj.read().unwrap();
            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                if instance.mono_name == "Option.Some" {
                    return instance.get_field(0).cloned();
                }
            }
        }
        None
    }

    /// Compare two Value instances for structural equality
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Uint(x), Value::Uint(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x.to_bits() == y.to_bits(),
            (Value::Double(x), Value::Double(y)) => x.to_bits() == y.to_bits(),
            (Value::Char(x), Value::Char(y)) => x == y,
            (Value::Nil, Value::Nil) => true,
            (Value::Str(x), Value::Str(y)) => x.as_bytes() == y.as_bytes(),
            (Value::VmRef(ref_a), Value::VmRef(ref_b)) => {
                // Nested heap object — recursive structural equality
                self.struct_eq(ref_a.id as i32, ref_b.id as i32)
            }
            _ => false,
        }
    }

    /// Pop a string value from the stack (returns string index)
    #[allow(dead_code)]
    fn pop_value_as_string_index(ram: &mut VirtualRAM) -> i32 {
        ram.pop_i32()
    }

    // ============================================================================
    // End Plan 087 Phase 2
    // ============================================================================

    /// Get string by index from the constant pool.
    /// 2026-08-22(侧栏串写根因):u16 → u32。运行期池会持续增长(字段读
    /// 重新驻留/STR_CAT 中间串/宿主桥 JSON),会话活跃几分钟即可越过
    /// 65535;u16 截断令新旧索引回绕互串(ash-gui 侧栏 alias/bash 条目
    /// 消失、ash 名字变 "." 的根因)。nanbox 字符串标签本就是 u32 负数
    /// 编码,容量到 2^31,此处放开没有表示性问题。
    pub fn get_string(&self, index: u32) -> Option<Vec<u8>> {
        // Plan 419 Phase 2: 池墓碑 canary(debug)—— rc=0 的非 pinned 条目
        // 已释放,读它即 use-after-free。
        #[cfg(debug_assertions)]
        {
            if self.pool_is_tombstone(index as usize) {
                panic!(
                    "[RC canary] string tombstone access: pool index {} was freed",
                    index
                );
            }
        }
        if crate::pool_log_all() {
            eprintln!(
                "[POOLLOG #{:>4}] CANARY-READ {}",
                crate::pool_log_seq(),
                index
            );
        }
        let strings = self.strings.read().unwrap();
        strings.get(index as usize).cloned()
    }
}
