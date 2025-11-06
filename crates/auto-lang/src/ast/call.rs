use super::{Expr, Name};
use auto_val::AutoStr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Call {
    pub name: Box<Expr>,
    pub args: Args,
}

impl Call {
    pub fn get_name_text(&self) -> AutoStr {
        match &self.name.as_ref() {
            Expr::Ident(name) => name.clone(),
            _ => panic!("Expected identifier, got {:?}", self.name),
        }
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
            _ => None,
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
