//! AST Transformer for Compile-Time Execution - Plan 095
//!
//! Transforms AST by evaluating `#if`, `#for`, `#is`, and `#{}` constructs.
//! Uses VmInterpreter for expression evaluation.

use crate::ast::{Code, Expr, HashBrace, HashFor, HashIf, HashIfElse, HashIs, HashIsBranch, Stmt};
use crate::error::{AutoResult, ComptimeError};
use crate::interpreter::VmInterpreter;
use auto_val::Value;
use miette::SourceSpan;
use std::collections::HashMap;

/// Compile-Time Execution Engine
///
/// Transforms the AST by evaluating compile-time constructs.
/// Uses an embedded `VmInterpreter` for expression evaluation.
pub struct CTEE {
    /// Embedded VM interpreter for expression evaluation
    vm: VmInterpreter,
    /// Built-in compile-time constants (OS, ARCH, DEBUG, etc.)
    builtins: HashMap<String, Value>,
    /// Target platform OS
    target_os: String,
    /// Target platform architecture
    target_arch: String,
}

impl Default for CTEE {
    fn default() -> Self {
        Self::new()
    }
}

impl CTEE {
    /// Create a new CTEE with default settings
    pub fn new() -> Self {
        let mut ctee = Self {
            vm: VmInterpreter::new(),
            builtins: HashMap::new(),
            target_os: std::env::consts::OS.to_string(),
            target_arch: std::env::consts::ARCH.to_string(),
        };
        ctee.init_builtins();
        ctee
    }

    /// Create CTEE for a specific target platform
    pub fn with_target(os: &str, arch: &str) -> Self {
        let mut ctee = Self::new();
        ctee.target_os = os.to_string();
        ctee.target_arch = arch.to_string();
        ctee.init_builtins();
        ctee
    }

    /// Initialize built-in compile-time constants
    fn init_builtins(&mut self) {
        self.builtins.insert(
            "OS".to_string(),
            Value::Str(self.target_os.clone().into()),
        );
        self.builtins.insert(
            "ARCH".to_string(),
            Value::Str(self.target_arch.clone().into()),
        );
        self.builtins.insert("DEBUG".to_string(), Value::Bool(cfg!(debug_assertions)));
        self.builtins.insert("VERSION".to_string(), Value::Str("0.1.0".into()));
    }

    /// Get a built-in constant value
    pub fn get_builtin(&self, name: &str) -> Option<&Value> {
        self.builtins.get(name)
    }

    /// Set a custom compile-time constant
    pub fn set_builtin(&mut self, name: String, value: Value) {
        self.builtins.insert(name, value);
    }

    /// Transform AST by evaluating all comptime constructs
    pub fn transform(&mut self, code: &mut Code) -> AutoResult<()> {
        let mut new_stmts = Vec::new();
        for stmt in code.stmts.drain(..) {
            let transformed = self.transform_stmt(stmt)?;
            new_stmts.extend(transformed);
        }
        code.stmts = new_stmts;
        Ok(())
    }

    /// Transform a single statement
    fn transform_stmt(&mut self, stmt: Stmt) -> AutoResult<Vec<Stmt>> {
        match stmt {
            Stmt::HashIf(hash_if) => self.transform_hash_if(hash_if),
            Stmt::HashFor(hash_for) => self.transform_hash_for(hash_for),
            Stmt::HashIs(hash_is) => self.transform_hash_is(hash_is),
            Stmt::HashBrace(hash_brace) => self.transform_hash_brace(hash_brace),
            other => Ok(vec![other]),
        }
    }

    /// Transform #if - evaluate condition and keep only matching branch
    fn transform_hash_if(&mut self, hash_if: HashIf) -> AutoResult<Vec<Stmt>> {
        // Evaluate the condition
        let cond_value = self.eval_expr(&hash_if.cond)?;

        if Self::is_truthy(&cond_value) {
            // Keep then branch
            let mut result = Vec::new();
            for stmt in hash_if.then_block.stmts {
                let transformed = self.transform_stmt(stmt)?;
                result.extend(transformed);
            }
            Ok(result)
        } else if let Some(else_block) = hash_if.else_block {
            // Handle else branch
            match else_block {
                HashIfElse::Block(body) => {
                    let mut result = Vec::new();
                    for stmt in body.stmts {
                        let transformed = self.transform_stmt(stmt)?;
                        result.extend(transformed);
                    }
                    Ok(result)
                }
                HashIfElse::ElseIf(nested_if) => self.transform_hash_if(*nested_if),
            }
        } else {
            // No else branch, remove entire #if
            Ok(vec![])
        }
    }

    /// Transform #for - unroll loop at compile time
    fn transform_hash_for(&mut self, hash_for: HashFor) -> AutoResult<Vec<Stmt>> {
        // Evaluate the iterable
        let iter_value = self.eval_expr(&hash_for.iter)?;

        // Get iteration values
        let values = self.value_to_iter(&iter_value)?;

        // Unroll loop
        let var_name = hash_for.var.to_string();
        let mut result = Vec::new();

        for value in values {
            // Set loop variable as builtin
            self.builtins.insert(var_name.clone(), value);

            // Transform body with loop variable set
            for stmt in hash_for.body.stmts.clone() {
                let transformed = self.transform_stmt(stmt)?;
                result.extend(transformed);
            }
        }

        // Remove loop variable
        self.builtins.remove(&var_name);
        Ok(result)
    }

    /// Transform #is - pattern match at compile time
    fn transform_hash_is(&mut self, hash_is: HashIs) -> AutoResult<Vec<Stmt>> {
        let target_value = self.eval_expr(&hash_is.target)?;

        for branch in hash_is.branches {
            match branch {
                HashIsBranch::EqBranch(pattern, body) => {
                    let pattern_value = self.eval_expr(&pattern)?;
                    if Self::values_equal(&target_value, &pattern_value) {
                        let mut result = Vec::new();
                        for stmt in body.stmts {
                            let transformed = self.transform_stmt(stmt)?;
                            result.extend(transformed);
                        }
                        return Ok(result);
                    }
                }
                HashIsBranch::IfBranch(cond, body) => {
                    let cond_value = self.eval_expr(&cond)?;
                    if Self::is_truthy(&cond_value) {
                        let mut result = Vec::new();
                        for stmt in body.stmts {
                            let transformed = self.transform_stmt(stmt)?;
                            result.extend(transformed);
                        }
                        return Ok(result);
                    }
                }
                HashIsBranch::ElseBranch(body) => {
                    let mut result = Vec::new();
                    for stmt in body.stmts {
                        let transformed = self.transform_stmt(stmt)?;
                        result.extend(transformed);
                    }
                    return Ok(result);
                }
            }
        }

        // No match found
        Ok(vec![])
    }

    /// Transform #{} - evaluate expression and substitute result
    fn transform_hash_brace(&mut self, hash_brace: HashBrace) -> AutoResult<Vec<Stmt>> {
        let value = self.eval_expr(&hash_brace.expr)?;
        let expr = self.value_to_expr(&value);
        Ok(vec![Stmt::Expr(expr)])
    }

    /// Evaluate an expression at compile time using VmInterpreter
    fn eval_expr(&mut self, expr: &Expr) -> AutoResult<Value> {
        // Check for compile_error() call - Plan 095 Task 5.2
        if let Expr::Call(call) = expr {
            if let Expr::Ident(name) = call.name.as_ref() {
                if name.to_string() == "compile_error" {
                    // Extract error message from arguments
                    let msg = if call.args.is_empty() {
                        "compile error".to_string()
                    } else if let Some(arg) = call.args.get(0) {
                        let arg_expr = arg.get_expr();
                        if let Expr::Str(s) = arg_expr {
                            s.to_string()
                        } else {
                            arg.repr().to_string()
                        }
                    } else {
                        "compile error".to_string()
                    };
                    return Err(ComptimeError::CompileError {
                        message: msg,
                        span: SourceSpan::new(0usize.into(), 0usize.into()),
                    }.into());
                }
            }
        }

        // For simple expressions, handle directly for performance
        match expr {
            // Literals
            Expr::Int(i) => return Ok(Value::Int(*i)),
            Expr::I64(i) => return Ok(Value::Int(*i as i32)), // Truncate for now
            Expr::Bool(b) => return Ok(Value::Bool(*b)),
            Expr::Str(s) => return Ok(Value::Str(s.clone())),
            Expr::Nil | Expr::Null => return Ok(Value::Nil),

            // Identifier lookup (check builtins first)
            Expr::Ident(name) => {
                let name_str = name.to_string();
                if let Some(value) = self.builtins.get(&name_str) {
                    return Ok(value.clone());
                }
                // Fall through to VM evaluation
            }

            _ => {}
        }

        // For complex expressions, convert to code and run via VmInterpreter
        let code = format!("{}\n", expr);
        self.vm.run(&code)
    }

    /// Check if a value is truthy
    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Nil => false,
            _ => true,
        }
    }

    /// Check if two values are equal
    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(ai), Value::Int(bi)) => ai == bi,
            (Value::Bool(ab), Value::Bool(bb)) => ab == bb,
            (Value::Str(as_), Value::Str(bs)) => as_ == bs,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    /// Convert a Value to an iterator of Values
    fn value_to_iter(&self, value: &Value) -> AutoResult<Vec<Value>> {
        match value {
            Value::Array(_arr) => {
                // Extract values from AutoVal array
                // For now, return empty (needs proper array iteration)
                Ok(vec![])
            }
            Value::Int(end) => Ok((0..*end).map(Value::Int).collect()),
            _ => {
                let msg = format!("Cannot iterate over {:?}", value);
                Err(crate::error::SyntaxError::Generic {
                    message: msg,
                    span: miette::SourceSpan::new(0usize.into(), 0usize.into()),
                }.into())
            }
        }
    }

    /// Convert a Value back to an Expr literal
    fn value_to_expr(&self, value: &Value) -> Expr {
        match value {
            Value::Nil => Expr::Nil,
            Value::Int(i) => Expr::I64(*i as i64),
            Value::Bool(b) => Expr::Bool(*b),
            Value::Str(s) => Expr::Str(s.clone()),
            Value::Float(f) => Expr::Double(*f as f64, "".into()),
            _ => Expr::Nil,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_constants() {
        let ctee = CTEE::new();
        assert!(ctee.get_builtin("OS").is_some());
        assert!(ctee.get_builtin("ARCH").is_some());
        assert!(ctee.get_builtin("DEBUG").is_some());
        assert!(ctee.get_builtin("VERSION").is_some());
    }

    #[test]
    fn test_is_truthy() {
        assert!(CTEE::is_truthy(&Value::Bool(true)));
        assert!(!CTEE::is_truthy(&Value::Bool(false)));
        assert!(CTEE::is_truthy(&Value::Int(1)));
        assert!(!CTEE::is_truthy(&Value::Int(0)));
        assert!(CTEE::is_truthy(&Value::Str("hello".into())));
        assert!(!CTEE::is_truthy(&Value::Str("".into())));
        assert!(!CTEE::is_truthy(&Value::Nil));
    }

    #[test]
    fn test_values_equal() {
        assert!(CTEE::values_equal(&Value::Int(42), &Value::Int(42)));
        assert!(!CTEE::values_equal(&Value::Int(42), &Value::Int(43)));
        assert!(CTEE::values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(CTEE::values_equal(
            &Value::Str("hello".into()),
            &Value::Str("hello".into())
        ));
    }

    #[test]
    fn test_compile_error_intrinsic() {
        // Test compile_error() by parsing code that calls it
        use crate::parser::Parser;

        let mut ctee = CTEE::new();

        // Parse: compile_error("unsupported platform")
        let mut parser = Parser::from("compile_error(\"unsupported platform\")");
        let ast = parser.parse().unwrap();

        // Get the expression from the first statement
        if let Some(crate::ast::Stmt::Expr(expr)) = ast.stmts.first() {
            let result = ctee.eval_expr(expr);
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("compile_error"));
            assert!(err_msg.contains("unsupported platform"));
        } else {
            panic!("Expected Expr statement");
        }
    }
}
