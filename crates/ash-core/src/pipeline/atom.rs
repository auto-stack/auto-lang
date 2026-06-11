//! Atom type system — semantic type tags for pipeline data
//!
//! An `Atom` wraps a `Value` with a semantic `AtomType` tag so downstream
//! commands know *what kind* of data is flowing through the pipeline,
//! not just its raw structure.

use auto_val::Value;
use crate::cmd::value_helpers::format_value_for_display;

/// Semantic type tag for pipeline data.
///
/// Each variant describes what the data *means*, enabling commands to
/// make intelligent decisions without runtime inspection of Value fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomType {
    // ── File system ──────────────────────────────────────
    /// Single file/directory entry with metadata
    FileEntry,
    /// Array of file entries (e.g. `ls` output)
    FileList,

    // ── Process ──────────────────────────────────────────
    /// Single process entry
    ProcessEntry,
    /// Array of process entries (e.g. `ps` output)
    ProcessList,

    // ── System info ──────────────────────────────────────
    /// Disk/storage entry
    DiskEntry,
    /// CPU information record
    CpuInfo,
    /// Memory information record
    MemoryInfo,
    /// Composite system information
    SystemInfo,

    // ── Search / data ────────────────────────────────────
    /// Search match results
    MatchList,
    /// Word/line/char count result
    CountResult,

    // ── Generic structured ───────────────────────────────
    /// Generic table (array of uniform objects)
    Table,
    /// Generic record (single object)
    Record,

    // ── Text / scalar ────────────────────────────────────
    /// Plain unstructured text
    Text,
    /// Filesystem path string
    Path,

    // ── Build / run ──────────────────────────────────────
    /// Build operation result
    BuildResult,
    /// Run operation result
    RunResult,

    // ── Meta ─────────────────────────────────────────────
    /// Help text / documentation
    HelpInfo,
    /// Untyped / unknown data
    Nothing,
}

/// A typed data atom flowing through the pipeline.
///
/// Wraps a `Value` with a semantic `AtomType` tag. Commands produce Atoms
/// to indicate what their output means; downstream commands inspect the
/// type tag to decide how to process the data.
#[derive(Debug, Clone)]
pub struct Atom {
    /// The actual data payload
    pub value: Value,
    /// Semantic type tag
    pub atom_type: AtomType,
}

impl Atom {
    /// Create a new Atom with an explicit type tag.
    pub fn new(value: Value, atom_type: AtomType) -> Self {
        Self { value, atom_type }
    }

    /// Create an Atom with type `Nothing` (untyped).
    pub fn nothing(value: Value) -> Self {
        Self::new(value, AtomType::Nothing)
    }

    /// Create a plain-text Atom.
    pub fn text(s: impl Into<String>) -> Self {
        Self::new(Value::str(&s.into()), AtomType::Text)
    }

    /// Create a FileList Atom.
    pub fn file_list(value: Value) -> Self {
        Self::new(value, AtomType::FileList)
    }

    /// Create a ProcessList Atom.
    pub fn process_list(value: Value) -> Self {
        Self::new(value, AtomType::ProcessList)
    }

    /// Create a SystemInfo Atom.
    pub fn system_info(value: Value) -> Self {
        Self::new(value, AtomType::SystemInfo)
    }

    /// Create a Path Atom.
    pub fn path(s: impl Into<String>) -> Self {
        Self::new(Value::str(&s.into()), AtomType::Path)
    }

    /// Create an empty Atom (Void value, Nothing type).
    pub fn empty() -> Self {
        Self::new(Value::Void, AtomType::Nothing)
    }

    /// Get the type tag.
    pub fn atom_type(&self) -> AtomType {
        self.atom_type
    }

    /// Check if this is a structured (non-text, non-nothing) Atom.
    pub fn is_structured(&self) -> bool {
        !matches!(self.atom_type, AtomType::Text | AtomType::Nothing)
    }

    /// Check if this Atom represents empty/no data.
    pub fn is_empty(&self) -> bool {
        matches!(self.value, Value::Nil | Value::Null | Value::Void)
    }

    /// Convert to display text (consumes the Atom).
    ///
    /// For string values, returns the raw content without quotes.
    /// For other types, delegates to `format_value_for_display`.
    pub fn into_text(self) -> String {
        match &self.value {
            Value::Str(_) => self.value.as_str().to_string(),
            other => format_value_for_display(other),
        }
    }

    /// Get display text without consuming.
    ///
    /// For string values, returns the raw content without quotes.
    /// For other types, delegates to `format_value_for_display`.
    pub fn as_text(&self) -> String {
        match &self.value {
            Value::Str(_) => self.value.as_str().to_string(),
            other => format_value_for_display(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_new() {
        let atom = Atom::new(Value::Int(42), AtomType::CountResult);
        assert_eq!(atom.atom_type(), AtomType::CountResult);
        assert!(!atom.is_empty());
    }

    #[test]
    fn test_atom_text() {
        let atom = Atom::text("hello");
        assert_eq!(atom.atom_type(), AtomType::Text);
        assert!(!atom.is_structured());
        assert_eq!(atom.as_text(), "hello");
    }

    #[test]
    fn test_atom_file_list() {
        let atom = Atom::file_list(Value::Void);
        assert_eq!(atom.atom_type(), AtomType::FileList);
        assert!(atom.is_structured());
    }

    #[test]
    fn test_atom_process_list() {
        let atom = Atom::process_list(Value::Void);
        assert_eq!(atom.atom_type(), AtomType::ProcessList);
        assert!(atom.is_structured());
    }

    #[test]
    fn test_atom_path() {
        let atom = Atom::path("/home/user");
        assert_eq!(atom.atom_type(), AtomType::Path);
        assert_eq!(atom.as_text(), "/home/user");
    }

    #[test]
    fn test_atom_empty() {
        let atom = Atom::empty();
        assert!(atom.is_empty());
        assert_eq!(atom.atom_type(), AtomType::Nothing);
    }

    #[test]
    fn test_atom_nothing() {
        let atom = Atom::nothing(Value::Int(99));
        assert_eq!(atom.atom_type(), AtomType::Nothing);
        assert!(!atom.is_structured());
    }

    #[test]
    fn test_atom_into_text() {
        let atom = Atom::text("world");
        let text = atom.into_text();
        assert_eq!(text, "world");
    }
}
