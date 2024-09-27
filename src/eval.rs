use crate::ast::{Code, Stmt, Expr, Op, Branch, Body, Var};
use crate::parser;
use crate::scope;
use crate::value::Value;

pub struct Evaler<'a> {
    universe: &'a mut scope::Universe,
}

impl<'a> Evaler<'a> {
    pub fn new(universe: &'a mut scope::Universe) -> Self {
        Evaler { universe }
    }

    pub fn interpret(&mut self, code: &str) -> Result<Value, String> {
        let ast = parser::parse(code, self.universe)?;
        Ok(self.eval(&ast))
    }

    pub fn eval(&mut self, code: &Code) -> Value {
        let mut value = Value::Nil;
        for stmt in code.stmts.iter() {
            value = self.eval_stmt(stmt);
        }
        value
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Stmt::For(cond, body) => self.eval_for(cond, body),
            Stmt::Var(var) => self.eval_var(var),
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

    fn eval_body(&mut self, body: &Body) -> Value {
        let mut value = Value::Nil;
        for stmt in body.stmts.iter() {
            value = self.eval_stmt(stmt);
        }
        value
    }

    fn eval_if(&mut self, branches: &Vec<Branch>, else_stmt: &Option<Body>) -> Value {
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

    fn eval_for(&mut self, cond: &Expr, body: &Body) -> Value {
        let mut value = Value::Nil;
        let mut max_loop = 100;
        while self.eval_expr(cond).is_true() && max_loop > 0 {
            value = self.eval_body(body);
            max_loop -= 1;
        }
        if max_loop <= 0 {
            println!("Warning: for loop max loop reached");
        }
        value
    }

    fn eval_var(&mut self, var: &Var) -> Value {
        let value = self.eval_expr(&var.expr);
        self.universe.set_local(&var.name.text, value);
        Value::Void
    }

    fn eval_bina(&mut self, left: &Expr, op: &Op, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        match op {
            Op::Add => self.add(left_value, right_value),
            Op::Sub => self.sub(left_value, right_value),
            Op::Mul => self.mul(left_value, right_value),
            Op::Div => self.div(left_value, right_value),
            Op::Eq | Op::Neq | Op::Lt | Op::Gt | Op::Le | Op::Ge => left_value.comp(op, &right_value),
            Op::Asn => self.asn(left, right_value),
            Op::Range => self.range(left, right),
            Op::RangeEq => self.range_eq(left, right),
            _ => Value::Nil,
        }
    }

    fn asn(&mut self, left: &Expr, right: Value) -> Value {
        if let Expr::Ident(name) = left {   
            // check if name exists
            self.universe.set_local(&name, right);
            Value::Void
        } else {
            panic!("Invalid assignment");
        }
    }

    fn range(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);
        match (&left_value, &right_value) {
            (Value::Integer(left), Value::Integer(right)) => Value::Range(*left, *right),
            _ => Value::Error(format!("Invalid range {}..{}", left_value, right_value)),
        }
    }

    fn range_eq(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);
        match (&left_value, &right_value) {
            (Value::Integer(left), Value::Integer(right)) => Value::RangeEq(*left, *right),
            _ => Value::Error(format!("Invalid range {}..={}", left_value, right_value)),
        }
    }

    fn eval_una(&mut self, op: &Op, e: &Expr) -> Value {
        let value = self.eval_expr(e);
        match op {
            Op::Add => value,
            Op::Sub => value.neg(),
            Op::Not => value.not(),
            _ => Value::Nil,
        }
    }

    fn lookup(&self, name: &str) -> Value {
        self.universe.get_local(name)
    }

    fn array(&mut self, elems: &Vec<Expr>) -> Value {
        let mut values = Vec::new();
        for elem in elems.iter() {
            values.push(self.eval_expr(elem));
        }
        Value::Array(values)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Integer(value) => Value::Integer(*value as i32),
            Expr::Float(value) => Value::Float(*value as f64),
            // Why not move here?
            Expr::Str(value) => Value::Str(value.clone()),
            Expr::Bool(value) => Value::Bool(*value),
            Expr::Ident(name) => self.lookup(name),
            Expr::Unary(op, e) => self.eval_una(op, e),
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Expr::Array(elems) => self.array(elems),
            Expr::Nil => Value::Nil,
        }
    }
}



