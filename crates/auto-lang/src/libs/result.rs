//! Option and Result type functions for AutoLang
//!
//! This module provides functions for working with Option and Result types,
//! which are used for optional values and error handling.

use auto_val::{Args, Arg, AutoStr, Value};

// ==================== Option Functions ====================

/// Create an Option with Some value
///
/// # Arguments
/// * `args` - Arguments containing the value to wrap
///
/// # Returns
/// An Option value wrapping the provided value
pub fn option_some(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Option_some requires a value".into());
    }

    // Return the value directly
    // TODO: Create Option<T> variant when added to Value enum
    match &args.args[0] {
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

/// Create an Option with None value
///
/// # Arguments
/// * `args` - Empty arguments
///
/// # Returns
/// A None Option value
pub fn option_none(_args: &Args) -> Value {
    // TODO: Create Option<T> variant when added to Value enum
    // For now, return Nil as a placeholder
    Value::Nil
}

/// Check if an Option is Some
///
/// # Arguments
/// * `args` - Arguments containing the Option to check
///
/// # Returns
/// Boolean true if the Option is Some
pub fn option_is_some(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Bool(false);
    }

    // TODO: Check if Option<T> variant when added to Value enum
    // For now, check if value is not Nil
    match &args.args[0] {
        Arg::Pos(Value::Nil) => Value::Bool(false),
        Arg::Pos(_) => Value::Bool(true),
        Arg::Pair(_, _) => Value::Bool(true),
        Arg::Name(_) => Value::Bool(true),
    }
}

/// Check if an Option is None
///
/// # Arguments
/// * `args` - Arguments containing the Option to check
///
/// # Returns
/// Boolean true if the Option is None
pub fn option_is_none(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Bool(true);
    }

    // TODO: Check if Option<T> variant when added to Value enum
    // For now, check if value is Nil
    match &args.args[0] {
        Arg::Pos(Value::Nil) => Value::Bool(true),
        Arg::Pos(_) => Value::Bool(false),
        Arg::Pair(_, _) => Value::Bool(false),
        Arg::Name(_) => Value::Bool(false),
    }
}

/// Unwrap an Option, returning the contained value
///
/// # Arguments
/// * `args` - Arguments containing the Option to unwrap
///
/// # Returns
/// The contained value, or an error if Option is None
pub fn option_unwrap(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Option_unwrap: no Option provided".into());
    }

    // TODO: Unwrap Option<T> variant when added to Value enum
    // For now, return the value directly
    match &args.args[0] {
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

/// Unwrap an Option, returning the contained value or a default
///
/// # Arguments
/// * `args` - Arguments containing the Option and default value
///
/// # Returns
/// The contained value if Some, otherwise the default value
pub fn option_unwrap_or(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Option_unwrap_or: no Option provided".into());
    }

    if args.args.len() < 2 {
        return Value::Error("Option_unwrap_or: no default value provided".into());
    }

    let opt = match &args.args[0] {
        Arg::Pos(value) => value,
        Arg::Pair(_, value) => value,
        Arg::Name(name) => return Value::Str(name.clone()),
    };

    let default = match &args.args[1] {
        Arg::Pos(value) => value,
        Arg::Pair(_, value) => value,
        Arg::Name(name) => return Value::Str(name.clone()),
    };

    // TODO: Unwrap Option<T> variant when added to Value enum
    // For now, return the value if not Nil, otherwise default
    match opt {
        Value::Nil => default.clone(),
        _ => opt.clone(),
    }
}

/// Unwrap an Option, returning the contained value or NULL
///
/// # Arguments
/// * `args` - Arguments containing the Option to unwrap
///
/// # Returns
/// The contained value if Some, otherwise NULL
pub fn option_unwrap_or_null(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Null;
    }

    // TODO: Unwrap Option<T> variant when added to Value enum
    // For now, return the value if not Nil, otherwise Null
    match &args.args[0] {
        Arg::Pos(Value::Nil) => Value::Null,
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

// ==================== Result Functions ====================

/// Create a Result with Ok value
///
/// # Arguments
/// * `args` - Arguments containing the value to wrap
///
/// # Returns
/// A Result value wrapping the provided value
pub fn result_ok(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Result_ok requires a value".into());
    }

    // TODO: Create Result<T, E> variant when added to Value enum
    // For now, return the value directly
    match &args.args[0] {
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

/// Create a Result with Err value
///
/// # Arguments
/// * `args` - Arguments containing the error message
///
/// # Returns
/// A Result value wrapping the error message
pub fn result_err(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Result_err requires an error message".into());
    }

    let error = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.clone(),
        Arg::Pos(Value::OwnedStr(s)) => AutoStr::from(s.as_str()),
        Arg::Pos(Value::Error(s)) => s.clone(),
        Arg::Name(name) => name.clone(),
        _ => AutoStr::from("unknown error"),
    };

    // TODO: Create Result<T, E> variant when added to Value enum
    // For now, return Error value
    Value::Error(error)
}

/// Check if a Result is Ok
///
/// # Arguments
/// * `args` - Arguments containing the Result to check
///
/// # Returns
/// Boolean true if the Result is Ok
pub fn result_is_ok(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Bool(false);
    }

    // TODO: Check if Result<T, E> variant when added to Value enum
    // For now, check if value is not Error
    match &args.args[0] {
        Arg::Pos(Value::Error(_)) => Value::Bool(false),
        Arg::Pos(_) => Value::Bool(true),
        Arg::Pair(_, _) => Value::Bool(true),
        Arg::Name(_) => Value::Bool(true),
    }
}

/// Check if a Result is Err
///
/// # Arguments
/// * `args` - Arguments containing the Result to check
///
/// # Returns
/// Boolean true if the Result is Err
pub fn result_is_err(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Bool(false);
    }

    // TODO: Check if Result<T, E> variant when added to Value enum
    // For now, check if value is Error
    match &args.args[0] {
        Arg::Pos(Value::Error(_)) => Value::Bool(true),
        Arg::Pos(_) => Value::Bool(false),
        Arg::Pair(_, _) => Value::Bool(false),
        Arg::Name(_) => Value::Bool(false),
    }
}

/// Unwrap a Result, returning the contained value
///
/// # Arguments
/// * `args` - Arguments containing the Result to unwrap
///
/// # Returns
/// The contained value, or an error if Result is Err
pub fn result_unwrap(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Result_unwrap: no Result provided".into());
    }

    // TODO: Unwrap Result<T, E> variant when added to Value enum
    // For now, return the value if not Error
    match &args.args[0] {
        Arg::Pos(Value::Error(e)) => Value::Error(format!("Result_unwrap: called on Err: {}", e).into()),
        Arg::Pos(value) => value.clone(),
        Arg::Pair(_, value) => value.clone(),
        Arg::Name(name) => Value::Str(name.clone()),
    }
}

/// Unwrap a Result, returning the contained error message
///
/// # Arguments
/// * `args` - Arguments containing the Result to unwrap
///
/// # Returns
/// The error message, or an error if Result is Ok
pub fn result_unwrap_err(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Result_unwrap_err: no Result provided".into());
    }

    // TODO: Unwrap Result<T, E> variant when added to Value enum
    // For now, return error message if Error variant
    match &args.args[0] {
        Arg::Pos(Value::Error(e)) => Value::Str(e.clone()),
        Arg::Pos(_) => Value::Error("Result_unwrap_err: called on Ok".into()),
        Arg::Pair(_, _) => Value::Error("Result_unwrap_err: called on Ok".into()),
        Arg::Name(_) => Value::Error("Result_unwrap_err: called on Ok".into()),
    }
}

/// Unwrap a Result, returning the contained value or a default
///
/// # Arguments
/// * `args` - Arguments containing the Result and default value
///
/// # Returns
/// The contained value if Ok, otherwise the default value
pub fn result_unwrap_or(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("Result_unwrap_or: no Result provided".into());
    }

    if args.args.len() < 2 {
        return Value::Error("Result_unwrap_or: no default value provided".into());
    }

    let res = match &args.args[0] {
        Arg::Pos(value) => value,
        Arg::Pair(_, value) => value,
        Arg::Name(name) => return Value::Str(name.clone()),
    };

    let default = match &args.args[1] {
        Arg::Pos(value) => value,
        Arg::Pair(_, value) => value,
        Arg::Name(name) => return Value::Str(name.clone()),
    };

    // TODO: Unwrap Result<T, E> variant when added to Value enum
    // For now, return the value if not Error, otherwise default
    match res {
        Value::Error(_) => default.clone(),
        _ => res.clone(),
    }
}

/// Unwrap a Result, returning the error or a default error message
///
/// # Arguments
/// * `args` - Arguments containing the Result and default error
///
/// # Returns
/// The error message if Err, otherwise the default error message
pub fn result_unwrap_err_or(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Str("no Result provided".into());
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

    // TODO: Unwrap Result<T, E> variant when added to Value enum
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

    // ==================== Option Tests ====================

    #[test]
    fn test_option_some() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = option_some(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_option_none() {
        let args = Args { args: vec![] };
        let result = option_none(&args);
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_option_is_some() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = option_is_some(&args);
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_option_is_none() {
        let args = Args {
            args: vec![Arg::Pos(Value::Nil)],
        };
        let result = option_is_none(&args);
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_option_unwrap_or_some() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Int(42)),
                Arg::Pos(Value::Int(100)),
            ],
        };
        let result = option_unwrap_or(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_option_unwrap_or_none() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Nil),
                Arg::Pos(Value::Int(100)),
            ],
        };
        let result = option_unwrap_or(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 100),
            _ => panic!("Expected Int"),
        }
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_ok() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = result_ok(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_result_err() {
        let args = Args {
            args: vec![Arg::Pos(Value::Str("test error".into()))],
        };
        let result = result_err(&args);
        match result {
            Value::Error(e) => assert_eq!(e.as_str(), "test error"),
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_result_is_ok() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = result_is_ok(&args);
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_result_is_err() {
        let args = Args {
            args: vec![Arg::Pos(Value::Error("test error".into()))],
        };
        let result = result_is_err(&args);
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_result_unwrap_ok() {
        let args = Args {
            args: vec![Arg::Pos(Value::Int(42))],
        };
        let result = result_unwrap(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_result_unwrap_err() {
        let args = Args {
            args: vec![Arg::Pos(Value::Error("test error".into()))],
        };
        let result = result_unwrap_err(&args);
        match result {
            Value::Str(s) => assert_eq!(s.as_str(), "test error"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_result_unwrap_or_ok() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Int(42)),
                Arg::Pos(Value::Int(100)),
            ],
        };
        let result = result_unwrap_or(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_result_unwrap_or_err() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Error("test error".into())),
                Arg::Pos(Value::Int(100)),
            ],
        };
        let result = result_unwrap_or(&args);
        match result {
            Value::Int(n) => assert_eq!(n, 100),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_result_unwrap_err_or_err() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Error("actual error".into())),
                Arg::Pos(Value::Str("default error".into())),
            ],
        };
        let result = result_unwrap_err_or(&args);
        match result {
            Value::Str(s) => assert_eq!(s.as_str(), "actual error"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_result_unwrap_err_or_ok() {
        let args = Args {
            args: vec![
                Arg::Pos(Value::Int(42)),
                Arg::Pos(Value::Str("default error".into())),
            ],
        };
        let result = result_unwrap_err_or(&args);
        match result {
            Value::Str(s) => assert_eq!(s.as_str(), "default error"),
            _ => panic!("Expected Str"),
        }
    }
}
