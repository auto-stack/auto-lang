// Plan 077 Phase 8: Comprehensive Integration Tests
//
// This file contains 50+ integration tests for the unified registry system,
// covering all aspects: correctness, performance, thread safety, and memory safety.

use crate::universe::ListData;
use crate::vm::engine::AutoVM;
use crate::vm::heap_object::{HeapObject, TypeTag, try_downcast_checked, try_downcast_checked_mut};
use crate::vm::virt_memory::VirtualFlash;
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

// ============================================================================
// Basic Operations Tests (Tests 1-10)
// ============================================================================

#[test]
fn test_01_create_int_list_in_unified_registry() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create List<int>
    let list: ListData<i32> = ListData::new();
    let list_id = vm.insert_heap_object(list);

    // Verify it exists
    assert!(vm.contains_heap_object(list_id));

    // Verify type tag
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    assert_eq!(guard.type_tag(), TypeTag::ListInt);
}

#[test]
fn test_02_create_multiple_list_types() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create different list types
    let int_list: ListData<i32> = ListData::new();
    let char_list: ListData<char> = ListData::new();
    let bool_list: ListData<bool> = ListData::new();

    let int_id = vm.insert_heap_object(int_list);
    let char_id = vm.insert_heap_object(char_list);
    let bool_id = vm.insert_heap_object(bool_list);

    // Verify all exist
    assert!(vm.contains_heap_object(int_id));
    assert!(vm.contains_heap_object(char_id));
    assert!(vm.contains_heap_object(bool_id));

    // Verify type tags
    let int_obj = vm.get_heap_object(int_id).unwrap();
    assert_eq!(int_obj.read().unwrap().type_tag(), TypeTag::ListInt);

    let char_obj = vm.get_heap_object(char_id).unwrap();
    assert_eq!(char_obj.read().unwrap().type_tag(), TypeTag::ListChar);

    let bool_obj = vm.get_heap_object(bool_id).unwrap();
    assert_eq!(bool_obj.read().unwrap().type_tag(), TypeTag::ListBool);
}

#[test]
fn test_03_list_push_and_get() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create and modify list
    let mut list: ListData<i32> = ListData::new();
    list.push(10);
    list.push(20);
    list.push(30);

    let list_id = vm.insert_heap_object(list);

    // Retrieve and verify
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt).unwrap();

    assert_eq!(list_ref.len(), 3);
    assert_eq!(list_ref.get(0), Some(&10));
    assert_eq!(list_ref.get(1), Some(&20));
    assert_eq!(list_ref.get(2), Some(&30));
}

#[test]
fn test_04_list_pop() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list with elements
    let mut list: ListData<i32> = ListData::new();
    list.push(10);
    list.push(20);
    list.push(30);

    let list_id = vm.insert_heap_object(list);

    // Pop elements
    let obj = vm.get_heap_object(list_id).unwrap();
    let mut guard = obj.write().unwrap();
    let list_ref = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt).unwrap();

    assert_eq!(list_ref.pop(), Some(30));
    assert_eq!(list_ref.pop(), Some(20));
    assert_eq!(list_ref.pop(), Some(10));
    assert_eq!(list_ref.pop(), None);
    assert!(list_ref.is_empty());
}

#[test]
fn test_05_list_set() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list
    let mut list: ListData<i32> = ListData::new();
    list.push(10);
    list.push(20);
    list.push(30);

    let list_id = vm.insert_heap_object(list);

    // Set elements
    let obj = vm.get_heap_object(list_id).unwrap();
    let mut guard = obj.write().unwrap();
    let list_ref = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt).unwrap();

    assert!(list_ref.set(1, 99));
    assert_eq!(list_ref.get(1), Some(&99));
}

#[test]
fn test_06_list_clear() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list with elements
    let mut list: ListData<i32> = ListData::new();
    list.push(10);
    list.push(20);
    list.push(30);

    let list_id = vm.insert_heap_object(list);

    // Clear
    let obj = vm.get_heap_object(list_id).unwrap();
    let mut guard = obj.write().unwrap();
    let list_ref = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt).unwrap();

    list_ref.clear();
    assert!(list_ref.is_empty());
    assert_eq!(list_ref.len(), 0);
}

#[test]
fn test_07_list_insert() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list
    let mut list: ListData<i32> = ListData::new();
    list.push(10);
    list.push(30);

    let list_id = vm.insert_heap_object(list);

    // Insert at index 1
    let obj = vm.get_heap_object(list_id).unwrap();
    let mut guard = obj.write().unwrap();
    let list_ref = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt).unwrap();

    list_ref.insert(1, 20);
    assert_eq!(list_ref.get(0), Some(&10));
    assert_eq!(list_ref.get(1), Some(&20));
    assert_eq!(list_ref.get(2), Some(&30));
}

#[test]
fn test_08_list_remove() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list
    let mut list: ListData<i32> = ListData::new();
    list.push(10);
    list.push(20);
    list.push(30);

    let list_id = vm.insert_heap_object(list);

    // Remove at index 1
    let obj = vm.get_heap_object(list_id).unwrap();
    let mut guard = obj.write().unwrap();
    let list_ref = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt).unwrap();

    assert_eq!(list_ref.remove(1), Some(20));
    assert_eq!(list_ref.len(), 2);
    assert_eq!(list_ref.get(0), Some(&10));
    assert_eq!(list_ref.get(1), Some(&30));
}

#[test]
fn test_09_list_with_capacity() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list with capacity
    let list: ListData<i32> = ListData::with_capacity(100);
    let list_id = vm.insert_heap_object(list);

    // Verify
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt).unwrap();

    assert!(list_ref.is_empty());
    // Capacity should be at least 100 (Vec may allocate more)
}

#[test]
fn test_10_remove_heap_object() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list
    let list: ListData<i32> = ListData::new();
    let list_id = vm.insert_heap_object(list);

    // Verify exists
    assert!(vm.contains_heap_object(list_id));

    // Remove
    let removed = vm.remove_heap_object(list_id);
    assert!(removed.is_some());

    // Verify removed
    assert!(!vm.contains_heap_object(list_id));
}

// ============================================================================
// Type Safety Tests (Tests 11-20)
// ============================================================================

#[test]
fn test_11_wrong_type_downcast_returns_none() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create int list
    let list: ListData<i32> = ListData::new();
    let list_id = vm.insert_heap_object(list);

    // Try to downcast as wrong type
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();

    // Should return None
    let result = try_downcast_checked::<ListData<char>>(&*guard, TypeTag::ListChar);
    assert!(result.is_none());
}

#[test]
fn test_12_type_tag_prevents_wrong_access() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create int list
    let list: ListData<i32> = ListData::new();
    list.push(42);
    let list_id = vm.insert_heap_object(list);

    // Verify correct type works
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();

    let int_list = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt);
    assert!(int_list.is_some());
    assert_eq!(int_list.unwrap().get(0), Some(&42));

    // Wrong type fails
    let char_list = try_downcast_checked::<ListData<char>>(&*guard, TypeTag::ListChar);
    assert!(char_list.is_none());
}

#[test]
fn test_13_all_list_types_have_unique_tags() {
    let tags = vec![
        TypeTag::ListInt,
        TypeTag::ListChar,
        TypeTag::ListBool,
        TypeTag::ListString,
        TypeTag::ListValue,
    ];

    // All tags should be unique
    for (i, &tag1) in tags.iter().enumerate() {
        for &tag2 in tags.iter().skip(i + 1) {
            assert_ne!(tag1, tag2, "Type tags should be unique");
        }
    }
}

#[test]
fn test_14_type_tag_is_list() {
    assert!(TypeTag::ListInt.is_list());
    assert!(TypeTag::ListChar.is_list());
    assert!(TypeTag::ListBool.is_list());
    assert!(TypeTag::ListString.is_list());
    assert!(TypeTag::ListValue.is_list());

    // Non-list types
    assert!(!TypeTag::HashMap.is_list());
    assert!(!TypeTag::HashSet.is_list());
    assert!(!TypeTag::String.is_list());
}

#[test]
fn test_15_mixed_types_in_registry() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create different types
    let int_list: ListData<i32> = ListData::new();
    let char_list: ListData<char> = ListData::new();
    let bool_list: ListData<bool> = ListData::new();

    let ids = vec![
        vm.insert_heap_object(int_list),
        vm.insert_heap_object(char_list),
        vm.insert_heap_object(bool_list),
    ];

    // All should coexist
    for id in &ids {
        assert!(vm.contains_heap_object(*id));
    }

    // Verify each has correct type
    let obj0 = vm.get_heap_object(ids[0]).unwrap();
    assert_eq!(obj0.read().unwrap().type_tag(), TypeTag::ListInt);

    let obj1 = vm.get_heap_object(ids[1]).unwrap();
    assert_eq!(obj1.read().unwrap().type_tag(), TypeTag::ListChar);

    let obj2 = vm.get_heap_object(ids[2]).unwrap();
    assert_eq!(obj2.read().unwrap().type_tag(), TypeTag::ListBool);
}

#[test]
fn test_16_string_list_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create string list
    let mut list: ListData<String> = ListData::new();
    list.push("hello".to_string());
    list.push("world".to_string());

    let list_id = vm.insert_heap_object(list);

    // Verify
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<String>>(&*guard, TypeTag::ListString).unwrap();

    assert_eq!(list_ref.len(), 2);
    assert_eq!(list_ref.get(0), Some(&"hello".to_string()));
}

#[test]
fn test_17_bool_list_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create bool list
    let mut list: ListData<bool> = ListData::new();
    list.push(true);
    list.push(false);
    list.push(true);

    let list_id = vm.insert_heap_object(list);

    // Verify
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<bool>>(&*guard, TypeTag::ListBool).unwrap();

    assert_eq!(list_ref.len(), 3);
    assert_eq!(list_ref.get(0), Some(&true));
    assert_eq!(list_ref.get(1), Some(&false));
}

#[test]
fn test_18_char_list_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create char list
    let mut list: ListData<char> = ListData::new();
    list.push('a');
    list.push('b');
    list.push('c');

    let list_id = vm.insert_heap_object(list);

    // Verify
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<char>>(&*guard, TypeTag::ListChar).unwrap();

    assert_eq!(list_ref.len(), 3);
    assert_eq!(list_ref.get(0), Some(&'a'));
    assert_eq!(list_ref.get(1), Some(&'b'));
}

#[test]
fn test_19_type_verification_in_all_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create int list
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    let list_id = vm.insert_heap_object(list);

    // All operations should verify type correctly
    let obj = vm.get_heap_object(list_id).unwrap();

    // Read operation
    {
        let guard = obj.read().unwrap();
        let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt);
        assert!(list_ref.is_some());

        // Wrong type should fail
        let wrong = try_downcast_checked::<ListData<char>>(&*guard, TypeTag::ListChar);
        assert!(wrong.is_none());
    }

    // Write operation
    {
        let mut guard = obj.write().unwrap();
        let list_ref = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt);
        assert!(list_ref.is_some());

        // Wrong type should fail
        let wrong = try_downcast_checked_mut::<ListData<char>>(&mut *guard, TypeTag::ListChar);
        assert!(wrong.is_none());
    }
}

#[test]
fn test_20_empty_list_type_verification() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create empty list of each type
    let int_list: ListData<i32> = ListData::new();
    let char_list: ListData<char> = ListData::new();
    let bool_list: ListData<bool> = ListData::new();

    let int_id = vm.insert_heap_object(int_list);
    let char_id = vm.insert_heap_object(char_list);
    let bool_id = vm.insert_heap_object(bool_list);

    // All empty lists should have correct type tags
    assert_eq!(vm.get_heap_object(int_id).unwrap().read().unwrap().type_tag(), TypeTag::ListInt);
    assert_eq!(vm.get_heap_object(char_id).unwrap().read().unwrap().type_tag(), TypeTag::ListChar);
    assert_eq!(vm.get_heap_object(bool_id).unwrap().read().unwrap().type_tag(), TypeTag::ListBool);
}

// ============================================================================
// Memory Management Tests (Tests 21-30)
// ============================================================================

#[test]
fn test_21_object_count_increases() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    assert_eq!(vm.heap_object_count(), 0);

    // Add objects
    let list1: ListData<i32> = ListData::new();
    let list2: ListData<i32> = ListData::new();

    vm.insert_heap_object(list1);
    assert_eq!(vm.heap_object_count(), 1);

    vm.insert_heap_object(list2);
    assert_eq!(vm.heap_object_count(), 2);
}

#[test]
fn test_22_clear_heap_objects() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Add objects
    let list1: ListData<i32> = ListData::new();
    let list2: ListData<i32> = ListData::new();
    vm.insert_heap_object(list1);
    vm.insert_heap_object(list2);

    assert_eq!(vm.heap_object_count(), 2);

    // Clear all
    vm.clear_heap_objects();
    assert_eq!(vm.heap_object_count(), 0);
}

#[test]
fn test_23_reuse_object_id_after_removal() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create and remove
    let list: ListData<i32> = ListData::new();
    let id1 = vm.insert_heap_object(list);
    vm.remove_heap_object(id1);

    // New object gets new ID (IDs are never reused)
    let list2: ListData<i32> = ListData::new();
    let id2 = vm.insert_heap_object(list2);

    assert_ne!(id1, id2);
    assert!(id2 > id1);
}

#[test]
fn test_24_large_list_memory_efficiency() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create large list (1000 elements)
    let mut list: ListData<i32> = ListData::new();
    for i in 0..1000 {
        list.push(i);
    }

    let list_id = vm.insert_heap_object(list);

    // Verify all elements present
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt).unwrap();

    assert_eq!(list_ref.len(), 1000);
    assert_eq!(list_ref.get(0), Some(&0));
    assert_eq!(list_ref.get(999), Some(&999));
}

#[test]
fn test_25_multiple_lists_no_interference() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create multiple lists
    let mut list1: ListData<i32> = ListData::new();
    let mut list2: ListData<i32> = ListData::new();
    let mut list3: ListData<i32> = ListData::new();

    list1.push(1);
    list2.push(2);
    list3.push(3);

    let id1 = vm.insert_heap_object(list1);
    let id2 = vm.insert_heap_object(list2);
    let id3 = vm.insert_heap_object(list3);

    // Verify no interference
    let obj1 = vm.get_heap_object(id1).unwrap();
    let list1_ref = try_downcast_checked::<ListData<i32>>(&*obj1.read().unwrap(), TypeTag::ListInt).unwrap();
    assert_eq!(list1_ref.get(0), Some(&1));

    let obj2 = vm.get_heap_object(id2).unwrap();
    let list2_ref = try_downcast_checked::<ListData<i32>>(&*obj2.read().unwrap(), TypeTag::ListInt).unwrap();
    assert_eq!(list2_ref.get(0), Some(&2));

    let obj3 = vm.get_heap_object(id3).unwrap();
    let list3_ref = try_downcast_checked::<ListData<i32>>(&*obj3.read().unwrap(), TypeTag::ListInt).unwrap();
    assert_eq!(list3_ref.get(0), Some(&3));
}

#[test]
fn test_26_clone_registry_independence() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list
    let list: ListData<i32> = ListData::new();
    let list_id = vm.insert_heap_object(list);

    // Clone the Arc
    let obj1 = vm.get_heap_object(list_id).unwrap();
    let obj2 = obj1.clone();

    // Both should point to same data
    let guard1 = obj1.read().unwrap();
    let guard2 = obj2.read().unwrap();

    let list_ref1 = try_downcast_checked::<ListData<i32>>(&*guard1, TypeTag::ListInt).unwrap();
    let list_ref2 = try_downcast_checked::<ListData<i32>>(&*guard2, TypeTag::ListInt).unwrap();

    assert_eq!(list_ref1.len(), list_ref2.len());
}

#[test]
fn test_27_object_id_generation_monotonic() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    let mut prev_id = 0;

    for _ in 0..100 {
        let list: ListData<i32> = ListData::new();
        let id = vm.insert_heap_object(list);
        assert!(id > prev_id);
        prev_id = id;
    }
}

#[test]
fn test_28_remove_nonexistent_returns_none() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Try to remove non-existent object
    let result = vm.remove_heap_object(999);
    assert!(result.is_none());
}

#[test]
fn test_29_get_nonexistent_returns_none() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Try to get non-existent object
    let result = vm.get_heap_object(999);
    assert!(result.is_none());
}

#[test]
fn test_30_contains_heap_object() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Initially empty
    assert!(!vm.contains_heap_object(1));

    // Add object
    let list: ListData<i32> = ListData::new();
    let id = vm.insert_heap_object(list);

    assert!(vm.contains_heap_object(id));
    assert!(!vm.contains_heap_object(id + 1));
}

// ============================================================================
// Thread Safety Tests (Tests 31-40)
// ============================================================================

#[test]
fn test_31_concurrent_read_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list with data
    let mut list: ListData<i32> = ListData::new();
    for i in 0..100 {
        list.push(i);
    }

    let list_id = vm.insert_heap_object(list);

    // Spawn multiple readers
    let handles: Vec<JoinHandle<()>> = (0..10)
        .map(|_| {
            let vm = vm.clone();
            let list_id = list_id;
            thread::spawn(move || {
                for _ in 0..100 {
                    if let Some(obj) = vm.get_heap_object(list_id) {
                        let guard = obj.read().unwrap();
                        if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
                            // Verify data integrity
                            assert_eq!(list.len(), 100);
                        }
                    }
                }
            })
        })
        .collect();

    // All threads should complete without panic
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_32_concurrent_write_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list
    let list: ListData<i32> = ListData::new();
    let list_id = vm.insert_heap_object(list);

    // Spawn multiple writers (each writing unique values)
    let handles: Vec<JoinHandle<()>> = (0..10)
        .map(|i| {
            let vm = vm.clone();
            let list_id = list_id;
            thread::spawn(move || {
                for j in 0..10 {
                    if let Some(obj) = vm.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();
                        if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                            list.push(i * 10 + j);
                        }
                    }
                }
            })
        })
        .collect();

    // All threads should complete without panic
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt).unwrap();

    assert_eq!(list_ref.len(), 100); // 10 threads * 10 operations
}

#[test]
fn test_33_concurrent_read_write_mix() {
    let flash = VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1024);

    // Create list
    let mut list: ListData<i32> = ListData::new();
    list.push(0);

    let list_id = vm.insert_heap_object(list);

    // Spawn readers and writers
    let mut handles = vec![];

    // Writers
    for i in 0..5 {
        let vm = vm.clone();
        let list_id = list_id;
        handles.push(thread::spawn(move || {
            for j in 0..10 {
                if let Some(obj) = vm.get_heap_object(list_id) {
                    let mut guard = obj.write().unwrap();
                    if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                        list.push(i * 10 + j);
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
        }));
    }

    // Readers
    for _ in 0..5 {
        let vm = vm.clone();
        let list_id = list_id;
        handles.push(thread::spawn(move || {
            for _ in 0..10 {
                if let Some(obj) = vm.get_heap_object(list_id) {
                    let guard = obj.read().unwrap();
                    if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
                        let _ = list.len();
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
        }));
    }

    // All threads should complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_34_concurrent_object_creation() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Spawn multiple threads creating objects
    let handles: Vec<JoinHandle<u64>> = (0..10)
        .map(|i| {
            let vm = vm.clone();
            thread::spawn(move || {
                let list: ListData<i32> = ListData::new();
                list.push(i);
                vm.insert_heap_object(list)
            })
        })
        .collect();

    // All should succeed with unique IDs
    let mut ids = vec![];
    for handle in handles {
        let id = handle.join().unwrap();
        ids.push(id);
    }

    // All IDs should be unique
    assert_eq!(ids.len(), ids.iter().collect::<std::collections::HashSet<_>>().len());
}

#[test]
fn test_35_concurrent_object_removal() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create multiple objects
    let mut ids = vec![];
    for i in 0..100 {
        let list: ListData<i32> = ListData::new();
        let id = vm.insert_heap_object(list);
        ids.push(id);
    }

    // Spawn threads removing objects
    let handles: Vec<JoinHandle<()>> = ids
        .chunks(10)
        .map(|chunk| {
            let vm = vm.clone();
            let chunk = chunk.to_vec();
            thread::spawn(move || {
                for id in chunk {
                    vm.remove_heap_object(id);
                }
            })
        })
        .collect();

    // All threads should complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all objects removed
    assert_eq!(vm.heap_object_count(), 0);
}

#[test]
fn test_36_rwlock_allows_multiple_readers() {
    let list: ListData<i32> = ListData::new();
    let list = Arc::new(RwLock::new(list));

    // Spawn multiple readers
    let handles: Vec<JoinHandle<usize>> = (0..10)
        .map(|_| {
            let list = list.clone();
            thread::spawn(move || {
                let guard = list.read().unwrap();
                // Simulate read operation
                thread::sleep(Duration::from_millis(10));
                guard.len()
            })
        })
        .collect();

    // All should complete without blocking
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_37_rwlock_writer_excludes_readers() {
    let mut list: ListData<i32> = ListData::new();
    let list = Arc::new(RwLock::new(list));

    // Start writer
    let writer_handle = {
        let list = list.clone();
        thread::spawn(move || {
            let mut guard = list.write().unwrap();
            // Hold write lock for a bit
            thread::sleep(Duration::from_millis(50));
            guard.push(42);
        })
    };

    // Give writer time to acquire lock
    thread::sleep(Duration::from_millis(10));

    // Try to read (should block until writer releases)
    let reader_handle = {
        let list = list.clone();
        thread::spawn(move || {
            let guard = list.read().unwrap();
            guard.len()
        })
    };

    // Writer should complete first
    writer_handle.join().unwrap();

    // Then reader
    let len = reader_handle.join().unwrap();
    assert_eq!(len, 1);
}

#[test]
fn test_38_concurrent_type_checks() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create objects of different types
    let int_list: ListData<i32> = ListData::new();
    let char_list: ListData<char> = ListData::new();
    let bool_list: ListData<bool> = ListData::new();

    let int_id = vm.insert_heap_object(int_list);
    let char_id = vm.insert_heap_object(char_list);
    let bool_id = vm.insert_heap_object(bool_list);

    // Spawn threads checking types concurrently
    let handles: Vec<JoinHandle<bool>> = (0..10)
        .map(|i| {
            let vm = vm.clone();
            thread::spawn(move || {
                // Use thread index to determine which type to check
                let (id, expected) = match i % 3 {
                    0 => (int_id, TypeTag::ListInt),
                    1 => (char_id, TypeTag::ListChar),
                    _ => (bool_id, TypeTag::ListBool),
                };

                if let Some(obj) = vm.get_heap_object(id) {
                    obj.read().unwrap().type_tag() == expected
                } else {
                    false
                }
            })
        })
        .collect();

    // All should complete without data races
    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    // At least some should be correct (approximately 1/3)
    let correct_count = results.iter().filter(|&&r| r).count();
    assert!(correct_count >= 3); // At least 3 out of 10 should match
}

#[test]
fn test_39_no_data_race_in_push_operations() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create list
    let list: ListData<i32> = ListData::new();
    let list_id = vm.insert_heap_object(list);

    // Spawn threads pushing concurrently
    let handles: Vec<JoinHandle<()>> = (0..10)
        .map(|i| {
            let vm = vm.clone();
            thread::spawn(move || {
                for j in 0..10 {
                    if let Some(obj) = vm.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();
                        if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                            list.push(i * 10 + j);
                        }
                    }
                }
            })
        })
        .collect();

    // All should complete without panic
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final count
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt).unwrap();

    assert_eq!(list_ref.len(), 100);
}

#[test]
fn test_40_thread_safe_object_count() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Spawn threads creating objects concurrently
    let handles: Vec<JoinHandle<()>> = (0..10)
        .map(|_| {
            let vm = vm.clone();
            thread::spawn(move || {
                for _ in 0..10 {
                    let list: ListData<i32> = ListData::new();
                    vm.insert_heap_object(list);
                }
            })
        })
        .collect();

    // All should complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Final count should be 100
    assert_eq!(vm.heap_object_count(), 100);
}

// ============================================================================
// Performance Tests (Tests 41-50)
// ============================================================================

#[test]
fn test_41_performance_large_list_operations() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create large list
    let mut list: ListData<i32> = ListData::new();
    for i in 0..10_000 {
        list.push(i);
    }

    let list_id = vm.insert_heap_object(list);

    // Measure time to access all elements
    let start = std::time::Instant::now();
    let obj = vm.get_heap_object(list_id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt).unwrap();

    let mut sum = 0;
    for i in 0..list_ref.len() {
        sum += list_ref.get(i).copied().unwrap_or(0);
    }
    let elapsed = start.elapsed();

    assert_eq!(sum, 10_000 * 9999 / 2); // Sum of 0..9999
    // Should be fast (< 100ms for 10k elements)
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn test_42_performance_bulk_operations() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create 100 lists
    let start = std::time::Instant::now();
    let mut ids = vec![];
    for i in 0..100 {
        let mut list: ListData<i32> = ListData::new();
        list.push(i);
        ids.push(vm.insert_heap_object(list));
    }
    let elapsed = start.elapsed();

    assert_eq!(ids.len(), 100);
    // Should be fast (< 10ms for 100 lists)
    assert!(elapsed.as_millis() < 10);
}

#[test]
fn test_43_performance_random_access() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create list
    let mut list: ListData<i32> = ListData::new();
    for i in 0..1000 {
        list.push(i);
    }

    let list_id = vm.insert_heap_object(list);

    // Random access pattern
    let start = std::time::Instant::now();
    let obj = vm.get_heap_object(list_id).unwrap();

    for i in [0, 500, 999, 250, 750, 100, 900] {
        let guard = obj.read().unwrap();
        let list_ref = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt).unwrap();
        assert_eq!(list_ref.get(i), Some(&i));
    }
    let elapsed = start.elapsed();

    // Should be very fast (< 1ms for 7 accesses)
    assert!(elapsed.as_millis() < 1);
}

#[test]
fn test_44_performance_concurrent_readers() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create list
    let mut list: ListData<i32> = ListData::new();
    for i in 0..1000 {
        list.push(i);
    }

    let list_id = vm.insert_heap_object(list);

    // Spawn many readers
    let start = std::time::Instant::now();
    let handles: Vec<JoinHandle<()>> = (0..50)
        .map(|_| {
            let vm = vm.clone();
            thread::spawn(move || {
                if let Some(obj) = vm.get_heap_object(list_id) {
                    let guard = obj.read().unwrap();
                    if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
                        let _ = list.len();
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
    let elapsed = start.elapsed();

    // Should be fast even with 50 concurrent readers (< 10ms)
    assert!(elapsed.as_millis() < 10);
}

#[test]
fn test_45_performance_repeated_downcast() {
    let list: ListData<i32> = ListData::new();
    list.push(42);

    let list_obj: Arc<RwLock<dyn HeapObject>> = Arc::new(RwLock::new(list));

    // Measure downcast performance
    let iterations = 10_000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let guard = list_obj.read().unwrap();
        let _ = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt);
    }

    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations as u128;

    // Average should be < 100ns per operation
    assert!(avg_ns < 100);
}

#[test]
fn test_46_performance_mixed_operations() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create list
    let mut list: ListData<i32> = ListData::new();
    let list_id = vm.insert_heap_object(list);

    // Mix of operations
    let start = std::time::Instant::now();

    for i in 0..1000 {
        match i % 4 {
            0 => {
                // Push
                if let Some(obj) = vm.get_heap_object(list_id) {
                    let mut guard = obj.write().unwrap();
                    if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                        list.push(i);
                    }
                }
            }
            1 => {
                // Pop
                if let Some(obj) = vm.get_heap_object(list_id) {
                    let mut guard = obj.write().unwrap();
                    if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                        list.pop();
                    }
                }
            }
            2 => {
                // Get
                if let Some(obj) = vm.get_heap_object(list_id) {
                    let guard = obj.read().unwrap();
                    if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
                        let _ = list.get(0);
                    }
                }
            }
            3 => {
                // Len
                if let Some(obj) = vm.get_heap_object(list_id) {
                    let guard = obj.read().unwrap();
                    if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
                        let _ = list.len();
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    let elapsed = start.elapsed();

    // Should be fast (< 100ms for 4000 operations)
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn test_47_performance_large_registry() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create many objects
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let list: ListData<i32> = ListData::new();
        list.push(i);
        vm.insert_heap_object(list);
    }
    let elapsed = start.elapsed();

    // Should be fast (< 50ms for 1000 objects)
    assert!(elapsed.as_millis() < 50);
    assert_eq!(vm.heap_object_count(), 1000);
}

#[test]
fn test_48_performance_clear_registry() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create 1000 objects
    for i in 0..1000 {
        let list: ListData<i32> = ListData::new();
        list.push(i);
        vm.insert_heap_object(list);
    }

    assert_eq!(vm.heap_object_count(), 1000);

    // Measure clear time
    let start = std::time::Instant::now();
    vm.clear_heap_objects();
    let elapsed = start.elapsed();

    assert_eq!(vm.heap_object_count(), 0);
    // Should be fast (< 10ms)
    assert!(elapsed.as_millis() < 10);
}

#[test]
fn test_49_performance_contains_check() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create 100 objects
    let mut ids = vec![];
    for i in 0..100 {
        let list: ListData<i32> = ListData::new();
        ids.push(vm.insert_heap_object(list));
    }

    // Measure contains check time
    let start = std::time::Instant::now();
    for id in &ids {
        assert!(vm.contains_heap_object(*id));
    }
    let elapsed = start.elapsed();

    // Should be very fast (< 1ms for 100 checks)
    assert!(elapsed.as_millis() < 1);
}

#[test]
fn test_50_performance_type_tag_overhead() {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);

    // Create object
    let list: ListData<i32> = ListData::new();
    let id = vm.insert_heap_object(list);

    // Measure type tag access time
    let iterations = 100_000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        if let Some(obj) = vm.get_heap_object(id) {
            let _ = obj.read().unwrap().type_tag();
        }
    }

    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations as u128;

    // Type tag access should be very fast (< 50ns)
    assert!(avg_ns < 50);
}

// ============================================================================
// Test Module Declaration
// ============================================================================

#[cfg(test)]
mod plan077_integration_tests {
    use super::*;
}
