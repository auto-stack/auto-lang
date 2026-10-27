//! PLAN-843 结构拆分（自 codegen.rs 原样搬移；算法/行为零变化）。
use super::*;

impl Codegen {
    /// PLAN-596 T4:类型名是否 dep 声明 crate 的面(仅 dep 侧可发 mono 后缀)。
    pub(super) fn type_is_dep_face(&self, type_name: &str) -> bool {
        match self.rust_native_map.get(type_name) {
            Some((crate_name, _)) => self.declared_dep_crates.contains(crate_name),
            None => false,
        }
    }

    // Plan 087 Phase 3: Infer expression type using the infer module
    // Returns the inferred type
    pub(super) fn infer_expr_type(&mut self, expr: &Expr) -> crate::ast::Type {
        // Handle Dot expressions for field access type inference (e.g., a.x.type)
        if let Expr::Dot(obj, field) = expr {
            // First infer the type of the object (e.g., 'a' in 'a.x')
            let obj_ty = self.infer_expr_type(obj);

            // Look up field type from type template
            let field_name = field.as_ref();

            match &obj_ty {
                // User-defined types (type A { x int })
                Type::User(type_decl) => {
                    if let Some(member) = type_decl
                        .members
                        .iter()
                        .find(|m| m.name.as_ref() == field_name)
                    {
                        return member.ty.clone();
                    }
                }
                // Generic instances (Point<int>, List<int>, etc.)
                Type::GenericInstance(inst) => {
                    if let Some(template) = self
                        .generic_registry
                        .get_template(&inst.base_name.to_string())
                    {
                        if let Some(field_def) =
                            template.fields.iter().find(|f| f.name == field_name)
                        {
                            // Substitute type parameters with actual types
                            let generic_params: Vec<crate::ast::Name> = template
                                .generic_params
                                .iter()
                                .filter_map(|p| match p {
                                    crate::ast::GenericParam::Type(tp) => Some(tp.name.clone()),
                                    crate::ast::GenericParam::Const(_) => None,
                                })
                                .collect();
                            return field_def.field_type.substitute(&generic_params, &inst.args);
                        }
                    }
                }
                _ => {}
            }
            // Fall through to default inference if field type not found
        }

        // Check if this is a function call and we know the return type
        if let Expr::Call(call) = expr {
            // Extract function name from call expression
            let func_name = match call.name.as_ref() {
                Expr::Ident(name) => Some(name.to_string()),
                // Plan 197 Bug D: resolve method call return types (e.g., Tool.echo())
                Expr::Dot(obj, method) => Some(format!("{}.{}", self.expr_to_name(obj), method)),
                _ => None,
            };

            if let Some(name) = func_name {
                if let Some(ret_ty) = self.fn_return_types.get(&name) {
                    // We know the return type from fn_return_types
                    // Sync with infer_ctx for future lookups
                    let key = crate::ast::Name::from(&name);
                    if !self.infer_ctx.type_env.contains_key(&key) {
                        self.infer_ctx.type_env.insert(key, ret_ty.clone());
                    }
                    return ret_ty.clone();
                }
            }
        }

        // Ensure infer_ctx type_env is synced with var_types
        for (name, ty) in &self.var_types {
            let key = crate::ast::Name::from(name.as_str());
            if !self.infer_ctx.type_env.contains_key(&key) {
                self.infer_ctx.type_env.insert(key, ty.clone());
            }
        }
        // Use the infer module's comprehensive type inference
        infer_expr(&mut self.infer_ctx, expr)
    }

    // Plan 073 Stage A.5: Check if we should use float/double arithmetic
    // Returns true if either operand is a float/double or contains one recursively
    pub(super) fn is_float_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        // Check if either operand is a float/double literal or contains one recursively
        self.contains_float(lhs) || self.contains_float(rhs)
    }

    // Plan 117: Recursively check if expression contains float/double literals or variables
    pub(super) fn contains_float(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Float(_, _) | Expr::Double(_, _) => true,
            Expr::Ident(name) => self
                .var_types
                .get(name.as_ref())
                .map(|t| matches!(t, Type::Float | Type::Double))
                .unwrap_or(false),
            Expr::Bina(lhs, _, rhs) => self.contains_float(lhs) || self.contains_float(rhs),
            Expr::Unary(_, inner) => self.contains_float(inner),
            Expr::Block(body) => body.stmts.iter().any(|s| self.stmt_contains_float(s)),
            Expr::Dot(obj, field) => self.contains_float(obj) || self.field_is_float(obj, field),
            Expr::Call(call) => call
                .args
                .args
                .iter()
                .map(|a| a.get_expr())
                .any(|e| self.contains_float(&e)),
            _ => false,
        }
    }

    // Plan 117: Check if a statement contains float expressions
    fn stmt_contains_float(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e) => self.contains_float(e),
            Stmt::Store(s) => self.contains_float(&s.expr),
            _ => false,
        }
    }

    // Plan 073 Stage A.5: Check if we should use double precision (f64) vs float (f32)
    pub(super) fn is_double_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        self.contains_double(lhs) || self.contains_double(rhs)
    }

    // Plan 117: Recursively check if expression contains double literals
    pub(super) fn contains_double(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Double(_, _) => true,
            Expr::Ident(name) => self
                .var_types
                .get(name.as_ref())
                .map(|t| matches!(t, Type::Double))
                .unwrap_or(false),
            Expr::Bina(lhs, _, rhs) => self.contains_double(lhs) || self.contains_double(rhs),
            Expr::Unary(_, inner) => self.contains_double(inner),
            Expr::Block(body) => body.stmts.iter().any(|s| self.stmt_contains_double(s)),
            Expr::Dot(obj, field) => self.contains_double(obj) || self.field_is_double(obj, field),
            Expr::Call(call) => call
                .args
                .args
                .iter()
                .map(|a| a.get_expr())
                .any(|e| self.contains_double(&e)),
            _ => false,
        }
    }

    // Plan 117: Check if a statement contains double expressions
    fn stmt_contains_double(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e) => self.contains_double(e),
            Stmt::Store(s) => self.contains_double(&s.expr),
            _ => false,
        }
    }

    fn field_is_float(&self, obj: &Expr, field: &crate::ast::Name) -> bool {
        if let Expr::Ident(type_name) = obj {
            let type_name_str = type_name.as_ref();
            if let Some(vt) = self.var_types.get(type_name_str) {
                let type_name_for_lookup = match vt {
                    Type::User(td) => td.name.as_ref(),
                    _ => type_name_str,
                };
                if let Some(template) = self.generic_registry.get_template(type_name_for_lookup) {
                    for f in &template.fields {
                        if f.name == field.as_ref() {
                            return matches!(f.field_type, Type::Float | Type::Double);
                        }
                    }
                }
            }
        }
        false
    }

    fn field_is_double(&self, obj: &Expr, field: &crate::ast::Name) -> bool {
        if let Expr::Ident(type_name) = obj {
            let type_name_str = type_name.as_ref();
            if let Some(vt) = self.var_types.get(type_name_str) {
                let type_name_for_lookup = match vt {
                    Type::User(td) => td.name.as_ref(),
                    _ => type_name_str,
                };
                if let Some(template) = self.generic_registry.get_template(type_name_for_lookup) {
                    for f in &template.fields {
                        if f.name == field.as_ref() {
                            return matches!(f.field_type, Type::Double);
                        }
                    }
                }
            }
        }
        false
    }

    // Plan 117: Check if expression is an integer type that needs coercion to float
    pub(super) fn needs_float_coercion(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Int(_) | Expr::I8(_) | Expr::Byte(_) | Expr::U8(_) => true,
            Expr::Ident(name) => {
                // Check variable type from type inference
                self.var_types
                    .get(name.as_ref())
                    .map(|t| matches!(t, Type::Int | Type::Byte))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    // Plan 117: Check if expression is an i64/u64 type that needs coercion to f64
    pub(super) fn needs_double_coercion(&self, expr: &Expr) -> bool {
        match expr {
            Expr::I64(_)
            | Expr::U64(_)
            | Expr::Int(_)
            | Expr::I8(_)
            | Expr::U8(_)
            | Expr::Byte(_)
            | Expr::Uint(_) => true,
            Expr::Float(_, _) => true, // f32 needs promotion to f64
            Expr::Ident(name) => self
                .var_types
                .get(name.as_ref())
                .map(|t| {
                    matches!(
                        t,
                        Type::I64 | Type::U64 | Type::Int | Type::Float | Type::Uint | Type::USize
                    )
                })
                .unwrap_or(false),
            // Plan 378: method call `recv.method()` returning an integer or
            // float must be promoted to f64 before double arithmetic
            // (e.g. `s.to_uint() + 1.5` needs U64_TO_F64 / I64_TO_F64).
            Expr::Dot(receiver, method) => self
                .lookup_dot_method_type(receiver.as_ref(), method.as_ref())
                .map(|t| {
                    matches!(
                        t,
                        Type::I64
                            | Type::U64
                            | Type::Int
                            | Type::Byte
                            | Type::Uint
                            | Type::USize
                            | Type::Float
                    )
                })
                .unwrap_or(false),
            Expr::Call(call) => {
                if let Expr::Dot(receiver, method) = call.name.as_ref() {
                    self.lookup_dot_method_type(receiver.as_ref(), method.as_ref())
                        .map(|t| {
                            matches!(
                                t,
                                Type::I64
                                    | Type::U64
                                    | Type::Int
                                    | Type::Byte
                                    | Type::Uint
                                    | Type::USize
                                    | Type::Float
                            )
                        })
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Plan 378: Look up the return type of a `receiver.method()` call.
    ///
    /// Shared by the 64-bit detection helpers (`contains_u64`, `is_u64_expr`,
    /// `needs_double_coercion`) so method calls returning I64/U64 (e.g.
    /// `"42".to_uint()`) are recognised and given the correct 2-slot stack
    /// treatment. The key fallback order mirrors the existing method-return
    /// lookup at line ~1637-1642 and the qualifier forms populated by
    /// `build_fn_return_types` / `enrich_fn_return_types_from_type_store`
    /// (`{type}.{method}`, `{Type}.{method}`, `auto.{type}.{method}`, bare
    /// `{method}`), plus the receiver-qualified `{varname}.{method}` form.
    pub(super) fn lookup_dot_method_type(&self, receiver: &Expr, method: &str) -> Option<&Type> {
        // 1. Receiver-name-qualified: "s.to_uint" (matches when the call site
        //    itself was registered, or TypeStore keyed on the variable name).
        let recv_name = self.expr_to_name(receiver);
        if recv_name != "Unknown" {
            let key = format!("{}.{}", recv_name, method);
            if let Some(t) = self.fn_return_types.get(&key) {
                return Some(t);
            }
        }

        // 2. Type-qualified: "str.to_uint", "Str.to_uint", "auto.str.to_uint".
        //    Resolve the receiver's type name from var_types (for an Ident) or
        //    fall back to infer_object_type for literals/chains.
        let type_name = if let Expr::Ident(name) = receiver {
            self.var_types
                .get(name.as_ref())
                .map(|t| self.type_name_from_type(t))
        } else {
            None
        };
        let type_name = type_name.or_else(|| {
            // For literal receivers (e.g. "42".to_uint()) infer_object_type
            // already maps ObjectType -> type name via its stdlib fallbacks.
            let ot = self.infer_object_type(receiver);
            match ot {
                ObjectType::String => Some("str".to_string()),
                ObjectType::Int => Some("int".to_string()),
                ObjectType::Uint => Some("uint".to_string()),
                ObjectType::Bool => Some("bool".to_string()),
                ObjectType::Float => Some("float".to_string()),
                ObjectType::Double => Some("double".to_string()),
                ObjectType::Char => Some("char".to_string()),
                ObjectType::Array => Some("List".to_string()),
                _ => None,
            }
        });

        if let Some(tn) = type_name {
            // lowercase.type
            let lower = format!("{}.{}", tn.to_lowercase(), method);
            if let Some(t) = self.fn_return_types.get(&lower) {
                return Some(t);
            }
            // TitleCase.type
            let mut chars = tn.chars();
            if let Some(first) = chars.next() {
                let titled: String = first.to_uppercase().collect::<String>() + chars.as_str();
                let title_key = format!("{}.{}", titled, method);
                if let Some(t) = self.fn_return_types.get(&title_key) {
                    return Some(t);
                }
            }
            // auto.{type}.{method}
            let qualified = format!("auto.{}.{}", tn.to_lowercase(), method);
            if let Some(t) = self.fn_return_types.get(&qualified) {
                return Some(t);
            }
        }

        // 3. Bare method name: "to_uint"
        self.fn_return_types.get(method)
    }

    /// Plan 378: Does `receiver.method()` return a 64-bit integer (I64/U64/USize/Uint)?
    #[inline]
    pub(super) fn dot_method_returns_64(&self, receiver: &Expr, method: &str) -> bool {
        self.lookup_dot_method_type(receiver, method)
            .map(|t| matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint))
            .unwrap_or(false)
    }

    // Check if expression is specifically u64 (vs i64) for choosing the right coercion opcode
    pub(super) fn is_u64_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::U64(_) => true,
            Expr::Ident(name) => self
                .var_types
                .get(name.as_ref())
                .map(|t| matches!(t, Type::U64))
                .unwrap_or(false),
            // Plan 378: method call `recv.method()` returning a 64-bit value.
            // is_u64_expr is stricter than contains_u64 — it only treats the
            // value as U64 (for emitting U64_TO_F64 vs I64_TO_F64).
            Expr::Dot(receiver, method) => self
                .lookup_dot_method_type(receiver.as_ref(), method.as_ref())
                .map(|t| matches!(t, Type::U64))
                .unwrap_or(false),
            Expr::Call(call) => {
                // Method-call form: Expr::Call { name: Expr::Dot(recv, method), .. }
                if let Expr::Dot(receiver, method) = call.name.as_ref() {
                    self.lookup_dot_method_type(receiver.as_ref(), method.as_ref())
                        .map(|t| matches!(t, Type::U64))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    // Check if this is a string operation (either operand is a string type)
    // Used to emit STR_CAT instead of ADD for string concatenation
    // Excludes cases where one operand is an integer literal (not string concat)
    pub(super) fn is_string_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        // If either operand is a numeric literal, this cannot be string concatenation
        if matches!(
            lhs,
            Expr::Int(_)
                | Expr::Float(_, _)
                | Expr::Double(_, _)
                | Expr::U64(_)
                | Expr::I64(_)
                | Expr::U8(_)
                | Expr::Byte(_)
        ) || matches!(
            rhs,
            Expr::Int(_)
                | Expr::Float(_, _)
                | Expr::Double(_, _)
                | Expr::U64(_)
                | Expr::I64(_)
                | Expr::U8(_)
                | Expr::Byte(_)
        ) {
            return false;
        }
        self.is_string_expr(lhs) || self.is_string_expr(rhs)
    }

    pub(super) fn is_u64_operation(&self, lhs: &Expr, rhs: &Expr) -> bool {
        self.contains_u64(lhs) || self.contains_u64(rhs)
    }

    pub(super) fn contains_u64(&self, expr: &Expr) -> bool {
        match expr {
            Expr::U64(_) | Expr::I64(_) => true,
            Expr::Cast { target_type, .. } => matches!(
                target_type,
                Type::U64 | Type::I64 | Type::USize | Type::Uint
            ),
            Expr::Ident(name) => self
                .var_types
                .get(name.as_ref())
                .map(|t| matches!(t, Type::U64 | Type::I64))
                .unwrap_or(false),
            // Plan 378: bare field access (recv.field) where the field itself
            // is 64-bit, or a method name without parens. For method calls we
            // rely on the Expr::Call{Dot} arm below; here we recurse into the
            // receiver so e.g. (a.to_u64()).field is still detected.
            Expr::Dot(receiver, method) => {
                if self.dot_method_returns_64(receiver.as_ref(), method.as_ref()) {
                    return true;
                }
                self.contains_u64(receiver.as_ref())
            }
            Expr::Call(call) => {
                if let Expr::Ident(fn_name) = call.name.as_ref() {
                    self.fn_return_types
                        .get(fn_name.as_ref())
                        .map(|t| matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint))
                        .unwrap_or(false)
                } else if let Expr::Dot(receiver, method) = call.name.as_ref() {
                    // Plan 378: method call `recv.method()` returning a 64-bit
                    // value (e.g. "42".to_uint()).
                    self.dot_method_returns_64(receiver.as_ref(), method.as_ref())
                } else {
                    false
                }
            }
            Expr::Bina(lhs, _, rhs) => self.contains_u64(lhs) || self.contains_u64(rhs),
            Expr::Unary(_, inner) => self.contains_u64(inner),
            _ => false,
        }
    }

    // Type hint for f-string parts to guide BUILD_FSTR value popping
    // Reflects the ACTUAL stack representation, not the logical type
    pub(super) fn expr_type_hint(&self, expr: &Expr) -> FStrPartType {
        match expr {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => FStrPartType::String,
            // Float compiles to CONST_F32 (4 bytes, 1 slot)
            Expr::Float(_, _) => FStrPartType::Float32,
            // Double compiles to CONST_F64 (8 bytes, 2 slots)
            Expr::Double(_, _) => FStrPartType::Float64,
            // u64/i64 literals compile to 2-slot values
            Expr::U64(_) | Expr::I64(_) => FStrPartType::Uint64,
            Expr::Ident(name) => {
                let name_str = name.to_string();
                if let Some(ty) = self.var_types.get(&name_str) {
                    match ty {
                        Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice => {
                            FStrPartType::String
                        }
                        Type::Float => FStrPartType::Float32,
                        Type::Double => FStrPartType::Float64,
                        Type::U64 | Type::I64 | Type::Uint | Type::USize => FStrPartType::Uint64,
                        // All other locals are stored as i32 (1 slot)
                        _ => FStrPartType::Int,
                    }
                } else {
                    FStrPartType::Int
                }
            }
            // Plan 378: bare field access `obj.field` where the field is a
            // 64-bit (u64/i64/f64) or string type. Without this arm, a u64
            // field used directly in an f-string would be miscounted as 1 slot.
            Expr::Dot(obj, field) => {
                if let Expr::Ident(var_name) = obj.as_ref() {
                    if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                        let type_name = match var_type {
                            Type::User(td) => Some(td.name.to_string()),
                            Type::GenericInstance(inst) => Some(inst.base_name.to_string()),
                            _ => None,
                        };
                        if let Some(tn) = type_name {
                            if let Some(ct) = self.generic_registry.get_type(&tn) {
                                if let Some(ft) = ct.field_type(field.as_ref()) {
                                    return match ft {
                                        Type::StrFixed(_)
                                        | Type::StrOwned
                                        | Type::CStrLit
                                        | Type::StrSlice => FStrPartType::String,
                                        Type::Float => FStrPartType::Float32,
                                        Type::Double => FStrPartType::Float64,
                                        Type::U64 | Type::I64 | Type::Uint | Type::USize => {
                                            FStrPartType::Uint64
                                        }
                                        _ => FStrPartType::Int,
                                    };
                                }
                            }
                        }
                    }
                }
                FStrPartType::Int
            }
            Expr::Bina(lhs, _, rhs) => {
                // Check if binary result is a u64 or f64 operation (2 slots)
                if self.is_u64_operation(lhs, rhs) {
                    FStrPartType::Uint64
                } else if self.is_double_operation(lhs, rhs) {
                    FStrPartType::Float64
                } else if self.is_float_operation(lhs, rhs) {
                    FStrPartType::Float32
                } else {
                    FStrPartType::Int
                }
            }
            Expr::Call(call) => {
                // Check fn_return_types for the called function
                if let Expr::Ident(fn_name) = call.name.as_ref() {
                    if let Some(ret_ty) = self.fn_return_types.get(fn_name.as_ref()) {
                        match ret_ty {
                            Type::Double => FStrPartType::Float64,
                            Type::U64 | Type::I64 | Type::USize | Type::Uint => {
                                FStrPartType::Uint64
                            }
                            Type::Float => FStrPartType::Float32,
                            Type::StrFixed(_) | Type::StrOwned => FStrPartType::String,
                            _ => FStrPartType::Int,
                        }
                    } else {
                        FStrPartType::Int
                    }
                } else if let Expr::Dot(receiver, method) = call.name.as_ref() {
                    // Plan 378: method call `recv.method()` — return-type-aware slot
                    // hint so 2-slot u64/i64 results (e.g. s.to_uint()) aren't
                    // miscounted as 1 slot in BUILD_FSTR.
                    // Plan 437 §0.6.H②续：先查 fn_return_types 的限定名
                    // （"math.cos" 等 stdlib 模块函数——lookup_dot_method_type
                    // 只看 receiver 变量类型，模块前缀查不到 → 误落 Int，
                    // f64(1.0) 位模式按 int 格式化得 0）。目录键为 canonical
                    // "auto.math.cos"，短名按 to_canonical 规则双查。
                    let qualified = format!("{}.{}", self.expr_to_name(receiver), method);
                    let canonical = if qualified.starts_with("auto.")
                        || qualified.starts_with("rust.")
                        || qualified.starts_with("py.")
                    {
                        None
                    } else {
                        qualified
                            .split_once('.')
                            .map(|(p, r)| format!("auto.{}.{}", p.to_lowercase(), r))
                    };
                    let ret_ty = self
                        .fn_return_types
                        .get(&qualified)
                        .or_else(|| canonical.as_ref().and_then(|c| self.fn_return_types.get(c)));
                    if let Some(ret_ty) = ret_ty {
                        return match ret_ty {
                            Type::Double => FStrPartType::Float64,
                            Type::U64 | Type::I64 | Type::USize | Type::Uint => {
                                FStrPartType::Uint64
                            }
                            Type::Float => FStrPartType::Float32,
                            Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice => {
                                FStrPartType::String
                            }
                            _ => FStrPartType::Int,
                        };
                    }
                    match self.lookup_dot_method_type(receiver.as_ref(), method.as_ref()) {
                        Some(Type::Double) => FStrPartType::Float64,
                        Some(Type::U64 | Type::I64 | Type::USize | Type::Uint) => {
                            FStrPartType::Uint64
                        }
                        Some(Type::Float) => FStrPartType::Float32,
                        Some(
                            Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice,
                        ) => FStrPartType::String,
                        _ => FStrPartType::Int,
                    }
                } else {
                    FStrPartType::Int
                }
            }
            _ => FStrPartType::Int,
        }
    }

    /// Plan 212 Phase 2.2: Check if expression is an opaque constructor call
    /// Returns the crate name (e.g., "regex") if the expression constructs an opaque type.
    pub(super) fn infer_opaque_crate_from_expr(&self, expr: &Expr) -> Option<String> {
        // Handle: Type.new(...).unwrap() or Type.parse(...)
        if let Expr::Call(call) = expr {
            if let Expr::Dot(obj, method) = call.name.as_ref() {
                if method.as_str() == "unwrap" || method.as_str() == "expect" {
                    return self.infer_opaque_crate_from_expr(obj);
                }
                if let Expr::Ident(type_name) = obj.as_ref() {
                    let m = method.as_str();
                    if m == "new" || m == "parse" || m == "now" {
                        // Check stdlib module names (lowercase) directly
                        match type_name.as_ref() {
                            "url" => return Some("url".to_string()),
                            _ => {}
                        }
                        if let Some((crate_name, _)) = self.rust_native_map.get(type_name.as_str())
                        {
                            match crate_name.as_str() {
                                "regex" | "url" | "semver" | "chrono" | "sha2" => {
                                    return Some(crate_name.clone())
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        if let Expr::Dot(inner, method) = expr {
            if method.as_str() == "unwrap" || method.as_str() == "expect" {
                return self.infer_opaque_crate_from_expr(inner);
            }
        }
        None
    }

    /// Infer return type for native function calls not in fn_return_types.
    /// Returns the current last_expr_type as default to preserve existing behavior,
    /// except for well-known int-returning natives where it overrides to Int.
    pub(super) fn infer_native_return_type(&self, name: &str) -> ObjectType {
        // Plan 454 E(§M 缺口③):obj 族返回型别标注——keys/values 产 List,
        // find 产元素(Option 载荷,TAG_NULL=None)。无此标注时下游 .length/
        // for-in/.field 全部按 Int 接收者退化(GET_FIELD 错址 → 静默 0)。
        if name.starts_with("auto.obj.")
            || name.starts_with("obj.")
            || name.starts_with("Object.keys")
            || name.starts_with("Object.values")
        {
            return match name.rsplit('.').next().unwrap_or(name) {
                "keys" | "values" => ObjectType::Array,
                "find" => ObjectType::NestedObject,
                _ => ObjectType::NestedObject,
            };
        }
        // Known int-returning natives (without fn_return_types entry)
        // Match both "auto.str.len" and "str.len" forms since the compiler
        // may use either depending on how the method was resolved.
        if name == "auto.hashmap.get_int"
            || name == "hashmap.get_int"
            || name == "auto.list.len"
            || name == "list.len"
            || name == "auto.str.len"
            || name == "str.len"
            || name == "auto.hashmap.size"
            || name == "hashmap.size"
            || name == "auto.list.find"
            || name == "list.find"
            || name == "str.char_at"
            || name == "auto.str.char_at"
            || name == "str.index_of"
            || name == "auto.str.index_of"
        {
            return ObjectType::Int;
        }
        // Math functions return f64
        if name.starts_with("auto.math.") {
            return ObjectType::Double;
        }
        // Rand functions
        if name == "auto.rand.thread_rng" {
            return ObjectType::NestedObject; // opaque handle (i32 heap id)
        }
        if name == "auto.rng.gen_range" || name == "auto.rng.gen" || name == "auto.rand.random" {
            return ObjectType::Int;
        }
        // Log/tracing functions return void
        if name == "auto.log.noop"
            || name == "Log.debug"
            || name == "Log.info"
            || name == "Log.warn"
            || name == "Log.error"
        {
            return ObjectType::Void;
        }
        // Regex opaque shims — constructors return Int (handle), methods vary
        if name.starts_with("auto.re_opaque.") {
            if name == "auto.re_opaque.new" {
                return ObjectType::NestedObject; // opaque handle
            }
            if name == "auto.re_opaque.is_match" {
                return ObjectType::Int; // bool as i32
            }
            return ObjectType::String; // find, find_all, replace_all, captures
        }
        // Url opaque shims
        if name.starts_with("auto.url_opaque.") {
            if name == "auto.url_opaque.parse" || name == "auto.url_opaque.join" {
                return ObjectType::NestedObject; // opaque handle
            }
            if name == "auto.url_opaque.port" {
                return ObjectType::Int;
            }
            if name == "auto.url_opaque.query_pairs" {
                return ObjectType::NestedObject; // list handle
            }
            return ObjectType::String; // scheme, host_str, path, fragment, origin
        }
        // Semver opaque shims
        if name.starts_with("auto.semver_opaque.") {
            if name == "auto.semver_opaque.parse" {
                return ObjectType::NestedObject; // opaque handle
            }
            if name == "auto.semver_opaque.major"
                || name == "auto.semver_opaque.minor"
                || name == "auto.semver_opaque.patch"
                || name == "auto.semver_opaque.cmp_gt"
            {
                return ObjectType::Int;
            }
            return ObjectType::String; // pre, to_string
        }
        // hashmap.get (unspecialized) returns unknown type — don't inherit
        // stale String from argument compilation (e.g., map.get(str_key))
        if name == "auto.hashmap.get" {
            return ObjectType::NestedObject;
        }
        // Plan 368 W5: Char.to_str returns String.
        if Self::char_to_str_returns_string(name) {
            return ObjectType::String;
        }
        // Plan 368 W8: string methods. infer_native_return_type is consulted by
        // the native-call codegen path to set last_expr_type / last_was_native_void.
        // Without these arms, a string method's return type defaulted to the stale
        // last_expr_type (often Void after compiling the receiver+args), which made
        // `return s.upper()` / `return s.replace(...)` drop the value. Match both
        // "auto.str.X" and "str.X" forms. Keep in sync with infer_call_spec_return_type
        // and the native_catalog str/Str entries.
        let str_method = name
            .strip_prefix("auto.str.")
            .or_else(|| name.strip_prefix("str."))
            .or_else(|| name.strip_prefix("Str."));
        if let Some(m) = str_method {
            match m {
                "trim" | "replace" | "replace_first" | "to_upper" | "to_lower" | "upper"
                | "lower" | "to_uppercase" | "to_lowercase" | "substr" | "sub" | "slice"
                | "repeat" | "reverse" => return ObjectType::String,
                "split" | "lines" | "split_once" => return ObjectType::Array,
                "len" | "find" | "char_at" | "parse_int" | "match_count" => return ObjectType::Int,
                "contains" | "starts_with" | "ends_with" | "is_empty" => return ObjectType::Bool,
                "parse_float" => return ObjectType::Float,
                _ => {}
            }
        }
        // Default: preserve current last_expr_type (no change)
        self.last_expr_type
    }

    /// Plan 368 W5: Char.to_str (codepoint → single-char String). Same purpose
    /// as the str_method arms above — without this, the codegen treats
    /// Char.to_str as returning the stale last_expr_type (Void after compiling
    /// the codepoint arg), so its String result gets dropped on direct use.
    fn char_to_str_returns_string(name: &str) -> bool {
        matches!(name, "Char.to_str" | "auto.char.to_str" | "char.to_str")
    }

    /// Infer return type for CALL_SPEC based on method name.
    /// Prevents stale `last_expr_type` from causing wrong opcode selection.
    pub(super) fn infer_call_spec_return_type(&self, method: &str) -> ObjectType {
        match method {
            // Methods returning int
            "get_int" | "len" | "char_at" | "parse_int" | "to_int" | "find" | "rfind" | "count"
            | "cmp" | "hash" | "abs" | "size" => ObjectType::Int,
            // Methods returning string (get_str is explicitly string-typed; plain "get" returns unknown)
            "get_str" | "substr" | "trim" | "to_str" | "to_string" | "join" | "format"
            | "lower" | "upper" | "replace" | "slice" | "scheme" | "host" | "host_str" | "path"
            | "query" | "fragment" | "origin" | "username" | "password" | "from_utf8"
            | "from_utf8_lossy" | "from_str" => ObjectType::String,
            // Methods returning float/double
            "to_float" | "to_double" | "sqrt" | "sin" | "cos" | "tan" | "pow" | "log" | "exp"
            | "floor" | "ceil" | "round" => ObjectType::Double,
            // Methods returning bool
            "contains" | "starts_with" | "ends_with" | "is_empty" | "has_key" | "has_prefix" => {
                ObjectType::Bool
            }
            // Methods returning a collection or opaque handle (nested object)
            "keys" | "values" | "entries" | "split" | "lines" | "chars" | "graphemes"
            | "as_bytes" | "bytes" | "par_iter" | "par_iter_mut" | "into_iter" | "iter"
            | "iter_mut" | "sample" | "gen" | "gen_range" | "into_inner" | "captures_iter"
            | "find_iter" | "matches" => ObjectType::NestedObject,
            // Methods returning void
            "push" | "insert" | "insert_int" | "insert_str" | "remove" | "clear" | "sort"
            | "reverse" | "print" | "println" | "write" => ObjectType::Void,
            // Default: unknown method — preserve current last_expr_type
            // (changed from self.last_expr_type to NestedObject for opaque handle
            // safety, but that caused regressions in str.trim etc.)
            _ => self.last_expr_type,
        }
    }

    pub(super) fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => true,
            Expr::Ident(name) => {
                // Check inferred type for the variable
                let ty = self.infer_ctx.type_env.get(name);
                matches!(ty, Some(Type::StrFixed(_)) | Some(Type::StrOwned))
                    || matches!(
                        self.var_types.get(name.as_ref()),
                        Some(Type::StrFixed(_))
                            | Some(Type::StrOwned)
                            | Some(Type::CStrLit | Type::StrSlice)
                    )
            }
            Expr::Index(container, _index) => {
                // String slicing: "hello"[0..2] produces a string
                self.is_string_expr(container)
            }
            Expr::Dot(obj, field) => {
                // Field access: check if the field's type is string
                if let Expr::Ident(var_name) = obj.as_ref() {
                    if let Some(var_type) = self.var_types.get(var_name.as_ref()) {
                        let type_name = match var_type {
                            Type::User(td) => td.name.to_string(),
                            Type::GenericInstance(inst) => inst.base_name.to_string(),
                            _ => return false,
                        };
                        if let Some(ct) = self.generic_registry.get_type(&type_name) {
                            if let Some(ft) = ct.field_type(field.as_ref()) {
                                return matches!(
                                    ft,
                                    Type::StrFixed(_)
                                        | Type::StrOwned
                                        | Type::CStrLit
                                        | Type::StrSlice
                                );
                            }
                        }
                    }
                }
                false
            }
            // Function call: check fn_return_types for str return type
            Expr::Call(call) => {
                if let Expr::Ident(fn_name) = call.name.as_ref() {
                    if let Some(ret_ty) = self.fn_return_types.get(fn_name.as_ref()) {
                        return matches!(
                            ret_ty,
                            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit
                        );
                    }
                }
                // Also check method calls that return string
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    let obj_type = self.infer_object_type(obj.as_ref());
                    let ret = self.stdlib_method_return_type(obj_type, method.as_ref());
                    return ret == ObjectType::String;
                }
                false
            }
            // Binary add with string operand: string concatenation chain
            Expr::Bina(lhs, op, rhs) => {
                if matches!(op, Op::Add) {
                    return self.is_string_expr(lhs) || self.is_string_expr(rhs);
                }
                false
            }
            _ => false,
        }
    }

    /// Stdlib method return type for chained calls (e.g., str.slice().trim())
    fn stdlib_method_return_type(&self, receiver_type: ObjectType, method: &str) -> ObjectType {
        match (receiver_type, method) {
            // String methods returning String
            (
                ObjectType::String,
                "trim" | "slice" | "sub" | "repeat" | "to_upper" | "to_lower" | "to_uppercase"
                | "to_lowercase" | "replace",
            ) => ObjectType::String,
            // String methods returning Int
            (ObjectType::String, "len" | "length" | "find" | "char_at" | "parse_int") => {
                ObjectType::Int
            }
            // String methods returning Bool
            (ObjectType::String, "starts_with" | "ends_with" | "contains" | "is_empty") => {
                ObjectType::Bool
            }
            // Int/Uint methods returning String
            (ObjectType::Int | ObjectType::Uint, "to_string" | "to_hex") => ObjectType::String,
            // Int bitwise methods returning Int (enables chaining: x.and(3).shl(4))
            (
                ObjectType::Int | ObjectType::Uint,
                "and" | "or" | "xor" | "not" | "shl" | "shr" | "sar" | "rol" | "ror" | "count_ones"
                | "leading_zeros" | "trailing_zeros" | "flip" | "bitrev" | "bit_read" | "bit_on"
                | "bit_off" | "bit_flip",
            ) => ObjectType::Int,
            // Int bit_test returns Bool
            (ObjectType::Int | ObjectType::Uint, "bit_test") => ObjectType::Bool,
            // Array methods returning Int
            (ObjectType::Array, "len" | "length") => ObjectType::Int,
            // Array methods returning Array (chained)
            (ObjectType::Array, "map" | "filter" | "slice" | "reverse") => ObjectType::Array,
            _ => ObjectType::NestedObject,
        }
    }

    /// Plan 569 D2: 接收者 py-可能判定（py-类型侧表三源静态查表）。
    /// - Ident 接收者 → py_typed_vars（`var x = <py-ffi 调用>` 落盘传导）
    /// - Call 接收者 → 被调名 ∈ py_native_map（py-ffi 调用点）/ py_modules
    ///   前缀（模块点调）/ fn_may_py_returns（用户 fn 返回传播）
    /// - 其余窄形态 → last_expr_may_py 粘性位（保守：错报=obj_* 组合子
    ///   Auto 臂照常分派，漏报=维持现状规避）。
    pub(super) fn receiver_may_py(&self, obj: &Expr) -> bool {
        match obj {
            Expr::Ident(name) => self.py_typed_vars.contains(name.as_str()),
            Expr::Call(call) => match call.name.as_ref() {
                Expr::Ident(callee) => {
                    self.py_native_map.contains_key(callee.as_str())
                        || self.fn_may_py_returns.contains(callee.as_str())
                        || (callee.as_str().contains('.')
                            && callee
                                .as_str()
                                .split('.')
                                .next()
                                .map(|p| self.py_modules.contains(p))
                                .unwrap_or(false))
                }
                _ => false,
            },
            _ => self.last_expr_may_py,
        }
    }

    // Plan 073: Convert expression to ObjectType for object field tracking
    pub(crate) fn infer_object_type(&self, expr: &Expr) -> ObjectType {
        match expr {
            Expr::Float(_, _) => ObjectType::Float,
            Expr::Double(_, _) => ObjectType::Double,
            Expr::Int(_) | Expr::I8(_) | Expr::I64(_) => ObjectType::Int,
            Expr::Uint(_) | Expr::U64(_) => ObjectType::Uint,
            Expr::U8(_) => ObjectType::Int, // U8 arithmetic returns plain int
            Expr::Byte(_) => ObjectType::Byte, // Plan 118: Byte has its own type for hex formatting
            Expr::Str(_) | Expr::CStr(_) => ObjectType::String,
            Expr::Char(_) => ObjectType::Char,
            Expr::Bool(_) => ObjectType::Bool,
            // Plan 197 Task 3: Method chaining — resolve return type from fn_return_types
            Expr::Call(call) => {
                // Try to resolve return type from fn_return_types
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    let fn_name =
                        format!("{}.{}", self.expr_to_name(obj.as_ref()), method.as_ref());
                    if let Some(ret_ty) = self.fn_return_types.get(&fn_name) {
                        self.type_to_object_type(ret_ty)
                    } else {
                        // Infer the receiver's type first
                        let obj_type = self.infer_object_type(obj.as_ref());
                        let type_name = match obj_type {
                            ObjectType::NestedObject => {
                                // Try to get the actual type name from fn_return_types
                                if let Expr::Call(inner_call) = obj.as_ref() {
                                    if let Expr::Dot(inner_obj, inner_method) =
                                        inner_call.name.as_ref()
                                    {
                                        let inner_fn = format!(
                                            "{}.{}",
                                            self.expr_to_name(inner_obj.as_ref()),
                                            inner_method.as_ref()
                                        );
                                        if let Some(inner_ret) = self.fn_return_types.get(&inner_fn)
                                        {
                                            self.type_name_from_type(inner_ret)
                                        } else {
                                            return ObjectType::NestedObject;
                                        }
                                    } else {
                                        return ObjectType::NestedObject;
                                    }
                                } else {
                                    return ObjectType::NestedObject;
                                }
                            }
                            ObjectType::String => "str".to_string(),
                            ObjectType::Int => "int".to_string(),
                            ObjectType::Uint => "uint".to_string(),
                            ObjectType::Bool => "bool".to_string(),
                            ObjectType::Array => "Array".to_string(),
                            _ => return ObjectType::NestedObject,
                        };
                        let qualified = format!("{}.{}", type_name, method.as_ref());
                        if let Some(ret_ty) = self.fn_return_types.get(&qualified) {
                            self.type_to_object_type(ret_ty)
                        } else {
                            // Stdlib method return type inference for chained calls
                            self.stdlib_method_return_type(obj_type, method.as_ref())
                        }
                    }
                } else if let Expr::Ident(name) = call.name.as_ref() {
                    if let Some(ret_ty) = self.fn_return_types.get(name.as_ref()) {
                        self.type_to_object_type(ret_ty)
                    } else {
                        ObjectType::NestedObject
                    }
                } else {
                    ObjectType::NestedObject
                }
            }
            // Plan 073: Nested object, node, pair and array types
            Expr::Object(_)
            | Expr::Node(_)
            | Expr::Bina(_, _, _)
            | Expr::If(_)
            | Expr::Lambda(_)
            | Expr::Closure(_)
            | Expr::Pair(_) => ObjectType::NestedObject,
            Expr::Array(_) => ObjectType::Array,
            // Plan 118 Phase 4: Check variable types for identifier expressions
            Expr::Ident(name) => {
                let name_str = name.to_string();
                if let Some(var_type) = self.var_types.get(&name_str) {
                    self.type_to_object_type(var_type)
                } else {
                    ObjectType::Int
                }
            }
            // Plan 201: Resolve field access type for self.field.method() chains
            Expr::Dot(obj, field) => {
                if let Expr::Ident(ident_name) = obj.as_ref() {
                    if let Some(var_type) = self.var_types.get(ident_name.as_str()) {
                        let field_type = self.resolve_field_type(var_type, field.as_str());
                        self.type_to_object_type(&field_type)
                    } else {
                        ObjectType::Int
                    }
                } else {
                    // Nested dot: try to resolve from outer expression type
                    let outer_type = self.infer_object_type(obj.as_ref());
                    match outer_type {
                        ObjectType::NestedObject => {
                            // Try fn_return_types for chained calls
                            let obj_name = self.expr_to_name(obj.as_ref());
                            if let Some(ret_ty) = self.fn_return_types.get(&obj_name) {
                                let field_type = self.resolve_field_type(ret_ty, field.as_str());
                                self.type_to_object_type(&field_type)
                            } else {
                                ObjectType::Int
                            }
                        }
                        _ => ObjectType::Int,
                    }
                }
            }
            // For other expressions, default to Int
            Expr::Cast { target_type, .. } => match target_type {
                Type::Int | Type::I64 => ObjectType::Int,
                Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
                Type::Float => ObjectType::Float,
                Type::Double => ObjectType::Double,
                Type::Byte => ObjectType::Byte,
                Type::Bool => ObjectType::Bool,
                Type::StrFixed(_) | Type::StrOwned => ObjectType::String,
                Type::Char => ObjectType::Char,
                _ => ObjectType::Int,
            },
            Expr::Index(arr, _) => {
                let arr_type = self.infer_object_type(arr.as_ref());
                match arr_type {
                    ObjectType::Array => ObjectType::Int,
                    _ => ObjectType::Int,
                }
            }
            Expr::Unary(_, inner) => self.infer_object_type(inner.as_ref()),
            Expr::View(inner) | Expr::Mut(inner) | Expr::Move(inner) | Expr::Take(inner) => {
                self.infer_object_type(inner.as_ref())
            }
            Expr::To { target_type, .. } => self.type_to_object_type(target_type),
            Expr::FStr(_) => ObjectType::String,
            Expr::GenName(name) => self
                .var_types
                .get(name.as_str())
                .map(|t| self.type_to_object_type(t))
                .unwrap_or(ObjectType::Int),
            Expr::Ref(name) => self
                .var_types
                .get(name.as_str())
                .map(|t| self.type_to_object_type(t))
                .unwrap_or(ObjectType::Int),
            Expr::Block(body) => body
                .stmts
                .last()
                .and_then(|stmt| {
                    if let crate::ast::Stmt::Expr(expr) = stmt {
                        Some(self.infer_object_type(expr))
                    } else {
                        None
                    }
                })
                .unwrap_or(ObjectType::Int),
            Expr::Some(inner) | Expr::Ok(inner) => self.infer_object_type(inner.as_ref()),
            Expr::None | Expr::Nil | Expr::Null => ObjectType::Int,
            _ => ObjectType::Int,
        }
    }

    /// Plan 197 Task 3: Try to infer a user-defined type name from an expression
    /// Check if a name matches a known Rust crate name.
    /// Used as fallback when dep/use.rust statements are dropped by parser error recovery.
    pub(super) fn is_known_rust_crate(name: &str) -> bool {
        matches!(
            name,
            "std"
                | "log"
                | "env_logger"
                | "regex"
                | "serde"
                | "serde_json"
                | "chrono"
                | "csv"
                | "walkdir"
                | "url"
                | "sha2"
                | "semver"
                | "anyhow"
                | "rand"
                | "rayon"
                | "crossbeam"
                | "tokio"
                | "clap"
                | "base64"
                | "percent_encoding"
                | "heapless"
                | "simplelog"
                | "tracing"
                | "flate2"
                | "tar"
                | "toml"
                | "reqwest"
                | "once_cell"
                | "parking_lot"
                | "dashmap"
                | "indexmap"
        )
    }

    /// Plan 383: 判断 `name` 是否是已定义的（顶层或方法）函数。
    /// 用于裸函数名在值位置（非调用）的函数引用解析。
    /// 检查 exports（含编译期已注册的函数地址），与既有 CALL reloc 的符号源一致。
    pub(super) fn is_defined_function(&self, name: &str) -> bool {
        self.exports.contains_key(name)
    }

    /// Plan 325: 判断 `name` 是否是已知 enum 类型名（本模块或跨模块）。
    /// 用于跨模块 enum 方法分发：本地 exports 可能不含被 use 模块的 enum 方法，
    /// 但只要 name 是已知 enum，就应构造 `EnumName.method` func_name 交 linker 解析。
    pub(super) fn is_enum_name(&self, name: &str) -> bool {
        // 本模块：types 含该 enum，或 enum_values/generic_registry 有以它为前缀的变体
        self.types.contains_key(name)
            || self.enum_values.keys().any(|k| k.starts_with(&format!("{}.", name)))
            || self.generic_registry.has_template(name)
            // 跨模块：type_store（含 use 导入的 enum）注册了该 enum 声明
            || self.type_store.read().map(|ts| ts.all_enum_decls().any(|(n, _)| n == name)).unwrap_or(false)
    }

    /// Plan 383: 返回函数 `name` 的参数个数（用于 CLOSURE 的 n_args 字段，
    /// 使 CALL_CLOSURE 能正确设置 current_fn_n_args）。
    /// 优先查 fn_params；查不到（script-run 路径 fn_params 可能为空）回退 0。
    pub(super) fn function_param_count(&self, name: &str) -> usize {
        self.fn_params.get(name).map(|p| p.len()).unwrap_or(0)
    }

    /// by looking up its return type in fn_return_types.
    /// Returns the type name (e.g., "Point") if found, or None.
    pub(super) fn infer_user_type_name(&self, expr: &Expr) -> Option<String> {
        if let Expr::Call(call) = expr {
            let fn_name = if let Expr::Dot(obj, method) = call.name.as_ref() {
                format!("{}.{}", self.expr_to_name(obj.as_ref()), method.as_ref())
            } else if let Expr::Ident(name) = call.name.as_ref() {
                name.to_string()
            } else {
                return None;
            };
            if let Some(ret_ty) = self.fn_return_types.get(&fn_name) {
                match ret_ty {
                    Type::User(td) => return Some(td.name.to_string()),
                    Type::Enum(ed) => return Some(ed.borrow().name.to_string()),
                    _ => {}
                }
            }
        }
        None
    }

    /// Plan 197 Task 3: Convert an expression to its dot-separated name for fn_return_types lookup
    pub(super) fn expr_to_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(name) => name.to_string(),
            Expr::Dot(obj, method) => format!("{}.{}", self.expr_to_name(obj), method),
            // Plan 197 Bug C: resolve chained call names for 3rd+ level method dispatch
            Expr::Call(call) => {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    format!("{}.{}", self.expr_to_name(obj), method)
                } else if let Expr::Ident(name) = call.name.as_ref() {
                    name.to_string()
                } else {
                    "Unknown".to_string()
                }
            }
            _ => "Unknown".to_string(),
        }
    }

    /// Resolve a field's type from a parent type (for self.field lookups)
    fn resolve_field_type(&self, parent_type: &Type, field_name: &str) -> Type {
        match parent_type {
            Type::User(td) => {
                // First try TypeDecl members (may be empty for synthetic types)
                for member in &td.members {
                    if member.name.as_str() == field_name {
                        return member.ty.clone();
                    }
                }
                // Fallback: look up in generic_registry (has field types)
                let type_name = td.name.to_string();
                if let Some(class_type) = self.generic_registry.get_type(&type_name) {
                    if let Some(field_type) = class_type.field_type(field_name) {
                        return field_type;
                    }
                }
                Type::Unknown
            }
            Type::GenericInstance(inst) => {
                if let Some(class_type) =
                    self.generic_registry.get_type(&inst.base_name.to_string())
                {
                    if let Some(field_type) = class_type.field_type(field_name) {
                        return field_type;
                    }
                }
                Type::Unknown
            }
            _ => Type::Unknown,
        }
    }

    /// Plan 197 Task 3: Map a Type to ObjectType for object field tracking
    pub(super) fn type_to_object_type(&self, ty: &Type) -> ObjectType {
        match ty {
            Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice => {
                ObjectType::String
            }
            Type::Char => ObjectType::Char,
            Type::Int | Type::I64 => ObjectType::Int,
            Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
            Type::Byte => ObjectType::Byte,
            Type::Float => ObjectType::Float,
            Type::Double => ObjectType::Double,
            Type::Bool => ObjectType::Bool,
            Type::Array(_) | Type::RuntimeArray(_) | Type::List(_) => ObjectType::Array,
            Type::Map(_, _) => ObjectType::NestedObject,
            _ => ObjectType::NestedObject,
        }
    }

    /// Extract a type name string from a Type for fn_return_types qualified name lookup
    fn type_name_from_type(&self, ty: &Type) -> String {
        match ty {
            Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice => {
                "str".to_string()
            }
            Type::Char => "char".to_string(),
            Type::Int | Type::I64 => "int".to_string(),
            Type::Uint | Type::U64 | Type::USize => "uint".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::User(name) => name.to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Check if a name is likely a type name (capitalized first letter)
    /// This is a heuristic following Rust/AutoLang naming conventions:
    /// - Type names: Capitalized (List, Point, MyType)
    /// - Variables/Functions: lowercase (list, foo, my_var)
    pub(super) fn is_type_name_heuristic(&self, name: &str) -> bool {
        name.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    /// Infer type name from a variable name (Plan 080: with explicit type tracking)
    /// Plan 197 Task 16: Infer the inner type of an Option value from the is target
    ///
    /// Given the is target expression, look up its type (e.g., ?str -> Option<str>)
    /// and return the inner type (e.g., str).
    pub(super) fn infer_option_inner_type(&self, target: &Expr) -> Type {
        if let Expr::Ident(name) = target {
            if let Some(ty) = self.var_types.get(name.as_ref()) {
                if let Type::Option(inner) = ty {
                    return (**inner).clone();
                }
                // If the type is not Option, still return it as a best guess
                return ty.clone();
            }
        }
        // Default: unknown type
        Type::Unknown
    }

    ///
    /// First checks the var_types map (tracked during let declarations)
    /// Plan 193: Infer precise source type for .to() conversion opcode selection.
    /// Uses var_types for identifiers, AST node type for literals.
    pub(super) fn infer_expr_type_for_conv(
        &self,
        expr: &Expr,
        runtime_type: ObjectType,
    ) -> ConvSrcType {
        match expr {
            Expr::I64(_) => ConvSrcType::I64,
            Expr::U64(_) => ConvSrcType::U64,
            Expr::Float(_, _) => ConvSrcType::F32,
            Expr::Double(_, _) => ConvSrcType::F64,
            Expr::Bool(_) => ConvSrcType::Bool,
            Expr::Str(_) | Expr::FStr(_) => ConvSrcType::Str,
            Expr::Ident(name) => {
                if let Some(ty) = self.var_types.get(name.as_str()) {
                    match ty {
                        Type::I64 => ConvSrcType::I64,
                        Type::U64 => ConvSrcType::U64,
                        Type::Float => ConvSrcType::F32,
                        Type::Double => ConvSrcType::F64,
                        Type::Bool => ConvSrcType::Bool,
                        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => ConvSrcType::Str,
                        _ => ConvSrcType::I32,
                    }
                } else {
                    match runtime_type {
                        ObjectType::Float => ConvSrcType::F32,
                        ObjectType::Double => ConvSrcType::F64,
                        ObjectType::Uint => ConvSrcType::U64,
                        ObjectType::Bool => ConvSrcType::Bool,
                        ObjectType::String => ConvSrcType::Str,
                        _ => ConvSrcType::I32,
                    }
                }
            }
            Expr::Dot(_, _) => match runtime_type {
                ObjectType::Float => ConvSrcType::F32,
                ObjectType::Double => ConvSrcType::F64,
                ObjectType::Uint => ConvSrcType::U64,
                ObjectType::Bool => ConvSrcType::Bool,
                ObjectType::String => ConvSrcType::Str,
                _ => ConvSrcType::I32,
            },
            _ => match runtime_type {
                ObjectType::Float => ConvSrcType::F32,
                ObjectType::Double => ConvSrcType::F64,
                ObjectType::Uint => ConvSrcType::U64,
                ObjectType::Bool => ConvSrcType::Bool,
                ObjectType::String => ConvSrcType::Str,
                _ => ConvSrcType::I32,
            },
        }
    }

    /// Resolve a method call: receiver_type + method_name -> qualified name -> native ID.
    ///
    /// Plan 203 Phase 3: Provides a centralised lookup path for instance method
    /// calls, trying qualified names first then falling back to short names.
    #[allow(dead_code)]
    fn resolve_method(&self, receiver_type: &str, method_name: &str) -> Option<u16> {
        let mut reg = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();

        // Try lowercase qualified: "auto.{type_lower}.{method}"
        let type_lower = receiver_type.to_lowercase();
        let qualified = format!("auto.{}.{}", type_lower, method_name);
        if let Some(id) = reg.resolve_qualified(&qualified) {
            return Some(id);
        }

        // Try title-case qualified: "auto.{Type}.{method}"
        let qualified_title = format!("auto.{}.{}", receiver_type, method_name);
        if let Some(id) = reg.resolve_qualified(&qualified_title) {
            return Some(id);
        }

        // Fallback to short-name lookup: "Type.method"
        let short_name = format!("{}.{}", receiver_type, method_name);
        reg.resolve_qualified(&short_name)
    }

    /// - "map", "dict" -> "Map"
    /// This is a fallback for when type information is not available
    pub(super) fn infer_type_from_var(&self, var_name: &str) -> Option<String> {
        // Plan 080: First check if we have explicit type information from var_types
        if let Some(ty) = self.var_types.get(var_name) {
            // Return the base type name (without generic parameters for now)
            match ty {
                Type::Int | Type::I64 => Some("int".to_string()),
                Type::Uint | Type::U64 | Type::Byte | Type::USize => Some("uint".to_string()),
                Type::Float | Type::Double => Some("float".to_string()),
                Type::Bool => Some("bool".to_string()),
                Type::Char => Some("char".to_string()),
                Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => Some("str".to_string()),
                Type::CStrLit => Some("str".to_string()),
                Type::Array(_) | Type::RuntimeArray(_) => Some("Array".to_string()),
                Type::List(_) => Some("List".to_string()),
                Type::Map(_, _) => Some("Map".to_string()), // Plan 160
                Type::Option(inner) => {
                    // Unwrap Option<T> → use inner type for method dispatch
                    // e.g., ?str calling .len() should dispatch as "str.len", not "Option.len"
                    match inner.as_ref() {
                        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => {
                            Some("str".to_string())
                        }
                        Type::List(_) => Some("List".to_string()),
                        Type::Map(_, _) => Some("Map".to_string()),
                        Type::Array(_) => Some("Array".to_string()),
                        Type::Int | Type::I64 => Some("int".to_string()),
                        Type::User(td) => Some(td.name.to_string()),
                        _ => Some("Option".to_string()),
                    }
                }
                Type::User(type_decl) => {
                    let name = type_decl.name.to_string();
                    // Option.Some/Option.None → "Option" for method dispatch
                    if name.starts_with("Option.") || name.starts_with("Result.") {
                        name.split('.').next().map(|s| s.to_string())
                    } else {
                        // Strip module prefix for method dispatch (e.g., "types.ToolRegistry" → "ToolRegistry")
                        let short = name.rsplit('.').next().unwrap_or(&name);
                        Some(short.to_string())
                    }
                }
                Type::GenericInstance(inst) => Some(inst.base_name.to_string()),
                _ => None,
            }
        } else {
            // Fallback: heuristic based on variable naming
            match var_name {
                "list" | "arr" | "array" | "vec" => Some("List".to_string()),
                "str" | "string" => Some("str".to_string()),
                "map" | "dict" | "hashmap" => Some("HashMap".to_string()),
                "set" => Some("HashSet".to_string()),
                "opt" | "option" => Some("Option".to_string()),
                "file" => Some("File".to_string()),
                "deque" => Some("VecDeque".to_string()),
                "bmap" | "treemap" => Some("BTreeMap".to_string()),
                _ => None,
            }
        }
    }

    // ========== Plan 194: Monomorphic dispatch for generic method calls ==========

    /// Plan 198 Phase 4: Resolve the Type for a constructor call like `List.new()`.
    ///
    /// Replaces the previous 120-line hardcoded if-chain. Uses a compact table
    /// for generic collections with known default type params, and falls back
    /// to TypeStore for user-defined types.
    pub(super) fn resolve_constructor_type(&self, type_name: &str) -> Option<Type> {
        // Generic collections with default type params
        const GENERIC_DEFAULTS: &[(&str, &str, &[Type])] = &[
            ("List", "List", &[Type::Int] as &[Type]),
            ("HashMap", "HashMap", &[Type::StrFixed(0), Type::Int]),
            ("HashSet", "HashSet", &[Type::StrFixed(0)]),
            ("Map", "HashMap", &[Type::StrFixed(0), Type::Int]),
            ("VecDeque", "VecDeque", &[Type::Int]),
            ("BTreeMap", "BTreeMap", &[Type::StrFixed(0), Type::Int]),
        ];

        if let Some((_, base, defaults)) = GENERIC_DEFAULTS.iter().find(|(n, _, _)| *n == type_name)
        {
            let inst = crate::ast::GenericInstance {
                base_name: crate::ast::Name::from(*base),
                args: defaults.to_vec(),
                source: None,
            };
            return Some(Type::GenericInstance(inst));
        }

        // Non-generic VM-native types that need Type::User
        const NON_GENERIC_TYPES: &[&str] = &["StringBuilder", "String"];
        if NON_GENERIC_TYPES.contains(&type_name) {
            let type_decl = crate::ast::TypeDecl {
                consts: Vec::new(),
                name: crate::ast::Name::from(type_name),
                kind: crate::ast::TypeDeclKind::UserType,
                parent: None,
                has: vec![],
                specs: vec![],
                spec_impls: vec![],
                generic_params: vec![],
                members: vec![],
                delegations: vec![],
                methods: vec![],
                attrs: vec![],
                impl_attrs: vec![],
                doc: None,
                is_pub: false,
            };
            return Some(Type::User(type_decl));
        }

        // Try TypeStore for user-defined types
        let ts = self.type_store.read().unwrap();
        if let Some(decl) = ts.lookup_type_decl_str(type_name) {
            if !decl.generic_params.is_empty() {
                let inst = crate::ast::GenericInstance {
                    base_name: decl.name.clone(),
                    args: vec![Type::Int; decl.generic_params.len()],
                    source: None,
                };
                return Some(Type::GenericInstance(inst));
            }
            let type_decl = (*decl).clone();
            return Some(Type::User(type_decl));
        }

        // Final fallback: check codegen's own type registry
        if self.types.contains_key(type_name) {
            let type_decl = crate::ast::TypeDecl {
                consts: Vec::new(),
                name: crate::ast::Name::from(type_name),
                kind: crate::ast::TypeDeclKind::UserType,
                parent: None,
                has: vec![],
                specs: vec![],
                spec_impls: vec![],
                generic_params: vec![],
                members: vec![],
                delegations: vec![],
                methods: vec![],
                attrs: vec![],
                impl_attrs: vec![],
                doc: None,
                is_pub: false,
            };
            return Some(Type::User(type_decl));
        }

        None
    }
}
