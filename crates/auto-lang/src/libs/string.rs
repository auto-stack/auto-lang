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

// ============================================================================
// Plan 025: Additional String Operations
// ============================================================================

/// Check if a string contains a substring
///
/// # Arguments
/// * `args` - Expected: (s: str, pattern: str)
///
/// # Example
/// ```auto
/// let contains = str_contains("hello world", "world")  // returns true
/// ```
pub fn str_contains(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_contains requires 2 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_contains expects string as first argument".into()),
    };

    let pattern = match &args.args[1] {
        Arg::Pos(Value::Str(p)) => p.as_str(),
        Arg::Pos(Value::OwnedStr(p)) => p.as_str(),
        _ => return Value::Error("str_contains expects string as second argument".into()),
    };

    Value::Bool(s.contains(pattern))
}

/// Check if a string starts with a prefix
///
/// # Arguments
/// * `args` - Expected: (s: str, prefix: str)
///
/// # Example
/// ```auto
/// let starts = str_starts_with("hello", "he")  // returns true
/// ```
pub fn str_starts_with(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_starts_with requires 2 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_starts_with expects string as first argument".into()),
    };

    let prefix = match &args.args[1] {
        Arg::Pos(Value::Str(p)) => p.as_str(),
        Arg::Pos(Value::OwnedStr(p)) => p.as_str(),
        _ => return Value::Error("str_starts_with expects string as second argument".into()),
    };

    Value::Bool(s.starts_with(prefix))
}

/// Check if a string ends with a suffix
///
/// # Arguments
/// * `args` - Expected: (s: str, suffix: str)
///
/// # Example
/// ```auto
/// let ends = str_ends_with("hello", "lo")  // returns true
/// ```
pub fn str_ends_with(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_ends_with requires 2 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_ends_with expects string as first argument".into()),
    };

    let suffix = match &args.args[1] {
        Arg::Pos(Value::Str(p)) => p.as_str(),
        Arg::Pos(Value::OwnedStr(p)) => p.as_str(),
        _ => return Value::Error("str_ends_with expects string as second argument".into()),
    };

    Value::Bool(s.ends_with(suffix))
}

/// Find the index of a substring (returns -1 if not found)
///
/// # Arguments
/// * `args` - Expected: (s: str, pattern: str)
///
/// # Example
/// ```auto
/// let index = str_find("hello world", "world")  // returns 6
/// let not_found = str_find("hello", "xyz")     // returns -1
/// ```
pub fn str_find(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_find requires 2 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_find expects string as first argument".into()),
    };

    let pattern = match &args.args[1] {
        Arg::Pos(Value::Str(p)) => p.as_str(),
        Arg::Pos(Value::OwnedStr(p)) => p.as_str(),
        _ => return Value::Error("str_find expects string as second argument".into()),
    };

    match s.find(pattern) {
        Some(index) => Value::Int(index as i32),
        None => Value::Int(-1),
    }
}

/// Trim whitespace from both ends of a string
///
/// # Arguments
/// * `args` - Expected: (s: str)
///
/// # Example
/// ```auto
/// let trimmed = str_trim("  hello  ")  // returns "hello"
/// ```
pub fn str_trim(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_trim requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::Str(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().trim())),
        Arg::Pos(Value::OwnedStr(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().trim())),
        _ => Value::Error("str_trim expects a string argument".into()),
    }
}

/// Trim whitespace from the left side of a string
///
/// # Arguments
/// * `args` - Expected: (s: str)
///
/// # Example
/// ```auto
/// let trimmed = str_trim_left("  hello  ")  // returns "hello  "
/// ```
pub fn str_trim_left(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_trim_left requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::Str(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().trim_start())),
        Arg::Pos(Value::OwnedStr(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().trim_start())),
        _ => Value::Error("str_trim_left expects a string argument".into()),
    }
}

/// Trim whitespace from the right side of a string
///
/// # Arguments
/// * `args` - Expected: (s: str)
///
/// # Example
/// ```auto
/// let trimmed = str_trim_right("  hello  ")  // returns "  hello"
/// ```
pub fn str_trim_right(args: &Args) -> Value {
    if args.args.is_empty() {
        return Value::Error("str_trim_right requires 1 argument".into());
    }

    match &args.args[0] {
        Arg::Pos(Value::Str(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().trim_end())),
        Arg::Pos(Value::OwnedStr(s)) => Value::OwnedStr(auto_val::Str::from_str(s.as_str().trim_end())),
        _ => Value::Error("str_trim_right expects a string argument".into()),
    }
}

/// Replace all occurrences of a pattern in a string
///
/// # Arguments
/// * `args` - Expected: (s: str, from: str, to: str)
///
/// # Example
/// ```auto
/// let replaced = str_replace("hello world", "world", "rust")  // returns "hello rust"
/// ```
pub fn str_replace(args: &Args) -> Value {
    if args.args.len() < 3 {
        return Value::Error("str_replace requires 3 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str().to_string(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str().to_string(),
        _ => return Value::Error("str_replace expects string as first argument".into()),
    };

    let from = match &args.args[1] {
        Arg::Pos(Value::Str(p)) => p.as_str(),
        Arg::Pos(Value::OwnedStr(p)) => p.as_str(),
        _ => return Value::Error("str_replace expects string as second argument".into()),
    };

    let to = match &args.args[2] {
        Arg::Pos(Value::Str(p)) => p.as_str(),
        Arg::Pos(Value::OwnedStr(p)) => p.as_str(),
        _ => return Value::Error("str_replace expects string as third argument".into()),
    };

    let result = s.replace(from, to);
    Value::OwnedStr(auto_val::Str::from_str(result.as_str()))
}

/// Split a string by a delimiter
///
/// # Arguments
/// * `args` - Expected: (s: str, delimiter: str)
///
/// # Example
/// ```auto
/// let parts = str_split("a,b,c", ",")  // returns array ["a", "b", "c"]
/// ```
pub fn str_split(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_split requires 2 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str().to_string(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str().to_string(),
        _ => return Value::Error("str_split expects string as first argument".into()),
    };

    let delimiter = match &args.args[1] {
        Arg::Pos(Value::Str(p)) => p.as_str().to_string(),
        Arg::Pos(Value::OwnedStr(p)) => p.as_str().to_string(),
        _ => return Value::Error("str_split expects string as second argument".into()),
    };

    let parts: Vec<Value> = if delimiter.is_empty() {
        // Split by characters
        s.chars().map(|c| Value::Str(c.to_string().into())).collect()
    } else {
        s.split(&delimiter)
            .map(|part| Value::OwnedStr(auto_val::Str::from_str(part)))
            .collect()
    };

    Value::Array(parts.into())
}

/// Join an array of strings with a delimiter
///
/// # Arguments
/// * `args` - Expected: (parts: array[str], delimiter: str)
///
/// # Example
/// ```auto
/// let parts = ["a", "b", "c"]
/// let joined = str_join(parts, ",")  // returns "a,b,c"
/// ```
pub fn str_join(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_join requires 2 arguments".into());
    }

    let parts = match &args.args[0] {
        Arg::Pos(Value::Array(arr)) => arr,
        _ => return Value::Error("str_join expects array as first argument".into()),
    };

    let delimiter = match &args.args[1] {
        Arg::Pos(Value::Str(d)) => d.as_str(),
        Arg::Pos(Value::OwnedStr(d)) => d.as_str(),
        _ => return Value::Error("str_join expects string as second argument".into()),
    };

    let strings: Result<Vec<String>, String> = parts
        .iter()
        .map(|v| match v {
            Value::Str(s) => Ok(s.as_str().to_string()),
            Value::OwnedStr(s) => Ok(s.as_str().to_string()),
            _ => Err("str_join: array must contain only strings".to_string()),
        })
        .collect();

    match strings {
        Ok(strs) => {
            let result = strs.join(delimiter);
            Value::OwnedStr(auto_val::Str::from_str(result.as_str()))
        }
        Err(e) => Value::Error(e.into()),
    }
}

/// Compare two strings (returns 0 if equal, < 0 if s1 < s2, > 0 if s1 > s2)
///
/// # Arguments
/// * `args` - Expected: (s1: str, s2: str)
///
/// # Example
/// ```auto
/// let cmp = str_compare("apple", "banana")  // returns negative number
/// let eq = str_compare("hello", "hello")    // returns 0
/// ```
pub fn str_compare(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_compare requires 2 arguments".into());
    }

    let s1 = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_compare expects string as first argument".into()),
    };

    let s2 = match &args.args[1] {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_compare expects string as second argument".into()),
    };

    match s1.cmp(s2) {
        std::cmp::Ordering::Less => Value::Int(-1),
        std::cmp::Ordering::Equal => Value::Int(0),
        std::cmp::Ordering::Greater => Value::Int(1),
    }
}

/// Check if two strings are equal (case-insensitive)
///
/// # Arguments
/// * `args` - Expected: (s1: str, s2: str)
///
/// # Example
/// ```auto
/// let eq = str_eq_ignore_case("HELLO", "hello")  // returns true
/// ```
pub fn str_eq_ignore_case(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_eq_ignore_case requires 2 arguments".into());
    }

    let s1 = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str().to_lowercase(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str().to_lowercase(),
        _ => return Value::Error("str_eq_ignore_case expects string as first argument".into()),
    };

    let s2 = match &args.args[1] {
        Arg::Pos(Value::Str(s)) => s.as_str().to_lowercase(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str().to_lowercase(),
        _ => return Value::Error("str_eq_ignore_case expects string as second argument".into()),
    };

    Value::Bool(s1 == s2)
}

/// Repeat a string n times
///
/// # Arguments
/// * `args` - Expected: (s: str, n: int)
///
/// # Example
/// ```auto
/// let repeated = str_repeat("ha", 3)  // returns "hahaha"
/// ```
pub fn str_repeat(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_repeat requires 2 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str().to_string(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str().to_string(),
        _ => return Value::Error("str_repeat expects string as first argument".into()),
    };

    let n = match &args.args[1] {
        Arg::Pos(Value::Int(n)) => *n,
        _ => return Value::Error("str_repeat expects int as second argument".into()),
    };

    if n < 0 {
        return Value::Error("str_repeat: count cannot be negative".into());
    }

    let result = s.repeat(n as usize);
    Value::OwnedStr(auto_val::Str::from_str(result.as_str()))
}

/// Get the character at a specific index
///
/// # Arguments
/// * `args` - Expected: (s: str, index: int)
///
/// # Example
/// ```auto
/// let ch = str_char_at("hello", 1)  // returns "e"
/// ```
pub fn str_char_at(args: &Args) -> Value {
    if args.args.len() < 2 {
        return Value::Error("str_char_at requires 2 arguments".into());
    }

    let s = match &args.args[0] {
        Arg::Pos(Value::Str(s)) => s.as_str(),
        Arg::Pos(Value::OwnedStr(s)) => s.as_str(),
        _ => return Value::Error("str_char_at expects string as first argument".into()),
    };

    let index = match &args.args[1] {
        Arg::Pos(Value::Int(i)) => *i,
        _ => return Value::Error("str_char_at expects int as second argument".into()),
    };

    if index < 0 {
        return Value::Error("str_char_at: index cannot be negative".into());
    }

    let chars: Vec<char> = s.chars().collect();
    if index as usize >= chars.len() {
        return Value::Error("str_char_at: index out of bounds".into());
    }

    Value::Str(chars[index as usize].to_string().into())
}
