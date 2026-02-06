// Plan 077 Phase 4: Unified Object Registry Integration Tests
// Tests for the unified object registry in BigVM Engine

use crate::universe::{ListData, ListStorage};
use crate::vm::engine::BigVM;
use crate::vm::heap_object::{HeapObject, TypeTag, downcast, downcast_mut, is_type};
use crate::vm::virt_memory::VirtualFlash;
use auto_val::Value;
use std::sync::Arc;

// ============================================================================
// Basic Registry Operations Tests
// ============================================================================

#[test]
fn test_engine_insert_and_get_heap_object() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert a ListData<i32>
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    let id = vm.insert_heap_object(list);

    // Verify object was inserted
    assert!(vm.contains_heap_object(id));
    assert_eq!(vm.heap_object_count(), 1);

    // Get the object back
    let obj = vm.get_heap_object(id).unwrap();
    let guard = obj.read().unwrap();

    // Verify it's the correct type
    assert_eq!(guard.type_tag(), TypeTag::ListInt);

    // Downcast and verify contents
    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();
    assert_eq!(list_ref.len(), 3);
    assert_eq!(list_ref.elems, vec![1, 2, 3]);
}

#[test]
fn test_engine_insert_multiple_heap_objects() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert multiple different types
    let mut int_list: ListData<i32> = ListData::new();
    int_list.push(1);
    int_list.push(2);

    let mut char_list: ListData<char> = ListData::new();
    char_list.push('a');
    char_list.push('b');

    let mut bool_list: ListData<bool> = ListData::new();
    bool_list.push(true);
    bool_list.push(false);

    let id1 = vm.insert_heap_object(int_list);
    let id2 = vm.insert_heap_object(char_list);
    let id3 = vm.insert_heap_object(bool_list);

    // Verify all objects were inserted
    assert_eq!(vm.heap_object_count(), 3);
    assert!(vm.contains_heap_object(id1));
    assert!(vm.contains_heap_object(id2));
    assert!(vm.contains_heap_object(id3));

    // Verify each object
    let obj1 = vm.get_heap_object(id1).unwrap();
    let guard1 = obj1.read().unwrap();
    assert_eq!(guard1.type_tag(), TypeTag::ListInt);
    let list1 = downcast::<ListData<i32>>(&*guard1).unwrap();
    assert_eq!(list1.elems, vec![1, 2]);

    let obj2 = vm.get_heap_object(id2).unwrap();
    let guard2 = obj2.read().unwrap();
    assert_eq!(guard2.type_tag(), TypeTag::ListChar);
    let list2 = downcast::<ListData<char>>(&*guard2).unwrap();
    assert_eq!(list2.elems, vec!['a', 'b']);

    let obj3 = vm.get_heap_object(id3).unwrap();
    let guard3 = obj3.read().unwrap();
    assert_eq!(guard3.type_tag(), TypeTag::ListBool);
    let list3 = downcast::<ListData<bool>>(&*guard3).unwrap();
    assert_eq!(list3.elems, vec![true, false]);
}

#[test]
fn test_engine_remove_heap_object() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert an object
    let mut list: ListData<i32> = ListData::new();
    list.push(42);

    let id = vm.insert_heap_object(list);
    assert_eq!(vm.heap_object_count(), 1);

    // Remove the object
    let obj = vm.remove_heap_object(id).unwrap();
    assert_eq!(vm.heap_object_count(), 0);
    assert!(!vm.contains_heap_object(id));

    // Verify the removed object can still be accessed
    let guard = obj.read().unwrap();
    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();
    assert_eq!(list_ref.elems, vec![42]);
}

#[test]
fn test_engine_remove_nonexistent_object() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Try to remove an object that doesn't exist
    let result = vm.remove_heap_object(999);
    assert!(result.is_none());
}

#[test]
fn test_engine_get_nonexistent_object() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Try to get an object that doesn't exist
    let result = vm.get_heap_object(999);
    assert!(result.is_none());
}

#[test]
fn test_engine_clear_heap_objects() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert multiple objects
    vm.insert_heap_object(ListData::<i32>::new());
    vm.insert_heap_object(ListData::<char>::new());
    vm.insert_heap_object(ListData::<bool>::new());

    assert_eq!(vm.heap_object_count(), 3);

    // Clear all objects
    vm.clear_heap_objects();

    assert_eq!(vm.heap_object_count(), 0);
}

// ============================================================================
// Object Mutation Tests
// ============================================================================

#[test]
fn test_engine_mutate_heap_object() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert a list
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);

    let id = vm.insert_heap_object(list);

    // Mutate the object through the registry
    let obj = vm.get_heap_object(id).unwrap();
    let mut guard = obj.write().unwrap();

    let list_ref = downcast_mut::<ListData<i32>>(&mut *guard).unwrap();
    list_ref.push(3);
    list_ref.push(4);

    assert_eq!(list_ref.len(), 4);
    assert_eq!(list_ref.elems, vec![1, 2, 3, 4]);
}

#[test]
fn test_engine_concurrent_read_write() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert a list
    let mut list: ListData<i32> = ListData::new();
    for i in 0..100 {
        list.push(i);
    }

    let id = vm.insert_heap_object(list);

    // Perform multiple operations
    for i in 100..200 {
        let obj = vm.get_heap_object(id).unwrap();
        let mut guard = obj.write().unwrap();
        let list_ref = downcast_mut::<ListData<i32>>(&mut *guard).unwrap();
        list_ref.push(i);
    }

    // Verify final state
    let obj = vm.get_heap_object(id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();

    assert_eq!(list_ref.len(), 200);
    assert_eq!(list_ref.get(0), Some(&0));
    assert_eq!(list_ref.get(199), Some(&199));
}

// ============================================================================
// Storage Strategy Tests
// ============================================================================

#[test]
fn test_engine_heap_object_with_storage() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert ListData with InlineInt64 storage
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    for i in 0..64 {
        list.push(i);
    }

    let id = vm.insert_heap_object(list);

    // Verify storage is preserved
    let obj = vm.get_heap_object(id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();

    assert_eq!(list_ref.get_storage(), ListStorage::InlineInt64);
    assert!(!list_ref.can_grow());
    assert_eq!(list_ref.max_capacity(), Some(64));
    assert_eq!(list_ref.len(), 64);
}

// ============================================================================
// TypeTag Verification Tests
// ============================================================================

#[test]
fn test_engine_type_tag_verification() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert different types and verify TypeTags
    let id_int = vm.insert_heap_object(ListData::<i32>::new());
    let id_char = vm.insert_heap_object(ListData::<char>::new());
    let id_bool = vm.insert_heap_object(ListData::<bool>::new());

    // Verify ListData<i32> has correct TypeTag
    let obj_int = vm.get_heap_object(id_int).unwrap();
    let guard_int = obj_int.read().unwrap();
    assert!(is_type(&*guard_int, TypeTag::ListInt));
    assert!(!is_type(&*guard_int, TypeTag::ListChar));

    // Verify ListData<char> has correct TypeTag
    let obj_char = vm.get_heap_object(id_char).unwrap();
    let guard_char = obj_char.read().unwrap();
    assert!(is_type(&*guard_char, TypeTag::ListChar));
    assert!(!is_type(&*guard_char, TypeTag::ListBool));

    // Verify ListData<bool> has correct TypeTag
    let obj_bool = vm.get_heap_object(id_bool).unwrap();
    let guard_bool = obj_bool.read().unwrap();
    assert!(is_type(&*guard_bool, TypeTag::ListBool));
    assert!(!is_type(&*guard_bool, TypeTag::ListInt));
}

// ============================================================================
// ID Generation Tests
// ============================================================================

#[test]
fn test_engine_heap_object_id_generation() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert multiple objects and verify IDs are sequential
    let id1 = vm.insert_heap_object(ListData::<i32>::new());
    let id2 = vm.insert_heap_object(ListData::<char>::new());
    let id3 = vm.insert_heap_object(ListData::<bool>::new());

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 2);
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_engine_empty_list_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert an empty list
    let list: ListData<i32> = ListData::new();
    let id = vm.insert_heap_object(list);

    // Verify empty list state
    let obj = vm.get_heap_object(id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();

    assert_eq!(list_ref.len(), 0);
    assert!(list_ref.is_empty());
}

#[test]
fn test_engine_large_list() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert a list with many elements
    let mut list: ListData<i32> = ListData::new();
    for i in 0..10000 {
        list.push(i);
    }

    let id = vm.insert_heap_object(list);

    // Verify all elements are present
    let obj = vm.get_heap_object(id).unwrap();
    let guard = obj.read().unwrap();
    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();

    assert_eq!(list_ref.len(), 10000);
    assert_eq!(list_ref.get(0), Some(&0));
    assert_eq!(list_ref.get(9999), Some(&9999));
}

// ============================================================================
// Multiple Lists Coexistence Tests
// ============================================================================

#[test]
fn test_engine_multiple_lists_coexist() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert into unified registry (new way)
    let mut new_list: ListData<i32> = ListData::new();
    new_list.push(1);
    new_list.push(2);
    let new_id = vm.insert_heap_object(new_list);

    // Plan 077 Phase 6: Old lists registry removed, all lists now use unified registry
    // Insert another list into unified registry (simulating what old code did)
    let mut another_list: ListData<i32> = ListData::new();
    another_list.push(3);
    another_list.push(4);
    let another_id = vm.insert_heap_object(another_list);

    // Verify both lists coexist in unified registry
    assert!(vm.contains_heap_object(new_id));
    assert!(vm.contains_heap_object(another_id));

    // Verify first list
    let new_obj = vm.get_heap_object(new_id).unwrap();
    let new_guard = new_obj.read().unwrap();
    let new_list_ref = downcast::<ListData<i32>>(&*new_guard).unwrap();
    assert_eq!(new_list_ref.elems, vec![1, 2]);

    // Verify second list
    let another_obj = vm.get_heap_object(another_id).unwrap();
    let another_guard = another_obj.read().unwrap();
    let another_list_ref = downcast::<ListData<i32>>(&*another_guard).unwrap();
    assert_eq!(another_list_ref.elems, vec![3, 4]);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_engine_many_objects() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert many objects
    let mut ids = Vec::new();
    for i in 0..1000 {
        let mut list: ListData<i32> = ListData::new();
        list.push(i);
        let id = vm.insert_heap_object(list);
        ids.push(id);
    }

    assert_eq!(vm.heap_object_count(), 1000);

    // Verify all objects
    for (i, &id) in ids.iter().enumerate() {
        let obj = vm.get_heap_object(id).unwrap();
        let guard = obj.read().unwrap();
        let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();
        assert_eq!(list_ref.len(), 1);
        assert_eq!(list_ref.get(0), Some(&(i as i32)));
    }
}

#[test]
fn test_engine_mixed_type_operations() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert different types
    let int_id = vm.insert_heap_object({
        let mut list: ListData<i32> = ListData::new();
        list.push(42);
        list
    });

    let char_id = vm.insert_heap_object({
        let mut list: ListData<char> = ListData::new();
        list.push('x');
        list
    });

    let bool_id = vm.insert_heap_object({
        let mut list: ListData<bool> = ListData::new();
        list.push(true);
        list
    });

    // Perform mixed operations
    // Int: push
    {
        let obj = vm.get_heap_object(int_id).unwrap();
        let mut guard = obj.write().unwrap();
        let list = downcast_mut::<ListData<i32>>(&mut *guard).unwrap();
        list.push(100);
    }

    // Char: pop
    {
        let obj = vm.get_heap_object(char_id).unwrap();
        let mut guard = obj.write().unwrap();
        let list = downcast_mut::<ListData<char>>(&mut *guard).unwrap();
        let popped = list.pop();
        assert_eq!(popped, Some('x'));
    }

    // Bool: get
    {
        let obj = vm.get_heap_object(bool_id).unwrap();
        let guard = obj.read().unwrap();
        let list = downcast::<ListData<bool>>(&*guard).unwrap();
        assert_eq!(list.get(0), Some(&true));
    }
}

// ============================================================================
// Clone and PartialEq Tests
// ============================================================================

#[test]
fn test_engine_heap_object_clone() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Insert a list
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    let id = vm.insert_heap_object(list);

    // Clone the Arc (reference, not deep clone)
    let obj1 = vm.get_heap_object(id).unwrap();
    let obj2 = vm.get_heap_object(id).unwrap();

    // Both point to the same object
    let guard1 = obj1.read().unwrap();
    let guard2 = obj2.read().unwrap();

    let list1 = downcast::<ListData<i32>>(&*guard1).unwrap();
    let list2 = downcast::<ListData<i32>>(&*guard2).unwrap();

    assert_eq!(list1, list2);
    assert_eq!(Arc::strong_count(&obj1), 3); // obj1, obj2, +1 in registry
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_engine_registry_performance() {
    let flash = VirtualFlash::new(1024);
    let vm = BigVM::new(flash, 1024);

    // Benchmark insertion
    let start = std::time::Instant::now();
    for i in 0..10000 {
        let mut list: ListData<i32> = ListData::new();
        list.push(i);
        vm.insert_heap_object(list);
    }
    let insert_duration = start.elapsed();

    // Benchmark retrieval
    let ids: Vec<u64> = (0..10000).collect();
    let start = std::time::Instant::now();
    for &id in &ids {
        let _ = vm.get_heap_object(id);
    }
    let retrieve_duration = start.elapsed();

    // Performance should be reasonable (these are loose checks)
    assert!(insert_duration.as_millis() < 1000, "Insertion took too long");
    assert!(retrieve_duration.as_millis() < 500, "Retrieval took too long");

    println!("Insert: {:?}, Retrieve: {:?}", insert_duration, retrieve_duration);
}
