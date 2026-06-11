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
        decode_f32(self.raw_nv[self.sp])
    }

    // Plan 073 Stage A: Double (f64) support
    #[inline(always)]
    pub fn push_f64(&mut self, val: f64) {
        if self.sp + 1 >= self.raw_nv.len() {
            let new_size = ((self.raw_nv.len() * 2).max(256)).max(self.sp + 2);
            self.raw_nv.resize(new_size, 0);
        }
        // Slot 1: raw f64 bits
        self.raw_nv[self.sp] = val.to_bits();
        self.sp += 1;
        // Slot 2: padding (encode_null as marker, matches codegen's 2-slot expectation)
        self.raw_nv[self.sp] = auto_val::encode_null();
        self.sp += 1;
    }

    #[inline(always)]
    pub fn pop_f64(&mut self) -> f64 {
        if self.sp < 2 { panic!("Stack Underflow"); }
        // Slot 2: padding marker (discard)
        self.sp -= 1;
        // Slot 1: raw f64 bits
        self.sp -= 1;
        f64::from_bits(self.raw_nv[self.sp])
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
    #[inline(always)]
    pub fn push_i64(&mut self, val: i64) {
        let low = (val & 0xFFFFFFFF) as i32;
        let high = ((val >> 32) & 0xFFFFFFFF) as i32;
        self.push_i32(low);
        self.push_i32(high);
    }

    #[inline(always)]
    pub fn pop_i64(&mut self) -> i64 {
        let high = self.pop_i32() as i64;
        let low = self.pop_i32() as i64;
        (high << 32) | (low & 0xFFFFFFFF)
    }

    // Plan 073 Stage A: u64 support
    #[inline(always)]
    pub fn push_u64(&mut self, val: u64) {
        let low = (val & 0xFFFFFFFF) as i32;
        let high = ((val >> 32) & 0xFFFFFFFF) as i32;
        self.push_i32(low);
        self.push_i32(high);
    }

    #[inline(always)]
    pub fn pop_u64(&mut self) -> u64 {
        let high = self.pop_u32() as u64;
        let low = self.pop_u32() as u64;
        (high << 32) | low
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
    /// f64 values occupy 2 slots (raw bits + null padding), while all other
    /// types occupy 1 slot. This helper inspects the top-of-stack to decide
    /// which pop method to use, returning both the raw NanoValue (or f64 bits)
    /// and a type tag so the caller can dispatch correctly.
    ///
    /// Returns `(bits, is_f64)`:
    /// - If the TOS is the null-padding marker of an f64 pair, pops 2 slots
    ///   and returns `(raw_f64_bits, true)`.
    /// - Otherwise pops 1 slot and returns `(nanboxed_value, false)`.
    #[inline(always)]
    pub fn pop_arith_operand(&mut self) -> (u64, bool) {
        // Check if TOS is the null-padding of a 2-slot f64.
        // The padding is always encode_null(), and the slot below it is the
        // raw f64 bits (which is never nanboxed since normal f64 != NaN).
        let tos = self.peek_nv(0);
        if auto_val::is_null(tos) {
            // Check slot below: if it's a raw f64 (not nanboxed), this is an f64 pair
            let below = self.peek_nv(1);
            if !auto_val::is_nanboxed(below) {
                // This is an f64 — use pop_f64 which correctly handles 2 slots
                let val = self.pop_f64();
                return (val.to_bits(), true);
            }
        }
        // Single-slot value (i32, f32, string, bool, object, etc.)
        let nv = self.pop_nv();
        (nv, false)
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
