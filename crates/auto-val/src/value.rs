use crate::array::*;
use crate::meta::*;
use crate::node::*;
use crate::obj::*;
use crate::pair::*;
use crate::string::*;
use crate::AutoStr;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    Byte(u8),
    Int(i32),
    Uint(u32),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(AutoStr),
    Array(Array),
    Pair(ValueKey, Box<Value>),
    Obj(Obj),
    Node(Node),
    Range(i32, i32),
    RangeEq(i32, i32),
    Fn(Fn),
    ExtFn(ExtFn),
    #[default]
    Nil,
    Lambda(AutoStr),
    Void,
    Widget(Widget),
    Model(Model),
    View(View),
    Meta(MetaID),
    Method(Method),
    Instance(Instance),
    Args(Args),
    Ref(AutoStr),
    Error(AutoStr),
    Grid(Grid),
    ConfigBody(ConfigBody),
}

// constructors
impl Value {
    pub fn str(text: impl Into<AutoStr>) -> Self {
        Value::Str(text.into())
    }

    pub fn error(text: impl Into<AutoStr>) -> Self {
        Value::Error(text.into())
    }

    pub fn empty_array() -> Self {
        Value::Array(Array::default())
    }

    pub fn array(items: impl Into<Array>) -> Self {
        Value::Array(items.into())
    }

    pub fn array_of(values: Vec<impl Into<Value>>) -> Self {
        Value::Array(Array::from_vec(values))
    }

    pub fn str_array(values: Vec<impl Into<AutoStr>>) -> Self {
        Value::Array(
            values
                .into_iter()
                .map(|s| Value::Str(s.into()))
                .collect::<Vec<Value>>()
                .into(),
        )
    }

    pub fn pair(key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        Value::Pair(key.into(), Box::new(value.into()))
    }

    pub fn obj() -> Self {
        Value::Obj(Obj::new())
    }

    pub fn repr(&self) -> AutoStr {
        match self {
            Value::Str(s) => s.clone(),
            _ => self.to_astr(),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }
}

impl Value {
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Value::Void)
    }
}

// arithmetic
impl Value {
    pub fn inc(&mut self) {
        match self {
            Value::Int(value) => *value += 1,
            Value::Uint(value) => *value += 1,
            _ => {}
        }
    }

    pub fn dec(&mut self) {
        match self {
            Value::Int(value) => *value -= 1,
            Value::Uint(value) => *value -= 1,
            _ => {}
        }
    }
}

fn float_eq(a: f64, b: f64) -> bool {
    let epsilon = 0.000001;
    (a - b).abs() < epsilon
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Char(value) => write!(f, "'{}'", value),
            Value::Str(value) => write!(f, "\"{}\"", value),
            Value::Int(value) => write!(f, "{}", value),
            Value::Uint(value) => write!(f, "{}u", value),
            Value::Byte(value) => write!(f, "0x{:X}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Bool(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "nil"),
            Value::Void => write!(f, "void"),
            Value::Array(value) => print_array(f, value),
            Value::Range(left, right) => write!(f, "{}..{}", left, right),
            Value::RangeEq(left, right) => write!(f, "{}..={}", left, right),
            Value::Error(value) => write!(f, "<Error: {}>", value),
            Value::Fn(fun) => write!(f, "fn {}", fun.sig.name),
            Value::ExtFn(fun) => write!(f, "<extfn {}>", fun.name),
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
            Value::Instance(instance) => write!(f, "{}", instance),
            Value::Grid(grid) => write!(f, "{}", grid),
            Value::ConfigBody(body) => write!(f, "{}", body),
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
            // TODO: add signed byte
            Value::Byte(value) => Value::Int(-(*value as i32)),
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
            (Value::Int(a), Value::Int(b)) => match op {
                Op::Eq => Value::Bool(a == b),
                Op::Neq => Value::Bool(a != b),
                Op::Lt => Value::Bool(a < b),
                Op::Gt => Value::Bool(a > b),
                Op::Le => Value::Bool(a <= b),
                Op::Ge => Value::Bool(a >= b),
                _ => Value::Nil,
            },
            (Value::Uint(a), Value::Uint(b)) => match op {
                Op::Eq => Value::Bool(a == b),
                Op::Neq => Value::Bool(a != b),
                Op::Lt => Value::Bool(a < b),
                Op::Gt => Value::Bool(a > b),
                Op::Le => Value::Bool(a <= b),
                Op::Ge => Value::Bool(a >= b),
                _ => Value::Nil,
            },
            (Value::Float(a), Value::Float(b)) => match op {
                Op::Eq => Value::Bool(float_eq(*a, *b)),
                Op::Neq => Value::Bool(!float_eq(*a, *b)),
                Op::Lt => Value::Bool(*a < *b),
                Op::Gt => Value::Bool(*a > *b),
                Op::Le => Value::Bool(*a <= *b),
                Op::Ge => Value::Bool(*a >= *b),
                _ => Value::Nil,
            },
            (Value::Bool(a), Value::Bool(b)) => match op {
                Op::Eq => Value::Bool(*a == *b),
                Op::Neq => Value::Bool(*a != *b),
                _ => Value::Nil,
            },
            (Value::Byte(a), Value::Byte(b)) => match op {
                Op::Eq => Value::Bool(a == b),
                Op::Neq => Value::Bool(a != b),
                Op::Lt => Value::Bool(a < b),
                Op::Gt => Value::Bool(a > b),
                Op::Le => Value::Bool(a <= b),
                Op::Ge => Value::Bool(a >= b),
                _ => Value::Nil,
            },
            (Value::Str(a), Value::Str(b)) => match op {
                Op::Eq => Value::Bool(a == b),
                Op::Neq => Value::Bool(a != b),
                _ => Value::Nil,
            },
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
            Value::Byte(value) => *value > 0,
            _ => false,
        }
    }

    pub fn to_bool(&self) -> bool {
        self.is_true()
    }
}

static NODE_NIL: Node = Node::empty();

// Quick Readers
impl Value {
    pub fn v_str(&self) -> Value {
        Value::Str(self.to_astr())
    }

    pub fn v_upper(&self) -> Value {
        match self {
            Value::Str(s) => Value::Str(s.to_uppercase()),
            _ => Value::Nil,
        }
    }

    pub fn v_lower(&self) -> Value {
        match self {
            Value::Str(s) => Value::Str(s.to_lowercase()),
            _ => Value::Nil,
        }
    }

    pub fn v_len(&self) -> Value {
        match self {
            Value::Str(s) => Value::Int(s.len() as i32),
            _ => Value::Nil,
        }
    }

    pub fn as_array(&self) -> &Array {
        match self {
            Value::Array(value) => value,
            _ => &ARRAY_EMPTY,
        }
    }

    pub fn to_str_vec(&self) -> Vec<AutoStr> {
        match self {
            Value::Array(value) => value.to_str_vec(),
            _ => vec![],
        }
    }

    pub fn as_obj(&self) -> &Obj {
        match self {
            Value::Obj(ref value) => value,
            _ => &OBJ_EMPTY,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Value::Str(value) => value.as_str(),
            _ => "",
        }
    }

    pub fn as_astr(&self) -> &AutoStr {
        match self {
            Value::Str(value) => value,
            _ => &ASTR_EMPTY,
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
            Value::Int(value) => *value as u32,
            _ => 0,
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            Value::Int(value) => *value,
            Value::Uint(value) => *value as i32,
            _ => 0,
        }
    }

    pub fn as_byte(&self) -> u8 {
        match self {
            Value::Byte(value) => *value,
            _ => 0,
        }
    }

    pub fn as_node(&self) -> &Node {
        match self {
            Value::Node(value) => value,
            _ => &NODE_NIL,
        }
    }

    pub fn update_node(&mut self, f: impl FnOnce(&mut Node)) {
        match self {
            Value::Node(value) => f(value),
            _ => {}
        }
    }

    pub fn pretty(&self, max_indent: usize) -> AutoStr {
        pretty(format!("{}", self).as_str(), max_indent)
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }

    pub fn to_astr(&self) -> AutoStr {
        match self {
            Value::Str(s) => s.clone(),
            Value::Nil => "".into(),
            _ => self.to_string().into(),
        }
    }

    pub fn to_astr_or(&self, default: &str) -> AutoStr {
        match self {
            Value::Str(s) => s.clone(),
            Value::Nil => default.into(),
            _ => self.to_string().into(),
        }
    }

    pub fn to_uint(&self) -> u32 {
        match self {
            Value::Int(n) => *n as u32,
            Value::Uint(n) => *n,
            _ => 0,
        }
    }

    pub fn name(&self) -> AutoStr {
        match self {
            Value::Node(node) => node.name.clone().into(),
            Value::Str(s) => s.clone().into(),
            Value::Fn(f) => f.sig.name.clone().into(),
            _ => self.to_astr(),
        }
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
    #[inline]
    pub fn repr(&self) -> &str {
        self.op()
    }

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
        // int => float
        (Value::Int(a), Value::Float(_)) => (Value::Float(*a as f64), b),
        (Value::Float(_), Value::Int(b)) => (a, Value::Float(*b as f64)),
        // byte => uint
        (Value::Byte(a), Value::Uint(b)) => (Value::Uint(*a as u32), Value::Uint(*b as u32)),
        (Value::Uint(a), Value::Byte(b)) => (Value::Uint(*a as u32), Value::Uint(*b as u32)),
        _ => (a, b),
    }
}

pub fn add(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    match (a, b) {
        (Value::Uint(left), Value::Uint(right)) => Value::Uint(left + right),
        (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
        (Value::Byte(left), Value::Byte(right)) => Value::Byte(left + right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
        // TODO: promote u32 or i32 to i64/u64
        // Current policy: convert rhs to lhs type if possible
        (Value::Uint(left), Value::Int(right)) => Value::Uint(left + right as u32),
        (Value::Int(left), Value::Uint(right)) => Value::Int(left + right as i32),
        // str
        (Value::Str(left), Value::Str(right)) => Value::Str(format!("{}{}", left, right).into()),
        // array
        (Value::Array(left), Value::Array(right)) => Value::Array({
            let mut res = left.clone();
            res.extend(&right);
            res
        }),
        // TODO: what if sum is bigger than u8?
        _ => Value::Nil,
    }
}

pub fn sub(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    match (a, b) {
        (Value::Int(left), Value::Int(right)) => Value::Int(left - right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
        // TODO: what if diff is negative?
        (Value::Byte(left), Value::Byte(right)) => Value::Byte(left - right),
        _ => Value::Nil,
    }
}

pub fn mul(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    match (a, b) {
        (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
        // TODO: what if product is bigger than u8?
        (Value::Byte(left), Value::Byte(right)) => Value::Byte(left * right),
        _ => Value::Nil,
    }
}

pub fn div(a: Value, b: Value) -> Value {
    let (a, b) = try_promote(a, b);
    if b == Value::Int(0) {
        // TODO: Value::Infinity?
        return Value::Nil;
    }
    match (a, b) {
        (Value::Int(left), Value::Int(right)) => Value::Int(left / right),
        (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
        (Value::Byte(left), Value::Byte(right)) => Value::Byte(left / right),
        _ => Value::Nil,
    }
}

pub fn comp(a: &Value, op: &Op, b: &Value) -> Value {
    a.comp(op, b)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Widget {
    pub name: AutoStr,
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
        self.values
            .iter()
            .find(|(k, _)| k.to_string() == key)
            .map(|(_, v)| v.clone())
    }
}

impl View {
    pub fn find(&self, key: &str) -> Option<Value> {
        self.nodes
            .iter()
            .find(|n| n.name == key)
            .map(|n| Value::Node(n.clone()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: AutoStr,
    pub target: Box<Value>,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.target, self.name)
    }
}

impl Method {
    pub fn new(target: Value, name: AutoStr) -> Self {
        Self {
            target: Box::new(target),
            name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub head: Vec<(ValueKey, Value)>,
    pub data: Vec<Vec<Value>>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            head: vec![],
            data: vec![],
        }
    }
}

impl Grid {
    pub fn to_array_of_objects(&self) -> Value {
        let colids = self
            .head
            .iter()
            .map(|(_, col)| col.as_obj().get_str_of("id"))
            .collect::<Vec<_>>();
        let mut result = Vec::new();
        for row in self.data.iter() {
            let mut obj = Obj::new();
            for (j, cell) in row.iter().enumerate() {
                obj.set(colids[j].clone(), cell.clone());
            }
            result.push(Value::Obj(obj));
        }
        Value::array(result)
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "grid")?;
        write!(f, "(")?;
        for (key, value) in &self.head {
            write!(f, "{}:{}", key, value)?;
            write!(f, ",")?;
        }
        write!(f, ")")?;
        write!(f, " {{")?;
        for (i, row) in self.data.iter().enumerate() {
            write!(f, "[")?;
            for (j, cell) in row.iter().enumerate() {
                write!(f, "{}", cell)?;
                if j < row.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
            if i < self.data.len() - 1 {
                write!(f, ";")?;
            }
        }
        write!(f, "}}")
    }
}

pub fn pretty(text: &str, max_indent: usize) -> AutoStr {
    let mut result = String::new();
    let mut indent = 0;
    let mut level = 0;
    let mut in_str = false;
    let tab = "    ";
    let mut last_c = ' ';

    for c in text.chars() {
        match c {
            ' ' if !in_str => {
                if last_c == ')' || level > max_indent {
                    result.push(c);
                }
            }
            ':' if !in_str => {
                result.push(c);
                result.push(' ');
            }
            '{' | '[' if !in_str => {
                if indent < max_indent {
                    result.push(c);
                    result.push('\n');
                    indent += 1;
                    result.push_str(&tab.repeat(indent));
                } else {
                    result.push(c);
                }
                level += 1;
            }
            '}' | ']' if !in_str => {
                if indent == max_indent {
                    if level <= max_indent {
                        result.push('\n');
                        indent -= 1;
                        result.push_str(&tab.repeat(indent));
                    }
                } else if indent < max_indent && indent > 0 {
                    result.push('\n');
                    indent -= 1;
                    result.push_str(&tab.repeat(indent));
                }
                result.push(c);
                level -= 1;
            }
            ',' if !in_str => {
                if indent == max_indent {
                    if level <= max_indent {
                        result.push(c);
                        result.push('\n');
                        result.push_str(&tab.repeat(indent));
                    } else {
                        result.push(c);
                    }
                } else if indent < max_indent {
                    result.push(c);
                    result.push('\n');
                    result.push_str(&tab.repeat(indent));
                } else {
                    result.push(c);
                }
            }
            ';' if !in_str => {
                result.push('\n');
                result.push_str(&tab.repeat(indent));
            }
            '"' => {
                in_str = !in_str;
                result.push(c);
            }
            _ => result.push(c),
        }
        last_c = c;
    }
    result.into()
}

impl From<AutoStr> for Value {
    fn from(astr: AutoStr) -> Value {
        Value::Str(astr)
    }
}

impl From<Obj> for Value {
    fn from(obj: Obj) -> Value {
        Value::Obj(obj)
    }
}

impl From<Node> for Value {
    fn from(node: Node) -> Value {
        Value::Node(node)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Str(s.into())
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

// impl From<Vec<Value>> for Value {
//     fn from(v: Vec<Value>) -> Value {
//         Value::Array(v)
//     }
// }

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Vec<T>) -> Value {
        let array = Array {
            values: v.into_iter().map(|v| v.into()).collect(),
        };
        Value::Array(array)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Str(s.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pair {
    pub key: ValueKey,
    pub value: Value,
}

impl Pair {
    pub fn new(key: ValueKey, value: Value) -> Self {
        Self { key, value }
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigItem {
    Pair(Pair),
    Object(Obj),
    Node(Node),
    Value(Value), // simple values
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigBody {
    pub items: Vec<ConfigItem>,
}

impl ConfigBody {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn add_pair(&mut self, pair: Pair) {
        self.items.push(ConfigItem::Pair(pair));
    }

    pub fn add_object(&mut self, object: Obj) {
        self.items.push(ConfigItem::Object(object));
    }

    pub fn add_node(&mut self, node: Node) {
        self.items.push(ConfigItem::Node(node));
    }

    pub fn add_val(&mut self, val: Value) {
        self.items.push(ConfigItem::Value(val));
    }

    pub fn to_node(self, name: impl Into<AutoStr>) -> Node {
        let mut node = Node::new(name);
        for item in self.items.into_iter() {
            match item {
                ConfigItem::Pair(pair) => node.set_prop(pair.key.to_string(), pair.value),
                ConfigItem::Object(object) => node.merge_obj(object),
                ConfigItem::Node(n) => node.add_kid(n),
                ConfigItem::Value(v) => node.set_prop("value".to_string(), v),
            }
        }
        node
    }
}

impl fmt::Display for ConfigItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigItem::Pair(pair) => write!(f, "{}: {}", pair.key, pair.value),
            ConfigItem::Object(object) => write!(f, "{}", object),
            ConfigItem::Node(node) => write!(f, "{}", node),
            ConfigItem::Value(v) => write!(f, "{}", v),
        }
    }
}

impl fmt::Display for ConfigBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for item in &self.items {
            write!(f, "{}", item)?;
        }
        write!(f, "}}")
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

    #[test]
    fn test_modify_obj() {
        let mut obj = Obj::new();
        let mut s1 = Obj::new();
        s1.set("timeout", 1000);
        let mut s2 = Obj::new();
        s2.set("timeout", 2000);
        let mut s3 = Obj::new();
        s3.set("timeout", 3000);
        obj.set("a", Value::from(vec![s1, s2, s3]));
        if let Some(Value::Array(a)) = obj.get_mut("a") {
            for s in a.iter_mut() {
                if let Value::Obj(o) = s {
                    if let Some(Value::Int(timeout)) = o.get_mut("timeout") {
                        *timeout += 1000;
                    }
                }
            }
        }
        assert_eq!(
            obj.get_array_of("a")[2].as_obj().get_uint_of("timeout"),
            4000
        );
    }

    #[test]
    fn test_pretty() {
        let text = r#"{"a":[[1, 2, 3], [4, 5, 6]], "b":[[7, 8, 9], [10, 11, 12]], "c":[[13, 14, 15], [16, 17, 18]]}"#;
        let pretty = pretty(text, 2);
        println!("{}", pretty);
    }
}
