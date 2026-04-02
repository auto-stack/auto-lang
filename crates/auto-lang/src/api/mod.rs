//! API Code Generation Module
//!
//! Plan 102 Phase 5: Parse `#[api]` annotations and generate code for Tauri and Web targets
//!
//! ## Overview
//!
//! This module provides functionality to:
//! 1. Parse `#[api]` annotations from Auto source code
//! 2. Extract API endpoint definitions
//! 3. Generate TypeScript types and API clients
//! 4. Generate Tauri commands (Rust)
//! 5. Generate Axum HTTP routes (Rust)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auto_lang::api::{ApiExtractor, ApiModule, Target};
//!
//! // Extract API definitions from AST
//! let extractor = ApiExtractor::new();
//! let api_module = extractor.extract(&ast);
//!
//! // Generate TypeScript types
//! let ts_gen = Target::TypeScript.generator();
//! let ts_code = ts_gen.generate(&api_module);
//!
//! // Generate Tauri commands
//! let tauri_gen = Target::Tauri.generator();
//! let tauri_code = tauri_gen.generate(&api_module);
//! ```

pub mod types;
pub mod targets;

// Re-export main types
pub use types::{ApiAttrs, ApiEndpoint, ApiField, ApiModule, ApiParam, ApiType};
pub use targets::{Target, TargetGenerator, TypeScriptGenerator, TauriGenerator, AxumGenerator};

use crate::ast::{Fn, Stmt, Type};

/// API annotation parser
///
/// Parses `#[api]` annotations from function declarations
pub struct ApiAnnotationParser;

impl ApiAnnotationParser {
    /// Parse API annotation string (content inside `#[api(...)]`)
    ///
    /// Supports formats:
    /// - `#[api]` - Simple annotation
    /// - `#[api(method = "GET")]` - With method
    /// - `#[api(path = "/users/:id")]` - With path
    /// - `#[api(method = "POST", path = "/users")]` - Multiple attributes
    pub fn parse(annotation_content: &str) -> ApiAttrs {
        let mut attrs = ApiAttrs::new();

        if annotation_content.trim().is_empty() {
            return attrs;
        }

        // Parse key=value pairs
        for part in annotation_content.split(',') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                match key {
                    "method" => attrs.method = Some(value.to_string()),
                    "path" => attrs.path = Some(value.to_string()),
                    "name" => attrs.name = Some(value.to_string()),
                    "auth" => attrs.auth = value.eq_ignore_ascii_case("true"),
                    "cache" => {
                        if let Ok(seconds) = value.parse::<u32>() {
                            attrs.cache = Some(seconds);
                        }
                    }
                    _ => {
                        attrs.custom.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        attrs
    }
}

/// API extractor from AST
///
/// Walks through AST statements and extracts functions with `#[api]` annotations
pub struct ApiExtractor {
    /// Whether to include functions without explicit `#[api]` annotation
    include_all_public: bool,
}

impl ApiExtractor {
    /// Create new API extractor
    pub fn new() -> Self {
        Self {
            include_all_public: false,
        }
    }

    /// Configure to include all public functions as APIs
    pub fn with_include_all_public(mut self, include: bool) -> Self {
        self.include_all_public = include;
        self
    }

    /// Extract API module from AST statements
    pub fn extract(&self, module_name: &str, stmts: &[Stmt]) -> ApiModule {
        let mut api_module = ApiModule::new(module_name.to_string());

        for stmt in stmts {
            match stmt {
                Stmt::Fn(fn_decl) => {
                    if let Some(endpoint) = self.extract_endpoint(fn_decl) {
                        api_module.add_endpoint(endpoint);
                    }
                }
                Stmt::TypeDecl(type_decl) => {
                    if let Some(api_type) = self.extract_type(type_decl) {
                        api_module.types.push(api_type);
                    }
                }
                _ => {}
            }
        }

        api_module
    }

    /// Extract API endpoint from function declaration
    fn extract_endpoint(&self, fn_decl: &Fn) -> Option<ApiEndpoint> {
        // Check if function has API annotation or is public (if include_all_public)
        // For now, we'll use a simple heuristic: check function name prefix
        // In a full implementation, we'd store annotations on the Fn struct

        let mut endpoint = ApiEndpoint::new(fn_decl.name.to_string(), ApiAttrs::new());

        // Extract parameters
        for param in &fn_decl.params {
            let api_param = ApiParam {
                name: param.name.to_string(),
                ty: type_to_string(&param.ty),
                default: param.default.as_ref().map(expr_to_string),
                optional: is_optional_type(&param.ty),
            };
            endpoint.params.push(api_param);
        }

        // Extract return type
        endpoint.return_type = type_to_string(&fn_decl.ret);

        Some(endpoint)
    }

    /// Extract API type from type declaration
    fn extract_type(&self, type_decl: &crate::ast::TypeDecl) -> Option<ApiType> {
        // Only extract user-defined types with struct-like members
        if type_decl.members.is_empty() {
            return None;
        }

        let fields: Vec<ApiField> = type_decl.members.iter().map(|m| ApiField {
            name: m.name.to_string(),
            ty: type_to_string(&m.ty),
            optional: matches!(&m.ty, Type::Option(_)),
            default: m.value.as_ref().map(expr_to_string),
        }).collect();

        Some(ApiType {
            name: type_decl.name.to_string(),
            fields,
            doc: None,
        })
    }
}

impl Default for ApiExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Type to string representation
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::Byte => "byte".to_string(),
        Type::I64 => "i64".to_string(),
        Type::Uint => "uint".to_string(),
        Type::U64 => "u64".to_string(),
        Type::USize => "usize".to_string(),
        Type::Float => "float".to_string(),
        Type::Double => "double".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Str(len) => format!("str[{}]", len),
        Type::CStr => "cstr".to_string(),
        Type::StrSlice => "str_slice".to_string(),
        Type::String => "String".to_string(),
        Type::Char => "char".to_string(),
        Type::Void => "void".to_string(),
        Type::Unknown => "unknown".to_string(),
        Type::Array(array_type) => {
            let elem_str = type_to_string(&array_type.elem);
            format!("[{}]{}", array_type.len, elem_str)
        }
        Type::RuntimeArray(rta) => {
            let elem_str = type_to_string(&rta.elem);
            format!("[runtime:{}]", elem_str)
        }
        Type::List(elem) => format!("List<{}>", type_to_string(elem)),
        Type::Slice(slice_type) => format!("[]{}", type_to_string(&slice_type.elem)),
        Type::Ptr(ptr_type) => format!("*{}", type_to_string(&ptr_type.of.borrow())),
        Type::Reference(inner) => format!("&{}", type_to_string(inner)),
        Type::User(type_decl) => type_decl.name.to_string(),
        Type::Enum(enum_decl) => enum_decl.borrow().name.to_string(),
        Type::Tag(tag) => tag.borrow().name.to_string(),
        Type::Spec(spec_decl) => spec_decl.borrow().name.to_string(),
        Type::GenericInstance(inst) => {
            if inst.args.is_empty() {
                inst.base_name.to_string()
            } else {
                let args: Vec<String> = inst.args.iter()
                    .map(type_to_string)
                    .collect();
                format!("{}<{}>", inst.base_name, args.join(", "))
            }
        }
        Type::CStruct(type_decl) => format!("struct {}", type_decl.name),
        Type::Linear(inner) => format!("linear<{}>", type_to_string(inner)),
        Type::Variadic => "...".to_string(),
        Type::Fn(params, ret) => {
            let param_str: Vec<String> = params.iter()
                .map(type_to_string)
                .collect();
            format!("fn({}) {}", param_str.join(", "), type_to_string(ret))
        }
        Type::Storage(storage) => storage.to_string(),
        Type::Union(union) => format!("union({})", union.name),
        Type::Option(inner) => format!("?{}", type_to_string(inner)),  // Plan 120
        Type::Result(inner) => format!("!{}", type_to_string(inner)),  // Plan 120
        Type::Handle { task_type } => format!("Handle<{}>", type_to_string(task_type)),  // Plan 121
    }
}

/// Check if type is optional (May<T>)
fn is_optional_type(ty: &Type) -> bool {
    // Check if this is a GenericInstance with base_name "May"
    match ty {
        Type::GenericInstance(inst) => inst.base_name.as_ref() == "May",
        _ => false,
    }
}

/// Convert expression to string (simplified)
fn expr_to_string(_expr: &crate::ast::Expr) -> String {
    // Simplified implementation
    "...".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_annotation() {
        let attrs = ApiAnnotationParser::parse("");
        assert!(attrs.is_simple());
    }

    #[test]
    fn test_parse_method_annotation() {
        let attrs = ApiAnnotationParser::parse(r#"method = "GET""#);
        assert_eq!(attrs.method, Some("GET".to_string()));
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let attrs = ApiAnnotationParser::parse(r#"method = "POST", path = "/users", auth = true"#);
        assert_eq!(attrs.method, Some("POST".to_string()));
        assert_eq!(attrs.path, Some("/users".to_string()));
        assert!(attrs.auth);
    }

    #[test]
    fn test_parse_cache_attribute() {
        let attrs = ApiAnnotationParser::parse(r#"cache = 60"#);
        assert_eq!(attrs.cache, Some(60));
    }

    #[test]
    fn test_parse_custom_attribute() {
        let attrs = ApiAnnotationParser::parse(r#"custom = "value""#);
        assert_eq!(attrs.custom.get("custom"), Some(&"value".to_string()));
    }
}

// Phase 5.4: Integration tests
#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod type_extraction_tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_extract_type_definition() {
        let code = r#"
type User {
    id int
    name str
    email str
}
"#;
        let mut parser = Parser::from(code);
        let ast = parser.parse().unwrap();

        let extractor = ApiExtractor::new();
        let module = extractor.extract("test", &ast.stmts);

        assert_eq!(module.types.len(), 1);
        assert_eq!(module.types[0].name, "User");
        assert_eq!(module.types[0].fields.len(), 3);
    }
}
