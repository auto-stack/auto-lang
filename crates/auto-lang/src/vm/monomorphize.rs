// Plan 076 Phase 2: Monomorphization Pass
// Generates specialized bytecode for each generic instantiation

use crate::ast::Type;
use crate::vm::generic::{GenericInstance, GenericTable};
use crate::vm::opcode::OpCode;
use std::collections::HashMap;

/// Monomorphization pass result
/// Contains the specialized bytecode for each generic instantiation
#[derive(Debug, Clone)]
pub struct MonomorphizedModule {
    /// Monomorphic name (e.g., "List_int")
    pub name: String,
    /// Specialized bytecode
    pub bytecode: Vec<u8>,
    /// Original generic instance
    pub instance: GenericInstance,
}

/// Monomorphizer - generates specialized bytecode for generic types
pub struct Monomorphizer {
    /// Generic instantiations table
    generics: GenericTable,
    /// Generated monomorphic modules
    modules: Vec<MonomorphizedModule>,
}

impl Monomorphizer {
    /// Create a new monomorphizer
    pub fn new() -> Self {
        Self {
            generics: GenericTable::new(),
            modules: Vec::new(),
        }
    }

    /// Add a generic instantiation to be monomorphized
    pub fn register_generic(&mut self, instance: GenericInstance) -> String {
        self.generics.register(instance)
    }

    /// Perform monomorphization pass
    /// Generates specialized bytecode for all registered generic instantiations
    pub fn monomorphize(&mut self) -> Vec<MonomorphizedModule> {
        self.modules.clear();

        for instance in self.generics.all() {
            let mono_name = instance.monomorphic_name();
            let bytecode = self.generate_specialized_bytecode(&instance);

            self.modules.push(MonomorphizedModule {
                name: mono_name,
                bytecode,
                instance: instance.clone(),
            });
        }

        self.modules.clone()
    }

    /// Generate specialized bytecode for a generic instantiation
    fn generate_specialized_bytecode(&self, instance: &GenericInstance) -> Vec<u8> {
        let mut bytecode = Vec::new();

        match instance.base_name.as_str() {
            "List" => {
                // Generate List<T> specialized bytecode
                self.generate_list_bytecode(instance, &mut bytecode);
            }
            _ => {
                // For other generic types, generate placeholder
                // TODO: Phase 4+ will handle user-defined generics
            }
        }

        bytecode
    }

    /// Generate specialized bytecode for List<T>
    fn generate_list_bytecode(&self, instance: &GenericInstance, bytecode: &mut Vec<u8>) {
        if let Some(elem_type) = instance.list_element_type() {
            match elem_type {
                Type::Int => {
                    // List<int> operations
                    bytecode.push(OpCode::CREATE_LIST_INT as u8);
                }
                Type::Str(_) => {
                    // List<string> operations
                    bytecode.push(OpCode::CREATE_LIST_STR as u8);
                }
                Type::Bool => {
                    // List<bool> operations
                    bytecode.push(OpCode::CREATE_LIST_BOOL as u8);
                }
                _ => {
                    // Fallback for unsupported types
                    // Will use native function calls in Phase 3
                }
            }
        }
    }

    /// Get the opcode for creating a list of a specific type
    pub fn get_list_create_opcode(elem_type: &Type) -> Option<OpCode> {
        match elem_type {
            Type::Int => Some(OpCode::CREATE_LIST_INT),
            Type::Str(_) => Some(OpCode::CREATE_LIST_STR),
            Type::Bool => Some(OpCode::CREATE_LIST_BOOL),
            _ => None,
        }
    }

    /// Get the opcode for pushing to a list of a specific type
    pub fn get_list_push_opcode(elem_type: &Type) -> Option<OpCode> {
        match elem_type {
            Type::Int => Some(OpCode::LIST_PUSH_INT),
            _ => None, // TODO: Add more push opcodes in Phase 3
        }
    }

    /// Get the opcode for popping from a list of a specific type
    pub fn get_list_pop_opcode(elem_type: &Type) -> Option<OpCode> {
        match elem_type {
            Type::Int => Some(OpCode::LIST_POP_INT),
            _ => None, // TODO: Add more pop opcodes in Phase 3
        }
    }

    /// Get the opcode for getting from a list of a specific type
    pub fn get_list_get_opcode(elem_type: &Type) -> Option<OpCode> {
        match elem_type {
            Type::Int => Some(OpCode::LIST_GET_INT),
            _ => None, // TODO: Add more get opcodes in Phase 3
        }
    }

    /// Get the opcode for setting in a list of a specific type
    pub fn get_list_set_opcode(elem_type: &Type) -> Option<OpCode> {
        match elem_type {
            Type::Int => Some(OpCode::LIST_SET_INT),
            _ => None, // TODO: Add more set opcodes in Phase 3
        }
    }

    /// Get all generated monomorphic modules
    pub fn modules(&self) -> &[MonomorphizedModule] {
        &self.modules
    }

    /// Get a specific monomorphic module by name
    pub fn get_module(&self, name: &str) -> Option<&MonomorphizedModule> {
        self.modules.iter().find(|m| m.name == name)
    }

    /// Check if a monomorphic module exists
    pub fn has_module(&self, name: &str) -> bool {
        self.modules.iter().any(|m| m.name == name)
    }
}

impl Default for Monomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to determine if a type is monomorphizable
pub fn is_monomorphizable(ty: &Type) -> bool {
    match ty {
        Type::List(_) => true,  // List<T> is monomorphizable
        Type::GenericInstance(_) => true,  // User-defined generics
        _ => false,
    }
}

/// Get all monomorphizable types from a type
pub fn collect_monomorphizable_types(ty: &Type) -> Vec<Type> {
    let mut results = Vec::new();

    match ty {
        Type::List(elem) => {
            results.push(ty.clone());
            // Recursively collect from element type
            results.extend(collect_monomorphizable_types(elem));
        }
        Type::GenericInstance(inst) => {
            results.push(ty.clone());
            // Recursively collect from type parameters
            for param in &inst.args {
                results.extend(collect_monomorphizable_types(param));
            }
        }
        _ => {}
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::generic::GenericInstance;

    #[test]
    fn test_monomorphizer_new() {
        let mono = Monomorphizer::new();
        assert_eq!(mono.generics.len(), 0);
        assert_eq!(mono.modules().len(), 0);
    }

    #[test]
    fn test_monomorphizer_register_generic() {
        let mut mono = Monomorphizer::new();

        let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
        let mono_name = mono.register_generic(instance);

        assert_eq!(mono_name, "List_int");
        assert_eq!(mono.generics.len(), 1);
    }

    #[test]
    fn test_monomorphize_list_int() {
        let mut mono = Monomorphizer::new();

        let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
        mono.register_generic(instance);

        let modules = mono.monomorphize();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "List_int");
        assert!(!modules[0].bytecode.is_empty());
        assert_eq!(modules[0].bytecode[0], OpCode::CREATE_LIST_INT as u8);
    }

    #[test]
    fn test_monomorphize_list_string() {
        let mut mono = Monomorphizer::new();

        let instance = GenericInstance::new("List".to_string(), vec![Type::Str(0)]);
        mono.register_generic(instance);

        let modules = mono.monomorphize();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "List_str");
        assert_eq!(modules[0].bytecode[0], OpCode::CREATE_LIST_STR as u8);
    }

    #[test]
    fn test_monomorphize_list_bool() {
        let mut mono = Monomorphizer::new();

        let instance = GenericInstance::new("List".to_string(), vec![Type::Bool]);
        mono.register_generic(instance);

        let modules = mono.monomorphize();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "List_bool");
        assert_eq!(modules[0].bytecode[0], OpCode::CREATE_LIST_BOOL as u8);
    }

    #[test]
    fn test_monomorphize_multiple_lists() {
        let mut mono = Monomorphizer::new();

        mono.register_generic(GenericInstance::new("List".to_string(), vec![Type::Int]));
        mono.register_generic(GenericInstance::new("List".to_string(), vec![Type::Str(0)]));
        mono.register_generic(GenericInstance::new("List".to_string(), vec![Type::Bool]));

        let modules = mono.monomorphize();

        assert_eq!(modules.len(), 3);

        // Check all modules were generated
        assert!(mono.has_module("List_int"));
        assert!(mono.has_module("List_str"));
        assert!(mono.has_module("List_bool"));
    }

    #[test]
    fn test_get_list_create_opcode() {
        assert_eq!(
            Monomorphizer::get_list_create_opcode(&Type::Int),
            Some(OpCode::CREATE_LIST_INT)
        );
        assert_eq!(
            Monomorphizer::get_list_create_opcode(&Type::Str(0)),
            Some(OpCode::CREATE_LIST_STR)
        );
        assert_eq!(
            Monomorphizer::get_list_create_opcode(&Type::Bool),
            Some(OpCode::CREATE_LIST_BOOL)
        );
        assert_eq!(
            Monomorphizer::get_list_create_opcode(&Type::Float),
            None // Unsupported
        );
    }

    #[test]
    fn test_get_list_push_opcode() {
        assert_eq!(
            Monomorphizer::get_list_push_opcode(&Type::Int),
            Some(OpCode::LIST_PUSH_INT)
        );
        assert_eq!(
            Monomorphizer::get_list_push_opcode(&Type::Str(0)),
            None // Not implemented yet
        );
    }

    #[test]
    fn test_is_monomorphizable() {
        assert!(is_monomorphizable(&Type::List(Box::new(Type::Int))));
        assert!(!is_monomorphizable(&Type::Int));
        assert!(!is_monomorphizable(&Type::Str(0)));
    }

    #[test]
    fn test_collect_monomorphizable_types() {
        let list_int = Type::List(Box::new(Type::Int));
        let types = collect_monomorphizable_types(&list_int);

        assert_eq!(types.len(), 1);
    }

    #[test]
    fn test_get_module_by_name() {
        let mut mono = Monomorphizer::new();

        let instance = GenericInstance::new("List".to_string(), vec![Type::Int]);
        mono.register_generic(instance);

        mono.monomorphize();

        let module = mono.get_module("List_int");
        assert!(module.is_some());
        assert_eq!(module.unwrap().name, "List_int");

        assert!(mono.get_module("NonExistent").is_none());
    }
}
