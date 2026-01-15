//! Comprehensive string library tests (Plan 025)
//!
//! Tests all string operations including:
//! - Basic string operations
//! - Search operations (contains, starts_with, ends_with, find)
//! - Transform operations (trim, replace)
//! - Split/Join operations
//! - Compare operations
//! - Utility operations (repeat, char_at)
//! - C FFI operations

#[cfg(test)]
mod tests {
    use auto_val::{Args, Value};
    use crate::libs::string::*;

    // ==================== Basic Operations Tests ====================

    #[test]
    fn test_str_new() {
        // str_new with no args returns an error
        let args = Args { args: vec![] };
        let result = str_new(&args);
        match result {
            Value::Error(_) => (), // Expected error
            _ => panic!("Expected Error for str_new with no args"),
        }
    }

    #[test]
    fn test_str_new_with_value() {
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
        let s = auto_val::Str::from_str("hello");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_len(&args);
        match result {
            Value::Int(len) => assert_eq!(len, 5),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_str_upper() {
        let s = auto_val::Str::from_str("hello");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_upper(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "HELLO"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_lower() {
        let s = auto_val::Str::from_str("HELLO");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_lower(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_sub() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Int(0)),
                auto_val::Arg::Pos(Value::Int(5)),
            ],
        };
        let result = str_sub(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    // ==================== Search Operations Tests ====================

    #[test]
    fn test_str_contains_found() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("world".into())),
            ],
        };
        let result = str_contains(&args);
        match result {
            Value::Bool(found) => assert!(found),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_str_contains_not_found() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("goodbye".into())),
            ],
        };
        let result = str_contains(&args);
        match result {
            Value::Bool(found) => assert!(!found),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_str_starts_with_true() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("hello".into())),
            ],
        };
        let result = str_starts_with(&args);
        match result {
            Value::Bool(result) => assert!(result),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_str_starts_with_false() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("world".into())),
            ],
        };
        let result = str_starts_with(&args);
        match result {
            Value::Bool(result) => assert!(!result),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_str_ends_with_true() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("world".into())),
            ],
        };
        let result = str_ends_with(&args);
        match result {
            Value::Bool(result) => assert!(result),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_str_ends_with_false() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("hello".into())),
            ],
        };
        let result = str_ends_with(&args);
        match result {
            Value::Bool(result) => assert!(!result),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_str_find_found() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("world".into())),
            ],
        };
        let result = str_find(&args);
        match result {
            Value::Int(index) => assert_eq!(index, 6),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_str_find_not_found() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("goodbye".into())),
            ],
        };
        let result = str_find(&args);
        match result {
            Value::Int(index) => assert_eq!(index, -1),
            _ => panic!("Expected Int"),
        }
    }

    // ==================== Transform Operations Tests ====================

    #[test]
    fn test_str_trim() {
        let s = auto_val::Str::from_str("  hello world  ");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_trim(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello world"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_trim_left() {
        let s = auto_val::Str::from_str("  hello world");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_trim_left(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello world"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_trim_right() {
        let s = auto_val::Str::from_str("hello world  ");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_trim_right(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello world"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_replace() {
        let s = auto_val::Str::from_str("hello world");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("world".into())),
                auto_val::Arg::Pos(Value::Str("AutoLang".into())),
            ],
        };
        let result = str_replace(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello AutoLang"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_replace_multiple() {
        let s = auto_val::Str::from_str("hello world hello");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str("hello".into())),
                auto_val::Arg::Pos(Value::Str("hi".into())),
            ],
        };
        let result = str_replace(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hi world hi"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    // ==================== Split/Join Operations Tests ====================

    #[test]
    fn test_str_split() {
        let s = auto_val::Str::from_str("hello,world,auto");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Str(",".into())),
            ],
        };
        let result = str_split(&args);
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_str_join() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Array(vec![
                    Value::Str("hello".into()),
                    Value::Str("world".into()),
                    Value::Str("auto".into()),
                ].into())),
                auto_val::Arg::Pos(Value::Str(",".into())),
            ],
        };
        let result = str_join(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hello,world,auto"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    // ==================== Compare Operations Tests ====================

    #[test]
    fn test_str_compare_less() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Str("apple".into())),
                auto_val::Arg::Pos(Value::Str("banana".into())),
            ],
        };
        let result = str_compare(&args);
        match result {
            Value::Int(val) => assert_eq!(val, -1),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_str_compare_equal() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Str("hello".into())),
                auto_val::Arg::Pos(Value::Str("hello".into())),
            ],
        };
        let result = str_compare(&args);
        match result {
            Value::Int(val) => assert_eq!(val, 0),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_str_compare_greater() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Str("zebra".into())),
                auto_val::Arg::Pos(Value::Str("apple".into())),
            ],
        };
        let result = str_compare(&args);
        match result {
            Value::Int(val) => assert_eq!(val, 1),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_str_eq_ignore_case_true() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Str("HELLO".into())),
                auto_val::Arg::Pos(Value::Str("hello".into())),
            ],
        };
        let result = str_eq_ignore_case(&args);
        match result {
            Value::Bool(val) => assert!(val),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_str_eq_ignore_case_false() {
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::Str("hello".into())),
                auto_val::Arg::Pos(Value::Str("world".into())),
            ],
        };
        let result = str_eq_ignore_case(&args);
        match result {
            Value::Bool(val) => assert!(!val),
            _ => panic!("Expected Bool"),
        }
    }

    // ==================== Utility Operations Tests ====================

    #[test]
    fn test_str_repeat() {
        let s = auto_val::Str::from_str("ha");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Int(3)),
            ],
        };
        let result = str_repeat(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), "hahaha"),
            _ => panic!("Expected OwnedStr"),
        }
    }

    #[test]
    fn test_str_char_at() {
        let s = auto_val::Str::from_str("hello");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Int(1)),
            ],
        };
        let result = str_char_at(&args);
        match result {
            Value::Str(c) => assert_eq!(c.as_str(), "e"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_str_char_at_out_of_bounds() {
        let s = auto_val::Str::from_str("hello");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Int(10)),
            ],
        };
        let result = str_char_at(&args);
        match result {
            Value::Error(_) => (), // Expected error
            _ => panic!("Expected Error"),
        }
    }

    // ==================== C FFI Operations Tests ====================

    #[test]
    fn test_cstr_new() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("hello".into()))],
        };
        let result = cstr_new(&args);
        match result {
            Value::CStr(cs) => {
                assert_eq!(cs.as_str(), "hello");
                assert_eq!(cs.len(), 5);
            }
            _ => panic!("Expected CStr"),
        }
    }

    #[test]
    fn test_cstr_len() {
        let cs = auto_val::CStr::from_str("hello");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::CStr(cs))],
        };
        let result = cstr_len(&args);
        match result {
            Value::Int(len) => assert_eq!(len, 5),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_cstr_to_str() {
        let cs = auto_val::CStr::from_str("hello");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::CStr(cs))],
        };
        let result = cstr_to_str(&args);
        match result {
            Value::Str(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_to_cstr() {
        let s = auto_val::Str::from_str("hello");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = to_cstr(&args);
        match result {
            Value::CStr(cs) => assert_eq!(cs.as_str(), "hello"),
            _ => panic!("Expected CStr"),
        }
    }

    #[test]
    fn test_cstr_null_terminated() {
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::Str("test".into()))],
        };
        let result = cstr_new(&args);
        match result {
            Value::CStr(cs) => {
                // Verify null terminator is present
                let ptr = cs.as_ptr();
                assert!(!ptr.is_null());
                unsafe {
                    assert_eq!(*ptr.add(4) as u8, 0); // Null terminator at position 4
                }
            }
            _ => panic!("Expected CStr"),
        }
    }

    // ==================== Edge Cases Tests ====================

    #[test]
    fn test_empty_string() {
        let s = auto_val::Str::from_str("");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_len(&args);
        match result {
            Value::Int(len) => assert_eq!(len, 0),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_unicode() {
        let s = auto_val::Str::from_str("你好世界");
        let args = Args {
            args: vec![auto_val::Arg::Pos(Value::OwnedStr(s))],
        };
        let result = str_len(&args);
        match result {
            Value::Int(len) => assert_eq!(len, 12), // 4 chars * 3 bytes each
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_str_repeat_zero() {
        let s = auto_val::Str::from_str("ha");
        let args = Args {
            args: vec![
                auto_val::Arg::Pos(Value::OwnedStr(s)),
                auto_val::Arg::Pos(Value::Int(0)),
            ],
        };
        let result = str_repeat(&args);
        match result {
            Value::OwnedStr(s) => assert_eq!(s.as_str(), ""),
            _ => panic!("Expected OwnedStr"),
        }
    }
}
