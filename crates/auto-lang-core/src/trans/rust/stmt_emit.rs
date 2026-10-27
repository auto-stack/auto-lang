//! PLAN-843 结构拆分（自 rust.rs 原样搬移；算法/行为零变化）。
use super::*;

impl RustTrans {
    /// Access the struct_fields cache (for pre-population from sibling files)
    pub fn struct_fields(&self) -> &HashMap<AutoStr, Vec<AutoStr>> {
        &self.struct_fields
    }

    /// Mutable access to the struct_fields cache
    pub fn struct_fields_mut(&mut self) -> &mut HashMap<AutoStr, Vec<AutoStr>> {
        &mut self.struct_fields
    }

    /// Plan 742 (D6): split statement glue from block-close emission — the
    /// if/for close brace occasionally lands on the same line as the NEXT
    /// statement (`}    let x = ...`), which breaks the per-line return-wrap
    /// pass (the line no longer starts with `return`). Only splits on
    /// statement keywords to stay away from struct/map literals.
    /// Plan 742 R1 (F-742-R1-01): `}` / keyword candidates must sit in code
    /// regions — a business string like `"} return secret"` must survive
    /// verbatim, not split mid-literal.
    fn split_glued_statements(line: &str) -> String {
        const KEYWORDS: [&str; 8] = [
            "let ", "return", "if ", "for ", "while ", "match ", "push(", "ok =",
        ];
        let mask = Self::code_region_mask(line);
        let bytes = line.as_bytes();
        let mut out = String::with_capacity(line.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'}' && mask[i] {
                // 找 } 之后的空白段，若空白后是语句关键字则在此断行
                let mut j = i + 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == 9) && mask[j] {
                    j += 1;
                }
                if j < bytes.len() && mask[j] {
                    let rest = &line[j..];
                    if KEYWORDS.iter().any(|k| rest.starts_with(k)) && j > i + 1 {
                        out.push_str("}\n");
                        i = j;
                        continue;
                    }
                }
            }
            let ch = line[i..].chars().next().unwrap_or(' ');
            out.push(ch);
            i += ch.len_utf8();
        }
        out
    }

    /// Plan 742: lower residual `List<T>.new(args)` constructor calls in
    /// body-stmt output to their Rust vec form. The api.rs body path
    /// (try_transpile_body) reaches the shared stmt emitter without the
    /// module-path registration that normally lowers these in place, and the
    /// verbatim `List<Note>.new(vec![])` shape is not valid Rust.
    /// `List<T>.new(vec![])` → `vec![]`; `List<T>.new([a, b])` → `vec![a, b]`;
    /// `List<T>.new(expr)` → `expr` (annotation carries the Vec<T> type).
    /// Plan 742 R1 (F-742-R1-01): the construct and its argument parens must
    /// sit in code regions — a business string like `"List<int>.new([])"`
    /// is literal text, not a construct, and must survive verbatim.
    fn lower_list_constructs(line: &str) -> String {
        let needle = ".new(";
        let mask = Self::code_region_mask(line);
        let bytes = line.as_bytes();
        let mut out = String::with_capacity(line.len());
        let mut i = 0;
        while i < bytes.len() {
            // Find `List<` opening (code region only).
            if mask[i] && line[i..].starts_with("List<") {
                if let Some(gt) = line[i..].find('>') {
                    let inner_end = i + gt;
                    // Matching `.new(` right after the closing '>'.
                    if mask[inner_end] && line[inner_end + 1..].starts_with(needle) {
                        let args_start = inner_end + 1 + needle.len();
                        // Find the matching closing ')' of `.new(` — depth
                        // changes only count parens in code regions.
                        let mut depth = 1;
                        let mut j = args_start;
                        while j < bytes.len() {
                            if mask[j] {
                                match bytes[j] {
                                    b'(' => depth += 1,
                                    b')' => {
                                        depth -= 1;
                                        if depth == 0 {
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            j += 1;
                        }
                        if j < bytes.len() {
                            let args = line[args_start..j].trim();
                            let lowered_args = if args.starts_with('[') {
                                format!("vec![{}]", &args[1..args.len().saturating_sub(1)])
                            } else {
                                args.to_string()
                            };
                            out.push_str(&lowered_args);
                            i = j + 1;
                            continue;
                        }
                    }
                }
            }
            let ch = line[i..].chars().next().unwrap_or(' ');
            out.push(ch);
            i += ch.len_utf8();
        }
        out
    }

    pub fn transpile_body_stmts(
        &mut self,
        body: &crate::ast::Body,
        params: &[(AutoStr, Type)],
    ) -> AutoResult<Vec<String>> {
        self.local_var_types.clear();
        self.http_stream_vars.clear();
        self.http_stream_async_vars.clear();
        self.http_builder_vars.clear();
        self.str_slice_pattern_bindings.clear();
        // Plan 016 (auto-down R1): loop-var tracking sets are function-scoped
        // too — a leaked `for a in ...` binding from an earlier fn shadows a
        // same-named match binding in a later fn (e.g. attrDel's `a` vs
        // applyOp's `Op::InsertText(a)`), suppressing call-site .as_str() on
        // its field reads (E0308).
        self.borrowed_iter_vars.clear();
        self.by_value_iter_bindings.clear();
        for (name, ty) in params {
            self.local_var_types.insert(name.clone(), ty.clone());
        }
        self.mutated_let_bindings.clear();
        let rust_types = self.rust_imported_type_names();
        self.mutated_let_bindings =
            Self::scan_mutated_bindings(body, &self.all_mut_method_names, &rust_types);
        self.later_used_locals.clear();
        // PLAN-010 T2: per-statement later-read sets (see the field doc).
        // A None from the collector (unwalked variant) means "assume used".
        let body_stmt_reads: Vec<Option<std::collections::HashSet<AutoStr>>> =
            body.stmts.iter().map(Self::stmt_read_idents).collect();
        // Plan 447 H5: same per-function is-scrutinee counts for the
        // body-only transpile path (used by UI/excerpt pipelines).
        self.fn_is_scrutinee_counts = Self::scan_is_scrutinee_uses(body);
        self.current_scope_depth = 0;
        let mut result = Vec::new();
        for (stmt_i, stmt) in body.stmts.iter().enumerate() {
            self.later_used_locals = body_stmt_reads[stmt_i + 1..].iter().fold(
                std::collections::HashSet::new(),
                |mut acc, s| {
                    match s {
                        Some(names) => {
                            acc.extend(names.iter().cloned());
                        }
                        // Unwalkable stmt later in the body — assume everything
                        // might be referenced; suppress the type defaults.
                        None => {
                            acc.insert(AutoStr::from("\u{0}unwalkable"));
                        }
                    }
                    acc
                },
            );
            let mut sink = Sink::new(AutoStr::from("body_stmt"));
            self.stmt(stmt, &mut sink)?;
            let raw = String::from_utf8(sink.done()?.to_vec())
                .map_err(|e| format!("Invalid UTF-8 in body stmt: {}", e))?;
            for line in raw.lines() {
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    let deglued = Self::split_glued_statements(trimmed);
                    for piece in deglued.split('\n') {
                        let lowered = Self::lower_list_constructs(piece);
                        if !lowered.is_empty() {
                            result.push(format!("    {}", lowered));
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    /// Plan 400 Phase 2: read-only access to accumulated uses.
    pub fn uses_ref(&self) -> &HashSet<AutoStr> {
        &self.uses
    }

    /// Mutable access to the fn_ret_types cache (for cross-module / sibling
    /// pre-population from the CLI single-file path).
    pub fn fn_ret_types_mut(&mut self) -> &mut HashMap<AutoStr, Type> {
        &mut self.fn_ret_types
    }

    /// Mutable access to the spec_decls cache (plan 371 defect A: lets callers
    /// pre-populate spec names from sibling files so cross-module specs resolve
    /// to Type::Spec / Box<dyn X> on the single-file transpile path).
    pub fn spec_decls_mut(&mut self) -> &mut HashMap<AutoStr, Vec<SpecMethod>> {
        &mut self.spec_decls
    }

    /// Plan 373: Resolve the return type of `object.method()` and check if it
    /// needs `.await`. Looks up `fn_ret_types` by qualified key ("Type.method")
    /// or bare method name.
    pub(super) fn call_needs_await(&self, object: &Expr, method_name: &AutoStr) -> bool {
        // Try bare name first (common for same-file calls).
        if let Some(ret) = self.fn_ret_types.get(method_name.as_str()) {
            if Self::type_is_async(ret) {
                return true;
            }
        }
        // Try qualified "Type.method" for calls on self.field.
        let type_name = match object {
            Expr::Dot(inner, field) => {
                // self.field.method() — resolve field's type.
                if let Expr::Ident(name) = inner.as_ref() {
                    if name.as_str() == "self" {
                        self.local_var_types
                            .get(field.as_str())
                            .and_then(Self::type_name_of)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(tn) = type_name {
            let qualified = format!("{}.{}", tn, method_name);
            if let Some(ret) = self.fn_ret_types.get(qualified.as_str()) {
                return Self::type_is_async(ret);
            }
        }
        false
    }

    pub(super) fn output_call(
        &mut self,
        call: &Call,
        out: &mut impl Write,
        newline: bool,
    ) -> AutoResult<()> {
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
                        let fmt = if self.needs_debug_format(expr) {
                            "{:?}"
                        } else {
                            "{}"
                        };
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
                        format_string.push_str(if self.needs_debug_format(e) {
                            " {:?}"
                        } else {
                            " {}"
                        });
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

    /// PLAN-714 r3 R3-T3: try 体是否含 return（递归 If/For/Block）。
    fn try_body_has_return(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|st| match st {
            Stmt::Return(_) => true,
            Stmt::If(if_) => {
                if_.branches
                    .iter()
                    .any(|b| Self::try_body_has_return(&b.body.stmts))
                    || if_
                        .else_
                        .as_ref()
                        .is_some_and(|b| Self::try_body_has_return(&b.stmts))
            }
            Stmt::For(f) => Self::try_body_has_return(&f.body.stmts),
            Stmt::Block(b) => Self::try_body_has_return(&b.stmts),
            _ => false,
        })
    }

    /// PLAN-714 r3 R3-T3: try 体 Return(e) → Return(Some(e)) 改写（递归）。
    fn rewrite_try_returns(stmts: &mut [Stmt]) {
        for st in stmts.iter_mut() {
            match st {
                Stmt::Return(expr) => {
                    let inner = expr.as_ref().clone();
                    *expr = Box::new(Expr::Call(crate::ast::Call {
                        name: Box::new(Expr::Ident("Some".into())),
                        args: crate::ast::Args {
                            args: vec![crate::ast::Arg::Pos(inner)],
                        },
                        ret: Type::Unknown,
                        type_args: Vec::new(),
                        generic_args: Vec::new(),
                        pos: None,
                    }));
                }
                Stmt::If(if_) => {
                    for b in if_.branches.iter_mut() {
                        Self::rewrite_try_returns(&mut b.body.stmts);
                    }
                    if let Some(else_body) = if_.else_.as_mut() {
                        Self::rewrite_try_returns(&mut else_body.stmts);
                    }
                }
                Stmt::For(f) => Self::rewrite_try_returns(&mut f.body.stmts),
                Stmt::Block(b) => Self::rewrite_try_returns(&mut b.stmts),
                _ => {}
            }
        }
    }

    pub(super) fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<bool> {
        match stmt {
            Stmt::Expr(expr) => {
                self.expr(expr, &mut sink.body)?;
                // No semicolon for expressions in expression position
                // (handled by body() method)
                Ok(true)
            }

            // Plan 364 Phase 8 F2: Stmt::Block — a bare `{ ... }` block.
            // Previously hit `_ => Err(...)`. Emit a Rust block and recurse
            // into the inner statements via the same stmt() entry, mirroring
            // emit_loop_body's delegation pattern.
            Stmt::Block(body) => {
                sink.body.write(b"{\n")?;
                self.indent();
                for inner in &body.stmts {
                    self.print_indent(&mut sink.body)?;
                    self.stmt(inner, sink)?;
                    if matches!(inner, Stmt::Expr(_)) {
                        sink.body.write(b";")?;
                    }
                    sink.body.write(b"\n")?;
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
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
                // Statement-position `is` is a `match` block; if its arms
                // end in a value-bearing expression (e.g. map.insert(...)
                // returning Option), the match's type isn't `()` → E0308.
                // A trailing `;` discards the value, making it a unit stmt.
                sink.body.write(b";")?;
                Ok(true)
            }

            Stmt::Use(use_stmt) => {
                self.use_stmt(use_stmt, &mut sink.body)?;
                Ok(true)
            }

            Stmt::TypeDecl(type_decl) => {
                // Plan 417-E2 followup (F1): install this type's associated-
                // type bindings for the whole emission — method signatures,
                // bodies and field types referencing the assoc name render as
                // the bound type. Saved/restored around the call so an early
                // return inside type_decl() can't leak the table.
                let saved_bindings = std::mem::take(&mut self.current_assoc_bindings);
                for spec_impl in &type_decl.spec_impls {
                    for (assoc_name, bound_ty) in &spec_impl.assoc_bindings {
                        self.current_assoc_bindings
                            .insert(assoc_name.as_str().into(), bound_ty.clone());
                    }
                }
                let result = self.type_decl(type_decl, sink);
                self.current_assoc_bindings = saved_bindings;
                result?;
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
                // If this is a unit-return function (Void or no explicit type), emit plain return;
                // Auto void functions may return 0, Nil, None, or any expr — all become return;
                let is_unit_fn = self
                    .current_fn_ret_type
                    .as_ref()
                    .map(|t| matches!(t, Type::Void | Type::Unknown))
                    .unwrap_or(true);
                if is_unit_fn {
                    // Check if the return expression is trivially void-compatible
                    let is_void_expr = matches!(
                        expr.as_ref(),
                        Expr::Nil | Expr::None | Expr::Null | Expr::Bool(_)
                    );
                    if is_void_expr {
                        sink.body.write(b"return;")?;
                        return Ok(true);
                    }
                    // Int literals in return: keep as-is (e.g., "return 0;" in main)
                    // Only Nil/None/Null/Bool are truly void-compatible
                }
                sink.body.write(b"return ")?;
                // Plan 232: If returning a &str parameter, add .to_string()
                if let Expr::Ident(name) = expr.as_ref() {
                    if self.current_fn_str_params.contains(name) {
                        write!(sink.body, "{}.to_string()", name)?;
                        sink.body.write(b";")?;
                        return Ok(true);
                    }
                }
                // PLAN-010 T2 (009 ledger 008_arc_dyn_spec): `return
                // map.get(k)` in a fn returning Option<T> (owned, non-Copy
                // inner) yields Option<&T> — E0308. Clone the inner.
                if let Expr::Call(call) = expr.as_ref() {
                    if matches!(call.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "get")
                        && call.args.args.len() == 1
                    {
                        if let Some(Type::Option(inner)) = &self.current_fn_ret_type {
                            if !Self::is_primitive_copy(inner)
                                && !matches!(inner.as_ref(), Type::Unknown | Type::Void)
                            {
                                // stmt() has already written `return ` here.
                                self.expr(expr, &mut sink.body)?;
                                sink.body.write(b".cloned();")?;
                                return Ok(true);
                            }
                        }
                    }
                }
                // If return type is String and expr produces &str, add .to_string()
                let needs_to_string =
                    self.ret_type_needs_string_coercion() && self.expr_needs_string_coercion(expr);
                // Plan 013 (B1/BUG2): returning `self.field` of an owned
                // non-Copy type from a &self method needs `.clone()`.
                let needs_self_clone = Self::is_self_dot(expr) && self.ret_type_is_owned_noncopy();
                // Plan 399 Phase 11.3: returning a global var (Lazy<Mutex<Vec<T>>>)
                // emits `*G.lock().unwrap()` — the Vec is borrowed from a temporary
                // guard. To return an owned Vec, emit `G.lock().unwrap().clone()`
                // (no `*`: Deref on the guard forwards .clone() to the Vec). Doing
                // it inline (skipping self.expr which prepends `*`) avoids the
                // `*guard.clone()` precedence trap (MutexGuard isn't Clone).
                let global_ret_name = match expr.as_ref() {
                    Expr::Ident(name)
                        if self.is_global_var(name) && self.ret_type_is_owned_noncopy() =>
                    {
                        Some(name.clone())
                    }
                    _ => None,
                };
                if let Some(name) = &global_ret_name {
                    let static_name = self.global_var_static_name(name);
                    write!(sink.body, "{}.lock().unwrap().clone()", static_name)?;
                    sink.body.write(b";")?;
                    return Ok(true);
                }
                // Plan 399 Phase 11.4: `return Some(iter_var)` where iter_var is
                // a borrowed loop variable — clone the inner to avoid &T vs T.
                if let Expr::Call(call) = expr.as_ref() {
                    if let Expr::Ident(callee) = call.name.as_ref() {
                        if callee.as_ref() == "Some" {
                            if let Some(crate::ast::Arg::Pos(inner)) = call.args.args.first() {
                                if let Expr::Ident(inner_name) = inner {
                                    if self.borrowed_iter_vars.contains(inner_name) {
                                        write!(sink.body, "return Some(")?;
                                        self.expr(inner, &mut sink.body)?;
                                        write!(sink.body, ".clone());")?;
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                }
                self.expr(expr, &mut sink.body)?;
                if needs_to_string {
                    sink.body.write(b".to_string()")?;
                }
                if needs_self_clone && !needs_to_string {
                    sink.body.write(b".clone()")?;
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

            // Plan 387: Auto actor `task Name { state; fn start/stop; on {...} }`
            Stmt::TaskDef(td) => {
                self.program_has_actors = true;
                self.task_decl(td, sink)?;
                Ok(true)
            }

            // PLAN-714 r2 R2-T1: try/catch statement arm — catch_unwind form,
            // mirror of the PLAN-710 G-B arm in ui_gen/rust.rs (33a5d56c3).
            // Unlocks the back-module path (api_gen::transpile_back_module_to_rs):
            // corpus fsys.at find-in-files `try` previously fell into the
            // `_ => Err(unsupported statement)` arm below, skipping the whole
            // module (api_impl E0432) and cascading into the front route-A
            // `.ok()?` stub form. Body/catch/finally recurse through this same
            // stmt() entry (Stmt::Block delegation pattern — Expr members get
            // `;` appended here, the same discipline as Block). AssertUnwindSafe
            // carries &mut writes to outer locals (outer `var` emits `let mut`);
            // `catch (e)` binds the panic payload message via
            // auto_lang::a2r_std::panic_message (VM catch-frame STORE_LOCAL
            // same-source — vm/codegen.rs Stmt::Try); unbound empty catch
            // discards the payload via a `_`-prefixed binding (no unused
            // warning; corpus fsys.at:208 shape). finally follows the match
            // (VM same-shape deviation: an error raised inside the catch body
            // propagates before finally — neither side nests handlers).
            // Boundary inherited from 710 §2: return/break/continue inside the
            // try body stay closure-local (semantic change if they targeted an
            // enclosing loop outside the try — corpus zero instances;
            // break/continue targeting loops inside the body are unaffected).
            Stmt::Try(t) => {
                // r3 R3-T3: 闭包内 return 传播形——fn 非 unit 且体含 return
                // 时，闭包标注 `-> Option<T>`、体 Return 包 Some、Ok(Some(v))
                // => return v 传播（VM return-in-try=从 fn 返回同源）；Ok(None)
                // => return Default::default()（体必 return 的 corpus 形下不可
                // 达；条件 return 形=242 边界注记）。体预变换：借位迭代变量
                // 的 Ident 绑定/赋值改 .clone()（E0308 &String→String 株）。
                let ret_nonunit = !matches!(
                    self.current_fn_ret_type.as_ref(),
                    None | Some(Type::Void) | Some(Type::Unknown)
                );
                let propagate = ret_nonunit && Self::try_body_has_return(&t.body.stmts);
                let mut body = t.body.clone();
                if propagate {
                    Self::rewrite_try_returns(&mut body.stmts);
                }
                let try_body = body;
                if propagate {
                    let ret_ty = self
                        .rust_type_name(self.current_fn_ret_type.as_ref().expect("checked above"));
                    write!(sink.body, "match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Option<{ret_ty}> {{\n")?;
                } else {
                    sink.body.write(
                        b"match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {\n",
                    )?;
                }
                self.indent();
                for inner in &try_body.stmts {
                    self.print_indent(&mut sink.body)?;
                    self.stmt(inner, sink)?;
                    if matches!(inner, Stmt::Expr(_)) {
                        sink.body.write(b";")?;
                    }
                    sink.body.write(b"\n")?;
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}))")?;
                if propagate {
                    sink.body.write(
                        b" { Ok(Some(__v)) => return __v, Ok(None) => return Default::default(), ",
                    )?;
                } else {
                    sink.body.write(b" { Ok(_) => {}, ")?;
                }
                if t.catch_param.is_some() || !t.catch_body.stmts.is_empty() {
                    sink.body.write(b"Err(__p) => ")?;
                    if t.catch_body.stmts.is_empty() {
                        // Bound but empty handler: keep the payload accessible
                        // under its .at name (let it warn-free via usage in
                        // consumer code; empty-body binding is a rare shape).
                        if let Some(p) = &t.catch_param {
                            write!(
                                sink.body,
                                "{{ let {p} = auto_lang::a2r_std::panic_message(&__p); }}"
                            )?;
                        } else {
                            sink.body.write(b"{}")?;
                        }
                    } else {
                        sink.body.write(b"{\n")?;
                        self.indent();
                        if let Some(p) = &t.catch_param {
                            self.print_indent(&mut sink.body)?;
                            write!(
                                sink.body,
                                "let {p} = auto_lang::a2r_std::panic_message(&__p);"
                            )?;
                            sink.body.write(b"\n")?;
                        }
                        for inner in &t.catch_body.stmts {
                            self.print_indent(&mut sink.body)?;
                            self.stmt(inner, sink)?;
                            if matches!(inner, Stmt::Expr(_)) {
                                sink.body.write(b";")?;
                            }
                            sink.body.write(b"\n")?;
                        }
                        self.dedent();
                        self.print_indent(&mut sink.body)?;
                        sink.body.write(b"}")?;
                    }
                    sink.body.write(b" }")?;
                } else if propagate {
                    // Empty unbound catch under propagation: r2 swallow
                    // semantics preserved（match 臂 ()——中位安全；尾位
                    // 空 catch+传播形=242 边界注记）。
                    sink.body.write(b"Err(_p714) => {} }")?;
                } else {
                    // Empty unbound catch (corpus fsys.at:208): discard the
                    // payload; `_`-prefixed name dodges the unused warning.
                    sink.body.write(b"Err(_p714) => {} }")?;
                }
                if let Some(fb) = &t.finally_body {
                    sink.body.write(b"; {\n")?;
                    self.indent();
                    for inner in &fb.stmts {
                        self.print_indent(&mut sink.body)?;
                        self.stmt(inner, sink)?;
                        if matches!(inner, Stmt::Expr(_)) {
                            sink.body.write(b";")?;
                        }
                        sink.body.write(b"\n")?;
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
                Ok(true)
            }

            _ => Err(format!("Rust Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    // Plan 387: translate an Auto actor `task Name { ... }` into Rust.
    // Emits: a state struct, an impl block (new/start/stop/handle_msg), and a
    // spawn helper. See Plan 387 §12.2 for the frozen template.
    fn task_decl(&mut self, td: &TaskDef, sink: &mut Sink) -> AutoResult<()> {
        let name = td.name.as_str();
        self.a2r_std_used.set(true);

        // D2 (Plan 387): derive the message type from the `on` patterns.
        // Returns (type_name, optional enum source) — the enum is emitted before
        // the struct when named variants are present.
        let (msg_type, msg_enum_src) = self.derive_task_msg_type(&td.name, &td.on_block)?;

        // Record this task's state-field names so compile_task_body can rewrite
        // bare identifiers to `self.<field>`.
        let prev_state = self.task_state_fields.clone();
        self.task_state_fields
            .extend(td.state.iter().map(|(n, _, _)| n.as_str().to_string()));

        if let Some(enum_src) = &msg_enum_src {
            sink.body.write(enum_src.as_bytes())?;
            // Register this task's variant names (per-task, so two tasks may
            // declare the same variant — call() disambiguates by receiver task).
            let task_name = name_of(&td.name).to_string();
            let variants = self.task_variants.entry(task_name).or_default();
            for (pattern, _, _) in &td.on_block.handlers {
                use crate::ast::TaskMsgPattern as P;
                let vname: Option<&crate::ast::Name> = match pattern {
                    P::Simple(n) => Some(n),
                    P::WithBindings { variant, .. } => Some(variant),
                    _ => None,
                };
                if let Some(vn) = vname {
                    variants.insert(vn.as_str().to_string());
                }
            }
        }
        self.emit_task_struct(td, sink)?;
        self.emit_task_impl(td, sink, &msg_type)?;
        self.emit_task_spawn_helper(td, sink, &msg_type)?;

        self.task_state_fields = prev_state;
        Ok(())
    }

    /// Emit `struct Name { state_fields }`.
    fn emit_task_struct(&mut self, td: &TaskDef, sink: &mut Sink) -> AutoResult<()> {
        // Plan 395-followup: a task with a Box<dyn Fn(...)> state field (closure
        // default) can't derive Clone/Debug — `dyn Fn` implements neither. The
        // actor is moved into the spawned task (no Clone/Debug needed), so emit
        // no derive for such structs.
        let has_closure_field = td
            .state
            .iter()
            .any(|(_f, _m, init)| matches!(init, Expr::Closure(_)));
        if !has_closure_field {
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "#[derive(Clone, Debug)]")?;
        }
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "struct {} {{", name_of(&td.name))?;
        self.indent();
        for (field, _mutable, init) in &td.state {
            // Plan 390 §15.10: a closure-literal default (`cb = fn(e) {...}`)
            // is a Box<dyn Fn(...)> field (closures capture — not fn-pointers).
            let ty_str = if let Expr::Closure(closure) = init {
                self.closure_field_rust_type(closure)
            } else {
                let ty = self.infer_type_from_expr(init);
                // Plan 387: task state integer fields use i64 (matches the VM's
                // i64-backed actor state and the default i64 message-binding type).
                match &ty {
                    crate::ast::Type::Int => "i64".to_string(),
                    _ => self.rust_type_name(&ty),
                }
            };
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "{}: {},", field.as_str(), ty_str)?;
        }
        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n\n")?;
        Ok(())
    }

    /// Emit `impl Name { fn new() -> Self; async fn start; async fn stop; async fn handle_msg }`.
    fn emit_task_impl(&mut self, td: &TaskDef, sink: &mut Sink, msg_type: &str) -> AutoResult<()> {
        let name = name_of(&td.name);

        // new() — initialize state fields from their `= init` expressions.
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "impl {} {{", name)?;
        self.indent();

        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "pub fn new() -> Self {{")?;
        self.indent();
        if td.state.is_empty() {
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "Self {{}}")?;
        } else {
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "Self {{")?;
            self.indent();
            for (field, _mutable, init) in &td.state {
                self.print_indent(&mut sink.body)?;
                sink.body.write(field.as_str().as_bytes())?;
                sink.body.write(b": ")?;
                // Plan 390 §15.10: closure defaults are boxed closures —
                // `Box::new(move |..| ..)` (move = own captures for 'static).
                let is_closure = matches!(init, Expr::Closure(_));
                if is_closure {
                    sink.body.write(b"Box::new(move ")?;
                }
                self.expr(init, &mut sink.body)?;
                if is_closure {
                    sink.body.write(b")")?;
                }
                sink.body.write(b",\n")?;
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }
        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n\n")?;

        // start hook — runs once at spawn, before the message loop.
        self.emit_task_hook(&td.name, td.start_hook.as_ref(), "start", sink)?;

        // stop hook — runs after the mailbox closes (Tier 2 wiring; emit stub for Tier 1).
        self.emit_task_hook(&td.name, td.stop_hook.as_ref(), "stop", sink)?;

        // handle_msg — the message dispatcher.
        self.emit_task_handle_msg(td, sink, msg_type)?;

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n\n")?;
        Ok(())
    }

    /// Emit a start/stop hook as `async fn <name>(&mut self) -> Result<(), ...>`.
    /// If the hook is absent, emit an empty stub so the spawn helper can always call it.
    fn emit_task_hook(
        &mut self,
        _task_name: &str,
        hook: Option<&Fn>,
        hook_name: &str,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        self.print_indent(&mut sink.body)?;
        writeln!(
            sink.body,
            "pub async fn {}(&mut self) -> Result<(), Box<dyn std::error::Error>> {{",
            hook_name
        )?;
        self.indent();
        match hook {
            Some(f) => {
                // Compile the hook body in a `self`-method context so bare state
                // identifiers resolve to `self.<field>` (Plan 387 compile_task_body).
                self.compile_task_body(&f.body, sink, true)?;
                // Append Ok(()) unless the body already ends in a tail return/expr.
                // (body() does this for fn_decl, but hooks compile via compile_task_body,
                // so we mirror body()'s tail logic here.)
                let needs_ok = f.body.stmts.is_empty()
                    || !f
                        .body
                        .stmts
                        .last()
                        .map(|s| self.is_returnable(s))
                        .unwrap_or(false);
                if needs_ok {
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"Ok(())\n")?;
                }
            }
            None => {
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"Ok(())\n")?;
            }
        }
        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n\n")?;
        Ok(())
    }

    /// Emit `async fn handle_msg(&mut self, msg: M, reply_tx: NopReply) -> Result<...>`
    /// whose body is a `match msg { <pat> => <body>, ... _ => <else> }`.
    fn emit_task_handle_msg(
        &mut self,
        td: &TaskDef,
        sink: &mut Sink,
        msg_type: &str,
    ) -> AutoResult<()> {
        self.print_indent(&mut sink.body)?;
        writeln!(
            sink.body,
            "pub async fn handle_msg(&mut self, msg: {}, reply_tx: a2r_std::task::NopReply) \
             -> Result<(), Box<dyn std::error::Error>> {{",
            msg_type
        )?;
        self.indent();

        // handler bodies may reference bare state-field names that must resolve to
        // `self.<field>`; compile_task_body toggles in_task_body for that rewrite.
        let has_state = !td.state.is_empty();
        // If msg_type is the generated enum "<Task>Msg", patterns must qualify
        // variants with the enum name (e.g. `CounterMsg::Add(v)`).
        let enum_name: Option<&str> = if msg_type == &format!("{}Msg", name_of(&td.name)) {
            Some(msg_type)
        } else {
            None
        };
        // String messages match by reference (`msg.as_str()`) so literal patterns
        // are plain `"ping"` (not `"ping".to_string()`, which would move `msg`
        // out from under the `_` arm). i64/bool/enum match the value directly.
        let is_string_msg = msg_type == "String";
        let match_subject = if is_string_msg { "msg.as_str()" } else { "msg" };

        if td.on_block.handlers.is_empty() && td.on_block.else_handler.is_none() {
            // No handlers at all — empty match is unreachable, emit a no-op.
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "let _ = msg;")?;
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "let _ = &reply_tx;")?;
        } else {
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "match {} {{", match_subject)?;
            self.indent();
            for (pattern, guard, body) in &td.on_block.handlers {
                self.print_indent(&mut sink.body)?;
                self.emit_task_pattern(pattern, enum_name, &mut sink.body)?;
                // Plan 387 follow-up P5: wire guards (`Add(val) if val > 1 ->`)
                // as Rust match-arm guards. The guard may reference state fields
                // and the pattern bindings, so compile it with the same
                // in_task_body state rewrite as handler bodies.
                if let Some(g) = guard {
                    let saved = self.in_task_body;
                    self.in_task_body = has_state;
                    write!(sink.body, " if ")?;
                    self.expr(g, &mut sink.body)?;
                    self.in_task_body = saved;
                }
                sink.body.write(b" => {\n")?;
                self.indent();
                self.compile_task_body(body, sink, has_state)?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}\n")?;
            }
            // else arm (or empty wildcard if no explicit else)
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"_ => {\n")?;
            self.indent();
            if let Some(else_body) = &td.on_block.else_handler {
                self.compile_task_body(else_body, sink, has_state)?;
            } else {
                self.print_indent(&mut sink.body)?;
                writeln!(sink.body, "let _ = msg;")?;
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
            self.dedent();
            self.print_indent(&mut sink.body)?;
            // Close `match msg { ... }` as a statement (semicolon) so the trailing
            // `Ok(())` is a separate expression.
            sink.body.write(b"};\n")?;
        }

        // handle_msg always succeeds (errors inside bodies propagate via `?` to here).
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"Ok(())\n")?;
        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n")?;

        Ok(())
    }

    /// Emit a single message pattern as a `match` arm LHS.
    /// `enum_name` is Some when the message type is a generated enum (named
    /// variants present); in that case literal patterns wrap into
    /// `EnumName::Literal(value)`. Named patterns emit `EnumName::Variant(bindings)`.
    fn emit_task_pattern(
        &self,
        p: &TaskMsgPattern,
        enum_name: Option<&str>,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        use TaskMsgPattern as P;
        match p {
            P::Literal(LiteralValue::Int(n)) => {
                if let Some(en) = enum_name {
                    write!(out, "{}::Literal({}i64)", en, n)?;
                } else {
                    write!(out, "{}i64", n)?;
                }
                Ok(())
            }
            P::Literal(LiteralValue::String(s)) => {
                if let Some(en) = enum_name {
                    write!(out, "{}::Literal({:?}.to_string())", en, s.as_str())?;
                } else {
                    // In scalar String context the match subject is `msg.as_str()`,
                    // so the pattern is a plain &str literal.
                    write!(out, "{:?}", s.as_str())?;
                }
                Ok(())
            }
            P::Literal(LiteralValue::Bool(b)) => {
                if let Some(en) = enum_name {
                    write!(out, "{}::Literal({})", en, b)?;
                } else {
                    write!(out, "{}", b)?;
                }
                Ok(())
            }
            P::Simple(vname) => {
                if let Some(en) = enum_name {
                    write!(out, "{}::{}", en, vname.as_str())?;
                    Ok(())
                } else {
                    Err(format!(
                        "Rust Transpiler (Plan 387): simple variant pattern `{}` requires an enum message type",
                        vname.as_str()
                    )
                    .into())
                }
            }
            P::WithBindings { variant, bindings } => {
                if let Some(en) = enum_name {
                    write!(out, "{}::{}(", en, variant.as_str())?;
                    for (i, (b, _bty)) in bindings.iter().enumerate() {
                        write!(out, "{}", b.as_str())?;
                        if i < bindings.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, ")")?;
                    Ok(())
                } else {
                    Err(format!(
                        "Rust Transpiler (Plan 387): variant pattern `{}` requires an enum message type",
                        variant.as_str()
                    )
                    .into())
                }
            }
            P::TypeBinding { name, .. } => {
                // TypeBinding matches any value of a type; emit as a binding
                // (the msg itself). Only valid in scalar (non-enum) context.
                if enum_name.is_none() {
                    write!(out, "{}", name.as_str())?;
                    Ok(())
                } else {
                    Err(format!(
                        "Rust Transpiler (Plan 387): TypeBinding pattern in enum context not yet supported"
                    )
                    .into())
                }
            }
            other => Err(format!(
                "Rust Transpiler (Plan 387): unsupported message pattern {:?}",
                other
            )
            .into()),
        }
    }

    /// Emit the per-task spawn helper `fn spawn_<name>(...) -> TaskRef<M>` (§16: no
    /// __rt parameter — any function can spawn, not just main).
    /// Plan 390 §5 Phase B (M1): the helper takes the task's state fields as
    /// positional parameters (so `Task.spawn("Name", cap, v1, v2)` →
    /// `spawn_<name>(v1, v2)` constructs the actor with those values instead of
    /// the defaults). Backward compatible: a no-init spawn still works because
    /// every param has a default (the state field's declared initializer).
    fn emit_task_spawn_helper(
        &mut self,
        td: &TaskDef,
        sink: &mut Sink,
        msg_type: &str,
    ) -> AutoResult<()> {
        let name = name_of(&td.name);
        let has_stop = td.stop_hook.is_some();
        // Plan 390 §5 Phase B (M1): for tasks with state fields, emit TWO helpers:
        //   spawn_<name>()            — no args, uses Name::new() (declared defaults)
        //   spawn_<name>_with(f1, f2) — required state-field args (no defaults;
        //                               Rust doesn't support default fn params)
        // Task.spawn call sites pick _with only when init args are present.
        // For tasks with NO state fields, emit a single spawn_<name>() (as before).
        let snake = snake_of(&td.name);
        if !td.state.is_empty() {
            // Build the _with parameter list (no defaults — Rust doesn't allow them).
            let mut params_str = String::new();
            let mut construct_fields = String::new();
            for (field, _mutable, init) in &td.state {
                if !params_str.is_empty() {
                    params_str.push_str(", ");
                }
                let field_str = field.to_string();
                // Plan 390 §15.10: closure-literal fields take Box<dyn Fn(...)>
                // (same as the struct field type) instead of `/* unknown */`.
                let ty_name = if let Expr::Closure(closure) = init {
                    self.closure_field_rust_type(closure)
                } else {
                    let ty = self.infer_type_from_expr(init);
                    self.rust_type_name(&ty)
                };
                params_str.push_str(&field_str);
                params_str.push_str(": ");
                params_str.push_str(&ty_name);
                if !construct_fields.is_empty() {
                    construct_fields.push_str(", ");
                }
                construct_fields.push_str(&field_str);
                construct_fields.push_str(": ");
                construct_fields.push_str(&field_str);
            }
            // spawn_<name>_with — takes all state fields as required params.
            self.emit_spawn_body(
                &snake,
                "_with",
                Some(&params_str),
                &name,
                &format!("{} {{ {} }}", name, construct_fields),
                msg_type,
                has_stop,
                sink,
            )?;
        }
        // spawn_<name> — no args, default-constructs via ::new() (always emitted;
        // matches the no-init Task.spawn call site).
        self.emit_spawn_body(
            &snake,
            "",
            None,
            &name,
            &format!("{}::new()", name),
            msg_type,
            has_stop,
            sink,
        )?;
        Ok(())
    }

    /// Emit one spawn helper variant. `suffix` is "" or "_with"; `params` is the
    /// parameter list (None = no params); `ctor_expr` is the actor construction
    /// expression (`Name::new()` or `Name { f1: f1, ... }`).
    fn emit_spawn_body(
        &mut self,
        snake: &str,
        suffix: &str,
        params: Option<&str>,
        name: &str,
        ctor_expr: &str,
        msg_type: &str,
        has_stop: bool,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let params_str = params.unwrap_or("");
        self.print_indent(&mut sink.body)?;
        writeln!(
            sink.body,
            "pub fn spawn_{}{}({}) -> a2r_std::task::TaskRef<{}> {{",
            snake, suffix, params_str, msg_type
        )?;
        self.indent();
        self.print_indent(&mut sink.body)?;
        // Plan 387 archive fix: `a2r_std::task::channel` pairs the TaskRef with a
        // shared in-flight counter and the actor-side receiver. The loop calls
        // `rx.mark_processed()` after each message so `drain_all` (called at the
        // end of generated main) can wait until every sent message is fully
        // handled — the old fixed-16-yield drain silently lost messages when a
        // handler awaited internally.
        writeln!(
            sink.body,
            "let (taskref, mut rx) = a2r_std::task::channel::<{}>();",
            msg_type
        )?;
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "let join = tokio::spawn(async move {{")?;
        self.indent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "let mut actor = {};", ctor_expr)?;
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "let _ = actor.start().await;")?;
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "while let Some(msg) = rx.recv().await {{")?;
        self.indent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "let reply_tx = a2r_std::task::NopReply;")?;
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "let _ = actor.handle_msg(msg, reply_tx).await;")?;
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "rx.mark_processed();")?;
        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;
        if has_stop {
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "let _ = actor.stop().await;")?;
        }
        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}});")?;
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "a2r_std::task::track_join(join);")?;
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "taskref")?;
        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n\n")?;
        Ok(())
    }

    /// Plan 387 W4 + follow-up: if `arg` is a task message-variant name (bare
    /// ident like `Reset`) or a variant constructor (`Add(5)`), return its
    /// fully-qualified Rust form (`CounterMsg::Reset` / `CounterMsg::Add(5)`).
    ///
    /// The enum is chosen from the RECEIVER's task when known (`receiver_task`
    /// — e.g. the handle variable's task in `h.send(Add(5))`), which
    /// disambiguates two tasks that declare the same variant name. When the
    /// receiver task is unknown, a variant declared by exactly ONE task still
    /// resolves (backward compat); a variant declared by several falls back to
    /// None (left unrewritten → the compile error names the ambiguity).
    pub(super) fn rewrite_msg_variant_arg(
        &mut self,
        arg: &Expr,
        receiver_task: Option<&str>,
    ) -> Option<String> {
        // Resolve variant → (task, enum_name), preferring the receiver's task.
        fn enum_for<'a>(
            task_variants: &'a std::collections::HashMap<String, std::collections::HashSet<String>>,
            variant: &str,
            receiver_task: Option<&str>,
        ) -> Option<String> {
            if let Some(task) = receiver_task {
                if task_variants
                    .get(task)
                    .map(|vs| vs.contains(variant))
                    .unwrap_or(false)
                {
                    return Some(format!("{}Msg", task));
                }
            }
            // Unknown receiver: resolve only if a single task declares it.
            let owners: Vec<&String> = task_variants
                .iter()
                .filter(|(_, vs)| vs.contains(variant))
                .map(|(t, _)| t)
                .collect();
            match owners.as_slice() {
                [task] => Some(format!("{}Msg", task)),
                _ => None,
            }
        }

        match arg {
            // Reset → EnumName::Reset
            Expr::Ident(name) => enum_for(&self.task_variants, name.as_str(), receiver_task)
                .map(|enum_name| format!("{}::{}", enum_name, name.as_str())),
            // Add(5) → EnumName::Add(5); the call name is an Ident equal to a variant.
            Expr::Call(c) => {
                let variant_name = match c.name.as_ref() {
                    Expr::Ident(n) => Some(n.as_str().to_string()),
                    _ => None,
                }?;
                let enum_name = enum_for(&self.task_variants, &variant_name, receiver_task)?;
                // Render the args by writing to a string buffer via self.expr.
                let mut buf: Vec<u8> = Vec::new();
                for (i, a) in c.args.args.iter().enumerate() {
                    if let Arg::Pos(e) = a {
                        // Recursively rewrite nested variants + render.
                        if let Some(rw) = self.rewrite_msg_variant_arg(e, receiver_task) {
                            use std::io::Write;
                            let _ = write!(buf, "{}", rw);
                        } else {
                            // Render normally into buf.
                            let mut sub: Vec<u8> = Vec::new();
                            let _ = self.expr(e, &mut sub);
                            buf.extend_from_slice(&sub);
                            // Plan 387 follow-up P3: a String-typed variant payload
                            // (`Greet(name: String)`) needs owned String — a bare
                            // string-literal arg (`Greet("bob")`) must become
                            // `.to_string()` (mirrors the bare `h.send("...")`
                            // path for String-message tasks).
                            if matches!(e, Expr::Str(_) | Expr::CStr(_)) {
                                buf.extend_from_slice(b".to_string()");
                            }
                        }
                    }
                    if i < c.args.args.len() - 1 {
                        buf.extend_from_slice(b", ");
                    }
                }
                Some(format!(
                    "{}::{}({})",
                    enum_name,
                    variant_name,
                    String::from_utf8_lossy(&buf)
                ))
            }
            _ => None,
        }
    }

    // Plan 387 helper: compile a task hook/handler body. `rewrite_self` is true
    // when the body may reference bare state-field names that must become `self.<field>`.
    fn compile_task_body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
        rewrite_self: bool,
    ) -> AutoResult<()> {
        // For Tier 1, state fields are integer scalars; the existing stmt() path
        // already handles `count = count + 1` as a Store with the field name. We
        // need bare reads/writes of state-field names to target `self.<field>`.
        // The cleanest approach: temporarily register the state fields as locals
        // isn't enough (they live on `self`). Instead we walk statements and, for
        // identifiers that are state fields, prefix with `self.`. This is done
        // inline below for the common cases (Store name, Expr::Ident read).
        let saved = self.in_task_body;
        self.in_task_body = rewrite_self;
        for stmt in &body.stmts {
            self.print_indent(&mut sink.body)?;
            self.stmt(stmt, sink)?;
            // Mirror body()'s per-statement formatting: Expr/Store need a trailing
            // semicolon+newline; other statement types handle their own terminator.
            match stmt {
                Stmt::Expr(_) => {
                    sink.body.write(b";\n")?;
                }
                Stmt::Store(_) => {
                    sink.body.write(b";\n")?;
                }
                _ => {
                    sink.body.write(b"\n")?;
                }
            }
        }
        self.in_task_body = saved;
        Ok(())
    }

    /// Plan 391 D1: detect a `.len()` / `.length()` method call expression (in
    /// either AST form: `Expr::Call{ name: Expr::Dot(_, m) }` or the legacy
    /// `Expr::Call{ name: Expr::Bina(_, Dot, m) }`). Used by store()/assignment
    /// to decide whether to set `len_i32_cast_suppressed` for a wide-typed binding.
    /// Plan 396/B11(a): the `.len()` cast target when compared against `other`.
    /// Rust len() is usize; the default emission casts it to i64 (Auto's int),
    /// but comparing against a u32 field/var then mismatches (E0308). Resolve
    /// the partner's type (Ident via local_var_types; `self.field` via a
    /// unique field-name scan across known structs) and cast len() to MATCH.
    /// Returns None when unresolvable (caller keeps the default i64 cast).
    pub(super) fn partner_len_cast(&self, other: &Expr) -> Option<&'static str> {
        let ty = match other {
            Expr::Ident(n) => self.local_var_types.get(n.as_str()).cloned(),
            Expr::Dot(_, f) => {
                let mut hit: Option<Type> = None;
                for fields in self.struct_field_types.values() {
                    for (n, t) in fields {
                        if n.as_str() == f.as_str() {
                            if hit.is_some() {
                                return None; // ambiguous field name
                            }
                            hit = Some(t.clone());
                        }
                    }
                }
                hit
            }
            _ => None,
        }?;
        Some(match ty {
            Type::Int | Type::I64 => " as i64",
            Type::Uint => " as u32",
            Type::USize => " as usize",
            Type::U64 => " as u64",
            _ => return None,
        })
    }

    // Variable declaration
    pub(super) fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // P738-R5-02：json 值生产绑定的 def-use 跟踪（receiver 方法闭包
        // 判据；不影响发射——与 json_value_vars 的历史用途分开）。
        if matches!(store.kind, StoreKind::Let | StoreKind::Var) {
            if let Expr::Call(call) = &store.expr {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if let Expr::Ident(owner) = obj.as_ref() {
                        if matches!(owner.as_str(), "json" | "Json")
                            && matches!(
                                method.as_str(),
                                "parse" | "get" | "get_at" | "get_str" | "get_u64"
                            )
                        {
                            self.json_value_bindings.insert(store.name.clone());
                        }
                    }
                }
            }
        }
        // PLAN-714 r3 R3-T3: 借位迭代变量的声明绑定 → `.clone()`（窄门：
        // Var/Let 声明 + 声明型 String 族 + init=裸 Ident∈borrowed_iter_
        // vars——E0308 &String→String 株；型门压住 1609 注记的 is-arm
        // 假阳性顾虑——非 String 型不进此门，Clone 语义等价）。
        if matches!(store.kind, StoreKind::Var | StoreKind::Let)
            && matches!(
                store.ty,
                Type::StrOwned | Type::StrFixed(_) | Type::StrSlice | Type::CStrLit
            )
            && matches!(
                &store.expr,
                Expr::Ident(n) if self.borrowed_iter_vars.contains(n)
            )
        {
            let mut patched = store.clone();
            if let Expr::Ident(n) = &patched.expr {
                let name = n.clone();
                patched.expr = Expr::Call(crate::ast::Call {
                    name: Box::new(Expr::Dot(Box::new(Expr::Ident(name)), "clone".into())),
                    args: crate::ast::Args::default(),
                    ret: Type::Unknown,
                    type_args: Vec::new(),
                    generic_args: Vec::new(),
                    pos: None,
                });
            }
            return self.store(&patched, out);
        }

        // Track local variable type for string concat detection
        // When type is Unknown, try to infer from the expression
        let effective_ty = if matches!(store.ty, Type::Unknown) {
            self.infer_type_from_expr(&store.expr)
        } else if matches!(&store.expr, Expr::NullCoalesce(_, _))
            && matches!(store.ty, Type::Option(_))
        {
            // Plan 019 Phase 1: the parser annotates `var x = a ?? b` with
            // the lhs Option type; the coalesce yields the INNER type —
            // infer instead (keeps later uses well-typed).
            let inner = self.infer_type_from_expr(&store.expr);
            if matches!(inner, Type::Unknown) {
                store.ty.clone()
            } else {
                inner
            }
        } else {
            store.ty.clone()
        };
        // PLAN-596 T-07 (DIV-DEP-8): an un-annotated let bound from a dep
        // ctor chain (`let u = Url.parse(..).unwrap()`) types as the dep
        // type, so downstream `.to(str)`/print resolve Display (three-way
        // parity) instead of staying Unknown.
        let effective_ty = if matches!(effective_ty, Type::Unknown) {
            match self.dep_ctor_type(&store.expr) {
                Some(t) => Self::placeholder_user_type(t),
                None => effective_ty,
            }
        } else {
            effective_ty
        };
        // PLAN-681 T-05（N2/P1a）：无注解 let 从 bare List 索引派生（含
        // .clone() 链）或 Value 局部字段读——元素是动态记录桶,登记
        // User("Value") 占位（后续字段读走 name["field"] 索引形,P2）。
        let effective_ty =
            if matches!(effective_ty, Type::Unknown) && self.expr_is_json_value_read(&store.expr) {
                Self::placeholder_user_type("Value".into())
            } else {
                effective_ty
            };
        // Plan 433 A1: a prior fn-call-inferred entry (scan_call_init_bindings
        // at fn_decl) is more precise than a fresh Unknown inference — keep it.
        let keep_existing = matches!(effective_ty, Type::Unknown)
            && self
                .local_var_types
                .get(store.name.as_str())
                .map(|t| !matches!(t, Type::Unknown))
                .unwrap_or(false);
        if !keep_existing {
            self.local_var_types
                .insert(store.name.clone(), effective_ty.clone());
        }
        // PLAN-724 T-06: HTTP 客户端变量分型登记——流生产者绑定（async/
        // sync 面按当前上下文定型，决定 .next()/.is_done()/.close()/for-in
        // 的发射形态）；http.request(...) 绑定（.send 分派）。
        {
            let producer = match &store.expr {
                Expr::Call(call) => match call.name.as_ref() {
                    Expr::Dot(obj, method) => {
                        matches!(obj.as_ref(), Expr::Ident(m) if m.as_str() == "http")
                            && matches!(
                                method.as_str(),
                                "get_stream" | "post_stream" | "post_stream_with_headers"
                            )
                    }
                    Expr::Ident(fname) => matches!(
                        fname.as_str(),
                        "get_stream" | "post_stream" | "post_stream_with_headers"
                    ),
                    _ => false,
                },
                _ => false,
            };
            if producer {
                if self.in_async_ctx.get() {
                    self.http_stream_async_vars.insert(store.name.clone());
                } else {
                    self.http_stream_vars.insert(store.name.clone());
                }
            }
            // builder：直接 http.request(...)，或方法链根仍是 builder 变量
            // / http.request(...)（`let b2 = b.header(..)` 重绑定保持分型）。
            let builder = match &store.expr {
                Expr::Call(call) => match call.name.as_ref() {
                    Expr::Dot(obj, method) => {
                        (matches!(obj.as_ref(), Expr::Ident(m) if m.as_str() == "http")
                            && method.as_str() == "request")
                            || (method.as_str() != "send" && expr_root_is_builder_var(self, obj))
                    }
                    Expr::Ident(fname) => fname.as_str() == "request",
                    _ => false,
                },
                _ => false,
            };
            if builder {
                self.http_builder_vars.insert(store.name.clone());
            }
        }

        // Plan 387 follow-up: record `let h = Task.spawn("Counter", cap)` so
        // `h.send(Variant)` can resolve the message enum from the receiver's
        // task (disambiguates same-named variants across tasks).
        if let Expr::Call(call) = &store.expr {
            if let Expr::Dot(obj, method) = call.name.as_ref() {
                if method.as_str() == "spawn" {
                    if let Expr::Ident(receiver) = obj.as_ref() {
                        if receiver.as_str() == "Task" {
                            if let Some(Arg::Pos(Expr::Str(name))) = call.args.args.first() {
                                let task_name_str: &str = name_of(name);
                                self.handle_task_map
                                    .insert(store.name.clone(), task_name_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Detect json.get() assignments and mark the variable as JSON value type
        // so that .to_int() and .len() use value_to_int/value_len helpers
        if !matches!(
            store.ty,
            Type::StrSlice
                | Type::StrOwned
                | Type::StrFixed(_)
                | Type::CStrLit
                | Type::List(_)
                | Type::Int
                | Type::Float
                | Type::Bool
        ) {
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
                self.var_spec_map
                    .insert(store.name.clone(), spec_name.clone());
            }
        }

        // Handle C variables and struct fields (should not be generated)
        match store.kind {
            StoreKind::CVar | StoreKind::Field => {
                return Ok(());
            }
            _ => {}
        }

        // Plan 415-A (242 #2): `var/let m Map<K,V> = {k: v, ...}` — an object
        // literal with a declared map type emits `HashMap::from([(K, V), ...])`.
        // The generic object path writes struct-literal syntax (`{k: v}`),
        // which is invalid Rust for maps (003_map_func emitted non-compiling
        // `{Accept: ...}`). Local Var/Let only — shared/const/global wrappers
        // keep their initializers unhandled (no known source combines them
        // with map literals today). Str keys/values wrap in `String::from`
        // (the mapped `HashMap<String, _>` needs owned); other key/value
        // expressions pass through the normal emitter.
        let map_kv_types: Option<(Type, Type)> = match &effective_ty {
            Type::Map(k, v) => Some(((**k).clone(), (**v).clone())),
            Type::GenericInstance(inst)
                if matches!(inst.base_name.as_str(), "Map" | "HashMap" | "BTreeMap")
                    && inst.args.len() >= 2 =>
            {
                Some((inst.args[0].clone(), inst.args[1].clone()))
            }
            _ => None,
        };
        if matches!(store.kind, StoreKind::Var | StoreKind::Let)
            && matches!(&store.expr, Expr::Object(_))
        {
            if let Some((k_ty, v_ty)) = map_kv_types {
                // Plan 577 ① (DEBTS 016 Phase 0): the EMPTY literal `{}` for a
                // declared Map type emits `HashMap::new()` — the annotation
                // carries the generics (bare `{}` is the unit type, E0308).
                if matches!(&store.expr, Expr::Object(pairs) if pairs.is_empty()) {
                    let mut_kw = if matches!(store.kind, StoreKind::Var)
                        || (matches!(store.kind, StoreKind::Let)
                            && self.mutated_let_bindings.contains(store.name.as_ref()))
                    {
                        "mut "
                    } else {
                        ""
                    };
                    let ty_name = self.rust_type_name(&store.ty);
                    write!(
                        out,
                        "let {}{}: {} = std::collections::HashMap::new()",
                        mut_kw,
                        Self::rust_ident(store.name.as_str()),
                        ty_name
                    )?;
                    return Ok(());
                }
                let k_is_str = matches!(
                    k_ty,
                    Type::StrOwned | Type::StrFixed(_) | Type::StrSlice | Type::CStrLit
                );
                let v_is_str = matches!(
                    v_ty,
                    Type::StrOwned | Type::StrFixed(_) | Type::StrSlice | Type::CStrLit
                );
                let mut_kw = if matches!(store.kind, StoreKind::Var)
                    || (matches!(store.kind, StoreKind::Let)
                        && self.mutated_let_bindings.contains(store.name.as_ref()))
                {
                    "mut "
                } else {
                    ""
                };
                let ty_name = self.rust_type_name(&store.ty);
                write!(
                    out,
                    "let {}{}: {} = std::collections::HashMap::from([",
                    mut_kw,
                    Self::rust_ident(store.name.as_str()),
                    ty_name
                )?;
                if let Expr::Object(pairs) = &store.expr {
                    for (i, pair) in pairs.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ")?;
                        }
                        write!(out, "(")?;
                        match (&pair.key, k_is_str) {
                            (crate::ast::Key::StrKey(s), true) => {
                                write!(out, "String::from(\"{}\")", escape_str(s.as_str()))?;
                            }
                            _ => {
                                self.expr(&Expr::from(pair.key.clone()), out)?;
                            }
                        }
                        write!(out, ", ")?;
                        match (pair.value.as_ref(), v_is_str) {
                            (Expr::Str(s), true) => {
                                write!(out, "String::from(\"{}\")", escape_str(s.as_str()))?;
                            }
                            _ => {
                                self.expr(&pair.value, out)?;
                            }
                        }
                        write!(out, ")")?;
                    }
                }
                write!(out, "])")?;
                return Ok(());
            }
        }

        // Plan 6B-4.19: shared var → static NAME: Lazy<Mutex<T>> = Lazy::new(|| Mutex::new(...));
        // Plan 523 H5: 字面量初始化 → std const `static NAME: Mutex<T> = ...`
        // （访问位 `*NAME.lock().unwrap()` 形态不变；str 字面量静态为 &str 型）。
        if matches!(store.kind, StoreKind::Shared) {
            let static_name = self.global_var_static_name(&store.name);
            if Self::global_store_is_const_init(store) {
                let ty = if matches!(
                    store.ty,
                    Type::StrFixed(_) | Type::StrSlice | Type::StrOwned | Type::CStrLit
                ) {
                    "&str".to_string()
                } else {
                    self.rust_type_name(&store.ty)
                };
                write!(out, "static {}: Mutex<{}> = Mutex::new(", static_name, ty)?;
                self.expr(&store.expr, out)?;
                write!(out, ")")?;
                return Ok(());
            }
            let ty = self.rust_type_name(&store.ty);
            write!(
                out,
                "static {}: Lazy<Mutex<{}>> = Lazy::new(|| Mutex::new(",
                static_name, ty
            )?;
            self.expr(&store.expr, out)?;
            write!(out, "))")?;
            return Ok(());
        }

        // Plan 6B-3.4: const declaration → const NAME: &str = "...";
        if matches!(store.kind, StoreKind::Const) {
            // C8: `str`-typed consts emit `&str` (string literals are the only
            // const-evaluable values; matches hand-written `&'static str`).
            let ty_name = if matches!(
                store.ty,
                Type::StrFixed(_) | Type::StrSlice | Type::StrOwned
            ) {
                "&str".to_string()
            } else {
                self.rust_type_name(&store.ty)
            };
            // C8: top-level `pub const` → emits `pub const`.
            if store.is_pub {
                write!(out, "pub const {}: {} = ", store.name, ty_name)?;
            } else {
                write!(out, "const {}: {} = ", store.name, ty_name)?;
            }
            self.expr(&store.expr, out)?;
            return Ok(());
        }

        // Plan 151: Generate static Lazy<Mutex<T>> for global variables
        // Plan 523 H5: 字面量初始化 → std const static Mutex（同 Shared 位）。
        // Plan 523 W3(四路 runner 回链修正):fn 内名命中全局的 let/var
        // 初始化 → 全局写块形(参考写穿语义,b43 怪序;fn 内 static 遮蔽
        // 形得 10/11 ≠ 参考 5/5/6)。
        if self.is_global_var(&store.name) && self.in_fn_body {
            let static_name = self.global_var_static_name(&store.name);
            write!(out, "{{ let __a2r_gv = ")?;
            self.expr(&store.expr, out)?;
            write!(out, "; *{}.lock().unwrap() = __a2r_gv; }}", static_name)?;
            return Ok(());
        }
        if self.is_global_var(&store.name) {
            let static_name = self.global_var_static_name(&store.name);
            if Self::global_store_is_const_init(store) {
                let ty = if matches!(
                    store.ty,
                    Type::StrFixed(_) | Type::StrSlice | Type::StrOwned | Type::CStrLit
                ) {
                    "&str".to_string()
                } else {
                    self.rust_type_name(&store.ty)
                };
                write!(out, "static {}: Mutex<{}> = Mutex::new(", static_name, ty)?;
                self.expr(&store.expr, out)?;
                write!(out, ")")?;
                return Ok(());
            }
            let ty = self.rust_type_name(&store.ty);

            // Generate: static NAME: Lazy<Mutex<T>> = Lazy::new(|| Mutex::new(...))
            // NOTE: no trailing ';' here — the caller (Stmt::Store handler) adds
            // exactly one ';'. Emitting one here produced `static ...();;`
            // (double semicolon), which is a compile error in Rust.
            write!(
                out,
                "static {}: Lazy<Mutex<{}>> = Lazy::new(|| Mutex::new(",
                static_name, ty
            )?;
            self.expr(&store.expr, out)?;
            write!(out, "))")?;
            return Ok(());
        }

        // Type inference for Unknown types
        // Plan 204 Phase 1E: Also skip type annotation when the rendered type
        // contains "/* unknown */" (e.g., Option</* unknown */>, [/* unknown */; N])
        // Plan 019 Phase 1: `var c = x.field ?? fallback` — the coalesce
        // yields the INNER type, but the parser-annotated store.ty keeps the
        // Option wrapper, mis-declaring the local and breaking every later
        // use. Annotate with the (already-computed) effective type instead.
        let mut ty_name = if matches!(&store.expr, Expr::NullCoalesce(_, _))
            && matches!(store.ty, Type::Option(_))
            && !matches!(effective_ty, Type::Unknown)
        {
            self.rust_type_name(&effective_ty)
        } else {
            self.rust_type_name(&store.ty)
        };
        // PLAN-668 R-24（P666-D1 同族）：局部 `var/let x []T = []` 后续
        // push/就地写（mutated_let_bindings 在案）——&[T] 不可变借无 push，
        // 局部槽改 Vec<T>（025 sys-monitor cores/procs/disks/users 实案；
        // 返回位/参数位 &[T] 语义不变，仅局部绑定改道）。
        if matches!(store.ty, Type::Slice(_))
            && (matches!(store.kind, StoreKind::Var)
                || self.mutated_let_bindings.contains(store.name.as_ref()))
        {
            if let Type::Slice(slice) = &store.ty {
                if !matches!(&*slice.elem, Type::Spec(_)) {
                    ty_name = format!("Vec<{}>", self.rust_type_name(&slice.elem));
                }
            }
        }
        // Plan 391 D2: `let v: Option<T> = m.get(k)` where T is a non-Copy
        // container — Rust's HashMap/Vec::get returns Option<&T>, so the user's
        // owned annotation `Option<Vec<String>>` triggers E0308. Rewrite the
        // annotation to `Option<&T>`. (The unannotated path already lets Rust
        // infer Option<&T> correctly — see is_stmt.) Only fires when the
        // initializer is a `.get(...)` call (canonical borrowing-returning
        // lookup) and T is not a primitive-Copy type (i32/bool/… copy fine).
        // Covers both AST shapes: `?T` → Type::Option(_) and `Option<T>` →
        // Type::GenericInstance { base_name: "Option", .. }.
        let init_is_borrowing_get = matches!(&store.expr,
            Expr::Call(c) if matches!(c.name.as_ref(),
                Expr::Dot(_, m) if m.as_str() == "get"));
        let option_inner_from_get: Option<&Type> = if init_is_borrowing_get {
            match &store.ty {
                Type::Option(inner) => Some(inner.as_ref()),
                Type::GenericInstance(inst)
                    if inst.base_name.as_str() == "Option" && inst.args.len() == 1 =>
                {
                    Some(&inst.args[0])
                }
                _ => None,
            }
        } else {
            None
        };
        let ty_name = if let Some(inner) = option_inner_from_get {
            if !Self::is_primitive_copy(inner)
                && !matches!(inner, Type::Reference(_) | Type::Unknown | Type::Void)
            {
                format!("Option<&{}>", self.rust_type_name(inner))
            } else {
                ty_name
            }
        } else {
            ty_name
        };
        // Skip type annotation for: Unknown types, error propagation (?), closures, or unknown placeholders
        let is_error_propagate = matches!(&store.expr, Expr::ErrorPropagate(_));
        let mut has_unknown = matches!(store.ty, Type::Unknown)
            || ty_name.contains("/* unknown */")
            || is_error_propagate;

        // Check if the expression is a closure - closures should not have explicit type annotations
        // because Rust infers closure types automatically
        let is_closure = matches!(store.expr, Expr::Closure(_));

        // Check if expression is a borrow (&x or &mut x) - type should be a reference.
        // Plan 310 Phase 2: only treat as borrow if the inner binding is at a
        // borrow tier (BorrowView/BorrowMut). Escape tiers (Clone/RcRefCell)
        // produce owned/Rc values, so the type must NOT get a & prefix.
        let borrow_inner: Option<&Expr> = match &store.expr {
            Expr::View(inner) => Some(inner.as_ref()),
            Expr::Dot(obj, f) if f.as_str() == "view" || f.as_str() == "mut" => Some(obj.as_ref()),
            _ => None,
        };
        let is_mut_form = matches!(&store.expr, Expr::Dot(_, f) if f.as_str() == "mut");
        let (is_borrow, is_mut_borrow) = match borrow_inner {
            Some(inner) => {
                use crate::trans::escape::OwnershipTier;
                let name = match inner {
                    Expr::Ident(n) | Expr::Ref(n) => Some(n.as_str()),
                    _ => None,
                };
                // Copy types: no borrow annotation (value copied, not referenced).
                // Use strict primitive check (String is NOT Copy in Rust).
                let is_copy = name
                    .and_then(|n| self.local_var_types.get(n))
                    .map(|ty| Self::is_primitive_copy(ty))
                    .unwrap_or(false);
                if is_copy {
                    (false, false)
                } else {
                    let tier = name
                        .map(|n| self.current_escape_tier(n))
                        .unwrap_or(OwnershipTier::Owned);
                    let is_real_borrow = tier.is_borrow();
                    (
                        is_real_borrow && !is_mut_form,
                        is_real_borrow && is_mut_form,
                    )
                }
            }
            None => (false, false),
        };

        let mut ty_name = if is_borrow && matches!(store.ty, Type::StrOwned | Type::StrFixed(_)) {
            "&str".to_string()
        } else if is_mut_borrow && matches!(store.ty, Type::StrOwned | Type::StrFixed(_)) {
            "&mut str".to_string()
        } else if is_borrow && !matches!(store.ty, Type::Unknown) {
            format!("&{}", ty_name)
        } else if is_mut_borrow && !matches!(store.ty, Type::Unknown) {
            format!("&mut {}", ty_name)
        } else {
            ty_name
        };

        // Check if expression is an Array of spec instances (for unknown-type fallback)
        let spec_array_type: Option<String> = if has_unknown {
            if let Expr::Array(elems) = &store.expr {
                let spec_name = elems.iter().find_map(|e| {
                    if let Expr::Ident(name) = e {
                        self.var_spec_map.get(name).cloned()
                    } else {
                        None
                    }
                });
                spec_name.map(|sn| format!("Vec<Box<dyn {}>>", sn))
            } else {
                None
            }
        } else {
            None
        };

        // PLAN-010 T2 (009 ledger 026_list_basic): `let list = List.new()`
        // with no element type anywhere in scope renders `let list = Vec::new()`
        // — E0282 (unconstrained Vec<_>). Auto would infer from usage; with no
        // usage to infer from, default the element to i64 (Auto's int-default,
        // same rule as `let x = 42` → i64) and annotate explicitly. Skipped
        // when the binding is read later — that read may pin the real element
        // type via inference (`return Ok(messages)` over Vec<Message>;
        // 08_generics/007), and a forced annotation would override it.
        let init_is_bare_list_new = matches!(&store.expr, Expr::Call(c)
            if c.args.args.is_empty()
                && matches!(c.name.as_ref(),
                    Expr::Dot(obj, m)
                        if m.as_str() == "new"
                            && matches!(obj.as_ref(),
                                Expr::Ident(n) | Expr::GenName(n) if n.as_str() == "List")));
        let list_ty_unconstrained = match &store.ty {
            Type::Unknown => true,
            Type::List(inner) => matches!(inner.as_ref(), Type::Unknown),
            _ => false,
        };
        if init_is_bare_list_new
            && list_ty_unconstrained
            && !self.later_used_locals.contains(store.name.as_str())
        {
            ty_name = "Vec<i64>".to_string();
            has_unknown = false;
        }
        // PLAN-010 T2 (009 ledger 002_option_construct): `let b = None` with
        // no constrained type — E0282. Default to Option<i64> (Auto's
        // int-default rule; the Option twin of the List.new default above).
        // Only the literal `None` form: bare `nil` placeholders keep Rust's
        // inference from later uses (annotating them would override a
        // differently-typed constraint elsewhere).
        let init_is_bare_none = matches!(&store.expr, Expr::None);
        let none_ty_unconstrained = match &store.ty {
            Type::Unknown => true,
            Type::Option(inner) => matches!(inner.as_ref(), Type::Unknown),
            _ => false,
        };
        if init_is_bare_none
            && none_ty_unconstrained
            && !self.later_used_locals.contains(store.name.as_str())
        {
            ty_name = "Option<i64>".to_string();
            has_unknown = false;
        }

        // Skip type annotation if: Unknown type, type contains unknown, or closure expression
        // Exception: spec array expressions need explicit type annotation for dyn Trait
        // Plan 391 D3: also skip when a List<str-family> annotation would conflict with a
        // borrowed iterator source. `let parts List<str> = x.split(".")` transpiles to
        // x.split(".").collect::<Vec<_>>() which yields Vec<&str>; forcing Vec<String>
        // is E0308. Let Rust infer (Vec<&str>) — mirrors the unannotated form.
        let is_borrowed_split_source = matches!(&store.ty,
            Type::List(elem) if matches!(elem.as_ref(),
                Type::StrSlice | Type::StrOwned | Type::StrFixed(_) | Type::CStrLit))
            && matches!(&store.expr, Expr::Call(call)
                if matches!(call.name.as_ref(),
                    Expr::Dot(_, m) if m.as_str() == "split"));
        let skip_type_annotation =
            ((has_unknown || is_closure) && spec_array_type.is_none()) || is_borrowed_split_source;

        let safe_name = Self::rust_ident(store.name.as_str());
        // Plan 419 §4.5: 闭包写捕获绑定 —— 声明升级 Rc<RefCell<T>>
        // (全限定路径,免 use 前导;访问点由 Ident 改写 borrow/borrow_mut)。
        if self.is_write_captured(store.name.as_str()) {
            write!(
                out,
                "let {} = std::rc::Rc::new(std::cell::RefCell::new(",
                safe_name
            )?;
            self.expr(&store.expr, out)?;
            write!(out, "))")?;
            return Ok(());
        }
        // Plan 399 Phase 11.5: a `let` binding that is mutated later (push/assign)
        // needs `let mut` — Auto's `let` allows mutation, Rust's doesn't.
        let needs_mut = matches!(store.kind, StoreKind::Let)
            && self.mutated_let_bindings.contains(store.name.as_ref());
        if skip_type_annotation {
            // No type annotation - let Rust infer the type
            match store.kind {
                StoreKind::Let if needs_mut => {
                    write!(out, "let mut {} = ", safe_name)?;
                }
                StoreKind::Let => {
                    write!(out, "let {} = ", safe_name)?;
                }
                StoreKind::Var => {
                    write!(out, "let mut {} = ", safe_name)?;
                }
                _ => {
                    write!(out, "let {} = ", safe_name)?;
                }
            }
        } else {
            // Explicit type annotation for non-closure expressions
            let ty_str = spec_array_type.as_deref().unwrap_or(&ty_name);
            match store.kind {
                StoreKind::Let if needs_mut => {
                    write!(out, "let mut {}: {} = ", safe_name, ty_str)?;
                }
                StoreKind::Let => {
                    write!(out, "let {}: {} = ", safe_name, ty_str)?;
                }
                StoreKind::Var => {
                    write!(out, "let mut {}: {} = ", safe_name, ty_str)?;
                }
                _ => {
                    write!(out, "let {}: {} = ", safe_name, ty_str)?;
                }
            }
        }

        // Plan 032 G2-core: init from a `Some(x)` is-arm binding over a
        // `.get()` scrutinee — the binding is `&V`; the new local must OWN
        // its value (often mutated right after: `var updated = x;
        // updated.push(...)`) → emit `x.clone()`.
        if let Expr::Ident(name) = &store.expr {
            if self.get_ref_bindings.contains(name.as_str()) {
                write!(out, "{}.clone()", Self::rust_ident(name.as_str()))?;
                return Ok(());
            }
        }

        // Plan 159 6B-2.2: Wrap array elements in Box::new() for []Spec types
        let is_spec_slice =
            matches!(&store.ty, Type::Slice(slice) if matches!(&*slice.elem, Type::Spec(_)));
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
        } else if matches!(&store.ty, Type::List(_) | Type::Array(_)) {
            // List<T> or Array<T> (Vec<T>) with Array literal → vec![...]
            if let Expr::Array(elems) = &store.expr {
                write!(out, "vec![")?;
                // Check if element type is String — need .to_string() on &str literals
                let elem_ty = match &store.ty {
                    Type::List(inner) => Some(inner.as_ref() as &Type),
                    Type::Array(arr) => Some(&arr.elem as &Type),
                    _ => None,
                };
                let elem_is_string = elem_ty.map_or(false, |ty| {
                    matches!(ty, Type::StrOwned | Type::StrSlice | Type::StrFixed(_))
                });
                for (i, elem) in elems.iter().enumerate() {
                    self.expr(elem, out)?;
                    if elem_is_string && matches!(elem, Expr::Str(_) | Expr::CStr(_)) {
                        write!(out, ".to_string()")?;
                    }
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
            // Plan 391 D1: for `let x: u64/i64/usize = <expr>.len()` (or .length()),
            // suppress the default `as i32` cast. `.len()` returns usize in Rust;
            // casting to i32 truncates and conflicts with a wider annotation.
            // Save/restore so nested lets (rare: closures) don't clobber the flag.
            let saved_suppress = self.len_i32_cast_suppressed;
            self.len_i32_cast_suppressed = matches!(store.ty, Type::U64 | Type::I64 | Type::USize)
                && Self::expr_is_len_call(&store.expr);
            // PLAN-668 R-24：str 族 let + if 初始化器——臂尾字面量入
            // String 槽需 .to_string()（经 ret_type_needs_string_coercion
            // 的 if 臂发射位消费）。
            let saved_let_coerce = self.let_init_str_coercion;
            self.let_init_str_coercion = matches!(
                store.ty,
                Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
            ) && matches!(store.expr, Expr::If(_));
            // PLAN-026 T-03 配套: int let 绑定 + List<int> 索引 rhs ——
            // db 层元素 i64,窄化 as i32(否则 let x = ns[i] 推断 i64,
            // 与 i32 语境(模型字段/字面比较)连环 E0308/E0277)。
            let narrow_i32 =
                matches!(store.ty, Type::Int) && self.expr_has_int_list_index(&store.expr);
            if narrow_i32 {
                write!(out, "(")?;
            }
            self.expr(&store.expr, out)?;
            if narrow_i32 {
                write!(out, ") as i32")?;
            }
            self.let_init_str_coercion = saved_let_coerce;
            self.len_i32_cast_suppressed = saved_suppress;
            // Auto-clone: when assigning from a non-Copy struct field (e.g., let path = node.name)
            // the struct field is moved, but the struct may still be used later
            // Skip for pointer types — *mut T / *const T are Copy
            // Plan 387 follow-up: also skip TaskRef-typed values (move-only,
            // no Clone impl) — e.g. `let h = w.sink` must move, not clone.
            if !matches!(store.ty, Type::Ptr(_)) && !self.expr_is_taskref(&store.expr) {
                if let Expr::Dot(obj, _field) = &store.expr {
                    if let Expr::Ident(obj_name) = obj.as_ref() {
                        let obj_is_copy = self
                            .local_var_types
                            .get(obj_name)
                            .map(|ty| Self::is_copy_type(ty))
                            .unwrap_or(true);
                        if !obj_is_copy {
                            write!(out, ".clone()")?;
                        }
                    }
                }
                // Plan 348 H3: Auto-clone on plain identifier rebind of a
                // non-Copy local (e.g. `var t = s` / `let t = s` where `s` is a
                // String). Without this, a2r emits `let t = s;` — a Rust move —
                // and any later use of `s` fails to compile (E0382: borrow of
                // moved value). The escape analyzer does not classify simple
                // print-after-move as Clone tier, so we apply a conservative
                // type-based heuristic here, mirroring the Expr::Dot arm above.
                //
                // Use the strict `is_primitive_copy` check (NOT `is_copy_type`)
                // because String/Vec/HashMap are NOT Copy in Rust even though
                // `is_copy_type` loosely treats them as such. Skip globals
                // (their reads already deref a MutexGuard — `*G.lock()` — and
                // must stay by-value) and self-rebinds (`let s = s`).
                if let Expr::Ident(src_name) = &store.expr {
                    if src_name != &store.name
                        && !self.is_global_var(src_name)
                        && !matches!(store.ty, Type::Reference(_) | Type::Ptr(_))
                    {
                        // Plan 434 (242 #18-2): a str-SLICE-typed source is
                        // &str at runtime when it's a `str` param (clone keeps
                        // &str, so `let cur: String = cur_in.clone()` is
                        // E0308). `.to_string()` is correct for both &str
                        // params and String returns labeled `str`.
                        if let Some(ty) = self.local_var_types.get(src_name) {
                            if matches!(ty, Type::StrFixed(_) | Type::StrSlice | Type::CStrLit) {
                                write!(out, ".to_string()")?;
                            } else if !matches!(ty, Type::Unknown) && !Self::is_primitive_copy(ty) {
                                write!(out, ".clone()")?;
                            }
                        }
                    }
                }
            }
        }

        // Add integer cast when assigning json.as_int() result to int/uint variable
        // json.as_int() returns i64, but int needs i32 and uint needs u32
        if matches!(store.ty, Type::Int | Type::Uint) {
            if let Expr::Call(call) = &store.expr {
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if let Expr::Ident(name) = obj.as_ref() {
                        if name == "json" && method == "as_int" {
                            if matches!(store.ty, Type::Int) {
                                write!(out, " as i64")?;
                            } else {
                                write!(out, " as u32")?;
                            }
                        }
                    }
                }
            }
        }

        // When assigning a string literal to a String/Str type, add .to_string()
        // because Rust string literals are &str, but String type needs conversion
        if matches!(
            store.ty,
            Type::StrOwned | Type::StrFixed(_) | Type::StrSlice | Type::CStrLit
        ) {
            if matches!(&store.expr, Expr::Str(_) | Expr::CStr(_)) {
                write!(out, ".to_string()")?;
            }
        }

        // self.field assignment in &self context needs .clone()
        if Self::is_self_dot(&store.expr) {
            write!(out, ".clone()")?;
        }

        // PLAN-681 T-05（N2/P4）：typed let 收 Value 读收口——声明型与
        // Value 读不配时按目标型降链（str→as_str、int→as_i64、bool→
        // as_bool、List→as_array；bare List 已映射 Vec<Value>）。发射尾端
        // 追加,与既有 auto-clone 尾缀共存（Value 读自身无 String 语义,
        // 先前的 .clone() 包在降链内侧仍合法）。
        if !matches!(store.ty, Type::Unknown) && self.expr_is_json_value_read(&store.expr) {
            match &store.ty {
                Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit => {
                    write!(out, ".as_str().unwrap_or_default().to_string()")?;
                }
                Type::Int | Type::I64 => {
                    write!(out, ".as_i64().unwrap_or(0)")?;
                }
                Type::Bool => {
                    write!(out, ".as_bool().unwrap_or(false)")?;
                }
                Type::List(_) | Type::Array(_) => {
                    write!(out, ".as_array().cloned().unwrap_or_default()")?;
                }
                _ => {}
            }
        }

        // Plan 376F: Integer type conversion for Store assignments.
        // When `let x: i32 = <u32 expr>` or `let x: u32 = <i32 expr>`,
        // insert the appropriate cast. The declared type (store.ty) is the
        // target; the expression type is inferred from local_var_types.
        if !matches!(store.ty, Type::Unknown) {
            // Get the expression's inferred type
            let expr_ty = self.infer_type_from_expr(&store.expr);
            let need_cast = match (&store.ty, &expr_ty) {
                (Type::Int, Type::Uint) => Some(" as i64"),
                (Type::Uint, Type::Int) => Some(" as u32"),
                (Type::USize, Type::Int) => Some(" as usize"),
                (Type::USize, Type::Uint) => Some(" as usize"),
                (Type::Int, Type::USize) => Some(" as i64"),
                (Type::Uint, Type::USize) => Some(" as u32"),
                _ => None,
            };
            if let Some(cast) = need_cast {
                write!(out, "{}", cast)?;
            }
        }

        Ok(())
    }

    // Function declaration
    /// Plan 399 Phase 11.5: collect names of bindings that are mutated after
    /// declaration. Recognizes `name.push(...)`/`.insert(...)`/`.extend(...)`
    /// method calls and `name = ...` assignments. Used to upgrade `let name` to
    /// `let mut name` so the mutation compiles (Auto's `let` allows mutation;
    /// Rust's doesn't).
    /// Plan 433 A1: collect `let/var x = fn(...)` bindings (no type annotation)
    /// whose callee has a known return type, recursively through control-flow
    /// bodies. Feeds local_var_types at fn_decl entry (see fn_decl).
    pub(super) fn scan_call_init_bindings(
        stmts: &[Stmt],
        ret_types: &HashMap<AutoStr, Type>,
        out: &mut Vec<(AutoStr, Type)>,
    ) {
        use crate::ast::Stmt;
        fn visit(
            stmts: &[Stmt],
            ret_types: &HashMap<AutoStr, Type>,
            out: &mut Vec<(AutoStr, Type)>,
        ) {
            for stmt in stmts {
                match stmt {
                    Stmt::Store(store) => {
                        if matches!(store.ty, Type::Unknown) {
                            if let Expr::Call(call) = &store.expr {
                                match call.name.as_ref() {
                                    Expr::Ident(fname) => {
                                        if let Some(ret) = ret_types.get(fname.as_str()) {
                                            if !matches!(ret, Type::Unknown | Type::Void) {
                                                out.push((store.name.clone(), ret.clone()));
                                            }
                                        }
                                    }
                                    // Plan 514 W1 (洞 B 配套): static 构造
                                    // `Type.new(...)` 的返回类型 = 该类型自身
                                    // ——此前不记录,导致 `var h = Holder.new(1)`
                                    // 的 h 类型缺失,`h.set(...)` 被内建
                                    // set→insert 遮蔽(m09 洞)。Dot 与 Bina-Dot
                                    // 两种调用名形态都认。
                                    Expr::Dot(obj, m) if m.as_str() == "new" => {
                                        if let Expr::Ident(tname) = obj.as_ref() {
                                            out.push((
                                                store.name.clone(),
                                                RustTrans::placeholder_user_type(tname.clone()),
                                            ));
                                        }
                                    }
                                    Expr::Bina(obj, Op::Dot, m) => {
                                        let is_new = matches!(m.as_ref(), Expr::Ident(id) if id.as_str() == "new");
                                        if is_new {
                                            if let Expr::Ident(tname) = obj.as_ref() {
                                                out.push((
                                                    store.name.clone(),
                                                    RustTrans::placeholder_user_type(tname.clone()),
                                                ));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Stmt::If(if_) => {
                        for br in &if_.branches {
                            visit(&br.body.stmts, ret_types, out);
                        }
                        if let Some(e) = &if_.else_ {
                            visit(&e.stmts, ret_types, out);
                        }
                    }
                    Stmt::For(for_) => {
                        if let Some(init) = &for_.init {
                            visit(std::slice::from_ref(init.as_ref()), ret_types, out);
                        }
                        visit(&for_.body.stmts, ret_types, out);
                    }
                    Stmt::Try(t) => {
                        visit(&t.body.stmts, ret_types, out);
                    }
                    Stmt::Block(b) => visit(&b.stmts, ret_types, out),
                    _ => {}
                }
            }
        }
        visit(stmts, ret_types, out);
    }

    pub(super) fn scan_mutated_bindings(
        body: &crate::ast::Body,
        mut_method_names: &std::collections::HashSet<AutoStr>,
        rust_types: &std::collections::HashSet<AutoStr>,
    ) -> std::collections::HashSet<AutoStr> {
        let mut out = std::collections::HashSet::new();
        let mut methods: std::collections::HashSet<AutoStr> = mut_method_names.clone();
        methods.extend(
            [
                "push", "insert", "extend", "pop", "remove", "retain", "next", "clear", "sort_by",
                "sort", "swap", "truncate", "drain", "splice", "resize",
            ]
            .iter()
            .map(|m| AutoStr::from(*m)),
        );
        // methods now carries the built-in mutating methods PLUS every user
        // `mut fn` name (PLAN-010 T2: `x.mutFn()` needs `let mut x`, E0596).

        // PLAN-592: rust 导入类型绑定传播——let 的 init 为 `Type.method(...)`
        // (Type ∈ use.rs 导入)或已标记 rust 绑定的方法调用链时,该绑定视为
        // rust 型。其后的任意方法调用保守视作 mutating:转译期无 rustdoc 元
        // 数据,&mut self 不可知(E0596 防御优先;代价仅为产物侧 unused_mut
        // 告警,不影响本仓警告基线)。
        fn receiver_root(e: &crate::ast::Expr) -> Option<&crate::ast::Name> {
            match e {
                crate::ast::Expr::Call(c) => match c.name.as_ref() {
                    crate::ast::Expr::Dot(obj, _) => receiver_root(obj),
                    crate::ast::Expr::Ident(n) => Some(n),
                    _ => None,
                },
                crate::ast::Expr::Ident(n) => Some(n),
                _ => None,
            }
        }
        fn collect_rust_typed(
            stmts: &[Stmt],
            rust_types: &std::collections::HashSet<AutoStr>,
            rust_typed: &mut std::collections::HashSet<AutoStr>,
        ) {
            for stmt in stmts {
                match stmt {
                    Stmt::Store(s) => {
                        if let Some(root) = receiver_root(&s.expr) {
                            let r = root.as_str();
                            if rust_types.contains(r) || rust_typed.contains(r) {
                                rust_typed.insert(s.name.clone());
                            }
                        }
                        if let crate::ast::Expr::Block(b) = &s.expr {
                            collect_rust_typed(&b.stmts, rust_types, rust_typed);
                        }
                    }
                    Stmt::If(i) => {
                        for br in &i.branches {
                            collect_rust_typed(&br.body.stmts, rust_types, rust_typed);
                        }
                        if let Some(e) = &i.else_ {
                            collect_rust_typed(&e.stmts, rust_types, rust_typed);
                        }
                    }
                    Stmt::For(f) => collect_rust_typed(&f.body.stmts, rust_types, rust_typed),
                    Stmt::Block(b) => collect_rust_typed(&b.stmts, rust_types, rust_typed),
                    Stmt::Try(t) => collect_rust_typed(&t.body.stmts, rust_types, rust_typed),
                    _ => {}
                }
            }
        }
        let mut rust_typed: std::collections::HashSet<AutoStr> = Default::default();
        for _ in 0..4 {
            let before = rust_typed.len();
            collect_rust_typed(&body.stmts, rust_types, &mut rust_typed);
            if rust_typed.len() == before {
                break;
            }
        }

        fn visit_expr(
            expr: &crate::ast::Expr,
            out: &mut std::collections::HashSet<AutoStr>,
            methods: &std::collections::HashSet<AutoStr>,
            rust_typed: &std::collections::HashSet<AutoStr>,
        ) {
            // `name.push(...)` etc → name is mutated
            if let crate::ast::Expr::Call(call) = expr {
                if let crate::ast::Expr::Dot(obj, method) = call.name.as_ref() {
                    if let crate::ast::Expr::Ident(name) = obj.as_ref() {
                        // PLAN-592: rust 型绑定的任意方法调用保守视作 mutating
                        if methods.contains(method.as_str()) || rust_typed.contains(name.as_str()) {
                            out.insert(name.clone());
                        }
                    }
                }
                // Recurse into call args
                for arg in &call.args.args {
                    if let crate::ast::Arg::Pos(e) = arg {
                        visit_expr(e, out, methods, rust_typed);
                    }
                }
            }
            // Plan 523 H2: 赋值 place 根(任意深度)——`x = e`/`x.f = e`/
            // `x.f.g = e`/`x[i] = e` 及复合赋值族,根绑定需 mut。此前
            // AST 层零覆盖,一级 place 靠 post-process 正则补(其
            // `\b{name}\.\w+\s*=` 不匹配两级 place——b36 考古:
            // `o.first.v = ...` 后 `let o` 缺 mut,rustc E0594)。
            if let crate::ast::Expr::Bina(lhs, op, _) = expr {
                if matches!(
                    op,
                    Op::Asn | Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq | Op::ModEq
                ) {
                    let mut root = lhs.as_ref();
                    loop {
                        match root {
                            crate::ast::Expr::Ident(n) => {
                                out.insert(n.clone());
                                break;
                            }
                            crate::ast::Expr::Dot(o, _) | crate::ast::Expr::Index(o, _) => {
                                root = o.as_ref()
                            }
                            _ => break,
                        }
                    }
                }
            }
        }
        fn visit_stmt(
            stmt: &Stmt,
            out: &mut std::collections::HashSet<AutoStr>,
            methods: &std::collections::HashSet<AutoStr>,
            rust_typed: &std::collections::HashSet<AutoStr>,
        ) {
            match stmt {
                Stmt::Expr(expr) => visit_expr(expr, out, methods, rust_typed),
                Stmt::Store(store) => visit_expr(&store.expr, out, methods, rust_typed),
                Stmt::If(if_) => {
                    for b in &if_.branches {
                        for s in &b.body.stmts {
                            visit_stmt(s, out, methods, rust_typed);
                        }
                    }
                    if let Some(e) = &if_.else_ {
                        for s in &e.stmts {
                            visit_stmt(s, out, methods, rust_typed);
                        }
                    }
                }
                Stmt::For(for_) => {
                    for s in &for_.body.stmts {
                        visit_stmt(s, out, methods, rust_typed);
                    }
                }
                Stmt::Block(b) => {
                    for s in &b.stmts {
                        visit_stmt(s, out, methods, rust_typed);
                    }
                }
                Stmt::Try(t) => {
                    for s in &t.body.stmts {
                        visit_stmt(s, out, methods, rust_typed);
                    }
                }
                _ => {}
            }
        }
        for stmt in &body.stmts {
            visit_stmt(stmt, &mut out, &methods, &rust_typed);
        }
        out
    }

    /// PLAN-010 T2: idents READ anywhere in a statement (best effort).
    /// None = statement shape this walker doesn't understand — callers must
    /// treat the statement as potentially reading every local.
    pub(super) fn stmt_read_idents(stmt: &Stmt) -> Option<std::collections::HashSet<AutoStr>> {
        match stmt {
            Stmt::Expr(e) => Self::expr_read_idents(e),
            Stmt::Store(s) => Self::expr_read_idents(&s.expr),
            Stmt::Return(e) => Self::expr_read_idents(e),
            // Blank-line placeholders and loop control carry no reads.
            Stmt::EmptyLine(_) | Stmt::Break | Stmt::Continue => Some(Default::default()),
            _ => None,
        }
    }

    fn expr_read_idents(e: &Expr) -> Option<std::collections::HashSet<AutoStr>> {
        let mut out = std::collections::HashSet::new();
        if Self::expr_read_idents_into(e, &mut out) {
            Some(out)
        } else {
            None
        }
    }

    /// false = hit an unwalked shape (conservative: caller assumes "used").
    fn expr_read_idents_into(e: &Expr, out: &mut std::collections::HashSet<AutoStr>) -> bool {
        match e {
            Expr::Ident(n) => {
                out.insert(n.clone());
                true
            }
            Expr::Call(c) => {
                if !Self::expr_read_idents_into(c.name.as_ref(), out) {
                    return false;
                }
                for arg in &c.args.args {
                    match arg {
                        Arg::Pos(x) | Arg::Pair(_, x) => {
                            if !Self::expr_read_idents_into(x, out) {
                                return false;
                            }
                        }
                        Arg::Name(_) => {}
                    }
                }
                true
            }
            Expr::Bina(l, _, r) => {
                Self::expr_read_idents_into(l, out) && Self::expr_read_idents_into(r, out)
            }
            Expr::Dot(o, _) => Self::expr_read_idents_into(o, out),
            Expr::Index(b, i) => {
                Self::expr_read_idents_into(b, out) && Self::expr_read_idents_into(i, out)
            }
            Expr::Some(x)
            | Expr::Ok(x)
            | Expr::Err(x)
            | Expr::ErrorPropagate(x)
            | Expr::View(x) => Self::expr_read_idents_into(x, out),
            Expr::NullCoalesce(l, r) => {
                Self::expr_read_idents_into(l, out) && Self::expr_read_idents_into(r, out)
            }
            Expr::Array(xs) => xs.iter().all(|x| Self::expr_read_idents_into(x, out)),
            Expr::Cast { expr, .. } | Expr::To { expr, .. } => {
                Self::expr_read_idents_into(expr, out)
            }
            // Leaves with no idents (all literal kinds + nil family).
            Expr::Int(_)
            | Expr::Uint(_)
            | Expr::I8(_)
            | Expr::U8(_)
            | Expr::I64(_)
            | Expr::U64(_)
            | Expr::Byte(_)
            | Expr::Float(_, _)
            | Expr::Double(_, _)
            | Expr::Bool(_)
            | Expr::Char(_)
            | Expr::Str(_)
            | Expr::CStr(_)
            | Expr::Nil
            | Expr::Null
            | Expr::None => true,
            _ => false,
        }
    }

    /// Plan 447 H5: count `is` statements per identifier scrutinee across one
    /// function body (nested into if/for/try/block bodies and is-arm bodies).
    /// A scrutinee matched >= 2 times can't be matched by value each time
    /// (E0382 move), so is_stmt emits `match &v` for it instead.
    pub(super) fn scan_is_scrutinee_uses(body: &crate::ast::Body) -> HashMap<AutoStr, usize> {
        fn visit_expr(expr: &crate::ast::Expr, out: &mut HashMap<AutoStr, usize>) {
            if let crate::ast::Expr::Is(is_expr) = expr {
                if let crate::ast::Expr::Ident(name) = &is_expr.target {
                    *out.entry(name.clone()).or_insert(0) += 1;
                }
                for b in &is_expr.branches {
                    let body = match b {
                        crate::ast::IsBranch::EqBranch(_, body) => body,
                        crate::ast::IsBranch::IfBranch(_, body) => body,
                        crate::ast::IsBranch::ElseBranch(body) => body,
                    };
                    for s in &body.stmts {
                        visit_stmt(s, out);
                    }
                }
            }
        }
        fn visit_stmt(stmt: &Stmt, out: &mut HashMap<AutoStr, usize>) {
            match stmt {
                Stmt::Expr(expr) => visit_expr(expr, out),
                Stmt::Is(is_stmt) => {
                    if let crate::ast::Expr::Ident(name) = &is_stmt.target {
                        *out.entry(name.clone()).or_insert(0) += 1;
                    }
                    for b in &is_stmt.branches {
                        let body = match b {
                            crate::ast::IsBranch::EqBranch(_, body) => body,
                            crate::ast::IsBranch::IfBranch(_, body) => body,
                            crate::ast::IsBranch::ElseBranch(body) => body,
                        };
                        for s in &body.stmts {
                            visit_stmt(s, out);
                        }
                    }
                }
                Stmt::If(if_) => {
                    for b in &if_.branches {
                        visit_expr(&b.cond, out);
                        for s in &b.body.stmts {
                            visit_stmt(s, out);
                        }
                    }
                    if let Some(e) = &if_.else_ {
                        for s in &e.stmts {
                            visit_stmt(s, out);
                        }
                    }
                }
                Stmt::For(for_) => {
                    for s in &for_.body.stmts {
                        visit_stmt(s, out);
                    }
                }
                Stmt::Block(b) => {
                    for s in &b.stmts {
                        visit_stmt(s, out);
                    }
                }
                Stmt::Try(t) => {
                    for s in &t.body.stmts {
                        visit_stmt(s, out);
                    }
                    for s in &t.catch_body.stmts {
                        visit_stmt(s, out);
                    }
                    if let Some(f) = &t.finally_body {
                        for s in &f.stmts {
                            visit_stmt(s, out);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut out = HashMap::new();
        for stmt in &body.stmts {
            visit_stmt(stmt, &mut out);
        }
        out
    }

    pub(super) fn use_stmt(&mut self, use_stmt: &Use, out: &mut impl Write) -> AutoResult<()> {
        // Plan 417-D2: register imported fn signatures (single-file
        // mode has no project discovery — see the helper).
        self.register_import_signatures(use_stmt);
        // Plan 376U: crate-root files (lib.at, */mod.at) use top-level `use X: sym`
        // as public re-exports, so render `pub use` even when the source has no
        // explicit `pub` prefix. (Mirrors Rust's `pub use` in lib.rs / mod.rs.)
        let is_reexport = self.is_crate_root && !use_stmt.is_pub;
        let pub_kw = if use_stmt.is_pub || is_reexport {
            "pub "
        } else {
            ""
        };
        match use_stmt.kind {
            UseKind::Auto => {
                let module = use_stmt.paths.join(".");
                if self.assembly_modules.contains(&module) {
                    self.local_modules.insert(module.clone());
                    if use_stmt.is_wildcard {
                        write!(out, "{pub_kw}use crate::{module}::*;")?;
                    } else if !use_stmt.items.is_empty() {
                        write!(
                            out,
                            "{pub_kw}use crate::{module}::{{{}}};",
                            use_stmt.items.join(", ")
                        )?;
                    }
                    return Ok(());
                }
                // PLAN-738 §5.3：Rust 目录声明 unsupported（a2r-std 无该宿主
                // 模块，如 io/net——金样时代的 a2r_std::io 属幻影映射）时不
                // 发射导入，不冒称宿主 provider；逐符号调用点仍由 provider
                // 门拒绝。模块级 unsupported 是目录结论，非逐符号面。
                if use_stmt.paths.first().is_some_and(|p| p.as_str() == "auto") {
                    if let Some(module) = use_stmt.paths.get(1) {
                        if crate::stdlib_assembly::providers::unsupported_reason(
                            module.as_str(),
                            "rust",
                        )
                        .is_some()
                        {
                            return Ok(());
                        }
                    }
                }
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
                // In merge mode, skip module imports entirely (all code in one file)
                if !self.local_modules.is_empty()
                    && use_stmt.items.is_empty()
                    && !use_stmt.is_wildcard
                    && use_stmt.paths.len() == 1
                {
                    let mod_name = use_stmt.paths[0].as_str();
                    if self.local_modules.contains(mod_name) {
                        if self.merge_mode {
                            return Ok(()); // skip: functions already in merged file
                        }
                        // Module already declared via mod X; at file header.
                        // Plan 545: use X (bare, no items) = namespace-only —
                        // import the module name itself (Rust 2018 style); flat
                        // glob requires explicit `use X: *` (wildcard arm above
                        // at the shared emitter). Qualified `X.foo()` resolves
                        // through the namespace import.
                        self.glob_imported_modules.insert(mod_name.to_string());
                        write!(out, "{}use crate::{};", pub_kw, mod_name)?;
                        return Ok(());
                    }
                }

                // Map Auto stdlib to Rust modules
                // Join all path segments into a single Rust path
                if !use_stmt.paths.is_empty() {
                    let full_path = use_stmt
                        .paths
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join("::");
                    // In multi-file mode, bare module names (e.g., "types") that are
                    // NOT in local_modules → generate correct cross-module reference
                    let mod_name = use_stmt.paths[0].as_str();
                    let is_multi_file_bare = (!self.local_modules.is_empty()
                        || !self.sibling_modules.is_empty())
                        && use_stmt.paths.len() == 1
                        && !mod_name.contains("::")
                        && !self.local_modules.contains(mod_name);
                    // Map known Auto stdlib modules to a2r_std
                    let rust_path = if is_multi_file_bare {
                        if self.merge_mode {
                            // In merge mode, skip cross-module imports (all in one file)
                            return Ok(());
                        }
                        if self.sibling_modules.contains(mod_name) {
                            // Same directory → use super::X
                            self.glob_imported_modules.insert(mod_name.to_string());
                            format!("super::{}", mod_name)
                        } else {
                            // Different directory → use crate::X
                            self.glob_imported_modules.insert(mod_name.to_string());
                            format!("crate::{}", mod_name)
                        }
                    } else if full_path.starts_with("super::")
                        && (!self.local_modules.is_empty()
                            || !self.sibling_modules.is_empty()
                            || self.is_dir_module)
                    {
                        // In multi-file mode, Auto's `use super.X` means "parent directory's X"
                        // Extract just the module name (first segment after super::) for dir_children lookup
                        let after_super = &full_path[7..];
                        let crate_mod = if let Some(colon_pos) = after_super.find("::") {
                            &after_super[..colon_pos]
                        } else {
                            after_super
                        };
                        self.glob_imported_modules.insert(crate_mod.to_string());
                        // Build the replacement prefix based on whether it's a dir child
                        let prefix = if self.is_dir_module && self.dir_children.contains(crate_mod)
                        {
                            // Directory module: X is a child module → self::X
                            format!("self::{}", crate_mod)
                        } else if !self.is_dir_module && self.sibling_modules.contains(crate_mod) {
                            // Non-dir module: X is a known sibling (same directory) → super::X
                            format!("super::{}", crate_mod)
                        } else if !self.is_dir_module && !self.module_types.contains_key(crate_mod)
                        {
                            // Non-dir module: X is not a top-level module → likely a sibling
                            format!("super::{}", crate_mod)
                        } else {
                            // X is a crate-level module → crate::X
                            format!("crate::{}", crate_mod)
                        };
                        // Replace super::module with the computed prefix, keeping the rest of the path
                        if after_super.len() > crate_mod.len() {
                            format!("{}{}", prefix, &after_super[crate_mod.len()..])
                        } else {
                            prefix
                        }
                    } else if full_path.starts_with("auto::") {
                        let rest = &full_path[6..];
                        match rest {
                            "math" | "str" | "time" | "env" | "json" | "file" | "fs" | "http"
                            | "list" | "hashmap" | "hashset" | "btreemap" | "vecdeque" | "char"
                            | "conv" | "io" | "log" | "path" | "net" | "process" | "sys"
                            | "sse" | "may" | "sqlite" | "redis" => {
                                self.a2r_std_used.set(true);
                                format!("a2r_std::{}", rest)
                            }
                            _ => format!("crate::{}", rest),
                        }
                    } else if use_stmt.paths.len() == 1
                        && !use_stmt.paths[0].as_str().contains("::")
                    {
                        // Single-file mode: bare module name (e.g., "types", "settings")
                        // Check if it's a known stdlib module or a local crate module.
                        //
                        // NOTE (Plan 347): `regex` is intentionally NOT in this list.
                        // a2r_std has no `regex` module (the entry pointed at a phantom
                        // `a2r_std::regex`), so routing `use auto.regex` there always failed
                        // to compile. Treating `regex` like any other crate module lets the
                        // regex parity library (wrapped as `pub mod regex`) resolve correctly,
                        // and gives non-parity users a clearer "unresolved module" error
                        // instead of a misleading one.
                        let mod_name = use_stmt.paths[0].as_str();
                        match mod_name {
                            "math" | "str" | "time" | "env" | "json" | "file" | "fs" | "http"
                            | "list" | "hashmap" | "hashset" | "btreemap" | "vecdeque" | "char"
                            | "conv" | "io" | "log" | "path" | "net" | "process" | "sys"
                            | "sse" | "may" | "sqlite" | "redis" => {
                                self.a2r_std_used.set(true);
                                format!("a2r_std::{}", mod_name)
                            }
                            _ => format!("crate::{}", mod_name),
                        }
                    } else {
                        // Check if the first segment is a known crate module or stdlib
                        let first_seg = use_stmt.paths[0].as_str();
                        let is_stdlib = matches!(
                            first_seg,
                            "math"
                                | "str"
                                | "time"
                                | "env"
                                | "json"
                                | "file"
                                | "fs"
                                | "http"
                                | "list"
                                | "hashmap"
                                | "hashset"
                                | "btreemap"
                                | "vecdeque"
                                | "char"
                                | "conv"
                                | "io"
                                | "log"
                                | "path"
                                | "net"
                                | "process"
                                | "sys"
                                | "sse"
                                | "may"
                                | "sqlite"
                                | "redis"
                        );
                        if is_stdlib {
                            self.a2r_std_used.set(true);
                            format!("a2r_std::{}", full_path)
                        } else if self.module_types.contains_key(first_seg)
                            || self.dep_crates.contains(&AutoStr::from(first_seg))
                            || first_seg == "serde"
                            || first_seg == "chrono"
                        {
                            // Known crate module → prefix with crate::
                            format!("crate::{}", full_path)
                        } else {
                            full_path.replace("auto::", "crate::")
                        }
                    };
                    // PLAN-738 T-11：核心模块导入闭包——导入语句发射
                    // `use a2r_std::<core>::{...}` 即真实引用：具名项逐个
                    // 验证绑定+公共签名（适配面符号拒绝并指明限定拼写）；
                    // wildcard 只记台账（§5.3 不因未引用 unsupported 拒绝），
                    // 其裸名调用在 call 收集器按公共面唯一归属闭包。
                    if let Some(core) = rust_path
                        .strip_prefix("a2r_std::")
                        .filter(|rest| !rest.contains("::"))
                        .filter(|core| {
                            crate::stdlib_assembly::validate::CORE_MODULES.contains(core)
                        })
                    {
                        let core = core.to_string();
                        if use_stmt.is_wildcard {
                            self.stdlib_wildcard_modules.insert(core.clone());
                        } else {
                            for item in &use_stmt.items {
                                if let Some(proof) =
                                    crate::stdlib_assembly::host::verify_rust_import(
                                        &core,
                                        item,
                                        self.assembly_runtime,
                                    )?
                                {
                                    self.assembly_references.push(proof);
                                }
                                self.stdlib_named_imports
                                    .insert(item.as_str().to_string(), core.clone());
                            }
                        }
                    }
                    if use_stmt.is_wildcard {
                        write!(out, "{}use {}::*;", pub_kw, rust_path)?;
                    } else if !use_stmt.items.is_empty() {
                        write!(
                            out,
                            "{}use {}::{{{}}};",
                            pub_kw,
                            rust_path,
                            use_stmt.items.join(", ")
                        )?;
                    } else if is_multi_file_bare {
                        // Plan 545: bare module import = namespace-only —
                        // `use super::X;` / `use crate::X;` brings the module
                        // name in scope for qualified `X::foo()` paths; flat
                        // glob requires explicit `use X: *` (wildcard arm).
                        write!(out, "{}use {};", pub_kw, rust_path)?;
                    } else if full_path.starts_with("super::")
                        && (!self.local_modules.is_empty()
                            || !self.sibling_modules.is_empty()
                            || self.is_dir_module)
                    {
                        // Multi-segment super:: path in directory module context.
                        // Only add wildcard if the last segment is a known module name,
                        // NOT if it's a type/function name (e.g., GateType, AgentTurn).
                        let last_seg = use_stmt.paths.last().map(|s| s.as_str()).unwrap_or("");
                        let is_last_mod = self.dir_children.contains(last_seg)
                            || self.module_types.contains_key(last_seg)
                            || self.local_modules.contains(last_seg);
                        if is_last_mod {
                            write!(out, "{}use {}::*;", pub_kw, rust_path)?;
                        } else {
                            write!(out, "{}use {};", pub_kw, rust_path)?;
                        }
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
                // Plan 610 ⑥ (004 §3.5): C imports lower to a generated FFI
                // face — the manifest is the shared IR (builtin extractor or
                // JSON beside the source). Unknown headers keep the pre-610
                // ignore behavior.
                self.emit_use_c_ffi(use_stmt, out)?;
            }
            UseKind::Rust => {
                // Direct Rust imports: join paths with :: to form full Rust path
                if !use_stmt.paths.is_empty() {
                    let full_path = use_stmt
                        .paths
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join("::");

                    // Companion trait imports that methods on this crate require
                    let companion_imports: &[(&str, &str)] = &[
                        ("rand", "use rand::Rng;"),
                        ("rand::seq", "use rand::seq::SliceRandom;"),
                        ("rayon", "use rayon::prelude::*;"),
                        ("sha2", "use sha2::Digest;"),
                        ("clap", "use clap::Parser;"),
                        ("serde_json", "use serde_json::Value;"),
                        (
                            "unicode_segmentation",
                            "use unicode_segmentation::UnicodeSegmentation;",
                        ),
                        ("toml", "use toml::Value;"),
                        ("mime_guess", "use mime_guess::MimeGuess;"),
                        (
                            "percent_encoding",
                            "use percent_encoding::{percent_encode, NON_ALPHANUMERIC};",
                        ),
                        ("urlencoding", "use urlencoding::encode;"),
                        ("hex", "use hex;"),
                        // Plan 013 (B14): bridge crates whose many types are
                        // referenced unqualified in the Auto source (e.g.
                        // `auto_val.Value.Str` → a2r emits bare `Value::Str`).
                        // A glob import resolves all of them without enumerating.
                        ("auto_val", "use auto_val::*;"),
                        ("auto_atom", "use auto_atom::*;"),
                    ];

                    let already_emitted = self.uses.contains(full_path.as_str());
                    if !already_emitted {
                        // Check if a companion import upgrades this to a wildcard
                        let companion_wildcard = companion_imports
                            .iter()
                            .find(|(prefix, _)| {
                                full_path == *prefix
                                    || full_path.starts_with(&format!("{}::", prefix))
                            })
                            .and_then(|(_, line)| {
                                line.strip_prefix("use ").and_then(|s| s.strip_suffix(';'))
                            })
                            .filter(|companion| {
                                // Only upgrade for wildcard companions (e.g., rayon::prelude::*)
                                // Don't upgrade for specific trait imports (e.g., rand::Rng)
                                companion.ends_with("::*")
                                    && companion.starts_with(&format!("{}::", full_path))
                            });

                        if use_stmt.is_wildcard {
                            write!(out, "{}use {}::*;", pub_kw, full_path)?;
                        } else if let Some(wc) = companion_wildcard {
                            write!(out, "{}use {};", pub_kw, wc)?;
                            // Track the wildcard path so companion loop doesn't re-emit it
                            self.uses.insert(wc.to_string().into());
                        } else if !use_stmt.items.is_empty() {
                            write!(
                                out,
                                "{}use {}::{{{}}};",
                                pub_kw,
                                full_path,
                                use_stmt.items.join(", ")
                            )?;
                        } else {
                            write!(out, "{}use {};", pub_kw, full_path)?;
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
                            if !import_line.is_empty()
                                && *import_line != format!("use {};", full_path)
                            {
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
                                    if u_str.starts_with(&format!(
                                        "{}::*",
                                        companion_path.split("::").next().unwrap_or("")
                                    )) && companion_path.starts_with(&format!(
                                        "{}::",
                                        u_str.trim_end_matches("::*")
                                    )) {
                                        return true;
                                    }
                                    // Brace-expansion dedup: "crate::{a, b}" vs existing "crate::a"
                                    if let Some(brace_pos) = companion_path.find("::{") {
                                        let crate_path = &companion_path[..brace_pos];
                                        if u_str == crate_path {
                                            return true;
                                        }
                                        if let Some(items_str) = companion_path
                                            .strip_prefix(&format!("{}::{{", crate_path))
                                        {
                                            let items_str =
                                                items_str.strip_suffix('}').unwrap_or(items_str);
                                            let companion_items: Vec<&str> =
                                                items_str.split(',').map(|s| s.trim()).collect();
                                            if u_str.starts_with(&format!("{}::", crate_path)) {
                                                let item_name = u_str
                                                    .strip_prefix(&format!("{}::", crate_path))
                                                    .unwrap_or("");
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
                    "use.py imports are not supported in Rust target".to_string(),
                ));
            }
        }
        Ok(())
    }
}
