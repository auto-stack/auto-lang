// Plan 076 Phase 2: Monomorphization Tests
// Tests for generating specialized bytecode for generic types

use crate::vm::codegen::Codegen;
use crate::vm::monomorphize::{Monomorphizer, is_monomorphizable, collect_monomorphizable_types};
use crate::vm::opcode::OpCode;
use crate::ast::Type;

#[test]
fn test_monomorphizer_list_int() {
    let mut mono = Monomorphizer::new();

    let instance = crate::vm::generic::GenericInstance::new(
        "List".to_string(),
        vec![Type::Int]
    );
    mono.register_generic(instance);

    let modules = mono.monomorphize();

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "List_int");
    assert_eq!(modules[0].bytecode[0], OpCode::CREATE_LIST_INT as u8);
}

#[test]
fn test_monomorphizer_list_string() {
    let mut mono = Monomorphizer::new();

    let instance = crate::vm::generic::GenericInstance::new(
        "List".to_string(),
        vec![Type::StrFixed(0)]
    );
    mono.register_generic(instance);

    let modules = mono.monomorphize();

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "List_str");
    assert_eq!(modules[0].bytecode[0], OpCode::CREATE_LIST_STR as u8);
}

#[test]
fn test_codegen_monomorphize() {
    let mut codegen = Codegen::new();

    // Track some generics
    codegen.track_generic(&Type::List(Box::new(Type::Int)));
    codegen.track_generic(&Type::List(Box::new(Type::StrFixed(0))));
    codegen.track_generic(&Type::List(Box::new(Type::Bool)));

    // Perform monomorphization
    let modules = codegen.monomorphize();

    assert_eq!(modules.len(), 3);

    // Check each module has correct bytecode
    let list_int = modules.iter().find(|m| m.name == "List_int").unwrap();
    assert_eq!(list_int.bytecode[0], OpCode::CREATE_LIST_INT as u8);

    let list_str = modules.iter().find(|m| m.name == "List_str").unwrap();
    assert_eq!(list_str.bytecode[0], OpCode::CREATE_LIST_STR as u8);

    let list_bool = modules.iter().find(|m| m.name == "List_bool").unwrap();
    assert_eq!(list_bool.bytecode[0], OpCode::CREATE_LIST_BOOL as u8);
}

#[test]
fn test_is_monomorphizable() {
    assert!(is_monomorphizable(&Type::List(Box::new(Type::Int))));
    assert!(!is_monomorphizable(&Type::Int));
    assert!(!is_monomorphizable(&Type::StrFixed(0)));
    assert!(!is_monomorphizable(&Type::Bool));
}

#[test]
fn test_collect_monomorphizable_types() {
    let list_int = Type::List(Box::new(Type::Int));
    let types = collect_monomorphizable_types(&list_int);

    assert_eq!(types.len(), 1);
    assert!(matches!(types[0], Type::List(_)));
}

#[test]
fn test_codegen_has_monomorphic_module() {
    let mut codegen = Codegen::new();

    codegen.track_generic(&Type::List(Box::new(Type::Int)));

    assert!(codegen.has_monomorphic_module("List_int"));
    assert!(!codegen.has_monomorphic_module("List_str"));
}

#[test]
fn test_codegen_get_monomorphic_name_checked() {
    let codegen = Codegen::new();

    let list_int = Type::List(Box::new(Type::Int));
    let mono_name = codegen.get_monomorphic_name_checked(&list_int);

    assert_eq!(mono_name, Some("List_int".to_string()));

    let int_type = Type::Int;
    let mono_name = codegen.get_monomorphic_name_checked(&int_type);

    assert_eq!(mono_name, None);
}

#[test]
fn test_monomorphizer_multiple_modules() {
    let mut mono = Monomorphizer::new();

    mono.register_generic(crate::vm::generic::GenericInstance::new(
        "List".to_string(),
        vec![Type::Int]
    ));
    mono.register_generic(crate::vm::generic::GenericInstance::new(
        "List".to_string(),
        vec![Type::StrFixed(0)]
    ));
    mono.register_generic(crate::vm::generic::GenericInstance::new(
        "List".to_string(),
        vec![Type::Bool]
    ));

    let modules = mono.monomorphize();

    assert_eq!(modules.len(), 3);
    assert!(mono.has_module("List_int"));
    assert!(mono.has_module("List_str"));
    assert!(mono.has_module("List_bool"));
}

#[test]
fn test_monomorphizer_get_module_by_name() {
    let mut mono = Monomorphizer::new();

    mono.register_generic(crate::vm::generic::GenericInstance::new(
        "List".to_string(),
        vec![Type::Int]
    ));

    mono.monomorphize();

    let module = mono.get_module("List_int");
    assert!(module.is_some());
    assert_eq!(module.unwrap().name, "List_int");

    assert!(mono.get_module("NonExistent").is_none());
}
