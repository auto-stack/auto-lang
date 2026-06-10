//! Shell value types
//!
//! Provides structured data types for shell commands that integrate with AutoLang's Value.

/// Shell value types
#[derive(Debug, Clone, PartialEq)]
pub enum ShellValue {
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// Array of values
    Array(Vec<ShellValue>),
    /// Object with key-value pairs
    Object(indexmap::IndexMap<String, ShellValue>),
    /// Null value
    Null,
}

impl ShellValue {
    /// Create a null value
    pub fn null() -> Self {
        ShellValue::Null
    }

    /// Create an integer value
    pub fn int(value: i64) -> Self {
        ShellValue::Int(value)
    }

    /// Create a float value
    pub fn float(value: f64) -> Self {
        ShellValue::Float(value)
    }

    /// Create a string value
    pub fn string(value: impl Into<String>) -> Self {
        ShellValue::String(value.into())
    }

    /// Create a boolean value
    pub fn bool(value: bool) -> Self {
        ShellValue::Bool(value)
    }
}

impl std::fmt::Display for ShellValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellValue::Int(n) => write!(f, "{}", n),
            ShellValue::Float(n) => write!(f, "{}", n),
            ShellValue::String(s) => write!(f, "{}", s),
            ShellValue::Bool(b) => write!(f, "{}", b),
            ShellValue::Null => write!(f, "null"),
            ShellValue::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            ShellValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let _ = ShellValue::int(42);
        let _ = ShellValue::float(3.14);
        let _ = ShellValue::string("hello");
        let _ = ShellValue::bool(true);
        let _ = ShellValue::null();
    }

    #[test]
    fn test_value_display() {
        assert_eq!(ShellValue::int(42).to_string(), "42");
        assert_eq!(ShellValue::float(3.14).to_string(), "3.14");
        assert_eq!(ShellValue::string("hello").to_string(), "hello");
        assert_eq!(ShellValue::bool(true).to_string(), "true");
        assert_eq!(ShellValue::null().to_string(), "null");
    }

    #[test]
    fn test_array_display() {
        let arr = ShellValue::Array(vec![
            ShellValue::int(1),
            ShellValue::int(2),
            ShellValue::int(3),
        ]);
        assert_eq!(arr.to_string(), "[1, 2, 3]");
    }
}
