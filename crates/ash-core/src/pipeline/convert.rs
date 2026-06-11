//! Type inference and conversion tools for Atom pipeline
//!
//! Provides:
//! - `infer_atom_type()` — guess the semantic type from Value structure
//! - `file_entries_to_atom()` — convert AshFileEntry slice to Atom
//! - Helpers for common command outputs

use auto_val::{Value, Obj, Array};
use super::atom::{Atom, AtomType};
use super::atom_pipeline::AtomPipeline;
use crate::data::types::AshFileEntry;
use crate::data::convert::file_entry_to_value;

/// Infer an AtomType by inspecting the structure of a Value.
///
/// This is used by the bridge layer when converting legacy `PipelineData`
/// to `AtomPipeline` without explicit type information.
///
/// ## Inference rules
///
/// | Structure | Inferred type |
/// |-----------|--------------|
/// | Array of objects with `name`+`type` fields | FileList |
/// | Array of objects with `pid` field | ProcessList |
/// | Object with `name`+`type` fields | FileEntry |
/// | Object with `pid` field | ProcessEntry |
/// | Object with `cpu` or `memory` or `disk` field | SystemInfo |
/// | Object (generic) | Record |
/// | Array of uniform objects | Table |
/// | String | Text |
/// | Int/Float/Bool | Nothing (scalar) |
/// | Nil/Null/Void | Nothing |
pub fn infer_atom_type(value: &Value) -> AtomType {
    match value {
        Value::Str(_) => AtomType::Text,
        Value::Array(arr) => infer_array_type(arr),
        Value::Obj(obj) => infer_obj_type(obj),
        Value::Nil | Value::Null | Value::Void => AtomType::Nothing,
        _ => AtomType::Nothing,
    }
}

/// Infer type from an Array by looking at the first element.
fn infer_array_type(arr: &Array) -> AtomType {
    if arr.values.is_empty() {
        return AtomType::Nothing;
    }

    // Check first element for type clues
    let first = &arr.values[0];
    match first {
        Value::Obj(obj) => {
            if has_file_entry_fields(obj) {
                AtomType::FileList
            } else if has_process_entry_fields(obj) {
                AtomType::ProcessList
            } else {
                AtomType::Table
            }
        }
        Value::Str(_) => AtomType::Text, // array of strings = text list
        _ => AtomType::Table,
    }
}

/// Infer type from a single Object by looking at its fields.
fn infer_obj_type(obj: &Obj) -> AtomType {
    if has_file_entry_fields(obj) {
        AtomType::FileEntry
    } else if has_process_entry_fields(obj) {
        AtomType::ProcessEntry
    } else if has_system_info_fields(obj) {
        AtomType::SystemInfo
    } else {
        AtomType::Record
    }
}

/// Check if an object has file entry fields (name + type).
fn has_file_entry_fields(obj: &Obj) -> bool {
    obj.get("name").is_some() && obj.get("type").is_some()
}

/// Check if an object has process entry fields (pid).
fn has_process_entry_fields(obj: &Obj) -> bool {
    obj.get("pid").is_some()
}

/// Check if an object has system info fields (cpu/memory/disk).
fn has_system_info_fields(obj: &Obj) -> bool {
    obj.get("cpu").is_some() || obj.get("memory").is_some() || obj.get("disk").is_some()
}

// ── Conversion helpers ──────────────────────────────────────

/// Convert a slice of AshFileEntry to an Atom with FileList type.
pub fn file_entries_to_atom(entries: &[AshFileEntry]) -> Atom {
    let value = file_entries_to_value(entries);
    Atom::file_list(value)
}

/// Convert a slice of AshFileEntry to AtomPipeline.
pub fn file_entries_to_pipeline(entries: &[AshFileEntry]) -> AtomPipeline {
    AtomPipeline::from_atom(file_entries_to_atom(entries))
}

/// Convert a slice of AshFileEntry to Value::Array.
fn file_entries_to_value(entries: &[AshFileEntry]) -> Value {
    let values: Vec<Value> = entries.iter().map(file_entry_to_value).collect();
    Value::Array(Array { values })
}

/// Create a text AtomPipeline from a string.
pub fn text_pipeline(s: impl Into<String>) -> AtomPipeline {
    AtomPipeline::text(s)
}

/// Create an empty AtomPipeline.
pub fn empty_pipeline() -> AtomPipeline {
    AtomPipeline::empty()
}

/// Create a Path Atom from a string.
pub fn path_atom(s: impl Into<String>) -> Atom {
    Atom::path(s)
}

/// Create a Path AtomPipeline.
pub fn path_pipeline(s: impl Into<String>) -> AtomPipeline {
    AtomPipeline::from_atom(Atom::path(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file_entry_obj(name: &str, file_type: &str) -> Value {
        let mut obj = Obj::new();
        obj.set("name", Value::str(name));
        obj.set("type", Value::str(file_type));
        obj.set("size", Value::Int(100));
        Value::Obj(obj)
    }

    fn make_process_entry_obj(pid: i32, name: &str) -> Value {
        let mut obj = Obj::new();
        obj.set("pid", Value::Int(pid));
        obj.set("name", Value::str(name));
        Value::Obj(obj)
    }

    fn make_system_info_obj() -> Value {
        let mut obj = Obj::new();
        obj.set("cpu", Value::str("x86"));
        obj.set("memory", Value::Int(8192));
        Value::Obj(obj)
    }

    fn make_generic_obj() -> Value {
        let mut obj = Obj::new();
        obj.set("foo", Value::Int(1));
        obj.set("bar", Value::Int(2));
        Value::Obj(obj)
    }

    #[test]
    fn test_infer_string() {
        assert_eq!(infer_atom_type(&Value::str("hello")), AtomType::Text);
    }

    #[test]
    fn test_infer_nil() {
        assert_eq!(infer_atom_type(&Value::Nil), AtomType::Nothing);
    }

    #[test]
    fn test_infer_void() {
        assert_eq!(infer_atom_type(&Value::Void), AtomType::Nothing);
    }

    #[test]
    fn test_infer_int() {
        assert_eq!(infer_atom_type(&Value::Int(42)), AtomType::Nothing);
    }

    #[test]
    fn test_infer_file_entry_obj() {
        let v = make_file_entry_obj("test.txt", "file");
        assert_eq!(infer_atom_type(&v), AtomType::FileEntry);
    }

    #[test]
    fn test_infer_process_entry_obj() {
        let v = make_process_entry_obj(1234, "bash");
        assert_eq!(infer_atom_type(&v), AtomType::ProcessEntry);
    }

    #[test]
    fn test_infer_system_info_obj() {
        let v = make_system_info_obj();
        assert_eq!(infer_atom_type(&v), AtomType::SystemInfo);
    }

    #[test]
    fn test_infer_generic_obj() {
        let v = make_generic_obj();
        assert_eq!(infer_atom_type(&v), AtomType::Record);
    }

    #[test]
    fn test_infer_file_list_array() {
        let arr = Array {
            values: vec![
                make_file_entry_obj("a.txt", "file"),
                make_file_entry_obj("b.txt", "file"),
            ],
        };
        assert_eq!(infer_atom_type(&Value::Array(arr)), AtomType::FileList);
    }

    #[test]
    fn test_infer_process_list_array() {
        let arr = Array {
            values: vec![
                make_process_entry_obj(1, "init"),
                make_process_entry_obj(2, "bash"),
            ],
        };
        assert_eq!(infer_atom_type(&Value::Array(arr)), AtomType::ProcessList);
    }

    #[test]
    fn test_infer_empty_array() {
        let arr = Array { values: vec![] };
        assert_eq!(infer_atom_type(&Value::Array(arr)), AtomType::Nothing);
    }

    #[test]
    fn test_infer_table_array() {
        let arr = Array {
            values: vec![make_generic_obj(), make_generic_obj()],
        };
        assert_eq!(infer_atom_type(&Value::Array(arr)), AtomType::Table);
    }

    #[test]
    fn test_file_entries_to_atom() {
        let entries = vec![
            AshFileEntry {
                name: "test.txt".into(),
                file_type: crate::data::types::FileType::File,
                size: 100,
                modified: None,
                permissions: None,
                owner: None,
                target: None,
            },
        ];
        let atom = file_entries_to_atom(&entries);
        assert_eq!(atom.atom_type(), AtomType::FileList);
        assert!(atom.is_structured());
    }

    #[test]
    fn test_text_pipeline() {
        let p = text_pipeline("hello");
        assert!(p.is_text());
        assert_eq!(p.as_text(), "hello");
    }

    #[test]
    fn test_path_pipeline() {
        let p = path_pipeline("/tmp");
        assert_eq!(p.atom_type(), AtomType::Path);
    }
}
