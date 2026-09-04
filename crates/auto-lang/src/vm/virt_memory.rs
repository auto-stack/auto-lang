#![allow(unused_unsafe)]

/// Virtual Memory Model for AutoVM
///
/// Implements the "Digital Twin" memory architecture:
/// - VirtualFlash: Read-only code space
/// - VirtualRAM: Read-write data space (Stack + Heap)
use crate::vm::codegen::ObjectType;
use std::collections::HashMap;
use auto_val::{NanoValue, encode_i32, decode_i32,
    encode_f32, decode_f32, encode_string, decode_string};

/// A 32-bit word in the virtual machine
/// Simplified to just i32 for now to avoid union issues
#[derive(Clone, Copy, Default, Debug)]
pub struct Word {
    pub i: i32,
}

impl Word {
    pub fn with_i32(val: i32) -> Self {
        Self { i: val }
    }

    pub fn with_u32(val: u32) -> Self {
        Self { i: val as i32 }
    }

    pub fn with_f32(val: f32) -> Self {
        Self { i: unsafe { f32::to_bits(val).cast_signed() } }
    }
}

/// Simulates MCU Flash (Code Space)
/// Contains bytecode and constant data
pub struct VirtualFlash {
    pub memory: Vec<u8>,
    // Map function IDs/Fragment IDs to addresses in memory
    // TODO: Use actual specific ID type later
    pub symbol_map: HashMap<u32, usize>,
    // Plan 073: Object keys metadata for object literal creation
    // Each entry is a Vec of keys for one object literal (indexed by key_index)
    pub object_keys: Vec<Vec<auto_val::ValueKey>>,
    // Plan 073: Object field types for runtime value conversion
    pub object_types: Vec<Vec<ObjectType>>,
    /// Exports by name for CALL_SPEC dynamic dispatch
    pub exports_by_name: HashMap<String, u32>,
    /// Plan 199 Phase 7: Reverse map — bytecode offset to function name
    pub addr_to_name: HashMap<u32, String>,
}

impl VirtualFlash {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            symbol_map: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            exports_by_name: HashMap::new(),
            addr_to_name: HashMap::new(),
        }
    }

    pub fn new_with_code(code: Vec<u8>) -> Self {
        Self {
            memory: code,
            symbol_map: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            exports_by_name: HashMap::new(),
            addr_to_name: HashMap::new(),
        }
    }

    // Plan 073: Create VirtualFlash with code, object_keys, and object_types
    pub fn new_with_code_and_keys(
        code: Vec<u8>,
        object_keys: Vec<Vec<auto_val::ValueKey>>,
        object_types: Vec<Vec<crate::vm::codegen::ObjectType>>,
    ) -> Self {
        Self {
            memory: code,
            symbol_map: HashMap::new(),
            object_keys,
            object_types,
            exports_by_name: HashMap::new(),
            addr_to_name: HashMap::new(),
        }
    }

    /// Create VirtualFlash from raw bytecode (no metadata).
    /// Used by debugger for disassembly.
    pub fn from_vec(code: Vec<u8>) -> Self {
        Self {
            memory: code,
            symbol_map: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            exports_by_name: HashMap::new(),
            addr_to_name: HashMap::new(),
        }
    }

    /// Plan 128: Create VirtualFlash from Vec with full metadata
    ///
    /// Used by VMLoader to create the frozen bytecode from CompiledPackage
    pub fn from_vec_with_metadata(
        code: Vec<u8>,
        exports: HashMap<String, u32>,
        object_keys: Vec<Vec<auto_val::ValueKey>>,
        object_types: Vec<Vec<ObjectType>>,
    ) -> Self {
        // Keep exports_by_name for CALL_SPEC dynamic dispatch
        let exports_by_name = exports.clone();

        // Plan 199 Phase 7: Build reverse map (address -> function name)
        let addr_to_name: HashMap<u32, String> = exports_by_name
            .iter()
            .map(|(name, &addr)| (addr, name.clone()))
            .collect();

        // Convert string exports to u32 symbol map
        // For now, we use a simple hash-based ID for symbols
        let symbol_map: HashMap<u32, usize> = exports
            .into_iter()
            .map(|(name, offset)| {
                // Use a simple hash of the name as the symbol ID
                let id = Self::name_to_symbol_id(&name);
                (id, offset as usize)
            })
            .collect();

        Self {
            memory: code,
            symbol_map,
            object_keys,
            object_types,
            exports_by_name,
            addr_to_name,
        }
    }

    /// Convert a name to a symbol ID (simple hash-based approach)
    fn name_to_symbol_id(name: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish() as u32
    }

    #[inline(always)]
    pub fn read_u8(&self, addr: usize) -> u8 {
        if addr >= self.memory.len() {
            eprintln!("WARNING: Flash read_u8 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0; // Return 0 (NOP) as safe default
        }
        self.memory[addr]
    }

    #[inline(always)]
    pub fn read_i32(&self, addr: usize) -> i32 {
        if addr + 4 > self.memory.len() {
            eprintln!("WARNING: Flash read_i32 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 4];
        i32::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_i16(&self, addr: usize) -> i16 {
        if addr + 2 > self.memory.len() {
            eprintln!("WARNING: Flash read_i16 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 2];
        i16::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u16(&self, addr: usize) -> u16 {
        if addr + 2 > self.memory.len() {
            eprintln!("WARNING: Flash read_u16 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 2];
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u32(&self, addr: usize) -> u32 {
        if addr + 4 > self.memory.len() {
            eprintln!("WARNING: Flash read_u32 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_f32(&self, addr: usize) -> f32 {
        if addr + 4 > self.memory.len() {
            eprintln!("WARNING: Flash read_f32 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0.0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 4];
        f32::from_le_bytes(bytes.try_into().unwrap())
    }

    // Plan 073 Stage A: Double precision support
    #[inline(always)]
    pub fn read_f64(&self, addr: usize) -> f64 {
        if addr + 8 > self.memory.len() {
            eprintln!("WARNING: Flash read_f64 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0.0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 8];
        f64::from_le_bytes(bytes.try_into().unwrap())
    }

    // Plan 073 Stage A: 64-bit integer support
    #[inline(always)]
    pub fn read_i64(&self, addr: usize) -> i64 {
        if addr + 8 > self.memory.len() {
            eprintln!("WARNING: Flash read_i64 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 8];
        i64::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u64(&self, addr: usize) -> u64 {
        if addr + 8 > self.memory.len() {
            eprintln!("WARNING: Flash read_u64 out of bounds: addr={}, len={}", addr, self.memory.len());
            return 0; // Return safe default
        }
        let bytes = &self.memory[addr..addr + 8];
        u64::from_le_bytes(bytes.try_into().unwrap())
    }
}

/// Simulates MCU SRAM (Data Space)
/// Contains the Stack and Heap (though Heap is currently simulated via Rust heap for objects in Phase 1)
/// Phase 1: Pure stack machine
pub struct VirtualRAM {
    pub raw: Vec<i32>,
    /// Plan 221: NaN-boxed stack
    pub raw_nv: Vec<NanoValue>,
    pub sp: usize, // Stack Pointer (Index of the next free slot)
    pub bp: usize, // Base Pointer (Index of the current frame)
    /// Range storage: (start, end, is_inclusive)
    pub ranges: Vec<(i32, i32, bool)>,
}

impl VirtualRAM {
    pub fn new(size: usize) -> Self {
        Self {
            raw: vec![0; size],
            raw_nv: vec![0u64; size],
            sp: 0,
            bp: 0,
            ranges: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn push_i32(&mut self, val: i32) {
        if self.sp >= self.raw_nv.len() {
            // Double the stack capacity
            let new_size = (self.raw_nv.len() * 2).max(256);
            self.raw_nv.resize(new_size, 0);
        }
        self.raw_nv[self.sp] = encode_i32(val);
        self.sp += 1;
    }

    #[inline(always)]
    pub fn pop_i32(&mut self) -> i32 {
        if self.sp == 0 { panic!("Stack Underflow"); }
        self.sp -= 1;
        decode_i32(self.raw_nv[self.sp])
    }

    // Plan 073 Stage A: Float support
    #[inline(always)]
    pub fn push_f32(&mut self, val: f32) {
        if self.sp >= self.raw_nv.len() { panic!("Stack Overflow"); }
        self.raw_nv[self.sp] = encode_f32(val);
        self.sp += 1;
    }

    #[inline(always)]
    pub fn pop_f32(&mut self) -> f32 {
        if self.sp == 0 { panic!("Stack Underflow"); }
        self.sp -= 1;
        // Plan 437: tag 驱动解码——f32 槽位上可能是 f64 值（math.* 等 f64
        // 返回值经 float 声明变量直通存储），按实际 tag 解码而非位重解释。
        let nv = self.raw_nv[self.sp];
        if auto_val::is_f64(nv) {
            auto_val::decode_f64(nv) as f32
        } else if auto_val::is_i32(nv) {
            auto_val::decode_i32(nv) as f32
        } else {
            decode_f32(nv)
        }
    }

    // Plan 073 Stage A: Double (f64) support
    // Plan 377: f64 单槽化 —— 单个 NanoValue 装得下完整 f64（encode_f64 直接位模式）。
    #[inline(always)]
    pub fn push_f64(&mut self, val: f64) {
        self.push_nv(auto_val::encode_f64(val));
    }

    #[inline(always)]
    pub fn pop_f64(&mut self) -> f64 {
        auto_val::decode_f64(self.pop_nv())
    }

    // Plan 073 Stage A: Unsigned integer support
    #[inline(always)]
    pub fn push_u32(&mut self, val: u32) {
        self.push_i32(val as i32);
    }

    #[inline(always)]
    pub fn pop_u32(&mut self) -> u32 {
        self.pop_i32() as u32
    }

    // Plan 073 Stage A: 64-bit integer support
    // Plan 377: i64 单槽化（48 位 payload 内联编码）。>2^47 的值无法内联，本函数
    // 会 panic —— 调用方（engine opcode / native）应改用 AutoVM::push_i64_vm
    // （heap-aware 版本，溢出时 BigInt 堆装箱，见 plan 377 §4.3）。
    // 现实场景全部 < 2^47（见 §2.6），故此 panic 不会被普通程序触发。
    #[inline(always)]
    pub fn push_i64(&mut self, val: i64) {
        match auto_val::try_encode_i64(val) {
            Some(nv) => self.push_nv(nv),
            None => panic!(
                "push_i64({}) 超出 48 位内联范围（[-2^47, 2^47)），本层无法堆装箱；\
                 engine/native 调用方应改用 AutoVM::push_i64_vm（heap-aware）", val
            ),
        }
    }

    #[inline(always)]
    pub fn pop_i64(&mut self) -> i64 {
        // Plan 377: 弹出 64 位有符号整数。i64 与 u64 编码对称（仅 tag 不同），
        // 此处统一处理 I64/U64/i32 任意 tag，避免 push/pop 标签不一致导致的截断。
        // 注意：TAG_BIGINT（>2^48 堆装箱值）无法在此解引用（本层无 VM 访问），
        // 遇到时 panic —— 调用方（engine opcode / native）应改用 AutoVM::pop_i64_vm
        // （heap-aware 版本，见 plan 377 §4.3）。
        let nv = self.pop_nv();
        match auto_val::tag_of(nv) {
            t if t == 8 => auto_val::decode_i64(nv),      // TAG_I64
            t if t == 9 => auto_val::decode_u64(nv) as i64, // TAG_U64（按有符号读）
            t if t == 0xA => panic!(
                "pop_i64 遇到 TAG_BIGINT（>2^48 堆装箱值），本层无法解引用；\
                 engine/native 调用方应改用 AutoVM::pop_i64_vm（heap-aware）"
            ),
            _ => auto_val::decode_i32(nv) as i64,         // 兼容 i32 操作数
        }
    }

    // Plan 073 Stage A: u64 support
    // Plan 377: u64 单槽化（48 位 payload 内联编码）。>=2^48 的值无法内联，本函数
    // 会 panic —— 调用方（engine opcode / native）应改用 AutoVM::push_u64_vm
    // （heap-aware 版本，溢出时 BigInt 堆装箱，见 plan 377 §4.3）。
    // 现实场景全部 < 2^48（见 §2.6），故此 panic 不会被普通程序触发。
    #[inline(always)]
    pub fn push_u64(&mut self, val: u64) {
        match auto_val::try_encode_u64(val) {
            Some(nv) => self.push_nv(nv),
            None => panic!(
                "push_u64({}) 超出 48 位内联范围（[0, 2^48)），本层无法堆装箱；\
                 engine/native 调用方应改用 AutoVM::push_u64_vm（heap-aware）", val
            ),
        }
    }

    #[inline(always)]
    pub fn pop_u64(&mut self) -> u64 {
        // Plan 377: 弹出 64 位无符号整数。统一处理 I64/U64/i32 任意 tag。
        // 注意：TAG_BIGINT（>2^48 堆装箱值）无法在此解引用（本层无 VM 访问），
        // 遇到时 panic —— 调用方（engine opcode / native）应改用 AutoVM::pop_u64_vm
        // （heap-aware 版本，见 plan 377 §4.3）。
        let nv = self.pop_nv();
        match auto_val::tag_of(nv) {
            t if t == 9 => auto_val::decode_u64(nv),      // TAG_U64
            t if t == 8 => auto_val::decode_i64(nv) as u64, // TAG_I64（按无符号读）
            t if t == 0xA => panic!(
                "pop_u64 遇到 TAG_BIGINT（>2^48 堆装箱值），本层无法解引用；\
                 engine/native 调用方应改用 AutoVM::pop_u64_vm（heap-aware）"
            ),
            _ => auto_val::decode_i32(nv) as u32 as u64,  // 兼容 i32 操作数
        }
    }

    pub fn read_i32(&self, addr: usize) -> i32 { decode_i32(self.raw_nv[addr]) }

    pub fn write_i32(&mut self, addr: usize, val: i32) { self.raw_nv[addr] = encode_i32(val); }

    // For manual viewing
    pub fn top(&self) -> Option<i32> {
        if self.sp == 0 { None } else { Some(decode_i32(self.raw_nv[self.sp - 1])) }
    }

    // ---- Plan 221: NanoValue operations ----

    #[inline(always)]
    pub fn push_nv(&mut self, val: NanoValue) {
        if self.sp >= self.raw_nv.len() {
            let new_size = (self.raw_nv.len() * 2).max(256);
            self.raw_nv.resize(new_size, 0);
        }
        self.raw_nv[self.sp] = val;
        self.sp += 1;
    }

    #[inline(always)]
    pub fn pop_nv(&mut self) -> NanoValue {
        if self.sp == 0 {
            panic!("Stack Underflow (nanbox)");
        }
        self.sp -= 1;
        self.raw_nv[self.sp]
    }

    /// Peek at the Nth value from top of stack without popping.
    /// peek_nv(0) returns the top, peek_nv(1) returns one below, etc.
    #[inline(always)]
    pub fn peek_nv(&self, offset: usize) -> NanoValue {
        if self.sp <= offset {
            panic!("Stack Underflow (nanbox peek)");
        }
        self.raw_nv[self.sp - 1 - offset]
    }

    /// Pop a typed arithmetic operand from the stack.
    ///
    /// Plan 377: 全值单槽化后，每个操作数恒占 1 个栈槽。本 helper 弹出单个
    /// NanoValue，按是否为 f64（非 nanboxed）返回 `(bits, is_f64)`，供多态
    /// 算术 opcode（ADD/SUB/MUL/DIV/NEG）分派：
    /// - f64 → `(raw_f64_bits, true)`，调用方用 `f64::from_bits` 解码。
    /// - 其它（i32/f32/string/bool/object/i64/u64/bigint）→ `(nanboxed_value, false)`。
    #[inline(always)]
    pub fn pop_arith_operand(&mut self) -> (u64, bool) {
        let nv = self.pop_nv();
        let is_f64 = !auto_val::is_nanboxed(nv);
        (nv, is_f64)
    }

    /// Plan 550 T03: 弹出二元算术操作数对并拒收 null（TAG_NULL）。
    ///
    /// null 位模式直接 decode_i32 参与运算是 539 实证的静默垃圾病灶
    /// （`null + 1` → -2147483646、`"a" + null` → 垃圾数字入串拼接），
    /// 本计划全族统一翻转为可 try-catch 捕获的 Python 风格 TypeError。
    ///
    /// 只拒 TAG_NULL：`null`/`nil`/`None` 三拼写经 PUSH_NIL 同落
    /// encode_null（PLAN-053 P-053-2 归一）。历史 i32 哨兵编码
    /// （-1 / i32::MIN+1）与真实整数在算术槽不可区分，不在守卫范围
    /// （EQ 判等的 null-family 兼容语义不变，见 nv_is_null_family）。
    #[inline(always)]
    pub fn pop_arith_pair_non_null(
        &mut self,
        op: &str,
    ) -> Result<((u64, bool), (u64, bool)), crate::vm::engine::VMError> {
        let b = self.pop_arith_operand();
        let a = self.pop_arith_operand();
        if arith_operand_is_null(&a) || arith_operand_is_null(&b) {
            return Err(null_binop_type_error(op, a, b));
        }
        Ok((a, b))
    }

    /// Plan 550 T03: 弹出一元算术操作数并拒收 null（TAG_NULL）。
    #[inline(always)]
    pub fn pop_arith_operand_non_null(
        &mut self,
        op: &str,
    ) -> Result<(u64, bool), crate::vm::engine::VMError> {
        let a = self.pop_arith_operand();
        if arith_operand_is_null(&a) {
            return Err(null_unop_type_error(op));
        }
        Ok(a)
    }

    /// Write a raw NanoValue at an address (preserves type tag).
    #[inline(always)]
    pub fn write_nv(&mut self, addr: usize, val: NanoValue) {
        self.raw_nv[addr] = val;
    }

    /// Read a raw NanoValue from an address (preserves type tag).
    #[inline(always)]
    pub fn read_nv(&self, addr: usize) -> NanoValue {
        self.raw_nv[addr]
    }

    #[inline(always)]
    pub fn push_string(&mut self, idx: u32) {
        self.push_nv(encode_string(idx));
    }

    #[inline(always)]
    pub fn pop_string(&mut self) -> u32 {
        decode_string(self.pop_nv())
    }

    /// Pop a value that is known to be a string reference, returning the string pool index.
    #[inline(always)]
    pub fn pop_str_idx(&mut self) -> usize {
        decode_string(self.pop_nv()) as usize
    }

    /// Push a string pool index as a tagged reference.
    #[inline(always)]
    pub fn push_str_idx(&mut self, idx: u32) {
        self.push_nv(encode_string(idx));
    }
}

// ---- Plan 550 T03: null 算术守卫的共享消息助手 ----
// 栈帧纪律（539 三次溢出教训）：守卫体保持 2-3 行，消息构造集中到
// 模块级函数，不内联大块进热递归 match 臂。

/// Plan 550 T03: 算术操作数的 Python 风格类型名（TypeError 消息渲染）。
/// 粗粒度映射：nanbox 可判定的标量类型 + object 兜底（堆对象具体
/// 类型名需 VM 堆访问，不在本层）。
#[inline(always)]
pub fn nv_py_type_name(nv: NanoValue) -> &'static str {
    if !auto_val::is_nanboxed(nv) {
        return "float"; // f64 直接位模式（非 nanboxed）
    }
    if auto_val::is_null(nv) {
        "NoneType"
    } else if auto_val::is_f32(nv) {
        "float"
    } else if auto_val::is_string(nv) {
        "str"
    } else if auto_val::is_bool(nv) {
        "bool"
    } else if auto_val::is_object(nv) || auto_val::is_list(nv) {
        "object"
    } else {
        "int"
    }
}

/// Plan 550 T03: 二元算术 null 守卫的 TypeError 消息（Python 格式，
/// a=左操作数、b=右操作数，按操作数次序渲染类型名）。
pub fn null_binop_type_error(op: &str, a: (u64, bool), b: (u64, bool)) -> crate::vm::engine::VMError {
    crate::vm::engine::VMError::RuntimeError(format!(
        "TypeError: unsupported operand type(s) for {}: '{}' and '{}'",
        op,
        nv_py_type_name(a.0),
        nv_py_type_name(b.0)
    ))
}

/// Plan 550 T03: 一元算术 null 守卫的 TypeError 消息（Python 格式）。
pub fn null_unop_type_error(op: &str) -> crate::vm::engine::VMError {
    crate::vm::engine::VMError::RuntimeError(format!(
        "TypeError: bad operand type for unary {}: 'NoneType'",
        op
    ))
}

/// Plan 550 T06: 显式类型转换（.to(int)/.to(float)）null 输入的
/// TypeError 消息（Python 风格；原 -1/-1.0 静默臂是 539 T05 自加的
/// 兼容臂，本计划翻案——TYPE_TO_* 仅由 Expr::To 显式转换发射，
/// `??`/迭代哨兵等内部路径不经过，无内部依赖）。
pub fn null_to_type_error(op: &str) -> crate::vm::engine::VMError {
    crate::vm::engine::VMError::RuntimeError(format!(
        "TypeError: {}() argument must be a string or a real number, not 'NoneType'",
        op
    ))
}

/// Plan 550 T03: 算术操作数是否为 TAG_NULL（f64 槽不可能是 null）。
#[inline(always)]
fn arith_operand_is_null(operand: &(u64, bool)) -> bool {
    !operand.1 && auto_val::is_null(operand.0)
}
