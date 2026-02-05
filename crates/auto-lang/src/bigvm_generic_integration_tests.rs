// Plan 076 Phase 5: BigVM Generic Type Integration Tests
// Comprehensive integration tests for generic type support in BigVM

use crate::vm::codegen::Codegen;
use crate::vm::generic::{GenericInstance, GenericTable, extract_generic_instance};
use crate::vm::monomorphize::{Monomorphizer, is_monomorphizable, collect_monomorphizable_types};
use crate::vm::opcode::OpCode;
use crate::ast::Type;

// ============================================================================
// Generic Type Tracking Integration Tests
// ============================================================================

#[test]
fn test_codegen_tracks_list_int() {
    let mut codegen = Codegen::new();

    // Track List<int>
    let list_int = Type::List(Box::new(Type::Int));
    let mono_name = codegen.track_generic(&list_int);

    assert_eq!(mono_name, Some("List_int".to_string()));
    assert_eq!(codegen.generics.len(), 1);
    assert!(codegen.generics.contains("List_int"));
}

#[test]
fn test_codegen_tracks_multiple_list_types() {
    let mut codegen = Codegen::new();

    // Track multiple list instantiations
    codegen.track_generic(&Type::List(Box::new(Type::Int)));
    codegen.track_generic(&Type::List(Box::new(Type::Str(0))));
    codegen.track_generic(&Type::List(Box::new(Type::Bool)));

    assert_eq!(codegen.generics.len(), 3);

    let lists = codegen.get_list_instantiations();
    assert_eq!(lists.len(), 3);
}

#[test]
fn test_codegen_preserves_generics_across_compilations() {
    let mut codegen = Codegen::new();

    // First compilation
    codegen.track_generic(&Type::List(Box::new(Type::Int)));
    assert_eq!(codegen.generics.len(), 1);

    // Second compilation (simulated by not clearing)
    codegen.track_generic(&Type::List(Box::new(Type::Str(0))));
    assert_eq!(codegen.generics.len(), 2);

    // Generics should persist
    let instances = codegen.get_generic_instantiations();
    assert_eq!(instances.len(), 2);
}

// ============================================================================
// Monomorphization Integration Tests
// ============================================================================

#[test]
fn test_monomorphize_single_list_int() {
    let mut mono = Monomorphizer::new();

    let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
    mono.register_generic(instance);

    let modules = mono.monomorphize();

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "List_int");
    assert!(!modules[0].bytecode.is_empty());
}

#[test]
fn test_monomorphize_multiple_instantiations() {
    let mut mono = Monomorphizer::new();

    mono.register_generic(GenericInstance::new("List".to_string(), vec![Type::Int]));
    mono.register_generic(GenericInstance::new("List".to_string(), vec![Type::Str(0)]));
    mono.register_generic(GenericInstance::new("List".to_string(), vec![Type::Bool]));

    let modules = mono.monomorphize();

    assert_eq!(modules.len(), 3);
    assert!(mono.has_module("List_int"));
    assert!(mono.has_module("List_str"));
    assert!(mono.has_module("List_bool"));
}

#[test]
fn test_monomorphize_nested_list() {
    let mut mono = Monomorphizer::new();

    let nested_type = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let instance = extract_generic_instance(&nested_type).unwrap();

    mono.register_generic(instance);
    let modules = mono.monomorphize();

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "List_List_int");
}

// ============================================================================
// Opcode Generation Integration Tests
// ============================================================================

#[test]
fn test_get_list_create_opcode_int() {
    let elem_type = Type::Int;
    let opcode = Monomorphizer::get_list_create_opcode(&elem_type);

    assert_eq!(opcode, Some(OpCode::CREATE_LIST_INT));
}

#[test]
fn test_get_list_create_opcode_string() {
    let elem_type = Type::Str(0);
    let opcode = Monomorphizer::get_list_create_opcode(&elem_type);

    assert_eq!(opcode, Some(OpCode::CREATE_LIST_STR));
}

#[test]
fn test_get_list_create_opcode_bool() {
    let elem_type = Type::Bool;
    let opcode = Monomorphizer::get_list_create_opcode(&elem_type);

    assert_eq!(opcode, Some(OpCode::CREATE_LIST_BOOL));
}

#[test]
fn test_get_list_create_opcode_unsupported() {
    let elem_type = Type::Float;
    let opcode = Monomorphizer::get_list_create_opcode(&elem_type);

    assert_eq!(opcode, None);
}

#[test]
fn test_get_list_push_opcode_int() {
    let elem_type = Type::Int;
    let opcode = Monomorphizer::get_list_push_opcode(&elem_type);

    assert_eq!(opcode, Some(OpCode::LIST_PUSH_INT));
}

#[test]
fn test_get_list_pop_opcode_int() {
    let elem_type = Type::Int;
    let opcode = Monomorphizer::get_list_pop_opcode(&elem_type);

    assert_eq!(opcode, Some(OpCode::LIST_POP_INT));
}

#[test]
fn test_get_list_get_opcode_int() {
    let elem_type = Type::Int;
    let opcode = Monomorphizer::get_list_get_opcode(&elem_type);

    assert_eq!(opcode, Some(OpCode::LIST_GET_INT));
}

#[test]
fn test_get_list_set_opcode_int() {
    let elem_type = Type::Int;
    let opcode = Monomorphizer::get_list_set_opcode(&elem_type);

    assert_eq!(opcode, Some(OpCode::LIST_SET_INT));
}

// ============================================================================
// Generic Instance Extraction Tests
// ============================================================================

#[test]
fn test_extract_generic_instance_from_list() {
    let list_int = Type::List(Box::new(Type::Int));
    let instance = extract_generic_instance(&list_int).unwrap();

    assert_eq!(instance.base_name, "List");
    assert_eq!(instance.params.len(), 1);
    assert_eq!(instance.monomorphic_name(), "List_int");
}

#[test]
fn test_extract_generic_instance_from_nested_list() {
    let nested_list = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let instance = extract_generic_instance(&nested_list).unwrap();

    assert_eq!(instance.base_name, "List");
    assert_eq!(instance.params.len(), 1);
    assert_eq!(instance.monomorphic_name(), "List_List_int");
}

#[test]
fn test_extract_generic_instance_from_non_generic() {
    let int_type = Type::Int;
    let instance = extract_generic_instance(&int_type);

    assert!(instance.is_none());
}

// ============================================================================
// Monomorphizable Type Detection Tests
// ============================================================================

#[test]
fn test_is_monomorphizable_list() {
    let list_int = Type::List(Box::new(Type::Int));
    assert!(is_monomorphizable(&list_int));
}

#[test]
fn test_is_monomorphizable_non_generic() {
    assert!(!is_monomorphizable(&Type::Int));
    assert!(!is_monomorphizable(&Type::Str(0)));
    assert!(!is_monomorphizable(&Type::Bool));
}

#[test]
fn test_collect_monomorphizable_types_from_list() {
    let list_int = Type::List(Box::new(Type::Int));
    let types = collect_monomorphizable_types(&list_int);

    assert_eq!(types.len(), 1);
}

#[test]
fn test_collect_monomorphizable_types_from_nested_list() {
    let nested_list = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let types = collect_monomorphizable_types(&nested_list);

    // Should collect the outer List and the inner List
    assert_eq!(types.len(), 2);
}

// ============================================================================
// Generic Instance Naming Tests
// ============================================================================

#[test]
fn test_generic_instance_display_list_int() {
    let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
    assert_eq!(format!("{}", instance), "List<int>");
}

#[test]
fn test_generic_instance_display_multiple_params() {
    let instance = GenericInstance::new(
        "MyType".to_string(),
        vec![Type::Int, Type::Bool]
    );
    assert_eq!(format!("{}", instance), "MyType<int, bool>");
}

#[test]
fn test_generic_instance_monomorphic_name_list_int() {
    let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
    assert_eq!(instance.monomorphic_name(), "List_int");
}

#[test]
fn test_generic_instance_monomorphic_name_list_str() {
    let instance = GenericInstance::new("List".to_string(), vec![Type::Str(0)]);
    assert_eq!(instance.monomorphic_name(), "List_str");
}

#[test]
fn test_generic_instance_monomorphic_name_list_bool() {
    let instance = GenericInstance::new("List".to_string(), vec![Type::Bool]);
    assert_eq!(instance.monomorphic_name(), "List_bool");
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
    assert!(list_int.list_element_type().is_some());

    let list_str = GenericInstance::new("List".to_string(), vec![Type::Str(0)]);
    assert!(list_str.list_element_type().is_some());

    let wrong_params = GenericInstance::new("List".to_string(), vec![]);
    assert!(wrong_params.list_element_type().is_none());

    let not_list = GenericInstance::new("MyType".to_string(), vec![Type::Int]);
    assert!(not_list.list_element_type().is_none());
}

// ============================================================================
// GenericTable Tests
// ============================================================================

#[test]
fn test_generic_table_register_and_contains() {
    let mut table = GenericTable::new();

    let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
    let mono_name = table.register(instance);

    assert_eq!(mono_name, "List_int");
    assert!(table.contains("List_int"));
}

#[test]
fn test_generic_table_multiple_registrations() {
    let mut table = GenericTable::new();

    table.register(GenericInstance::new("List".to_string(), vec![Type::Int]));
    table.register(GenericInstance::new("List".to_string(), vec![Type::Str(0)]));
    table.register(GenericInstance::new("List".to_string(), vec![Type::Bool]));

    assert_eq!(table.len(), 3);
    assert!(table.contains("List_int"));
    assert!(table.contains("List_str"));
    assert!(table.contains("List_bool"));
}

#[test]
fn test_generic_table_list_instantiations() {
    let mut table = GenericTable::new();

    table.register(GenericInstance::new("List".to_string(), vec![Type::Int]));
    table.register(GenericInstance::new("List".to_string(), vec![Type::Str(0)]));

    let lists = table.list_instantiations();
    assert_eq!(lists.len(), 2);
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

// ============================================================================
// End-to-End Integration Tests
// ============================================================================

#[test]
fn test_end_to_end_track_and_monomorphize() {
    // Step 1: Track generics during compilation (simulated)
    let mut codegen = Codegen::new();
    codegen.track_generic(&Type::List(Box::new(Type::Int)));
    codegen.track_generic(&Type::List(Box::new(Type::Str(0))));

    // Step 2: Extract generics from codegen
    let instances = codegen.get_generic_instantiations();
    assert_eq!(instances.len(), 2);

    // Step 3: Monomorphize
    let mut mono = Monomorphizer::new();
    for instance in instances {
        mono.register_generic(instance);
    }

    let modules = mono.monomorphize();
    assert_eq!(modules.len(), 2);

    // Step 4: Verify correct bytecode generation
    assert!(mono.has_module("List_int"));
    assert!(mono.has_module("List_str"));

    let int_module = mono.get_module("List_int").unwrap();
    assert_eq!(int_module.bytecode[0], OpCode::CREATE_LIST_INT as u8);

    let str_module = mono.get_module("List_str").unwrap();
    assert_eq!(str_module.bytecode[0], OpCode::CREATE_LIST_STR as u8);
}

#[test]
fn test_end_to_end_monomorphizable_workflow() {
    // Simulate compilation workflow
    let mut codegen = Codegen::new();

    // Source code contains: List<int>, List<string>, List<bool>
    let types = vec![
        Type::List(Box::new(Type::Int)),
        Type::List(Box::new(Type::Str(0))),
        Type::List(Box::new(Type::Bool)),
    ];

    // Track all generics
    for ty in &types {
        codegen.track_generic(ty);
    }

    // Collect all monomorphizable types
    let mut all_monomorphizable = Vec::new();
    for ty in &types {
        all_monomorphizable.extend(collect_monomorphizable_types(ty));
    }

    assert_eq!(all_monomorphizable.len(), 3);

    // Monomorphize all
    let mut mono = Monomorphizer::new();
    for instance in codegen.get_generic_instantiations() {
        mono.register_generic(instance);
    }

    let modules = mono.monomorphize();
    assert_eq!(modules.len(), 3);
}
