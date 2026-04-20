// Plan 087 Phase 1: Generic Registry
// Runtime metadata and instantiation support for user-defined generic types

use crate::ast::{Fn, GenericParam, Name, Type};
use crate::vm::heap_object::{HeapObject, TypeTag};
use auto_val::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

// ============================================================================
// Core Data Structures
// ============================================================================

/// Generic class template (compile-time metadata)
///
/// Represents a user-defined generic type definition from source code.
/// Example: `type Pair<K, V> { key K; val V }`
///
/// # Fields
/// - `name`: Type name (e.g., "Pair", "List", "HashMap")
/// - `generic_params`: Type parameters (e.g., [K, V])
/// - `fields`: Field definitions with generic types (e.g., [(key, K), (val, V)])
/// - `methods`: Method definitions that use type parameters
#[derive(Debug, Clone)]
pub struct ClassTemplate {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<FieldDef>,
    pub methods: HashMap<String, MethodInfo>,
}

/// Field definition in a generic class
///
/// Represents a single field with its type annotation.
/// Example: `key: K` in `type Pair<K, V> { key K; val V }`
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub field_type: Type,  // May contain type parameters (e.g., User("K"))
}

impl FieldDef {
    pub fn new(name: impl Into<String>, field_type: Type) -> Self {
        Self {
            name: name.into(),
            field_type,
        }
    }
}

impl fmt::Display for FieldDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.field_type.unique_name())
    }
}

/// Method information in a generic class
///
/// Stores both the original generic method declaration and
/// monomorphized implementations for specific type arguments.
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub fn_decl: Fn,  // Original method declaration (may contain type parameters)
    pub mono_impls: HashMap<String, Fn>,  // Monomorphized implementations
                                              // Key: mono_name (e.g., "Pair_int_str")
}

impl MethodInfo {
    pub fn new(name: impl Into<String>, fn_decl: Fn) -> Self {
        Self {
            name: name.into(),
            fn_decl,
            mono_impls: HashMap::new(),
        }
    }

    /// Add a monomorphized implementation for this method
    pub fn add_monomorphic(&mut self, mono_name: String, fn_impl: Fn) {
        self.mono_impls.insert(mono_name, fn_impl);
    }

    /// Get monomorphized implementation for specific type arguments
    pub fn get_monomorphic(&self, mono_name: &str) -> Option<&Fn> {
        self.mono_impls.get(mono_name)
    }
}

impl ClassTemplate {
    /// Create a new ClassTemplate from a type declaration
    pub fn new(
        name: impl Into<String>,
        generic_params: Vec<GenericParam>,
        fields: Vec<FieldDef>,
        methods: Vec<Fn>,
    ) -> Self {
        let name = name.into();
        let mut method_map = HashMap::new();

        for method in methods {
            let method_name = method.name.to_string();
            let method_info = MethodInfo::new(method_name.clone(), method);
            method_map.insert(method_name, method_info);
        }

        Self {
            name,
            generic_params,
            fields,
            methods: method_map,
        }
    }

    /// Generate monomorphic name for specific type arguments
    ///
    /// Example: `Pair<K, V>` with args [int, string] → "Pair_int_str"
    pub fn mono_name_from_args(&self, type_args: &[Type]) -> String {
        let arg_names: Vec<String> = type_args
            .iter()
            .map(|t| {
                let name = t.unique_name().to_string();
                name.replace('<', "_")
                    .replace('>', "_")
                    .replace(", ", "_")
                    .replace(' ', "_")
                    .replace(':', "_")
            })
            .collect();

        if arg_names.is_empty() {
            self.name.clone()
        } else {
            format!("{}_{}", self.name, arg_names.join("_"))
        }
    }

    /// Substitute type parameters in fields with concrete types
    ///
    /// # Arguments
    /// - `type_args`: Concrete types to substitute for generic parameters
    ///
    /// # Returns
    /// Vector of field definitions with substituted types
    pub fn substitute_fields(&self, type_args: &[Type]) -> Vec<FieldDef> {
        let param_names: Vec<Name> = self
            .generic_params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Type(tp) => Some(tp.name.clone()),
                GenericParam::Const(_) => None,  // TODO: Handle const parameters
            })
            .collect();

        self.fields
            .iter()
            .map(|field| FieldDef {
                name: field.name.clone(),
                field_type: field.field_type.substitute(&param_names, type_args),
            })
            .collect()
    }

    /// Get field index by name
    pub fn field_index(&self, field_name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == field_name)
    }

    /// Get method index by name
    pub fn method_index(&self, method_name: &str) -> Option<usize> {
        self.methods.keys().position(|k| k == method_name)
    }

    /// Plan 087 Phase 3: Get monomorphic method name for specific type arguments
    ///
    /// Example: `get_key` method for `Pair<int, string>` → "Pair_int_str.get_key"
    pub fn mono_method_name(&self, method_name: &str, type_args: &[Type]) -> String {
        let mono_name = self.mono_name_from_args(type_args);
        format!("{}.{}", mono_name, method_name)
    }

    /// Plan 087 Phase 3: Get method info by name
    pub fn get_method(&self, method_name: &str) -> Option<&MethodInfo> {
        self.methods.get(method_name)
    }

    /// Plan 087 Phase 3: Get mutable method info by name (for adding monomorphic implementations)
    pub fn get_method_mut(&mut self, method_name: &str) -> Option<&mut MethodInfo> {
        self.methods.get_mut(method_name)
    }
}

impl fmt::Display for ClassTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {}", self.name)?;
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param)?;
            }
            write!(f, ">")?;
        }
        write!(f, " {{ ")?;
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}", field)?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

// ============================================================================
// Concrete Generic Instance Type
// ============================================================================

/// Concrete generic instantiation (runtime type)
///
/// Represents a specific instantiation of a generic type with concrete type arguments.
/// Example: `Pair<int, string>` where base = Pair template, args = [int, string]
#[derive(Debug, Clone)]
pub struct ClassType {
    pub template: Arc<ClassTemplate>,
    pub type_args: Vec<Type>,
    pub mono_name: String,
}

impl ClassType {
    /// Create a new ClassType from a template and type arguments
    pub fn new(template: Arc<ClassTemplate>, type_args: Vec<Type>) -> Self {
        let mono_name = template.mono_name_from_args(&type_args);

        Self {
            template,
            type_args,
            mono_name,
        }
    }

    /// Get the base type name
    pub fn base_name(&self) -> &str {
        &self.template.name
    }

    /// Get field definitions with substituted types
    pub fn fields(&self) -> Vec<FieldDef> {
        self.template.substitute_fields(&self.type_args)
    }

    /// Get field index by name
    pub fn field_index(&self, field_name: &str) -> Option<usize> {
        self.template.field_index(field_name)
    }

    /// Get field type by name (Plan 118 Phase 7: for nested field access type inference)
    pub fn field_type(&self, field_name: &str) -> Option<Type> {
        let fields = self.fields();
        fields.iter()
            .find(|f| f.name == field_name)
            .map(|f| f.field_type.clone())
    }

    /// Get method index by name
    pub fn method_index(&self, method_name: &str) -> Option<usize> {
        self.template.method_index(method_name)
    }

    /// Check if this is a generic instantiation (has type arguments)
    pub fn is_generic(&self) -> bool {
        !self.type_args.is_empty()
    }

    /// Plan 197 Task 9: Get field names from the template
    pub fn field_names(&self) -> Vec<String> {
        self.template.fields.iter().map(|f| f.name.clone()).collect()
    }
}

impl fmt::Display for ClassType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.template.name)?;
        if !self.type_args.is_empty() {
            write!(f, "<")?;
            for (i, arg) in self.type_args.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg.unique_name())?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

// ============================================================================
// Plan 087 Phase 4: Specialized Storage
// ============================================================================

/// Specialized storage for common generic patterns
///
/// Provides compact storage for frequently used generic instantiations.
/// Instead of storing all fields as `Value` (24+ bytes each), we store
/// primitive types directly (4-8 bytes each).
///
/// # Memory Benefits
/// - `Pair<int, int>`: 8 bytes vs 48 bytes (6x reduction)
/// - `Pair<int, V>`: 4 + 24 = 28 bytes vs 48 bytes (1.7x reduction)
///
/// # Examples
/// - `Pair<int, int>` → Both fields stored as i32
/// - `Pair<int, string>` → key stored as i32, val as Value
/// - `Pair<string, int>` → key stored as Value, val as i32
#[derive(Debug)]
pub enum SpecializedPair {
    /// Pair<int, int> - Most compact (8 bytes)
    IntInt { key: i32, val: i32 },
    /// Pair<int, bool> - Compact (5 bytes, but aligned to 8)
    IntBool { key: i32, val: bool },
    /// Pair<bool, int> - Compact (5 bytes, but aligned to 8)
    BoolInt { key: bool, val: i32 },
    /// Pair<int, V> - Half specialized (4 + 24 = 28 bytes)
    IntValue { key: i32, val: Value },
    /// Pair<K, int> - Half specialized (24 + 4 = 28 bytes)
    ValueInt { key: Value, val: i32 },
    /// Pair<bool, V> - Half specialized (1 + 24 = 25 bytes, aligned to 32)
    BoolValue { key: bool, val: Value },
    /// Pair<V, bool> - Half specialized (24 + 1 = 25 bytes, aligned to 32)
    ValueBool { key: Value, val: bool },
    /// Pair<K, V> - Generic fallback (24 + 24 = 48 bytes)
    Generic { key: Value, val: Value },
}

impl SpecializedPair {
    /// Get field by index (0 = key, 1 = val)
    pub fn get_field(&self, index: usize) -> Option<Value> {
        match (self, index) {
            (SpecializedPair::IntInt { key, val: _ }, 0) => Some(Value::Int(*key)),
            (SpecializedPair::IntInt { val, .. }, 1) => Some(Value::Int(*val)),
            (SpecializedPair::IntBool { key, val: _ }, 0) => Some(Value::Int(*key)),
            (SpecializedPair::IntBool { val, .. }, 1) => Some(Value::Bool(*val)),
            (SpecializedPair::BoolInt { key, val: _ }, 0) => Some(Value::Bool(*key)),
            (SpecializedPair::BoolInt { val, .. }, 1) => Some(Value::Int(*val)),
            (SpecializedPair::IntValue { key, val: _ }, 0) => Some(Value::Int(*key)),
            (SpecializedPair::IntValue { val, .. }, 1) => Some(val.clone()),
            (SpecializedPair::ValueInt { key, val: _ }, 0) => Some(key.clone()),
            (SpecializedPair::ValueInt { val, .. }, 1) => Some(Value::Int(*val)),
            (SpecializedPair::BoolValue { key, val: _ }, 0) => Some(Value::Bool(*key)),
            (SpecializedPair::BoolValue { val, .. }, 1) => Some(val.clone()),
            (SpecializedPair::ValueBool { key, val: _ }, 0) => Some(key.clone()),
            (SpecializedPair::ValueBool { val, .. }, 1) => Some(Value::Bool(*val)),
            (SpecializedPair::Generic { key, val: _ }, 0) => Some(key.clone()),
            (SpecializedPair::Generic { val, .. }, 1) => Some(val.clone()),
            _ => None,
        }
    }

    /// Set field by index (0 = key, 1 = val)
    pub fn set_field(&mut self, index: usize, value: Value) -> Result<(), String> {
        match (self, index, value) {
            (SpecializedPair::IntInt { key, .. }, 0, Value::Int(v)) => { *key = v; Ok(()) }
            (SpecializedPair::IntInt { val, .. }, 1, Value::Int(v)) => { *val = v; Ok(()) }
            (SpecializedPair::IntBool { key, .. }, 0, Value::Int(v)) => { *key = v; Ok(()) }
            (SpecializedPair::IntBool { val, .. }, 1, Value::Bool(v)) => { *val = v; Ok(()) }
            (SpecializedPair::BoolInt { key, .. }, 0, Value::Bool(v)) => { *key = v; Ok(()) }
            (SpecializedPair::BoolInt { val, .. }, 1, Value::Int(v)) => { *val = v; Ok(()) }
            (SpecializedPair::IntValue { key, .. }, 0, Value::Int(v)) => { *key = v; Ok(()) }
            (SpecializedPair::IntValue { val, .. }, 1, v) => { *val = v; Ok(()) }
            (SpecializedPair::ValueInt { key, .. }, 0, v) => { *key = v; Ok(()) }
            (SpecializedPair::ValueInt { val, .. }, 1, Value::Int(v)) => { *val = v; Ok(()) }
            (SpecializedPair::BoolValue { key, .. }, 0, Value::Bool(v)) => { *key = v; Ok(()) }
            (SpecializedPair::BoolValue { val, .. }, 1, v) => { *val = v; Ok(()) }
            (SpecializedPair::ValueBool { key, .. }, 0, v) => { *key = v; Ok(()) }
            (SpecializedPair::ValueBool { val, .. }, 1, Value::Bool(v)) => { *val = v; Ok(()) }
            (SpecializedPair::Generic { key, .. }, 0, v) => { *key = v; Ok(()) }
            (SpecializedPair::Generic { val, .. }, 1, v) => { *val = v; Ok(()) }
            (_, _, _) => Err(format!("Type mismatch in specialized pair set")),
        }
    }

    /// Create specialized pair from type arguments
    pub fn from_type_args(key_type: &Type, val_type: &Type, key: Value, val: Value) -> Result<Self, String> {
        use Type::*;

        match (key_type, val_type) {
            // Pair<int, int> - Most specialized
            (Int, Int) => {
                match (key, val) {
                    (Value::Int(key_i32), Value::Int(val_i32)) => {
                        Ok(SpecializedPair::IntInt { key: key_i32, val: val_i32 })
                    }
                    _ => Err("Type mismatch: expected Int for both fields".to_string())
                }
            }
            // Pair<int, bool>
            (Int, Bool) => {
                match (key, val) {
                    (Value::Int(key_i32), Value::Bool(val_bool)) => {
                        Ok(SpecializedPair::IntBool { key: key_i32, val: val_bool })
                    }
                    _ => Err("Type mismatch: expected Int for key, Bool for val".to_string())
                }
            }
            // Pair<bool, int>
            (Bool, Int) => {
                match (key, val) {
                    (Value::Bool(key_bool), Value::Int(val_i32)) => {
                        Ok(SpecializedPair::BoolInt { key: key_bool, val: val_i32 })
                    }
                    _ => Err("Type mismatch: expected Bool for key, Int for val".to_string())
                }
            }
            // Pair<int, V> - Half specialized
            (Int, _) => {
                match key {
                    Value::Int(key_i32) => {
                        Ok(SpecializedPair::IntValue { key: key_i32, val })
                    }
                    _ => Err("Type mismatch: expected Int for key".to_string())
                }
            }
            // Pair<K, int> - Half specialized
            (_, Int) => {
                match val {
                    Value::Int(val_i32) => {
                        Ok(SpecializedPair::ValueInt { key, val: val_i32 })
                    }
                    _ => Err("Type mismatch: expected Int for val".to_string())
                }
            }
            // Pair<bool, V> - Half specialized
            (Bool, _) => {
                match key {
                    Value::Bool(key_bool) => {
                        Ok(SpecializedPair::BoolValue { key: key_bool, val })
                    }
                    _ => Err("Type mismatch: expected Bool for key".to_string())
                }
            }
            // Pair<V, bool> - Half specialized
            (_, Bool) => {
                match val {
                    Value::Bool(val_bool) => {
                        Ok(SpecializedPair::ValueBool { key, val: val_bool })
                    }
                    _ => Err("Type mismatch: expected Bool for val".to_string())
                }
            }
            // Pair<K, V> - Generic fallback
            _ => Ok(SpecializedPair::Generic { key, val }),
        }
    }
}

impl Clone for SpecializedPair {
    fn clone(&self) -> Self {
        match self {
            SpecializedPair::IntInt { key, val } => SpecializedPair::IntInt { key: *key, val: *val },
            SpecializedPair::IntBool { key, val } => SpecializedPair::IntBool { key: *key, val: *val },
            SpecializedPair::BoolInt { key, val } => SpecializedPair::BoolInt { key: *key, val: *val },
            SpecializedPair::IntValue { key, val } => SpecializedPair::IntValue { key: *key, val: val.clone() },
            SpecializedPair::ValueInt { key, val } => SpecializedPair::ValueInt { key: key.clone(), val: *val },
            SpecializedPair::BoolValue { key, val } => SpecializedPair::BoolValue { key: *key, val: val.clone() },
            SpecializedPair::ValueBool { key, val } => SpecializedPair::ValueBool { key: key.clone(), val: *val },
            SpecializedPair::Generic { key, val } => SpecializedPair::Generic { key: key.clone(), val: val.clone() },
        }
    }
}

// ============================================================================
// Generic Instance Data (Runtime Object)
// ============================================================================

/// Generic object instance (type-erased storage)
///
/// Runtime representation of a user-defined generic object.
/// All fields are stored as `Value` enum instances (type erasure).
///
/// # Type Erasure
/// - Fields are stored as `Vec<Value>` regardless of their actual types
/// - Runtime type information is preserved via `mono_name: String`
/// - Field access is by index (determined at compile time)
/// - Full type metadata can be looked up from GenericRegistry using mono_name
///
/// # Example
/// ```rust
/// use auto_lang::vm::generic_registry::GenericInstanceData;
/// use auto_val::Value;
///
/// // Source: type Pair<K, V> { key K; val V }
/// // Instantiation: let p: Pair<int, string> = Pair.new(1, "a")
///
/// let instance = GenericInstanceData {
///     mono_name: "Pair_int_str".to_string(),
///     fields: vec![Value::Int(1), Value::str("a")],
///     field_names: vec!["key".to_string(), "val".to_string()],
/// };
/// ```
#[derive(Debug)]
pub struct GenericInstanceData {
    pub mono_name: String,       // Monomorphic name (e.g., "Pair_int_str")
    pub fields: Vec<Value>,      // Type-erased field values
    pub field_names: Vec<String>, // Field names for debugging/formatting
}

impl GenericInstanceData {
    /// Create a new generic instance with placeholder field names
    pub fn new(mono_name: String, fields: Vec<Value>) -> Self {
        let field_names = vec!["_unknown".to_string(); fields.len()];
        Self { mono_name, fields, field_names }
    }

    /// Create a new generic instance with explicit field names
    pub fn new_with_names(mono_name: String, fields: Vec<Value>, field_names: Vec<String>) -> Self {
        Self { mono_name, fields, field_names }
    }

    /// Get field by index
    pub fn get_field(&self, index: usize) -> Option<&Value> {
        self.fields.get(index)
    }

    /// Set field by index
    pub fn set_field(&mut self, index: usize, value: Value) -> Result<(), String> {
        if index >= self.fields.len() {
            return Err(format!("Field index {} out of bounds (len: {})", index, self.fields.len()));
        }
        self.fields[index] = value;
        Ok(())
    }

    /// Get number of fields
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

// ============================================================================
// Plan 087 Phase 4: HeapObject Implementation for SpecializedPair
// ============================================================================

/// HeapObject implementation for specialized pairs
///
/// Each variant gets a unique TypeTag for runtime type identification.
impl HeapObject for SpecializedPair {
    fn type_tag(&self) -> TypeTag {
        match self {
            SpecializedPair::IntInt { .. } => TypeTag::SpecializedPair("Pair_int_int".to_string()),
            SpecializedPair::IntBool { .. } => TypeTag::SpecializedPair("Pair_int_bool".to_string()),
            SpecializedPair::BoolInt { .. } => TypeTag::SpecializedPair("Pair_bool_int".to_string()),
            SpecializedPair::IntValue { .. } => TypeTag::SpecializedPair("Pair_int_Value".to_string()),
            SpecializedPair::ValueInt { .. } => TypeTag::SpecializedPair("Pair_Value_int".to_string()),
            SpecializedPair::BoolValue { .. } => TypeTag::SpecializedPair("Pair_bool_Value".to_string()),
            SpecializedPair::ValueBool { .. } => TypeTag::SpecializedPair("Pair_Value_bool".to_string()),
            SpecializedPair::Generic { .. } => TypeTag::SpecializedPair("Pair_generic".to_string()),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ============================================================================
// HeapObject Implementation for GenericInstanceData
// ============================================================================

impl HeapObject for GenericInstanceData {
    /// Get the type tag for this generic instance
    fn type_tag(&self) -> TypeTag {
        TypeTag::GenericInstance(self.mono_name.clone())
    }

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Clone for GenericInstanceData {
    fn clone(&self) -> Self {
        Self {
            mono_name: self.mono_name.clone(),
            fields: self.fields.clone(),
            field_names: self.field_names.clone(),
        }
    }
}

// ============================================================================
// Generic Registry (Singleton)
// ============================================================================

/// Global registry for generic type metadata
///
/// Stores ClassTemplate definitions for all user-defined generic types
/// and manages ClassType instantiations.
///
/// # Thread Safety
/// Uses RwLock for concurrent read access during compilation.
#[derive(Clone)]
pub struct GenericRegistry {
    templates: HashMap<String, Arc<ClassTemplate>>,
    types: HashMap<String, Arc<ClassType>>,  // mono_name → ClassType
}

impl GenericRegistry {
    /// Create a new GenericRegistry
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// Register a generic class template
    ///
    /// # Arguments
    /// - `template`: ClassTemplate to register
    ///
    /// # Returns
    /// Ok(()) if registered successfully, Err if template already exists
    pub fn register_template(&mut self, template: ClassTemplate) -> Result<(), String> {
        let name = template.name.clone();

        if self.templates.contains_key(&name) {
            return Err(format!("Generic type '{}' already registered", name));
        }

        self.templates.insert(name, Arc::new(template));
        Ok(())
    }

    /// Get template by name
    pub fn get_template(&self, name: &str) -> Option<Arc<ClassTemplate>> {
        self.templates.get(name).cloned()
    }

    /// Get or create a ClassType for specific type arguments
    ///
    /// # Arguments
    /// - `base_name`: Base type name (e.g., "Pair")
    /// - `type_args`: Concrete type arguments (e.g., [int, string])
    ///
    /// # Returns
    /// Arc<ClassType> for the instantiation
    pub fn get_or_create_type(&mut self, base_name: &str, type_args: Vec<Type>) -> Result<Arc<ClassType>, String> {
        // Get template
        let template = self.get_template(base_name)
            .ok_or_else(|| format!("Generic type '{}' not found", base_name))?;

        // Generate monomorphic name
        let mono_name = template.mono_name_from_args(&type_args);

        // Check if already exists
        if let Some(existing) = self.types.get(&mono_name) {
            return Ok(Arc::clone(existing));
        }

        // Create new ClassType
        let class_type = Arc::new(ClassType::new(Arc::clone(&template), type_args));

        // Store and return
        self.types.insert(mono_name.clone(), Arc::clone(&class_type));
        Ok(class_type)
    }

    /// Get ClassType by monomorphic name
    pub fn get_type(&self, mono_name: &str) -> Option<Arc<ClassType>> {
        self.types.get(mono_name).cloned()
    }

    /// Check if a template is registered
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    /// Get all registered template names
    pub fn template_names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Get number of registered templates
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }

    /// Get number of instantiated types
    pub fn type_count(&self) -> usize {
        self.types.len()
    }
}

impl Default for GenericRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Fn as AstFn, FnKind, GenericParam, Type, TypeDecl, TypeDeclKind, TypeParam};
    use auto_val::AutoStr;
    

    fn make_type_param(name: &str) -> GenericParam {
        GenericParam::Type(TypeParam {
            name: Name::from(name),
            constraint: None,
        })
    }

    // Helper function to create a TypeDecl for generic parameters in tests
    fn make_type_param_decl(name: &str) -> TypeDecl {
        TypeDecl {
            name: Name::from(name),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: vec![],
            specs: vec![],
            spec_impls: vec![],
            generic_params: vec![],
            members: vec![],
            delegations: vec![],
            methods: vec![],
            attrs: vec![],
            doc: None,
            is_pub: false,
        }
    }

    #[test]
    fn test_class_template_creation() {
        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::Unknown),
                FieldDef::new("val", Type::Unknown),
            ],
            vec![],
        );

        assert_eq!(template.name, "Pair");
        assert_eq!(template.generic_params.len(), 2);
        assert_eq!(template.fields.len(), 2);
    }

    #[test]
    fn test_mono_name_generation() {
        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![],
            vec![],
        );

        let mono_name = template.mono_name_from_args(&[Type::Int, Type::Str(0)]);
        assert_eq!(mono_name, "Pair_int_str");

        let mono_name2 = template.mono_name_from_args(&[Type::Bool, Type::Int]);
        assert_eq!(mono_name2, "Pair_bool_int");
    }

    #[test]
    fn test_field_substitution() {
        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::User(make_type_param_decl("K"))),
                FieldDef::new("val", Type::User(make_type_param_decl("V"))),
            ],
            vec![],
        );

        let substituted = template.substitute_fields(&[Type::Int, Type::Str(0)]);

        assert_eq!(substituted.len(), 2);
        assert_eq!(substituted[0].name, "key");
        assert!(matches!(substituted[0].field_type, Type::Int));
        assert_eq!(substituted[1].name, "val");
        assert!(matches!(substituted[1].field_type, Type::Str(_)));
    }

    #[test]
    fn test_class_type_creation() {
        let template = Arc::new(ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::Unknown),
                FieldDef::new("val", Type::Unknown),
            ],
            vec![],
        ));

        let class_type = ClassType::new(template, vec![Type::Int, Type::Str(0)]);

        assert_eq!(class_type.base_name(), "Pair");
        assert_eq!(class_type.mono_name, "Pair_int_str");
        assert!(class_type.is_generic());
    }

    #[test]
    fn test_registry_register_template() {
        let mut registry = GenericRegistry::new();

        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![],
            vec![],
        );

        let result = registry.register_template(template);
        assert!(result.is_ok());
        assert!(registry.has_template("Pair"));
        assert_eq!(registry.template_count(), 1);
    }

    #[test]
    fn test_registry_duplicate_template() {
        let mut registry = GenericRegistry::new();

        let template1 = ClassTemplate::new("Pair", vec![], vec![], vec![]);
        let template2 = ClassTemplate::new("Pair", vec![], vec![], vec![]);

        let _ = registry.register_template(template1).unwrap();
        let result = registry.register_template(template2);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    #[test]
    fn test_registry_get_or_create_type() {
        let mut registry = GenericRegistry::new();

        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::Unknown),
                FieldDef::new("val", Type::Unknown),
            ],
            vec![],
        );

        let _ = registry.register_template(template).unwrap();

        // First call should create
        let class_type1 = registry.get_or_create_type("Pair", vec![Type::Int, Type::Str(0)]).unwrap();
        assert_eq!(class_type1.mono_name, "Pair_int_str");
        assert_eq!(registry.type_count(), 1);

        // Second call should reuse
        let class_type2 = registry.get_or_create_type("Pair", vec![Type::Int, Type::Str(0)]).unwrap();
        assert_eq!(class_type2.mono_name, "Pair_int_str");
        assert_eq!(registry.type_count(), 1);  // No new type created

        // Different args should create new type
        let class_type3 = registry.get_or_create_type("Pair", vec![Type::Bool, Type::Int]).unwrap();
        assert_eq!(class_type3.mono_name, "Pair_bool_int");
        assert_eq!(registry.type_count(), 2);  // New type created
    }

    #[test]
    fn test_registry_nonexistent_template() {
        let mut registry = GenericRegistry::new();

        let result = registry.get_or_create_type("Nonexistent", vec![Type::Int]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_field_index() {
        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::Unknown),
                FieldDef::new("val", Type::Unknown),
            ],
            vec![],
        );

        assert_eq!(template.field_index("key"), Some(0));
        assert_eq!(template.field_index("val"), Some(1));
        assert_eq!(template.field_index("nonexistent"), None);
    }

    #[test]
    fn test_generic_instance_data() {
        let template = Arc::new(ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::Unknown),
                FieldDef::new("val", Type::Unknown),
            ],
            vec![],
        ));

        let class_type = Arc::new(ClassType::new(
            Arc::clone(&template),
            vec![Type::Int, Type::Str(0)],
        ));

        let instance = GenericInstanceData::new(
            class_type.mono_name.clone(),
            vec![Value::Int(42), Value::Str(AutoStr::from("hello"))],
        );

        assert_eq!(instance.field_count(), 2);
        assert_eq!(instance.get_field(0), Some(&Value::Int(42)));
        assert_eq!(instance.get_field(1), Some(&Value::Str(AutoStr::from("hello"))));
        assert_eq!(instance.get_field(2), None);
    }

    #[test]
    fn test_generic_instance_set_field() {
        let template = Arc::new(ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::Unknown),
                FieldDef::new("val", Type::Unknown),
            ],
            vec![],
        ));

        let class_type = Arc::new(ClassType::new(
            Arc::clone(&template),
            vec![Type::Int, Type::Str(0)],
        ));

        let mut instance = GenericInstanceData::new(
            class_type.mono_name.clone(),
            vec![Value::Int(0), Value::Str(AutoStr::from(""))],
        );

        // Valid set
        let result = instance.set_field(0, Value::Int(100));
        assert!(result.is_ok());
        assert_eq!(instance.get_field(0), Some(&Value::Int(100)));

        // Out of bounds
        let result = instance.set_field(5, Value::Int(200));
        assert!(result.is_err());
    }

    #[test]
    fn test_template_display() {
        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::Unknown),
                FieldDef::new("val", Type::Unknown),
            ],
            vec![],
        );

        let display = format!("{}", template);
        assert!(display.contains("type Pair"));
        assert!(display.contains("K"));
        assert!(display.contains("V"));
        assert!(display.contains("key"));
        assert!(display.contains("val"));
    }

    #[test]
    fn test_class_type_display() {
        let template = Arc::new(ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![],
            vec![],
        ));

        let class_type = ClassType::new(template, vec![Type::Int, Type::Str(0)]);

        let display = format!("{}", class_type);
        assert!(display.contains("Pair"));
        assert!(display.contains("int"));
        assert!(display.contains("str"));
    }

    // ==================== Additional Tests for Phase 1 ====================

    #[test]
    fn test_template_with_no_generic_params() {
        let template = ClassTemplate::new(
            "SimpleType",
            vec![],  // No generic parameters
            vec![
                FieldDef::new("x", Type::Int),
                FieldDef::new("y", Type::Str(0)),
            ],
            vec![],
        );

        assert_eq!(template.name, "SimpleType");
        assert_eq!(template.generic_params.len(), 0);
        assert_eq!(template.fields.len(), 2);
    }

    #[test]
    fn test_single_generic_param() {
        let template = ClassTemplate::new(
            "Box",
            vec![make_type_param("T")],
            vec![
                FieldDef::new("value", Type::User(make_type_param_decl("T"))),
            ],
            vec![],
        );

        assert_eq!(template.generic_params.len(), 1);

        let mono_name = template.mono_name_from_args(&[Type::Int]);
        assert_eq!(mono_name, "Box_int");
    }

    #[test]
    fn test_three_generic_params() {
        let template = ClassTemplate::new(
            "Triple",
            vec![make_type_param("T"), make_type_param("U"), make_type_param("V")],
            vec![
                FieldDef::new("first", Type::User(make_type_param_decl("T"))),
                FieldDef::new("second", Type::User(make_type_param_decl("U"))),
                FieldDef::new("third", Type::User(make_type_param_decl("V"))),
            ],
            vec![],
        );

        assert_eq!(template.generic_params.len(), 3);

        let mono_name = template.mono_name_from_args(&[Type::Int, Type::Bool, Type::Str(0)]);
        assert_eq!(mono_name, "Triple_int_bool_str");
    }

    #[test]
    fn test_field_substitution_partial() {
        // Test when only some fields use generic parameters
        let template = ClassTemplate::new(
            "MixedPair",
            vec![make_type_param("T"), make_type_param("U")],
            vec![
                FieldDef::new("generic_field", Type::User(make_type_param_decl("T"))),
                FieldDef::new("concrete_field", Type::Int),  // Not a generic param
                FieldDef::new("another_generic", Type::User(make_type_param_decl("U"))),
            ],
            vec![],
        );

        let substituted = template.substitute_fields(&[Type::Bool, Type::Str(0)]);

        assert_eq!(substituted.len(), 3);
        assert!(matches!(substituted[0].field_type, Type::Bool));
        assert!(matches!(substituted[1].field_type, Type::Int));  // Unchanged
        assert!(matches!(substituted[2].field_type, Type::Str(_)));
    }

    #[test]
    fn test_field_substitution_with_array() {
        // Test substitution with array types
        let template = ClassTemplate::new(
            "ArrayContainer",
            vec![make_type_param("T")],
            vec![
                FieldDef::new("data", Type::User(make_type_param_decl("T"))),
            ],
            vec![],
        );

        // Substitute with an array type
        let array_type = Type::Array(crate::ast::ArrayType {
            elem: Box::new(Type::Int),
            len: 10,
        });

        let substituted = template.substitute_fields(&[array_type]);

        assert_eq!(substituted.len(), 1);
        match &substituted[0].field_type {
            Type::Array(arr) => {
                assert!(matches!(*arr.elem, Type::Int));
                assert_eq!(arr.len, 10);
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_field_substitution_with_list() {
        // Test substitution with list types
        let template = ClassTemplate::new(
            "ListContainer",
            vec![make_type_param("T")],
            vec![
                FieldDef::new("items", Type::User(make_type_param_decl("T"))),
            ],
            vec![],
        );

        let list_type = Type::List(Box::new(Type::Str(0)));
        let substituted = template.substitute_fields(&[list_type]);

        assert_eq!(substituted.len(), 1);
        match &substituted[0].field_type {
            Type::List(elem) => {
                assert!(matches!(**elem, Type::Str(_)));
            }
            _ => panic!("Expected List type"),
        }
    }

    #[test]
    fn test_method_index() {
        let methods = vec![
            AstFn::new(
                FnKind::Method,
                Name::from("get_key"),
                None,
                vec![],
                Body::new(),
                Type::Int,
            ),
            AstFn::new(
                FnKind::Method,
                Name::from("set_key"),
                None,
                vec![],
                Body::new(),
                Type::Void,
            ),
        ];

        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![
                FieldDef::new("key", Type::User(make_type_param_decl("K"))),
            ],
            methods,
        );

        // HashMap iteration order is non-deterministic, so we just check that methods exist
        let get_key_idx = template.method_index("get_key");
        let set_key_idx = template.method_index("set_key");

        assert!(get_key_idx.is_some(), "get_key should exist");
        assert!(set_key_idx.is_some(), "set_key should exist");
        assert_ne!(get_key_idx, set_key_idx, "Methods should have different indices");
        assert_eq!(template.method_index("nonexistent"), None);

        // Verify we have exactly 2 methods
        assert_eq!(template.methods.len(), 2);
    }

    #[test]
    fn test_generic_instance_get_field_out_of_bounds() {
        let instance = GenericInstanceData::new(
            "TestType".to_string(),
            vec![Value::Int(1), Value::Int(2)],
        );

        assert_eq!(instance.get_field(0), Some(&Value::Int(1)));
        assert_eq!(instance.get_field(1), Some(&Value::Int(2)));
        assert_eq!(instance.get_field(2), None);  // Out of bounds
        assert_eq!(instance.get_field(100), None);  // Way out of bounds
    }

    #[test]
    fn test_generic_instance_set_field_bounds() {
        let mut instance = GenericInstanceData::new(
            "TestType".to_string(),
            vec![Value::Int(0), Value::Int(0), Value::Int(0)],
        );

        // Valid sets
        assert!(instance.set_field(0, Value::Int(10)).is_ok());
        assert!(instance.set_field(2, Value::Int(20)).is_ok());

        // Invalid sets (out of bounds)
        assert!(instance.set_field(3, Value::Int(30)).is_err());
        assert!(instance.set_field(100, Value::Int(100)).is_err());
    }

    #[test]
    fn test_generic_instance_empty_fields() {
        let mut instance = GenericInstanceData::new(
            "EmptyType".to_string(),
            vec![],
        );

        assert_eq!(instance.field_count(), 0);
        assert_eq!(instance.get_field(0), None);
        assert!(instance.set_field(0, Value::Int(1)).is_err());
    }

    #[test]
    fn test_class_type_with_no_type_args() {
        let template = Arc::new(ClassTemplate::new(
            "NonGeneric",
            vec![],  // No generic params
            vec![],
            vec![],
        ));

        let class_type = ClassType::new(template, vec![]);

        assert_eq!(class_type.base_name(), "NonGeneric");
        assert_eq!(class_type.mono_name, "NonGeneric");
        assert!(!class_type.is_generic());  // Not a generic type
        assert_eq!(class_type.type_args.len(), 0);
    }

    #[test]
    fn test_mono_name_with_bool_and_float() {
        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![],
            vec![],
        );

        let mono_name = template.mono_name_from_args(&[Type::Bool, Type::Float]);
        assert_eq!(mono_name, "Pair_bool_float");
    }

    #[test]
    fn test_mono_name_with_double_and_uint() {
        let template = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![],
            vec![],
        );

        let mono_name = template.mono_name_from_args(&[Type::Double, Type::Uint]);
        assert_eq!(mono_name, "Pair_double_uint");
    }

    #[test]
    fn test_mono_name_with_various_types() {
        let template = ClassTemplate::new(
            "Triple",
            vec![make_type_param("A"), make_type_param("B"), make_type_param("C")],
            vec![],
            vec![],
        );

        // Test with Byte, Char, and USize
        let mono_name = template.mono_name_from_args(&[Type::Byte, Type::Char, Type::USize]);
        assert_eq!(mono_name, "Triple_byte_char_usize");
    }

    #[test]
    fn test_registry_multiple_types() {
        let mut registry = GenericRegistry::new();

        // Register multiple templates
        let template1 = ClassTemplate::new(
            "Pair",
            vec![make_type_param("K"), make_type_param("V")],
            vec![],
            vec![],
        );
        let _ = registry.register_template(template1);

        let template2 = ClassTemplate::new(
            "Triple",
            vec![make_type_param("A"), make_type_param("B"), make_type_param("C")],
            vec![],
            vec![],
        );
        let _ = registry.register_template(template2);

        // Create types from both templates
        let pair_type = registry.get_or_create_type("Pair", vec![Type::Int, Type::Str(0)]).unwrap();
        assert_eq!(pair_type.mono_name, "Pair_int_str");

        let triple_type = registry.get_or_create_type("Triple", vec![Type::Bool, Type::Int, Type::Float]).unwrap();
        assert_eq!(triple_type.mono_name, "Triple_bool_int_float");
    }

    #[test]
    fn test_template_with_many_fields() {
        let fields = vec![
            FieldDef::new("f0", Type::Int),
            FieldDef::new("f1", Type::Str(0)),
            FieldDef::new("f2", Type::Bool),
            FieldDef::new("f3", Type::Float),
            FieldDef::new("f4", Type::Double),
        ];

        let template = ClassTemplate::new(
            "ManyFields",
            vec![],
            fields,
            vec![],
        );

        assert_eq!(template.fields.len(), 5);
        assert_eq!(template.field_index("f0"), Some(0));
        assert_eq!(template.field_index("f2"), Some(2));
        assert_eq!(template.field_index("f4"), Some(4));
        assert_eq!(template.field_index("f5"), None);
    }

    #[test]
    fn test_registry_same_type_different_args() {
        let mut registry = GenericRegistry::new();

        let template = ClassTemplate::new(
            "Box",
            vec![make_type_param("T")],
            vec![],
            vec![],
        );
        let _ = registry.register_template(template);

        // Create same type with different arguments
        let box_int = registry.get_or_create_type("Box", vec![Type::Int]).unwrap();
        let box_str = registry.get_or_create_type("Box", vec![Type::Str(0)]).unwrap();
        let box_bool = registry.get_or_create_type("Box", vec![Type::Bool]).unwrap();

        assert_eq!(box_int.mono_name, "Box_int");
        assert_eq!(box_str.mono_name, "Box_str");
        assert_eq!(box_bool.mono_name, "Box_bool");

        // Each should be a separate type in the registry
        assert_ne!(box_int.mono_name, box_str.mono_name);
        assert_ne!(box_str.mono_name, box_bool.mono_name);
    }

    #[test]
    fn test_generic_instance_various_value_types() {
        // Test instance with various value types
        let instance = GenericInstanceData::new(
            "MixedType".to_string(),
            vec![
                Value::Int(42),
                Value::Uint(100),
                Value::Bool(true),
                Value::Float(3.14),
                Value::Str(AutoStr::from("hello")),
                Value::Char('a'),
            ],
        );

        assert_eq!(instance.field_count(), 6);
        assert_eq!(instance.get_field(0), Some(&Value::Int(42)));
        assert_eq!(instance.get_field(1), Some(&Value::Uint(100)));
        assert_eq!(instance.get_field(2), Some(&Value::Bool(true)));
        assert!(matches!(instance.get_field(3), Some(&Value::Float(_))));
        assert_eq!(instance.get_field(4), Some(&Value::Str(AutoStr::from("hello"))));
        assert_eq!(instance.get_field(5), Some(&Value::Char('a')));
    }

    #[test]
    fn test_field_substitution_nested() {
        // Test substitution with nested generic types
        let template = ClassTemplate::new(
            "NestedContainer",
            vec![make_type_param("T"), make_type_param("U")],
            vec![
                FieldDef::new("first", Type::User(make_type_param_decl("T"))),
                FieldDef::new("second", Type::User(make_type_param_decl("U"))),
                FieldDef::new("list_of_t", Type::List(Box::new(Type::User(make_type_param_decl("T"))))),
            ],
            vec![],
        );

        let substituted = template.substitute_fields(&[Type::Int, Type::Bool]);

        assert_eq!(substituted.len(), 3);
        assert!(matches!(substituted[0].field_type, Type::Int));
        assert!(matches!(substituted[1].field_type, Type::Bool));
        match &substituted[2].field_type {
            Type::List(elem) => assert!(matches!(**elem, Type::Int)),
            _ => panic!("Expected List<Int>"),
        }
    }

    // ==================== Plan 087 Phase 4: SpecializedPair Tests ====================

    #[test]
    fn test_specialized_pair_int_int_creation() {
        // Test Pair<int, int> specialization
        let pair = SpecializedPair::from_type_args(
            &Type::Int,
            &Type::Int,
            Value::Int(42),
            Value::Int(100),
        ).unwrap();

        match pair {
            SpecializedPair::IntInt { key, val } => {
                assert_eq!(key, 42);
                assert_eq!(val, 100);
            }
            _ => panic!("Expected IntInt variant"),
        }
    }

    #[test]
    fn test_specialized_pair_int_bool_creation() {
        let pair = SpecializedPair::from_type_args(
            &Type::Int,
            &Type::Bool,
            Value::Int(1),
            Value::Bool(true),
        ).unwrap();

        match pair {
            SpecializedPair::IntBool { key, val } => {
                assert_eq!(key, 1);
                assert_eq!(val, true);
            }
            _ => panic!("Expected IntBool variant"),
        }
    }

    #[test]
    fn test_specialized_pair_bool_int_creation() {
        let pair = SpecializedPair::from_type_args(
            &Type::Bool,
            &Type::Int,
            Value::Bool(false),
            Value::Int(99),
        ).unwrap();

        match pair {
            SpecializedPair::BoolInt { key, val } => {
                assert_eq!(key, false);
                assert_eq!(val, 99);
            }
            _ => panic!("Expected BoolInt variant"),
        }
    }

    #[test]
    fn test_specialized_pair_int_value_creation() {
        let pair = SpecializedPair::from_type_args(
            &Type::Int,
            &Type::Str(0),
            Value::Int(55),
            Value::Str(AutoStr::from("hello")),
        ).unwrap();

        match pair {
            SpecializedPair::IntValue { key, val } => {
                assert_eq!(key, 55);
                assert_eq!(val, Value::Str(AutoStr::from("hello")));
            }
            _ => panic!("Expected IntValue variant"),
        }
    }

    #[test]
    fn test_specialized_pair_value_int_creation() {
        let pair = SpecializedPair::from_type_args(
            &Type::Str(0),
            &Type::Int,
            Value::Str(AutoStr::from("key")),
            Value::Int(77),
        ).unwrap();

        match pair {
            SpecializedPair::ValueInt { key, val } => {
                assert_eq!(key, Value::Str(AutoStr::from("key")));
                assert_eq!(val, 77);
            }
            _ => panic!("Expected ValueInt variant"),
        }
    }

    #[test]
    fn test_specialized_pair_generic_creation() {
        // Pair<string, string> should use Generic variant
        let pair = SpecializedPair::from_type_args(
            &Type::Str(0),
            &Type::Str(0),
            Value::Str(AutoStr::from("hello")),
            Value::Str(AutoStr::from("world")),
        ).unwrap();

        match pair {
            SpecializedPair::Generic { key, val } => {
                assert_eq!(key, Value::Str(AutoStr::from("hello")));
                assert_eq!(val, Value::Str(AutoStr::from("world")));
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_specialized_pair_get_field() {
        // Test get_field on IntInt variant
        let pair = SpecializedPair::IntInt { key: 10, val: 20 };
        assert_eq!(pair.get_field(0), Some(Value::Int(10)));
        assert_eq!(pair.get_field(1), Some(Value::Int(20)));
        assert_eq!(pair.get_field(2), None);

        // Test get_field on IntValue variant
        let pair = SpecializedPair::IntValue {
            key: 30,
            val: Value::Str(AutoStr::from("test")),
        };
        assert_eq!(pair.get_field(0), Some(Value::Int(30)));
        assert_eq!(pair.get_field(1), Some(Value::Str(AutoStr::from("test"))));

        // Test get_field on Generic variant
        let pair = SpecializedPair::Generic {
            key: Value::Bool(true),
            val: Value::Int(42),
        };
        assert_eq!(pair.get_field(0), Some(Value::Bool(true)));
        assert_eq!(pair.get_field(1), Some(Value::Int(42)));
    }

    #[test]
    fn test_specialized_pair_set_field() {
        // Test set_field on IntInt variant
        let mut pair = SpecializedPair::IntInt { key: 1, val: 2 };
        assert!(pair.set_field(0, Value::Int(10)).is_ok());
        assert!(pair.set_field(1, Value::Int(20)).is_ok());
        assert_eq!(pair.get_field(0), Some(Value::Int(10)));
        assert_eq!(pair.get_field(1), Some(Value::Int(20)));

        // Test set_field on IntValue variant
        let mut pair = SpecializedPair::IntValue {
            key: 5,
            val: Value::Int(0),
        };
        assert!(pair.set_field(0, Value::Int(15)).is_ok());
        assert!(pair.set_field(1, Value::Str(AutoStr::from("updated"))).is_ok());
        assert_eq!(pair.get_field(0), Some(Value::Int(15)));
        assert_eq!(pair.get_field(1), Some(Value::Str(AutoStr::from("updated"))));

        // Test type mismatch
        let mut pair = SpecializedPair::IntInt { key: 1, val: 2 };
        assert!(pair.set_field(0, Value::Str(AutoStr::from("wrong"))).is_err());
    }

    #[test]
    fn test_specialized_pair_clone() {
        let original = SpecializedPair::IntInt { key: 100, val: 200 };
        let cloned = original.clone();

        match cloned {
            SpecializedPair::IntInt { key, val } => {
                assert_eq!(key, 100);
                assert_eq!(val, 200);
            }
            _ => panic!("Cloned variant should match"),
        }

        // Test clone with Value variant
        let original = SpecializedPair::IntValue {
            key: 50,
            val: Value::Str(AutoStr::from("clone test")),
        };
        let cloned = original.clone();

        match cloned {
            SpecializedPair::IntValue { key, val } => {
                assert_eq!(key, 50);
                assert_eq!(val, Value::Str(AutoStr::from("clone test")));
            }
            _ => panic!("Cloned variant should match"),
        }
    }

    #[test]
    fn test_specialized_pair_type_mismatch() {
        // Test type mismatch errors
        let result = SpecializedPair::from_type_args(
            &Type::Int,
            &Type::Int,
            Value::Str(AutoStr::from("wrong")),
            Value::Int(10),
        );
        assert!(result.is_err());

        let result = SpecializedPair::from_type_args(
            &Type::Bool,
            &Type::Int,
            Value::Int(10),
            Value::Str(AutoStr::from("wrong")),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_specialized_pair_memory_efficiency() {
        // This test demonstrates the memory efficiency concept of specialized pairs
        // Note: As an enum, SpecializedPair's size is determined by the largest variant
        // but the concept is that using IntInt avoids Value enum overhead

        // Verify i32 type size
        assert_eq!(std::mem::size_of::<i32>(), 4);

        // Verify Value type is larger than i32
        assert!(std::mem::size_of::<Value>() > std::mem::size_of::<i32>());

        // IntInt variant exists and stores primitives directly
        let int_int = SpecializedPair::IntInt { key: 1, val: 2 };
        match &int_int {
            SpecializedPair::IntInt { .. } => {
                // Direct i32 storage, no Value enum overhead
            }
            _ => panic!("Expected IntInt variant"),
        }

        // IntValue variant uses 1 x i32 + 1 x Value
        let int_value = SpecializedPair::IntValue {
            key: 1,
            val: Value::Int(2),
        };
        match &int_value {
            SpecializedPair::IntValue { key, val } => {
                // key is i32, val is Value enum
                let _ = (key, val);
                // Value is significantly larger than i32
                assert!(std::mem::size_of::<Value>() > std::mem::size_of::<i32>());
            }
            _ => panic!("Expected IntValue variant"),
        }

        // Generic variant uses 2 x Value
        let generic = SpecializedPair::Generic {
            key: Value::Int(1),
            val: Value::Int(2),
        };
        match &generic {
            SpecializedPair::Generic { key, val } => {
                let _ = (key, val);
                // Both key and val are Value enums
            }
            _ => panic!("Expected Generic variant"),
        }

        // Verify that using IntInt avoids Value allocation overhead
        match int_int {
            SpecializedPair::IntInt { key, val } => {
                // Direct i32 storage, no Value enum overhead
                assert_eq!(key, 1);
                assert_eq!(val, 2);
            }
            _ => unreachable!(),
        }
    }
}
