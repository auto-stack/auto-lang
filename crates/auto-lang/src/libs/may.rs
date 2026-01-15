//! May<T> type functions for AutoLang
//!
//! This module provides the unified May<T> type, which combines the semantics
//! of Option<T> and Result<T, E> into a single three-state type:
//! - Empty (no value, like None)
//! - Value (has value, like Some/Ok)
//! - Error (has error, like Err)
//!
//! Syntax sugar: `?T` is equivalent to `May<T>` (e.g., `?int`, `?str`)

use auto_val::{Args, Arg, AutoStr, Value};

// ==================== Creation Functions ====================

/// Create an Empty May (no value)
///
/// # Arguments
/// * `args` - Empty arguments
///
/// # Returns
/// An Empty May value
pub fn may_empty(_args: &Args) -> Value {
    // TODO: Create May<T> variant when added to Value enum
    // For now, return Nil as a placeholder for Empty
    Value::Nil
}

/// Create a May with a value
///
/// # Arguments
/// * `args` - Arguments containing the value to wrap
///
/// # Returns
/// A May value wrapping the provided value
pub fn may_value(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("May_value requires a value".into());
    }

    // Return the value directly
    // TODO: Create May<T> variant when added to Value enum
    match &args.args[0] {
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

/// Create a May with an error
///
/// # Arguments
/// * `args` - Arguments containing the error message
///
/// # Returns
/// A May value wrapping the error message
pub fn may_error(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("May_error requires an error message".into());
    }

    let error = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.clone(),
        Arg::Pos(Value::OwnedStr(s)) => AutoStr::from(s.as_str()),
        Arg::Pos(Value::Error(s)) => s.clone(),
        Arg::Name(name) => name.clone(),
        _ => AutoStr::from("unknown error"),
    };

    // TODO: Create May<T> variant when added to Value enum
    // For now, return Error value
    Value::Error(error)
}

// ==================== Inspection Functions ====================

/// Check if a May is Empty
///
/// # Arguments
/// * `args` - Arguments containing the May to check
///
/// # Returns
/// Boolean true if the May is Empty
pub fn may_is_empty(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Bool(true);
    }

    // TODO: Check if May<T> variant when added to Value enum
    // For now, check if value is Nil (Empty)
    match &args.args[0] {
        Arg::Pos(Value::Nil) => Value::Bool(true),
        Arg::Pos(_) => Value::Bool(false),
        Arg::Pair(_, _) => Value::Bool(false),
        Arg::Name(_) => Value::Bool(false),
    }
}

/// Check if a May has a Value
///
/// # Arguments
/// * `args` - Arguments containing the May to check
///
/// # Returns
/// Boolean true if the May has a Value
pub fn may_is_value(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Bool(false);
    }

    // TODO: Check if May<T> variant when added to Value enum
    // For now, check if value is not Nil and not Error
    match &args.args[0] {
        Arg::Pos(Value::Nil) => Value::Bool(false),
        Arg::Pos(Value::Error(_)) => Value::Bool(false),
        Arg::Pos(_) => Value::Bool(true),
        Arg::Pair(_, _) => Value::Bool(true),
        Arg::Name(_) => Value::Bool(true),
    }
}

/// Check if a May has an Error
///
/// # Arguments
/// * `args` - Arguments containing the May to check
///
/// # Returns
/// Boolean true if the May has an Error
pub fn may_is_error(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Bool(false);
    }

    // TODO: Check if May<T> variant when added to Value enum
    // For now, check if value is Error
    match &args.args[0] {
        Arg::Pos(Value::Error(_)) => Value::Bool(true),
        Arg::Pos(_) => Value::Bool(false),
        Arg::Pair(_, _) => Value::Bool(false),
        Arg::Name(_) => Value::Bool(false),
    }
}

// ==================== Unwrapping Functions ====================

/// Unwrap a May, returning the contained value
///
/// # Arguments
/// * `args` - Arguments containing the May to unwrap
///
/// # Returns
/// The contained value, or an error if May is Empty or Error
pub fn may_unwrap(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("May_unwrap: no May provided".into());
    }

    // TODO: Unwrap May<T> variant when added to Value enum
    // For now, return the value if not Error and not Nil
    match &args.args[0] {
        Arg::Pos(Value::Nil) => {
            Value::Error("May_unwrap: called on Empty state".into())
        }
        Arg::Pos(Value::Error(e)) => {
            Value::Error(format!("May_unwrap: called on Error state: {}", e).into())
        }
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

/// Unwrap a May, returning the contained value or a default
///
/// # Arguments
/// * `args` - Arguments containing the May and default value
///
/// # Returns
/// The contained value if Value, otherwise the default value
pub fn may_unwrap_or(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("May_unwrap_or: no May provided".into());
    }

    if args.args.len() < 2 {
        return Value::Error("May_unwrap_or: no default value provided".into());
    }

    let may = match &args.args[0] {
        Arg::Pos(value) => value,
        Arg::Pair(_, value) => value,
        Arg::Name(name) => return Value::Str(name.clone()),
    };

    let default = match &args.args[1] {
        Arg::Pos(value) => value,
        Arg::Pair(_, value) => value,
        Arg::Name(name) => return Value::Str(name.clone()),
    };

    // TODO: Unwrap May<T> variant when added to Value enum
    // For now, return the value if not Nil/Error, otherwise default
    match may {
        Value::Nil | Value::Error(_) => default.clone(),
        _ => may.clone(),
    }
}

/// Unwrap a May, returning the contained value or NULL
///
/// # Arguments
/// * `args` - Arguments containing the May to unwrap
///
/// # Returns
/// The contained value if Value, otherwise NULL
pub fn may_unwrap_or_null(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Null;
    }

    // TODO: Unwrap May<T> variant when added to Value enum
    // For now, return the value if not Nil/Error, otherwise Null
    match &args.args[0] {
        Arg::Pos(Value::Nil) => Value::Null,
        Arg::Pos(Value::Error(_)) => Value::Null,
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

/// Unwrap a May, returning the contained error message
///
/// # Arguments
/// * `args` - Arguments containing the May to unwrap
///
/// # Returns
/// The error message, or an error if May is not Error
pub fn may_unwrap_error(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("May_unwrap_error: no May provided".into());
    }

    // TODO: Unwrap May<T> variant when added to Value enum
    // For now, return error message if Error variant
    match &args.args[0] {
        Arg::Pos(Value::Error(e)) => Value::Str(e.clone()),
        Arg::Pos(_) => Value::Error("May_unwrap_error: not in Error state".into()),
        Arg::Pair(_, _) => Value::Error("May_unwrap_error: not in Error state".into()),
        Arg::Name(_) => Value::Error("May_unwrap_error: not in Error state".into()),
    }
}

/// Unwrap a May, returning the error or a default error message
///
/// # Arguments
/// * `args` - Arguments containing the May and default error
///
/// # Returns
/// The error message if Error, otherwise the default error message
pub fn may_unwrap_error_or(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Str("no May provided".into());
    }

    let default_error = if args.args.len() >= 2 {
        match &args.args[1] {
            Arg::Pos(Value::Str(s)) => s.clone(),
            Arg::Pos(Value::OwnedStr(s)) => AutoStr::from(s.as_str()),
            Arg::Name(name) => name.clone(),
            _ => AutoStr::from("no error"),
        }
    } else {
        AutoStr::from("no error")
    };

    // TODO: Unwrap May<T> variant when added to Value enum
    // For now, return error message if Error variant
    match &args.args[0] {
        Arg::Pos(Value::Error(e)) => Value::Str(e.clone()),
        Arg::Pos(_) => Value::Str(default_error),
        Arg::Pair(_, _) => Value::Str(default_error),
        Arg::Name(_) => Value::Str(default_error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Arg;

    // ==================== Creation Tests ====================

    #[test]
    fn test_may_empty() {
        let args = Args { args: vec![] };
        let result = may_empty(&args);
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_may_value() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = may_value(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_may_error() {
        let args = Args {
            args: vec![Arg::Pos(Value::Str("test error".into()))],
        };
        let result = may_error(&args);
        match result {
            Value::Error(e) => assert_eq!(e.as_str(), "test error"),
            _ => panic!("Expected Error"),
        }
    }

    // ==================== Inspection Tests ====================

    #[test]
    fn test_may_is_empty_true() {
        let args = Args {
            args: vec![Arg::Pos(Value::Nil)],
        };
        let result = may_is_empty(&args);
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_may_is_empty_false() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = may_is_empty(&args);
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_may_is_value_true() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = may_is_value(&args);
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_may_is_value_false() {
        let args = Args {
            args: vec![Arg::Pos(Value::Nil)],
        };
        let result = may_is_value(&args);
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_may_is_error_true() {
        let args = Args {
            args: vec![Arg::Pos(Value::Error("test error".into()))],
        };
        let result = may_is_error(&args);
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_may_is_error_false() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = may_is_error(&args);
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool"),
        }
    }

    // ==================== Unwrap Tests ====================

    #[test]
    fn test_may_unwrap_value() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = may_unwrap(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_may_unwrap_or_value() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Int(42)),
                Arg::Pos(Value::Int(100)),
            ],
        };
        let result = may_unwrap_or(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_may_unwrap_or_empty() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Nil),
                Arg::Pos(Value::Int(100)),
            ],
        };
        let result = may_unwrap_or(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 100),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_may_unwrap_or_error() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Error("error".into())),
                Arg::Pos(Value::Int(100)),
            ],
        };
        let result = may_unwrap_or(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 100),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_may_unwrap_or_null() {
        let args = Args {
            args: vec![Arg::Pos(Value::Nil)],
        };
        let result = may_unwrap_or_null(&args);
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_may_unwrap_error() {
        let args = Args {
            args: vec![Arg::Pos(Value::Error("test error".into()))],
        };
        let result = may_unwrap_error(&args);
        match result {
            Value::Str(s) => assert_eq!(s.as_str(), "test error"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_may_unwrap_error_or_error() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Error("actual error".into())),
                Arg::Pos(Value::Str("default error".into())),
            ],
        };
        let result = may_unwrap_error_or(&args);
        match result {
            Value::Str(s) => assert_eq!(s.as_str(), "actual error"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_may_unwrap_error_or_value() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Int(42)),
                Arg::Pos(Value::Str("default error".into())),
            ],
        };
        let result = may_unwrap_error_or(&args);
        match result {
            Value::Str(s) => assert_eq!(s.as_str(), "default error"),
            _ => panic!("Expected Str"),
        }
    }
}
