use crate::ast::{Code, Stmt, Expr, Op, Branch, Body};
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
            Stmt::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
        }
    }

    fn add(&self, left: Value, right: Value) -> Value {
        match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left + right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
            _ => Value::Nil,
        }
    }

    fn sub(&self, left: Value, right: Value) -> Value {
        match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left - right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
            _ => Value::Nil,
        }
    }

    fn mul(&self, left: Value, right: Value) -> Value {
        match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left * right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
            _ => Value::Nil,
        }
    }

    fn div(&self, left: Value, right: Value) -> Value {
        match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Value::Integer(left / right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
            _ => Value::Nil,
        }
    }

    fn eval_body(&self, body: &Body) -> Value {
        let mut value = Value::Nil;
        for stmt in body.stmts.iter() {
            value = self.eval_stmt(stmt);
        }
        value
    }

    fn eval_if(&self, branches: &Vec<Branch>, else_stmt: &Option<Body>) -> Value {
        for branch in branches.iter() {
            let cond = self.eval_expr(&branch.cond);
            if cond.is_true() {
                return self.eval_body(&branch.body);
            }
        }
        if let Some(else_stmt) = else_stmt {
            return self.eval_body(else_stmt);
        }
        Value::Nil
    }
    
    fn eval_bina(&self, left: &Expr, op: &Op, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        match op {
            Op::Add => self.add(left_value, right_value),
            Op::Sub => self.sub(left_value, right_value),
            Op::Mul => self.mul(left_value, right_value),
            Op::Div => self.div(left_value, right_value),
            Op::Eq | Op::Neq | Op::Lt | Op::Gt | Op::Le | Op::Ge => left_value.comp(op, &right_value),
            _ => Value::Nil,
        }
    }

    fn eval_una(&self, op: &Op, e: &Expr) -> Value {
        let value = self.eval_expr(e);
        match op {
            Op::Add => value,
            Op::Sub => value.neg(),
            Op::Not => value.not(),
            _ => Value::Nil,
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
            Expr::Unary(op, e) => self.eval_una(op, e),
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::Nil => Value::Nil,
        }
    }
}


