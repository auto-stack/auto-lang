use crate::ast::*;
use crate::parser;
use crate::scope;
use crate::value::{Value, Obj, ValueKey};

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
            Stmt::For(name, expr, body) => self.eval_for(&name.text, expr, body),
            Stmt::Var(var) => self.eval_var(var),
            Stmt::Fn(fn_decl) => self.eval_fn(fn_decl),
            Stmt::TypeDecl(type_decl) => self.type_decl(type_decl),
            // TODO: no need to eval widget as it only needs to be translated into UI calls
            Stmt::Widget(_) => Value::Nil,
        }
    }

    fn try_promote(&self, left: Value, right: Value) -> (Value, Value) {
        match (&left, &right) {
            (Value::Int(_), Value::Int(_)) => (left, right),
            (Value::Float(_), Value::Float(_)) => (left, right),
            (Value::Int(left), Value::Float(_)) => (Value::Float(*left as f64), right),
            (Value::Float(_), Value::Int(right)) => (left, Value::Float(*right as f64)),
            _ => (left, right),
        }
    }

    fn add(&self, left: Value, right: Value) -> Value {
        let (left, right) = self.try_promote(left, right);
        match (left, right) {
            (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
            _ => Value::Nil,
        }
    }

    fn sub(&self, left: Value, right: Value) -> Value {
        let (left, right) = self.try_promote(left, right);
        match (left, right) {
            (Value::Int(left), Value::Int(right)) => Value::Int(left - right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
            _ => Value::Nil,
        }
    }

    fn mul(&self, left: Value, right: Value) -> Value {
        let (left, right) = self.try_promote(left, right);
        match (left, right) {
            (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
            (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
            _ => Value::Nil,
        }
    }

    fn div(&self, left: Value, right: Value) -> Value {
        let (left, right) = self.try_promote(left, right);
        match (left, right) {
            (Value::Int(left), Value::Int(right)) => Value::Int(left / right),
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

    fn eval_for(&mut self, name: &str, range: &Expr, body: &Body) -> Value {
        let mut max_loop = 100;
        let range = self.eval_expr(range);
        match range {
            Value::Range(start, end) => {
                for i in start..end {
                    self.universe.set_local(&name, Value::Int(i));
                    self.eval_body(body);
                    max_loop -= 1;
                }
            }
            Value::RangeEq(start, end) => {
                for i in start..=end {
                    self.universe.set_local(&name, Value::Int(i));
                    self.eval_body(body);
                    max_loop -= 1;
                }
            }
            Value::Array(values) => {
                for i in 0..values.len() {
                    self.universe.set_local(&name, values[i].clone());
                    self.eval_body(body);
                    max_loop -= 1;
                }
            }
            _ => {
                return Value::Error(format!("Invalid range {}", range));
            }
        }
        if max_loop <= 0 {
            return Value::Error("Max loop reached".to_string());
        } else {
            return Value::Void;
        }
    }

    fn eval_var(&mut self, var: &Var) -> Value {
        println!("eval var: {}", var);
        let value = self.eval_expr(&var.expr);
        self.universe.set_local(&var.name.text, value);
        Value::Void
    }

    fn eval_fn(&mut self, fn_decl: &Fn) -> Value {
        self.universe
            .set_global(&fn_decl.name.text, Value::Fn(fn_decl.clone()));
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
            Op::Eq | Op::Neq | Op::Lt | Op::Gt | Op::Le | Op::Ge => {
                left_value.comp(op, &right_value)
            }
            Op::Asn => self.asn(left, right_value),
            Op::Range => self.range(left, right),
            Op::RangeEq => self.range_eq(left, right),
            Op::Dot => self.dot(left, right),
            _ => Value::Nil,
        }
    }

    fn asn(&mut self, left: &Expr, right: Value) -> Value {
        if let Expr::Ident(name) = left {
            // TODO: check if name already exists
            self.universe.set_local(&name.text, right);
            Value::Void
        } else {
            panic!("Invalid assignment");
        }
    }

    fn range(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);
        match (&left_value, &right_value) {
            (Value::Int(left), Value::Int(right)) => Value::Range(*left, *right),
            _ => Value::Error(format!("Invalid range {}..{}", left_value, right_value)),
        }
    }

    fn range_eq(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);
        match (&left_value, &right_value) {
            (Value::Int(left), Value::Int(right)) => Value::RangeEq(*left, *right),
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
        self.universe.lookup_val(name).unwrap_or(Value::Nil)
    }

    fn array(&mut self, elems: &Vec<Expr>) -> Value {
        let mut values = Vec::new();
        for elem in elems.iter() {
            values.push(self.eval_expr(elem));
        }
        Value::Array(values)
    }

    fn object(&mut self, pairs: &Vec<(Key, Expr)>) -> Value {
        let mut obj = Obj::new();
        for (key, value) in pairs.iter() {
            obj.set(self.eval_key(key), self.eval_expr(value));
        }
        Value::Object(obj)
    }

    fn eval_key(&mut self, key: &Key) -> ValueKey {
        match key {
            Key::NamedKey(name) => ValueKey::Str(name.text.clone()),
            Key::IntKey(value) => ValueKey::Int(*value),
            Key::BoolKey(value) => ValueKey::Bool(*value),
        }
    }

    fn call(&mut self, call: &Call) -> Value {
        println!("call name: {:?}", call.name);
        let mut name = self.eval_expr(&call.name);
        if name == Value::LambdaStub {
            // Try to lookup lambda in SymbolTable
            let lambda = self.universe.get_symbol(&call.get_name());
            if let Some(meta) = lambda {
                match meta {
                    scope::Meta::Fn(fn_decl) => name = Value::Fn(fn_decl.clone()),
                    _ => return Value::Error(format!("Invalid lambda {}", name)),
                }
            }
        }
        let arg_vals: Vec<Value> = call.args.array.iter().map(|arg| self.eval_expr(arg)).collect();
        match name {
            Value::Fn(fn_decl) => self.eval_fn_call(&fn_decl, &call.args),
            Value::ExtFn(fp) => fp(&arg_vals),
            _ => Value::Error(format!("Invalid function call {}", name)),
        }
    }

    fn eval_fn_call(&mut self, fn_decl: &Fn, args: &Args) -> Value {
        self.universe.enter_scope();
        for (i, arg) in args.array.iter().enumerate() {
            let val = self.eval_expr(arg);
            let name = &fn_decl.params[i].name.text;
            self.universe.set_local(&name, val);
        }
        for (name, expr) in args.map.iter() {
            let val = self.eval_expr(expr);
            println!("set local {} = {}", name.text, val);
            self.universe.set_local(&name.text, val);
        }
        let result = self.eval_body(&fn_decl.body);
        self.universe.exit_scope();
        result
    }

    fn index(&mut self, array: &Expr, index: &Expr) -> Value {
        let array = self.eval_expr(array);
        let index_value = self.eval_expr(index);
        let mut idx = match index_value {
            Value::Int(index) => index,
            // TODO: support negative index
            // TODO: support range index
            _ => return Value::Error(format!("Invalid index {}", index_value)),
        };
        match array {
            Value::Array(values) => {
                let len = values.len();
                if idx >= len as i32 {
                    return Value::Error(format!("Index out of bounds {}", idx));
                }
                if idx < -(len as i32) {
                    return Value::Error(format!("Index out of bounds {}", idx));
                }
                if idx < 0 {
                    idx = len as i32 + idx;
                }
                values[idx as usize].clone()
            }
            _ => Value::Error(format!("Invalid array {}", array)),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Int(value) => Value::Int(*value),
            Expr::Float(value) => Value::Float(*value),
            // Why not move here?
            Expr::Str(value) => Value::Str(value.clone()),
            Expr::Bool(value) => Value::Bool(*value),
            Expr::Ident(name) => self.lookup(&name.text),
            Expr::Unary(op, e) => self.eval_una(op, e),
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Expr::Array(elems) => self.array(elems),
            Expr::Call(call) => self.call(call),
            Expr::Index(array, index) => self.index(array, index),
            Expr::Object(pairs) => self.object(pairs),
            Expr::TypeInst(name, entries) => self.type_inst(name, entries),
            Expr::Lambda(_) => Value::LambdaStub,
            Expr::Nil => Value::Nil,
        }
    }

    fn type_inst(&mut self, name: &Expr, entries: &Vec<(Key, Expr)>) -> Value {
        Value::Void
    }

    fn type_decl(&mut self, type_decl: &TypeDecl) -> Value {
        Value::Void
    }

    fn dot(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let res: Option<Value> = match &left_value {
            Value::Object(obj) => match right {
                Expr::Ident(name) => obj.lookup(&name.text),
                Expr::Int(key) => obj.lookup(&key.to_string()),
                Expr::Bool(key) => obj.lookup(&key.to_string()),
                _ => None,
            }
            _ => None,
        };
        res.unwrap_or(Value::Error(format!("Invalid object {}", left_value)))
    }
}
