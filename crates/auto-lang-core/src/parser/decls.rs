//! PLAN-843 结构拆分（自 parser.rs 原样搬移；算法/行为零变化）。
use super::*;

impl<'a> Parser<'a> {
    /// Parse enum declarations supporting 3 forms:
    ///
    /// - **Scalar**: `enum Color { Red, Green }` or `enum HttpCode u16 { OK = 200 }`
    /// - **Homogeneous**: `enum Vertex Point { LeftTop, RightTop }`
    /// - **Heterogeneous**: `enum Msg { Quit, Move Point, Write string }`
    ///
    /// Also handles `tag` keyword (deprecated) by redirecting here.
    pub(super) fn enum_stmt(&mut self) -> AutoResult<Stmt> {
        // Support both 'enum' and 'tag' keywords (tag is deprecated)
        if self.is_kind(TokenKind::Tag) || self.is_kind(TokenKind::Enum) {
            self.next(); // skip 'enum' or 'tag'
        }
        let name: AutoStr = self.cur.text.clone().into();
        self.next();

        // === Parse optional generic params: enum Name<T> { ... } ===
        let mut generic_params = Vec::new();
        if self.cur.kind == TokenKind::Lt {
            self.next();
            generic_params.push(self.parse_generic_param()?);
            while self.cur.kind == TokenKind::Comma {
                self.next();
                generic_params.push(self.parse_generic_param()?);
            }
            self.expect(TokenKind::Gt)?;
        }

        let kind;
        let items;

        if self.is_kind(TokenKind::LBrace) {
            // enum Name { ... } — examine body to determine Scalar vs Heterogeneous
            let parsed = self.parse_enum_body(&name, &generic_params)?;
            kind = parsed.0;
            items = parsed.1;
        } else if self.is_kind(TokenKind::Ident) {
            let next_text = self.cur.text.to_string();
            if Self::is_integer_type_name(&next_text) {
                // enum Name u16 { ... } → Scalar with repr_type
                let repr_type = self.parse_type()?;
                let scalar_items = self.parse_scalar_enum_items()?;
                kind = EnumKind::Scalar {
                    repr_type: Some(repr_type),
                };
                items = scalar_items;
            } else {
                // Check if it's a known type (for Homogeneous)
                let type_lookup = self.lookup_type(&next_text);
                let borrowed = type_lookup.borrow();
                match *borrowed {
                    Type::User(_) | Type::Tag(_) | Type::Enum(_) => {
                        let payload_type = borrowed.clone();
                        drop(borrowed);
                        self.next(); // skip type name
                        let homo_items = self.parse_homo_enum_items()?;
                        kind = EnumKind::Homogeneous { payload_type };
                        items = homo_items;
                    }
                    _ => {
                        let span = pos_to_span(self.cur.pos);
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "expected '{{' or known type after '{}', got '{}'",
                                name, next_text
                            ),
                            span,
                        }
                        .into());
                    }
                }
            }
        } else {
            let span = pos_to_span(self.cur.pos);
            return Err(SyntaxError::Generic {
                message: format!("expected '{{' or type after '{}'", name),
                span,
            }
            .into());
        }

        let enum_decl = EnumDecl {
            name: name.clone(),
            items,
            kind,
            is_pub: false,
            doc: self.take_docs(),
            // Plan 364 W1: consume any leftover impl-level dotted macros so they
            // don't leak to the next decl. Enums render them before the enum decl
            // (imperfect placement; enums rarely carry impl macros).
            attrs: {
                let mut a = std::mem::take(&mut self.raw_attrs);
                a.append(&mut self.impl_attrs);
                a
            },
        };
        self.register_enum_decl(&enum_decl, &generic_params);
        Ok(Stmt::EnumDecl(enum_decl))
    }

    /// Helper: check if a string is a known integer type name.
    fn is_integer_type_name(s: &str) -> bool {
        matches!(
            s,
            "int"
                | "uint"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "byte"
        )
    }

    /// Parse the body of `enum Name { ... }`, determining Scalar vs Heterogeneous
    /// by checking if any items have payload types.
    fn parse_enum_body(
        &mut self,
        name: &AutoStr,
        generic_params: &[crate::ast::GenericParam],
    ) -> AutoResult<(EnumKind, Vec<EnumItem>)> {
        // Set current type params for generic parsing
        let prev_type_params = std::mem::replace(
            &mut self.current_type_params,
            generic_params
                .iter()
                .map(|gp| match gp {
                    crate::ast::GenericParam::Type(tp) => tp.name.clone(),
                    crate::ast::GenericParam::Const(cp) => cp.name.clone(),
                })
                .collect(),
        );

        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        let mut items = Vec::new();
        let mut methods = Vec::new();
        let mut has_any_payload = false;
        let mut last_val = 0i32;

        while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
            // Check for methods inside enum body
            if self.is_kind(TokenKind::Fn) || self.is_kind(TokenKind::Static) {
                let is_static = self.is_kind(TokenKind::Static);
                if is_static {
                    self.next(); // skip static
                    self.skip_empty_lines();
                }
                let fn_stmt = self.fn_decl_stmt(&Name::from(name.as_str()))?;
                if let Stmt::Fn(fn_expr) = fn_stmt {
                    // Mark as static if needed
                    let mut fn_expr = fn_expr;
                    if is_static {
                        fn_expr.is_static = true;
                    }
                    methods.push(fn_expr);
                }
                self.expect_eos(false)?;
                self.skip_empty_lines();
                continue;
            }

            // Plan 382: variant attributes `#[from]` / `#[error("...")]`.
            // Lexed as Hash + LSquare + ident (+ optional parenthesized args).
            // Captured as raw strings (e.g. "from", "error(\"client: {0}\")") and
            // attached to the EnumItem; next() auto-skips doc comments.
            let mut variant_attrs: Vec<AutoStr> = Vec::new();
            while self.is_kind(TokenKind::Hash) {
                self.next(); // skip '#'
                self.expect(TokenKind::LSquare)?;
                let mut attr_str = String::new();
                if self.is_kind(TokenKind::Ident) {
                    attr_str.push_str(&self.cur.text);
                    self.next(); // skip the attribute name
                    if let Some(args) = self.collect_annotation_args() {
                        attr_str.push_str(&args);
                    }
                }
                self.expect(TokenKind::RSquare)?;
                variant_attrs.push(attr_str.into());
                self.skip_empty_lines();
            }

            // Parse variant
            let item_name: AutoStr = self.cur.text.clone().into();
            self.next();

            let mut scalar_value = None;
            let mut value_explicit = false;
            let mut payload_type = None;
            let mut payload_types: Vec<Type> = vec![];

            if self.is_kind(TokenKind::Asn) {
                // Variant = value (Scalar form)
                self.next();
                let value = self.parse_ints()?;
                let value = self.get_int_expr(&value);
                scalar_value = Some(value as i32);
                value_explicit = true;
                last_val = value as i32 + 1;
            } else if self.is_kind(TokenKind::LBrace) {
                // Plan 201 Phase 1B: Struct-like variant: Name { field1 Type, field2 Type, ... }
                let fields = self.parse_enum_variant_fields()?;
                has_any_payload = true;
                items.push(EnumItem {
                    name: item_name,
                    scalar_value: None,
                    value_explicit: false,
                    payload_type: None,
                    payload_types: vec![],
                    fields,
                    attrs: variant_attrs,
                });
                self.expect_eos(false)?;
                self.skip_empty_lines();
                continue;
            } else if self.is_kind(TokenKind::LParen) {
                // Tuple variant: ToolUse(str, str, str)
                self.next(); // consume '('
                let mut types = vec![];
                loop {
                    types.push(self.parse_type()?);
                    if self.is_kind(TokenKind::Comma) {
                        self.next(); // consume ','
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                if types.len() == 1 {
                    payload_type = Some(types.into_iter().next().unwrap());
                } else {
                    payload_types = types;
                }
                has_any_payload = true;
            } else if self.is_kind(TokenKind::Ident)
                || self.is_kind(TokenKind::Question)
                || self.is_kind(TokenKind::Not)
            {
                // Single-type variant: Text str, Some ?int, Err !str
                payload_type = Some(self.parse_type()?);
                has_any_payload = true;
            } else {
                // No value, no type — plain variant, auto-increment for scalar
                scalar_value = if last_val != 0 { Some(last_val) } else { None };
                last_val += 1;
            }

            items.push(EnumItem {
                name: item_name,
                scalar_value,
                value_explicit,
                payload_type,
                payload_types,
                fields: vec![],
                attrs: variant_attrs,
            });
            self.expect_eos(false)?;
            self.skip_empty_lines();
        }

        self.expect(TokenKind::RBrace)?;

        // Restore type params
        self.current_type_params = prev_type_params;

        let kind = if has_any_payload || !methods.is_empty() || !generic_params.is_empty() {
            EnumKind::Heterogeneous {
                generic_params: generic_params.to_vec(),
                methods,
            }
        } else {
            EnumKind::Scalar { repr_type: None }
        };

        Ok((kind, items))
    }

    /// Parse scalar enum items for `enum Name u16 { A, B = 2, C }`.
    /// Items only have `= value` or auto-increment, no payload types.
    fn parse_scalar_enum_items(&mut self) -> AutoResult<Vec<EnumItem>> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut items = Vec::new();
        let mut last_val = 0i32;
        while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
            let mut item = EnumItem {
                name: self.cur.text.clone().into(),
                scalar_value: None,
                value_explicit: false,
                payload_type: None,
                payload_types: vec![],
                fields: vec![],
                attrs: vec![],
            };
            self.next();
            if self.is_kind(TokenKind::Asn) {
                self.next(); // skip =
                let value = self.parse_ints()?;
                let value = self.get_int_expr(&value);
                item.scalar_value = Some(value as i32);
                item.value_explicit = true;
                last_val = value as i32 + 1;
            } else {
                item.scalar_value = if last_val != 0 { Some(last_val) } else { None };
                last_val += 1;
            }
            // Handle separator
            if self.is_kind(TokenKind::Comma) {
                self.next();
            } else if self.is_kind(TokenKind::Newline) {
                self.next();
            } else if !self.is_kind(TokenKind::RBrace) {
                self.skip_empty_lines();
            }
            self.skip_empty_lines();
            items.push(item);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(items)
    }

    /// Parse homogeneous enum items for `enum Vertex Point { LeftTop, RightTop }`.
    /// Just variant names, no types or values.
    fn parse_homo_enum_items(&mut self) -> AutoResult<Vec<EnumItem>> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut items = Vec::new();
        while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
            let item_name: AutoStr = self.cur.text.clone().into();
            self.next();
            items.push(EnumItem {
                name: item_name,
                scalar_value: None,
                value_explicit: false,
                payload_type: None,
                payload_types: vec![],
                fields: vec![],
                attrs: vec![],
            });
            self.expect_eos(false)?;
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(items)
    }

    /// Plan 201 Phase 1B: Parse struct-like enum variant fields.
    /// Syntax: { name Type, name Type, ... }
    fn parse_enum_variant_fields(&mut self) -> AutoResult<Vec<crate::ast::EnumField>> {
        use crate::ast::EnumField;
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut fields = Vec::new();
        while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
            // Plan 442 C2: skip `#[...]` attributes on variant fields — musk
            // backend's serde passthrough (#[serde(default, skip_serializing_if
            // = "...")]) previously leaked its tokens into the field parse
            // ("undefined variable: default" + RBrace cascade). The VM drops
            // the attr text; a2r reads it from raw_attrs the same way struct
            // fields do.
            while self.is_kind(TokenKind::Hash) {
                // `#` not followed by `[` → corpus-style line comment (musk
                // relay_store's hand-written `# PLAN-032 ...` notes); skip to
                // end of line instead of erroring.
                let is_attr = if let Ok(tok) = self.lexer.next() {
                    let a = tok.kind == TokenKind::LSquare;
                    self.lexer.push_token(tok);
                    a
                } else {
                    false
                };
                if is_attr {
                    let _ = self.parse_fn_annotations()?;
                } else {
                    while !self.is_kind(TokenKind::Newline)
                        && !self.is_kind(TokenKind::EOF)
                        && !self.is_kind(TokenKind::RBrace)
                    {
                        self.next();
                    }
                }
                self.skip_empty_lines();
            }
            if self.is_kind(TokenKind::RBrace) || self.is_kind(TokenKind::EOF) {
                break;
            }
            let name: AutoStr = self.cur.text.clone().into();
            self.next();
            // Plan 396/B11(b): accept both  and
            // (auto-ai's error.at declares struct-variant fields colon-style).
            if self.is_kind(TokenKind::Colon) {
                self.next();
            }
            let field_type = self.parse_type()?;
            fields.push(EnumField { name, field_type });
            if self.is_kind(TokenKind::Comma) {
                self.next();
            }
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(fields)
    }

    /// Register an enum declaration in the parser's scope and type store.
    fn register_enum_decl(
        &mut self,
        enum_decl: &EnumDecl,
        generic_params: &[crate::ast::GenericParam],
    ) {
        match &enum_decl.kind {
            EnumKind::Scalar { .. } => {
                self.define(enum_decl.name.as_str(), Meta::Enum(enum_decl.clone()));
                self.type_store
                    .write()
                    .unwrap()
                    .register_enum_decl(enum_decl.clone());
                // Register variants as constants (Plan 127: for task message enums)
                for item in &enum_decl.items {
                    let store = Store {
                        kind: StoreKind::Let,
                        attrs: vec![],
                        is_pub: false,
                        name: item.name.clone(),
                        ty: Type::Int,
                        expr: Expr::Int(item.value()),
                    };
                    self.define(item.name.as_str(), Meta::Store(store));
                }
            }
            EnumKind::Heterogeneous { .. } => {
                self.define(
                    enum_decl.name.as_str(),
                    Meta::Type(Type::Enum(shared(enum_decl.clone()))),
                );
                self.type_store
                    .write()
                    .unwrap()
                    .register_enum_decl(enum_decl.clone());
                // Register variants as constructors (Tag-style)
                // Nil variants without payload are accessible as EnumName.VariantName
            }
            EnumKind::Homogeneous { .. } => {
                self.define(
                    enum_decl.name.as_str(),
                    Meta::Type(Type::Enum(shared(enum_decl.clone()))),
                );
                self.type_store
                    .write()
                    .unwrap()
                    .register_enum_decl(enum_decl.clone());
            }
        }
        // Suppress unused variable warning
        let _ = generic_params;
    }

    // ========================================================================
    // Plan 121: Task/Msg System - Task Definition Parsing
    // ========================================================================

    #[allow(dead_code)]
    fn fn_cdecl_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip keyword `c`

        // parse function name
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;

        // Capture the position of the function name for LSP
        // self.prev now points to the name token after expect()
        let name_pos = self.prev.pos;

        // Plan 091: Use InferenceContext for scope management
        self.infer_ctx.push_fn_scope();

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        // Plan 073 Stage A.5: Parse return type annotation -> type
        // Support: fn foo() -> float { ... }
        let mut ret_type = Type::Unknown;
        if self.is_kind(TokenKind::Arrow) {
            // Explicit return type annotation with ->
            self.next(); // consume ->
            ret_type = self.parse_type()?;
        } else if self.is_type_name() {
            // Type without -> (older syntax or inferred)
            ret_type = self.parse_type()?;
        }
        // TODO: determine return type with last stmt if it's not specified

        // exit function scope
        self.exit_fn_scope();

        // no body for c fn decl
        let body = Body::new();
        let fn_expr = Fn::new(
            FnKind::CFunction,
            name.clone(),
            None,
            params,
            body,
            ret_type,
        );
        let fn_stmt = Stmt::Fn(fn_expr.clone());

        // define function in scope
        self.define(name.as_str(), Meta::Fn(fn_expr.clone()));

        // Register symbol location for LSP
        let loc = SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
        // Plan 091: Use wrapper method
        self.define_symbol_location(name.clone(), loc);

        Ok(fn_stmt)
    }

    /// Parse function annotations: [c], [vm], [c,vm], [pub]
    ///
    /// Parse function annotations: #[c], #[vm], #[rs], #[c,vm], #[with(T as Spec)], etc.
    /// Note: `pub` is handled separately as a keyword prefix, not an annotation.
    /// Annotations must start with # prefix (Rust-style).
    /// Returns FnAnnotations struct.
    ///
    /// Plan 306 Phase 3: Collect a parenthesized argument list following an
    /// annotation name. If the current token is `(`, consumes through the matching
    /// `)` and returns the rendered text e.g. `"(0, 100, 1)"` or `("Combat")`;
    /// otherwise returns `None` and consumes nothing.

    fn collect_annotation_args(&mut self) -> Option<String> {
        if !self.is_kind(TokenKind::LParen) {
            return None;
        }
        let mut s = String::from("(");
        let mut depth = 1;
        self.next(); // skip (
        while depth > 0 && !self.is_kind(TokenKind::EOF) {
            if self.is_kind(TokenKind::LParen) {
                depth += 1;
                s.push('(');
            } else if self.is_kind(TokenKind::RParen) {
                depth -= 1;
                if depth > 0 {
                    s.push(')');
                }
            } else if self.is_kind(TokenKind::Comma) {
                s.push_str(", ");
            } else if self.is_kind(TokenKind::Asn) {
                s.push_str(" = ");
            } else if self.is_kind(TokenKind::Str) {
                s.push('"');
                s.push_str(&self.cur.text);
                s.push('"');
            } else {
                s.push_str(&self.cur.text);
            }
            if depth > 0 {
                self.next();
            }
        }
        self.next(); // skip final )
        s.push(')');
        Some(s)
    }

    /// Parse function annotations: [c], [vm], [c,vm], [pub]
    ///
    /// Parse function annotations: #[c], #[vm], #[rs], #[c,vm], #[with(T as Spec)], etc.
    /// Note: `pub` is handled separately as a keyword prefix, not an annotation.
    /// Annotations must start with # prefix (Rust-style).
    /// Returns FnAnnotations struct.
    ///
    /// Plan 061: Added support for #[with(T as Spec)] generic constraints
    /// Plan 083: Added support for #[rs] (Rust transpiler)
    /// Plan 260: Added support for #[test], refactored to struct
    /// Plan 364 W1: does the current `ident` start a dotted annotation path
    /// (`ident . ident`)? Lookahead without consuming (restore via push_token,
    /// same idiom as is_fn_annotation).
    fn is_annotation_dotted_path(&mut self) -> bool {
        if self.cur.kind != TokenKind::Ident {
            return false;
        }
        let mut tokens = Vec::new();
        tokens.push(self.cur.clone()); // first ident
        self.next();
        if self.cur.kind != TokenKind::Dot {
            tokens.push(self.cur.clone()); // non-dot token (already read from stream)
            for t in tokens.into_iter().rev() {
                self.lexer.push_token(t);
            }
            self.next();
            return false;
        }
        tokens.push(self.cur.clone()); // '.'
        self.next();
        if self.cur.kind != TokenKind::Ident {
            tokens.push(self.cur.clone());
            for t in tokens.into_iter().rev() {
                self.lexer.push_token(t);
            }
            self.next();
            return false;
        }
        tokens.push(self.cur.clone()); // segment ident
        for t in tokens.into_iter().rev() {
            self.lexer.push_token(t);
        }
        self.next();
        true
    }

    pub(super) fn parse_fn_annotations(&mut self) -> AutoResult<FnAnnotations> {
        let mut ann = FnAnnotations::default();

        // Parse all annotation blocks: #[...] #[...] ...
        while self.is_kind(TokenKind::Hash) {
            // Plan 442 C2: `#` NOT followed by `[` is a corpus-style line
            // comment (musk relay_store's hand-written `# PLAN-032 ...`
            // notes). Skip to end of line and return default annotations
            // instead of erroring — struct/enum field loops route every
            // field through this fn, so the tolerance lives here.
            let next_is_attr = if let Ok(tok) = self.lexer.next() {
                let a = tok.kind == TokenKind::LSquare;
                self.lexer.push_token(tok);
                a
            } else {
                false
            };
            if !next_is_attr {
                while !self.is_kind(TokenKind::Newline)
                    && !self.is_kind(TokenKind::EOF)
                    && !self.is_kind(TokenKind::RBrace)
                {
                    self.next();
                }
                if self.is_kind(TokenKind::Newline) {
                    self.next();
                }
                // Plan 442 C2 (relay_store tail): keep looping over the whole
                // RUN of `#` lines instead of returning after the first.
                // Body loops (type/ext) fall through to member parsing in the
                // SAME iteration after this returns — a lone `return` left
                // cur on the next `#` line, which type_member then ate as a
                // field name ("Expected end of statement, got Ident<…>" ×20
                // cascade). A single `#` line before a field worked by
                // accident (the fall-through parsed that field), which is why
                // hand-crafted minimal repros never triggered.
                continue;
            }

            self.next(); // skip #

            if self.is_kind(TokenKind::LSquare) {
                self.next(); // skip [

                while self.is_kind(TokenKind::Ident)
                    // Plan 560 T09：`with` 关键字化后 #[with(...)] 注解名
                    // 撞位（§10"零冲突"假设漏了注解名位）——此处同Ident 收。
                    || self.is_kind(TokenKind::With)
                {
                    let annot = self.cur.text.clone();

                    // Plan 364 W1: dotted annotation path — `#[zbus.interface]`.
                    // Any `a.b.c` name is an attribute-macro path: collect it
                    // verbatim (`.` → `::`), attach `(...)` args, and pass
                    // through unconditionally. Single unknown idents still error.
                    if self.is_annotation_dotted_path() {
                        self.next(); // skip first ident
                        let mut path = annot.to_string();
                        while self.is_kind(TokenKind::Dot) {
                            self.next(); // skip '.'
                            let seg = self.cur.text.clone();
                            self.next(); // skip segment
                            path.push_str("::");
                            path.push_str(&seg);
                        }
                        if let Some(args) = self.collect_annotation_args() {
                            path.push_str(&args);
                        }
                        self.impl_attrs.push(path.into());
                        // Check for ] or ,
                        if self.is_kind(TokenKind::RSquare) {
                            self.next(); // skip ]
                            break;
                        }
                        if self.is_kind(TokenKind::Comma) {
                            self.next(); // skip ,
                            continue;
                        }
                        continue;
                    }

                    match annot.as_str() {
                        "c" => ann.has_c = true,
                        "vm" => ann.has_vm = true,
                        "rs" => ann.has_rs = true,
                        "test" => ann.has_test = true,
                        // Plan 306 Phase 3: GDScript variable annotations.
                        // Capture full text (name + optional args) for verbatim @-prefix emission.
                        "export"
                        | "onready"
                        | "export_range"
                        | "export_enum"
                        | "export_group"
                        | "export_subgroup"
                        | "export_flags"
                        | "export_node_path"
                        | "export_file"
                        | "export_dir"
                        | "export_multiline"
                        | "export_color_no_alpha" => {
                            let mut attr_str = annot.to_string();
                            self.next(); // skip the annotation name
                            let args = self.collect_annotation_args();
                            if let Some(a) = &args {
                                attr_str.push_str(a);
                            }
                            ann.store_attrs.push(attr_str.into());
                            // Plan 610 ⑤: fn-level `#[export]` / `#[export(system)]`
                            // — cdylib export face. GDScript var semantics above
                            // are untouched; fn declarations read export_abi
                            // (fn paths never read store_attrs, var paths never
                            // read export_abi).
                            if annot == "export" {
                                ann.export_abi = Some(parse_export_abi_arg(args));
                            }
                            // Check for ] or ,
                            if self.is_kind(TokenKind::RSquare) {
                                self.next(); // skip ]
                                break;
                            }
                            if self.is_kind(TokenKind::Comma) {
                                self.next(); // skip ,
                                continue;
                            }
                            continue;
                        }
                        // Plan 306 Phase 3: GDScript script-level (file-wide) annotations.
                        // Unlike the var-level annotations above, these apply to the whole
                        // script and are collected into the parser (→ Code.file_attrs) for
                        // top-of-file `@tool` / `@icon(...)` emission. They do NOT attach
                        // to the following declaration; that decl is parsed normally below.
                        "tool" | "icon" => {
                            let mut attr_str = annot.to_string();
                            self.next(); // skip the annotation name
                            if let Some(args) = self.collect_annotation_args() {
                                attr_str.push_str(&args);
                            }
                            self.file_attrs.push(attr_str.into());
                            if self.is_kind(TokenKind::RSquare) {
                                self.next(); // skip ]
                                break;
                            }
                            if self.is_kind(TokenKind::Comma) {
                                self.next(); // skip ,
                                continue;
                            }
                            continue;
                        }
                        "single" => {
                            // Plan 121: #[single] annotation for singleton tasks
                            // This is handled by the caller, just skip here
                        }
                        "rust" => {
                            // Plan 555 T02: #[rust] 文件级 pragma——脚本模式
                            // 显式压回通道（.as + #[rust] → Normal）。仅登记
                            // parser.rust_pragma，不改变既有 rust/rs 注解的
                            // 逐声明语义。不自行 skip——循环尾部统一 next()。
                            self.rust_pragma = true;
                        }
                        "script" => {
                            // Plan 550 T10: #[script] 文件级 pragma——脚本模式
                            // 生产者门控标注。仅登记 parser.script_pragma（→
                            // CompileSession.script_marked），不挂到任何声明，
                            // 不改变编译/运行行为（.as 管线归 W1）。
                            // 注：不自行 skip——循环尾部统一 next()（同
                            // single/async 臂约定，只有 with 例外）。
                            self.script_pragma = true;
                        }
                        "with" => {
                            // Plan 061: Parse #[with(T, U as Spec<V>)]
                            self.next(); // skip 'with'
                            ann.with_params = self.parse_with_params()?;
                        }
                        "async" => {
                            // async is inferred from return type ~T (Handle), not annotation
                        }
                        "derive" | "serde" | "tokio" | "allow" | "cfg" => {
                            // Plan 159 Phase 6B-2: Pass-through annotations for Rust transpiler
                            // Collect the raw attribute text: #[derive(Debug, Clone)] -> "derive(Debug, Clone)"
                            let mut attr_str = annot.to_string();
                            self.next(); // skip the annotation name
                            if let Some(args) = self.collect_annotation_args() {
                                attr_str.push_str(&args);
                            }
                            self.raw_attrs.push(attr_str.into());
                            // Check for ] or ,
                            if self.is_kind(TokenKind::RSquare) {
                                self.next(); // skip ]
                                break;
                            }
                            if self.is_kind(TokenKind::Comma) {
                                self.next(); // skip ,
                                continue;
                            }
                            continue;
                        }
                        // Plan 312: #[api(method = "GET", path = "/api/notes/:id")]
                        // Makes the function an HTTP endpoint when run via AutoVM.
                        "api" => {
                            self.next(); // skip 'api'
                            let args = self.collect_annotation_args().unwrap_or_default();
                            // Parse "method = \"GET\", path = \"/api/notes\"" from args
                            let (method, path) = parse_api_args(&args);
                            ann.api_endpoint = Some(crate::ast::ApiAttrs { method, path });
                            // Check for ] or ,
                            if self.is_kind(TokenKind::RSquare) {
                                self.next(); // skip ]
                                break;
                            }
                            if self.is_kind(TokenKind::Comma) {
                                self.next(); // skip ,
                                continue;
                            }
                            continue;
                        }
                        _ => {
                            return Err(SyntaxError::Generic {
                                message: format!("Unknown annotation '{}'. Valid: #[c], #[vm], #[rs], #[single], #[async], #[with(...)], #[c,vm,rs], #[derive(...)], #[serde(...)]. Use 'pub' keyword prefix for visibility.", annot),
                                span: pos_to_span(self.cur.pos),
                            }.into());
                        }
                    }

                    // Skip 'with' since we already consumed it above
                    if annot != "with" {
                        self.next(); // skip the annotation identifier (c, vm, or pub)
                    }

                    if self.is_kind(TokenKind::Comma) {
                        self.next(); // skip ,
                        continue;
                    }

                    if self.is_kind(TokenKind::RSquare) {
                        self.next(); // skip ]
                        break;
                    } else {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Expected ',', ']', or annotation, found {:?}",
                                self.kind()
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }
            } else {
                return Err(SyntaxError::Generic {
                    message: format!("Expected '[' after '#', found {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }

            // Skip any whitespace/newlines between multiple annotation blocks
            self.skip_empty_lines();
        }

        Ok(ann)
    }

    /// Plan 061: Parse with(...) parameter list: (T, U as Spec<V>)
    /// Returns Vec<TypeParam> with constraints populated
    fn parse_with_params(&mut self) -> AutoResult<Vec<crate::ast::TypeParam>> {
        use crate::ast::TypeParam;

        self.expect(TokenKind::LParen)?; // expect '('

        let mut params = Vec::new();

        loop {
            self.skip_empty_lines();

            // Check for ) to end the list
            if self.is_kind(TokenKind::RParen) {
                break;
            }

            // Parse parameter name
            if !self.is_kind(TokenKind::Ident) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected type parameter name in #[with(...)], got {:?}",
                        self.cur.text
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }

            let name = self.parse_name()?;

            // Check for 'as' keyword for constraint
            // Plan 364 W3: `#[with(T as A + B)]` — multiple bounds joined with `+`.
            let mut constraint = Vec::new();
            if self.is_kind(TokenKind::As) {
                self.next(); // skip 'as'
                constraint.push(self.parse_type()?);
                while self.is_kind(TokenKind::Add) {
                    self.next(); // skip '+'
                    constraint.push(self.parse_type()?);
                }
            }

            params.push(TypeParam { name, constraint });

            self.skip_empty_lines();

            // Check for , or )
            if self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                continue;
            } else if self.is_kind(TokenKind::RParen) {
                break;
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected ',' or ')' in #[with(...)], got {:?}",
                        self.cur.text
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        self.expect(TokenKind::RParen)?; // expect ')'

        Ok(params)
    }

    // Function Declaration
    pub fn fn_decl_stmt(&mut self, parent_name: &str) -> AutoResult<Stmt> {
        let declaration_start = self.cur.pos.pos;
        // Check for annotations: #[c], #[vm], #[rs], #[c,vm] BEFORE fn keyword
        let mut ann = if self.is_kind(TokenKind::Hash) {
            self.parse_fn_annotations()?
        } else {
            FnAnnotations::default()
        };

        // Skip empty lines after annotations
        self.skip_empty_lines();

        self.next(); // skip keyword `fn`

        let mut is_vm = ann.has_vm;
        let mut is_c = ann.has_c;
        let _is_rs = ann.has_rs;

        // Backwards compatibility: check for fn.c and fn.vm after fn keyword
        if !is_vm && !is_c && self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
            let sub_kind = self.cur.text.clone();
            if sub_kind == "c" {
                is_c = true;
                self.next(); // skip 'c' keyword
            } else if sub_kind == "vm" {
                is_vm = true;
                self.next(); // skip 'vm' keyword
            }
        }

        // parse function name (Plan 368 FU-1: supports dotted `Type.method` form)
        let (name, dotted_parent) = self.parse_fn_name()?;
        // A dotted top-level decl (`pub fn JsonValue.as_int(...)`) overrides the
        // caller-supplied parent_name — it declares a method on `Type`. For
        // non-dotted names the caller's parent_name (e.g. from `ext Type {}`)
        // still applies.
        let parent_name: &str = if let Some(ref p) = dotted_parent {
            p.as_str()
        } else {
            parent_name
        };

        // Capture the position of the function name for LSP
        // self.prev now points to the name token after parse_name()
        let name_pos = self.prev.pos;

        // Plan 052: Parse generic parameters if present: fn foo<T, N u32>(...)
        let mut generic_params = self.parse_generic_params()?;

        // Plan 061: Merge with_params from #[with(T as Spec)] into generic_params
        // with_params override any params from <T> with the same name
        if !ann.with_params.is_empty() {
            use crate::ast::GenericParam;

            for with_param in ann.with_params {
                // Check if this param already exists in generic_params
                let existing_idx = generic_params.iter().position(|p| match p {
                    GenericParam::Type(tp) => tp.name == with_param.name,
                    GenericParam::Const(cp) => cp.name == with_param.name,
                });

                if let Some(idx) = existing_idx {
                    // Replace existing param with the with_param (which has constraint)
                    generic_params[idx] = GenericParam::Type(with_param.clone());
                } else {
                    // Add new param from with_params
                    generic_params.push(GenericParam::Type(with_param.clone()));
                    // Also add to current_type_params for scope
                    self.current_type_params.push(with_param.name.clone());
                }
            }
        }

        // Plan 091: Use InferenceContext for scope management
        self.infer_ctx.push_fn_scope();

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        //

        // parse return type
        let mut ret_type = Type::Unknown;
        let mut ret_type_name: Option<AutoStr> = None;
        // Plan 073 Stage A.5: Support -> type return type annotation
        if self.is_kind(TokenKind::Arrow) {
            // Explicit return type annotation with ->
            self.next(); // consume ->
            ret_type = self.parse_type()?;
            if self.is_kind(TokenKind::Ident) {
                ret_type_name = Some(self.cur.text.clone());
            }
            self.skip_empty_lines();
        }
        // TODO: determine return type with last stmt if it's not specified
        // Support: Ident (int, str), LSquare ([]int), Star (*int), Question (?T), Not (!T - Plan 121), LParen (tuple)
        else if self.is_kind(TokenKind::Ident)
            || self.is_kind(TokenKind::LSquare)
            || self.is_kind(TokenKind::Star)
            || self.is_kind(TokenKind::Question)
            || self.is_kind(TokenKind::Not)
            || self.is_kind(TokenKind::Tilde)
            || self.is_kind(TokenKind::LParen)
        {
            if self.is_kind(TokenKind::Ident) {
                ret_type_name = Some(self.cur.text.clone());
            }
            ret_type = self.parse_type()?;
            // Skip empty lines after return type (but not for C/VM functions - they need the newlines for EOS)
            if !(is_c || is_vm) {
                self.skip_empty_lines();
            }
            // For C or VM functions, after return type we accept semicolon, newline, or nothing
            // For regular functions, we also accept semicolon for forward declarations
            // For methods in type blocks (parent_name is not empty), we also accept newline
            let allow_newline = is_c || is_vm || !parent_name.is_empty();
            if !allow_newline
                && !self.is_kind(TokenKind::LBrace)
                && !self.is_kind(TokenKind::Semi)
                && !self.is_kind(TokenKind::EOF)
                && !self.is_kind(TokenKind::Hash)
            {
                return Err(SyntaxError::Generic {
                    message: format!("Expected '{{' or ';', found {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        } else if self.is_kind(TokenKind::LBrace) {
            ret_type = Type::Void;
        } else if self.is_kind(TokenKind::Semi) {
            // For functions without a return type and ending with semicolon (forward declaration)
            ret_type = Type::Void;
        } else if is_c || is_vm {
            // For C or VM functions without return type and no semicolon (newline-terminated)
            ret_type = Type::Void;
        }

        // if has parent_name, define `self` in current scope
        if !parent_name.is_empty() {
            let parent_type = self.find_type_for_name(parent_name);
            let ty = parent_type.unwrap_or(Type::Unknown);
            self.define(
                "self",
                Meta::Store(Store {
                    kind: StoreKind::Let,
                    attrs: vec![],
                    is_pub: false,
                    name: "self".into(),
                    ty,
                    expr: Expr::Ident("self".into()),
                }),
            );
        }

        // parse function body
        let body = if !is_vm && !is_c {
            // For regular functions, check if this is a forward declaration (semicolon)
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
                Body::new()
            } else {
                // Plan 028 T14：fn 体内的语句同样不能被方言声明关键字拦截
                //（`msg.blocks = ...` 是局部赋值，非 msg 声明）。
                // PLAN-778 R1（F778-R1-02）：Err 平衡恢复——`body()?` 提前退出
                // 会把 in_fn_body=true 泄漏给恢复点后续语句。
                let prev_fn_body = self.in_fn_body;
                self.in_fn_body = true;
                let body_res = self.body();
                self.in_fn_body = prev_fn_body;
                let b = body_res?;
                b
            }
        } else {
            // For VM and C functions, check if there's a semicolon or function body
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
            } else if self.is_kind(TokenKind::LBrace) {
                // Skip the function body for C/VM functions
                self.skip_block()?;
            }
            Body::new()
        };

        // exit function scope
        self.exit_fn_scope();

        // parent name for method?
        let parent = if parent_name.is_empty() {
            None
        } else {
            Some(parent_name.into())
        };
        let kind = if is_vm {
            FnKind::VmFunction
        } else if is_c {
            FnKind::CFunction
        } else {
            FnKind::Function
        };

        // Create function, preserving return type name if type is Unknown
        let mut fn_expr = if matches!(ret_type, Type::Unknown) {
            if let Some(ret_name) = ret_type_name {
                Fn::with_ret_name(kind, name.clone(), parent, params, body, ret_type, ret_name)
            } else {
                Fn::new(kind, name.clone(), parent, params, body, ret_type)
            }
        } else {
            Fn::new(kind, name.clone(), parent, params, body, ret_type)
        };

        fn_expr.span = Some((
            declaration_start,
            (self.prev.pos.pos + self.prev.pos.len).saturating_sub(declaration_start),
        ));

        // Plan 417-E3: surface `<T>`/`[T]` generic params (with `has` bounds)
        // on the AST node, mirroring fn_decl_stmt_with_annotations (which
        // assigns type_params after the same merge). Without this the params
        // were parsed for scope tracking only and never reached consumers
        // (a2r `<T: Bound>` emission, VM generic-receiver dispatch).
        if !generic_params.is_empty() {
            let mut type_params: Vec<crate::ast::TypeParam> = Vec::new();
            let mut const_params: Vec<crate::ast::ConstParam> = Vec::new();
            for gp in &generic_params {
                match gp {
                    crate::ast::GenericParam::Type(tp) => type_params.push(tp.clone()),
                    crate::ast::GenericParam::Const(cp) => const_params.push(cp.clone()),
                }
            }
            if !type_params.is_empty() {
                fn_expr.type_params = type_params;
            }
            if !const_params.is_empty() {
                fn_expr.const_params = const_params;
            }
        }

        // Plan 163: Set is_pub flag
        fn_expr.is_pub = ann.has_pub;
        // Plan 260: Set is_test flag
        fn_expr.is_test = ann.has_test;
        // Plan 312: Set api_attrs from #[api(method, path)] annotation
        fn_expr.api_attrs = ann.api_endpoint.take();

        // Attach doc comments
        fn_expr.doc = self.take_docs();

        let fn_stmt = Stmt::Fn(fn_expr.clone());
        let unique_name = if parent_name.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", parent_name, name).into()
        };

        // define function in scope
        self.define(unique_name.as_str(), Meta::Fn(fn_expr.clone()));

        // Register symbol location for LSP
        // Use the saved name_pos which is the position of the function name
        let loc = SymbolLocation::new(
            name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
            name_pos.at,
            name_pos.pos,
        );
        // Plan 091: Use wrapper method
        self.define_symbol_location(unique_name.clone(), loc);

        Ok(fn_stmt)
    }

    // Function Declaration with pre-parsed annotations
    pub fn fn_decl_stmt_with_annotations(
        &mut self,
        parent_name: &str,
        has_c: bool,
        has_vm: bool,
        has_rs: bool,
        is_static: bool,
        is_pub: bool,
        with_params: Vec<crate::ast::TypeParam>,
        has_test: bool,
    ) -> AutoResult<Stmt> {
        let declaration_start = self.cur.pos.pos;
        self.next(); // skip keyword `fn`

        let is_c = has_c;
        let is_vm = has_vm;
        let _is_rs = has_rs;

        // parse function name (Plan 368 FU-1: supports dotted `Type.method` form)
        let (name, dotted_parent) = self.parse_fn_name()?;
        // A dotted top-level decl (`pub fn JsonValue.as_int(...)`) overrides the
        // caller-supplied parent_name — it declares a method on `Type`. For
        // non-dotted names the caller's parent_name (e.g. from `ext Type {}`)
        // still applies.
        let parent_name: &str = if let Some(ref p) = dotted_parent {
            p.as_str()
        } else {
            parent_name
        };

        // Capture the position of the function name for LSP
        // self.prev now points to the name token after parse_name()
        let name_pos = self.prev.pos;

        // Plan 052: Parse generic parameters if present: fn foo<T, N u32>(...)
        let generic_params = self.parse_generic_params()?;

        // Plan 061: Merge with_params from #[with(...)] with generic_params from <T>
        // with_params take precedence (can override constraints from <T>)
        let mut type_params: Vec<crate::ast::TypeParam> = Vec::new();

        // First, extract type params from generic_params
        for gp in &generic_params {
            if let crate::ast::GenericParam::Type(tp) = gp {
                type_params.push(tp.clone());
            }
        }

        // Then, merge with with_params — Plan 364 W3: repeated
        // `#[with(T as A, T as B)]` aggregates bounds into the same Vec
        // (`T as A + B`).
        for wp in with_params {
            let existing_idx = type_params.iter().position(|tp| tp.name == wp.name);
            if let Some(idx) = existing_idx {
                let mut merged = type_params[idx].clone();
                merged.constraint.extend(wp.constraint.clone());
                type_params[idx] = merged;
            } else {
                type_params.push(wp);
            }
        }

        // Plan 091: Use InferenceContext for scope management
        self.infer_ctx.push_fn_scope();

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        //

        // parse return type
        let mut ret_type = Type::Unknown;
        let mut ret_type_name: Option<AutoStr> = None;
        // Plan 073 Stage A.5: Support -> type return type annotation
        if self.is_kind(TokenKind::Arrow) {
            // Explicit return type annotation with ->
            self.next(); // consume ->
            ret_type = self.parse_type()?;
            if self.is_kind(TokenKind::Ident) {
                ret_type_name = Some(self.cur.text.clone());
            }
            self.skip_empty_lines();
        }
        // TODO: determine return type with last stmt if it's not specified
        // Support: Ident (int, str), LSquare ([]int), Star (*int), Question (?T), Not (!T - Plan 121), LParen (tuple)
        else if self.is_kind(TokenKind::Ident)
            || self.is_kind(TokenKind::LSquare)
            || self.is_kind(TokenKind::Star)
            || self.is_kind(TokenKind::Question)
            || self.is_kind(TokenKind::Not)
            || self.is_kind(TokenKind::Tilde)
            || self.is_kind(TokenKind::LParen)
        {
            if self.is_kind(TokenKind::Ident) {
                ret_type_name = Some(self.cur.text.clone());
            }
            ret_type = self.parse_type()?;
            // Skip empty lines after return type (but not for C/VM functions - they need the newlines for EOS)
            if !(is_c || is_vm) {
                self.skip_empty_lines();
            }
            // For C or VM functions, after return type we accept semicolon, newline, or nothing
            // For regular functions, we also accept semicolon for forward declarations
            // For methods in type blocks (parent_name is not empty), we also accept newline
            let allow_newline = is_c || is_vm || !parent_name.is_empty();
            if !allow_newline
                && !self.is_kind(TokenKind::LBrace)
                && !self.is_kind(TokenKind::Semi)
                && !self.is_kind(TokenKind::EOF)
                && !self.is_kind(TokenKind::Hash)
            {
                return Err(SyntaxError::Generic {
                    message: format!("Expected '{{' or ';', found {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        } else if self.is_kind(TokenKind::LBrace) {
            ret_type = Type::Void;
        } else if self.is_kind(TokenKind::Semi) {
            // For functions without a return type and ending with semicolon (forward declaration)
            ret_type = Type::Void;
        } else if is_c || is_vm {
            // For C or VM functions without return type and no semicolon (newline-terminated)
            ret_type = Type::Void;
        }

        // if has parent_name, define `self` in current scope
        if !parent_name.is_empty() {
            let parent_type = self.find_type_for_name(parent_name);
            let ty = parent_type.unwrap_or(Type::Unknown);
            self.define(
                "self",
                Meta::Store(Store {
                    kind: StoreKind::Let,
                    attrs: vec![],
                    is_pub: false,
                    name: "self".into(),
                    ty,
                    expr: Expr::Ident("self".into()),
                }),
            );
        }

        // parse function body
        let body = if !is_vm && !is_c {
            // For regular functions, check if this is a forward declaration (semicolon)
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
                Body::new()
            } else {
                self.body()?
            }
        } else {
            // For VM and C functions, check if there's a semicolon or function body
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
            } else if self.is_kind(TokenKind::LBrace) {
                // Skip the function body for C/VM functions
                self.skip_block()?;
            }
            Body::new()
        };

        // exit function scope
        self.exit_fn_scope();

        // parent name for method?
        let parent = if parent_name.is_empty() {
            None
        } else {
            Some(parent_name.into())
        };
        let kind = if is_vm {
            FnKind::VmFunction
        } else if is_c {
            FnKind::CFunction
        } else {
            FnKind::Function
        };

        // Create function, preserving return type name if type is Unknown
        let mut fn_expr = if matches!(ret_type, Type::Unknown) {
            if let Some(ret_name) = ret_type_name {
                Fn::with_ret_name(kind, name.clone(), parent, params, body, ret_type, ret_name)
            } else {
                Fn::new(kind, name.clone(), parent, params, body, ret_type)
            }
        } else {
            Fn::new(kind, name.clone(), parent, params, body, ret_type)
        };

        fn_expr.span = Some((
            declaration_start,
            (self.prev.pos.pos + self.prev.pos.len).saturating_sub(declaration_start),
        ));

        // Plan 061: Set type_params from #[with(...)] and <T>
        fn_expr.type_params = type_params;
        // Plan 052-followup (417 项④): surface const generic params (<N u32>)
        let const_params: Vec<crate::ast::ConstParam> = generic_params
            .iter()
            .filter_map(|gp| {
                if let crate::ast::GenericParam::Const(cp) = gp {
                    Some(cp.clone())
                } else {
                    None
                }
            })
            .collect();
        if !const_params.is_empty() {
            fn_expr.const_params = const_params;
        }

        // Set is_static flag
        fn_expr.is_static = is_static;

        // Plan 163: Set is_pub flag
        fn_expr.is_pub = is_pub;

        // Plan 260: Set is_test flag
        fn_expr.is_test = has_test;

        // Plan 312: Set api_attrs from pending #[api] annotation
        fn_expr.api_attrs = self.pending_api_attrs.take();

        // Plan 610 ⑤: Set export_abi from pending #[export] annotation
        if let Some(abi) = self.pending_export_abi.take() {
            fn_expr.export_abi = Some(abi);
        }

        // Plan 364 W2: fn-level pass-through attrs (`#[tokio.main]`, dotted macros)
        fn_expr.attrs = std::mem::take(&mut self.impl_attrs);

        let fn_stmt = Stmt::Fn(fn_expr.clone());
        let unique_name = if parent_name.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", parent_name, name).into()
        };

        // define function in scope
        self.define(unique_name.as_str(), Meta::Fn(fn_expr.clone()));

        // Register symbol location for LSP
        // Use the saved name_pos which is the position of the function name
        let loc = SymbolLocation::new(
            name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
            name_pos.at,
            name_pos.pos,
        );
        // Plan 091: Use wrapper method
        self.define_symbol_location(unique_name.clone(), loc);

        Ok(fn_stmt)
    }

    pub fn sep_params(&mut self) -> AutoResult<()> {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return Ok(());
        }
        if self.is_kind(TokenKind::RParen) || self.is_kind(TokenKind::VBar) {
            return Ok(());
        }
        let pos = self.cur.pos;
        Err(SyntaxError::UnexpectedToken {
            expected: "parameter separator (comma, newline, ), or |)".to_string(),
            found: format!("{:?}", self.kind()),
            span: pos_to_span(pos),
        }
        .into())
    }

    // parse function parameters
    pub fn fn_params(&mut self) -> AutoResult<Vec<Param>> {
        let mut params = Vec::new();
        self.skip_empty_lines();

        // Plan 088 Phase 3, Plan 122: Support parameter mode keywords
        // Trinity of Resources: view, mut, move (copy deprecated, take deprecated)
        // Loop until we hit a non-parameter (non-ident or non-mode keyword)
        loop {
            // 1. Check for parameter mode keyword (optional, defaults to View)
            let mut mode = ParamMode::default(); // Default: View

            if self.is_kind(TokenKind::View) {
                mode = ParamMode::View;
                self.next(); // skip 'view'
            } else if self.is_kind(TokenKind::Mut) {
                mode = ParamMode::Mut;
                self.next(); // skip 'mut'
            } else if self.is_kind(TokenKind::Move) {
                mode = ParamMode::Move;
                self.next(); // skip 'move'
            } else if self.is_kind(TokenKind::Take) {
                // Plan 122: 'take' is deprecated, use 'move' instead
                let span = pos_to_span(self.cur.pos);
                self.warn(Warning::DeprecatedFeature {
                    name: "take".to_string(),
                    message: "use 'move' instead".to_string(),
                    span,
                });
                mode = ParamMode::Move;
                self.next(); // skip 'take'
            } else if self.is_kind(TokenKind::Copy) {
                // Plan 122: 'copy' is removed from param mode, use 'move' + .clone() at call site
                let span = pos_to_span(self.cur.pos);
                self.add_error(SyntaxError::Generic {
                    message: "'copy' parameter mode is removed. Use 'move' and explicit .clone() at the call site.".to_string(),
                    span,
                }.into());
                mode = ParamMode::Move; // Error recovery: use Move
                self.next(); // skip 'copy'
            }

            // 2. Check for parameter name (required)
            // 2026-08-22(tag 软关键字):软关键字(tag/type/...)在参数名
            // 位置就是普通标识符 —— cur_is_soft_ident 判定。
            if !self.is_kind(TokenKind::Ident) && !self.cur_is_soft_ident() {
                // If no ident after mode keyword, it's an error
                if mode != ParamMode::default() {
                    return Err(SyntaxError::Generic {
                        message: format!("Expected parameter name after '{}'", mode),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
                // No mode keyword and no ident means we're done with parameters
                break;
            }

            // param name (or destructure pattern: TypeName(inner_name))
            // Plan 380 P3: detect `Path(id)` — after parsing the ident name,
            // check if next token is LParen (destructure pattern).
            let mut destructure = None;
            let name = self.cur.text.clone();
            let name_pos = self.cur.pos;
            self.next(); // skip name

            // Check for destructure pattern: TypeName(inner_name)
            if self.is_kind(TokenKind::LParen) {
                self.next(); // skip (
                let inner = self.cur.text.clone();
                self.next(); // skip inner_name
                self.expect(TokenKind::RParen)?;
                destructure = Some(crate::ast::DestructureParam {
                    wrapper_type: name.clone(),
                    inner_name: inner.clone(),
                });
                // Override name to inner_name (function body uses inner_name)
                // but keep original name for scope registration
            }

            // 3. param type (skip ':' if present for type annotation)
            let mut ty = Type::Int;
            if self.is_kind(TokenKind::Colon) {
                self.next(); // skip ':'
            }
            if self.is_type_name() {
                ty = self.parse_type()?;
            }

            // 4. default val
            let mut default = None;
            if self.is_kind(TokenKind::Asn) {
                self.next(); // skip =
                let expr = self.parse_expr()?;
                default = Some(expr);
            }

            // 5. define param in current scope (currently in fn scope)
            // Plan 380 P3: for destructure params, register inner_name in scope
            let scope_name = if let Some(ref d) = destructure {
                d.inner_name.clone()
            } else {
                name.clone()
            };
            let var = Store {
                kind: StoreKind::Var,
                attrs: vec![],
                is_pub: false,
                name: scope_name.clone(),
                expr: default.clone().unwrap_or(Expr::Nil),
                ty: ty.clone(),
            };
            // TODO: should we consider Meta::Param instead of Meta::Var?
            self.define(scope_name.as_str(), Meta::Store(var.clone()));

            // Register symbol location for LSP (use inner_name for destructure)
            let loc = SymbolLocation::new(
                name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
                name_pos.at,
                name_pos.pos,
            );
            // Plan 091: Use wrapper method
            self.define_symbol_location(scope_name.clone(), loc);

            // 6. Plan 088: Create parameter with explicit mode
            // Plan 380 P3: For destructure params, Param.name = inner_name
            // (function body references inner_name; a2r uses destructure field
            // to emit `TypeName(inner_name): Type` in Rust output).
            params.push(Param {
                name: scope_name,
                ty,
                default,
                mode,
                destructure,
            });
            self.sep_params()?;
            self.skip_empty_lines();
        }

        // Handle variadic arguments (...) for C functions
        if self.is_kind(TokenKind::Range) {
            self.next(); // skip .. (Range token)
            if !self.is_kind(TokenKind::Dot) {
                return Err(SyntaxError::Generic {
                    message: "Expected '...' for variadic arguments, found '..'".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            self.next(); // skip final .
                         // Add a variadic marker parameter
                         // Use a special name to indicate variadic
            params.push(Param {
                name: "...".into(),
                ty: Type::Variadic,
                default: None,
                mode: ParamMode::default(),
                destructure: None,
            });
        }

        Ok(params)
    }

    pub fn spec_decl_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `spec` keyword

        let name = self.parse_name()?;

        // Plan 057: Parse generic parameters - e.g., spec Storage<T>, spec Storage<T, N u32>
        let generic_params = self.parse_generic_params()?;

        // Plan 397: Parse optional supertrait bounds — `spec Name<T>: Bound1 + Bound2 { }`.
        // Bounds are opaque identifier strings (Send/Sync/etc. are not Auto types);
        // emitted verbatim by a2r. `:` sits after generics, before `{`.
        let mut bounds: Vec<String> = Vec::new();
        if self.is_kind(TokenKind::Colon) {
            self.next(); // skip ':'
            let b = self.parse_name()?;
            bounds.push(b.to_string());
            while self.is_kind(TokenKind::Add) {
                self.next(); // skip '+'
                let b = self.parse_name()?;
                bounds.push(b.to_string());
            }
        }

        // Plan 057: Populate type parameter scope for use in method signatures
        for param in &generic_params {
            if let GenericParam::Type(tp) = param {
                self.current_type_params.push(tp.name.clone());
            } else if let GenericParam::Const(cp) = param {
                self.current_const_params
                    .insert(cp.name.clone(), cp.typ.clone());
            }
        }

        // Parse spec body
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        let mut methods = Vec::new();
        // Plan 417-E2 (DIV-TRAIT-LANG-1): `type Item` members declare the
        // spec's associated types. Collected in declaration order; each name
        // is pushed into current_type_params so later method signatures
        // resolve `Item` to a bare Type::User reference (the form every
        // downstream consumer substitutes on).
        let mut associated_types: Vec<crate::ast::AssociatedType> = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            if self.is_kind(TokenKind::Fn) {
                let method = self.spec_method()?;
                methods.push(method);
                self.expect_eos(false)?;
            } else if self.is_kind(TokenKind::Type) {
                self.next(); // skip `type`
                let name = self.parse_name()?;
                associated_types.push(crate::ast::AssociatedType {
                    name: name.clone(),
                    bound: None, // bound syntax (`type Item has T`) not accepted yet
                });
                self.current_type_params.push(name);
                self.expect_eos(false)?;
            } else {
                return Err(SyntaxError::Generic {
                    message: "Expected method declaration in spec".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            self.skip_empty_lines();
        }

        self.expect(TokenKind::RBrace)?;

        // Plan 057 / 417-E2: Clear type parameters after parsing spec body
        for param in &generic_params {
            if let GenericParam::Type(_tp) = param {
                self.current_type_params.pop();
            } else if let GenericParam::Const(cp) = param {
                self.current_const_params.remove(&cp.name);
            }
        }
        for _ in &associated_types {
            self.current_type_params.pop();
        }

        // Plan 057: Use SpecDecl::with_generic_params if we have generic params
        let mut spec_decl = if generic_params.is_empty() {
            SpecDecl::new(name, methods)
        } else {
            SpecDecl::with_generic_params(name, generic_params, methods)
        };
        // Plan 397: attach supertrait bounds (parsed above)
        spec_decl.bounds = bounds;
        // Plan 417-E2: attach associated types (parsed above)
        spec_decl.associated_types = associated_types;

        // Register spec in scope
        self.define(spec_decl.name.as_str(), Meta::Spec(spec_decl.clone()));

        // Note: Plan 091 - Universe.register_spec() removed
        // Spec is already registered via define() -> TypeStore.register_spec_decl()

        Ok(Stmt::SpecDecl(spec_decl))
    }

    fn spec_method(&mut self) -> AutoResult<SpecMethod> {
        self.expect(TokenKind::Fn)?;
        let name = self.parse_name()?;

        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        // Parse return type
        let ret = if self.is_type_name() {
            self.parse_type()?
        } else {
            Type::Void // Default to void
        };

        // Parse optional method body (default implementation)
        let body = if self.is_kind(TokenKind::LBrace) {
            Some(Box::new(crate::ast::Expr::Block(self.body()?)))
        } else {
            None // Just signature, no default implementation
        };

        Ok(SpecMethod {
            name,
            params,
            ret,
            body,
        })
    }

    /// Parse a type alias statement: `type List<T> = List<T, DefaultStorage>;`

    pub fn type_decl_stmt(&mut self) -> AutoResult<Stmt> {
        self.type_decl_stmt_with_annotation(false, false, Vec::new())
    }

    pub fn type_decl_stmt_with_annotation(
        &mut self,
        has_c_annotation: bool,
        is_pub: bool,
        with_params: Vec<crate::ast::TypeParam>,
    ) -> AutoResult<Stmt> {
        // TODO: deal with scope
        self.next(); // skip `type` keyword

        // Check for #[c] annotation before the type name (if not already provided)
        let has_c_annotation = if !has_c_annotation && self.is_kind(TokenKind::Hash) {
            self.parse_fn_annotations()?.has_c
        } else {
            has_c_annotation
        };

        // Skip empty lines after annotation
        self.skip_empty_lines();

        if self.is_kind(TokenKind::Dot) {
            self.next();
            let sub_kind = self.parse_name()?;
            if sub_kind == "c" {
                let name = self.parse_name()?;

                // Capture the position of the type name for LSP
                let name_pos = self.prev.pos;

                let decl = TypeDecl {
                    consts: Vec::new(),
                    kind: TypeDeclKind::CType,
                    name: name.clone(),
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
                };
                // put type in scope
                self.define(name.as_str(), Meta::Type(Type::CStruct(decl.clone())));

                // Register symbol location for LSP
                let loc =
                    SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
                // Plan 091: Use wrapper method
                self.define_symbol_location(name.clone(), loc);

                return Ok(Stmt::TypeDecl(decl));
            }
        }

        // If we have [c] annotation, treat as C type
        let kind = if has_c_annotation {
            TypeDeclKind::CType
        } else {
            TypeDeclKind::UserType
        };

        let name = self.parse_name()?;

        // Parse generic parameters (optional) - e.g., type List<T> { ... }, type Inline<T, const N u32> { ... }
        let mut generic_params = Vec::new();
        if self.cur.kind == TokenKind::Lt {
            self.next(); // Consume '<'

            generic_params.push(self.parse_generic_param()?);

            while self.cur.kind == TokenKind::Comma {
                self.next(); // Consume ','
                generic_params.push(self.parse_generic_param()?);
            }

            self.expect(TokenKind::Gt)?; // Consume '>'
        }

        // Check if this is a type alias (has `=` after name and params)
        if self.cur.kind == TokenKind::Asn {
            // This is a type alias: type List<T> = List<T, DefaultStorage>;
            self.next(); // Consume '='

            // Plan 317: Record type alias: `type Note = { id: int; name: str }`
            // The `=` is followed by `{` (record body), not a type name. Parse
            // the fields and create a UserType (same as `type Note { ... }`).
            if self.cur.kind == TokenKind::LBrace {
                // Fall through to the normal type body parsing below (don't
                // consume the `{` — the body parser expects it).
            } else {
                let target = self.parse_type()?;
                self.expect(TokenKind::Semi)?;

                // Extract just the names from GenericParam for TypeAlias
                let params: Vec<Name> = generic_params
                    .into_iter()
                    .filter_map(|p| match p {
                        GenericParam::Type(tp) => Some(tp.name),
                        GenericParam::Const(_) => None, // Const params not supported in type aliases
                    })
                    .collect();

                // Plan 091: Store type alias in Database (removed Universe dependency)
                if let Some(ref db) = self.db {
                    if let Ok(mut db) = db.write() {
                        db.insert_type_alias(name.clone(), (params.clone(), target.clone()));
                    }
                }

                return Ok(Stmt::TypeAlias(TypeAlias {
                    name,
                    params,
                    target,
                }));
            } // close else (non-record type alias)
        }

        // Plan 052: Populate current_const_params map with const generic parameters
        // and current_type_params with type generic parameters
        // This allows parameters to be used in method signatures within the type body
        for param in &generic_params {
            if let crate::ast::GenericParam::Type(tp) = param {
                self.current_type_params.push(tp.name.clone());
            } else if let crate::ast::GenericParam::Const(cp) = param {
                self.current_const_params
                    .insert(cp.name.clone(), cp.typ.clone());
            }
        }

        // Capture the position of the type name for LSP
        // self.prev now points to the name token after parse_name()
        let name_pos = self.prev.pos;

        // Plan 364 W3: merge #[with(T as A)] constraints into the type's
        // generic params (repeated #[with(T as A, T as B)] aggregates).
        let mut generic_params = generic_params;
        for wp in with_params {
            let existing_idx = generic_params.iter().position(|gp| match gp {
                crate::ast::GenericParam::Type(tp) => tp.name == wp.name,
                crate::ast::GenericParam::Const(_) => false,
            });
            if let Some(idx) = existing_idx {
                if let crate::ast::GenericParam::Type(tp) = &mut generic_params[idx] {
                    tp.constraint.extend(wp.constraint.clone());
                }
            } else {
                generic_params.push(crate::ast::GenericParam::Type(wp));
            }
        }

        let mut decl = TypeDecl {
            consts: Vec::new(),
            kind: kind.clone(),
            name: name.clone(),
            parent: None,
            specs: Vec::new(),
            spec_impls: Vec::new(), // Plan 057
            has: Vec::new(),
            generic_params,
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
            attrs: std::mem::take(&mut self.raw_attrs), // Plan 159 Phase 6B-2: collect derive/serde attrs
            impl_attrs: std::mem::take(&mut self.impl_attrs), // Plan 364 W1: dotted macro attrs → impl 块
            doc: None,
            is_pub: false, // Plan 163: default private
        };
        // Plan 163: Override is_pub from annotation
        if is_pub {
            decl.is_pub = true;
        }

        // Attach doc comments
        decl.doc = self.take_docs();
        // println!(
        //     "Defining type {} in scope {}",
        //     name,
        //     self.scope.borrow().cur_spot
        // );

        // put type in scope
        self.define(name.as_str(), Meta::Type(Type::User(decl.clone())));

        // Plan 087: Also register to type_registry for REPL support
        if let Some(ref registry) = self.type_registry {
            registry
                .borrow_mut()
                .register_type(name.to_string(), Type::User(decl.clone()));
        }

        // Register symbol location for LSP
        // Use the saved name_pos which is the position of the type name
        let loc = SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
        // Plan 091: Use wrapper method
        self.define_symbol_location(name.clone(), loc);

        // For C types, there's no body - they're opaque types
        if kind == TypeDeclKind::CType {
            // C types are opaque, no body needed
            // Type already registered above at line 3356
            return Ok(Stmt::TypeDecl(decl));
        }

        // Plan 368 FU-1: forward/opaque type declaration `type X` (no body).
        // stdlib `.at` files declare types this way (json.at `type JsonValue`,
        // http.at `type Server`, ...). The name is already registered above, so
        // `self JsonValue` params and `JsonValue?` returns resolve. Return an
        // empty TypeDecl (same shape the CType path above relies on). Only fire
        // when the next token is an end-of-statement/block token — NOT `is`,
        // `as`, `has`, or `{` (those continue to the body-parse path below).
        if matches!(
            self.cur.kind,
            TokenKind::Newline | TokenKind::Semi | TokenKind::EOF | TokenKind::RBrace
        ) {
            if self.is_kind(TokenKind::Semi) {
                self.next(); // consume optional trailing ';'
            }
            return Ok(Stmt::TypeDecl(decl));
        }

        // deal with `is` keyword (single inheritance)
        let mut parent = None;
        if self.is_kind(TokenKind::Is) {
            self.next(); // skip `is` keyword
            let parent_name = self.parse_name()?;
            // Lookup parent type
            if let Some(meta) = self.lookup_meta(parent_name.as_str()) {
                if let Meta::Type(Type::User(parent_decl)) = meta.as_ref() {
                    parent = Some(Box::new(Type::User(parent_decl.clone())));
                } else {
                    return Err(SyntaxError::Generic {
                        message: format!("'{}' is not a user type", parent_name),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            } else {
                return Err(SyntaxError::Generic {
                    message: format!("Parent type '{}' not found", parent_name),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }
        decl.parent = parent;

        // Plan 057: deal with `as` keyword - parse spec implementations with optional type arguments
        let mut specs = Vec::new(); // Backwards compatibility: names only
        let mut spec_impls = Vec::new(); // Plan 057: Generic spec implementations
        if self.is_kind(TokenKind::As) {
            self.next(); // skip `as` keyword
                         // Parse one or more spec names with optional type arguments
            while !self.is_kind(TokenKind::LBrace) && !self.is_kind(TokenKind::Has) {
                if !specs.is_empty() {
                    self.expect(TokenKind::Comma)?;
                }
                let spec_name = self.parse_name()?;

                // Plan 057: Check for type arguments: as Storage<T>
                // Plan 417-E2: `Item=int` inside the brackets is a NAMED
                // associated-type binding (collected separately); a bare type
                // is a positional argument binding the spec's generic params.
                let mut assoc_bindings: Vec<(crate::ast::Name, Type)> = Vec::new();
                let type_args = if self.is_kind(TokenKind::Lt) {
                    self.next(); // skip `<`
                    let mut args = Vec::new();
                    loop {
                        self.skip_empty_lines();
                        let ty = self.parse_type()?;
                        self.skip_empty_lines();

                        if self.is_kind(TokenKind::Asn) {
                            // Named binding: `Item=int`. The left side must be
                            // a bare name (assoc types parse as Type::User
                            // with an empty decl).
                            let assoc_name = match &ty {
                                Type::User(u)
                                    if u.members.is_empty()
                                        && u.methods.is_empty()
                                        && u.generic_params.is_empty() =>
                                {
                                    u.name.clone()
                                }
                                _ => {
                                    return Err(SyntaxError::Generic {
                                        message: format!(
                                            "Expected associated type name before '=' in spec binding for '{}', got {}",
                                            spec_name,
                                            ty.unique_name()
                                        ),
                                        span: pos_to_span(self.cur.pos),
                                    }
                                    .into());
                                }
                            };
                            self.next(); // skip '='
                            self.skip_empty_lines();
                            let bound_ty = self.parse_type()?;
                            assoc_bindings.push((assoc_name, bound_ty));
                            self.skip_empty_lines();
                        } else {
                            args.push(ty);
                        }

                        if self.is_kind(TokenKind::Gt) {
                            self.next(); // skip `>`
                            break;
                        } else if self.is_kind(TokenKind::Comma) {
                            self.next(); // skip `,`
                            continue;
                        } else {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "Expected '>' or ',' in type argument list, got {}",
                                    self.cur.text
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    }
                    args
                } else {
                    Vec::new() // No type arguments
                };

                // Add to backwards-compatible specs list
                specs.push(spec_name.clone());

                // Plan 057/417-E2: record the impl if it carries positional
                // type arguments OR named associated-type bindings
                if !type_args.is_empty() || !assoc_bindings.is_empty() {
                    spec_impls.push(crate::ast::SpecImpl {
                        spec_name,
                        type_args,
                        assoc_bindings,
                    });
                }
            }
        }
        decl.specs = specs;
        decl.spec_impls = spec_impls; // Plan 057

        // deal with `has` keyword
        let mut has = Vec::new();
        if self.is_kind(TokenKind::Has) {
            self.next(); // skip `has` keyword
            while !self.is_kind(TokenKind::LBrace) {
                if !has.is_empty() {
                    self.expect(TokenKind::Colon)?; // skip ,
                }
                let typ = self.parse_type()?;
                has.push(typ);
            }
        }
        decl.has = has;

        // type body (for user types)
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        // list of members, methods, or delegations
        let mut members = Vec::new();
        let mut methods = Vec::new();
        let mut delegations = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            // Check for annotations: #[c], #[vm], #[rs], #[c,vm] before function declarations
            // Note: pub is a keyword prefix, not an annotation (pub fn, pub static fn)
            let ann = self.parse_fn_annotations()?;

            self.skip_empty_lines(); // Skip newlines after annotations

            // Check if this annotation should be skipped for current compile destination
            let should_skip = match self.compile_dest {
                CompileDest::TransC if ann.has_vm && !ann.has_c => true, // Skip #[vm] in C transpiler
                CompileDest::TransC if ann.has_rs && !ann.has_c => true, // Skip #[rs] in C transpiler
                CompileDest::TransRust if ann.has_vm && !ann.has_rs => true, // Skip #[vm] in Rust transpiler
                CompileDest::TransRust if ann.has_c && !ann.has_rs => true, // Skip #[c] in Rust transpiler
                CompileDest::Interp if ann.has_c && !ann.has_vm => true, // Skip #[c] in interpreter
                CompileDest::Interp if ann.has_rs && !ann.has_vm => true, // Skip #[rs] in interpreter
                _ => false,
            };

            if should_skip {
                // Skip the entire function declaration
                if self.is_kind(TokenKind::Static)
                    || self.is_kind(TokenKind::Mut)
                    || self.is_kind(TokenKind::Fn)
                {
                    let is_static = self.is_kind(TokenKind::Static);
                    let is_mut = self.is_kind(TokenKind::Mut);
                    if is_static {
                        self.next(); // skip static
                    }
                    if is_mut {
                        self.next(); // skip mut
                    }
                    // Parse with actual flags to correctly handle the function syntax
                    let _ = self.fn_decl_stmt_with_annotations(
                        &name,
                        ann.has_c,
                        ann.has_vm,
                        ann.has_rs,
                        is_static,
                        ann.has_pub,
                        ann.with_params.clone(),
                        ann.has_test,
                    );
                }
                self.expect_eos(false)?;
                continue;
            }

            // Plan 6B-4.19: Handle `pub` keyword prefix inside type body
            let local_has_pub =
                if self.cur.text.as_str() == "pub" && self.cur.kind == TokenKind::Ident {
                    self.next(); // consume "pub"
                    true
                } else {
                    ann.has_pub
                };

            // Check for static fn, mut fn, or fn
            if self.is_kind(TokenKind::Static)
                || self.is_kind(TokenKind::Mut)
                || self.is_kind(TokenKind::Fn)
            {
                let is_static = self.is_kind(TokenKind::Static);
                let is_mut = self.is_kind(TokenKind::Mut);
                if is_static {
                    self.next(); // skip static keyword
                }
                if is_mut {
                    self.next(); // skip mut keyword
                }

                // Now expect fn keyword
                if !self.is_kind(TokenKind::Fn) {
                    return Err(SyntaxError::Generic {
                        message: format!("expected 'fn' after 'static', found {:?}", self.kind()),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }

                let fn_stmt = self.fn_decl_stmt_with_annotations(
                    &name,
                    ann.has_c,
                    ann.has_vm,
                    ann.has_rs,
                    is_static,
                    local_has_pub,
                    ann.with_params.clone(),
                    ann.has_test,
                )?;
                if let Stmt::Fn(mut fn_expr) = fn_stmt {
                    // Plan 163: Set is_mut flag for mutable methods
                    if is_mut {
                        fn_expr.is_mut = true;
                    }
                    methods.push(fn_expr);
                }
            } else if self.is_kind(TokenKind::Has) {
                // Parse member-level delegation: `has member Type for Spec`
                let delegation = self.parse_delegation()?;
                delegations.push(delegation);
            } else {
                let member = self.type_member(&name)?;
                members.push(member);
            }
            self.expect_eos(false)?; // Not first statement in type body
        }
        self.expect(TokenKind::RBrace)?;

        // add members and methods from parent type (inheritance)
        if let Some(ref parent_type) = decl.parent {
            match parent_type.as_ref() {
                Type::User(parent_decl) => {
                    // Inherit members from parent
                    for m in parent_decl.members.iter() {
                        members.push(m.clone());
                    }
                    // Inherit methods from parent
                    for meth in parent_decl.methods.iter() {
                        // change meth's parent to self
                        let mut inherited_meth = meth.clone();
                        inherited_meth.parent = Some(name.clone());
                        // register this method as Self.method
                        let unique_name = format!("{}.{}", &name, &inherited_meth.name);
                        self.define(unique_name.as_str(), Meta::Fn(inherited_meth.clone()));
                        methods.push(inherited_meth);
                    }
                }
                _ => {
                    // System types cannot be inherited
                }
            }
        }

        // add members and methods of compose types
        for comp in decl.has.iter() {
            match comp {
                Type::User(decl) => {
                    for m in decl.members.iter() {
                        members.push(m.clone());
                    }
                    for meth in decl.methods.iter() {
                        // change meth's parent to self
                        let mut compose_meth = meth.clone();
                        compose_meth.parent = Some(name.clone());
                        // register this method as Self::method
                        let unique_name = format!("{}::{}", &name, &compose_meth.name);
                        self.define(unique_name.as_str(), Meta::Fn(compose_meth.clone()));
                        methods.push(compose_meth);
                    }
                }
                _ => {
                    // System Types not supported for Compose yet
                }
            }
        }
        decl.members = members;
        decl.methods = methods;
        decl.delegations = delegations;

        // Check trait conformance if type declares specs (Plan 057: defer check for ext blocks)
        // Note: If methods are empty, skip conformance check as methods may be added later via ext blocks
        if !decl.specs.is_empty() && !decl.methods.is_empty() {
            for spec_name in &decl.specs {
                if let Some(meta) = self.lookup_meta(spec_name.as_str()) {
                    if let Meta::Spec(spec_decl) = meta.as_ref() {
                        // Use TraitChecker to verify conformance
                        if let Err(errors) =
                            crate::trait_checker::TraitChecker::check_conformance(&decl, spec_decl)
                        {
                            // Add all conformance errors to parser errors
                            for error in errors {
                                self.add_error(error);
                            }
                        }
                    }
                } else {
                    // Spec not found - this is an error
                    self.add_error(
                        SyntaxError::Generic {
                            message: format!(
                                "Type '{}' declares spec '{}' but spec is not defined",
                                name, spec_name
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into(),
                    );
                }
            }
        }

        // Plan 057: Check generic spec implementations (spec_impls)
        // Same logic: skip check if methods are empty (ext block case)
        if !decl.spec_impls.is_empty() && !decl.methods.is_empty() {
            for spec_impl in &decl.spec_impls {
                if let Some(meta) = self.lookup_meta(spec_impl.spec_name.as_str()) {
                    if let Meta::Spec(spec_decl) = meta.as_ref() {
                        // Validate type argument count
                        if spec_impl.type_args.len() != spec_decl.generic_params.len() {
                            self.add_error(
                                SyntaxError::Generic {
                                    message: format!(
                                        "Type '{}' implements spec '{}' with {} type argument(s) but spec expects {}",
                                        name,
                                        spec_impl.spec_name,
                                        spec_impl.type_args.len(),
                                        spec_decl.generic_params.len()
                                    ),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into(),
                            );
                        } else {
                            // Use TraitChecker to verify conformance
                            if let Err(errors) =
                                crate::trait_checker::TraitChecker::check_conformance(
                                    &decl, spec_decl,
                                )
                            {
                                for error in errors {
                                    self.add_error(error);
                                }
                            }
                        }
                    }
                } else {
                    // Spec not found - this is an error
                    self.add_error(
                        SyntaxError::Generic {
                            message: format!(
                                "Type '{}' declares generic spec '{}' but spec is not defined",
                                name, spec_impl.spec_name
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into(),
                    );
                }
            }
        }

        self.define(name.as_str(), Meta::Type(Type::User(decl.clone())));
        Ok(Stmt::TypeDecl(decl))
    }

    pub fn type_member(&mut self, type_name: &str) -> AutoResult<Member> {
        // Capture the position of the field name for LSP
        let name = self.parse_name()?;
        let name_pos = self.prev.pos;

        // Plan 317: Support `name: Type` (colon-separated) in addition to
        // `name Type` (space-separated). Record type aliases use colons.
        if self.is_kind(TokenKind::Colon) {
            self.next(); // consume ':'
        }

        let ty = self.parse_type()?;
        let mut value = None;
        if self.is_kind(TokenKind::Asn) {
            self.next(); // skip =
            let expr = self.parse_expr()?;
            value = Some(expr);
        }
        let store = Store {
            name: name.clone(),
            ty: ty.clone(),
            expr: match &value {
                Some(expr) => expr.clone(),
                None => Expr::Nil,
            },
            kind: StoreKind::Field,
            attrs: vec![],
            is_pub: false,
        };
        self.define(name.as_str(), Meta::Store(store));

        // Register field location for LSP with qualified name "TypeName.fieldName"
        let qualified_name = format!("{}.{}", type_name, name);
        let loc = SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
        // Plan 091: Use wrapper method
        self.define_symbol_location(qualified_name.clone().into(), loc);

        // Plan 163: Include per-field attributes from raw_attrs
        let mut member = Member::new(name, ty, value);
        if !self.raw_attrs.is_empty() {
            member.attrs = std::mem::take(&mut self.raw_attrs);
        }
        Ok(member)
    }

    /// Parse member-level delegation: `has member Type for Spec`
    /// Example: `has core WarpDrive for Engine`
    fn parse_delegation(&mut self) -> AutoResult<crate::ast::Delegation> {
        self.expect(TokenKind::Has)?; // skip `has` keyword

        // Parse member name
        let member_name = self.parse_name()?;

        // Parse member type
        let member_type = self.parse_type()?;

        // Expect `for` keyword
        self.expect(TokenKind::For)?;

        // Parse spec name
        let spec_name = self.parse_name()?;

        Ok(crate::ast::Delegation {
            member_name,
            member_type,
            spec_name,
        })
    }

    pub fn union_stmt(&mut self) -> AutoResult<Stmt> {
        self.expect(TokenKind::Union)?;
        let name = self.parse_name()?;
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut fields = Vec::new();
        let mut field_index = 0;
        while !self.is_kind(TokenKind::RBrace) {
            let f = self.union_field()?;
            fields.push(f);
            let is_first = field_index == 0;
            self.expect_eos(is_first)?;
            field_index += 1;
        }
        self.expect(TokenKind::RBrace)?;

        let union = Union {
            name: name.clone(),
            fields: fields.clone(),
        };

        self.define(name.as_str(), Meta::Type(Type::Union(union)));
        Ok(Stmt::Union(Union { name, fields }))
    }

    pub fn union_field(&mut self) -> AutoResult<UnionField> {
        let name = self.parse_name()?;
        let ty = self.parse_type()?;
        Ok(UnionField { name, ty })
    }
}
