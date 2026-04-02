//! VM Context for native function execution
//!
//! Plan 091: Independent of Universe - uses internal storage

use super::types::VmRefData;
use auto_val::{AutoStr, Type, Value};
use std::cell::RefCell;
use std::collections::HashMap;

/// Context passed to native VM functions
///
/// Provides access to:
/// - Type lookup (built-in types)
/// - VM reference management (for StringBuilder, HashMap, etc.)
pub struct VmContext {
    /// Whether to skip type checking
    pub skip_check: bool,
    /// VM reference storage
    vm_refs: HashMap<usize, RefCell<VmRefData>>,
    /// Counter for VM reference IDs
    vmref_counter: usize,
}

impl VmContext {
    /// Create a new VmContext
    pub fn new() -> Self {
        Self {
            skip_check: false,
            vm_refs: HashMap::new(),
            vmref_counter: 0,
        }
    }

    /// Create a context with skip_check enabled
    pub fn skip_check() -> Self {
        let mut ctx = Self::new();
        ctx.skip_check = true;
        ctx
    }

    /// Report an error value
    pub fn error(&self, msg: impl Into<AutoStr>) -> Value {
        Value::Error(msg.into())
    }

    /// Lookup a type by name
    /// Returns auto_val::Type for compatibility with Value/Instance types
    pub fn lookup_type(&self, name: &str) -> Type {
        match name {
            "int" => Type::Int,
            "uint" => Type::Uint,
            "float" => Type::Float,
            "double" => Type::Double,
            "bool" => Type::Bool,
            "byte" => Type::Byte,
            "char" => Type::Char,
            "str" => Type::Str,
            "string" => Type::String,
            "cstr" => Type::CStr,
            "void" => Type::Void,
            _ => Type::User(name.into()),
        }
    }

    /// Add a VM reference and return its unique ID
    pub fn add_vmref(&mut self, data: VmRefData) -> usize {
        let id = self.vmref_counter;
        self.vmref_counter += 1;
        self.vm_refs.insert(id, RefCell::new(data));
        id
    }

    /// Get a VM reference by ID
    pub fn get_vmref(&self, id: usize) -> Option<&RefCell<VmRefData>> {
        self.vm_refs.get(&id)
    }

    /// Drop a VM reference by ID
    pub fn drop_vmref(&mut self, id: usize) {
        self.vm_refs.remove(&id);
    }
}

impl Default for VmContext {
    fn default() -> Self {
        Self::new()
    }
}

/// VM function signature
pub type NativeFn = fn(&mut VmContext, Value) -> Value;

/// VM method signature
pub type NativeMethod = fn(&mut VmContext, &mut Value, Vec<Value>) -> Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_context() {
        let ctx = VmContext::new();
        assert!(!ctx.skip_check);
    }

    #[test]
    fn test_lookup_type() {
        let ctx = VmContext::new();
        let ty = ctx.lookup_type("int");
        assert!(matches!(ty, Type::Int));
    }
}
