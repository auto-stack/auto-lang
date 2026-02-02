/// Virtual Memory Model for BigVM
///
/// Implements the "Digital Twin" memory architecture:
/// - VirtualFlash: Read-only code space
/// - VirtualRAM: Read-write data space (Stack + Heap)
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
}

impl VirtualFlash {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            symbol_map: HashMap::new(),
        }
    }

    pub fn new_with_code(code: Vec<u8>) -> Self {
        Self {
            memory: code,
            symbol_map: HashMap::new(),
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
