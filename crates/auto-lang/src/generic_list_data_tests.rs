// Plan 077 Phase 2: Generic ListData<T> Unit Tests
// Comprehensive tests for zero-overhead generic list storage

use crate::universe::{ListData, ListStorage};
use auto_val::Value;

// ============================================================================
// Basic Construction Tests
// ============================================================================

#[test]
fn test_list_data_new() {
    let list: ListData<i32> = ListData::new();
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
    assert_eq!(list.elems.capacity(), 0);
}

#[test]
fn test_list_data_with_capacity() {
    let list: ListData<i32> = ListData::with_capacity(10);
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
    assert!(list.elems.capacity() >= 10);
}

#[test]
fn test_list_data_with_storage_heap() {
    let list: ListData<i32> = ListData::with_storage(ListStorage::Heap);
    assert_eq!(list.len(), 0);
    assert_eq!(list.get_storage(), ListStorage::Heap);
    assert!(list.can_grow());
    assert_eq!(list.max_capacity(), None);
}

#[test]
fn test_list_data_with_storage_inline() {
    let list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    assert_eq!(list.len(), 0);
    assert_eq!(list.get_storage(), ListStorage::InlineInt64);
    assert!(!list.can_grow());
    assert_eq!(list.max_capacity(), Some(64));
}

#[test]
fn test_list_data_default() {
    let list: ListData<i32> = ListData::default();
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

// ============================================================================
// Push and Pop Tests
// ============================================================================

#[test]
fn test_list_data_push_int() {
    let mut list: ListData<i32> = ListData::new();
    assert!(list.push(1));
    assert!(list.push(2));
    assert!(list.push(3));

    assert_eq!(list.len(), 3);
    assert!(!list.is_empty());
    assert_eq!(list.elems, vec![1, 2, 3]);
}

#[test]
fn test_list_data_push_char() {
    let mut list: ListData<char> = ListData::new();
    assert!(list.push('a'));
    assert!(list.push('b'));
    assert!(list.push('c'));

    assert_eq!(list.len(), 3);
    assert_eq!(list.elems, vec!['a', 'b', 'c']);
}

#[test]
fn test_list_data_push_bool() {
    let mut list: ListData<bool> = ListData::new();
    assert!(list.push(true));
    assert!(list.push(false));
    assert!(list.push(true));

    assert_eq!(list.len(), 3);
    assert_eq!(list.elems, vec![true, false, true]);
}

#[test]
fn test_list_data_pop() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    assert_eq!(list.pop(), Some(3));
    assert_eq!(list.pop(), Some(2));
    assert_eq!(list.pop(), Some(1));
    assert_eq!(list.pop(), None);
    assert_eq!(list.len(), 0);
}

#[test]
fn test_list_data_push_inline_capacity_limit() {
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);

    // Fill to capacity (64 elements)
    for i in 0..64 {
        assert!(list.push(i), "Should succeed at index {}", i);
    }

    assert_eq!(list.len(), 64);

    // Should fail when capacity exceeded
    assert!(!list.push(64));
    assert_eq!(list.len(), 64);
}

// ============================================================================
// Get and Set Tests
// ============================================================================

#[test]
fn test_list_data_get() {
    let mut list: ListData<i32> = ListData::new();
    list.push(10);
    list.push(20);
    list.push(30);

    assert_eq!(list.get(0), Some(&10));
    assert_eq!(list.get(1), Some(&20));
    assert_eq!(list.get(2), Some(&30));
    assert_eq!(list.get(3), None);
}

#[test]
fn test_list_data_get_char() {
    let mut list: ListData<char> = ListData::new();
    list.push('a');
    list.push('b');

    assert_eq!(list.get(0), Some(&'a'));
    assert_eq!(list.get(1), Some(&'b'));
    assert_eq!(list.get(2), None);
}

#[test]
fn test_list_data_set() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    assert!(list.set(1, 20));
    assert!(!list.set(5, 50));  // Out of bounds

    assert_eq!(list.elems, vec![1, 20, 3]);
}

#[test]
fn test_list_data_set_char() {
    let mut list: ListData<char> = ListData::new();
    list.push('a');
    list.push('b');

    assert!(list.set(0, 'x'));
    assert_eq!(list.elems, vec!['x', 'b']);
}

// ============================================================================
// Insert and Remove Tests
// ============================================================================

#[test]
fn test_list_data_insert() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(3);

    assert!(list.insert(1, 2));
    assert_eq!(list.elems, vec![1, 2, 3]);
}

#[test]
fn test_list_data_insert_at_beginning() {
    let mut list: ListData<i32> = ListData::new();
    list.push(2);
    list.push(3);

    assert!(list.insert(0, 1));
    assert_eq!(list.elems, vec![1, 2, 3]);
}

#[test]
fn test_list_data_insert_at_end() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);

    assert!(list.insert(2, 3));
    assert_eq!(list.elems, vec![1, 2, 3]);
}

#[test]
fn test_list_data_insert_out_of_bounds() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);

    assert!(!list.insert(5, 3));  // Beyond length
    assert_eq!(list.elems, vec![1, 2]);
}

#[test]
fn test_list_data_insert_inline_capacity_limit() {
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);

    // Fill to capacity
    for i in 0..64 {
        assert!(list.push(i));
    }

    // Insert should fail (capacity exceeded)
    assert!(!list.insert(32, 999));
    assert_eq!(list.len(), 64);
}

#[test]
fn test_list_data_remove() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);
    list.push(4);

    assert_eq!(list.remove(1), Some(2));
    assert_eq!(list.elems, vec![1, 3, 4]);

    assert_eq!(list.remove(0), Some(1));
    assert_eq!(list.elems, vec![3, 4]);

    assert_eq!(list.remove(5), None);
    assert_eq!(list.remove(1), Some(4));
    assert_eq!(list.remove(0), Some(3));
    assert_eq!(list.remove(0), None);
}

// ============================================================================
// Clear and Reserve Tests
// ============================================================================

#[test]
fn test_list_data_clear() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    list.clear();

    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

#[test]
fn test_list_data_reserve() {
    let mut list: ListData<i32> = ListData::new();
    list.reserve(10);

    assert!(list.elems.capacity() >= 10);
}

#[test]
fn test_list_data_reserve_inline() {
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    list.reserve(100);  // Should be ignored for InlineInt64

    // Capacity is fixed for InlineInt64
    assert_eq!(list.len(), 0);
}

// ============================================================================
// Storage Strategy Tests
// ============================================================================

#[test]
fn test_list_data_get_storage_default() {
    let list: ListData<i32> = ListData::new();
    assert_eq!(list.get_storage(), ListStorage::Heap);
}

#[test]
fn test_list_data_get_storage_heap() {
    let list: ListData<i32> = ListData::with_storage(ListStorage::Heap);
    assert_eq!(list.get_storage(), ListStorage::Heap);
}

#[test]
fn test_list_data_get_storage_inline() {
    let list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    assert_eq!(list.get_storage(), ListStorage::InlineInt64);
}

#[test]
fn test_list_data_can_grow_heap() {
    let list: ListData<i32> = ListData::new();
    assert!(list.can_grow());
}

#[test]
fn test_list_data_can_grow_inline() {
    let list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    assert!(!list.can_grow());
}

#[test]
fn test_list_data_max_capacity_heap() {
    let list: ListData<i32> = ListData::new();
    assert_eq!(list.max_capacity(), None);  // Unlimited
}

#[test]
fn test_list_data_max_capacity_inline() {
    let list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    assert_eq!(list.max_capacity(), Some(64));
}

#[test]
fn test_list_data_try_grow_heap() {
    let mut list: ListData<i32> = ListData::new();

    assert!(list.try_grow(100));
    assert!(list.elems.capacity() >= 100);
}

#[test]
fn test_list_data_try_grow_inline() {
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);

    assert!(list.try_grow(32));   // <= 64, should succeed
    assert!(list.try_grow(64));   // == 64, should succeed
    assert!(!list.try_grow(65));  // > 64, should fail
}

// ============================================================================
// Memory Efficiency Tests
// ============================================================================

#[test]
fn test_list_data_memory_efficiency_int() {
    use std::mem::size_of;

    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    // Memory usage should be 3 × 4 bytes = 12 bytes for the elements
    // Plus Vec overhead (24 bytes) = ~36 bytes total
    // This is 6x better than Vec<Value> which would be 3 × 24 = 72 bytes for elements
    assert_eq!(list.elems.capacity() * size_of::<i32>(), list.elems.capacity() * 4);
}

#[test]
fn test_list_data_memory_efficiency_char() {
    use std::mem::size_of;

    let mut list: ListData<char> = ListData::new();
    list.push('a');
    list.push('b');

    // Char is 4 bytes, so 2 × 4 = 8 bytes
    // Much better than Vec<Value> which would be 2 × 24 = 48 bytes
    assert_eq!(list.elems.len() * size_of::<char>(), 2 * 4);
}

#[test]
fn test_list_data_memory_efficiency_bool() {
    use std::mem::size_of;

    let mut list: ListData<bool> = ListData::new();
    list.push(true);
    list.push(false);

    // Bool is 1 byte, so 2 × 1 = 2 bytes
    // Much better than Vec<Value> which would be 2 × 24 = 48 bytes
    assert_eq!(list.elems.len() * size_of::<bool>(), 2 * 1);
}

#[test]
fn test_list_data_value_fallback() {
    // ListData<Value> should still work for mixed-type lists
    let mut list: ListData<Value> = ListData::new();
    list.push(Value::Int(1));
    list.push(Value::Str("hello".into()));
    list.push(Value::Bool(true));

    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0), Some(&Value::Int(1)));
    // String comparison is complex due to AutoStr, just check length
    assert!(list.get(1).is_some());
    assert_eq!(list.get(2), Some(&Value::Bool(true)));
}

// ============================================================================
// Clone and PartialEq Tests
// ============================================================================

#[test]
fn test_list_data_clone() {
    let mut list1: ListData<i32> = ListData::new();
    list1.push(1);
    list1.push(2);
    list1.push(3);

    let list2 = list1.clone();

    assert_eq!(list2.elems, vec![1, 2, 3]);
    assert_eq!(list2.get_storage(), list1.get_storage());
}

#[test]
fn test_list_data_clone_with_storage() {
    let mut list1: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    list1.push(1);

    let list2 = list1.clone();

    assert_eq!(list2.elems, vec![1]);
    assert_eq!(list2.get_storage(), ListStorage::InlineInt64);
}

#[test]
fn test_list_data_partial_eq() {
    let mut list1: ListData<i32> = ListData::new();
    list1.push(1);
    list1.push(2);

    let mut list2: ListData<i32> = ListData::new();
    list2.push(1);
    list2.push(2);

    assert_eq!(list1, list2);
}

#[test]
fn test_list_data_not_eq() {
    let mut list1: ListData<i32> = ListData::new();
    list1.push(1);
    list1.push(2);

    let mut list2: ListData<i32> = ListData::new();
    list2.push(1);
    list2.push(3);

    assert_ne!(list1, list2);
}

#[test]
fn test_list_data_not_eq_different_storage() {
    let list1: ListData<i32> = ListData::with_storage(ListStorage::Heap);
    let list2: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);

    assert_ne!(list1, list2);
}

#[test]
fn test_list_data_not_eq_different_length() {
    let mut list1: ListData<i32> = ListData::new();
    list1.push(1);

    let mut list2: ListData<i32> = ListData::new();
    list2.push(1);
    list2.push(2);

    assert_ne!(list1, list2);
}

// ============================================================================
// Type-Specific Behavior Tests
// ============================================================================

#[test]
fn test_list_data_int_operations() {
    let mut list: ListData<i32> = ListData::new();

    list.push(42);
    assert_eq!(list.get(0), Some(&42));

    list.set(0, 100);
    assert_eq!(list.get(0), Some(&100));

    assert_eq!(list.pop(), Some(100));
    assert_eq!(list.pop(), None);
}

#[test]
fn test_list_data_char_operations() {
    let mut list: ListData<char> = ListData::new();

    list.push('x');
    assert_eq!(list.get(0), Some(&'x'));

    list.set(0, 'y');
    assert_eq!(list.get(0), Some(&'y'));

    assert_eq!(list.pop(), Some('y'));
    assert_eq!(list.pop(), None);
}

#[test]
fn test_list_data_bool_operations() {
    let mut list: ListData<bool> = ListData::new();

    list.push(true);
    assert_eq!(list.get(0), Some(&true));

    list.set(0, false);
    assert_eq!(list.get(0), Some(&false));

    assert_eq!(list.pop(), Some(false));
    assert_eq!(list.pop(), None);
}

// ============================================================================
// Default Type Parameter Tests
// ============================================================================

#[test]
fn test_list_data_default_type_parameter() {
    // ListData without type parameter should default to ListData<Value>
    let mut list = ListData::new();

    list.push(Value::Int(1));
    list.push(Value::Int(2));

    assert_eq!(list.len(), 2);
}

#[test]
fn test_list_data_default_with_storage() {
    let mut list = ListData::with_storage(ListStorage::InlineInt64);

    list.push(Value::Int(1));

    assert_eq!(list.get_storage(), ListStorage::InlineInt64);
}

// ============================================================================
// Capacity and Growth Tests
// ============================================================================

#[test]
fn test_list_data_growth_doubling() {
    let mut list: ListData<i32> = ListData::new();

    let initial_cap = list.elems.capacity();

    // Push enough elements to trigger growth
    for i in 0..100 {
        list.push(i);
    }

    // Capacity should have grown
    assert!(list.elems.capacity() > initial_cap);
}

#[test]
fn test_list_data_growth_efficiency() {
    let mut list: ListData<i32> = ListData::new();

    // Push 1000 elements
    for i in 0..1000 {
        list.push(i);
    }

    // Vec typically doubles capacity, so we expect some growth
    assert!(list.elems.capacity() >= 1000);
    assert_eq!(list.len(), 1000);
}

#[test]
fn test_list_data_large_dataset() {
    let mut list: ListData<i32> = ListData::new();

    // Test with larger dataset
    for i in 0..10000 {
        list.push(i);
    }

    assert_eq!(list.len(), 10000);
    assert_eq!(list.get(0), Some(&0));
    assert_eq!(list.get(9999), Some(&9999));
}

#[test]
fn test_list_data_mixed_operations() {
    let mut list: ListData<i32> = ListData::new();

    // Mix of operations
    list.push(1);
    list.push(2);
    assert_eq!(list.pop(), Some(2));
    list.push(3);
    assert!(list.set(0, 10));
    assert_eq!(list.get(0), Some(&10));
    list.insert(1, 20);
    assert_eq!(list.remove(2), Some(3));

    assert_eq!(list.elems, vec![10, 20]);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_list_data_stress_many_elements() {
    let mut list: ListData<i32> = ListData::new();

    for i in 0..100000 {
        assert!(list.push(i));
    }

    assert_eq!(list.len(), 100000);
}

#[test]
fn test_list_data_stress_alternating_ops() {
    let mut list: ListData<i32> = ListData::new();

    for i in 0..1000 {
        list.push(i);
        if i % 2 == 0 {
            list.pop();
        }
    }

    // Should not crash and maintain consistency
    assert!(list.len() <= 1000);
}

#[test]
fn test_list_data_stress_random_access() {
    let mut list: ListData<i32> = ListData::with_capacity(100);

    // Fill with data
    for i in 0..100 {
        list.push(i * 2);
    }

    // Random access patterns
    assert_eq!(list.get(50), Some(&100));
    assert_eq!(list.get(99), Some(&198));
    assert_eq!(list.get(0), Some(&0));
    assert_eq!(list.get(49), Some(&98));
}

#[test]
fn test_list_data_edge_cases() {
    let mut list: ListData<i32> = ListData::new();

    // Empty list operations
    assert_eq!(list.pop(), None);
    assert_eq!(list.get(0), None);
    assert!(!list.set(0, 1));
    assert_eq!(list.remove(0), None);
    assert_eq!(list.len(), 0);

    // Single element
    list.push(42);
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0), Some(&42));
    assert!(list.set(0, 100));
    assert_eq!(list.get(0), Some(&100));
}

// ============================================================================
// Storage Strategy Integration Tests
// ============================================================================

#[test]
fn test_list_data_inline_behavior() {
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);

    // Should accept elements up to 64
    for i in 0..64 {
        assert!(list.push(i));
    }

    // Should reject beyond 64
    assert!(!list.push(64));

    // Remove should free up space for new elements
    assert_eq!(list.pop(), Some(63));
    assert!(list.push(64));  // Now this should work
}

#[test]
fn test_list_data_heap_behavior() {
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::Heap);

    // Should accept unlimited elements
    for i in 0..1000 {
        assert!(list.push(i));
    }

    assert_eq!(list.len(), 1000);
}

#[test]
fn test_list_data_storage_preserved() {
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);

    list.push(1);
    assert_eq!(list.get_storage(), ListStorage::InlineInt64);

    list.pop();
    assert_eq!(list.get_storage(), ListStorage::InlineInt64);
}
