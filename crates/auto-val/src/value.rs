
use crate::AutoStr;
use crate::array::*;
use crate::obj::*;
use crate::pair::*;
use crate::string::*;
use crate::meta::*;
use crate::node::*;
use std::fmt::{self, Display, Formatter};



#[derive(Debug, Clone, PartialEq)]
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
}

// constructors
impl Value {
    pub fn str(text: impl Into<AutoStr>) -> Self {
        Value::Str(text.into())
    }

    pub fn error(text: impl Into<AutoStr>) -> Self {
        Value::Error(text.into())
    }

    pub fn array(items: impl Into<Array>) -> Self {
        Value::Array(items.into())
    }

    pub fn str_array(values: Vec<impl Into<AutoStr>>) -> Self {
        Value::Array(values.into_iter().map(|s| Value::Str(s.into())).collect::<Vec<Value>>().into())
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
            (Value::Byte(a), Value::Byte(b)) => {
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

static NODE_NIL: Node = Node {
    name: AutoStr::new(),
    args: Args::EMPTY,
    props: Obj::EMPTY,
    nodes: vec![],
    body: MetaID::Nil,
};

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
            _ => {},
        }
    }

    pub fn pretty(&self, max_indent: usize) -> String {
        pretty(format!("{}", self).as_str(), max_indent)
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }

    pub fn to_astr(&self) -> AutoStr {
        match self {
            Value::Str(s) => s.clone(),
            _ => self.to_string().into(),
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




impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.args.is_empty() {
            write!(f, " {}", self.args)?;
        }
        if !self.props.is_empty() {
            write!(f, " {{")?;
            for (i, (key, value)) in self.props.iter().enumerate() {
                write!(f, "{}: {}", key, value)?;
                if i < self.props.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "}}")?;
        }
        if !self.nodes.is_empty() {
            write!(f, " [")?;
            for (i, node) in self.nodes.iter().enumerate() {
                write!(f, "{}", node)?;
                if i < self.nodes.len() - 1 {
                    write!(f, "; ")?;
                }
            }
            write!(f, "]")?;
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

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub head: Vec<(ValueKey, Value)>,
    pub data: Vec<Vec<Value>>,
}

impl Default for Grid {
    fn default() -> Self {
        Self { head: vec![], data: vec![] }
    }
}

impl Grid {
    pub fn to_array_of_objects(&self) -> Value {
        let colids = self.head.iter().map(|(_, col)| col.as_obj().get_str_of("id")).collect::<Vec<_>>();
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

pub fn pretty(text: &str, max_indent: usize) -> String {
    let mut result = String::new();
    let mut indent = 0;
    let mut level = 0;
    let mut in_str = false;
    let tab = "    ";
    
    for c in text.chars() {
        match c {
            ' ' if !in_str => {
                if level > max_indent {
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
            '"' => {
                in_str = !in_str;
                result.push(c);
            }
            _ => result.push(c)
        }
    }
    result
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

impl<T> From<Vec<T>> for Value where T: Into<Value> {
    fn from(v: Vec<T>) -> Value {
        let array = Array { values: v.into_iter().map(|v| v.into()).collect() };
        Value::Array(array)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Str(s.into())
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
        assert_eq!(obj.get_array_of("a")[2].as_obj().get_uint_of("timeout"), 4000);
    }

    #[test]
    fn test_pretty() {
        let text = r#"{"a":[[1, 2, 3], [4, 5, 6]], "b":[[7, 8, 9], [10, 11, 12]], "c":[[13, 14, 15], [16, 17, 18]]}"#;
        let pretty = pretty(text, 2);
        println!("{}", pretty);
    }

}
