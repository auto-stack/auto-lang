//! Conversion between AutoLang Value and Shell Value
//!
//! Provides bidirectional conversion between AutoLang's Value type and Shell's ShellValue type.

use auto_val::Value;
use auto_val::AutoStr;

use super::value::ShellValue;

/// Convert AutoLang Value to Shell Value
pub fn auto_to_shell(value: &Value) -> ShellValue {
    // TODO: Implement conversion in Phase 2
    match value {
        Value::Int(n) => ShellValue::Int(*n as i64),
        Value::Float(n) => ShellValue::Float(*n),
        Value::Str(s) => ShellValue::String(s.to_string()),
        Value::Nil => ShellValue::Null,
        Value::Bool(b) => ShellValue::Bool(*b),
        _ => ShellValue::Null,
    }
}

/// Convert Shell Value to AutoLang Value
pub fn shell_to_auto(value: &ShellValue) -> Value {
    match value {
        ShellValue::Int(n) => Value::Int(*n as i32),
        ShellValue::Float(n) => Value::Float(*n),
        ShellValue::String(s) => Value::str(s.as_str()),
        ShellValue::Bool(b) => Value::Bool(*b),
        ShellValue::Null => Value::Nil,
        _ => Value::Nil, // TODO: Handle arrays, objects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_to_shell_int() {
        let auto = Value::Int(42);
        let shell = auto_to_shell(&auto);
        assert_eq!(shell, ShellValue::Int(42));
    }

    #[test]
    fn test_shell_to_auto_int() {
        let shell = ShellValue::int(42);
        let auto = shell_to_auto(&shell);
        assert_eq!(auto, Value::Int(42));
    }

    #[test]
    fn test_auto_to_shell_string() {
        let auto = Value::str("hello");
        let shell = auto_to_shell(&auto);
        assert_eq!(shell, ShellValue::String("hello".to_string()));
    }

    #[test]
    fn test_roundtrip_int() {
        let original = Value::Int(42);
        let shell = auto_to_shell(&original);
        let converted = shell_to_auto(&shell);
        assert_eq!(original, converted);
    }
}
