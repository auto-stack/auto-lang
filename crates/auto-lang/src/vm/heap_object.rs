// Plan 077 Phase 1: HeapObject Trait
// Unified object registry infrastructure for BigVM

use std::any::Any;

/// Trait for all heap-allocated objects in BigVM
///
/// This trait enables the unified object registry design where a single
/// DashMap<u64, Arc<RwLock<dyn HeapObject>>> can store objects of any type.
///
/// # Type Safety
///
/// The trait combines `Any` (for downcasting) with runtime type tags
/// to provide both compile-time and runtime type safety.
///
/// # Example
///
/// ```rust
/// use crate::vm::heap_object::{HeapObject, TypeTag};
/// use std::any::Any;
///
/// pub struct MyData {
///     value: i32,
/// }
///
/// impl HeapObject for MyData {
///     fn type_tag(&self) -> TypeTag { TypeTag::CustomType }
///     fn as_any(&self) -> &dyn Any { self }
///     fn as_any_mut(&mut self) -> &mut dyn Any { self }
/// }
///
/// // Usage in unified registry:
/// let obj: Arc<RwLock<dyn HeapObject>> = Arc::new(RwLock::new(MyData { value: 42 }));
/// let guard = obj.read();
/// let my_data = guard.as_any().downcast_ref::<MyData>().unwrap();
/// assert_eq!(my_data.value, 42);
/// ```
pub trait HeapObject: Any + Send + Sync {
    /// Get the type tag for runtime type checking
    ///
    /// This returns a TypeTag enum value that identifies the concrete type
    /// of the object at runtime. Used for:
    /// - Type checking before downcasting
    /// - Debugging and logging
    /// - Error messages
    fn type_tag(&self) -> TypeTag;

    /// Convert to `Any` for downcasting
    ///
    /// This enables safe downcasting to the concrete type using
    /// `Any::downcast_ref()` and `Any::downcast_mut()`.
    fn as_any(&self) -> &dyn Any;

    /// Convert to mutable `Any` for downcasting
    ///
    /// This enables mutable downcasting for modifying the concrete type.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Type tags for all heap-allocated objects in BigVM
///
/// Each variant corresponds to a concrete type that implements `HeapObject`.
/// Used for runtime type checking and debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeTag {
    // List types (monomorphized generics)
    /// `ListData<i32>` - List of 32-bit integers
    ListInt,
    /// `ListData<char>` - List of characters
    ListChar,
    /// `ListData<bool>` - List of booleans
    ListBool,
    /// `ListData<String>` - List of strings
    ListString,
    /// `ListData<Value>` - Fallback for mixed-type lists
    ListValue,

    // Map types
    /// `HashMapData<K, V>` - Hash map implementation
    HashMap,
    /// `TreeMapData<K, V>` - Tree map implementation
    TreeMap,

    // Set types
    /// `HashSetData<T>` - Hash set implementation
    HashSet,
    /// `TreeSetData<T>` - Tree set implementation
    TreeSet,

    // String types
    /// BigVM string object
    String,
    /// Byte string / bytes object
    Bytes,

    // Future types
    /// User-defined or custom types
    CustomType,
}

impl TypeTag {
    /// Get the name of this type tag as a string
    pub fn name(&self) -> &'static str {
        match self {
            TypeTag::ListInt => "List<int>",
            TypeTag::ListChar => "List<char>",
            TypeTag::ListBool => "List<bool>",
            TypeTag::ListString => "List<string>",
            TypeTag::ListValue => "List<Value>",
            TypeTag::HashMap => "HashMap",
            TypeTag::TreeMap => "TreeMap",
            TypeTag::HashSet => "HashSet",
            TypeTag::TreeSet => "TreeSet",
            TypeTag::String => "String",
            TypeTag::Bytes => "Bytes",
            TypeTag::CustomType => "CustomType",
        }
    }

    /// Check if this type tag represents a list type
    pub fn is_list(&self) -> bool {
        matches!(self,
            TypeTag::ListInt |
            TypeTag::ListChar |
            TypeTag::ListBool |
            TypeTag::ListString |
            TypeTag::ListValue
        )
    }

    /// Check if this type tag represents a map type
    pub fn is_map(&self) -> bool {
        matches!(self, TypeTag::HashMap | TypeTag::TreeMap)
    }

    /// Check if this type tag represents a set type
    pub fn is_set(&self) -> bool {
        matches!(self, TypeTag::HashSet | TypeTag::TreeSet)
    }
}

impl std::fmt::Display for TypeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Helper function to check if an object is of a specific type
///
/// # Example
///
/// ```rust
/// use crate::vm::heap_object::{is_type, TypeTag};
///
/// let obj: &dyn HeapObject = &my_list;
/// if is_type(obj, TypeTag::ListInt) {
///     println!("This is a List<int>");
/// }
/// ```
pub fn is_type(obj: &dyn HeapObject, tag: TypeTag) -> bool {
    obj.type_tag() == tag
}

/// Helper function to downcast a HeapObject to a concrete type
///
/// Returns `None` if the object is not of the requested type.
///
/// # Example
///
/// ```rust
/// use crate::vm::heap_object::downcast;
///
/// let obj: Arc<RwLock<dyn HeapObject>> = registry.get(&id).unwrap();
/// let guard = obj.read();
///
/// if let Some(list_int) = downcast::<ListData<i32>>(&*guard) {
///     println!("Got List<int> with {} elements", list_int.len());
/// }
/// ```
pub fn downcast<T: Any>(obj: &dyn HeapObject) -> Option<&T> {
    obj.as_any().downcast_ref::<T>()
}

/// Mutable version of downcast helper
///
/// Returns `None` if the object is not of the requested type.
///
/// # Example
///
/// ```rust
/// use crate::vm::heap_object::downcast_mut;
///
/// let obj: Arc<RwLock<dyn HeapObject>> = registry.get(&id).unwrap();
/// let mut guard = obj.write();
///
/// if let Some(list_int) = downcast_mut::<ListData<i32>>(&mut *guard) {
///     list_int.push(42);
/// }
/// ```
pub fn downcast_mut<T: Any>(obj: &mut dyn HeapObject) -> Option<&mut T> {
    obj.as_any_mut().downcast_mut::<T>()
}

/// Helper function to get type tag as a string
///
/// Convenience function for debugging and error messages.
///
/// # Example
///
/// ```rust
/// use crate::vm::heap_object::type_name;
///
/// let obj: Arc<RwLock<dyn HeapObject>> = registry.get(&id).unwrap();
/// let guard = obj.read();
/// println!("Object type: {}", type_name(&*guard));
/// ```
pub fn type_name(obj: &dyn HeapObject) -> &'static str {
    obj.type_tag().name()
}

// ============================================================================
// Plan 077 Phase 7: Optimized Downcast Helpers
// ============================================================================

/// Optimized checked downcast with type tag verification
///
/// This function combines type tag checking and downcasting into a single
/// operation to reduce overhead in hot paths. It's designed to be inlined
/// for maximum performance.
///
/// Returns `None` if:
/// - The type tag doesn't match the expected tag
/// - The downcast fails (shouldn't happen if tags match)
///
/// # Performance
///
/// - Inline-able for zero function call overhead
/// - Single type tag check before downcast
/// - Optimized for hot paths (e.g., LIST_PUSH_INT, LIST_GET_INT)
///
/// # Example
///
/// ```rust
/// use crate::vm::heap_object::{try_downcast_checked, TypeTag};
/// use crate::universe::ListData;
///
/// let obj: Arc<RwLock<dyn HeapObject>> = registry.get(&id).unwrap();
/// let guard = obj.read().unwrap();
///
/// if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
///     // Use list...
/// }
/// ```
#[inline]
pub fn try_downcast_checked<T: Any>(obj: &dyn HeapObject, expected_tag: TypeTag) -> Option<&T> {
    // Fast path: type tag matches (most common case)
    if obj.type_tag() == expected_tag {
        obj.as_any().downcast_ref::<T>()
    } else {
        None
    }
}

/// Mutable version of optimized checked downcast
///
/// Same as `try_downcast_checked` but for mutable access.
///
/// # Example
///
/// ```rust
/// use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
/// use crate::universe::ListData;
///
/// let obj: Arc<RwLock<dyn HeapObject>> = registry.get(&id).unwrap();
/// let mut guard = obj.write().unwrap();
///
/// if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
///     list.push(42);
/// }
/// ```
#[inline]
pub fn try_downcast_checked_mut<T: Any>(obj: &mut dyn HeapObject, expected_tag: TypeTag) -> Option<&mut T> {
    // Fast path: type tag matches (most common case)
    if obj.type_tag() == expected_tag {
        obj.as_any_mut().downcast_mut::<T>()
    } else {
        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    // Mock implementations for testing
    struct MockIntList {
        elems: Vec<i32>,
    }

    impl HeapObject for MockIntList {
        fn type_tag(&self) -> TypeTag { TypeTag::ListInt }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    struct MockCharList {
        elems: Vec<char>,
    }

    impl HeapObject for MockCharList {
        fn type_tag(&self) -> TypeTag { TypeTag::ListChar }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    struct MockString {
        value: String,
    }

    impl HeapObject for MockString {
        fn type_tag(&self) -> TypeTag { TypeTag::String }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_type_tag_names() {
        assert_eq!(TypeTag::ListInt.name(), "List<int>");
        assert_eq!(TypeTag::ListChar.name(), "List<char>");
        assert_eq!(TypeTag::ListBool.name(), "List<bool>");
        assert_eq!(TypeTag::ListString.name(), "List<string>");
        assert_eq!(TypeTag::ListValue.name(), "List<Value>");
        assert_eq!(TypeTag::HashMap.name(), "HashMap");
        assert_eq!(TypeTag::TreeMap.name(), "TreeMap");
        assert_eq!(TypeTag::HashSet.name(), "HashSet");
        assert_eq!(TypeTag::TreeSet.name(), "TreeSet");
        assert_eq!(TypeTag::String.name(), "String");
        assert_eq!(TypeTag::Bytes.name(), "Bytes");
        assert_eq!(TypeTag::CustomType.name(), "CustomType");
    }

    #[test]
    fn test_type_tag_display() {
        assert_eq!(format!("{}", TypeTag::ListInt), "List<int>");
        assert_eq!(format!("{}", TypeTag::HashMap), "HashMap");
        assert_eq!(format!("{}", TypeTag::String), "String");
    }

    #[test]
    fn test_type_tag_is_list() {
        assert!(TypeTag::ListInt.is_list());
        assert!(TypeTag::ListChar.is_list());
        assert!(TypeTag::ListBool.is_list());
        assert!(TypeTag::ListString.is_list());
        assert!(TypeTag::ListValue.is_list());
        assert!(!TypeTag::HashMap.is_list());
        assert!(!TypeTag::String.is_list());
    }

    #[test]
    fn test_type_tag_is_map() {
        assert!(TypeTag::HashMap.is_map());
        assert!(TypeTag::TreeMap.is_map());
        assert!(!TypeTag::ListInt.is_map());
        assert!(!TypeTag::HashSet.is_map());
    }

    #[test]
    fn test_type_tag_is_set() {
        assert!(TypeTag::HashSet.is_set());
        assert!(TypeTag::TreeSet.is_set());
        assert!(!TypeTag::HashMap.is_set());
        assert!(!TypeTag::ListInt.is_set());
    }

    #[test]
    fn test_type_tag_equality() {
        assert_eq!(TypeTag::ListInt, TypeTag::ListInt);
        assert_ne!(TypeTag::ListInt, TypeTag::ListChar);
        assert_ne!(TypeTag::HashMap, TypeTag::HashSet);
    }

    #[test]
    fn test_type_tag_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(TypeTag::ListInt);
        set.insert(TypeTag::ListChar);
        set.insert(TypeTag::HashMap);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&TypeTag::ListInt));
        assert!(set.contains(&TypeTag::ListChar));
        assert!(set.contains(&TypeTag::HashMap));
        assert!(!set.contains(&TypeTag::String));
    }

    #[test]
    fn test_heap_object_type_tag() {
        let list = MockIntList { elems: vec![1, 2, 3] };
        assert_eq!(list.type_tag(), TypeTag::ListInt);

        let char_list = MockCharList { elems: vec!['a', 'b'] };
        assert_eq!(char_list.type_tag(), TypeTag::ListChar);

        let string = MockString { value: "hello".to_string() };
        assert_eq!(string.type_tag(), TypeTag::String);
    }

    #[test]
    fn test_is_type_helper() {
        let list = MockIntList { elems: vec![1, 2, 3] };

        assert!(is_type(&list, TypeTag::ListInt));
        assert!(!is_type(&list, TypeTag::ListChar));
        assert!(!is_type(&list, TypeTag::String));
    }

    #[test]
    fn test_downcast_success() {
        let list = MockIntList { elems: vec![1, 2, 3] };
        let obj: &dyn HeapObject = &list;

        let downcasted = downcast::<MockIntList>(obj);
        assert!(downcasted.is_some());
        assert_eq!(downcasted.unwrap().elems, vec![1, 2, 3]);
    }

    #[test]
    fn test_downcast_failure() {
        let list = MockIntList { elems: vec![1, 2, 3] };
        let obj: &dyn HeapObject = &list;

        let downcasted = downcast::<MockCharList>(obj);
        assert!(downcasted.is_none());
    }

    #[test]
    fn test_downcast_mut_success() {
        let mut list = MockIntList { elems: vec![1, 2, 3] };
        let obj: &mut dyn HeapObject = &mut list;

        let downcasted = downcast_mut::<MockIntList>(obj);
        assert!(downcasted.is_some());

        let list_ref = downcasted.unwrap();
        list_ref.elems.push(4);
        assert_eq!(list_ref.elems, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_downcast_mut_failure() {
        let mut list = MockIntList { elems: vec![1, 2, 3] };
        let obj: &mut dyn HeapObject = &mut list;

        let downcasted = downcast_mut::<MockCharList>(obj);
        assert!(downcasted.is_none());
    }

    #[test]
    fn test_type_name_helper() {
        let list = MockIntList { elems: vec![1, 2, 3] };
        let obj: &dyn HeapObject = &list;

        assert_eq!(type_name(obj), "List<int>");
    }

    #[test]
    fn test_arc_rwlock_heap_object() {
        let list = MockIntList { elems: vec![1, 2, 3] };
        let obj: Arc<RwLock<dyn HeapObject>> = Arc::new(RwLock::new(list));

        // Test read access
        let guard = obj.read().unwrap();
        assert_eq!(guard.type_tag(), TypeTag::ListInt);
        assert!(is_type(&*guard, TypeTag::ListInt));

        let downcasted = downcast::<MockIntList>(&*guard);
        assert!(downcasted.is_some());
        assert_eq!(downcasted.unwrap().elems, vec![1, 2, 3]);
    }

    #[test]
    fn test_arc_rwlock_heap_object_mut() {
        let list = MockIntList { elems: vec![1, 2, 3] };
        let obj: Arc<RwLock<dyn HeapObject>> = Arc::new(RwLock::new(list));

        // Test write access
        let mut guard = obj.write().unwrap();
        assert_eq!(guard.type_tag(), TypeTag::ListInt);

        let downcasted = downcast_mut::<MockIntList>(&mut *guard);
        assert!(downcasted.is_some());

        let list_ref = downcasted.unwrap();
        list_ref.elems.push(4);
        assert_eq!(list_ref.elems, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_multiple_object_types_in_registry() {
        use dashmap::DashMap;

        let mut registry: DashMap<u64, Arc<RwLock<dyn HeapObject>>> = DashMap::new();

        // Insert different object types
        registry.insert(1, Arc::new(RwLock::new(MockIntList { elems: vec![1, 2, 3] })));
        registry.insert(2, Arc::new(RwLock::new(MockCharList { elems: vec!['a', 'b'] })));
        registry.insert(3, Arc::new(RwLock::new(MockString { value: "hello".to_string() })));

        // Retrieve and verify types
        let obj1 = registry.get(&1).unwrap();
        let guard1 = obj1.read().unwrap();
        assert_eq!(guard1.type_tag(), TypeTag::ListInt);
        let list1 = downcast::<MockIntList>(&*guard1).unwrap();
        assert_eq!(list1.elems, vec![1, 2, 3]);

        let obj2 = registry.get(&2).unwrap();
        let guard2 = obj2.read().unwrap();
        assert_eq!(guard2.type_tag(), TypeTag::ListChar);
        let list2 = downcast::<MockCharList>(&*guard2).unwrap();
        assert_eq!(list2.elems, vec!['a', 'b']);

        let obj3 = registry.get(&3).unwrap();
        let guard3 = obj3.read().unwrap();
        assert_eq!(guard3.type_tag(), TypeTag::String);
        let string = downcast::<MockString>(&*guard3).unwrap();
        assert_eq!(string.value, "hello");
    }

    #[test]
    fn test_clone_type_tag() {
        let tag = TypeTag::ListInt;
        let cloned = tag.clone();
        assert_eq!(tag, cloned);
    }

    #[test]
    fn test_copy_type_tag() {
        let tag = TypeTag::HashMap;
        let copied = tag;
        assert_eq!(tag, copied);
    }

    // Plan 077 Phase 7: Tests for optimized downcast helpers
    #[test]
    fn test_try_downcast_checked_success() {
        let list = MockIntList { elems: vec![1, 2, 3] };

        // Correct type tag should succeed
        let result = crate::vm::heap_object::try_downcast_checked::<MockIntList>(&list, TypeTag::ListInt);
        assert!(result.is_some());
        assert_eq!(result.unwrap().elems, vec![1, 2, 3]);
    }

    #[test]
    fn test_try_downcast_checked_wrong_type() {
        let list = MockIntList { elems: vec![1, 2, 3] };

        // Wrong type tag should fail
        let result = crate::vm::heap_object::try_downcast_checked::<MockIntList>(&list, TypeTag::ListChar);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_downcast_checked_mut_success() {
        let mut list = MockIntList { elems: vec![1, 2, 3] };

        // Correct type tag should succeed
        let result = crate::vm::heap_object::try_downcast_checked_mut::<MockIntList>(&mut list, TypeTag::ListInt);
        assert!(result.is_some());
        result.unwrap().elems.push(4);
        assert_eq!(list.elems, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_try_downcast_checked_mut_wrong_type() {
        let mut list = MockIntList { elems: vec![1, 2, 3] };

        // Wrong type tag should fail
        let result = crate::vm::heap_object::try_downcast_checked_mut::<MockIntList>(&mut list, TypeTag::ListChar);
        assert!(result.is_none());
    }

    #[test]
    fn test_optimized_downcast_performance() {
        // This test verifies the optimized helpers work correctly
        use crate::vm::heap_object::{try_downcast_checked, try_downcast_checked_mut};

        let list = MockIntList { elems: vec![1, 2, 3] };

        // Read path
        let read_result = try_downcast_checked::<MockIntList>(&list, TypeTag::ListInt);
        assert!(read_result.is_some());

        // Write path
        let mut list2 = MockIntList { elems: vec![4, 5, 6] };
        let write_result = try_downcast_checked_mut::<MockIntList>(&mut list2, TypeTag::ListInt);
        assert!(write_result.is_some());
        write_result.unwrap().elems.push(7);
        assert_eq!(list2.elems, vec![4, 5, 6, 7]);
    }
}
