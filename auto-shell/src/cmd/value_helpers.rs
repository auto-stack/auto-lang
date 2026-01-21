//! Helper methods for working with Auto values in shell commands
//!
//! This module provides convenience functions for converting between
//! shell data and Auto's Value types.

use auto_val::{Value, Obj, Array, AutoStr};

/// Build a file entry object (for ls command output)
pub fn build_file_entry(
    name: impl Into<AutoStr>,
    file_type: impl Into<AutoStr>,
    size: Option<i64>,
    modified: Option<String>,
    permissions: Option<String>,
) -> Value {
    let mut obj = Obj::new();
    obj.set("name", Value::str(name));
    obj.set("type", Value::str(file_type));

    if let Some(s) = size {
        obj.set("size", Value::Int(s as i32));
    }

    if let Some(m) = modified {
        obj.set("modified", Value::str(m));
    }

    if let Some(p) = permissions {
        obj.set("permissions", Value::str(p));
    }

    Value::Obj(obj)
}

/// Format a Value for display
///
/// This converts structured Auto values to human-readable text output.
/// - Arrays become tables (when possible)
/// - Objects become key-value lists
/// - Primitives use their Display implementation
pub fn format_value_for_display(val: &Value) -> String {
    match val {
        Value::Array(arr) => {
            // Try to format as table if all elements are objects
            format_array_as_table(arr)
        }
        Value::Obj(obj) => {
            // Format as key-value list
            format_obj_as_record(obj)
        }
        _ => val.to_string(),
    }
}

/// Format an Array as a table (if all elements are objects)
fn format_array_as_table(arr: &Array) -> String {
    // Check if all elements are objects
    if arr.values.is_empty() {
        return String::new();
    }

    let all_objects = arr.iter().all(|v| matches!(v, Value::Obj(_)));
    if !all_objects {
        // Not all objects, use default string representation
        return arr.to_string();
    }

    // Collect all object keys to determine columns
    let mut columns: Vec<String> = Vec::new();
    for item in arr.iter() {
        if let Value::Obj(obj) = item {
            for (key, _) in obj.iter() {
                let key_str = key.to_string();
                if !columns.contains(&key_str) {
                    columns.push(key_str);
                }
            }
        }
    }

    // If no columns, return empty
    if columns.is_empty() {
        return String::new();
    }

    // Sort columns with common preferences (ls -l format for long, name first for short)
    columns.sort_by(|a, b| {
        // For long format (has permissions), use: permissions, owner, size, modified, name
        // For short format, use: name, type, size, modified
        let has_long_format = a == "permissions" || a == "owner" || b == "permissions" || b == "owner";

        if has_long_format {
            // Long format: permissions, owner, size, modified, name
            let long_priority = ["permissions", "owner", "size", "modified", "name"];
            let a_pos = long_priority.iter().position(|&p| p == a).unwrap_or(usize::MAX);
            let b_pos = long_priority.iter().position(|&p| p == b).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos).then_with(|| a.cmp(b))
        } else {
            // Short format: name, type, size, modified
            let short_priority = ["name", "type", "size", "modified"];
            let a_pos = short_priority.iter().position(|&p| p == a).unwrap_or(usize::MAX);
            let b_pos = short_priority.iter().position(|&p| p == b).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos).then_with(|| a.cmp(b))
        }
    });

    // Calculate column widths
    let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();

    for item in arr.iter() {
        if let Value::Obj(obj) = item {
            for (i, col) in columns.iter().enumerate() {
                if let Some(value) = obj.get(col.as_str()) {
                    let value_str = format_value_for_table(&value);
                    widths[i] = widths[i].max(value_str.len());
                }
            }
        }
    }

    // Build table rows
    let mut result = String::new();

    // Header row with capitalized column names
    let header: Vec<String> = columns.iter().enumerate()
        .map(|(i, col)| {
            let title = match col.as_str() {
                "permissions" => "Permissions",
                "owner" => "Owner",
                "size" => "Size",
                "modified" => "Modified",
                "name" => "Name",
                "type" => "Type",
                _ => &col,
            };
            format!("{:<width$}", title, width = widths[i])
        })
        .collect();
    result.push_str(&header.join("  "));
    result.push('\n');

    // Data rows
    for item in arr.iter() {
        if let Value::Obj(obj) = item {
            let row: Vec<String> = columns.iter().enumerate()
                .map(|(i, col)| {
                    if let Some(value) = obj.get(col.as_str()) {
                        let value_str = format_value_for_table(&value);
                        format!("{:<width$}", value_str, width = widths[i])
                    } else {
                        format!("{:<width$}", "", width = widths[i])
                    }
                })
                .collect();
            result.push_str(&row.join("  "));
            result.push('\n');
        }
    }

    result.trim_end().to_string()
}

/// Format a Value for table cell display (without extra quotes for strings)
fn format_value_for_table(val: &Value) -> String {
    match val {
        Value::Str(s) => s.to_string(),  // No quotes for table cells
        Value::Int(i) => i.to_string(),
        Value::Uint(u) => u.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Nil => "nil".to_string(),
        Value::Null => "null".to_string(),
        Value::Void => "void".to_string(),
        _ => val.to_string(),
    }
}

fn format_obj_as_record(obj: &Obj) -> String {
    let mut parts = Vec::new();
    for (key, val) in obj.iter() {
        parts.push(format!("{}: {}", key, val));
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_file_entry() {
        let entry = build_file_entry(
            "test.txt",
            "file",
            Some(1024),
            Some("2025-01-22 10:00".to_string()),
            Some("-rw-r--r--".to_string()),
        );

        if let Value::Obj(obj) = entry {
            assert_eq!(obj.get("name").unwrap(), Value::str("test.txt"));
            assert_eq!(obj.get("type").unwrap(), Value::str("file"));
            assert_eq!(obj.get("size").unwrap(), Value::Int(1024));
        } else {
            panic!("Expected Obj");
        }
    }

    #[test]
    fn test_build_file_entry_minimal() {
        let entry = build_file_entry("test", "file", None, None, None);

        if let Value::Obj(obj) = entry {
            assert_eq!(obj.get("name").unwrap(), Value::str("test"));
            assert_eq!(obj.get("type").unwrap(), Value::str("file"));
            assert!(obj.get("size").is_none());
            assert!(obj.get("modified").is_none());
            assert!(obj.get("permissions").is_none());
        } else {
            panic!("Expected Obj");
        }
    }

    #[test]
    fn test_format_primitive() {
        let val = Value::Int(42);
        let formatted = format_value_for_display(&val);
        assert_eq!(formatted, "42");
    }

    #[test]
    fn test_format_string() {
        let val = Value::str("hello");
        let formatted = format_value_for_display(&val);
        // Value::Str adds quotes in Display implementation
        assert_eq!(formatted, "\"hello\"");
    }

    #[test]
    fn test_format_obj() {
        let mut obj = Obj::new();
        obj.set("key", Value::str("value"));
        obj.set("count", Value::Int(42));

        let val = Value::Obj(obj);
        let formatted = format_value_for_display(&val);
        // format_obj_as_record outputs "key: \"value\", count: 42"
        assert!(formatted.contains("key:"));
        assert!(formatted.contains("\"value\""));
        assert!(formatted.contains("count:"));
        assert!(formatted.contains("42"));
    }
}
