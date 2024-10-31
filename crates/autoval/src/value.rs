use std::fmt::{self, Display, Formatter};
use std::collections::BTreeMap;

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
    Fn(Fn),
    ExtFn(ExtFn),
    Nil,
    Lambda,
    Void,
    Error(String),
}

impl Into<Value> for String {
    fn into(self) -> Value {
        Value::Str(self)
    }
}

impl Into<Value> for i32 {
    fn into(self) -> Value {
        Value::Int(self)
    }
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
            Value::Fn(_) => write!(f, "fn"),
            Value::ExtFn(_) => write!(f, "extfn"),
            Value::Lambda => write!(f, "lambda"),
            Value::Pair(key, value) => write!(f, "{}: {}", key, value),
            Value::Object(value) => print_object(f, value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Not,
    LSquare,
    LParen,
    LBrace,
    Asn,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Range,
    RangeEq,
    Dot,
    Colon,
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

    pub fn to_bool(&self) -> bool {
        self.is_true()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub sig: Sig,
    pub fun: fn(&Vec<Value>) -> Value,
}

/// Function signature
#[derive(Debug, Clone, PartialEq)]
pub struct Sig {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Box<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    User(TypeInfo),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub name: String,
    pub members: Vec<Member>,
    pub methods: Vec<Fn>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Member {
    pub name: String,
    pub ty: Box<Type>,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::User(type_info) => write!(f, "{}", type_info),
        }
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct ExtFn {
    pub fun: fn(&Vec<Value>) -> Value,
}

impl PartialEq for ExtFn {
    fn eq(&self, other: &Self) -> bool {
        self.fun == other.fun
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Add => write!(f, "(op +)"),
            Op::Sub => write!(f, "(op -)"),
            Op::Mul => write!(f, "(op *)"),
            Op::Div => write!(f, "(op /)"),
            Op::Not => write!(f, "(op !)"),
            Op::LSquare => write!(f, "(op [)"),
            Op::Asn => write!(f, "(op =)"),
            Op::Eq => write!(f, "(op ==)"),
            Op::Neq => write!(f, "(op !=)"),
            Op::Lt => write!(f, "(op <)"),
            Op::Gt => write!(f, "(op >)"),
            Op::Le => write!(f, "(op <=)"),
            Op::Ge => write!(f, "(op >=)"),
            Op::Range => write!(f, "(op ..)"),
            Op::RangeEq => write!(f, "(op ..=)"),
            Op::Dot => write!(f, "(op .)"),
            Op::LParen => write!(f, "(op ()"),
            Op::LBrace => write!(f, "(op {{)"),
            Op::Colon => write!(f, "(op :)"),
        }
    }
}

impl Op {
    pub fn op(&self) -> &str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Not => "!",
            Op::LSquare => "[",
            Op::LParen => "(",
            Op::LBrace => "{",
            Op::Asn => "=",
            Op::Eq => "==",
            Op::Neq => "!=",
            Op::Lt => "<",
            Op::Gt => ">",
            Op::Le => "<=",
            Op::Ge => ">=",
            Op::Range => "..",
            Op::RangeEq => "..=",
            Op::Dot => ".",
            Op::Colon => ":",
        }
    }
}

fn try_promote(a: Value, b: Value) -> (Value, Value) {
    match (&a, &b) {
        (Value::Int(_), Value::Int(_)) => (a, b),
        (Value::Float(_), Value::Float(_)) => (a, b),
        (Value::Int(a), Value::Float(_)) => (Value::Float(*a as f64), b),
        (Value::Float(_), Value::Int(b)) => (a, Value::Float(*b as f64)),
        _ => (a, b),
    }
}

pub fn add(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    match (a, b) {
        (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
        _ => Value::Nil,
    }
}

pub fn sub(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    match (a, b) {
        (Value::Int(left), Value::Int(right)) => Value::Int(left - right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
        _ => Value::Nil,
    }
}

pub fn mul(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    match (a, b) {
        (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
        _ => Value::Nil,
    }
}

pub fn div(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    match (a, b) {
        (Value::Int(left), Value::Int(right)) => Value::Int(left / right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
        _ => Value::Nil,
    }
}

pub fn comp(a: &Value, op: &Op, b: &Value) -> Value {
    a.comp(op, b)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_vals() {
        let a = Value::Int(1);
        println!("{}", a);

        let b = Value::Float(1.0);
        println!("{}", b);

        let c = add(a, b);
        assert_eq!(c, Value::Float(2.0));
    }
}
