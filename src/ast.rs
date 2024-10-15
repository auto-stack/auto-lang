use std::fmt;

#[derive(Debug)]
pub struct Code {
    pub stmts: Vec<Stmt>,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(code ")?;
        let last = self.stmts.len();
        let last = if last > 0 { last - 1 } else { 0 };
        for (i, stmt) in self.stmts.iter().enumerate() {
            write!(f, "{}", stmt)?;
            if i < last {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub cond: Expr,
    pub body: Body,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(branch {} {})", self.cond, self.body)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub text: String,
}

impl Name {
    pub fn new(text: String) -> Name {
        Name { text }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(name {})", self.text)
    }
}


#[derive(Debug, Clone)]
pub struct Var {
    pub name: Name,
    pub expr: Expr,
}


#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    If(Vec<Branch>, Option<Body>),
    For(Name, Expr, Body),
    Var(Var),
    Fn(Fn),
}

#[derive(Debug, Clone)]
pub struct Body {
    pub stmts: Vec<Stmt>,
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(body ")?;
        for stmt in self.stmts.iter() {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}
impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expr(expr) => write!(f, "(stmt {})", expr),
            Stmt::If(branches, else_stmt) => {
                write!(f, "(if ")?;
                for branch in branches.iter() {
                    write!(f, "{}", branch)?;
                }
                if let Some(else_stmt) = else_stmt {
                    write!(f, " (else {})", else_stmt)?;
                }
                Ok(())
            },
            Stmt::For(name, expr, body) => write!(f, "(for {} {} {})", name, expr, body),
            Stmt::Var(var) => write!(f, "(var {} {})", var.name, var.expr),
            Stmt::Fn(fn_decl) => write!(f, "{}", fn_decl),
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
    Asn,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Range,
    RangeEq,
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
            Op::LParen => write!(f, "(op ()"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    // value exprs
    Integer(i32),
    Float(f64),
    Bool(bool),
    Str(String),
    Ident(String),
    // composite exprs
    Unary(Op, Box<Expr>),
    Bina(Box<Expr>, Op, Box<Expr>),
    Array(Vec<Expr>),
    Call(/*name*/Box<Expr>, /*args*/Vec<Expr>),
    // stmt exprs
    If(Vec<Branch>, Option<Body>),
    Nil,
}

fn fmt_call(f: &mut fmt::Formatter, name: &Expr, args: &Vec<Expr>) -> fmt::Result {
    write!(f, "(call ")?;
    write!(f, "{}", name)?;
    write!(f, " (args")?;
    for arg in args.iter() {
        write!(f, " {}", arg)?;
    }
    write!(f, ")")?;
    Ok(())
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Integer(i) => write!(f, "(int {})", i),
            Expr::Float(v) => write!(f, "(float {})", v),
            Expr::Bool(b) => write!(f, "({})", b),
            Expr::Str(s) => write!(f, "(\"{}\")", s),
            Expr::Ident(i) => write!(f, "(name {})", i),
            Expr::Bina(l, op, r) => write!(f, "(bina {} {} {})", l, op, r),
            Expr::Unary(op, e) => write!(f, "(una {} {})", op, e),
            Expr::Array(elems) => write!(f, "(array {:?})", elems),
            Expr::If(branches, else_stmt) => write!(f, "(if {:?} {:?})", branches, else_stmt),
            Expr::Call(name, args) => fmt_call(f, name, args),
            Expr::Nil => write!(f, "(nil)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Name,
    pub default: Option<Expr>,
}

impl PartialEq for Param {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(param {}", self.name.text)?;
        if let Some(default) = &self.default {
            write!(f, "={}", default)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Fn {
    pub name: Name,
    pub params: Vec<Param>,
    pub body: Body,
}

impl PartialEq for Fn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.params == other.params
    }
}

impl fmt::Display for Fn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(fn {} (params ", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            write!(f, "{}", param)?;
            if i < self.params.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ") {}", self.body)
    }
}
