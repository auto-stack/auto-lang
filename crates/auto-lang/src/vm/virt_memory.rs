/// Virtual Memory Model for AutoVM
///
/// Implements the "Digital Twin" memory architecture:
/// - VirtualFlash: Read-only code space
/// - VirtualRAM: Read-write data space (Stack + Heap)
use crate::vm::codegen::ObjectType;
use std::collections::HashMap;

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
        Self { i: unsafe { std::mem::transmute(val) } }
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
}

impl VirtualFlash {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            symbol_map: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
        }
    }

    pub fn new_with_code(code: Vec<u8>) -> Self {
        Self {
            memory: code,
            symbol_map: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
        }
    }

    // Plan 073: Create VirtualFlash with code, object_keys, and object_types
    pub fn new_with_code_and_keys(code: Vec<u8>, object_keys: Vec<Vec<auto_val::ValueKey>>) -> Self {
        Self {
            memory: code,
            symbol_map: HashMap::new(),
            object_keys,
            object_types: Vec::new(), // Will be populated separately
        }
    }

    #[inline(always)]
    pub fn read_u8(&self, addr: usize) -> u8 {
        self.memory[addr]
    }

    #[inline(always)]
    pub fn read_i32(&self, addr: usize) -> i32 {
        let bytes = &self.memory[addr..addr + 4];
        i32::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_i16(&self, addr: usize) -> i16 {
        let bytes = &self.memory[addr..addr + 2];
        i16::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u16(&self, addr: usize) -> u16 {
        let bytes = &self.memory[addr..addr + 2];
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u32(&self, addr: usize) -> u32 {
        let bytes = &self.memory[addr..addr + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_f32(&self, addr: usize) -> f32 {
        let bytes = &self.memory[addr..addr + 4];
        f32::from_le_bytes(bytes.try_into().unwrap())
    }

    // Plan 073 Stage A: Double precision support
    #[inline(always)]
    pub fn read_f64(&self, addr: usize) -> f64 {
        let bytes = &self.memory[addr..addr + 8];
        f64::from_le_bytes(bytes.try_into().unwrap())
    }

    // Plan 073 Stage A: 64-bit integer support
    #[inline(always)]
    pub fn read_i64(&self, addr: usize) -> i64 {
        let bytes = &self.memory[addr..addr + 8];
        i64::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u64(&self, addr: usize) -> u64 {
        let bytes = &self.memory[addr..addr + 8];
        u64::from_le_bytes(bytes.try_into().unwrap())
    }
}

/// Simulates MCU SRAM (Data Space)
/// Contains the Stack and Heap (though Heap is currently simulated via Rust heap for objects in Phase 1)
/// Phase 1: Pure stack machine
pub struct VirtualRAM {
    pub raw: Vec<i32>,
    pub sp: usize, // Stack Pointer (Index of the next free slot)
    pub bp: usize, // Base Pointer (Index of the current frame)
}

impl VirtualRAM {
    pub fn new(size: usize) -> Self {
        Self {
            raw: vec![0; size],
            sp: 0,
            bp: 0,
        }
    }

    #[inline(always)]
    pub fn push_i32(&mut self, val: i32) {
        if self.sp >= self.raw.len() {
            panic!("Stack Overflow"); // Todo: Return Result
        }
        self.raw[self.sp] = val;
        self.sp += 1;
    }

    #[inline(always)]
    pub fn pop_i32(&mut self) -> i32 {
        if self.sp == 0 {
            panic!("Stack Underflow");
        }
        self.sp -= 1;
        self.raw[self.sp]
    }

    // Plan 073 Stage A: Float support
    #[inline(always)]
    pub fn push_f32(&mut self, val: f32) {
        // Use bit transmute to store f32 in i32 slot
        let bits: i32 = unsafe { std::mem::transmute(val) };
        self.push_i32(bits);
    }

    #[inline(always)]
    pub fn pop_f32(&mut self) -> f32 {
        let bits = self.pop_i32();
        unsafe { std::mem::transmute(bits) }
    }

    // Plan 073 Stage A: Double (f64) support
    // Note: f64 takes 2 slots in our 32-bit VM
    #[inline(always)]
    pub fn push_f64(&mut self, val: f64) {
        // Use bit transmute to split f64 into two i32 slots
        let bits: u64 = unsafe { std::mem::transmute(val) };
        let low = (bits & 0xFFFFFFFF) as i32;
        let high = ((bits >> 32) & 0xFFFFFFFF) as i32;
        self.push_i32(low);  // Push low part first
        self.push_i32(high); // Then high part
    }

    #[inline(always)]
    pub fn pop_f64(&mut self) -> f64 {
        let high = self.pop_i32() as u64;
        let low = self.pop_i32() as u64;
        let bits = (high << 32) | low;
        unsafe { std::mem::transmute(bits) }
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
    // Note: i64 takes 2 slots in our 32-bit VM
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

    #[inline(always)]
    pub fn read_i32(&self, addr: usize) -> i32 {
        if addr >= self.raw.len() {
            panic!("Memory Access Out of Bounds");
        }
        self.raw[addr]
    }

    #[inline(always)]
    pub fn write_i32(&mut self, addr: usize, val: i32) {
        if addr >= self.raw.len() {
            panic!("Memory Write Out of Bounds");
        }
        self.raw[addr] = val;
    }

    // For manual viewing
    pub fn top(&self) -> Option<i32> {
        if self.sp == 0 {
            None
        } else {
            Some(self.raw[self.sp - 1])
        }
    }
}
