// Plan 081 Phase 1: AutoVM as Default Execution Mode
// AutoVM is now the default execution engine for all AutoLang code
// Plan 091: Evaluator option removed, always uses AutoVM

use crate::error::AutoResult;

/// Execution engine selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionEngine {
    /// AutoVM bytecode VM (default, faster)
    AutoVM,
    /// Legacy evaluator (deprecated, redirects to AutoVM)
    #[deprecated(since = "0.10.0", note = "Use AutoVM instead (Plan 091)")]
    Evaluator,
}

impl ExecutionEngine {
    /// Get the default execution engine
    ///
    /// **Plan 081**: AutoVM is now the default execution engine.
    pub fn default_engine() -> Self {
        ExecutionEngine::AutoVM
    }

    /// Get execution engine from environment variable
    ///
    /// Environment variable: `AUTO_EXECUTION_ENGINE`
    /// Values: "autovm", "vm" (evaluator option deprecated)
    pub fn from_env() -> Option<Self> {
        std::env::var("AUTO_EXECUTION_ENGINE")
            .ok()
            .map(|engine_str| {
                match engine_str.to_lowercase().as_str() {
                    "autovm" | "vm" => ExecutionEngine::AutoVM,
                    "evaluator" | "eval" | "tree" => {
                        // Plan 091: Evaluator deprecated, log warning and use AutoVM
                        eprintln!("WARNING: 'evaluator' engine is deprecated. Using AutoVM instead.");
                        ExecutionEngine::AutoVM
                    }
                    _ => ExecutionEngine::default_engine(),
                }
            })
            .or(Some(ExecutionEngine::default_engine()))
    }

    /// Get the execution engine (compile-time default with env override)
    pub fn get() -> Self {
        Self::from_env().unwrap_or_else(Self::default_engine)
    }
}

/// Execute code using the selected engine
pub fn execute_with_engine(engine: ExecutionEngine, code: &str) -> AutoResult<String> {
    match engine {
        ExecutionEngine::AutoVM => {
            // Use AutoVM (compile to bytecode, execute)
            crate::run_autovm(code)
        }
        #[allow(deprecated)]
        ExecutionEngine::Evaluator => {
            // Plan 091: Evaluator redirects to AutoVM
            crate::run_autovm(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_engine() {
        let engine = ExecutionEngine::default_engine();
        assert_eq!(engine, ExecutionEngine::AutoVM);
        println!("Default engine: {:?}", engine);
    }

    #[test]
    fn test_engine_from_env() {
        let original = std::env::var("AUTO_EXECUTION_ENGINE").ok();

        std::env::set_var("AUTO_EXECUTION_ENGINE", "autovm");
        let engine = ExecutionEngine::from_env().unwrap();
        assert_eq!(engine, ExecutionEngine::AutoVM);

        // Restore original value
        match original {
            Some(val) => std::env::set_var("AUTO_EXECUTION_ENGINE", val),
            None => std::env::remove_var("AUTO_EXECUTION_ENGINE"),
        }
    }
}
