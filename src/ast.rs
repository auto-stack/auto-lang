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

#[derive(Debug)]
pub struct Branch {
    pub cond: Expr,
    pub body: Body,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(branch {} {})", self.cond, self.body)
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    If(Vec<Branch>, Option<Body>),
    For(Expr, Body),
}

#[derive(Debug)]
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
            Stmt::For(cond, body) => write!(f, "(for {} {})", cond, body),
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
    Asn,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
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
        }
    }
}

#[derive(Debug)]
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
    Nil,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Integer(i) => write!(f, "(int {})", i),
            Expr::Float(v) => write!(f, "(float {})", v),
            Expr::Bool(b) => write!(f, "({})", b),
            Expr::Str(s) => write!(f, "(\"{}\")", s),
            Expr::Ident(i) => write!(f, "({})", i),
            Expr::Bina(l, op, r) => write!(f, "(bina {} {} {})", l, op, r),
            Expr::Unary(op, e) => write!(f, "(una {} {})", op, e),
            Expr::Nil => write!(f, "(nil)"),
        }
    }
}

