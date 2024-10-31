use crate::ast::*;
use crate::parser;
use crate::scope;
use autoval::value::{Value, Op, Obj, ValueKey, ExtFn, Fn as FnVal};
use autoval::value::{add, sub, mul, div, comp};
use crate::error_pos;

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

    pub fn load_file(&mut self, filename: &str) -> Result<Value, String> {
        let code = std::fs::read_to_string(filename).map_err(|e| format!("Failed to read file: {}", e))?;
        self.interpret(&code)
    }

    pub fn eval(&mut self, code: &Code) -> Value {
        let mut value = Value::Nil;
        for stmt in code.stmts.iter() {
            value = self.eval_stmt(stmt);
        }
        value
    }

    pub fn dump_scope(&self) {
        self.universe.dump();
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Stmt::For(name, expr, body) => self.eval_for(&name.text, expr, body),
            Stmt::Var(var) => self.eval_var(var),
            Stmt::Fn(_) => Value::Nil,
            Stmt::TypeDecl(type_decl) => self.type_decl(type_decl),
            // TODO: no need to eval widget as it only needs to be translated into UI calls
            Stmt::Widget(_) => Value::Nil,
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
        let value = self.eval_expr(&var.expr);
        self.universe.set_local(&var.name.text, value);
        Value::Void
    }

    fn eval_bina(&mut self, left: &Expr, op: &Op, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        match op {
            Op::Add => add(left_value, right_value),
            Op::Sub => sub(left_value, right_value),
            Op::Mul => mul(left_value, right_value),
            Op::Div => div(left_value, right_value),
            Op::Eq | Op::Neq | Op::Lt | Op::Gt | Op::Le | Op::Ge => {
                comp(&left_value, &op, &right_value)
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
        // lookup value
        self.universe.lookup_val(name).unwrap_or(Value::Nil)
    }

    fn array(&mut self, elems: &Vec<Expr>) -> Value {
        let mut values = Vec::new();
        for elem in elems.iter() {
            values.push(self.eval_expr(elem));
        }
        Value::Array(values)
    }

    fn object(&mut self, pairs: &Vec<Pair>) -> Value {
        let mut obj = Obj::new();
        for pair in pairs.iter() {
            obj.set(self.eval_key(&pair.key), self.eval_expr(&pair.value));
        }
        Value::Object(obj)
    }

    fn pair(&mut self, pair: &Pair) -> Value {
        let key = self.eval_key(&pair.key);
        let value = self.eval_expr(pair.value.as_ref());
        Value::Pair(key, Box::new(value))
    }

    fn eval_key(&mut self, key: &Key) -> ValueKey {
        match key {
            Key::NamedKey(name) => ValueKey::Str(name.text.clone()),
            Key::IntKey(value) => ValueKey::Int(*value),
            Key::BoolKey(value) => ValueKey::Bool(*value),
            Key::StrKey(value) => ValueKey::Str(value.clone()),
        }
    }

    fn call(&mut self, call: &Call) -> Value {
        let name = self.eval_expr(&call.name);
        if name != Value::Nil {
            match name {
                Value::ExtFn(ExtFn { fun }) => {
                    let arg_vals: Vec<Value> = call.args.array
                        .iter().map(|arg| self.eval_expr(arg)).collect();
                    return fun(&arg_vals);
                }
                Value::Lambda => {
                    // Try to lookup lambda in SymbolTable
                    let meta = self.universe.lookup_meta(&call.get_name());
                    if let Some(meta) = meta {
                        match meta.as_ref() {
                            scope::Meta::Fn(fn_decl) => {
                                return self.eval_fn_call(fn_decl, &call.args);
                            }
                            _ => {
                                return Value::Error(format!("Invalid lambda {}", name));
                            }
                        }
                    } else {
                        return Value::Error(format!("Invalid lambda {}", name));
                    }
                }
                _ => {
                    return Value::Error(format!("Invalid function call {}", name));
                }
            }
        }
        // Lookup Fn meta
        let meta = self.universe.lookup_meta(&call.get_name());
        if let Some(meta) = meta {
            match meta.as_ref() {
                scope::Meta::Fn(fn_decl) => {
                    return self.eval_fn_call(fn_decl, &call.args);
                }
                _ => return Value::Error(format!("Invalid lambda {}", name)),
            }
        }
        Value::Error(format!("Invalid call {}", name))
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
            Expr::Pair(pair) => self.pair(pair),
            Expr::Object(pairs) => self.object(pairs),
            Expr::TypeInst(name, entries) => self.type_inst(name, entries),
            Expr::Lambda(_) => Value::Lambda,
            Expr::Node(node) => self.node(node),
            Expr::Nil => Value::Nil,
        }
    }

    fn type_inst(&mut self, _name: &Expr, _entries: &Vec<Pair>) -> Value {
        Value::Void
    }

    fn type_decl(&mut self, _type_decl: &TypeDecl) -> Value {
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

    fn node(&mut self, node: &Node) -> Value {
        let mut obj = Obj::new();
        for (i, arg) in node.args.array.iter().enumerate() {
            let name = format!("arg{}", i);
            obj.set(ValueKey::Str(name), self.eval_expr(arg));
        }
        for (key, value) in node.props.iter() {
            obj.set(self.eval_key(key), self.eval_expr(value));
        }
        Value::Object(obj)
    }
}
