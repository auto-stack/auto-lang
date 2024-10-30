use std::fmt::{self, Display, Formatter};
use std::collections::BTreeMap;
use crate::ast;
use crate::ast::Op;

#[derive(Debug, Clone, PartialEq, Hash, Ord, Eq, PartialOrd)]
pub enum ValueKey {
    Str(String),
    Int(i32),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    values: BTreeMap<ValueKey, Value>,
}

impl Display for ValueKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueKey::Str(s) => write!(f, "{}", s),
            ValueKey::Int(i) => write!(f, "{}", i),
            ValueKey::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl Obj {
    pub fn new() -> Self {
        Obj { values: BTreeMap::new() }
    }

    pub fn get(&self, key: &ValueKey) -> Option<Value> {
        self.values.get(key).cloned()
    }

    pub fn set(&mut self, key: ValueKey, value: Value) {
        self.values.insert(key, value);
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        self.values.iter().find(|(k, _)| match k {
            ValueKey::Str(s) => s == name,
            ValueKey::Int(i) => i.to_string() == name,
            ValueKey::Bool(b) => b.to_string() == name,
        }).map(|(_, v)| v.clone())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Pair(ValueKey, Box<Value>),
    Object(Obj),
    Range(i32, i32),
    RangeEq(i32, i32),
    Fn(ast::Fn),
    ExtFn(fn(&Vec<Value>) -> Value),
    Nil,
    LambdaStub,
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

fn print_object(f: &mut Formatter<'_>, obj: &Obj) -> fmt::Result {
    write!(f, "{{")?;
    for (i, (k, v)) in obj.values.iter().enumerate() {
        write!(f, "{}: {}", k, v)?;
        if i < obj.values.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "}}")
}

impl Display for Value {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(value) => write!(f, "{}", value),
            Value::Int(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Bool(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "nil"),
            Value::Void => write!(f, ""),
            Value::Array(value) => print_array(f, value),
            Value::Range(left, right) => write!(f, "{}..{}", left, right),
            Value::RangeEq(left, right) => write!(f, "{}..={}", left, right),
            Value::Error(value) => write!(f, "Error: {}", value),
            Value::Fn(value) => write!(f, "{}", value),
            Value::ExtFn(_) => write!(f, "extfn"),
            Value::Pair(key, value) => write!(f, "{}: {}", key, value),
            Value::Object(value) => print_object(f, value),
            Value::LambdaStub => write!(f, "lambda"),
        }
    }
}

impl Value {
    pub fn neg(&self) -> Value {
        match self {
            Value::Int(value) => Value::Int(-value),
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
            (Value::Int(a), Value::Int(b)) => {
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
            Value::Int(value) => *value > 0,
            Value::Float(value) => *value > 0.0,
            Value::Str(value) => value.len() > 0,
            _ => false,
        }
    }
}


