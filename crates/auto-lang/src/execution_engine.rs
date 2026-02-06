// Plan 073 Phase 9.3: Execution Engine Selection
// Provides configuration to choose between AutoVM and Evaluator

use crate::error::AutoResult;

/// Execution engine selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionEngine {
    /// AutoVM bytecode VM (default, faster)
    AutoVM,
    /// TreeWalker evaluator (legacy, slower)
    Evaluator,
}

impl ExecutionEngine {
    /// Get the default execution engine based on compile-time features
    pub fn default_engine() -> Self {
        // Priority: AutoVM (if enabled) > Evaluator (fallback)
        #[cfg(feature = "use-bigvm")]
        {
            return ExecutionEngine::AutoVM;
        }

        #[cfg(not(feature = "use-bigvm"))]
        {
            return ExecutionEngine::Evaluator;
        }
    }

    /// Get execution engine from environment variable
    ///
    /// Environment variable: `AUTO_EXECUTION_ENGINE`
    /// Values: "autovm", "evaluator", "vm", "eval"
    pub fn from_env() -> Option<Self> {
        std::env::var("AUTO_EXECUTION_ENGINE")
            .ok()
            .map(|engine_str| {
                match engine_str.to_lowercase().as_str() {
                    "autovm" | "vm" => ExecutionEngine::AutoVM,
                    "evaluator" | "eval" | "tree" => ExecutionEngine::Evaluator,
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
        ExecutionEngine::Evaluator => {
            execute_with_evaluator(code)
        }
    }
}

/// Execute code using the TreeWalker evaluator
fn execute_with_evaluator(code: &str) -> AutoResult<String> {
    use crate::interp::Interpreter;
    let mut interpreter = Interpreter::new();
    interpreter.interpret(code)?;
    Ok(interpreter.result.repr().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_engine() {
        let engine = ExecutionEngine::default_engine();
        // Should be AutoVM if feature is enabled, else Evaluator
        println!("Default engine: {:?}", engine);
    }

    #[test]
    fn test_engine_from_env() {
        // Test env override (in a controlled way)
        let original = std::env::var("AUTO_EXECUTION_ENGINE").ok();

        std::env::set_var("AUTO_EXECUTION_ENGINE", "autovm");
        let engine = ExecutionEngine::from_env().unwrap();
        assert_eq!(engine, ExecutionEngine::AutoVM);

        std::env::set_var("AUTO_EXECUTION_ENGINE", "evaluator");
        let engine = ExecutionEngine::from_env().unwrap();
        assert_eq!(engine, ExecutionEngine::Evaluator);

        // Restore original value
        match original {
            Some(val) => std::env::set_var("AUTO_EXECUTION_ENGINE", val),
            None => std::env::remove_var("AUTO_EXECUTION_ENGINE"),
        }
    }
}
