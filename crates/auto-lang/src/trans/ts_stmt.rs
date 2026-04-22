use crate::ast::*;
use crate::AutoResult;
use std::io::Write;
use super::{TypeScriptTrans, ToStrError};

impl TypeScriptTrans {
    pub fn stmt(&mut self, stmt: &Stmt, out: &mut impl Write) -> AutoResult<()> {
        match stmt {
            // Expression statements
            Stmt::Expr(expr) => {
                self.expr(expr, out)?; // Uses ts_expr
                out.write(b";")?;
                Ok(())
            }

            // Store (variable assignment) - with type annotation
            Stmt::Store(store) => {
                // TypeScript: const for let (immutable), let for var (mutable)
                match store.kind {
                    StoreKind::Let => out.write(b"const ").to()?,
                    StoreKind::Var => out.write(b"let ").to()?,
                    StoreKind::Const => out.write(b"const ").to()?,
                    _ => {} // Field, CVar, Shared don't need declaration prefix
                };
                out.write_all(store.name.as_bytes())?;

                // Add type annotation if type is known
                if !matches!(store.ty, Type::Unknown) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&store.ty).as_bytes())?; // Uses ts_types
                }

                out.write(b" = ")?;
                self.expr(&store.expr, out)?;
                out.write(b";")?;
                Ok(())
            }

            // Function declarations - with type annotations
            Stmt::Fn(func) => {
                self.fn_decl(func, out)?;
                Ok(())
            }

            // If statements
            Stmt::If(if_stmt) => {
                self.if_stmt(if_stmt, out)?;
                Ok(())
            }

            // For loops
            Stmt::For(for_loop) => {
                self.for_loop(for_loop, out)?;
                Ok(())
            }

            // Break statements
            Stmt::Break => {
                out.write(b"break;")?;
                Ok(())
            }

            // Return statement
            Stmt::Return(expr) => {
                out.write(b"return ")?;
                self.expr(expr, out)?;
                out.write(b";")?;
                Ok(())
            }

            // Node statements (e.g. loop wrap)
            Stmt::Node(node) => {
                self.expr(&Expr::Node(node.clone()), out)?;
                if node.name != "loop" {
                    out.write(b";")?;
                }
                Ok(())
            }

            // Pattern matching (is)
            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, out)?;
                Ok(())
            }

            // Empty lines
            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    out.write(b"\n")?;
                }
                Ok(())
            }

            // Type declarations → interface
            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, out)?;
                Ok(())
            }

            // Enum declarations → const enum
            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, out)?;
                Ok(())
            }

            // Type Aliases
            Stmt::TypeAlias(type_alias) => {
                self.type_alias(type_alias, out)?;
                Ok(())
            }

            // Union declarations
            Stmt::Union(union) => {
                self.union_decl(union, out)?;
                Ok(())
            }

            // Tag declarations
            Stmt::Tag(tag) => {
                self.tag_decl(tag, out)?;
                Ok(())
            }

            // Type extensions
            Stmt::Ext(ext) => {
                self.ext_decl(ext, out)?;
                Ok(())
            }

            // Spec (interface) declarations
            Stmt::SpecDecl(spec_decl) => {
                self.spec_decl(spec_decl, out)?;
                Ok(())
            }

            // Continue
            Stmt::Continue => {
                out.write(b"continue;")?;
                Ok(())
            }

            // Comments
            Stmt::Comment(comment) => {
                out.write(b"// ")?;
                out.write_all(comment.as_bytes())?;
                Ok(())
            }

            // Use statements → import
            Stmt::Use(use_stmt) => {
                self.use_stmt(use_stmt, out)?;
                Ok(())
            }

            // Unsupported statements
            _ => Err(format!("TypeScript Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    pub fn fn_decl(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
        // Detect async: functions returning ~T (Handle/Future) are async in TypeScript
        let is_async_fn = Self::has_await_expr(&func.body.stmts)
            || matches!(func.ret, Type::Handle { .. })
            || matches!(&func.ret, Type::GenericInstance(inst) if inst.base_name == "Future");

        if is_async_fn {
            out.write(b"async ")?;
        }
        out.write(b"function ")?;
        out.write_all(func.name.as_bytes())?;
        out.write(b"(")?;

        // Parameters with type annotations
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;

            // Add type annotation
            if !matches!(param.ty, Type::Unknown) {
                out.write(b": ")?;
                out.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
            }
        }

        out.write(b")")?;

        // Return type annotation
        if !matches!(func.ret, Type::Unknown | Type::Void) {
            out.write(b": ")?;
            out.write_all(Self::type_to_ts(&func.ret).as_bytes())?;
        } else if matches!(func.ret, Type::Void) {
            out.write(b": void")?;
        }

        // Function body
        self.body(&func.body, out)?;

        Ok(())
    }

    pub fn body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        out.write(b" {")?;
        for stmt in &body.stmts {
            out.write(b"\n    ")?;
            self.stmt(stmt, out)?;
        }
        out.write(b"\n}")?;
        Ok(())
    }

    pub fn if_stmt(&mut self, if_stmt: &If, out: &mut impl Write) -> AutoResult<()> {
        // Process first branch as "if"
        if let Some(first_branch) = if_stmt.branches.first() {
            out.write(b"if (")?;
            self.expr(&first_branch.cond, out)?;
            out.write(b")")?;
            self.if_body(&first_branch.body, out)?;
        }

        // Process remaining branches as "else if"
        for branch in if_stmt.branches.iter().skip(1) {
            out.write(b" else if (")?;
            self.expr(&branch.cond, out)?;
            out.write(b")")?;
            self.if_body(&branch.body, out)?;
        }

        // Process else if present
        if let Some(else_) = &if_stmt.else_ {
            out.write(b" else")?;
            self.if_body(else_, out)?;
        }

        Ok(())
    }

    pub fn for_loop(&mut self, for_loop: &For, out: &mut impl Write) -> AutoResult<()> {
        match &for_loop.iter {
            Iter::Cond => {
                out.write(b"while (")?;
                self.expr(&for_loop.range, out)?;
                out.write(b")")?;
                self.if_body(&for_loop.body, out)?;
            }
            Iter::Ever => {
                out.write(b"while (true)")?;
                self.if_body(&for_loop.body, out)?;
            }
            Iter::Named(name) => {
                // If the range is an Expr::Range, we can generate a traditional for loop
                if let Expr::Range(range) = &for_loop.range {
                    out.write(b"for (let ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b" = ")?;
                    self.expr(&range.start, out)?;
                    out.write(b"; ")?;
                    out.write_all(name.as_bytes())?;
                    if range.eq {
                        out.write(b" <= ")?;
                    } else {
                        out.write(b" < ")?;
                    }
                    self.expr(&range.end, out)?;
                    out.write(b"; ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b"++)")?;
                    self.if_body(&for_loop.body, out)?;
                } else {
                    // For-each over array: for x in arr -> for (const x of arr)
                    out.write(b"for (const ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b" of ")?;
                    self.expr(&for_loop.range, out)?;
                    out.write(b")")?;
                    self.if_body(&for_loop.body, out)?;
                }
            }
            Iter::Indexed(index, name) => {
                // For-each with index over array: for i, x in arr -> for (let i = 0; i < arr.length; i++) { const x = arr[i]; }
                if let Expr::Range(range) = &for_loop.range {
                    // indexed range iteration
                    out.write(b"for (let ")?;
                    out.write_all(index.as_bytes())?;
                    out.write(b" = 0, ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b" = ")?;
                    self.expr(&range.start, out)?;
                    out.write(b"; ")?;
                    out.write_all(name.as_bytes())?;
                    if range.eq { out.write(b" <= ")?; } else { out.write(b" < ")?; }
                    self.expr(&range.end, out)?;
                    out.write(b"; ")?;
                    out.write_all(index.as_bytes())?;
                    out.write(b"++, ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b"++)")?;
                    self.if_body(&for_loop.body, out)?;
                } else {
                    // We need a unique inner variable, or just a block
                    out.write(b"for (let ")?;
                    out.write_all(index.as_bytes())?;
                    out.write(b" = 0; ")?;
                    out.write_all(index.as_bytes())?;
                    out.write(b" < ")?;
                    self.expr(&for_loop.range, out)?;
                    out.write(b".length; ")?;
                    out.write_all(index.as_bytes())?;
                    out.write(b"++) {\n        const ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b" = ")?;
                    self.expr(&for_loop.range, out)?;
                    out.write(b"[")?;
                    out.write_all(index.as_bytes())?;
                    out.write(b"];")?;

                    for stmt in &for_loop.body.stmts {
                        out.write(b"\n        ")?;
                        self.stmt(stmt, out)?;
                    }

                    out.write(b"\n    }")?;
                }
            }
            _ => {
                return Err(format!("TypeScript Transpiler: unsupported for loop iteration: {:?}", for_loop.iter).into());
            }
        }
        Ok(())
    }

    pub fn is_stmt(&mut self, is_stmt: &Is, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"switch (")?;
        self.expr(&is_stmt.target, out)?;
        out.write(b") {")?;

        for branch in &is_stmt.branches {
            match branch {
                IsBranch::EqBranch(patterns, body) => {
                    for pat in patterns.iter() {
                        out.write(b"\n        case ")?;
                        self.expr(pat, out)?;
                        out.write(b":")?;
                    }
                    self.switch_case_body(body, out)?;
                    out.write(b"\n            break;")?;
                }
                IsBranch::IfBranch(expr, body) => {
                    out.write(b"\n        case ")?;
                    self.expr(expr, out)?;
                    out.write(b":")?;
                    self.switch_case_body(body, out)?;
                    out.write(b"\n            break;")?;
                }
                IsBranch::ElseBranch(body) => {
                    out.write(b"\n        default:")?;
                    self.switch_case_body(body, out)?;
                }
            }
        }

        out.write(b"\n    }")?;
        Ok(())
    }

    pub fn switch_case_body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        for stmt in &body.stmts {
            out.write(b"\n            ")?;
            self.stmt(stmt, out)?;
        }
        Ok(())
    }

    pub fn if_body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        if body.stmts.is_empty() {
            out.write(b" {}")?;
        } else if body.stmts.len() == 1 {
            out.write(b" {\n        ")?;
            self.stmt(&body.stmts[0], out)?;
            out.write(b"\n    }")?;
        } else {
            out.write(b" {")?;
            for stmt in &body.stmts {
                out.write(b"\n        ")?;
                self.stmt(stmt, out)?;
            }
            out.write(b"\n    }")?;
        }
        Ok(())
    }

    /// Generate TypeScript class for type declaration
    pub fn type_decl(&mut self, type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"class ")?;
        out.write_all(type_decl.name.as_bytes())?;

        // Generic type parameters: class Foo<T, U>
        if !type_decl.generic_params.is_empty() {
            out.write(b"<")?;
            for (i, param) in type_decl.generic_params.iter().enumerate() {
                if i > 0 { out.write(b", ")?; }
                match param {
                    GenericParam::Type(tp) => out.write_all(tp.name.as_bytes())?,
                    GenericParam::Const(cp) => out.write_all(cp.name.as_bytes())?,
                }
            }
            out.write(b">")?;
        }

        // Inheritance: class Child extends Parent
        if let Some(ref parent) = type_decl.parent {
            out.write(b" extends ")?;
            out.write_all(Self::type_to_ts(parent).as_bytes())?;
        }

        // Spec implementations: class Pigeon implements Flyer
        if !type_decl.specs.is_empty() {
            out.write(b" implements ")?;
            for (i, spec) in type_decl.specs.iter().enumerate() {
                if i > 0 { out.write(b", ")?; }
                out.write_all(spec.as_bytes())?;
            }
        }

        out.write(b" {")?;


        // Members as properties
        for member in &type_decl.members {
            out.write(b"\n    ")?;
            out.write_all(member.name.as_bytes())?;
            out.write(b": ")?;

            let member_type = if !matches!(member.ty, Type::Unknown) {
                Self::type_to_ts(&member.ty)
            } else {
                "any".to_string()
            };
            out.write_all(member_type.as_bytes())?;
            out.write(b";")?;
        }

        // Constructor
        if !type_decl.members.is_empty() {
            out.write(b"\n\n    constructor(")?;
            for (i, member) in type_decl.members.iter().enumerate() {
                if i > 0 {
                    out.write(b", ")?;
                }
                out.write_all(member.name.as_bytes())?;
                if !matches!(member.ty, Type::Unknown) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&member.ty).as_bytes())?;
                }
            }
            out.write(b") {")?;
            for member in &type_decl.members {
                out.write(b"\n        this.")?;
                out.write_all(member.name.as_bytes())?;
                out.write(b" = ")?;
                out.write_all(member.name.as_bytes())?;
                out.write(b";")?;
            }
            out.write(b"\n    }")?;
        }

        // Methods
        for method in &type_decl.methods {
            out.write(b"\n\n    ")?;
            out.write_all(method.name.as_bytes())?;
            out.write(b"(")?;

            // Skip 'self' parameter — TypeScript methods use implicit `this`
            let mut first = true;
            for param in method.params.iter() {
                if param.name == "self" {
                    continue;
                }
                if !first {
                    out.write(b", ")?;
                }
                first = false;
                out.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                }
            }

            out.write(b")")?;

            // Return type
            if !matches!(method.ret, Type::Unknown | Type::Void) {
                out.write(b": ")?;
                out.write_all(Self::type_to_ts(&method.ret).as_bytes())?;
            } else if matches!(method.ret, Type::Void) {
                out.write(b": void")?;
            }

            // Method body — add `return` before the last expression
            // if the method has a non-void return type (TS method body
            // does not auto-return like arrow functions)
            let needs_return = !matches!(method.ret, Type::Unknown | Type::Void);
            out.write(b" {")?;
            let stmts = &method.body.stmts;
            for (i, stmt) in stmts.iter().enumerate() {
                out.write(b"\n        ")?;
                let is_last = i == stmts.len() - 1;
                if is_last && needs_return {
                    if let Stmt::Expr(expr) = stmt {
                        out.write(b"return ")?;
                        self.expr(expr, out)?;
                        out.write(b";")?;
                    } else {
                        self.stmt(stmt, out)?;
                    }
                } else {
                    self.stmt(stmt, out)?;
                }
            }
            out.write(b"\n    }")?;
        }

        out.write(b"\n}")?;
        Ok(())
    }

    /// Generate TypeScript `interface` for spec declaration
    pub fn spec_decl(&mut self, spec_decl: &SpecDecl, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"interface ")?;
        out.write_all(spec_decl.name.as_bytes())?;

        // Generic type parameters
        if !spec_decl.generic_params.is_empty() {
            out.write(b"<")?;
            for (i, param) in spec_decl.generic_params.iter().enumerate() {
                if i > 0 { out.write(b", ")?; }
                match param {
                    GenericParam::Type(tp) => out.write_all(tp.name.as_bytes())?,
                    GenericParam::Const(cp) => out.write_all(cp.name.as_bytes())?,
                }
            }
            out.write(b">")?;
        }

        out.write(b" {")?;

        for method in &spec_decl.methods {
            out.write(b"\n    ")?;
            out.write_all(method.name.as_bytes())?;
            out.write(b"(")?;
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 { out.write(b", ")?; }
                out.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                }
            }
            out.write(b")")?;
            if !matches!(method.ret, Type::Unknown | Type::Void) {
                out.write(b": ")?;
                out.write_all(Self::type_to_ts(&method.ret).as_bytes())?;
            } else if matches!(method.ret, Type::Void) {
                out.write(b": void")?;
            }
            out.write(b";")?;
        }

        out.write(b"\n}\n")?;
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

    pub fn enum_decl(&mut self, enum_decl: &EnumDecl, out: &mut impl Write) -> AutoResult<()> {
        match &enum_decl.kind {
            EnumKind::Scalar { .. } => {
                // C-style scalar enum: emit TypeScript const enum
                out.write(b"const enum ")?;
                out.write_all(enum_decl.name.as_bytes())?;
                out.write(b" {")?;

                for (i, item) in enum_decl.items.iter().enumerate() {
                    if i > 0 {
                        out.write(b",")?;
                    }
                    out.write(b"\n    ")?;
                    out.write_all(item.name.as_bytes())?;

                    // If there's an explicit non-zero value, use it
                    if item.value() != 0 {
                        out.write(b" = ")?;
                        write!(out, "{}", item.value())?;
                    }
                }

                out.write(b"\n}")?;
            }
            EnumKind::Homogeneous { payload_type } => {
                // Generate TS discriminated union: type Name = { _tag: "V1", value: T } | ...
                out.write(b"type ")?;
                out.write_all(enum_decl.name.as_bytes())?;
                out.write(b" =\n")?;

                for (i, item) in enum_decl.items.iter().enumerate() {
                    if i > 0 { out.write(b"\n    | ")?; } else { out.write(b"    ")?; }
                    out.write(b"{ _tag: \"")?;
                    out.write_all(item.name.as_bytes())?;
                    out.write(b"\", value: ")?;
                    out.write_all(Self::type_to_ts(payload_type).as_bytes())?;
                    out.write(b" }")?;
                }
                out.write(b";\n")?;
            }
            EnumKind::Heterogeneous { .. } => {
                // Reuse tag code generation: convert EnumDecl to Tag
                let tag = Self::enum_decl_to_tag(enum_decl);
                self.tag_decl(&tag, out)?;
            }
        }
        Ok(())
    }

    pub fn type_alias(&mut self, type_alias: &TypeAlias, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"type ")?;
        out.write_all(type_alias.name.as_bytes())?;
        
        if !type_alias.params.is_empty() {
            out.write(b"<")?;
            for (i, param) in type_alias.params.iter().enumerate() {
                if i > 0 { out.write(b", ")?; }
                out.write_all(param.as_bytes())?;
            }
            out.write(b">")?;
        }

        out.write(b" = ")?;
        out.write_all(Self::type_to_ts(&type_alias.target).as_bytes())?;
        out.write(b";\n")?;
        Ok(())
    }

    pub fn union_decl(&mut self, union: &Union, out: &mut impl Write) -> AutoResult<()> {
        // C-like unions are represented as objects with optional fields
        out.write(b"interface ")?;
        out.write_all(union.name.as_bytes())?;
        out.write(b" {")?;

        for member in &union.fields {
            out.write(b"\n    ")?;
            out.write_all(member.name.as_bytes())?;
            out.write(b"?: ")?;
            out.write_all(Self::type_to_ts(&member.ty).as_bytes())?;
            out.write(b";")?;
        }

        out.write(b"\n}\n")?;
        Ok(())
    }

    pub fn tag_decl(&mut self, tag: &Tag, out: &mut impl Write) -> AutoResult<()> {
        // TS algebraic data types: type Name = { type: "Option1", value: T } | ...
        out.write(b"type ")?;
        out.write_all(tag.name.as_bytes())?;

        if !tag.generic_params.is_empty() {
            out.write(b"<")?;
            for (i, param) in tag.generic_params.iter().enumerate() {
                if i > 0 { out.write(b", ")?; }
                match param {
                    GenericParam::Type(tp) => out.write_all(tp.name.as_bytes())?,
                    GenericParam::Const(cp) => {
                        out.write_all(cp.name.as_bytes())?;
                        out.write(b" extends ")?;
                        out.write_all(Self::type_to_ts(&cp.typ).as_bytes())?;
                    }
                }
            }
            out.write(b">")?;
        }
        out.write(b" =\n")?;

        for (i, field) in tag.fields.iter().enumerate() {
            if i > 0 { out.write(b"\n    | ")?; } else { out.write(b"    ")?; }
            out.write(b"{ _tag: \"")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"\", value: ")?;
            out.write_all(Self::type_to_ts(&field.ty).as_bytes())?;
            out.write(b" }")?;
        }
        out.write(b";\n\n")?;

        // Generate a const object with factory functions
        out.write(b"const ")?;
        out.write_all(tag.name.as_bytes())?;
        out.write(b" = {")?;
        for (i, field) in tag.fields.iter().enumerate() {
            if i > 0 { out.write(b",")?; }
            out.write(b"\n    ")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b": ")?;
            
            // Generic params for factory function
            if !tag.generic_params.is_empty() {
                out.write(b"<")?;
                for (j, param) in tag.generic_params.iter().enumerate() {
                    if j > 0 { out.write(b", ")?; }
                    match param {
                        GenericParam::Type(tp) => out.write_all(tp.name.as_bytes())?,
                        GenericParam::Const(cp) => {
                            out.write_all(cp.name.as_bytes())?;
                            out.write(b" extends ")?;
                            out.write_all(Self::type_to_ts(&cp.typ).as_bytes())?;
                        }
                    }
                }
                out.write(b">")?;
            }
            out.write(b"(value: ")?;
            out.write_all(Self::type_to_ts(&field.ty).as_bytes())?;
            out.write(b") => ({ _tag: \"")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"\", value })")?;
        }
        out.write(b"\n};\n")?;

        Ok(())
    }

    pub fn ext_decl(&mut self, ext: &Ext, out: &mut impl Write) -> AutoResult<()> {
        for method in &ext.methods {
            out.write_all(ext.target.as_bytes())?;
            out.write(b".prototype.")?;
            out.write_all(method.name.as_bytes())?;
            out.write(b" = function(")?;

            // Skip 'self' parameter — TypeScript methods use implicit `this`
            let mut first = true;
            for param in method.params.iter() {
                if param.name == "self" {
                    continue;
                }
                if !first { out.write(b", ")?; }
                first = false;
                out.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                }
            }
            out.write(b")")?;

            if !matches!(method.ret, Type::Unknown | Type::Void) {
                out.write(b": ")?;
                out.write_all(Self::type_to_ts(&method.ret).as_bytes())?;
            } else if matches!(method.ret, Type::Void) {
                out.write(b": void")?;
            }

            // Method body — add `return` before the last expression
            // if the method has a non-void return type (TS function()
            // does not auto-return like arrow functions)
            let needs_return = !matches!(method.ret, Type::Unknown | Type::Void);
            out.write(b" {")?;
            let stmts = &method.body.stmts;
            for (i, stmt) in stmts.iter().enumerate() {
                out.write(b"\n    ")?;
                let is_last = i == stmts.len() - 1;
                if is_last && needs_return {
                    if let Stmt::Expr(expr) = stmt {
                        out.write(b"return ")?;
                        self.expr(expr, out)?;
                        out.write(b";")?;
                    } else {
                        self.stmt(stmt, out)?;
                    }
                } else {
                    self.stmt(stmt, out)?;
                }
            }
            out.write(b"\n}")?;
            out.write(b";\n")?;
        }
        Ok(())
    }

    pub fn use_stmt(&mut self, use_stmt: &Use, out: &mut impl Write) -> AutoResult<()> {
        // Convert Auto use to TypeScript import
        let module_name = if let Some(ref mp) = use_stmt.module_path {
            mp.display().to_string()
                .replace("pac.", "./")
                .replace("super.", "../")
        } else if !use_stmt.paths.is_empty() {
            use_stmt.paths.join("/")
        } else {
            "unknown".to_string()
        };

        if use_stmt.is_wildcard {
            write!(out, "import * from \"{}\";", module_name)?;
        } else if use_stmt.items.is_empty() {
            write!(out, "import \"{}\";", module_name)?;
        } else {
            write!(out, "import {{ {} }} from \"{}\";",
                use_stmt.items.join(", "), module_name)?;
        }
        Ok(())
    }

    /// Check if a list of statements contains any await expressions
    fn has_await_expr(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Return(expr) => {
                    if Self::expr_has_await(expr) {
                        return true;
                    }
                }
                Stmt::Store(store) => {
                    if Self::expr_has_await(&store.expr) {
                        return true;
                    }
                }
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        if Self::body_has_await(&branch.body) {
                            return true;
                        }
                    }
                    if let Some(else_) = &if_stmt.else_ {
                        if Self::body_has_await(else_) {
                            return true;
                        }
                    }
                }
                Stmt::For(for_loop) => {
                    if Self::body_has_await(&for_loop.body) {
                        return true;
                    }
                }
                Stmt::Block(body) | Stmt::Fn(Fn { body, .. }) => {
                    if Self::has_await_expr(&body.stmts) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn body_has_await(body: &Body) -> bool {
        Self::has_await_expr(&body.stmts)
    }

    fn expr_has_await(expr: &Expr) -> bool {
        match expr {
            Expr::Await { .. } => true,
            Expr::Call(call) => {
                Self::expr_has_await(&call.name) ||
                    call.args.args.iter().any(|arg| match arg {
                        Arg::Pos(e) => Self::expr_has_await(e),
                        Arg::Pair(_, e) => Self::expr_has_await(e),
                        Arg::Name(_) => false,
                    })
            }
            Expr::Bina(l, _, r) => Self::expr_has_await(l) || Self::expr_has_await(r),
            Expr::Unary(_, e) => Self::expr_has_await(e),
            Expr::Dot(l, _) => Self::expr_has_await(l),
            Expr::Index(a, i) => Self::expr_has_await(a) || Self::expr_has_await(i),
            _ => false,
        }
    }
}
