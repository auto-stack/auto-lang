//! Scope Manager - minimal scope functionality for Parser
//! Plan 091: Part of Universe removal

use crate::ast::Type;
use auto_val::AutoStr;
use std::collections::HashMap;

/// Type alias definition: (parameters, target type)
pub type TypeAliasDef = (Vec<AutoStr>, Type);

/// Minimal scope manager
#[derive(Debug, Default, Clone)]
pub struct ScopeManager {
    type_aliases: HashMap<AutoStr, TypeAliasDef>,
    env_vals: HashMap<AutoStr, AutoStr>,
}

impl ScopeManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Lookup a type by name (stub - returns None for now)
    pub fn lookup_type(&self, _name: &str) -> Option<crate::ast::Type> {
        // Plan 091: This should integrate with TypeStore
        None
    }
    
    /// Lookup metadata by name (stub - returns None for now)
    pub fn lookup_meta(&self, _name: &str) -> Option<crate::scope::Meta> {
        // Plan 091: This should integrate with Database
        None
    }

    pub fn lookup_type_alias(&self, name: &str) -> Option<&TypeAliasDef> {
        self.type_aliases.get(name)
    }

    pub fn register_type_alias(&mut self, name: AutoStr, params: Vec<AutoStr>, target: Type) {
        self.type_aliases.insert(name, (params, target));
    }

    pub fn get_env_val(&self, name: &str) -> Option<AutoStr> {
        self.env_vals.get(name).cloned()
    }

    pub fn set_env_val(&mut self, name: AutoStr, value: AutoStr) {
        self.env_vals.insert(name, value);
    }
}
