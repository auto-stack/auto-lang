use std::fmt::{self, Display, Formatter};

use crate::ast::Op;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i32),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Range(i32, i32),
    RangeEq(i32, i32),
    Nil,
    Void,
    Error(String),
}

fn float_eq(a: f64, b: f64) -> bool {
    let epsilon = 0.000001;
    (a - b).abs() < epsilon
}

fn print_array(f: &mut Formatter<'_>, value: &Vec<Value>) -> fmt::Result {
    write!(f, "[")?;
    for (i, v) in value.iter().enumerate() {
        write!(f, "{}", v)?;
        if i < value.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "]")
}

impl Display for Value {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(value) => write!(f, "{}", value),
            Value::Integer(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Bool(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "nil"),
            Value::Void => write!(f, ""),
            Value::Array(value) => print_array(f, value),
            Value::Range(left, right) => write!(f, "{}..{}", left, right),
            Value::RangeEq(left, right) => write!(f, "{}..={}", left, right),
            Value::Error(value) => write!(f, "Error: {}", value),
        }
    }
}

impl Value {
    pub fn neg(&self) -> Value {
        match self {
            Value::Integer(value) => Value::Integer(-value),
            Value::Float(value) => Value::Float(-value),
            _ => Value::Nil,
        }
    }

    pub fn not(&self) -> Value {
        match self {
            Value::Bool(value) => Value::Bool(!value),
            Value::Nil => Value::Bool(true),
            _ => Value::Nil,
        }
    }

    pub fn comp(&self, op: &Op, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                match op {
                    Op::Eq => Value::Bool(a == b),
                    Op::Neq => Value::Bool(a != b),
                    Op::Lt => Value::Bool(a < b),
                    Op::Gt => Value::Bool(a > b),
                    Op::Le => Value::Bool(a <= b),
                    Op::Ge => Value::Bool(a >= b),
                    _ => Value::Nil,
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                match op {
                    Op::Eq => Value::Bool(float_eq(*a, *b)),
                    Op::Neq => Value::Bool(!float_eq(*a, *b)),
                    Op::Lt => Value::Bool(*a < *b),
                    Op::Gt => Value::Bool(*a > *b),
                    Op::Le => Value::Bool(*a <= *b),
                    Op::Ge => Value::Bool(*a >= *b),
                    _ => Value::Nil,
                }
            }
            (Value::Bool(a), Value::Bool(b)) => {
                match op {
                    Op::Eq => Value::Bool(*a == *b),
                    Op::Neq => Value::Bool(*a != *b),
                    _ => Value::Nil,
                }
            }
            _ => Value::Nil,
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            Value::Bool(value) => *value,
            Value::Integer(value) => *value > 0,
            Value::Float(value) => *value > 0.0,
            Value::Str(value) => value.len() > 0,
            _ => false,
        }
    }
}


