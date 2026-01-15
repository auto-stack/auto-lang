//! String operation functions for AutoLang
//!
//! Provides built-in functions for creating and manipulating owned strings.

use auto_val::{Arg, Args, Value};
use auto_val::StrSlice;

/// Create a new owned string from a string literal and capacity hint
///
/// # Arguments
/// * `args` - Expected: (text: str, capacity: int)
///
/// # Example
/// ```auto
/// let s = str_new("hello", 10)
/// ```
pub fn str_new(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_new requires at least 1 argument".into());
    }

    let text_arg = &args.args[0];

    let text_str = match text_arg {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_new expects a string argument".into()),
    };

    // Create OwnedStr from the string
    Value::OwnedStr(auto_val::Str::from_str(text_str))
}

/// Get the length of a string
///
/// # Arguments
/// * `args` - Expected: (s: str)
///
/// # Example
/// ```auto
/// let len = str_len("hello")  // returns 5
/// ```
pub fn str_len(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_len requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::Str(s)) => Value::Int(s.len() as i32),
        Arg::Pos(Value::OwnedStr(s)) => Value::Int(s.len() as i32),
        _ => Value::Error("str_len expects a string argument".into()),
    }
}

/// Append one string to another
///
/// # Arguments
/// * `args` - Expected: (s: str, other: str)
///
/// # Example
/// ```auto
/// let s = str_new("hello", 10)
/// let result = str_append(s, " world")  // returns "hello world"
/// ```
pub fn str_append(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_append requires 2 arguments".into());
    }

    let base_arg = &args.args[0];
    let other_arg = &args.args[1];

    let base_str = match base_arg {
        Arg::Pos(Value::Str(s)) => s.as_str().to_string(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str().to_string(),
        _ => return Value::Error("str_append expects string arguments".into()),
    };

    let other_str = match other_arg {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_append expects string arguments".into()),
    };

    let combined = format!("{}{}", base_str, other_str);
    Value::OwnedStr(auto_val::Str::from_str(combined.as_str()))
}

/// Convert string to uppercase
///
/// # Arguments
/// * `args` - Expected: (s: str)
///
/// # Example
/// ```auto
/// let upper = str_upper("hello")  // returns "HELLO"
/// ```
pub fn str_upper(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_upper requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::Str(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().to_uppercase().as_str())),
        Arg::Pos(Value::OwnedStr(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().to_uppercase().as_str())),
        _ => Value::Error("str_upper expects a string argument".into()),
    }
}

/// Convert string to lowercase
///
/// # Arguments
/// * `args` - Expected: (s: str)
///
/// # Example
/// ```auto
/// let lower = str_lower("HELLO")  // returns "hello"
/// ```
pub fn str_lower(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_lower requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::Str(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().to_lowercase().as_str())),
        Arg::Pos(Value::OwnedStr(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().to_lowercase().as_str())),
        _ => Value::Error("str_lower expects a string argument".into()),
    }
}

/// Get a substring
///
/// # Arguments
/// * `args` - Expected: (s: str, start: int, end: int)
///
/// # Example
/// ```auto
/// let sub = str_sub("hello", 1, 4)  // returns "ell"
/// ```
pub fn str_sub(args: &Args) -> Value {
    if args.args.len() < 3 {
        return Value::Error("str_sub requires 3 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str().to_string(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str().to_string(),
        _ => return Value::Error("str_sub expects a string as first argument".into()),
    };

    let start = match &args.args[1] {
        Arg::Pos(Value::Int(n)) => *n as usize,
        _ => return Value::Error("str_sub expects int for start".into()),
    };

    let end = match &args.args[2] {
        Arg::Pos(Value::Int(n)) => *n as usize,
        _ => return Value::Error("str_sub expects int for end".into()),
    };

    if start > end || end > s.len() {
        return Value::Error("str_sub: invalid range".into());
    }

    let substring = &s[start..end];
    Value::OwnedStr(auto_val::Str::from_str(substring))
}

/// Create a string slice from a string (Phase 3 - EXPERIMENTAL)
///
/// # Safety
///
/// **WARNING**: This is an unsafe experimental function for Phase 3!
/// The borrow checker should prevent use-after-free, but it's not yet implemented.
/// Use at your own risk!
///
/// # Arguments
/// * `args` - Expected: (s: str)
///
/// # Example
/// ```auto
/// let s = str_new("hello", 10)
/// let slice = str_slice(s)  // Creates a borrow
/// ```
///
/// # Lifetime Safety
///
/// The returned `StrSlice` must not outlive the source string.
/// Once Phase 3 borrow checker is complete, this will be enforced at compile time.
pub fn str_slice(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_slice requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::Str(s)) => unsafe {
            // Create a borrowed slice - UNSAFE without borrow checker!
            Value::StrSlice(StrSlice::from_auto_str(s))
        },
        Arg::Pos(Value::OwnedStr(s)) => unsafe {
            // Create a borrowed slice from owned string - UNSAFE!
            // The slice must not outlive the OwnedStr
            Value::StrSlice(StrSlice::from_str(s.as_str()))
        },
        _ => Value::Error("str_slice expects a string argument".into()),
    }
}

/// Get the length of a string slice
///
/// # Arguments
/// * `args` - Expected: (slice: str_slice)
///
/// # Example
/// ```auto
/// let s = str_new("hello", 10)
/// let slice = str_slice(s)
/// let len = str_slice_len(slice)  // returns 5
/// ```
pub fn str_slice_len(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_slice_len requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::StrSlice(slice)) => Value::Int(slice.len() as i32),
        // Also support regular strings for convenience
        Arg::Pos(Value::Str(s)) => Value::Int(s.len() as i32),
        Arg::Pos(Value::OwnedStr(s)) => Value::Int(s.len() as i32),
        _ => Value::Error("str_slice_len expects a str_slice or string argument".into()),
    }
}

/// Get a byte from a string slice by index
///
/// # Arguments
/// * `args` - Expected: (slice: str_slice, index: int)
///
/// # Example
/// ```auto
/// let s = str_new("hello", 10)
/// let slice = str_slice(s)
/// let byte = str_slice_get(slice, 0)  // returns 104 (ASCII 'h')
/// ```
pub fn str_slice_get(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_slice_get requires 2 arguments".into());
    }

    let slice = match &args.args[0] {
        Arg::Pos(Value::StrSlice(s)) => s,
        Arg::Pos(Value::Str(s)) => unsafe {
            // Support regular strings by creating temporary slice
            &StrSlice::from_auto_str(s)
        },
        Arg::Pos(Value::OwnedStr(s)) => unsafe {
            &StrSlice::from_str(s.as_str())
        },
        _ => return Value::Error("str_slice_get expects a str_slice or string as first argument".into()),
    };

    let index = match &args.args[1] {
        Arg::Pos(Value::Int(n)) => *n as usize,
        _ => return Value::Error("str_slice_get expects int for index".into()),
    };

    match slice.get_byte(index) {
        Some(byte) => Value::Int(byte as i32),
        None => Value::Error("str_slice_get: index out of bounds".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_new() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("hello".into()))],
        };
        let result = str_new(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_len() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("hello".into()))],
        };
        let result = str_len(&args);
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_str_append() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Str("hello".into())),
                auto_val::Arg::Pos(Value::Str(" world".into())),
            ],
        };
        let result = str_append(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello world"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_upper() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("hello".into()))],
        };
        let result = str_upper(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "HELLO"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_lower() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("HELLO".into()))],
        };
        let result = str_lower(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_sub() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Str("hello".into())),
                auto_val::Arg::Pos(Value::Int(1)),
                auto_val::Arg::Pos(Value::Int(4)),
            ],
        };
        let result = str_sub(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "ell"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_slice() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("hello".into()))],
        };
        let result = str_slice(&args);
        match result {
            Value::StrSlice(slice) => {
                assert_eq!(slice.len(), 5);
                assert!(!slice.is_empty());
            },
            _ => panic!("Expected StrSlice"),
        }
    }

    #[test]
    fn test_str_slice_len() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("hello".into()))],
        };
        let slice = str_slice(&args);
        let len_args = Args {
            args: vec![auto_val::Arg::Pos(slice)],
        };
        let result = str_slice_len(&len_args);
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_str_slice_get() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("hello".into()))],
        };
        let slice = str_slice(&args);
        let get_args = Args {
            args: vec![
                auto_val::Arg::Pos(slice),
                auto_val::Arg::Pos(Value::Int(0)),
            ],
        };
        let result = str_slice_get(&get_args);
        // 'h' is ASCII 104
        assert_eq!(result, Value::Int(104));
    }
}
