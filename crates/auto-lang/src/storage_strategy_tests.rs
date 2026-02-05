// Plan 076 Phase 4: Storage Strategy Runtime Tests
// Tests for List<T, S> storage strategies in BigVM

use crate::vm::list_storage::{ListStorage, HeapStorage, InlineInt64Storage};
use crate::universe::{ListData, ListStorage as UnivListStorage};

#[test]
fn test_list_storage_enum() {
    assert_eq!(ListStorage::Heap.name(), "Heap");
    assert_eq!(ListStorage::InlineInt64.name(), "InlineInt64");

    assert!(ListStorage::Heap.can_grow());
    assert!(!ListStorage::InlineInt64.can_grow());

    assert_eq!(ListStorage::Heap.capacity(), None);
    assert_eq!(ListStorage::InlineInt64.capacity(), Some(64));
}

#[test]
fn test_heap_storage_basic() {
    let mut storage = HeapStorage::new();

    assert_eq!(storage.len(), 0);
    assert!(storage.is_empty());
    assert_eq!(storage.capacity(), 0);

    storage.push(auto_val::Value::Int(42));
    storage.push(auto_val::Value::Int(100));

    assert_eq!(storage.len(), 2);
    assert!(!storage.is_empty());
    assert_eq!(storage.get(0), Some(&auto_val::Value::Int(42)));
    assert_eq!(storage.get(1), Some(&auto_val::Value::Int(100)));
}

#[test]
fn test_heap_storage_capacity() {
    let mut storage = HeapStorage::new();
    assert_eq!(storage.capacity(), 0);

    storage.reserve(100);
    assert!(storage.capacity() >= 100);

    storage.try_grow(200);
    assert!(storage.capacity() >= 200);
}

#[test]
fn test_inline_storage_basic() {
    let mut storage = InlineInt64Storage::new();

    assert_eq!(storage.len(), 0);
    assert!(storage.is_empty());
    assert_eq!(storage.capacity(), 64);

    assert!(storage.push(auto_val::Value::Int(42)));
    assert!(storage.push(auto_val::Value::Int(100)));

    assert_eq!(storage.len(), 2);
    assert_eq!(storage.get(0), Some(&auto_val::Value::Int(42)));
    assert_eq!(storage.get(1), Some(&auto_val::Value::Int(100)));
}

#[test]
fn test_inline_storage_capacity_limit() {
    let mut storage = InlineInt64Storage::new();

    // Fill to capacity
    for i in 0..64 {
        assert!(storage.push(auto_val::Value::Int(i)), "Should succeed at index {}", i);
    }

    // Should fail when capacity exceeded
    assert!(!storage.push(auto_val::Value::Int(64)));
    assert_eq!(storage.len(), 64);
}

#[test]
fn test_inline_storage_pop() {
    let mut storage = InlineInt64Storage::new();
    storage.push(auto_val::Value::Int(42));
    storage.push(auto_val::Value::Int(100));

    assert_eq!(storage.pop(), Some(auto_val::Value::Int(100)));
    assert_eq!(storage.pop(), Some(auto_val::Value::Int(42)));
    assert_eq!(storage.pop(), None);
}

#[test]
fn test_inline_storage_set_get() {
    let mut storage = InlineInt64Storage::new();
    storage.push(auto_val::Value::Int(1));
    storage.push(auto_val::Value::Int(2));

    assert!(storage.set(0, auto_val::Value::Int(10)));
    assert_eq!(storage.get(0), Some(&auto_val::Value::Int(10)));
    assert_eq!(storage.get(1), Some(&auto_val::Value::Int(2)));

    // Out of bounds
    assert!(!storage.set(5, auto_val::Value::Int(99)));
    assert_eq!(storage.get(5), None);
}

#[test]
fn test_inline_storage_insert_remove() {
    let mut storage = InlineInt64Storage::new();
    storage.push(auto_val::Value::Int(1));
    storage.push(auto_val::Value::Int(3));

    assert!(storage.insert(1, auto_val::Value::Int(2)));
    assert_eq!(storage.len(), 3);
    assert_eq!(storage.get(1), Some(&auto_val::Value::Int(2)));

    assert_eq!(storage.remove(1), Some(auto_val::Value::Int(2)));
    assert_eq!(storage.len(), 2);
    assert_eq!(storage.get(1), Some(&auto_val::Value::Int(3)));
}

#[test]
fn test_inline_storage_clear() {
    let mut storage = InlineInt64Storage::new();
    storage.push(auto_val::Value::Int(1));
    storage.push(auto_val::Value::Int(2));
    storage.push(auto_val::Value::Int(3));

    storage.clear();

    assert_eq!(storage.len(), 0);
    assert!(storage.is_empty());
}

#[test]
fn test_list_data_heap_default() {
    let list = ListData::new();

    assert_eq!(list.get_storage(), UnivListStorage::Heap);
    assert!(list.can_grow());
    assert_eq!(list.max_capacity(), None);
}

#[test]
fn test_list_data_inline() {
    let list = ListData::with_storage(UnivListStorage::InlineInt64);

    assert_eq!(list.get_storage(), UnivListStorage::InlineInt64);
    assert!(!list.can_grow());
    assert_eq!(list.max_capacity(), Some(64));
}

#[test]
fn test_list_data_heap_push() {
    let mut list = ListData::new();

    assert!(list.push(auto_val::Value::Int(1)));
    assert!(list.push(auto_val::Value::Int(2)));

    assert_eq!(list.len(), 2);
}

#[test]
fn test_list_data_inline_push() {
    let mut list = ListData::with_storage(UnivListStorage::InlineInt64);

    // Can push up to 64 elements
    for i in 0..64 {
        assert!(list.push(auto_val::Value::Int(i)), "Should succeed at index {}", i);
    }

    // Should fail on 65th element
    assert!(!list.push(auto_val::Value::Int(64)));
    assert_eq!(list.len(), 64);
}

#[test]
fn test_list_data_inline_set_get() {
    let mut list = ListData::with_storage(UnivListStorage::InlineInt64);

    list.push(auto_val::Value::Int(1));
    list.push(auto_val::Value::Int(2));

    assert!(list.set(0, auto_val::Value::Int(10)));
    assert_eq!(list.get(0), Some(&auto_val::Value::Int(10)));

    // Out of bounds
    assert!(!list.set(10, auto_val::Value::Int(99)));
}

#[test]
fn test_list_data_inline_insert() {
    let mut list = ListData::with_storage(UnivListStorage::InlineInt64);

    list.push(auto_val::Value::Int(1));
    list.push(auto_val::Value::Int(3));

    assert!(list.insert(1, auto_val::Value::Int(2)));
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(1), Some(&auto_val::Value::Int(2)));

    // Should fail if capacity exceeded
    for _ in 3..64 {
        list.push(auto_val::Value::Int(0));
    }
    assert!(!list.insert(0, auto_val::Value::Int(99)));
}

#[test]
fn test_list_data_try_grow_heap() {
    let mut list = ListData::new();

    assert!(list.try_grow(100));
    assert!(list.can_grow());
}

#[test]
fn test_list_data_try_grow_inline() {
    let mut list = ListData::with_storage(UnivListStorage::InlineInt64);

    assert!(list.try_grow(32));  // <= 64, should succeed
    assert!(list.try_grow(64));  // == 64, should succeed
    assert!(!list.try_grow(65)); // > 64, should fail
}

#[test]
fn test_list_data_reserve_inline() {
    let mut list = ListData::with_storage(UnivListStorage::InlineInt64);

    // reserve should be a no-op for InlineInt64
    list.reserve(100);
    assert_eq!(list.max_capacity(), Some(64));
}

#[test]
fn test_list_data_pop() {
    let mut heap_list = ListData::new();
    heap_list.push(auto_val::Value::Int(1));
    heap_list.push(auto_val::Value::Int(2));

    assert_eq!(heap_list.pop(), Some(auto_val::Value::Int(2)));
    assert_eq!(heap_list.pop(), Some(auto_val::Value::Int(1)));
    assert_eq!(heap_list.pop(), None);

    let mut inline_list = ListData::with_storage(UnivListStorage::InlineInt64);
    inline_list.push(auto_val::Value::Int(1));
    inline_list.push(auto_val::Value::Int(2));

    assert_eq!(inline_list.pop(), Some(auto_val::Value::Int(2)));
    assert_eq!(inline_list.pop(), Some(auto_val::Value::Int(1)));
    assert_eq!(inline_list.pop(), None);
}

#[test]
fn test_list_data_clear() {
    let mut heap_list = ListData::new();
    heap_list.push(auto_val::Value::Int(1));
    heap_list.push(auto_val::Value::Int(2));
    heap_list.clear();

    assert_eq!(heap_list.len(), 0);
    assert!(heap_list.is_empty());

    let mut inline_list = ListData::with_storage(UnivListStorage::InlineInt64);
    inline_list.push(auto_val::Value::Int(1));
    inline_list.push(auto_val::Value::Int(2));
    inline_list.clear();

    assert_eq!(inline_list.len(), 0);
    assert!(inline_list.is_empty());
}
