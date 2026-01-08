use super::{Expr, Name, Type};
use crate::ast::{AtomWriter, ToAtomStr};
use auto_val::AutoStr;
use std::{fmt, io as stdio};

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
use auto_val::{Arg as AutoValArg, Array, Node as AutoNode, Value, ValueKey};

impl AtomWriter for Arg {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self {
            Arg::Pos(expr) => write!(f, "{}", expr.to_atom_str())?,
            Arg::Name(name) => write!(f, "name(\"{}\")", name)?,
            Arg::Pair(key, expr) => write!(f, "pair(name(\"{}\"), {})", key, expr.to_atom_str())?,
        }
        Ok(())
    }
}

impl ToNode for Arg {
    fn to_node(&self) -> AutoNode {
        match self {
            Arg::Pos(expr) => expr.to_node(), // Changed from expr.to_atom().to_node()
            Arg::Name(name) => {
                let mut node = AutoNode::new("name");
                node.add_arg(AutoValArg::Pos(Value::Str(name.clone())));
                node
            }
            Arg::Pair(key, expr) => {
                let mut node = AutoNode::new("pair");
                node.add_arg(AutoValArg::Pos(Value::str(key.as_str())));
                node.add_arg(AutoValArg::Pos(Value::str(&*expr.to_atom())));
                node
            }
        }
    }
}

impl ToAtom for Arg {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for Args {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "args")?;
        for arg in &self.args {
            write!(f, "({})", arg.to_atom_str())?;
        }
        Ok(())
    }
}

impl ToNode for Args {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("args");
        let items: Vec<Value> = self
            .args
            .iter()
            .map(|arg| Value::str(&*arg.to_atom()))
            .collect();
        node.add_arg(AutoValArg::Pos(Value::array(Array::from_vec(items))));
        node
    }
}

impl ToAtom for Args {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for Call {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "call(name(\"{}\")) {{", self.name.to_atom_str())?;
        for arg in &self.args.args {
            write!(f, " {}", arg.to_atom_str())?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToNode for Call {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("call");
        node.add_kid(self.name.to_node()); // Changed from name.to_atom().to_node()
        node.add_kid(self.args.to_node());

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", Value::str(&*self.ret.to_atom()));
        }

        node
    }
}

impl ToAtom for Call {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_to_atom_pos() {
        let arg = Arg::Pos(Expr::Int(42));
        let atom = arg.to_atom();
        assert_eq!(atom, "int(42)");
    }

    #[test]
    fn test_arg_to_atom_name() {
        let arg = Arg::Name("x".into());
        let atom = arg.to_atom();
        assert!(
            atom.contains("x"),
            "Expected atom to contain 'x', got: {}",
            atom
        );
    }

    #[test]
    fn test_arg_to_atom_pair() {
        let arg = Arg::Pair("key".into(), Expr::Int(42));
        let atom = arg.to_atom();
        // Should be in format "(pair key int(42))"
        assert!(
            atom.contains("pair"),
            "Expected atom to contain 'pair', got: {}",
            atom
        );
        assert!(
            atom.contains("key"),
            "Expected atom to contain 'key', got: {}",
            atom
        );
        assert!(
            atom.contains("int(42)"),
            "Expected atom to contain 'int(42)', got: {}",
            atom
        );
    }

    #[test]
    fn test_args_to_atom_empty() {
        let args = Args::new();
        let atom = args.to_atom();
        // Should be in format "(args)"
        assert!(
            atom.contains("args"),
            "Expected atom to contain 'args', got: {}",
            atom
        );
    }

    #[test]
    fn test_args_to_atom_with_args() {
        let mut args = Args::new();
        args.args.push(Arg::Pos(Expr::Int(1)));
        args.args.push(Arg::Pos(Expr::Int(2)));
        let atom = args.to_atom();
        // Should be in format "(args int(1) int(2))"
        assert!(
            atom.contains("args"),
            "Expected atom to contain 'args', got: {}",
            atom
        );
        assert!(
            atom.contains("int(1)"),
            "Expected atom to contain 'int(1)', got: {}",
            atom
        );
        assert!(
            atom.contains("int(2)"),
            "Expected atom to contain 'int(2)', got: {}",
            atom
        );
    }

    #[test]
    fn test_call_to_atom_simple() {
        let call = Call {
            name: Box::new(Expr::Ident("print".into())),
            args: Args::new(),
            ret: Type::Unknown,
        };
        let atom = call.to_atom();
        // Should be in format "(call ident(print) (args))"
        assert!(
            atom.contains("call"),
            "Expected atom to contain 'call', got: {}",
            atom
        );
        assert!(
            atom.contains("print"),
            "Expected atom to contain 'print', got: {}",
            atom
        );
    }

    #[test]
    fn test_call_to_atom_with_return_type() {
        let call = Call {
            name: Box::new(Expr::Ident("getInt".into())),
            args: Args::new(),
            ret: Type::Int,
        };
        let atom = call.to_atom();
        // Should contain return type info
        assert!(
            atom.contains("call"),
            "Expected atom to contain 'call', got: {}",
            atom
        );
        assert!(
            atom.contains("getInt"),
            "Expected atom to contain 'getInt', got: {}",
            atom
        );
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
        // Should be in format "(call ident(print) (args int(42)))"
        assert!(
            atom.contains("call"),
            "Expected atom to contain 'call', got: {}",
            atom
        );
        assert!(
            atom.contains("print"),
            "Expected atom to contain 'print', got: {}",
            atom
        );
        assert!(
            atom.contains("int(42)"),
            "Expected atom to contain 'int(42)', got: {}",
            atom
        );
    }
}
