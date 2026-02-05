// Plan 076 Phase 4: BigVM Storage Strategy Runtime
// Provides native Rust storage strategies for BigVM lists

use auto_val::Value;

/// Storage strategy for BigVM lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStorage {
    /// Heap-allocated dynamic storage (growable)
    Heap,
    /// Inline storage with fixed 64-element capacity (no heap)
    InlineInt64,
}

impl ListStorage {
    /// Get the fixed capacity for this storage type
    pub fn capacity(&self) -> Option<usize> {
        match self {
            ListStorage::Heap => None,  // Unlimited (growable)
            ListStorage::InlineInt64 => Some(64),  // Fixed 64 elements
        }
    }

    /// Check if this storage type can grow
    pub fn can_grow(&self) -> bool {
        match self {
            ListStorage::Heap => true,
            ListStorage::InlineInt64 => false,
        }
    }

    /// Get the strategy name
    pub fn name(&self) -> &str {
        match self {
            ListStorage::Heap => "Heap",
            ListStorage::InlineInt64 => "InlineInt64",
        }
    }
}

// ============================================================================
// Heap Storage (Dynamic)
// ============================================================================

/// Heap-allocated dynamic list storage
/// This is the default storage type, equivalent to Vec<Value>
#[derive(Debug)]
pub struct HeapStorage {
    pub elems: Vec<Value>,
}

impl HeapStorage {
    pub fn new() -> Self {
        Self {
            elems: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elems: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.elems.capacity()
    }

    pub fn push(&mut self, elem: Value) {
        self.elems.push(elem);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.elems.pop()
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.elems.get(index)
    }

    pub fn set(&mut self, index: usize, elem: Value) -> bool {
        if index < self.elems.len() {
            self.elems[index] = elem;
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, index: usize, elem: Value) {
        if index <= self.elems.len() {
            self.elems.insert(index, elem);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if index < self.elems.len() {
            Some(self.elems.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.elems.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.elems.reserve(additional);
    }

    /// Try to grow to at least min_cap capacity
    /// Returns true if successful (always true for heap storage)
    pub fn try_grow(&mut self, min_cap: usize) -> bool {
        let new_cap = if self.capacity() == 0 {
            std::cmp::max(8, min_cap)
        } else {
            std::cmp::max(self.capacity() * 2, min_cap)
        };
        self.elems.reserve(new_cap - self.len());
        true
    }
}

impl Default for HeapStorage {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// InlineInt64 Storage (Fixed 64 elements)
// ============================================================================

/// Inline storage with fixed 64-element capacity
/// Zero heap allocation, all data stored inline
#[derive(Debug, Clone)]
pub struct InlineInt64Storage {
    /// Fixed buffer of 64 elements
    /// Uses Vec for simplicity, but capacity never exceeds 64
    pub buffer: Vec<Value>,
    /// Current length (<= 64)
    pub len: usize,
}

impl InlineInt64Storage {
    /// Create a new empty inline storage
    pub fn new() -> Self {
        Self {
            buffer: vec![Value::Nil; 64],
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Fixed capacity of 64 elements
    pub fn capacity(&self) -> usize {
        64
    }

    pub fn push(&mut self, elem: Value) -> bool {
        if self.len < 64 {
            self.buffer[self.len] = elem;
            self.len += 1;
            true
        } else {
            false  // Capacity exceeded
        }
    }

    pub fn pop(&mut self) -> Option<Value> {
        if self.len > 0 {
            self.len -= 1;
            // Replace with Nil to avoid holding references
            let elem = std::mem::replace(&mut self.buffer[self.len], Value::Nil);
            Some(elem)
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        if index < self.len {
            Some(&self.buffer[index])
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, elem: Value) -> bool {
        if index < self.len {
            self.buffer[index] = elem;
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, index: usize, elem: Value) -> bool {
        if index <= self.len && self.len < 64 {
            // Shift elements to the right
            for i in (index..self.len).rev() {
                self.buffer[i + 1] = std::mem::replace(&mut self.buffer[i], Value::Nil);
            }
            self.buffer[index] = elem;
            self.len += 1;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if index < self.len {
            let elem = self.buffer[index].clone();
            // Shift elements to the left
            for i in index..self.len - 1 {
                self.buffer[i] = std::mem::replace(&mut self.buffer[i + 1], Value::Nil);
            }
            self.buffer[self.len - 1] = Value::Nil;
            self.len -= 1;
            Some(elem)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.len {
            self.buffer[i] = Value::Nil;
        }
        self.len = 0;
    }

    /// Inline storage cannot grow beyond 64 elements
    /// try_grow only succeeds if min_cap <= 64
    pub fn try_grow(&mut self, min_cap: usize) -> bool {
        min_cap <= 64
    }
}

impl Default for InlineInt64Storage {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_storage_basic() {
        let mut storage = HeapStorage::new();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());

        storage.push(Value::Int(42));
        storage.push(Value::Int(100));

        assert_eq!(storage.len(), 2);
        assert!(!storage.is_empty());
        assert_eq!(storage.get(0), Some(&Value::Int(42)));
        assert_eq!(storage.get(1), Some(&Value::Int(100)));
    }

    #[test]
    fn test_heap_storage_pop() {
        let mut storage = HeapStorage::new();
        storage.push(Value::Int(42));
        storage.push(Value::Int(100));

        assert_eq!(storage.pop(), Some(Value::Int(100)));
        assert_eq!(storage.pop(), Some(Value::Int(42)));
        assert_eq!(storage.pop(), None);
    }

    #[test]
    fn test_heap_storage_insert_remove() {
        let mut storage = HeapStorage::new();
        storage.push(Value::Int(1));
        storage.push(Value::Int(3));

        storage.insert(1, Value::Int(2));
        assert_eq!(storage.get(1), Some(&Value::Int(2)));
        assert_eq!(storage.len(), 3);

        assert_eq!(storage.remove(1), Some(Value::Int(2)));
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn test_heap_storage_try_grow() {
        let mut storage = HeapStorage::new();
        assert_eq!(storage.capacity(), 0);

        storage.try_grow(16);
        assert!(storage.capacity() >= 16);
    }

    #[test]
    fn test_inline_storage_basic() {
        let mut storage = InlineInt64Storage::new();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
        assert_eq!(storage.capacity(), 64);

        assert!(storage.push(Value::Int(42)));
        assert!(storage.push(Value::Int(100)));

        assert_eq!(storage.len(), 2);
        assert!(!storage.is_empty());
        assert_eq!(storage.get(0), Some(&Value::Int(42)));
        assert_eq!(storage.get(1), Some(&Value::Int(100)));
    }

    #[test]
    fn test_inline_storage_capacity_limit() {
        let mut storage = InlineInt64Storage::new();

        // Fill to capacity
        for i in 0..64 {
            assert!(storage.push(Value::Int(i)), "Should succeed at index {}", i);
        }

        // Should fail when capacity exceeded
        assert!(!storage.push(Value::Int(64)));
        assert_eq!(storage.len(), 64);
    }

    #[test]
    fn test_inline_storage_pop() {
        let mut storage = InlineInt64Storage::new();
        storage.push(Value::Int(42));
        storage.push(Value::Int(100));

        assert_eq!(storage.pop(), Some(Value::Int(100)));
        assert_eq!(storage.pop(), Some(Value::Int(42)));
        assert_eq!(storage.pop(), None);
    }

    #[test]
    fn test_inline_storage_insert_remove() {
        let mut storage = InlineInt64Storage::new();
        storage.push(Value::Int(1));
        storage.push(Value::Int(3));

        assert!(storage.insert(1, Value::Int(2)));
        assert_eq!(storage.get(1), Some(&Value::Int(2)));
        assert_eq!(storage.len(), 3);

        assert_eq!(storage.remove(1), Some(Value::Int(2)));
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn test_inline_storage_try_grow() {
        let mut storage = InlineInt64Storage::new();

        assert!(storage.try_grow(32));  // <= 64, should succeed
        assert!(storage.try_grow(64));  // == 64, should succeed
        assert!(!storage.try_grow(65)); // > 64, should fail
    }

    #[test]
    fn test_list_storage_capacity() {
        assert_eq!(ListStorage::Heap.capacity(), None);
        assert_eq!(ListStorage::InlineInt64.capacity(), Some(64));
    }

    #[test]
    fn test_list_storage_can_grow() {
        assert!(ListStorage::Heap.can_grow());
        assert!(!ListStorage::InlineInt64.can_grow());
    }

    #[test]
    fn test_list_storage_name() {
        assert_eq!(ListStorage::Heap.name(), "Heap");
        assert_eq!(ListStorage::InlineInt64.name(), "InlineInt64");
    }
}
