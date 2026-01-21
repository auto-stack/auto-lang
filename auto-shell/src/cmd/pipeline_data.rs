//! Pipeline data handling for structured data flows
//!
//! This module provides the PipelineData enum which allows commands to pass
//! either structured Auto values (zero-copy) or plain text (for external commands).

use auto_val::Value;
use crate::cmd::value_helpers::format_value_for_display;

/// Pipeline data can be structured (Value) or text (for external commands)
///
/// This enables two modes:
/// - **Value mode**: Zero-copy structured data between Auto-shell commands
/// - **Text mode**: Plain text for external commands and legacy compatibility
#[derive(Debug, Clone)]
pub enum PipelineData {
    /// Structured Auto value (zero-copy between commands)
    Value(Value),

    /// Plain text (for external commands, legacy compatibility)
    Text(String),
}

impl PipelineData {
    /// Create PipelineData from an Auto value
    pub fn from_value(val: Value) -> Self {
        PipelineData::Value(val)
    }

    /// Create PipelineData from text
    pub fn from_text(s: String) -> Self {
        PipelineData::Text(s)
    }

    /// Create empty PipelineData (no input)
    pub fn empty() -> Self {
        PipelineData::Text(String::new())
    }

    /// Get reference to Value if this is Value mode
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            PipelineData::Value(v) => Some(v),
            _ => None,
        }
    }

    /// Convert to text (for display or external commands)
    ///
    /// For Value mode, uses format_value_for_display to format arrays as tables
    /// and objects as records. For Text mode, returns the string directly.
    pub fn into_text(self) -> String {
        match self {
            PipelineData::Value(v) => format_value_for_display(&v),
            PipelineData::Text(s) => s,
        }
    }

    /// Get as text without consuming
    pub fn as_text(&self) -> String {
        match self {
            PipelineData::Value(v) => format_value_for_display(v),
            PipelineData::Text(s) => s.clone(),
        }
    }

    /// Check if this contains structured data
    pub fn is_value(&self) -> bool {
        matches!(self, PipelineData::Value(_))
    }

    /// Check if this contains text
    pub fn is_text(&self) -> bool {
        matches!(self, PipelineData::Text(_))
    }

    /// Check if this is empty
    pub fn is_empty(&self) -> bool {
        match self {
            PipelineData::Value(v) => {
                matches!(v, Value::Nil | Value::Null | Value::Void)
            }
            PipelineData::Text(s) => s.is_empty(),
        }
    }
}

impl From<Value> for PipelineData {
    fn from(val: Value) -> Self {
        PipelineData::Value(val)
    }
}

impl From<String> for PipelineData {
    fn from(s: String) -> Self {
        PipelineData::Text(s)
    }
}

impl From<&str> for PipelineData {
    fn from(s: &str) -> Self {
        PipelineData::Text(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::{Obj, Array};

    #[test]
    fn test_pipeline_data_from_value() {
        let val = Value::Int(42);
        let data = PipelineData::from_value(val.clone());

        assert!(data.is_value());
        assert!(!data.is_text());
        assert_eq!(data.as_value(), Some(&val));
    }

    #[test]
    fn test_pipeline_data_from_text() {
        let text = "hello".to_string();
        let data = PipelineData::from_text(text.clone());

        assert!(data.is_text());
        assert!(!data.is_value());
        assert_eq!(data.into_text(), text);
    }

    #[test]
    fn test_pipeline_data_empty() {
        let data = PipelineData::empty();
        assert!(data.is_empty());
    }

    #[test]
    fn test_pipeline_data_from_value_trait() {
        let val = Value::str("test");
        let data: PipelineData = val.clone().into();
        assert!(data.is_value());
        assert_eq!(data.as_value(), Some(&val));
    }

    #[test]
    fn test_pipeline_data_from_string_trait() {
        let s = "hello".to_string();
        let data: PipelineData = s.clone().into();
        assert!(data.is_text());
    }

    #[test]
    fn test_pipeline_data_complex_value() {
        let mut obj = Obj::new();
        obj.set("name", Value::str("test"));
        obj.set("count", Value::Int(42));

        let val = Value::Obj(obj);
        let data = PipelineData::from_value(val);

        assert!(data.is_value());
        let text = data.into_text();
        assert!(text.contains("name"));
        assert!(text.contains("count"));
    }

    #[test]
    fn test_pipeline_data_nil_is_empty() {
        let data = PipelineData::from_value(Value::Nil);
        assert!(data.is_empty());
    }

    #[test]
    fn test_pipeline_data_void_is_empty() {
        let data = PipelineData::from_value(Value::Void);
        assert!(data.is_empty());
    }

    #[test]
    fn test_pipeline_data_array_not_empty() {
        let arr = Array::from(vec![Value::Int(1), Value::Int(2)]);
        let data = PipelineData::from_value(Value::Array(arr));
        assert!(!data.is_empty());
    }
}
