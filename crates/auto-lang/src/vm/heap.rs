use crate::error::AutoResult;

/// A simple linear allocator that simulates a heap in a contiguous memory block.
/// It uses a bump pointer allocation strategy.
#[derive(Debug)]
pub struct LinearAllocator {
    memory: Vec<u8>,
    top: usize,
}

impl LinearAllocator {
    pub fn new(capacity: usize) -> Self {
        Self {
            memory: vec![0; capacity],
            top: 0,
        }
    }

    /// Allocates `size` bytes and returns the offset (ptr).
    /// Returns error if out of memory.
    pub fn alloc(&mut self, size: usize) -> AutoResult<u32> {
        if self.top + size > self.memory.len() {
            return Err(crate::error::RuntimeError::OutOfMemory.into());
        }

        let ptr = self.top as u32;
        self.top += size;
        Ok(ptr)
    }

    /// "Frees" memory by checking if it's at the top (LIFO).
    /// In a strict linear model (like a stack), we can only free the most recently allocated object.
    /// If AutoLang relies on RAII, it usually implies scoping.
    /// For simulation, we might provide a `reset` to a previous benchmark/watermark.
    pub fn free_top(&mut self, size: usize) {
        if size <= self.top {
            self.top -= size;
        } else {
            // Error or clamp?
            self.top = 0;
        }
    }

    pub fn read_bytes(&self, ptr: u32, size: usize) -> &[u8] {
        let start = ptr as usize;
        &self.memory[start..start + size]
    }

    pub fn write_bytes(&mut self, ptr: u32, data: &[u8]) {
        let start = ptr as usize;
        let end = start + data.len();
        if end <= self.memory.len() {
            self.memory[start..end].copy_from_slice(data);
        }
    }
}
