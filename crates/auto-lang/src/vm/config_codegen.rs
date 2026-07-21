// Plan 075 Phase 1: ConfigCodegen Implementation
// Compiles config files to bytecode that builds nested object structures

use crate::ast::{Code, Stmt, Store, Expr};
use crate::vm::codegen::Codegen;
use crate::vm::opcode::OpCode;
use crate::vm::loader::Module;
use crate::error::{AutoResult, AutoError};
use auto_val::{Op, ValueKey};
use std::collections::HashMap;

/// ConfigCodegen transforms configuration files into bytecode that builds
/// a unified object structure.
///
/// Input (config.at):
/// ```auto
/// server: { host: "localhost", port: 8080 }
/// database: { name: "mydb" }
/// debug: true
/// ```
///
/// Output: bytecode that creates a single object with all fields:
/// ```text
/// LOAD_STR "localhost"
/// LOAD_CONST 8080
/// LOAD_STR "mydb"
/// CONST_1  // true
/// CREATE_OBJ keys=["server", "database", "debug"]
/// RET
/// ```
/// A literal value extracted from a config expression for compile-time
/// condition evaluation (Plan 364 Step 2). Only the types used in manifest
/// `if` guards are supported: string / int / bool.
#[derive(Debug, Clone)]
enum ConstVal {
    Str(String),
    Int(i32),
    Bool(bool),
}

pub struct ConfigCodegen {
    /// Base codegen for opcode emission
    base: Codegen,
    /// Collected field paths (e.g., ["server.host", "debug"])
    field_paths: Vec<String>,
    /// Collected field values (expressions to compile)
    field_values: Vec<Expr>,
    /// Evaluation-time variables (Plan 364 Step 2).
    ///
    /// `var x = expr` records `variables["x"] = expr` WITHOUT emitting a field.
    /// References (`Expr::Ident("x")`) are substituted with the recorded value
    /// at collect time via `substitute_vars`. This mirrors how old auto-man 0.1.3
    /// treated manifest `var`s: pure substitution, not config root fields.
    variables: HashMap<String, Expr>,
}

impl ConfigCodegen {
    /// Create a new ConfigCodegen instance
    pub fn new() -> Self {
        Self {
            base: Codegen::new(),
            field_paths: Vec::new(),
            field_values: Vec::new(),
            variables: HashMap::new(),
        }
    }

    /// Define an evaluation-time variable before compilation (Plan 364 Step 2).
    ///
    /// Used to inject manifest context variables like `port` so that
    /// `if port == "win32"` guards resolve at flatten time. This mirrors old
    /// auto-man 0.1.3's `env.set_global("port", port_name)`.
    pub fn define_var(&mut self, name: &str, value: Expr) {
        self.variables.insert(name.to_string(), value);
    }

    /// Compile config file to bytecode
    ///
    /// Collects all field assignments and creates a single object.
    pub fn compile_config(&mut self, code: &Code) -> AutoResult<()> {
        // Phase 0 (Plan 364 Step 2): Pre-evaluate control flow and variables.
        // Flatten `if`/`for`/`var` into a flat list of data statements at the
        // AST level, so the rest of the pipeline stays purely declarative.
        let flat = self.flatten_config_stmts(&code.stmts)?;

        // Phase 1: Collect all field assignments
        for stmt in &flat {
            self.collect_config_stmt(stmt)?;
        }

        // Phase 2: Compile field values (in normal order so they're pushed correctly)
        for expr in self.field_values.iter() {
            self.base.compile_expr(expr)?;
        }

        // Phase 3: Create object with all fields
        if !self.field_paths.is_empty() {
            self.create_config_object()?;
        }

        // Return the config object
        // RET instruction: opcode (1 byte) + n_args (1 byte)
        self.base.code.push(OpCode::RET as u8);
        self.base.code.push(0); // n_args = 0 for config return

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Plan 364 Step 2: AST pre-evaluation (flatten)
    // -------------------------------------------------------------------------

    /// Flatten a statement list by pre-evaluating `if`/`for`/`var` into plain
    /// data statements (Store/Expr/Node/EmptyLine).
    ///
    /// `if` selects a branch via [`eval_const_cond`]; `for` unrolls a literal
    /// iterator; `var` records a substitution without emitting a field.
    fn flatten_config_stmts(&mut self, stmts: &[Stmt]) -> AutoResult<Vec<Stmt>> {
        let mut out = Vec::new();
        for stmt in stmts {
            self.flatten_one(stmt, &mut out)?;
        }
        Ok(out)
    }

    fn flatten_one(&mut self, stmt: &Stmt, out: &mut Vec<Stmt>) -> AutoResult<()> {
        match stmt {
            Stmt::If(if_stmt) => {
                // Pick the first branch whose condition evaluates truthy.
                let mut chosen: Option<&[Stmt]> = None;
                for branch in &if_stmt.branches {
                    if self.eval_const_cond(&branch.cond)? {
                        chosen = Some(&branch.body.stmts);
                        break;
                    }
                }
                let body_stmts = match chosen {
                    Some(s) => s,
                    None => match &if_stmt.else_ {
                        Some(body) => &body.stmts[..],
                        None => return Ok(()),
                    },
                };
                // Recurse: branch body may itself contain if/for/var.
                for inner in body_stmts {
                    self.flatten_one(inner, out)?;
                }
            }
            Stmt::For(for_stmt) => {
                // Only literal iterators are supported in config mode.
                let iter_name = match &for_stmt.iter {
                    crate::ast::Iter::Named(name) => name.to_string(),
                    _ => {
                        return Err(AutoError::Msg(format!(
                            "Config mode only supports `for x in <literal>`; got {:?}",
                            for_stmt.iter
                        )));
                    }
                };
                let items = self.eval_literal_iter(&for_stmt.range)?;
                for item in items {
                    self.variables.insert(iter_name.clone(), item);
                    for inner in &for_stmt.body.stmts {
                        self.flatten_one(inner, out)?;
                    }
                }
                self.variables.remove(&iter_name);
            }
            Stmt::Store(store) => {
                use crate::ast::StoreKind;
                match store.kind {
                    StoreKind::Var => {
                        // Record substitution; do not emit a field.
                        let expr = self.substitute_vars(&store.expr);
                        self.variables
                            .insert(store.name.to_string(), expr);
                    }
                    StoreKind::Let | StoreKind::Const => {
                        // Treat plain `let`/`const` assignments as variables too
                        // (manifests sometimes use `let`), AND as a field if it
                        // looks like a config value. To stay safe and match the
                        // pre-Step-2 behavior for `let`, record it as a variable.
                        let expr = self.substitute_vars(&store.expr);
                        self.variables.insert(store.name.to_string(), expr);
                    }
                    _ => {
                        out.push(stmt.clone());
                    }
                }
            }
            // Data statements pass through unchanged.
            Stmt::EmptyLine(_)
            | Stmt::Expr(_)
            | Stmt::Node(_)
            | Stmt::Comment(_)
            | Stmt::Dep(_) => {
                out.push(stmt.clone());
            }
            _ => {
                return Err(AutoError::Msg(format!(
                    "Config mode does not support statement: {:?}",
                    stmt
                )));
            }
        }
        Ok(())
    }

    /// Evaluate a condition expression to a boolean at compile time.
    ///
    /// Supports the subset needed by manifest `if port == "win32"` guards:
    /// - `Expr::Bool(b)` → b
    /// - `Expr::Bina(lhs, Op::Eq|Op::Neq, rhs)` → string/int/bool equality
    /// - `Expr::Ident(name)` → resolve via `variables`, then recurse
    fn eval_const_cond(&self, expr: &Expr) -> AutoResult<bool> {
        match expr {
            Expr::Bool(b) => Ok(*b),
            Expr::Bina(lhs, op, rhs) => {
                let lv = self.const_value(lhs)?;
                let rv = self.const_value(rhs)?;
                match op {
                    Op::Eq => Ok(Self::values_equal(&lv, &rv)),
                    Op::Neq => Ok(!Self::values_equal(&lv, &rv)),
                    other => Err(AutoError::Msg(format!(
                        "Config mode condition only supports == / !=, got {:?}",
                        other
                    ))),
                }
            }
            Expr::Ident(name) => {
                if let Some(val) = self.variables.get(name.as_str()) {
                    self.eval_const_cond(val)
                } else {
                    Err(AutoError::Msg(format!(
                        "Config mode: undefined variable in condition: {}",
                        name
                    )))
                }
            }
            _ => Err(AutoError::Msg(format!(
                "Config mode: cannot evaluate condition at compile time: {:?}",
                expr
            ))),
        }
    }

    /// Resolve an expression to a comparable literal value (after substitution).
    /// Returns a small `ConstVal` so equality is straightforward.
    fn const_value(&self, expr: &Expr) -> AutoResult<ConstVal> {
        match expr {
            Expr::Str(s) => Ok(ConstVal::Str(s.to_string())),
            Expr::Int(i) => Ok(ConstVal::Int(*i)),
            Expr::Bool(b) => Ok(ConstVal::Bool(*b)),
            Expr::Ident(name) => {
                if let Some(val) = self.variables.get(name.as_str()) {
                    self.const_value(val)
                } else {
                    Err(AutoError::Msg(format!(
                        "Config mode: undefined variable: {}",
                        name
                    )))
                }
            }
            other => Err(AutoError::Msg(format!(
                "Config mode: non-literal value in condition: {:?}",
                other
            ))),
        }
    }

    fn values_equal(a: &ConstVal, b: &ConstVal) -> bool {
        match (a, b) {
            (ConstVal::Str(x), ConstVal::Str(y)) => x == y,
            (ConstVal::Int(x), ConstVal::Int(y)) => x == y,
            (ConstVal::Bool(x), ConstVal::Bool(y)) => x == y,
            _ => false,
        }
    }

    /// Unroll a literal iterator (`[a, b, c]` of string/int, or a bare string
    /// iterated char-by-char is NOT supported — only arrays).
    fn eval_literal_iter(&mut self, range: &Expr) -> AutoResult<Vec<Expr>> {
        match range {
            Expr::Array(items) => {
                // Substitute each item so referenced vars resolve now.
                Ok(items.iter().map(|e| self.substitute_vars(e)).collect())
            }
            _ => Err(AutoError::Msg(format!(
                "Config mode only supports literal array iterators, got {:?}",
                range
            ))),
        }
    }

    /// Recursively replace `Expr::Ident(name)` with the recorded variable
    /// value when `name` exists in `variables`. Returns a clone with
    /// substitutions applied at every level (Object fields, Array items, etc.).
    fn substitute_vars(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Ident(name) => match self.variables.get(name.as_str()) {
                Some(val) => val.clone(),
                None => expr.clone(),
            },
            // Recurse into composite expressions so nested idents resolve too.
            Expr::Bina(l, op, r) => Expr::Bina(
                Box::new(self.substitute_vars(l)),
                *op,
                Box::new(self.substitute_vars(r)),
            ),
            Expr::Unary(op, e) => Expr::Unary(*op, Box::new(self.substitute_vars(e))),
            Expr::Array(items) => {
                Expr::Array(items.iter().map(|e| self.substitute_vars(e)).collect())
            }
            Expr::Object(pairs) => {
                let pairs = pairs
                    .iter()
                    .map(|p| crate::ast::Pair {
                        key: p.key.clone(),
                        value: Box::new(self.substitute_vars(&p.value)),
                    })
                    .collect();
                Expr::Object(pairs)
            }
            Expr::Pair(p) => Expr::Pair(crate::ast::Pair {
                key: p.key.clone(),
                value: Box::new(self.substitute_vars(&p.value)),
            }),
            Expr::Dot(obj, name) => {
                Expr::Dot(Box::new(self.substitute_vars(obj)), name.clone())
            }
            _ => expr.clone(),
        }
    }

    /// Collect field assignments from statements
    fn collect_config_stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            // Ignore empty lines
            Stmt::EmptyLine(_) => {
                // Do nothing
            }
            // Parse field assignments: server.host = "localhost"
            Stmt::Store(store) => {
                self.collect_store_field(store)?;
            }
            // Evaluate expressions and add to config
            Stmt::Expr(expr) => {
                self.collect_expr_field(expr)?;
            }
            // Node statements (like app("name") {...}) are treated as expressions
            Stmt::Node(node) => {
                // Convert Node to Expr::Node and collect it
                let node_expr = crate::ast::Expr::Node(node.clone());
                self.collect_expr_field(&node_expr)?;
            }
            _ => {
                return Err(AutoError::Msg(
                    format!("Config mode does not support statement: {:?}", stmt)
                ));
            }
        }
        Ok(())
    }

    /// Collect a store statement as a field assignment
    fn collect_store_field(&mut self, store: &Store) -> AutoResult<()> {
        // Use the full dotted name as the field path
        // e.g., "server.host" stays as "server.host"
        let field_path = store.name.to_string();

        // Substitute any evaluation-time variables referenced in the value
        // (Plan 364 Step 2: e.g. `kernel: kernel_config` → resolved value).
        let expr = self.substitute_vars(&store.expr);

        // Track this field. A later assignment to the same path overrides the
        // earlier one (mirrors object semantics: `{ a: 1, a: 2 }` → a == 2).
        if let Some(pos) = self.field_paths.iter().position(|p| *p == field_path) {
            self.field_values[pos] = expr;
        } else {
            self.field_paths.push(field_path);
            self.field_values.push(expr);
        }

        Ok(())
    }

    /// Collect an expression statement as an anonymous field (or named if Pair)
    fn collect_expr_field(&mut self, expr: &Expr) -> AutoResult<()> {
        let (field_name, expr) = if let Expr::Pair(pair) = expr {
            // Unpack pair: key: value -> map key to field_name
            let key_str = match &pair.key {
                crate::ast::Key::NamedKey(name) => name.to_string(),
                crate::ast::Key::StrKey(s) => s.to_string(),
                _ => format!("_expr{}", self.field_values.len()), // Fallback
            };
            (key_str, *pair.value.clone())
        } else {
            // Generate anonymous field name for other expressions
            (format!("_expr{}", self.field_values.len()), expr.clone())
        };

        // Substitute evaluation-time variables in the value (Plan 364 Step 2).
        let expr = self.substitute_vars(&expr);

        // Track this field. Same-name override semantics as collect_store_field.
        if let Some(pos) = self.field_paths.iter().position(|p| *p == field_name) {
            self.field_values[pos] = expr;
        } else {
            self.field_paths.push(field_name);
            self.field_values.push(expr);
        }

        Ok(())
    }

    /// Create the config object with all collected fields
    fn create_config_object(&mut self) -> AutoResult<()> {
        // Register keys in object_keys pool
        let keys: Vec<ValueKey> = self.field_paths
            .iter()
            .map(|s| ValueKey::Str(s.clone().into()))
            .collect();

        let key_index = self.base.object_keys.len() as u16;
        self.base.object_keys.push(keys);

        // Plan 073: Infer field types from field values
        let types: Vec<crate::vm::codegen::ObjectType> = self.field_values.iter()
            .map(|expr| self.base.infer_object_type(expr))
            .collect();
        self.base.object_types.push(types);

        // Emit CREATE_OBJ with key_index and field count
        let field_count = self.field_paths.len() as u8;
        self.base.code.push(OpCode::CREATE_OBJ as u8);
        self.base.code.extend_from_slice(&key_index.to_le_bytes());
        self.base.code.push(field_count);

        Ok(())
    }

    /// Finish compilation and return the module
    pub fn finish(self, name: String) -> Module {
        self.base.finish(name)
    }

    /// Get the base codegen for advanced usage
    pub fn base(&mut self) -> &mut Codegen {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_source(source: &str) -> Code {
        let mut parser = Parser::from(source);
        parser.parse().unwrap()
    }

    #[test]
    fn test_config_codegen_simple_fields() {
        // Auto Config uses colon syntax (JSON/Atom style)
        let source = r#"
host: "localhost"
port: 8080
debug: true
"#;

        let code = parse_source(source);
        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Verify bytecode contains expected opcodes
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");

        // Should have one CREATE_OBJ call with 3 fields
        let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
        assert_eq!(create_obj_count, 1, "Expected 1 CREATE_OBJ opcode");

        // Check field count (should be 3)
        if let Some(idx) = bytecode.iter().position(|&x| x == 0x2E) {
            let field_count = bytecode[idx + 3]; // +3 for opcode + 2-byte index
            assert_eq!(field_count, 3, "Expected 3 fields in object");
        }
    }

    #[test]
    fn test_config_codegen_nested_fields() {
        // Auto Config: nested objects use { } blocks with colon syntax
        let source = r#"
server: { host: "localhost", port: 5432 }
database: { name: "mydb" }
"#;

        let code = parse_source(source);
        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Verify bytecode was generated
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");

        // Should have at least one CREATE_OBJ for the top-level config
        let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
        assert!(create_obj_count >= 1, "Expected at least 1 CREATE_OBJ opcode");
    }

    #[test]
    fn test_config_codegen_with_expressions() {
        // Auto Config: fields use colon syntax
        let source = r#"
max_connections: 10
timeout: 30
"#;

        let code = parse_source(source);
        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Verify bytecode was generated
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");

        // Should have one CREATE_OBJ call with 2 fields
        let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
        assert_eq!(create_obj_count, 1, "Expected 1 CREATE_OBJ opcode");
    }

    #[test]
    fn test_config_codegen_empty_config() {
        let source = "";

        let mut parser = Parser::from(source);
        let code = parser.parse().unwrap();

        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Should have RET opcode but no CREATE_OBJ
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x71), "Expected RET opcode (0x71)");
        assert!(!bytecode.contains(&0x2E), "Should not have CREATE_OBJ for empty config");
    }

    // ---------------------------------------------------------------------
    // Plan 364 Step 2: AST pre-evaluation (if / var / for) — end-to-end
    // ---------------------------------------------------------------------
    //
    // These tests go through the full pipeline (parser → ConfigCodegen →
    // AutoVM) via `AutoConfig::from_code`, so they verify real behavior,
    // not just bytecode shape. `port` is injected exactly like `auto build`
    // does for manifest evaluation.

    use crate::config::AutoConfig;
    use auto_val::Obj;

    fn eval_with_port(source: &str, port: &str) -> AutoConfig {
        let mut args = Obj::new();
        args.set("port", auto_val::Value::str(port));
        AutoConfig::from_code(source, &args).unwrap()
    }

    #[test]
    fn test_config_if_else_picks_matching_branch() {
        // port == "win32" → x = 1
        let source = r#"
if port == "win32" {
    x: 1
} else {
    x: 2
}
"#;
        let cfg = eval_with_port(source, "win32");
        assert_eq!(cfg.root.get_prop("x").to_astr().as_str(), "1");

        // port == "linux" → else → x = 2
        let cfg = eval_with_port(source, "linux");
        assert_eq!(cfg.root.get_prop("x").to_astr().as_str(), "2");
    }

    #[test]
    fn test_config_var_declaration_and_reference() {
        // Mirrors SCU001's `var kernel_config = {...}` then `kernel: kernel_config`.
        let source = r#"
var kernel_config = { mode: "lockstep", mpu: true }
kernel: kernel_config
"#;
        let cfg = eval_with_port(source, "win32");
        // `kernel_config` itself must NOT be a root field (it's a variable).
        // `kernel` must resolve to the recorded object.
        let kernel = cfg.root.get_prop("kernel");
        let repr = kernel.repr().to_string();
        assert!(repr.contains("lockstep"),
            "kernel should resolve to the var's object value (containing 'lockstep'), got {}",
            repr);
    }

    #[test]
    fn test_config_var_redeclared_takes_latest() {
        // Re-declaring the same var before reference takes the latest value.
        let source = r#"
var arch = "arm"
var arch = "armv7"
target: arch
"#;
        let cfg = eval_with_port(source, "win32");
        assert_eq!(cfg.root.get_prop("target").to_astr().as_str(), "armv7");
    }

    #[test]
    fn test_config_for_unrolls_literal_array() {
        let source = r#"
ports: [8080, 9090]
"#;
        // This is a baseline: array literal as a field value.
        let cfg = eval_with_port(source, "win32");
        let ports = cfg.root.get_prop("ports");
        assert!(matches!(ports, auto_val::Value::Array(_)),
            "ports should be an array, got {:?}", ports);
    }

    #[test]
    fn test_config_if_in_nested_manifest_style() {
        // SCU001-style: top-level data plus a port-guarded block.
        let source = r#"
app: "SCU001"
if port == "win32" {
    builder: "iar"
    toolchain: "arm"
} else {
    builder: "make"
}
"#;
        let cfg = eval_with_port(source, "win32");
        assert_eq!(cfg.root.get_prop("app").to_astr().as_str(), "SCU001");
        assert_eq!(cfg.root.get_prop("builder").to_astr().as_str(), "iar");
        assert_eq!(cfg.root.get_prop("toolchain").to_astr().as_str(), "arm");

        let cfg2 = eval_with_port(source, "linux");
        assert_eq!(cfg2.root.get_prop("builder").to_astr().as_str(), "make");
        // toolchain should not exist when else branch taken (empty/nil)
        assert_ne!(cfg2.root.get_prop("toolchain").to_astr().as_str(), "arm");
    }
}
