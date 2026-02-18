//! VM Context for native function execution
//!
//! Plan 091: Replaces Evaler in VmFunction/VmMethod signatures

use crate::universe::{Universe, VmRefData};
use auto_val::{AutoStr, Type, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Shared Universe reference
pub type SharedUniverse = Rc<RefCell<Universe>>;

/// Context passed to native VM functions
///
/// Provides access to:
/// - Type lookup
/// - VM reference management (for StringBuilder, HashMap, etc.)
/// - Universe access for advanced operations
pub struct VmContext {
    /// Whether to skip type checking
    pub skip_check: bool,
    /// Universe for type lookup and scope management
    universe: SharedUniverse,
}

impl VmContext {
    /// Create a new VmContext with a fresh Universe
    pub fn new() -> Self {
        Self {
            skip_check: false,
            universe: Rc::new(RefCell::new(Universe::new())),
        }
    }

    /// Create a VmContext with an existing Universe
    pub fn with_universe(universe: SharedUniverse) -> Self {
        Self {
            skip_check: false,
            universe,
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
        self.universe
            .borrow()
            .lookup_type_meta(name)
            .and_then(|meta| {
                if let crate::scope::Meta::Type(ty) = meta.as_ref() {
                    // Convert ast::Type to auto_val::Type
                    Some(self.convert_ast_type(ty))
                } else {
                    None
                }
            })
            .unwrap_or(Type::Any)
    }

    /// Convert ast::Type to auto_val::Type
    /// Maps complex AST types to simpler runtime types
    fn convert_ast_type(&self, ty: &crate::ast::Type) -> Type {
        use crate::ast::Type as AstType;
        match ty {
            // Primitive types - direct mapping
            AstType::Int => Type::Int,
            AstType::Uint => Type::Uint,
            AstType::USize => Type::Uint,
            AstType::I64 => Type::Int,
            AstType::U64 => Type::Uint,
            AstType::Float => Type::Float,
            AstType::Double => Type::Double,
            AstType::Bool => Type::Bool,
            AstType::Byte => Type::Byte,
            AstType::Char => Type::Char,
            AstType::Str(_) => Type::Str,
            AstType::CStr => Type::CStr,
            AstType::StrSlice => Type::Str,
            AstType::Void => Type::Void,
            AstType::Unknown => Type::Any,

            // User-defined types
            AstType::User(decl) => Type::User(decl.name.clone().into()),
            AstType::Enum(decl) => Type::Enum(decl.borrow().name.clone().into()),
            AstType::Tag(t) => Type::Tag(t.borrow().name.clone().into()),
            AstType::Union(u) => Type::Union(u.name.clone().into()),

            // Complex types - map to User with type name
            AstType::List(_) => Type::User("List".into()),
            AstType::Slice(_) => Type::User("Slice".into()),
            AstType::Array(_) => Type::Array,
            AstType::RuntimeArray(_) => Type::Array,
            AstType::Ptr(_) => Type::Ptr,
            AstType::Reference(inner) => self.convert_ast_type(inner),
            AstType::GenericInstance(inst) => Type::User(inst.base_name.clone().into()),
            AstType::Storage(_) => Type::User("Storage".into()),
            AstType::Fn(_, _) => Type::User("Fn".into()),

            // Special types
            AstType::Spec(spec) => Type::User(spec.borrow().name.clone().into()),
            AstType::CStruct(decl) => Type::User(decl.name.clone().into()),
            AstType::Linear(inner) => self.convert_ast_type(inner),
            AstType::Variadic => Type::Any,
        }
    }

    /// Get access to the Universe
    pub fn universe(&self) -> &SharedUniverse {
        &self.universe
    }

    /// Add a VM reference and return its unique ID
    /// Delegates to Universe's vm_refs storage
    pub fn add_vmref(&self, data: VmRefData) -> usize {
        self.universe.borrow_mut().add_vmref(data)
    }

    /// Drop a VM reference by ID
    /// Delegates to Universe's vm_refs storage
    pub fn drop_vmref(&self, id: usize) {
        self.universe.borrow_mut().drop_vmref(id);
    }
}

impl Default for VmContext {
    fn default() -> Self {
        Self::new()
    }
}

/// VM function signature without Evaler dependency
pub type NativeFn = fn(&mut VmContext, Value) -> Value;

/// VM method signature without Evaler dependency
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
