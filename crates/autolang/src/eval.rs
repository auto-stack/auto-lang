use crate::ast::*;
use crate::parser;
use crate::scope;
use crate::scope::Meta;
use autoval::{Value, Op, Obj, ValueKey, ExtFn, MetaID, Sig, Method, Type};
use autoval;
use autoval::{add, sub, mul, div, comp};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use crate::ast;
use crate::error_pos;

pub enum EvalTempo {
    IMMEDIATE,
    LAZY,
}

pub enum EvalMode {
    SCRIPT, // normal evaluation
    CONFIG, // combine every pair/object in the same scope to one object; returns a big object
    TEMPLATE, // evaluate every statement into a string, and join them with newlines
}

pub struct Evaler {
    universe: Rc<RefCell<scope::Universe>>,
    // configure whether to evaluate a node immediately or lazily
    tempo_for_nodes: HashMap<String, EvalTempo>,
    // evaluation mode
    mode: EvalMode,
}

impl Evaler {
    pub fn new(universe: Rc<RefCell<scope::Universe>>) -> Self {
        let mut evaler = Evaler { universe, tempo_for_nodes: HashMap::new(), mode: EvalMode::SCRIPT };
        evaler.set_tempo("center", EvalTempo::LAZY);
        evaler.set_tempo("top", EvalTempo::LAZY);
        evaler.set_tempo("left", EvalTempo::LAZY);
        evaler.set_tempo("right", EvalTempo::LAZY);
        evaler.set_tempo("bottom", EvalTempo::LAZY);
        evaler
    }

    pub fn with_mode(mut self, mode: EvalMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_tempo(&mut self, name: &str, tempo: EvalTempo) {
        self.tempo_for_nodes.insert(name.to_string(), tempo);
    }

    pub fn interpret(&mut self, code: &str) -> Result<Value, String> {
        let ast = parser::parse(code, &mut *self.universe.borrow_mut())?;
        Ok(self.eval(&ast))
    }

    pub fn load_file(&mut self, filename: &str) -> Result<Value, String> {
        let code = std::fs::read_to_string(filename).map_err(|e| format!("Failed to read file: {}", e))?;
        self.interpret(&code)
    }

    pub fn eval(&mut self, code: &Code) -> Value {
        match self.mode {
            EvalMode::SCRIPT => {
                let mut value = Value::Nil;
                for stmt in code.stmts.iter() {
                    value = self.eval_stmt(stmt);
                    if value.is_error() {
                        panic!("Error: {}", value);
                    }
                }
                value
            }
            EvalMode::CONFIG => {
                let mut obj = Obj::new();
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt);
                    match val {
                        Value::Pair(key, value) => {
                            obj.set(key, *value);
                        }
                        Value::Obj(o) => {
                            obj.merge(&o);
                        }
                        _ => {}
                    }
                }
                Value::Obj(obj)
            }
            EvalMode::TEMPLATE => {
                let mut result = Vec::new();
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt);
                    match val {
                        Value::Str(s) => result.push(s),
                        _ => result.push(val.to_string()),
                    }
                }
                Value::Str(result.join("\n"))
            }
        }
    }

    pub fn dump_scope(&self) {
        self.universe.borrow().dump();
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Stmt::For(for_stmt) => self.eval_for(for_stmt),
            Stmt::Store(store) => self.eval_store(store),
            Stmt::Fn(_) => Value::Nil,
            Stmt::TypeDecl(type_decl) => self.type_decl(type_decl),
            Stmt::Widget(widget) => self.eval_widget(widget),
            Stmt::Node(node) => self.eval_node(node),
        }
    }

    fn eval_body(&mut self, body: &Body) -> Value {
        let mut res = Vec::new();
        for stmt in body.stmts.iter() {
            res.push(self.eval_stmt(stmt));
        }
        match self.mode {
            EvalMode::SCRIPT => res.last().unwrap_or(&Value::Nil).clone(),
            EvalMode::CONFIG => Value::Array(res),
            EvalMode::TEMPLATE => {
                Value::Str(res.iter().map(|v| {
                    match v {
                        Value::Str(s) => s.clone(),
                        _ => v.to_string(),
                    }
                }).collect::<Vec<String>>().join("\n"))
            }
        }
    }

    fn eval_loop_body(&mut self, body: &Body, is_mid: bool, is_new_line: bool) -> Value {
        self.universe.borrow_mut().set_local_val("is_mid", Value::Bool(is_mid));
        let mut res = Vec::new();
        let sep = if is_new_line { "\n" } else { "" };
        for stmt in body.stmts.iter() {
            res.push(self.eval_stmt(stmt));
        }
        match self.mode {
            EvalMode::SCRIPT => res.last().unwrap_or(&Value::Nil).clone(),
            EvalMode::CONFIG => Value::Array(res),
            EvalMode::TEMPLATE => Value::Str(res.iter().map(|v| {
                match v {
                    Value::Str(s) => s.clone(),
                    _ => v.to_string(),
                }
            }).collect::<Vec<String>>().join(sep))
        }
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

    fn eval_iter(&mut self, iter: &Iter, idx: usize, item: Value) {
        match iter {
            Iter::Indexed(index, iter) => {
                self.universe.borrow_mut().set_local_val(&index.text, Value::Int(idx as i32));
                // println!("set index {}, iter: {}, item: {}", index.text, iter.text, item.clone());
                self.universe.borrow_mut().set_local_val(&iter.text, item);
            },
            Iter::Named(iter) => self.universe.borrow_mut().set_local_val(&iter.text, item),
        }
    }

    fn eval_for(&mut self, for_stmt: &For) -> Value {
        let iter = &for_stmt.iter;
        let body = &for_stmt.body;
        let mut max_loop = 1000;
        let range = self.eval_expr(&for_stmt.range);
        let mut res = Vec::new();
        let mut is_mid = true;
        let is_new_line = for_stmt.new_line;
        let sep = if for_stmt.new_line { "\n" } else { "" };
        self.universe.borrow_mut().enter_scope();
        match range {
            Value::Range(start, end) => {
                let len = (end - start) as usize;
                for (idx, n) in (start..end).enumerate() {
                    if idx == len - 1 {
                        is_mid = false;
                    }
                    self.eval_iter(iter, idx, Value::Int(n));
                    res.push(self.eval_loop_body(body, is_mid, is_new_line));
                    max_loop -= 1;
                }
            }
            Value::RangeEq(start, end) => {
                let len = (end - start) as usize;
                for (idx, n) in (start..=end).enumerate() {
                    if idx == len - 1 {
                        is_mid = false;
                    }
                    self.eval_iter(iter, idx, Value::Int(n));
                    res.push(self.eval_loop_body(body, is_mid, is_new_line));
                    max_loop -= 1;
                }
            }
            Value::Array(values) => {
                let len = values.len();
                for (idx, item) in values.iter().enumerate() {
                    if idx == len - 1 {
                        is_mid = false;
                    }
                    self.eval_iter(iter, idx, item.clone());
                    res.push(self.eval_loop_body(body, is_mid, is_new_line));
                    max_loop -= 1;
                }
            }
            _ => {
                return Value::Error(format!("Invalid range {}", range));
            }
        }
        self.universe.borrow_mut().exit_scope();
        if max_loop <= 0 {
            return Value::Error("Max loop reached".to_string());
        } else {
            match self.mode {
                EvalMode::SCRIPT => Value::Void,
                EvalMode::CONFIG => Value::Array(res),
                EvalMode::TEMPLATE => Value::Str(res.iter().map(|v| {
                    match v {
                        Value::Str(s) => s.clone(),
                        _ => v.to_string(),
                    }
                }).collect::<Vec<String>>().join(sep))
            }
        }
    }

    fn eval_store(&mut self, store: &Store) -> Value {
        let value = match &store.expr {
            Expr::Ref(target) => Value::Ref(target.text.clone()),
            _ => self.eval_expr(&store.expr),
        };
        self.universe.borrow_mut().define(store.name.text.as_str(), Rc::new(scope::Meta::Store(store.clone())));
        self.universe.borrow_mut().set_local_val(&store.name.text, value);
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

    fn asn(&mut self, left: &Expr, val: Value) -> Value {
        match left {
            Expr::Ident(name) => {
                // check ref
                let left_val = self.lookup(&name.text);
                match left_val {
                    Value::Ref(target) => {
                        println!("ref: {}", target);
                        if self.universe.borrow().exists(&target) {
                            self.universe.borrow_mut().update_val(&target, val);
                        } else {
                            panic!("Invalid assignment, variable (ref {} -> {}) not found", name.text, target);
                        }
                    }
                    _ => {
                        println!("not ref: {}, {}", name.text, val);
                        if self.universe.borrow().exists(&name.text) {
                            self.universe.borrow_mut().update_val(&name.text, val);
                        } else {
                            panic!("Invalid assignment, variable {} not found", name.text);
                        }
                    }
                }
                Value::Void
            },
            Expr::Bina(left, op, right) => {
                match op {
                    Op::Dot => { // a.b = expr
                        match left.as_ref() {
                            Expr::Ident(name) => {
                                // find object `left`
                                let obj = self.update_obj(&name.text, move |o| {
                                    match right.as_ref() {
                                        Expr::Ident(rname) => o.set(rname.text.clone(), val),
                                        _ => {}
                                    }
                                });
                                Value::Void
                            }
                            _ => {
                                Value::Error(format!("Invalid assignment {}", left))
                            }
                        }
                    }
                    _ => Value::Error(format!("Invalid bina target of asn {} = {}", left, val)),
                }
            }
            Expr::Index(array, index) => {
                match array.as_ref() {
                    Expr::Ident(name) => {
                        let idx = self.eval_expr(index);
                        self.update_array(&name.text, idx, val);
                        Value::Void
                    }
                    _ => Value::Error(format!("Invalid target of asn index {} = {}", left, val)),
                }
            }
            _ => Value::Error(format!("Invalid target of asn {} = {}", left, val)),
        }
    }

    fn update_obj(&mut self, name: &str, f: impl FnOnce(&mut Obj)) -> Value {
        self.universe.borrow_mut().update_obj(name, f);
        Value::Void
    }

    fn update_array(&mut self, name: &str, idx: Value, val: Value) -> Value {
        self.universe.borrow_mut().update_array(name, idx, val);
        Value::Void
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
        self.universe.borrow().lookup_val(name).unwrap_or(Value::Nil)
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
        Value::Obj(obj)
    }

    fn pair(&mut self, pair: &Pair) -> Value {
        let key = self.eval_key(&pair.key);
        let value = self.eval_expr(pair.value.as_ref());
        Value::Pair(key, Box::new(value))
    }

    fn eval_key(&self, key: &Key) -> ValueKey {
        match key {
            Key::NamedKey(name) => ValueKey::Str(name.text.clone()),
            Key::IntKey(value) => ValueKey::Int(*value),
            Key::BoolKey(value) => ValueKey::Bool(*value),
            Key::StrKey(value) => ValueKey::Str(value.clone()),
        }
    }

    fn eval_args(&mut self, args: &ast::Args) -> autoval::Args {
        let array: Vec<Value> = args.array.iter().map(|arg| self.eval_expr(arg)).collect();
        let mut named: Vec<(ValueKey, Value)> = Vec::new();
        for (key, value) in args.map.iter() {
            let key_val = ValueKey::Str(key.text.clone());
            let value_val = self.eval_expr(value);
            named.push((key_val, value_val));
        }
        autoval::Args { array, named }
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
                Value::ExtFn(extfn) => {
                    let args_val = self.eval_args(&call.args);
                    return (extfn.fun)(&args_val);
                }
                Value::Lambda(name) => {
                    // Try to lookup lambda in SymbolTable
                    let meta = self.universe.borrow().lookup_meta(&name);
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
                Value::Method(method) => {
                    return self.eval_method(&method, &call.args);
                }
                _ => {
                    return Value::Error(format!("Invalid function call {}", name));
                }
            }
        }
        // Lookup Fn meta
        let meta = self.universe.borrow().lookup_meta(&call.get_name());
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

    pub fn eval_method(&mut self, method: &Method, args: &Args) -> Value {
        let target = &method.target;
        let name = &method.name;
        // methods for Any
        match target.as_ref() {
            Value::Str(_) => {
                let method = self.universe.borrow().types.lookup_method(Type::Str, name.clone());
                if let Some(method) = method {
                    return method(&target);
                }
            }
            _ => {
                let method = self.universe.borrow().types.lookup_method(Type::Any, name.clone());
                if let Some(method) = method {
                    return method(&target);
                }
            }
        }
        Value::Error(format!("Invalid method {} on {}", name, target))
    }

    fn eval_fn_call_with_sig(&mut self, sig: &Sig, args: &Args) -> Value {
        let meta = self.universe.borrow().lookup_sig(sig).unwrap();
        match meta.as_ref() {
            scope::Meta::Fn(fn_decl) => self.eval_fn_call(fn_decl, args),
            _ => Value::Error(format!("Invalid function call {}", sig.name)),
        }
    }

    pub fn eval_fn_call(&mut self, fn_decl: &Fn, args: &Args) -> Value {
        self.universe.borrow_mut().enter_scope();
        for (i, arg) in args.array.iter().enumerate() {
            let val = self.eval_expr(arg);
            let name = &fn_decl.params[i].name.text;
            self.universe.borrow_mut().set_local_val(&name, val);
        }
        for (name, expr) in args.map.iter() {
            let val = self.eval_expr(expr);
            self.universe.borrow_mut().set_local_val(&name.text, val);
        }
        let result = self.eval_body(&fn_decl.body);
        self.universe.borrow_mut().exit_scope();
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
            Expr::Ref(target) => {
                let target_val = self.eval_expr(&Expr::Ident(target.clone()));
                target_val
            }
            Expr::Ident(name) => {
                let res = self.lookup(&name.text);
                match res {
                    Value::Ref(target) => {
                        let target_val = self.eval_expr(&Expr::Ident(Name::new(target)));
                        target_val
                    }
                    Value::Nil => {
                        // try to lookup in meta and builtins
                        let meta = self.universe.borrow().lookup_meta(&name.text);
                        if let Some(meta) = meta {
                            return Value::Meta(to_meta_id(&meta));
                        }
                        // Try builtin
                        self.universe.borrow().lookup_builtin(&name.text).unwrap_or(Value::Nil)
                    }
                    _ => res,
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
            Expr::Ref(name) => Value::Ref(name.text.clone()),
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
            Value::Obj(obj) => match right {
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
            _ => {
                // try to lookup method
                match right {
                    Expr::Ident(name) => {
                        // TODO: too long
                        if self.universe.borrow().types.lookup_method_for_value(&left_value, name.text.clone()).is_some() {
                            Some(Value::Method(Method::new(left_value.clone(), name.text.clone())))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        };
        res.unwrap_or(Value::Error(format!("Invalid dot expression {}.{}", left_value, right)))
    }

    fn eval_widget(&mut self, widget: &Widget) -> Value {
        let name = &widget.name.text;
        // model
        let mut vars = Vec::new();
        for var in widget.model.vars.iter() {
            let value = self.eval_expr(&var.expr);
            vars.push((ValueKey::Str(var.name.text.clone()), value.clone()));
            self.universe.borrow_mut().set_local_val(&var.name.text, value);
        }
        let model = autoval::Model { values: vars };
        // view
        let view_id = format!("{}.view", name);
        self.universe.borrow_mut().define(&view_id, Rc::new(Meta::View(widget.view.clone())));
        let widget_value = autoval::Widget { name: name.clone(), model, view_id: MetaID::View(view_id) };
        let value = Value::Widget(widget_value);
        self.universe.borrow_mut().set_local_val(name, value.clone());
        self.universe.borrow_mut().widget = value.clone();
        value
    }

    fn eval_mid(&mut self, node: &Node) -> Value {
        let is_mid = self.universe.borrow().lookup_val("is_mid").unwrap_or(Value::Bool(false)).as_bool();
        let args = &node.args.array;
        let mut res = Value::Str("".to_string());
        if args.len() >= 1 {
            if is_mid { // mid 
                let mid = self.eval_expr(&args[0]);
                res = mid;
            }
        }
        if args.len() >= 2 {
            if !is_mid { // last
                let last = self.eval_expr(&args[1]);
                res = last;
            }
        }
        res
    }

    fn eval_node(&mut self, node: &Node) -> Value {
        let args_array = node.args.array.iter().map(|arg| self.eval_expr(arg)).collect();
        let args_named = node.args.map.iter().map(|(key, value)| {
            (ValueKey::Str(key.text.clone()), self.eval_expr(value))
        }).collect();
        let args = autoval::Args { array: args_array, named: args_named };
        let mut nodes = Vec::new();
        let mut props = BTreeMap::new();
        let mut body = MetaID::Nil;
        let name = &node.name.text;
        if name == "mid" {
            return self.eval_mid(&node);
        }
        let tempo = self.tempo_for_nodes.get(name).unwrap_or(&EvalTempo::IMMEDIATE);
        match tempo {
            EvalTempo::IMMEDIATE => {
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
            }
            EvalTempo::LAZY => {
                // push node body to scope meta
                // TODO: support multiple nodes of same name
                body = MetaID::Body(name.clone());
                println!("define global {}", name);
                self.universe.borrow_mut().define_global(&name, Rc::new(Meta::Body(node.body.clone())));
            }
        }
        Value::Node(autoval::Node { name: node.name.text.clone(), args, props, nodes, body })
    }

    fn eval_value_node_body(&mut self, node_val: &mut Value) {
        self.universe.borrow_mut().enter_scope();
        match node_val {
            Value::Node(ref mut node) => {
                let props = &mut node.props;
                let nodes = &mut node.nodes;
                let mut stmts = Vec::new();
                {
                    let scope = self.universe.borrow();
                    let meta = scope.lookup_meta(&node.name);
                    stmts = meta.map(|m| {
                        match m.as_ref() {
                            scope::Meta::Body(body) => body.stmts.clone(),
                            _ => Vec::new(),
                        }
                    }).unwrap();
                }
                for stmt in stmts.iter() {
                    let val = self.eval_stmt(stmt);
                    match val {
                        Value::Node(node) => {nodes.push(node);},
                        Value::Pair(key, value) => {props.insert(key, *value);},
                        _ => {},
                    }
                }
            },
            _ => {},
        };
        self.universe.borrow_mut().exit_scope();
    }

    fn fstr(&mut self, fstr: &FStr) -> Value {
        let parts: Vec<String> = fstr.parts.iter().map(|part| {
            let val = self.eval_expr(part);
            match val {
                Value::Str(s) => s,
                _ => val.to_string(),
            }
        }).collect();
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
        params.push(autoval::Param {
            name: param.name.text.clone(),
            ty: Box::new(to_value_type(&param.ty)),
        });
    }
    let ret = to_value_type(&fn_decl.ret.as_ref().unwrap());
    Sig { name: fn_decl.name.text.clone(), params, ret }
}

fn to_value_type(ty: &ast::Type) -> autoval::Type {
    match ty {
        ast::Type::Int => autoval::Type::Int,
        ast::Type::Float => autoval::Type::Float,
        ast::Type::Bool => autoval::Type::Bool,
        ast::Type::Str => autoval::Type::Str,
        ast::Type::Array(_) => autoval::Type::Array,
        ast::Type::User(type_decl) => autoval::Type::User(type_decl.name.text.clone()),
        ast::Type::Unknown => autoval::Type::Any,
    }
}
