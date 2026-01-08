use super::{Expr, Name, Type};
use auto_val::AutoStr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Call {
    pub name: Box<Expr>,
    pub args: Args,
    pub ret: Type,
}

impl Call {
    pub fn get_name_text(&self) -> AutoStr {
        match &self.name.as_ref() {
            Expr::Ident(name) => name.clone(),
            _ => panic!("Expected identifier, got {:?}", self.name),
        }
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(call {})", self.args)
    }
}

#[derive(Debug, Clone)]
pub struct Args {
    // pub array: Vec<Expr>,
    // pub map: Vec<(Name, Expr)>,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub enum Arg {
    Pos(Expr),
    Name(Name),
    Pair(Name, Expr),
}

impl Arg {
    pub fn get_expr(&self) -> Expr {
        match self {
            Arg::Pos(expr) => expr.clone(),
            Arg::Name(name) => Expr::Str(name.clone()),
            Arg::Pair(_, expr) => expr.clone(),
        }
    }

    pub fn repr(&self) -> AutoStr {
        match self {
            Arg::Pos(expr) => expr.repr(),
            Arg::Name(name) => name.clone(),
            Arg::Pair(key, expr) => format!("{}:{}", key, expr.repr()).into(),
        }
    }

    pub fn to_code(&self) -> AutoStr {
        match self {
            Arg::Pos(expr) => expr.to_code(),
            Arg::Name(name) => name.clone(),
            Arg::Pair(key, expr) => format!("{}:{}", key, expr.to_code()).into(),
        }
    }
}

impl Args {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }
    pub fn get(&self, idx: usize) -> Option<Arg> {
        self.args.get(idx).cloned()
    }

    pub fn lookup(&self, name: &str) -> Option<Arg> {
        for arg in self.args.iter() {
            match arg {
                Arg::Name(n) => {
                    if n == name {
                        return Some(arg.clone());
                    }
                }
                Arg::Pair(n, _) => {
                    if n == name {
                        return Some(arg.clone());
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn id(&self) -> AutoStr {
        let empty = "".into();
        let id = match self.args.first() {
            Some(Arg::Name(name)) => name.clone(),
            Some(Arg::Pair(k, v)) => {
                if k == "id" {
                    v.repr().clone()
                } else {
                    empty
                }
            }
            Some(Arg::Pos(p)) => match p {
                Expr::Str(s) => s.clone(),
                Expr::Ident(n) => n.clone(),
                _ => empty,
            },
            _ => empty,
        };
        let id = if id.is_empty() {
            // try all args
            let arg = self.args.iter().find_map(|arg| match arg {
                Arg::Pair(k, v) => {
                    if k == "id" {
                        Some(v.repr().clone())
                    } else {
                        None
                    }
                }
                _ => None,
            });
            if let Some(arg) = arg {
                arg
            } else {
                id
            }
        } else {
            id
        };
        id
    }

    pub fn major(&self) -> Option<&Arg> {
        self.args.first()
    }

    pub fn first_arg(&self) -> Option<Expr> {
        let Some(arg) = self.args.first() else {
            return None;
        };
        match arg {
            Arg::Pos(expr) => Some(expr.clone()),
            Arg::Name(n) => Some(Expr::Ident(n.clone())),
            Arg::Pair(key, expr) => {
                // If it's an id: value pair, return the value
                if key == "id" {
                    Some(expr.clone())
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(args")?;
        if !self.args.is_empty() {
            for arg in self.args.iter() {
                write!(f, " {}", arg)?;
            }
        }
        write!(f, ")")
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Pos(expr) => write!(f, "{}", expr),
            Arg::Name(name) => write!(f, "(name {})", name),
            Arg::Pair(name, expr) => write!(f, "(pair (name {}) {})", name, expr),
        }
    }
}

pub fn fmt_call(f: &mut fmt::Formatter, call: &Call) -> fmt::Result {
    write!(f, "(call ")?;
    write!(f, "{}", call.name)?;
    if !call.args.is_empty() {
        write!(f, " {}", call.args)?;
    }
    write!(f, ")")?;
    Ok(())
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Array, Arg as AutoValArg, Node as AutoNode, Value, ValueKey};

impl ToNode for Arg {
    fn to_node(&self) -> AutoNode {
        match self {
            Arg::Pos(expr) => expr.to_atom().to_node(),
            Arg::Name(name) => {
                let mut node = AutoNode::new("name");
                node.add_arg(AutoValArg::Pos(Value::Str(name.clone())));
                node
            }
            Arg::Pair(key, expr) => {
                let mut node = AutoNode::new("pair");
                node.add_arg(AutoValArg::Pos(Value::str(key.as_str())));
                node.add_arg(AutoValArg::Pos(expr.to_atom()));
                node
            }
        }
    }
}

impl ToAtom for Arg {
    fn to_atom(&self) -> Value {
        match self {
            Arg::Pos(expr) => expr.to_atom(),
            Arg::Name(name) => Value::Str(name.clone()),
            Arg::Pair(key, expr) => Value::Pair(ValueKey::Str(key.clone()), Box::new(expr.to_atom())),
        }
    }
}

impl ToNode for Args {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("args");
        let items: Vec<Value> = self.args.iter().map(|arg| arg.to_atom()).collect();
        node.add_arg(AutoValArg::Pos(Value::array(Array::from_vec(items))));
        node
    }
}

impl ToAtom for Args {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

impl ToNode for Call {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("call");
        node.add_kid(self.name.to_atom().to_node());
        node.add_kid(self.args.to_node());

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", self.ret.to_atom());
        }

        node
    }
}

impl ToAtom for Call {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_to_atom_pos() {
        let arg = Arg::Pos(Expr::Int(42));
        let atom = arg.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "int");
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }

    #[test]
    fn test_arg_to_atom_name() {
        let arg = Arg::Name("x".into());
        let atom = arg.to_atom();

        match atom {
            Value::Str(s) => assert_eq!(s, "x"),
            _ => panic!("Expected Str, got {:?}", atom),
        }
    }

    #[test]
    fn test_arg_to_atom_pair() {
        let arg = Arg::Pair("key".into(), Expr::Int(42));
        let atom = arg.to_atom();

        match atom {
            Value::Pair(key, value) => {
                match key {
                    ValueKey::Str(s) => assert_eq!(s, "key"),
                    _ => panic!("Expected Str key"),
                }
                match &*value {
                    Value::Node(node) => assert_eq!(node.name, "int"),
                    _ => panic!("Expected Node value"),
                }
            }
            _ => panic!("Expected Pair, got {:?}", atom),
        }
    }

    #[test]
    fn test_args_to_atom_empty() {
        let args = Args::new();
        let atom = args.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "args");
                assert_eq!(node.args.args.len(), 1); // Has empty array arg
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }

    #[test]
    fn test_args_to_atom_with_args() {
        let mut args = Args::new();
        args.args.push(Arg::Pos(Expr::Int(1)));
        args.args.push(Arg::Pos(Expr::Int(2)));
        let atom = args.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "args");
                assert_eq!(node.args.args.len(), 1);
                match &node.args.args[0] {
                    AutoValArg::Pos(Value::Array(arr)) => {
                        assert_eq!(arr.len(), 2);
                    }
                    _ => panic!("Expected Array arg"),
                }
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }

    #[test]
    fn test_call_to_atom_simple() {
        let call = Call {
            name: Box::new(Expr::Ident("print".into())),
            args: Args::new(),
            ret: Type::Unknown,
        };
        let atom = call.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "call");
                assert_eq!(node.nodes.len(), 2); // name + args
                assert!(!node.has_prop("return")); // Unknown omitted
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }

    #[test]
    fn test_call_to_atom_with_return_type() {
        let call = Call {
            name: Box::new(Expr::Ident("getInt".into())),
            args: Args::new(),
            ret: Type::Int,
        };
        let atom = call.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "call");
                assert_eq!(node.get_prop("return"), Value::str("int"));
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }

    #[test]
    fn test_call_to_atom_with_args() {
        let mut args = Args::new();
        args.args.push(Arg::Pos(Expr::Int(42)));

        let call = Call {
            name: Box::new(Expr::Ident("print".into())),
            args,
            ret: Type::Unknown,
        };
        let atom = call.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "call");
                assert_eq!(node.nodes.len(), 2);
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }
}
