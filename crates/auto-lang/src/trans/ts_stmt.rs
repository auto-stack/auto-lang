use crate::ast::*;
use crate::AutoResult;
use std::io::Write;
use auto_val::AutoStr;
use super::{Sink, TypeScriptTrans, ToStrError};
use super::super::escape_str;

#[allow(unused_variables)]
impl TypeScriptTrans {
    pub fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        match stmt {
            // Expression statements
            Stmt::Expr(expr) => {
                self.expr(expr, sink)?; // Uses ts_expr
                sink.body.write(b";")?;
                Ok(())
            }

            // Store (variable assignment) - with type annotation
            Stmt::Store(store) => {
                // TypeScript: const for let (immutable), let for var (mutable)
                match store.kind {
                    StoreKind::Let => sink.body.write(b"const ").to()?,
                    StoreKind::Var => sink.body.write(b"let ").to()?,
                    StoreKind::Const => sink.body.write(b"const ").to()?,
                    _ => {} // Field, CVar, Shared don't need declaration prefix
                };
                sink.body.write_all(store.name.as_bytes())?;

                // Add type annotation if type is known
                if !matches!(store.ty, Type::Unknown) {
                    sink.body.write(b": ")?;
                    sink.body.write_all(Self::type_to_ts(&store.ty).as_bytes())?; // Uses ts_types
                }

                sink.body.write(b" = ")?;
                self.expr(&store.expr, sink)?;
                sink.body.write(b";")?;
                Ok(())
            }

            // Function declarations - with type annotations
            Stmt::Fn(func) => {
                self.fn_decl(func, sink)?;
                Ok(())
            }

            // If statements
            Stmt::If(if_stmt) => {
                self.if_stmt(if_stmt, sink)?;
                Ok(())
            }

            // For loops
            Stmt::For(for_loop) => {
                self.for_loop(for_loop, sink)?;
                Ok(())
            }

            // Break statements
            Stmt::Break => {
                sink.body.write(b"break;")?;
                Ok(())
            }

            // Return statement
            Stmt::Return(expr) => {
                sink.body.write(b"return ")?;
                self.expr(expr, sink)?;
                sink.body.write(b";")?;
                Ok(())
            }

            // Node statements (e.g. loop wrap)
            Stmt::Node(node) => {
                self.expr(&Expr::Node(node.clone()), sink)?;
                if node.name != "loop" {
                    sink.body.write(b";")?;
                }
                Ok(())
            }

            // Pattern matching (is)
            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, sink)?;
                Ok(())
            }

            // Empty lines
            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    sink.body.write(b"\n")?;
                }
                Ok(())
            }

            // Type declarations → interface
            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, sink)?;
                Ok(())
            }

            // Enum declarations → const enum
            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, sink)?;
                Ok(())
            }

            // Type Aliases
            Stmt::TypeAlias(type_alias) => {
                self.type_alias(type_alias, sink)?;
                Ok(())
            }

            // Union declarations
            Stmt::Union(union) => {
                self.union_decl(union, sink)?;
                Ok(())
            }

            // Tag declarations
            Stmt::Tag(tag) => {
                self.tag_decl(tag, sink)?;
                Ok(())
            }

            // Type extensions
            Stmt::Ext(ext) => {
                self.ext_decl(ext, sink)?;
                Ok(())
            }

            // Spec (interface) declarations
            Stmt::SpecDecl(spec_decl) => {
                self.spec_decl(spec_decl, sink)?;
                Ok(())
            }

            // Continue
            Stmt::Continue => {
                sink.body.write(b"continue;")?;
                Ok(())
            }

            // Comments
            Stmt::Comment(comment) => {
                sink.body.write(b"// ")?;
                sink.body.write_all(comment.as_bytes())?;
                Ok(())
            }

            // Use statements → import
            Stmt::Use(use_stmt) => {
                self.use_stmt(use_stmt, sink)?;
                Ok(())
            }

            // Unsupported statements
            _ => Err(format!("TypeScript Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    pub fn fn_decl(&mut self, func: &Fn, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        // Detect async: functions returning ~T (Handle/Future) are async in TypeScript
        let is_async_fn = Self::has_await_expr(&func.body.stmts)
            || matches!(func.ret, Type::Handle { .. })
            || matches!(&func.ret, Type::GenericInstance(inst) if inst.base_name == "Future");

        if is_async_fn {
            sink.body.write(b"async ")?;
        }
        sink.body.write(b"function ")?;
        sink.body.write_all(func.name.as_bytes())?;
        sink.body.write(b"(")?;

        // Parameters with type annotations
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                sink.body.write(b", ")?;
            }
            sink.body.write_all(param.name.as_bytes())?;

            // Add type annotation
            if !matches!(param.ty, Type::Unknown) {
                sink.body.write(b": ")?;
                sink.body.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
            }
        }

        sink.body.write(b")")?;

        // Return type annotation
        if !matches!(func.ret, Type::Unknown | Type::Void) {
            sink.body.write(b": ")?;
            sink.body.write_all(Self::type_to_ts(&func.ret).as_bytes())?;
        } else if matches!(func.ret, Type::Void) {
            sink.body.write(b": void")?;
        }

        // Function body
        let needs_return = !matches!(func.ret, Type::Unknown | Type::Void) && func.name != "main";
        if needs_return && !func.body.stmts.is_empty() {
            self.open_block(sink)?;
            let stmts = &func.body.stmts;
            for (i, stmt) in stmts.iter().enumerate() {
                sink.record();
                sink.set_source_line(func.body.source_lines.get(i).copied().unwrap_or(0));
                sink.body.write(b"\n")?;
                self.print_indent(sink)?;
                let is_last = i == stmts.len() - 1;
                if is_last {
                    if let Stmt::Expr(expr) = stmt {
                        sink.body.write(b"return ")?;
                        self.expr(expr, sink)?;
                        sink.body.write(b";")?;
                    } else {
                        self.stmt(stmt, sink)?;
                    }
                } else {
                    self.stmt(stmt, sink)?;
                }
            }
            sink.record();
            self.close_block(sink)?;
        } else {
            self.body(&func.body, sink)?;
        }

        Ok(())
    }

    pub fn body(&mut self, body: &Body, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        self.open_block(sink)?;
        for (i, stmt) in body.stmts.iter().enumerate() {
            sink.record();
            sink.set_source_line(body.source_lines.get(i).copied().unwrap_or(0));
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            self.stmt(stmt, sink)?;
        }
        sink.record();
        self.close_block(sink)
    }

    pub fn if_stmt(&mut self, if_stmt: &If, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        // Process first branch as "if"
        if let Some(first_branch) = if_stmt.branches.first() {
            sink.body.write(b"if (")?;
            self.expr(&first_branch.cond, sink)?;
            sink.body.write(b")")?;
            self.if_body(&first_branch.body, sink)?;
        }

        // Process remaining branches as "else if"
        for branch in if_stmt.branches.iter().skip(1) {
            sink.body.write(b" else if (")?;
            self.expr(&branch.cond, sink)?;
            sink.body.write(b")")?;
            self.if_body(&branch.body, sink)?;
        }

        // Process else if present
        if let Some(else_) = &if_stmt.else_ {
            sink.body.write(b" else")?;
            self.if_body(else_, sink)?;
        }

        Ok(())
    }

    pub fn for_loop(&mut self, for_loop: &For, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        match &for_loop.iter {
            Iter::Cond => {
                sink.body.write(b"while (")?;
                self.expr(&for_loop.range, sink)?;
                sink.body.write(b")")?;
                self.if_body(&for_loop.body, sink)?;
            }
            Iter::Ever => {
                sink.body.write(b"while (true)")?;
                self.if_body(&for_loop.body, sink)?;
            }
            Iter::Named(name) => {
                // If the range is an Expr::Range, we can generate a traditional for loop
                if let Expr::Range(range) = &for_loop.range {
                    sink.body.write(b"for (let ")?;
                    sink.body.write_all(name.as_bytes())?;
                    sink.body.write(b" = ")?;
                    self.expr(&range.start, sink)?;
                    sink.body.write(b"; ")?;
                    sink.body.write_all(name.as_bytes())?;
                    if range.eq {
                        sink.body.write(b" <= ")?;
                    } else {
                        sink.body.write(b" < ")?;
                    }
                    self.expr(&range.end, sink)?;
                    sink.body.write(b"; ")?;
                    sink.body.write_all(name.as_bytes())?;
                    sink.body.write(b"++)")?;
                    self.if_body(&for_loop.body, sink)?;
                } else {
                    // For-each over array: for x in arr -> for (const x of arr)
                    sink.body.write(b"for (const ")?;
                    sink.body.write_all(name.as_bytes())?;
                    sink.body.write(b" of ")?;
                    self.expr(&for_loop.range, sink)?;
                    sink.body.write(b")")?;
                    self.if_body(&for_loop.body, sink)?;
                }
            }
            Iter::Indexed(index, name) => {
                // For-each with index over array: for i, x in arr -> for (let i = 0; i < arr.length; i++) { const x = arr[i]; }
                if let Expr::Range(range) = &for_loop.range {
                    // indexed range iteration
                    sink.body.write(b"for (let ")?;
                    sink.body.write_all(index.as_bytes())?;
                    sink.body.write(b" = 0, ")?;
                    sink.body.write_all(name.as_bytes())?;
                    sink.body.write(b" = ")?;
                    self.expr(&range.start, sink)?;
                    sink.body.write(b"; ")?;
                    sink.body.write_all(name.as_bytes())?;
                    if range.eq { sink.body.write(b" <= ")?; } else { sink.body.write(b" < ")?; }
                    self.expr(&range.end, sink)?;
                    sink.body.write(b"; ")?;
                    sink.body.write_all(index.as_bytes())?;
                    sink.body.write(b"++, ")?;
                    sink.body.write_all(name.as_bytes())?;
                    sink.body.write(b"++)")?;
                    self.if_body(&for_loop.body, sink)?;
                } else {
                    // We need a unique inner variable, or just a block
                    sink.body.write(b"for (let ")?;
                    sink.body.write_all(index.as_bytes())?;
                    sink.body.write(b" = 0; ")?;
                    sink.body.write_all(index.as_bytes())?;
                    sink.body.write(b" < ")?;
                    self.expr(&for_loop.range, sink)?;
                    sink.body.write(b".length; ")?;
                    sink.body.write_all(index.as_bytes())?;
                    sink.body.write(b"++)")?;
                    self.open_block(sink)?;
                    sink.body.write(b"\n")?;
                    self.print_indent(sink)?;
                    sink.body.write(b"const ")?;
                    sink.body.write_all(name.as_bytes())?;
                    sink.body.write(b" = ")?;
                    self.expr(&for_loop.range, sink)?;
                    sink.body.write(b"[")?;
                    sink.body.write_all(index.as_bytes())?;
                    sink.body.write(b"];")?;

                    for (i, stmt) in for_loop.body.stmts.iter().enumerate() {
                        sink.record();
                        sink.set_source_line(for_loop.body.source_lines.get(i).copied().unwrap_or(0));
                        sink.body.write(b"\n")?;
                        self.print_indent(sink)?;
                        self.stmt(stmt, sink)?;
                    }
                    sink.record();

                    self.close_block(sink)?;
                }
            }
            Iter::Destructured(key, val) => {
                // for (k, v) in map -> for (const [k, v] of Object.entries(map))
                sink.body.write(b"for (const [")?;
                sink.body.write_all(key.as_bytes())?;
                sink.body.write(b", ")?;
                sink.body.write_all(val.as_bytes())?;
                sink.body.write(b"] of Object.entries(")?;
                self.expr(&for_loop.range, sink)?;
                sink.body.write(b"))")?;
                self.if_body(&for_loop.body, sink)?;
            }
            _ => {
                return Err(format!("TypeScript Transpiler: unsupported for loop iteration: {:?}", for_loop.iter).into());
            }
        }
        Ok(())
    }

    pub fn is_stmt(
        &mut self,
        is_stmt: &Is,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let out = &mut sink.body;
        if self.can_use_switch_is(is_stmt) {
            self.emit_switch_is(is_stmt, sink)?;
        } else {
            self.emit_if_is(is_stmt, sink)?;
        }
        Ok(())
    }

    fn can_use_switch_is(
        &self,
        is_stmt: &Is,
    ) -> bool {
        for branch in &is_stmt.branches {
            match branch {
                IsBranch::EqBranch(patterns, _) => {
                    for pat in patterns {
                        if !self.is_switchable_pattern(pat) {
                            return false;
                        }
                    }
                }
                IsBranch::ElseBranch(_) => {}
                IsBranch::IfBranch(_, _) => return false,
            }
        }
        true
    }

    fn is_switchable_pattern(
        &self,
        pat: &Expr,
    ) -> bool {
        match pat {
            Expr::Cover(cover) => match cover {
                crate::ast::Cover::Tag(tag_cover) => {
                    let real_bindings: Vec<&AutoStr> = tag_cover.bindings.iter()
                        .filter(|b| b.as_str() != "_")
                        .collect();
                    self.scalar_enums.contains(&tag_cover.kind) && real_bindings.is_empty()
                }
            }
            Expr::Int(_) | Expr::Uint(_) | Expr::Float(_, _) | Expr::Bool(_) | Expr::Str(_) | Expr::Ident(_) | Expr::Nil | Expr::Null => true,
            _ => false,
        }
    }

    fn emit_switch_is(
        &mut self,
        is_stmt: &Is,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let out = &mut sink.body;
        sink.body.write(b"switch (")?;
        self.expr(&is_stmt.target, sink)?;
        sink.body.write(b")")?;
        self.open_block(sink)?;

        for branch in &is_stmt.branches {
            match branch {
                IsBranch::EqBranch(patterns, body) => {
                    for pat in patterns {
                        sink.body.write(b"\n")?;
                        self.print_indent(sink)?;
                        sink.body.write(b"case ")?;
                        self.emit_switch_case_value(pat, sink)?;
                        sink.body.write(b":")?;
                    }
                    self.emit_switch_body(body, sink)?;
                }
                IsBranch::ElseBranch(body) => {
                    sink.body.write(b"\n")?;
                    self.print_indent(sink)?;
                    sink.body.write(b"default:")?;
                    self.emit_switch_body(body, sink)?;
                }
                IsBranch::IfBranch(_, _) => {}
            }
        }

        self.close_block(sink)
    }

    fn emit_switch_case_value(
        &mut self,
        pat: &Expr,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let out = &mut sink.body;
        match pat {
            Expr::Cover(cover) => match cover {
                crate::ast::Cover::Tag(tag_cover) => {
                    sink.body.write_all(tag_cover.kind.as_bytes())?;
                    sink.body.write(b".")?;
                    sink.body.write_all(tag_cover.tag.as_bytes())?;
                }
            }
            Expr::Int(i) => {
                write!(&mut sink.body, "{}", i)?;
            }
            Expr::Uint(u) => {
                write!(&mut sink.body, "{}", u)?;
            }
            Expr::Float(f, _) => {
                write!(&mut sink.body, "{}", f)?;
            }
            Expr::Bool(b) => {
                sink.body.write(if *b { b"true" } else { b"false" })?;
            }
            Expr::Str(s) => {
                sink.body.write(b"\"")?;
                sink.body.write_all(escape_str(s).as_bytes())?;
                sink.body.write(b"\"")?;
            }
            Expr::Ident(name) => {
                sink.body.write_all(name.as_bytes())?;
            }
            Expr::Nil | Expr::Null => {
                sink.body.write(b"null")?;
            }
            _ => {
                self.expr(pat, sink)?;
            }
        }
        Ok(())
    }

    fn emit_switch_body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let out = &mut sink.body;
        self.indent();
        for (i, stmt) in body.stmts.iter().enumerate() {
            sink.record();
            sink.set_source_line(body.source_lines.get(i).copied().unwrap_or(0));
            let out = &mut sink.body;
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            self.stmt(stmt, sink)?;
        }
        sink.record();
        let out = &mut sink.body;
        sink.body.write(b"\n")?;
        self.print_indent(sink)?;
        sink.body.write(b"break;")?;
        self.dedent();
        Ok(())
    }

    fn emit_if_is(
        &mut self,
        is_stmt: &Is,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let out = &mut sink.body;
        // TypeScript has no pattern matching; use a chain of if/else if with _tag checks.
        let target_var = format!("__auto_is_{}", self.is_counter);
        self.is_counter += 1;
        self.print_indent(sink)?;
        sink.body.write(b"const ")?;
        sink.body.write(target_var.as_bytes())?;
        sink.body.write(b" = ")?;
        self.expr(&is_stmt.target, sink)?;
        sink.body.write(b";")?;

        for (i, branch) in is_stmt.branches.iter().enumerate() {
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            if i == 0 {
                sink.body.write(b"if (")?;
            } else {
                sink.body.write(b"else if (")?;
            }

            match branch {
                IsBranch::EqBranch(patterns, _) => {
                    for (j, pat) in patterns.iter().enumerate() {
                        if j > 0 { sink.body.write(b" || ")?; }
                        self.emit_is_condition(&target_var, pat, sink)?;
                    }
                }
                IsBranch::IfBranch(expr, _) => {
                    self.expr(expr, sink)?;
                }
                IsBranch::ElseBranch(_) => {
                    // 'else' has no condition; this arm is handled after the loop.
                    continue;
                }
            }

            sink.body.write(b")")?;
            self.open_block(sink)?;

            // Emit bindings for the first pattern (EqBranch) or the IfBranch expression pattern
            match branch {
                IsBranch::EqBranch(patterns, _) if !patterns.is_empty() => {
                    self.emit_is_bindings(&target_var, &patterns[0], sink)?;
                }
                IsBranch::IfBranch(_, _) => {}
                _ => {}
            }

            // Body
            let body = match branch {
                IsBranch::EqBranch(_, body) => body,
                IsBranch::IfBranch(_, body) => body,
                IsBranch::ElseBranch(body) => body,
            };
            for (j, stmt) in body.stmts.iter().enumerate() {
                sink.record();
                sink.set_source_line(body.source_lines.get(j).copied().unwrap_or(0));
                sink.body.write(b"\n")?;
                self.print_indent(sink)?;
                self.stmt(stmt, sink)?;
            }
            sink.record();
            self.close_block(sink)?;
        }

        // Handle else/default branch
        if let Some(else_branch) = is_stmt.branches.iter().find(|b| matches!(b, IsBranch::ElseBranch(_))) {
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            sink.body.write(b"else")?;
            self.open_block(sink)?;
            if let IsBranch::ElseBranch(body) = else_branch {
                for (i, stmt) in body.stmts.iter().enumerate() {
                    sink.record();
                    sink.set_source_line(body.source_lines.get(i).copied().unwrap_or(0));
                    sink.body.write(b"\n")?;
                    self.print_indent(sink)?;
                    self.stmt(stmt, sink)?;
                }
                sink.record();
            }
            self.close_block(sink)?;
        }

        Ok(())
    }

    fn emit_is_condition(
        &mut self,
        target_var: &str,
        pat: &Expr,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let out = &mut sink.body;
        match pat {
            Expr::Cover(cover) => {
                match cover {
                    crate::ast::Cover::Tag(tag_cover) => {
                        let real_bindings: Vec<&AutoStr> = tag_cover.bindings.iter()
                            .filter(|b| b.as_str() != "_")
                            .collect();
                        if self.scalar_enums.contains(&tag_cover.kind) && real_bindings.is_empty() {
                            sink.body.write(target_var.as_bytes())?;
                            sink.body.write(b" === ")?;
                            sink.body.write_all(tag_cover.kind.as_bytes())?;
                            sink.body.write(b".")?;
                            sink.body.write_all(tag_cover.tag.as_bytes())?;
                        } else {
                            sink.body.write(target_var.as_bytes())?;
                            sink.body.write(b"._tag === \"")?;
                            sink.body.write_all(tag_cover.tag.as_bytes())?;
                            sink.body.write(b"\"")?;
                        }
                    }
                }
            }
            Expr::Int(i) => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === ")?;
                write!(&mut sink.body, "{}", i)?;
            }
            Expr::Uint(u) => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === ")?;
                write!(&mut sink.body, "{}", u)?;
            }
            Expr::Float(f, _) => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === ")?;
                write!(&mut sink.body, "{}", f)?;
            }
            Expr::Bool(b) => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === ")?;
                sink.body.write(if *b { b"true" } else { b"false" })?;
            }
            Expr::Str(s) => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === \"")?;
                sink.body.write_all(escape_str(s).as_bytes())?;
                sink.body.write(b"\"")?;
            }
            Expr::Ident(name) => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === ")?;
                sink.body.write_all(name.as_bytes())?;
            }
            Expr::Nil | Expr::Null => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === null")?;
            }
            _ => {
                sink.body.write(target_var.as_bytes())?;
                sink.body.write(b" === ")?;
                self.expr(pat, sink)?;
            }
        }
        Ok(())
    }

    fn emit_is_bindings(
        &mut self,
        target_var: &str,
        pat: &Expr,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let out = &mut sink.body;
        let cover = match pat {
            Expr::Cover(cover) => cover,
            _ => return Ok(()),
        };
        match cover {
            crate::ast::Cover::Tag(tag_cover) => {
                let real_bindings: Vec<&AutoStr> = tag_cover.bindings.iter()
                    .filter(|b| b.as_str() != "_")
                    .collect();
                if real_bindings.is_empty() {
                    return Ok(());
                }
                sink.body.write(b"\n")?;
                self.print_indent(sink)?;
                if real_bindings.len() == 1 {
                    sink.body.write(b"const ")?;
                    sink.body.write_all(real_bindings[0].as_bytes())?;
                    sink.body.write(b" = ")?;
                    sink.body.write(target_var.as_bytes())?;
                    sink.body.write(b".value;")?;
                } else {
                    sink.body.write(b"const [")?;
                    for (i, b) in real_bindings.iter().enumerate() {
                        if i > 0 { sink.body.write(b", ")?; }
                        sink.body.write_all(b.as_bytes())?;
                    }
                    sink.body.write(b"] = ")?;
                    sink.body.write(target_var.as_bytes())?;
                    sink.body.write(b".value;")?;
                }
            }
        }
        Ok(())
    }

    pub fn if_body(&mut self, body: &Body, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        if body.stmts.is_empty() {
            sink.body.write(b" {}")?;
        } else {
            self.open_block(sink)?;
            for (i, stmt) in body.stmts.iter().enumerate() {
                sink.record();
                sink.set_source_line(body.source_lines.get(i).copied().unwrap_or(0));
                sink.body.write(b"\n")?;
                self.print_indent(sink)?;
                self.stmt(stmt, sink)?;
            }
            sink.record();
            self.close_block(sink)?;
        }
        Ok(())
    }

    /// Generate TypeScript class for type declaration
    pub fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        sink.body.write(b"class ")?;
        sink.body.write_all(type_decl.name.as_bytes())?;

        // Generic type parameters: class Foo<T, U>
        if !type_decl.generic_params.is_empty() {
            sink.body.write(b"<")?;
            for (i, param) in type_decl.generic_params.iter().enumerate() {
                if i > 0 { sink.body.write(b", ")?; }
                match param {
                    GenericParam::Type(tp) => sink.body.write_all(tp.name.as_bytes())?,
                    GenericParam::Const(cp) => sink.body.write_all(cp.name.as_bytes())?,
                }
            }
            sink.body.write(b">")?;
        }

        // Inheritance: class Child extends Parent
        if let Some(ref parent) = type_decl.parent {
            sink.body.write(b" extends ")?;
            sink.body.write_all(Self::type_to_ts(parent).as_bytes())?;
        }

        // Spec implementations: class Pigeon implements Flyer
        if !type_decl.specs.is_empty() {
            sink.body.write(b" implements ")?;
            for (i, spec) in type_decl.specs.iter().enumerate() {
                if i > 0 { sink.body.write(b", ")?; }
                sink.body.write_all(spec.as_bytes())?;
            }
        }

        self.open_block(sink)?;

        // Members as properties
        for member in &type_decl.members {
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            sink.body.write_all(member.name.as_bytes())?;
            sink.body.write(b": ")?;

            let member_type = if !matches!(member.ty, Type::Unknown) {
                Self::type_to_ts(&member.ty)
            } else {
                "any".to_string()
            };
            sink.body.write_all(member_type.as_bytes())?;
            sink.body.write(b";")?;
        }

        // Constructor
        if !type_decl.members.is_empty() {
            sink.body.write(b"\n\n")?;
            self.print_indent(sink)?;
            sink.body.write(b"constructor(")?;
            for (i, member) in type_decl.members.iter().enumerate() {
                if i > 0 {
                    sink.body.write(b", ")?;
                }
                sink.body.write_all(member.name.as_bytes())?;
                if !matches!(member.ty, Type::Unknown) {
                    sink.body.write(b": ")?;
                    sink.body.write_all(Self::type_to_ts(&member.ty).as_bytes())?;
                }
            }
            sink.body.write(b")")?;
            self.open_block(sink)?;
            for member in &type_decl.members {
                sink.body.write(b"\n")?;
                self.print_indent(sink)?;
                sink.body.write(b"this.")?;
                sink.body.write_all(member.name.as_bytes())?;
                sink.body.write(b" = ")?;
                sink.body.write_all(member.name.as_bytes())?;
                sink.body.write(b";")?;
            }
            self.close_block(sink)?;
        }

        // Methods
        for method in &type_decl.methods {
            sink.body.write(b"\n\n")?;
            self.print_indent(sink)?;
            sink.body.write_all(method.name.as_bytes())?;
            sink.body.write(b"(")?;

            // Skip 'self' parameter — TypeScript methods use implicit `this`
            let mut first = true;
            for param in method.params.iter() {
                if param.name == "self" {
                    continue;
                }
                if !first {
                    sink.body.write(b", ")?;
                }
                first = false;
                sink.body.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    sink.body.write(b": ")?;
                    sink.body.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                }
            }

            sink.body.write(b")")?;

            // Return type
            if !matches!(method.ret, Type::Unknown | Type::Void) {
                sink.body.write(b": ")?;
                sink.body.write_all(Self::type_to_ts(&method.ret).as_bytes())?;
            } else if matches!(method.ret, Type::Void) {
                sink.body.write(b": void")?;
            }

            // Method body — add `return` before the last expression
            // if the method has a non-void return type
            let needs_return = !matches!(method.ret, Type::Unknown | Type::Void);
            self.open_block(sink)?;
            let stmts = &method.body.stmts;
            for (i, stmt) in stmts.iter().enumerate() {
                sink.record();
                sink.set_source_line(method.body.source_lines.get(i).copied().unwrap_or(0));
                sink.body.write(b"\n")?;
                self.print_indent(sink)?;
                let is_last = i == stmts.len() - 1;
                if is_last && needs_return {
                    if let Stmt::Expr(expr) = stmt {
                        sink.body.write(b"return ")?;
                        self.expr(expr, sink)?;
                        sink.body.write(b";")?;
                    } else {
                        self.stmt(stmt, sink)?;
                    }
                } else {
                    self.stmt(stmt, sink)?;
                }
            }
            sink.record();
            self.close_block(sink)?;
        }

        self.close_block(sink)
    }

    /// Generate TypeScript `interface` for spec declaration
    pub fn spec_decl(&mut self, spec_decl: &SpecDecl, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        sink.body.write(b"interface ")?;
        sink.body.write_all(spec_decl.name.as_bytes())?;

        // Generic type parameters
        if !spec_decl.generic_params.is_empty() {
            sink.body.write(b"<")?;
            for (i, param) in spec_decl.generic_params.iter().enumerate() {
                if i > 0 { sink.body.write(b", ")?; }
                match param {
                    GenericParam::Type(tp) => sink.body.write_all(tp.name.as_bytes())?,
                    GenericParam::Const(cp) => sink.body.write_all(cp.name.as_bytes())?,
                }
            }
            sink.body.write(b">")?;
        }

        sink.body.write(b" {")?;
        self.open_block(sink)?;

        for method in &spec_decl.methods {
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            sink.body.write_all(method.name.as_bytes())?;
            sink.body.write(b"(")?;
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 { sink.body.write(b", ")?; }
                sink.body.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    sink.body.write(b": ")?;
                    sink.body.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                }
            }
            sink.body.write(b")")?;
            if !matches!(method.ret, Type::Unknown | Type::Void) {
                sink.body.write(b": ")?;
                sink.body.write_all(Self::type_to_ts(&method.ret).as_bytes())?;
            } else if matches!(method.ret, Type::Void) {
                sink.body.write(b": void")?;
            }
            sink.body.write(b";")?;
        }

        self.close_block(sink)?;
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

    fn enum_item_payload_type(item: &EnumItem) -> Option<AutoStr> {
        if item.has_tuple_payload() {
            let parts: Vec<String> = item.payload_types.iter()
                .map(|t| Self::type_to_ts(t))
                .collect();
            Some(format!("[{}]", parts.join(", ")).into())
        } else if let Some(ty) = &item.payload_type {
            Some(Self::type_to_ts(ty).into())
        } else if item.has_fields() {
            Some("any".into())
        } else {
            None
        }
    }

    pub fn enum_decl(&mut self, enum_decl: &EnumDecl, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        match &enum_decl.kind {
            EnumKind::Scalar { .. } => {
                self.scalar_enums.insert(enum_decl.name.clone().into());
                // C-style scalar enum: emit TypeScript const enum
                sink.body.write(b"const enum ")?;
                sink.body.write_all(enum_decl.name.as_bytes())?;
                self.open_block(sink)?;

                for (i, item) in enum_decl.items.iter().enumerate() {
                    if i > 0 {
                        sink.body.write(b",")?;
                    }
                    sink.body.write(b"\n")?;
                    self.print_indent(sink)?;
                    sink.body.write_all(item.name.as_bytes())?;

                    // If there's an explicit non-zero value, use it
                    if item.value() != 0 {
                        sink.body.write(b" = ")?;
                        write!(&mut sink.body, "{}", item.value())?;
                    }
                }

                self.close_block(sink)?;
            }
            EnumKind::Homogeneous { .. } | EnumKind::Heterogeneous { .. } => {
                // Generate TS discriminated union: type Name = { _tag: "V1", value: T } | ...
                sink.body.write(b"type ")?;
                sink.body.write_all(enum_decl.name.as_bytes())?;
                sink.body.write(b" =\n")?;

                for (i, item) in enum_decl.items.iter().enumerate() {
                    if i > 0 { sink.body.write(b"\n    | ")?; } else { sink.body.write(b"    ")?; }
                    sink.body.write(b"{ _tag: \"")?;
                    sink.body.write_all(item.name.as_bytes())?;
                    sink.body.write(b"\"")?;
                    if let Some(ty) = Self::enum_item_payload_type(item) {
                        sink.body.write(b", value: ")?;
                        sink.body.write_all(ty.as_bytes())?;
                    }
                    sink.body.write(b" }")?;
                }
                sink.body.write(b";\n\n")?;

                // Factory object
                sink.body.write(b"const ")?;
                sink.body.write_all(enum_decl.name.as_bytes())?;
                sink.body.write(b" =")?;
                self.open_block(sink)?;
                for (i, item) in enum_decl.items.iter().enumerate() {
                    if i > 0 { sink.body.write(b",")?; }
                    sink.body.write(b"\n")?;
                    self.print_indent(sink)?;
                    sink.body.write_all(item.name.as_bytes())?;
                    sink.body.write(b": ")?;
                    if let Some(ty) = Self::enum_item_payload_type(item) {
                        sink.body.write(b"(value: ")?;
                        sink.body.write_all(ty.as_bytes())?;
                        sink.body.write(b") => ({ _tag: \"")?;
                        sink.body.write_all(item.name.as_bytes())?;
                        sink.body.write(b"\" as const, value })")?;
                    } else {
                        sink.body.write(b"() => ({ _tag: \"")?;
                        sink.body.write_all(item.name.as_bytes())?;
                        sink.body.write(b"\" as const })")?;
                    }
                }
                self.close_block(sink)?;
                sink.body.write(b";\n")?;
            }
        }
        Ok(())
    }

    pub fn type_alias(&mut self, type_alias: &TypeAlias, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        sink.body.write(b"type ")?;
        sink.body.write_all(type_alias.name.as_bytes())?;
        
        if !type_alias.params.is_empty() {
            sink.body.write(b"<")?;
            for (i, param) in type_alias.params.iter().enumerate() {
                if i > 0 { sink.body.write(b", ")?; }
                sink.body.write_all(param.as_bytes())?;
            }
            sink.body.write(b">")?;
        }

        sink.body.write(b" = ")?;
        sink.body.write_all(Self::type_to_ts(&type_alias.target).as_bytes())?;
        sink.body.write(b";\n")?;
        Ok(())
    }

    pub fn union_decl(&mut self, union: &Union, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        // C-like unions are represented as objects with optional fields
        sink.body.write(b"interface ")?;
        sink.body.write_all(union.name.as_bytes())?;
        self.open_block(sink)?;

        for member in &union.fields {
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            sink.body.write_all(member.name.as_bytes())?;
            sink.body.write(b"?: ")?;
            sink.body.write_all(Self::type_to_ts(&member.ty).as_bytes())?;
            sink.body.write(b";")?;
        }

        self.close_block(sink)?;
        sink.body.write(b"\n")?;
        Ok(())
    }

    pub fn tag_decl(&mut self, tag: &Tag, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        // TS algebraic data types: type Name = { type: "Option1", value: T } | ...
        sink.body.write(b"type ")?;
        sink.body.write_all(tag.name.as_bytes())?;

        if !tag.generic_params.is_empty() {
            sink.body.write(b"<")?;
            for (i, param) in tag.generic_params.iter().enumerate() {
                if i > 0 { sink.body.write(b", ")?; }
                match param {
                    GenericParam::Type(tp) => sink.body.write_all(tp.name.as_bytes())?,
                    GenericParam::Const(cp) => {
                        sink.body.write_all(cp.name.as_bytes())?;
                        sink.body.write(b" extends ")?;
                        sink.body.write_all(Self::type_to_ts(&cp.typ).as_bytes())?;
                    }
                }
            }
            sink.body.write(b">")?;
        }
        sink.body.write(b" =\n")?;

        for (i, field) in tag.fields.iter().enumerate() {
            if i > 0 { sink.body.write(b"\n    | ")?; } else { sink.body.write(b"    ")?; }
            sink.body.write(b"{ _tag: \"")?;
            sink.body.write_all(field.name.as_bytes())?;
            sink.body.write(b"\", value: ")?;
            sink.body.write_all(Self::type_to_ts(&field.ty).as_bytes())?;
            sink.body.write(b" }")?;
        }
        sink.body.write(b";\n\n")?;

        // Generate a const object with factory functions
        sink.body.write(b"const ")?;
        sink.body.write_all(tag.name.as_bytes())?;
        sink.body.write(b" =")?;
        self.open_block(sink)?;
        for (i, field) in tag.fields.iter().enumerate() {
            if i > 0 { sink.body.write(b",")?; }
            sink.body.write(b"\n")?;
            self.print_indent(sink)?;
            sink.body.write_all(field.name.as_bytes())?;
            sink.body.write(b": ")?;
            
            // Generic params for factory function
            if !tag.generic_params.is_empty() {
                sink.body.write(b"<")?;
                for (j, param) in tag.generic_params.iter().enumerate() {
                    if j > 0 { sink.body.write(b", ")?; }
                    match param {
                        GenericParam::Type(tp) => sink.body.write_all(tp.name.as_bytes())?,
                        GenericParam::Const(cp) => {
                            sink.body.write_all(cp.name.as_bytes())?;
                            sink.body.write(b" extends ")?;
                            sink.body.write_all(Self::type_to_ts(&cp.typ).as_bytes())?;
                        }
                    }
                }
                sink.body.write(b">")?;
            }
            sink.body.write(b"(value: ")?;
            sink.body.write_all(Self::type_to_ts(&field.ty).as_bytes())?;
            sink.body.write(b") => ({ _tag: \"")?;
            sink.body.write_all(field.name.as_bytes())?;
            sink.body.write(b"\", value })")?;
        }
        self.close_block(sink)?;
        sink.body.write(b";\n")?;

        Ok(())
    }

    pub fn ext_decl(&mut self, ext: &Ext, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        for method in &ext.methods {
            sink.body.write_all(ext.target.as_bytes())?;
            sink.body.write(b".prototype.")?;
            sink.body.write_all(method.name.as_bytes())?;
            sink.body.write(b" = function(")?;

            // Skip 'self' parameter — TypeScript methods use implicit `this`
            let mut first = true;
            for param in method.params.iter() {
                if param.name == "self" {
                    continue;
                }
                if !first { sink.body.write(b", ")?; }
                first = false;
                sink.body.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    sink.body.write(b": ")?;
                    sink.body.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                }
            }
            sink.body.write(b")")?;

            if !matches!(method.ret, Type::Unknown | Type::Void) {
                sink.body.write(b": ")?;
                sink.body.write_all(Self::type_to_ts(&method.ret).as_bytes())?;
            } else if matches!(method.ret, Type::Void) {
                sink.body.write(b": void")?;
            }

            // Method body — add `return` before the last expression
            // if the method has a non-void return type
            let needs_return = !matches!(method.ret, Type::Unknown | Type::Void);
            self.open_block(sink)?;
            let stmts = &method.body.stmts;
            for (i, stmt) in stmts.iter().enumerate() {
                sink.record();
                sink.set_source_line(method.body.source_lines.get(i).copied().unwrap_or(0));
                sink.body.write(b"\n")?;
                self.print_indent(sink)?;
                let is_last = i == stmts.len() - 1;
                if is_last && needs_return {
                    if let Stmt::Expr(expr) = stmt {
                        sink.body.write(b"return ")?;
                        self.expr(expr, sink)?;
                        sink.body.write(b";")?;
                    } else {
                        self.stmt(stmt, sink)?;
                    }
                } else {
                    self.stmt(stmt, sink)?;
                }
            }
            sink.record();
            self.close_block(sink)?;
            sink.body.write(b";\n")?;
        }
        Ok(())
    }

    pub fn use_stmt(&mut self, use_stmt: &Use, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
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
            write!(&mut sink.body, "import * from \"{}\";", module_name)?;
        } else if use_stmt.items.is_empty() {
            write!(&mut sink.body, "import \"{}\";", module_name)?;
        } else {
            write!(&mut sink.body, "import {{ {} }} from \"{}\";",
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
