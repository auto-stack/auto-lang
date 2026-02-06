// Plan 076 Phase 4: AutoVM List Storage with Strategy Support
// Unified list storage that supports both Heap and InlineInt64 strategies

use auto_val::Value;
use crate::vm::list_storage::{ListStorage, HeapStorage, InlineInt64Storage};

/// AutoVM list storage with pluggable storage strategy
/// This replaces `crate::universe::ListData` for AutoVM-specific lists
#[derive(Debug)]
pub enum AutoVMListStorage {
    /// Heap-allocated dynamic storage (unlimited capacity)
    Heap(HeapStorage),
    /// Inline storage with fixed 64-element capacity
    InlineInt64(InlineInt64Storage),
}

impl AutoVMListStorage {
    /// Create a new list with the specified storage strategy
    pub fn new(storage: ListStorage) -> Self {
        match storage {
            ListStorage::Heap => AutoVMListStorage::Heap(HeapStorage::new()),
            ListStorage::InlineInt64 => AutoVMListStorage::InlineInt64(InlineInt64Storage::new()),
        }
    }

    /// Create a new list with initial capacity (only for Heap storage)
    pub fn with_capacity(storage: ListStorage, capacity: usize) -> Self {
        match storage {
            ListStorage::Heap => AutoVMListStorage::Heap(HeapStorage::with_capacity(capacity)),
            ListStorage::InlineInt64 => AutoVMListStorage::InlineInt64(InlineInt64Storage::new()),
        }
    }

    /// Get the storage strategy type
    pub fn storage_type(&self) -> ListStorage {
        match self {
            AutoVMListStorage::Heap(_) => ListStorage::Heap,
            AutoVMListStorage::InlineInt64(_) => ListStorage::InlineInt64,
        }
    }

    /// Get the number of elements in the list
    pub fn len(&self) -> usize {
        match self {
            AutoVMListStorage::Heap(storage) => storage.len(),
            AutoVMListStorage::InlineInt64(storage) => storage.len(),
        }
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        match self {
            AutoVMListStorage::Heap(storage) => storage.is_empty(),
            AutoVMListStorage::InlineInt64(storage) => storage.is_empty(),
        }
    }

    /// Get the current capacity
    /// Returns None for unbounded Heap storage
    pub fn capacity(&self) -> Option<usize> {
        match self {
            AutoVMListStorage::Heap(storage) => Some(storage.capacity()),
            AutoVMListStorage::InlineInt64(storage) => Some(storage.capacity()),
        }
    }

    /// Push an element to the end of the list
    /// Returns true if successful, false if capacity exceeded (InlineInt64 only)
    pub fn push(&mut self, elem: Value) -> bool {
        match self {
            AutoVMListStorage::Heap(storage) => {
                storage.push(elem);
                true
            }
            AutoVMListStorage::InlineInt64(storage) => storage.push(elem),
        }
    }

    /// Pop an element from the end of the list
    /// Returns None if the list is empty
    pub fn pop(&mut self) -> Option<Value> {
        match self {
            AutoVMListStorage::Heap(storage) => storage.pop(),
            AutoVMListStorage::InlineInt64(storage) => storage.pop(),
        }
    }

    /// Get a reference to the element at the specified index
    /// Returns None if the index is out of bounds
    pub fn get(&self, index: usize) -> Option<&Value> {
        match self {
            AutoVMListStorage::Heap(storage) => storage.get(index),
            AutoVMListStorage::InlineInt64(storage) => storage.get(index),
        }
    }

    /// Set the element at the specified index
    /// Returns true if successful, false if index is out of bounds
    pub fn set(&mut self, index: usize, elem: Value) -> bool {
        match self {
            AutoVMListStorage::Heap(storage) => storage.set(index, elem),
            AutoVMListStorage::InlineInt64(storage) => storage.set(index, elem),
        }
    }

    /// Insert an element at the specified index
    /// Returns true if successful, false if index is invalid or capacity exceeded
    pub fn insert(&mut self, index: usize, elem: Value) -> bool {
        match self {
            AutoVMListStorage::Heap(storage) => {
                storage.insert(index, elem);
                true
            }
            AutoVMListStorage::InlineInt64(storage) => storage.insert(index, elem),
        }
    }

    /// Remove the element at the specified index
    /// Returns None if the index is out of bounds
    pub fn remove(&mut self, index: usize) -> Option<Value> {
        match self {
            AutoVMListStorage::Heap(storage) => storage.remove(index),
            AutoVMListStorage::InlineInt64(storage) => storage.remove(index),
        }
    }

    /// Clear all elements from the list
    pub fn clear(&mut self) {
        match self {
            AutoVMListStorage::Heap(storage) => storage.clear(),
            AutoVMListStorage::InlineInt64(storage) => storage.clear(),
        }
    }

    /// Reserve additional capacity (only for Heap storage)
    pub fn reserve(&mut self, additional: usize) {
        match self {
            AutoVMListStorage::Heap(storage) => storage.reserve(additional),
            AutoVMListStorage::InlineInt64(_) => {
                // Inline storage has fixed capacity, do nothing
            }
        }
    }

    /// Try to grow to at least min_cap capacity
    /// Returns true if successful
    pub fn try_grow(&mut self, min_cap: usize) -> bool {
        match self {
            AutoVMListStorage::Heap(storage) => storage.try_grow(min_cap),
            AutoVMListStorage::InlineInt64(storage) => storage.try_grow(min_cap),
        }
    }

    /// Convert to Vec<Value> (creates a new Vec)
    /// Useful for interoperability with existing code
    pub fn to_vec(&self) -> Vec<Value> {
        match self {
            AutoVMListStorage::Heap(storage) => storage.elems.clone(),
            AutoVMListStorage::InlineInt64(storage) => {
                storage.buffer[..storage.len].to_vec()
            }
        }
    }

    /// Get direct access to elements as a slice
    /// Returns empty slice for unsupported operations
    pub fn as_slice(&self) -> &[Value] {
        match self {
            AutoVMListStorage::Heap(storage) => &storage.elems,
            AutoVMListStorage::InlineInt64(storage) => &storage.buffer[..storage.len],
        }
    }

    /// Get mutable access to elements (for legacy compatibility)
    /// Note: This creates a new Vec for InlineInt64, changes won't persist
    #[deprecated(note = "Use typed methods instead")]
    pub fn elems_mut(&mut self) -> &mut Vec<Value> {
        match self {
            AutoVMListStorage::Heap(storage) => &mut storage.elems,
            AutoVMListStorage::InlineInt64(_) => {
                // This is a limitation - InlineInt64 can't return &mut Vec<Value>
                // Callers should use the typed methods instead
                static mut DUMMY: Vec<Value> = Vec::new();
                unsafe { &mut DUMMY }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigvm_list_storage_heap() {
        let mut list = AutoVMListStorage::new(ListStorage::Heap);

        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
        assert!(list.push(Value::Int(42)));
        assert!(list.push(Value::Int(100)));

        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0), Some(&Value::Int(42)));
        assert_eq!(list.get(1), Some(&Value::Int(100)));
    }

    #[test]
    fn test_bigvm_list_storage_inline() {
        let mut list = AutoVMListStorage::new(ListStorage::InlineInt64);

        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
        assert_eq!(list.capacity(), Some(64));
        assert!(list.push(Value::Int(42)));
        assert!(list.push(Value::Int(100)));

        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0), Some(&Value::Int(42)));
        assert_eq!(list.get(1), Some(&Value::Int(100)));
    }

    #[test]
    fn test_bigvm_list_storage_inline_capacity_limit() {
        let mut list = AutoVMListStorage::new(ListStorage::InlineInt64);

        // Fill to capacity
        for i in 0..64 {
            assert!(list.push(Value::Int(i)), "Should succeed at index {}", i);
        }

        // Should fail when capacity exceeded
        assert!(!list.push(Value::Int(64)));
        assert_eq!(list.len(), 64);
    }

    #[test]
    fn test_bigvm_list_storage_pop() {
        let mut heap_list = AutoVMListStorage::new(ListStorage::Heap);
        heap_list.push(Value::Int(42));
        heap_list.push(Value::Int(100));

        assert_eq!(heap_list.pop(), Some(Value::Int(100)));
        assert_eq!(heap_list.pop(), Some(Value::Int(42)));
        assert_eq!(heap_list.pop(), None);

        let mut inline_list = AutoVMListStorage::new(ListStorage::InlineInt64);
        inline_list.push(Value::Int(42));
        inline_list.push(Value::Int(100));

        assert_eq!(inline_list.pop(), Some(Value::Int(100)));
        assert_eq!(inline_list.pop(), Some(Value::Int(42)));
        assert_eq!(inline_list.pop(), None);
    }

    #[test]
    fn test_bigvm_list_storage_set_get() {
        let mut heap_list = AutoVMListStorage::new(ListStorage::Heap);
        heap_list.push(Value::Int(1));
        heap_list.push(Value::Int(2));

        assert!(heap_list.set(0, Value::Int(10)));
        assert_eq!(heap_list.get(0), Some(&Value::Int(10)));

        let mut inline_list = AutoVMListStorage::new(ListStorage::InlineInt64);
        inline_list.push(Value::Int(1));
        inline_list.push(Value::Int(2));

        assert!(inline_list.set(0, Value::Int(10)));
        assert_eq!(inline_list.get(0), Some(&Value::Int(10)));
    }

    #[test]
    fn test_bigvm_list_storage_insert_remove() {
        let mut heap_list = AutoVMListStorage::new(ListStorage::Heap);
        heap_list.push(Value::Int(1));
        heap_list.push(Value::Int(3));

        assert!(heap_list.insert(1, Value::Int(2)));
        assert_eq!(heap_list.len(), 3);
        assert_eq!(heap_list.remove(1), Some(Value::Int(2)));
        assert_eq!(heap_list.len(), 2);

        let mut inline_list = AutoVMListStorage::new(ListStorage::InlineInt64);
        inline_list.push(Value::Int(1));
        inline_list.push(Value::Int(3));

        assert!(inline_list.insert(1, Value::Int(2)));
        assert_eq!(inline_list.len(), 3);
        assert_eq!(inline_list.remove(1), Some(Value::Int(2)));
        assert_eq!(inline_list.len(), 2);
    }

    #[test]
    fn test_bigvm_list_storage_try_grow() {
        let mut heap_list = AutoVMListStorage::new(ListStorage::Heap);
        assert_eq!(heap_list.capacity(), Some(0));
        assert!(heap_list.try_grow(16));
        assert!(heap_list.capacity() >= Some(16));

        let mut inline_list = AutoVMListStorage::new(ListStorage::InlineInt64);
        assert!(inline_list.try_grow(32));  // <= 64, should succeed
        assert!(inline_list.try_grow(64));  // == 64, should succeed
        assert!(!inline_list.try_grow(65)); // > 64, should fail
    }

    #[test]
    fn test_bigvm_list_storage_to_vec() {
        let mut heap_list = AutoVMListStorage::new(ListStorage::Heap);
        heap_list.push(Value::Int(1));
        heap_list.push(Value::Int(2));
        heap_list.push(Value::Int(3));

        let vec = heap_list.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], Value::Int(1));
        assert_eq!(vec[1], Value::Int(2));
        assert_eq!(vec[2], Value::Int(3));

        let mut inline_list = AutoVMListStorage::new(ListStorage::InlineInt64);
        inline_list.push(Value::Int(1));
        inline_list.push(Value::Int(2));
        inline_list.push(Value::Int(3));

        let vec = inline_list.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], Value::Int(1));
        assert_eq!(vec[1], Value::Int(2));
        assert_eq!(vec[2], Value::Int(3));
    }

    #[test]
    fn test_bigvm_list_storage_clear() {
        let mut heap_list = AutoVMListStorage::new(ListStorage::Heap);
        heap_list.push(Value::Int(1));
        heap_list.push(Value::Int(2));
        heap_list.clear();

        assert_eq!(heap_list.len(), 0);
        assert!(heap_list.is_empty());

        let mut inline_list = AutoVMListStorage::new(ListStorage::InlineInt64);
        inline_list.push(Value::Int(1));
        inline_list.push(Value::Int(2));
        inline_list.clear();

        assert_eq!(inline_list.len(), 0);
        assert!(inline_list.is_empty());
    }
}
