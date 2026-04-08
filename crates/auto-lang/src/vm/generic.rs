// Plan 076 Phase 1: Generic Type Support for AutoVM
// Provides type parameter tracking and generic instantiation tables

use crate::ast::Type;
use std::collections::HashMap;
use std::fmt;

/// Represents a single generic type instantiation
/// Example: List<int>, List<string>, MyType<bool, int>
#[derive(Debug, Clone)]
pub struct GenericInstance {
    /// Base type name (e.g., "List", "MyType")
    pub base_name: String,
    /// Type parameters (e.g., [Int, Str] for List<int>)
    pub params: Vec<Type>,
}

impl GenericInstance {
    /// Create a new generic instance
    pub fn new(base_name: String, params: Vec<Type>) -> Self {
        Self { base_name, params }
    }

    /// Generate a unique monomorphic name for this instantiation
    /// Examples:
    /// - List<int> → "List_int"
    /// - List<string> → "List_str"
    /// - MyType<bool, int> → "MyType_bool_int"
    pub fn monomorphic_name(&self) -> String {
        let param_names: Vec<String> = self.params.iter()
            .map(|t| Self::type_to_simple_name(t))
            .collect();

        format!("{}_{}", self.base_name, param_names.join("_"))
    }

    /// Convert Type to a simple name for monomorphic naming
    fn type_to_simple_name(ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Uint => "uint".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            Type::Float => "f32".to_string(),
            Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Char => "char".to_string(),
            Type::Str(_) | Type::String => "str".to_string(),
            Type::CStr => "cstr".to_string(),
            Type::List(inner) => format!("List_{}", Self::type_to_simple_name(inner)),
            Type::Map(k, v) => format!("Map_{}_{}", Self::type_to_simple_name(k), Self::type_to_simple_name(v)),
            Type::User(type_decl) => type_decl.name.to_string(),
            _ => format!("unknown_{:?}", ty),
        }
    }

    /// Check if this is a List instantiation
    pub fn is_list(&self) -> bool {
        self.base_name == "List"
    }

    /// Get the element type if this is a List<T>
    pub fn list_element_type(&self) -> Option<&Type> {
        if self.is_list() && self.params.len() == 1 {
            self.params.first()
        } else {
            None
        }
    }
}

impl fmt::Display for GenericInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let param_names: Vec<String> = self.params.iter()
            .map(|t| t.unique_name().to_string())
            .collect();
        write!(f, "{}<{}>", self.base_name, param_names.join(", "))
    }
}

/// Table tracking all generic type instantiations during compilation
#[derive(Debug, Clone)]
pub struct GenericTable {
    /// Map from monomorphic name → GenericInstance
    /// Example: "List_int" → GenericInstance { base_name: "List", params: [Int] }
    instances: HashMap<String, GenericInstance>,
}

impl GenericTable {
    /// Create a new generic table
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// Register a generic instantiation
    /// Returns the monomorphic name for this instantiation
    pub fn register(&mut self, instance: GenericInstance) -> String {
        let mono_name = instance.monomorphic_name();
        self.instances.insert(mono_name.clone(), instance);
        mono_name
    }

    /// Check if a generic instantiation has been registered
    pub fn contains(&self, mono_name: &str) -> bool {
        self.instances.contains_key(mono_name)
    }

    /// Get a generic instantiation by monomorphic name
    pub fn get(&self, mono_name: &str) -> Option<&GenericInstance> {
        self.instances.get(mono_name)
    }

    /// Get all registered instantiations
    pub fn all(&self) -> Vec<&GenericInstance> {
        self.instances.values().collect()
    }

    /// Get all List instantiations
    pub fn list_instantiations(&self) -> Vec<&GenericInstance> {
        self.instances.values()
            .filter(|inst| inst.is_list())
            .collect()
    }

    /// Get the number of registered instantiations
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Clear all instantiations
    pub fn clear(&mut self) {
        self.instances.clear();
    }
}

impl Default for GenericTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract generic instance information from a Type
/// Returns None if the type is not a generic instantiation
pub fn extract_generic_instance(ty: &Type) -> Option<GenericInstance> {
    match ty {
        // List<T> is a built-in generic
        Type::List(elem) => Some(GenericInstance::new(
            "List".to_string(),
            vec![*(elem.clone())],
        )),
        Type::Map(k, v) => Some(GenericInstance::new(
            "Map".to_string(),
            vec![*(k.clone()), *(v.clone())],
        )),

        // User-defined generics: MyType<int, bool>
        Type::GenericInstance(inst) => Some(GenericInstance::new(
            inst.base_name.to_string(),
            inst.args.clone(),
        )),

        // Not a generic type
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Type;

    #[test]
    fn test_generic_instance_list_int() {
        let instance = GenericInstance::new(
            "List".to_string(),
            vec![Type::Int],
        );

        assert_eq!(instance.monomorphic_name(), "List_int");
        assert!(instance.is_list());
        assert!(instance.list_element_type().is_some());
    }

    #[test]
    fn test_generic_instance_list_string() {
        let instance = GenericInstance::new(
            "List".to_string(),
            vec![Type::Str(0)],
        );

        assert_eq!(instance.monomorphic_name(), "List_str");
        assert!(instance.is_list());
    }

    #[test]
    fn test_generic_instance_multiple_params() {
        let instance = GenericInstance::new(
            "MyType".to_string(),
            vec![Type::Int, Type::Bool],
        );

        assert_eq!(instance.monomorphic_name(), "MyType_int_bool");
        assert!(!instance.is_list());
        assert!(instance.list_element_type().is_none());
    }

    #[test]
    fn test_generic_table_register() {
        let mut table = GenericTable::new();

        let list_int = GenericInstance::new("List".to_string(), vec![Type::Int]);
        let mono_name = table.register(list_int);

        assert_eq!(mono_name, "List_int");
        assert!(table.contains("List_int"));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_generic_table_multiple_instantiations() {
        let mut table = GenericTable::new();

        table.register(GenericInstance::new("List".to_string(), vec![Type::Int]));
        table.register(GenericInstance::new("List".to_string(), vec![Type::Str(0)]));
        table.register(GenericInstance::new("MyType".to_string(), vec![Type::Bool]));

        assert_eq!(table.len(), 3);
        assert_eq!(table.list_instantiations().len(), 2);
    }

    #[test]
    fn test_extract_generic_instance_list() {
        let list_int = Type::List(Box::new(Type::Int));
        let instance = extract_generic_instance(&list_int).unwrap();

        assert_eq!(instance.base_name, "List");
        assert_eq!(instance.params.len(), 1);
        assert_eq!(instance.monomorphic_name(), "List_int");
    }

    #[test]
    fn test_extract_generic_instance_non_generic() {
        let int_type = Type::Int;
        let instance = extract_generic_instance(&int_type);

        assert!(instance.is_none());
    }

    #[test]
    fn test_generic_instance_display() {
        let instance = GenericInstance::new(
            "List".to_string(),
            vec![Type::Int],
        );

        assert_eq!(format!("{}", instance), "List<int>");
    }
}
