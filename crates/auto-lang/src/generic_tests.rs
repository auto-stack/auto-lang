// Plan 076 Phase 1: Generic Type Support Tests
// Tests for type parameter parsing and tracking in AutoVM codegen

use crate::vm::codegen::Codegen;
use crate::vm::generic::{GenericTable, GenericInstance, extract_generic_instance};
use crate::ast::Type;

#[test]
fn test_generic_table_list_int() {
    let mut table = GenericTable::new();

    let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
    let mono_name = table.register(instance);

    assert_eq!(mono_name, "List_int");
    assert!(table.contains("List_int"));
    assert_eq!(table.len(), 1);
}

#[test]
fn test_generic_table_multiple_lists() {
    let mut table = GenericTable::new();

    // Register List<int>
    table.register(GenericInstance::new("List".to_string(), vec![Type::Int]));
    // Register List<string>
    table.register(GenericInstance::new("List".to_string(), vec![Type::Str(0)]));
    // Register List<bool>
    table.register(GenericInstance::new("List".to_string(), vec![Type::Bool]));

    assert_eq!(table.len(), 3);
    assert_eq!(table.list_instantiations().len(), 3);
}

#[test]
fn test_codegen_track_generic_list_int() {
    let mut codegen = Codegen::new();

    // Track List<int>
    let list_int = Type::List(Box::new(Type::Int));
    let mono_name = codegen.track_generic(&list_int);

    assert_eq!(mono_name, Some("List_int".to_string()));
    assert_eq!(codegen.generics.len(), 1);
    assert!(codegen.generics.contains("List_int"));
}

#[test]
fn test_codegen_track_generic_list_string() {
    let mut codegen = Codegen::new();

    // Track List<string>
    let list_str = Type::List(Box::new(Type::Str(0)));
    let mono_name = codegen.track_generic(&list_str);

    assert_eq!(mono_name, Some("List_str".to_string()));
    assert_eq!(codegen.generics.len(), 1);
    assert!(codegen.generics.contains("List_str"));
}

#[test]
fn test_codegen_track_multiple_generics() {
    let mut codegen = Codegen::new();

    // Track multiple instantiations
    codegen.track_generic(&Type::List(Box::new(Type::Int)));
    codegen.track_generic(&Type::List(Box::new(Type::Str(0))));
    codegen.track_generic(&Type::List(Box::new(Type::Bool)));

    assert_eq!(codegen.generics.len(), 3);

    let lists = codegen.get_list_instantiations();
    assert_eq!(lists.len(), 3);
}

#[test]
fn test_codegen_get_monomorphic_name() {
    let codegen = Codegen::new();

    let list_int = Type::List(Box::new(Type::Int));
    let mono_name = codegen.get_monomorphic_name(&list_int);

    assert_eq!(mono_name, Some("List_int".to_string()));
}

#[test]
fn test_extract_generic_instance_list() {
    let list_int = Type::List(Box::new(Type::Int));
    let instance = extract_generic_instance(&list_int).unwrap();

    assert_eq!(instance.base_name, "List");
    assert_eq!(instance.params.len(), 1);
    // Can't compare Type directly (no PartialEq), just check length
}

#[test]
fn test_extract_generic_instance_non_generic() {
    let int_type = Type::Int;
    let instance = extract_generic_instance(&int_type);

    assert!(instance.is_none());
}

#[test]
fn test_generic_instance_is_list() {
    let list_int = GenericInstance::new("List".to_string(), vec![Type::Int]);
    assert!(list_int.is_list());

    let my_type = GenericInstance::new("MyType".to_string(), vec![Type::Bool]);
    assert!(!my_type.is_list());
}

#[test]
fn test_generic_instance_list_element_type() {
    let list_int = GenericInstance::new("List".to_string(), vec![Type::Int]);
    // Just check that element type is present (can't compare Type directly)
    assert!(list_int.list_element_type().is_some());

    let list_str = GenericInstance::new("List".to_string(), vec![Type::Str(0)]);
    assert!(list_str.list_element_type().is_some());

    // Wrong parameter count
    let empty = GenericInstance::new("List".to_string(), vec![]);
    assert!(empty.list_element_type().is_none());

    // Not a list
    let my_type = GenericInstance::new("MyType".to_string(), vec![Type::Int]);
    assert!(my_type.list_element_type().is_none());
}

#[test]
fn test_generic_instance_nested_list() {
    let nested_list = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let instance = extract_generic_instance(&nested_list).unwrap();

    assert_eq!(instance.base_name, "List");
    assert_eq!(instance.params.len(), 1);

    // Inner type should be List<int>
    match &instance.params[0] {
        Type::List(inner) => {
            // Check inner is int by unique_name
            assert_eq!(inner.unique_name().to_string(), "int");
        }
        _ => panic!("Expected List<Int>"),
    }

    assert_eq!(instance.monomorphic_name(), "List_List_int");
}

#[test]
fn test_generic_table_clear() {
    let mut table = GenericTable::new();

    table.register(GenericInstance::new("List".to_string(), vec![Type::Int]));
    table.register(GenericInstance::new("List".to_string(), vec![Type::Str(0)]));

    assert_eq!(table.len(), 2);

    table.clear();

    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
}

#[test]
fn test_generic_instance_display() {
    let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
    assert_eq!(format!("{}", instance), "List<int>");

    let instance2 = GenericInstance::new(
        "MyType".to_string(),
        vec![Type::Int, Type::Bool]
    );
    assert_eq!(format!("{}", instance2), "MyType<int, bool>");
}

#[test]
fn test_codegen_preserves_generics_after_tracking() {
    let mut codegen = Codegen::new();

    // Track some generics
    codegen.track_generic(&Type::List(Box::new(Type::Int)));
    codegen.track_generic(&Type::List(Box::new(Type::Str(0))));

    // Generics table should persist
    assert_eq!(codegen.generics.len(), 2);

    // Get all instantiations
    let instances = codegen.get_generic_instantiations();
    assert_eq!(instances.len(), 2);

    // Get list instantiations specifically
    let lists = codegen.get_list_instantiations();
    assert_eq!(lists.len(), 2);
}
