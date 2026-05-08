//! Plan 125: Phase 3.3 - Implicit Union Generator
//!
//! This module generates implicit union types from task on-block patterns.
//!
//! ## Overview
//!
//! When a task defines an `on` block with mixed patterns:
//!
//! ```auto
//! on(ctx) {
//!     "ping" => { ctx.reply("pong") }
//!     msg string => { write_to_disk(msg) }
//!     amount int if amount > 10000 => { approve(amount) }
//! }
//! ```
//!
//! The implicit union generator creates an envelope type:
//!
//! ```rust
//! enum TaskEnvelope {
//!     LiteralPing,
//!     StringValue(String),
//!     IntValue(i64),
//! }
//! ```

use crate::ast::{LiteralValue, Name, TaskMsgPattern, TaskOnBlock, Type};
use auto_val::AutoStr;
use std::fmt;

/// Information about an implicit union type
#[derive(Debug, Clone)]
pub struct ImplicitUnionInfo {
    /// The task name (used to generate envelope name)
    pub task_name: AutoStr,
    /// Literal patterns collected from handlers
    pub literals: Vec<LiteralValue>,
    /// Type binding patterns collected from handlers
    pub type_bindings: Vec<(Name, Type)>,
    /// The generated envelope name
    pub envelope_name: AutoStr,
}

impl ImplicitUnionInfo {
    /// Create a new ImplicitUnionInfo for a task
    pub fn new(task_name: &str) -> Self {
        Self {
            task_name: task_name.into(),
            literals: Vec::new(),
            type_bindings: Vec::new(),
            envelope_name: format!("{}Envelope", task_name).into(),
        }
    }

    /// Create from an on-block's patterns
    pub fn from_on_block(task_name: &str, on_block: &TaskOnBlock) -> Self {
        let mut info = Self::new(task_name);

        for (pattern, _guard, _body) in &on_block.handlers {
            info.add_pattern(pattern);
        }

        info
    }

    /// Add a pattern to the union info
    pub fn add_pattern(&mut self, pattern: &TaskMsgPattern) {
        match pattern {
            TaskMsgPattern::Literal(lit) => {
                self.add_literal(lit.clone());
            }
            TaskMsgPattern::TypeBinding { name, type_expr } => {
                self.add_type_binding(name.clone(), type_expr.as_ref().clone());
            }
            // Phase 1/2 patterns are not part of the implicit union
            TaskMsgPattern::Simple(_) | TaskMsgPattern::WithBindings { .. } => {}
        }
    }

    /// Add a literal pattern (deduplicated)
    pub fn add_literal(&mut self, lit: LiteralValue) {
        if !self.literals.contains(&lit) {
            self.literals.push(lit);
        }
    }

    /// Add a type binding pattern (deduplicated by name)
    pub fn add_type_binding(&mut self, name: Name, type_expr: Type) {
        if !self.type_bindings.iter().any(|(n, _)| n == &name) {
            self.type_bindings.push((name, type_expr));
        }
    }

    /// Check if the union is empty (no patterns)
    pub fn is_empty(&self) -> bool {
        self.literals.is_empty() && self.type_bindings.is_empty()
    }

    /// Get the total number of variants
    pub fn variant_count(&self) -> usize {
        self.literals.len() + self.type_bindings.len()
    }

    /// Generate a variant name for a literal value
    pub fn literal_to_variant_name(lit: &LiteralValue) -> String {
        match lit {
            LiteralValue::String(s) => {
                // Convert string to PascalCase variant name
                let pascal = to_pascal_case(s.as_str());
                format!("Literal{}", pascal)
            }
            LiteralValue::Int(n) => format!("LiteralInt{}", n),
            LiteralValue::Uint(n) => format!("LiteralUint{}", n),
            LiteralValue::Float(i, frac) => format!("LiteralFloat{}{}", i, frac),
            LiteralValue::Bool(b) => format!("Literal{}", if *b { "True" } else { "False" }),
            LiteralValue::Char(c) => format!("LiteralChar{}", c),
        }
    }

    /// Generate a variant name for a type binding
    pub fn type_binding_to_variant_name(name: &Name) -> String {
        to_pascal_case(name.as_str())
    }

    /// Generate Rust enum definition
    pub fn generate_rust_enum(&self) -> String {
        if self.is_empty() {
            return format!("// No implicit union needed for {}\n", self.task_name);
        }

        let mut code = format!("pub enum {} {{\n", self.envelope_name);

        // Generate literal variants
        for lit in &self.literals {
            let variant_name = Self::literal_to_variant_name(lit);
            code.push_str(&format!("    {},\n", variant_name));
        }

        // Generate type binding variants
        for (name, type_expr) in &self.type_bindings {
            let variant_name = Self::type_binding_to_variant_name(name);
            let rust_type = type_to_rust_type(type_expr);
            code.push_str(&format!("    {}({}),\n", variant_name, rust_type));
        }

        code.push_str("}\n");
        code
    }

    /// Generate the message envelope struct
    pub fn generate_envelope_struct(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        format!(
            r#"/// Message envelope for {} task
pub struct {}Message {{
    /// Message context (sender, trace info, reply channel)
    pub context: MessageContext,
    /// The actual message payload
    pub payload: {envelope},
}}
"#,
            self.task_name,
            to_pascal_case(self.task_name.as_str()),
            envelope = self.envelope_name
        )
    }

    /// Generate complete Rust code for the implicit union
    pub fn generate_rust_code(&self) -> String {
        if self.is_empty() {
            return format!("// No implicit union needed for {}\n", self.task_name);
        }

        let mut code = String::new();

        // Add comment header
        code.push_str(&format!(
            "// Auto-generated implicit union for {} task\n",
            self.task_name
        ));
        code.push_str(&format!(
            "// Generated by Plan 125 Phase 3.3 Implicit Union Generator\n\n"
        ));

        // Generate enum
        code.push_str(&self.generate_rust_enum());
        code.push('\n');

        // Generate envelope struct
        code.push_str(&self.generate_envelope_struct());

        code
    }

    /// Generate C enum definition for transpilation
    pub fn generate_c_enum(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut code = format!("typedef enum {{\n");

        // Generate tags for each variant
        for (i, lit) in self.literals.iter().enumerate() {
            let variant_name = Self::literal_to_variant_name(lit);
            code.push_str(&format!("    _{}_Tag = {},\n", variant_name, i));
        }

        let offset = self.literals.len();
        for (i, (name, _type_expr)) in self.type_bindings.iter().enumerate() {
            let variant_name = Self::type_binding_to_variant_name(name);
            code.push_str(&format!(
                "    _{}_Tag = {},\n",
                variant_name,
                offset + i
            ));
        }

        code.push_str(&format!("}} {}_tag;\n\n", self.envelope_name));

        // Generate union for payload
        code.push_str(&format!("typedef union {{\n"));
        for (name, type_expr) in &self.type_bindings {
            let variant_name = Self::type_binding_to_variant_name(name);
            let c_type = type_to_c_type(type_expr);
            code.push_str(&format!("    {} {}_value;\n", c_type, variant_name.to_lowercase()));
        }
        code.push_str(&format!("}} {}_payload;\n\n", self.envelope_name));

        // Generate struct
        code.push_str(&format!(
            "typedef struct {{\n\
             {}_tag tag;\n\
             {}_payload payload;\n\
             }} {};\n",
            self.envelope_name,
            self.envelope_name,
            self.envelope_name
        ));

        code
    }
}

impl fmt::Display for ImplicitUnionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ImplicitUnionInfo({})", self.envelope_name)?;
        writeln!(f, "  Literals: {:?}", self.literals)?;
        writeln!(f, "  TypeBindings: {:?}", self.type_bindings)?;
        Ok(())
    }
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else {
            capitalize_next = true;
        }
    }

    // Handle empty or all-non-alphanumeric strings
    if result.is_empty() {
        return "Unnamed".to_string();
    }

    result
}

/// Convert a Type to Rust type string
fn type_to_rust_type(ty: &Type) -> String {
    match ty {
        Type::Int => "i64".to_string(),
        Type::Uint => "u64".to_string(),
        Type::I64 => "i64".to_string(),
        Type::U64 => "u64".to_string(),
        Type::Float => "f32".to_string(),
        Type::Double => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Byte => "u8".to_string(),
        Type::Char => "char".to_string(),
        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => "String".to_string(),
        Type::CStrLit => "std::ffi::CString".to_string(),
        Type::Void => "()".to_string(),
        Type::USize => "usize".to_string(),
        Type::Array(arr) => {
            let elem_type = type_to_rust_type(&arr.elem);
            format!("[{}; {}]", elem_type, arr.len)
        }
        Type::List(elem) => {
            let elem_type = type_to_rust_type(elem);
            format!("Vec<{}>", elem_type)
        }
        Type::Map(k, v) => {
            format!("std::collections::HashMap<{}, {}>", type_to_rust_type(k), type_to_rust_type(v))
        }
        Type::Option(inner) => {
            let inner_type = type_to_rust_type(inner);
            format!("Option<{}>", inner_type)
        }
        Type::Result(inner) => {
            let inner_type = type_to_rust_type(inner);
            format!("Result<{}, Box<dyn std::error::Error>>", inner_type)
        }
        Type::User(type_decl) => type_decl.name.to_string(),
        Type::Unknown => "_".to_string(),
        _ => format!("{:?}", ty),
    }
}

/// Convert a Type to C type string
fn type_to_c_type(ty: &Type) -> String {
    match ty {
        Type::Int | Type::I64 => "int64_t".to_string(),
        Type::Uint | Type::U64 => "uint64_t".to_string(),
        Type::Float => "float".to_string(),
        Type::Double => "double".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Byte => "uint8_t".to_string(),
        Type::Char => "char".to_string(),
        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => "char*".to_string(),
        Type::Void => "void".to_string(),
        Type::USize => "size_t".to_string(),
        Type::Array(arr) => {
            let elem_type = type_to_c_type(&arr.elem);
            format!("{}[{}]", elem_type, arr.len)
        }
        Type::User(type_decl) => type_decl.name.to_string(),
        Type::Unknown => "void*".to_string(),
        _ => "void*".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implicit_union_new() {
        let info = ImplicitUnionInfo::new("TestTask");
        assert_eq!(info.task_name, "TestTask");
        assert_eq!(info.envelope_name, "TestTaskEnvelope");
        assert!(info.is_empty());
    }

    #[test]
    fn test_add_literal_string() {
        let mut info = ImplicitUnionInfo::new("TestTask");
        info.add_literal(LiteralValue::String("ping".into()));
        assert_eq!(info.literals.len(), 1);
        assert!(info.literals.contains(&LiteralValue::String("ping".into())));
    }

    #[test]
    fn test_add_literal_dedup() {
        let mut info = ImplicitUnionInfo::new("TestTask");
        info.add_literal(LiteralValue::String("ping".into()));
        info.add_literal(LiteralValue::String("ping".into()));
        assert_eq!(info.literals.len(), 1);
    }

    #[test]
    fn test_add_type_binding() {
        let mut info = ImplicitUnionInfo::new("TestTask");
        info.add_type_binding("msg".into(), Type::StrFixed(0));
        assert_eq!(info.type_bindings.len(), 1);
        assert_eq!(info.type_bindings[0].0, "msg");
    }

    #[test]
    fn test_literal_to_variant_name_string() {
        let name = ImplicitUnionInfo::literal_to_variant_name(&LiteralValue::String("ping".into()));
        assert_eq!(name, "LiteralPing");
    }

    #[test]
    fn test_literal_to_variant_name_int() {
        let name = ImplicitUnionInfo::literal_to_variant_name(&LiteralValue::Int(404));
        assert_eq!(name, "LiteralInt404");
    }

    #[test]
    fn test_literal_to_variant_name_bool() {
        let name_true = ImplicitUnionInfo::literal_to_variant_name(&LiteralValue::Bool(true));
        let name_false = ImplicitUnionInfo::literal_to_variant_name(&LiteralValue::Bool(false));
        assert_eq!(name_true, "LiteralTrue");
        assert_eq!(name_false, "LiteralFalse");
    }

    #[test]
    fn test_type_binding_to_variant_name() {
        let name = ImplicitUnionInfo::type_binding_to_variant_name(&"msg".into());
        assert_eq!(name, "Msg");
    }

    #[test]
    fn test_generate_rust_enum() {
        let mut info = ImplicitUnionInfo::new("TestTask");
        info.add_literal(LiteralValue::String("ping".into()));
        info.add_type_binding("msg".into(), Type::StrFixed(0));

        let code = info.generate_rust_enum();
        assert!(code.contains("pub enum TestTaskEnvelope"));
        assert!(code.contains("LiteralPing"));
        assert!(code.contains("Msg(String)"));
    }

    #[test]
    fn test_generate_rust_enum_int() {
        let mut info = ImplicitUnionInfo::new("TestTask");
        info.add_literal(LiteralValue::Int(404));

        let code = info.generate_rust_enum();
        assert!(code.contains("LiteralInt404"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("ping"), "Ping");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("my-variable"), "MyVariable");
        assert_eq!(to_pascal_case(""), "Unnamed");
    }

    #[test]
    fn test_type_to_rust_type() {
        assert_eq!(type_to_rust_type(&Type::Int), "i64");
        assert_eq!(type_to_rust_type(&Type::Bool), "bool");
        assert_eq!(type_to_rust_type(&Type::StrFixed(0)), "String");
        assert_eq!(type_to_rust_type(&Type::Void), "()");
    }

    #[test]
    fn test_from_on_block() {
        use crate::ast::{Body, Expr};
        use crate::token::Pos;
        use auto_val::Op;

        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut on_block = crate::ast::TaskOnBlock::new(pos);

        // Add literal pattern
        on_block.add_handler_with_guard(
            TaskMsgPattern::Literal(LiteralValue::String("ping".into())),
            None,
            Body::new(),
        );

        // Add type binding pattern with guard
        on_block.add_handler_with_guard(
            TaskMsgPattern::TypeBinding {
                name: "amount".into(),
                type_expr: Box::new(Type::Int),
            },
            Some(Expr::Bina(
                Box::new(Expr::Ident("amount".into())),
                Op::Gt,
                Box::new(Expr::Int(10000)),
            )),
            Body::new(),
        );

        let info = ImplicitUnionInfo::from_on_block("WorkerTask", &on_block);
        assert_eq!(info.literals.len(), 1);
        assert_eq!(info.type_bindings.len(), 1);
        assert_eq!(info.envelope_name, "WorkerTaskEnvelope");
    }

    #[test]
    fn test_generate_c_enum() {
        let mut info = ImplicitUnionInfo::new("TestTask");
        info.add_literal(LiteralValue::String("ping".into()));
        info.add_type_binding("amount".into(), Type::Int);

        let code = info.generate_c_enum();
        assert!(code.contains("typedef enum"));
        assert!(code.contains("_LiteralPing_Tag"));
        assert!(code.contains("_Amount_Tag"));
    }

    #[test]
    fn test_variant_count() {
        let mut info = ImplicitUnionInfo::new("TestTask");
        assert_eq!(info.variant_count(), 0);

        info.add_literal(LiteralValue::String("ping".into()));
        assert_eq!(info.variant_count(), 1);

        info.add_type_binding("msg".into(), Type::StrFixed(0));
        assert_eq!(info.variant_count(), 2);
    }

    #[test]
    fn test_empty_union() {
        let info = ImplicitUnionInfo::new("EmptyTask");
        assert!(info.is_empty());
        assert!(info.generate_rust_enum().contains("No implicit union needed"));
    }
}
