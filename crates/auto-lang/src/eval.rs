use crate::ast;
use crate::ast::*;
use crate::error::AutoResult;
use crate::scope;
use crate::scope::Meta;
use crate::universe::Universe;
use auto_val;
use auto_val::{add, comp, div, mul, sub};
use auto_val::{Array, AutoStr, MetaID, Method, Obj, Op, Sig, Type, Value, ValueData, ValueKey};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
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
    // skip_check
    skip_check: bool,
}

impl Evaler {
    pub fn new(universe: Rc<RefCell<Universe>>) -> Self {
        Evaler {
            universe,
            tempo_for_nodes: HashMap::new(),
            mode: EvalMode::SCRIPT,
            skip_check: false,
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

    pub fn skip_check(&mut self) {
        self.skip_check = true;
    }

    pub fn eval(&mut self, code: &Code) -> AutoResult<Value> {
        match self.mode {
            EvalMode::SCRIPT => {
                let mut value = Value::Nil;
                for stmt in code.stmts.iter() {
                    value = self.eval_stmt(stmt)?;
                    // Don't panic on errors - let them propagate as error values
                    // This allows tests to check for errors using Result::Err
                }
                Ok(value)
            }
            EvalMode::CONFIG => {
                if code.stmts.len() == 1 {
                    let first_val = self.eval_stmt(&code.stmts[0])?;
                    // For Array, we need to process it to consolidate nodes
                    if matches!(first_val, Value::Array(_)) {
                        // Process the array using the same logic as multi-statement case
                        match first_val {
                            Value::Array(arr) => {
                                use std::collections::HashMap;
                                use auto_val::Array;

                                // First, check if array contains any config items (nodes, pairs, objs, instances)
                                // If not, return the array as-is (pure value array)
                                let has_config_items = arr.values.iter().any(|item| {
                                    matches!(item,
                                        Value::Node(_) |
                                        Value::Pair(_, _) |
                                        Value::Obj(_) |
                                        Value::Instance(_)
                                    )
                                });

                                if !has_config_items {
                                    return Ok(Value::Array(arr));
                                }

                                // Has config items, need to consolidate
                                let mut nodes_by_name: HashMap<AutoStr, Vec<auto_val::Node>> = HashMap::new();
                                let mut other_items: Vec<Value> = Vec::new();

                                // First pass: separate nodes from other items
                                for item in arr.values.into_iter() {
                                    match item {
                                        Value::Node(n) => {
                                            nodes_by_name.entry(n.name.clone()).or_default().push(n);
                                        }
                                        Value::Pair(key, value) => {
                                            let mut node = auto_val::Node::new("root");
                                            node.set_prop(key, *value);
                                            return Ok(Value::Node(node));
                                        }
                                        Value::Obj(o) => {
                                            let mut node = auto_val::Node::new("root");
                                            node.merge_obj(o);
                                            return Ok(Value::Node(node));
                                        }
                                        Value::Instance(inst) => {
                                            // Convert instance to node with type name as node name
                                            let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                            // Add instance fields as node properties
                                            for (k, v) in inst.fields.iter() {
                                                kid_node.set_prop(k.clone(), v.clone());
                                            }
                                            nodes_by_name.entry(kid_node.name.clone()).or_default().push(kid_node);
                                        }
                                        _ => {
                                            if !item.is_void() {
                                                other_items.push(item);
                                            }
                                        }
                                    }
                                }

                                // Consolidate into a root node
                                let mut node = auto_val::Node::new("root");

                                // Second pass: add consolidated nodes
                                for (name, nodes) in nodes_by_name.into_iter() {
                                    if nodes.len() == 1 {
                                        // Single node: add as kid
                                        node.add_kid(nodes.into_iter().next().unwrap());
                                    } else {
                                        // Multiple nodes with same name: create plural form property
                                        let plural_name = format!("{}s", name); // dir -> dirs
                                        let node_values: Vec<Value> = nodes.into_iter().map(|n| Value::Node(n)).collect();
                                        node.set_prop(plural_name, Value::Array(Array::from_vec(node_values)));
                                    }
                                }

                                // Handle remaining non-node items
                                for item in other_items.into_iter() {
                                    node.set_prop(item.to_astr(), item);
                                }

                                return Ok(Value::Node(node));
                            }
                            _ => unreachable!(),
                        }
                    }
                    match first_val {
                        Value::Obj(_) => {
                            return Ok(first_val);
                        }
                        Value::Node(n) => {
                            let mut node = auto_val::Node::new("root");
                            node.add_kid(n);
                            return Ok(Value::Node(node));
                        }
                        Value::Pair(k, v) => {
                            let mut node = auto_val::Node::new("root");
                            node.set_prop(k, *v);
                            return Ok(Value::Node(node));
                        }
                        _ => {
                            return Err(Error::new(
                                ErrorKind::InvalidInput,
                                "Invalid configuration statement",
                            )
                            .into());
                        }
                    }
                }
                let mut node = auto_val::Node::new("root");
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt)?;
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
                        Value::Block(arr) => {
                            for item in arr.values.into_iter() {
                                match item {
                                    Value::Node(n) => {
                                        node.add_kid(n);
                                    }
                                    Value::Pair(key, value) => {
                                        node.set_prop(key, *value);
                                    }
                                    Value::Obj(o) => {
                                        node.merge_obj(o);
                                    }
                                    Value::Instance(inst) => {
                                        // Convert instance to node with type name as node name
                                        let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                        // Add instance fields as node properties
                                        for (k, v) in inst.fields.iter() {
                                            kid_node.set_prop(k.clone(), v.clone());
                                        }
                                        node.add_kid(kid_node);
                                    }
                                    _ => {
                                    }
                                }
                            }
                        }
                        Value::Array(arr) => {
                            eprintln!("DEBUG: Processing Array with {} items", arr.len());
                            use std::collections::HashMap;
                            use auto_val::{Array, ValueKey};

                            // Group nodes by name for consolidation
                            let mut nodes_by_name: HashMap<AutoStr, Vec<auto_val::Node>> = HashMap::new();
                            let mut other_items: Vec<Value> = Vec::new();

                            // First pass: separate nodes from other items
                            for item in arr.values.into_iter() {
                                eprintln!("DEBUG: Array item type = {}, value = {}",
                                    match &item {
                                        Value::Node(_) => "Node",
                                        Value::Pair(_, _) => "Pair",
                                        Value::Obj(_) => "Obj",
                                        Value::Instance(_) => "Instance",
                                        _ => "Other"
                                    },
                                    item.repr()
                                );
                                match item {
                                    Value::Node(n) => {
                                        nodes_by_name.entry(n.name.clone()).or_default().push(n);
                                    }
                                    Value::Pair(key, value) => {
                                        node.set_prop(key, *value);
                                    }
                                    Value::Obj(o) => {
                                        node.merge_obj(o);
                                    }
                                    Value::Instance(inst) => {
                                        // Convert instance to node with type name as node name
                                        let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                        // Add instance fields as node properties
                                        for (k, v) in inst.fields.iter() {
                                            kid_node.set_prop(k.clone(), v.clone());
                                        }
                                        nodes_by_name.entry(kid_node.name.clone()).or_default().push(kid_node);
                                    }
                                    _ => {
                                        if !item.is_void() {
                                            other_items.push(item);
                                        }
                                    }
                                }
                            }

                            // Second pass: add consolidated nodes
                            for (name, nodes) in nodes_by_name.into_iter() {
                                if nodes.len() == 1 {
                                    // Single node: add as kid
                                    node.add_kid(nodes.into_iter().next().unwrap());
                                } else {
                                    // Multiple nodes with same name: create plural form property
                                    let plural_name = format!("{}s", name); // dir -> dirs
                                    let node_values: Vec<Value> = nodes.into_iter().map(|n| Value::Node(n)).collect();
                                    node.set_prop(plural_name, Value::Array(Array::from_vec(node_values)));
                                }
                            }

                            // Handle remaining non-node items
                            for item in other_items.into_iter() {
                                node.set_prop(item.to_astr(), item);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Value::Node(node))
            }
            EvalMode::TEMPLATE => {
                let mut result = Vec::new();
                for stmt in code.stmts.iter() {
                    let val = self.eval_stmt(stmt)?;
                    if !val.is_nil() {
                        result.push(val.to_astr());
                    }
                }
                Ok(Value::Str(result.join("\n").into()))
            }
        }
    }

    pub fn dump_scope(&self) {
        self.universe.borrow().dump();
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> AutoResult<Value> {
        match stmt {
            Stmt::Use(use_stmt) => Ok(self.eval_use(use_stmt)),
            Stmt::Expr(expr) => Ok(self.eval_expr(expr)),
            Stmt::If(if_) => self.eval_if(if_),
            Stmt::For(for_stmt) => self.eval_for(for_stmt),
            Stmt::Block(body) => self.eval_body(body),
            Stmt::Store(store) => Ok(self.eval_store(store)),
            Stmt::Fn(_) => Ok(Value::Nil),
            Stmt::TypeDecl(type_decl) => Ok(self.type_decl(type_decl)),
            Stmt::Node(node) => self.eval_node(node),
            Stmt::Is(stmt) => self.eval_is(stmt),
            Stmt::EnumDecl(_) => Ok(Value::Nil),
            Stmt::OnEvents(on) => Ok(self.eval_on_events(on)),
            Stmt::Comment(_) => Ok(Value::Nil),
            Stmt::Alias(_) => Ok(Value::Void),
            Stmt::EmptyLine(_) => Ok(Value::Void),
            Stmt::Union(_) => Ok(Value::Void),
            Stmt::Tag(_) => Ok(Value::Void),
            Stmt::SpecDecl(spec_decl) => Ok(self.spec_decl(spec_decl)),
            Stmt::Break => Ok(Value::Void),
        }
    }

    fn eval_use(&mut self, use_: &Use) -> Value {
        match use_.kind {
            ast::UseKind::Auto => self.eval_use_auto(use_),
            ast::UseKind::C => self.eval_use_c(use_),
            ast::UseKind::Rust => self.eval_use_rust(use_),
        }
    }

    fn eval_use_auto(&mut self, use_stmt: &ast::Use) -> Value {
        // Construct module path from paths (e.g., ["auto", "io"] -> "auto.io")
        let module_path = use_stmt.paths.join(".");

        // Check if module exists in VM registry
        let registry = crate::vm::VM_REGISTRY.lock().unwrap();
        let module = match registry.get_module(&module_path) {
            Some(m) => m,
            None => {
                return Value::Error(format!("Module '{}' not found", module_path).into());
            }
        };

        // Register all types from this module in the universe
        // (Types need to be available even if not explicitly imported)
        for (type_name, _type_entry) in module.types.iter() {
            let type_decl = ast::TypeDecl {
                name: type_name.clone(),
                kind: ast::TypeDeclKind::UserType,
                parent: None,
                has: vec![],
                specs: vec![],
                members: vec![],
                delegations: vec![],
                methods: vec![],
            };
            self.universe.borrow_mut().define_type(
                type_name.clone(),
                std::rc::Rc::new(crate::scope::Meta::Type(ast::Type::User(type_decl))),
            );
        }
        drop(registry);

        // Register each imported item in current scope
        for item_name in &use_stmt.items {
            // Check if it's a function
            if let Some(_func_entry) = crate::vm::VM_REGISTRY
                .lock()
                .unwrap()
                .get_function(&module_path, item_name)
            {
                // Create a VmFunction metadata entry
                let fn_decl = ast::Fn::new(
                    ast::FnKind::VmFunction,
                    item_name.clone(),
                    None,
                    vec![],
                    ast::Body::new(),
                    ast::Type::Unknown,
                );

                // Register in current scope
                self.universe.borrow_mut().define(
                    item_name.clone(),
                    std::rc::Rc::new(crate::scope::Meta::Fn(fn_decl)),
                );
            }
        }

        Value::Void
    }

    fn eval_use_c(&mut self, _use_stmt: &ast::Use) -> Value {
        // TODO: Implement C library loading
        Value::Void
    }

    fn eval_use_rust(&mut self, _use_stmt: &ast::Use) -> Value {
        // TODO: Implement Rust library loading
        Value::Void
    }

    fn collect_config_body(&mut self, vals: Vec<Value>) -> Vec<Value> {
        vals
    }

    fn eval_body(&mut self, body: &Body) -> AutoResult<Value> {
        self.enter_scope();
        let mut res = Vec::new();
        for stmt in body.stmts.iter() {
            res.push(self.eval_stmt(stmt)?);
        }
        let res = match self.mode {
            EvalMode::SCRIPT => Ok(res.last().unwrap_or(&Value::Nil).clone()),
            EvalMode::CONFIG => Ok(Value::Block(Array::from_vec(self.collect_config_body(res)))),
            EvalMode::TEMPLATE => Ok(Value::Str(
                res.iter()
                    .map(|v| match v {
                        Value::Str(s) => s.clone(),
                        _ => v.to_astr(),
                    })
                    .collect::<Vec<AutoStr>>()
                    .join("\n")
                    .into(),
            )),
        };
        self.exit_scope();
        res
    }

    fn eval_loop_body(
        &mut self,
        body: &Body,
        is_mid: bool,
        is_new_line: bool,
    ) -> AutoResult<Value> {
        self.universe
            .borrow_mut()
            .set_local_val("is_mid", Value::Bool(is_mid));
        let mut res = Vec::new();
        let sep = if is_new_line { "\n" } else { "" };
        for stmt in body.stmts.iter() {
            res.push(self.eval_stmt(stmt)?);
        }
        Ok(match self.mode {
            EvalMode::SCRIPT => res.last().unwrap_or(&Value::Nil).clone(),
            EvalMode::CONFIG => Value::Array(Array::from_vec(self.collect_config_body(res))),
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
        })
    }

    fn eval_is(&mut self, stmt: &Is) -> AutoResult<Value> {
        let t = &stmt.target;
        for br in &stmt.branches {
            match br {
                IsBranch::EqBranch(expr, body) => {
                    // Resolve ValueRefs before comparison
                    let target_val = self.eval_expr(t);
                    let expr_val = self.eval_expr(&expr);

                    let target_resolved = self.resolve_or_clone(&target_val);
                    let expr_resolved = self.resolve_or_clone(&expr_val);

                    // Convert back to Value for comparison
                    let target_value = Value::from_data(target_resolved);
                    let expr_value = Value::from_data(expr_resolved);

                    let cond = target_value == expr_value;
                    if cond {
                        return self.eval_body(&body);
                    }
                }
                // TODO: implement other types of is-branch
                _ => {
                    return Ok(Value::Void);
                }
            }
        }
        Ok(Value::Void)
    }

    fn eval_if(&mut self, if_: &If) -> AutoResult<Value> {
        for branch in if_.branches.iter() {
            let cond = self.eval_expr(&branch.cond);

            // Resolve ValueRef before checking truthiness
            let cond_is_true = match &cond {
                Value::ValueRef(_vid) => {
                    if let Some(data) = self.resolve_value(&cond) {
                        let borrowed_data = data.borrow();
                        match &*borrowed_data {
                            ValueData::Bool(b) => *b,
                            ValueData::Int(i) => *i > 0,
                            ValueData::Uint(u) => *u > 0,
                            ValueData::Float(f) => *f > 0.0,
                            ValueData::Str(s) => s.len() > 0,
                            ValueData::Byte(b) => *b > 0,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => cond.is_true(),
            };

            if cond_is_true {
                return self.eval_body(&branch.body);
            }
        }
        if let Some(else_stmt) = &if_.else_ {
            return self.eval_body(else_stmt);
        }
        Ok(Value::Void)
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
            Iter::Call(_) => {
                todo!()
            }
            Iter::Ever => {
                // No iteration variable for infinite loops
            }
            Iter::Cond => {
                // No iteration variable for conditional loops
            }
        }
    }

    fn eval_for(&mut self, for_stmt: &For) -> AutoResult<Value> {
        let iter = &for_stmt.iter;
        let body = &for_stmt.body;
        let mut max_loop = 1000;

        // Execute init statement if present
        if let Some(init_stmt) = &for_stmt.init {
            self.eval_stmt(init_stmt)?;
        }

        // Handle conditional for loop: for condition { ... }
        if matches!(iter, Iter::Cond) {
            let mut res = Array::new();
            self.universe.borrow_mut().enter_scope();
            loop {
                if max_loop <= 0 {
                    self.universe.borrow_mut().exit_scope();
                    return Ok(Value::error("Max loop reached"));
                }
                max_loop -= 1;

                let cond = self.eval_expr(&for_stmt.range);
                let cond_is_true = cond.is_true();

                if !cond_is_true {
                    break;
                }

                match self.eval_loop_body(body, false, for_stmt.new_line) {
                    Ok(val) => {
                        if let Value::Array(arr) = &val {
                            res.extend(arr);
                        } else {
                            res.push(val);
                        }
                    }
                    Err(e) => {
                        self.universe.borrow_mut().exit_scope();
                        return Err(e);
                    }
                }
            }
            self.universe.borrow_mut().exit_scope();

            return Ok(match self.mode {
                EvalMode::SCRIPT => Value::Void,
                EvalMode::CONFIG => Value::Array(res),
                EvalMode::TEMPLATE => Value::Str(
                    res.iter()
                        .filter(|v| match v {
                            Value::Nil => false,
                            _ => true,
                        })
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                        .into(),
                ),
            });
        }

        let range = self.eval_expr(&for_stmt.range);

        // Resolve ValueRef for range/array operations
        let range_resolved = match &range {
            Value::ValueRef(_vid) => {
                if let Some(data) = self.resolve_value(&range) {
                    let borrowed_data = data.borrow();
                    let data_clone = borrowed_data.clone();
                    drop(borrowed_data);
                    Some(Value::from_data(data_clone))
                } else {
                    None
                }
            }
            _ => Some(range.clone()),
        };

        let range_final = match range_resolved {
            Some(v) => v,
            None => return Ok(Value::error(format!("Invalid range {}", range))),
        };

        let mut res = Array::new();
        let mut is_mid = true;
        let is_new_line = for_stmt.new_line;
        let sep = if for_stmt.new_line { "\n" } else { "" };
        self.universe.borrow_mut().enter_scope();
        match range_final {
            Value::Range(start, end) => {
                let len = (end - start) as usize;
                for (idx, n) in (start..end).enumerate() {
                    if idx == len - 1 {
                        is_mid = false;
                    }
                    self.eval_iter(iter, idx, Value::Int(n));
                    match self.eval_loop_body(body, is_mid, is_new_line) {
                        Ok(val) => {
                            if let Value::Array(arr) = &val {
                                res.extend(arr);
                            } else {
                                res.push(val);
                            }
                        }
                        Err(e) => {
                            self.universe.borrow_mut().exit_scope();
                            return Err(e);
                        }
                    }
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
                    match self.eval_loop_body(body, is_mid, is_new_line) {
                        Ok(val) => {
                            if let Value::Array(arr) = &val {
                                res.extend(arr);
                            } else {
                                res.push(val);
                            }
                        }
                        Err(e) => {
                            self.universe.borrow_mut().exit_scope();
                            return Err(e);
                        }
                    }
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
                    match self.eval_loop_body(body, is_mid, is_new_line) {
                        Ok(val) => {
                            if let Value::Array(arr) = &val {
                                res.extend(arr);
                            } else {
                                res.push(val);
                            }
                        }
                        Err(e) => {
                            self.universe.borrow_mut().exit_scope();
                            return Err(e);
                        }
                    }
                    max_loop -= 1;
                }
            }
            _ => {
                return Ok(Value::error(format!("Invalid range {}", range_final)));
            }
        }
        self.universe.borrow_mut().exit_scope();
        if max_loop <= 0 {
            Ok(Value::error("Max loop reached"))
        } else {
            Ok(match self.mode {
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
            })
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

    fn eval_range(&mut self, range: &Range) -> Value {
        if range.eq {
            self.range_eq(&range.start, &range.end)
        } else {
            self.range(&range.start, &range.end)
        }
    }

    fn eval_bina(&mut self, left: &Expr, op: &Op, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        // Resolve ValueRef for arithmetic operations
        let left_resolved = self.resolve_or_clone(&left_value);
        let right_resolved = self.resolve_or_clone(&right_value);

        match op {
            Op::Add => {
                // Convert resolved ValueData back to Value for add()
                add(
                    Value::from_data(left_resolved.clone()),
                    Value::from_data(right_resolved.clone()),
                )
            }
            Op::Sub => sub(
                Value::from_data(left_resolved.clone()),
                Value::from_data(right_resolved.clone()),
            ),
            Op::Mul => mul(
                Value::from_data(left_resolved.clone()),
                Value::from_data(right_resolved.clone()),
            ),
            Op::Div => div(
                Value::from_data(left_resolved.clone()),
                Value::from_data(right_resolved.clone()),
            ),
            Op::Eq | Op::Neq | Op::Lt | Op::Gt | Op::Le | Op::Ge => comp(
                &Value::from_data(left_resolved),
                &op,
                &Value::from_data(right_resolved),
            ),
            Op::Asn => self.eval_asn(left, right_value),
            Op::Range => self.range(left, right),
            Op::RangeEq => self.range_eq(left, right),
            Op::Dot => self.dot(left, right),
            _ => Value::Nil,
        }
    }

    fn eval_asn(&mut self, left: &Expr, val: Value) -> Value {
        match left {
            // Case 1: Simple identifier: x = value
            Expr::Ident(name) => {
                // check ref
                let left_val = self.lookup(&name);
                match left_val {
                    Value::Ref(target) => {
                        // println!("ref: {}", target); // LSP: disabled
                        if self.universe.borrow().exists(&target) {
                            self.universe.borrow_mut().update_val(&target, val);
                        } else {
                            // Variable not found - return error with suggestion
                            let candidates = self.universe.borrow().get_defined_names();
                            let suggestion = if let Some(s) =
                                crate::error::find_best_match(&target, &candidates)
                            {
                                format!(". Did you mean '{}'?", s)
                            } else {
                                String::new()
                            };
                            return Value::Error(
                                format!(
                                    "Variable (ref {} -> {}) not found{}",
                                    name, target, suggestion
                                )
                                .into(),
                            );
                        }
                    }
                    _ => {
                        if self.universe.borrow().exists(&name) {
                            self.universe.borrow_mut().update_val(&name, val);
                        } else {
                            // Variable not found - return error with suggestion
                            let candidates = self.universe.borrow().get_defined_names();
                            let suggestion = if let Some(s) =
                                crate::error::find_best_match(&name, &candidates)
                            {
                                format!(". Did you mean '{}'?", s)
                            } else {
                                String::new()
                            };
                            return Value::Error(
                                format!("Variable {} not found{}", name, suggestion).into(),
                            );
                        }
                    }
                }
                Value::Void
            }

            // Case 2: Nested access: obj.field = value or obj.inner.field = value or obj.arr[0] = value
            Expr::Bina(left_obj, op, right_field) if *op == Op::Dot => {
                // Convert right-hand side to ValueData and allocate (only for nested assignment)
                let right_data = val.into_data();
                let right_vid = self.universe.borrow_mut().alloc_value(right_data);

                match left_obj.as_ref() {
                    // Simple case: obj.field = value
                    Expr::Ident(obj_name) => {
                        if let Some(obj_vid) = self.lookup_vid(obj_name) {
                            // Check if right_field is an index expression (obj.arr[0] = value)
                            match &**right_field {
                                Expr::Index(arr_field, index_expr) => {
                                    if let Expr::Ident(arr_name) = &**arr_field {
                                        let idx_val = self.eval_expr(index_expr);
                                        if let Value::Int(i) = idx_val {
                                            let path = auto_val::AccessPath::Nested(
                                                Box::new(auto_val::AccessPath::Field(
                                                    arr_name.clone(),
                                                )),
                                                Box::new(auto_val::AccessPath::Index(i as usize)),
                                            );
                                            match self
                                                .universe
                                                .borrow_mut()
                                                .update_nested(obj_vid, &path, right_vid)
                                            {
                                                Ok(()) => Value::Void,
                                                Err(e) => Value::error(format!(
                                                    "Failed to assign to array element: {:?}",
                                                    e
                                                )),
                                            }
                                        } else {
                                            Value::error("Array index must be integer")
                                        }
                                    } else {
                                        Value::error(format!("Invalid array target"))
                                    }
                                }
                                _ => {
                                    // Regular field access: obj.field = value
                                    let field_name = self.expr_to_astr(right_field);
                                    let path = auto_val::AccessPath::Field(field_name);
                                    match self
                                        .universe
                                        .borrow_mut()
                                        .update_nested(obj_vid, &path, right_vid)
                                    {
                                        Ok(()) => Value::Void,
                                        Err(e) => Value::error(format!(
                                            "Failed to assign to field: {:?}",
                                            e
                                        )),
                                    }
                                }
                            }
                        } else {
                            Value::error(format!("Variable not found: {}", obj_name))
                        }
                    }
                    // Nested case: obj.inner.field = value or arr[0].field = value
                    _ => {
                        // Extract the top-level identifier from the nested path
                        // We need to rebuild the path as Nested(top_level_field, rest_of_path)
                        // Actually, for cases like obj.inner.field, we need to:
                        // 1. Look up obj (top-level identifier)
                        // 2. Build path for inner.field
                        // So we need to extract the first component separately

                        // For now, handle the common case: arr[0].field
                        // The left_obj is arr[0] (Index expression)
                        // We need to get the array name and index
                        if let Expr::Index(array, index) = left_obj.as_ref() {
                            if let Expr::Ident(arr_name) = array.as_ref() {
                                if let Some(arr_vid) = self.lookup_vid(arr_name) {
                                    let idx_val = self.eval_expr(index);
                                    if let Value::Int(i) = idx_val {
                                        let field_name = self.expr_to_astr(right_field);
                                        let path = auto_val::AccessPath::Nested(
                                            Box::new(auto_val::AccessPath::Index(i as usize)),
                                            Box::new(auto_val::AccessPath::Field(field_name)),
                                        );
                                        match self
                                            .universe
                                            .borrow_mut()
                                            .update_nested(arr_vid, &path, right_vid)
                                        {
                                            Ok(()) => Value::Void,
                                            Err(e) => Value::error(format!(
                                                "Failed to assign to nested field: {:?}",
                                                e
                                            )),
                                        }
                                    } else {
                                        Value::error("Array index must be integer")
                                    }
                                } else {
                                    Value::error(format!("Array not found: {}", arr_name))
                                }
                            } else {
                                Value::error(format!("Invalid assignment target: {}", left_obj))
                            }
                        } else {
                            // Handle obj.inner.field case
                            // left_obj is obj.inner (Bina expression)
                            // We need to find the top-level identifier
                            let top_level = self.extract_top_level_identifier(left_obj);
                            if let Some(obj_name) = top_level {
                                if let Some(obj_vid) = self.lookup_vid(&obj_name) {
                                    // Build path for the rest (inner), excluding the top-level identifier
                                    let inner_path = match self
                                        .build_path_excluding_top_level(left_obj, &obj_name)
                                    {
                                        Ok(path) => path,
                                        Err(e) => {
                                            return Value::error(format!(
                                                "Invalid access path: {}",
                                                e
                                            ))
                                        }
                                    };

                                    // Add the rightmost field to complete the path
                                    let right_field_name = self.expr_to_astr(right_field);
                                    let full_path = auto_val::AccessPath::Nested(
                                        Box::new(inner_path),
                                        Box::new(auto_val::AccessPath::Field(right_field_name)),
                                    );

                                    match self
                                        .universe
                                        .borrow_mut()
                                        .update_nested(obj_vid, &full_path, right_vid)
                                    {
                                        Ok(()) => Value::Void,
                                        Err(e) => Value::error(format!(
                                            "Failed to assign to nested field: {:?}",
                                            e
                                        )),
                                    }
                                } else {
                                    Value::error(format!("Variable not found: {}", obj_name))
                                }
                            } else {
                                Value::error(format!("Invalid assignment target"))
                            }
                        }
                    }
                }
            }

            // Case 3: Array index: arr[0] = value or matrix[0][1] = value
            Expr::Index(array, index) => {
                // Convert right-hand side to ValueData and allocate (only for nested assignment)
                let right_data = val.into_data();
                let right_vid = self.universe.borrow_mut().alloc_value(right_data);

                match array.as_ref() {
                    // Simple case: arr[0] = value
                    Expr::Ident(arr_name) => {
                        if let Some(arr_vid) = self.lookup_vid(arr_name) {
                            let idx_val = self.eval_expr(index);
                            if let Value::Int(i) = idx_val {
                                let path = auto_val::AccessPath::Index(i as usize);
                                match self
                                    .universe
                                    .borrow_mut()
                                    .update_nested(arr_vid, &path, right_vid)
                                {
                                    Ok(()) => Value::Void,
                                    Err(e) => {
                                        Value::error(format!("Failed to assign to index: {:?}", e))
                                    }
                                }
                            } else {
                                Value::error("Array index must be integer")
                            }
                        } else {
                            Value::error(format!("Array not found: {}", arr_name))
                        }
                    }
                    // Nested case: matrix[0][1] = value
                    Expr::Index(nested_array, nested_index) => {
                        // Extract top-level array name
                        if let Expr::Ident(arr_name) = nested_array.as_ref() {
                            if let Some(arr_vid) = self.lookup_vid(arr_name) {
                                let idx_val = self.eval_expr(index);
                                if let Value::Int(i) = idx_val {
                                    // Build nested path: [nested_index][i]
                                    let nested_idx_val = self.eval_expr(nested_index);
                                    if let Value::Int(nested_i) = nested_idx_val {
                                        let path = auto_val::AccessPath::Nested(
                                            Box::new(auto_val::AccessPath::Index(
                                                nested_i as usize,
                                            )),
                                            Box::new(auto_val::AccessPath::Index(i as usize)),
                                        );
                                        match self
                                            .universe
                                            .borrow_mut()
                                            .update_nested(arr_vid, &path, right_vid)
                                        {
                                            Ok(()) => Value::Void,
                                            Err(e) => Value::error(format!(
                                                "Failed to assign to nested index: {:?}",
                                                e
                                            )),
                                        }
                                    } else {
                                        Value::error("Nested array index must be integer")
                                    }
                                } else {
                                    Value::error("Array index must be integer")
                                }
                            } else {
                                Value::error(format!("Array not found: {}", arr_name))
                            }
                        } else {
                            Value::error(format!("Invalid assignment target"))
                        }
                    }
                    // Case: obj.items[0] = value or obj.inner.arr[0] = value
                    Expr::Bina(left_obj, op, right_field) if *op == Op::Dot => {
                        // We need to handle obj.items[0] where:
                        // - left_obj could be an identifier or another Bina expression
                        // - right_field is the field containing the array
                        match left_obj.as_ref() {
                            // Simple case: obj.items[0] = value
                            Expr::Ident(obj_name) => {
                                if let Some(obj_vid) = self.lookup_vid(obj_name) {
                                    let field_name = self.expr_to_astr(right_field);
                                    let idx_val = self.eval_expr(index);
                                    if let Value::Int(i) = idx_val {
                                        let path = auto_val::AccessPath::Nested(
                                            Box::new(auto_val::AccessPath::Field(field_name)),
                                            Box::new(auto_val::AccessPath::Index(i as usize)),
                                        );
                                        match self
                                            .universe
                                            .borrow_mut()
                                            .update_nested(obj_vid, &path, right_vid)
                                        {
                                            Ok(()) => Value::Void,
                                            Err(e) => Value::error(format!(
                                                "Failed to assign to nested array element: {:?}",
                                                e
                                            )),
                                        }
                                    } else {
                                        Value::error("Array index must be integer")
                                    }
                                } else {
                                    Value::error(format!("Object not found: {}", obj_name))
                                }
                            }
                            // Nested case: obj.inner.items[0] = value
                            _ => {
                                let top_level = self.extract_top_level_identifier(array);
                                if let Some(obj_name) = top_level {
                                    if let Some(obj_vid) = self.lookup_vid(&obj_name) {
                                        // Build the full path: inner.items[0]

                                        // Build path for the left_obj part (e.g., inner)
                                        let left_path = match self.build_access_path(left_obj) {
                                            Ok(path) => path,
                                            Err(e) => {
                                                return Value::error(format!(
                                                    "Invalid access path: {}",
                                                    e
                                                ))
                                            }
                                        };

                                        // Build path for right_field + index
                                        let field_name = self.expr_to_astr(right_field);
                                        let idx_val = self.eval_expr(index);
                                        if let Value::Int(i) = idx_val {
                                            let field_idx_path = auto_val::AccessPath::Nested(
                                                Box::new(auto_val::AccessPath::Field(field_name)),
                                                Box::new(auto_val::AccessPath::Index(i as usize)),
                                            );

                                            // Combine left_path with field_idx_path
                                            let full_path = auto_val::AccessPath::Nested(
                                                Box::new(left_path),
                                                Box::new(field_idx_path),
                                            );

                                            match self
                                                .universe
                                                .borrow_mut()
                                                .update_nested(obj_vid, &full_path, right_vid)
                                            {
                                                Ok(()) => Value::Void,
                                                Err(e) => Value::error(format!(
                                                    "Failed to assign to deeply nested array element: {:?}",
                                                    e
                                                )),
                                            }
                                        } else {
                                            Value::error("Array index must be integer")
                                        }
                                    } else {
                                        Value::error(format!("Object not found: {}", obj_name))
                                    }
                                } else {
                                    Value::error(format!("Invalid assignment target"))
                                }
                            }
                        }
                    }
                    _ => Value::error(format!("Invalid assignment target")),
                }
            }

            _ => Value::error(format!("Invalid target of asn {} = {}", left, val)),
        }
    }

    /// Helper: Convert expression to AutoStr (for field names)
    fn expr_to_astr(&self, expr: &Expr) -> AutoStr {
        match expr {
            Expr::Ident(name) => name.clone(),
            Expr::Str(s) => s.clone().into(),
            Expr::Int(i) => i.to_string().into(),
            _ => expr.repr().into(),
        }
    }

    /// Helper: Recursively build AccessPath from expression (without top-level identifier)
    /// Examples:
    /// - `field` → Field("field")
    /// - `inner.field` → Nested(Field("inner"), Field("field"))
    /// - `arr[0]` → Index(0)
    /// - `arr[0].field` → Nested(Index(0), Field("field"))
    /// - `matrix[0][1]` → Nested(Index(0), Index(1))
    fn build_access_path(&mut self, expr: &Expr) -> Result<auto_val::AccessPath, String> {
        match expr {
            // Case 1: Simple field access (base case for recursion)
            Expr::Ident(name) => Ok(auto_val::AccessPath::Field(name.clone())),

            // Case 2: Nested field access: obj.field or arr[0].field
            Expr::Bina(left, op, right) if *op == Op::Dot => {
                // Recursively build path for left side, then add right side
                let left_path = self.build_access_path(left)?;
                let right_field = self.expr_to_astr(right);
                Ok(auto_val::AccessPath::Nested(
                    Box::new(left_path),
                    Box::new(auto_val::AccessPath::Field(right_field)),
                ))
            }

            // Case 3: Array indexing: arr[0] or matrix[0][1]
            Expr::Index(array, index_expr) => {
                // Evaluate the index expression
                let idx_val = self.eval_expr(index_expr);
                if let Value::Int(i) = idx_val {
                    // Check if the array itself is indexed (for matrix[0][1])
                    if matches!(array.as_ref(), Expr::Index(_, _)) {
                        // Nested array indexing
                        let left_path = self.build_access_path(array)?;
                        Ok(auto_val::AccessPath::Nested(
                            Box::new(left_path),
                            Box::new(auto_val::AccessPath::Index(i as usize)),
                        ))
                    } else {
                        // Simple array indexing
                        Ok(auto_val::AccessPath::Index(i as usize))
                    }
                } else {
                    Err(format!("Array index must be integer, got {}", idx_val))
                }
            }

            _ => Err(format!("Invalid access path expression: {}", expr)),
        }
    }

    /// Helper: Extract top-level identifier from a nested expression
    /// Examples:
    /// - `obj` → Some("obj")
    /// - `obj.field` → Some("obj")
    /// - `obj.inner.field` → Some("obj")
    /// - `arr[0]` → Some("arr")
    /// - `arr[0].field` → Some("arr")
    fn extract_top_level_identifier(&self, expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Ident(name) => Some(name.clone()),
            Expr::Bina(left, _, _) => self.extract_top_level_identifier(left),
            Expr::Index(array, _) => self.extract_top_level_identifier(array),
            _ => None,
        }
    }

    /// Helper: Build path from expression, excluding the top-level identifier
    /// Examples:
    /// - `field` → Field("field")
    /// - `obj.inner` → Field("inner")  (excludes "obj")
    /// - `arr[0]` → Index(0)  (excludes "arr")
    /// - `obj.level1.level2` → Nested(Field("level1"), Field("level2"))  (excludes "obj")
    fn build_path_excluding_top_level(
        &mut self,
        expr: &Expr,
        top_level: &str,
    ) -> Result<auto_val::AccessPath, String> {
        match expr {
            Expr::Ident(name) if name == top_level => Err(format!(
                "Expression is just the top-level identifier: {}",
                name
            )),
            Expr::Ident(name) => Ok(auto_val::AccessPath::Field(name.clone())),
            Expr::Bina(left, op, right) if *op == Op::Dot => {
                // Check if left is the top-level identifier
                if let Expr::Ident(name) = left.as_ref() {
                    if name == top_level {
                        // This is where we are: obj.level1 where obj is top-level
                        // But right might be further nested, so we need to check
                        match &**right {
                            // If right is also a Bina (further nesting), recurse
                            Expr::Bina(_inner_left, inner_op, inner_right)
                                if *inner_op == Op::Dot =>
                            {
                                let left_path =
                                    auto_val::AccessPath::Field(self.expr_to_astr(right));
                                let right_field = self.expr_to_astr(inner_right);
                                Ok(auto_val::AccessPath::Nested(
                                    Box::new(left_path),
                                    Box::new(auto_val::AccessPath::Field(right_field)),
                                ))
                            }
                            // Right is a simple identifier
                            _ => {
                                let field_name = self.expr_to_astr(right);
                                Ok(auto_val::AccessPath::Field(field_name))
                            }
                        }
                    } else {
                        // Nested case: obj.inner.field where inner != top_level
                        let left_path = self.build_path_excluding_top_level(left, top_level)?;
                        let right_field = self.expr_to_astr(right);
                        Ok(auto_val::AccessPath::Nested(
                            Box::new(left_path),
                            Box::new(auto_val::AccessPath::Field(right_field)),
                        ))
                    }
                } else {
                    // Recursively handle nested left side
                    let left_path = self.build_path_excluding_top_level(left, top_level)?;
                    let right_field = self.expr_to_astr(right);
                    Ok(auto_val::AccessPath::Nested(
                        Box::new(left_path),
                        Box::new(auto_val::AccessPath::Field(right_field)),
                    ))
                }
            }
            Expr::Index(array, index_expr) => {
                // Check if array is the top-level identifier
                if let Expr::Ident(name) = array.as_ref() {
                    if name == top_level {
                        // Simple case: arr[0] where arr is top-level
                        let idx_val = self.eval_expr(index_expr);
                        if let Value::Int(i) = idx_val {
                            Ok(auto_val::AccessPath::Index(i as usize))
                        } else {
                            Err(format!("Array index must be integer, got {}", idx_val))
                        }
                    } else {
                        // Nested case: shouldn't happen normally
                        Err(format!("Unexpected nested index"))
                    }
                } else {
                    // Nested case: matrix[0][1]
                    let left_path = self.build_path_excluding_top_level(array, top_level)?;
                    let idx_val = self.eval_expr(index_expr);
                    if let Value::Int(i) = idx_val {
                        Ok(auto_val::AccessPath::Nested(
                            Box::new(left_path),
                            Box::new(auto_val::AccessPath::Index(i as usize)),
                        ))
                    } else {
                        Err(format!("Array index must be integer, got {}", idx_val))
                    }
                }
            }
            _ => Err(format!("Invalid expression: {}", expr)),
        }
    }

    #[allow(dead_code)]
    fn update_obj(&mut self, name: &str, f: impl FnOnce(&mut Obj)) -> Value {
        self.universe.borrow_mut().update_obj(name, f);
        Value::Void
    }

    #[allow(dead_code)]
    fn update_array(&mut self, name: &str, idx: Value, val: Value) -> Value {
        self.universe.borrow_mut().update_array(name, idx, val);
        Value::Void
    }

    fn range(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        // Resolve ValueRef for range operations
        let left_resolved = self.resolve_or_clone(&left_value);
        let right_resolved = self.resolve_or_clone(&right_value);

        match (&left_resolved, &right_resolved) {
            (auto_val::ValueData::Int(left), auto_val::ValueData::Int(right)) => {
                Value::Range(*left, *right)
            }
            _ => Value::error(format!("Invalid range {}..{}", left_value, right_value)),
        }
    }

    fn range_eq(&mut self, left: &Expr, right: &Expr) -> Value {
        let left_value = self.eval_expr(left);
        let right_value = self.eval_expr(right);

        // Resolve ValueRef for range operations
        let left_resolved = self.resolve_or_clone(&left_value);
        let right_resolved = self.resolve_or_clone(&right_value);

        match (&left_resolved, &right_resolved) {
            (auto_val::ValueData::Int(left), auto_val::ValueData::Int(right)) => {
                Value::RangeEq(*left, *right)
            }
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
        // lookup value - now returns Value::ValueRef(vid)
        self.universe
            .borrow()
            .lookup_val(name)
            .unwrap_or(Value::Nil)
    }

    /// Get value ID directly without wrapping
    fn lookup_vid(&self, name: &str) -> Option<auto_val::ValueID> {
        self.universe.borrow().lookup_val_id(name)
    }

    /// Resolve Value::Ref to actual data
    fn resolve_value(&self, value: &Value) -> Option<Rc<RefCell<auto_val::ValueData>>> {
        match value {
            Value::ValueRef(vid) => self.universe.borrow().get_value(*vid),
            _ => None, // Inline values don't have stored data
        }
    }

    /// Helper: Resolve Ref or clone inline value
    fn resolve_or_clone(&self, val: &Value) -> auto_val::ValueData {
        match val {
            Value::ValueRef(vid) => self
                .universe
                .borrow()
                .get_value(*vid)
                .map(|cell| cell.borrow().clone())
                .unwrap_or(auto_val::ValueData::Nil),
            _ => val.clone().into_data(),
        }
    }

    fn eval_array(&mut self, elems: &Vec<Expr>) -> Value {
        let mut values = Array::new();
        for elem in elems.iter() {
            let v = self.eval_expr(elem);
            if !v.is_void() {
                values.push(v);
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
    fn eval_call(&mut self, call: &Call) -> AutoResult<Value> {
        // Check if this is a method call like `file.close()`
        if let Expr::Bina(left, op, right) = &*call.name {
            if *op == Op::Dot {
                // This is a dot expression - check if it's a method call
                // Evaluate the left side to get the instance
                let instance = self.eval_expr(left);

                // Resolve ValueRef if needed
                let instance_resolved = match &instance {
                    Value::ValueRef(_vid) => {
                        if let Some(data) = self.resolve_value(&instance) {
                            let borrowed_data = data.borrow();
                            let data_clone = borrowed_data.clone();
                            drop(borrowed_data);
                            Some(Value::from_data(data_clone))
                        } else {
                            None
                        }
                    }
                    _ => Some(instance.clone()),
                };

                if let Some(inst) = instance_resolved {
                    // Check if it's an instance and try to call as a VM method
                    if let Value::Instance(ref inst_data) = &inst {
                        if let Expr::Ident(method_name) = &**right {
                            // Look up the method in the VM registry
                            let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                            let method = registry
                                .get_method(&inst_data.ty.name(), method_name.as_str())
                                .cloned();
                            drop(registry);

                            if let Some(method) = method {
                                // Evaluate arguments (Arg::Pos contains Expr, not Value)
                                let mut arg_vals = Vec::new();
                                for arg in call.args.args.iter() {
                                    match arg {
                                        ast::Arg::Pos(expr) => {
                                            arg_vals.push(self.eval_expr(expr));
                                        }
                                        _ => {}
                                    }
                                }

                                // Call the VM method with the instance
                                let uni = self.universe.clone();
                                return Ok(method(uni, &mut inst.clone(), arg_vals));
                            }
                        }
                    }
                }
            }
        }

        // Regular function call (non-method)
        let name = self.eval_expr(&call.name);
        if name == Value::Nil {
            return Ok(Value::error(format!(
                "Invalid function name to call {}",
                call.name
            )));
        }

        // Resolve ValueRef before matching on function type
        let name_resolved = match &name {
            Value::ValueRef(_vid) => {
                if let Some(data) = self.resolve_value(&name) {
                    let borrowed_data = data.borrow();
                    let data_clone = borrowed_data.clone();
                    drop(borrowed_data);
                    Some(Value::from_data(data_clone))
                } else {
                    None
                }
            }
            _ => Some(name.clone()),
        };

        let name_final = match name_resolved {
            Some(v) => v,
            None => {
                return Ok(Value::error(format!(
                    "Invalid function name to call {}",
                    call.name
                )))
            }
        };

        match name_final {
            // Value::Type(Type::User(u)) => {
            // return self.eval_type_new(&u, &call.args);
            // }
            Value::Meta(meta_id) => match meta_id {
                MetaID::Fn(sig) => {
                    return self.eval_fn_call_with_sig(&sig, &call.args);
                }
                // MetaID::Type(name) => {
                // return self.eval_type_new(&name, &call.args);
                // }
                _ => {
                    // println!("Strange function call {}", meta_id); // LSP: disabled
                }
            },
            Value::ExtFn(extfn) => {
                let args_val = self.eval_args(&call.args);
                return Ok((extfn.fun)(&args_val));
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
                            return Ok(Value::error(format!("Invalid lambda {}", name)));
                        }
                    }
                } else {
                    return Ok(Value::error(format!("Invalid lambda {}", name)));
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
                return Ok(Value::error(format!("Invalid function call {}", name)));
            }
        }

        // Lookup Fn meta
        let meta = self.universe.borrow().lookup_meta(&call.get_name_text());
        if let Some(meta) = meta {
            match meta.as_ref() {
                scope::Meta::Fn(fn_decl) => {
                    return self.eval_fn_call(fn_decl, &call.args);
                }
                _ => {
                    return Ok(Value::error(format!(
                        "Invalid lambda {}",
                        call.get_name_text()
                    )))
                }
            }
        } else {
            // convert call to node intance
            // println!("call {} not found, try to eval node", call.get_name_text()); // LSP: disabled
            let node: Node = call.clone().into();
            return self.eval_node(&node);
        }
    }

    pub fn eval_type_new(&mut self, name: &str, args: &auto_val::Args) -> Value {
        let typ = self.universe.borrow().lookup_type(name);
        match typ {
            ast::Type::User(type_decl) => {
                let instance = self.eval_instance(&type_decl, args);
                return instance;
            }
            _ => Value::error(format!("Invalid type instance of {}", name)),
        }
    }

    fn eval_instance(&mut self, type_decl: &TypeDecl, args: &auto_val::Args) -> Value {
        let ty = self.eval_type(&type_decl);
        let fields = self.eval_fields(&type_decl, args);
        Value::Instance(auto_val::Instance { ty, fields })
    }

    fn eval_type(&mut self, type_decl: &TypeDecl) -> Type {
        Type::User(type_decl.name.clone())
    }

    fn eval_fields(&mut self, type_decl: &TypeDecl, args: &auto_val::Args) -> Obj {
        let members = &type_decl.members;
        // TODO: remove unnecessary clone
        let mut fields = Obj::new();

        // First, mix in fields from composed types
        for has_type in &type_decl.has {
            if let ast::Type::User(has_decl) = has_type {
                // Add default values for fields from composed type
                for member in &has_decl.members {
                    if !fields.has(member.name.clone()) {
                        match &member.value {
                            Some(default_value) => {
                                let val_data = self.eval_expr(default_value).into_data();
                                let vid = self.universe.borrow_mut().alloc_value(val_data);
                                fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                            }
                            None => {
                                // No default value, set to nil
                                fields.set(member.name.clone(), Value::Nil);
                            }
                        }
                    }
                }
            }
        }

        // Then, add fields from direct arguments
        for (j, arg) in args.args.iter().enumerate() {
            // let val_arg = self.eval_arg(arg);
            match arg {
                auto_val::Arg::Pair(key, val) => {
                    for member in members.iter() {
                        if key.to_string() == member.name {
                            // If val is a ValueRef, we need to get the actual value from universe
                            let val_to_store = match val {
                                auto_val::Value::ValueRef(_vid) => {
                                    if let Some(data) = self.resolve_value(val) {
                                        let borrowed = data.borrow();
                                        let cloned = borrowed.clone();
                                        drop(borrowed);
                                        Some(Value::from_data(cloned))
                                    } else {
                                        None
                                    }
                                }
                                _ => Some(val.clone()),
                            };
                            if let Some(v) = val_to_store {
                                let val_data = v.into_data();
                                let vid = self.universe.borrow_mut().alloc_value(val_data);
                                fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                            }
                        }
                    }
                }
                auto_val::Arg::Pos(value) => {
                    if j < members.len() {
                        let member = &members[j];
                        // If value is a ValueRef, we need to get the actual value from universe
                        let val_to_store = match value {
                            auto_val::Value::ValueRef(_vid) => {
                                if let Some(data) = self.resolve_value(value) {
                                    let borrowed = data.borrow();
                                    let cloned = borrowed.clone();
                                    drop(borrowed);
                                    Some(Value::from_data(cloned))
                                } else {
                                    None
                                }
                            }
                            _ => Some(value.clone()),
                        };
                        if let Some(v) = val_to_store {
                            let val_data = v.into_data();
                            let vid = self.universe.borrow_mut().alloc_value(val_data);
                            fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                        }
                    }
                }
                auto_val::Arg::Name(name) => {
                    for member in members.iter() {
                        if *name == member.name {
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
                    let val_data = self.eval_expr(value).into_data();
                    let vid = self.universe.borrow_mut().alloc_value(val_data);
                    fields.set(member.name.clone(), auto_val::Value::ValueRef(vid));
                }
                None => {}
            }
        }
        fields
    }

    pub fn eval_method(&mut self, method: &Method, args: &Args) -> AutoResult<Value> {
        let target = &method.target;
        let name = &method.name;
        // methods for Any
        match target.as_ref() {
            Value::Str(s) => {
                let method_fn = self
                    .universe
                    .borrow()
                    .types
                    .lookup_method(Type::Str, name.clone());
                if let Some(method_fn) = method_fn {
                    return Ok(method_fn(&target));
                } else {
                    // println!("wrong method?: {}", s); // LSP: disabled
                }
            }
            Value::Instance(inst) => {
                // First, try to find the method directly in the type
                let meth = self.universe.borrow().lookup_meta(&method.name);
                if let Some(meta) = meth {
                    match meta.as_ref() {
                        Meta::Fn(fn_decl) => {
                            // println!("Eval Method: {}", fn_decl.name); // LSP: disabled
                            // println!("Current Scope: {}", self.universe.borrow().cur_spot); // LSP: disabled
                            // self.enter_scope();
                            self.universe.borrow_mut().set_local_obj(&inst.fields);
                            let mut args = args.clone();
                            let self_ref = Arg::Pair("self".into(), Expr::Ident("x".into()));
                            args.args.insert(0, self_ref);
                            // args.args.push(Arg::Pair("self".into(), inst));
                            let res = self.eval_fn_call(fn_decl, &args)?;
                            // self.exit_scope();
                            return Ok(res);
                        }
                        _ => {
                            return Ok(Value::error(format!("wrong meta for method: {}", meta)));
                        }
                    }
                }

                // Method not found directly, check delegations
                // Get the type declaration to check for delegations
                // Collect delegation info first to avoid borrow issues
                let mut delegation_target: Option<Value> = None;
                let mut delegated_method_name: Option<AutoStr> = None;

                match &inst.ty {
                    auto_val::Type::User(type_name) => {
                        // Lookup the TypeDecl from universe
                        let type_name_clone = type_name.clone();
                        if let Some(meta) = self.universe.borrow().lookup_meta(&type_name_clone) {
                            if let Meta::Type(ast::Type::User(type_decl)) = meta.as_ref() {
                                for delegation in &type_decl.delegations {
                                    // Check if this delegation handles the method
                                    let spec_name = delegation.spec_name.clone();
                                    let member_name = delegation.member_name.clone();
                                    if let Some(spec_meta) = self.universe.borrow().lookup_meta(&spec_name) {
                                        if let Meta::Spec(spec_decl) = spec_meta.as_ref() {
                                            // Check if the spec has this method
                                            if spec_decl.methods.iter().any(|m| m.name == method.name) {
                                                // Found delegation! Get the delegated member value
                                                if let Some(member_value) = inst.fields.lookup(&member_name) {
                                                    // Resolve ValueRef if needed
                                                    let resolved_member = match member_value {
                                                        Value::ValueRef(_vid) => {
                                                            if let Some(data) = self.resolve_value(&member_value) {
                                                                let borrowed = data.borrow();
                                                                let cloned = borrowed.clone();
                                                                drop(borrowed);
                                                                Some(Value::from_data(cloned))
                                                            } else {
                                                                None
                                                            }
                                                        }
                                                        _ => Some(member_value.clone()),
                                                    };

                                                    if resolved_member.is_some() {
                                                        delegation_target = resolved_member;
                                                        delegated_method_name = Some(method.name.clone());
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // If we found a delegation, call the method on the delegated member
                if let (Some(target), Some(method_name)) = (delegation_target, delegated_method_name) {
                    let delegated_method = Method {
                        target: Box::new(target),
                        name: method_name,
                    };
                    return self.eval_method(&delegated_method, args);
                }
            }
            _ => {
                let method_fn = self
                    .universe
                    .borrow()
                    .types
                    .lookup_method(Type::Any, name.clone());
                if let Some(method_fn) = method_fn {
                    return Ok(method_fn(&target));
                }
            }
        }
        Ok(Value::error(format!(
            "Invalid method {} on {}",
            name, target
        )))
    }

    fn eval_fn_call_with_sig(&mut self, sig: &Sig, args: &Args) -> AutoResult<Value> {
        let meta = self.universe.borrow().lookup_sig(sig).unwrap();
        match meta.as_ref() {
            scope::Meta::Fn(fn_decl) => self.eval_fn_call(fn_decl, args),
            _ => Ok(Value::error(format!("Invalid function call {}", sig.name))),
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

    fn eval_fn_arg(&mut self, arg: &Arg, i: usize, params: &Vec<Param>) -> Value {
        match arg {
            Arg::Pair(name, expr) => {
                let val = self.eval_expr(expr);
                let name = &name;
                self.universe.borrow_mut().set_local_val(&name, val.clone());
                val
            }
            Arg::Pos(expr) => {
                let val = self.eval_expr(expr);
                // Only set local variable if params has this index
                // VM functions have empty params, so we skip setting local vars
                if i < params.len() {
                    let name = &params[i].name;
                    self.universe.borrow_mut().set_local_val(&name, val.clone());
                }
                val
            }
            Arg::Name(name) => {
                self.universe
                    .borrow_mut()
                    .set_local_val(name.as_str(), Value::Str(name.clone()));
                Value::Str(name.clone())
            }
        }
    }

    pub fn eval_vm_fn_call(&mut self, fn_decl: &Fn, args: &Vec<Value>) -> Value {
        // Look up the function in the VM registry
        let registry = crate::vm::VM_REGISTRY.lock().unwrap();

        // Search all modules for the function
        let func_entry = registry
            .modules()
            .values()
            .find_map(|module| module.functions.get(fn_decl.name.as_str()))
            .cloned();

        drop(registry);

        match func_entry {
            Some(func_entry) => {
                // Call the Rust function with universe and first argument
                let uni = self.universe.clone();

                // For single-argument functions like open()
                if args.len() == 1 {
                    (func_entry.func)(uni, args[0].clone())
                } else {
                    // For multi-argument functions (not yet supported)
                    Value::Error(
                        format!(
                            "VM functions with {} arguments not yet supported",
                            args.len()
                        )
                        .into(),
                    )
                }
            }
            None => Value::Error(format!("VM function '{}' not found", fn_decl.name).into()),
        }
    }

    pub fn eval_fn_call(&mut self, fn_decl: &Fn, args: &Args) -> AutoResult<Value> {
        // TODO: 需不需要一个单独的 enter_call()
        // println!("scope before enter: {}", self.universe.borrow().cur_spot); // LSP: disabled
        self.universe.borrow_mut().enter_fn(&fn_decl.name);
        // println!("scope after enter: {}", self.universe.borrow().cur_spot); // LSP: disabled
        // println!(
        //     "enter call scope {}",
        //     self.universe.borrow().current_scope().sid
        // );
        let mut arg_vals = Vec::new();
        for (i, arg) in args.args.iter().enumerate() {
            arg_vals.push(self.eval_fn_arg(arg, i, &fn_decl.params));
        }
        let result = match fn_decl.kind {
            FnKind::Function | FnKind::Lambda => {
                let result = self.eval_body(&fn_decl.body)?;
                self.exit_scope();
                Ok(result)
            }
            FnKind::VmFunction => {
                let result = self.eval_vm_fn_call(fn_decl, &arg_vals);
                self.exit_scope();
                Ok(result)
            }
            _ => Ok(Value::Error(
                format!("Fn {} eval not supported ", fn_decl.name).into(),
            )),
        };
        result
    }

    fn index(&mut self, array: &Expr, index: &Expr) -> Value {
        let mut array_value = self.eval_expr(array);
        let index_value = self.eval_expr(index);
        let mut idx = match index_value {
            Value::Int(index) => index,
            // TODO: support negative index
            // TODO: support range index
            _ => return Value::error(format!("Invalid index {}", index_value)),
        };

        // Resolve ValueRef to actual value
        if let Value::ValueRef(_vid) = &array_value {
            if let Some(data) = self.resolve_value(&array_value) {
                let borrowed_data = data.borrow();
                let data_clone = borrowed_data.clone();
                drop(borrowed_data);
                array_value = Value::from_data(data_clone);
            }
        }

        match array_value {
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
            _ => Value::error(format!("Invalid array {}", array_value)),
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Byte(value) => Value::Byte(*value),
            Expr::Uint(value) => Value::Uint(*value),
            Expr::Int(value) => Value::Int(*value),
            Expr::I8(value) => Value::I8(*value),
            Expr::U8(value) => Value::U8(*value),
            Expr::I64(value) => Value::I64(*value),
            Expr::Float(value, _) => Value::Float(*value),
            Expr::Double(value, _) => Value::Double(*value),
            // Why not move here?
            Expr::Char(value) => Value::Char(*value),
            Expr::Str(value) => Value::Str(value.clone().into()),
            Expr::CStr(value) => Value::Str(value.clone().into()),
            Expr::Bool(value) => Value::Bool(*value),
            Expr::Ref(target) => {
                let target_val = self.eval_expr(&Expr::Ident(target.clone()));
                target_val
            }
            Expr::Ident(name) => self.eval_ident(name),
            Expr::GenName(name) => Value::Str(name.into()),
            Expr::Unary(op, e) => self.eval_una(op, e),
            Expr::Bina(left, op, right) => self.eval_bina(left, op, right),
            Expr::Range(range) => self.eval_range(range),
            Expr::If(if_) => match self.eval_if(if_) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in if expression: {:?}", e).into()),
            },
            Expr::Array(elems) => self.eval_array(elems),
            Expr::Call(call) => match self.eval_call(call) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in call: {:?}", e).into()),
            },
            Expr::Node(node) => match self.eval_node(node) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in node: {:?}", e).into()),
            },
            Expr::Index(array, index) => self.index(array, index),
            Expr::Pair(pair) => self.pair(pair),
            Expr::Object(pairs) => self.object(pairs),
            Expr::Block(body) => match self.eval_body(body) {
                Ok(v) => v,
                Err(e) => Value::Error(format!("Error in block: {:?}", e).into()),
            },
            Expr::Lambda(lambda) => Value::Lambda(lambda.name.clone().into()),
            Expr::FStr(fstr) => self.fstr(fstr),
            Expr::Grid(grid) => self.grid(grid),
            Expr::Cover(cover) => self.cover(cover),
            Expr::Uncover(_) => Value::Void,
            Expr::Null => Value::Null,
            Expr::Nil => Value::Nil,
        }
    }

    fn cover(&mut self, _cover: &Cover) -> Value {
        Value::Void
    }

    fn eval_ident(&mut self, name: &AutoStr) -> Value {
        // let univ = self.universe.borrow_mut();
        // return Some(RefMut::map(univ, |map| map.get_mut_val(name).unwrap()));

        let res = self.lookup(&name);
        match res {
            Value::Ref(target) => {
                let target_val = self.eval_expr(&Expr::Ident(target));
                target_val
            }
            Value::Nil => {
                // Try types
                let typ = self.universe.borrow().lookup_type(name);
                if !matches!(typ, ast::Type::Unknown) {
                    let vty: auto_val::Type = typ.into();
                    return Value::Type(vty);
                }
                // try to lookup in meta and builtins
                let meta = self.universe.borrow().lookup_meta(&name);
                if let Some(meta) = meta {
                    return Value::Meta(to_meta_id(&meta));
                }
                // Try builtin
                let v = self
                    .universe
                    .borrow()
                    .lookup_builtin(&name)
                    .unwrap_or(Value::Nil);

                if !v.is_nil() {
                    return v;
                }
                if self.skip_check {
                    Value::Str(name.clone())
                } else {
                    Value::Nil
                }
            }
            _ => res,
        }
    }

    fn type_decl(&mut self, type_decl: &TypeDecl) -> Value {
        // Register the type itself
        let type_meta = scope::Meta::Type(ast::Type::User(type_decl.clone()));
        self.universe.borrow_mut().define(type_decl.name.clone(), std::rc::Rc::new(type_meta));

        // Mix in methods from composed types (has relationships)
        for has_type in &type_decl.has {
            if let ast::Type::User(has_decl) = has_type {
                // Register each method from the composed type
                for method in &has_decl.methods {
                    // Create fully qualified method name: TypeName::method_name
                    let method_name: AutoStr = format!("{}::{}", type_decl.name, method.name).into();

                    // Clone the method and update its name to reflect the new owner
                    let mut mixed_method = method.clone();
                    mixed_method.name = type_decl.name.clone();

                    // Register in universe with qualified name
                    self.universe.borrow_mut().define(
                        method_name,
                        std::rc::Rc::new(scope::Meta::Fn(mixed_method))
                    );
                }
            }
        }

        // Also register the type's own methods
        for method in &type_decl.methods {
            let method_name: AutoStr = format!("{}::{}", type_decl.name, method.name).into();
            self.universe.borrow_mut().define(
                method_name,
                std::rc::Rc::new(scope::Meta::Fn(method.clone()))
            );
        }

        Value::Void
    }

    fn spec_decl(&mut self, _spec_decl: &ast::SpecDecl) -> Value {
        // The spec is already registered in the parser
        // In the future, we might want to:
        // - Create a vtable for trait methods
        // - Store trait metadata for runtime checks
        // - Support trait object creation
        // For now, just return Void as the spec is already in scope
        Value::Void
    }

    fn dot_node(&mut self, node: &auto_val::Node, right: &Expr) -> Option<Value> {
        let Expr::Ident(name) = right else {
            return None;
        };
        if name == "name" {
            return Some(Value::Str(node.name.clone()));
        }
        if name == "id" {
            return Some(Value::Str(node.id.clone()));
        }
        let mut name = name.clone();
        // 1. lookup in the props
        let v = node.get_prop(&name);
        if v.is_nil() {
            // 2.1 check if nodes with the name exists
            let nodes = node.get_nodes(&name);
            if nodes.len() > 1 {
                return Some(Value::array_of(nodes.iter().map(|n| n.clone()).collect()));
            } else if nodes.len() == 1 {
                return Some(Value::Node(nodes[0].clone()));
            }
            // 2.2 lookup in sub nodes
            if name.ends_with("s") {
                name = name[..name.len() - 1].into();
            }
            let nodes = node.get_nodes(&name);
            if nodes.len() > 1 {
                Some(Value::array_of(nodes.iter().map(|n| n.clone()).collect()))
            } else if nodes.len() == 1 {
                Some(Value::Node(nodes[0].clone()))
            } else {
                None
            }
        } else {
            Some(v)
        }
    }

    fn enum_val(&mut self, en: &AutoStr, name: &AutoStr) -> Value {
        // find enum's decl
        let typ = self.universe.borrow().lookup_type(en);
        match typ {
            ast::Type::Enum(en) => {
                // lookup enum value in Enum's items
                match en.borrow().get_item(name) {
                    Some(item) => Value::Int(item.value),
                    None => Value::Nil,
                }
            }
            _ => Value::Nil,
        }
    }

    fn dot(&mut self, left: &Expr, right: &Expr) -> Value {
        let mut left_value = self.eval_expr(left);

        // Resolve ValueRef to actual value
        if let Value::ValueRef(_vid) = &left_value {
            if let Some(data) = self.resolve_value(&left_value) {
                let borrowed_data = data.borrow();
                let data_clone = borrowed_data.clone();
                drop(borrowed_data);
                left_value = Value::from_data(data_clone);
            }
        }

        let res: Option<Value> = match &left_value {
            Value::Type(typ) => {
                match typ {
                    Type::Enum(en) => {
                        // lookup enum value in Enum's items
                        match right {
                            Expr::Ident(name) => Some(self.enum_val(en, name)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            Value::Meta(meta_id) => {
                // lookup meta
                match meta_id {
                    MetaID::Enum(name) => {
                        let right_name = right.repr();
                        Some(self.enum_val(name, &AutoStr::from(right_name)))
                    }
                    _ => None,
                }
            }
            Value::Obj(obj) => match right {
                Expr::Ident(name) => {
                    let field_value = obj.lookup(&name);
                    // Recursively resolve ValueRef from field lookup
                    match &field_value {
                        Some(Value::ValueRef(vid)) => {
                            if let Some(data) = self.resolve_value(&Value::ValueRef(*vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                field_value.clone()
                            }
                        }
                        _ => field_value,
                    }
                }
                Expr::Int(key) => {
                    let field_value = obj.lookup(&key.to_string());
                    match &field_value {
                        Some(Value::ValueRef(vid)) => {
                            if let Some(data) = self.resolve_value(&Value::ValueRef(*vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                field_value.clone()
                            }
                        }
                        _ => field_value,
                    }
                }
                Expr::Bool(key) => {
                    let field_value = obj.lookup(&key.to_string());
                    match &field_value {
                        Some(Value::ValueRef(vid)) => {
                            if let Some(data) = self.resolve_value(&Value::ValueRef(*vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                field_value.clone()
                            }
                        }
                        _ => field_value,
                    }
                }
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
                        Some(Value::ValueRef(vid)) => {
                            // Dereference the ValueRef to get the actual value
                            if let Some(data) = self.resolve_value(&Value::ValueRef(vid)) {
                                let borrowed_data = data.borrow();
                                let data_clone = borrowed_data.clone();
                                drop(borrowed_data);
                                Some(Value::from_data(data_clone))
                            } else {
                                None
                            }
                        }
                        Some(v) => Some(v),
                        None => {
                            // not a field, try method
                            let typ = instance.ty.name();
                            let combined_name: AutoStr = format!("{}::{}", typ, name).into();
                            // println!("Combined name: {}", combined_name); // LSP: disabled
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

    fn eval_mid(&mut self, node: &Node) -> AutoResult<Value> {
        // Resolve ValueRef before converting to bool
        let is_mid_value = self
            .universe
            .borrow()
            .lookup_val("is_mid")
            .unwrap_or(Value::Bool(false));

        let is_mid = match &is_mid_value {
            Value::ValueRef(_vid) => {
                if let Some(data) = self.resolve_value(&is_mid_value) {
                    let borrowed_data = data.borrow();
                    match &*borrowed_data {
                        ValueData::Bool(b) => *b,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Value::Bool(b) => *b,
            _ => false,
        };

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
                let val = self.eval_stmt(stmt)?;
                res = val;
            }
        }
        Ok(res)
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

    fn eval_on_events(&mut self, events: &ast::OnEvents) -> Value {
        // TODO: currently only supports for AutoConfig
        let mut nd = auto_val::Node::new("on");
        for branch in events.branches.iter() {
            let mut ev = auto_val::Node::new("ev");
            match branch {
                Event::Arrow(arrow) => {
                    if let Some(src) = &arrow.src {
                        ev.set_prop("src", src.to_code());
                    } else {
                        ev.set_prop("src", "DEFAULT");
                    }
                    if let Some(dest) = &arrow.dest {
                        ev.set_prop("dest", dest.to_code());
                    } else {
                        ev.set_prop("dest", "None");
                    }
                    if let Some(handler) = &arrow.with {
                        ev.set_prop("with", handler.to_code());
                    } else {
                        ev.set_prop("with", "()");
                    }
                    nd.add_kid(ev);
                }
                Event::CondArrow(cond) => {
                    let src = if let Some(src) = &cond.src {
                        src.to_code()
                    } else {
                        "DEFAULT".into()
                    };
                    ev.set_prop("src", src.clone());
                    ev.set_prop("dest", "CONDITION");
                    ev.set_prop("with", cond.cond.to_code());
                    nd.add_kid(ev);
                    for arrow in cond.subs.iter() {
                        // println!("NEWSUB!!!! {}", arrow.with.clone().unwrap().to_code()); // LSP: disabled
                        let mut sub = auto_val::Node::new("ev");
                        sub.set_prop("src", src.clone());
                        if let Some(dest) = &arrow.dest {
                            sub.set_prop("dest", dest.to_code());
                        } else {
                            sub.set_prop("dest", "DEFAULT");
                        }
                        if let Some(handler) = &arrow.with {
                            sub.set_prop("with", handler.to_code());
                        } else {
                            sub.set_prop("with", "()");
                        }
                        nd.add_kid(sub);
                    }
                }
            }
        }
        Value::Node(nd)
    }

    // TODO: should node only be used in config mode?
    pub fn eval_node(&mut self, node: &Node) -> AutoResult<Value> {
        let name = node.name.clone();
        let expr = Expr::Ident(name);
        let name_expr = self.eval_expr(&expr);
        let args = self.eval_args(&node.args);
        if let Value::Type(Type::User(type_decl)) = name_expr {
            // println!("EVAL TYPE _NEWNWNWN"); // LSP: disabled
            return Ok(self.eval_type_new(&type_decl, &args));
        }

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
                    let val = self.eval_stmt(stmt)?;
                    match val {
                        Value::Str(s) => {
                            let mut n = auto_val::Node::new("text");
                            n.text = s.clone();
                            // NEW: Use kids API with node name as key
                            // Will be added to kids after nd is created
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
                        Value::Array(arr) | Value::Block(arr) => {
                            for item in arr.values.into_iter() {
                                match item {
                                    Value::Pair(key, value) => {
                                        props.set(key, *value);
                                    }
                                    Value::Obj(o) => {
                                        props.merge(&o);
                                    }
                                    Value::Node(n) => {
                                        nodes.push(n);
                                    }
                                    Value::Instance(inst) => {
                                        // Convert instance to node with type name as node name
                                        let mut kid_node = auto_val::Node::new(&inst.ty.name());
                                        // Add instance fields as node properties
                                        for (k, v) in inst.fields.iter() {
                                            kid_node.set_prop(k.clone(), v.clone());
                                        }
                                        nodes.push(kid_node);
                                    }
                                    _ => {
                                        props.set(item.to_astr(), item);
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
                // println!("define global {}", name); // LSP: disabled
                self.universe
                    .borrow_mut()
                    .define_global(&name, Rc::new(Meta::Body(node.body.clone())));
            }
        }
        let mut nd = auto_val::Node::new(name);
        // id is not specified, try use first argument as id
        if !node.id.is_empty() {
            nd.id = node.id.clone();
        } else {
            let first_arg = node.args.first_arg();
            if let Some(Expr::Ident(ident)) = first_arg {
                let v = self.eval_ident(&ident);
                let v = self.universe.borrow().deref_val(v);
                if let Value::Str(s) = v {
                    nd.id = s;
                }
            }
        }
        let ndid = nd.id.clone();
        nd.args = args.clone(); // Keep for backward compatibility

        // NEW: Populate unified props with args
        nd.num_args = args.args.len();
        for arg in args.args.iter() {
            match arg {
                auto_val::Arg::Pos(expr) => {
                    // Positional arg: use empty key
                    nd.set_prop("", expr.clone());
                }
                auto_val::Arg::Name(name) => {
                    // Named arg: use name as key with Str value
                    nd.set_prop(name.as_str(), Value::Str(name.clone()));
                }
                auto_val::Arg::Pair(key, _) => {
                    // Pair arg: extract key and value
                    match key {
                        auto_val::ValueKey::Str(k) => {
                            nd.set_prop(k.as_str(), arg.get_val());
                        }
                        auto_val::ValueKey::Int(i) => {
                            nd.set_prop(i.to_string(), arg.get_val());
                        }
                        auto_val::ValueKey::Bool(b) => {
                            nd.set_prop(b.to_string(), arg.get_val());
                        }
                    }
                }
            }
        }

        nd.merge_obj(props);

        // NEW: Use kids API instead of nodes and body_ref
        // Use integer indices as keys to preserve order and allow duplicates
        for (idx, node) in nodes.iter().enumerate() {
            nd.add_node_kid(idx as i32, node.clone());
        }
        if body != MetaID::Nil {
            nd.set_kids_ref(body.clone());
        }

        let nd = Value::Node(nd);
        // save value to scope
        if !ndid.is_empty() {
            self.universe.borrow_mut().set_global(ndid, nd.clone());
        }

        Ok(nd)
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
                    // Resolve ValueRef before converting to string
                    Value::ValueRef(_vid) => {
                        if let Some(data) = self.resolve_value(&val) {
                            let borrowed_data = data.borrow();
                            let data_clone = borrowed_data.clone();
                            drop(borrowed_data);
                            let resolved_val = Value::from_data(data_clone);
                            resolved_val.to_astr()
                        } else {
                            val.to_astr()
                        }
                    }
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
        scope::Meta::Node(nd) => MetaID::Node(nd.id.clone()),
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
        ast::Type::Uint => auto_val::Type::Uint,
        ast::Type::USize => auto_val::Type::Uint, // TODO: should be U64?
        ast::Type::Float => auto_val::Type::Float,
        ast::Type::Double => auto_val::Type::Double,
        ast::Type::Bool => auto_val::Type::Bool,
        ast::Type::Char => auto_val::Type::Char,
        ast::Type::Str(_) => auto_val::Type::Str,
        ast::Type::CStr => auto_val::Type::CStr,
        ast::Type::Array(_) => auto_val::Type::Array,
        ast::Type::Ptr(_) => auto_val::Type::Ptr,
        ast::Type::User(type_decl) => auto_val::Type::User(type_decl.name.clone()),
        ast::Type::Enum(decl) => auto_val::Type::Enum(decl.borrow().name.clone()),
        ast::Type::Spec(decl) => auto_val::Type::User(decl.borrow().name.clone()),
        ast::Type::Union(u) => auto_val::Type::Union(u.name.clone()),
        ast::Type::Tag(tag) => auto_val::Type::Tag(tag.borrow().name.clone()),
        ast::Type::Linear(inner) => to_value_type(inner),  // Linear wraps inner type
        ast::Type::Void => auto_val::Type::Void,
        ast::Type::Unknown => auto_val::Type::Any,
        ast::Type::CStruct(_) => auto_val::Type::Void,
    }
}

pub fn eval_basic_expr(expr: &Expr) -> Value {
    match expr {
        Expr::Str(s) => Value::Str(s.clone().into()),
        Expr::Byte(b) => Value::Byte(*b),
        Expr::Int(i) => Value::Int(*i),
        Expr::Float(f, _) => Value::Float(*f),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Char(c) => Value::Char(*c),
        _ => Value::error(format!("Unsupported basic expression: {:?}", expr)),
    }
}
