// =============================================================================
// ExecutionEngine: Runtime execution state (ephemeral, per-run)
// =============================================================================
//
// The ExecutionEngine contains ONLY runtime state:
// - Variable values
// - Call stack management
// - VM references (file handles, collections, etc.)
//
// This is deliberately separated from compile-time concerns (types, symbols)
// which live in the Database.
//
// Phase 1.5: Basic ExecutionEngine structure (placeholder for future migration)
// Phase 2: Extract runtime logic from Universe
// Phase 3: Full integration with Database

use std::collections::HashMap;
use auto_val::{AutoStr, Obj};

/// Runtime execution engine (ephemeral, per-run)
///
/// Contains ONLY runtime state needed during execution.
/// Deliberately separated from compile-time concerns (Database).
///
/// # Architecture
///
/// ```text
/// Compile-time (persistent)    Runtime (ephemeral)
/// ┌──────────────────┐         ┌──────────────────┐
/// │   Database        │         │ ExecutionEngine  │
/// │ - Types           │   ←→    │ - Values         │
/// │ - Symbols         │         │ - VM Refs        │
/// │ - Fragments       │         │ - Stack          │
/// └──────────────────┘         └──────────────────┘
/// ```
///
/// # Phase 1.5: Structure Only
///
/// Currently a placeholder structure to establish the separation.
/// In Phase 2, runtime logic will be migrated from Universe to here.
pub struct ExecutionEngine {
    /// Environment variables
    pub env_vals: HashMap<AutoStr, String>,

    /// Command-line arguments
    pub args: Obj,

    /// Execution state (placeholder for Phase 2)
    /// This will hold values, stack, VM refs, etc.
    _state_placeholder: usize,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new() -> Self {
        Self {
            env_vals: HashMap::new(),
            args: Obj::new(),
            _state_placeholder: 0,
        }
    }

    /// Set environment variable
    pub fn set_env_val(&mut self, name: &str, value: String) {
        self.env_vals.insert(name.into(), value);
    }

    /// Get environment variable
    pub fn get_env_val(&self, name: &str) -> Option<&str> {
        self.env_vals.get(name).map(|s| s.as_str())
    }

    /// Set arguments
    pub fn set_args(&mut self, args: &Obj) {
        self.args = args.clone();
    }

    /// Get arguments
    pub fn get_args(&self) -> &Obj {
        &self.args
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Value;

    #[test]
    fn test_execution_engine_new() {
        let engine = ExecutionEngine::new();
        assert_eq!(engine.env_vals.len(), 0);
    }

    #[test]
    fn test_env_vals() {
        let mut engine = ExecutionEngine::new();

        engine.set_env_val("TEST", "value".to_string());
        assert_eq!(engine.get_env_val("TEST"), Some("value"));
        assert_eq!(engine.get_env_val("MISSING"), None);
    }

    #[test]
    fn test_args() {
        let mut engine = ExecutionEngine::new();
        let mut args = Obj::new();
        args.set("key", Value::Int(100));

        engine.set_args(&args);

        let retrieved = engine.get_args();
        assert_eq!(retrieved.get("key"), Some(Value::Int(100)));
    }
}

