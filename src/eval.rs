use crate::ast::{Code, Stmt, Expr};
use crate::value::Value;

pub struct Evaler {
}

impl Evaler {
    pub fn new() -> Self {
        Evaler { }
    }

    pub fn eval(&self, code: &Code) -> Value {
        let mut value = Value::Nil;
        for stmt in code.stmts.iter() {
            value = self.eval_stmt(stmt);
        }
        value
    }

    fn eval_stmt(&self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
        }
    }

    fn eval_expr(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Integer(value) => Value::Integer(*value as i32),
            Expr::Float(value) => Value::Float(*value as f64),
            // Why not move here?
            Expr::Str(value) => Value::Str(value.clone()),
            Expr::Bool(value) => Value::Bool(*value),
            Expr::Ident(value) => Value::Nil,
        }
    }
}
