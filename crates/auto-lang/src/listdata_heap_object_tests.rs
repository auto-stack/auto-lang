// Plan 077 Phase 3: ListData HeapObject Implementation Tests
// Tests for HeapObject trait implementations for generic ListData<T>

use crate::vm::types::ListData;
use crate::vm::heap_object::{HeapObject, TypeTag, downcast, downcast_mut, is_type, type_name};
use auto_val::Value;
use std::sync::{Arc, RwLock};

// ============================================================================
// ListData<i32> HeapObject Tests
// ============================================================================

#[test]
fn test_listdata_int_type_tag() {
    let list: ListData<i32> = ListData::new();
    assert_eq!(list.type_tag(), TypeTag::ListInt);
}

#[test]
fn test_listdata_int_is_list() {
    let list: ListData<i32> = ListData::new();
    assert!(is_type(&list, TypeTag::ListInt));
    assert!(!is_type(&list, TypeTag::ListChar));
    assert!(!is_type(&list, TypeTag::ListBool));
}

#[test]
fn test_listdata_int_type_name() {
    let list: ListData<i32> = ListData::new();
    assert_eq!(type_name(&list), "List<int>");
}

#[test]
fn test_listdata_int_as_any() {
    let list: ListData<i32> = ListData::new();
    let any: &dyn std::any::Any = list.as_any();

    // Verify we can downcast back to ListData<i32>
    assert!(any.downcast_ref::<ListData<i32>>().is_some());
    assert!(any.downcast_ref::<ListData<i32>>().is_some());
}

#[test]
fn test_listdata_int_as_any_mut() {
    let mut list: ListData<i32> = ListData::new();
    let any_mut: &mut dyn std::any::Any = list.as_any_mut();

    // Verify we can downcast back to mutable ListData<i32>
    assert!(any_mut.downcast_mut::<ListData<i32>>().is_some());
}

#[test]
fn test_listdata_int_downcast() {
    let list: ListData<i32> = ListData::new();
    let heap_obj: &dyn HeapObject = &list;

    // Downcast should succeed
    let downcasted = downcast::<ListData<i32>>(heap_obj);
    assert!(downcasted.is_some());
}

#[test]
fn test_listdata_int_downcast_wrong_type() {
    let list: ListData<i32> = ListData::new();
    let heap_obj: &dyn HeapObject = &list;

    // Downcast to wrong type should fail
    let downcasted = downcast::<ListData<char>>(heap_obj);
    assert!(downcasted.is_none());
}

#[test]
fn test_listdata_int_downcast_mut() {
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);

    let heap_obj: &mut dyn HeapObject = &mut list;

    // Downcast mut should succeed
    if let Some(list_ref) = downcast_mut::<ListData<i32>>(heap_obj) {
        list_ref.push(3);
        assert_eq!(list_ref.len(), 3);
    } else {
        panic!("Downcast failed");
    }
}

// ============================================================================
// ListData<char> HeapObject Tests
// ============================================================================

#[test]
fn test_listdata_char_type_tag() {
    let list: ListData<char> = ListData::new();
    assert_eq!(list.type_tag(), TypeTag::ListChar);
}

#[test]
fn test_listdata_char_is_list() {
    let list: ListData<char> = ListData::new();
    assert!(is_type(&list, TypeTag::ListChar));
    assert!(!is_type(&list, TypeTag::ListInt));
    assert!(!is_type(&list, TypeTag::ListBool));
}

#[test]
fn test_listdata_char_type_name() {
    let list: ListData<char> = ListData::new();
    assert_eq!(type_name(&list), "List<char>");
}

#[test]
fn test_listdata_char_downcast() {
    let mut list: ListData<char> = ListData::new();
    list.push('a');
    list.push('b');

    let heap_obj: &dyn HeapObject = &list;

    // Downcast should succeed
    let downcasted = downcast::<ListData<char>>(heap_obj);
    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().len(), 2);
}

// ============================================================================
// ListData<bool> HeapObject Tests
// ============================================================================

#[test]
fn test_listdata_bool_type_tag() {
    let list: ListData<bool> = ListData::new();
    assert_eq!(list.type_tag(), TypeTag::ListBool);
}

#[test]
fn test_listdata_bool_is_list() {
    let list: ListData<bool> = ListData::new();
    assert!(is_type(&list, TypeTag::ListBool));
    assert!(!is_type(&list, TypeTag::ListInt));
    assert!(!is_type(&list, TypeTag::ListChar));
}

#[test]
fn test_listdata_bool_type_name() {
    let list: ListData<bool> = ListData::new();
    assert_eq!(type_name(&list), "List<bool>");
}

#[test]
fn test_listdata_bool_downcast() {
    let mut list: ListData<bool> = ListData::new();
    list.push(true);
    list.push(false);

    let heap_obj: &dyn HeapObject = &list;

    // Downcast should succeed
    let downcasted = downcast::<ListData<bool>>(heap_obj);
    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().len(), 2);
}

// ============================================================================
// ListData<String> HeapObject Tests
// ============================================================================

#[test]
fn test_listdata_string_type_tag() {
    let list: ListData<String> = ListData::new();
    assert_eq!(list.type_tag(), TypeTag::ListString);
}

#[test]
fn test_listdata_string_is_list() {
    let list: ListData<String> = ListData::new();
    assert!(is_type(&list, TypeTag::ListString));
    assert!(!is_type(&list, TypeTag::ListInt));
    assert!(!is_type(&list, TypeTag::ListChar));
}

#[test]
fn test_listdata_string_type_name() {
    let list: ListData<String> = ListData::new();
    assert_eq!(type_name(&list), "List<string>");
}

#[test]
fn test_listdata_string_downcast() {
    let mut list: ListData<String> = ListData::new();
    list.push("hello".to_string());
    list.push("world".to_string());

    let heap_obj: &dyn HeapObject = &list;

    // Downcast should succeed
    let downcasted = downcast::<ListData<String>>(heap_obj);
    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().len(), 2);
}

// ============================================================================
// ListData<Value> HeapObject Tests
// ============================================================================

#[test]
fn test_listdata_value_type_tag() {
    let list: ListData<Value> = ListData::new();
    assert_eq!(list.type_tag(), TypeTag::ListValue);
}

#[test]
fn test_listdata_value_is_list() {
    let list: ListData<Value> = ListData::new();
    assert!(is_type(&list, TypeTag::ListValue));
    assert!(!is_type(&list, TypeTag::ListInt));
    assert!(!is_type(&list, TypeTag::ListChar));
}

#[test]
fn test_listdata_value_type_name() {
    let list: ListData<Value> = ListData::new();
    assert_eq!(type_name(&list), "List<Value>");
}

#[test]
fn test_listdata_value_downcast() {
    let mut list: ListData<Value> = ListData::new();
    list.push(Value::Int(1));
    list.push(Value::Bool(true));

    let heap_obj: &dyn HeapObject = &list;

    // Downcast should succeed
    let downcasted = downcast::<ListData<Value>>(heap_obj);
    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().len(), 2);
}

// ============================================================================
// Arc<RwLock<dyn HeapObject>> Integration Tests
// ============================================================================

#[test]
fn test_listdata_int_in_registry() {
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert ListData<i32>
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);
    list.push(3);

    registry.insert(1, Arc::new(RwLock::new(list)));

    // Retrieve and verify
    let obj = registry.get(&1).unwrap();
    let guard = obj.read().unwrap();

    assert_eq!(guard.type_tag(), TypeTag::ListInt);

    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();
    assert_eq!(list_ref.len(), 3);
    assert_eq!(list_ref.elems, vec![1, 2, 3]);
}

#[test]
fn test_listdata_char_in_registry() {
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert ListData<char>
    let mut list: ListData<char> = ListData::new();
    list.push('a');
    list.push('b');

    registry.insert(1, Arc::new(RwLock::new(list)));

    // Retrieve and verify
    let obj = registry.get(&1).unwrap();
    let guard = obj.read().unwrap();

    assert_eq!(guard.type_tag(), TypeTag::ListChar);

    let list_ref = downcast::<ListData<char>>(&*guard).unwrap();
    assert_eq!(list_ref.len(), 2);
    assert_eq!(list_ref.elems, vec!['a', 'b']);
}

#[test]
fn test_listdata_bool_in_registry() {
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert ListData<bool>
    let mut list: ListData<bool> = ListData::new();
    list.push(true);
    list.push(false);

    registry.insert(1, Arc::new(RwLock::new(list)));

    // Retrieve and verify
    let obj = registry.get(&1).unwrap();
    let guard = obj.read().unwrap();

    assert_eq!(guard.type_tag(), TypeTag::ListBool);

    let list_ref = downcast::<ListData<bool>>(&*guard).unwrap();
    assert_eq!(list_ref.len(), 2);
    assert_eq!(list_ref.elems, vec![true, false]);
}

#[test]
fn test_listdata_string_in_registry() {
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert ListData<String>
    let mut list: ListData<String> = ListData::new();
    list.push("hello".to_string());
    list.push("world".to_string());

    registry.insert(1, Arc::new(RwLock::new(list)));

    // Retrieve and verify
    let obj = registry.get(&1).unwrap();
    let guard = obj.read().unwrap();

    assert_eq!(guard.type_tag(), TypeTag::ListString);

    let list_ref = downcast::<ListData<String>>(&*guard).unwrap();
    assert_eq!(list_ref.len(), 2);
    assert_eq!(list_ref.elems, vec!["hello".to_string(), "world".to_string()]);
}

#[test]
fn test_listdata_value_in_registry() {
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert ListData<Value>
    let mut list: ListData<Value> = ListData::new();
    list.push(Value::Int(42));
    list.push(Value::Bool(true));
    list.push(Value::Str("test".into()));

    registry.insert(1, Arc::new(RwLock::new(list)));

    // Retrieve and verify
    let obj = registry.get(&1).unwrap();
    let guard = obj.read().unwrap();

    assert_eq!(guard.type_tag(), TypeTag::ListValue);

    let list_ref = downcast::<ListData<Value>>(&*guard).unwrap();
    assert_eq!(list_ref.len(), 3);
}

#[test]
fn test_multiple_listdata_types_in_registry() {
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert different ListData types
    let mut int_list: ListData<i32> = ListData::new();
    int_list.push(1);
    int_list.push(2);

    let mut char_list: ListData<char> = ListData::new();
    char_list.push('a');
    char_list.push('b');

    let mut bool_list: ListData<bool> = ListData::new();
    bool_list.push(true);
    bool_list.push(false);

    registry.insert(1, Arc::new(RwLock::new(int_list)));
    registry.insert(2, Arc::new(RwLock::new(char_list)));
    registry.insert(3, Arc::new(RwLock::new(bool_list)));

    // Retrieve and verify each type
    let obj1 = registry.get(&1).unwrap();
    let guard1 = obj1.read().unwrap();
    assert_eq!(guard1.type_tag(), TypeTag::ListInt);
    let list1 = downcast::<ListData<i32>>(&*guard1).unwrap();
    assert_eq!(list1.elems, vec![1, 2]);

    let obj2 = registry.get(&2).unwrap();
    let guard2 = obj2.read().unwrap();
    assert_eq!(guard2.type_tag(), TypeTag::ListChar);
    let list2 = downcast::<ListData<char>>(&*guard2).unwrap();
    assert_eq!(list2.elems, vec!['a', 'b']);

    let obj3 = registry.get(&3).unwrap();
    let guard3 = obj3.read().unwrap();
    assert_eq!(guard3.type_tag(), TypeTag::ListBool);
    let list3 = downcast::<ListData<bool>>(&*guard3).unwrap();
    assert_eq!(list3.elems, vec![true, false]);
}

#[test]
fn test_listdata_mutation_via_registry() {
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert ListData<i32>
    let mut list: ListData<i32> = ListData::new();
    list.push(1);
    list.push(2);

    registry.insert(1, Arc::new(RwLock::new(list)));

    // Mutate through registry
    let obj = registry.get(&1).unwrap();
    let mut guard = obj.write().unwrap();

    let list_ref = downcast_mut::<ListData<i32>>(&mut *guard).unwrap();
    list_ref.push(3);
    list_ref.push(4);

    assert_eq!(list_ref.len(), 4);
    assert_eq!(list_ref.elems, vec![1, 2, 3, 4]);
}

#[test]
fn test_listdata_storage_preserved_in_registry() {
    use crate::vm::types::ListStorage;
    use dashmap::DashMap;

    let registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

    // Insert ListData<i32> with InlineInt64 storage
    let mut list: ListData<i32> = ListData::with_storage(ListStorage::InlineInt64);
    list.push(1);
    list.push(2);

    registry.insert(1, Arc::new(RwLock::new(list)));

    // Retrieve and verify storage strategy is preserved
    let obj = registry.get(&1).unwrap();
    let guard = obj.read().unwrap();

    let list_ref = downcast::<ListData<i32>>(&*guard).unwrap();
    assert_eq!(list_ref.get_storage(), ListStorage::InlineInt64);
    assert!(!list_ref.can_grow());
    assert_eq!(list_ref.max_capacity(), Some(64));
}
