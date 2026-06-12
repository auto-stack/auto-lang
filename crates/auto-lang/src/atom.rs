//! Atom integration for AutoLang.
//!
//! The core `Atom`, `AtomBuilder`, and error types have moved to the
//! `auto-atom` crate. This module re-exports them for backward compatibility
//! and provides `AtomReader`, which parses Auto code using the AutoLang
//! interpreter.

// Re-export core Atom types from the lightweight auto-atom crate.
pub use auto_atom::{Atom, AtomBuilder, AtomError, AtomResult, EMPTY};

use auto_val::AutoStr;

/// Reader for Atom data from Auto code
///
/// AtomReader provides a convenient way to parse Auto code and directly
/// extract Atom data structures without the overhead of AutoConfig.
///
/// # Examples
///
/// ```ignore
/// use auto_lang::atom::AtomReader;
///
/// let mut reader = AtomReader::new();
/// let atom = reader.parse("config { name: \"test\"; value: 42; }").unwrap();
/// assert!(atom.is_node());
/// ```
pub struct AtomReader {
    /// Plan 091: Use AutoVM-based interpreter
    interp: crate::interpreter::AutoInterpreter,
}

impl AtomReader {
    /// Creates a new AtomReader
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::AtomReader;
    ///
    /// let reader = AtomReader::new();
    /// ```
    pub fn new() -> Self {
        Self {
            interp: crate::interpreter::AutoInterpreter::new(),
        }
    }

    /// Parses Auto code and returns an Atom
    ///
    /// This method evaluates the provided Auto code in CONFIG mode and
    /// converts the result into an Atom.
    ///
    /// # Arguments
    ///
    /// * `code` - The Auto code to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(Atom)` if parsing succeeds, otherwise returns an error.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use auto_lang::atom::AtomReader;
    ///
    /// let mut reader = AtomReader::new();
    /// let atom = reader.parse("atom { x: 1; y: 2; }").unwrap();
    /// assert!(atom.is_node());
    /// ```
    pub fn parse(&mut self, code: impl Into<auto_val::AutoStr>) -> AtomResult<Atom> {
        let code = code.into();
        let result = self.interp
            .eval(code.as_str())
            .map_err(|e| AtomError::ConversionFailed(format!("Failed to parse code: {}", e)))?;

        // Special handling for bare arrays and objects
        match result {
            auto_val::Value::Array(a) => {
                // Return the array directly
                return Ok(Atom::Array(a));
            }
            auto_val::Value::Obj(o) => {
                // Return the object directly
                return Ok(Atom::Obj(o));
            }
            other => Atom::new(other),
        }
    }

    /// Reads an Atom from a file
    ///
    /// This method reads the contents of a file and parses it as Auto code
    /// to produce an Atom.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// Returns `Ok(Atom)` if reading and parsing succeed, otherwise returns an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::AtomReader;
    /// use std::path::Path;
    ///
    /// let mut reader = AtomReader::new();
    /// # // Assuming test.at exists
    /// # // let atom = reader.read(Path::new("test.at")).unwrap();
    /// ```
    pub fn read(&mut self, path: impl AsRef<std::path::Path>) -> AtomResult<Atom> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            AtomError::ConversionFailed(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        self.parse(content)
    }
}

impl Default for AtomReader {
    fn default() -> Self {
        Self::new()
    }
}
