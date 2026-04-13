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

use super::{Sink, Trans};
use crate::ast::*;
use crate::database::Database;
use crate::parser::Parser;
// Plan 091: Universe removed
use crate::{AutoResult, Rc};
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

    // Hybrid: Support both Universe (deprecated) and Database (new)
    // Phase 066: Migrating to Database-based architecture
    db: Option<Arc<RwLock<Database>>>, // New (Phase 066)

    edition: RustEdition,

    // Transpiler internal state (not from Database or Universe)
    _current_fn: Option<AutoStr>,
    _current_scope: Option<crate::scope::Sid>,

    // Cache for struct field names (for positional arg mapping)
    struct_fields: HashMap<AutoStr, Vec<AutoStr>>,

    // Cache for tag type names (for tag construction detection)
    tag_types: HashSet<AutoStr>,

    // Plan 159 Phase 6B-2.2: Cache for spec declarations (for impl Trait for Type)
    spec_decls: HashMap<AutoStr, Vec<SpecMethod>>,

    // Plan 151: Global variables (top-level var declarations)
    // Tracks global variables that need Lazy<Mutex<T>> wrapper
    global_vars: HashSet<AutoStr>,

}

impl RustTrans {
    pub fn new(_name: AutoStr) -> Self {
        Self {
            indent: 0,
            uses: HashSet::new(),
            db: None, // New (Phase 066)
            edition: RustEdition::E2021,
            _current_fn: None,
            _current_scope: None,
            struct_fields: HashMap::new(),
            tag_types: HashSet::new(),
            spec_decls: HashMap::new(),
            global_vars: HashSet::new(),
        }
    }

    /// Create transpiler with Database (Phase 066: new API)
    pub fn with_database(db: Arc<RwLock<Database>>) -> Self {
        Self {
            indent: 0,
            uses: HashSet::new(),
            db: Some(db),
            edition: RustEdition::E2021,
            _current_fn: None,
            _current_scope: None,
            struct_fields: HashMap::new(),
            tag_types: HashSet::new(),
            spec_decls: HashMap::new(),
            global_vars: HashSet::new(),
        }
    }

    #[deprecated(note = "Use with_database() instead (Phase 066)")]
    pub fn set_scope(&mut self, _scope: Shared<crate::scope_manager::ScopeManager>) {
        // Plan 091: scope removed, no-op
    }

    pub fn set_edition(&mut self, edition: RustEdition) {
        self.edition = edition;
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

    fn rust_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Byte => "u8".to_string(),
            Type::Int => "i32".to_string(),
            Type::Uint => "u32".to_string(),
            Type::USize => "usize".to_string(),
            Type::Float | Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::Str(_) => "String".to_string(),
            Type::CStr => "&str".to_string(),
            Type::StrSlice => "&str".to_string(), // Borrowed string slice (Phase 3)
            Type::String => "String".to_string(), // Owned dynamic string (Plan 155)
            Type::Array(arr) => {
                format!("[{}; {}]", self.rust_type_name(&arr.elem), arr.len)
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
                format!("{}<{}>", inst.base_name, args.join(", "))
            }
            Type::Storage(storage) => {
                // Storage types are marker types, just use the name
                format!("{}", storage)
            }
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            // Plan 120: Option and Result types
            Type::Option(inner) => format!("Option<{}>", self.rust_type_name(inner)),
            Type::Result(inner) => format!("Result<{}, String>", self.rust_type_name(inner)),
            // Plan 121: Handle type - maps to Arc<TaskHandle<T>>
            Type::Handle { task_type } => format!("std::sync::Arc<TaskHandle<{}>>", self.rust_type_name(task_type)),
        }
    }

    /// Emit a2r standard library import
    /// Uses the crate's a2r_std module instead of embedding
    fn emit_a2r_stdlib(&self, out: &mut impl Write) -> AutoResult<()> {
        writeln!(out, "// a2r Standard Library (from crate)")?;
        writeln!(out, "#[allow(unused_imports)]")?;
        writeln!(out, "use auto_lang::a2r_std::*;")?;
        writeln!(out)?;
        Ok(())
    }

    // is_enum_type() moved to unified helper methods (line 83)
    // Old implementation removed in Phase 066

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
            } else if *c == '\\' {
                write!(out, "'\\\\'")
            } else {
                write!(out, "'{}'", c)
            }
            .map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", s).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", s).map_err(Into::into),
            Expr::Ident(name) => {
                // Plan 151: Global variable access - add .lock().unwrap() pattern
                if self.is_global_var(name) {
                    let static_name = self.global_var_static_name(name);
                    write!(out, "{}.lock().unwrap()", static_name)
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
                write!(out, ")").map_err(Into::into)
            }
            Expr::None => write!(out, "None").map_err(Into::into),
            Expr::Ok(e) => {
                write!(out, "Ok(")?;
                self.expr(e, out)?;
                write!(out, ")").map_err(Into::into)
            }
            Expr::Err(e) => {
                write!(out, "Err(")?;
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

                                // Check if lhs is a type name (starts with uppercase)
                                let is_type_name = if let Expr::Ident(lhs_name) = lhs.as_ref() {
                                    lhs_name
                                        .chars()
                                        .next()
                                        .map(|c| c.is_uppercase())
                                        .unwrap_or(false)
                                } else {
                                    false
                                };

                                if matches!(lhs.as_ref(), Expr::Ident(_))
                                    && (is_enum_variant || is_type_name)
                                {
                                    // Type::Variant or Type::method()
                                    self.expr(lhs, out)?;
                                    write!(out, "::")?;
                                    self.expr(rhs, out)?;
                                } else {
                                    // expr.field or expr.method()
                                    self.expr(lhs, out)?;
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
                write!(out, "{}", op_str)?;
                self.expr(expr, out)?;
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
                write!(out, "[")?;
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
                self.expr(idx, out)?;
                write!(out, "]").map_err(Into::into)
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
                        // **Phase 1.3: Tag Types**
                        // Tag patterns: Atom.Int(i) -> Atom::Int(i)
                        // Empty variants (elem == "_"): Coin.Penny -> Coin::Penny
                        if tag_cover.elem.as_str() == "_" {
                            write!(out, "{}::{}", tag_cover.kind, tag_cover.tag)
                                .map_err(Into::into)
                        } else {
                            write!(
                                out,
                                "{}::{}({})",
                                tag_cover.kind, tag_cover.tag, tag_cover.elem
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
                // Check if this is a loop expression
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

                    for (i, arg) in node.args.args.iter().enumerate() {
                        match arg {
                            Arg::Pos(expr) => {
                                // Positional arg - map to actual field name from cached field names
                                let field_name = if i < field_names.len() {
                                    field_names[i].clone()
                                } else {
                                    format!("field{}", i).into()
                                };
                                write!(out, "{}: ", field_name)?;
                                self.expr(expr, out)?;
                            }
                            Arg::Name(name) => {
                                // Named arg without value
                                write!(out, "{}: ", name)?;
                            }
                            Arg::Pair(key, expr) => {
                                // Named argument: field: value
                                write!(out, "{}: ", key)?;
                                self.expr(expr, out)?;
                            }
                        }
                        if i < node.args.args.len() - 1 || !node.body.stmts.is_empty() {
                            write!(out, ", ")?;
                        }
                    }

                    // Handle body statements (field initializers)
                    for (i, stmt) in node.body.stmts.iter().enumerate() {
                        match stmt {
                            Stmt::Store(store) => {
                                write!(out, "{}: ", store.name)?;
                                self.expr(&store.expr, out)?;
                            }
                            Stmt::Expr(Expr::Pair(pair)) => {
                                // Named field initializer: x: 3
                                let field_name = match &pair.key {
                                    crate::ast::Key::NamedKey(name) => name.clone(),
                                    crate::ast::Key::IntKey(n) => format!("{}", n).into(),
                                    crate::ast::Key::BoolKey(b) => format!("{}", b).into(),
                                    crate::ast::Key::StrKey(s) => s.clone(),
                                };
                                write!(out, "{}: ", field_name)?;
                                self.expr(&pair.value, out)?;
                            }
                            _ => {}
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
                            write!(out, "{}", s.replace("\"", r##"\""##))?;
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
                                    // Check if this expression needs a semicolon
                                    let needs_semicolon = if !is_last {
                                        true
                                    } else {
                                        // Check if it's a call to print or other void functions
                                        match expr {
                                            Expr::Call(call) => {
                                                if let Expr::Ident(name) = call.name.as_ref() {
                                                    name == "print" || name == "println"
                                                } else {
                                                    false
                                                }
                                            }
                                            _ => false,
                                        }
                                    };

                                    if needs_semicolon {
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
                                    // Check if this expression needs a semicolon
                                    let needs_semicolon = if !is_last {
                                        true
                                    } else {
                                        // Check if it's a call to print or other void functions
                                        match expr {
                                            Expr::Call(call) => {
                                                if let Expr::Ident(name) = call.name.as_ref() {
                                                    name == "print" || name == "println"
                                                } else {
                                                    false
                                                }
                                            }
                                            _ => false,
                                        }
                                    };

                                    if needs_semicolon {
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
                // Use heuristic: if object is an identifier starting with uppercase
                if let Expr::Ident(type_name) = object.as_ref() {
                    let is_type_name = type_name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    if is_type_name {
                        // Type::Variant (enum) or Type::method (static method)
                        write!(out, "{}::{}", type_name, field)?;
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
                    Type::Str(_) | Type::String | Type::StrSlice | Type::CStr => {
                        // x.to(str) / x.to(String) → x.to_string()
                        self.expr(expr, out)?;
                        write!(out, ".to_string()")?;
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
                        // Fallback: treat as cast (same as .as())
                        write!(out, "(")?;
                        self.expr(expr, out)?;
                        write!(out, " as {})", self.rust_type_name(target_type))?;
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

            _ => Err(format!("Rust Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // Special case for print function
        if let Expr::Ident(name) = call.name.as_ref() {
            if name == "print" {
                return self.print_call(call, out);
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

        // Plan 151 Phase 1.3: Method call translations
        // Translate Auto method names to Rust equivalents
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if let Expr::Ident(method_name) = rhs.as_ref() {
                    let rust_method = match method_name.as_str() {
                        // String methods
                        "to_lower" => Some("to_lowercase"),
                        "to_upper" => Some("to_uppercase"),
                        "length" => Some("len"),
                        "is_empty" => Some("is_empty"),
                        "trim" => Some("trim"),
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
                        self.expr(lhs, out)?;
                        write!(out, ".{}(", rust_name)?;
                        // Special handling for .contains() - auto-borrow string args
                        if method_name.as_str() == "contains" {
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
                _ => {} // fall through to regular method handling
            }

            let rust_method = match method_name.as_str() {
                // String methods
                "to_lower" => Some("to_lowercase"),
                "to_upper" => Some("to_uppercase"),
                "length" => Some("len"),
                "is_empty" => Some("is_empty"),
                "trim" => Some("trim"),
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
                self.expr(object, out)?;
                write!(out, ".{}(", rust_name)?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    self.arg(arg, out)?;
                    if i < call.args.args.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, ")")?;
                return Ok(());
            }

            // Check for type name static method: Type.method(args) -> Type::method(args)
            if let Expr::Ident(type_name) = object.as_ref() {
                let is_type = type_name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                if is_type {
                    // Check for tag construction: Type.Variant(args)
                    if self.tag_types.contains(type_name) {
                        write!(out, "{}::{}", type_name, method_name)?;
                        write!(out, "(")?;
                        if let Some(Arg::Pos(expr)) = call.args.args.first() {
                            self.expr(expr, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // Static method: Type::method(args)
                    write!(out, "{}::{}", type_name, method_name)?;
                    write!(out, "(")?;
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

            // Regular method call: object.method(args)
            self.expr(object, out)?;
            write!(out, ".{}(", method_name)?;
            for (i, arg) in call.args.args.iter().enumerate() {
                self.arg(arg, out)?;
                if i < call.args.args.len() - 1 {
                    write!(out, ", ")?;
                }
            }
            write!(out, ")")?;
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
                            // Tag construction: TypeName::VariantName(arg)
                            write!(out, "{}::{}", type_name, variant_name)?;
                            write!(out, "(")?;
                            if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                self.expr(expr, out)?;
                            }
                            write!(out, ")")?;
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

        // Normal function call
        self.expr(&call.name, out)?;
        write!(out, "(")?;
        for (i, arg) in call.args.args.iter().enumerate() {
            self.arg(arg, out)?;
            if i < call.args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, ")").map_err(Into::into)
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

        for (i, arg) in args.args.iter().enumerate() {
            match arg {
                Arg::Pos(expr) => {
                    // Positional arg - map to actual field name from cached field names
                    let field_name = if i < field_names.len() {
                        field_names[i].clone()
                    } else {
                        format!("field{}", i).into()
                    };
                    write!(out, "{}: ", field_name)?;
                    self.expr(expr, out)?;
                }
                Arg::Name(name) => {
                    // Named arg without value
                    write!(out, "{}: ", name)?;
                }
                Arg::Pair(key, expr) => {
                    // Named argument: field: value
                    write!(out, "{}: ", key)?;
                    self.expr(expr, out)?;
                }
            }
            if i < args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, " }}").map_err(Into::into)
    }

    fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Pos(expr) => self.expr(expr, out),
            Arg::Name(name) => write!(out, "{}", name).map_err(Into::into),
            Arg::Pair(_, expr) => self.expr(expr, out),
        }
    }

    fn print_call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // print("hello") -> println!("hello")
        // print(value) -> println!("{}", value)
        // print(f"...") -> println!("...", args)
        // print("text:", value) -> println!("text: {}", value)

        if call.args.args.is_empty() {
            write!(out, "println!()")?;
            return Ok(());
        }

        // Check if first argument is an f-string
        if let Arg::Pos(first_arg) = &call.args.args[0] {
            if let Expr::FStr(fstr) = first_arg {
                // Generate println! with f-string format
                write!(out, "println!(\"")?;

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
                            // Expression placeholder
                            write!(out, "{{}}")?;
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
                        write!(out, "println!(\"{}\")", s)?;
                        return Ok(());
                    }
                    _ => {
                        // Single non-string argument: use format string
                        write!(out, "println!(\"{{}}\", ")?;
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

                // Add placeholders for remaining args
                for _ in call.args.args.iter().skip(1) {
                    format_string.push_str(" {}");
                }

                write!(out, "println!(\"{}\"", format_string)?;

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
        write!(out, "println!(\"")?;
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

            Stmt::Return(expr) => {
                sink.body.write(b"return ")?;
                self.expr(expr, &mut sink.body)?;
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

            _ => Err(format!("Rust Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    // Variable declaration
    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // Handle C variables and struct fields (should not be generated)
        match store.kind {
            StoreKind::CVar | StoreKind::Field => {
                return Ok(());
            }
            _ => {}
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
        if matches!(store.ty, Type::Unknown) {
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
            // Check if the expression is a closure - closures should not have explicit type annotations
            // because Rust infers closure types automatically
            let is_closure = matches!(store.expr, Expr::Closure(_));

            // Check if expression is a borrow (&x or &mut x) - type should be a reference
            let is_borrow = matches!(&store.expr, Expr::View(_))
                || matches!(&store.expr, Expr::Dot(_, f) if f.as_str() == "view");
            let is_mut_borrow = matches!(&store.expr, Expr::Dot(_, f) if f.as_str() == "mut");

            let ty_name = if is_borrow && matches!(store.ty, Type::String | Type::Str(_)) {
                "&str".to_string()
            } else if is_mut_borrow && matches!(store.ty, Type::String | Type::Str(_)) {
                "&mut str".to_string()
            } else {
                self.rust_type_name(&store.ty)
            };

            if is_closure {
                // For closures, don't add type annotation - let Rust infer it
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
                match store.kind {
                    StoreKind::Let => {
                        write!(out, "let {}: {} = ", store.name, ty_name)?;
                    }
                    StoreKind::Var => {
                        write!(out, "let mut {}: {} = ", store.name, ty_name)?;
                    }
                    _ => {
                        write!(out, "let {}: {} = ", store.name, ty_name)?;
                    }
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
        } else {
            self.expr(&store.expr, out)?;
        }

        // When assigning a string literal to a String/Str type, add .to_string()
        // because Rust string literals are &str, but String type needs conversion
        if matches!(store.ty, Type::String | Type::Str(_)) {
            if let Expr::Str(_) = &store.expr {
                write!(out, ".to_string()")?;
            }
        }

        Ok(())
    }

    // Function declaration
    fn fn_decl(&mut self, fn_decl: &Fn, sink: &mut Sink) -> AutoResult<()> {
        // Skip C/VM function declarations (implemented externally)
        if matches!(fn_decl.kind, FnKind::CFunction | FnKind::VmFunction) {
            return Ok(());
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
        if fn_decl.is_pub {
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

        // Parameters
        write!(sink.body, "(")?;

        // Add &self as first parameter for methods
        if is_method && !fn_decl.is_static {
            // Plan 163: &mut self for mut methods
            if fn_decl.is_mut {
                write!(sink.body, "&mut self")?;
            } else {
                write!(sink.body, "&self")?;
            }
            if !fn_decl.params.is_empty() {
                write!(sink.body, ", ")?;
            }
        }

        for (i, param) in fn_decl.params.iter().enumerate() {
            write!(
                sink.body,
                "{}: {}",
                param.name,
                self.rust_type_name(&param.ty)
            )?;
            if i < fn_decl.params.len() - 1 {
                write!(sink.body, ", ")?;
            }
        }
        write!(sink.body, ")")?;

        // Return type - unwrap Future/Handle for async fn (Rust's async fn wraps implicitly)
        if !matches!(fn_decl.ret, Type::Void) {
            let ret_str = if is_async_fn {
                match &fn_decl.ret {
                    Type::Handle { task_type } => self.rust_type_name(task_type),
                    Type::GenericInstance(inst) if inst.base_name == "Future" => {
                        self.rust_type_name(inst.args.first().unwrap_or(&Type::Unknown))
                    }
                    other => self.rust_type_name(other),
                }
            } else {
                self.rust_type_name(&fn_decl.ret)
            };
            write!(sink.body, " -> {}", ret_str)?;
        }

        // Function body
        write!(sink.body, " ")?;
        // Plan 091: scope removed
        self.body(&fn_decl.body, sink, &fn_decl.ret, "")?;
        // Plan 091: scope removed

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
                    for stmt in &for_stmt.body.stmts {
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
                            _ => {}
                        }
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                } else {
                    // Array iteration: for x in arr
                    self.expr(&for_stmt.range, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    // Body
                    self.indent();
                    for stmt in &for_stmt.body.stmts {
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
                            _ => {}
                        }
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
            Iter::Ever => {
                // Infinite loop: loop { body }
                sink.body.write(b"loop {\n")?;
                self.indent();
                for stmt in &for_stmt.body.stmts {
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
                        _ => {}
                    }
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            Iter::Cond => {
                // Conditional loop: while condition { ... }
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
                for stmt in &for_stmt.body.stmts {
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
                        _ => {}
                    }
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            _ => {}
        }
        Ok(())
    }

    // If statement
    fn if_stmt(&mut self, if_: &If, sink: &mut Sink) -> AutoResult<()> {
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
                        // Check if this expression needs a semicolon
                        // Add semicolon if: not last OR it's a void function call (like print)
                        let needs_semicolon = if !is_last {
                            true
                        } else {
                            // Check if it's a call to print or other void functions
                            match expr {
                                Expr::Call(call) => {
                                    if let Expr::Ident(name) = call.name.as_ref() {
                                        name == "print" || name == "println"
                                    } else {
                                        false
                                    }
                                }
                                _ => false,
                            }
                        };

                        if needs_semicolon {
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
                    Stmt::Return(ret) => {
                        sink.body.write(b"return ")?;
                        self.expr(ret, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    _ => {}
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
                        // Check if this expression needs a semicolon
                        let needs_semicolon = if !is_last {
                            true
                        } else {
                            // Check if it's a call to print or other void functions
                            match expr {
                                Expr::Call(call) => {
                                    if let Expr::Ident(name) = call.name.as_ref() {
                                        name == "print" || name == "println"
                                    } else {
                                        false
                                    }
                                }
                                _ => false,
                            }
                        };

                        if needs_semicolon {
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
                    _ => {}
                }
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }

        Ok(())
    }

    // Is statement (pattern matching)
    /// Write a match arm body: single expression inline, or block for multiple statements
    fn write_match_arm_body(&mut self, body: &Body, sink: &mut Sink) -> AutoResult<()> {
        if body.stmts.is_empty() {
            sink.body.write(b"{}")?;
        } else if body.stmts.len() == 1 {
            // Single statement: write inline
            match &body.stmts[0] {
                Stmt::Expr(expr) => self.expr(expr, &mut sink.body)?,
                Stmt::Return(ret) => {
                    sink.body.write(b"return ")?;
                    self.expr(ret, &mut sink.body)?;
                }
                _ => {
                    // For other statement types, use a block
                    sink.body.write(b"{\n")?;
                    self.indent();
                    for stmt in &body.stmts {
                        self.print_indent(&mut sink.body)?;
                        self.stmt(stmt, sink)?;
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
        self.expr(&is_stmt.target, &mut sink.body)?;
        sink.body.write(b" {\n")?;
        self.indent();

        for branch in &is_stmt.branches {
            self.print_indent(&mut sink.body)?;

            match branch {
                IsBranch::EqBranch(expr, body) => {
                    self.expr(expr, &mut sink.body)?;
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
        match use_stmt.kind {
            UseKind::Auto => {
                // Map Auto stdlib to Rust modules
                for path in &use_stmt.paths {
                    let rust_path = path.replace("auto.", "crate::");
                    write!(out, "use {};", rust_path)?;
                    self.uses.insert(path.clone());
                }
            }
            UseKind::C => {
                // Ignore C imports for Rust transpiler
            }
            UseKind::Rust => {
                // Direct Rust imports: join paths with :: to form full Rust path
                if !use_stmt.paths.is_empty() {
                    let full_path = use_stmt.paths.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("::");
                    if use_stmt.items.is_empty() {
                        write!(out, "use {};", full_path)?;
                    } else {
                        write!(out, "use {}::{{{}}};", full_path, use_stmt.items.join(", "))?;
                    }
                    self.uses.insert(full_path.into());
                }
            }
        }
        Ok(())
    }

    // Type declaration (struct)
    fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
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
        for attr in &type_decl.attrs {
            write!(sink.body, "#[{}]\n", attr)?;
        }

        // Plan 163: Output pub prefix
        if type_decl.is_pub {
            write!(sink.body, "pub ")?;
        }

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
                    "{}: {},",
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
                            self.rust_type_name(&param.ty)
                        )?;
                    }

                    // Return type
                    if !matches!(spec_method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&spec_method.ret))?;
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
                                self.rust_type_name(&param.ty)
                            )?;
                        }

                        // Return type
                        if !matches!(method.ret, Type::Void) {
                            write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
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
        // Plan 163: Output pub prefix
        if enum_decl.is_pub {
            sink.body.write(b"pub ")?;
        }

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
            EnumKind::Heterogeneous { .. } => {
                // Reuse tag code generation: convert EnumDecl to Tag
                let tag = Self::enum_decl_to_tag(enum_decl);
                self.tag_decl(&tag, sink)?;
            }
        }

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
        match &ext.trait_name {
            Some(trait_name) => {
                write!(sink.body, "impl {} for {}", trait_name, ext.target)?;
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
                    self.rust_type_name(&param.ty)
                )?;
            }

            // Return type
            if !matches!(method.ret, Type::Void) {
                write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
            } else {
                write!(sink.body, ")")?;
            }

            writeln!(sink.body, ";")?;
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
            if !matches!(stmt, Stmt::EmptyLine(_)) {
                self.print_indent(&mut sink.body)?;
            }

            let is_last = i == body.stmts.len() - 1;

            if is_last && has_return && self.is_returnable(stmt) {
                // Last statement in a non-void function: expression position (no semicolon)
                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
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
                            if name == "print" || name == "println" {
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

        // Phase 2: Split into declarations and main
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main: Vec<Stmt> = Vec::new();

        for stmt in ast.stmts.into_iter() {
            if stmt.is_decl() {
                // Plan 151: Register global variables (top-level var declarations)
                if let Stmt::Store(store) = &stmt {
                    if matches!(store.kind, StoreKind::Var) {
                        self.register_global_var(store.name.clone());
                    }
                }
                decls.push(stmt);
            } else {
                match stmt {
                    Stmt::For(_) => main.push(stmt),
                    Stmt::If(_) => main.push(stmt),
                    Stmt::Expr(_) => main.push(stmt),
                    Stmt::Store(_) => main.push(stmt),
                    Stmt::Break => main.push(stmt),
                    Stmt::Use(use_stmt) => {
                        self.use_stmt(&use_stmt, &mut sink.body)?;
                        sink.body.write(b"\n")?;
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
        for (i, decl) in decls.iter().enumerate() {
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
            let is_async = Self::has_await(&main);
            if is_async {
                sink.body.write(b"#[tokio::main]\n")?;
                sink.body.write(b"async fn main() {\n")?;
            } else {
                sink.body.write(b"fn main() {\n")?;
            }
            self.indent();

            for stmt in main.iter() {
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
