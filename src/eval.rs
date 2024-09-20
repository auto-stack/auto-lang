use crate::ast::{Code, Stmt, Expr, Op};
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
    
    
    

    fn eval_bina(&self, left: &Expr, op: &Op, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        match op {
            Op::Add => self.add(left_value, right_value),
            Op::Sub => self.sub(left_value, right_value),
            Op::Mul => self.mul(left_value, right_value),
            Op::Div => self.div(left_value, right_value),
            _ => Value::Nil,
            // Add more binary operations as needed
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
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::Nil => Value::Nil,
        }
    }
}
