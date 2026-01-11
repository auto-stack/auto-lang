use crate::Value;

/// Trait for converting Rust types to AutoLang Value
///
/// This trait is similar to ToString, but for AutoLang values.
/// It's used by macros to support external variable interpolation.
pub trait ToAutoValue {
    fn to_auto_value(&self) -> Value;
}

// Implementations for basic types
impl ToAutoValue for i32 {
    fn to_auto_value(&self) -> Value {
        Value::Int(*self)
    }
}

impl ToAutoValue for u32 {
    fn to_auto_value(&self) -> Value {
        Value::Uint(*self)
    }
}

impl ToAutoValue for i64 {
    fn to_auto_value(&self) -> Value {
        Value::Int(*self as i32)
    }
}

impl ToAutoValue for u64 {
    fn to_auto_value(&self) -> Value {
        Value::Uint(*self as u32)
    }
}

impl ToAutoValue for f64 {
    fn to_auto_value(&self) -> Value {
        Value::Double(*self)
    }
}

impl ToAutoValue for f32 {
    fn to_auto_value(&self) -> Value {
        Value::Float((*self) as f64)
    }
}

impl ToAutoValue for bool {
    fn to_auto_value(&self) -> Value {
        Value::Bool(*self)
    }
}

impl ToAutoValue for &str {
    fn to_auto_value(&self) -> Value {
        Value::Str((*self).into())
    }
}

impl ToAutoValue for String {
    fn to_auto_value(&self) -> Value {
        Value::Str(self.clone().into())
    }
}

// Implement for Value itself (identity)
impl ToAutoValue for Value {
    fn to_auto_value(&self) -> Value {
        self.clone()
    }
}

// Generic implementation for references to types that implement Clone and ToAutoValue
impl<T: ToAutoValue + Clone> ToAutoValue for &T where T: Sized {
    fn to_auto_value(&self) -> Value {
        (*self).to_auto_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_to_value() {
        let val: Value = 42.to_auto_value();
        assert_eq!(val, Value::Int(42));
    }

    #[test]
    fn test_str_to_value() {
        let val: Value = "hello".to_auto_value();
        assert!(matches!(val, Value::Str(_)));
    }

    #[test]
    fn test_bool_to_value() {
        let val: Value = true.to_auto_value();
        assert_eq!(val, Value::Bool(true));
    }
}
