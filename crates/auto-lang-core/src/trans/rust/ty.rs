//! PLAN-843 结构拆分（自 rust.rs 原样搬移；算法/行为零变化）。
use super::*;

impl RustTrans {
    /// Plan musk-022 CRUD 智能扩展: register a struct type so isolated fn
    /// transpilation resolves `Type { field: val }` construction. Mirrors
    /// the whole-module pre-scan.
    pub fn register_type(&mut self, name: &str, fields: Vec<(&str, Type)>) {
        let name_a: AutoStr = AutoStr::from(name);
        let field_names: Vec<AutoStr> = fields.iter().map(|(n, _)| AutoStr::from(*n)).collect();
        self.struct_fields.insert(name_a.clone(), field_names);
        self.struct_field_types.insert(
            name_a,
            fields
                .into_iter()
                .map(|(n, t)| (AutoStr::from(n), t))
                .collect(),
        );
    }

    /// Mutable access to known_enum_names (plan 372 follow-up: lets callers
    /// pre-populate enum names from sibling files so cross-module enum errors
    /// like `Err(AgentError::Config(...))` don't get wrongly Box::new'd).
    pub fn known_enum_names_mut(&mut self) -> &mut std::collections::HashSet<AutoStr> {
        &mut self.known_enum_names
    }

    /// Scan statements for Err(X) calls; if all use the same enum type, return it
    pub(super) fn infer_err_enum(&self, stmts: &[Stmt]) -> Option<AutoStr> {
        let mut found_enum: Option<AutoStr> = None;
        for stmt in stmts {
            let result = self.scan_stmt_err_enum(stmt);
            match result {
                Some(Some(enum_name)) => match &found_enum {
                    Some(existing) if *existing != enum_name => return None,
                    _ => found_enum = Some(enum_name),
                },
                Some(None) => return None,
                None => {}
            }
        }
        found_enum
    }

    /// Plan 381 (Layer 2): true when the call is an enum-variant construction
    /// (`Type.Variant(...)` where Type is a known enum). Such calls must not
    /// get the unknown-callee fallback auto-borrow (.as_str() on owned String
    /// payloads → E0308).
    pub(super) fn is_enum_variant_ctor(&self, call: &Call) -> bool {
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if let Expr::Ident(type_name) = obj.as_ref() {
                if self.known_enum_names.contains(type_name) {
                    return true;
                }
            }
        }
        false
    }

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

    /// Check if current function's return type maps to Rust String (needs &str -> String coercion)
    pub(super) fn ret_type_needs_string_coercion(&self) -> bool {
        // PLAN-668 R-24：str 族 let-if 初始化语境同 fn 返回位强制。
        self.let_init_str_coercion
            || self.current_fn_ret_type.as_ref().map_or(false, |ty| {
                matches!(
                    ty,
                    Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                )
            })
    }

    /// Plan 013 (B1/BUG2): Check if the current function's return type is a
    /// non-Copy owned type (String, struct, enum, List, Map, Option, etc.).
    /// Returning `self.field` of such a type from a `&self` method needs an
    /// explicit `.clone()` to avoid E0507 (cannot move out of &self).
    pub(super) fn ret_type_is_owned_noncopy(&self) -> bool {
        match &self.current_fn_ret_type {
            // Copy primitives — returning self.field of these is fine.
            None
            | Some(
                Type::Byte
                | Type::Int
                | Type::Uint
                | Type::USize
                | Type::I64
                | Type::U64
                | Type::Float
                | Type::Double
                | Type::Bool
                | Type::Char,
            ) => false,
            // Everything else (String variants, List, Map, User, Enum, Tag,
            // Option, Result, Tuple, …) is owned and non-Copy.
            Some(_) => true,
        }
    }

    /// Plan 364 W3: render a trait-bound type — specs emit their bare name
    /// (a bound `T: Greeter` must not become `T: Box<dyn Greeter>`); all other
    /// types fall through to the normal type rendering.
    pub(super) fn rust_bound_name(&self, ty: &Type) -> String {
        match ty {
            Type::User(usr) => self.qualify_type_name(&usr.name.to_string()),
            Type::Spec(spec) => spec.borrow().name.to_string(),
            _ => self.rust_type_name(ty),
        }
    }

    pub(super) fn rust_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Byte => "u8".to_string(),
            // Plan 399 Phase 11.1: Auto `int` → Rust `i64` (was i64). The HTTP
            // backend's types.rs uses i64 (auto_type_to_rust), so a2r emitting i32
            // caused db.rs/types.rs type mismatches (post_process_db_rs patched
            // with a blunt code.replace("i32","i64")). Now unified at the source.
            Type::Int => "i64".to_string(),
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
                // List<T> transpiles to Vec<T> in Rust.
                // PLAN-681 T-05（N2 动态记录面）：bare List（无元素型——VM
                // 动态桶，元素可为异构记录）此前缺省 Vec<String>——记录
                // 元素的字段访问/字面量构造全部 E0609/语法断（bps
                // tree_util 株）。动态桶语义源 = VM List（异构值），缺省
                // 改 Vec<serde_json::Value>：元素按 Value 访问器降链，
                // 记录字面量走 json! 构造（emit_json_value 路径）。
                let elem_name = if matches!(elem.as_ref(), Type::Unknown) {
                    "serde_json::Value".to_string() // bare List = dynamic bucket
                } else {
                    self.rust_type_name(elem)
                };
                format!("Vec<{}>", elem_name)
            }
            Type::Map(k, v) => {
                let k_name = if matches!(k.as_ref(), Type::Unknown) {
                    "String".to_string() // bare Map defaults to HashMap<String, String>
                } else {
                    self.rust_type_name(k)
                };
                let v_name = if matches!(v.as_ref(), Type::Unknown) {
                    "String".to_string()
                } else {
                    self.rust_type_name(v)
                };
                format!("std::collections::HashMap<{}, {}>", k_name, v_name)
            }
            Type::Slice(slice) => {
                // []T → &[T], but []Spec → Vec<Box<dyn Spec>> (dynamic polymorphism).
                // Plan 016 Phase A A.4: []byte → Vec<u8> (not &[u8]) so it matches
                // owned byte sources like HTTP body_bytes() → Vec<u8> without
                // needing auto-borrow at call sites.
                if matches!(&*slice.elem, Type::Spec(_)) {
                    format!("Vec<{}>", self.rust_type_name(&slice.elem))
                } else if matches!(&*slice.elem, Type::Byte) {
                    "Vec<u8>".to_string()
                } else {
                    format!("&[{}]", self.rust_type_name(&slice.elem))
                }
            }
            Type::Ptr(ptr) => {
                // **Phase 1.1: Pointer Types (test: 005_pointer)**
                // AutoLang *T transpiles to Rust raw pointer *mut T
                // This is for raw pointer operations like @ (address-of) and .* (dereference)
                // Plan 610 ⑤⑥: `*void` (opaque handles / C void*) maps to *mut c_void —
                // the pre-610 `*mut void` output was never valid Rust (no corpus depended).
                if matches!(&*ptr.of.borrow(), Type::Void) {
                    "*mut std::ffi::c_void".to_string()
                } else {
                    format!("*mut {}", self.rust_type_name(&*ptr.of.borrow()))
                }
            }
            Type::Reference(inner) => {
                // Plan 052: Reference transpiles to &T in Rust
                format!("&{}", self.rust_type_name(inner))
            }
            Type::User(usr) => {
                // Plan 371 (defect A): if this bare User type name is actually a
                // known spec (from this file's pre-scan OR a sibling .at file's
                // pre-populated spec_decls), emit Box<dyn X> — the parser failed
                // to mark it Type::Spec because the spec lives in another module
                // / was declared later. Without this, `role Role` (Role a spec)
                // emits bare `Role` (E0782). Mirrors what Type::Spec does (below).
                let name = usr.name.to_string();
                // Plan 380 P1 (defect D): "dyn Trait" (from parse_type_base dyn
                // branch) — render verbatim, do NOT qualify (no crate:: prefix).
                if name.starts_with("dyn ") {
                    name
                } else if name.starts_with("Self::") {
                    // Plan 417-E2: associated-type reference rewritten for trait
                    // emission (`Self::Item`) — verbatim, never qualified.
                    name
                } else if self.current_fn_type_params.contains(usr.name.as_str()) {
                    // Plan 417-E3: generic type parameter of the current fn —
                    // bare param name, no Box<dyn>/qualification treatment.
                    name
                } else if let Some(bound_ty) = self.current_assoc_bindings.get(usr.name.as_str()) {
                    // Plan 417-E2 followup (F1): associated-type reference
                    // inside the TypeDecl being emitted — render the
                    // implementer's binding (Rust is static; the bare assoc
                    // name would not resolve). Covers method signatures, local
                    // var annotations and field types.
                    self.rust_type_name(bound_ty)
                } else if self.spec_decls.contains_key(name.as_str()) {
                    format!("Box<dyn {}>", name)
                } else {
                    self.qualify_type_name(&name)
                }
            }
            Type::Enum(en) => self.qualify_type_name(&en.borrow().name.to_string()),
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
                // Plan 387 §16 P0-2: TaskRef<T> is a first-class actor-handle type
                // mapping to the a2r-std runtime type.
                if inst.base_name == "TaskRef" {
                    self.a2r_std_used.set(true);
                    return format!("a2r_std::task::TaskRef<{}>", args.join(", "));
                }
                // Plan 190: Use short_name from RustSource if available
                let base = if let Some(ref source) = inst.source {
                    source.short_name().to_string()
                } else {
                    inst.base_name.to_string()
                };
                // Plan 390 §15.10/§15.11 (L2 转正): `Box<Fn>` → `Box<dyn Fn>`,
                // `Arc<Tool>` → `Arc<dyn Tool>` — a spec inside a Box/Arc
                // container is a trait object (`dyn`). The value side matches:
                // `Arc(spec_bound_ident)` renders `Arc::from(x)` (Box→Arc
                // conversion, single wrap — see ArcExpr), so the field type and
                // the constructed value agree. A `Fn(...)` signature arg
                // (Type::Fn) becomes `dyn Fn(...)` (closure trait) instead of
                // the fn-pointer `fn(...)`.
                if matches!(base.as_str(), "Box" | "Arc") && inst.args.len() == 1 {
                    if let Some(arg_ty) = inst.args.first() {
                        match arg_ty {
                            Type::Spec(spec) => {
                                // Plan 395-followup: bare `Box<Fn>` → `Box<dyn Fn + Send + Sync>`
                                // (Send for tokio::spawn'd actors; rust-ref parity).
                                if spec.borrow().name == "Fn" {
                                    return format!("{}<dyn Fn + Send + Sync>", base);
                                }
                                return format!("{}<dyn {}>", base, spec.borrow().name);
                            }
                            Type::User(usr) => {
                                // Plan 390 §15.11-followup (cross-module spec): a spec
                                // IMPORTED via `use mod: Spec` reaches here as Type::User
                                // (not Type::Spec — that's only for same-module declarations).
                                // The §15.11 single-wrap (Arc<dyn T>) only matched Type::Spec,
                                // so cross-module `Arc<Tool>` rendered `Arc<Box<dyn Tool>>`
                                // (the inner User→Box<dyn T> fallback, then Arc-wrapped) — a
                                // double-wrap mismatching same-module single-wrap storage.
                                // Mirror the Type::Spec branch: if the User name is a known spec
                                // (spec_decls covers same-module + sibling imports), render single
                                // Arc<dyn T>. (Fn handled above via Type::Spec; a cross-module Fn
                                // spec also lands here — give it the same + Send + Sync treatment.)
                                let name = usr.name.to_string();
                                if self.spec_decls.contains_key(name.as_str()) {
                                    if name == "Fn" {
                                        return format!("{}<dyn Fn + Send + Sync>", base);
                                    }
                                    return format!("{}<dyn {}>", base, name);
                                }
                                // Not a known spec — fall through to default rendering below.
                            }
                            Type::Fn(params, ret) => {
                                let param_str: Vec<String> =
                                    params.iter().map(|p| self.rust_type_name(p)).collect();
                                let ret_str = if matches!(&**ret, Type::Void) {
                                    String::new()
                                } else {
                                    format!(" -> {}", self.rust_type_name(ret))
                                };
                                // Plan 395-followup: `+ Send + Sync` — closure fields/params may be moved
                                // into a tokio::spawn'd actor (future must be Send); rust-ref uses
                                // `Arc<dyn Fn(...) + Send + Sync>`. Generated closures capture only
                                // fn-pointers/Copy values, so they satisfy the bounds.
                                return format!(
                                    "{}<dyn Fn({}){} + Send + Sync>",
                                    base,
                                    param_str.join(", "),
                                    ret_str
                                );
                            }
                            _ => {}
                        }
                    }
                }
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
            // Plan 121 + 387 follow-up P4: the legacy `Handle<T>` type maps to the
            // current actor handle `a2r_std::task::TaskRef<T>` (the old
            // `std::sync::Arc<TaskHandle<T>>` referenced a type that was removed
            // from a2r-std in §17.1 — any latent use would not compile).
            Type::Handle { task_type } => {
                format!("a2r_std::task::TaskRef<{}>", self.rust_type_name(task_type))
            }
            Type::Rust(source) => source.short_name().to_string(),
            Type::Tuple(ts) => {
                let elems: Vec<String> = ts.iter().map(|t| self.rust_type_name(t)).collect();
                format!("({})", elems.join(", "))
            }
        }
    }

    /// Plan 264: Qualify a type name with its module path.
    /// Handles both bare names ("ForgeSession") and dotted paths ("forge.ForgeSession").
    /// If the type is defined in another module, returns `crate::module::Type`.
    /// If defined in the current module, returns bare `Type`.
    /// In merge_mode, all types are in one file — always return bare name.
    fn qualify_type_name(&self, name: &str) -> String {
        // Skip well-known Rust/std types that should never be qualified
        match name {
            "String" | "Vec" | "HashMap" | "HashSet" | "Option" | "Result" | "Box" | "Rc"
            | "Arc" | "Mutex" | "RwLock" | "IoError" | "Error" | "Display" | "Debug" | "Ok"
            | "Err" | "Some" | "None" | "Self" => return name.to_string(),
            _ => {}
        }

        // Plan 347: The Auto VM exposes `StringBuilder` as a built-in type.
        // It has no Rust-native equivalent, so map it to the a2r-std runtime
        // implementation (`a2r_std::StringBuilder`). Emit it fully-qualified so
        // it resolves regardless of whether the glob `use a2r_std::*` import is
        // present (e.g. in merge mode).
        if name == "StringBuilder" {
            self.a2r_std_used.set(true);
            return "a2r_std::StringBuilder".to_string();
        }

        // Plan 415-B1: the sqlite module's opaque connection handle. The
        // distinct `SqliteDb` name (not `Db`) avoids hijacking user structs;
        // map it to the a2r-std rusqlite wrapper, fully-qualified so it
        // resolves without the glob `use a2r_std::*` import (StringBuilder
        // precedent above).
        if name == "SqliteDb" {
            self.a2r_std_used.set(true);
            return "a2r_std::sqlite::SqliteDb".to_string();
        }

        // Plan 415-B2: the redis module's client handle, same rationale as
        // SqliteDb above (distinct name, fully-qualified a2r-std mapping).
        if name == "RedisClient" {
            self.a2r_std_used.set(true);
            return "a2r_std::redis::RedisClient".to_string();
        }

        // PLAN-729 T-05: 服务端文件响应描述符——同 StringBuilder/SqliteDb
        // 先例，全限定映射到 a2r-std 公共类型（无 glob import 也解析）。
        if name == "FileResponse" {
            self.a2r_std_used.set(true);
            return "a2r_std::http::FileResponse".to_string();
        }

        // PLAN-730 T-06: 服务端上传三类型（注入能力/会话快照/收据）——
        // 同 FileResponse 先例全限定映射（opaque，无用户构造面）。
        if matches!(name, "UploadRequest" | "UploadSession" | "UploadReceipt") {
            self.a2r_std_used.set(true);
            return format!("a2r_std::http::{name}");
        }

        // Merge mode: all types are in one file, skip crate:: prefix
        if self.merge_mode {
            if let Some(dot_pos) = name.rfind('.') {
                return name[dot_pos + 1..].to_string();
            }
            return name.to_string();
        }

        // Handle dotted paths like "forge.ForgeSession"
        if let Some(dot_pos) = name.rfind('.') {
            let prefix = &name[..dot_pos];
            let bare = &name[dot_pos + 1..];

            // Check if prefix is a known module and bare name is a type in it
            if let Some(types) = self.module_types.get(prefix) {
                if types.contains(bare) {
                    if prefix == self.current_module_name {
                        return bare.to_string();
                    }
                    // Convert dotted prefix to :: path: "forge" → "crate::forge"
                    let rust_prefix = prefix.replace('.', "::");
                    if prefix.contains('.') {
                        return format!("crate::{}::{}", rust_prefix, bare);
                    }
                    return format!("crate::{}::{}", prefix, bare);
                }
            }

            // Prefix not a known module — try to resolve bare name
            for (mod_name, types) in &self.module_types {
                if types.contains(bare) {
                    if *mod_name == self.current_module_name {
                        return bare.to_string();
                    }
                    return format!("crate::{}::{}", mod_name, bare);
                }
            }

            // Fallback: convert all dots to ::
            return name.replace('.', "::");
        }

        // Bare name: look up which module defines it
        for (mod_name, types) in &self.module_types {
            if types.contains(name) {
                if *mod_name == self.current_module_name {
                    return name.to_string();
                }
                return format!("crate::{}::{}", mod_name, name);
            }
        }
        name.to_string()
    }

    /// Is `name` an imported concrete type (not a trait)? Used by
    /// [`rust_return_type_name`] to suppress the `impl` prefix for enums /
    /// structs / aliases that reach this file via `use X: Name` (Auto) or
    /// `use X::{Y}` (Rust) imports. Mirrors the brace-expansion fuzzy match
    /// already used for type qualification (see call sites of `self.uses`).
    ///
    /// A bare `IntoResponse` referenced without any import (Plan 380 P2 golden
    /// `015_impl_trait_return`) is intentionally NOT matched — it has no
    /// `self.uses` entry, so the `impl` prefix is preserved for real traits.
    fn is_imported_concrete_type(&self, name: &str) -> bool {
        self.uses.iter().any(|u| {
            let s = u.as_str();
            s == name
                || s.ends_with(&format!("::{}", name))
                // brace expansion: "error::{AgentError, ToolError}" contains name
                || s.contains(&format!("{{{}}}", name))
                || s.contains(&format!("{}, ", name))
                || s.contains(&format!(", {}", name))
        })
    }

    /// Plan 204 Phase 1B: Return type mapping for function return positions.
    /// Auto `str` (parsed as `StrSlice`) should produce Rust `String` in return
    /// position, while parameters keep `&str` for borrowed semantics.
    pub(super) fn rust_return_type_name(&self, ty: &Type) -> String {
        match ty {
            // str/CStr in return position -> String (owned, safe default)
            Type::StrSlice | Type::CStrLit => "String".to_string(),
            // Option<str> / Option<cstr> -> Option<String>
            // Plan 019: container inner type uses rust_type_name (not
            // rust_return_type_name) because `impl Trait` cannot nest inside
            // generic args (Rust forbids `Option<impl T>`). rust_type_name still
            // maps StrSlice→String, and renders spec names as Box<dyn X>
            // (matching Type::User-via-spec in rust_type_name) instead of the
            // illegal `impl X`.
            Type::Option(inner) => {
                format!("Option<{}>", self.rust_type_name(inner))
            }
            // Result<str> -> Result<String, E> where E is inferred or Box<dyn Error>
            Type::Result(inner) => {
                let err_type = match &self.current_fn_err_type {
                    Some(enum_name) => enum_name.to_string(),
                    None => "Box<dyn std::error::Error>".to_string(),
                };
                format!("Result<{}, {}>", self.rust_type_name(inner), err_type)
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
                // Rust forbids `impl Trait` nested inside generic args
                // (e.g. `Result<impl Trait, E>` is illegal — `impl Trait` is
                // only valid at the top level of a return type). So inner type
                // args use `rust_type_name` (str→String preserved, no `impl`
                // prefix) rather than `rust_return_type_name`. This stops
                // `Result<Node, E>` (Node via `use.rust auto_atom::*`) from
                // becoming `Result<impl Node, E>`.
                let args: Vec<String> = inst.args.iter().map(|t| self.rust_type_name(t)).collect();
                let base = if let Some(ref source) = inst.source {
                    source.short_name().to_string()
                } else {
                    inst.base_name.to_string()
                };
                // Plan 390 §15.10/§15.11 (L2 转正): same Box/Arc + spec/Fn-arg →
                // `dyn` special case as rust_type_name (return position:
                // `Arc<Tool>` → `Arc<dyn Tool>`, `Box<Fn(A)>` → `Box<dyn Fn(A)>`).
                if matches!(base.as_str(), "Box" | "Arc") && inst.args.len() == 1 {
                    if let Some(arg_ty) = inst.args.first() {
                        match arg_ty {
                            Type::Spec(spec) => {
                                // Plan 395-followup: bare `Box<Fn>` → `Box<dyn Fn + Send + Sync>`
                                // (Send for tokio::spawn'd actors; rust-ref parity).
                                if spec.borrow().name == "Fn" {
                                    return format!("{}<dyn Fn + Send + Sync>", base);
                                }
                                return format!("{}<dyn {}>", base, spec.borrow().name);
                            }
                            Type::User(usr) => {
                                // Plan 390 §15.11-followup (cross-module spec): see the
                                // matching block in rust_type_name (return position variant).
                                // A spec IMPORTED via `use mod: Spec` is Type::User here, not
                                // Type::Spec; without this branch `Arc<Tool>` (Tool imported)
                                // rendered `Arc<Box<dyn Tool>>` (double-wrap). Render single-wrap
                                // Arc<dyn T> when the User name is a known spec.
                                let name = usr.name.to_string();
                                if self.spec_decls.contains_key(name.as_str()) {
                                    if name == "Fn" {
                                        return format!("{}<dyn Fn + Send + Sync>", base);
                                    }
                                    return format!("{}<dyn {}>", base, name);
                                }
                                // Not a known spec — fall through.
                            }
                            Type::Fn(params, ret) => {
                                let param_str: Vec<String> =
                                    params.iter().map(|p| self.rust_type_name(p)).collect();
                                let ret_str = if matches!(&**ret, Type::Void) {
                                    String::new()
                                } else {
                                    format!(" -> {}", self.rust_type_name(ret))
                                };
                                // Plan 395-followup: `+ Send + Sync` — closure fields/params may be moved
                                // into a tokio::spawn'd actor (future must be Send); rust-ref uses
                                // `Arc<dyn Fn(...) + Send + Sync>`. Generated closures capture only
                                // fn-pointers/Copy values, so they satisfy the bounds.
                                return format!(
                                    "{}<dyn Fn({}){} + Send + Sync>",
                                    base,
                                    param_str.join(", "),
                                    ret_str
                                );
                            }
                            _ => {}
                        }
                    }
                }
                // Plan 380 P2: ~SpecName in return position → impl SpecName.
                // Auto's ~TraitName means "return something implementing this trait".
                // In Rust that's `impl TraitName`. We detect trait names by:
                // 1. Known spec (declared in this compilation), OR
                // 2. A bare PascalCase ident with no type args that isn't a known
                //    concrete type (String, Vec, Option, Result, etc.) — this covers
                //    external Rust traits like axum's IntoResponse.
                if inst.args.is_empty() {
                    if self.spec_decls.contains_key(base.as_str()) {
                        return format!("impl {}", base);
                    }
                    // Heuristic: bare PascalCase ident that isn't a concrete type
                    // → likely a trait name → prefix `impl`.
                    let is_concrete = matches!(
                        base.as_str(),
                        "String"
                            | "Vec"
                            | "Option"
                            | "Result"
                            | "Box"
                            | "Arc"
                            | "Rc"
                            | "Cell"
                            | "RefCell"
                            | "Mutex"
                    );
                    let is_pascal = base
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    // Plan 018: mirror the Type::User guard — local or imported
                    // concrete types are never traits, even when bare + PascalCase.
                    if (is_pascal && !is_concrete)
                        && !self.local_struct_types.contains(base.as_str())
                        && !self.is_imported_concrete_type(base.as_str())
                    {
                        return format!("impl {}", base);
                    }
                }
                format!("{}<{}>", base, args.join(", "))
            }
            // Tuple: recurse in case inner types need mapping
            Type::Tuple(ts) => {
                let elems: Vec<String> = ts.iter().map(|t| self.rust_return_type_name(t)).collect();
                format!("({})", elems.join(", "))
            }
            // Handle type: recurse for inner type (Plan 387 follow-up P4: maps
            // to the current TaskRef handle; see rust_type_name).
            Type::Handle { task_type } => {
                format!(
                    "a2r_std::task::TaskRef<{}>",
                    self.rust_return_type_name(task_type)
                )
            }
            // Plan 380 P2: Type::User in return position — if it's a bare
            // PascalCase name (not a known concrete type), treat as trait:
            // ~IntoResponse → impl IntoResponse (via Future unwrap in caller).
            Type::User(usr) => {
                let name = usr.name.to_string();
                // PLAN-729 T-05: 服务端文件响应描述符——返回位是具体 a2r-std
                // 类型，不是 trait（PascalCase 启发式会误判成 impl，先例同
                // SqliteDb/RedisClient）。
                if name == "FileResponse" {
                    self.a2r_std_used.set(true);
                    return "a2r_std::http::FileResponse".to_string();
                }

                // PLAN-730 T-06: 上传三类型返回位（含 ~UploadReceipt 解包后）。
                if matches!(
                    name.as_str(),
                    "UploadRequest" | "UploadSession" | "UploadReceipt"
                ) {
                    self.a2r_std_used.set(true);
                    return format!("a2r_std::http::{name}");
                }
                // Plan 417-E3: a generic type parameter of the current fn is
                // never a trait — emit the bare param name (`-> T`), not
                // `impl T` (illegal in Rust and wrong for bounds dispatch).
                if self.current_fn_type_params.contains(usr.name.as_str()) {
                    return name;
                }
                // Plan 417-E2 followup (F1): an associated-type reference in
                // return position renders as the implementer's binding — never
                // as `impl Item` (the PascalCase heuristic below would
                // misfire on the bare assoc name).
                if let Some(bound_ty) = self.current_assoc_bindings.get(usr.name.as_str()) {
                    return self.rust_type_name(bound_ty);
                }
                if self.spec_decls.contains_key(name.as_str()) {
                    return format!("impl {}", name);
                }
                let is_concrete = matches!(
                    name.as_str(),
                    "String"
                        | "Vec"
                        | "Option"
                        | "Result"
                        | "Box"
                        | "Arc"
                        | "Rc"
                        | "Cell"
                        | "RefCell"
                        | "Mutex"
                );
                let is_pascal = name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                // C8: a type declared in this file is a concrete struct, never a
                // trait — skip the `impl` prefix (Option<AgentMode> must stay
                // `Option<AgentMode>`, and `-> AgentMode` must not be `-> impl
                // AgentMode`).
                //
                // Plan 018: also skip `impl` for types that reach this file via
                // imports (`use auto_ai_client: ClientError`, `use error: AgentError`,
                // `use.rust std::path::PathBuf`, …) — they are concrete enums/structs/
                // aliases, not traits. `None` is Auto's unit type (→ Rust `()`),
                // also never a trait despite its uppercase initial.
                //
                // Plan 417-E2: `Self::Item` is an associated-type reference in a
                // trait method signature — concrete once the impl binds it,
                // never a trait. Never prefix with `impl`.
                if self.local_struct_types.contains(name.as_str())
                    || self.is_imported_concrete_type(name.as_str())
                    // Plan 447 ③-P1: locally-declared ENUMS are concrete too
                    // (p_kind/p_peek return TokenKind; `impl TokenKind` is E0404)
                    || self.known_enum_names.contains(name.as_str())
                    || name == "None"
                    || name.starts_with("Self::")
                {
                    self.rust_type_name(ty)
                } else if is_pascal && !is_concrete {
                    format!("impl {}", name)
                } else {
                    self.rust_type_name(ty)
                }
            }
            // Plan 399 Phase 11.3: []T in return position → Vec<T> (owned), not
            // &[T] (borrowed). rust_type_name emits &[T] for slices (correct for
            // params/fields), but a fn returning &[T] over a MutexGuard can't
            // outlive the guard → use owned Vec<T> in return position. byte/Spec
            // already become Vec<u8>/Vec<Box<dyn>> in rust_type_name.
            Type::Slice(slice) => {
                format!("Vec<{}>", self.rust_type_name(&slice.elem))
            }
            // All other types delegate to rust_type_name
            _ => self.rust_type_name(ty),
        }
    }

    /// Parameter type mapping: Auto str → Rust &str for function parameters.
    /// Call sites borrow String args with & prefix.
    pub(super) fn rust_param_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::StrFixed(_) | Type::StrSlice | Type::StrOwned | Type::CStrLit => {
                "&str".to_string()
            }
            _ => self.rust_type_name(ty),
        }
    }

    /// Plan 347: Resolve the effective Rust type name for a function parameter,
    /// applying inferred types for untyped params. Untyped params default to
    /// `int` at parse time; if such a param is matched against `Ok`/`Err`
    /// patterns in the body (i.e. it is in `result_idents`), emit it as
    /// `Result<String, String>` instead.
    pub(super) fn effective_param_type_name(
        &self,
        param: &crate::ast::Param,
        result_idents: &std::collections::HashSet<String>,
    ) -> String {
        if matches!(param.ty, Type::Int) && result_idents.contains(param.name.as_str()) {
            // Inferred Result type for an untyped param matched against Ok/Err.
            return "Result<String, String>".to_string();
        }
        self.rust_param_type_name(&param.ty)
    }

    /// Map Auto builtin type names to their Rust equivalents.
    /// Returns Some(rust_name) if the ident is a builtin type, None otherwise.
    pub(super) fn auto_type_to_rust(name: &str) -> Option<&'static str> {
        match name {
            "List" => Some("Vec"),
            "Map" => Some("HashMap"),
            "Set" => Some("HashSet"),
            _ => None,
        }
    }

    /// Check if a type should use `&mut` in merge mode (context types passed through function chains).
    /// These types are used as mutable state objects in parser/eval/typeinfer chains.
    pub(super) fn is_merge_mut_type(ty: &Type) -> bool {
        match ty {
            Type::User(usr) => matches!(
                usr.name.as_str(),
                "Parser" | "TypeEnv" | "EvalEnv" | "Codegen" | "BVMState"
            ),
            _ => false,
        }
    }

    /// Plan 347: StringBuilder is the Auto VM's shared mutable output buffer.
    /// When it is a function parameter, the transpiler must pass it by `&mut`
    /// reference (NOT by value + `.clone()`), because the parser threads a
    /// single accumulator through recursive calls whose appends must accumulate
    /// into one buffer. By-value + clone compiles only formally and silently
    /// drops every recursive append, producing empty/garbage output.
    ///
    /// This helper selects StringBuilder params for the `&mut` param-emission
    /// and `&mut` call-site path (mirroring the merge-mode context-type path),
    /// so callers pass `&mut sb` and callees declare `sb: &mut a2r_std::StringBuilder`.
    pub(super) fn is_sb_ref_type(ty: &Type) -> bool {
        match ty {
            Type::User(usr) => usr.name.as_str() == "StringBuilder",
            _ => false,
        }
    }

    pub(super) fn is_auto_list_expr(&self, obj: &Expr) -> bool {
        match obj {
            Expr::Ident(name) => {
                // PLAN-019:全局 var List 纳入识别(db.at 平行表接收者)——
                // 仅查局部表时全局 List 的 .set(idx,v) 跌进 Plan 514 W1 的
                // Vec::insert 重映射(插入语义,右移后续元素),多 Tab 模型
                // 表腐坏实测(tab_root_node [3,1] vs 期望 [3,4],UI 挂起)。
                // 镜像 recv_is_list_like 的 PLAN-018 全局覆盖。
                if let Some(ty) = self.global_var_types.get(name.as_str()) {
                    return matches!(ty, Type::List(_) | Type::Array(_));
                }
                self.local_var_types
                    .get(name.as_str())
                    .map(|ty| matches!(ty, Type::List(_)))
                    .unwrap_or(false)
            }
            Expr::Dot(inner, field) => {
                if let Expr::Ident(owner) = inner.as_ref() {
                    if let Some(Type::User(usr)) = self.local_var_types.get(owner.as_str()) {
                        if let Some(fields) = self.struct_field_types.get(usr.name.as_str()) {
                            if fields
                                .iter()
                                .any(|(f, t)| f == field && matches!(t, Type::List(_)))
                            {
                                return true;
                            }
                        }
                    }
                    // fall through: the owner may be resolvable via inference
                    // even when the direct local lookup missed
                }
                // Plan 447 ③-前置(②列基线修复): chained receivers —
                // `a.ens.get(i).pays` — the inner expression's type resolves
                // recursively (242 #18-1 machinery), so field-of-element
                // lookups work instead of falling through to Vec::get
                // (E0277 i64 index / Option compare downstream).
                let inner_ty = self.infer_type_from_expr(inner);
                if let Type::User(usr) = inner_ty {
                    if let Some(fields) = self.struct_field_types.get(usr.name.as_str()) {
                        return fields
                            .iter()
                            .any(|(f, t)| f == field && matches!(t, Type::List(_)));
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check if a type implements Copy (primitive types, string slices, etc.).
    /// Non-Copy types (structs, enums, HashMap, Unknown) need .clone() when moved.
    /// Slice/Array/List are treated as Copy for call-site purposes (passed by reference in Rust).
    /// Unknown is treated as non-Copy for safety (conservative ownership handling).
    pub(super) fn is_copy_type(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Int | Type::Uint | Type::USize | Type::I64 | Type::U64
            | Type::Float | Type::Double | Type::Bool | Type::Char | Type::Byte
            | Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit
            | Type::Void
            | Type::Slice(_) | Type::Array(_) | Type::List(_)
            // Plan 383: Rust 的 fn 指针类型实现 Copy —— 函数引用按值传递，
            // 不需要 .clone()。让 apply(handler) 输出干净的 handler。
            | Type::Fn(_, _)
        )
    }

    /// Plan 016 Phase 4: does a by-value param of this type TAKE OWNERSHIP of
    /// the argument in the emitted Rust? a2r maps Auto `List<T>` params to an
    /// owned `Vec<T>` (NOT `&[T]`), so a List arg MOVES despite is_copy_type's
    /// borrow convention (which models the `str` → `&str` param mapping).
    /// Structs/enums/Maps (already !is_copy_type) move too. Used by the
    /// call-site Dot-arg clone and the defect-4 borrowed-iter-field gate.
    pub(super) fn param_takes_ownership(ty: &Type) -> bool {
        match ty {
            Type::List(_) => true,
            _ => !Self::is_copy_type(ty),
        }
    }

    /// Plan 310 Phase 2: Strict primitive-Copy check for escape-tier codegen.
    /// Only types that are genuinely Copy in Rust AND whose local_var_types
    /// entry reliably matches the generated Rust type. Excludes String/str
    /// (whose Auto type may not match the generated Rust type) and composite
    /// types. Used by emit_borrow to decide "just copy the value".
    pub(super) fn is_primitive_copy(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Int
                | Type::Uint
                | Type::USize
                | Type::I64
                | Type::U64
                | Type::Float
                | Type::Double
                | Type::Bool
                | Type::Char
                | Type::Byte
        )
    }

    /// Plan 310 Phase 4.2: Check if a type contains an *indirect* self-reference
    /// (e.g. `List<Self>`, `Option<Self>`, `Map<_, Self>`). Direct self-reference
    /// (`Type::User(self_name)` without a wrapper) is handled separately as a
    /// hard error. This detects the wrapper cases that compile but may form
    /// reference cycles, triggering a W0008 warning.
    ///
    /// Recurses into List/Option/Result/Map/GenericInstance wrappers. Does NOT
    /// recurse into bare `Type::User(other_type)` — that would require global
    /// type-graph analysis (deferred). Only same-name indirect refs are flagged.
    pub(super) fn type_contains_self_indirect(ty: &Type, self_name: &str) -> bool {
        match ty {
            // Direct self-reference is NOT "indirect" — handled as hard error.
            Type::User(td) if td.name.as_str() == self_name => false,
            // Other user types: don't recurse (no global analysis).
            Type::User(_) => false,
            // Wrapper types: recurse into inner type(s).
            Type::List(inner)
            | Type::Result(inner)
            | Type::Option(inner)
            | Type::Reference(inner)
            | Type::Linear(inner) => {
                // inner is Box<Type>
                Self::type_contains_self_indirect(inner, self_name)
            }
            Type::Map(k, v) => {
                Self::type_contains_self_indirect(k, self_name)
                    || Self::type_contains_self_indirect(v, self_name)
            }
            Type::GenericInstance(inst) => {
                // Check base name (e.g. Vec<Node> where inst.base_name == "Vec")
                // and recurse into type args.
                inst.args
                    .iter()
                    .any(|arg| Self::type_contains_self_indirect(arg, self_name))
            }
            Type::Array(arr) => Self::type_contains_self_indirect(&arr.elem, self_name),
            Type::Tuple(types) => types
                .iter()
                .any(|t| Self::type_contains_self_indirect(t, self_name)),
            _ => false,
        }
    }

    pub(super) fn rust_ident(name: &str) -> std::borrow::Cow<'_, str> {
        // Note: self, super, crate are NOT included — they are path segments
        // that must not be escaped. "Self" (uppercase) is also not escaped
        // since it's used as a type name.
        const RUST_KEYWORDS: &[&str] = &[
            "match", "type", "async", "fn", "let", "if", "else", "for", "while", "loop", "return",
            "break", "continue", "struct", "enum", "trait", "impl", "pub", "mut", "ref", "move",
            "mod", "use", "where", "as", "in", "static", "const", "unsafe", "extern", "dyn",
            // Plan 577 ⑤ (DEBTS 016 Phase 0 / 022 final 行): Rust
            // reserved-for-future words — r#-escapable in all editions;
            // `final` was the 022 breakage (E0530). `gen` (edition-2024
            // keyword) deliberately NOT included: emission targets edition
            // 2021 where `gen` is a valid identifier.
            "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
            "unsized", "virtual", "yield",
        ];
        if RUST_KEYWORDS.contains(&name) {
            std::borrow::Cow::Owned(format!("r#{}", name))
        } else {
            std::borrow::Cow::Borrowed(name)
        }
    }

    /// Extract the type name from a Type (for qualified key lookup).
    pub(super) fn type_name_of(ty: &Type) -> Option<String> {
        match ty {
            Type::User(td) => Some(td.name.to_string()),
            Type::GenericInstance(inst) => Some(inst.base_name.to_string()),
            _ => None,
        }
    }

    /// PLAN-010 T2: is this expression's type a known VALUE scalar (int/
    /// float/bool/char/byte)? Only such receivers can never be Option/Result
    /// in the generated Rust, so `x.?` / `x ?? d` lower to the bare value.
    /// Str-family is deliberately excluded: a str-typed iter var materializes
    /// as `Result<DirEntry, io::Error>` in read_dir loops (cookbook/file),
    /// where `entry?` is real propagation. Unknown/composite types keep the
    /// legacy untyped lowering.
    pub(super) fn expr_type_is_known_scalar(&self, e: &Expr) -> bool {
        let ty = match e {
            Expr::Ident(n) => match self.local_var_types.get(n.as_str()) {
                Some(t) => t.clone(),
                None => return false,
            },
            _ => self.infer_type_from_expr(e),
        };
        matches!(
            ty,
            Type::Int
                | Type::Uint
                | Type::USize
                | Type::I64
                | Type::U64
                | Type::Float
                | Type::Double
                | Type::Bool
                | Type::Char
                | Type::Byte
        )
    }

    /// Infer Rust type from an Auto expression (for let-bound variables without type annotation).
    pub(super) fn infer_type_from_expr(&self, expr: &Expr) -> Type {
        match expr {
            // Plan 523 H1: struct 构造字面量 `Name { field: e }` 解析为
            // Expr::Node(Pair args)。未注解 `let p = Point { ... }` 此前推成
            // Unknown,433 A1 字段优先护栏(object_has_such_field)落空——
            // 字段名撞内置方法名(count/len/push 族)时发射成方法调用
            // (b35 考古: `c.count() = c.count() + c.step`)。已知 struct 名
            // 即登记 User 占位类型,护栏按名命中。
            Expr::Node(node) => {
                if self.struct_fields.contains_key(&node.name)
                    || self.struct_field_types.contains_key(&node.name)
                {
                    Self::placeholder_user_type(node.name.clone())
                } else {
                    Type::Unknown
                }
            }
            Expr::Call(call) => {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    // method is AutoStr, obj is Expr
                    match method.as_str() {
                        // Methods that return String
                        "substr" | "sub" | "slice" | "to_lower" | "to_upper" | "trim"
                        | "trim_left" | "trim_right" | "to_string" | "replace"
                        | "replace_first" | "repeat" => {
                            return Type::StrOwned;
                        }
                        // Plan 417-E1 (DIV-A2R-CHAR-AT-1): `char_at` returns the
                        // code point as an int (the emitter writes `... as i32`;
                        // the VM types it as int natively). It was wrongly listed
                        // with the String-returning methods, so
                        // `c = s.char_at(i); c = c + 32` bound c as a String and
                        // the add emitted `format!` concatenation (E0308). The
                        // parity string_utils lib worked around it with explicit
                        // `var c int` annotations.
                        "char_at" => {
                            return Type::Int;
                        }
                        // PLAN-781 阶段 E：byte_len 返回 UTF-8 字节数（i32）。
                        "byte_len" => {
                            return Type::Int;
                        }
                        // Plan 434 (242 #18-1): `.clone()` preserves the
                        // receiver's type; a List `.get(i)` yields the elem.
                        // Both feed the index/clone-chain inference above.
                        "clone" => {
                            return self.infer_type_from_expr(obj);
                        }
                        // Plan 016 Phase 4: `opt.unwrap_or(d)` /
                        // `opt.unwrap_or_else(f)` yield the Option's inner
                        // type (fall back to the default arg's type) — locals
                        // bound from them (`let t = found.unwrap_or(missing())`)
                        // were Unknown, so the struct/enum move guards (e.g.
                        // the vec![t] element clone) missed them (E0382).
                        "unwrap_or" => {
                            if let Type::Option(inner) = self.infer_type_from_expr(obj) {
                                return *inner;
                            }
                            if let Some(Arg::Pos(d)) = call.args.args.first() {
                                let d_ty = self.infer_type_from_expr(d);
                                if !matches!(d_ty, Type::Unknown) {
                                    return d_ty;
                                }
                            }
                        }
                        "unwrap_or_else" => {
                            if let Type::Option(inner) = self.infer_type_from_expr(obj) {
                                return *inner;
                            }
                        }
                        // Plan 019 Phase 1: split renders as
                        // `.map(|s| s.to_string()).collect::<Vec<String>>()`
                        // — type it so index args resolve their element type
                        // for the &str borrow gates (an Unknown left the
                        // `.as_str()` adaptation dark).
                        "split" => {
                            return Type::List(Box::new(Type::StrOwned));
                        }
                        // Plan 531 P523-2①: `process.args()` = List 契约
                        // (P524:[程序路径]+CLI 透传;VM 参考侧 shim_process_args
                        // 同形,a2r 侧由宿主 shim 提供)。绑定型 List<str> 驱动
                        // .get(i) 索引化 + 实参 .as_str() 借用——aavm.at 入口
                        // `argv.get(1)` 此前 Unknown 落 Vec::get Option 形态
                        // (E0308)。`env.args`(a2r_std String 契约)不在列。
                        "args" => {
                            if matches!(obj.as_ref(),
                                Expr::Ident(n) if n.as_str() == "process")
                            {
                                return Type::List(Box::new(Type::StrOwned));
                            }
                        }
                        "get" => {
                            if let Type::List(elem) = self.infer_type_from_expr(obj) {
                                return *elem;
                            }
                        }
                        // stdlib module functions that return String
                        _ => {
                            if let Expr::Ident(module) = obj.as_ref() {
                                match (module.as_str(), method.as_str()) {
                                    ("json", "as_string")
                                    | ("json", "get_str")
                                    | ("json", "to_string")
                                    | ("json", "keys")
                                    | ("fs", "read_text")
                                    | ("fs", "read_to_string")
                                    | ("fs", "walk")
                                    | ("shell", "exec")
                                    | ("regex", "find_all")
                                    | ("io", "read_line")
                                    | ("env", "args")
                                    | ("env", "get") => {
                                        return Type::StrOwned;
                                    }
                                    _ => {}
                                }
                            }
                            // Plan 514 W3: 用户方法(`p.text()` 族)——接收者
                            // 类型已知时经 fn_ret_types 的 "Type.method"
                            // qualified 键解析声明返回(裸方法名兜底),镜像
                            // Ident 调用臂。方法化 lib 的 `var x = p.text()`
                            // 局部此前推断为 Unknown,赋值借用/实参强转/
                            // string_like 判定全套哑火(E0382/E0308/E0658)。
                            let type_name = if let Expr::Ident(n) = obj.as_ref() {
                                match self.local_var_types.get(n) {
                                    Some(Type::User(td)) => Some(td.name.to_string()),
                                    Some(Type::Enum(d)) => Some(d.borrow().name.to_string()),
                                    _ => None,
                                }
                            } else {
                                None
                            };
                            let ret = type_name
                                .and_then(|t| {
                                    let key: AutoStr = format!("{}.{}", t, method).into();
                                    self.fn_ret_types.get(&key).cloned()
                                })
                                .or_else(|| self.fn_ret_types.get(method).cloned());
                            if let Some(ret) = ret {
                                if !matches!(ret, Type::Unknown | Type::Void) {
                                    return ret;
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
                    // Plan 310 Phase 0.2: Union construction infers its type
                    // so that downstream field-access sites can detect union vars.
                    if self.union_types.contains(fn_name) {
                        // Reconstruct a minimal Type::User carrying the union name;
                        // only the name field is consulted by field-access detection.
                        return Type::User(crate::ast::TypeDecl::builtin(fn_name.as_str()));
                    }
                    // Plan 387 follow-up: a user-struct constructor call
                    // (`Worker(...)`) infers the struct type, so locals bound
                    // from it (e.g. `let w = Worker(...)`) resolve field-access
                    // types (incl. TaskRef fields) instead of staying Unknown.
                    if self.struct_fields.contains_key(fn_name) {
                        return Type::User(crate::ast::TypeDecl::builtin(fn_name.as_str()));
                    }
                    // Plan 016 Phase 4: a user fn with a DECLARED return type
                    // (`missing() -> Node`, `findBlock(..) -> BlockNode?`) —
                    // the prescan registers every fn's ret, so locals bound
                    // from such calls resolve instead of staying Unknown.
                    if let Some(ret) = self.fn_ret_types.get(fn_name.as_str()) {
                        if !matches!(ret, Type::Unknown | Type::Void) {
                            return ret.clone();
                        }
                    }
                }
                Type::Unknown
            }
            Expr::Array(items) => {
                // Array literal — infer as List if items exist
                if let Some(first) = items.first() {
                    let elem_ty = self.infer_type_from_expr(first);
                    Type::List(Box::new(elem_ty))
                } else {
                    Type::Unknown
                }
            }
            Expr::Tuple(items) => {
                // Tuple literal — infer as Type::Tuple (Plan 018 §14 W1). Without
                // this, `let key = (a, b, c)` is Unknown, so the HashMap insert
                // key handler's "key is String" assumption wrongly appends
                // `.to_string()` (E0277: tuple has no Display).
                let elem_types: Vec<Type> =
                    items.iter().map(|e| self.infer_type_from_expr(e)).collect();
                Type::Tuple(elem_types)
            }
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => Type::StrSlice,
            Expr::Int(_) => Type::Int,
            Expr::Float(_, _) => Type::Float,
            Expr::Bool(_) => Type::Bool,
            Expr::NullCoalesce(lhs, _rhs) => {
                // ?? unwraps Option — infer the inner type from lhs
                let lhs_ty = self.infer_type_from_expr(lhs);
                match lhs_ty {
                    Type::Option(inner_ty) => *inner_ty,
                    other => other,
                }
            }
            Expr::Dot(obj, field) => {
                // Infer type from struct field access: obj.field
                if let Expr::Ident(var_name) = obj.as_ref() {
                    // Check local variable types first
                    if let Some(var_ty) = self.local_var_types.get(var_name) {
                        let type_name = match var_ty {
                            Type::User(td) => td.name.clone(),
                            Type::Enum(ed) => ed.borrow().name.clone(),
                            Type::GenericInstance(inst) => inst.base_name.clone(),
                            _ => var_ty.unique_name(),
                        };
                        if let Some(fields) = self.struct_field_types.get(&type_name) {
                            for (fname, fty) in fields {
                                if fname.as_str() == field.as_str() {
                                    return fty.clone();
                                }
                            }
                        }
                    }
                    // Check if variable name matches a known struct (for dot-access on params)
                    if let Some(fields) = self.struct_field_types.get(var_name.as_str()) {
                        for (fname, fty) in fields {
                            if fname.as_str() == field.as_str() {
                                return fty.clone();
                            }
                        }
                    }
                }
                // Plan 434 (242 #18-1): generalize — resolve the receiver's
                // type recursively (e.g. `a.fns[i].clone().params`), then look
                // the field up in struct_field_types. Covers field chains whose
                // object is itself an index/clone/call expression; previously
                // only bare-Ident receivers resolved.
                let obj_ty = self.infer_type_from_expr(obj);
                if !matches!(obj_ty, Type::Unknown) {
                    let type_name = match &obj_ty {
                        Type::User(td) => td.name.clone(),
                        Type::Enum(ed) => ed.borrow().name.clone(),
                        Type::GenericInstance(inst) => inst.base_name.clone(),
                        _ => obj_ty.unique_name(),
                    };
                    if let Some(fields) = self.struct_field_types.get(&type_name) {
                        for (fname, fty) in fields {
                            if fname.as_str() == field.as_str() {
                                return fty.clone();
                            }
                        }
                    }
                }
                Type::Unknown
            }
            // Plan 376F: Infer type from plain identifier via local_var_types
            Expr::Ident(name) => {
                if let Some(ty) = self.local_var_types.get(name) {
                    return ty.clone();
                }
                // Plan 389 R2: a bare function name in value position (e.g. a
                // task state field default `cb = noop_event`) is a fn item —
                // infer its fn-pointer type so the field isn't emitted as
                // `/* unknown */`. Param types come from the prescan
                // (fn_param_types); the return type from the Plan 373 cache
                // (also prescanned by Plan 389), defaulting to Void.
                if let Some(params) = self.fn_param_types.get(name) {
                    let ret = self.fn_ret_types.get(name).cloned().unwrap_or(Type::Void);
                    return Type::Fn(params.clone(), Box::new(ret));
                }
                Type::Unknown
            }
            // Plan 376F: Binary arithmetic — infer from operands
            Expr::Bina(lhs, op, rhs) => {
                // For arithmetic ops, the result type follows the "wider" operand.
                let lt = self.infer_type_from_expr(lhs);
                let rt = self.infer_type_from_expr(rhs);
                match op {
                    auto_val::Op::Add
                    | auto_val::Op::Sub
                    | auto_val::Op::Mul
                    | auto_val::Op::Div
                    | auto_val::Op::Mod => {
                        // If either is float, result is float; otherwise follow int/uint
                        if matches!(lt, Type::Float | Type::Double)
                            || matches!(rt, Type::Float | Type::Double)
                        {
                            Type::Float
                        } else if matches!(lt, Type::Uint) && matches!(rt, Type::Uint) {
                            Type::Uint
                        } else if matches!(lt, Type::Int) && matches!(rt, Type::Int) {
                            Type::Int
                        } else if matches!(lt, Type::Uint) || matches!(rt, Type::Uint) {
                            Type::Uint // mixed int/uint → uint (Auto semantics)
                        } else if matches!(lt, Type::Int) || matches!(rt, Type::Int) {
                            Type::Int
                        } else if !matches!(lt, Type::Unknown) {
                            lt
                        } else {
                            rt
                        }
                    }
                    _ => Type::Unknown,
                }
            }
            // Plan 376F: Cast expression (x as T) → T
            Expr::Cast { target_type, .. } => target_type.clone(),
            // Plan 434 (242 #18-1): indexing a List infers the element type,
            // so locals bound through `x.f[i].clone()` chains keep their
            // struct type (previously the whole chain collapsed to Unknown,
            // and `.get()` on such locals fell to the Option-form
            // post-process regexes instead of AST-level index+clone).
            Expr::Index(array, _idx) => match self.infer_type_from_expr(array) {
                Type::List(elem) => *elem,
                _ => Type::Unknown,
            },
            // Plan 242 #8: closure literal → Fn(param types, ret). Previously
            // fell to Unknown, so let-bound closures / closure-typed fields
            // lost their type (masked by task-struct special-casing).
            Expr::Closure(closure) => {
                let params: Vec<Type> = closure
                    .params
                    .iter()
                    .map(|p| p.ty.clone().unwrap_or(Type::Unknown))
                    .collect();
                let ret = closure.ret.clone().unwrap_or(Type::Void);
                Type::Fn(params, Box::new(ret))
            }
            _ => Type::Unknown,
        }
    }

    /// Plan 396 §2.1: element type of an iterated collection expression.
    /// `coll_var` (List<T>/[]T/[N]T) or `base.field` where base's struct
    /// field is a collection. Returns None when the chain can't be resolved.
    pub(super) fn iter_elem_type(&self, range: &Expr) -> Option<Type> {
        fn elem_of(ty: &Type) -> Option<Type> {
            match ty {
                Type::List(e) => Some((**e).clone()),
                Type::Slice(sl) => Some((*sl.elem).clone()),
                Type::Array(a) => Some((*a.elem).clone()),
                _ => None,
            }
        }
        match range {
            Expr::Ident(name) => {
                let ty = self.local_var_types.get(name.as_str())?;
                elem_of(ty)
            }
            Expr::Dot(base, field) => {
                let base_name = match base.as_ref() {
                    Expr::Ident(n) => n,
                    _ => return None,
                };
                let base_ty = self.local_var_types.get(base_name.as_str())?;
                match base_ty {
                    Type::User(type_decl) => {
                        let fields = self.struct_field_types.get(type_decl.name.as_str())?;
                        let (_, f_ty) =
                            fields.iter().find(|(n, _)| n.as_str() == field.as_str())?;
                        elem_of(f_ty)
                    }
                    other => elem_of(other),
                }
            }
            _ => None,
        }
    }

    /// Derive the Rust message type for a task's `on` block (Plan 387 D2).
    /// Returns `(type_name, optional_enum_source)`:
    ///   - all int literals  → (`i64`, None)
    ///   - all string literals → (`String`, None)
    ///   - any named variant (Simple/WithBindings) → (`<Task>Msg`, Some(enum source))
    ///     mixing int/string literals with named variants adds a `Literal(..)` arm.
    /// TypeBinding (Tier 2 basic) → scalar of the bound type.
    pub(super) fn derive_task_msg_type(
        &self,
        task_name: &crate::ast::Name,
        on: &TaskOnBlock,
    ) -> AutoResult<(String, Option<String>)> {
        use TaskMsgPattern as P;
        // Classify patterns.
        let mut has_named = false;
        let mut has_int_lit = false;
        let mut has_str_lit = false;
        let mut has_bool_lit = false;
        for (p, _, _) in &on.handlers {
            match p {
                P::Simple(_) | P::WithBindings { .. } => has_named = true,
                P::Literal(LiteralValue::Int(_)) => has_int_lit = true,
                P::Literal(LiteralValue::String(_)) => has_str_lit = true,
                P::Literal(LiteralValue::Bool(_)) => has_bool_lit = true,
                P::TypeBinding { type_expr, .. } => {
                    // Map a few common Auto types to Rust scalars; default i64.
                    let t = self.rust_type_name(type_expr);
                    return Ok((t, None));
                }
                _ => {}
            }
        }

        // Pure-scalar cases (no named variants).
        if !has_named {
            if has_int_lit && !has_str_lit && !has_bool_lit {
                return Ok(("i64".to_string(), None));
            }
            if has_str_lit && !has_int_lit && !has_bool_lit {
                return Ok(("String".to_string(), None));
            }
            if has_bool_lit && !has_int_lit && !has_str_lit {
                return Ok(("bool".to_string(), None));
            }
        }

        // Named variants (or mixed) → generate an enum.
        let enum_name = format!("{}Msg", name_of(task_name));
        let mut src = String::new();
        src.push_str("#[derive(Clone, Debug)]\n");
        src.push_str(&format!("enum {} {{\n", enum_name));
        // Collect variants from named patterns (dedup by variant name).
        let mut seen: Vec<String> = Vec::new();
        for (p, _, _) in &on.handlers {
            match p {
                P::Simple(vname) => {
                    let n = vname.as_str().to_string();
                    if !seen.contains(&n) {
                        seen.push(n.clone());
                        src.push_str(&format!("    {},\n", n));
                    }
                }
                P::WithBindings { variant, bindings } => {
                    let n = variant.as_str().to_string();
                    if !seen.contains(&n) {
                        seen.push(n.clone());
                        // Plan 387 follow-up P3: use the DECLARED binding type
                        // when given (`Add(val: String)` → `Add(String)`); fall
                        // back to i64 for untyped bindings (most common; VM
                        // tests use ints).
                        let fields: Vec<String> = bindings
                            .iter()
                            .map(|(_, bty)| match bty {
                                Some(ty) => self.rust_type_name(ty),
                                None => "i64".to_string(),
                            })
                            .collect();
                        if fields.is_empty() {
                            src.push_str(&format!("    {},\n", n));
                        } else {
                            src.push_str(&format!("    {}({}),\n", n, fields.join(", ")));
                        }
                    }
                }
                _ => {}
            }
        }
        // Mixed literals → fold into a single Literal variant.
        if has_int_lit || has_str_lit || has_bool_lit {
            if has_str_lit {
                src.push_str("    Literal(String),\n");
            } else if has_bool_lit {
                src.push_str("    Literal(bool),\n");
            } else {
                src.push_str("    Literal(i64),\n");
            }
        }
        src.push_str("}\n\n");
        Ok((enum_name, Some(src)))
    }

    /// Plan 390 §15.10: Rust type of a task state field whose default is a
    /// closure literal — `Box<dyn Fn(params) -> ret>` (closures capture, unlike
    /// fn-pointers). Built as `Box<Fn(...)>` and rendered through the Box/Arc
    /// special case so the output is byte-identical to an explicit `Box<Fn>`
    /// annotation.
    pub(super) fn closure_field_rust_type(&self, closure: &crate::ast::Closure) -> String {
        let params: Vec<crate::ast::Type> = closure
            .params
            .iter()
            .map(|p| p.ty.clone().unwrap_or(crate::ast::Type::Unknown))
            .collect();
        let ret = closure.ret.clone().unwrap_or(crate::ast::Type::Void);
        let boxed_ty = crate::ast::Type::GenericInstance(crate::ast::GenericInstance {
            base_name: "Box".into(),
            args: vec![crate::ast::Type::Fn(params, Box::new(ret))],
            source: None,
        });
        self.rust_type_name(&boxed_ty)
    }

    /// Plan 514 W1 → Plan 523 H1 提升为关联函数: minimal User 占位类型
    /// (名字即类型;按名查询注册表生效)。原 nested fn 于 scan_call_init_bindings
    /// 内,H1(struct 构造字面量类型推断)同需。
    pub(super) fn placeholder_user_type(name: AutoStr) -> Type {
        Type::User(TypeDecl {
            consts: Vec::new(),
            name,
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(),
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
            attrs: Vec::new(),
            impl_attrs: Vec::new(),
            doc: None,
            is_pub: false,
        })
    }

    /// Plan 523 H3: 未注解 fn 的返回类型推断——扫描 body 的 return 表达式
    /// （嵌套控制流内）与顶层尾表达式，经 infer_type_from_expr 推型。首个
    /// 非 Unknown 型胜出；形态冲突（discriminant 不同）回退 Unknown（保守）。
    /// b38 考古：`fn mk() { return [7, 8, 9] }` 此前发射 unit，
    /// `for w in mk()` rustc E0308。
    pub(super) fn infer_fn_ret_type(&self, body: &crate::ast::Body) -> Type {
        let mut found: Option<Type> = None;
        let mut conflict = false;
        fn visit_stmts(
            tr: &RustTrans,
            stmts: &[Stmt],
            found: &mut Option<Type>,
            conflict: &mut bool,
        ) {
            for stmt in stmts {
                match stmt {
                    Stmt::Return(ret) => {
                        let t = tr.infer_type_from_expr(ret);
                        if !matches!(t, Type::Unknown | Type::Void) {
                            match found {
                                Some(prev) => {
                                    if std::mem::discriminant(prev) != std::mem::discriminant(&t) {
                                        *conflict = true;
                                    }
                                }
                                None => *found = Some(t),
                            }
                        }
                    }
                    Stmt::If(if_) => {
                        for b in &if_.branches {
                            visit_stmts(tr, &b.body.stmts, found, conflict);
                        }
                        if let Some(e) = &if_.else_ {
                            visit_stmts(tr, &e.stmts, found, conflict);
                        }
                    }
                    Stmt::For(f) => visit_stmts(tr, &f.body.stmts, found, conflict),
                    Stmt::Block(b) => visit_stmts(tr, &b.stmts, found, conflict),
                    Stmt::Try(t) => {
                        visit_stmts(tr, &t.body.stmts, found, conflict);
                        visit_stmts(tr, &t.catch_body.stmts, found, conflict);
                    }
                    _ => {}
                }
            }
        }
        visit_stmts(self, &body.stmts, &mut found, &mut conflict);
        if conflict {
            return Type::Unknown;
        }
        // 只认显式 return 语句——顶层尾表达式（语句位调用值弃置常见）
        // 假阳率高，不入本推断面（parser.rs 8720 同款 TODO 的保守子集）。
        found.unwrap_or(Type::Unknown)
    }

    /// PLAN-592: use.rs 导入的类型名(uses 中的大写首字母项)。供
    /// scan_mutated_bindings 的 rust 型绑定保守 mut 标记。std 包装器/
    /// 智能指针族排除——它们已有专用发射路径与既有 golden 校准
    /// (17_rust_std/003_sync:Arc 的 lock() 是 &self,保守 mut 反而错)。
    pub(super) fn rust_imported_type_names(&self) -> std::collections::HashSet<AutoStr> {
        /// 已知 &self 语义的面(方法调用不需接收者 mut)——由既有 a2r/cookbook
        /// golden 校准(全量 golden 扫描:immutable let + 方法调用的 init 类型)
        const STD_EXEMPT: &[&str] = &[
            "String",
            "Vec",
            "Arc",
            "Mutex",
            "Box",
            "Rc",
            "RefCell",
            "OnceCell",
            "Cell",
            "Duration",
            "Instant",
            "PathBuf",
            "Path",
            "Command",
            "SystemTime",
            "Option",
            "Result",
            "Cow",
            // chrono / io / regex / url / semver / num 家族(golden 校准)
            "Utc",
            "Local",
            "DateTime",
            "NaiveDateTime",
            "NaiveDate",
            "NaiveTime",
            "BufReader",
            "BufWriter",
            "Regex",
            "Url",
            "Version",
            "VersionReq",
            "Complex",
        ];
        self.uses
            .iter()
            .filter(|u| {
                u.as_str()
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                    && !STD_EXEMPT.contains(&u.as_str())
            })
            .cloned()
            .collect()
    }

    /// Plan 610 ⑤: pointer-param classification for `#[export]` fns.
    /// `*T` with a NAMED pointee (local/foreign struct) is a HANDLE (opaque
    /// session): the body receives `&mut T` via a null-safe wrapper deref.
    /// Builtin-scalar/void pointees are BUFFER pointers — passed through raw,
    /// element access goes through the generated kit (`cabi_*` helpers).
    /// `*void` and bare `ptr` (unresolved ident) both mean opaque `*mut c_void`
    /// buffers here.
    pub(super) fn export_ptr_pointee_is_handle(ty: &Type) -> bool {
        if let Type::Ptr(p) = ty {
            match &*p.of.borrow() {
                Type::User(usr) => usr.name.as_str() != "ptr",
                Type::Rust(_) => true,
                _ => false,
            }
        } else {
            false
        }
    }

}
