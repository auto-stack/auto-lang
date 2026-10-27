//! PLAN-843 结构拆分（自 parser.rs 原样搬移；算法/行为零变化）。
use super::*;

impl<'a> Parser<'a> {
    fn parse_ptr_type(&mut self) -> AutoResult<Type> {
        self.next(); // skip `*`

        // Handle optional `const`/`mut` qualifier (e.g., `*const T`, `*mut T`)
        // Plan 059 Phase 1: Enable generic type fields in structs
        // Note: `const` and `mut` are now keywords (TokenKind), not identifiers
        if self.is_kind(TokenKind::Const) || self.is_kind(TokenKind::Mut) {
            self.next(); // skip const/mut qualifier
        }

        let typ = self.parse_type()?;
        Ok(Type::Ptr(PtrType { of: shared(typ) }))
    }

    fn parse_array_type(&mut self) -> AutoResult<Type> {
        // parse array type name, e.g. `[10]int` or `[]int` (slice)
        // Note: `[~]T` syntax for dynamic lists has been removed - use `List` type instead
        self.next(); // skip `[`

        // Check for slice type: []T (empty brackets)
        if self.is_kind(TokenKind::RSquare) {
            self.next(); // consume `]`

            // Plan 398 §12: Parse element type recursively via parse_type (not
            // parse_ident). This allows nested slice types like `[][]RenderedCell`
            // (slice of slices) and `[](str, RenderedCell)` (slice of tuples) as
            // pub type fields — needed by ash-gui's back/api.at for the VM path
            // (Core scenario). Previously parse_ident only accepted a single
            // identifier, so `[][]T` / `[](tuple)` failed with "Expected term,
            // got RBrace", silently dropping the whole module's symbols.
            let elem_ty = self.parse_type()?;
            let slice_ty_name = elem_ty.unique_name().to_string();

            // Check if slice type already exists
            let slice_meta = self.lookup_meta(&format!("[]{}", slice_ty_name));
            match slice_meta {
                Some(meta) => {
                    if let Meta::Type(slice_ty) = meta.as_ref() {
                        return Ok(slice_ty.clone());
                    } else {
                        return Err(SyntaxError::Generic {
                            message: format!("Expected slice type, got {:?}", meta),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }
                None => {
                    // Create new slice type
                    use crate::ast::SliceType;
                    let slice_ty = Type::Slice(SliceType {
                        elem: Box::new(elem_ty),
                    });
                    // Plan 091: Removed Universe.define_type() - type is returned directly
                    return Ok(slice_ty);
                }
            }
        }

        // Parse array size: [N]T or [expr]T (Plan 052: Runtime arrays)
        // Strategy: Check if next token is literal int (constant array) or something else (runtime array)
        let is_constant_size = self.is_kind(TokenKind::Int)
            || self.is_kind(TokenKind::Uint)
            || self.is_kind(TokenKind::I8)
            || self.is_kind(TokenKind::U8)
            || self.is_kind(TokenKind::RSquare);

        if !is_constant_size {
            // Plan 052: Parse as RuntimeArray (runtime-sized array)
            // Parse size expression (can be any expression: variable, function call, etc.)
            let size_expr = self.parse_expr()?;

            self.expect(TokenKind::RSquare)?; // skip `]`

            // Parse element type
            // Plan 398 §12: use parse_type (recursive) instead of parse_ident,
            // so runtime arrays can have nested/tuple element types too.
            let elem_ty = self.parse_type()?;

            // Create RuntimeArrayType
            use crate::ast::RuntimeArrayType;
            return Ok(Type::RuntimeArray(RuntimeArrayType {
                elem: Box::new(elem_ty),
                size_expr: Box::new(size_expr),
            }));
        }

        // Parse static array: [N]T (constant size)
        let array_size = if self.is_kind(TokenKind::Int)
            || self.is_kind(TokenKind::Uint)
            || self.is_kind(TokenKind::I8)
            || self.is_kind(TokenKind::U8)
        {
            let size = self.parse_ints()?;
            // println!("got int {}", size); // LSP: disabled
            self.get_usize(&size)?
        } else if self.is_kind(TokenKind::RSquare) {
            0
        } else if self.is_kind(TokenKind::Ident) {
            // may be an ident to int variable
            // NOTE: the value should be determined at compile time
            let name = self.parse_name()?;
            let meta = self.lookup_meta(&name);
            let Some(m) = meta else {
                return Err(SyntaxError::Generic {
                    message: format!("Array Size of {} is not found", name),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            };
            match m.as_ref() {
                Meta::Store(store) => {
                    // evaluate store value at compile time
                    // TODO: maybe recursive
                    match store.expr {
                        Expr::Int(n) => n as usize,
                        Expr::Uint(n) => n as usize,
                        Expr::U8(n) => n as usize,
                        Expr::I8(n) => n as usize,
                        _ => 0 as usize,
                    }
                }
                _ => 0,
            }
        } else {
            let message = format!("Expected Array Size or Empty, got {}", self.peek().kind);
            let span = pos_to_span(self.cur.pos);
            return Err(SyntaxError::Generic { message, span }.into());
        };
        self.expect(TokenKind::RSquare)?; // skip `]`

        // parse array elem type
        // Plan 398 §12: use parse_type (recursive) instead of parse_ident,
        // so static arrays can have nested/tuple element types too.
        let ty = self.parse_type()?;
        let array_ty_name = format!("[{}]{}", array_size, ty.unique_name());
        let arry_meta = self.lookup_meta(&array_ty_name);
        match arry_meta {
            Some(meta) => {
                if let Meta::Type(array_ty) = meta.as_ref() {
                    return Ok(array_ty.clone());
                } else {
                    return Err(SyntaxError::Generic {
                        message: format!("Expected array type, got {:?}", meta),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
            None => {
                let array_ty = Type::Array(ArrayType {
                    elem: Box::new(ty.clone()),
                    len: array_size,
                });
                // Plan 091: Removed Universe.define_type() - type is returned directly
                return Ok(array_ty);
            }
        }
    }

    /// Parse function type: fn(Params)ReturnType (Plan 060)
    /// Examples: fn(int)str, fn(int, bool)void, fn()int
    fn parse_fn_type(&mut self) -> AutoResult<Type> {
        self.expect(TokenKind::Fn)?; // Consume 'fn'
        self.parse_fn_signature()
    }

    /// Plan 390 §15.10: parse `(params) [ret]` — the signature shared by
    /// fn-pointer types (`fn(a, b) ret`, via parse_fn_type) and the Fn spec's
    /// closure-trait form (`Fn(a, b)`, via parse_ident_or_generic_type).
    /// Both produce `Type::Fn(params, ret)`; a2r renders a Fn-type arg inside
    /// a Box/Arc container as `dyn Fn(...)` (closure trait) and elsewhere as
    /// `fn(...)` (fn pointer).
    fn parse_fn_signature(&mut self) -> AutoResult<Type> {
        use crate::ast::Type;

        self.expect(TokenKind::LParen)?; // Consume '('

        // Parse parameters
        let mut params = Vec::new();
        if !self.is_kind(TokenKind::RParen) {
            // Parse first parameter type
            params.push(self.parse_type()?);

            // Parse remaining parameter types
            while self.is_kind(TokenKind::Comma) {
                self.next(); // Consume ','
                params.push(self.parse_type()?);
            }
        }

        self.expect(TokenKind::RParen)?; // Consume ')'

        // Parse return type (if present)
        let ret = if self.is_type_name() {
            Box::new(self.parse_type()?)
        } else {
            Box::new(Type::Void)
        };

        Ok(Type::Fn(params, ret))
    }

    /// Parse a single type parameter (e.g., T, K, V)
    #[allow(dead_code)]
    fn parse_type_param(&mut self) -> AutoResult<crate::ast::TypeParam> {
        use crate::ast::TypeParam;

        match self.cur.kind {
            TokenKind::Ident => {
                let name = self.parse_name()?;
                Ok(TypeParam {
                    name,
                    constraint: Vec::new(),
                })
            }
            _ => Err(SyntaxError::Generic {
                message: format!("Expected type parameter, got {}", self.cur.text),
                span: pos_to_span(self.cur.pos),
            }
            .into()),
        }
    }

    /// Parse a generic parameter (Plan 052)
    /// Can be either:
    /// - Type parameter: `T` (no type annotation)
    /// - Const parameter: `N u32` (with type annotation)
    pub(super) fn parse_generic_param(&mut self) -> AutoResult<crate::ast::GenericParam> {
        use crate::ast::{ConstParam, GenericParam, TypeParam};

        // First, parse the parameter name
        if self.cur.kind != TokenKind::Ident {
            return Err(SyntaxError::Generic {
                message: format!("Expected generic parameter name, got {}", self.cur.text),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }

        let name = self.parse_name()?;

        // Plan 417-E3 (DIV-TRAIT-VM-1): bounded type parameter `T has Spec`.
        // `has` is a reserved keyword (TokenKind::Has, also used by
        // `has field Type for SpecName` delegation), so it cannot collide
        // with a const-parameter type name. Multiple bounds join with `+`
        // (`T has A + B`), mirroring the `#[with(T as A + B)]` collection
        // (Plan 364 W3).
        if self.is_kind(TokenKind::Has) {
            self.next(); // skip 'has'
            let mut constraint = Vec::new();
            constraint.push(self.parse_type()?);
            while self.is_kind(TokenKind::Add) {
                self.next(); // skip '+'
                constraint.push(self.parse_type()?);
            }
            return Ok(GenericParam::Type(TypeParam { name, constraint }));
        }

        // Check if next token looks like a type
        // If next token is a type keyword or identifier, it's a const parameter
        if self.next_token_is_type() {
            // Const parameter: `N u32`
            let typ = self.parse_type()?;

            Ok(GenericParam::Const(ConstParam {
                name,
                typ,
                default: None, // TODO: Support default values
            }))
        } else {
            // Type parameter: just `T`
            Ok(GenericParam::Type(TypeParam {
                name,
                constraint: Vec::new(),
            }))
        }
    }

    /// Parse generic parameter list: <T, N u32>
    /// Returns Vec<GenericParam> and adds type parameters to current_type_params
    pub(super) fn parse_generic_params(&mut self) -> AutoResult<Vec<crate::ast::GenericParam>> {
        // Plan 368 FU-1: accept BOTH `<T>` and `[T]` as function generic-param
        // lists. stdlib `.at` files use `[T]` (e.g. json.at `fn encode[T](...)`),
        // while `may.at` uses `<T>`. In the position where this is called (right
        // after a fn name, before `(`), a `[` can ONLY be generics (there's no
        // preceding type for it to be an array of), so the disambiguation is safe.
        let (open, close, close_text) = if self.is_kind(TokenKind::Lt) {
            (TokenKind::Lt, TokenKind::Gt, "'>'")
        } else if self.is_kind(TokenKind::LSquare) {
            (TokenKind::LSquare, TokenKind::RSquare, "']'")
        } else {
            return Ok(Vec::new());
        };

        self.next(); // skip open (`<` or `[`)
        let mut params = Vec::new();

        // Parse comma-separated generic parameters
        loop {
            self.skip_empty_lines();
            params.push(self.parse_generic_param()?);
            self.skip_empty_lines();

            if self.is_kind(close) {
                self.next(); // skip close (`>` or `]`)
                break;
            } else if self.is_kind(TokenKind::Comma) {
                self.next(); // skip `,`
                continue;
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected {} or ',' in generic parameter list, got {}",
                        close_text, self.cur.text
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        // Add type parameters to scope for use in function body
        for param in &params {
            if let crate::ast::GenericParam::Type(tp) = param {
                self.current_type_params.push(tp.name.clone());
            } else if let crate::ast::GenericParam::Const(cp) = param {
                // Store const parameter with its actual type for type resolution
                self.current_const_params
                    .insert(cp.name.clone(), cp.typ.clone());
            }
        }

        Ok(params)
    }

    /// Parse identifier type or generic instance (e.g., List, List<int>)
    fn parse_ident_or_generic_type(&mut self) -> AutoResult<Type> {
        use crate::ast::Type;

        let ident = self.parse_ident()?;

        match ident {
            Expr::Ident(name) => {
                // Special case: Dynamic storage type (Plan 055)
                if name.as_str() == "Dynamic" {
                    return Ok(Type::Storage(crate::ast::StorageType {
                        kind: crate::ast::StorageKind::Dynamic,
                    }));
                }

                // Plan 091: Handle primitive types directly (don't depend on Universe)
                match name.as_str() {
                    "int" | "i32" => return Ok(Type::Int),
                    "uint" | "u32" => return Ok(Type::Uint),
                    "float" => return Ok(Type::Float),
                    "double" | "f64" => return Ok(Type::Double),
                    "bool" => return Ok(Type::Bool),
                    "str" => return Ok(Type::StrSlice),
                    "Str" => return Ok(Type::StrOwned),
                    "cstr" => return Ok(Type::CStrLit),
                    "byte" | "u8" => return Ok(Type::Byte),
                    "char" | "i8" => return Ok(Type::Char),
                    "void" => return Ok(Type::Void),
                    _ => {}
                }

                // Plan 390 §15.10: `Fn(...)` closure-trait signature form
                // (`Box<Fn(A)>` → `Box<dyn Fn(A)>`). Parse like a fn-pointer
                // signature and produce Type::Fn — the transpiler renders a
                // Fn-type arg inside a Box/Arc container as `dyn Fn(...)`.
                // A bare `Fn` (no parens) falls through to the spec lookup
                // below (Type::Spec → `Box<dyn Fn>`).
                if name.as_str() == "Fn" && self.is_kind(TokenKind::LParen) {
                    return self.parse_fn_signature();
                }

                // Handle dotted module paths: flow.FlowSpec -> User("flow.FlowSpec")
                if self.is_kind(TokenKind::Dot) {
                    let mut qualified = name.to_string();
                    while self.is_kind(TokenKind::Dot) {
                        self.next(); // consume .
                        let next = self.cur.text.clone();
                        self.next(); // consume ident
                        qualified.push('.');
                        qualified.push_str(&next);
                    }
                    // Check for generic args after the qualified name
                    if self.cur.kind == TokenKind::Lt && self.next_token_is_type() {
                        return self.parse_generic_instance(AutoStr::from(qualified));
                    }
                    return Ok(Type::User(TypeDecl {
                        consts: Vec::new(),
                        name: AutoStr::from(qualified),
                        kind: TypeDeclKind::UserType,
                        parent: None,
                        has: Vec::new(),
                        specs: Vec::new(),
                        spec_impls: Vec::new(),
                        generic_params: Vec::new(),
                        members: Vec::new(),
                        delegations: Vec::new(),
                        methods: Vec::new(),
                        attrs: vec![],
                        impl_attrs: vec![],
                        doc: None,
                        is_pub: false,
                    }));
                }

                // Check if this is a generic instance (e.g., List<int>, May<string>)
                if self.cur.kind == TokenKind::Lt {
                    // Context check: make sure < is followed by a type
                    // We need to look ahead to see if this looks like a generic instance
                    if self.next_token_is_type() {
                        return self.parse_generic_instance(name);
                    }
                }
                // Plan 348 G2: Support square-bracket generic syntax in type
                // position (e.g., Handle[int], List[int], May[int]). This only
                // applies when a `[` immediately follows a type-name identifier,
                // so it does not conflict with array types (`[N]T`) which start
                // with `[`. We require a lookahead type check to avoid consuming
                // an index expression in other contexts.
                if self.cur.kind == TokenKind::LSquare && self.next_token_is_type() {
                    return self.parse_generic_instance_delim(
                        name,
                        TokenKind::LSquare,
                        TokenKind::RSquare,
                    );
                }
                // Support Result(T, E) style generic with parens
                if self.cur.kind == TokenKind::LParen
                    && (name == "Result" || name == "Option" || name == "List" || name == "Map")
                {
                    self.next(); // consume (
                    let mut args = vec![self.parse_type()?];
                    while self.cur.kind == TokenKind::Comma {
                        self.next(); // consume ,
                        args.push(self.parse_type()?);
                    }
                    self.expect(TokenKind::RParen)?;
                    // Plan 514 W1 (P511-5 探针暴露): 圆括号形态的 `List(T)` 是
                    // VM 内建列表（auto.list，29_list_shims 语义），须与尖括号
                    // `List<T>`（:10921 → Type::List → a2r Vec<T>）同型；此前落
                    // GenericInstance 会让 a2r 发射未解析的 `List<T>`。用户自定义
                    // 双参泛型 `type List<T, S>`（a2r/10_collections/002）不受影响
                    // —— 其 args.len() == 2，保持 GenericInstance。
                    if name == "List" && args.len() == 1 {
                        return Ok(Type::List(Box::new(args.pop().unwrap())));
                    }
                    return Ok(Type::GenericInstance(GenericInstance {
                        base_name: name,
                        args,
                        source: None,
                    }));
                }

                // Plan 052: Check if this identifier is a const generic parameter
                if let Some(const_ty) = self.current_const_params.get(&name) {
                    // This is a const parameter - return its actual type (e.g., uint)
                    return Ok(const_ty.clone());
                }

                // Plan 058: Check if this is a type alias
                let type_alias = self
                    .scope
                    .lookup_type_alias(name.as_str())
                    .map(|(params, target)| (params.clone(), target.clone()));

                if let Some((alias_params, alias_target)) = type_alias {
                    // Check if there are type arguments (e.g., List<int>)
                    if self.cur.kind == TokenKind::Lt && self.next_token_is_type() {
                        // Parse type arguments
                        self.next(); // Consume '<'
                        let mut type_args = Vec::new();
                        type_args.push(self.parse_type()?);

                        while self.cur.kind == TokenKind::Comma {
                            self.next(); // Consume ','
                            type_args.push(self.parse_type()?);
                        }

                        self.expect(TokenKind::Gt)?; // Consume '>'

                        // Check if the number of type arguments matches the alias parameters
                        if type_args.len() != alias_params.len() {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "Type alias '{}' expects {} type parameter(s), but got {}",
                                    name,
                                    alias_params.len(),
                                    type_args.len()
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }

                        // Substitute type arguments into the target type
                        let substituted = alias_target.substitute(&alias_params, &type_args);
                        return Ok(substituted);
                    } else if !alias_params.is_empty() {
                        // Type alias requires parameters but none were provided
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Type alias '{}' requires {} type parameter(s), but none were provided",
                                name,
                                alias_params.len()
                            ),
                            span: pos_to_span(self.cur.pos),
                        }.into());
                    } else {
                        // No type arguments needed - return the target type directly
                        return Ok(alias_target);
                    }
                }

                // Check if this identifier is a type parameter in the current scope
                if self.current_type_params.contains(&name) {
                    // This is a type parameter - return it as a user type for now
                    // The type system will handle substitution later
                    return Ok(Type::User(TypeDecl {
                        consts: Vec::new(),
                        name: name.clone(),
                        kind: TypeDeclKind::UserType,
                        parent: None,
                        has: Vec::new(),
                        specs: Vec::new(),
                        spec_impls: Vec::new(), // Plan 057
                        generic_params: Vec::new(),
                        members: Vec::new(),
                        delegations: Vec::new(),
                        methods: Vec::new(),
                        attrs: vec![],
                        impl_attrs: vec![],
                        doc: None,
                        is_pub: false,
                    }));
                }

                // Regular type name - look it up in the type registry
                Ok(self.lookup_type(&name).borrow().clone())
            }
            _ => Err(SyntaxError::Generic {
                message: format!("Expected type, got ident {:?}", ident),
                span: pos_to_span(self.cur.pos),
            }
            .into()),
        }
    }

    /// Parse generic instance (e.g., List<int>, Map<str, int>, MyType<T, U>)
    fn parse_generic_instance(&mut self, base_name: Name) -> AutoResult<Type> {
        // Default delimiter is angle brackets `<...>` (e.g. List<int>, Handle<int>).
        self.parse_generic_instance_delim(base_name, TokenKind::Lt, TokenKind::Gt)
    }

    /// Plan 348 G2: Parse a generic type instance with an explicit delimiter
    /// pair. Supports both angle brackets `Ident<T>` and square brackets
    /// `Ident[T]` (e.g. `Handle[int]`, `List[int]`) in type position.
    fn parse_generic_instance_delim(
        &mut self,
        base_name: Name,
        open: TokenKind,
        close: TokenKind,
    ) -> AutoResult<Type> {
        use crate::ast::{GenericInstance, Type};

        self.expect(open)?;

        let mut args = Vec::new();
        args.push(self.parse_type()?);

        while self.cur.kind == TokenKind::Comma {
            self.next(); // Consume ','
            args.push(self.parse_type()?);
        }

        self.expect(close)?;

        // Check if base_name refers to a user-defined generic Tag or TypeDecl
        let base_type = self.lookup_type(&base_name);

        // Plan 055: Check if we need to fill in default parameters from environment
        // Clone needed data to avoid borrow issues
        let needs_default_param = {
            let base_type_ref = base_type.borrow();
            match &*base_type_ref {
                Type::User(type_decl) if !type_decl.generic_params.is_empty() => {
                    // User-defined generic TypeDecl with type parameters
                    // Check if user provided fewer parameters than expected
                    if args.len() < type_decl.generic_params.len() {
                        Some((base_name.clone(), type_decl.generic_params.len()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        };

        let mut args = args;
        if let Some((type_name, expected_params)) = needs_default_param {
            // Fill in missing parameters from environment
            // For now, only support List<T, S> pattern where S has a default
            if type_name.as_str() == "List" && args.len() == 1 && expected_params == 2 {
                // Get default storage from environment
                let default_storage = self
                    .scope
                    .get_env_val("DEFAULT_STORAGE")
                    .unwrap_or_else(|| "Heap".into()); // Fallback to Heap for PC

                // Parse the storage type from environment string
                let storage_type = self.parse_storage_from_env(&default_storage)?;
                args.push(storage_type);
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Type '{}' expects {} parameter(s), but got {}",
                        type_name,
                        expected_params,
                        args.len()
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        let base_type_ref = base_type.borrow();

        match &*base_type_ref {
            Type::Tag(tag_shared) if !tag_shared.borrow().generic_params.is_empty() => {
                // User-defined generic Tag with type parameters
                // Perform substitution and create new Tag instance
                let tag = tag_shared.borrow().clone();
                drop(base_type_ref); // Drop borrow before registering

                // Collect generic parameter names
                let param_names: Vec<_> = tag
                    .generic_params
                    .iter()
                    .map(|gp| match gp {
                        crate::ast::GenericParam::Type(tp) => tp.name.clone(),
                        crate::ast::GenericParam::Const(cp) => cp.name.clone(),
                    })
                    .collect();

                // Substitute type parameters in tag fields
                let substituted_fields: Vec<_> = tag
                    .fields
                    .iter()
                    .map(|field| TagField {
                        name: field.name.clone(),
                        ty: field.ty.substitute(&param_names, &args),
                    })
                    .collect();

                // Create new substituted tag
                let substituted_tag = Tag {
                    name: format!(
                        "{}_{}",
                        base_name,
                        args.iter()
                            .map(|t| t.unique_name().to_string())
                            .collect::<Vec<_>>()
                            .join("_")
                    )
                    .into(),
                    generic_params: Vec::new(), // No type parameters in instantiated tag
                    fields: substituted_fields,
                    methods: tag.methods.clone(),
                };

                // Register the substituted tag
                let subst_name = substituted_tag.name.clone();
                self.define(
                    subst_name.as_str(),
                    Meta::Type(Type::Tag(shared(substituted_tag.clone()))),
                );

                return Ok(Type::Tag(shared(substituted_tag)));
            }
            // Plan 190: Rust type with generic params (e.g., HashMap<str, int>)
            Type::Rust(rust_source) => {
                return Ok(Type::GenericInstance(GenericInstance {
                    base_name: base_name.clone(),
                    args,
                    source: Some(rust_source.clone()),
                }));
            }
            Type::User(type_decl) if !type_decl.generic_params.is_empty() => {
                // User-defined generic TypeDecl with type parameters
                // TODO: Implement TypeDecl substitution (similar to Tag substitution)
                // For now, return GenericInstance
                drop(base_type_ref);
                return Ok(Type::GenericInstance(GenericInstance {
                    base_name,
                    args,
                    source: None,
                }));
            }
            _ => {
                // Either built-in type or non-generic user-defined type
                drop(base_type_ref);
            }
        }

        // Special handling for built-in generic types
        // List<T> has dedicated Type variant (May<T> is now a generic tag)
        // Fixed<N> is a Storage type (Plan 055)
        match base_name.as_str() {
            "List" => {
                // Plan 055: Support both List<T> (1 param) and List<T, S> (2 params)
                if args.len() == 1 {
                    // List<int> → Type::List(Box::new(int))
                    // Storage will be determined at runtime by VM
                    Ok(Type::List(Box::new(args.into_iter().next().unwrap())))
                } else if args.len() == 2 {
                    // List<int, Heap> → Return GenericInstance for full type
                    // This allows the transpiler to see both parameters
                    Ok(Type::GenericInstance(GenericInstance {
                        base_name,
                        args,
                        source: None,
                    }))
                } else {
                    Err(SyntaxError::Generic {
                        message: format!(
                            "List expects 1 or 2 type parameter(s), but got {}",
                            args.len()
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into())
                }
            }
            "Map" => match args.len() {
                2 => Ok(Type::Map(
                    Box::new(args[0].clone()),
                    Box::new(args[1].clone()),
                )),
                // Plan 396/B11(b):  (single arg) leaves the value type
                // unspecified — default V to Unknown, mirroring the bare
                // shortcut (parser.rs common-collection default). The previous
                // hard error rejected valid sources (auto-ai tool.at's
                // ), silently skipping whole-module transpile.
                1 => Ok(Type::Map(
                    Box::new(args[0].clone()),
                    Box::new(Type::Unknown),
                )),
                n => Err(SyntaxError::Generic {
                    message: format!(
                        "Map expects 1 or 2 type parameters (Map<K> / Map<K, V>), but got {}",
                        n
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into()),
            },
            "Fixed" if args.len() == 1 => {
                // Fixed<N> storage type - parse capacity from first argument
                // The capacity should be a constant expression (int literal or const)
                let capacity = match &args[0] {
                    Type::Int => {
                        // For now, Fixed<int> will use default capacity
                        // TODO: Parse actual integer value from expression
                        64
                    }
                    Type::Uint => {
                        // For now, Fixed<uint> will use default capacity
                        // TODO: Parse actual integer value from expression
                        64
                    }
                    _ => {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Fixed storage requires integer capacity, got {}",
                                args[0]
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                };

                Ok(Type::Storage(crate::ast::StorageType {
                    kind: crate::ast::StorageKind::Fixed { capacity },
                }))
            }
            _ => {
                // User-defined generic instance (including May<T> from stdlib)
                Ok(Type::GenericInstance(GenericInstance {
                    base_name,
                    args,
                    source: None,
                }))
            }
        }
    }

    /// Parse a storage type name from environment variable string
    /// For Plan 055: Convert "Heap" → Type::Storage(Dynamic), "InlineInt64" → proper type
    fn parse_storage_from_env(&mut self, storage_name: &str) -> AutoResult<Type> {
        match storage_name {
            "Heap" | "Dynamic" => Ok(Type::Storage(crate::ast::StorageType {
                kind: crate::ast::StorageKind::Dynamic,
            })),
            "InlineInt64" | "Fixed" => Ok(Type::Storage(crate::ast::StorageType {
                kind: crate::ast::StorageKind::Fixed { capacity: 64 },
            })),
            _ => {
                // Try to parse as Fixed<N> pattern
                if storage_name.starts_with("Fixed<") && storage_name.ends_with(">") {
                    let inner = &storage_name[6..storage_name.len() - 1];
                    if let Ok(capacity) = inner.parse::<usize>() {
                        return Ok(Type::Storage(crate::ast::StorageType {
                            kind: crate::ast::StorageKind::Fixed { capacity },
                        }));
                    }
                }

                // Fallback: treat as Dynamic
                Ok(Type::Storage(crate::ast::StorageType {
                    kind: crate::ast::StorageKind::Dynamic,
                }))
            }
        }
    }

    #[allow(dead_code)]
    fn parse_ident_type(&mut self) -> AutoResult<Type> {
        let ident = self.parse_ident()?;
        match ident {
            Expr::Ident(name) => Ok(self.lookup_type(&name).borrow().clone()),
            _ => Err(SyntaxError::Generic {
                message: format!("Expected type, got ident {:?}", ident),
                span: pos_to_span(self.cur.pos),
            }
            .into()),
        }
    }

    pub(super) fn is_type_name(&mut self) -> bool {
        self.is_kind(TokenKind::Ident) // normal types like `int`
        || self.is_kind(TokenKind::Question) // Option types like `?int` (Plan 120)
        || self.is_kind(TokenKind::Not) // Result types like `!int` (Plan 120)
        || self.is_kind(TokenKind::Tilde) // Future types like `~int` (Plan 124)
        || self.is_kind(TokenKind::LSquare) // array types like `[5]int`
        || self.is_kind(TokenKind::Star) // ptr types like `*int`
        || self.is_kind(TokenKind::At) // ref types like `@int`
        || self.is_kind(TokenKind::Fn) // function types like `fn(int)str` (Plan 060)
        || self.is_keyword_as_type() // keywords that can serve as type names (e.g., Link)
    }

    /// Check if the current token is a keyword that can also serve as a type name.
    /// Some keywords like `Link`, `Type` are PascalCase identifiers that users
    /// may use as enum/type names.
    fn is_keyword_as_type(&self) -> bool {
        matches!(self.cur.kind, TokenKind::Link | TokenKind::Type)
    }

    pub fn parse_type(&mut self) -> AutoResult<Type> {
        let base = self.parse_type_base()?;
        // Plan 368 FU-1: support suffix `?` on any type (e.g. `JsonValue?`,
        // `T?`, `int?`). stdlib `.at` files declare nullable return types this
        // way (json.at `fn parse(s str) JsonValue?;`). The prefix `?T` form
        // (handled inside parse_type_base) stays for backward compat.
        let mut ty = base;
        while self.is_kind(TokenKind::Question) {
            self.next(); // consume '?'
            ty = Type::Option(Box::new(ty));
        }
        Ok(ty)
    }

    /// Parse the base type (without a trailing `?` suffix). Plan 368 FU-1 split
    /// this out of parse_type so the suffix-`?` loop can wrap any base type.
    fn parse_type_base(&mut self) -> AutoResult<Type> {
        match self.cur.kind {
            // Plan 380 P1 (defect D): `dyn Trait` — allows Arc<dyn Client> /
            // Box<dyn Trait> field types (the `dyn` keyword inside generic
            // args). Represented as a User type whose name is "dyn Trait", which
            // the transpiler renders verbatim (see rust.rs Type::User dyn-prefix
            // guard).
            TokenKind::Ident if self.cur.text.as_str() == "dyn" => {
                self.next(); // consume 'dyn'
                let trait_type = self.parse_type()?;
                // Use the trait's display name (e.g. "Client")
                let mut dyn_name = format!("dyn {}", trait_type.unique_name());
                // Plan 599 (004 §5④): `+ Send [+ Sync]` marker-trait bounds —
                // engine shapes need Box<dyn MasterPty + Send> (the reader-
                // thread field spelling). Appended verbatim to the spelling.
                while self.is_kind(TokenKind::Add) {
                    self.next(); // consume '+'
                    if !self.is_kind(TokenKind::Ident) {
                        return Err(SyntaxError::Generic {
                            message: "dyn trait object: expected trait name after '+'".to_string(),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                    let bound = self.cur.text.clone();
                    self.next();
                    dyn_name.push_str(&format!(" + {}", bound));
                }
                Ok(Type::User(TypeDecl {
                    consts: Vec::new(),
                    name: dyn_name.into(),
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
                    impl_attrs: vec![],
                    doc: None,
                    is_pub: false,
                }))
            }
            TokenKind::At => {
                // Plan 384 A3: `@T` → Type::Reference(T) (Rust `&T`).
                // Enables reference-aware call-site injection: when an extern
                // function's param is declared `@T` (in extern_sigs.at), a2r
                // borrows the argument (`&arg`) at the call site.
                self.next(); // Consume '@'
                let inner_type = self.parse_type()?;
                Ok(Type::Reference(Box::new(inner_type)))
            }
            TokenKind::Question => {
                // Plan 120: Parse ?T as Type::Option(T)
                self.next(); // Consume '?'
                let inner_type = self.parse_type()?;
                Ok(Type::Option(Box::new(inner_type)))
            }
            TokenKind::Not => {
                // Plan 120: Parse !T as Type::Result(T)
                // Plan 121: ! without type means Result<void> (e.g., fn main() !)
                self.next(); // Consume '!'
                             // Check if there's a type following the '!'
                if self.is_type_name() {
                    let inner_type = self.parse_type()?;
                    Ok(Type::Result(Box::new(inner_type)))
                } else {
                    // ! without inner type means Result<void>
                    Ok(Type::Result(Box::new(Type::Void)))
                }
            }
            TokenKind::Tilde => {
                // Plan 124/321: ~ is syntactic sugar for delayed/async types.
                // ~T → Future<T> (existing async semantics)
                // ~Iter<T> → Iter<T> (generator sequence, NOT wrapped in Future)
                // ~Stream<T> → Stream<T> (async stream, NOT wrapped in Future)
                self.next(); // Consume '~'
                let inner_type = self.parse_type()?;
                // Check if inner is Iter<T> or Stream<T> — these are direct types, not Future
                match &inner_type {
                    Type::GenericInstance(inst)
                        if inst.base_name.as_str() == "Iter"
                            || inst.base_name.as_str() == "Stream" =>
                    {
                        // ~Iter<T> / ~Stream<T> → return directly (no Future wrapping)
                        Ok(inner_type)
                    }
                    _ => {
                        // ~T → Future<T> (default: async value)
                        Ok(Type::GenericInstance(GenericInstance {
                            base_name: "Future".into(),
                            args: vec![inner_type],
                            source: None,
                        }))
                    }
                }
            }
            TokenKind::Ident => self.parse_ident_or_generic_type(),
            TokenKind::Star => self.parse_ptr_type(),
            TokenKind::LSquare => self.parse_array_type(),
            TokenKind::Fn => self.parse_fn_type(), // Plan 060: function types like fn(int)str
            TokenKind::LParen => {
                // Plan 200: Tuple type (T1, T2, ...)
                self.next(); // skip (
                             // Plan 391 D5: `()` is the unit type (Rust `()`), e.g. Result<(), str>.
                             // Map to Type::Void — rust.rs type_name already emits it as `()`.
                if self.is_kind(TokenKind::RParen) {
                    self.next(); // skip )
                    return Ok(Type::Void);
                }
                let mut types = vec![self.parse_type()?];
                while self.is_kind(TokenKind::Comma) {
                    self.next(); // skip ,
                    types.push(self.parse_type()?);
                }
                self.expect(TokenKind::RParen)?;
                if types.len() == 1 {
                    // Single-element "tuple" is just a grouped type
                    Ok(types.remove(0))
                } else {
                    Ok(Type::Tuple(types))
                }
            }
            // Allow keyword tokens that can also serve as type/ident names (e.g., Link, Path, Type, Color)
            _ if self
                .cur
                .text
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false) =>
            {
                // Keywords that look like type names (PascalCase) should be treated as identifiers
                self.parse_ident_or_generic_type()
            }
            _ => {
                let message = format!("Expected type, got {}", self.cur.text);
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        }
    }

    // TODO: 暂时只检查3种情况：
    // 1，简单名称；
    // 2，点号表达式最左侧的名称
    // 3, 函数调用，如果函数名不存在，表示是一个节点实例
    /// Check if a name refers to an imported module (from `use` statements)
    pub(super) fn is_imported_module(&self, name: &str) -> bool {
        for use_path in &self.use_imports {
            if let Some(alias) = use_path.rsplit('.').next() {
                if alias == name {
                    return true;
                }
            }
        }
        false
    }

    pub fn find_type_for_expr(&mut self, expr: &Expr) -> AutoResult<Type> {
        match expr {
            // function name, find it's decl
            Expr::Ident(ident) => {
                let meta = self.lookup_meta(ident);
                let Some(meta) = meta else {
                    return Err(SyntaxError::Generic {
                        message: format!("Function name not found! {}", ident),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                };
                match meta.as_ref() {
                    Meta::Fn(fun) => {
                        return Ok(fun.ret.clone());
                    }
                    _ => {}
                }
            }
            // method or function in a struct
            Expr::Bina(lhs, op, rhs) => {
                if let Op::Dot = op {
                    match &**lhs {
                        Expr::Ident(lname) => {
                            // lookup meta for left name
                            let meta = self.lookup_meta(lname.as_str());
                            let Some(meta) = meta else {
                                return Err(SyntaxError::Generic {
                                    message: format!("Left name not found! {}.{}", lname, rhs),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into());
                            };
                            match meta.as_ref() {
                                Meta::Type(typ) => match typ {
                                    Type::Tag(tag) => {
                                        if let Expr::Ident(rname) = &**rhs {
                                            let rtype = tag.borrow().get_field_type(rname);
                                            return Ok(rtype);
                                        }
                                    }
                                    Type::User(_) => {
                                        // Allow type names for static method calls (e.g., HashMap.new())
                                        // The return type will be determined by the evaluator
                                        if let Expr::Ident(rname) = &**rhs {
                                            // For static methods, try to look up the method signature
                                            let combined_name = format!("{}::{}", lname, rname);
                                            if let Some(method_meta) =
                                                self.lookup_meta(&combined_name)
                                            {
                                                if let Meta::Fn(fn_decl) = method_meta.as_ref() {
                                                    return Ok(fn_decl.ret.clone());
                                                }
                                            }
                                            // Default: assume it returns the type itself (like constructors)
                                            return Ok(typ.clone());
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Err(SyntaxError::Generic {
            message: format!("Meta not found! {}", expr),
            span: pos_to_span(self.cur.pos),
        }
        .into())
    }

    pub fn return_type(&mut self, call_name: &Expr) -> AutoResult<Type> {
        match call_name {
            // function name, find it's decl
            Expr::Ident(ident) => {
                let meta = self.lookup_meta(ident);
                let Some(meta) = meta else {
                    return Ok(Type::Unknown);
                };
                match meta.as_ref() {
                    Meta::Fn(fun) => {
                        return Ok(fun.ret.clone());
                    }
                    _ => {}
                }
            }
            // method or function in a struct
            Expr::Bina(lhs, op, rhs) => {
                if let Op::Dot = op {
                    match &**lhs {
                        Expr::Ident(lname) => {
                            // lookup meta for left name
                            let meta = self.lookup_meta(lname.as_str());
                            let Some(meta) = meta else {
                                return Err(SyntaxError::Generic {
                                    message: format!("Left name not found! {}.{}", lname, rhs),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into());
                            };
                            match meta.as_ref() {
                                Meta::Type(typ) => match typ {
                                    Type::Tag(tag) => {
                                        if let Expr::Ident(rname) = &**rhs {
                                            if tag.borrow().has_field(rname) {
                                                return Ok(typ.clone());
                                            }
                                            let rtype = tag.borrow().get_field_type(rname);
                                            return Ok(rtype);
                                        }
                                    }
                                    Type::User(_) => {
                                        // Allow type names for static method calls (e.g., HashMap.new())
                                        if let Expr::Ident(rname) = &**rhs {
                                            let combined_name = format!("{}::{}", lname, rname);
                                            if let Some(method_meta) =
                                                self.lookup_meta(&combined_name)
                                            {
                                                if let Meta::Fn(fn_decl) = method_meta.as_ref() {
                                                    return Ok(fn_decl.ret.clone());
                                                }
                                            }
                                            return Ok(typ.clone());
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Plan 052/060: Handle Expr::Dot for generic type method calls
            // e.g., List<int, Heap>.new() where lhs is GenName("List<int, Heap>")
            Expr::Dot(lhs, rhs) => {
                // Extract base type name from GenName or Ident
                let base_name = match lhs.as_ref() {
                    Expr::GenName(name) => {
                        // Extract base name from "List<int, Heap>" -> "List"
                        let name_str = name.as_str();
                        if let Some(pos) = name_str.find('<') {
                            &name_str[..pos]
                        } else {
                            name_str
                        }
                    }
                    Expr::Ident(name) => name.as_str(),
                    _ => {
                        return Ok(Type::Unknown);
                    }
                };

                // Lookup meta for the base type
                let meta = self.lookup_meta(base_name);
                let Some(meta) = meta else {
                    return Ok(Type::Unknown);
                };

                match meta.as_ref() {
                    Meta::Type(typ) => match typ {
                        Type::User(_decl) => {
                            // Check for static methods (e.g., List.new not List::new)
                            // rhs is AutoStr (field/method name)
                            let combined_name = format!("{}.{}", base_name, rhs);
                            if let Some(method_meta) = self.lookup_meta(&combined_name) {
                                if let Meta::Fn(fn_decl) = method_meta.as_ref() {
                                    return Ok(fn_decl.ret.clone());
                                }
                            }
                            // Return the type itself if no method found
                            return Ok(typ.clone());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        // TODO: should check all types or print error
        Ok(Type::Unknown)
    }
}
