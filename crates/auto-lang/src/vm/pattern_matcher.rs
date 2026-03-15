//! Plan 125: Phase 3.5 - Pattern Matcher for Task Message Routing
//!
//! This module implements pattern matching for task `on` blocks.
//!
//! ## Overview
//!
//! When a message arrives at a task, the pattern matcher determines which
//! handler to invoke based on the message pattern:
//!
//! ```auto
//! on(ctx) {
//!     "ping" => { ctx.reply("pong") }              // Literal match
//!     msg string => { handle(msg) }                 // Type binding
//!     amount int if amount > 10000 => { ... }      // Type + guard
//!     Reset => { reset() }                         // Simple variant
//!     Add(val) => { count += val }                 // Variant with binding
//! }
//! ```

use crate::ast::{LiteralValue, TaskMsgPattern};
use auto_val::{AutoStr, Value};
use std::fmt;

/// Result of pattern matching
#[derive(Debug)]
pub struct MatchResult {
    /// The matched bindings (variable name -> value)
    pub bindings: Vec<(String, Value)>,
}

impl MatchResult {
    /// Create a new match result with bindings
    pub fn new(bindings: Vec<(String, Value)>) -> Self {
        Self { bindings }
    }

    /// Create an empty match result (for patterns without bindings)
    pub fn empty() -> Self {
        Self { bindings: Vec::new() }
    }

    /// Check if there are any bindings
    pub fn has_bindings(&self) -> bool {
        !self.bindings.is_empty()
    }
}

/// Pattern matcher for task message routing
pub struct PatternMatcher;

impl PatternMatcher {
    /// Try to match a message against a pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern to match against
    /// * `message` - The message value
    ///
    /// # Returns
    ///
    /// * `Some(MatchResult)` - Pattern matched, with bindings
    /// * `None` - Pattern did not match
    pub fn match_pattern(pattern: &TaskMsgPattern, message: &Value) -> Option<MatchResult> {
        match pattern {
            TaskMsgPattern::Literal(lit) => Self::match_literal(lit, message),
            TaskMsgPattern::TypeBinding { name, type_expr } => {
                Self::match_type_binding(name.as_str(), type_expr, message)
            }
            TaskMsgPattern::Simple(variant_name) => {
                // For simple variants, check if the message is an object with
                // a __variant field matching the variant name
                Self::match_simple_variant(variant_name.as_str(), message)
            }
            TaskMsgPattern::WithBindings { variant, bindings } => {
                Self::match_variant_with_bindings(variant.as_str(), bindings, message)
            }
        }
    }

    /// Match a literal pattern
    fn match_literal(lit: &LiteralValue, message: &Value) -> Option<MatchResult> {
        let matches = match (lit, message) {
            (LiteralValue::String(s), Value::Str(v)) => s.as_str() == v.as_str(),
            (LiteralValue::Int(n), Value::Int(v)) => *n == *v as i64,
            (LiteralValue::Int(n), Value::I64(v)) => *n == *v,
            (LiteralValue::Uint(n), Value::Uint(v)) => *n == *v as u64,
            (LiteralValue::Bool(b), Value::Bool(v)) => *b == *v,
            (LiteralValue::Char(c), Value::Char(v)) => *c == *v,
            _ => false,
        };

        if matches {
            Some(MatchResult::empty())
        } else {
            None
        }
    }

    /// Match a type binding pattern
    fn match_type_binding(name: &str, type_expr: &crate::ast::Type, message: &Value) -> Option<MatchResult> {
        use crate::ast::Type;

        // Check if the message matches the expected type
        let type_matches = match (type_expr, message) {
            // String types
            (Type::Str(_), Value::Str(_)) => true,
            (Type::StrSlice, Value::Str(_)) => true,
            (Type::CStr, Value::Str(_)) => true,

            // Integer types (Value has Int(i32) and I64(i64))
            (Type::Int, Value::Int(_)) => true,
            (Type::Int, Value::I64(_)) => true,
            (Type::I64, Value::I64(_)) => true,
            (Type::I64, Value::Int(_)) => true,

            // Unsigned integer types (Value has Uint(u32), no U64)
            (Type::Uint, Value::Uint(_)) => true,
            (Type::U64, Value::Uint(_)) => true,  // U64 type matches Uint value
            (Type::Byte, Value::Byte(_)) => true,
            (Type::Byte, Value::U8(_)) => true,

            // Float types
            (Type::Float, Value::Float(_)) => true,
            (Type::Double, Value::Double(_)) => true,
            (Type::Double, Value::Float(_)) => true,

            // Bool type
            (Type::Bool, Value::Bool(_)) => true,

            // Char type
            (Type::Char, Value::Char(_)) => true,

            // Void (only matches Nil)
            (Type::Void, Value::Nil) => true,

            // Unknown type accepts anything
            (Type::Unknown, _) => true,

            // Object type
            (Type::User(_), Value::Obj(_)) => true,

            // Array types
            (Type::Array(_), Value::Array(_)) => true,
            (Type::List(_), Value::Array(_)) => true,

            // Option types
            (Type::Option(_), Value::Nil) => true,
            (Type::Option(inner), v) => Self::type_binding_matches_type(inner, v),

            // Generic instance
            (Type::GenericInstance(_), _) => true,

            // Default: no match
            _ => false,
        };

        if type_matches {
            Some(MatchResult::new(vec![(name.to_string(), message.clone())]))
        } else {
            None
        }
    }

    /// Check if a value matches a type (for nested type checking)
    fn type_binding_matches_type(type_expr: &crate::ast::Type, message: &Value) -> bool {
        use crate::ast::Type;

        match (type_expr, message) {
            (Type::Str(_), Value::Str(_)) => true,
            (Type::Int, Value::Int(_) | Value::I64(_)) => true,
            (Type::Uint, Value::Uint(_)) => true,
            (Type::Bool, Value::Bool(_)) => true,
            (Type::Option(_), _) => true, // Option accepts anything
            (Type::Unknown, _) => true,
            _ => false,
        }
    }

    /// Match a simple variant pattern (e.g., `Reset`)
    fn match_simple_variant(variant_name: &str, message: &Value) -> Option<MatchResult> {
        match message {
            Value::Obj(obj) => {
                // Check if the object has a __variant field matching the name
                if let Some(Value::Str(variant)) = obj.get(AutoStr::from("__variant")) {
                    if variant.as_str() == variant_name {
                        return Some(MatchResult::empty());
                    }
                }
                None
            }
            Value::Str(s) => {
                // String messages match simple variants by value
                if s.as_str() == variant_name {
                    Some(MatchResult::empty())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Match a variant with bindings (e.g., `Add(val)`)
    fn match_variant_with_bindings(
        variant_name: &str,
        binding_names: &[AutoStr],
        message: &Value,
    ) -> Option<MatchResult> {
        match message {
            Value::Obj(obj) => {
                // Check if the object has a __variant field matching the name
                let variant = obj.get(AutoStr::from("__variant"))?;
                if let Value::Str(v) = variant {
                    if v.as_str() != variant_name {
                        return None;
                    }
                } else {
                    return None;
                }

                // Extract bindings from the object
                let mut bindings = Vec::new();
                for name in binding_names {
                    if let Some(value) = obj.get(name.clone()) {
                        bindings.push((name.to_string(), value.clone()));
                    } else {
                        // Binding not found
                        return None;
                    }
                }

                Some(MatchResult::new(bindings))
            }
            _ => None,
        }
    }

    /// Compare two values with a binary operator
    fn compare_values(left: &Value, op: &auto_val::Op, right: &Value) -> Result<bool, String> {
        use auto_val::Op;

        match op {
            Op::Gt => Self::compare_ordered(left, right, |a, b| a > b),
            Op::Lt => Self::compare_ordered(left, right, |a, b| a < b),
            Op::Ge => Self::compare_ordered(left, right, |a, b| a >= b),
            Op::Le => Self::compare_ordered(left, right, |a, b| a <= b),
            Op::Eq => Ok(Self::values_equal(left, right)),
            Op::Neq => Ok(!Self::values_equal(left, right)),
            _ => Err(format!("Invalid guard operator: {:?}", op)),
        }
    }

    /// Compare ordered values (numeric types)
    fn compare_ordered<F>(left: &Value, right: &Value, cmp: F) -> Result<bool, String>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(cmp(*a as i64, *b as i64)),
            (Value::Int(a), Value::I64(b)) => Ok(cmp(*a as i64, *b)),
            (Value::I64(a), Value::Int(b)) => Ok(cmp(*a, *b as i64)),
            (Value::I64(a), Value::I64(b)) => Ok(cmp(*a, *b)),
            (Value::Uint(a), Value::Uint(b)) => Ok(cmp(*a as i64, *b as i64)),
            (Value::Float(a), Value::Float(b)) => {
                let a_ord = a.to_bits() as i64;
                let b_ord = b.to_bits() as i64;
                Ok(cmp(a_ord, b_ord))
            }
            (Value::Double(a), Value::Double(b)) => {
                let a_ord = a.to_bits() as i64;
                let b_ord = b.to_bits() as i64;
                Ok(cmp(a_ord, b_ord))
            }
            _ => Err(format!("Cannot compare {:?} and {:?}", left, right)),
        }
    }

    /// Check if two values are equal
    fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Int(a), Value::I64(b)) => (*a as i64) == *b,
            (Value::I64(a), Value::Int(b)) => *a == (*b as i64),
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::Uint(a), Value::Uint(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a.as_str() == b.as_str(),
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => left == right,
        }
    }
}

impl fmt::Display for MatchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MatchResult(")?;
        for (i, (name, value)) in self.bindings.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} = {:?}", name, value)?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Type;
    use auto_val::Obj;

    #[test]
    fn test_match_literal_string() {
        let pattern = TaskMsgPattern::Literal(LiteralValue::String("ping".into()));
        let message = Value::str("ping");

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
        assert!(!result.unwrap().has_bindings());
    }

    #[test]
    fn test_match_literal_string_no_match() {
        let pattern = TaskMsgPattern::Literal(LiteralValue::String("ping".into()));
        let message = Value::str("pong");

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_literal_int() {
        let pattern = TaskMsgPattern::Literal(LiteralValue::Int(404));
        let message = Value::Int(404);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
    }

    #[test]
    fn test_match_literal_bool() {
        let pattern = TaskMsgPattern::Literal(LiteralValue::Bool(true));
        let message = Value::Bool(true);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());

        let pattern = TaskMsgPattern::Literal(LiteralValue::Bool(false));
        let message = Value::Bool(false);
        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
    }

    #[test]
    fn test_match_type_binding_string() {
        let pattern = TaskMsgPattern::TypeBinding {
            name: "msg".into(),
            type_expr: Box::new(Type::Str(0)),
        };
        let message = Value::str("hello");

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.has_bindings());
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].0, "msg");
    }

    #[test]
    fn test_match_type_binding_int() {
        let pattern = TaskMsgPattern::TypeBinding {
            name: "amount".into(),
            type_expr: Box::new(Type::Int),
        };
        let message = Value::Int(42);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.bindings[0].0, "amount");
    }

    #[test]
    fn test_match_type_binding_wrong_type() {
        let pattern = TaskMsgPattern::TypeBinding {
            name: "msg".into(),
            type_expr: Box::new(Type::Str(0)),
        };
        let message = Value::Int(42);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_simple_variant() {
        let pattern = TaskMsgPattern::Simple("Reset".into());

        // Create object with __variant field
        let mut obj = Obj::new();
        obj.set(AutoStr::from("__variant"), Value::str("Reset"));

        let message = Value::Obj(obj);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
    }

    #[test]
    fn test_values_equal() {
        assert!(PatternMatcher::values_equal(&Value::Int(1), &Value::Int(1)));
        assert!(PatternMatcher::values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(PatternMatcher::values_equal(&Value::str("a"), &Value::str("a")));
        assert!(!PatternMatcher::values_equal(&Value::Int(1), &Value::Int(2)));
    }

    #[test]
    fn test_compare_ordered() {
        assert!(PatternMatcher::compare_values(&Value::Int(10), &auto_val::Op::Gt, &Value::Int(5)).unwrap());
        assert!(PatternMatcher::compare_values(&Value::Int(5), &auto_val::Op::Lt, &Value::Int(10)).unwrap());
        assert!(PatternMatcher::compare_values(&Value::Int(5), &auto_val::Op::Le, &Value::Int(5)).unwrap());
        assert!(PatternMatcher::compare_values(&Value::Int(5), &auto_val::Op::Ge, &Value::Int(5)).unwrap());
    }

    #[test]
    fn test_match_result_display() {
        let result = MatchResult::new(vec![
            ("x".to_string(), Value::Int(1)),
            ("y".to_string(), Value::str("hello")),
        ]);

        let display = format!("{}", result);
        assert!(display.contains("x"));
        assert!(display.contains("y"));
    }

    #[test]
    fn test_match_literal_uint() {
        let pattern = TaskMsgPattern::Literal(LiteralValue::Uint(200));
        let message = Value::Uint(200);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
    }

    #[test]
    fn test_match_literal_char() {
        let pattern = TaskMsgPattern::Literal(LiteralValue::Char('a'));
        let message = Value::Char('a');

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
    }

    #[test]
    fn test_match_type_binding_bool() {
        let pattern = TaskMsgPattern::TypeBinding {
            name: "flag".into(),
            type_expr: Box::new(Type::Bool),
        };
        let message = Value::Bool(true);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
        assert_eq!(result.unwrap().bindings[0].0, "flag");
    }

    #[test]
    fn test_match_type_binding_unknown() {
        // Unknown type should match any value
        let pattern = TaskMsgPattern::TypeBinding {
            name: "value".into(),
            type_expr: Box::new(Type::Unknown),
        };
        let message = Value::Int(42);

        let result = PatternMatcher::match_pattern(&pattern, &message);
        assert!(result.is_some());
    }
}
