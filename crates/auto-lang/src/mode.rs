// Plan 081 Phase 2: Execution Mode Selection
//
// This module defines the ExecutionMode enum which specifies how AutoLang code
// should be executed or transpiled.

/// Execution or transpilation mode for AutoLang code
///
/// **Plan 081**: Each package or dependency can specify its execution mode.
/// This allows mixing AutoVM bytecode, C transpilation, Rust transpilation,
/// and Evaluator interpretation within a single project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionMode {
    /// AutoVM bytecode execution (default)
    /// Code is compiled to ABC bytecode and executed on the AutoVM virtual machine
    AutoVM,

    /// TreeWalker evaluator (legacy, slower)
    /// Code is interpreted directly using the TreeWalker interpreter
    Evaluator,

    /// C transpilation (a2c)
    /// Code is transpiled to C for embedded systems or native compilation
    C,

    /// Rust transpilation (a2r)
    /// Code is transpiled to Rust for native applications
    Rust,
}

impl ExecutionMode {
    /// Parse execution mode from string
    ///
    /// # Examples
    ///
    /// ```
    /// use auto_lang::mode::ExecutionMode;
    ///
    /// assert_eq!(ExecutionMode::from_str("autovm"), Some(ExecutionMode::AutoVM));
    /// assert_eq!(ExecutionMode::from_str("vm"), Some(ExecutionMode::AutoVM));
    /// assert_eq!(ExecutionMode::from_str("c"), Some(ExecutionMode::C));
    /// assert_eq!(ExecutionMode::from_str("rust"), Some(ExecutionMode::Rust));
    /// assert_eq!(ExecutionMode::from_str("evaluator"), Some(ExecutionMode::Evaluator));
    /// assert_eq!(ExecutionMode::from_str("invalid"), None);
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "autovm" | "vm" | "bytecode" => Some(ExecutionMode::AutoVM),
            "evaluator" | "eval" | "tree" | "treewalker" => Some(ExecutionMode::Evaluator),
            "c" | "a2c" | "transpile-c" => Some(ExecutionMode::C),
            "rust" | "a2r" | "transpile-rust" => Some(ExecutionMode::Rust),
            _ => None,
        }
    }

    /// Convert execution mode to string representation
    ///
    /// # Examples
    ///
    /// ```
    /// use auto_lang::mode::ExecutionMode;
    ///
    /// assert_eq!(ExecutionMode::AutoVM.as_str(), "autovm");
    /// assert_eq!(ExecutionMode::Evaluator.as_str(), "evaluator");
    /// assert_eq!(ExecutionMode::C.as_str(), "c");
    /// assert_eq!(ExecutionMode::Rust.as_str(), "rust");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::AutoVM => "autovm",
            ExecutionMode::Evaluator => "evaluator",
            ExecutionMode::C => "c",
            ExecutionMode::Rust => "rust",
        }
    }

    /// Check if this mode requires compilation (as opposed to interpretation)
    pub fn requires_compilation(&self) -> bool {
        matches!(self, ExecutionMode::AutoVM | ExecutionMode::C | ExecutionMode::Rust)
    }

    /// Check if this mode is a transpilation mode (to C or Rust)
    pub fn is_transpilation(&self) -> bool {
        matches!(self, ExecutionMode::C | ExecutionMode::Rust)
    }

    /// Check if this mode uses bytecode VM
    pub fn is_bytecode(&self) -> bool {
        matches!(self, ExecutionMode::AutoVM)
    }

    /// Check if this mode uses interpreter
    pub fn is_interpreter(&self) -> bool {
        matches!(self, ExecutionMode::Evaluator)
    }
}

impl Default for ExecutionMode {
    fn default() -> Self {
        // Plan 081: AutoVM is the default execution mode
        ExecutionMode::AutoVM
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ExecutionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ExecutionMode::from_str(s)
            .ok_or_else(|| format!("Invalid execution mode: '{}'. Expected: autovm, evaluator, c, or rust", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        // AutoVM variants
        assert_eq!(ExecutionMode::from_str("autovm"), Some(ExecutionMode::AutoVM));
        assert_eq!(ExecutionMode::from_str("vm"), Some(ExecutionMode::AutoVM));
        assert_eq!(ExecutionMode::from_str("bytecode"), Some(ExecutionMode::AutoVM));

        // Evaluator variants
        assert_eq!(ExecutionMode::from_str("evaluator"), Some(ExecutionMode::Evaluator));
        assert_eq!(ExecutionMode::from_str("eval"), Some(ExecutionMode::Evaluator));
        assert_eq!(ExecutionMode::from_str("tree"), Some(ExecutionMode::Evaluator));

        // C variants
        assert_eq!(ExecutionMode::from_str("c"), Some(ExecutionMode::C));
        assert_eq!(ExecutionMode::from_str("a2c"), Some(ExecutionMode::C));

        // Rust variants
        assert_eq!(ExecutionMode::from_str("rust"), Some(ExecutionMode::Rust));
        assert_eq!(ExecutionMode::from_str("a2r"), Some(ExecutionMode::Rust));

        // Invalid
        assert_eq!(ExecutionMode::from_str("invalid"), None);
        assert_eq!(ExecutionMode::from_str(""), None);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(ExecutionMode::AutoVM.as_str(), "autovm");
        assert_eq!(ExecutionMode::Evaluator.as_str(), "evaluator");
        assert_eq!(ExecutionMode::C.as_str(), "c");
        assert_eq!(ExecutionMode::Rust.as_str(), "rust");
    }

    #[test]
    fn test_default() {
        assert_eq!(ExecutionMode::default(), ExecutionMode::AutoVM);
    }

    #[test]
    fn test_requires_compilation() {
        assert!(ExecutionMode::AutoVM.requires_compilation());
        assert!(ExecutionMode::C.requires_compilation());
        assert!(ExecutionMode::Rust.requires_compilation());
        assert!(!ExecutionMode::Evaluator.requires_compilation());
    }

    #[test]
    fn test_is_transpilation() {
        assert!(ExecutionMode::C.is_transpilation());
        assert!(ExecutionMode::Rust.is_transpilation());
        assert!(!ExecutionMode::AutoVM.is_transpilation());
        assert!(!ExecutionMode::Evaluator.is_transpilation());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ExecutionMode::AutoVM), "autovm");
        assert_eq!(format!("{}", ExecutionMode::C), "c");
    }

    #[test]
    fn test_from_str_trait() {
        use std::str::FromStr;

        assert_eq!(ExecutionMode::from_str("autovm").unwrap(), ExecutionMode::AutoVM);
        assert_eq!(ExecutionMode::from_str("c").unwrap(), ExecutionMode::C);

        // Option doesn't have is_err(), use is_none() instead
        assert!(ExecutionMode::from_str("invalid").is_none());
    }
}
