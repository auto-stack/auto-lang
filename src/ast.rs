pub struct Code {
    pub stmts: Vec<Stmt>,
}

pub enum Stmt {
    Expr(Expr),
}

pub enum Expr {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Ident(String),
}