use crate::ast::*;
use crate::parser;
use crate::scope;
use crate::scope::Meta;
use autoval::value::{Value, Op, Obj, ValueKey, ExtFn, MetaID, Sig};
use autoval::value;
use autoval::value::{add, sub, mul, div, comp};
use std::rc::Rc;
use std::collections::BTreeMap;
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
            Stmt::Widget(widget) => self.eval_widget(widget),
            Stmt::Node(node) => self.eval_node(node),
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
            if self.universe.exists(&name.text) {
                self.universe.update_val(&name.text, right);
            } else {
                panic!("Invalid assignment, variable {} not found", name.text);
            }
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
                Value::Meta(meta_id) => {
                    match meta_id {
                        MetaID::Fn(sig) => {
                            return self.eval_fn_call_with_sig(&sig, &call.args);
                        }
                        _ => {
                            println!("Strange function call {}", meta_id);
                        }
                    }
                }
                Value::ExtFn(ExtFn { fun }) => {
                    let arg_vals: Vec<Value> = call.args.array
                        .iter().map(|arg| self.eval_expr(arg)).collect();
                    return fun(&arg_vals);
                }
                Value::Lambda(name) => {
                    // Try to lookup lambda in SymbolTable
                    let meta = self.universe.lookup_meta(&name);
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
                Value::Widget(_widget) => {
                    let node: Node = call.clone().into();
                    return self.eval_node(&node);
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
                _ => return Value::Error(format!("Invalid lambda {}", call.get_name())),
            }
        } else {
            // convert call to node intance
            let node: Node = call.clone().into();
            return self.eval_node(&node);
        }
    }

    fn eval_fn_call_with_sig(&mut self, sig: &Sig, args: &Args) -> Value {
        let meta = self.universe.lookup_sig(sig).unwrap();
        match meta.as_ref() {
            scope::Meta::Fn(fn_decl) => self.eval_fn_call(fn_decl, args),
            _ => Value::Error(format!("Invalid function call {}", sig.name)),
        }
    }

    pub fn eval_fn_call(&mut self, fn_decl: &Fn, args: &Args) -> Value {
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

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Int(value) => Value::Int(*value),
            Expr::Float(value) => Value::Float(*value),
            // Why not move here?
            Expr::Str(value) => Value::Str(value.clone()),
            Expr::Bool(value) => Value::Bool(*value),
            Expr::Ident(name) => {
                let res = self.lookup(&name.text);
                if res == Value::Nil {
                    // try to lookup in meta and builtins
                    let meta = self.universe.lookup_meta(&name.text);
                    if let Some(meta) = meta {
                        return Value::Meta(to_meta_id(&meta));
                    }
                    // Try builtin
                    self.universe.lookup_builtin(&name.text).unwrap_or(Value::Nil)
                } else {
                    res
                }
            },
            Expr::Unary(op, e) => self.eval_una(op, e),
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Expr::Array(elems) => self.array(elems),
            Expr::Call(call) => self.call(call),
            Expr::Index(array, index) => self.index(array, index),
            Expr::Pair(pair) => self.pair(pair),
            Expr::Object(pairs) => self.object(pairs),
            Expr::TypeInst(name, entries) => self.type_inst(name, entries),
            Expr::Lambda(lambda) => Value::Lambda(lambda.name.text.clone()),
            Expr::FStr(fstr) => self.fstr(fstr),
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
            Value::Widget(widget) => match right {
                Expr::Ident(name) => {
                    match name.text.as_str() {
                        "model" => Some(Value::Model(widget.model.clone())),
                        "view" => Some(Value::Meta(widget.view_id.clone())),
                        _ => None,
                    }
                }
                _ => None,
            }
            Value::Model(model) => match right {
                Expr::Ident(name) => model.find(&name.text),
                _ => None,
            }
            Value::View(view) => match right {
                Expr::Ident(name) => view.find(&name.text),
                _ => None,
            }
            _ => None,
        };
        res.unwrap_or(Value::Error(format!("Invalid object {}", left_value)))
    }

    fn eval_widget(&mut self, widget: &Widget) -> Value {
        let name = &widget.name.text;
        // model
        let mut vars = Vec::new();
        for var in widget.model.vars.iter() {
            let value = self.eval_expr(&var.expr);
            vars.push((ValueKey::Str(var.name.text.clone()), value.clone()));
            self.universe.set_local(&var.name.text, value);
        }
        let model = value::Model { values: vars };
        // view
        let view_meta = self.universe.define("view_id", Rc::new(Meta::View(widget.view.clone())));
        let widget_value = value::Widget { name: name.clone(), model, view_id: MetaID::View("view_id".to_string())};

        // let mut nodes = Vec::new();
        // for (_, node) in widget.view.nodes.iter() {
        //     nodes.push(self.eval_node(node));
        // }
        // let view = value::View { nodes };
        // let widget_value = value::Widget { name: name.clone(), model, view };
        let value = Value::Widget(widget_value);
        self.universe.set_local(name, value.clone());
        self.universe.widget = value.clone();
        value
    }

    fn eval_node(&mut self, node: &Node) -> Value {
        let args_array = node.args.array.iter().map(|arg| self.eval_expr(arg)).collect();
        let args_named = node.args.map.iter().map(|(key, value)| {
            (ValueKey::Str(key.text.clone()), self.eval_expr(value))
        }).collect();
        let args = value::Args { array: args_array, named: args_named };
        let mut nodes = Vec::new();
        let mut props = BTreeMap::new();
        for stmt in node.body.stmts.iter() {
            let val = self.eval_stmt(stmt);
            match val {
                Value::Node(node) => nodes.push(node),
                Value::Pair(key, value) => {
                    props.insert(key, *value);
                }
                _ => {}
            }
        }
        Value::Node(value::Node { name: node.name.text.clone(), args, props, nodes })
    }

    fn fstr(&mut self, fstr: &FStr) -> Value {
        let parts: Vec<String> = fstr.parts.iter().map(|part| self.eval_expr(part).to_string()).collect();
        Value::Str(parts.join(""))
    }
}

fn to_meta_id(meta: &Rc<scope::Meta>) -> MetaID {
    match meta.as_ref() {
        scope::Meta::Fn(fn_decl) => MetaID::Fn(to_value_sig(&fn_decl)),
        _ => MetaID::Nil,
    }
}

fn to_value_sig(fn_decl: &Fn) -> Sig {
    let mut params = Vec::new();
    for param in fn_decl.params.iter() {
        params.push(value::Param {
            name: param.name.text.clone(),
            ty: Box::new(to_value_type(&param.ty)),
        });
    }
    let ret = to_value_type(&fn_decl.ret.as_ref().unwrap());
    Sig { name: fn_decl.name.text.clone(), params, ret }
}

fn to_value_type(ty: &Type) -> value::Type {
    match ty {
        Type::Int => value::Type::Int,
        Type::Float => value::Type::Float,
        Type::Bool => value::Type::Bool,
        Type::Str => value::Type::Str,
        Type::User(type_decl) => value::Type::User(to_value_type_info(type_decl)),
    }
}

fn to_value_type_info(type_decl: &TypeDecl) -> value::TypeInfo {
    value::TypeInfo { name: type_decl.name.text.clone(), members: Vec::new(), methods: Vec::new() }
}
