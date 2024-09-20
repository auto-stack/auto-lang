#[derive(Debug)]
pub struct Code {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
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
    Bina(Box<Expr>, Op, Box<Expr>),
    Nil,
}