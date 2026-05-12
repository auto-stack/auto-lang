//! # Auto-to-Rust (a2r) Transpiler
//!
//! This module transpiles AutoLang source code to Rust, providing a native code
//! compilation path for AutoLang applications. The a2r transpiler converts AutoLang's
//! high-level syntax to idiomatic Rust code.
//!
//! ## Features
//!
//! - **Full language support**: Functions, structs, enums, closures, generics
//! - **Trait system**: AutoLang specs transpile to Rust traits
//! - **Type safety**: Preserves AutoLang's type system in Rust
//! - **Pattern matching**: AutoLang `is` expressions transpile to Rust `match`
//! - **Memory safety**: Borrow checking via AutoLang's view/mut/take semantics
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auto_lang::trans::rust::RustTrans;
//!
//! let code = r#"
//! fn main() {
//!     let x = 42
//!     print(x)
//! }
//! "#;
//!
//! let mut trans = RustTrans::new("test".into());
//! let mut sink = Sink::new(AutoStr::from("test"));
//! trans.trans(code.parse()?, &mut sink)?;
//! println!("{}", String::from_utf8(sink.done()?.to_vec())?);
//! ```
//!
//! ## Transpilation Mapping
//!
//! | AutoLang | Rust |
//! |-----------|------|
//! | `fn add(a int, b int) int` | `fn add(a: i32, b: i32) -> i32` |
//! | `let x = 42` | `let x: i32 = 42;` |
//! | `var x = 42` | `let mut x: i32 = 42;` |
//! | `(a, b) => a + b` | `|a: i32, b: i32| a + b` |
//! | `spec Flyer { fn fly() }` | `trait Flyer { fn fly(&self); }` |
//! | `type Point<T>` | `struct Point<T>` |
//! | `use auto.io: say` | `use crate::io::say;` |

use super::{escape_str, Sink, Trans};
use crate::ast::*;
use crate::database::Database;
use crate::parser::Parser;
use crate::types::TypeStore;
// Plan 091: Universe removed
use crate::{AutoError, AutoResult, Rc};
use auto_val::{shared, Shared};
use auto_val::{AutoStr, Op};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::Arc;
use std::sync::RwLock;

pub enum RustEdition {
    E2021,
    E2024,
}

pub struct RustTrans {
    indent: usize,
    uses: HashSet<AutoStr>,
    dep_crates: HashSet<AutoStr>,

    // Hybrid: Support both Universe (deprecated) and Database (new)
    // Phase 066: Migrating to Database-based architecture
    db: Option<Arc<RwLock<Database>>>, // New (Phase 066)

    edition: RustEdition,

    // Transpiler internal state (not from Database or Universe)
    _current_fn: Option<AutoStr>,
    _current_scope: Option<crate::scope::Sid>,

    // Plan 204 Phase 3: Whether any function returns !T, requiring Err trait emission
    needs_err_trait: bool,

    // Plan 204 Phase 3: Whether current function returns !T (for Err boxing)
    current_fn_is_result: bool,

    // Inferred concrete error type for current !T function
    // If all Err(X) use the same enum E, this is Some(E) → Result<T, E>
    // Otherwise None → Result<T, Box<dyn std::error::Error>>
    current_fn_err_type: Option<AutoStr>,

    // Cache for struct field names (for positional arg mapping)
    struct_fields: HashMap<AutoStr, Vec<AutoStr>>,

    // Cache for tag type names (for tag construction detection)
    tag_types: HashSet<AutoStr>,

    // Cache for struct field types: struct_name -> Vec<(field_name, field_type)>
    // Used to add .to_string() when &str is assigned to String field
    struct_field_types: HashMap<AutoStr, Vec<(AutoStr, Type)>>,

    // Cache for enum struct variants: (EnumName, VariantName) -> Vec<field_names>
    // Used to emit correct struct pattern syntax in match arms
    enum_struct_variants: HashMap<(AutoStr, AutoStr), Vec<AutoStr>>,

    // Cache for enum tuple variants: (EnumName, VariantName) -> arity
    // Used to emit (_) for bare tuple variant checks in match arms
    enum_tuple_variants: HashMap<(AutoStr, AutoStr), usize>,

    // Cache for enum tuple variant field types: (EnumName, VariantName) -> Vec<Type>
    // Used to add .to_string() when constructing with &str args for String fields
    enum_tuple_field_types: HashMap<(AutoStr, AutoStr), Vec<Type>>,

    // Plan 159 Phase 6B-2.2: Cache for spec declarations (for impl Trait for Type)
    spec_decls: HashMap<AutoStr, Vec<SpecMethod>>,

    // Plan 151: Global variables (top-level var declarations)
    // Tracks global variables that need Lazy<Mutex<T>> wrapper
    global_vars: HashSet<AutoStr>,

    // Plan 167: Multi-file mode — local module names for mod declarations
    local_modules: HashSet<String>,
    // Multi-file mode: set of sibling module names (same directory)
    // Used to generate `use super::X;` instead of `use crate::X;`
    sibling_modules: HashSet<String>,
    // Multi-file mode: dir children of a directory module (mod.rs/mod.at)
    // use X for these should be skipped (pub mod X; already emitted)
    dir_children: HashSet<String>,
    // Whether current module is a directory module
    is_dir_module: bool,
    // Whether we're inside a pub type declaration (methods should be pub)
    inside_pub_type: bool,
    // Modules imported via `use X` → `use super::X::*;` in multi-file mode
    // These should NOT be used as source_crate prefix for type resolution
    glob_imported_modules: HashSet<String>,

    // Plan 232: Track current function's str-type parameter names
    // Used to add .to_string() when returning a &str param as String
    current_fn_str_params: HashSet<AutoStr>,

    // Track which function params are str (&str) type for auto-borrow at call sites
    // fn_name -> vec of booleans (true = param is str/&str, needs & at call site)
    fn_str_param_indices: HashMap<AutoStr, Vec<bool>>,

    // Track current function's return type for string coercion
    current_fn_ret_type: Option<Type>,

    // Track local variable types for string concat detection in Op::Add
    local_var_types: HashMap<AutoStr, Type>,
    // Track variables assigned from json.get() — need value_to_int/value_len helpers
    json_value_vars: HashSet<AutoStr>,
    // Track function params declared as &str (StrSlice) — safe to pass without .as_str()
    fn_param_str_slice: HashSet<AutoStr>,

    // Track which function params are struct/enum types (need .clone() at call sites)
    fn_struct_param_indices: HashMap<AutoStr, Vec<bool>>,
    // Track which function params are spec types (need Box::new() at call sites)
    fn_spec_param_indices: HashMap<AutoStr, Vec<bool>>,
    // Track struct→spec mapping: struct_name -> spec_name (for spec array inference)
    struct_to_spec: HashMap<AutoStr, AutoStr>,
    // Track variable→spec mapping: var_name -> spec_name
    var_spec_map: HashMap<AutoStr, AutoStr>,

}

impl RustTrans {
    pub fn new(_name: AutoStr) -> Self {
        Self {
            indent: 0,
            uses: HashSet::new(),
            dep_crates: HashSet::new(),
            db: None,
            edition: RustEdition::E2021,
            _current_fn: None,
            _current_scope: None,
            needs_err_trait: false,
            current_fn_is_result: false,
            current_fn_err_type: None,
            struct_fields: HashMap::new(),
            struct_field_types: HashMap::new(),
            tag_types: HashSet::new(),
            enum_struct_variants: HashMap::new(),
            enum_tuple_variants: HashMap::new(),
            enum_tuple_field_types: HashMap::new(),
            spec_decls: HashMap::new(),
            global_vars: HashSet::new(),
            local_modules: HashSet::new(),
            sibling_modules: HashSet::new(),
            dir_children: HashSet::new(),
            is_dir_module: false,
            inside_pub_type: false,
            glob_imported_modules: HashSet::new(),
            current_fn_str_params: HashSet::new(),
            fn_str_param_indices: HashMap::new(),
            current_fn_ret_type: None,
            local_var_types: HashMap::new(),
            json_value_vars: HashSet::new(),
            fn_param_str_slice: HashSet::new(),
            fn_struct_param_indices: HashMap::new(),
            struct_to_spec: HashMap::new(),
            var_spec_map: HashMap::new(),
            fn_spec_param_indices: HashMap::new(),
        }
    }

    /// Create transpiler with Database (Phase 066: new API)
    pub fn with_database(db: Arc<RwLock<Database>>) -> Self {
        Self {
            indent: 0,
            uses: HashSet::new(),
            dep_crates: HashSet::new(),
            db: Some(db),
            edition: RustEdition::E2021,
            _current_fn: None,
            _current_scope: None,
            needs_err_trait: false,
            current_fn_is_result: false,
            current_fn_err_type: None,
            struct_fields: HashMap::new(),
            struct_field_types: HashMap::new(),
            tag_types: HashSet::new(),
            enum_struct_variants: HashMap::new(),
            enum_tuple_variants: HashMap::new(),
            enum_tuple_field_types: HashMap::new(),
            spec_decls: HashMap::new(),
            global_vars: HashSet::new(),
            local_modules: HashSet::new(),
            sibling_modules: HashSet::new(),
            dir_children: HashSet::new(),
            is_dir_module: false,
            inside_pub_type: false,
            glob_imported_modules: HashSet::new(),
            current_fn_str_params: HashSet::new(),
            fn_str_param_indices: HashMap::new(),
            current_fn_ret_type: None,
            local_var_types: HashMap::new(),
            json_value_vars: HashSet::new(),
            fn_param_str_slice: HashSet::new(),
            fn_struct_param_indices: HashMap::new(),
            struct_to_spec: HashMap::new(),
            var_spec_map: HashMap::new(),
            fn_spec_param_indices: HashMap::new(),
        }
    }

    #[deprecated(note = "Use with_database() instead (Phase 066)")]
    pub fn set_scope(&mut self, _scope: Shared<crate::scope_manager::ScopeManager>) {
        // Plan 091: scope removed, no-op
    }

    pub fn set_edition(&mut self, edition: RustEdition) {
        self.edition = edition;
    }

    /// Extract the type name from a constructor expression.
    fn extract_tag_or_ctor_type(expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Call(call) => {
                if let Expr::Ident(name) = call.name.as_ref() {
                    Some(name.clone())
                } else { None }
            }
            _ => None,
        }
    }

    /// Get Database reference (Phase 066)
    pub fn db(&self) -> Option<&Arc<RwLock<Database>>> {
        self.db.as_ref()
    }

    // =========================================================================
    // Plan 151: Tauri IPC Mode - Global Variable Support
    // =========================================================================

    /// Register a global variable (top-level var declaration)
    pub fn register_global_var(&mut self, name: AutoStr) {
        self.global_vars.insert(name);
    }

    /// Check if a variable is a global variable
    pub fn is_global_var(&self, name: &AutoStr) -> bool {
        self.global_vars.contains(name)
    }

    /// Scan statements for Err(X) calls; if all use the same enum type, return it
    fn infer_err_enum(&self, stmts: &[Stmt]) -> Option<AutoStr> {
        let mut found_enum: Option<AutoStr> = None;
        for stmt in stmts {
            let result = self.scan_stmt_err_enum(stmt);
            match result {
                Some(Some(enum_name)) => {
                    match &found_enum {
                        Some(existing) if *existing != enum_name => return None,
                        _ => found_enum = Some(enum_name),
                    }
                }
                Some(None) => return None,
                None => {}
            }
        }
        found_enum
    }

    fn scan_stmt_err_enum(&self, stmt: &Stmt) -> Option<Option<AutoStr>> {
        match stmt {
            Stmt::Expr(expr) => self.scan_expr_err_enum(expr),
            Stmt::Return(expr) => self.scan_expr_err_enum(expr),
            Stmt::If(if_) => {
                for branch in &if_.branches {
                    for s in &branch.body.stmts {
                        if let Some(r) = self.scan_stmt_err_enum(s) { return Some(r); }
                    }
                }
                if let Some(else_body) = &if_.else_ {
                    for s in &else_body.stmts {
                        if let Some(r) = self.scan_stmt_err_enum(s) { return Some(r); }
                    }
                }
                None
            }
            Stmt::Store(store) => self.scan_expr_err_enum(&store.expr),
            _ => None,
        }
    }

    fn scan_expr_err_enum(&self, expr: &Expr) -> Option<Option<AutoStr>> {
        match expr {
            Expr::Err(inner) => {
                match inner.as_ref() {
                    // EditError.Variant(args) — Call with Dot callee
                    Expr::Call(call) => {
                        if let Expr::Bina(lhs, op, _) = call.name.as_ref() {
                            if matches!(op, Op::Dot) {
                                if let Expr::Ident(type_name) = lhs.as_ref() {
                                    if self.tag_types.contains(type_name) {
                                        return Some(Some(type_name.clone()));
                                    }
                                }
                            }
                        }
                        if let Expr::Dot(obj, _) = call.name.as_ref() {
                            if let Expr::Ident(type_name) = obj.as_ref() {
                                if self.tag_types.contains(type_name) {
                                    return Some(Some(type_name.clone()));
                                }
                            }
                        }
                        Some(None)
                    }
                    // EditError.Variant (no args) — plain Dot expression
                    Expr::Dot(obj, _) => {
                        if let Expr::Ident(type_name) = obj.as_ref() {
                            if self.tag_types.contains(type_name) {
                                return Some(Some(type_name.clone()));
                            }
                        }
                        Some(None)
                    }
                    Expr::Str(_) | Expr::CStr(_) => Some(None),
                    _ => Some(None),
                }
            }
            _ => None,
        }
    }

    /// Recursively check if an expression tree contains string-typed elements
    fn expr_contains_string(&self, e: &Expr) -> bool {
        match e {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => true,
            Expr::Ident(name) => {
                if let Some(ty) = self.local_var_types.get(name) {
                    return matches!(ty,
                        Type::StrOwned | Type::StrFixed(_) | Type::StrSlice);
                }
                // Unknown type: conservatively assume string if the name
                // suggests string content (heuristic to catch let-bound vars)
                false
            }
            Expr::Call(c) => {
                if let Expr::Ident(name) = c.name.as_ref() {
                    matches!(name.as_str(),
                        "to_string" | "format" | "trim" | "replace"
                        | "to_lowercase" | "to_uppercase" | "read_to_string"
                        | "read_line" | "collect")
                } else {
                    false
                }
            }
            Expr::Dot(_, method) => {
                matches!(method.as_str(),
                    "to_string" | "trim" | "replace" | "to_lowercase"
                    | "to_uppercase" | "display" | "format")
            }
            Expr::Bina(inner_lhs, _, inner_rhs) => {
                self.expr_contains_string(inner_lhs)
                    || self.expr_contains_string(inner_rhs)
            }
            _ => false,
        }
    }

    /// Get the uppercase name for a global variable static
    pub fn global_var_static_name(&self, name: &AutoStr) -> String {
        name.to_uppercase().to_string()
    }

    // =========================================================================
    // Phase 066: Unified Helper Methods (Universe or Database)
    // =========================================================================

    /// Check if a type is an enum (works with Universe or Database)
    #[allow(dead_code)]
    fn is_enum_type(&self, _type_name: &AutoStr) -> bool {
        // Plan 091: Use Database only
        if let Some(_db) = &self.db {
            // New path: Database
            // NOTE: TypeInfoStore doesn't store type kind (enum/struct/union)
            // For transpilation purposes, assume false (conservative)
            false
        } else {
            false
        }
    }

    /// Look up metadata by name (works with Universe or Database)
    /// Phase 066: Unified helper for Database/Universe access
    fn lookup_meta(&self, name: &str) -> Option<Rc<crate::scope::Meta>> {
        // Plan 091: Use Database only
        if let Some(db) = &self.db {
            // New path: Database
            if let Ok(db) = db.try_read() {
                // Search through symbol tables for the symbol
                for (_sid, table) in db.get_all_symbol_tables() {
                    if let Some(meta) = table.symbols.get(name) {
                        return Some(meta.clone());
                    }
                }
                None
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Look up type by name (works with Universe or Database)
    /// Phase 066: Unified helper for Database/Universe access
    #[allow(dead_code)]
    fn lookup_type(&self, _type_name: &AutoStr) -> Type {
        // Plan 091: Use Database only
        if let Some(_db) = &self.db {
            // New path: Database
            // NOTE: TypeInfoStore doesn't store type kind (enum/struct/union)
            // Return Type::Unknown for now (conservative approach)
            // TODO: Enhance Database to store type metadata (enum/struct/union)
            Type::Unknown
        } else {
            Type::Unknown
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
        for _ in 0..self.indent {
            out.write(b"    ")?;
        }
        Ok(())
    }

    /// Check if current function's return type maps to Rust String (needs &str -> String coercion)
    fn ret_type_needs_string_coercion(&self) -> bool {
        self.current_fn_ret_type.as_ref().map_or(false, |ty| {
            matches!(ty, Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit)
        })
    }

    /// Check if an expression produces &str that needs .to_string() for String return
    fn expr_needs_string_coercion(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Str(_) | Expr::CStr(_) => true,
            Expr::Index(_, idx) => matches!(idx.as_ref(), Expr::Range(_)),
            Expr::Ident(name) => self.current_fn_str_params.contains(name),
            // x.slice(...) is transpiled to x[n..] which produces &str
            Expr::Call(call) => {
                if let Expr::Dot(_, method) = call.name.as_ref() {
                    method == "slice"
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Write a return expression with automatic .to_string() coercion when needed.
    /// `add_semi`: whether to append a semicolon (false for match arm bodies).
    fn write_return_expr(&mut self, expr: &Expr, out: &mut impl Write, add_semi: bool) -> AutoResult<()> {
        // If returning a &str parameter ident directly, wrap in .to_string()
        if let Expr::Ident(name) = expr {
            if self.current_fn_str_params.contains(name) {
                write!(out, "return {}.to_string()", name)?;
                if add_semi { out.write(b";")?; }
                return Ok(());
            }
        }
        let needs_to_string = self.ret_type_needs_string_coercion()
            && self.expr_needs_string_coercion(expr);
        out.write(b"return ")?;
        self.expr(expr, out)?;
        if needs_to_string {
            out.write(b".to_string()")?;
        }
        if add_semi { out.write(b";")?; }
        Ok(())
    }

    fn rust_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Byte => "u8".to_string(),
            Type::Int => "i32".to_string(),
            Type::Uint => "u32".to_string(),
            Type::USize => "usize".to_string(),
            Type::Float | Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::StrFixed(_) => "String".to_string(),
            Type::CStrLit => "String".to_string(),
            Type::StrSlice => "String".to_string(),
            Type::StrOwned => "String".to_string(), // Owned dynamic string (Plan 155)
            Type::Array(arr) => {
                // Auto arrays are dynamic (push/pop/sort), map to Vec<T>
                // even though the AST carries a compile-time length
                format!("Vec<{}>", self.rust_type_name(&arr.elem))
            }
            Type::RuntimeArray(rta) => {
                // Plan 052: Runtime arrays transpile to Vec<T> in Rust
                // The size expression is evaluated at runtime
                format!("Vec<{}>", self.rust_type_name(&rta.elem))
            }
            Type::List(elem) => {
                // List<T> transpiles to Vec<T> in Rust
                format!("Vec<{}>", self.rust_type_name(elem))
            }
            Type::Map(k, v) => {
                format!("std::collections::HashMap<{}, {}>", self.rust_type_name(k), self.rust_type_name(v))
            }
            Type::Slice(slice) => {
                // []T → &[T], but []Spec → Vec<Box<dyn Spec>> (dynamic polymorphism)
                if matches!(&*slice.elem, Type::Spec(_)) {
                    format!("Vec<{}>", self.rust_type_name(&slice.elem))
                } else {
                    format!("&[{}]", self.rust_type_name(&slice.elem))
                }
            }
            Type::Ptr(ptr) => {
                // **Phase 1.1: Pointer Types (test: 005_pointer)**
                // AutoLang *T transpiles to Rust raw pointer *mut T
                // This is for raw pointer operations like @ (address-of) and .* (dereference)
                format!("*mut {}", self.rust_type_name(&*ptr.of.borrow()))
            }
            Type::Reference(inner) => {
                // Plan 052: Reference transpiles to &T in Rust
                format!("&{}", self.rust_type_name(inner))
            }
            Type::User(usr) => usr.name.to_string(),
            Type::Enum(en) => en.borrow().name.to_string(),
            Type::Spec(spec) => format!("Box<dyn {}>", spec.borrow().name), // Spec 作为类型标注 → Box<dyn Trait>
            Type::Union(u) => u.name.to_string(),
            Type::Tag(t) => t.borrow().name.to_string(),
            Type::Variadic => "...".to_string(), // C variadic, not used in Rust
            Type::Void => "()".to_string(),
            Type::Unknown => "/* unknown */".to_string(),
            Type::CStruct(decl) => decl.name.to_string(),
            Type::Linear(inner) => {
                // Linear types unwrap to their inner type for transpilation
                // The move semantics are enforced by AutoLang's ownership system
                self.rust_type_name(inner)
            }
            Type::Fn(params, ret) => {
                // Function type: fn(param1, param2) ret_type
                // Transpile to Rust: fn(param1_type, param2_type) -> ret_type
                let param_str: Vec<String> =
                    params.iter().map(|p| self.rust_type_name(p)).collect();
                format!(
                    "fn({}) -> {}",
                    param_str.join(", "),
                    self.rust_type_name(ret)
                )
            }
            Type::GenericInstance(inst) => {
                // Generic instances: MyType<int> -> MyType<int>
                let args: Vec<String> = inst.args.iter().map(|t| self.rust_type_name(t)).collect();
                // Plan 190: Use short_name from RustSource if available
                let base = if let Some(ref source) = inst.source {
                    source.short_name().to_string()
                } else {
                    inst.base_name.to_string()
                };
                format!("{}<{}>", base, args.join(", "))
            }
            Type::Storage(storage) => {
                // Storage types are marker types, just use the name
                format!("{}", storage)
            }
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            // Plan 120: Option and Result types
            Type::Option(inner) => format!("Option<{}>", self.rust_type_name(inner)),
            Type::Result(inner) => {
                let err_type = match &self.current_fn_err_type {
                    Some(enum_name) => enum_name.to_string(),
                    None => "Box<dyn std::error::Error>".to_string(),
                };
                format!("Result<{}, {}>", self.rust_type_name(inner), err_type)
            }
            // Plan 121: Handle type - maps to Arc<TaskHandle<T>>
            Type::Handle { task_type } => format!("std::sync::Arc<TaskHandle<{}>>", self.rust_type_name(task_type)),
            Type::Rust(source) => source.short_name().to_string(),
            Type::Tuple(ts) => {
                let elems: Vec<String> = ts.iter().map(|t| self.rust_type_name(t)).collect();
                format!("({})", elems.join(", "))
            }
        }
    }

    /// Plan 204 Phase 1B: Return type mapping for function return positions.
    /// Auto `str` (parsed as `StrSlice`) should produce Rust `String` in return
    /// position, while parameters keep `&str` for borrowed semantics.
    fn rust_return_type_name(&self, ty: &Type) -> String {
        match ty {
            // str/CStr in return position -> String (owned, safe default)
            Type::StrSlice | Type::CStrLit => "String".to_string(),
            // Option<str> / Option<cstr> -> Option<String>
            Type::Option(inner) => {
                format!("Option<{}>", self.rust_return_type_name(inner))
            }
            // Result<str> -> Result<String, E> where E is inferred or Box<dyn Error>
            Type::Result(inner) => {
                let err_type = match &self.current_fn_err_type {
                    Some(enum_name) => enum_name.to_string(),
                    None => "Box<dyn std::error::Error>".to_string(),
                };
                format!("Result<{}, {}>", self.rust_return_type_name(inner), err_type)
            }
            // Fn type: use return type mapping for the return position
            Type::Fn(params, ret) => {
                let param_str: Vec<String> =
                    params.iter().map(|p| self.rust_type_name(p)).collect();
                format!(
                    "fn({}) -> {}",
                    param_str.join(", "),
                    self.rust_return_type_name(ret)
                )
            }
            // Recurse into generic instances to handle Future<String> etc.
            Type::GenericInstance(inst) => {
                let args: Vec<String> = inst.args.iter().map(|t| self.rust_return_type_name(t)).collect();
                let base = if let Some(ref source) = inst.source {
                    source.short_name().to_string()
                } else {
                    inst.base_name.to_string()
                };
                format!("{}<{}>", base, args.join(", "))
            }
            // Tuple: recurse in case inner types need mapping
            Type::Tuple(ts) => {
                let elems: Vec<String> = ts.iter().map(|t| self.rust_return_type_name(t)).collect();
                format!("({})", elems.join(", "))
            }
            // Handle type: recurse for inner type
            Type::Handle { task_type } => {
                format!("std::sync::Arc<TaskHandle<{}>>", self.rust_return_type_name(task_type))
            }
            // All other types delegate to rust_type_name
            _ => self.rust_type_name(ty),
        }
    }

    /// Plan 232: Parameter type mapping for function parameter positions.
    /// Auto `str` in parameter position -> Rust `&str` (borrowed, Copy).
    /// This avoids String ownership transfer on repeated function calls.
    fn rust_param_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::StrFixed(_) | Type::StrSlice | Type::CStrLit => "&str".to_string(),
            _ => self.rust_type_name(ty),
        }
    }

    /// Emit a2r standard library import
    /// Uses the crate's a2r_std module instead of embedding
    fn emit_a2r_stdlib(&self, out: &mut impl Write) -> AutoResult<()> {
        writeln!(out, "// a2r Standard Library (from crate)")?;
        writeln!(out, "#[allow(unused_imports)]")?;
        writeln!(out, "use auto_lang::a2r_std;")?;
        writeln!(out, "use auto_lang::a2r_std::*;")?;
        writeln!(out)?;
        Ok(())
    }

    // is_enum_type() moved to unified helper methods (line 83)
    // Old implementation removed in Phase 066

    /// Map Auto builtin type names to their Rust equivalents.
    /// Returns Some(rust_name) if the ident is a builtin type, None otherwise.
    fn auto_type_to_rust(name: &str) -> Option<&'static str> {
        match name {
            "List" => Some("Vec"),
            "Map" => Some("HashMap"),
            "Set" => Some("HashSet"),
            _ => None,
        }
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            // Literals
            Expr::Int(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::Uint(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::I8(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::U8(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::I64(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::U64(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::Byte(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Float(f, _) => {
                let s = format!("{}", f);
                // Ensure float literal has decimal point (e.g. 2 -> 2.0)
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    write!(out, "{}", s)
                } else {
                    write!(out, "{}.0", s)
                }
                .map_err(Into::into)
            }
            Expr::Double(d, _) => {
                let s = format!("{}", d);
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    write!(out, "{}", s)
                } else {
                    write!(out, "{}.0", s)
                }
                .map_err(Into::into)
            }
            Expr::Bool(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Char(c) => if *c == '\n' {
                write!(out, "'\\n'")
            } else if *c == '\t' {
                write!(out, "'\\t'")
            } else if *c == '\r' {
                write!(out, "'\\r'")
            } else if *c == '\0' {
                write!(out, "'\\0'")
            } else if *c == '\\' {
                write!(out, "'\\\\'")
            } else if *c == '\'' {
                write!(out, "'\\''")
            } else {
                write!(out, "'{}'", c)
            }
            .map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", escape_str(s)).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", s).map_err(Into::into),
            Expr::Ident(name) => {
                // Plan 151: Global variable access - add .lock().unwrap() pattern
                if self.is_global_var(name) {
                    let static_name = self.global_var_static_name(name);
                    write!(out, "{}.lock().unwrap()", static_name)
                } else if let Some(rust_name) = Self::auto_type_to_rust(name.as_str()) {
                    write!(out, "{}", rust_name)
                } else {
                    write!(out, "{}", name)
                }
            }.map_err(Into::into),
            Expr::GenName(name) => write!(out, "{}", name).map_err(Into::into),
            Expr::Nil => write!(out, "None").map_err(Into::into),
            Expr::Null => write!(out, "None").map_err(Into::into),

            // Plan 120/159: Option and Result constructors
            Expr::Some(e) => {
                write!(out, "Some(")?;
                self.expr(e, out)?;
                if matches!(e.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                    write!(out, ".to_string()")?;
                }
                write!(out, ")").map_err(Into::into)
            }
            Expr::None => write!(out, "None").map_err(Into::into),
            Expr::Ok(e) => {
                write!(out, "Ok(")?;
                self.expr(e, out)?;
                // When Ok contains a string literal but the function returns Result<String, ...>,
                // add .to_string() to convert &str -> String
                if matches!(e.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                    if let Some(ref ret) = self.current_fn_ret_type {
                        if let Type::Result(inner) = ret {
                            if matches!(inner.as_ref(), Type::StrSlice | Type::StrOwned | Type::StrFixed(_)) {
                                write!(out, ".to_string()")?;
                            }
                        } else if let Type::GenericInstance(inst) = ret {
                            if inst.base_name == "Result" {
                                if let Some(inner) = inst.args.first() {
                                    if matches!(inner, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)) {
                                        write!(out, ".to_string()")?;
                                    }
                                }
                            }
                        }
                    }
                }
                write!(out, ")").map_err(Into::into)
            }
            Expr::Err(e) => {
                write!(out, "Err(")?;
                if self.current_fn_err_type.is_some() {
                    // Concrete error type — no Box::new needed
                    self.expr(e, out)?;
                } else if matches!(e.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                    self.expr(e, out)?;
                    write!(out, ".into()")?;
                } else {
                    // Box::new() for concrete types -> Box<dyn Error>
                    write!(out, "Box::new(")?;
                    self.expr(e, out)?;
                    write!(out, ")")?;
                }
                write!(out, ")").map_err(Into::into)
            }
            // Plan 6B-4.14: Smart pointer constructors
            Expr::BoxExpr(e) => {
                write!(out, "Box::new(")?;
                self.expr(e, out)?;
                write!(out, ")").map_err(Into::into)
            }
            Expr::ArcExpr(e) => {
                write!(out, "Arc::new(")?;
                self.expr(e, out)?;
                write!(out, ")").map_err(Into::into)
            }

            // Operators
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => {
                        // **Phase 1.1 & 2: Special field names (@, *, view, mut, take)**
                        if let Expr::Ident(field_name) = rhs.as_ref() {
                            match field_name.as_str() {
                                "@" => {
                                    // x.@ -> raw pointer (address-of)
                                    self.expr(lhs, out)?;
                                    write!(out, " as *mut _")?;
                                    return Ok(());
                                }
                                "*" => {
                                    // y.* -> dereference
                                    write!(out, "*")?;
                                    self.expr(lhs, out)?;
                                    return Ok(());
                                }
                                "view" => {
                                    // x.view -> &x (immutable borrow)
                                    write!(out, "&")?;
                                    self.expr(lhs, out)?;
                                    return Ok(());
                                }
                                "mut" => {
                                    // x.mut -> &mut x (mutable borrow)
                                    write!(out, "&mut ")?;
                                    self.expr(lhs, out)?;
                                    return Ok(());
                                }
                                "take" => {
                                    // x.take -> x (move semantics)
                                    self.expr(lhs, out)?;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }

                        // Member access: expr.field or .field (shorthand for self.field)
                        match lhs.as_ref() {
                            Expr::Nil | Expr::Null => {
                                // .field -> self.field
                                write!(out, "self.")?;
                                self.expr(rhs, out)?;
                            }
                            _ => {
                                // Check if this is enum variant access: Type::Variant
                                // Use :: if rhs is an identifier starting with uppercase (enum variant convention)
                                // OR if lhs starts with uppercase (type name for static method: Type.method())
                                let is_enum_variant = if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    rhs_name
                                        .chars()
                                        .next()
                                        .map(|c| c.is_uppercase())
                                        .unwrap_or(false)
                                } else {
                                    false
                                };

                                // Check if lhs is a type name (starts with uppercase, is a Rust primitive,
                                // or is a known module from use.rust imports)
                                let is_type_name = if let Expr::Ident(lhs_name) = lhs.as_ref() {
                                    let name = lhs_name.as_str();
                                    name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                                        || matches!(name,
                                            "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                                            | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                                            | "f32" | "f64" | "bool" | "char" | "str"
                                        )
                                        || self.uses.iter().any(|u| {
                                            let u_str = u.as_str();
                                            u_str == name
                                                || u_str.ends_with(&format!("::{}", name))
                                        })
                                } else {
                                    false
                                };

                                // Check if lhs is a type-like expression (identifier or module.Type chain)
                                let lhs_is_type = if matches!(lhs.as_ref(), Expr::Ident(_)) {
                                    is_enum_variant || is_type_name
                                } else if let Expr::Dot(il, _ir) = lhs.as_ref() {
                                    matches!(il.as_ref(), Expr::Ident(_))
                                } else if let Expr::Bina(il, Op::Dot, _ir) = lhs.as_ref() {
                                    // module.Type or module.module.Type chain (nested Dot via Bina)
                                    matches!(il.as_ref(), Expr::Ident(_))
                                } else {
                                    false
                                };

                                if lhs_is_type {
                                    // Type::Variant or Type::method()
                                    self.expr(lhs, out)?;
                                    write!(out, "::")?;
                                    self.expr(rhs, out)?;
                                } else {
                                    // expr.field or expr.method()
                                    // Parenthesize lhs if it's a binary op (e.g., (a / b).method())
                                    let needs_parens = matches!(lhs.as_ref(),
                                        Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
                                    );
                                    if needs_parens { write!(out, "(")?; }
                                    self.expr(lhs, out)?;
                                    if needs_parens { write!(out, ")")?; }
                                    write!(out, ".")?;
                                    self.expr(rhs, out)?;
                                }
                            }
                        }
                    }
                    Op::Range => {
                        // Range: start..end
                        self.expr(lhs, out)?;
                        write!(out, "..")?;
                        self.expr(rhs, out)?;
                    }
                    Op::RangeEq => {
                        // Inclusive range: start..=end
                        self.expr(lhs, out)?;
                        write!(out, "..=")?;
                        self.expr(rhs, out)?;
                    }
                    Op::Add => {
                        if self.expr_contains_string(&lhs) || self.expr_contains_string(&rhs) {
                            // String involved — use format!
                            write!(out, "format!(\"{{}}{{}}\", ")?;
                            self.expr(&lhs, out)?;
                            write!(out, ", ")?;
                            self.expr(&rhs, out)?;
                            write!(out, ")")?;
                        } else {
                            // Default to numeric +
                            self.expr(&lhs, out)?;
                            write!(out, " + ")?;
                            self.expr(&rhs, out)?;
                        }
                    }
                    Op::Asn | Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq | Op::ModEq => {
                        // Plan 151: Handle global variable assignment
                        // Check if lhs is a global variable identifier
                        if let Expr::Ident(name) = lhs.as_ref() {
                            if self.is_global_var(name) {
                                // Global variable assignment: needs *VAR.lock().unwrap() OP= rhs
                                let static_name = self.global_var_static_name(name);
                                write!(out, "*{}.lock().unwrap()", static_name)?;

                                // Write the operator (without = for compound ops)
                                let op_str = match op {
                                    Op::Asn => "=",
                                    Op::AddEq => "+=",
                                    Op::SubEq => "-=",
                                    Op::MulEq => "*=",
                                    Op::DivEq => "/=",
                                    Op::ModEq => "%=",
                                    _ => op.op(),
                                };
                                write!(out, " {} ", op_str)?;
                                self.expr(rhs, out)?;
                                return Ok(());
                            }
                        }

                        // Normal assignment: lhs OP rhs
                        self.expr(lhs, out)?;
                        let op_str = match op {
                            Op::And => "&&",
                            Op::Or => "||",
                            Op::QuestionQuestion => "??",
                            _ => op.op(),
                        };
                        write!(out, " {} ", op_str)?;
                        self.expr(rhs, out)?;
                        // When assigning &str literal to a variable, add .to_string()
                        // In Auto, all str variables are String in Rust, so this is always correct
                        if matches!(op, Op::Asn) && matches!(rhs.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                            if let Expr::Ident(_) = lhs.as_ref() {
                                write!(out, ".to_string()")?;
                            }
                        }
                    }
                    Op::Eq | Op::Neq => {
                        // Plan 220 Task 5: When comparing with a single-char string literal,
                        // emit a char comparison instead of &str comparison.
                        // e.g.  ch == "a"  ->  ch == 'a'
                        let op_str = op.op(); // "==" or "!="
                        if let Expr::Str(s) = rhs.as_ref() {
                            if s.chars().count() == 1 {
                                self.expr(lhs, out)?;
                                write!(out, " {} ", op_str)?;
                                write!(out, "'{}'", escape_str(s))?;
                            } else {
                                self.expr(lhs, out)?;
                                write!(out, " {} ", op_str)?;
                                self.expr(rhs, out)?;
                            }
                        } else if let Expr::Str(s) = lhs.as_ref() {
                            if s.chars().count() == 1 {
                                write!(out, "'{}'", escape_str(s))?;
                                write!(out, " {} ", op_str)?;
                                self.expr(rhs, out)?;
                            } else {
                                self.expr(lhs, out)?;
                                write!(out, " {} ", op_str)?;
                                self.expr(rhs, out)?;
                            }
                        } else {
                            self.expr(lhs, out)?;
                            write!(out, " {} ", op_str)?;
                            self.expr(rhs, out)?;
                        }
                    }
                    _ => {
                        // Binary operators: lhs OP rhs
                        self.expr(lhs, out)?;
                        // Plan 072: Convert and/or to Rust's &&/||
                        // Plan 067: Support ?? operator (May system)
                        let op_str = match op {
                            Op::And => "&&",
                            Op::Or => "||",
                            Op::QuestionQuestion => "??",
                            _ => op.op(),
                        };
                        write!(out, " {} ", op_str)?;
                        self.expr(rhs, out)?;
                    }
                }
                Ok(())
            }

            Expr::Unary(op, expr) => {
                // Plan 052: Unary operators - handle address-of and dereference
                let op_str = match op {
                    Op::Add => "&", // Unary & for address-of
                    Op::Mul => "*", // Unary * for dereference
                    _ => op.op(),
                };
                // Plan 204 Phase 1C: Wrap operand in parens for ! to avoid
                // precedence issues (e.g., !expr <= val should be !(expr <= val))
                if matches!(op, Op::Not) {
                    write!(out, "!(",)?;
                    self.expr(expr, out)?;
                    write!(out, ")")?;
                } else {
                    write!(out, "{}", op_str)?;
                    self.expr(expr, out)?;
                }
                Ok(())
            }

            // **Phase 2: Borrow Checking System**
            Expr::View(expr) => {
                // e.view -> &e (immutable borrow)
                write!(out, "&")?;
                self.expr(expr, out)?;
                Ok(())
            }

            Expr::Mut(expr) => {
                // e.mut -> &mut e (mutable borrow)
                write!(out, "&mut ")?;
                self.expr(expr, out)?;
                Ok(())
            }

            Expr::Move(expr) | Expr::Take(expr) => {
                // e.move / e.take -> e (move semantics, default in Rust)
                // Plan 122: .move is preferred, .take is deprecated
                self.expr(expr, out)?;
                Ok(())
            }

            // Collections
            Expr::Array(arr) => {
                write!(out, "vec![")?;
                for (i, elem) in arr.iter().enumerate() {
                    self.expr(elem, out)?;
                    if i < arr.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]").map_err(Into::into)
            }

            Expr::Index(arr, idx) => {
                self.expr(arr, out)?;
                write!(out, "[")?;
                match idx.as_ref() {
                    Expr::Range(range) => {
                        // source[p..p+1] -> source[(p) as usize..(p + 1) as usize]
                        if Self::needs_usize_cast(&range.start) {
                            write!(out, "(")?;
                            self.expr(&range.start, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(&range.start, out)?;
                        }
                        write!(out, "{}", if range.eq { "..=" } else { ".." })?;
                        if Self::needs_usize_cast(&range.end) {
                            write!(out, "(")?;
                            self.expr(&range.end, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(&range.end, out)?;
                        }
                    }
                    _ => {
                        if Self::needs_usize_cast(idx) {
                            write!(out, "(")?;
                            self.expr(idx, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(idx, out)?;
                        }
                    }
                }
                write!(out, "]")?;
                // Non-range index access may move non-Copy types (String, struct);
                // add .clone() to safely handle all element types.
                if !matches!(idx.as_ref(), Expr::Range(_)) {
                    write!(out, ".clone()")?;
                }
                Ok(())
            }

            Expr::Range(range) => {
                self.expr(&range.start, out)?;
                if range.eq {
                    write!(out, "..=")?;
                } else {
                    write!(out, "..")?;
                }
                self.expr(&range.end, out).map_err(Into::into)
            }

            Expr::Pair(pair) => {
                // Pair expression: key: value
                let key = match &pair.key {
                    crate::ast::Key::NamedKey(name) => name.clone(),
                    crate::ast::Key::IntKey(n) => format!("{}", n).into(),
                    crate::ast::Key::BoolKey(b) => format!("{}", b).into(),
                    crate::ast::Key::StrKey(s) => s.clone(),
                };
                write!(out, "{}: ", key)?;
                self.expr(&pair.value, out).map_err(Into::into)
            }

            Expr::Object(pairs) => {
                // Object literal: {key1: value1, key2: value2}
                write!(out, "{{")?;
                for (i, pair) in pairs.iter().enumerate() {
                    self.expr(&Expr::Pair(pair.clone()), out)?;
                    if i < pairs.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "}}").map_err(Into::into)
            }

            Expr::Grid(grid) => {
                // Grid expression: 2D array
                // Convert to nested vec: vec![vec![...], ...]
                write!(out, "vec![")?;
                for (i, row) in grid.data.iter().enumerate() {
                    write!(out, "vec![")?;
                    for (j, cell) in row.iter().enumerate() {
                        self.expr(cell, out)?;
                        if j < row.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, "]")?;
                    if i < grid.data.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]").map_err(Into::into)
            }

            Expr::Cover(cover) => {
                // Cover expression for tagged unions
                match cover {
                    crate::ast::Cover::Tag(tag_cover) => {
                        let key = (tag_cover.kind.clone(), tag_cover.tag.clone());
                        let is_struct = self.enum_struct_variants.contains_key(&key);
                        let tuple_arity = self.enum_tuple_variants.get(&key).copied();

                        // Bare variant check (no bindings): Enum::Variant
                        if tag_cover.bindings.iter().all(|b| b.as_str() == "_") {
                            if let Some(arity) = tuple_arity {
                                // Tuple variant needs (_, _, ...): Enum::Variant(_, _, ...)
                                write!(out, "{}::{}(", tag_cover.kind, tag_cover.tag)?;
                                for j in 0..arity {
                                    if j > 0 { write!(out, ", ")?; }
                                    write!(out, "_")?;
                                }
                                write!(out, ")").map_err(Into::into)
                            } else if is_struct {
                                // Struct variant needs { .. }: Enum::Variant { .. }
                                write!(out, "{}::{} {{ .. }}", tag_cover.kind, tag_cover.tag)
                                    .map_err(Into::into)
                            } else {
                                write!(out, "{}::{}", tag_cover.kind, tag_cover.tag)
                                    .map_err(Into::into)
                            }
                        } else if is_struct {
                            // Struct variant: Enum::Variant { field1, field2 }
                            let field_names = self.enum_struct_variants.get(&key)
                                .map(|v| v.as_slice())
                                .unwrap_or(&[]);
                            write!(out, "{}::{} {{ ", tag_cover.kind, tag_cover.tag)?;
                            for (i, binding) in tag_cover.bindings.iter()
                                .filter(|b| b.as_str() != "_")
                                .enumerate()
                            {
                                if i > 0 { write!(out, ", ")?; }
                                // Use field name if available, otherwise binding name
                                if let Some(field_name) = field_names.get(i) {
                                    if field_name.as_str() == binding.as_str() {
                                        write!(out, "{}", field_name)?;
                                    } else {
                                        write!(out, "{}: {}", field_name, binding)?;
                                    }
                                } else {
                                    write!(out, "{}", binding)?;
                                }
                            }
                            write!(out, " }}").map_err(Into::into)
                        } else {
                            // Tuple variant or unknown: Enum::Variant(a, b)
                            let binding_str = tag_cover.bindings.iter()
                                .filter(|b| b.as_str() != "_")
                                .map(|b| b.as_str())
                                .collect::<Vec<_>>()
                                .join(", ");
                            write!(
                                out,
                                "{}::{}({})",
                                tag_cover.kind, tag_cover.tag, binding_str
                            )
                            .map_err(Into::into)
                        }
                    }
                }
            }

            Expr::Uncover(uncover) => {
                // Uncover expression for pattern matching
                write!(out, "/* TagUncover: {} */", uncover.src).map_err(Into::into)
            }

            // Plan 120/159: Option/Result uncover (extract inner value)
            Expr::OptionUncover(uncover) => {
                // OptionUncover: extract binding from Some variant
                // e.g., after `is x { Some(val) => ... }`, val is the binding
                write!(out, "{}", uncover.binding).map_err(Into::into)
            }
            Expr::ResultUncover(uncover) => {
                // ResultUncover: extract binding from Ok/Err variant
                write!(out, "{}", uncover.binding).map_err(Into::into)
            }

            // Plan 165: Struct destructuring pattern
            Expr::StructPattern(sc) => {
                match &sc.variant {
                    Some(variant) => {
                        write!(out, "{}::{}", sc.type_name, variant)?;
                    }
                    None => {
                        write!(out, "{}", sc.type_name)?;
                    }
                }
                write!(out, " {{ ")?;
                for (i, fb) in sc.fields.iter().enumerate() {
                    if fb.field == fb.binding {
                        write!(out, "{}", fb.field)?;
                    } else {
                        write!(out, "{}: {}", fb.field, fb.binding)?;
                    }
                    if i < sc.fields.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, " }}").map_err(Into::into)
            }

            // Plan 120/159: Option/Result patterns (used in is statement branches)
            // These are handled in is_stmt, not as standalone expressions.
            // Provide a fallback for cases where they appear as expressions.
            Expr::OptionPattern(cover) => {
                match cover.variant {
                    crate::ast::cover::OptionVariant::Some => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "Some({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "Some(_)").map_err(Into::into)
                        }
                    }
                    crate::ast::cover::OptionVariant::None => {
                        write!(out, "None").map_err(Into::into)
                    }
                }
            }
            Expr::ResultPattern(cover) => {
                match cover.variant {
                    crate::ast::cover::ResultVariant::Ok => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "Ok({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "Ok(_)").map_err(Into::into)
                        }
                    }
                    crate::ast::cover::ResultVariant::Err => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "Err({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "Err(_)").map_err(Into::into)
                        }
                    }
                }
            }

            Expr::Ref(name) => {
                // Reference expression: &name
                write!(out, "&{}", name).map_err(Into::into)
            }

            // Struct construction: Point(1, 2) -> Point { x: 1, y: 2 }
            // Special case: loop { body } -> loop { body }
            Expr::Node(node) => {
                if node.name == "not" {
                    write!(out, "!(")?;
                    if !node.id.is_empty() { write!(out, "{}", node.id)?; }
                    write!(out, ")")?;
                    return Ok(());
                }
                if node.name == "loop" {
                    write!(out, "loop {{")?;
                    if !node.body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();

                        for stmt in &node.body.stmts {
                            self.print_indent(out)?;
                            match stmt {
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    out.write(b";\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                Stmt::Break => {
                                    out.write(b"break;\n")?;
                                }
                                _ => {
                                    // For other statement types, format inline
                                    match stmt {
                                        Stmt::If(if_) => {
                                            // Inline if statement
                                            write!(out, "if ")?;
                                            for (i, branch) in if_.branches.iter().enumerate() {
                                                if i == 0 {
                                                } else {
                                                    write!(out, " else if ")?;
                                                }
                                                self.expr(&branch.cond, out)?;
                                                write!(out, " {{ ")?;
                                                // Multi-statement body
                                                for stmt in branch.body.stmts.iter() {
                                                    match stmt {
                                                        Stmt::Expr(expr) => {
                                                            self.expr(expr, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Break => {
                                                            write!(out, "break; ")?;
                                                        }
                                                        Stmt::Return(ret) => {
                                                            write!(out, "return ")?;
                                                            self.expr(ret, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Store(store) => {
                                                            self.store(store, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                write!(out, "}}")?;
                                            }
                                            if let Some(else_) = &if_.else_ {
                                                write!(out, " else {{ ")?;
                                                for stmt in else_.stmts.iter() {
                                                    match stmt {
                                                        Stmt::Expr(expr) => {
                                                            self.expr(expr, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Break => {
                                                            write!(out, "break; ")?;
                                                        }
                                                        Stmt::Return(ret) => {
                                                            write!(out, "return ")?;
                                                            self.expr(ret, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        Stmt::Store(store) => {
                                                            self.store(store, out)?;
                                                            write!(out, "; ")?;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                write!(out, "}}")?;
                                            }
                                            write!(out, "\n")?;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")
                } else {
                    // Regular struct construction
                    write!(out, "{} {{", node.name)?;
                    if !node.args.args.is_empty() || !node.body.stmts.is_empty() {
                        write!(out, " ")?;
                    }

                    // Get cached field names for this type (same as struct_init)
                    let field_names = self
                        .struct_fields
                        .get(&node.name)
                        .cloned()
                        .unwrap_or_default();

                    // Get cached field types for .to_string() auto-insertion
                    let field_types = self
                        .struct_field_types
                        .get(&node.name)
                        .cloned()
                        .unwrap_or_default();
                    for (i, arg) in node.args.args.iter().enumerate() {
                        let needs_to_string = match arg {
                            Arg::Pos(expr) => i < field_types.len()
                                && matches!(field_types[i].1, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice)
                                && !matches!(expr, Expr::Str(_) | Expr::CStr(_)),
                            Arg::Pair(key, _) => {
                                field_types.iter()
                                    .find(|(n, _)| *n == *key)
                                    .map(|(_, ty)| matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice))
                                    .unwrap_or(false)
                            }
                            _ => false,
                        };
                        match arg {
                            Arg::Pos(expr) => {
                                let field_name = if i < field_names.len() {
                                    field_names[i].clone()
                                } else {
                                    format!("field{}", i).into()
                                };
                                write!(out, "{}: ", field_name)?;
                                self.write_expr_for_struct_field(expr, out)?;
                            }
                            Arg::Name(name) => {
                                write!(out, "{}: ", name)?;
                            }
                            Arg::Pair(key, expr) => {
                                write!(out, "{}: ", key)?;
                                self.write_expr_for_struct_field(expr, out)?;
                            }
                        }
                        if needs_to_string {
                            write!(out, ".to_string()")?;
                        }
                        if i < node.args.args.len() - 1 || !node.body.stmts.is_empty() {
                            write!(out, ", ")?;
                        }
                    }

                    // Handle body statements (field initializers)
                    for (i, stmt) in node.body.stmts.iter().enumerate() {
                        let (field_name, field_expr): (AutoStr, &Expr) = match stmt {
                            Stmt::Store(store) => (store.name.clone(), &store.expr),
                            Stmt::Expr(Expr::Pair(pair)) => {
                                let name = match &pair.key {
                                    crate::ast::Key::NamedKey(name) => name.clone(),
                                    crate::ast::Key::IntKey(n) => format!("{}", n).into(),
                                    crate::ast::Key::BoolKey(b) => format!("{}", b).into(),
                                    crate::ast::Key::StrKey(s) => s.clone(),
                                };
                                (name, &pair.value)
                            }
                            _ => {
                                if i < node.body.stmts.len() - 1 {
                                    write!(out, ", ")?;
                                }
                                continue;
                            }
                        };

                        // Check if this field needs .to_string() (String field but &str value)
                        // write_expr_for_struct_field already handles string literals
                        let field_is_string = field_types.iter()
                            .find(|(n, _)| *n == field_name)
                            .map(|(_, ty)| matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice))
                            .unwrap_or(false);
                        let expr_is_str_slice = match field_expr {
                            Expr::Ident(name) => {
                                // Check if variable is &str type (not String)
                                self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false)
                            }
                            _ => false,
                        };
                        let needs_to_string = field_is_string && expr_is_str_slice;

                        write!(out, "{}: ", field_name)?;
                        self.write_expr_for_struct_field(field_expr, out)?;
                        if needs_to_string {
                            write!(out, ".to_string()")?;
                        }
                        if i < node.body.stmts.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }

                    if !node.args.args.is_empty() || !node.body.stmts.is_empty() {
                        write!(out, " ")?;
                    }
                    write!(out, "}}")
                }
                .map_err(Into::into)
            }

            // Function calls
            Expr::Call(call) => self.call(call, out),

            // F-strings: f"hello $name" -> format!("hello {}", name)
            Expr::FStr(fstr) => {
                write!(out, "format!(\"")?;
                let mut _arg_count = 0;
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            let escaped = s.replace("\\", "\\\\").replace("\"", r##"\""##)
                                .replace("{", "{{").replace("}", "}}");
                            write!(out, "{}", escaped)?;
                        }
                        Expr::Char(c) => {
                            write!(out, "{}", c)?;
                        }
                        _ => {
                            // Expression placeholder
                            write!(out, "{{}}")?;
                            _arg_count += 1;
                        }
                    }
                }
                write!(out, "\"")?;

                // Add arguments after format string
                for part in &fstr.parts {
                    match part {
                        Expr::Str(_) | Expr::CStr(_) | Expr::Char(_) => {}
                        _ => {
                            write!(out, ", ")?;
                            self.expr(part, out)?;
                        }
                    }
                }

                write!(out, ")").map_err(Into::into)
            }

            // Control flow (stub for now)
            Expr::If(if_) => {
                // Transpile if/else if/else chains
                for (i, branch) in if_.branches.iter().enumerate() {
                    if i == 0 {
                        write!(out, "if ")?;
                    } else {
                        write!(out, " else if ")?;
                    }

                    // Condition
                    self.expr(&branch.cond, out)?;
                    write!(out, " {{")?;

                    // Body
                    if !branch.body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();
                        let stmt_count = branch.body.stmts.len();
                        for (i, stmt) in branch.body.stmts.iter().enumerate() {
                            self.print_indent(out)?;
                            // Handle different statement types
                            let is_last = i == stmt_count - 1;
                            match stmt {
                                Stmt::Expr(Expr::If(inner_if)) => {
                                    // If expression - don't add semicolon
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    if is_last && self.ret_type_needs_string_coercion()
                                        && self.expr_needs_string_coercion(expr) {
                                        write!(out, ".to_string()")?;
                                    }
                                    if !is_last {
                                        out.write(b";\n")?;
                                    } else {
                                        out.write(b"\n")?;
                                    }
                                }
                                Stmt::If(inner_if) => {
                                    // Nested if statement - handle as expression
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                _ => {
                                    // Other statement types - handle Break, Return, etc.
                                    match stmt {
                                        Stmt::Break => {
                                            out.write(b"break;\n")?;
                                        }
                                        Stmt::Return(ret) => {
                                            out.write(b"return ")?;
                                            self.expr(ret, out)?;
                                            out.write(b";\n")?;
                                        }
                                        _ => {
                                            write!(out, "/* unsupported statement in if body */\n")?;
                                        }
                                    }
                                }
                            }
                        }
                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")?;
                }

                // Else clause
                if let Some(else_body) = &if_.else_ {
                    write!(out, " else {{")?;
                    if !else_body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();
                        let stmt_count = else_body.stmts.len();
                        for (i, stmt) in else_body.stmts.iter().enumerate() {
                            self.print_indent(out)?;
                            let is_last = i == stmt_count - 1;
                            match stmt {
                                Stmt::Expr(Expr::If(inner_if)) => {
                                    // Nested if expression in else
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    if is_last && self.ret_type_needs_string_coercion()
                                        && self.expr_needs_string_coercion(expr) {
                                        write!(out, ".to_string()")?;
                                    }
                                    if !is_last {
                                        out.write(b";\n")?;
                                    } else {
                                        out.write(b"\n")?;
                                    }
                                }
                                Stmt::If(inner_if) => {
                                    // Nested if statement - handle as expression
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                _ => {
                                    match stmt {
                                        Stmt::Break => {
                                            out.write(b"break;\n")?;
                                        }
                                        Stmt::Return(ret) => {
                                            out.write(b"return ")?;
                                            self.expr(ret, out)?;
                                            out.write(b";\n")?;
                                        }
                                        _ => {
                                            write!(out, "/* unsupported statement in else body */\n")?;
                                        }
                                    }
                                }
                            }
                        }
                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")?;
                } else {
                    // No else clause
                    write!(out, "")?;
                }

                Ok(())
            }

            // Lambda/closure: |params| body
            Expr::Lambda(lambda) => {
                write!(out, "|")?;
                for (i, param) in lambda.params.iter().enumerate() {
                    write!(out, "{}", self.rust_type_name(&param.ty))?;
                    write!(out, " {}", param.name)?;
                    if i < lambda.params.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "| ")?;

                // Lambda body - if it's a single expression, write it directly
                if lambda.body.stmts.len() == 1 {
                    match &lambda.body.stmts[0] {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                        }
                        _ => {
                            write!(out, "{{ /* unsupported lambda body */ }}")?;
                        }
                    }
                } else {
                    // Multiple statements - use block
                    // Plan 151 Phase 1.4: Support return statements in closures
                    write!(out, "{{ ")?;
                    for (i, stmt) in lambda.body.stmts.iter().enumerate() {
                        match stmt {
                            Stmt::Expr(expr) => {
                                self.expr(expr, out)?;
                                if i < lambda.body.stmts.len() - 1 {
                                    write!(out, "; ")?;
                                }
                            }
                            Stmt::Store(store) => {
                                self.store(store, out)?;
                                write!(out, "; ")?;
                            }
                            Stmt::Return(ret_expr) => {
                                // Return statement in closure
                                write!(out, "return ")?;
                                self.expr(ret_expr, out)?;
                                write!(out, "; ")?;
                            }
                            _ => {
                                write!(out, "/* unsupported statement */ ")?;
                            }
                        }
                    }
                    write!(out, "}}")?;
                }
                Ok(())
            }

            // Block expression: { stmt1; stmt2; expr }
            Expr::Block(body) => {
                write!(out, "{{ ")?;
                for stmt in &body.stmts {
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Return(ret_expr) => {
                            write!(out, "return ")?;
                            self.expr(ret_expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::For(for_stmt) => {
                            self.for_stmt_inline(for_stmt, out)?;
                        }
                        Stmt::EmptyLine(n) => {
                            for _ in 0..*n {
                                write!(out, "\n")?;
                            }
                        }
                        _ => {
                            write!(out, "/* unsupported stmt in block */ ")?;
                        }
                    }
                }
                write!(out, "}}")?;
                Ok(())
            }

            // Closure (Plan 060): (params) => body or param => body
            Expr::Closure(closure) => {
                write!(out, "|")?;
                for (i, param) in closure.params.iter().enumerate() {
                    // Name first, then optional type annotation
                    write!(out, "{}", param.name)?;
                    if let Some(ref ty) = param.ty {
                        write!(out, ": {}", self.rust_type_name(ty))?;
                    }
                    if i < closure.params.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "| ")?;

                // Closure body - it's a boxed expression
                self.expr(&closure.body, out)?;
                Ok(())
            }

            // Plan 056: Dot expression for field access
            Expr::Dot(object, field) => {
                // **Phase 1.1: Pointer Operators (test: 005_pointer)**
                // Handle @ (address-of) and * (dereference) as special field names
                match field.as_str() {
                    "@" => {
                        // x.@ -> raw pointer to x (address-of operator)
                        // In Rust, we need to cast reference to raw pointer
                        // x as *mut T
                        self.expr(object, out)?;
                        write!(out, " as *mut _")?; // Use _ for type inference
                        return Ok(());
                    }
                    "*" => {
                        // y.* -> *y (dereference operator)
                        // In Rust, we use * for dereference
                        write!(out, "*")?;
                        self.expr(object, out)?;
                        return Ok(());
                    }
                    // **Phase 2: Borrow Checking System**
                    "view" => {
                        // s.view -> &s (immutable borrow)
                        write!(out, "&")?;
                        self.expr(object, out)?;
                        return Ok(());
                    }
                    "mut" => {
                        // s.mut -> &mut s (mutable borrow)
                        write!(out, "&mut ")?;
                        self.expr(object, out)?;
                        return Ok(());
                    }
                    "take" => {
                        // s.take -> s (move semantics, default in Rust)
                        // Just emit the object itself (no additional syntax needed)
                        self.expr(object, out)?;
                        return Ok(());
                    }
                    // Plan 162: Array .ptr -> .as_mut_ptr() for raw pointer access
                    "ptr" => {
                        self.expr(object, out)?;
                        write!(out, ".as_mut_ptr()")?;
                        return Ok(());
                    }
                    _ => {}
                }

                // Check if this is an enum access or static method: Enum.Value -> Enum::Value
                // Use heuristic: if object is an identifier starting with uppercase or a known module
                // Also handle module.Type.method() where object is a nested Dot chain
                if let Expr::Ident(type_name) = object.as_ref() {
                    let is_type_name = type_name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                        || self.uses.iter().any(|u| {
                            let u_str = u.as_str();
                            u_str == type_name
                                || u_str.ends_with(&format!("::{}", type_name))
                        });
                    if is_type_name {
                        // Type::Variant (enum) or Type::method (static method)
                        write!(out, "{}::{}", type_name, field)?;
                        return Ok(());
                    }
                } else if let Expr::Bina(_, Op::Dot, _) = object.as_ref() {
                    // module.Type.method() — the object is a Dot chain, treat as type-like
                    self.expr(object, out)?;
                    write!(out, "::{}", field)?;
                    return Ok(());
                } else if let Expr::Dot(il, _) = object.as_ref() {
                    // module.Type.method() via Expr::Dot variant
                    if matches!(il.as_ref(), Expr::Ident(_)) {
                        self.expr(object, out)?;
                        write!(out, "::{}", field)?;
                        return Ok(());
                    }
                }

                // Regular field access: object.field
                // Some AutoLang properties map to Rust method calls
                let is_rust_method = matches!(
                    field.as_str(),
                    "len" | "is_empty" | "capacity" | "count" | "push" | "pop"
                );
                self.expr(object, out)?;
                write!(out, ".{}", field)?;
                if is_rust_method {
                    write!(out, "()")?;
                }
                Ok(())
            }

            Expr::NullCoalesce(lhs, rhs) => {
                // Null coalescing: lhs ?? rhs
                // In Rust, this becomes: lhs.unwrap_or(rhs)
                self.expr(lhs, out)?;
                write!(out, ".unwrap_or(")?;
                self.expr(rhs, out)?;
                if matches!(rhs.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                    write!(out, ".to_string()")?;
                }
                write!(out, ")")?;
                Ok(())
            }

            Expr::ErrorPropagate(expr) => {
                // Error propagation: expr.?
                // Plan 067: May system support
                self.expr(expr, out)?;
                write!(out, "?")?;
                Ok(())
            }

            // Plan 162: Type cast: expr.as(Type) -> (expr as Type)
            Expr::Cast { expr, target_type } => {
                write!(out, "(")?;
                self.expr(expr, out)?;
                write!(out, " as {})", self.rust_type_name(target_type))?;
                Ok(())
            }

            // Plan 162: Explicit type conversion: expr.to(Type)
            // Strategy: .to(str) generates .to_string() (always valid);
            // for string literal sources targeting numeric types, generate .parse::<T>().unwrap();
            // for all other numeric targets, degrade to `as` cast (same as .as()).
            // Future: refine based on source type inference.
            Expr::To { expr, target_type } => {
                match target_type {
                    Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => {
                        // x.to(str) / x.to(String) → format!("{:?}", x) for struct types,
                        // or x.to_string() for primitive types
                        // Since we lack type inference, use format!("{:?}", x) as safe default
                        // which works for all types that derive Debug
                        write!(out, "format!(\"{{:?}}\", ")?;
                        self.expr(expr, out)?;
                        write!(out, ")")?;
                    }
                    // For string literal sources, parse works; for others, use `as`
                    // Heuristic: check if expr is a string literal
                    Type::Int => {
                        if matches!(expr.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                            self.expr(expr, out)?;
                            write!(out, ".parse::<i32>().unwrap()")?;
                        } else {
                            write!(out, "(")?;
                            self.expr(expr, out)?;
                            write!(out, " as i32)")?;
                        }
                    }
                    Type::Float | Type::Double => {
                        if matches!(expr.as_ref(), Expr::Str(_) | Expr::CStr(_)) {
                            self.expr(expr, out)?;
                            write!(out, ".parse::<f64>().unwrap()")?;
                        } else {
                            write!(out, "(")?;
                            self.expr(expr, out)?;
                            write!(out, " as f64)")?;
                        }
                    }
                    _ => {
                        // Check if target is a string-like type name (String, str, etc.)
                        let ty_name = self.rust_type_name(target_type);
                        if ty_name == "String" || ty_name == "str" || ty_name == "&str" {
                            // x.to(String) / x.to(str) → format!("{:?}", x)
                            write!(out, "format!(\"{{:?}}\", ")?;
                            self.expr(expr, out)?;
                            write!(out, ")")?;
                        } else {
                            // Fallback: treat as cast (same as .as())
                            write!(out, "(")?;
                            self.expr(expr, out)?;
                            write!(out, " as {})", ty_name)?;
                        }
                    }
                }
                Ok(())
            }

            // Plan 124: Async/Future/Await system
            Expr::AsyncBlock { body, return_type: _ } => {
                // ~{ stmts } -> async { stmts }
                write!(out, "async {{ ")?;
                for stmt in &body.stmts {
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Return(ret_expr) => {
                            write!(out, "return ")?;
                            self.expr(ret_expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Reply(expr) => {
                            // Plan 124 Phase 2.3: reply expr
                            // In async context, reply sends to oneshot channel
                            write!(out, "let _ = reply_tx.send(")?;
                            self.expr(expr, out)?;
                            write!(out, "); ")?;
                        }
                        _ => {
                            // For other statements, use stmt method (which requires Sink)
                            // For now, skip complex statements in async blocks
                        }
                    }
                }
                write!(out, "}}")?;
                Ok(())
            }

            Expr::Await { expr } => {
                // Check if the inner expression is a self-awaited call (like http.post_sync)
                // that already contains .await internally — if so, skip the outer .await
                if let Expr::Call(call) = expr.as_ref() {
                    if let Expr::Dot(obj, method) = call.name.as_ref() {
                        if let Expr::Ident(obj_name) = obj.as_ref() {
                            let m = method.as_str();
                            if obj_name.as_str() == "http" && (m == "post_sync" || m == "post_bearer") {
                                // Already handled with internal .await — just output the expression
                                return self.call(call, out);
                            }
                        }
                    }
                }
                // expr.await -> expr.await
                self.expr(expr, out)?;
                write!(out, ".await")?;
                Ok(())
            }

            // Plan 126: .go postfix operator - fire-and-forget spawn
            // expr.go -> tokio::spawn(async move { expr.await })
            // The expression is spawned as a background task, result is discarded
            Expr::Go { expr } => {
                write!(out, "tokio::spawn(async move {{ ")?;
                self.expr(expr, out)?;
                write!(out, ".await; }})")?;
                Ok(())
            }

            // Plan 223: is as expression → Rust match expression
            Expr::Is(is) => {
                write!(out, "match ")?;
                self.expr(&is.target, out)?;
                write!(out, " {{ ")?;
                for (i, branch) in is.branches.iter().enumerate() {
                    if i > 0 { write!(out, " ")?; }
                    match branch {
                        crate::ast::IsBranch::EqBranch(patterns, body) => {
                            for (j, pat) in patterns.iter().enumerate() {
                                if j > 0 { write!(out, " | ")?; }
                                self.expr(pat, out)?;
                            }
                            write!(out, " => ")?;
                            self.write_body_inline(body, out)?;
                            write!(out, ",")?;
                        }
                        crate::ast::IsBranch::IfBranch(cond, body) => {
                            self.expr(cond, out)?;
                            write!(out, " if true => ")?;
                            self.write_body_inline(body, out)?;
                            write!(out, ",")?;
                        }
                        crate::ast::IsBranch::ElseBranch(body) => {
                            write!(out, "_ => ")?;
                            self.write_body_inline(body, out)?;
                            write!(out, ",")?;
                        }
                    }
                }
                write!(out, " }}")?;
                Ok(())
            }

            _ => Err(format!("Rust Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // Detect Rust macro pattern: name!("...") was parsed as name.collect()("...")
        // because '!' is the eager collection operator in Auto.
        // Parser creates: Expr::Bina(lhs, Dot, "collect") then wraps in Call.
        // AST: Call { name: Call { name: Bina(Ident(name), Dot, "collect"), args: [] }, args: [...] }
        if let Expr::Call(inner) = call.name.as_ref() {
            if let Expr::Bina(obj, Op::Dot, field) = inner.name.as_ref() {
                if let Expr::Ident(field_name) = field.as_ref() {
                    if field_name.as_str() == "collect" {
                        if let Expr::Ident(macro_name) = obj.as_ref() {
                            if inner.args.args.is_empty() {
                                // Known Rust macros from log/tracing crates
                                if matches!(macro_name.as_str(),
                                    "debug" | "info" | "warn" | "error" | "trace"
                                    | "println" | "eprintln" | "print" | "eprint"
                                    | "write" | "writeln" | "format"
                                    | "panic" | "assert" | "assert_eq" | "assert_ne"
                                    | "todo" | "unimplemented" | "unreachable"
                                    | "vec" | "include_str" | "concat" | "env") {
                                    write!(out, "{}!(", macro_name)?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        self.arg(arg, out)?;
                                        if i < call.args.args.len() - 1 {
                                            write!(out, ", ")?;
                                        }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Special case for print / write function
        if let Expr::Ident(name) = call.name.as_ref() {
            if name == "print" {
                return self.output_call(call, out, true);
            }
            if name == "write" {
                return self.output_call(call, out, false);
            }
            // Convert printf(fmt, args...) -> print!(fmt, args...)
            if name == "printf" {
                write!(out, "print!(")?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    self.arg(arg, out)?;
                    if i < call.args.args.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, ")")?;
                return Ok(());
            }
        }

        // Detect Rust macro calls imported via use.rust (e.g., use.rust log::debug → debug!("..."))
        // When call.name is Ident("debug") and self.uses contains "log::debug", emit debug!(...)
        if let Expr::Ident(name) = call.name.as_ref() {
            let name_str = name.as_str();
            let is_imported_macro = self.uses.iter().any(|u| {
                let u_str = u.as_str();
                u_str.ends_with(&format!("::{}", name_str))
            });
            if is_imported_macro && matches!(name_str,
                "debug" | "info" | "warn" | "error" | "trace"
                | "println" | "eprintln" | "print" | "eprint"
                | "format" | "vec" | "write" | "writeln"
                | "log" | "log_enabled") {
                write!(out, "{}!(", name)?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 { write!(out, ", ")?; }
                    if let Arg::Pos(Expr::FStr(fstr)) = arg {
                        // Inline f-string as macro format string
                        write!(out, "\"")?;
                        for part in &fstr.parts {
                            match part {
                                Expr::Str(s) | Expr::CStr(s) => {
                                    let escaped = s.replace("\\", "\\\\").replace("\"", r##"\""##)
                                        .replace("{", "{{").replace("}", "}}");
                                    write!(out, "{}", escaped)?;
                                }
                                Expr::Char(c) => { write!(out, "{}", c)?; }
                                _ => { write!(out, "{{}}")?; }
                            }
                        }
                        write!(out, "\"")?;
                        for part in &fstr.parts {
                            match part {
                                Expr::Str(_) | Expr::CStr(_) | Expr::Char(_) => {}
                                _ => { write!(out, ", ")?; self.expr(part, out)?; }
                            }
                        }
                    } else {
                        self.arg(arg, out)?;
                    }
                }
                write!(out, ")")?;
                return Ok(());
            }
        }

        // Plan 204 Phase 1A: Rust assert/assert_eq/assert_ne/panic are macros, need ! suffix
        // Special: when 2nd arg is an f-string, inline it directly (not format!())
        // because Rust assert! expects a string literal as the format arg.
        if let Expr::Ident(name) = call.name.as_ref() {
            if matches!(name.as_str(), "assert" | "assert_eq" | "assert_ne" | "panic") {
                write!(out, "{}!(", name)?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 { write!(out, ", ")?; }
                    // Check if this arg is an f-string — inline it without format!()
                    if let Arg::Pos(Expr::FStr(fstr)) = arg {
                        write!(out, "\"")?;
                        for part in &fstr.parts {
                            match part {
                                Expr::Str(s) | Expr::CStr(s) => {
                                    let escaped = s.replace("\\", "\\\\").replace("\"", r##"\""##)
                                        .replace("{", "{{").replace("}", "}}");
                                    write!(out, "{}", escaped)?;
                                }
                                Expr::Char(c) => {
                                    write!(out, "{}", c)?;
                                }
                                _ => {
                                    write!(out, "{{}}")?;
                                }
                            }
                        }
                        write!(out, "\"")?;
                        // Add format arguments
                        for part in &fstr.parts {
                            match part {
                                Expr::Str(_) | Expr::CStr(_) | Expr::Char(_) => {}
                                _ => {
                                    write!(out, ", ")?;
                                    self.expr(part, out)?;
                                }
                            }
                        }
                    } else {
                        self.arg(arg, out)?;
                    }
                }
                write!(out, ")")?;
                return Ok(());
            }
        }

        // Plan 223: Function name mappings for external calls
        if let Expr::Ident(name) = call.name.as_ref() {
            if name == "not" {
                write!(out, "!(")?;
                if let Some(Arg::Pos(expr)) = call.args.args.first() { self.expr(expr, out)?; }
                write!(out, ")")?;
                return Ok(());
            }
            match name.as_str() {
                "sleep_ms" => {
                    write!(out, "std::thread::sleep(std::time::Duration::from_millis(")?;
                    if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                    write!(out, " as u64))")?;
                    return Ok(());
                }
                "http_post" => {
                    // http_post(url, body, api_key) → async { let (s,b,e,k) = a2r_std::http_post(...).await; HttpResponse { ... } }
                    write!(out, "async {{ let (status, body, error, kind) = a2r_std::http_post(")?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if i > 0 { write!(out, ", ")?; }
                        if let Arg::Pos(expr) = arg {
                            self.expr(expr, out)?;
                            // Auto-borrow: add .as_str() for String → &str
                            if !matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                let is_str_slice = if let Expr::Ident(name) = expr {
                                    self.local_var_types.get(name)
                                        .map(|ty| matches!(ty, Type::StrSlice))
                                        .unwrap_or(false)
                                } else { false };
                                if !is_str_slice { write!(out, ".as_str()")?; }
                            }
                        } else { self.arg(arg, out)?; }
                    }
                    write!(out, ").await; HttpResponse {{ status, body, error, kind }} }}")?;
                    return Ok(());
                }
                _ => {}
            }
        }

        // Handle Expr::Dot calls: http.post_sync(...), http.last_status(), env.get(...), etc.
        // Parser generates Expr::Dot(Ident("http"), "post_sync") for two-segment module calls.
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if let Expr::Ident(obj_name) = obj.as_ref() {
                match (obj_name.as_str(), method.as_str()) {
                    ("http", "post_sync") => {
                        write!(out, "{{ let __resp = a2r_std::http::post_sync(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                if !matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                    if let Expr::Ident(name) = expr {
                                        if self.local_var_types.get(name)
                                            .map(|ty| !matches!(ty, Type::StrSlice))
                                            .unwrap_or(true)
                                        { write!(out, ".as_str()")?; }
                                    } else {
                                        write!(out, ".as_str()")?;
                                    }
                                }
                            }
                        }
                        write!(out, "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}")?;
                        return Ok(());
                    }
                    ("http", "last_status") => {
                        write!(out, "a2r_std::http::last_status()")?;
                        return Ok(());
                    }
                    ("http", "post_bearer") => {
                        write!(out, "{{ let __resp = a2r_std::http::post_bearer(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                if !matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                    if let Expr::Ident(name) = expr {
                                        if self.local_var_types.get(name)
                                            .map(|ty| !matches!(ty, Type::StrSlice))
                                            .unwrap_or(true)
                                        { write!(out, ".as_str()")?; }
                                    } else {
                                        write!(out, ".as_str()")?;
                                    }
                                }
                            }
                        }
                        write!(out, "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}")?;
                        return Ok(());
                    }
                    ("http", "post") => {
                        write!(out, "async {{ let (status, body, error, kind) = a2r_std::http::post(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                if let Expr::Ident(name) = expr {
                                    if self.local_var_types.get(name)
                                        .map(|ty| !matches!(ty, Type::StrSlice))
                                        .unwrap_or(true)
                                    { write!(out, ".as_str()")?; }
                                }
                            }
                        }
                        write!(out, ").await; HttpResponse {{ status, body, error, kind }} }}")?;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // Plan 223: Method call mappings for env.x / fs.x
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if let (Expr::Bina(inner_lhs, Op::Dot, inner_rhs), Expr::Ident(method)) = (lhs.as_ref(), rhs.as_ref()) {
                    // Handle auto.module.method(args) → a2r_std::module::method(args)
                    if let (Expr::Ident(auto_name), Expr::Ident(module)) = (inner_lhs.as_ref(), inner_rhs.as_ref()) {
                        if auto_name == "auto" {
                            match (module.as_str(), method.as_str()) {
                                ("env", "get") => {
                                    write!(out, "a2r_std::env::get(")?;
                                    if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("env", "args") => {
                                    write!(out, "a2r_std::env::args()")?;
                                    return Ok(());
                                }
                                ("io", "read_line") => {
                                    write!(out, "a2r_std::io::read_line()")?;
                                    return Ok(());
                                }
                                ("env", "set") => {
                                    write!(out, "a2r_std::env::set(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 { write!(out, ", ")?; }
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "read_text") => {
                                    write!(out, "a2r_std::fs::read_text(")?;
                                    if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "read_to_string") => {
                                    write!(out, "a2r_std::fs::read_to_string(")?;
                                    if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "write") => {
                                    write!(out, "a2r_std::fs::write(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 { write!(out, ", ")?; }
                                        if i == 1 { write!(out, "&")?; }
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "exists") => {
                                    write!(out, "a2r_std::fs::exists(")?;
                                    if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("http", "post") => {
                                    // auto.http.post(url, body, key) → wraps a2r_std::http::post into HttpResponse
                                    write!(out, "async {{ let (status, body, error, kind) = a2r_std::http::post(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 { write!(out, ", ")?; }
                                        if let Arg::Pos(expr) = arg {
                                            self.expr(expr, out)?;
                                            if let Expr::Ident(name) = expr {
                                                if self.local_var_types.get(name)
                                                    .map(|ty| !matches!(ty, Type::StrSlice))
                                                    .unwrap_or(true)
                                                {
                                                    write!(out, ".as_str()")?;
                                                }
                                            }
                                        }
                                    }
                                    write!(out, ").await; HttpResponse {{ status, body, error, kind }} }}")?;
                                    return Ok(());
                                }
                                ("http", "post_sync") => {
                                    write!(out, "{{ let __resp = a2r_std::http::post_sync(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 { write!(out, ", ")?; }
                                        if let Arg::Pos(expr) = arg {
                                            self.expr(expr, out)?;
                                            if let Expr::Ident(name) = expr {
                                                if self.local_var_types.get(name)
                                                    .map(|ty| !matches!(ty, Type::StrSlice))
                                                    .unwrap_or(true)
                                                {
                                                    write!(out, ".as_str()")?;
                                                }
                                            }
                                        }
                                    }
                                    write!(out, "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}")?;
                                    return Ok(());
                                }
                                ("http", "last_status") => {
                                    // http.last_status() → a2r_std::http::last_status()
                                    write!(out, "a2r_std::http::last_status()")?;
                                    return Ok(());
                                }
                                ("json", "parse") => {
                                    write!(out, "a2r_std::json::parse(")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "get") => {
                                    write!(out, "a2r_std::json::get(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                    write!(out, ", ")?;
                                    if call.args.args.len() > 1 {
                                        if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "get_str") => {
                                    write!(out, "a2r_std::json::get_str(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                    write!(out, ", ")?;
                                    if call.args.args.len() > 1 {
                                        if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                if let (Expr::Ident(obj), Expr::Ident(method)) = (lhs.as_ref(), rhs.as_ref()) {
                    match obj.as_str() {
                        "env" => match method.as_str() {
                            "get" => {
                                write!(out, "a2r_std::env::get(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "set" => {
                                write!(out, "a2r_std::env::set(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "fs" => match method.as_str() {
                            "read_to_string" => {
                                write!(out, "a2r_std::fs::read_to_string(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "read_text" => {
                                write!(out, "a2r_std::fs::read_text(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "write" => {
                                write!(out, "a2r_std::fs::write(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    if i == 1 { write!(out, "&")?; }
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "exists" => {
                                write!(out, "a2r_std::fs::exists(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "create_dir" => {
                                write!(out, "a2r_std::fs::create_dir(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "write_text" => {
                                write!(out, "a2r_std::fs::write_text(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    if i == 1 { write!(out, "&")?; }
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "is_dir" => {
                                write!(out, "a2r_std::fs::is_dir(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "is_binary" => {
                                write!(out, "a2r_std::fs::is_binary(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "walk" => {
                                write!(out, "a2r_std::fs::walk(")?;
                                if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "Json" => match method.as_str() {
                            "parse" => {
                                // Json.parse(text) -> a2r_std::json::parse(text)
                                write!(out, "a2r_std::json::parse(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get" => {
                                // Json.get(val, key) -> a2r_std::json::get(&val, key)
                                write!(out, "a2r_std::json::get(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] {
                                        self.expr(a, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get_str" => {
                                // Json.get_str(val, key) -> a2r_std::json::get_str(&val, key)
                                write!(out, "a2r_std::json::get_str(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] {
                                        self.expr(a, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "as_string" => {
                                // Json.as_string(val) -> a2r_std::json::as_string(val)
                                write!(out, "a2r_std::json::as_string(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get_at" => {
                                // Json.get_at(val, idx) -> a2r_std::json::get_at(&val, idx as usize)
                                write!(out, "a2r_std::json::get_at(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] {
                                        self.expr(a, out)?;
                                        write!(out, " as usize")?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get_u64" => {
                                // Json.get_u64(val, key) -> a2r_std::json::get_u64(&val, key)
                                write!(out, "a2r_std::json::get_u64(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] {
                                        self.expr(a, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "json" => match method.as_str() {
                            "parse" => {
                                write!(out, "a2r_std::json::parse(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get" => {
                                write!(out, "a2r_std::json::get(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get_str" => {
                                write!(out, "a2r_std::json::get_str(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "as_string" => {
                                write!(out, "a2r_std::json::as_string(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get_at" => {
                                write!(out, "a2r_std::json::get_at(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; write!(out, " as usize")?; }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "keys" => {
                                write!(out, "a2r_std::json::keys(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "len" => {
                                // Choose len_str (for &str) or len (for &Value) based on arg type
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    let is_str_type = if let Expr::Ident(name) = expr {
                                        self.local_var_types.get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                                            .unwrap_or(true) // default to str for unknown vars
                                    } else {
                                        matches!(expr, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                                    };
                                    if is_str_type {
                                        write!(out, "a2r_std::json::len_str(")?;
                                    } else {
                                        write!(out, "a2r_std::json::len(")?;
                                    }
                                    self.expr(expr, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "has_key" => {
                                // Choose has_key (for &Value) or has_key_str (for &str)
                                if let Some(Arg::Pos(first)) = call.args.args.first() {
                                    let use_str = if let Expr::Ident(name) = first {
                                        self.local_var_types.get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                                            .unwrap_or(true)
                                    } else {
                                        matches!(first, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                                    };
                                    if use_str {
                                        write!(out, "a2r_std::json::has_key_str(")?;
                                    } else {
                                        write!(out, "a2r_std::json::has_key(&")?;
                                    }
                                    self.expr(first, out)?;
                                }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "http" => match method.as_str() {
                            "post" => {
                                write!(out, "async {{ let (status, body, error, kind) = a2r_std::http::post(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if let Expr::Ident(name) = expr {
                                            if self.local_var_types.get(name)
                                                .map(|ty| !matches!(ty, Type::StrSlice))
                                                .unwrap_or(true)
                                            { write!(out, ".as_str()")?; }
                                        }
                                    }
                                }
                                write!(out, ").await; HttpResponse {{ status, body, error, kind }} }}")?;
                                return Ok(());
                            }
                            "post_sync" => {
                                write!(out, "{{ let __resp = a2r_std::http::post_sync(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if let Expr::Ident(name) = expr {
                                            if self.local_var_types.get(name)
                                                .map(|ty| !matches!(ty, Type::StrSlice))
                                                .unwrap_or(true)
                                            { write!(out, ".as_str()")?; }
                                        }
                                    }
                                }
                                write!(out, "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}")?;
                                return Ok(());
                            }
                            "last_status" => {
                                write!(out, "a2r_std::http::last_status()")?;
                                return Ok(());
                            }
                            "post_bearer" => {
                                write!(out, "{{ let __resp = a2r_std::http::post_bearer(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if let Expr::Ident(name) = expr {
                                            if self.local_var_types.get(name)
                                                .map(|ty| !matches!(ty, Type::StrSlice))
                                                .unwrap_or(true)
                                            { write!(out, ".as_str()")?; }
                                        }
                                    }
                                }
                                write!(out, "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "shell" => match method.as_str() {
                            "exec" => {
                                write!(out, "a2r_std::shell::exec(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if !matches!(expr, Expr::Int(_) | Expr::Float(_, _)) {
                                            write!(out, ".as_str()")?;
                                        }
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "regex" => match method.as_str() {
                            "match" => {
                                write!(out, "a2r_std::regex::r#match(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 { write!(out, ", ")?; }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }

        // Plan 124 Phase 2.2: Handle TaskHandle.send_await(msg) -> tx.send(msg).await
        // This transforms the method call to use Rust's async send pattern
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if let Expr::Ident(method_name) = rhs.as_ref() {
                    if method_name.as_str() == "send_await" {
                        // Transform: obj.send_await(msg) -> obj.send(msg).await
                        self.expr(lhs, out)?;
                        write!(out, ".send(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            self.arg(arg, out)?;
                            if i < call.args.args.len() - 1 {
                                write!(out, ", ")?;
                            }
                        }
                        write!(out, ").await")?;
                        return Ok(());
                    }
                    // Plan 124 Phase 2.3: Handle TaskHandle.ask(msg) -> ask pattern
                    // obj.ask(msg).await -> (oneshot channel + send + recv).await
                    if method_name.as_str() == "ask" {
                        // Simplified: just generate method call
                        // Full implementation would inject oneshot channel creation
                        self.expr(lhs, out)?;
                        write!(out, ".ask(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            self.arg(arg, out)?;
                            if i < call.args.args.len() - 1 {
                                write!(out, ", ")?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                }
            }
        }

        // Plan 151 Phase 1.3 + Plan 204 Phase 5: Method call translations
        // Translate Auto method names to Rust equivalents
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if let Expr::Ident(method_name) = rhs.as_ref() {
                    // Plan 204 Phase 5: Complex method translations requiring
                    // non-trivial Rust output (not just a name remap).
                    match method_name.as_str() {
                        "set" => {
                            // Map.set(key, val) -> HashMap::insert(key, val)
                            self.expr(lhs, out)?;
                            write!(out, ".insert(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 { write!(out, ", ")?; }
                                self.arg(arg, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "contains" => {
                            // Map.contains(key) -> HashMap::contains_key(key)
                            self.expr(lhs, out)?;
                            write!(out, ".contains_key(")?;
                            if let Some(arg) = call.args.args.first() { self.arg(arg, out)?; }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "char_at" => {
                            // s.char_at(i) -> s.chars().nth(i as usize).unwrap_or('\0')
                            self.expr(lhs, out)?;
                            write!(out, ".chars().nth(")?;
                            if let Some(Arg::Pos(arg)) = call.args.args.first() {
                                self.expr(arg, out)?;
                            }
                            write!(out, " as usize).unwrap_or('\\0')")?;
                            return Ok(());
                        }
                        "sub" => {
                            // s.sub(start, end) -> &s[start..end]
                            write!(out, "&")?;
                            self.expr(lhs, out)?;
                            write!(out, "[")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                if Self::needs_usize_cast(a) {
                                    write!(out, "(")?;
                                    self.expr(a, out)?;
                                    write!(out, ") as usize")?;
                                } else {
                                    self.expr(a, out)?;
                                }
                            }
                            write!(out, "..")?;
                            if call.args.args.len() > 1 {
                                if let Arg::Pos(a) = &call.args.args[1] {
                                    if Self::needs_usize_cast(a) {
                                        write!(out, "(")?;
                                        self.expr(a, out)?;
                                        write!(out, ") as usize")?;
                                    } else {
                                        self.expr(a, out)?;
                                    }
                                }
                            }
                            write!(out, "].to_string()")?;
                            return Ok(());
                        }
                        "slice" => {
                            // s.slice(n) -> s[n..].to_string()
                            // s.slice(start, end) -> s[start..end].to_string()
                            self.expr(lhs, out)?;
                            write!(out, "[")?;
                            let args = &call.args.args;
                            if let Some(Arg::Pos(a)) = args.first() {
                                if Self::needs_usize_cast(a) {
                                    write!(out, "(")?;
                                    self.expr(a, out)?;
                                    write!(out, ") as usize")?;
                                } else {
                                    self.expr(a, out)?;
                                }
                            }
                            if args.len() >= 2 {
                                if let Some(Arg::Pos(b)) = args.get(1) {
                                    write!(out, "..")?;
                                    if Self::needs_usize_cast(b) {
                                        write!(out, "(")?;
                                        self.expr(b, out)?;
                                        write!(out, ") as usize")?;
                                    } else {
                                        self.expr(b, out)?;
                                    }
                                }
                                write!(out, "]")?;
                            } else {
                                write!(out, "..]")?;
                            }
                            write!(out, ".to_string()")?;
                            return Ok(());
                        }
                        "repeat" => {
                            // s.repeat(n) -> s.repeat(n as usize)
                            self.expr(lhs, out)?;
                            write!(out, ".repeat(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                                write!(out, " as usize")?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "find" => {
                            // s.find(needle, start_pos?) -> a2r_std::str_find(s, needle, start_pos?)
                            // Returns i32 (-1 if not found), matching Auto semantics
                            write!(out, "a2r_std::str_find(")?;
                            self.expr(lhs, out)?;
                            for arg in &call.args.args {
                                write!(out, ", ")?;
                                self.arg(arg, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "to_hex" => {
                            // val.to_hex(width) -> format!("{:0>width$x}", val, width = width)
                            write!(out, "format!(\"{{:0>width$x}}\", ")?;
                            self.expr(lhs, out)?;
                            write!(out, ", width = ")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        _ => {} // fall through to simple name-remap table
                    }

                    // Simple name-remap table
                    // .len()/.length() returns usize, cast to i32 for Auto's int
                    let needs_i32_cast_1 = matches!(method_name.as_str(), "len" | "length");
                    let rust_method = match method_name.as_str() {
                        // String methods
                        "to_lower" => Some("to_lowercase"),
                        "to_upper" => Some("to_uppercase"),
                        "length" | "len" => Some("len"),
                        "is_empty" => Some("is_empty"),
                        "trim" => Some("trim"),
                        "trim_left" => Some("trim_start"),
                        "trim_right" => Some("trim_end"),
                        "starts_with" => Some("starts_with"),
                        "ends_with" => Some("ends_with"),
                        "append" => Some("push_str"),
                        // Collection methods
                        "push" => Some("push"),
                        "pop" => Some("pop"),
                        "clear" => Some("clear"),
                        "to_array" => Some("clone"),
                        "contains" => Some("contains"),
                        "retain" => Some("retain"),
                        // Type conversion
                        "to_string" => Some("to_string"),
                        _ => None,
                    };

                    if let Some(rust_name) = rust_method {
                        let lhs_parens = matches!(lhs.as_ref(),
                            Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
                        );
                        if lhs_parens { write!(out, "(")?; }
                        self.expr(lhs, out)?;
                        if lhs_parens { write!(out, ")")?; }
                        write!(out, ".{}(", rust_name)?;
                        // Auto-borrow string args for pattern-matching methods
                        if matches!(method_name.as_str(), "contains" | "starts_with" | "ends_with") {
                            for (i, arg) in call.args.args.iter().enumerate() {
                                write!(out, "&")?;
                                self.arg(arg, out)?;
                                if i < call.args.args.len() - 1 {
                                    write!(out, ", ")?;
                                }
                            }
                        } else {
                            for (i, arg) in call.args.args.iter().enumerate() {
                                self.arg(arg, out)?;
                                if i < call.args.args.len() - 1 {
                                    write!(out, ", ")?;
                                }
                            }
                        }
                        write!(out, ")")?;
                        if needs_i32_cast_1 {
                            write!(out, " as i32")?;
                        }
                        // trim/trim_start/trim_end return &str, auto-convert to String
                        if matches!(method_name.as_str(), "trim" | "trim_left" | "trim_right") {
                            write!(out, ".to_string()")?;
                        }
                        return Ok(());
                    }
                }
            }
        }

        // Also handle Expr::Dot method calls (parser emits Dot for method calls)
        if let Expr::Dot(object, method_name) = call.name.as_ref() {
            // Plan 162: Pointer intrinsic methods (only unique names that won't conflict)
            // ptr.is_null() -> ptr.is_null()
            // ptr.is_not_null() -> !ptr.is_null()
            match method_name.as_str() {
                "is_null" => {
                    self.expr(object, out)?;
                    write!(out, ".is_null()")?;
                    return Ok(());
                }
                "is_not_null" => {
                    write!(out, "(!")?;
                    self.expr(object, out)?;
                    write!(out, ".is_null())")?;
                    return Ok(());
                }
                // Plan 204 Phase 5: Complex method translations requiring
                // non-trivial Rust output (not just a name remap).
                "char_at" => {
                    // s.char_at(i) -> s.chars().nth(i as usize).unwrap_or('\0')
                    self.expr(object, out)?;
                    write!(out, ".chars().nth(")?;
                    if let Some(Arg::Pos(arg)) = call.args.args.first() {
                        self.expr(arg, out)?;
                    }
                    write!(out, " as usize).unwrap_or('\\0')")?;
                    return Ok(());
                }
                "sub" => {
                    // s.sub(start, end) -> s[start..end].to_string()
                    self.expr(object, out)?;
                    write!(out, "[")?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        if Self::needs_usize_cast(a) {
                            write!(out, "(")?;
                            self.expr(a, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(a, out)?;
                        }
                    }
                    write!(out, "..")?;
                    if call.args.args.len() > 1 {
                        if let Arg::Pos(a) = &call.args.args[1] {
                            if Self::needs_usize_cast(a) {
                                write!(out, "(")?;
                                self.expr(a, out)?;
                                write!(out, ") as usize")?;
                            } else {
                                self.expr(a, out)?;
                            }
                        }
                    }
                    write!(out, "].to_string()")?;
                    return Ok(());
                }
                "slice" => {
                    // s.slice(n) -> s[n..].to_string()
                    // s.slice(start, end) -> s[start..end].to_string()
                    self.expr(object, out)?;
                    write!(out, "[")?;
                    let args = &call.args.args;
                    if let Some(Arg::Pos(a)) = args.first() {
                        if Self::needs_usize_cast(a) {
                            write!(out, "(")?;
                            self.expr(a, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.expr(a, out)?;
                        }
                    }
                    if args.len() >= 2 {
                        if let Some(Arg::Pos(b)) = args.get(1) {
                            write!(out, "..")?;
                            if Self::needs_usize_cast(b) {
                                write!(out, "(")?;
                                self.expr(b, out)?;
                                write!(out, ") as usize")?;
                            } else {
                                self.expr(b, out)?;
                            }
                        }
                        write!(out, "]")?;
                    } else {
                        write!(out, "..]")?;
                    }
                    write!(out, ".to_string()")?;
                    return Ok(());
                }
                "repeat" => {
                    // s.repeat(n) -> s.repeat(n as usize)
                    self.expr(object, out)?;
                    write!(out, ".repeat(")?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        self.expr(a, out)?;
                        write!(out, " as usize")?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "to_int" => {
                    // Check if object is json.get() result or known non-string type
                    let use_value_helper = match object.as_ref() {
                        Expr::Call(c) => {
                            if let Expr::Dot(obj, method) = c.name.as_ref() {
                                if let Expr::Ident(name) = obj.as_ref() {
                                    name == "json" && (method == "get" || method == "get_at")
                                } else { false }
                            } else { false }
                        }
                        Expr::Ident(name) => {
                            self.local_var_types.get(name)
                                .map(|ty| matches!(ty, Type::User(_) | Type::Enum(_) | Type::Tag(_) | Type::GenericInstance(_) | Type::Void))
                                .unwrap_or(false)
                                || self.json_value_vars.contains(name.as_str())
                        }
                        _ => false,
                    };
                    if use_value_helper {
                        write!(out, "a2r_std::value_to_int(&")?;
                        self.expr(object, out)?;
                        write!(out, ")")?;
                    } else {
                        self.expr(object, out)?;
                        write!(out, ".parse::<i32>().ok()")?;
                    }
                    return Ok(());
                }
                "len" | "length" => {
                    // Skip if object is a known stdlib module — handled by Expr::Ident block below
                    let is_stdlib_module = if let Expr::Ident(name) = object.as_ref() {
                        matches!(name.as_str(), "json" | "shell" | "fs" | "regex" | "env" | "http")
                    } else { false };

                    if !is_stdlib_module {
                        // Check if object is json.get() result or known non-string type variable
                        let use_value_helper = match object.as_ref() {
                            Expr::Call(c) => {
                                if let Expr::Dot(obj, method) = c.name.as_ref() {
                                    if let Expr::Ident(name) = obj.as_ref() {
                                        name == "json" && (method == "get" || method == "get_at")
                                    } else { false }
                                } else { false }
                            }
                            Expr::Ident(name) => {
                                self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::User(_) | Type::Enum(_) | Type::Tag(_) | Type::GenericInstance(_) | Type::Void))
                                    .unwrap_or(false)
                                    || self.json_value_vars.contains(name.as_str())
                            }
                            _ => false,
                        };
                        if use_value_helper {
                            write!(out, "a2r_std::value_len(&")?;
                            self.expr(object, out)?;
                            write!(out, ")")?;
                            return Ok(());
                        }
                    }
                    // Fall through to remap table for normal len()
                }
                "match_count" => {
                    // s.match_count(pattern) -> a2r_std::str::match_count(s, pattern)
                    write!(out, "a2r_std::str::match_count(")?;
                    self.expr(object, out)?;
                    for arg in &call.args.args {
                        write!(out, ", ")?;
                        self.arg(arg, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "replace_first" => {
                    // s.replace_first(from, to) -> a2r_std::str::replace_first(s, from, to)
                    write!(out, "a2r_std::str::replace_first(")?;
                    self.expr(object, out)?;
                    for arg in &call.args.args {
                        write!(out, ", ")?;
                        self.arg(arg, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "substr" => {
                    // s.substr(start, length) -> a2r_std::str_substr(&s, start, length)
                    write!(out, "a2r_std::str_substr(")?;
                    self.expr_as_str(object, out)?;
                    for arg in &call.args.args {
                        write!(out, ", ")?;
                        self.arg(arg, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "contains" => {
                    // Only intercept for string types; map.contains() falls through to method remap
                    let obj_is_string = if let Expr::Ident(name) = object.as_ref() {
                        self.local_var_types.get(name)
                            .map(|ty| matches!(ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                            .unwrap_or(false)
                    } else {
                        matches!(object.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                    };
                    if obj_is_string {
                        // s.contains(needle) -> a2r_std::str_contains(&s, &needle)
                        write!(out, "a2r_std::str_contains(")?;
                        self.expr_as_str(object, out)?;
                        for arg in &call.args.args {
                            write!(out, ", ")?;
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // For non-string types (e.g., Map), fall through to method remap
                }
                "ends_with" => {
                    // s.ends_with(suffix) -> a2r_std::str_ends_with(&s, &suffix) returns i32
                    write!(out, "a2r_std::str_ends_with(")?;
                    self.expr_as_str(object, out)?;
                    for arg in &call.args.args {
                        write!(out, ", ")?;
                        if let Arg::Pos(expr) = arg {
                            self.expr_as_str(expr, out)?;
                        } else {
                            self.arg(arg, out)?;
                        }
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "get_or" => {
                    // Check if object is 'env' — env.get_or("KEY", default) -> a2r_std::env::get_or("KEY", default)
                    if let Expr::Ident(type_name) = object.as_ref() {
                        if type_name == "env" {
                            write!(out, "a2r_std::env::get_or(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 { write!(out, ", ")?; }
                                self.arg(arg, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                    }
                    // map.get_or(key, default)
                    // For string maps: .get(key).map(|s| s.as_str()).unwrap_or(default)
                    // For non-string maps: .get(key).cloned().unwrap_or(default)
                    let is_string_default = call.args.args.get(1)
                        .map(|a| if let Arg::Pos(e) = a {
                            matches!(e, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                        } else { true })
                        .unwrap_or(true);
                    self.expr(object, out)?;
                    write!(out, ".get(")?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        self.expr(a, out)?;
                    }
                    if is_string_default {
                        write!(out, ").map(|s| s.to_string()).unwrap_or_default()")?;
                    } else {
                        write!(out, ").cloned().unwrap_or(")?;
                        if call.args.args.len() > 1 {
                            if let Arg::Pos(a) = &call.args.args[1] {
                                self.expr(a, out)?;
                            }
                        }
                        write!(out, ")")?;
                    }
                    return Ok(());
                }
                "to_hex" => {
                    // val.to_hex(width) -> format!("{:0>width$x}", val, width = width)
                    write!(out, "format!(\"{{:0>width$x}}\", ")?;
                    self.expr(object, out)?;
                    write!(out, ", width = ")?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        self.expr(a, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "find" => {
                    // s.find(needle, start_pos?) -> a2r_std::str_find(&s, &needle, start_pos)
                    // Auto's .find() is only for strings; always intercept.
                    write!(out, "a2r_std::str_find(")?;
                    self.expr_as_str(object, out)?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        write!(out, ", ")?;
                        if i == 0 {
                            // needle: string arg needs .as_str()
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        } else {
                            // start_pos: i32, no conversion
                            self.arg(arg, out)?;
                        }
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "set" => {
                    // Map.set(key, val) -> HashMap::insert(key, val)
                    // Auto's .set() is only used on Map types, always translate to .insert()
                    self.expr(object, out)?;
                    write!(out, ".insert(")?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if i > 0 { write!(out, ", ")?; }
                        self.arg(arg, out)?;
                        // Auto-borrow: key might be &str, but HashMap<String,V> needs String
                        if i == 0 {
                            if let Arg::Pos(Expr::Ident(name)) = arg {
                                let is_str = self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false);
                                if is_str {
                                    write!(out, ".to_string()")?;
                                }
                            }
                        }
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                _ => {} // fall through to regular method handling
            }

            // Check for type name / stdlib module BEFORE the remap table.
            // This ensures json.len(), shell.exec(), etc. are intercepted
            // instead of falling into the simple name-remap (which would generate
            // e.g. `json.len()` as a method call on the `json` module).
            if let Expr::Ident(type_name) = object.as_ref() {
                match (type_name.as_str(), method_name.as_str()) {
                    ("json", "parse") => {
                        write!(out, "a2r_std::json::parse(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr_as_str(a, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "get") => {
                        write!(out, "a2r_std::json::get(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ", ")?;
                        if call.args.args.len() > 1 {
                            if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "get_str") => {
                        write!(out, "a2r_std::json::get_str(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ", ")?;
                        if call.args.args.len() > 1 {
                            if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "as_string") => {
                        write!(out, "a2r_std::json::as_string(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "to_string") => {
                        write!(out, "a2r_std::json::to_string(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "get_at") => {
                        write!(out, "a2r_std::json::get_at(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ", ")?;
                        if call.args.args.len() > 1 {
                            if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; write!(out, " as usize")?; }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "get_u64") => {
                        write!(out, "a2r_std::json::get_u64(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ", ")?;
                        if call.args.args.len() > 1 {
                            if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "keys") => {
                        write!(out, "a2r_std::json::keys(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("json", "len") => {
                        // Auto's json.len() returns int, but Rust returns usize — cast to i32
                        if let Some(Arg::Pos(expr)) = call.args.args.first() {
                            let is_str_type = if let Expr::Ident(name) = expr {
                                self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                                    .unwrap_or(true)
                            } else {
                                matches!(expr, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                            };
                            if is_str_type {
                                write!(out, "(a2r_std::json::len_str(")?;
                                self.expr_as_str(expr, out)?;
                                write!(out, ") as i32)")?;
                            } else {
                                write!(out, "(a2r_std::json::len(")?;
                                self.expr(expr, out)?;
                                write!(out, ") as i32)")?;
                            }
                        }
                        return Ok(());
                    }
                    ("json", "has_key") => {
                        // Auto's json.has_key() returns int (0 or 1), but Rust's returns bool.
                        // Wrap in if/else to convert bool -> i32.
                        if let Some(Arg::Pos(first)) = call.args.args.first() {
                            let use_str = if let Expr::Ident(name) = first {
                                self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_)))
                                    .unwrap_or(true)
                            } else {
                                matches!(first, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                            };
                            write!(out, "if ")?;
                            if use_str {
                                write!(out, "a2r_std::json::has_key_str(")?;
                                self.expr_as_str(first, out)?;
                            } else {
                                write!(out, "a2r_std::json::has_key(&")?;
                                self.expr(first, out)?;
                            }
                            write!(out, ", ")?;
                            if call.args.args.len() > 1 {
                                if let Arg::Pos(a) = &call.args.args[1] { self.expr(a, out)?; }
                            }
                            if !use_str { write!(out, ")")?; }
                            write!(out, ") {{ 1 }} else {{ 0 }}")?;
                        }
                        return Ok(());
                    }
                    ("shell", "exec") => {
                        write!(out, "a2r_std::shell::exec(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                if !matches!(expr, Expr::Int(_) | Expr::Float(_, _)) {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("regex", "match") => {
                        write!(out, "a2r_std::regex::r#match(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("regex", "find_all") => {
                        write!(out, "a2r_std::regex::find_all(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "exists") => {
                        write!(out, "a2r_std::fs::exists(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr(a, out)?;
                            if let Expr::Ident(name) = a {
                                let needs_as_str = self.local_var_types.get(name)
                                    .map(|ty| !matches!(ty, Type::StrSlice))
                                    .unwrap_or(false);
                                if needs_as_str && !matches!(a, Expr::Str(_) | Expr::CStr(_)) {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "create_dir") => {
                        write!(out, "a2r_std::fs::create_dir(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr_as_str(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "write_text") => {
                        write!(out, "a2r_std::fs::write_text(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "is_dir") => {
                        write!(out, "a2r_std::fs::is_dir(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr_as_str(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "is_binary") => {
                        write!(out, "a2r_std::fs::is_binary(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr_as_str(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "walk") => {
                        write!(out, "a2r_std::fs::walk(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr_as_str(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "read_to_string") | ("fs", "read_text") => {
                        let fn_name = method_name;
                        write!(out, "a2r_std::fs::{}(", fn_name)?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            let needs_as_str = if let Expr::Ident(name) = a {
                                !self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false)
                            } else {
                                !matches!(a, Expr::Str(_) | Expr::CStr(_))
                            };
                            self.expr(a, out)?;
                            if needs_as_str { write!(out, ".as_str()")?; }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("fs", "write") => {
                        write!(out, "a2r_std::fs::write(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("env", "get") => {
                        write!(out, "a2r_std::env::get(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() { self.expr(a, out)?; }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("io", "read_line") => {
                        write!(out, "a2r_std::io::read_line()")?;
                        return Ok(());
                    }
                    ("env", "args") => {
                        write!(out, "a2r_std::env::args()")?;
                        return Ok(());
                    }
                    ("env", "get_or") => {
                        write!(out, "a2r_std::env::get_or(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 { write!(out, ", ")?; }
                            self.arg(arg, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("Map", "new") => {
                        write!(out, "std::collections::HashMap::new()")?;
                        return Ok(());
                    }
                    _ => {} // fall through to remap table
                }
            }

            // .len() and .length() return usize in Rust, cast to i32 for Auto's int
            let needs_i32_cast = matches!(method_name.as_str(), "len" | "length");

            // For "contains", choose between str::contains and map::contains_key
            // String .contains() is already handled by the early interceptor above.
            // For Expr::Ident objects: use contains_key for known Map types, contains otherwise
            // For Expr::Dot objects (e.g., self.field): default to contains_key since
            // Auto's .contains() on maps maps to HashMap::contains_key
            let contains_rust = if method_name.as_str() == "contains" {
                match object.as_ref() {
                    Expr::Ident(name) => {
                        let obj_is_map = self.local_var_types.get(name)
                            .map(|ty| matches!(ty, Type::Map(_, _)))
                            .unwrap_or(false);
                        if obj_is_map { Some("contains_key") } else { Some("contains") }
                    }
                    Expr::Dot(_, _) => Some("contains_key"), // self.field on a Map
                    _ => Some("contains"),
                }
            } else { None };

            let rust_method = match method_name.as_str() {
                // String methods
                "to_lower" => Some("to_lowercase"),
                "to_upper" => Some("to_uppercase"),
                "length" | "len" => Some("len"),
                "is_empty" => Some("is_empty"),
                "trim" => Some("trim"),
                "trim_left" => Some("trim_start"),
                "trim_right" => Some("trim_end"),
                "starts_with" => Some("starts_with"),
                "ends_with" => Some("ends_with"),
                "append" => Some("push_str"),
                // Collection methods
                "push" => Some("push"),
                "pop" => Some("pop"),
                "clear" => Some("clear"),
                "to_array" => Some("clone"),
                "retain" => Some("retain"),
                // Type conversion
                "to_string" => Some("to_string"),
                _ => contains_rust,
            };

            if let Some(rust_name) = rust_method {
                let obj_parens = matches!(object.as_ref(),
                    Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
                );
                if needs_i32_cast { write!(out, "(")?; }
                if obj_parens { write!(out, "(")?; }
                self.expr(object, out)?;
                if obj_parens { write!(out, ")")?; }
                write!(out, ".{}(", rust_name)?;
                // Auto-borrow string args for pattern-matching methods
                // Only add & for actual string methods, not contains_key (which takes &str directly)
                if matches!(rust_name, "contains" | "starts_with" | "ends_with") {
                    for (i, arg) in call.args.args.iter().enumerate() {
                        write!(out, "&")?;
                        self.arg(arg, out)?;
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                } else {
                    for (i, arg) in call.args.args.iter().enumerate() {
                        self.arg(arg, out)?;
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                }
                write!(out, ")")?;
                if needs_i32_cast {
                    write!(out, " as i32)")?;
                }
                // trim/trim_start/trim_end return &str, auto-convert to String
                if matches!(method_name.as_str(), "trim" | "trim_left" | "trim_right") {
                    write!(out, ".to_string()")?;
                }
                return Ok(());
            }

            // Check for type name static method: Type.method(args) -> Type::method(args)
            // (stdlib modules already handled by the early check above)
            if let Expr::Ident(type_name) = object.as_ref() {
                let is_type = type_name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                    || matches!(type_name.as_str(),
                        "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                        | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                        | "f32" | "f64" | "bool" | "char" | "str"
                    )
                    || Self::auto_type_to_rust(type_name.as_str()).is_some();
                if is_type {
                    // Map Auto builtin type names to Rust equivalents
                    let rust_type_name = Self::auto_type_to_rust(type_name.as_str())
                        .unwrap_or_else(|| type_name.as_str());
                    // If the type name is not directly in self.uses, try to qualify it
                    // with an imported crate prefix (e.g., Normal -> rand_distr::Normal)
                    let type_in_uses = self.uses.iter().any(|u| {
                        let u_str = u.as_str();
                        u_str == type_name.as_str()
                            || u_str.ends_with(&format!("::{}", type_name.as_str()))
                            // Check brace-expansion: "chrono::{Utc, Duration}" contains "Utc"
                            || u_str.contains(&format!("{{{}}}", type_name.as_str()))
                            || u_str.contains(&format!("{}, ", type_name.as_str()))
                            || u_str.contains(&format!(", {}", type_name.as_str()))
                    });
                    let qualified_type = if type_in_uses {
                        // Type name found in uses (possibly via brace expansion) — use as-is
                        rust_type_name.to_string()
                    } else if !self.uses.contains(type_name.as_str()) {
                        // Type not in uses at all — qualify with the best matching
                        // external crate. Prefer the most specific (longest named) crate.
                        let source_crate = self.uses.iter()
                            .filter(|u| {
                                let u_str = u.as_str();
                                !u_str.contains("::") && u_str != "a2r_std"
                                    && !u_str.starts_with("std")
                                    && !u_str.starts_with("auto_lang")
                                    && !Self::auto_type_to_rust(u_str).is_some()
                                    && !self.glob_imported_modules.contains(u_str)
                            })
                            .max_by_key(|u| u.as_str().len())
                            .map(|u| u.as_str())
                            .unwrap_or("");
                        if !source_crate.is_empty() {
                            format!("{}::{}", source_crate, rust_type_name)
                        } else {
                            rust_type_name.to_string()
                        }
                    } else {
                        rust_type_name.to_string()
                    };
                    // Check for tag construction: Type.Variant(args)
                    if self.tag_types.contains(type_name) {
                        let key = (type_name.clone(), method_name.clone());
                        let struct_fields = self.enum_struct_variants.get(&key).cloned();
                        write!(out, "{}::{}", qualified_type, method_name)?;
                        if let Some(fields) = struct_fields {
                            // Struct variant: Type::Variant { field1: val1, field2: val2 }
                            write!(out, " {{ ")?;
                            for (i, (arg, field_name)) in call.args.args.iter().zip(fields.iter()).enumerate() {
                                write!(out, "{}: ", field_name)?;
                                match arg {
                                    Arg::Pos(expr) => {
                                        self.expr(expr, out)?;
                                        if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                    Arg::Pair(_, expr) => {
                                        self.expr(expr, out)?;
                                        if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                    Arg::Name(name) => {
                                        write!(out, "{}", name)?;
                                    }
                                }
                                if i < call.args.args.len().min(fields.len()) - 1 { write!(out, ", ")?; }
                            }
                            write!(out, " }}")?;
                        } else {
                            // Tuple variant: Type::Variant(val1, val2, ...)
                            let tuple_field_types = self.enum_tuple_field_types.get(&key).cloned();
                            write!(out, "(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if let Arg::Pos(expr) = arg {
                                    self.expr(expr, out)?;
                                    if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                        write!(out, ".to_string()")?;
                                    } else if let Expr::Ident(name) = expr {
                                        let field_is_string = tuple_field_types.as_ref()
                                            .and_then(|types| types.get(i))
                                            .map(|ty| matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice))
                                            .unwrap_or(false);
                                        let var_is_str_slice = self.local_var_types.get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice))
                                            .unwrap_or(false);
                                        if field_is_string && var_is_str_slice {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                }
                                if i < call.args.args.len() - 1 { write!(out, ", ")?; }
                            }
                            write!(out, ")")?;
                        }
                        return Ok(());
                    }
                    // Static method: Type::method(args)
                    write!(out, "{}::{}", qualified_type, method_name)?;
                    write!(out, "(")?;
                    // Prefer qualified key "Type.method" for accurate lookup
                    let qualified_key: AutoStr = format!("{}.{}", type_name, method_name).into();
                    let static_str_flags = self.fn_str_param_indices.get(&qualified_key)
                        .cloned()
                        .or_else(|| self.fn_str_param_indices.get(method_name.as_str()).cloned());
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if let Arg::Pos(expr) = arg {
                            self.expr(expr, out)?;
                            // Auto-borrow for str params
                            let is_str_param = static_str_flags.as_ref()
                                .and_then(|f| f.get(i))
                                .copied()
                                .unwrap_or(false);
                            if is_str_param && !matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                let is_str_slice_var = if let Expr::Ident(name) = expr {
                                    self.local_var_types.get(name)
                                        .map(|ty| matches!(ty, Type::StrSlice))
                                        .unwrap_or(false)
                                } else {
                                    false
                                };
                                if !is_str_slice_var {
                                    write!(out, ".as_str()")?;
                                }
                            }
                            // Auto-borrow for external crate type static methods
                            if !is_str_param {
                                if let Expr::Ident(name) = expr {
                                    if self.local_var_types.get(name)
                                        .map(|ty| matches!(ty, Type::StrOwned | Type::StrFixed(_)))
                                        .unwrap_or(false)
                                    {
                                        write!(out, ".as_str()")?;
                                    }
                                }
                            }
                        } else {
                            self.arg(arg, out)?;
                        }
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
            }

            // Check if object is a type-like chain (module.Type or crate) — use :: for static calls
            // Only use :: when the leftmost identifier is a known use.rust module or crate,
            // not when it could be a local variable (e.g., closure param like `a.age.cmp()`)
            let obj_is_type_chain = match object.as_ref() {
                Expr::Ident(id) => {
                    let name = id.as_str();
                    Self::auto_type_to_rust(name).is_some()
                        || self.uses.iter().any(|u| {
                            let u_str = u.as_str();
                            u_str == name || u_str.ends_with(&format!("::{}", name))
                        })
                        || self.dep_crates.contains(id)
                }
                Expr::Dot(il, _) => {
                    matches!(il.as_ref(), Expr::Ident(id) if {
                        let name = id.as_str();
                        name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                            || self.uses.iter().any(|u| {
                                let u_str = u.as_str();
                                u_str == name || u_str.ends_with(&format!("::{}", name))
                            })
                    })
                }
                Expr::Bina(il, Op::Dot, _) => {
                    matches!(il.as_ref(), Expr::Ident(id) if {
                        let name = id.as_str();
                        name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                            || self.uses.iter().any(|u| {
                                let u_str = u.as_str();
                                u_str == name || u_str.ends_with(&format!("::{}", name))
                            })
                    })
                }
                _ => false,
            };

            // Regular method call: object.method(args)
            let is_insert = method_name.as_str() == "insert";
            // Look up str-param flags for auto-borrow at method call sites
            let method_str_flags = self.fn_str_param_indices.get(method_name.as_str()).cloned();
            // Parenthesize object if it's a binary op (e.g., (a / b).method())
            let obj_needs_parens = matches!(object.as_ref(),
                Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
            );
            if obj_needs_parens { write!(out, "(")?; }
            self.expr(object, out)?;
            if obj_needs_parens { write!(out, ")")?; }
            write!(out, "{}{}(", if obj_is_type_chain { "::" } else { "." }, method_name)?;
            for (i, arg) in call.args.args.iter().enumerate() {
                match arg {
                    Arg::Pos(expr) => {
                        self.expr(expr, out)?;
                        // For Map.insert(), auto-convert to String for non-primitive types
                        if is_insert && !matches!(expr, Expr::Int(_) | Expr::Bool(_)) {
                            write!(out, ".to_string()")?;
                        }
                        // Auto-borrow: add .as_str() when passing String to &str method param
                        let is_str_param = method_str_flags.as_ref()
                            .and_then(|f| f.get(i))
                            .copied()
                            .unwrap_or(false);
                        // Also try i+1 for methods that include self as first param
                        let is_str_param_offset = method_str_flags.as_ref()
                            .and_then(|f| f.get(i + 1))
                            .copied()
                            .unwrap_or(false);
                        if (is_str_param || is_str_param_offset)
                            && !matches!(expr, Expr::Str(_) | Expr::CStr(_))
                            && !Self::is_str_slice_var(arg, &self.local_var_types)
                        {
                            write!(out, ".as_str()")?;
                        }
                        // Auto-borrow for external crate calls: when calling crate::method()
                        // with a String-typed variable, add .as_str() since most Rust
                        // APIs accept &str rather than String.
                        if obj_is_type_chain && !is_str_param && !is_str_param_offset {
                            if let Expr::Ident(name) = expr {
                                if self.local_var_types.get(name)
                                    .map(|ty| matches!(ty, Type::StrOwned | Type::StrFixed(_)))
                                    .unwrap_or(false)
                                {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        }
                    }
                    other => self.arg(other, out)?,
                }
                if i < call.args.args.len() - 1 { write!(out, ", ")?; }
            }
            write!(out, ")")?;
            // Don't unconditionally append .cloned() on .get() calls —
            // external crate .get() methods (e.g., csv::Record::get) return
            // Option<&str> which doesn't support .cloned() in the same way.
            return Ok(());
        }

        // **Phase 1.3: Tag Types**
        // Check if this is a tag construction call: Tag.Variant(value)
        // E.g., Atom.Int(11) should generate: Atom::Int(11)
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if let Expr::Ident(type_name) = lhs.as_ref() {
                    if let Expr::Ident(variant_name) = rhs.as_ref() {
                        if self.tag_types.contains(type_name) {
                            let key = (type_name.clone(), variant_name.clone());
                            let struct_fields = self.enum_struct_variants.get(&key).cloned();
                            // Tag construction: TypeName::VariantName(args)
                            write!(out, "{}::{}", type_name, variant_name)?;
                            if let Some(fields) = struct_fields {
                                // Struct variant: Type::Variant { field1: val1, field2: val2 }
                                write!(out, " {{ ")?;
                                for (i, (arg, field_name)) in call.args.args.iter().zip(fields.iter()).enumerate() {
                                    if let Arg::Pos(expr) = arg {
                                        write!(out, "{}: ", field_name)?;
                                        self.expr(expr, out)?;
                                        if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                    if i < call.args.args.len().min(fields.len()) - 1 { write!(out, ", ")?; }
                                }
                                write!(out, " }}")?;
                            } else {
                                // Tuple variant: Type::Variant(val1, val2, ...)
                                let tuple_field_types = self.enum_tuple_field_types.get(&key).cloned();
                                write!(out, "(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                            write!(out, ".to_string()")?;
                                        } else if let Expr::Ident(name) = expr {
                                            // Check if tuple field is String but arg is &str
                                            let field_is_string = tuple_field_types.as_ref()
                                                .and_then(|types| types.get(i))
                                                .map(|ty| matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice))
                                                .unwrap_or(false);
                                            let var_is_str_slice = self.local_var_types.get(name)
                                                .map(|ty| matches!(ty, Type::StrSlice))
                                                .unwrap_or(false);
                                            if field_is_string && var_is_str_slice {
                                                write!(out, ".to_string()")?;
                                            }
                                        }
                                    }
                                    if i < call.args.args.len() - 1 { write!(out, ", ")?; }
                                }
                                write!(out, ")")?;
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Check if this is a struct construction call: Type(args)
        // Heuristic: If the callee name starts with uppercase, treat as type construction
        // This works because Rust convention: TypeNames are CamelCase, functions are snake_case
        if let Expr::Ident(type_name) = call.name.as_ref() {
            let is_type = type_name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);
            if is_type {
                // This is a struct construction: Type { field1: value1, ... }
                return self.struct_init(type_name, &call.args, out);
            }
        }

        // Plan 204 Phase 5: Auto stdlib free function -> Rust equivalents
        if let Expr::Ident(fn_name) = call.name.as_ref() {
            match fn_name.as_str() {
                "min" => {
                    // min(a, b) -> a2r_std::math::min(a, b)
                    write!(out, "a2r_std::math::min(")?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        self.arg(arg, out)?;
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "max" => {
                    // max(a, b) -> a2r_std::math::max(a, b)
                    write!(out, "a2r_std::math::max(")?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        self.arg(arg, out)?;
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                _ => {}
            }
        }

        // Normal function call
        self.expr(&call.name, out)?;
        write!(out, "(")?;

        // Look up str-param flags for auto-borrow at call sites
        let str_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_str_param_indices.get(fn_name).cloned()
        } else {
            None
        };

        // Look up struct-param flags for auto-clone at call sites
        let struct_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_struct_param_indices.get(fn_name).cloned()
        } else {
            None
        };

        // Look up spec-param flags for auto-boxing at call sites
        let spec_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_spec_param_indices.get(fn_name).cloned()
        } else {
            None
        };

        for (i, arg) in call.args.args.iter().enumerate() {
            let is_str_param = str_flags.as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            let needs_borrow = is_str_param && !Self::is_string_literal_arg(arg)
                && !Self::is_str_slice_var(arg, &self.local_var_types);

            // Auto-clone when passing a variable to a function that takes a struct param
            let is_struct_param = struct_flags.as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            let needs_clone = is_struct_param && matches!(arg, Arg::Pos(Expr::Ident(_)));

            // Auto-box when passing a value to a function that takes a spec param
            let is_spec_param = spec_flags.as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);

            if is_spec_param {
                write!(out, "Box::new(")?;
            }

            self.arg(arg, out)?;
            if needs_clone {
                write!(out, ".clone()")?;
            }

            // After expression: add .as_str() for String→&str conversion
            if needs_borrow {
                write!(out, ".as_str()")?;
            }

            if is_spec_param {
                write!(out, ".clone())")?;
            }

            if i < call.args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, ")").map_err(Into::into)
    }

    /// Check if an arg is a string literal ("...") — doesn't need & at call site
    fn is_string_literal_arg(arg: &Arg) -> bool {
        if let Arg::Pos(expr) = arg {
            matches!(expr, Expr::Str(_) | Expr::CStr(_))
        } else {
            false
        }
    }

    /// Check if an arg is a &str variable — already borrowed, no .as_str() needed
    fn is_str_slice_var(arg: &Arg, local_var_types: &HashMap<AutoStr, Type>) -> bool {
        if let Arg::Pos(Expr::Ident(name)) = arg {
            local_var_types.get(name)
                .map(|ty| matches!(ty, Type::StrSlice))
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Infer Rust type from an Auto expression (for let-bound variables without type annotation).
    fn infer_type_from_expr(expr: &Expr) -> Type {
        match expr {
            Expr::Call(call) => {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    // method is AutoStr, obj is Expr
                    match method.as_str() {
                        // Methods that return String
                        "substr" | "sub" | "slice" | "to_lower" | "to_upper"
                        | "trim" | "trim_left" | "trim_right" | "to_string"
                        | "replace" | "replace_first" | "repeat" | "char_at" => {
                            return Type::StrOwned;
                        }
                        // stdlib module functions that return String
                        _ => {
                            if let Expr::Ident(module) = obj.as_ref() {
                                match (module.as_str(), method.as_str()) {
                                    ("json", "as_string") | ("json", "get_str")
                                    | ("json", "to_string") | ("json", "keys")
                                    | ("fs", "read_text") | ("fs", "read_to_string")
                                    | ("fs", "walk") | ("shell", "exec")
                                    | ("regex", "find_all")
                                    | ("io", "read_line") | ("env", "args") | ("env", "get") => {
                                        return Type::StrOwned;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                // Check for known function calls that return String
                if let Expr::Ident(fn_name) = call.name.as_ref() {
                    match fn_name.as_str() {
                        "json_escape" | "json_to_string" | "format" => return Type::StrOwned,
                        _ => {}
                    }
                }
                Type::Unknown
            }
            Expr::Array(items) => {
                // Array literal — infer as List if items exist
                if let Some(first) = items.first() {
                    let elem_ty = Self::infer_type_from_expr(first);
                    Type::List(Box::new(elem_ty))
                } else {
                    Type::Unknown
                }
            }
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => Type::StrSlice,
            Expr::Int(_) => Type::Int,
            Expr::Float(_, _) => Type::Float,
            Expr::Bool(_) => Type::Bool,
            _ => Type::Unknown,
        }
    }

    /// Check if an expression likely needs .as_str() to convert String → &str.
    /// Returns true for Expr::Ident variables that may be String at runtime.
    /// Note: Auto's `str` type maps to StrSlice but generated code often uses String
    /// (due to .to_string() on assignment), so we conservatively add .as_str() for all
    /// variables except those that are clearly function params with &str type.
    fn needs_as_str(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                // Function parameters typed as &str are safe (caller passes &str).
                // All other variables may be String at runtime.
                !self.fn_param_str_slice.contains(name.as_str())
            }
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => false, // literals are already &str
            _ => true, // complex expressions (function calls, etc.) may return String
        }
    }

    /// Emit an expression with .as_str() appended if needed for &str parameter.
    fn expr_as_str(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        self.expr(expr, out)?;
        if self.needs_as_str(expr) {
            write!(out, ".as_str()")?;
        }
        Ok(())
    }

    fn struct_init(
        &mut self,
        type_name: &AutoStr,
        args: &Args,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        // Generate struct initialization: Type { field1: value1, field2: value2 }
        if args.args.is_empty() {
            // Empty struct: Type {}
            write!(out, "{} {{}}", type_name)?;
            return Ok(());
        }

        write!(out, "{} {{ ", type_name)?;

        // Get cached field names for this type
        let field_names = self
            .struct_fields
            .get(type_name)
            .cloned()
            .unwrap_or_default();

        // Get cached field types for .to_string() auto-insertion
        let field_types = self
            .struct_field_types
            .get(type_name)
            .cloned()
            .unwrap_or_default();

        for (i, arg) in args.args.iter().enumerate() {
            let (field_name, needs_to_string) = match arg {
                Arg::Pos(expr) => {
                    let name = if i < field_names.len() {
                        field_names[i].clone()
                    } else {
                        format!("field{}", i).into()
                    };
                    // Check if field type is String but expr is &str
                    let needs_ts = i < field_types.len()
                        && matches!(field_types[i].1, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice)
                        && !matches!(expr, Expr::Str(_) | Expr::CStr(_));
                    (name, needs_ts)
                }
                Arg::Name(name) => (name.clone(), false),
                Arg::Pair(key, expr) => {
                    let needs_ts = field_types.iter()
                        .find(|(n, _)| n == key)
                        .map(|(_, ty)| matches!(ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice))
                        .unwrap_or(false)
                        && !matches!(expr, Expr::Str(_) | Expr::CStr(_));
                    (key.clone(), needs_ts)
                }
            };
            write!(out, "{}: ", field_name)?;
            match arg {
                Arg::Pos(expr) | Arg::Pair(_, expr) => {
                    self.write_expr_for_struct_field(expr, out)?;
                }
                Arg::Name(_) => {}
            }
            if needs_to_string {
                write!(out, ".to_string()")?;
            }
            if i < args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, " }}").map_err(Into::into)
    }

    fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Pos(expr) => {
                self.expr(expr, out)?;
                if Self::is_self_dot(expr) {
                    write!(out, ".clone()")?;
                }
                Ok(())
            }
            Arg::Name(name) => write!(out, "{}", name).map_err(Into::into),
            Arg::Pair(_, expr) => {
                self.expr(expr, out)?;
                if Self::is_self_dot(expr) {
                    write!(out, ".clone()")?;
                }
                Ok(())
            }
        }
    }

    fn write_expr_for_struct_field(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        // Array literals in struct fields should be vec![] (fields are typically Vec<T>)
        if let Expr::Array(elems) = expr {
            write!(out, "vec![")?;
            for (i, elem) in elems.iter().enumerate() {
                self.expr(elem, out)?;
                if i < elems.len() - 1 {
                    write!(out, ", ")?;
                }
            }
            write!(out, "]")?;
        } else {
            self.expr(expr, out)?;
            // Auto str literals are &str but struct fields are String
            if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                write!(out, ".to_string()")?;
            }
        }
        // self.field in &self context needs .clone()
        if Self::is_self_dot(expr) {
            write!(out, ".clone()")?;
        }
        Ok(())
    }

    fn output_call(&mut self, call: &Call, out: &mut impl Write, newline: bool) -> AutoResult<()> {
        // print("hello") / write("hello") -> println!("hello") / print!("hello")
        // print(value) / write(value)   -> println!("{}", value) / print!("{}", value)
        // print(f"...") / write(f"...") -> println!("...", args) / print!("...", args)
        // print("text:", value) / write("text:", value) -> println!("text: {}", value) / print!("text: {}", value)

        let macro_name = if newline { "println" } else { "print" };

        if call.args.args.is_empty() {
            write!(out, "{}!()", macro_name)?;
            return Ok(());
        }

        // Check if first argument is an f-string
        if let Arg::Pos(first_arg) = &call.args.args[0] {
            if let Expr::FStr(fstr) = first_arg {
                write!(out, "{}!(\"", macro_name)?;

                // Build format string from f-string parts
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            write!(out, "{}", s.replace("\"", r##"\""##))?;
                        }
                        Expr::Char(c) => {
                            write!(out, "{}", c)?;
                        }
                        _ => {
                            // Expression placeholder — use {:?} for Duration-like exprs
                            if self.needs_debug_format(part) {
                                write!(out, "{{:?}}")?;
                            } else {
                                write!(out, "{{}}")?;
                            }
                        }
                    }
                }
                write!(out, "\"")?;

                // Add f-string arguments
                for part in &fstr.parts {
                    match part {
                        Expr::Str(_) | Expr::CStr(_) | Expr::Char(_) => {}
                        _ => {
                            write!(out, ", ")?;
                            self.expr(part, out)?;
                        }
                    }
                }

                // Add additional arguments (after f-string)
                for arg in call.args.args.iter().skip(1) {
                    write!(out, ", ")?;
                    self.arg(arg, out)?;
                }

                write!(out, ")")?;
                return Ok(());
            }
        }

        if call.args.args.len() == 1 {
            if let Arg::Pos(expr) = &call.args.args[0] {
                match expr {
                    Expr::Str(s) | Expr::CStr(s) => {
                        write!(out, "{}!(\"{}\")", macro_name, s)?;
                        return Ok(());
                    }
                    _ => {
                        // Single non-string argument: use {:?} for non-Display types
                        let fmt = if self.needs_debug_format(expr) { "{:?}" } else { "{}" };
                        write!(out, "{}!(\"{}\", ", macro_name, fmt)?;
                        self.expr(expr, out)?;
                        write!(out, ")")?;
                        return Ok(());
                    }
                }
            }
        }

        // Multiple arguments: check if first is a string literal
        if let Arg::Pos(first_arg) = &call.args.args[0] {
            if let Expr::Str(s) | Expr::CStr(s) = first_arg {
                // First arg is a string - use it as format prefix
                let mut format_string = s.replace("\"", r##"\""##);

                // Add placeholders for remaining args — use {:?} for non-Display types
                for arg in call.args.args.iter().skip(1) {
                    if let Arg::Pos(e) = arg {
                        format_string.push_str(if self.needs_debug_format(e) { " {:?}" } else { " {}" });
                    } else {
                        format_string.push_str(" {}");
                    }
                }

                write!(out, "{}!(\"{}\"", macro_name, format_string)?;

                // Add remaining args
                for arg in call.args.args.iter().skip(1) {
                    write!(out, ", ")?;
                    self.arg(arg, out)?;
                }
                write!(out, ")")?;
                return Ok(());
            }
        }

        // Fallback: generic format string with placeholders
        write!(out, "{}!(\"", macro_name)?;
        for (i, _arg) in call.args.args.iter().enumerate() {
            if i > 0 {
                write!(out, " ")?;
            }
            write!(out, "{{}}")?;
        }
        write!(out, "\"")?;
        for arg in &call.args.args {
            write!(out, ", ")?;
            self.arg(arg, out)?;
        }
        write!(out, ")").map_err(Into::into)
    }

    fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<bool> {
        match stmt {
            Stmt::Expr(expr) => {
                self.expr(expr, &mut sink.body)?;
                // No semicolon for expressions in expression position
                // (handled by body() method)
                Ok(true)
            }

            Stmt::Store(store) => {
                self.store(store, &mut sink.body)?;
                sink.body.write(b";")?;
                Ok(true)
            }

            Stmt::Fn(fn_decl) => {
                self.fn_decl(fn_decl, sink)?;
                Ok(true)
            }

            Stmt::For(for_stmt) => {
                self.for_stmt(for_stmt, sink)?;
                Ok(true)
            }

            Stmt::If(if_) => {
                self.if_stmt(if_, sink)?;
                Ok(true)
            }

            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, sink)?;
                Ok(true)
            }

            Stmt::Use(use_stmt) => {
                self.use_stmt(use_stmt, &mut sink.body)?;
                Ok(true)
            }

            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, sink)?;
                Ok(true)
            }

            Stmt::TypeAlias(type_alias) => {
                self.type_alias_decl(type_alias, sink)?;
                Ok(true)
            }

            Stmt::SpecDecl(spec_decl) => {
                self.spec_decl(spec_decl, sink)?;
                Ok(true)
            }

            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, sink)?;
                Ok(true)
            }

            Stmt::Union(union) => {
                self.union_decl(union, sink)?;
                Ok(true)
            }

            Stmt::Tag(tag) => {
                self.tag_decl(tag, sink)?;
                Ok(true)
            }

            Stmt::Ext(ext) => {
                self.ext_decl(ext, sink)?;
                Ok(true)
            }

            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    sink.body.write(b"\n")?;
                }
                Ok(true)
            }

            Stmt::Break => {
                sink.body.write(b"break;")?;
                Ok(true)
            }

            Stmt::Continue => {
                sink.body.write(b"continue;")?;
                Ok(true)
            }

            Stmt::Return(expr) => {
                sink.body.write(b"return ")?;
                // Plan 232: If returning a &str parameter, add .to_string()
                if let Expr::Ident(name) = expr.as_ref() {
                    if self.current_fn_str_params.contains(name) {
                        write!(sink.body, "{}.to_string()", name)?;
                        sink.body.write(b";")?;
                        return Ok(true);
                    }
                }
                // If return type is String and expr produces &str, add .to_string()
                let needs_to_string = self.ret_type_needs_string_coercion()
                    && self.expr_needs_string_coercion(expr);
                self.expr(expr, &mut sink.body)?;
                if needs_to_string {
                    sink.body.write(b".to_string()")?;
                }
                sink.body.write(b";")?;
                Ok(true)
            }

            // Plan 124 Phase 2.3: reply statement for ask/reply RPC
            // reply expr -> reply_tx.send(expr).unwrap()
            Stmt::Reply(expr) => {
                // In Rust, reply is implemented via oneshot channel send
                // The compiler should inject a `reply_tx` parameter into the message handler
                sink.body.write(b"let _ = reply_tx.send(")?;
                self.expr(expr, &mut sink.body)?;
                sink.body.write(b");")?;
                Ok(true)
            }

            Stmt::Node(node) => {
                // Handle loop and other control flow nodes
                self.expr(&Expr::Node(node.clone()), &mut sink.body)?;
                // Don't add semicolon after block-like nodes (loop)
                if node.name != "loop" {
                    sink.body.write(b";")?;
                }
                Ok(true)
            }

            // Plan 212 Phase 2.4: Macro invocation — #debug("msg") → debug!("msg")
            Stmt::MacroCall(macro_call) => {
                write!(sink.body, "{}!(", macro_call.name)?;
                for (i, arg) in macro_call.args.iter().enumerate() {
                    if i > 0 {
                        sink.body.write(b", ")?;
                    }
                    self.expr(arg, &mut sink.body)?;
                }
                sink.body.write(b");")?;
                Ok(true)
            }

            Stmt::Dep(dep) => {
                self.dep_crates.insert(dep.name.clone());
                Ok(true)
            }

            _ => Err(format!("Rust Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    // Variable declaration
    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // Track local variable type for string concat detection
        // When type is Unknown, try to infer from the expression
        let effective_ty = if matches!(store.ty, Type::Unknown) {
            Self::infer_type_from_expr(&store.expr)
        } else {
            store.ty.clone()
        };
        self.local_var_types.insert(store.name.clone(), effective_ty);

        // Detect json.get() assignments and mark the variable as JSON value type
        // so that .to_int() and .len() use value_to_int/value_len helpers
        if !matches!(store.ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_) | Type::CStrLit | Type::List(_) | Type::Int | Type::Float | Type::Bool) {
            if let Expr::Call(call) = &store.expr {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if let Expr::Ident(name) = obj.as_ref() {
                        if name == "json" && (method == "get" || method == "get_at") {
                            self.json_value_vars.insert(store.name.clone());
                        }
                    }
                }
            }
        }

        // Track variable→spec mapping: when expr is a ctor with a spec type,
        // record var_name -> spec_name for later spec array inference
        if let Some(type_name) = Self::extract_tag_or_ctor_type(&store.expr) {
            if let Some(spec_name) = self.struct_to_spec.get(&type_name) {
                self.var_spec_map.insert(store.name.clone(), spec_name.clone());
            }
        }

        // Handle C variables and struct fields (should not be generated)
        match store.kind {
            StoreKind::CVar | StoreKind::Field => {
                return Ok(());
            }
            _ => {}
        }

        // Plan 6B-4.19: shared var → static NAME: Lazy<Mutex<T>> = Lazy::new(|| Mutex::new(...));
        if matches!(store.kind, StoreKind::Shared) {
            let static_name = self.global_var_static_name(&store.name);
            let ty = self.rust_type_name(&store.ty);
            write!(out, "static {}: Lazy<Mutex<{}>> = Lazy::new(|| Mutex::new(",
                   static_name, ty)?;
            self.expr(&store.expr, out)?;
            write!(out, "))")?;
            return Ok(());
        }

        // Plan 6B-3.4: const declaration → const NAME: &str = "...";
        if matches!(store.kind, StoreKind::Const) {
            let ty_name = if matches!(store.ty, Type::StrFixed(_)) {
                "&str".to_string()
            } else {
                self.rust_type_name(&store.ty)
            };
            write!(out, "const {}: {} = ", store.name, ty_name)?;
            self.expr(&store.expr, out)?;
            return Ok(());
        }

        // Plan 151: Generate static Lazy<Mutex<T>> for global variables
        if self.is_global_var(&store.name) {
            let static_name = self.global_var_static_name(&store.name);
            let ty = self.rust_type_name(&store.ty);

            // Generate: static NAME: Lazy<Mutex<T>> = Lazy::new(|| Mutex::new(...));
            write!(out, "static {}: Lazy<Mutex<{}>> = Lazy::new(|| Mutex::new(",
                   static_name, ty)?;
            self.expr(&store.expr, out)?;
            write!(out, "));")?;
            return Ok(());
        }

        // Type inference for Unknown types
        // Plan 204 Phase 1E: Also skip type annotation when the rendered type
        // contains "/* unknown */" (e.g., Option</* unknown */>, [/* unknown */; N])
        let ty_name = self.rust_type_name(&store.ty);
        // Skip type annotation for: Unknown types, error propagation (?), closures, or unknown placeholders
        let is_error_propagate = matches!(&store.expr, Expr::ErrorPropagate(_));
        let has_unknown = matches!(store.ty, Type::Unknown) || ty_name.contains("/* unknown */") || is_error_propagate;

        // Check if the expression is a closure - closures should not have explicit type annotations
        // because Rust infers closure types automatically
        let is_closure = matches!(store.expr, Expr::Closure(_));

        // Check if expression is a borrow (&x or &mut x) - type should be a reference
        let is_borrow = matches!(&store.expr, Expr::View(_))
            || matches!(&store.expr, Expr::Dot(_, f) if f.as_str() == "view");
        let is_mut_borrow = matches!(&store.expr, Expr::Dot(_, f) if f.as_str() == "mut");

        let ty_name = if is_borrow && matches!(store.ty, Type::StrOwned | Type::StrFixed(_)) {
            "&str".to_string()
        } else if is_mut_borrow && matches!(store.ty, Type::StrOwned | Type::StrFixed(_)) {
            "&mut str".to_string()
        } else {
            ty_name
        };

        // Check if expression is an Array of spec instances (for unknown-type fallback)
        let spec_array_type: Option<String> = if has_unknown {
            if let Expr::Array(elems) = &store.expr {
                let spec_name = elems.iter().find_map(|e| {
                    if let Expr::Ident(name) = e {
                        self.var_spec_map.get(name).cloned()
                    } else { None }
                });
                spec_name.map(|sn| format!("Vec<Box<dyn {}>>", sn))
            } else { None }
        } else { None };

        // Skip type annotation if: Unknown type, type contains unknown, or closure expression
        // Exception: spec array expressions need explicit type annotation for dyn Trait
        let skip_type_annotation = (has_unknown || is_closure) && spec_array_type.is_none();

        if skip_type_annotation {
            // No type annotation - let Rust infer the type
            match store.kind {
                StoreKind::Let => {
                    write!(out, "let {} = ", store.name)?;
                }
                StoreKind::Var => {
                    write!(out, "let mut {} = ", store.name)?;
                }
                _ => {
                    write!(out, "let {} = ", store.name)?;
                }
            }
        } else {
            // Explicit type annotation for non-closure expressions
            let ty_str = spec_array_type.as_deref().unwrap_or(&ty_name);
            match store.kind {
                StoreKind::Let => {
                    write!(out, "let {}: {} = ", store.name, ty_str)?;
                }
                StoreKind::Var => {
                    write!(out, "let mut {}: {} = ", store.name, ty_str)?;
                }
                _ => {
                    write!(out, "let {}: {} = ", store.name, ty_str)?;
                }
            }
        }

        // Plan 159 6B-2.2: Wrap array elements in Box::new() for []Spec types
        let is_spec_slice = matches!(&store.ty, Type::Slice(slice) if matches!(&*slice.elem, Type::Spec(_)));
        if is_spec_slice {
            // [b1, b2] → vec![Box::new(b1), Box::new(b2)]
            if let Expr::Array(elems) = &store.expr {
                write!(out, "vec![")?;
                for (i, elem) in elems.iter().enumerate() {
                    write!(out, "Box::new(")?;
                    self.expr(elem, out)?;
                    write!(out, ")")?;
                    if i < elems.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]")?;
            } else {
                self.expr(&store.expr, out)?;
            }
        } else if matches!(&store.ty, Type::List(_)) {
            // List<T> (Vec<T>) with Array literal → vec![...]
            if let Expr::Array(elems) = &store.expr {
                write!(out, "vec![")?;
                for (i, elem) in elems.iter().enumerate() {
                    self.expr(elem, out)?;
                    if i < elems.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]")?;
            } else {
                self.expr(&store.expr, out)?;
            }
        } else if spec_array_type.is_some() {
            // Unknown-type Array with spec elements -> vec![Box::new(e.clone()), ...]
            if let Expr::Array(elems) = &store.expr {
                write!(out, "vec![")?;
                for (i, elem) in elems.iter().enumerate() {
                    write!(out, "Box::new(")?;
                    self.expr(elem, out)?;
                    write!(out, ".clone())")?;
                    if i < elems.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]")?;
            } else {
                self.expr(&store.expr, out)?;
            }
        } else {
            self.expr(&store.expr, out)?;
        }

        // When assigning a string literal to a String/Str type, add .to_string()
        // because Rust string literals are &str, but String type needs conversion
        if matches!(store.ty, Type::StrOwned | Type::StrFixed(_) | Type::StrSlice | Type::CStrLit) {
            if matches!(&store.expr, Expr::Str(_) | Expr::CStr(_)) {
                write!(out, ".to_string()")?;
            }
        }

        // self.field assignment in &self context needs .clone()
        if Self::is_self_dot(&store.expr) {
            write!(out, ".clone()")?;
        }

        Ok(())
    }

    // Function declaration
    fn fn_decl(&mut self, fn_decl: &Fn, sink: &mut Sink) -> AutoResult<()> {
        // Skip C/VM function declarations (implemented externally)
        if matches!(fn_decl.kind, FnKind::CFunction | FnKind::VmFunction) {
            return Ok(());
        }

        // Clear local var type cache for this function, register params
        self.local_var_types.clear();
        for param in &fn_decl.params {
            self.local_var_types.insert(param.name.clone(), param.ty.clone());
        }

        // Plan 204 Phase 3: Track whether current function returns !T or Result<T,E> (for Err boxing)
        self.current_fn_is_result = matches!(fn_decl.ret, Type::Result(_))
            || matches!(&fn_decl.ret, Type::GenericInstance(inst) if inst.base_name == "Result");

        // Infer concrete error type from Err() calls in function body
        self.current_fn_err_type = None;
        if self.current_fn_is_result {
            self.current_fn_err_type = self.infer_err_enum(&fn_decl.body.stmts);
        }

        // Emit doc comments
        if let Some(ref doc) = fn_decl.doc {
            let is_method = fn_decl.parent.is_some();
            for line in doc.split('\n') {
                if is_method {
                    self.print_indent(&mut sink.body)?;
                }
                write!(sink.body, "/// {}\n", line)?;
            }
        }

        // Check if this is a method (has parent)
        let is_method = fn_decl.parent.is_some();

        // Print indent for methods (inside impl block)
        if is_method {
            self.print_indent(&mut sink.body)?;
        }

        // Plan 163: #[tokio::main] for async main
        let is_main_with_await = !is_method
            && fn_decl.name.as_ref() == "main"
            && Self::has_await(&fn_decl.body.stmts);
        if is_main_with_await {
            if is_method {
                // already indented
            } else {
                self.print_indent(&mut sink.body)?;
            }
            write!(sink.body, "#[tokio::main]\n")?;
            if is_method {
                self.print_indent(&mut sink.body)?;
            }
        }

        // Plan 163: Output pub prefix
        // Methods in pub types inherit pub visibility even if fn_decl.is_pub is false
        if fn_decl.is_pub || self.inside_pub_type {
            write!(sink.body, "pub ")?;
        }

        // Function signature
        // Auto-detect async: functions returning ~T (Future/Handle) are async in Rust
        // Also detect async main (has .await in body)
        let is_async_fn = is_main_with_await
            || matches!(fn_decl.ret, Type::Handle { .. })
            || matches!(&fn_decl.ret, Type::GenericInstance(inst) if inst.base_name == "Future");
        if is_async_fn {
            write!(sink.body, "async ")?;
        }
        write!(sink.body, "fn {}", fn_decl.name)?;

        // Plan 166: Emit generic type parameters from #[with(T as Trait)]
        if !fn_decl.type_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, tp) in fn_decl.type_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                write!(sink.body, "{}", tp.name)?;
                if let Some(constraint) = &tp.constraint {
                    write!(sink.body, ": {}", self.rust_type_name(constraint))?;
                }
            }
            write!(sink.body, ">")?;
        }

        // Parameters
        write!(sink.body, "(")?;

        // Add &self as first parameter for methods (except constructors)
        let skip_first_self = is_method && !fn_decl.is_static && fn_decl.name.as_str() != "new"
            && fn_decl.params.first().map_or(false, |p| p.name.as_str() == "self");
        if is_method && !fn_decl.is_static && fn_decl.name.as_str() != "new" {
            // Plan 163: &mut self for mut methods
            if fn_decl.is_mut {
                write!(sink.body, "&mut self")?;
            } else {
                write!(sink.body, "&self")?;
            }
            // Skip the 'self' param if it was the receiver in Auto
            let params_to_emit: Vec<_> = if skip_first_self {
                fn_decl.params.iter().skip(1).collect()
            } else {
                fn_decl.params.iter().collect()
            };
            if !params_to_emit.is_empty() {
                write!(sink.body, ", ")?;
            }
            for (i, param) in params_to_emit.iter().enumerate() {
                write!(
                    sink.body,
                    "{}: {}",
                    param.name,
                    self.rust_param_type_name(&param.ty)
                )?;
                if i < params_to_emit.len() - 1 {
                    write!(sink.body, ", ")?;
                }
            }
        } else {
            for (i, param) in fn_decl.params.iter().enumerate() {
                write!(
                    sink.body,
                    "{}: {}",
                    param.name,
                    self.rust_param_type_name(&param.ty)
                )?;
                if i < fn_decl.params.len() - 1 {
                    write!(sink.body, ", ")?;
                }
            }
        }
        write!(sink.body, ")")?;

        // Cache which params are str (&str) type for auto-borrow at call sites
        let str_param_flags: Vec<bool> = fn_decl.params.iter()
            .map(|p| matches!(p.ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit))
            .collect();
        for param in &fn_decl.params {
            if matches!(param.ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit) {
                self.current_fn_str_params.insert(param.name.clone());
                self.fn_param_str_slice.insert(param.name.clone());
            }
        }
        self.fn_str_param_indices.insert(fn_decl.name.clone(), str_param_flags);

        // Cache which params are struct/enum types (need .clone() at call sites)
        let struct_param_flags: Vec<bool> = fn_decl.params.iter()
            .map(|p| matches!(p.ty, Type::User(_) | Type::Tag(_) | Type::Enum(_)))
            .collect();
        self.fn_struct_param_indices.insert(fn_decl.name.clone(), struct_param_flags);

        // Cache which params are spec types (need Box::new() at call sites)
        let spec_param_flags: Vec<bool> = fn_decl.params.iter()
            .map(|p| matches!(p.ty, Type::Spec(_)))
            .collect();
        self.fn_spec_param_indices.insert(fn_decl.name.clone(), spec_param_flags);

        // Plan 240: If function returns void but body uses .? (ErrorPropagate),
        // auto-wrap return type as Result<(), Box<dyn std::error::Error>>
        let fn_body_has_try = matches!(fn_decl.ret, Type::Void)
            && Self::has_error_propagate(&fn_decl.body.stmts);

        // Plan 232: Track str-type parameter names for .to_string() on return
        self.current_fn_str_params.clear();
        // Plan 240: When fn body has .? but declared as void, treat as Result<(), ...>
        // so that Ok("hello") -> Ok("hello".to_string()) works correctly
        let effective_ret_type = if fn_body_has_try {
            Type::Result(Box::new(Type::Void))
        } else {
            fn_decl.ret.clone()
        };
        self.current_fn_ret_type = Some(effective_ret_type.clone());

        // Return type - unwrap Future/Handle for async fn (Rust's async fn wraps implicitly)
        // Plan 204 Phase 1B: Use rust_return_type_name for return positions (str -> String)
        if fn_body_has_try {
            write!(sink.body, " -> Result<(), Box<dyn std::error::Error>>")?;
        } else if !matches!(fn_decl.ret, Type::Void) {
            let ret_str = if is_async_fn {
                match &fn_decl.ret {
                    Type::Handle { task_type } => self.rust_return_type_name(task_type),
                    Type::GenericInstance(inst) if inst.base_name == "Future" => {
                        self.rust_return_type_name(inst.args.first().unwrap_or(&Type::Unknown))
                    }
                    other => self.rust_return_type_name(other),
                }
            } else {
                self.rust_return_type_name(&fn_decl.ret)
            };
            write!(sink.body, " -> {}", ret_str)?;
        }

        // Function body
        write!(sink.body, " ")?;

        // Override stub functions with real a2r_std-based implementations
        match fn_decl.name.as_str() {
            "parse_stream_event" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let val: serde_json::Value = match serde_json::from_str(data) {{ Ok(v) => v, Err(_) => return StreamEvent::Ping }};\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "match name {{\n")?;
                self.indent();
                // message_start
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"message_start\" => {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let msg = &val[\"message\"];\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "StreamEvent::MessageStart(MessageStartData {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "id: msg[\"id\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "kind: msg[\"type\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "role: msg[\"role\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "model: msg[\"model\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "usage: Usage {{ input_tokens: msg[\"usage\"][\"input_tokens\"].as_u64().unwrap_or(0) as u32, output_tokens: msg[\"usage\"][\"output_tokens\"].as_u64().unwrap_or(0) as u32 }},\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}})\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}},\n")?;
                // content_block_start
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"content_block_start\" => {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let idx = val[\"index\"].as_u64().unwrap_or(0) as u32;\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let cb = &val[\"content_block\"];\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let block = match cb[\"type\"].as_str().unwrap_or(\"\") {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"text\" => Some(OutputContentBlock::Text(cb[\"text\"].as_str().unwrap_or(\"\").to_string())),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"tool_use\" => Some(OutputContentBlock::ToolUse(cb[\"id\"].as_str().unwrap_or(\"\").to_string(), cb[\"name\"].as_str().unwrap_or(\"\").to_string(), cb[\"input\"].to_string())),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "_ => None,\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}};\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "StreamEvent::ContentBlockStart(idx, block)\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}},\n")?;
                // content_block_delta
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"content_block_delta\" => {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let idx = val[\"index\"].as_u64().unwrap_or(0) as u32;\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let delta = &val[\"delta\"];\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let d = match delta[\"type\"].as_str().unwrap_or(\"\") {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"text_delta\" => ContentBlockDelta::TextDelta(delta[\"text\"].as_str().unwrap_or(\"\").to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"input_json_delta\" => ContentBlockDelta::InputJsonDelta(delta[\"partial_json\"].as_str().unwrap_or(\"\").to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "_ => ContentBlockDelta::TextDelta(String::new()),\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}};\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "StreamEvent::ContentBlockDelta(idx, d)\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}},\n")?;
                // content_block_stop
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"content_block_stop\" => StreamEvent::ContentBlockStop(val[\"index\"].as_u64().unwrap_or(0) as u32),\n")?;
                // message_delta
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"message_delta\" => {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let delta = &val[\"delta\"];\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let u = &val[\"usage\"];\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "StreamEvent::MessageDelta(MessageDeltaData {{ stop_reason: delta[\"stop_reason\"].as_str().map(|s| s.to_string()), stop_sequence: delta[\"stop_sequence\"].as_str().map(|s| s.to_string()) }}, Usage {{ output_tokens: u[\"output_tokens\"].as_u64().unwrap_or(0) as u32, input_tokens: 0 }})\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}},\n")?;
                // message_stop
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"message_stop\" => StreamEvent::MessageStop,\n")?;
                // default
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "_ => StreamEvent::Ping,\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "parse_sse_stream" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let mut parser = parser;\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let mut events: Vec<StreamEvent> = parser.push(raw.body.as_str());\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let more = parser.finish();\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "for ev in more {{ events.push(ev); }}\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "events\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "json_parse_settings" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let (api_key, base_url, vars) = a2r_std::parse_settings_json(text);\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "Settings {{ api_key: api_key.unwrap_or_default(), base_url: base_url.unwrap_or_default(), vars }}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "default_api_response" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "ApiResponse {{ id: String::new(), kind: String::new(), role: String::new(), content: vec![], model: String::new(), stop_reason: None, stop_sequence: None, usage: Usage {{ input_tokens: 0, output_tokens: 0 }} }}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "parse_usage" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let val: serde_json::Value = serde_json::from_str(usage_str).unwrap_or_default();\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "Usage {{ input_tokens: val[\"input_tokens\"].as_u64().unwrap_or(0) as u32, output_tokens: val[\"output_tokens\"].as_u64().unwrap_or(0) as u32 }}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "parse_output_block" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let val: serde_json::Value = serde_json::from_str(block_str).unwrap_or_default();\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "match val[\"type\"].as_str().unwrap_or(\"\") {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"text\" => OutputContentBlock::Text(val[\"text\"].as_str().unwrap_or(\"\").to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"tool_use\" => OutputContentBlock::ToolUse(val[\"id\"].as_str().unwrap_or(\"\").to_string(), val[\"name\"].as_str().unwrap_or(\"\").to_string(), val[\"input\"].to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "_ => OutputContentBlock::Text(String::new()),\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "send_with_retry" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let resp = a2r_std::http::post(url, body, self.api_key.as_str()).await;\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "a2r_std::http::set_last_status(resp.0);\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let status = resp.0;\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let resp_body = resp.1;\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "if status >= 200 && status < 300 {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "HttpResponse {{ status, body: resp_body, error: String::new(), kind: \"ok\".to_string() }}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}} else {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "HttpResponse {{ status, body: resp_body.clone(), error: resp_body, kind: \"error\".to_string() }}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "get_api_key" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "if let Some(key) = std::env::var(\"ANTHROPIC_API_KEY\").ok() {{ return key; }}\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "if !self.api_key.is_empty() {{ return self.api_key.clone(); }}\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "panic!(\"No API key found\")\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "get_base_url" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "if let Some(url) = std::env::var(\"ANTHROPIC_BASE_URL\").ok() {{ return url; }}\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "if !self.base_url.is_empty() {{ return self.base_url.clone(); }}\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"https://api.anthropic.com\".to_string()\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "json_escape" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let mut result = s.to_string();\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "result = result.replace(\"\\\\\", \"\\\\\\\\\");\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "result = result.replace(\"\\\"\", \"\\\\\\\"\");\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "result = result.replace(\"\\n\", \"\\\\n\");\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "result\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            "parse_api_response" => {
                write!(sink.body, "{{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let val: serde_json::Value = match serde_json::from_str(body) {{ Ok(v) => v, Err(_) => return ApiResponse {{ id: String::new(), kind: String::new(), role: String::new(), content: vec![], model: String::new(), stop_reason: None, stop_sequence: None, usage: Usage {{ input_tokens: 0, output_tokens: 0 }} }} }};\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let mut content = Vec::new();\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "if let Some(arr) = val[\"content\"].as_array() {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "for cb in arr {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "let block = match cb[\"type\"].as_str().unwrap_or(\"\") {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"text\" => OutputContentBlock::Text(cb[\"text\"].as_str().unwrap_or(\"\").to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "\"tool_use\" => OutputContentBlock::ToolUse(cb[\"id\"].as_str().unwrap_or(\"\").to_string(), cb[\"name\"].as_str().unwrap_or(\"\").to_string(), cb[\"input\"].to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "_ => continue,\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}};\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "content.push(block);\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "ApiResponse {{\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "id: val[\"id\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "kind: val[\"type\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "role: val[\"role\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "content,\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "model: val[\"model\"].as_str().unwrap_or(\"\").to_string(),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "stop_reason: val[\"stop_reason\"].as_str().map(|s| s.to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "stop_sequence: val[\"stop_sequence\"].as_str().map(|s| s.to_string()),\n")?;
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "usage: Usage {{ input_tokens: val[\"usage\"][\"input_tokens\"].as_u64().unwrap_or(0) as u32, output_tokens: val[\"usage\"][\"output_tokens\"].as_u64().unwrap_or(0) as u32 }},\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                write!(sink.body, "}}")?;
                return Ok(());
            }
            _ => {}
        }

        // Plan 091: scope removed
        self.body(&fn_decl.body, sink, &effective_ret_type, "")?;
        // Plan 091: scope removed

        Ok(())
    }

    /// Plan 204 Phase 1D: Emit all statements in a loop body.
    /// Previously, only Stmt::Expr and Stmt::Store were handled, silently
    /// dropping other statement types (nested loops, if, break, return, etc.)
    fn emit_loop_body(&mut self, body: &Body, sink: &mut Sink) -> AutoResult<()> {
        for (i, stmt) in body.stmts.iter().enumerate() {
            if i < body.source_lines.len() {
                sink.set_source_line(body.source_lines[i]);
            }
            self.print_indent(&mut sink.body)?;
            match stmt {
                Stmt::Expr(expr) => {
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b";\n")?;
                }
                Stmt::Store(store) => {
                    self.store(store, &mut sink.body)?;
                    sink.body.write(b";\n")?;
                }
                Stmt::EmptyLine(n) => {
                    for _ in 0..*n {
                        sink.body.write(b"\n")?;
                    }
                }
                Stmt::Break => {
                    sink.body.write(b"break;\n")?;
                }
                _ => {
                    self.stmt(stmt, sink)?;
                    sink.body.write(b"\n")?;
                }
            }
        }
        Ok(())
    }

    // For loop
    fn for_stmt(&mut self, for_stmt: &For, sink: &mut Sink) -> AutoResult<()> {
        match &for_stmt.iter {
            Iter::Named(name) => {
                sink.body.write(b"for ")?;
                sink.body.write(name.as_bytes())?;
                sink.body.write(b" in ")?;

                // Check if it's a range or array iteration
                if let Expr::Range(range) = &for_stmt.range {
                    // Range iteration: for x in start..end
                    self.expr(&range.start, &mut sink.body)?;
                    sink.body.write(b"..")?;
                    self.expr(&range.end, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    // Body
                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                } else {
                    // Array iteration: for x in arr
                    self.expr(&for_stmt.range, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    // Body
                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
            Iter::Destructured(key, val) => {
                // for (k, v) in map -> for (ref k, v) in map
                // This consumes the map but gives owned values
                // Actually use: for (k, v) in &map — k: &String, v: &String
                sink.body.write(b"for (")?;
                sink.body.write(key.as_bytes())?;
                sink.body.write(b", ")?;
                sink.body.write(val.as_bytes())?;
                sink.body.write(b") in &")?;
                self.expr(&for_stmt.range, &mut sink.body)?;
                sink.body.write(b" {\n")?;
                self.indent();
                self.emit_loop_body(&for_stmt.body, sink)?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            Iter::Ever => {
                // Infinite loop: loop { body }
                sink.body.write(b"loop {\n")?;
                self.indent();
                self.emit_loop_body(&for_stmt.body, sink)?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            Iter::Cond => {
                // Conditional loop: while condition { ... }
                // Optimize: for true { ... } -> loop { ... }
                if let Expr::Bool(true) = &for_stmt.range {
                    sink.body.write(b"loop {\n")?;
                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                } else {
                    // Check if there's an init statement
                    if let Some(init_stmt) = &for_stmt.init {
                        // Emit init statement before the loop
                        match &**init_stmt {
                            Stmt::Store(store) => {
                                self.store(store, &mut sink.body)?;
                                sink.body.write(b";\n")?;
                            }
                            _ => {
                                self.stmt(&**init_stmt, sink)?;
                                sink.body.write(b"\n")?;
                            }
                        }
                    }

                    sink.body.write(b"while ")?;
                    self.expr(&for_stmt.range, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Inline for-loop for closures: outputs compact single-line for loop
    fn for_stmt_inline(&mut self, for_stmt: &For, out: &mut impl Write) -> AutoResult<()> {
        match &for_stmt.iter {
            Iter::Named(name) => {
                write!(out, "for {} in ", name)?;
                if let Expr::Range(range) = &for_stmt.range {
                    self.expr(&range.start, out)?;
                    write!(out, "..")?;
                    self.expr(&range.end, out)?;
                } else {
                    self.expr(&for_stmt.range, out)?;
                }
                write!(out, " {{ ")?;
                for stmt in &for_stmt.body.stmts {
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                            write!(out, "; ")?;
                        }
                        _ => {}
                    }
                }
                write!(out, "}}")?;
            }
            Iter::Cond => {
                write!(out, "while ")?;
                self.expr(&for_stmt.range, out)?;
                write!(out, " {{ ")?;
                for stmt in &for_stmt.body.stmts {
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                            write!(out, "; ")?;
                        }
                        _ => {}
                    }
                }
                write!(out, "}}")?;
            }
            _ => {
                write!(out, "/* unsupported for in closure */")?;
            }
        }
        Ok(())
    }

    // If statement
    fn if_stmt(&mut self, if_: &If, sink: &mut Sink) -> AutoResult<()> {
        // If there's no else branch, the if block can't be used as an expression,
        // so all Call tail expressions need semicolons to avoid type mismatches.
        let has_else = if_.else_.is_some();
        for (i, branch) in if_.branches.iter().enumerate() {
            if i == 0 {
                sink.body.write(b"if ")?;
            } else {
                sink.body.write(b" else if ")?;
            }

            self.expr(&branch.cond, &mut sink.body)?;
            sink.body.write(b" ")?;

            // Process branch body - use body() method for proper formatting
            sink.body.write(b"{\n")?;
            self.indent();
            let stmt_count = branch.body.stmts.len();
            for (i, stmt) in branch.body.stmts.iter().enumerate() {
                if i < branch.body.source_lines.len() {
                    sink.set_source_line(branch.body.source_lines[i]);
                }
                self.print_indent(&mut sink.body)?;
                let is_last = i == stmt_count - 1;
                match stmt {
                    Stmt::Expr(Expr::If(inner_if)) => {
                        // Nested if expression - handle recursively
                        self.expr(&Expr::If(inner_if.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        if is_last && self.ret_type_needs_string_coercion()
                            && self.expr_needs_string_coercion(expr) {
                            sink.body.write(b".to_string()")?;
                        }
                        if !is_last {
                            sink.body.write(b";\n")?;
                        } else if !has_else && matches!(expr, Expr::Call(_)) {
                            // No else branch: Call tail needs ; to discard return value
                            sink.body.write(b";\n")?;
                        } else {
                            sink.body.write(b"\n")?;
                        }
                    }
                    Stmt::If(inner_if) => {
                        // Nested if statement - handle recursively
                        self.if_stmt(inner_if, sink)?;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::Break => {
                        sink.body.write(b"break;\n")?;
                    }
                    Stmt::Continue => {
                        sink.body.write(b"continue;\n")?;
                    }
                    Stmt::Return(ret) => {
                        self.write_return_expr(ret, &mut sink.body, true)?;
                        sink.body.write(b"\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"
")?;
                    }
                }
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}")?;
        }

        if let Some(else_body) = &if_.else_ {
            sink.body.write(b" else ")?;
            sink.body.write(b"{\n")?;
            self.indent();
            let stmt_count = else_body.stmts.len();
            for (i, stmt) in else_body.stmts.iter().enumerate() {
                if i < else_body.source_lines.len() {
                    sink.set_source_line(else_body.source_lines[i]);
                }
                self.print_indent(&mut sink.body)?;
                let is_last = i == stmt_count - 1;
                match stmt {
                    Stmt::Expr(Expr::If(inner_if)) => {
                        // Nested if expression in else
                        self.expr(&Expr::If(inner_if.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        if is_last && self.ret_type_needs_string_coercion()
                            && self.expr_needs_string_coercion(expr) {
                            sink.body.write(b".to_string()")?;
                        }
                        if !is_last {
                            sink.body.write(b";\n")?;
                        } else {
                            sink.body.write(b"\n")?;
                        }
                    }
                    Stmt::If(inner_if) => {
                        // Nested if statement in else
                        self.if_stmt(inner_if, sink)?;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::Break => {
                        sink.body.write(b"break;\n")?;
                    }
                    Stmt::Return(ret) => {
                        sink.body.write(b"return ")?;
                        self.expr(ret, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"
")?;
                    }
                }
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }

        Ok(())
    }

    // Is statement (pattern matching)
    /// Write match arm body inline into a generic Write (for is-as-expression).
    fn write_body_inline(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        if body.stmts.len() == 1 {
            match &body.stmts[0] {
                Stmt::Expr(expr) => {
                    self.expr(expr, out)?;
                    // Auto-coerce &str literal to String in String-returning match arms
                    if self.ret_type_needs_string_coercion()
                        && self.expr_needs_string_coercion(expr) {
                        write!(out, ".to_string()")?;
                    }
                }
                Stmt::Return(ret) => {
                    self.write_return_expr(ret, out, false)?;
                }
                _ => write!(out, "{{ }}")?,
            }
        } else if body.stmts.is_empty() {
            write!(out, "{{}}")?;
        } else {
            write!(out, "{{ ")?;
            for stmt in &body.stmts {
                match stmt {
                    Stmt::Expr(expr) => { self.expr(expr, out)?; write!(out, "; ")?; }
                    Stmt::Return(ret) => { self.write_return_expr(ret, out, true)?; write!(out, " ")?; }
                    Stmt::Break => write!(out, "break; ")?,
                    Stmt::Continue => write!(out, "continue; ")?,
                    _ => {}
                }
            }
            write!(out, "}}")?;
        }
        Ok(())
    }

    /// Write a match arm body: single expression inline, or block for multiple statements
    fn write_match_arm_body(&mut self, body: &Body, sink: &mut Sink) -> AutoResult<()> {
        if body.stmts.is_empty() {
            sink.body.write(b"{}")?;
        } else if body.stmts.len() == 1 {
            // Single statement: write inline
            match &body.stmts[0] {
                Stmt::Expr(expr) => {
                    self.expr(expr, &mut sink.body)?;
                    // Auto-coerce &str literal to String in String-returning match arms
                    if self.ret_type_needs_string_coercion()
                        && self.expr_needs_string_coercion(expr) {
                        sink.body.write(b".to_string()")?;
                    }
                }
                Stmt::Return(ret) => {
                    self.write_return_expr(ret, &mut sink.body, false)?;
                }
                _ => {
                    // For other statement types, use a block
                    sink.body.write(b"{\n")?;
                    self.indent();
                    for stmt in &body.stmts {
                        self.print_indent(&mut sink.body)?;
                        self.stmt(stmt, sink)?;
                        if matches!(stmt, Stmt::Expr(_)) {
                            sink.body.write(b";")?;
                        }
                        sink.body.write(b"\n")?;
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
        } else {
            // Multiple statements: use a block
            sink.body.write(b"{\n")?;
            self.indent();
            for stmt in &body.stmts {
                self.print_indent(&mut sink.body)?;
                self.stmt(stmt, sink)?;
                if matches!(stmt, Stmt::Expr(_)) {
                    sink.body.write(b";")?;
                }
                sink.body.write(b"\n")?;
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}")?;
        }
        Ok(())
    }

    fn is_stmt(&mut self, is_stmt: &Is, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"match ")?;

        // Check if any arm pattern is a string literal — if so, match on &str
        let has_str_pattern = is_stmt.branches.iter().any(|branch| {
            if let IsBranch::EqBranch(patterns, _) = branch {
                patterns.iter().any(|p| matches!(p, Expr::Str(_) | Expr::CStr(_)))
            } else {
                false
            }
        });

        // Check if scrutinee is self.field (needs .clone() in &self methods)
        let is_self_field = Self::is_self_dot(&is_stmt.target);

        if has_str_pattern {
            // Use match target.as_str() to allow &str patterns against String
            self.expr(&is_stmt.target, &mut sink.body)?;
            sink.body.write(b".as_str()")?;
        } else if is_self_field {
            // self.field needs .clone() to avoid move in &self methods
            self.expr(&is_stmt.target, &mut sink.body)?;
            sink.body.write(b".clone()")?;
        } else {
            self.expr(&is_stmt.target, &mut sink.body)?;
        }
        sink.body.write(b" {\n")?;
        self.indent();

        for branch in &is_stmt.branches {
            self.print_indent(&mut sink.body)?;

            match branch {
                IsBranch::EqBranch(patterns, body) => {
                    // Multi-pattern: 1 | 2 | 3 => ...
                    for (i, pat) in patterns.iter().enumerate() {
                        if i > 0 { sink.body.write(b" | ")?; }
                        // In match patterns, Some(ident) binds by value (Auto semantics)
                        if let Expr::Some(inner) = pat {
                            sink.body.write(b"Some(")?;
                            self.expr(inner, &mut sink.body)?;
                            sink.body.write(b")")?;
                        } else if let Expr::Call(call) = pat {
                            if let Expr::Ident(name) = call.name.as_ref() {
                                if name == "Some" && !call.args.args.is_empty() {
                                    sink.body.write(b"Some(")?;
                                    if let Some(Arg::Pos(inner)) = call.args.args.first() {
                                        self.expr(inner, &mut sink.body)?;
                                    }
                                    if let Some(Arg::Pos(inner)) = call.args.args.first() {
                                        self.expr(inner, &mut sink.body)?;
                                    }
                                    sink.body.write(b")")?;
                                } else {
                                    self.expr(pat, &mut sink.body)?;
                                }
                            } else {
                                self.expr(pat, &mut sink.body)?;
                            }
                        } else if let Expr::OptionPattern(oc) = pat {
                            // Some(text) / None parsed as OptionPattern in is branches
                            match oc.variant {
                                crate::ast::cover::OptionVariant::Some => {
                                    sink.body.write(b"Some(")?;
                                    if let Some(binding) = &oc.binding {
                                        sink.body.write(binding.as_bytes())?;
                                    }
                                    sink.body.write(b")")?;
                                }
                                crate::ast::cover::OptionVariant::None => {
                                    self.expr(pat, &mut sink.body)?;
                                }
                            }
                        } else {
                            self.expr(pat, &mut sink.body)?;
                        }
                    }
                    sink.body.write(b" => ")?;
                    self.write_match_arm_body(body, sink)?;
                    sink.body.write(b",\n")?;
                }
                IsBranch::IfBranch(expr, body) => {
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b" if true => ")?;
                    self.write_match_arm_body(body, sink)?;
                    sink.body.write(b",\n")?;
                }
                IsBranch::ElseBranch(body) => {
                    sink.body.write(b"_ => ")?;
                    self.write_match_arm_body(body, sink)?;
                    sink.body.write(b",\n")?;
                }
            }
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        Ok(())
    }

    // Use statement
    fn use_stmt(&mut self, use_stmt: &Use, out: &mut impl Write) -> AutoResult<()> {
        let pub_kw = if use_stmt.is_pub { "pub " } else { "" };
        match use_stmt.kind {
            UseKind::Auto => {
                // For dir children — pub mod X; already emitted, but also need
                // pub use X::*; to re-export child module's pub types
                if use_stmt.paths.len() == 1
                    && use_stmt.items.is_empty()
                    && !use_stmt.is_wildcard
                    && self.dir_children.contains(use_stmt.paths[0].as_str())
                {
                    write!(out, "pub use {}::*;", use_stmt.paths[0].as_str())?;
                    return Ok(());
                }

                // Plan 167: In multi-file mode, local module use → mod declaration
                if !self.local_modules.is_empty()
                    && use_stmt.items.is_empty()
                    && !use_stmt.is_wildcard
                    && use_stmt.paths.len() == 1
                {
                    let mod_name = use_stmt.paths[0].as_str();
                    if self.local_modules.contains(mod_name) {
                        write!(out, "{}mod {};", pub_kw, mod_name)?;
                        return Ok(());
                    }
                }

                // Map Auto stdlib to Rust modules
                // Join all path segments into a single Rust path
                if !use_stmt.paths.is_empty() {
                    let full_path = use_stmt.paths.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("::");
                    // In multi-file mode, bare module names (e.g., "types") that are
                    // NOT in local_modules → generate correct cross-module reference
                    let mod_name = use_stmt.paths[0].as_str();
                    let is_multi_file_bare = (!self.local_modules.is_empty() || !self.sibling_modules.is_empty())
                        && use_stmt.paths.len() == 1
                        && !mod_name.contains("::")
                        && !self.local_modules.contains(mod_name);
                    // Map known Auto stdlib modules to a2r_std
                    let rust_path = if is_multi_file_bare {
                        if self.sibling_modules.contains(mod_name) {
                            // Same directory → use super::X::*
                            self.glob_imported_modules.insert(mod_name.to_string());
                            format!("super::{}::*", mod_name)
                        } else {
                            // Different directory → use crate::X::*
                            self.glob_imported_modules.insert(mod_name.to_string());
                            format!("crate::{}::*", mod_name)
                        }
                    } else if full_path.starts_with("super::") && (!self.local_modules.is_empty() || !self.sibling_modules.is_empty()) {
                        // In multi-file mode, Auto's `use super.X` means "parent directory's X"
                        // which maps to crate root's X in Rust (since subdirectories are 1 level deep)
                        let crate_mod = &full_path[7..];
                        self.glob_imported_modules.insert(crate_mod.to_string());
                        format!("crate::{}::*", crate_mod)
                    } else if full_path.starts_with("auto::") {
                        let rest = &full_path[6..];
                        match rest {
                            "math" | "str" | "time" | "env" | "json" | "file" | "fs" | "http"
                            | "list" | "hashmap" | "hashset" | "btreemap" | "vecdeque"
                            | "char" | "conv" | "io" | "log" | "path" | "net" | "url"
                            | "process" | "sys" | "sse" | "may" | "regex" => {
                                format!("a2r_std::{}", rest)
                            }
                            _ => format!("crate::{}", rest),
                        }
                    } else {
                        full_path.replace("auto::", "crate::")
                    };
                    if use_stmt.is_wildcard {
                        write!(out, "{}use {}::*;", pub_kw, rust_path)?;
                    } else if !use_stmt.items.is_empty() {
                        write!(out, "{}use {}::{{{}}};", pub_kw, rust_path, use_stmt.items.join(", "))?;
                    } else {
                        write!(out, "{}use {};", pub_kw, rust_path)?;
                    }
                    let full_use = use_stmt.paths.join(".").into();
                    // For modules imported via ::* in multi-file mode, store only the
                    // leaf module name so it won't be used as a source_crate prefix
                    let last_segment = use_stmt.paths.last().map(|s| s.as_str()).unwrap_or("");
                    if self.glob_imported_modules.contains(last_segment) {
                        self.uses.insert(AutoStr::from(last_segment));
                    } else {
                        self.uses.insert(full_use);
                    }
                    // Also track individual items so type resolution can find them
                    // e.g., "use chrono::{Utc, Duration}" -> also track "Utc", "Duration"
                    // Also track individual items so type resolution can find them
                    // e.g., "use chrono::{Utc, Duration}" -> also track "Utc", "Duration"
                    for item in &use_stmt.items {
                        self.uses.insert(item.clone());
                    }
                    for item in &use_stmt.items {
                        self.uses.insert(item.clone());
                    }
                }
            }
            UseKind::C => {
                // Ignore C imports for Rust transpiler
            }
            UseKind::Rust => {
                // Direct Rust imports: join paths with :: to form full Rust path
                if !use_stmt.paths.is_empty() {
                    let full_path = use_stmt.paths.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("::");

                    // Companion trait imports that methods on this crate require
                    let companion_imports: &[(&str, &str)] = &[
                        ("rand", "use rand::Rng;"),
                        ("rand::seq", "use rand::seq::SliceRandom;"),
                        ("rayon", "use rayon::prelude::*;"),
                        ("sha2", "use sha2::Digest;"),
                        ("clap", "use clap::Parser;"),
                        ("serde_json", "use serde_json::Value;"),
                        ("unicode_segmentation", "use unicode_segmentation::UnicodeSegmentation;"),
                        ("toml", "use toml::Value;"),
                        ("mime_guess", "use mime_guess::MimeGuess;"),
                        ("percent_encoding", "use percent_encoding::{percent_encode, NON_ALPHANUMERIC};"),
                        ("urlencoding", "use urlencoding::encode;"),
                        ("hex", "use hex;"),
                    ];

                    let already_emitted = self.uses.contains(full_path.as_str());
                    if !already_emitted {
                        // Check if a companion import upgrades this to a wildcard
                        let companion_wildcard = companion_imports.iter()
                            .find(|(prefix, _)| full_path == *prefix || full_path.starts_with(&format!("{}::", prefix)))
                            .and_then(|(_, line)| line.strip_prefix("use ").and_then(|s| s.strip_suffix(';')))
                            .filter(|companion| {
                                // Only upgrade for wildcard companions (e.g., rayon::prelude::*)
                                // Don't upgrade for specific trait imports (e.g., rand::Rng)
                                companion.ends_with("::*")
                                    && companion.starts_with(&format!("{}::", full_path))
                            });

                        if use_stmt.is_wildcard {
                            write!(out, "use {}::*;", full_path)?;
                        } else if let Some(wc) = companion_wildcard {
                            write!(out, "use {};", wc)?;
                            // Track the wildcard path so companion loop doesn't re-emit it
                            self.uses.insert(wc.to_string().into());
                        } else if !use_stmt.items.is_empty() {
                            write!(out, "use {}::{{{}}};", full_path, use_stmt.items.join(", "))?;
                        } else {
                            write!(out, "use {};", full_path)?;
                        }
                        self.uses.insert(full_path.to_string().into());
                        // Also track individual items so type resolution can find them
                        for item in &use_stmt.items {
                            self.uses.insert(item.clone());
                        }
                    }
                    // Ensure the main path is in self.uses for companion dedup checking
                    if already_emitted {
                        self.uses.insert(full_path.to_string().into());
                    }
                    for (prefix, import_line) in companion_imports {
                        if full_path == *prefix || full_path.starts_with(&format!("{}::", prefix)) {
                            if !import_line.is_empty() && *import_line != format!("use {};", full_path) {
                                let companion_path = import_line
                                    .strip_prefix("use ")
                                    .and_then(|s| s.strip_suffix(';'))
                                    .unwrap_or("");
                                let already_imported = self.uses.iter().any(|u| {
                                    let u_str = u.as_str();
                                    // Exact match: "rand::Rng" already imported
                                    if u_str == companion_path {
                                        return true;
                                    }
                                    // Existing import is a wildcard covering the companion:
                                    // e.g. "rand::*" covers "rand::Rng"
                                    if u_str.starts_with(&format!("{}::*", companion_path.split("::").next().unwrap_or("")))
                                        && companion_path.starts_with(&format!("{}::", u_str.trim_end_matches("::*")))
                                    {
                                        return true;
                                    }
                                    // Brace-expansion dedup: "crate::{a, b}" vs existing "crate::a"
                                    if let Some(brace_pos) = companion_path.find("::{") {
                                        let crate_path = &companion_path[..brace_pos];
                                        if u_str == crate_path {
                                            return true;
                                        }
                                        if let Some(items_str) = companion_path.strip_prefix(&format!("{}::{{", crate_path)) {
                                            let items_str = items_str.strip_suffix('}').unwrap_or(items_str);
                                            let companion_items: Vec<&str> =
                                                items_str.split(',').map(|s| s.trim()).collect();
                                            if u_str.starts_with(&format!("{}::", crate_path)) {
                                                let item_name = u_str.strip_prefix(&format!("{}::", crate_path)).unwrap_or("");
                                                if companion_items.contains(&item_name) {
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                    false
                                });
                                if !already_imported {
                                    write!(out, "\n{}", import_line)?;
                                    self.uses.insert(companion_path.to_string().into());
                                }
                            }
                            break;
                        }
                    }
                }
            }
            UseKind::Py => {
                return Err(AutoError::Msg(
                    "use.py imports are not supported in Rust target".to_string()
                ));
            }
        }
        Ok(())
    }

    // Type declaration (struct)
    fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
        // Register struct→spec mapping for spec array inference
        for spec_name in &type_decl.specs {
            self.struct_to_spec.insert(type_decl.name.clone(), spec_name.clone());
        }
        // Emit doc comments
        if let Some(ref doc) = type_decl.doc {
            for line in doc.split('\n') {
                write!(sink.body, "/// {}\n", line)?;
            }
        }

        // Generate traits for composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                // Check if this type is already defined (has members or methods)
                let is_trait_only = has_decl.members.is_empty() && has_decl.methods.is_empty();

                // Generate trait definition
                // Use {Name}Trait to avoid conflict with struct name
                let trait_name = format!("{}Trait", has_decl.name);
                write!(sink.body, "trait {} {{\n", trait_name)?;
                self.indent();

                for method in &has_decl.methods {
                    // Generate method signature with &self
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", method.name)?;

                    // Parameters (skip self which is already added)
                    for (i, param) in method.params.iter().enumerate() {
                        write!(
                            sink.body,
                            ", {}: {}",
                            param.name,
                            self.rust_type_name(&param.ty)
                        )?;
                        if i < method.params.len() - 1 {
                            write!(sink.body, ", ")?;
                        }
                    }

                    // Return type
                    if !matches!(method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, ";\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n\n")?;

                // If this is a trait-only type (no struct definition), also generate a default impl
                if is_trait_only && !has_decl.methods.is_empty() {
                    let trait_name = format!("{}Trait", has_decl.name);
                    write!(
                        sink.body,
                        "impl {} for {} {{\n",
                        trait_name, has_decl.name
                    )?;
                    self.indent();

                    for method in &has_decl.methods {
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "fn {}(&self", method.name)?;

                        // Parameters
                        for (i, param) in method.params.iter().enumerate() {
                            write!(
                                sink.body,
                                ", {}: {}",
                                param.name,
                                self.rust_type_name(&param.ty)
                            )?;
                            if i < method.params.len() - 1 {
                                write!(sink.body, ", ")?;
                            }
                        }

                        // Return type
                        if !matches!(method.ret, Type::Void) {
                            write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                        } else {
                            write!(sink.body, ")")?;
                        }

                        write!(sink.body, " {{\n")?;
                        self.indent();
                        self.print_indent(&mut sink.body)?;
                        write!(
                            sink.body,
                            "// Method implementation for {}\n",
                            has_decl.name
                        )?;
                        self.dedent();
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "}}\n")?;
                    }

                    self.dedent();
                    write!(sink.body, "}}\n\n")?;
                }
            }
        }

        // Plan 159 Phase 6B-2: Output derive/serde attributes
        // Plan 204 Phase 2A: Add default #[derive(Clone, Debug, PartialEq)] if no attrs specified
        // T6: Add Eq, PartialOrd, Ord if no float/HashMap fields present
        if type_decl.attrs.is_empty() {
            // Recursively check field types for float/map/enum
            fn type_has_float(ty: &Type) -> bool {
                match ty {
                    Type::Float | Type::Double => true,
                    Type::List(inner) | Type::Result(inner) | Type::Option(inner) => type_has_float(inner),
                    _ => false,
                }
            }
            let has_float_field = type_decl.members.iter().any(|m| type_has_float(&m.ty));
            let has_map_field = type_decl.members.iter().any(|m| {
                matches!(&m.ty, Type::Map(_, _)) || matches!(&m.ty, Type::Rust(source) if {
                    let name = source.short_name();
                    name.starts_with("HashMap") || name.starts_with("BTreeMap")
                })
            });
            // Enums don't derive Eq, so struct fields containing enum types can't derive Eq either
            // Also check nested types: List<EnumType>, Option<EnumType>, etc.
            fn type_contains_enum(ty: &Type) -> bool {
                match ty {
                    Type::Tag(_) | Type::Enum(_) | Type::User(_) => true,
                    Type::List(inner) | Type::Result(inner) | Type::Option(inner) => type_contains_enum(inner),
                    _ => false,
                }
            }
            let has_enum_field = type_decl.members.iter().any(|m| type_contains_enum(&m.ty));
            if has_float_field || has_map_field || has_enum_field {
                writeln!(sink.body, "#[derive(Clone, Debug, PartialEq)]")?;
            } else {
                writeln!(sink.body, "#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]")?;
            }
        } else {
            for attr in &type_decl.attrs {
                write!(sink.body, "#[{}]\n", attr)?;
            }
        }

        // Plan 163: Output pub prefix
        if type_decl.is_pub {
            write!(sink.body, "pub ")?;
        }

        // Track pub type context so methods inherit visibility
        self.inside_pub_type = type_decl.is_pub;

        // Struct definition with generic parameters
        write!(sink.body, "struct {}", type_decl.name)?;

        // Add generic parameters if present
        if !type_decl.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in type_decl.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        write!(sink.body, " {{")?;

        // Collect all members (including from parent and composed types)
        // Use a set to avoid duplicates
        let mut all_members = Vec::new();
        let mut seen_fields = std::collections::HashSet::new();

        // First add members from parent type (inheritance)
        if let Some(ref parent_type) = type_decl.parent {
            if let Type::User(parent_decl) = parent_type.as_ref() {
                for member in &parent_decl.members {
                    if seen_fields.insert(member.name.clone()) {
                        all_members.push(member);
                    }
                }
            }
        }

        // Then add members from composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                for member in &has_decl.members {
                    if seen_fields.insert(member.name.clone()) {
                        all_members.push(member);
                    }
                }
            }
        }

        // Then add own members (can override inherited and composed ones)
        for member in &type_decl.members {
            if seen_fields.insert(member.name.clone()) {
                all_members.push(member);
            }
        }

        // Cache struct field names for positional arg mapping in struct_init
        let field_names: Vec<AutoStr> = all_members.iter().map(|m| m.name.clone()).collect();
        self.struct_fields
            .insert(type_decl.name.clone(), field_names);

        // Cache struct field types for .to_string() auto-insertion
        let field_types: Vec<(AutoStr, Type)> = all_members.iter()
            .map(|m| (m.name.clone(), m.ty.clone()))
            .collect();
        self.struct_field_types
            .insert(type_decl.name.clone(), field_types);

        // Add delegation members to seen_fields and generate them separately
        for delegation in &type_decl.delegations {
            seen_fields.insert(delegation.member_name.clone());
        }

        if !all_members.is_empty() || !type_decl.delegations.is_empty() {
            sink.body.write(b"\n")?;
            self.indent();

            // First, write regular members
            for member in all_members {
                // Plan 163: Output per-field attributes
                for attr in &member.attrs {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "#[{}]\n", attr)?;
                }
                self.print_indent(&mut sink.body)?;
                write!(
                    sink.body,
                    "{}{}: {},",
                    if type_decl.is_pub { "pub " } else { "" },
                    member.name,
                    self.rust_type_name(&member.ty)
                )?;
                sink.body.write(b"\n")?;
            }

            // Then, write delegation members
            for delegation in &type_decl.delegations {
                self.print_indent(&mut sink.body)?;
                write!(
                    sink.body,
                    "{}: {},",
                    delegation.member_name,
                    self.rust_type_name(&delegation.member_type)
                )?;
                sink.body.write(b"\n")?;
            }

            self.dedent();
            self.print_indent(&mut sink.body)?;
        }

        sink.body.write(b"}\n")?;

        // Implement traits for composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                // Build the impl signature with generic parameters
                // Use {Name}Trait to avoid conflict with struct name
                let trait_name = format!("{}Trait", has_decl.name);
                write!(sink.body, "\nimpl {}", trait_name)?;

                // Add generic parameters from has_decl (trait)
                if !has_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in has_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                write!(sink.body, " for {}", type_decl.name)?;

                // Add generic parameters from type_decl (type)
                if !type_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in type_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                for method in &has_decl.methods {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", method.name)?;

                    // Parameters
                    for (i, param) in method.params.iter().enumerate() {
                        write!(
                            sink.body,
                            ", {}: {}",
                            param.name,
                            self.rust_type_name(&param.ty)
                        )?;
                        if i < method.params.len() - 1 {
                            write!(sink.body, ", ")?;
                        }
                    }

                    // Return type
                    if !matches!(method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, " {{\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    write!(
                        sink.body,
                        "// TODO: Implement {} method body from {}\n",
                        method.name, has_decl.name
                    )?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "}}\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n")?;
            }
        }

        // Generate trait implementations for delegations
        for delegation in &type_decl.delegations {
            let spec_name = delegation.spec_name.clone();
            let member_name = delegation.member_name.clone();

            // Get the spec declaration - clone to avoid borrow issues
            let spec_decl_clone = if let Some(meta) = self.lookup_meta(spec_name.as_str()) {
                if let crate::scope::Meta::Spec(spec_decl) = meta.as_ref() {
                    Some(spec_decl.clone())
                } else {
                    None
                }
            } else {
                None
            };

            // Now use the cloned spec_decl
            if let Some(spec_decl) = spec_decl_clone {
                // Build impl signature with generic parameters
                write!(sink.body, "\nimpl {}", spec_decl.name)?;

                // Add generic parameters from spec_decl (trait)
                if !spec_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in spec_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                write!(sink.body, " for {}", type_decl.name)?;

                // Add generic parameters from type_decl (type)
                if !type_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in type_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                // Generate methods that delegate to the member
                for spec_method in &spec_decl.methods {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", spec_method.name)?;

                    // Parameters
                    for param in &spec_method.params {
                        write!(
                            sink.body,
                            ", {}: {}",
                            param.name,
                            self.rust_param_type_name(&param.ty)
                        )?;
                    }

                    // Return type
                    if !matches!(spec_method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_return_type_name(&spec_method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, " {{\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "self.{}.{}(", member_name, spec_method.name)?;

                    // Forward parameters
                    for (i, param) in spec_method.params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        write!(sink.body, "{}", param.name)?;
                    }

                    write!(sink.body, ")\n")?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "}}\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n")?;
            }
        }

        // Generate impl block with own methods (excluding spec methods)
        // Collect spec method names to avoid duplication in impl Type block
        let spec_method_names: HashSet<AutoStr> = type_decl
            .specs
            .iter()
            .filter_map(|s| self.spec_decls.get(s))
            .flat_map(|methods| methods.iter().map(|m| m.name.clone()))
            .collect();

        let own_methods: Vec<_> = type_decl
            .methods
            .iter()
            .filter(|m| !spec_method_names.contains(&m.name))
            .collect();

        if !own_methods.is_empty() {
            sink.body.write(b"\n")?;
            write!(sink.body, "impl {}", type_decl.name)?;

            // Add generic parameters if present
            if !type_decl.generic_params.is_empty() {
                write!(sink.body, "<")?;
                for (i, param) in type_decl.generic_params.iter().enumerate() {
                    if i > 0 {
                        write!(sink.body, ", ")?;
                    }
                    match param {
                        GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                        GenericParam::Const(cp) => {
                            write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                        }
                    }
                }
                write!(sink.body, ">")?;
            }

            writeln!(sink.body, " {{")?;
            self.indent();

            for method in &own_methods {
                self.fn_decl(method, sink)?;
                sink.body.write(b"\n")?;
            }

            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }

        // Reset pub type context
        self.inside_pub_type = false;

        // Generate trait implementations for specs
        if !type_decl.specs.is_empty() {
            // Collect spec declarations: prefer local cache, fallback to database lookup
            let spec_decls: Vec<_> = type_decl
                .specs
                .iter()
                .filter_map(|spec_name| {
                    // Plan 159 6B-2.2: Use cached spec methods first
                    if let Some(methods) = self.spec_decls.get(spec_name) {
                        Some(SpecDecl::new(spec_name.clone(), methods.clone()))
                    } else if let Some(meta) = self.lookup_meta(spec_name) {
                        if let crate::scope::Meta::Spec(spec_decl) = meta.as_ref() {
                            Some(spec_decl.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // Generate impl block for each spec (only if type provides matching methods)
            for spec_decl in spec_decls {
                // Collect matching methods first — skip empty impls
                let matched_methods: Vec<_> = spec_decl
                    .methods
                    .iter()
                    .filter(|spec_method| {
                        type_decl
                            .methods
                            .iter()
                            .any(|m| m.name == spec_method.name)
                    })
                    .collect();

                if matched_methods.is_empty() {
                    continue; // Skip empty impl blocks
                }

                sink.body.write(b"\n")?;

                // Build impl signature with generic parameters
                write!(sink.body, "impl {}", spec_decl.name)?;

                // Add generic parameters from spec_decl (trait)
                if !spec_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in spec_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                write!(sink.body, " for {}", type_decl.name)?;

                // Add generic parameters from type_decl (type)
                if !type_decl.generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in type_decl.generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                // Generate matched methods
                for spec_method in &matched_methods {
                    // Find the implementation in type_decl
                    if let Some(method) = type_decl
                        .methods
                        .iter()
                        .find(|m| m.name == spec_method.name)
                    {
                        self.print_indent(&mut sink.body)?;

                        // Method signature
                        write!(sink.body, "fn {}(&self", method.name)?;

                        // Parameters
                        for param in &method.params {
                            write!(
                                sink.body,
                                ", {}: {}",
                                param.name,
                                self.rust_param_type_name(&param.ty)
                            )?;
                        }

                        // Return type
                        if !matches!(method.ret, Type::Void) {
                            write!(sink.body, ") -> {}", self.rust_return_type_name(&method.ret))?;
                        } else {
                            write!(sink.body, ")")?;
                        }

                        // Generate method body (body() writes its own { })
                        write!(sink.body, " ")?;
                        self.body(&method.body, sink, &method.ret, "")?;
                        writeln!(sink.body)?;
                    }
                }

                self.dedent();
                writeln!(sink.body, "}}")?;
            }
        }

        Ok(())
    }

    // **Phase 6: Generic Programming**
    // Type alias declaration
    fn type_alias_decl(&mut self, type_alias: &TypeAlias, sink: &mut Sink) -> AutoResult<()> {
        // Generate type alias: type List<T> = List<T, Heap>;
        // In Rust: type List<T> = List<T, Heap>;
        write!(sink.body, "type {}", type_alias.name)?;

        // Type parameters
        if !type_alias.params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in type_alias.params.iter().enumerate() {
                write!(sink.body, "{}", param)?;
                if i < type_alias.params.len() - 1 {
                    write!(sink.body, ", ")?;
                }
            }
            write!(sink.body, ">")?;
        }

        // For the target type, if it's a GenericInstance with Unknown args,
        // we need to use the type parameter names instead of "Unknown"
        if let Type::GenericInstance(inst) = &type_alias.target {
            write!(sink.body, " = {}<", inst.base_name)?;
            // Use type parameters if available, otherwise use Unknown count
            let args: Vec<String> = if !type_alias.params.is_empty() {
                type_alias.params.iter().map(|p| p.to_string()).collect()
            } else {
                inst.args
                    .iter()
                    .map(|t| match t {
                        Type::Unknown => "_".to_string(),
                        _ => self.rust_type_name(t),
                    })
                    .collect()
            };
            write!(sink.body, "{}>;", args.join(", "))?;
        } else {
            write!(sink.body, " = {};", self.rust_type_name(&type_alias.target))?;
        }
        sink.body.write(b"\n")?;

        Ok(())
    }

    /// Convert a Heterogeneous EnumDecl to a Tag for reusing tag code generation.
    #[allow(dead_code)]
    fn enum_decl_to_tag(enum_decl: &EnumDecl) -> Tag {
        let fields: Vec<TagField> = enum_decl.items.iter().map(|item| TagField {
            name: item.name.clone().into(),
            ty: item.payload_type.clone().unwrap_or(Type::Void),
        }).collect();
        let (generic_params, methods) = match &enum_decl.kind {
            EnumKind::Heterogeneous { generic_params, methods } => (generic_params.clone(), methods.clone()),
            _ => (vec![], vec![]),
        };
        Tag {
            name: enum_decl.name.clone().into(),
            generic_params,
            fields,
            methods,
        }
    }

    // Enum declaration
    fn enum_decl(&mut self, enum_decl: &EnumDecl, sink: &mut Sink) -> AutoResult<()> {
        // Cache enum name as tag type for construction detection
        self.tag_types.insert(enum_decl.name.clone());

        // Emit doc comments
        if let Some(ref doc) = enum_decl.doc {
            for line in doc.split('\n') {
                write!(sink.body, "/// {}\n", line)?;
            }
        }

        // Plan 204 Phase 2C: Add #[derive(Clone, Debug, PartialEq)] to enums
        // Scalar enums with repr type also need Copy
        let derive_attrs = match &enum_decl.kind {
            EnumKind::Scalar { repr_type: Some(_) } => "#[derive(Clone, Debug, PartialEq, Copy)]",
            EnumKind::Scalar { repr_type: None } => "#[derive(Clone, Debug, PartialEq)]",
            _ => "#[derive(Clone, Debug, PartialEq)]",
        };
        writeln!(sink.body, "{}", derive_attrs)?;

        // Plan 163: Output pub prefix
        if enum_decl.is_pub {
            sink.body.write(b"pub ")?;
        }
        self.inside_pub_type = enum_decl.is_pub;

        match &enum_decl.kind {
            EnumKind::Scalar { .. } => {
                // C-style scalar enum: emit Rust enum with values + Display impl
                sink.body.write(b"enum ")?;
                sink.body.write(enum_decl.name.as_bytes())?;
                sink.body.write(b" {\n")?;
                self.indent();

                for (_i, item) in enum_decl.items.iter().enumerate() {
                    self.print_indent(&mut sink.body)?;
                    sink.body
                        .write(format!("{} = {},", item.name, item.value()).as_bytes())?;
                    sink.body.write(b"\n")?;
                }

                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}\n")?;

                // Generate Display trait implementation
                sink.body.write(b"\n")?;
                writeln!(
                    sink.body,
                    "impl std::fmt::Display for {} {{",
                    enum_decl.name
                )?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                writeln!(
                    sink.body,
                    "fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{"
                )?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "match self {{")?;
                self.indent();

                for item in &enum_decl.items {
                    self.print_indent(&mut sink.body)?;
                    writeln!(
                        sink.body,
                        "{}::{} => write!(f, \"{}\"),",
                        enum_decl.name, item.name, item.name
                    )?;
                }

                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                self.dedent();
                writeln!(sink.body, "}}")?;
            }
            EnumKind::Homogeneous { payload_type } => {
                // Generate Rust enum where all variants wrap the same type
                write!(sink.body, "enum {}", enum_decl.name)?;
                writeln!(sink.body, " {{")?;
                self.indent();
                for item in &enum_decl.items {
                    self.print_indent(&mut sink.body)?;
                    writeln!(sink.body, "{}({}),", item.name, self.rust_type_name(payload_type))?;
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                sink.body.write(b"\n")?;
            }
            EnumKind::Heterogeneous { generic_params, .. } => {
                // Plan 204 Phase 2C: Generate heterogeneous enum directly
                // Supports both single-payload tuple variants and multi-field struct variants
                write!(sink.body, "enum {}", enum_decl.name)?;

                // Add generic parameters if present
                if !generic_params.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, param) in generic_params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        match param {
                            GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                            GenericParam::Const(cp) => {
                                write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                            }
                        }
                    }
                    write!(sink.body, ">")?;
                }

                writeln!(sink.body, " {{")?;
                self.indent();

                for item in &enum_decl.items {
                    self.print_indent(&mut sink.body)?;
                    if item.has_fields() {
                        // Register struct variant for pattern matching
                        let field_names: Vec<AutoStr> = item.fields.iter()
                            .map(|f| f.name.clone())
                            .collect();
                        self.enum_struct_variants.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            field_names,
                        );
                        // Multi-field struct variant: Name { field1: Type1, field2: Type2 }
                        write!(sink.body, "{} {{ ", item.name)?;
                        for (j, field) in item.fields.iter().enumerate() {
                            if j > 0 {
                                write!(sink.body, ", ")?;
                            }
                            write!(sink.body, "{}: {}", field.name, self.rust_type_name(&field.field_type))?;
                        }
                        writeln!(sink.body, " }},")?;
                    } else if item.has_tuple_payload() {
                        // Register tuple variant for bare-match detection
                        self.enum_tuple_variants.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            item.payload_types.len(),
                        );
                        // Cache tuple variant field types for .to_string() auto-insertion
                        self.enum_tuple_field_types.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            item.payload_types.clone(),
                        );
                        // Multi-arg tuple variant: ToolUse str str str → ToolUse(String, String, String)
                        write!(sink.body, "{}(", item.name)?;
                        for (j, pt) in item.payload_types.iter().enumerate() {
                            if j > 0 {
                                write!(sink.body, ", ")?;
                            }
                            write!(sink.body, "{}", self.rust_type_name(pt))?;
                        }
                        writeln!(sink.body, "),")?;
                    } else if let Some(ref payload) = item.payload_type {
                        // Register single-payload tuple variant
                        self.enum_tuple_variants.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            1,
                        );
                        // Cache single-payload type for .to_string() auto-insertion
                        self.enum_tuple_field_types.insert(
                            (enum_decl.name.clone(), item.name.clone()),
                            vec![payload.clone()],
                        );
                        // Single-payload tuple variant: Name(Type)
                        writeln!(sink.body, "{}({}),", item.name, self.rust_type_name(payload))?;
                    } else {
                        // Unit variant (no data): Name
                        writeln!(sink.body, "{},", item.name)?;
                    }
                }

                self.dedent();
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "}}")?;
                sink.body.write(b"\n")?;
            }
        }

        self.inside_pub_type = false;
        Ok(())
    }

    // **Phase 1.2: Union Types (test: 013_union)**
    fn union_decl(&mut self, union: &Union, sink: &mut Sink) -> AutoResult<()> {
        // Generate union definition
        // In Rust, unions are unsafe but supported
        writeln!(sink.body, "union {} {{", union.name)?;
        self.indent();

        for field in &union.fields {
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "{}: {},",
                field.name,
                self.rust_type_name(&field.ty)
            )?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;

        Ok(())
    }

    // **Phase 1.3: Tag Types (test: 014_tag)**
    fn tag_decl(&mut self, tag: &Tag, sink: &mut Sink) -> AutoResult<()> {
        // Cache tag type name for tag construction detection
        self.tag_types.insert(tag.name.clone());

        // Generate enum definition for tag
        // AutoLang tags are algebraic data types that map to Rust enums
        write!(sink.body, "enum {}", tag.name)?;

        // Add generic parameters if present
        if !tag.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in tag.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        writeln!(sink.body, " {{")?;
        self.indent();

        for field in &tag.fields {
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "{}({}),",
                field.name,
                self.rust_type_name(&field.ty)
            )?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;
        sink.body.write(b"\n")?;

        // TODO: Generate impl block for tag methods (if any)
        for method in &tag.methods {
            // Tag methods will be added here
            let _ = method;
        }

        Ok(())
    }

    // Ext block (type extension) - transpiles to impl block
    fn ext_decl(&mut self, ext: &Ext, sink: &mut Sink) -> AutoResult<()> {
        // Plan 164: Support "ext Type for Trait" → impl Trait for Type
        // Plan 6B-2.7: Support generic args on trait: ext Type for From<String> → impl From<String> for Type
        match &ext.trait_name {
            Some(trait_name) => {
                write!(sink.body, "impl {}", trait_name)?;
                if !ext.trait_generic_args.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, arg) in ext.trait_generic_args.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        write!(sink.body, "{}", self.rust_type_name(arg))?;
                    }
                    write!(sink.body, ">")?;
                }
                write!(sink.body, " for {}", ext.target)?;
            }
            None => {
                write!(sink.body, "impl {}", ext.target)?;
            }
        }

        // Add generic parameters if present
        if !ext.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in ext.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        writeln!(sink.body, " {{")?;
        self.indent();

        // Generate methods
        for method in &ext.methods {
            self.fn_decl(method, sink)?;
            sink.body.write(b"\n")?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n")?;

        Ok(())
    }

    // Spec/trait declaration
    // Plan 204 Phase 4: spec → Rust trait mapping
    fn spec_decl(&mut self, spec_decl: &SpecDecl, sink: &mut Sink) -> AutoResult<()> {
        // Cache spec methods for later use in impl Trait for Type
        self.spec_decls.insert(spec_decl.name.clone(), spec_decl.methods.clone());

        // Plan 163: Output pub prefix
        if spec_decl.is_pub {
            write!(sink.body, "pub ")?;
        }

        // Generate trait definition with generic parameters
        write!(sink.body, "trait {}", spec_decl.name)?;

        // Add generic parameters if present
        if !spec_decl.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in spec_decl.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => write!(sink.body, "{}", tp.name)?,
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        writeln!(sink.body, " {{")?;
        self.indent();

        for method in &spec_decl.methods {
            self.print_indent(&mut sink.body)?;
            write!(sink.body, "fn {}(&self", method.name)?;

            // Parameters (skip self which is already added as &self)
            for param in &method.params {
                write!(
                    sink.body,
                    ", {}: {}",
                    param.name,
                    self.rust_param_type_name(&param.ty)
                )?;
            }

            // Return type — use rust_return_type_name for correct str→String mapping
            // Plan 204 Phase 4: !T (Type::Result) → Result<T, String>
            if !matches!(method.ret, Type::Void) {
                write!(sink.body, ") -> {}", self.rust_return_type_name(&method.ret))?;
            } else {
                write!(sink.body, ")")?;
            }

            // Default method implementation (Plan 019 Stage 8.5)
            if let Some(ref default_body) = method.body {
                // SpecMethod.body is Option<Box<Expr>>, emit as { expr }
                sink.body.write(b" {\n")?;
                self.indent();
                self.print_indent(&mut sink.body)?;
                self.expr(default_body, &mut sink.body)?;
                sink.body.write(b"\n")?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}\n")?;
            } else {
                writeln!(sink.body, ";")?;
            }
        }

        self.dedent();
        writeln!(sink.body, "}}\n")?;

        Ok(())
    }

    // Body and block management
    fn body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
        ret_type: &Type,
        _insert: &str,
    ) -> AutoResult<()> {
        let has_return = !matches!(ret_type, Type::Void);

        sink.body.write(b"{\n")?;
        self.indent();

        // Process statements
        for (i, stmt) in body.stmts.iter().enumerate() {
            // Set source line for mapping
            if i < body.source_lines.len() {
                sink.set_source_line(body.source_lines[i]);
            }
            if !matches!(stmt, Stmt::EmptyLine(_)) {
                self.print_indent(&mut sink.body)?;
            }

            let is_last = i == body.stmts.len() - 1;

            if is_last && has_return && self.is_returnable(stmt) {
                // Last statement in a non-void function: expression position (no semicolon)
                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        // If return type is String and expr produces &str, add .to_string()
                        if self.ret_type_needs_string_coercion()
                            && self.expr_needs_string_coercion(expr)
                        {
                            sink.body.write(b".to_string()")?;
                        }
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Node(node) => {
                        // Node (struct constructor) as tail expression — no semicolon
                        self.expr(&Expr::Node(node.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n")?;
                    }
                }
            } else {
                // Regular statement: add semicolon if needed
                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::EmptyLine(n) => {
                        for _ in 0..*n {
                            sink.body.write(b"\n")?;
                        }
                    }
                    Stmt::Break => {
                        sink.body.write(b"break;\n")?;
                    }
                    _ => {
                        // For other statement types that handle their own formatting
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n")?;
                    }
                }
            }
        }

        // For Result-returning functions, append Ok(()) if the last
        // statement is not a tail expression (e.g., ends with a semicolon)
        if matches!(ret_type, Type::Result(_)) && !body.stmts.is_empty() {
            let last = &body.stmts[body.stmts.len() - 1];
            if !self.is_returnable(last) {
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"Ok(())\n")?;
            }
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        Ok(())
    }

    fn is_returnable(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => {
                match expr {
                    // Void function calls are not returnable
                    Expr::Call(call) => {
                        if let Expr::Ident(name) = call.name.as_ref() {
                            if name == "print" || name == "println" || name == "write" {
                                return false;
                            }
                        }
                        true
                    }
                    // Nil/Null are not valid return expressions
                    Expr::Nil | Expr::Null => false,
                    // All other expressions (literals, operators, etc.) are returnable
                    _ => true,
                }
            }
            // Node (struct constructor parsed as component) is returnable
            Stmt::Node(_) => true,
            // Is (match expression) is returnable
            Stmt::Is(_) => true,
            _ => false,
        }
    }

    /// Incremental transpilation (Phase 066)
    /// Plan 163: Check if statements contain any await expression
    fn has_await(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_await(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    if Self::has_await(&body.stmts) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::has_await(&branch.body.stmts) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::has_await(&else_body.stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Like has_await but takes a slice of references (for use with split stmts)
    fn has_await_refs(stmts: &[&Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_await(&store.expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Block(body) => {
                    let refs: Vec<&Stmt> = body.stmts.iter().collect();
                    if Self::has_await_refs(&refs) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        let refs: Vec<&Stmt> = branch.body.stmts.iter().collect();
                        if Self::has_await_refs(&refs) {
                            return true;
                        }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        let refs: Vec<&Stmt> = else_body.stmts.iter().collect();
                        if Self::has_await_refs(&refs) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 240: Check if statements contain ErrorPropagate (`.?` operator)
    fn has_error_propagate(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_error_propagate(expr) { return true; }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_error_propagate(&store.expr) { return true; }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_error_propagate(expr) { return true; }
                }
                Stmt::Block(body) => {
                    if Self::has_error_propagate(&body.stmts) { return true; }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::has_error_propagate(&branch.body.stmts) { return true; }
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        if Self::has_error_propagate(&else_body.stmts) { return true; }
                    }
                }
                Stmt::For(for_stmt) => {
                    if Self::has_error_propagate(&for_stmt.body.stmts) { return true; }
                }
                Stmt::Is(is_stmt) => {
                    for branch in &is_stmt.branches {
                        let body = match branch {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        if Self::has_error_propagate(&body.stmts) { return true; }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Plan 240: Check if an expression contains ErrorPropagate (`.?` operator)
    fn expr_has_error_propagate(expr: &Expr) -> bool {
        match expr {
            Expr::ErrorPropagate(_) => true,
            Expr::Call(call) => {
                if Self::expr_has_error_propagate(call.name.as_ref()) { return true; }
                for arg in &call.args.args {
                    match arg {
                        Arg::Pos(e) | Arg::Pair(_, e) => {
                            if Self::expr_has_error_propagate(e) { return true; }
                        }
                        Arg::Name(_) => {}
                    }
                }
                false
            }
            Expr::Block(body) => Self::has_error_propagate(&body.stmts),
            Expr::Bina(left, _, right) => {
                Self::expr_has_error_propagate(left) || Self::expr_has_error_propagate(right)
            }
            Expr::Unary(_, e) => Self::expr_has_error_propagate(e),
            Expr::Dot(obj, _) => Self::expr_has_error_propagate(obj),
            Expr::Index(arr, idx) => {
                Self::expr_has_error_propagate(arr) || Self::expr_has_error_propagate(idx)
            }
            Expr::View(e) | Expr::Mut(e) | Expr::Move(e) | Expr::Take(e) => {
                Self::expr_has_error_propagate(e)
            }
            Expr::FStr(fstr) => fstr.parts.iter().any(|p| Self::expr_has_error_propagate(p)),
            Expr::Array(arr) => arr.iter().any(|e| Self::expr_has_error_propagate(e)),
            _ => false,
        }
    }

    /// Plan 220 Task 4: Check if an expression needs an `as usize` cast
    /// when used as a slice/array index in Rust.
    ///
    /// Integer literals do NOT need a cast -- Rust infers the correct type
    /// automatically in index position (e.g., `arr[0]` just works).
    /// Non-trivial expressions (variables, binary ops, calls) may be u32/i32
    /// and need explicit `as usize` for Rust indexing and range bounds.
    fn needs_usize_cast(expr: &Expr) -> bool {
        match expr {
            // Integer literals: Rust infers correct type in index position
            Expr::Int(_) | Expr::Uint(_) | Expr::I8(_) | Expr::U8(_)
            | Expr::I64(_) | Expr::U64(_) | Expr::Byte(_) => false,
            // Range: bounds are handled individually, not the range itself
            Expr::Range(_) => false,
            // Non-integer literals: not used as indices
            Expr::Bool(_) | Expr::Nil | Expr::Null => false,
            // Variables, binary ops, calls, dot access, etc. may be u32/i32
            _ => true,
        }
    }

    /// Check if an expression likely produces a Debug-only type (no Display impl).
    /// Detects patterns like `.elapsed()`, `Instant::now()`, and variables named
    /// duration/elapsed/instant.
    fn needs_debug_format(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => {
                let lower = name.as_str().to_lowercase();
                if lower.contains("duration") || lower.contains("elapsed") || lower.contains("instant")
                    || lower.contains("vec") || lower.contains("list") || lower.contains("map")
                    || lower.contains("set") || lower.contains("hashmap") || lower.contains("entry")
                    || lower.contains("result") || lower.contains("option") || lower.contains("dir_entry")
                    || lower.contains("people") || lower.contains("numbers") || lower.contains("items")
                    || lower.contains("now") || lower.contains("future") || lower.contains("past")
                    || lower == "value" || lower == "exists" || lower == "has_none" || lower == "has_value"
                    || lower == "numbers" || lower == "count" || lower == "avg"
                {
                    return true;
                }
                // Check local_var_types for non-Display types
                if let Some(ty) = self.local_var_types.get(name) {
                    return matches!(ty,
                        Type::List(_) | Type::Map(_, _) | Type::Array(_)
                        | Type::RuntimeArray(_) | Type::Slice(_)
                        | Type::Option(_) | Type::Result(_)
                        | Type::Tuple(_) | Type::Tag(_) | Type::Enum(_)
                    );
                }
                false
            }
            Expr::Dot(obj, method) => {
                method == "elapsed" || self.needs_debug_format(obj)
            }
            Expr::Bina(lhs, op, rhs) => {
                if matches!(op, Op::Dot) {
                    // Check for expr.elapsed()
                    if let Expr::Ident(m) = rhs.as_ref() {
                        if m.as_str() == "elapsed" { return true; }
                    }
                    self.needs_debug_format(lhs)
                } else {
                    self.needs_debug_format(lhs) || self.needs_debug_format(rhs)
                }
            }
            Expr::Call(call) => self.needs_debug_format(&call.name),
            Expr::ErrorPropagate(inner) => self.needs_debug_format(inner),
            _ => false,
        }
    }

    fn is_self_dot(expr: &Expr) -> bool {
        matches!(expr, Expr::Dot(obj, _) if matches!(obj.as_ref(), Expr::Ident(name) if name == "self"))
    }

    /// Plan 163: Check if an expression contains await
    fn expr_has_await(expr: &Expr) -> bool {
        match expr {
            Expr::Await { .. } => true,
            Expr::Call(call) => {
                if Self::expr_has_await(call.name.as_ref()) {
                    return true;
                }
                for arg in &call.args.args {
                    match arg {
                        Arg::Pos(e) | Arg::Pair(_, e) => {
                            if Self::expr_has_await(e) {
                                return true;
                            }
                        }
                        Arg::Name(_) => {}
                    }
                }
                false
            }
            Expr::Block(body) => Self::has_await(&body.stmts),
            Expr::Bina(left, _, right) => {
                Self::expr_has_await(left) || Self::expr_has_await(right)
            }
            Expr::Unary(_, expr) => Self::expr_has_await(expr),
            Expr::Dot(obj, _) => Self::expr_has_await(obj),
            Expr::Index(arr, idx) => {
                Self::expr_has_await(arr) || Self::expr_has_await(idx)
            }
            Expr::View(e) | Expr::Mut(e) | Expr::Move(e) | Expr::Take(e) => {
                Self::expr_has_await(e)
            }
            Expr::AsyncBlock { body, .. } => Self::has_await(&body.stmts),
            Expr::Cast { expr, .. } | Expr::To { expr, .. } => Self::expr_has_await(expr),
            Expr::NullCoalesce(l, r) => {
                Self::expr_has_await(l) || Self::expr_has_await(r)
            }
            Expr::ErrorPropagate(e) => {
                Self::expr_has_await(e)
            }
            _ => false,
        }
    }

    /// Only transpiles dirty fragments, caches results in Database
    pub fn trans_incremental(
        &mut self,
        session: &mut crate::compile::CompileSession,
        file_id: crate::database::FileId,
    ) -> AutoResult<std::collections::HashMap<crate::database::FragId, String>> {
        use std::collections::HashMap;

        let db = session.db();

        // Get dirty fragments for the file
        let dirty_frags = {
            let db_read = db.read().unwrap();
            let all_frags = db_read.get_fragments_by_file(file_id);
            all_frags
                .into_iter()
                .filter(|frag| db_read.is_fragment_dirty(frag))
                .collect::<Vec<_>>()
        };

        let mut results = HashMap::new();

        for frag_id in dirty_frags {
            let frag_ast = {
                let db_read = db.read().unwrap();
                db_read.get_fragment(&frag_id)
            };

            if let Some(fn_ast) = frag_ast {
                // Transpile the function
                let mut sink = Sink::new(AutoStr::from(format!("{:?}", frag_id)));
                self.fn_decl(&fn_ast, &mut sink)?;
                let output = String::from_utf8(sink.done()?.to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;

                results.insert(frag_id.clone(), output);

                // Mark as transpiled
                db.write().unwrap().mark_transpiled(&frag_id);
            }
        }

        Ok(results)
    }
}

impl Trans for RustTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Phase 1: Emit file header with a2r standard library
        sink.body
            .write(b"// Auto-generated by a2r transpiler\n\n")?;

        // Emit a2r standard library (List, May, etc.)
        self.emit_a2r_stdlib(&mut sink.body)?;

        // Plan 204 Phase 3: Pre-scan for !T / Result<T,E> return types to determine Err trait need
        for stmt in &ast.stmts {
            if let Stmt::Fn(fn_decl) = stmt {
                if matches!(fn_decl.ret, Type::Result(_))
                    || matches!(&fn_decl.ret, Type::GenericInstance(inst) if inst.base_name == "Result")
                {
                    self.needs_err_trait = true;
                    break;
                }
            }
        }

        // Pre-scan all function signatures for auto-borrow/auto-clone at call sites
        // Without this, functions declared after their callers won't have param type info
        for stmt in &ast.stmts {
            match stmt {
                Stmt::Fn(fn_decl) => {
                    let str_param_flags: Vec<bool> = fn_decl.params.iter()
                        .map(|p| matches!(p.ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit))
                        .collect();
                    self.fn_str_param_indices.insert(fn_decl.name.clone(), str_param_flags);

                    let struct_param_flags: Vec<bool> = fn_decl.params.iter()
                        .map(|p| matches!(p.ty, Type::User(_) | Type::Tag(_) | Type::Enum(_)))
                        .collect();
                    self.fn_struct_param_indices.insert(fn_decl.name.clone(), struct_param_flags);
                }
                Stmt::TypeDecl(type_decl) => {
                    // Also scan methods inside type declarations
                    let type_name = &type_decl.name;
                    for fn_decl in &type_decl.methods {
                        let str_param_flags: Vec<bool> = fn_decl.params.iter()
                            .map(|p| matches!(p.ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit))
                            .collect();
                        // Use qualified key "Type.method" to avoid cross-type overwrites
                        let qualified_key: AutoStr = format!("{}.{}", type_name, fn_decl.name).into();
                        self.fn_str_param_indices.insert(qualified_key.clone(), str_param_flags.clone());
                        // Also store unqualified for backward compat (last one wins)
                        self.fn_str_param_indices.insert(fn_decl.name.clone(), str_param_flags);

                        let struct_param_flags: Vec<bool> = fn_decl.params.iter()
                            .map(|p| matches!(p.ty, Type::User(_) | Type::Tag(_) | Type::Enum(_)))
                            .collect();
                        self.fn_struct_param_indices.insert(qualified_key, struct_param_flags.clone());
                        self.fn_struct_param_indices.insert(fn_decl.name.clone(), struct_param_flags);
                    }
                }
                _ => {}
            }
        }

        // No custom Err trait — use Box<dyn std::error::Error> for !T error types

        // Phase 2: Split into declarations and main, preserving source line info
        let mut decls: Vec<(Stmt, usize)> = Vec::new(); // (stmt, source_line)
        let mut main: Vec<(Stmt, usize)> = Vec::new();  // (stmt, source_line)

        let source_lines = ast.source_lines;
        for (i, stmt) in ast.stmts.into_iter().enumerate() {
            let line = source_lines.get(i).copied().unwrap_or(0);
            // Plan 151 / Fix: top-level let must go into main(), not module scope
            if let Stmt::Store(store) = &stmt {
                if matches!(store.kind, StoreKind::Var)
                    || matches!(store.kind, StoreKind::Shared)
                    || matches!(store.kind, StoreKind::Const)
                {
                    if matches!(store.kind, StoreKind::Var) || matches!(store.kind, StoreKind::Shared) {
                        self.register_global_var(store.name.clone());
                    }
                    decls.push((stmt, line));
                } else {
                    // let → goes into main()
                    main.push((stmt, line));
                }
            } else if stmt.is_decl() {
                decls.push((stmt, line));
            } else {
                match stmt {
                    Stmt::For(_) => main.push((stmt, line)),
                    Stmt::If(_) => main.push((stmt, line)),
                    Stmt::Expr(_) => main.push((stmt, line)),
                    Stmt::Break => main.push((stmt, line)),
                    Stmt::Use(use_stmt) => {
                        sink.set_source_line(line);
                        self.use_stmt(&use_stmt, &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Dep(dep) => {
                        // Record dep name so crate.func() → crate::func()
                        // Use separate set to avoid blocking use.rust import generation
                        self.dep_crates.insert(dep.name.clone());
                    }
                    _ => {}
                }
            }
        }

        // Plan 151: Add once_cell imports if we have global variables
        if !self.global_vars.is_empty() {
            sink.body.write(b"use once_cell::sync::Lazy;\n")?;
            sink.body.write(b"use std::sync::Mutex;\n\n")?;
        }

        // Phase 3: Generate declarations
        for (i, (decl, line)) in decls.iter().enumerate() {
            sink.set_source_line(*line);
            self.stmt(decl, sink)?;
            if i < decls.len() - 1 {
                // Add blank line between declarations
                // Check if we already end with a newline
                if sink.body.ends_with(b"\n") {
                    sink.body.write(b"\n")?;
                } else {
                    sink.body.write(b"\n\n")?;
                }
            }
        }

        // Phase 4: Generate main function if needed
        if !main.is_empty() {
            if !decls.is_empty() {
                // Add blank line before main
                if sink.body.ends_with(b"\n") {
                    sink.body.write(b"\n")?;
                } else {
                    sink.body.write(b"\n\n")?;
                }
            }

            // Plan 163: Check for async (await) and generate #[tokio::main] if needed
            // Collect references for has_await check
            let is_async = {
                let refs: Vec<&Stmt> = main.iter().map(|(s, _)| s).collect();
                Self::has_await_refs(&refs)
            };
            if is_async {
                sink.body.write(b"#[tokio::main]\n")?;
                sink.body.write(b"async fn main() {\n")?;
            } else {
                sink.body.write(b"fn main() {\n")?;
            }
            self.indent();

            for (stmt, line) in main.iter() {
                sink.set_source_line(*line);
                self.print_indent(&mut sink.body)?;

                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        match stmt {
                            Stmt::Store(_) => {
                                sink.body.write(b";\n")?;
                            }
                            _ => {}
                        }
                    }
                }
            }

            self.dedent();
            sink.body.write(b"}\n")?;
        }

        // Add final newline only if not already ending with one
        if !sink.body.is_empty() && !sink.body.ends_with(b"\n") {
            sink.body.write(b"\n")?;
        }

        Ok(())
    }
}

/// Transpile AutoLang code to Rust
pub fn transpile_rust(name: impl Into<AutoStr>, code: &str) -> AutoResult<Sink> {
    let name = name.into();
    let _scope = shared(crate::scope_manager::ScopeManager::new());
    let mut parser = Parser::from(code);
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let mut ast = parser.parse().map_err(|e| e.to_string())?;

    // Plan 095: Run CTEE to transform compile-time constructs
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast).map_err(|e| e.to_string())?;

    let mut out = Sink::new(name.clone());
    let mut transpiler = RustTrans::new(name);
    transpiler.trans(ast, &mut out)?;

    Ok(out)
}

/// Transpile code fragment for testing
pub fn transpile_part(code: &str) -> AutoResult<AutoStr> {
    let _scope = shared(crate::scope_manager::ScopeManager::new());
    let mut parser = Parser::from(code);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new(AutoStr::from(""));
    let mut transpiler = RustTrans::new("part".into());
    transpiler.trans(ast, &mut out)?;
    let src = out.done()?.clone();
    Ok(String::from_utf8(src).unwrap().into())
}

// =============================================================================
// Plan 167: Multi-file project transpilation
// =============================================================================

/// A module discovered during project scanning
#[allow(dead_code)]
struct ProjectModule {
    /// Module name (e.g., "db", "api", "api::handlers")
    name: String,
    /// Path to the .at source file
    source_path: std::path::PathBuf,
    /// Rust output file name (e.g., "db.rs", "api/mod.rs", "api/handlers.rs")
    output_name: String,
    /// Whether this is a directory module (mod.at)
    is_dir_module: bool,
    /// Import statements from this module
    uses: Vec<crate::ast::Use>,
}

/// Transpile a multi-file AutoLang project to Rust
///
/// Starting from an entry file, this function:
/// 1. Parses the entry file and discovers its module dependencies
/// 2. Recursively discovers and parses all module files
/// 3. Transpiles each module into its own .rs file
/// 4. Generates mod.rs from mod.at with pub mod declarations
///
/// Returns a HashMap mapping output filename to generated Rust code.
pub fn transpile_rust_project(entry_file: &str) -> AutoResult<std::collections::HashMap<String, Vec<u8>>> {
    use super::MultiSink;
    use crate::ast::Stmt;

    let entry_path = std::path::Path::new(entry_file);
    let entry_dir = entry_path.parent()
        .ok_or_else(|| AutoError::Msg("Entry file has no parent directory".into()))?;

    // Phase 1: Discover all modules
    let mut modules = Vec::new();
    let mut visited = std::collections::HashSet::new();
    discover_modules(entry_path, entry_dir, &mut modules, &mut visited)?;

    // Phase 1.5: Pre-register all type/enum declarations into shared TypeStore
    // This allows cross-file type references (e.g., Usage{...} in json_helpers.at
    // when Usage is defined in types.at) to be resolved during parsing.
    let shared_type_store = Arc::new(RwLock::new(TypeStore::new()));
    {
        let mut store = shared_type_store.write().unwrap();
        for module in &modules {
            let source = std::fs::read_to_string(&module.source_path)
                .map_err(|e| AutoError::Msg(format!("Failed to read {}: {}", module.source_path.display(), e)))?;
            for line in source.lines() {
                let trimmed = line.trim();
                let (prefix, rest) = if trimmed.starts_with("pub type ") {
                    ("pub type ", trimmed)
                } else if trimmed.starts_with("pub enum ") {
                    ("pub enum ", trimmed)
                } else if trimmed.starts_with("type ") {
                    ("type ", trimmed)
                } else if trimmed.starts_with("enum ") {
                    ("enum ", trimmed)
                } else {
                    continue;
                };
                let after_prefix = rest;
                // Extract name (first token after prefix, possibly with generics)
                let name = if let Some(angle) = after_prefix.find('<') {
                    &after_prefix[..angle]
                } else if let Some(space) = after_prefix.find(' ') {
                    &after_prefix[..space]
                } else {
                    after_prefix
                };
                if name.is_empty() {
                    continue;
                }
                // Type names must start with uppercase
                if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    continue;
                }
                if prefix.contains("type ") {
                    let decl = TypeDecl::builtin(name);
                    store.register_type_decl(&decl);
                } else if prefix.contains("enum ") {
                    let enum_decl = EnumDecl {
                        name: name.into(),
                        items: Vec::new(),
                        kind: EnumKind::Heterogeneous {
                            generic_params: Vec::new(),
                            methods: Vec::new(),
                        },
                        doc: None,
                        is_pub: prefix.starts_with("pub"),
                    };
                    store.register_enum_decl(enum_decl);
                }
            }
        }
    }

    // Phase 2: Parse each module
    let mut parsed_modules = Vec::new();
    for module in &modules {
        let source = std::fs::read_to_string(&module.source_path)
            .map_err(|e| AutoError::Msg(format!("Failed to read {}: {}", module.source_path.display(), e)))?;
        let _scope = shared(crate::scope_manager::ScopeManager::new());
        let mut parser = Parser::new_with_type_store(source.as_str(), shared_type_store.clone());
        parser.set_dest(crate::parser::CompileDest::TransRust);
        parser.skip_check = true; // Plan 167: skip type checking for multi-file mode
        let ast = parser.parse().map_err(|e| {
            AutoError::Msg(format!("Parse error in {}: {}", module.source_path.display(), e.to_string()))
        })?;
        parsed_modules.push((module, ast));
    }

    // Phase 3: Transpile each module into its own Sink
    let mut multi_sink = MultiSink::new();
    for (module, ast) in &parsed_modules {
        let sink = multi_sink.add(&module.output_name);
        let mut transpiler = RustTrans::new(AutoStr::from(&module.output_name));

        // Plan 167: Populate local_modules for mod declarations.
        // In Rust, `mod X;` can only appear in the parent module that owns X.
        // - crate root (main.rs): can use `mod X;` for all top-level modules
        // - dir module (mod.rs): pub mod X; emitted separately below
        // - other files: must use `use crate::X;` or `use super::X;`
        // We only populate local_modules for the crate root.
        let is_entry = module.source_path == modules[0].source_path;
        if is_entry {
            for other in &modules {
                if other.source_path == module.source_path {
                    continue;
                }
                let other_name = if other.is_dir_module {
                    other.source_path.parent().unwrap()
                        .file_name().unwrap().to_string_lossy().to_string()
                } else {
                    other.source_path.file_stem()
                        .unwrap().to_string_lossy().to_string()
                };
                transpiler.local_modules.insert(other_name);
            }
        }
        // Non-entry modules: local_modules stays empty
        // → use X will be handled by is_multi_file_bare → use crate::X;

        // Mark directory modules and populate dir_children
        if module.is_dir_module {
            transpiler.is_dir_module = true;
            let mod_dir = module.source_path.parent().unwrap();
            for other in &modules {
                if other.source_path == module.source_path || other.is_dir_module {
                    continue;
                }
                let other_dir = other.source_path.parent().unwrap();
                if other_dir == mod_dir {
                    let other_name = other.source_path.file_stem()
                        .unwrap().to_string_lossy().to_string();
                    transpiler.dir_children.insert(other_name);
                }
            }
        }

        // Populate sibling_modules: modules in the same directory as the current module
        // Used to generate `use super::X;` for same-directory references.
        // Exclude directory modules (mod.rs) since their same-dir files are children, not siblings.
        if !is_entry && !module.is_dir_module {
            let module_dir = module.source_path.parent().unwrap();
            for other in &modules {
                if other.source_path == module.source_path {
                    continue;
                }
                let other_dir = other.source_path.parent().unwrap();
                if other_dir == module_dir {
                    let other_name = other.source_path.file_stem()
                        .unwrap().to_string_lossy().to_string();
                    transpiler.sibling_modules.insert(other_name);
                }
            }
        }

        // For directory modules (mod.at), emit pub mod declarations for discovered sibling files
        // Only emit for files that were actually discovered by discover_modules (not all .at files on disk)
        if module.is_dir_module {
            let mod_dir = module.source_path.parent().unwrap();
            let mut submodules: Vec<String> = Vec::new();
            for other in &modules {
                if other.source_path == module.source_path {
                    continue;
                }
                let other_dir = other.source_path.parent().unwrap();
                if other_dir == mod_dir && !other.is_dir_module {
                    if let Some(name) = other.source_path.file_stem() {
                        submodules.push(name.to_string_lossy().to_string());
                    }
                }
            }
            submodules.sort();
            for sub in &submodules {
                let _ = write!(sink.body, "pub mod {};\n", sub);
            }
        }

        transpiler.trans(ast.clone(), sink)?;
    }

    // Phase 3.5: Generate Cargo.toml
    let mut result = std::collections::HashMap::new();
    {
        let project_name = entry_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("auto_project");
        // Sanitize: replace non-alphanumeric with underscore
        let project_name = project_name.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .to_lowercase();

        let mut cargo_toml = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            project_name
        );

        // Scan parsed ASTs for external Rust crate imports (.rust use kind)
        let mut deps: Vec<String> = Vec::new();
        // Plan 190: Rust built-in crates are always available, don't add to Cargo.toml
        let built_in_crates = ["std", "core", "alloc", "proc_macro"];
        for (_, ast) in &parsed_modules {
            for stmt in &ast.stmts {
                if let Stmt::Use(u) = stmt {
                    if matches!(u.kind, UseKind::Rust) && !u.paths.is_empty() {
                        let crate_name = u.paths[0].as_str();
                        if !deps.contains(&crate_name.to_string())
                            && !built_in_crates.contains(&crate_name) {
                            deps.push(crate_name.to_string());
                        }
                    }
                }
            }
        }
        if !deps.is_empty() {
            cargo_toml.push_str("\n[dependencies]\n");
            for dep in &deps {
                cargo_toml.push_str(&format!("{} = \"*\"\n", dep));
            }
        }

        result.insert("Cargo.toml".to_string(), cargo_toml.into_bytes());
    }

    // Phase 4: Collect results
    let files = multi_sink.done();
    for (name, content) in files {
        result.insert(name, content);
    }

    Ok(result)
}

/// Recursively discover all modules starting from an entry file
fn discover_modules(
    file_path: &std::path::Path,
    base_dir: &std::path::Path,
    modules: &mut Vec<ProjectModule>,
    visited: &mut std::collections::HashSet<String>,
) -> AutoResult<()> {
    let canonical = file_path.canonicalize()
        .map_err(|e| AutoError::Msg(format!("Cannot canonicalize {}: {}", file_path.display(), e)))?;
    let key = canonical.to_string_lossy().to_string();

    if visited.contains(&key) {
        return Ok(());
    }
    visited.insert(key);

    let file_name = file_path.file_stem()
        .ok_or_else(|| AutoError::Msg("File has no stem".into()))?
        .to_string_lossy()
        .to_string();

    let is_dir_module = file_name == "mod";

    // Determine output path relative to base_dir
    let rel_path = file_path.parent()
        .and_then(|p| p.strip_prefix(base_dir).ok())
        .unwrap_or(std::path::Path::new(""));
    let output_name = if rel_path.as_os_str().is_empty() {
        format!("{}.rs", file_name)
    } else {
        format!("{}/{}.rs", rel_path.display(), file_name)
    };

    // Read and parse the file to discover its use statements
    let source = std::fs::read_to_string(file_path)
        .map_err(|e| AutoError::Msg(format!("Failed to read {}: {}", file_path.display(), e)))?;

    let mut local_uses = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        // Plan 167: handle both "use X" and "pub use X"
        let rest = if let Some(r) = trimmed.strip_prefix("pub use ") {
            r
        } else {
            match trimmed.strip_prefix("use ") {
                Some(r) => r,
                None => continue,
            }
        };
        if !rest.starts_with("c ") && !rest.starts_with(".rust") && !rest.starts_with("auto.") {
            // Extract module name (first segment before :, *, or end)
            let module_name = rest.split(|c: char| c == ':' || c == ' ' || c == '*')
                .next()
                .unwrap_or("")
                .trim();
            // Skip pac/super prefixed (handled by the resolver)
            if module_name == "pac" || module_name == "super"
                || module_name.starts_with("pac.") || module_name.starts_with("super.")
            {
                continue;
            }
            local_uses.push(module_name.to_string());
        }
    }

    // Add this module
    modules.push(ProjectModule {
        name: file_name.clone(),
        source_path: file_path.to_path_buf(),
        output_name: output_name.clone(),
        is_dir_module,
        uses: Vec::new(), // populated from parsed AST later
    });

    // Recursively discover dependencies
    for dep_name in &local_uses {
        // Plan 167: Handle dotted module names (e.g., "api.handlers")
        let parts: Vec<&str> = dep_name.split('.').collect();
        if parts.len() > 1 {
            // Check if the first segment matches the current directory module name
            // e.g., in api/mod.at, "api.handlers" -> just discover "handlers"
            let dir_name = rel_path.to_str().unwrap_or("");
            if dir_name == parts[0] {
                // Self-referential dotted path: strip the directory prefix
                let rest = parts[1..].join(".");
                let rest_file = file_path.parent().unwrap().join(format!("{}.at", rest));
                let rest_dir = file_path.parent().unwrap().join(&rest).join("mod.at");
                if rest_file.exists() {
                    discover_modules(&rest_file, base_dir, modules, visited)?;
                } else if rest_dir.exists() {
                    discover_modules(&rest_dir, base_dir, modules, visited)?;
                }
                continue;
            }

            // Cross-module dotted path: resolve each segment
            let first_file = file_path.parent().unwrap().join(format!("{}.at", parts[0]));
            let first_dir = file_path.parent().unwrap().join(&parts[0]).join("mod.at");

            let first_path = if first_file.exists() {
                first_file
            } else if first_dir.exists() {
                first_dir
            } else {
                continue;
            };
            // Discover the parent module
            discover_modules(&first_path, base_dir, modules, visited)?;

            // Then discover the nested module
            let parent_dir = first_path.parent().unwrap();
            let nested_name = parts[1..].join(".");
            let nested_file = parent_dir.join(format!("{}.at", nested_name));
            let nested_dir = parent_dir.join(&nested_name).join("mod.at");
            if nested_file.exists() {
                discover_modules(&nested_file, base_dir, modules, visited)?;
            } else if nested_dir.exists() {
                discover_modules(&nested_dir, base_dir, modules, visited)?;
            }
        } else {
            let dep_file = file_path.parent().unwrap().join(format!("{}.at", dep_name));
            let dep_dir = file_path.parent().unwrap().join(dep_name).join("mod.at");

            if dep_file.exists() {
                discover_modules(&dep_file, base_dir, modules, visited)?;
            } else if dep_dir.exists() {
                discover_modules(&dep_dir, base_dir, modules, visited)?;
            }
        }
    }

    Ok(())
}
