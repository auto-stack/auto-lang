//! # AutoVM-based Interpreter Interface
//!
//! This module provides a simple interpreter interface built on top of AutoVM,
//! designed to replace the legacy eval.rs and interp.rs implementations.
//!
//! ## Migration Guide
//!
//! Replace:
//! ```ignore
//! let mut evaler = Evaler::new(scope);
//! let result = evaler.eval(&ast)?;
//! ```
//!
//! With:
//! ```ignore
//! let mut interp = AutoInterpreter::new();
//! let result = interp.eval(code)?;
//! ```

use crate::ast::Code;
use crate::atom::Atom;
use crate::AutoResult;
use auto_val::Value;
use std::collections::HashMap;

mod vm_interpreter;

pub use vm_interpreter::VmInterpreter;

/// Debug logging macro - only prints when VM debug mode is enabled
macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if crate::is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

/// AutoVM-based interpreter with simple API
///
/// This is the recommended way to evaluate Auto code programmatically.
/// It wraps AutoVM and provides a simple interface similar to the old Evaler.
///
/// # Example
///
/// ```ignore
/// use auto_lang::interpreter::AutoInterpreter;
///
/// let mut interp = AutoInterpreter::new();
///
/// // Evaluate simple expressions
/// let result = interp.eval("1 + 2")?;
/// assert_eq!(result, Value::Int(3));
///
/// // Evaluate with persistent state
/// interp.eval("let x = 10")?;
/// let result = interp.eval("x + 5")?;
/// assert_eq!(result, Value::Int(15));
/// ```
pub struct AutoInterpreter {
    /// The underlying VM interpreter
    vm: VmInterpreter,

    /// Cached parsed code for incremental evaluation
    cache: HashMap<String, Code>,

    /// Whether to preserve state between evaluations
    _persistent: bool,

    /// F-string note character (for template evaluation)
    fstr_note: char,
}

impl AutoInterpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {
            vm: VmInterpreter::new(),
            cache: HashMap::new(),
            _persistent: true,
            fstr_note: '$',
        }
    }

    /// Create a stateless interpreter (each eval is independent)
    pub fn new_stateless() -> Self {
        Self {
            vm: VmInterpreter::new(),
            cache: HashMap::new(),
            _persistent: false,
            fstr_note: '$',
        }
    }

    /// Set the F-string note character for template evaluation
    ///
    /// This character is used as the prefix for F-string expressions in templates.
    /// Default is '$'.
    pub fn with_fstr_note(mut self, note: char) -> Self {
        self.fstr_note = note;
        self
    }

    /// Evaluate code and return the result
    ///
    /// # Arguments
    /// * `code` - Auto source code to evaluate
    ///
    /// # Returns
    /// The result value of the evaluation
    pub fn eval(&mut self, code: &str) -> AutoResult<Value> {
        self.vm.run(code)
    }

    /// Evaluate a template with F-string support
    ///
    /// The template is preprocessed using a "Flip" process:
    /// - Lines starting with `{fstr_note} ` (e.g., `$ `) are treated as pure Auto code.
    /// - All other lines are treated as TEXT with embedded expressions and wrapped in
    ///   backticks to form an AutoLang F-string.
    ///
    /// An optional prelude string can be provided, which is evaluated in the same scope
    /// before the generated template code. This allows variables to be injected.
    pub fn eval_template(&mut self, prelude: &str, template: &str) -> AutoResult<Value> {
        let mut flipped_code = String::from(prelude);
        // Ensure prelude ends with a newline to prevent concatenation issues
        if !prelude.is_empty() && !prelude.ends_with('\n') {
            flipped_code.push('\n');
        }
        let prefix = format!("{} ", self.fstr_note);

        flipped_code.push_str("var __out__ = \"\"\n");

        for line in template.lines() {
            if line.starts_with(&prefix) {
                // Code line: strip the prefix and append as is
                flipped_code.push_str(&line[prefix.len()..]);
                flipped_code.push('\n');
            } else {
                // Text line: wrap in F-string backticks
                // Note: we need to escape existing backticks in the text to be safe
                let escaped_line = line.replace("`", "\\`");
                flipped_code.push_str(&format!("__out__ = __out__ + `{}\n`\n", escaped_line));
            }
        }

        flipped_code.push_str("__out__\n");
        vm_debug!("DEBUG flipped_code:\n{}", flipped_code);

        // Evaluate the generated code
        self.eval(&flipped_code)
    }

    /// Evaluate a function call with arguments
    ///
    /// # Arguments
    /// * `fn_name` - Function name to call
    /// * `args` - Arguments to pass
    pub fn call(&mut self, fn_name: &str, args: Vec<Value>) -> AutoResult<Value> {
        self.vm.call(fn_name, args)
    }

    /// Set a global variable
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Variable value
    pub fn set_global(&mut self, name: &str, value: Value) {
        self.vm.set_global(name, value);
    }

    /// Get a global variable
    ///
    /// # Arguments
    /// * `name` - Variable name
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.vm.get_global(name)
    }

    /// Merge an Atom's values into the interpreter scope
    ///
    /// This method extracts values from an Atom and sets them as global variables.
    /// Used for passing data to templates in auto-gen.
    ///
    /// # Arguments
    /// * `atom` - The Atom containing data to merge
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut interp = AutoInterpreter::new();
    /// let atom = Atom::assemble(vec![
    ///     Value::pair("name", "Alice"),
    ///     Value::pair("age", 30),
    /// ]).unwrap();
    /// interp.merge_atom(&atom);
    /// // Now "name" and "age" are available as globals
    /// ```
    pub fn merge_atom(&mut self, atom: &Atom) {
        match atom {
            Atom::Node(node) => {
                // Set node properties as globals
                for (key, value) in node.props_iter() {
                    if let Some(name) = key.name() {
                        self.set_global(name, value.clone());
                    }
                }
            }
            Atom::Obj(obj) => {
                // Set object entries as globals
                for (key, value) in obj.iter() {
                    if let Some(name) = key.name() {
                        self.set_global(name, value.clone());
                    }
                }
            }
            Atom::Array(arr) => {
                // Set array elements as numbered globals
                for (i, value) in arr.iter().enumerate() {
                    self.set_global(&format!("item_{}", i), value.clone());
                }
            }
            Atom::Empty => {}
        }
    }

    /// Clear all state
    pub fn reset(&mut self) {
        self.vm.reset();
        self.cache.clear();
    }

    /// Check if the interpreter has a function defined
    pub fn has_function(&self, name: &str) -> bool {
        self.vm.has_function(name)
    }

    /// Get list of defined functions
    pub fn get_functions(&self) -> Vec<String> {
        self.vm.get_functions()
    }
}

impl Default for AutoInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: These tests are disabled until result extraction is implemented
    // Currently the interpreter returns Nil for all evaluations

    #[test]
    #[ignore = "Result extraction not yet implemented"]
    fn test_simple_eval() {
        let mut interp = AutoInterpreter::new();
        let result = interp.eval("1 + 2").unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    #[ignore = "Result extraction not yet implemented"]
    fn test_string_eval() {
        let mut interp = AutoInterpreter::new();
        let result = interp.eval(r#""hello" + " world""#).unwrap();
        assert!(matches!(result, Value::Str(_)));
    }

    #[test]
    #[ignore = "Function calling not yet implemented"]
    fn test_function_call() {
        let mut interp = AutoInterpreter::new();
        interp
            .eval(
                r#"
            fn add(a int, b int) int {
                a + b
            }
        "#,
            )
            .unwrap();
        let result = interp
            .call("add", vec![Value::Int(3), Value::Int(4)])
            .unwrap();
        assert_eq!(result, Value::Int(7));
    }

    // Test that parsing and compilation work (even without result extraction)
    #[test]
    fn test_eval_succeeds() {
        let mut interp = AutoInterpreter::new();
        // Should not error
        interp.eval("1 + 2").unwrap();
        interp.eval("fn main() { print(\"hello\") }").unwrap();
    }

    #[test]
    fn test_merge_atom_obj() {
        let mut interp = AutoInterpreter::new();
        let mut obj = auto_val::Obj::new();
        obj.set("name", Value::str("Alice"));
        obj.set("age", Value::Int(30));
        let atom = Atom::Obj(obj);

        interp.merge_atom(&atom);

        assert_eq!(interp.get_global("name"), Some(Value::str("Alice")));
        assert_eq!(interp.get_global("age"), Some(Value::Int(30)));
    }
}
