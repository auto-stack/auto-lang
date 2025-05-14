use crate::ast;
use crate::ast::*;
use crate::scope;
use crate::scope::Meta;
use crate::universe::Universe;
use auto_val;
use auto_val::{add, comp, div, mul, sub};
use auto_val::{
    Array, AutoStr, ConfigBody, ConfigItem, MetaID, Method, Obj, Op, Pair, Sig, Type, Value,
    ValueKey,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub enum EvalTempo {
    IMMEDIATE,
    LAZY,
}

pub enum EvalMode {
    SCRIPT,   // normal evaluation
    CONFIG,   // combine every pair/object in the same scope to one object; returns a big object
    TEMPLATE, // evaluate every statement into a string, and join them with newlines
}

pub struct Evaler {
    universe: Rc<RefCell<Universe>>,
    // configure whether to evaluate a node immediately or lazily
    tempo_for_nodes: HashMap<AutoStr, EvalTempo>,
    // evaluation mode
    mode: EvalMode,
}

impl Evaler {
    pub fn new(universe: Rc<RefCell<Universe>>) -> Self {
        Evaler {
            universe,
            tempo_for_nodes: HashMap::new(),
            mode: EvalMode::SCRIPT,
        }
    }

    pub fn with_mode(mut self, mode: EvalMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_mode(&mut self, mode: EvalMode) {
        self.mode = mode;
    }

    pub fn set_tempo(&mut self, name: &str, tempo: EvalTempo) {
        self.tempo_for_nodes.insert(name.into(), tempo);
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
                let mut node = auto_val::Node::new("root");
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt);
                    match val {
                        Value::Pair(key, value) => {
                            // first level pairs are viewed as variable declarations
                            // TODO: this should only happen in a Config scenario
                            let mut value = *value;
                            if let Some(name) = key.name() {
                                let mut scope = self.universe.borrow_mut();
                                if scope.has_arg(name) {
                                    let arg_val = scope.get_arg(name);
                                    // println!(
                                    // "replacing value of {} from {} to {}",
                                    // name, value, arg_val
                                    // );
                                    value = arg_val;
                                }
                                scope.set_local_val(name, value.clone());
                            }
                            node.set_prop(key, value);
                        }
                        Value::Obj(o) => {
                            node.merge_obj(o);
                        }
                        Value::Node(n) => {
                            node.add_kid(n);
                        }
                        Value::Array(arr) => {
                            for item in arr.values.into_iter() {
                                match item {
                                    Value::ConfigBody(body) => {
                                        for item in body.items.into_iter() {
                                            match item {
                                                ConfigItem::Pair(pair) => {
                                                    node.set_prop(pair.key, pair.value);
                                                }
                                                ConfigItem::Object(o) => {
                                                    node.merge_obj(o);
                                                }
                                                ConfigItem::Node(n) => {
                                                    node.add_kid(n.clone());
                                                }
                                                ConfigItem::Value(v) => {
                                                    node.set_prop(v.to_astr(), v);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Value::ConfigBody(body) => {
                            for item in body.items.into_iter() {
                                match item {
                                    ConfigItem::Pair(pair) => {
                                        node.set_prop(pair.key, pair.value);
                                    }
                                    ConfigItem::Object(o) => {
                                        node.merge_obj(o);
                                    }
                                    ConfigItem::Node(n) => {
                                        node.add_kid(n.clone());
                                    }
                                    ConfigItem::Value(v) => {
                                        node.set_prop(v.to_astr(), v);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Value::Node(node)
            }
            EvalMode::TEMPLATE => {
                let mut result = Vec::new();
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt);
                    if !val.is_nil() {
                        result.push(val.to_astr());
                    }
                }
                Value::Str(result.join("\n").into())
            }
        }
    }

    pub fn dump_scope(&self) {
        self.universe.borrow().dump();
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::Use(use_stmt) => self.eval_use(use_stmt),
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Stmt::For(for_stmt) => self.eval_for(for_stmt),
            Stmt::Block(body) => self.eval_body(body),
            Stmt::Store(store) => self.eval_store(store),
            Stmt::Fn(_) => Value::Nil,
            Stmt::TypeDecl(type_decl) => self.type_decl(type_decl),
            Stmt::Widget(widget) => self.eval_widget(widget),
            Stmt::Node(node) => self.eval_node(node),
            Stmt::When(_) => Value::Nil, // TODO: implement
            Stmt::EnumDecl(_) => Value::Nil,
            Stmt::Comment(_) => Value::Nil,
        }
    }

    fn eval_use(&mut self, use_: &Use) -> Value {
        println!("Got use {}", use_);
        Value::Int(25)
    }

    fn collect_config_body(&mut self, vals: Vec<Value>) -> ConfigBody {
        let mut body = ConfigBody::new();
        for val in vals.into_iter() {
            match val {
                Value::Pair(key, value) => body.add_pair(Pair::new(key, *value)),
                Value::Obj(o) => body.add_object(o),
                Value::Node(n) => body.add_node(n),
                _ => body.add_val(val),
            }
        }
        body
    }

    fn eval_body(&mut self, body: &Body) -> Value {
        self.enter_scope();
        let mut res = Vec::new();
        for stmt in body.stmts.iter() {
            res.push(self.eval_stmt(stmt));
        }
        let res = match self.mode {
            EvalMode::SCRIPT => res.last().unwrap_or(&Value::Nil).clone(),
            EvalMode::CONFIG => Value::ConfigBody(self.collect_config_body(res)),
            EvalMode::TEMPLATE => Value::Str(
                res.iter()
                    .map(|v| match v {
                        Value::Str(s) => s.clone(),
                        _ => v.to_astr(),
                    })
                    .collect::<Vec<AutoStr>>()
                    .join("\n")
                    .into(),
            ),
        };
        self.exit_scope();
        res
    }

    fn eval_loop_body(&mut self, body: &Body, is_mid: bool, is_new_line: bool) -> Value {
        self.universe
            .borrow_mut()
            .set_local_val("is_mid", Value::Bool(is_mid));
        let mut res = Vec::new();
        let sep = if is_new_line { "\n" } else { "" };
        for stmt in body.stmts.iter() {
            res.push(self.eval_stmt(stmt));
        }
        match self.mode {
            EvalMode::SCRIPT => res.last().unwrap_or(&Value::Nil).clone(),
            EvalMode::CONFIG => Value::ConfigBody(self.collect_config_body(res)),
            EvalMode::TEMPLATE => Value::Str(
                res.into_iter()
                    .filter(|v| !v.is_nil())
                    .map(|v| match v {
                        Value::Str(s) => s.clone(),
                        _ => v.to_astr(),
                    })
                    .collect::<Vec<AutoStr>>()
                    .join(sep)
                    .into(),
            ),
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
        Value::Void
    }

    fn eval_iter(&mut self, iter: &Iter, idx: usize, item: Value) {
        match iter {
            Iter::Indexed(index, iter) => {
                self.universe
                    .borrow_mut()
                    .set_local_val(&index, Value::Int(idx as i32));
                // println!("set index {}, iter: {}, item: {}", index.text, iter.text, item.clone());
                self.universe.borrow_mut().set_local_val(&iter, item);
            }
            Iter::Named(iter) => self.universe.borrow_mut().set_local_val(&iter, item),
        }
    }

    fn eval_for(&mut self, for_stmt: &For) -> Value {
        let iter = &for_stmt.iter;
        let body = &for_stmt.body;
        let mut max_loop = 1000;
        let range = self.eval_expr(&for_stmt.range);
        let mut res = Array::new();
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
                    let s = self.eval_loop_body(body, is_mid, is_new_line);
                    res.push(s);
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
                return Value::error(format!("Invalid range {}", range));
            }
        }
        self.universe.borrow_mut().exit_scope();
        if max_loop <= 0 {
            return Value::error("Max loop reached");
        } else {
            let result = match self.mode {
                EvalMode::SCRIPT => Value::Void,
                EvalMode::CONFIG => Value::Array(res),
                EvalMode::TEMPLATE => Value::Str(
                    res.iter()
                        .filter(|v| match v {
                            Value::Nil => false,
                            Value::Str(s) => !s.is_empty(),
                            _ => true,
                        })
                        .map(|v| v.to_astr())
                        .collect::<Vec<AutoStr>>()
                        .join(sep)
                        .into(),
                ),
            };
            result
        }
    }

    fn eval_store(&mut self, store: &Store) -> Value {
        let mut value = match &store.expr {
            Expr::Ref(target) => Value::Ref(target.clone().into()),
            _ => self.eval_expr(&store.expr),
        };
        // TODO: add general type coercion in assignment
        // int -> byte
        if matches!(store.ty, ast::Type::Byte) && matches!(value, Value::Int(_)) {
            value = Value::Byte(value.as_int() as u8);
        }
        self.universe.borrow_mut().define(
            store.name.as_str(),
            Rc::new(scope::Meta::Store(store.clone())),
        );
        self.universe.borrow_mut().set_local_val(&store.name, value);
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
            Op::Asn => self.eval_asn(left, right_value),
            Op::Range => self.range(left, right),
            Op::RangeEq => self.range_eq(left, right),
            Op::Dot => self.dot(left, right),
            _ => Value::Nil,
        }
    }

    fn eval_asn(&mut self, left: &Expr, val: Value) -> Value {
        match left {
            Expr::Ident(name) => {
                // check ref
                let left_val = self.lookup(&name);
                match left_val {
                    Value::Ref(target) => {
                        println!("ref: {}", target);
                        if self.universe.borrow().exists(&target) {
                            self.universe.borrow_mut().update_val(&target, val);
                        } else {
                            panic!(
                                "Invalid assignment, variable (ref {} -> {}) not found",
                                name, target
                            );
                        }
                    }
                    _ => {
                        if self.universe.borrow().exists(&name) {
                            self.universe.borrow_mut().update_val(&name, val);
                        } else {
                            panic!("Invalid assignment, variable {} not found", name);
                        }
                    }
                }
                Value::Void
            }
            Expr::Bina(left, op, right) => {
                match op {
                    Op::Dot => {
                        // a.b = expr
                        match left.as_ref() {
                            Expr::Ident(name) => {
                                // find object `left`
                                self.update_obj(&name, move |o| match right.as_ref() {
                                    Expr::Ident(rname) => o.set(rname.clone(), val),
                                    _ => {}
                                });
                                Value::Void
                            }
                            _ => Value::error(format!("Invalid assignment {}", left)),
                        }
                    }
                    _ => Value::error(format!("Invalid bina target of asn {} = {}", left, val)),
                }
            }
            Expr::Index(array, index) => match array.as_ref() {
                Expr::Ident(name) => {
                    let idx = self.eval_expr(index);
                    self.update_array(&name, idx, val);
                    Value::Void
                }
                _ => Value::error(format!("Invalid target of asn index {} = {}", left, val)),
            },
            _ => Value::error(format!("Invalid target of asn {} = {}", left, val)),
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
            _ => Value::error(format!("Invalid range {}..{}", left_value, right_value)),
        }
    }

    fn range_eq(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);
        match (&left_value, &right_value) {
            (Value::Int(left), Value::Int(right)) => Value::RangeEq(*left, *right),
            _ => Value::error(format!("Invalid range {}..={}", left_value, right_value)),
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
        self.universe
            .borrow()
            .lookup_val(name)
            .unwrap_or(Value::Nil)
    }

    fn eval_array(&mut self, elems: &Vec<Expr>) -> Value {
        let mut values = Array::new();
        for elem in elems.iter() {
            let v = self.eval_expr(elem);
            if !v.is_void() {
                if let Value::ConfigBody(b) = v {
                    // merge values
                    for i in b.items {
                        match i {
                            ConfigItem::Node(n) => values.push(n),
                            ConfigItem::Pair(p) => values.push(Value::pair(p.key, p.value)),
                            ConfigItem::Object(o) => values.push(o),
                            ConfigItem::Value(v) => values.push(v),
                        }
                    }
                } else {
                    values.push(self.eval_expr(elem));
                }
            }
        }
        Value::array(values)
    }

    fn object(&mut self, pairs: &Vec<ast::Pair>) -> Value {
        let mut obj = Obj::new();
        for pair in pairs.iter() {
            obj.set(self.eval_key(&pair.key), self.eval_expr(&pair.value));
        }
        Value::Obj(obj)
    }

    fn pair(&mut self, pair: &ast::Pair) -> Value {
        let key = self.eval_key(&pair.key);
        let value = self.eval_expr(&pair.value);
        Value::Pair(key, Box::new(value))
    }

    fn eval_key(&self, key: &Key) -> ValueKey {
        match key {
            Key::NamedKey(name) => ValueKey::Str(name.clone().into()),
            Key::IntKey(value) => ValueKey::Int(*value),
            Key::BoolKey(value) => ValueKey::Bool(*value),
            Key::StrKey(value) => ValueKey::Str(value.clone().into()),
        }
    }

    // TODO: 需要整理一下，逻辑比较乱
    fn eval_call(&mut self, call: &Call) -> Value {
        let name = self.eval_expr(&call.name);
        if name == Value::Nil {
            return Value::error(format!("Invalid function name to call {}", call.name));
        }

        match name {
            Value::Meta(meta_id) => match meta_id {
                MetaID::Fn(sig) => {
                    return self.eval_fn_call_with_sig(&sig, &call.args);
                }
                MetaID::Type(name) => {
                    return self.eval_type_new(&name, &call.args);
                }
                _ => {
                    println!("Strange function call {}", meta_id);
                }
            },
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
                            return Value::error(format!("Invalid lambda {}", name));
                        }
                    }
                } else {
                    return Value::error(format!("Invalid lambda {}", name));
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
                return Value::error(format!("Invalid function call {}", name));
            }
        }

        // Lookup Fn meta
        let meta = self.universe.borrow().lookup_meta(&call.get_name_text());
        if let Some(meta) = meta {
            match meta.as_ref() {
                scope::Meta::Fn(fn_decl) => {
                    return self.eval_fn_call(fn_decl, &call.args);
                }
                _ => return Value::error(format!("Invalid lambda {}", call.get_name_text())),
            }
        } else {
            // convert call to node intance
            println!("call {} not found, try to eval node", call.get_name_text());
            let node: Node = call.clone().into();
            return self.eval_node(&node);
        }
    }

    pub fn eval_type_new(&mut self, name: &str, args: &Args) -> Value {
        let meta = self.universe.borrow().lookup_meta(name);
        if let Some(meta) = meta {
            match meta.as_ref() {
                scope::Meta::Type(ty) => match ty {
                    ast::Type::User(type_decl) => {
                        let instance = self.eval_instance(type_decl, args);
                        return instance;
                    }
                    _ => Value::error(format!("Invalid type instance of {}", name)),
                },
                _ => Value::error(format!("Invalid type {}", name)),
            }
        } else {
            return Value::error(format!("Invalid type {}", name));
        }
    }

    fn eval_instance(&mut self, type_decl: &TypeDecl, args: &Args) -> Value {
        let ty = self.eval_type(&type_decl);
        let fields = self.eval_fields(&type_decl, args);
        Value::Instance(auto_val::Instance { ty, fields })
    }

    fn eval_type(&mut self, type_decl: &TypeDecl) -> Type {
        Type::User(type_decl.name.clone())
    }

    fn eval_fields(&mut self, type_decl: &TypeDecl, args: &Args) -> Obj {
        let members = &type_decl.members;
        // TODO: remove unnecessary clone
        let mut fields = Obj::new();
        for (j, arg) in args.args.iter().enumerate() {
            let val_arg = self.eval_arg(arg);
            match val_arg {
                auto_val::Arg::Pair(key, val) => {
                    for member in members.iter() {
                        if key.to_string() == member.name {
                            fields.set(member.name.clone(), val.clone());
                        }
                    }
                }
                auto_val::Arg::Pos(value) => {
                    if j < members.len() {
                        let member = &members[j];
                        fields.set(member.name.clone(), value);
                    }
                }
                auto_val::Arg::Name(name) => {
                    for member in members.iter() {
                        if name == member.name {
                            fields.set(member.name.clone(), Value::Str(name.clone()));
                        }
                    }
                }
            }
        }
        // check default field values
        for member in members.iter() {
            match &member.value {
                Some(value) => {
                    if fields.has(member.name.clone()) {
                        continue;
                    }
                    fields.set(member.name.clone(), self.eval_expr(value));
                }
                None => {}
            }
        }
        fields
    }

    pub fn eval_method(&mut self, method: &Method, args: &Args) -> Value {
        let target = &method.target;
        let name = &method.name;
        // methods for Any
        match target.as_ref() {
            Value::Str(s) => {
                let method = self
                    .universe
                    .borrow()
                    .types
                    .lookup_method(Type::Str, name.clone());
                if let Some(method) = method {
                    return method(&target);
                } else {
                    println!("wrong method?: {}", s);
                }
            }
            Value::Instance(inst) => {
                let method = self.universe.borrow().lookup_meta(&method.name);
                if let Some(meta) = method {
                    match meta.as_ref() {
                        Meta::Fn(fn_decl) => {
                            self.enter_scope();
                            self.universe.borrow_mut().set_local_obj(&inst.fields);
                            let res = self.eval_fn_call(fn_decl, args);
                            self.exit_scope();
                            return res;
                        }
                        _ => {
                            return Value::error(format!("wrong meta for method: {}", meta));
                        }
                    }
                }
            }
            _ => {
                let method = self
                    .universe
                    .borrow()
                    .types
                    .lookup_method(Type::Any, name.clone());
                if let Some(method) = method {
                    return method(&target);
                }
            }
        }
        Value::error(format!("Invalid method {} on {}", name, target))
    }

    fn eval_fn_call_with_sig(&mut self, sig: &Sig, args: &Args) -> Value {
        let meta = self.universe.borrow().lookup_sig(sig).unwrap();
        match meta.as_ref() {
            scope::Meta::Fn(fn_decl) => self.eval_fn_call(fn_decl, args),
            _ => Value::error(format!("Invalid function call {}", sig.name)),
        }
    }

    #[inline]
    fn enter_scope(&mut self) {
        self.universe.borrow_mut().enter_scope();
    }

    #[inline]
    fn exit_scope(&mut self) {
        self.universe.borrow_mut().exit_scope();
    }

    fn eval_fn_arg(&mut self, arg: &Arg, i: usize, params: &Vec<Param>) {
        match arg {
            Arg::Pair(name, expr) => {
                let val = self.eval_expr(expr);
                let name = &name;
                self.universe.borrow_mut().set_local_val(&name, val.clone());
            }
            Arg::Pos(expr) => {
                let val = self.eval_expr(expr);
                let name = &params[i].name;
                self.universe.borrow_mut().set_local_val(&name, val.clone());
            }
            Arg::Name(name) => {
                self.universe
                    .borrow_mut()
                    .set_local_val(name.as_str(), Value::Str(name.clone()));
            }
        }
    }

    pub fn eval_fn_call(&mut self, fn_decl: &Fn, args: &Args) -> Value {
        // TODO: 需不需要一个单独的 enter_call()
        self.enter_scope();
        println!(
            "enter call scope {}",
            self.universe.borrow().current_scope().sid
        );
        for (i, arg) in args.args.iter().enumerate() {
            self.eval_fn_arg(arg, i, &fn_decl.params);
        }
        let result = self.eval_body(&fn_decl.body);
        self.exit_scope();
        result
    }

    fn index(&mut self, array: &Expr, index: &Expr) -> Value {
        let array = self.eval_expr(array);
        let index_value = self.eval_expr(index);
        let mut idx = match index_value {
            Value::Int(index) => index,
            // TODO: support negative index
            // TODO: support range index
            _ => return Value::error(format!("Invalid index {}", index_value)),
        };
        match array {
            Value::Array(values) => {
                let len = values.len();
                if idx >= len as i32 {
                    return Value::error(format!("Index out of bounds {}", idx));
                }
                if idx < -(len as i32) {
                    return Value::error(format!("Index out of bounds {}", idx));
                }
                if idx < 0 {
                    idx = len as i32 + idx;
                }
                values[idx as usize].clone()
            }
            Value::Str(s) => {
                let idx = idx as usize;
                if idx >= s.len() {
                    return Value::error(format!("Index out of bounds {}", idx));
                }
                Value::Char(s.chars().nth(idx).unwrap())
            }
            _ => Value::error(format!("Invalid array {}", array)),
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Byte(value) => Value::Byte(*value),
            Expr::Uint(value) => Value::Uint(*value),
            Expr::Int(value) => Value::Int(*value),
            Expr::Float(value) => Value::Float(*value),
            // Why not move here?
            Expr::Char(value) => Value::Char(*value),
            Expr::Str(value) => Value::Str(value.clone().into()),
            Expr::Bool(value) => Value::Bool(*value),
            Expr::Ref(target) => {
                let target_val = self.eval_expr(&Expr::Ident(target.clone()));
                target_val
            }
            Expr::Ident(name) => {
                let res = self.lookup(&name);
                match res {
                    Value::Ref(target) => {
                        let target_val = self.eval_expr(&Expr::Ident(target));
                        target_val
                    }
                    Value::Nil => {
                        // try to lookup in meta and builtins
                        let meta = self.universe.borrow().lookup_meta(&name);
                        if let Some(meta) = meta {
                            return Value::Meta(to_meta_id(&meta));
                        }
                        // Try builtin
                        self.universe
                            .borrow()
                            .lookup_builtin(&name)
                            .unwrap_or(Value::Nil)
                    }
                    _ => res,
                }
            }
            Expr::Unary(op, e) => self.eval_una(op, e),
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::If(branches, else_stmt) => self.eval_if(branches, else_stmt),
            Expr::Array(elems) => self.eval_array(elems),
            Expr::Call(call) => self.eval_call(call),
            Expr::Node(node) => self.eval_node(node),
            Expr::Index(array, index) => self.index(array, index),
            Expr::Pair(pair) => self.pair(pair),
            Expr::Object(pairs) => self.object(pairs),
            Expr::Block(body) => self.eval_body(body),
            Expr::Lambda(lambda) => Value::Lambda(lambda.name.clone().into()),
            Expr::FStr(fstr) => self.fstr(fstr),
            Expr::Grid(grid) => self.grid(grid),
            Expr::Nil => Value::Nil,
        }
    }

    fn type_decl(&mut self, _type_decl: &TypeDecl) -> Value {
        Value::Void
    }

    fn dot_node(&mut self, node: &auto_val::Node, right: &Expr) -> Option<Value> {
        let Expr::Ident(name) = right else {
            return None;
        };
        if name == "name" {
            return Some(Value::Str(node.name.clone()));
        }
        let mut name = name.clone();
        // 1. lookup in the props
        let v = node.get_prop(&name);
        if v.is_nil() {
            // 2.1 check if nodes with the name exists
            let nodes = node.get_nodes(&name);
            if nodes.len() > 0 {
                return Some(Value::array_of(nodes.iter().map(|n| n.clone()).collect()));
            }
            // 2.2 lookup in sub nodes
            if name.ends_with("s") {
                name = name[..name.len() - 1].into();
            }
            let nodes = node.get_nodes(&name);
            if nodes.len() > 0 {
                Some(Value::array_of(nodes.iter().map(|n| n.clone()).collect()))
            } else {
                None
            }
        } else {
            Some(v)
        }
    }

    fn dot(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let res: Option<Value> = match &left_value {
            Value::Meta(meta_id) => {
                // lookup meta
                match meta_id {
                    MetaID::Enum(name) => {
                        let right_name = right.repr();
                        self.enum_val(name, &AutoStr::from(right_name))
                    }
                    _ => None,
                }
            }
            Value::Obj(obj) => match right {
                Expr::Ident(name) => obj.lookup(&name),
                Expr::Int(key) => obj.lookup(&key.to_string()),
                Expr::Bool(key) => obj.lookup(&key.to_string()),
                _ => None,
            },
            Value::Node(node) => self.dot_node(node, right),
            Value::Widget(widget) => match right {
                Expr::Ident(name) => match name.as_str() {
                    "model" => Some(Value::Model(widget.model.clone())),
                    "view" => Some(Value::Meta(widget.view_id.clone())),
                    _ => None,
                },
                _ => None,
            },
            Value::Model(model) => match right {
                Expr::Ident(name) => model.find(&name),
                _ => None,
            },
            Value::View(view) => match right {
                Expr::Ident(name) => view.find(&name),
                _ => None,
            },
            Value::Instance(instance) => match right {
                Expr::Ident(name) => {
                    let f = instance.fields.lookup(&name);
                    match f {
                        Some(v) => Some(v),
                        None => {
                            // not a field, try method
                            let typ = instance.ty.name();
                            let combined_name: AutoStr = format!("{}::{}", typ, name).into();
                            let method = self.universe.borrow().lookup_meta(&combined_name);
                            if let Some(meta) = method {
                                match meta.as_ref() {
                                    scope::Meta::Fn(_) => Some(Value::Method(Method::new(
                                        left_value.clone(),
                                        combined_name,
                                    ))),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },

            _ => {
                // try to lookup method
                match right {
                    Expr::Ident(name) => {
                        // TODO: too long
                        if self
                            .universe
                            .borrow()
                            .types
                            .lookup_method_for_value(&left_value, name.clone())
                            .is_some()
                        {
                            Some(Value::Method(Method::new(left_value.clone(), name.clone())))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        };
        res.unwrap_or(Value::error(format!(
            "Invalid dot expression {}.{}",
            left_value.name(),
            right
        )))
    }

    fn enum_val(&mut self, enum_name: &AutoStr, item_name: &AutoStr) -> Option<Value> {
        let meta = self.universe.borrow().lookup_meta(enum_name);
        if let Some(meta) = meta {
            match meta.as_ref() {
                Meta::Enum(enum_meta) => {
                    let item = enum_meta.get_item(item_name);
                    match item {
                        Some(item) => Some(Value::Int(item.value.clone())),
                        None => None,
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn eval_widget(&mut self, widget: &Widget) -> Value {
        let name = &widget.name;
        // model
        let mut vars = Vec::new();
        for var in widget.model.vars.iter() {
            let value = self.eval_expr(&var.expr);
            vars.push((ValueKey::Str(var.name.clone().into()), value.clone()));
            self.universe.borrow_mut().set_local_val(&var.name, value);
        }
        let model = auto_val::Model { values: vars };
        // view
        let view_id = format!("{}.view", name).into();
        self.universe
            .borrow_mut()
            .define(&view_id, Rc::new(Meta::View(widget.view.clone())));
        let widget_value = auto_val::Widget {
            name: name.clone(),
            model,
            view_id: MetaID::View(view_id),
        };
        let value = Value::Widget(widget_value);
        self.universe
            .borrow_mut()
            .set_local_val(name, value.clone());
        self.universe.borrow_mut().widget = value.clone();
        value
    }

    fn eval_mid(&mut self, node: &Node) -> Value {
        let is_mid = self
            .universe
            .borrow()
            .lookup_val("is_mid")
            .unwrap_or(Value::Bool(false))
            .as_bool();
        let args = &node.args.args;
        let mut res = Value::Str("".into());
        if args.len() >= 1 {
            if is_mid {
                // mid
                let mid = self.eval_expr(&args[0].get_expr());
                res = mid;
            }
        }
        if args.len() >= 2 {
            if !is_mid {
                // last
                let last = self.eval_expr(&args[1].get_expr());
                res = last;
            }
        }
        if is_mid && node.body.stmts.len() != 0 {
            for stmt in node.body.stmts.iter() {
                let val = self.eval_stmt(stmt);
                res = val;
            }
        }
        res
    }

    fn eval_arg(&mut self, arg: &ast::Arg) -> auto_val::Arg {
        match arg {
            ast::Arg::Name(name) => auto_val::Arg::Name(name.clone().into()),
            ast::Arg::Pair(name, expr) => {
                auto_val::Arg::Pair(ValueKey::Str(name.clone().into()), self.eval_expr(expr))
            }
            ast::Arg::Pos(expr) => auto_val::Arg::Pos(self.eval_expr(expr)),
        }
    }

    fn eval_args(&mut self, args: &ast::Args) -> auto_val::Args {
        let mut res = auto_val::Args::new();
        for arg in args.args.iter() {
            let val = self.eval_arg(arg);
            res.args.push(val);
        }
        res
    }

    // TODO: should node only be used in config mode?
    fn eval_node(&mut self, node: &Node) -> Value {
        let args = self.eval_args(&node.args);
        let mut nodes = Vec::new();
        let mut props = Obj::new();
        let mut body = MetaID::Nil;
        let name = &node.name;
        if name == "mid" {
            return self.eval_mid(&node);
        }
        let name: AutoStr = name.into();
        let tempo = self
            .tempo_for_nodes
            .get(&name)
            .unwrap_or(&EvalTempo::IMMEDIATE);

        match tempo {
            EvalTempo::IMMEDIATE => {
                // eval each stmts in body and extract props and sub nodes
                self.enter_scope();
                // put args as local values
                for arg in args.args.iter() {
                    match arg {
                        auto_val::Arg::Pair(name, value) => {
                            self.universe
                                .borrow_mut()
                                .set_local_val(&name.to_string().as_str(), value.clone());
                        }
                        _ => {}
                    }
                }
                for stmt in node.body.stmts.iter() {
                    let val = self.eval_stmt(stmt);
                    match val {
                        Value::Str(s) => {
                            let mut n = auto_val::Node::new("text");
                            n.text = s.clone();
                            nodes.push(n);
                        }
                        Value::Pair(key, value) => {
                            self.universe
                                .borrow_mut()
                                .set_local_val(&key.to_string(), *value.clone());
                            props.set(key, *value);
                        }
                        Value::Node(node) => {
                            nodes.push(node);
                        }
                        Value::Array(arr) => {
                            for item in arr.values.into_iter() {
                                match item {
                                    Value::ConfigBody(body) => {
                                        for item in body.items.into_iter() {
                                            match item {
                                                ConfigItem::Pair(pair) => {
                                                    props.set(pair.key, pair.value);
                                                }
                                                ConfigItem::Object(o) => {
                                                    props.merge(&o);
                                                }
                                                ConfigItem::Node(n) => {
                                                    nodes.push(n.clone());
                                                }
                                                ConfigItem::Value(v) => {
                                                    props.set(v.to_astr(), v);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Value::ConfigBody(body) => {
                            for item in body.items.into_iter() {
                                match item {
                                    ConfigItem::Pair(pair) => {
                                        props.set(pair.key, pair.value);
                                    }
                                    ConfigItem::Object(o) => {
                                        props.merge(&o);
                                    }
                                    ConfigItem::Node(n) => {
                                        nodes.push(n.clone());
                                    }
                                    ConfigItem::Value(v) => {
                                        props.set(v.to_astr(), v);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                self.exit_scope();
            }
            EvalTempo::LAZY => {
                // push node body to scope meta
                // TODO: support multiple nodes of same name
                body = MetaID::Body(name.clone().into());
                println!("define global {}", name);
                self.universe
                    .borrow_mut()
                    .define_global(&name, Rc::new(Meta::Body(node.body.clone())));
            }
        }
        Value::Node(auto_val::Node {
            name: node.name.clone().into(),
            args,
            props,
            text: AutoStr::new(),
            nodes,
            body,
        })
    }

    // fn eval_value_node_body(&mut self, node_val: &mut Value) {
    //     self.universe.borrow_mut().enter_scope();
    //     match node_val {
    //         Value::Node(ref mut node) => {
    //             let props = &mut node.props;
    //             let nodes = &mut node.nodes;
    //             let mut stmts = Vec::new();
    //             {
    //                 let scope = self.universe.borrow();
    //                 let meta = scope.lookup_meta(&node.name);
    //                 stmts = meta.map(|m| {
    //                     match m.as_ref() {
    //                         scope::Meta::Body(body) => body.stmts.clone(),
    //                         _ => Vec::new(),
    //                     }
    //                 }).unwrap();
    //             }
    //             for stmt in stmts.iter() {
    //                 let val = self.eval_stmt(stmt);
    //                 match val {
    //                     Value::Node(node) => {nodes.push(node);},
    //                     Value::Pair(key, value) => {props.set(key, *value);},
    //                     _ => {},
    //                 }
    //             }
    //         },
    //         _ => {},
    //     };
    //     self.universe.borrow_mut().exit_scope();
    // }

    fn fstr(&mut self, fstr: &FStr) -> Value {
        let parts: Vec<AutoStr> = fstr
            .parts
            .iter()
            .map(|part| {
                let val = self.eval_expr(part);
                match val {
                    Value::Str(s) => s,
                    _ => val.to_astr(),
                }
            })
            .collect();
        Value::Str(parts.join("").into())
    }

    fn grid(&mut self, grid: &Grid) -> Value {
        // head
        let mut head = Vec::new();
        let mut data = Vec::new();
        if grid.head.len() == 1 {
            let expr = &grid.head.args[0].get_expr();
            match expr {
                Expr::Array(array) => {
                    for elem in array.iter() {
                        if let Expr::Object(pairs) = elem {
                            for p in pairs.iter() {
                                match p.key.to_string().as_str() {
                                    "id" => {
                                        let id = self.eval_expr(&p.value);
                                        head.push((ValueKey::Str("id".to_string().into()), id));
                                    }
                                    k => {
                                        head.push((
                                            ValueKey::Str(k.to_string().into()),
                                            self.eval_expr(&p.value),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                Expr::Ident(_) => {
                    let val = self.eval_expr(expr);
                    if let Value::Array(array) = val {
                        for elem in array.into_iter() {
                            if let Value::Obj(obj) = &elem {
                                let id = obj.get_str("id");
                                match id {
                                    Some(id) => {
                                        head.push((ValueKey::Str(id.to_string().into()), elem));
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if head.len() == 0 {
            for arg in grid.head.args.iter() {
                match arg {
                    Arg::Pair(name, value) => {
                        head.push((ValueKey::Str(name.clone().into()), self.eval_expr(value)));
                    }
                    Arg::Pos(value) => match value {
                        Expr::Str(value) => {
                            head.push((
                                ValueKey::Str(value.clone().into()),
                                Value::Str(value.clone().into()),
                            ));
                        }
                        _ => {}
                    },
                    Arg::Name(name) => {
                        head.push((
                            ValueKey::Str(name.clone().into()),
                            Value::Str(name.clone().into()),
                        ));
                    }
                }
            }
        }
        for row in grid.data.iter() {
            let row_data = row.iter().map(|elem| self.eval_expr(elem)).collect();
            data.push(row_data);
        }
        Value::Grid(auto_val::Grid { head, data })
    }
}

fn to_meta_id(meta: &Rc<scope::Meta>) -> MetaID {
    match meta.as_ref() {
        scope::Meta::Fn(fn_decl) => MetaID::Fn(to_value_sig(&fn_decl)),
        scope::Meta::Type(type_decl) => MetaID::Type(type_decl.unique_name().into()),
        scope::Meta::Enum(enum_decl) => MetaID::Enum(enum_decl.unique_name()),
        _ => MetaID::Nil,
    }
}

fn to_value_sig(fn_decl: &Fn) -> Sig {
    let mut params = Vec::new();
    for param in fn_decl.params.iter() {
        params.push(auto_val::Param {
            name: param.name.clone().into(),
            ty: Box::new(to_value_type(&param.ty)),
        });
    }
    let ret = to_value_type(&fn_decl.ret);
    Sig {
        name: fn_decl.name.clone().into(),
        params,
        ret,
    }
}

fn to_value_type(ty: &ast::Type) -> auto_val::Type {
    match ty {
        ast::Type::Byte => auto_val::Type::Byte,
        ast::Type::Int => auto_val::Type::Int,
        ast::Type::Float => auto_val::Type::Float,
        ast::Type::Bool => auto_val::Type::Bool,
        ast::Type::Char => auto_val::Type::Char,
        ast::Type::Str => auto_val::Type::Str,
        ast::Type::Array(_) => auto_val::Type::Array,
        ast::Type::User(type_decl) => auto_val::Type::User(type_decl.name.clone()),
        ast::Type::Void => auto_val::Type::Void,
        ast::Type::Unknown => auto_val::Type::Any,
    }
}

pub fn eval_basic_expr(expr: &Expr) -> Value {
    match expr {
        Expr::Str(s) => Value::Str(s.clone().into()),
        Expr::Byte(b) => Value::Byte(*b),
        Expr::Int(i) => Value::Int(*i),
        Expr::Float(f) => Value::Float(*f),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Char(c) => Value::Char(*c),
        _ => Value::error(format!("Unsupported basic expression: {:?}", expr)),
    }
}
