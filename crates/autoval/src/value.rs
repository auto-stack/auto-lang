use std::fmt::{self, write, Display, Formatter};
use std::collections::BTreeMap;
use crate::types::{Type, TypeInfo};

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

impl IntoIterator for Obj {
    type Item = (ValueKey, Value);
    type IntoIter = std::collections::btree_map::IntoIter<ValueKey, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
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

impl Into<ValueKey> for i32 {
    fn into(self) -> ValueKey {
        ValueKey::Int(self)
    }
}

impl Into<ValueKey> for bool {
    fn into(self) -> ValueKey {
        ValueKey::Bool(self)
    }
} 

impl Into<ValueKey> for i64 {
    fn into(self) -> ValueKey {
        ValueKey::Int(self as i32)
    }
}

impl Into<ValueKey> for String {
    fn into(self) -> ValueKey {
        ValueKey::Str(self)
    }
}

impl Into<ValueKey> for &str {
    fn into(self) -> ValueKey {
        ValueKey::Str(self.to_string())
    }
}

impl From<Obj> for Value {
    fn from(obj: Obj) -> Value {
        Value::Obj(obj)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Str(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Bool(b)
    }
}

impl From<u8> for Value {
    fn from(u: u8) -> Value {
        Value::Uint(u as u32)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Value {
        Value::Int(i)
    }
}

impl From<u32> for Value {
    fn from(u: u32) -> Value {
        Value::Uint(u)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Value {
        Value::Float(f)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Value {
        Value::Int(i as i32)
    }
}

impl From<u64> for Value {
    fn from(u: u64) -> Value {
        Value::Uint(u as u32)
    }
}   

impl From<f32> for Value {
    fn from(f: f32) -> Value {
        Value::Float(f as f64)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Value {
        Value::Array(v)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Str(s.to_string())
    }
}

impl Obj {
    pub fn new() -> Self {
        Obj { values: BTreeMap::new() }
    }

    pub fn get(&self, key: impl Into<ValueKey>) -> Option<Value> {
        self.values.get(&key.into()).cloned()
    }

    pub fn get_or_nil(&self, key: impl Into<ValueKey>) -> Value {
        self.get(key).unwrap_or(Value::Nil)
    }

    pub fn set(&mut self, key: impl Into<ValueKey>, value: impl Into<Value>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        self.values.iter().find(|(k, _)| match k {
            ValueKey::Str(s) => s == name,
            ValueKey::Int(i) => i.to_string() == name,
            ValueKey::Bool(b) => b.to_string() == name,
        }).map(|(_, v)| v.clone())
    }

    pub fn get_or(&self, name: &str, default: Value) -> Value {
        self.lookup(name).unwrap_or(default)
    }

    pub fn get_str_or(&self, name: &str, default: &str) -> String {
        match self.get(ValueKey::Str(name.to_string())) {
            Some(Value::Str(s)) => s,
            _ => default.to_string(),
        }
    }

    pub fn get_str_of(&self, name: &str) -> String {
        self.get_str_or(name, "")
    }

    pub fn get_float_or(&self, name: &str, default: f64) -> f64 {
        match self.get(ValueKey::Str(name.to_string())) {
            Some(Value::Float(f)) => f,
            _ => default,
        }
    }

    pub fn get_uint_or(&self, name: &str, default: u32) -> u32 {
        match self.get(name) {
            Some(Value::Uint(u)) => u,
            Some(Value::Int(i)) => if i >= 0 { i as u32 } else { default },
            _ => default,
        }
    }

    pub fn get_uint_of(&self, name: &str) -> u32 {
        self.get_uint_or(name, 0)
    }

    pub fn get_bool_or(&self, name: &str, default: bool) -> bool {
        match self.get(ValueKey::Str(name.to_string())) {
            Some(Value::Bool(b)) => b,
            _ => default,
        }
    }

    pub fn get_bool_of(&self, name: &str) -> bool {
        self.get_bool_or(name, false)
    }

    pub fn get_array_or(&self, name: &str, default: &Vec<Value>) -> Vec<Value> {
        match self.get(ValueKey::Str(name.to_string())) {
            Some(Value::Array(a)) => a,
            _ => default.clone(),
        }
    }

    pub fn get_array_of(&self, name: &str) -> Vec<Value> {
        self.get_array_or(name, &vec![])
    }

    pub fn merge(&mut self, other: Obj) {
        for (key, value) in other.values {
            self.set(key, value);
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        print_object(f, self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Uint(u32),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Pair(ValueKey, Box<Value>),
    Obj(Obj),
    Node(Node),
    Range(i32, i32),
    RangeEq(i32, i32),
    Fn(Fn),
    ExtFn(ExtFn),
    Nil,
    Lambda(String),
    Void,
    Widget(Widget),
    Model(Model),
    View(View),
    Meta(MetaID),
    Method(Method),
    Args(Args),
    Ref(String),
    Error(String),
}

// constructors
impl Value {
    pub fn array() -> Self {
        Value::Array(vec![])
    }

    pub fn str_array(values: Vec<impl Into<String>>) -> Self {
        Value::Array(values.into_iter().map(|s| Value::Str(s.into())).collect())
    }

    pub fn obj() -> Self {
        Value::Obj(Obj::new())
    }
}

impl Value {
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
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
            Value::Uint(value) => write!(f, "{}", value),
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
            Value::Lambda(name) => write!(f, "lambda {}", name),
            Value::Pair(key, value) => write!(f, "{} : {}", key, value),
            Value::Obj(value) => print_object(f, value),
            Value::Node(node) => write!(f, "{}", node),
            Value::Widget(widget) => write!(f, "{}", widget),
            Value::Meta(meta) => write!(f, "{}", meta),
            Value::Model(model) => write!(f, "{}", model),
            Value::Method(method) => write!(f, "{}", method),
            Value::Args(args) => write!(f, "{}", args),
            Value::View(view) => write!(f, "{}", view),
            Value::Ref(target) => write!(f, "(ref {})", target),
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
            // TODO: check if uint is bigger than i32.MAX
            Value::Uint(value) => Value::Int(-(*value as i32)),
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
            (Value::Uint(a), Value::Uint(b)) => {
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
            Value::Uint(value) => *value > 0,
            Value::Float(value) => *value > 0.0,
            Value::Str(value) => value.len() > 0,
            _ => false,
        }
    }

    pub fn to_bool(&self) -> bool {
        self.is_true()
    }
}


static OBJ_NIL: Obj = Obj { values: BTreeMap::new() };
static ARRAY_NIL: Vec<Value> = vec![];
static STR_NIL: String = String::new();

// Quick Readers
impl Value {
    pub fn str(&self) -> String {
        format!("{}", self)
    }

    pub fn v_str(&self) -> Value {
        Value::Str(self.str())
    }

    pub fn v_up(&self) -> Value {
        match self {
            Value::Str(s) => Value::Str(s.to_uppercase()),
            _ => Value::Nil,
        }
    }

    pub fn as_array(&self) -> &Vec<Value> {
        match self {
            Value::Array(value) => value,
            _ => &ARRAY_NIL,
        }
    }

    pub fn as_obj(&self) -> &Obj {
        match self {
            Value::Obj(ref value) => value,
            _ => &OBJ_NIL,
        }
    }

    pub fn as_string(&self) -> &String {
        match self {
            Value::Str(value) => value,
            _ => &STR_NIL,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Value::Str(value) => value.as_str(),
            _ => "",
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(value) => *value,
            _ => false,
        }
    }

    pub fn as_uint(&self) -> u32 {
        match self {
            Value::Uint(value) => *value,
            _ => 0,
        }
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
    pub ret: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct ExtFn {
    pub fun: fn(&Args) -> Value,
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
        (Value::Uint(left), Value::Uint(right)) => Value::Uint(left + right),
        // TODO: promote u32 or i32 to i64
        (Value::Uint(left), Value::Int(right)) => Value::Int(left as i32 + right),
        (Value::Int(left), Value::Uint(right)) => Value::Int(left + right as i32),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Widget {
    pub name: String,
    pub model: Model,
    pub view_id: MetaID,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub values: Vec<(ValueKey, Value)>,
}

impl Model {
    pub fn new() -> Self {
        Self { values: vec![] }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub nodes: Vec<Node>,
}

impl View {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub args: Args,
    pub props: BTreeMap<ValueKey, Value>,
    pub nodes: Vec<Node>,
    pub body: MetaID,
}

impl Node {
    pub fn get_prop(&self, key: &str) -> Value {
        match self.props.get(&ValueKey::Str(key.to_string())) {
            Some(value) => value.clone(),
            None => Value::Nil,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub array: Vec<Value>,
    pub named: Vec<(ValueKey, Value)>,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(args")?;
        for arg in &self.array {
            write!(f, " {}", arg)?;
        }
        Ok(())
    }
}

impl Args {
    pub fn new() -> Self {
        Self { array: Vec::new(), named: Vec::new() }
    }

    pub fn array(values: Vec<impl Into<Value>>) -> Self {
        Self { array: values.into_iter().map(|v| v.into()).collect(), named: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty() && self.named.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetaID {
    Fn(Sig),
    Lambda(Sig),
    View(String),
    Body(String),
    Method(MethodMeta),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodMeta {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for MethodMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::{}", self.ty, self.name)
    }
}

impl fmt::Display for MetaID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetaID::Fn(sig) => write!(f, "<fn:{}>", sig),
            MetaID::Lambda(sig) => write!(f, "<lambda:{}>", sig),
            MetaID::View(id) => write!(f, "<view:{}>", id),
            MetaID::Body(id) => write!(f, "<body:{}>", id),
            MetaID::Nil => write!(f, "<meta-nil>"),
            MetaID::Method(method) => write!(f, "<method:{}>", method),
        }
    }
}

impl fmt::Display for Sig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for param in &self.params {
            write!(f, " {}", param)?;
        }
        write!(f, " -> {}", self.ret)
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.args.is_empty() {
            write!(f, "(")?;
            for (i, arg) in self.args.array.iter().enumerate() {
                write!(f, "{}", arg)?;
                if i < self.args.array.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            for (key, value) in &self.args.named {
                write!(f, "{}={}, ", key, value)?;
            }
            write!(f, ")")?;
        }
        if !self.props.is_empty() {
            write!(f, " {{")?;
            for (key, value) in &self.props {
                write!(f, " {}: {}", key, value)?;
            }
            write!(f, " }}")?;
        }
        if !self.nodes.is_empty() {
            write!(f, " {{")?;
            for node in &self.nodes {
                write!(f, " {}; ", node)?;
            }
            write!(f, " }}")?;
        }
        if self.body != MetaID::Nil {
            write!(f, " {}", self.body)?;
        }
        Ok(())
    }
}

impl fmt::Display for Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "widget {} {{", self.name)?;
        writeln!(f, "    {}", self.model)?;
        writeln!(f, "    {}", self.view_id)?;
        writeln!(f, "}}")
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "model {{")?;
        for (i, (key, value)) in self.values.iter().enumerate() {
            write!(f, " {} = {}", key, value)?;
            if i < self.values.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, " }}")
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "view {{")?;
        for node in &self.nodes {
            write!(f, " {}", node)?;
        }
        write!(f, " }}")
    }
}

impl Model {
    pub fn find(&self, key: &str) -> Option<Value> {
        self.values.iter().find(|(k, _)| k.to_string() == key).map(|(_, v)| v.clone())
    }
}

impl View {
    pub fn find(&self, key: &str) -> Option<Value> {
        self.nodes.iter().find(|n| n.name == key).map(|n| Value::Node(n.clone()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: String,
    pub target: Box<Value>,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.target, self.name)
    }
}

impl Method {
    pub fn new(target: Value, name: String) -> Self {
        Self { target: Box::new(target), name }
    }
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

    // #[test]
    // fn test_widget() {
    //     let model = Model { values: vec![(ValueKey::Str("a".to_string()), Value::Int(1))], };
    //     let node = Node { name: "button".to_string(), args: vec![Value::Str("Hello".to_string())], props: BTreeMap::new() };
    //     let view = View { nodes: vec![node] };
    //     let widget = Widget { name: "counter".to_string(), model, view };
    //     println!("{}", widget);
    // }

}
