//! PLAN-843 结构拆分（自 rust.rs 原样搬移；算法/行为零变化）。
use super::*;

impl RustTrans {
    pub(super) fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        let mut emitted = Vec::new();
        self.call_inner(call, &mut emitted)?;
        let text = std::str::from_utf8(&emitted)
            .map_err(|e| e.to_string())?
            .to_string();
        // R7-03：实参面从 AST 序列化（部分直发臂的文本丢参——AST 是
        // 真实调用事实）。
        let args_text = self.serialize_positional_args(call);
        self.collect_core_reference(call, &text, &args_text)?;
        out.write_all(&emitted)?;
        Ok(())
    }

    fn call_inner(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // PLAN-714 r3 R3-T2: 裸名 diff 内建映射（VM codegen intrinsics
        // 9915/9916/9917 同源——corpus fsys.at 裸名直调位；模块内无本地
        // 同名符号[fsys.at:245 注记原文]）。
        if let Expr::Ident(name) = call.name.as_ref() {
            let mapped = match name.as_str() {
                "diff_files" => Some("a2r_std::diff::diff_files"),
                "diff_dirs" => Some("a2r_std::diff::diff_dirs"),
                "diff_snapshots" => Some("a2r_std::diff::diff_snapshots"),
                "diff_files_window" => Some("a2r_std::diff::diff_files_window"),
                // PLAN-716 组B: 帧时间戳（供②——VM 9918/9919 同源 a2r 臂）。
                "frame_begin_ms" => Some("a2r_std::frame::begin_ms"),
                "frame_present_ms" => Some("a2r_std::frame::present_ms"),
                _ => None,
            };
            if let Some(q) = mapped {
                self.a2r_std_used.set(true);
                write!(out, "{q}(")?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    if let Arg::Pos(expr) = arg {
                        self.expr(expr, out)?;
                    } else {
                        self.arg(arg, out)?;
                    }
                }
                write!(out, ")")?;
                return Ok(());
            }
        }
        // Plan 032 G4 (auto-ai consumer): serde_json::Value variant
        // construction (`Value.String(x)` / `Value.Number(n)` after
        // `use.rust serde_json[::Value]`). The external enum's variants take
        // OWNED values, which no later branch knows (no tuple_field_types):
        // String needs `.to_string()` on &str payloads; Number needs a
        // serde_json::Number (int → `Number::from`, float-typed →
        // `Number::from_f64(...).unwrap_or(...)`). Handle before everything —
        // later generic branches would emit invalid shapes (consumers
        // sed-patched 37 such sites in daemon output).
        if let Expr::Dot(obj, variant) = call.name.as_ref() {
            if let Expr::Ident(tname) = obj.as_ref() {
                if tname.as_str() == "Value"
                    && matches!(variant.as_str(), "String" | "Number")
                    && call.args.args.len() == 1
                    && self.uses.iter().any(|u| {
                        let s = u.as_str();
                        s == "serde_json" || s == "serde_json::Value"
                    })
                {
                    if let Some(Arg::Pos(e)) = call.args.args.first() {
                        match variant.as_str() {
                            "String" => {
                                write!(out, "Value::String(")?;
                                self.expr(e, out)?;
                                write!(out, ".to_string())")?;
                            }
                            _ => {
                                let is_float = match e {
                                    Expr::Float(..) | Expr::Double(..) => true,
                                    // Params are registered into local_var_types
                                    // at fn_decl entry (typed float/Double decls).
                                    Expr::Ident(n) => matches!(
                                        self.local_var_types.get(n.as_str()),
                                        Some(Type::Float | Type::Double)
                                    ),
                                    _ => false,
                                };
                                write!(out, "Value::Number(serde_json::Number::")?;
                                if is_float {
                                    write!(out, "from_f64(")?;
                                } else {
                                    write!(out, "from(")?;
                                }
                                self.expr(e, out)?;
                                if is_float {
                                    write!(out, ").unwrap_or(serde_json::Number::from(0))")?;
                                } else {
                                    write!(out, ")")?;
                                }
                                write!(out, ")")?;
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }
        // Detect Rust macro patterns: name!("...") was parsed as name.collect()("...")
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
                                if matches!(
                                    macro_name.as_str(),
                                    "debug"
                                        | "info"
                                        | "warn"
                                        | "error"
                                        | "trace"
                                        | "println"
                                        | "eprintln"
                                        | "print"
                                        | "eprint"
                                        | "write"
                                        | "writeln"
                                        | "format"
                                        | "panic"
                                        | "assert"
                                        | "assert_eq"
                                        | "assert_ne"
                                        | "todo"
                                        | "unimplemented"
                                        | "unreachable"
                                        | "vec"
                                        | "include_str"
                                        | "concat"
                                        | "env"
                                ) {
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

        // Plan musk-022 CRUD 扩展: `List<T>.new(expr)` -> `expr` (List maps to Vec in
        // Rust, and the array literal expr is already a vec![...]; the .new()
        // wrapper is a no-op constructor that must be stripped, or Rust sees
        // `List<Message>.new(vec![...])` literally (fails: `List` isn't a value).
        // Matches `Expr::Dot(GenName("List"/"Array"), "new")` and the parsed
        // generic-ident forms. Only strips when there is exactly one positional arg.
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if method.as_str() == "new" {
                let is_collection_ctor = match obj.as_ref() {
                    Expr::GenName(n) => matches!(n.as_str(), "List" | "Array"),
                    Expr::Ident(n) => matches!(n.as_str(), "List" | "Array"),
                    // `List<Message>` parses as `List < Message` (Bina with Lt) since
                    // `<` is the comparison operator; recognize the generic form.
                    Expr::Bina(lhs, Op::Lt, _) => {
                        matches!(lhs.as_ref(), Expr::Ident(n) if matches!(n.as_str(), "List" | "Array"))
                    }
                    _ => false,
                };
                if is_collection_ctor {
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        if call.args.args.len() == 1 {
                            self.expr(a, out)?;
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Plan 387: `Task.spawn("Name", cap)` -> `spawn_<name>()` (default-construct).
        // Plan 390 §5 Phase B (M1): `Task.spawn("Name", cap, init1, init2, ...)`
        // -> `spawn_<name>_with(init1, init2, ...)` (required state-field args;
        // Rust doesn't support default fn params, so init-arg spawns use a
        // separate _with helper that takes all state fields as required params).
        // Arg 0 = name literal, arg 1 = capacity (ignored), args[2..] =
        // positional initializers for the task's state fields (declaration order).
        if let Expr::Dot(obj, method) = call.name.as_ref() {
            if method.as_str() == "spawn" {
                if let Expr::Ident(receiver) = obj.as_ref() {
                    if receiver.as_str() == "Task" {
                        // First arg is the task type name as a string literal.
                        if let Some(Arg::Pos(Expr::Str(name))) = call.args.args.first() {
                            self.a2r_std_used.set(true);
                            let has_init = call.args.args.len() > 2;
                            let suffix = if has_init { "_with" } else { "" };
                            write!(out, "spawn_{}{}(", snake_of(name), suffix)?;
                            // Forward init args (everything after name + capacity).
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i < 2 {
                                    continue; // skip name (0) and capacity (1)
                                }
                                if i > 2 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(expr) = arg {
                                    self.expr(expr, out)?;
                                }
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Plan 387 W4: `h.send(Variant)` / `h.send(Variant(args))` — when the arg
        // is a registered task message-variant name, rewrite it to the generated
        // enum constructor `EnumName::Variant` / `EnumName::Variant(args)` so the
        // generated TaskRef::<EnumName>::send gets a value of the right type.
        // Also: for a String-message task, `h.send("literal")` needs `.to_string()`
        // (the channel carries owned String, not &str).
        if let Expr::Dot(_obj, method) = call.name.as_ref() {
            if method.as_str() == "send" {
                if let Some(Arg::Pos(arg0)) = call.args.args.first() {
                    // Plan 387 follow-up: resolve the receiver's task from the
                    // handle variable (e.g. `h` in `h.send(Add(1))`) so
                    // same-named variants across tasks pick the right enum.
                    let receiver_task: Option<String> = match _obj.as_ref() {
                        Expr::Ident(v) => self.handle_task_map.get(v.as_str()).cloned(),
                        _ => None,
                    };
                    let rewritten = self.rewrite_msg_variant_arg(arg0, receiver_task.as_deref());
                    if let Some(rw) = rewritten {
                        self.expr(_obj, out)?;
                        write!(out, ".send({})", rw)?;
                        return Ok(());
                    }
                    // String-message task: bare string literal → owned String.
                    if matches!(arg0, Expr::Str(_) | Expr::CStr(_)) {
                        self.expr(_obj, out)?;
                        write!(out, ".send(")?;
                        self.expr(arg0, out)?;
                        write!(out, ".to_string())")?;
                        return Ok(());
                    }
                }
            }
        }

        // Plan 013 G6: generic typed-JSON decode.
        // `json.decode[T](text)` parses `[T]` as an index expression (Auto has no
        // turbofish syntax), so `call.name` is `Expr::Index(json.decode_callee, T_ident)`.
        // This bypasses the normal `("json","decode")` dispatch (which only sees
        // Bina/Dot callees). Intercept it here, before any other call handling, and
        // emit `serde_json::from_str::<T>(&text)`.
        if let Expr::Index(callee, ty_arg) = call.name.as_ref() {
            // callee should be `json.decode` in one of two AST forms.
            let is_json_decode = match callee.as_ref() {
                Expr::Bina(obj, op, rhs) if matches!(op, Op::Dot) => {
                    matches!(obj.as_ref(), Expr::Ident(n) if n == "json")
                        && matches!(rhs.as_ref(), Expr::Ident(m) if m == "decode")
                }
                Expr::Dot(obj, field) => {
                    matches!(obj.as_ref(), Expr::Ident(n) if n == "json") && field == "decode"
                }
                _ => false,
            };
            if is_json_decode {
                if let Expr::Ident(ty_name) = ty_arg.as_ref() {
                    self.a2r_std_used.set(true);
                    write!(out, "serde_json::from_str::<{}>(", ty_name)?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        self.expr_as_str(a, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
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
            // PLAN-010 T2 (009 ledger 006/007/009): Auto `say(x)` is the
            // print-with-newline builtin — previously emitted verbatim as an
            // unresolved `say(...)` call (E0425). Lower to println! with a
            // Display placeholder (works for str and value scalars alike).
            if name == "say" {
                write!(out, "println!(\"{{}}\"")?;
                for arg in &call.args.args {
                    write!(out, ", ")?;
                    self.arg(arg, out)?;
                }
                write!(out, ")")?;
                return Ok(());
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

        // Plan 310 Phase 0.2: Union construction `Union(field: val)` →
        // `Union::new_field(val)`. Rust requires `unsafe` to construct a union
        // via `Union { field: val }`; we route through the safe `new_<f>`
        // accessor generated in union_decl. Only the first named field is used
        // (union semantics: only one variant active at a time).
        if let Expr::Ident(type_name) = call.name.as_ref() {
            if self.union_types.contains(type_name) {
                if let Some(Arg::Pair(field_name, val_expr)) = call.args.args.first() {
                    write!(out, "{}::new_{}(", type_name, field_name)?;
                    self.expr(val_expr, out)?;
                    write!(out, ")")?;
                    return Ok(());
                }
                // Positional arg: use the first union field name
                if let Some(field) = call.args.args.first() {
                    write!(out, "{}::new_0(", type_name)?;
                    self.arg(field, out)?;
                    write!(out, ")")?;
                    return Ok(());
                }
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
            if is_imported_macro
                && matches!(
                    name_str,
                    "debug"
                        | "info"
                        | "warn"
                        | "error"
                        | "trace"
                        | "println"
                        | "eprintln"
                        | "print"
                        | "eprint"
                        | "format"
                        | "vec"
                        | "write"
                        | "writeln"
                        | "log"
                        | "log_enabled"
                )
            {
                write!(out, "{}!(", name)?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    if let Arg::Pos(Expr::FStr(fstr)) = arg {
                        // Inline f-string as macro format string
                        write!(out, "\"")?;
                        for part in &fstr.parts {
                            match part {
                                Expr::Str(s) | Expr::CStr(s) => {
                                    let escaped = s
                                        .replace("\\", "\\\\")
                                        .replace("\"", r##"\""##)
                                        .replace("{", "{{")
                                        .replace("}", "}}");
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

        // Plan 204 Phase 1A: Rust assert/assert_eq/assert_ne/panic are macros, need ! suffix
        // Special: when 2nd arg is an f-string, inline it directly (not format!())
        // because Rust assert! expects a string literal as the format arg.
        if let Expr::Ident(name) = call.name.as_ref() {
            if matches!(
                name.as_str(),
                "assert" | "assert_eq" | "assert_ne" | "panic"
            ) {
                write!(out, "{}!(", name)?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    // Check if this arg is an f-string — inline it without format!()
                    if let Arg::Pos(Expr::FStr(fstr)) = arg {
                        write!(out, "\"")?;
                        for part in &fstr.parts {
                            match part {
                                Expr::Str(s) | Expr::CStr(s) => {
                                    let escaped = s
                                        .replace("\\", "\\\\")
                                        .replace("\"", r##"\""##)
                                        .replace("{", "{{")
                                        .replace("}", "}}");
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
                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                    self.expr(expr, out)?;
                }
                write!(out, ")")?;
                return Ok(());
            }
            match name.as_str() {
                "sleep_ms" => {
                    write!(out, "std::thread::sleep(std::time::Duration::from_millis(")?;
                    if let Some(arg) = call.args.args.first() {
                        self.arg(arg, out)?;
                    }
                    write!(out, " as u64))")?;
                    return Ok(());
                }
                // PLAN-724 T-06：流自由函数面（707 手工接口）。接收者分型
                // 决定同步/异步 facade 形态；EOF 哨兵 "" 与 VM 契约一致。
                "stream_next" | "stream_is_done" | "stream_close" => {
                    let is_async = call
                        .args
                        .args
                        .first()
                        .and_then(|a| match a {
                            Arg::Pos(Expr::Ident(name)) => {
                                Some(self.http_stream_async_vars.contains(name))
                            }
                            _ => None,
                        })
                        .unwrap_or(false);
                    self.a2r_std_used.set(true);
                    let m = name.as_str();
                    if is_async {
                        match m {
                            "stream_next" => {
                                write!(out, "a2r_std::http::stream_next_async(&")?;
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    self.expr(expr, out)?;
                                }
                                write!(out, ").await")?;
                            }
                            "stream_is_done" => {
                                write!(out, "a2r_std::http::stream_is_done_async(&")?;
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    self.expr(expr, out)?;
                                }
                                write!(out, ")")?;
                            }
                            _ => {
                                write!(out, "a2r_std::http::stream_close_async(&")?;
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    self.expr(expr, out)?;
                                }
                                write!(out, ")")?;
                            }
                        }
                    } else {
                        match m {
                            "stream_next" => {
                                write!(out, "a2r_std::http::stream_next(&")?;
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    self.expr(expr, out)?;
                                }
                                write!(out, ")")?;
                            }
                            "stream_is_done" => {
                                write!(out, "a2r_std::http::stream_is_done(&")?;
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    self.expr(expr, out)?;
                                }
                                write!(out, ")")?;
                            }
                            _ => {
                                write!(out, "a2r_std::http::stream_close(&")?;
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    self.expr(expr, out)?;
                                }
                                write!(out, ")")?;
                            }
                        }
                    }
                    return Ok(());
                }
                // PLAN-724 T-06：裸名流生产者（use http: get_stream 形态）。
                "get_stream" | "post_stream" | "post_stream_with_headers" => {
                    self.a2r_std_used.set(true);
                    emit_http_verb_call(self, out, name.as_str(), call)?;
                    return Ok(());
                }
                "http_post" => {
                    // http_post(url, body, api_key) → async { let (s,b,e,k) = a2r_std::http_post(...).await; HttpResponse { ... } }
                    self.a2r_std_used.set(true);
                    write!(
                        out,
                        "async {{ let (status, body, error, kind) = a2r_std::http_post("
                    )?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ")?;
                        }
                        if let Arg::Pos(expr) = arg {
                            self.expr(expr, out)?;
                            // Auto-borrow: add .as_str() for String → &str
                            if !matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                let is_str_slice = if let Expr::Ident(name) = expr {
                                    self.local_var_types
                                        .get(name)
                                        .map(|ty| matches!(ty, Type::StrSlice))
                                        .unwrap_or(false)
                                } else {
                                    false
                                };
                                if !is_str_slice {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        } else {
                            self.arg(arg, out)?;
                        }
                    }
                    write!(
                        out,
                        ").await; HttpResponse {{ status, body, error, kind }} }}"
                    )?;
                    return Ok(());
                }
                "simple_hash" => {
                    self.a2r_std_used.set(true);
                    write!(out, "a2r_std::simple_hash(")?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        self.expr_as_str(a, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "time_now" => {
                    self.a2r_std_used.set(true);
                    write!(out, "a2r_std::time_now()")?;
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
                        self.a2r_std_used.set(true);
                        // PLAN-724 T-06：async 上下文走内核 async 面（同步桥接
                        // 面在 async 上下文会被响亮拒绝）。
                        let (fname, await_suffix) = if self.in_async_ctx.get() {
                            ("a2r_std::http::post_sync_async", ".await")
                        } else {
                            ("a2r_std::http::post_sync", "")
                        };
                        write!(out, "{{ let __resp = {}(", fname)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                let already_str = matches!(expr, Expr::Str(_) | Expr::CStr(_))
                                    || if let Expr::Ident(name) = expr {
                                        self.local_var_types
                                            .get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice))
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };
                                if !already_str {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        }
                        write!(
                            out,
                            "){}; a2r_std::http::set_last_status(__resp.0); __resp.1 }}",
                            await_suffix
                        )?;
                        return Ok(());
                    }
                    ("http", "get_sync") => {
                        self.a2r_std_used.set(true);
                        let (fname, await_suffix) = if self.in_async_ctx.get() {
                            ("a2r_std::http::get_sync_async", ".await")
                        } else {
                            ("a2r_std::http::get_sync", "")
                        };
                        write!(out, "{{ let __resp = {}(", fname)?;
                        if let Some(Arg::Pos(expr)) = call.args.args.first() {
                            self.expr(expr, out)?;
                            let already_str = matches!(expr, Expr::Str(_) | Expr::CStr(_))
                                || if let Expr::Ident(name) = expr {
                                    self.local_var_types
                                        .get(name)
                                        .map(|ty| matches!(ty, Type::StrSlice))
                                        .unwrap_or(false)
                                } else {
                                    false
                                };
                            if !already_str {
                                write!(out, ".as_str()")?;
                            }
                        }
                        write!(
                            out,
                            "){}; a2r_std::http::set_last_status(__resp.0); __resp.1 }}",
                            await_suffix
                        )?;
                        return Ok(());
                    }
                    ("http", "last_status") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::last_status() as i64")?;
                        return Ok(());
                    }
                    ("http", "post_bearer") => {
                        self.a2r_std_used.set(true);
                        write!(out, "{{ let __resp = a2r_std::http::post_bearer(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                let already_str = matches!(expr, Expr::Str(_) | Expr::CStr(_))
                                    || if let Expr::Ident(name) = expr {
                                        self.local_var_types
                                            .get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice))
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };
                                if !already_str {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        }
                        write!(
                            out,
                            ").await; a2r_std::http::set_last_status(__resp.0); __resp.1 }}"
                        )?;
                        return Ok(());
                    }
                    ("http", "post_bearer_sync") => {
                        self.a2r_std_used.set(true);
                        let (fname, await_suffix) = if self.in_async_ctx.get() {
                            ("a2r_std::http::post_bearer_sync_async", ".await")
                        } else {
                            ("a2r_std::http::post_bearer_sync", "")
                        };
                        write!(out, "{{ let __resp = {}(", fname)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                                let already_str = matches!(expr, Expr::Str(_) | Expr::CStr(_))
                                    || if let Expr::Ident(name) = expr {
                                        self.local_var_types
                                            .get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice))
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };
                                if !already_str {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        }
                        write!(
                            out,
                            "){}; a2r_std::http::set_last_status(__resp.0); __resp.1 }}",
                            await_suffix
                        )?;
                        return Ok(());
                    }
                    ("http", "post") => {
                        self.a2r_std_used.set(true);
                        // PLAN-724 T-06：元数分派——三参=历史认证 tuple（生成
                        // 的 HttpResponse 面）；两参=普通客户端 post（内核
                        // Response）。名称/元数/返回约定按 §5.1 冻结，不冲突。
                        if call.args.args.len() >= 3 {
                            write!(
                                out,
                                "async {{ let (status, body, error, kind) = a2r_std::http::post("
                            )?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(expr) = arg {
                                    self.expr(expr, out)?;
                                    if let Expr::Ident(name) = expr {
                                        if self
                                            .local_var_types
                                            .get(name)
                                            .map(|ty| !matches!(ty, Type::StrSlice))
                                            .unwrap_or(true)
                                        {
                                            write!(out, ".as_str()")?;
                                        }
                                    }
                                }
                            }
                            write!(
                                out,
                                ").await; HttpResponse {{ status, body, error, kind }} }}"
                            )?;
                            return Ok(());
                        }
                        let (fname, await_suffix) = if self.in_async_ctx.get() {
                            ("a2r_std::http::post_async", ".await")
                        } else {
                            ("a2r_std::http::post", "")
                        };
                        write!(out, "{}(", fname)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            }
                        }
                        write!(out, "){}", await_suffix)?;
                        return Ok(());
                    }
                    // PLAN-724 T-06：普通动词族（两参 post 同型）——async 上下文
                    // 发射 *_async().await；同步上下文发射同步桥接面。
                    ("http", "get") | ("http", "put") | ("http", "delete") => {
                        self.a2r_std_used.set(true);
                        let Expr::Dot(_, method) = call.name.as_ref() else {
                            unreachable!("http verb dispatch requires Dot name");
                        };
                        emit_http_verb_call(self, out, method.as_str(), call)?;
                        return Ok(());
                    }
                    // PLAN-724 T-06：流生产者——async 上下文产出 AsyncHTTPStream
                    // （内部记录变量分型），同步上下文产出 HTTPStream。
                    ("http", "get_stream") | ("http", "post_stream") => {
                        self.a2r_std_used.set(true);
                        let Expr::Dot(_, method) = call.name.as_ref() else {
                            unreachable!("http stream dispatch requires Dot name");
                        };
                        emit_http_verb_call(self, out, method.as_str(), call)?;
                        return Ok(());
                    }
                    ("http", "request") => {
                        // http.request(method, url) → a2r_std::http::request(method, url)
                        // (Plan 013 G6: returns a RequestBuilder for chained .header/.body/.timeout/.send.)
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::request(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // PLAN-727 T-06: 可取消文件传输公共面——transfer_download/
                    // transfer_upload 非阻塞提交，sync/async 同形发射（句柄
                    // 面无 async 变体；等待点在 transfer_wait 分叉）。
                    ("http", "transfer_download") | ("http", "transfer_upload") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::{}(", method)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // PLAN-727 T-06: transfer_wait——同步桥接面 / async await 面
                    // 分叉（唯一需要 async 变体的传输自由函数）。
                    ("http", "transfer_wait") => {
                        self.a2r_std_used.set(true);
                        if self.in_async_ctx.get() {
                            write!(out, "a2r_std::http::transfer_wait_async(&")?;
                        } else {
                            write!(out, "a2r_std::http::transfer_wait(&")?;
                        }
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        if self.in_async_ctx.get() {
                            write!(out, ").await")?;
                        } else {
                            write!(out, ")")?;
                        }
                        return Ok(());
                    }
                    // PLAN-729 T-05: 服务端文件响应构造——零 I/O 描述符，
                    // sync/async 同形（打开/发送在生成 handler 的宿主
                    // serve 面）。严格 options 与路径词法校验在描述符内。
                    // PLAN-730 T-06: 服务端上传自由函数族。receive/commit/
                    // reject 是 await 点（同步上下文无合法执行面——指名诊断
                    // 不发射伪同步调用）；metadata/error 纯同步。str 形参按
                    // 位次收敛 `.as_str()`（字面量/String/&str 三形皆合法；
                    // 首参句柄按值 move）。
                    ("http", "upload_receive")
                    | ("http", "upload_commit")
                    | ("http", "upload_reject") => {
                        self.a2r_std_used.set(true);
                        if !self.in_async_ctx.get() {
                            return Err(crate::error::AutoError::Msg(format!(
                                "http.{method} requires an async context: declare the handler with a `~UploadReceipt` return (~ awaits receive/commit)"
                            )));
                        }
                        // facade 形参为 impl AsRef<str>——字面量/String/&str
                        // 直传，无需收敛发射。
                        write!(out, "a2r_std::http::{}(", method)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ").await")?;
                        return Ok(());
                    }
                    ("http", "upload_metadata") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::upload_metadata(&")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("http", "upload_error") => {
                        self.a2r_std_used.set(true);
                        // facade message 形参为 impl AsRef<str>——直传。
                        write!(out, "a2r_std::http::upload_error(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("http", "file_response") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::file_response(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // PLAN-727 T-06: transfer_next_progress/cancel/error——非阻塞
                    // 观察面，sync/async 同形发射（&句柄 借用形）。
                    ("http", "transfer_next_progress")
                    | ("http", "transfer_cancel")
                    | ("http", "transfer_error") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::{}(&", method)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // Plan 349: file download/upload — direct a2r_std mapping.
                    ("http", "download") | ("http", "upload") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::{}(", method)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("http", "download_resume") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::download_resume(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                if i < 2 {
                                    self.expr_as_str(expr, out)?;
                                } else {
                                    self.expr(expr, out)?;
                                }
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("http", "post_stream_with_headers") => {
                        // http.post_stream_with_headers(url, body, headers) → facade
                        // (Plan 013 G6: returns an HTTPStream; PLAN-724: async 上下文
                        // 产出 AsyncHTTPStream。)
                        self.a2r_std_used.set(true);
                        let (fname, await_suffix) = if self.in_async_ctx.get() {
                            ("a2r_std::http::post_stream_with_headers_async", ".await")
                        } else {
                            ("a2r_std::http::post_stream_with_headers", "")
                        };
                        write!(out, "{}(", fname)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, "){}", await_suffix)?;
                        return Ok(());
                    }
                    ("json", "encode") | ("Json", "encode") => {
                        // Keep generic serialization behind the declared JSON producer.
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::json::encode(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr(a, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("str", "from_bytes") => {
                        // str.from_bytes(bytes) → a2r_std::str::from_bytes(bytes)
                        // (Plan 013 G6: UTF-8 lossy decode of an HTTP body.)
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::str::from_bytes(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr(a, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    ("process", "spawn") => {
                        // process.spawn(args) → a2r_std::process::spawn(args)
                        // (Plan 013 G6: detached spawn for daemon bootstrap. `args` is a
                        // Vec<String> whose [0] element is the program path.)
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::process::spawn(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr(a, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // Plan 223: Method call mappings for env.x / fs.x
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if let (Expr::Bina(inner_lhs, Op::Dot, inner_rhs), Expr::Ident(method)) =
                    (lhs.as_ref(), rhs.as_ref())
                {
                    // Handle auto.module.method(args) → a2r_std::module::method(args)
                    if let (Expr::Ident(auto_name), Expr::Ident(module)) =
                        (inner_lhs.as_ref(), inner_rhs.as_ref())
                    {
                        if auto_name == "auto" {
                            match (module.as_str(), method.as_str()) {
                                ("env", "get") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::env::get(")?;
                                    if let Some(arg) = call.args.args.first() {
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("env", "args") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::env::args()")?;
                                    return Ok(());
                                }
                                ("io", "read_line") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::io::read_line()")?;
                                    return Ok(());
                                }
                                // Plan 415-B1: auto.sqlite.* mirrors the bare
                                // `sqlite.*` dispatch (open takes &str).
                                ("sqlite", "open") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::sqlite::open(")?;
                                    if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                        write!(out, "&")?;
                                    }
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr_as_str(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("sqlite", "last_error") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::sqlite::last_error()")?;
                                    return Ok(());
                                }
                                // PLAN-714 r3 R3-T2: fs.metadata/copy_recursive
                                // （tuple 分发器——语句位 ns.method 调用面）。
                                ("fs", "metadata") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::file_size(")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr_as_str(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "copy_recursive") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::copy_recursive(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        if let Arg::Pos(expr) = arg {
                                            self.expr_as_str(expr, out)?;
                                        } else {
                                            self.arg(arg, out)?;
                                        }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "from_value") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::from_value(a2r_std::json!({{")?;
                                    if let Some(Arg::Pos(Expr::Object(pairs))) =
                                        call.args.args.first()
                                    {
                                        for (i, pair) in pairs.iter().enumerate() {
                                            if i > 0 {
                                                write!(out, ", ")?;
                                            }
                                            match &pair.key {
                                                crate::ast::Key::NamedKey(n) => {
                                                    write!(out, "\"{}\": ", n)?
                                                }
                                                crate::ast::Key::StrKey(k) => {
                                                    write!(out, "\"{}\": ", k)?
                                                }
                                                crate::ast::Key::IntKey(i) => {
                                                    write!(out, "\"{}\": ", i)?
                                                }
                                                crate::ast::Key::BoolKey(b) => {
                                                    write!(out, "\"{}\": ", b)?
                                                }
                                            }
                                            self.expr(&pair.value, out)?;
                                        }
                                    } else if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, "}})")?;
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("Regex", "test") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::re::test(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        if let Arg::Pos(expr) = arg {
                                            self.expr_as_str(expr, out)?;
                                        } else {
                                            self.arg(arg, out)?;
                                        }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("File", "read_bytes") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::read_bytes_list(")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr_as_str(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("File", "write_bytes") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::write_bytes_list(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        if let Arg::Pos(expr) = arg {
                                            self.expr(expr, out)?;
                                        } else {
                                            self.arg(arg, out)?;
                                        }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("env", "set") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::env::set(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "read_text") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::read_text(")?;
                                    if let Some(arg) = call.args.args.first() {
                                        if let Arg::Pos(a) = arg {
                                            self.expr_as_str(a, out)?;
                                        } else {
                                            self.arg(arg, out)?;
                                        }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "read_to_string") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::read_to_string(")?;
                                    if let Some(arg) = call.args.args.first() {
                                        // Plan 396 §2.3: a bare ident arg (a
                                        // PathBuf local/param) must be BORROWED —
                                        // moving it here breaks later uses
                                        // (E0382); &PathBuf coerces to &Path.
                                        if let Arg::Pos(Expr::Ident(_)) = arg {
                                            write!(out, "&")?;
                                        }
                                        if let Arg::Pos(a) = arg {
                                            self.expr_as_str(a, out)?;
                                        } else {
                                            self.arg(arg, out)?;
                                        }
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "write") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::write(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        if i == 1 {
                                            write!(out, "&")?;
                                        }
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "exists") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::exists(")?;
                                    // Plan 016 Phase A A.4: borrow arg as &str (fs.exists takes &str).
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr_as_str(a, out)?;
                                    } else if let Some(arg) = call.args.args.first() {
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("fs", "delete") | ("File", "delete") => {
                                    write!(out, "File::delete(")?;
                                    if let Some(arg) = call.args.args.first() {
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("http", "post") => {
                                    // auto.http.post(url, body, key) → wraps a2r_std::http::post into HttpResponse
                                    self.a2r_std_used.set(true);
                                    write!(out, "async {{ let (status, body, error, kind) = a2r_std::http::post(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        if let Arg::Pos(expr) = arg {
                                            self.expr(expr, out)?;
                                            if let Expr::Ident(name) = expr {
                                                if self
                                                    .local_var_types
                                                    .get(name)
                                                    .map(|ty| !matches!(ty, Type::StrSlice))
                                                    .unwrap_or(true)
                                                {
                                                    write!(out, ".as_str()")?;
                                                }
                                            }
                                        }
                                    }
                                    write!(
                                        out,
                                        ").await; HttpResponse {{ status, body, error, kind }} }}"
                                    )?;
                                    return Ok(());
                                }
                                ("http", "post_sync") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "{{ let __resp = a2r_std::http::post_sync(")?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        if let Arg::Pos(expr) = arg {
                                            self.expr(expr, out)?;
                                            let already_str =
                                                matches!(expr, Expr::Str(_) | Expr::CStr(_))
                                                    || if let Expr::Ident(name) = expr {
                                                        self.local_var_types
                                                            .get(name)
                                                            .map(|ty| matches!(ty, Type::StrSlice))
                                                            .unwrap_or(false)
                                                    } else {
                                                        false
                                                    };
                                            if !already_str {
                                                write!(out, ".as_str()")?;
                                            }
                                        }
                                    }
                                    write!(
                                        out,
                                        "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}"
                                    )?;
                                    return Ok(());
                                }
                                ("http", "last_status") => {
                                    // http.last_status() → a2r_std::http::last_status()
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::http::last_status()")?;
                                    return Ok(());
                                }
                                ("json", "parse") => {
                                    self.a2r_std_used.set(true);
                                    write!(
                                        out,
                                        "{}",
                                        if self.json_parse_as_opt {
                                            "a2r_std::json::parse_opt("
                                        } else {
                                            "a2r_std::json::parse("
                                        }
                                    )?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr_as_str(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "get") => {
                                    self.a2r_std_used.set(true);
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
                                ("json", "get_str") => {
                                    self.a2r_std_used.set(true);
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
                                ("json", "as_int") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::as_int(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "as_string") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::as_string(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "as_bool") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::as_bool(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "len") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::len(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "is_valid") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::is_valid(")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr_as_str(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                                ("json", "is_null") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::is_null(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
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
                        // PLAN-681: "Env" alias — capitalized VM builtin object
                        // form (Env.get), same normalization as Json→json above.
                        "env" | "Env" => match method.as_str() {
                            "get" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::env::get(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "set" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::env::set(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        // Plan 415-B1: sqlite module-level functions. `open`
                        // takes &str — borrow bare-ident String args (fs
                        // read_text precedent); `last_error` is nullary.
                        "sqlite" => match method.as_str() {
                            "open" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::sqlite::open(")?;
                                if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                    write!(out, "&")?;
                                }
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "last_error" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::sqlite::last_error()")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        // Plan 415-B2: redis module-level functions, mirroring
                        // the sqlite arm (open takes &str; last_error nullary).
                        "redis" => match method.as_str() {
                            "open" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::redis::open(")?;
                                if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                    write!(out, "&")?;
                                }
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "last_error" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::redis::last_error()")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "fs" => match method.as_str() {
                            "read_to_string" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::read_to_string(")?;
                                // Plan 396 §2.3: borrow bare-ident PathBuf args (see the
                                // (fs, read_to_string) dispatch above).
                                if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                    write!(out, "&")?;
                                }
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "read_text" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::read_text(")?;
                                if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                    write!(out, "&")?;
                                }
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "write" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::write(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        // Plan 368: borrow both path (i==0) and content (i==1)
                                        // as &str so owned strings (e.g. from concatenation) are
                                        // not moved out of scope at the call site.
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "exists" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::exists(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "create_dir" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::create_dir(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "metadata" => {
                                // PLAN-714 r3 R3-T2: fs.metadata→file_size（auto.fs.size
                                // int 字节长语义——缺失 -1，census §92 口径）。
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::file_size(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "copy_recursive" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::copy_recursive(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "write_text" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::write_text(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    // Plan 368: borrow both path (i==0) and content (i==1) as
                                    // &str (a2r_std::fs::write_text takes (&str, &str)). Borrowing
                                    // instead of moving lets an owned path String (e.g. built from
                                    // concatenation) be reused by a subsequent fs.read_text call.
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "append_text" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::append_text(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if i == 1 {
                                        write!(out, "&")?;
                                    }
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "is_dir" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::is_dir(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "is_binary" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::is_binary(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "file_size" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::file_size(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "walk" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::walk(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "tree" => {
                                // PLAN-681: fs.tree(path, depth) → a2r_std::fs::tree
                                // (JSON aligned with the VM fs_tree_walk shape).
                                // PLAN-681 T-05: host 签名 max_depth: i32——Auto
                                // int 实参（i64）窄化 as i32（宿主契约对齐）。
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::tree(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                if let Some(arg) = call.args.args.get(1) {
                                    write!(out, ", (")?;
                                    self.arg(arg, out)?;
                                    write!(out, ") as i32")?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "mkdir_all" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::mkdir_all(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "remove_file" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::remove_file(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "copy" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::copy(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        // time module: time.sleep(n) → a2r_std::sleep_ms(n as u64)
                        "time" => match method.as_str() {
                            "sleep" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::sleep_ms(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                    write!(out, " as u64")?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "now" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time_now()")?;
                                return Ok(());
                            }
                            "now_secs" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time::now_sec().to_string()")?;
                                return Ok(());
                            }
                            "now_sec" => {
                                // Plan 376U: bare numeric now_sec() (i32) for
                                // timestamp math (no .to_string() — caller casts).
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time::now_sec()")?;
                                return Ok(());
                            }
                            "now_ms" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time::now_ms()")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        // str module: str.uuid() → a2r_std::uuid(), str.from_uint(x) → x.to_string()
                        "str" => match method.as_str() {
                            "uuid" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::uuid()")?;
                                return Ok(());
                            }
                            "from_uint" | "from_int" => {
                                // str.from_uint(x) -> x.to_string()
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ".to_string()")?;
                                return Ok(());
                            }
                            "from_bytes" => {
                                // str.from_bytes(bytes) -> a2r_std::str::from_bytes(bytes)
                                // (Plan 013 G6: UTF-8 lossy decode of an HTTP body.)
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::str::from_bytes(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "Json" => match method.as_str() {
                            "parse" => {
                                // Json.parse(text) -> a2r_std::json::parse(text)
                                self.a2r_std_used.set(true);
                                write!(
                                    out,
                                    "{}",
                                    if self.json_parse_as_opt {
                                        "a2r_std::json::parse_opt("
                                    } else {
                                        "a2r_std::json::parse("
                                    }
                                )?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get" => {
                                // Json.get(val, key) -> a2r_std::json::get(&val, key)
                                self.a2r_std_used.set(true);
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
                                self.a2r_std_used.set(true);
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
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_string(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get_at" => {
                                // Json.get_at(val, idx) -> a2r_std::json::get_at(&val, idx as usize)
                                self.a2r_std_used.set(true);
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
                                self.a2r_std_used.set(true);
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
                            "as_int" => {
                                // Json.as_int(val) -> a2r_std::json::as_int(&val) as i32
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_int(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ") as i64")?;
                                return Ok(());
                            }
                            "is_null" => {
                                // Json.is_null(val) -> a2r_std::json::is_null(&val)
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::is_null(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "json" | "Json" => match method.as_str() {
                            "parse" => {
                                self.a2r_std_used.set(true);
                                write!(
                                    out,
                                    "{}",
                                    if self.json_parse_as_opt {
                                        "a2r_std::json::parse_opt("
                                    } else {
                                        "a2r_std::json::parse("
                                    }
                                )?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "from_value" => {
                                // PLAN-714 r3 R3-T2: json.from_value(struct-literal) →
                                // json! 构造（键加引号、值表达式直发——Vec<Value>/i64 均
                                // Serialize；宏经 a2r_std::json 再导出）。
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::from_value(a2r_std::json!({{")?;
                                if let Some(Arg::Pos(Expr::Object(pairs))) = call.args.args.first()
                                {
                                    for (i, pair) in pairs.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        match &pair.key {
                                            crate::ast::Key::NamedKey(n) => {
                                                write!(out, "\"{}\": ", n)?
                                            }
                                            crate::ast::Key::StrKey(k) => {
                                                write!(out, "\"{}\": ", k)?
                                            }
                                            crate::ast::Key::IntKey(i) => {
                                                write!(out, "\"{}\": ", i)?
                                            }
                                            crate::ast::Key::BoolKey(b) => {
                                                write!(out, "\"{}\": ", b)?
                                            }
                                        }
                                        self.expr(&pair.value, out)?;
                                    }
                                } else if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, "}})")?;
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "encode" => {
                                // Keep generic serialization behind the declared JSON producer.
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::encode(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get" => {
                                self.a2r_std_used.set(true);
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
                                self.a2r_std_used.set(true);
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
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_string(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "as_int" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_int(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ") as i64")?;
                                return Ok(());
                            }
                            "as_bool" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_bool(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "is_valid" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::is_valid(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "get_at" => {
                                self.a2r_std_used.set(true);
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
                            "keys" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::keys(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "len" => {
                                // Choose len_str (for &str) or len (for &Value) based on arg type
                                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                    let is_str_type = if let Expr::Ident(name) = expr {
                                        self.local_var_types
                                            .get(name)
                                            .map(|ty| {
                                                matches!(
                                                    ty,
                                                    Type::StrSlice
                                                        | Type::StrOwned
                                                        | Type::StrFixed(_)
                                                )
                                            })
                                            .unwrap_or(true) // default to str for unknown vars
                                    } else {
                                        matches!(expr, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                                    };
                                    if is_str_type {
                                        self.a2r_std_used.set(true);
                                        write!(out, "a2r_std::json::len_str(")?;
                                    } else {
                                        self.a2r_std_used.set(true);
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
                                        self.local_var_types
                                            .get(name)
                                            .map(|ty| {
                                                matches!(
                                                    ty,
                                                    Type::StrSlice
                                                        | Type::StrOwned
                                                        | Type::StrFixed(_)
                                                )
                                            })
                                            .unwrap_or(true)
                                    } else {
                                        matches!(
                                            first,
                                            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_)
                                        )
                                    };
                                    if use_str {
                                        self.a2r_std_used.set(true);
                                        write!(out, "a2r_std::json::has_key_str(")?;
                                    } else {
                                        self.a2r_std_used.set(true);
                                        write!(out, "a2r_std::json::has_key(&")?;
                                    }
                                    self.expr(first, out)?;
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
                            "as_int" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_int(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ") as i64")?;
                                return Ok(());
                            }
                            "is_null" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::is_null(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "type_of" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::value_type(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "http" => match method.as_str() {
                            "post" => {
                                self.a2r_std_used.set(true);
                                write!(out, "async {{ let (status, body, error, kind) = a2r_std::http::post(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if let Expr::Ident(name) = expr {
                                            if self
                                                .local_var_types
                                                .get(name)
                                                .map(|ty| !matches!(ty, Type::StrSlice))
                                                .unwrap_or(true)
                                            {
                                                write!(out, ".as_str()")?;
                                            }
                                        }
                                    }
                                }
                                write!(
                                    out,
                                    ").await; HttpResponse {{ status, body, error, kind }} }}"
                                )?;
                                return Ok(());
                            }
                            "post_sync" => {
                                self.a2r_std_used.set(true);
                                write!(out, "{{ let __resp = a2r_std::http::post_sync(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if let Expr::Ident(name) = expr {
                                            if self
                                                .local_var_types
                                                .get(name)
                                                .map(|ty| !matches!(ty, Type::StrSlice))
                                                .unwrap_or(true)
                                            {
                                                write!(out, ".as_str()")?;
                                            }
                                        }
                                    }
                                }
                                write!(
                                    out,
                                    "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}"
                                )?;
                                return Ok(());
                            }
                            "last_status" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::http::last_status()")?;
                                return Ok(());
                            }
                            "post_bearer" => {
                                self.a2r_std_used.set(true);
                                write!(out, "{{ let __resp = a2r_std::http::post_bearer(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                        if let Expr::Ident(name) = expr {
                                            if self
                                                .local_var_types
                                                .get(name)
                                                .map(|ty| !matches!(ty, Type::StrSlice))
                                                .unwrap_or(true)
                                            {
                                                write!(out, ".as_str()")?;
                                            }
                                        }
                                    }
                                }
                                write!(
                                    out,
                                    "); a2r_std::http::set_last_status(__resp.0); __resp.1 }}"
                                )?;
                                return Ok(());
                            }
                            "request" => {
                                // http.request(method, url) → a2r_std::http::request(method, url)
                                // (Plan 013 G6: returns a RequestBuilder for chained .header/.body/.timeout/.send.)
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::http::request(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "post_stream_with_headers" => {
                                // http.post_stream_with_headers(url, body, headers) → a2r_std::http::post_stream_with_headers(...)
                                // (Plan 013 G6: returns an HTTPStream for SSE.)
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::http::post_stream_with_headers(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "shell" => match method.as_str() {
                            "exec" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::shell::exec(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
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
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::re::r#match(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
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

        // Binary-dot module calls: str.uuid() → a2r_std::uuid(), str.from_uint(x) → x.to_string(), etc.
        // Handles both Expr::Bina(_, Dot, _) and Expr::Dot(_, _) AST forms.
        {
            let maybe_module_method: Option<(&Expr, &Name)> = match call.name.as_ref() {
                Expr::Bina(lhs, op, rhs) if matches!(op, Op::Dot) => {
                    if let Expr::Ident(method) = rhs.as_ref() {
                        Some((lhs.as_ref(), method))
                    } else {
                        None
                    }
                }
                Expr::Dot(obj, field) => Some((obj.as_ref(), field)),
                _ => None,
            };
            if let Some((obj, method_name)) = maybe_module_method {
                if let Expr::Ident(module) = obj {
                    match module.as_str() {
                        // File module (Plan 514 W1 / P511-5): `File.*` is the
                        // canonical VM name family (auto.file.*) — map it to
                        // INLINE std calls instead of a2r_std, because the
                        // merge-mode output (parity matrix leg ② compiles the
                        // merged lib standalone with zero deps) cannot link
                        // a2r_std. Error convention matches the VM shim
                        // (ffi/stdlib.rs shim_file_*: read error → empty
                        // string, write → 0/-1, exists → bool).
                        // PLAN-714 r3 R3-T2: Regex.test(text, pat)→bool（VM 实参序）。
                        "Regex" => match method_name.as_str() {
                            "test" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::re::test(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "replace" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::re::replace(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "File" => match method_name.as_str() {
                            "read_text" | "read_to_string" => {
                                write!(out, "std::fs::read_to_string(")?;
                                if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                    write!(out, "&")?;
                                }
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ").unwrap_or_default()")?;
                                return Ok(());
                            }
                            "write_text" | "write" => {
                                write!(out, "if std::fs::write(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    write!(out, "&")?;
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ").is_ok() {{ 0 }} else {{ -1 }}")?;
                                return Ok(());
                            }
                            "exists" => {
                                write!(out, "std::fs::metadata(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ").is_ok()")?;
                                return Ok(());
                            }
                            "read_bytes" => {
                                // PLAN-714 r3 R3-T2: read_bytes 的 list 形（Vec<Value>
                                // ——VM int-list 同源；配 `list` 动态型映射）。
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::read_bytes_list(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "write_bytes" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::write_bytes_list(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            // PLAN-687: chunked-read envelope — true chunked
                            // window read in a2r_std (byte-identical with the
                            // VM shim, P670-D1). Auto int = i64 → usize cast.
                            "read_text_range" => {
                                self.a2r_std_used.set(true);
                                write!(out, "auto_lang::a2r_std::fs::read_text_range(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    write!(out, "&(")?;
                                    self.expr_as_str(a, out)?;
                                    write!(out, ")")?;
                                }
                                for (i, arg) in call.args.args.iter().enumerate().skip(1) {
                                    write!(out, ", ")?;
                                    if let Arg::Pos(e) = arg {
                                        write!(out, "((")?;
                                        self.expr(e, out)?;
                                        write!(out, ").max(0) as u64 as usize)")?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        // PLAN-714 r3 R3-T2: fs.metadata/copy_recursive（Dot
                        // 分发器——corpus fsys.at 两段形调用面）。
                        "fs" => match method_name.as_str() {
                            "metadata" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::file_size(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "copy_recursive" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::copy_recursive(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    if let Arg::Pos(expr) = arg {
                                        self.expr_as_str(expr, out)?;
                                    } else {
                                        self.arg(arg, out)?;
                                    }
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "remove_dir" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::remove_dir(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "list_dir" => {
                                // VM fs.list_dir → JSON 目录清单——a2r 轨走
                                // fs::walk 的 JSON 串形（Plan 626 面同源）。
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::walk(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        // PLAN-714 r3 R3-T2: json.from_value(struct-literal)→
                        // json! 构造（Dot 分发器）。
                        "json" => match method_name.as_str() {
                            "from_value" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::from_value(a2r_std::json!({{")?;
                                if let Some(Arg::Pos(Expr::Object(pairs))) = call.args.args.first()
                                {
                                    for (i, pair) in pairs.iter().enumerate() {
                                        if i > 0 {
                                            write!(out, ", ")?;
                                        }
                                        match &pair.key {
                                            crate::ast::Key::NamedKey(n) => {
                                                write!(out, "\"{}\": ", n)?
                                            }
                                            crate::ast::Key::StrKey(k) => {
                                                write!(out, "\"{}\": ", k)?
                                            }
                                            crate::ast::Key::IntKey(i) => {
                                                write!(out, "\"{}\": ", i)?
                                            }
                                            crate::ast::Key::BoolKey(b) => {
                                                write!(out, "\"{}\": ", b)?
                                            }
                                        }
                                        self.expr(&pair.value, out)?;
                                    }
                                } else if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, "}})")?;
                                write!(out, ")")?;
                                return Ok(());
                            }
                            "to_value" => {
                                // json.to_value(json-text) → list（VM 语义）——
                                // a2r 轨=parse_list 的 Vec<Value>（for 直迭代面）。
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::parse_str_list(&(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, "))")?;
                                return Ok(());
                            }
                            _ => {}
                        },
                        "sys" => {
                            // PLAN-668 R-24（P666-D1 同族）：裸 `sys.*` 调用
                            // （VM 预绑原生面，语料无 use）→ a2r_std::sys 同名
                            // 直通（函数名一一对应；a2r_std_used 置位带上
                            // 依赖）。025 sys-monitor 后端 api.rs 实案。
                            // 边界宽度：Auto int = i64，a2r_std::sys 的
                            // 计数/MB/uptime 族返回 i32、core/kill 收 i32 参
                            // ——出入双侧强转（025 cores 循环 ci 实案）。
                            self.a2r_std_used.set(true);
                            let ret_i32 = matches!(
                                method_name.as_str(),
                                "cpu_count" | "mem_total_mb" | "mem_used_mb" | "uptime_s"
                            );
                            let arg_i32_first =
                                matches!(method_name.as_str(), "cpu_core_usage" | "kill");
                            if ret_i32 {
                                write!(out, "((a2r_std::sys::{}(", method_name)?;
                            } else {
                                write!(out, "(a2r_std::sys::{}(", method_name)?;
                            }
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if arg_i32_first && i == 0 {
                                    write!(out, "(")?;
                                    self.arg(arg, out)?;
                                    write!(out, ") as i32")?;
                                } else {
                                    self.arg(arg, out)?;
                                }
                            }
                            if ret_i32 {
                                write!(out, ")) as i64)")?;
                            } else {
                                write!(out, "))")?;
                            }
                            return Ok(());
                        }
                        "String" => {
                            // Plan 019 Phase 1: String.fromCharCode(n) → a
                            // one-char String from the code point (report B8).
                            if method_name.as_str() == "fromCharCode" {
                                write!(out, "char::from_u32(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    write!(out, "(")?;
                                    self.expr(a, out)?;
                                    write!(out, ") as u32")?;
                                } else {
                                    write!(out, "0")?;
                                }
                                write!(out, ").unwrap_or('\\u{{0}}').to_string()")?;
                                return Ok(());
                            }
                        }
                        "str" => match method_name.as_str() {
                            "uuid" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::uuid()")?;
                                return Ok(());
                            }
                            "from_uint" | "from_int" => {
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ".to_string()")?;
                                return Ok(());
                            }
                            "to_uint" => {
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ".parse::<u64>().unwrap_or(0)")?;
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
                            // Plan 514 W1 (洞 B): Map.set(key, val) ->
                            // HashMap::insert(key, val) — but only for
                            // non-user-type receivers. A type method named
                            // `set` (99_idiom2/m09 Holder.set) must pass
                            // through unchanged (Plan 393 E1 append guard).
                            let lhs_is_struct = if let Expr::Ident(name) = lhs.as_ref() {
                                self.local_var_types
                                    .get(name)
                                    .map(|ty| {
                                        matches!(
                                            ty,
                                            Type::User(_)
                                                | Type::Tag(_)
                                                | Type::Enum(_)
                                                | Type::GenericInstance(_)
                                        )
                                    })
                                    .unwrap_or(false)
                            } else {
                                false
                            };
                            if lhs_is_struct {
                                self.expr(lhs, out)?;
                                write!(out, ".set(")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        write!(out, ", ")?;
                                    }
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(());
                            }
                            self.expr(lhs, out)?;
                            write!(out, ".insert(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                self.arg(arg, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "contains" => {
                            // Only convert to contains_key if lhs is a known Map variable.
                            // For other cases (e.g., plan.content which is String), fall through
                            // to the later handler which decides based on type info.
                            if let Expr::Ident(name) = lhs.as_ref() {
                                let is_map = self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::Map(_, _)))
                                    .unwrap_or(false);
                                if is_map {
                                    self.expr(lhs, out)?;
                                    write!(out, ".contains_key(")?;
                                    if let Some(arg) = call.args.args.first() {
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(());
                                }
                            }
                            // Fall through — don't intercept, let later code handle it
                        }
                        // Plan 347: Auto VM exposes integer bitwise operations as
                        // methods on int (`.and`, `.or`, `.xor`, `.shl`, `.shr`,
                        // `.sar`, `.not`). Rust has no inherent methods with these
                        // names on integers, so map them to the equivalent Rust
                        // operator expressions. The VM uses wrapping/unsigned
                        // semantics (see vm/native.rs shims), which we mirror here.
                        "and" | "or" | "xor" => {
                            // val.and(mask) -> (val & mask), etc.
                            let op = match method_name.as_str() {
                                "and" => "&",
                                "or" => "|",
                                _ => "^", // xor
                            };
                            write!(out, "(")?;
                            self.expr(lhs, out)?;
                            write!(out, " {} (", op)?;
                            if let Some(Arg::Pos(arg)) = call.args.args.first() {
                                self.expr(arg, out)?;
                            }
                            write!(out, ") as i64))")?;
                            return Ok(());
                        }
                        "shl" => {
                            // val.shl(n) -> val.wrapping_shl(n as u32) (wrapping)
                            self.expr(lhs, out)?;
                            write!(out, ".wrapping_shl((")?;
                            if let Some(Arg::Pos(arg)) = call.args.args.first() {
                                self.expr(arg, out)?;
                            }
                            write!(out, ") as u32) as i64")?;
                            return Ok(());
                        }
                        "shr" => {
                            // val.shr(n) -> LOGICAL (unsigned) right shift:
                            // ((val as u32) >> (n as u32)) as i32
                            write!(out, "(((")?;
                            self.expr(lhs, out)?;
                            write!(out, ") as u32) >> (")?;
                            if let Some(Arg::Pos(arg)) = call.args.args.first() {
                                self.expr(arg, out)?;
                            }
                            write!(out, ") as u32)) as i64")?;
                            return Ok(());
                        }
                        "sar" => {
                            // val.sar(n) -> ARITHMETIC right shift: (val >> n)
                            write!(out, "(")?;
                            self.expr(lhs, out)?;
                            write!(out, ".wrapping_shr((")?;
                            if let Some(Arg::Pos(arg)) = call.args.args.first() {
                                self.expr(arg, out)?;
                            }
                            write!(out, ") as u32)) as i64")?;
                            return Ok(());
                        }
                        "not" => {
                            // val.not() -> (!val), no arguments.
                            write!(out, "(!")?;
                            self.expr(lhs, out)?;
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "char_at" => {
                            // s.char_at(i) -> s.chars().nth((i) as usize).unwrap_or('\0') as i32
                            // Plan 347: Auto's char_at returns the code point as
                            // an i32 (not a char), so the Rust equivalent must
                            // cast the char to i32. The index expression is also
                            // wrapped in parens before `as usize` because `as`
                            // binds tighter than `+`, so `i + 1 as usize` would
                            // parse as `i + (1 as usize)` (type error) instead of
                            // `(i + 1) as usize`.
                            self.expr(lhs, out)?;
                            write!(out, ".chars().nth((")?;
                            if let Some(Arg::Pos(arg)) = call.args.args.first() {
                                self.expr(arg, out)?;
                            }
                            write!(out, ") as usize).unwrap_or('\\0') as i64")?;
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
                            // Plan 019 Phase 1: string receivers slice by
                            // CHARS (Auto str positions are code-unit based);
                            // non-string receivers keep the byte-index form.
                            if self.expr_is_string_like(lhs) {
                                return self.emit_str_slice_chars(lhs, &call.args.args, out);
                            }
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
                            // Plan 016 Phase A A4 cat 7: borrow lhs (e.g. self.buf)
                            // via as_str to avoid moving a &mut self field (E0507).
                            // str_find accepts AsRef<str> so &str/&String both work.
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::str_find(")?;
                            self.expr_as_str(lhs, out)?;
                            for arg in &call.args.args {
                                write!(out, ", ")?;
                                self.arg(arg, out)?;
                            }
                            // Default start_pos = 0 if not provided
                            if call.args.args.len() < 2 {
                                write!(out, ", 0")?;
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
                        // Tuple field access: .get_0() -> .0, .get_1() -> .1, etc.
                        "get_0" => {
                            self.expr(lhs, out)?;
                            write!(out, ".0")?;
                            return Ok(());
                        }
                        "get_1" => {
                            self.expr(lhs, out)?;
                            write!(out, ".1")?;
                            return Ok(());
                        }
                        "get_2" => {
                            self.expr(lhs, out)?;
                            write!(out, ".2")?;
                            return Ok(());
                        }
                        _ => {} // fall through to simple name-remap table
                    }

                    // Plan 433 A1: struct `len` field read (Tok.len) — property,
                    // not the collection-length method.
                    if self.try_emit_struct_len_field(
                        lhs,
                        method_name,
                        call.args.args.len(),
                        out,
                    )? {
                        return Ok(());
                    }

                    // Simple name-remap table
                    // .len()/.length() returns usize, cast to i32 for Auto's int
                    // Plan 415-E1: the int cast only holds for the ZERO-ARG
                    // collection-length form. A `.len(n)` WITH an argument is a
                    // foreign builder setter (e.g. memmap2 MmapOptions::len)
                    // returning Self — casting that to i64 breaks the chain
                    // (`(opts.len(4) as i64).map_anon()` does not compile).
                    let needs_i32_cast_1 = matches!(method_name.as_str(), "len" | "length")
                        && call.args.args.is_empty();
                    let rust_method = match method_name.as_str() {
                        // String methods
                        "to_lower" | "lower" => Some("to_lowercase"),
                        "to_upper" | "upper" => Some("to_uppercase"),
                        "length" | "len" => Some("len"),
                        "is_empty" => Some("is_empty"),
                        "trim" => Some("trim"),
                        "trim_left" => Some("trim_start"),
                        "trim_right" => Some("trim_end"),
                        "starts_with" => Some("starts_with"),
                        "ends_with" => Some("ends_with"),
                        "find_last" => Some("rfind"),
                        "to_str" => Some("to_str"),
                        "append" => Some("push_str"),
                        // Collection methods
                        "push" => Some("push"),
                        "pop" => Some("pop"),
                        "drop" => Some("take"),
                        "clear" => Some("clear"),
                        "to_array" => Some("clone"),
                        "contains" => Some("contains"),
                        "retain" => Some("retain"),
                        // Type conversion
                        "to_string" => Some("to_string"),
                        // Plan 433 A1: Auto's universal `.str()` stringify (VM native)
                        // maps to Rust's Display to_string.
                        "str" => Some("to_string"),
                        // Plan 384 A9: `.delete()` is no longer unconditionally
                        // remapped to `.remove()` — that breaks axum Router
                        // `.delete(handler)`. Auto code that wants HashMap
                        // removal should call `.remove()` explicitly.
                        "delete" => Some("delete"),
                        _ => None,
                    };

                    if let Some(rust_name) = rust_method {
                        let lhs_parens = matches!(lhs.as_ref(),
                            Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
                        ) || Self::is_char_at_call(lhs.as_ref());
                        if lhs_parens {
                            write!(out, "(")?;
                        }
                        self.expr(lhs, out)?;
                        if lhs_parens {
                            write!(out, ")")?;
                        }
                        write!(out, ".{}(", rust_name)?;
                        // Auto-borrow string args for pattern-matching methods
                        // P532 W1 G19: replace 族同入(String::replace 首参
                        // Pattern 界;次参 &str——&String 双参 coerce 均合;
                        // ⑤腿 engine nat#1510 变量实参 E0277 实证)
                        if matches!(
                            method_name.as_str(),
                            "contains" | "starts_with" | "ends_with" | "replace" | "replace_first"
                        ) {
                            for (i, arg) in call.args.args.iter().enumerate() {
                                // Plan 380: char/&str literals are already valid
                                // Patterns — `&'"'` would be `&char` (E0277).
                                let already_pattern = matches!(
                                    arg,
                                    Arg::Pos(Expr::Char(_))
                                        | Arg::Pos(Expr::Str(_))
                                        | Arg::Pos(Expr::CStr(_))
                                );
                                if !already_pattern {
                                    write!(out, "&")?;
                                }
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
                        if needs_i32_cast_1 && !self.len_i32_cast_suppressed {
                            write!(out, " as i64")?;
                        }
                        // trim/trim_start/trim_end return &str, auto-convert to String
                        // Plan 380: skip when the callee `trim` returns void
                        // (e.g. Memory.trim() — `.to_string()` on `()` is E0599).
                        if matches!(method_name.as_str(), "trim" | "trim_left" | "trim_right") {
                            let trim_ret_is_void = self
                                .fn_ret_types
                                .get(method_name.as_str())
                                .map(|t| matches!(t, Type::Void))
                                .unwrap_or(false);
                            if !trim_ret_is_void {
                                write!(out, ".to_string()")?;
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }

        // Also handle Expr::Dot method calls (parser emits Dot for method calls)
        if let Expr::Dot(object, method_name) = call.name.as_ref() {
            // Plan 371 (defect B): Optional method dispatch. Auto's VM lets you
            // call `.as_string()` on a `JsonValue?` (None -> "", Some(v) -> v.as_string()).
            // Rust's Option has no such method. The common pattern is
            // `<json>.get(key).as_string()` where get returns Option<&Value>.
            // Detect that and emit a serde_json-compatible chain (the receiver
            // `args.get("path")` returns Option<&serde_json::Value>, not
            // auto_val::Value, so a2r_std::json::as_string_opt won't type-check).
            if method_name.as_str() == "as_string"
                && call.args.args.is_empty()
                && matches!(object.as_ref(), Expr::Call(c) if matches!(c.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "get"))
            {
                self.expr(object, out)?;
                write!(
                    out,
                    ".and_then(|v| v.as_str()).unwrap_or_default().to_string()"
                )?;
                return Ok(());
            }
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
                // list.get(i) -> list[i as usize].clone() for Auto List only
                // Rust Vec/HashMap .get() falls through to generic method call handler
                "get" => {
                    // Plan 415-B2: RedisClient.get(key) → a2r-std inherent
                    // method (&str borrow, sqlite exec precedent). Guarded
                    // inside this arm — a preceding arm would shadow the
                    // List-indexing logic below.
                    if call.args.args.len() == 1 {
                        let is_redis = if let Expr::Ident(name) = object.as_ref() {
                            self.local_var_types.get(name)
                                .map(|ty| matches!(ty, Type::User(usr) if usr.name.as_str() == "RedisClient"))
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        if is_redis {
                            self.a2r_std_used.set(true);
                            self.expr(object, out)?;
                            write!(out, ".get(")?;
                            if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                write!(out, "&")?;
                            }
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                    }
                    if call.args.args.len() == 1 {
                        if let Some(Arg::Pos(arg)) = call.args.args.first() {
                            // Plan 433 A1: resolve List-typed locals/params AND
                            // List-typed struct fields (`c.ins.get(i)`). A List
                            // .get index is always numeric in Auto semantics,
                            // so any single-arg .get on a List receiver indexes.
                            let is_auto_list = self.is_auto_list_expr(object);
                            if is_auto_list {
                                self.expr(object, out)?;
                                write!(out, "[(")?;
                                self.expr(arg, out)?;
                                write!(out, ") as usize]")?;
                                // C11 (Plan 018 §12 a2r-11): on an assignment LHS
                                // (in-place element mutation) skip `.clone()` —
                                // writing to the clone would be a no-op.
                                if self.assign_lhs_depth == 0 {
                                    write!(out, ".clone()")?;
                                }
                                return Ok(());
                            }
                        }
                    }
                }
                // Plan 347: Auto VM exposes integer bitwise operations as
                // methods on int (`.and`, `.or`, `.xor`, `.shl`, `.shr`,
                // `.sar`, `.not`). Rust has no inherent methods with these
                // names on integers, so map them to the equivalent Rust
                // operator expressions. The VM uses wrapping/unsigned
                // semantics (see vm/native.rs shims), which we mirror here.
                // NOTE: this `Expr::Dot` dispatch is the path the parser
                // actually emits for method calls (the `Expr::Bina` match
                // above is dead code kept for completeness).
                "and" | "or" | "xor" => {
                    // val.and(mask) -> ((val & mask) as i64), etc.
                    // Cast to i32 mirrors Auto's int type after the bitwise op.
                    let op = match method_name.as_str() {
                        "and" => "&",
                        "or" => "|",
                        _ => "^", // xor
                    };
                    write!(out, " ((")?;
                    self.expr(object, out)?;
                    write!(out, " {} ", op)?;
                    if let Some(Arg::Pos(arg)) = call.args.args.first() {
                        self.expr(arg, out)?;
                    }
                    write!(out, ") as i64)")?;
                    return Ok(());
                }
                "shl" => {
                    // val.shl(n) -> (val.wrapping_shl(n as u32) as i64) (wrapping)
                    write!(out, " (")?;
                    self.expr(object, out)?;
                    write!(out, ".wrapping_shl(")?;
                    if let Some(Arg::Pos(arg)) = call.args.args.first() {
                        self.expr(arg, out)?;
                    }
                    write!(out, " as u32) as i64)")?;
                    return Ok(());
                }
                "shr" => {
                    // val.shr(n) -> LOGICAL (unsigned) right shift:
                    // ((val as u32).wrapping_shr(n as u32) as i64)
                    // Casting to u32 first makes wrapping_shr unsigned (logical),
                    // matching Auto's `shr` semantics.
                    write!(out, " ((")?;
                    self.expr(object, out)?;
                    write!(out, " as u32).wrapping_shr(")?;
                    if let Some(Arg::Pos(arg)) = call.args.args.first() {
                        self.expr(arg, out)?;
                    }
                    write!(out, " as u32) as i64)")?;
                    return Ok(());
                }
                "sar" => {
                    // val.sar(n) -> ARITHMETIC (signed) right shift:
                    // (val.wrapping_shr(n as u32) as i64)
                    write!(out, " (")?;
                    self.expr(object, out)?;
                    write!(out, ".wrapping_shr(")?;
                    if let Some(Arg::Pos(arg)) = call.args.args.first() {
                        self.expr(arg, out)?;
                    }
                    write!(out, " as u32) as i64)")?;
                    return Ok(());
                }
                "not" => {
                    // val.not() -> (!val), no arguments.
                    write!(out, "(!")?;
                    self.expr(object, out)?;
                    write!(out, ")")?;
                    return Ok(());
                }
                // PLAN-724 T-06：HTTP 流接收者方法（next/is_done/close）。
                // async 分型变量走 stream_next_async（"" 哨兵与 VM 契约一致，
                // 不暴露内核 Option 形）；is_done/close 对两种 facade 均为
                // &self 方法，直发。非流接收者原样回落通用路径（不全局改写
                // 用户 next/close）。
                "next" | "is_done" | "close" => {
                    let stream_kind = if let Expr::Ident(name) = object.as_ref() {
                        if self.http_stream_async_vars.contains(name) {
                            Some(true)
                        } else if self.http_stream_vars.contains(name) {
                            Some(false)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(is_async) = stream_kind {
                        self.a2r_std_used.set(true);
                        match (method_name.as_str(), is_async) {
                            ("next", false) => {
                                self.expr(object, out)?;
                                write!(out, ".next()")?;
                            }
                            ("next", true) => {
                                write!(out, "a2r_std::http::stream_next_async(&")?;
                                self.expr(object, out)?;
                                write!(out, ").await")?;
                            }
                            ("is_done", false) => {
                                self.expr(object, out)?;
                                write!(out, ".is_done()")?;
                            }
                            ("is_done", true) => {
                                self.expr(object, out)?;
                                write!(out, ".is_done()")?;
                            }
                            _ => {
                                // close：两种 facade 均为 &self 方法。
                                self.expr(object, out)?;
                                write!(out, ".close()")?;
                            }
                        }
                        return Ok(());
                    }
                    // 非 HTTP 流接收者：回落通用路径。
                }
                // PLAN-727 T-06：builder multipart 方法——typed facade 方法
                // 直发（路径/文本以 str 形；预算与流式语义在 facade 内）。
                "multipart_file" | "multipart_text" => {
                    let is_builder = match object.as_ref() {
                        Expr::Ident(name) => self.http_builder_vars.contains(name),
                        other => {
                            expr_root_is_http_request(other)
                                || expr_root_is_builder_var(self, other)
                        }
                    };
                    if is_builder {
                        self.a2r_std_used.set(true);
                        self.expr(object, out)?;
                        write!(out, ".{}(", method_name)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            if let Arg::Pos(expr) = arg {
                                self.expr_as_str(expr, out)?;
                            } else {
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // 非 builder 接收者：回落通用路径。
                }
                // PLAN-724 T-06：builder .send 分派——链根是 http.request(...)
                // 或变量绑定自其。async 上下文发射 send_async().await。
                "send" => {
                    let is_builder = match object.as_ref() {
                        Expr::Ident(name) => self.http_builder_vars.contains(name),
                        other => {
                            expr_root_is_http_request(other)
                                || expr_root_is_builder_var(self, other)
                        }
                    };
                    if is_builder {
                        self.a2r_std_used.set(true);
                        self.expr(object, out)?;
                        if self.in_async_ctx.get() {
                            write!(out, ".send_async().await")?;
                        } else {
                            write!(out, ".send()")?;
                        }
                        return Ok(());
                    }
                    // 非 builder 接收者：回落通用路径。
                }
                // Plan 347: StringBuilder method dispatch. The a2r-std
                // `StringBuilder` runtime type exposes methods with the same
                // names as the Auto VM API (`append`, `append_char`, `build`),
                // so we only need to bypass the generic name-remap table (which
                // would otherwise rewrite `.append(s)` to `.push_str(s)`) for
                // receivers whose type is StringBuilder. `append_char(code)`
                // takes an i32 code point in Auto.
                "append" | "append_char" | "build" | "clear" => {
                    let is_sb = if let Expr::Ident(name) = object.as_ref() {
                        self.local_var_types.get(name)
                            .map(|ty| matches!(ty, Type::User(usr) if usr.name.as_str() == "StringBuilder"))
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if is_sb {
                        self.a2r_std_used.set(true);
                        self.expr(object, out)?;
                        write!(out, ".{}(", method_name)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            self.arg(arg, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // Not a StringBuilder — fall through to the generic remap.
                }
                // Plan 415-B1: sqlite handle methods. The a2r-std
                // `SqliteDb` wrapper exposes inherent methods with the same
                // names as the Auto surface, so emit the receiver-typed call
                // directly (bypassing the generic method path, which may
                // rewrite Auto-declared `SqliteDb.*` methods into module
                // free functions). `exec` takes the sql as &str.
                "exec" | "query" | "last_insert_rowid" => {
                    let is_sqlitedb = if let Expr::Ident(name) = object.as_ref() {
                        self.local_var_types.get(name)
                            .map(|ty| matches!(ty, Type::User(usr) if usr.name.as_str() == "SqliteDb"))
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if is_sqlitedb {
                        self.a2r_std_used.set(true);
                        self.expr(object, out)?;
                        write!(out, ".{}(", method_name)?;
                        if method_name == "exec" {
                            if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                write!(out, "&")?;
                            }
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                        } else {
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                self.arg(arg, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // Not a SqliteDb — fall through to the generic path.
                }
                // Plan 415-B2: redis client methods with NON-colliding names
                // get their own arm. `get`/`set` collide with pre-existing
                // arms of this match (List indexing / Map::insert rewrite
                // live inside those arm bodies) — a preceding arm here would
                // shadow them, so their RedisClient guards are embedded at
                // the top of the existing `"get"`/`"set"` arms instead.
                // The receiver must be a `var` binding: redis-rs connection
                // ops take &mut self, mirrored by Auto var → `let mut`.
                "ping" | "del" | "exists" => {
                    let is_redis = if let Expr::Ident(name) = object.as_ref() {
                        self.local_var_types.get(name)
                            .map(|ty| matches!(ty, Type::User(usr) if usr.name.as_str() == "RedisClient"))
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if is_redis {
                        self.a2r_std_used.set(true);
                        self.expr(object, out)?;
                        write!(out, ".{}(", method_name)?;
                        if method_name != "ping" {
                            if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                write!(out, "&")?;
                            }
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // Not a RedisClient — fall through to the generic path.
                }
                // Plan 204 Phase 5: Complex method translations requiring
                // non-trivial Rust output (not just a name remap).
                "char_at" => {
                    // s.char_at(i) -> s.chars().nth((i) as usize).unwrap_or('\0') as i32
                    // Plan 347: Auto's char_at returns the code point as an i32
                    // (not a char), so the Rust equivalent must cast the char to
                    // i32. The index expression is wrapped in parens before
                    // `as usize` because `as` binds tighter than `+`, so
                    // `i + 1 as usize` would parse as `i + (1 as usize)` (type
                    // error) instead of `(i + 1) as usize`.
                    self.expr(object, out)?;
                    write!(out, ".chars().nth((")?;
                    if let Some(Arg::Pos(arg)) = call.args.args.first() {
                        self.expr(arg, out)?;
                    }
                    write!(out, ") as usize).unwrap_or('\\0') as i64")?;
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
                    // Plan 019 Phase 1: string receivers slice by CHARS;
                    // non-string receivers keep the byte-index form.
                    if self.expr_is_string_like(object) {
                        return self.emit_str_slice_chars(object, &call.args.args, out);
                    }
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
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        Expr::Ident(name) => {
                            self.local_var_types
                                .get(name)
                                .map(|ty| {
                                    matches!(
                                        ty,
                                        Type::User(_)
                                            | Type::Enum(_)
                                            | Type::Tag(_)
                                            | Type::GenericInstance(_)
                                            | Type::Void
                                    )
                                })
                                .unwrap_or(false)
                                || self.json_value_vars.contains(name.as_str())
                        }
                        _ => false,
                    };
                    if use_value_helper {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::value_to_int(&")?;
                        self.expr(object, out)?;
                        write!(out, ")")?;
                    } else {
                        self.expr(object, out)?;
                        write!(out, ".parse::<i64>().ok()")?;
                    }
                    return Ok(());
                }
                "len" | "length" => {
                    // Plan 433 A1: bare `.len` on a known struct with a `len`
                    // FIELD (Tok.len) parses as a zero-arg method call but is a
                    // property read — emit the field access, not Vec::len().
                    if method_name.as_str() == "len" && call.args.args.is_empty() {
                        if let Expr::Ident(name) = object.as_ref() {
                            if let Some(Type::User(td)) = self.local_var_types.get(name.as_str()) {
                                let has_len_field = self
                                    .struct_field_types
                                    .get(td.name.as_str())
                                    .map(|fields| fields.iter().any(|(f, _)| f.as_str() == "len"))
                                    .unwrap_or(false);
                                if has_len_field {
                                    self.expr(object, out)?;
                                    write!(out, ".len")?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                    // Skip if object is a known stdlib module — handled by Expr::Ident block below.
                    // But if the name is a known local variable (e.g. param named "json"), it's NOT a module.
                    let is_stdlib_module = if let Expr::Ident(name) = object.as_ref() {
                        let name_is_local = self.local_var_types.contains_key(name);
                        !name_is_local
                            && matches!(
                                name.as_str(),
                                "json" | "Json" | "shell" | "fs" | "regex" | "env" | "http"
                            )
                    } else {
                        false
                    };

                    if !is_stdlib_module {
                        // Check if object is json.get() result or known non-string type variable
                        let use_value_helper = match object.as_ref() {
                            Expr::Call(c) => {
                                if let Expr::Dot(obj, method) = c.name.as_ref() {
                                    if let Expr::Ident(name) = obj.as_ref() {
                                        name == "json" && (method == "get" || method == "get_at")
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }
                            Expr::Ident(name) => {
                                self.local_var_types
                                    .get(name)
                                    .map(|ty| {
                                        matches!(
                                            ty,
                                            Type::User(_)
                                                | Type::Enum(_)
                                                | Type::Tag(_)
                                                | Type::GenericInstance(_)
                                                | Type::Void
                                        )
                                    })
                                    .unwrap_or(false)
                                    || self.json_value_vars.contains(name.as_str())
                            }
                            _ => false,
                        };
                        if use_value_helper {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::value_len(&")?;
                            self.expr(object, out)?;
                            write!(out, ")")?;
                            return Ok(());
                        }
                    }
                    // Fall through to remap table for normal len()
                }
                "byte_len" => {
                    // s.byte_len() -> a2r_std::str::str_byte_len(&s)
                    // （UTF-8 字节；VM 轨 nat#1526 同语义——PLAN-781 阶段 E）
                    self.a2r_std_used.set(true);
                    write!(out, "a2r_std::str::str_byte_len(")?;
                    self.expr_as_str(object, out)?;
                    write!(out, ")")?;
                    return Ok(());
                }
                "match_count" => {
                    // s.match_count(pattern) -> a2r_std::str::match_count(s, pattern)
                    self.a2r_std_used.set(true);
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
                    self.a2r_std_used.set(true);
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
                    // s.substr(start, end) -> a2r_std::str_substr(&s, start, end)
                    self.a2r_std_used.set(true);
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
                        self.local_var_types
                            .get(name)
                            .map(|ty| {
                                matches!(ty, Type::StrSlice | Type::StrOwned | Type::StrFixed(_))
                            })
                            .unwrap_or(false)
                    } else {
                        matches!(
                            object.as_ref(),
                            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_)
                        )
                    };
                    if obj_is_string {
                        // s.contains(needle) -> a2r_std::str_contains(&s, &needle)
                        self.a2r_std_used.set(true);
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
                    // Plan 380: char/&str literal args are valid str Patterns —
                    // use the native `obj.ends_with(arg)` (a2r_std::str_ends_with
                    // takes only &str — a char arg is E0308).
                    if call.args.args.len() == 1
                        && matches!(
                            call.args.args[0],
                            Arg::Pos(Expr::Char(_))
                                | Arg::Pos(Expr::Str(_))
                                | Arg::Pos(Expr::CStr(_))
                        )
                    {
                        self.expr(object, out)?;
                        write!(out, ".ends_with(")?;
                        self.arg(&call.args.args[0], out)?;
                        write!(out, ")")?;
                        return Ok(());
                    }
                    // s.ends_with(suffix) -> a2r_std::str_ends_with(&s, &suffix) returns i32
                    self.a2r_std_used.set(true);
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
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::get_or(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                self.arg(arg, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                    }
                    // map.get_or(key, default)
                    // For string maps: .get(key).map(|s| s.as_str()).unwrap_or(default)
                    // For non-string maps: .get(key).cloned().unwrap_or(default)
                    let is_string_default = call
                        .args
                        .args
                        .get(1)
                        .map(|a| {
                            if let Arg::Pos(e) = a {
                                matches!(e, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                            } else {
                                true
                            }
                        })
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
                "replace" if self.expr_is_string_like(object) => {
                    // P532 W1 G19: s.replace(from, to) —— String::replace 首参
                    // Pattern 界需 &str,通用发射对变量实参(String)E0277;
                    // 专用臂非字面量实参包 &(字面量裸传维持既有形态)。
                    // 守卫:仅字符串接收者——用户类型自有 replace 方法
                    // (Cell.replace(42) cookbook 形态)走通用发射。
                    self.expr(object, out)?;
                    write!(out, ".replace(")?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ")?;
                        }
                        let already_pattern = matches!(
                            arg,
                            Arg::Pos(Expr::Str(_))
                                | Arg::Pos(Expr::CStr(_))
                                | Arg::Pos(Expr::Char(_))
                        );
                        if !already_pattern {
                            write!(out, "&")?;
                        }
                        self.arg(arg, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(());
                }
                "find" => {
                    // s.find(needle)        -> a2r_std::str_find(&s, &needle)
                    // s.find(needle, start) -> a2r_std::str_find_from(&s, &needle, start)
                    // Auto's .find() is only for strings; always intercept.
                    self.a2r_std_used.set(true);
                    if call.args.args.len() >= 2 {
                        write!(out, "a2r_std::str_find_from(")?;
                    } else {
                        write!(out, "a2r_std::str_find(")?;
                    }
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
                    // Plan 415-B2: RedisClient.set(key, val) → a2r-std
                    // inherent method (both args &str borrows). Guarded
                    // inside this arm — a preceding arm would shadow the
                    // List/Map handling below.
                    if call.args.args.len() == 2 {
                        let is_redis = if let Expr::Ident(name) = object.as_ref() {
                            self.local_var_types.get(name)
                                .map(|ty| matches!(ty, Type::User(usr) if usr.name.as_str() == "RedisClient"))
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        if is_redis {
                            self.a2r_std_used.set(true);
                            self.expr(object, out)?;
                            write!(out, ".set(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(Expr::Ident(_)) = arg {
                                    write!(out, "&")?;
                                }
                                if let Arg::Pos(a) = arg {
                                    self.expr_as_str(a, out)?;
                                }
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                    }
                    // Plan 433 A1: list.set(i, v) -> list[i as usize] = v for
                    // Auto List receivers (in-place element write). The HashMap
                    // insert rewrite below shifts Vec elements instead of
                    // replacing — wrong semantics for Auto List.set.
                    if call.args.args.len() == 2 && self.is_auto_list_expr(object) {
                        let mut args_iter = call.args.args.iter();
                        let idx = args_iter.next();
                        let val = args_iter.next();
                        self.expr(object, out)?;
                        write!(out, "[(")?;
                        if let Some(Arg::Pos(a)) = idx {
                            self.expr(a, out)?;
                        }
                        write!(out, ") as usize] = ")?;
                        if let Some(Arg::Pos(a)) = val {
                            self.expr(a, out)?;
                        }
                        return Ok(());
                    }
                    // Plan 368: skip the Map.set -> HashMap::insert rewrite when the
                    // receiver is a known stdlib module identifier (env.set / fs.set /
                    // json.set / ...). Those are stdlib calls that must fall through
                    // to the (module, method) stdlib routing below; rewriting them to
                    // `<module>.insert(...)` produces invalid Rust (E0423: module has
                    // no method `insert`). `_ => {}` here lets the later dispatch
                    // handle `env.set` -> `a2r_std::env::set`.
                    let receiver_is_stdlib_module = matches!(object.as_ref(),
                        Expr::Ident(name) if matches!(name.as_str(),
                            "env" | "json" | "Json" | "fs" | "file" | "http" | "io"
                            | "shell" | "regex" | "math" | "sys" | "str" | "time" | "process"));
                    // Plan 514 W1 (洞 B): 同理跳过用户类型接收者——类型自有
                    // 方法名 `set`(99_idiom2/m09 Holder.set)须直通,不得重映射
                    // 为 HashMap::insert(Plan 393 E1 append 守卫同型)。
                    let receiver_is_user_type = matches!(object.as_ref(),
                        Expr::Ident(name) if self.local_var_types.get(name.as_str())
                            .map(|ty| matches!(ty,
                                Type::User(_) | Type::Tag(_) | Type::Enum(_)
                                | Type::GenericInstance(_)))
                            .unwrap_or(false));
                    if receiver_is_stdlib_module || receiver_is_user_type {
                        // fall through to the stdlib (module, method) routing.
                    } else {
                        // Map.set(key, val) -> HashMap::insert(key, val)
                        self.expr(object, out)?;
                        write!(out, ".insert(")?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ")?;
                            }
                            self.arg(arg, out)?;
                            // First arg: add as usize only for clearly integer expressions
                            if i == 0 {
                                if let Arg::Pos(expr) = arg {
                                    match expr {
                                        Expr::Int(_) => {
                                            write!(out, " as usize")?;
                                        }
                                        Expr::Ident(name) => {
                                            let ty = self.local_var_types.get(name);
                                            let is_str = ty.map_or(false, |t| {
                                                matches!(
                                                    t,
                                                    Type::StrSlice
                                                        | Type::StrOwned
                                                        | Type::StrFixed(_)
                                                )
                                            });
                                            if !is_str {
                                                // Not a known string; check if known int
                                                let is_int = ty.map_or(false, |t| {
                                                    matches!(t, Type::Int | Type::Uint)
                                                });
                                                if is_int {
                                                    write!(out, " as usize")?;
                                                }
                                                // Unknown type: skip, let post-processing handle
                                            }
                                        }
                                        _ => {} // Other exprs (calls, etc): no cast
                                    }
                                }
                            }
                            // Auto-borrow: key/value might be &str, but HashMap<String,V> needs String
                            if let Arg::Pos(expr) = arg {
                                if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                    write!(out, ".to_string()")?;
                                } else if let Expr::Ident(name) = expr {
                                    let is_str = self
                                        .local_var_types
                                        .get(name)
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
                    } // end else (non-stdlib Map.set rewrite)
                      // (stdlib module receiver: fall through to _ => {} below)
                }
                _ => {} // fall through to regular method handling
            }

            // env.* stdlib calls must work regardless of whether env is a local
            // variable (it could be shadowed by a local var named "env").
            // These always route to the a2r_std::env module.
            // Plan 368: env.set is now routed here too — the generic Map.set
            // -> HashMap::insert rewrite (which used to capture env.set and emit
            // `env.insert(...)`, producing invalid Rust E0423) now skips stdlib
            // module receivers, so env.set falls through to this handler.
            if let Expr::Ident(type_name) = object.as_ref() {
                if type_name == "env" {
                    match method_name.as_str() {
                        "get" => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::get(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "set" => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::set(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(expr) = arg {
                                    // env::set takes (&str, &str) — borrow owned
                                    // string args (e.g. a path/key built from concat)
                                    // so they are not moved.
                                    self.expr_as_str(expr, out)?;
                                } else {
                                    self.arg(arg, out)?;
                                }
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "get_or" => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::get_or(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(expr) = arg {
                                    self.expr_as_str(expr, out)?;
                                } else {
                                    self.arg(arg, out)?;
                                }
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        "args" => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::args()")?;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }

            // Check for type name / stdlib module BEFORE the remap table.
            // This ensures json.len(), shell.exec(), etc. are intercepted
            // instead of falling into the simple name-remap (which would generate
            // e.g. `json.len()` as a method call on the `json` module).
            if let Expr::Ident(type_name) = object.as_ref() {
                // If the identifier is a known local variable, skip stdlib routing
                let is_local_var = self.local_var_types.contains_key(type_name);
                if !is_local_var
                    && !self.local_modules.contains(type_name.as_str())
                    && !self.sibling_modules.contains(type_name.as_str())
                {
                    // Plan 368: Normalize "Json" → "json" for consistent module dispatch.
                    // PLAN-681: "Env" → "env" — the VM builtin object is capitalized
                    // (Env.get); the a2r dispatch tables key on lowercase.
                    let normalized_type = match type_name.as_str() {
                        "Json" => "json",
                        "Env" => "env",
                        _ => type_name.as_str(),
                    };
                    crate::stdlib_assembly::providers::require_reference_provider(
                        normalized_type,
                        method_name.as_str(),
                        "rust",
                    )?;
                    match (normalized_type, method_name.as_str()) {
                        ("json", "parse") => {
                            self.a2r_std_used.set(true);
                            write!(
                                out,
                                "{}",
                                if self.json_parse_as_opt {
                                    "a2r_std::json::parse_opt("
                                } else {
                                    "a2r_std::json::parse("
                                }
                            )?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("json", "get") => {
                            self.a2r_std_used.set(true);
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
                            // Plan 381 (Layer 2): `json.get(v, k)` returns the
                            // bridged Value — do NOT append .to_string() (that made
                            // every get() a String, breaking json.len/get_at/
                            // to_string/as_* chaining → E0308 &Value vs &String).
                            // Use `json.get_str` for string extraction.
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("json", "get_str") => {
                            self.a2r_std_used.set(true);
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
                        ("json", "as_string") => {
                            // Plan 381 (Layer 2): Value arg → as_string(&Value);
                            // string arg → as_string_str(&str). The unconditional
                            // _str variant broke `json.as_string(json.get(...))`
                            // (E0308 Option<&str>).
                            let is_value = call
                                .args
                                .args
                                .first()
                                .map_or(false, |a| self.json_arg_is_value(a));
                            self.a2r_std_used.set(true);
                            if is_value {
                                write!(out, "a2r_std::json::as_string(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                            } else {
                                write!(out, "a2r_std::json::as_string_str(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                            }
                            return Ok(());
                        }
                        ("json", "to_string") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::to_string(&")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("json", "get_at") => {
                            self.a2r_std_used.set(true);
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
                        ("json", "get_u64") => {
                            self.a2r_std_used.set(true);
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
                        ("json", "keys") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::keys(&")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("json", "len") => {
                            // Auto's json.len() returns int, but Rust returns usize — cast to i32
                            if let Some(Arg::Pos(expr)) = call.args.args.first() {
                                let is_str_type = if let Expr::Ident(name) = expr {
                                    self.local_var_types
                                        .get(name)
                                        .map(|ty| {
                                            matches!(
                                                ty,
                                                Type::StrSlice | Type::StrOwned | Type::StrFixed(_)
                                            )
                                        })
                                        .unwrap_or(true)
                                } else {
                                    matches!(expr, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                                };
                                if is_str_type {
                                    self.a2r_std_used.set(true);
                                    write!(out, "(a2r_std::json::len_str(")?;
                                    self.expr_as_str(expr, out)?;
                                    write!(out, ") as i64)")?;
                                } else {
                                    self.a2r_std_used.set(true);
                                    write!(out, "(a2r_std::json::len(&")?;
                                    self.expr(expr, out)?;
                                    write!(out, ") as i64)")?;
                                }
                            }
                            return Ok(());
                        }
                        ("json", "has_key") => {
                            // Auto's json.has_key() returns int (0 or 1), but Rust's returns bool.
                            // Wrap in if/else to convert bool -> i32.
                            if let Some(Arg::Pos(first)) = call.args.args.first() {
                                let use_str = if let Expr::Ident(name) = first {
                                    self.local_var_types
                                        .get(name)
                                        .map(|ty| {
                                            matches!(
                                                ty,
                                                Type::StrSlice | Type::StrOwned | Type::StrFixed(_)
                                            )
                                        })
                                        .unwrap_or(true)
                                } else {
                                    matches!(first, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                                };
                                write!(out, "if ")?;
                                if use_str {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::has_key_str(")?;
                                    self.expr_as_str(first, out)?;
                                } else {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::has_key(&")?;
                                    self.expr(first, out)?;
                                }
                                write!(out, ", ")?;
                                if call.args.args.len() > 1 {
                                    if let Arg::Pos(a) = &call.args.args[1] {
                                        self.expr(a, out)?;
                                    }
                                }
                                write!(out, ") {{ 1 }} else {{ 0 }}")?;
                            }
                            return Ok(());
                        }
                        ("json", "as_int") => {
                            // Plan 381 (Layer 2): Value arg → as_int(&Value).
                            let is_value = call
                                .args
                                .args
                                .first()
                                .map_or(false, |a| self.json_arg_is_value(a));
                            self.a2r_std_used.set(true);
                            if is_value {
                                write!(out, "a2r_std::json::as_int(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                            } else {
                                write!(out, "a2r_std::json::as_int_str(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ") as i64")?;
                            }
                            return Ok(());
                        }
                        ("json", "as_bool") => {
                            // Plan 381 (Layer 2): Value arg → as_bool(&Value).
                            let is_value = call
                                .args
                                .args
                                .first()
                                .map_or(false, |a| self.json_arg_is_value(a));
                            self.a2r_std_used.set(true);
                            if is_value {
                                write!(out, "a2r_std::json::as_bool(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                            } else {
                                write!(out, "a2r_std::json::as_bool_str(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                            }
                            return Ok(());
                        }
                        ("json", "is_valid") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::is_valid(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("json", "is_null") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::is_null(&")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("json", "type_of") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::value_type(&a2r_std::json::parse(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, "))")?;
                            return Ok(());
                        }
                        ("shell", "exec") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::shell::exec(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(expr) = arg {
                                    self.expr(expr, out)?;
                                    let skip_as_str =
                                        matches!(expr, Expr::Int(_) | Expr::Float(_, _))
                                            || if let Expr::Ident(name) = expr {
                                                self.local_var_types
                                                    .get(name)
                                                    .map(|ty| matches!(ty, Type::StrSlice))
                                                    .unwrap_or(false)
                                            } else {
                                                false
                                            };
                                    if !skip_as_str {
                                        write!(out, ".as_str()")?;
                                    }
                                }
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("regex", "match") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::re::r#match(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
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
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::re::find_all(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
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
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::exists(")?;
                            // Plan 016 Phase A A.4: use expr_as_str (handles owned
                            // String, &str params, str literals, and loop vars
                            // tracked as StrSlice — avoids the unstable str_as_str
                            // and the E0308 String-vs-&str mismatch).
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            } else if let Some(arg) = call.args.args.first() {
                                self.arg(arg, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "create_dir") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::create_dir(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        // Plan 742: mkdir_all 缺模块面路由（仅 method 臂有），
                        // repository.data_dir 的目录兜底需要它。
                        ("fs", "mkdir_all") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::mkdir_all(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        // Plan 742 D9: dir_names（Phase-1 JSON→SQLite 升级的收据枚举；
                        // list_dir 名被 Plan-626 VM-walk 面占用，故避让）。
                        ("fs", "dir_names") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::dir_names(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "write_text") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::write_text(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(expr) = arg {
                                    self.expr_as_str(expr, out)?;
                                } else {
                                    self.arg(arg, out)?;
                                }
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "append_text") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::append_text(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
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
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::is_dir(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "is_binary") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::is_binary(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "file_size") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::file_size(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "walk") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::walk(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "tree") => {
                            // PLAN-681: fs.tree(path, depth) → a2r_std::fs::tree —
                            // nested-JSON tree aligned byte-for-byte with the VM
                            // fs_tree_walk shape (TreeView node schema).
                            // PLAN-681 T-05: host 签名 max_depth: i32——Auto int
                            // 实参（i64）窄化 as i32（宿主契约对齐）。
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::tree(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            if let Some(Arg::Pos(a)) = call.args.args.get(1) {
                                write!(out, ", (")?;
                                self.expr(a, out)?;
                                write!(out, ") as i32")?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "read_to_string") | ("fs", "read_text") => {
                            let fn_name = method_name;
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::{}(", fn_name)?;
                            // Plan 368 R-AREG: use shared expr_as_str instead of stale
                            // local_var_types/StrSlice check, so owned String locals
                            // correctly get .as_str() appended.
                            // Plan 396 §2.3: borrow bare-ident PathBuf args — moving
                            // them breaks later uses (E0382); &PathBuf → &Path coercion.
                            if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                write!(out, "&")?;
                            }
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "write") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::write(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                if let Arg::Pos(expr) = arg {
                                    self.expr_as_str(expr, out)?;
                                } else {
                                    self.arg(arg, out)?;
                                }
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("fs", "delete") | ("File", "delete") => {
                            write!(out, "File::delete(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                self.arg(arg, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("env", "get") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::get(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("io", "read_line") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::io::read_line()")?;
                            return Ok(());
                        }
                        // Plan 415-B1: sqlite module fns on the Dot-path dispatch
                        // (mirrors the Bina-path arms; open takes &str).
                        ("sqlite", "open") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::sqlite::open(")?;
                            if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                write!(out, "&")?;
                            }
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("sqlite", "last_error") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::sqlite::last_error()")?;
                            return Ok(());
                        }
                        // Plan 415-B2: redis module fns on the Dot-path dispatch
                        // (mirrors the sqlite arms; open takes &str).
                        ("redis", "open") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::redis::open(")?;
                            if let Some(Arg::Pos(Expr::Ident(_))) = call.args.args.first() {
                                write!(out, "&")?;
                            }
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(());
                        }
                        ("redis", "last_error") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::redis::last_error()")?;
                            return Ok(());
                        }
                        ("Map", "new") => {
                            write!(out, "std::collections::HashMap::new()")?;
                            return Ok(());
                        }
                        // Plan 016 Phase A A4 cat 1: time builtin Dot-path dispatch
                        // (mirrors the Bina-path arms). Without this, `time.now_ms()`
                        // emits literally → E0423 "found module time".
                        ("time", "now_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::now_ms()")?;
                            return Ok(());
                        }
                        ("time", "sleep_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::sleep_ms(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, " as u64)")?;
                            return Ok(());
                        }
                        ("time", "now_sec") | ("time", "now_secs") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::now_sec()")?;
                            return Ok(());
                        }
                        ("time", "now") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::now()")?;
                            return Ok(());
                        }
                        // PLAN-716 Phase 2 (供⑨-b): frame 二段名 Dot-path 臂
                        // （time A4 先例同款——缺失时原文发射 → E0425 cannot
                        // find value `frame`，下游 plan-022 实录；裸名姊妹臂
                        // 在 call() 的 Expr::Ident 映射表）。
                        ("frame", "begin_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::frame::begin_ms()")?;
                            return Ok(());
                        }
                        ("frame", "present_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::frame::present_ms()")?;
                            return Ok(());
                        }
                        _ => {} // fall through to remap table
                    }
                } // if !is_local_var
            }

            // Dynamic Map methods: insert_int/get_int/insert_str/get_str
            // Auto's Map stores everything as strings; int values are encoded/decoded
            // via to_string()/parse(). These methods need inline code generation.
            match method_name.as_str() {
                "insert_int" => {
                    self.expr(object, out)?;
                    write!(out, ".insert(")?;
                    self.arg(&call.args.args[0], out)?;
                    write!(out, ".to_string(), (")?;
                    self.arg(&call.args.args[1], out)?;
                    write!(out, ").to_string())")?;
                    return Ok(());
                }
                "get_int" => {
                    write!(out, "(")?;
                    self.expr(object, out)?;
                    write!(out, ".get(&")?;
                    self.arg(&call.args.args[0], out)?;
                    write!(
                        out,
                        ".to_string()).and_then(|v| v.parse::<i64>().ok()).unwrap_or(0))"
                    )?;
                    return Ok(());
                }
                "insert_str" => {
                    self.expr(object, out)?;
                    write!(out, ".insert(")?;
                    self.arg(&call.args.args[0], out)?;
                    write!(out, ".to_string(), (")?;
                    self.arg(&call.args.args[1], out)?;
                    write!(out, ").to_string())")?;
                    return Ok(());
                }
                "get_str" => {
                    write!(out, "(")?;
                    self.expr(object, out)?;
                    write!(out, ".get(&")?;
                    self.arg(&call.args.args[0], out)?;
                    write!(out, ".to_string()).cloned().unwrap_or_default())")?;
                    return Ok(());
                }
                _ => {}
            }

            // Tag construction check for Expr::Dot format calls:
            // module.Type.Variant(args) via Expr::Dot(Expr::Dot(module, Type), Variant)
            // Type.Variant(args) via Expr::Dot(Ident(Type), Variant)
            {
                let mut dot_tag_match: Option<(Option<AutoStr>, AutoStr, AutoStr)> = None;
                // Two-level: Type.Variant via Expr::Dot(Ident(Type), Variant)
                if let Expr::Ident(type_name) = object.as_ref() {
                    if self.tag_types.contains(type_name) {
                        dot_tag_match = Some((None, type_name.clone(), method_name.clone()));
                    }
                }
                // Three-level: module.Type.Variant via Expr::Dot(Expr::Dot(Ident(module), Name(Type)), Name(Variant))
                if dot_tag_match.is_none() {
                    if let Expr::Dot(inner_obj, inner_type_name) = object.as_ref() {
                        if let Expr::Ident(mod_name) = inner_obj.as_ref() {
                            if self.tag_types.contains(inner_type_name)
                                || self.module_types.contains_key(mod_name.as_str())
                            {
                                dot_tag_match = Some((
                                    Some(mod_name.clone()),
                                    inner_type_name.clone(),
                                    method_name.clone(),
                                ));
                            }
                        }
                    }
                }
                // Three-level: module.Type.Variant via Expr::Dot(Expr::Bina(Ident(module), Dot, Ident(Type)), Name(Variant))
                if dot_tag_match.is_none() {
                    if let Expr::Bina(inner_lhs, inner_op, inner_rhs) = object.as_ref() {
                        if matches!(inner_op, Op::Dot) {
                            if let Expr::Ident(mod_name) = inner_lhs.as_ref() {
                                if let Expr::Ident(type_name) = inner_rhs.as_ref() {
                                    if self.tag_types.contains(type_name)
                                        || self.module_types.contains_key(mod_name.as_str())
                                    {
                                        dot_tag_match = Some((
                                            Some(mod_name.clone()),
                                            type_name.clone(),
                                            method_name.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some((mod_prefix, type_name, variant_name)) = dot_tag_match {
                    // Validate: variant name must start with uppercase (Tag.Variant convention)
                    // or be a known enum variant. Method names (lowercase) are not tag constructions.
                    let variant_is_upper = variant_name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    let key = (type_name.clone(), variant_name.clone());
                    let has_struct_fields = self.enum_struct_variants.contains_key(&key);
                    let has_tuple_fields = self.enum_tuple_field_types.contains_key(&key);
                    if variant_is_upper || has_struct_fields || has_tuple_fields {
                        let struct_fields = self.enum_struct_variants.get(&key).cloned();
                        if let Some(ref mp) = mod_prefix {
                            if self.merge_mode || mp.as_str() == self.current_module_name {
                                write!(out, "{}::{}::{}", mp, type_name, variant_name)?;
                            } else if self.module_types.contains_key(mp.as_str()) {
                                write!(out, "crate::{}::{}::{}", mp, type_name, variant_name)?;
                            } else {
                                write!(out, "{}::{}::{}", mp, type_name, variant_name)?;
                            }
                        } else {
                            write!(out, "{}::{}", type_name, variant_name)?;
                        }
                        if let Some(fields) = struct_fields {
                            write!(out, " {{ ")?;
                            for (i, (arg, field_name)) in
                                call.args.args.iter().zip(fields.iter()).enumerate()
                            {
                                match arg {
                                    Arg::Pos(expr) => {
                                        write!(out, "{}: ", field_name)?;
                                        self.expr(expr, out)?;
                                        if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                    // C7b: named-arg construction
                                    // (Tier.NeedsApproval(reason: "x")) — use the
                                    // pair's own key as the field name.
                                    Arg::Pair(key, expr) => {
                                        write!(out, "{}: ", key)?;
                                        self.expr(expr, out)?;
                                        if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                    Arg::Name(_) => {}
                                }
                                if i < call.args.args.len().min(fields.len()) - 1 {
                                    write!(out, ", ")?;
                                }
                            }
                            write!(out, " }}")?;
                        } else {
                            let tuple_field_types = self.enum_tuple_field_types.get(&key).cloned();
                            write!(out, "(")?;
                            for (i, arg) in call.args.args.iter().enumerate() {
                                if let Arg::Pos(expr) = arg {
                                    self.expr(expr, out)?;
                                    if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                        write!(out, ".to_string()")?;
                                    } else if let Expr::Ident(name) = expr {
                                        let field_is_string = tuple_field_types
                                            .as_ref()
                                            .and_then(|types| types.get(i))
                                            .map(|ty| {
                                                matches!(
                                                    ty,
                                                    Type::StrOwned
                                                        | Type::StrFixed(_)
                                                        | Type::StrSlice
                                                )
                                            })
                                            .unwrap_or(false);
                                        let var_is_str_slice = self
                                            .local_var_types
                                            .get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice))
                                            .unwrap_or(false);
                                        if field_is_string && var_is_str_slice {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                }
                                if i < call.args.args.len() - 1 {
                                    write!(out, ", ")?;
                                }
                            }
                            write!(out, ")")?;
                        }
                        return Ok(());
                    }
                }
            }

            // Plan 433 A1: struct `len` field read (Tok.len) — property,
            // not the collection-length method.
            if self.try_emit_struct_len_field(object, method_name, call.args.args.len(), out)? {
                return Ok(());
            }

            // .len() and .length() return usize in Rust, cast to i32 for Auto's int
            // Plan 415-E1: zero-arg form only — `.len(n)` WITH an argument is a
            // foreign builder setter (memmap2 MmapOptions::len) returning Self;
            // casting that to i64 breaks the method chain. (Mirror of the
            // Bina-path flag above.)
            let needs_i32_cast =
                matches!(method_name.as_str(), "len" | "length") && call.args.args.is_empty();

            // For "contains", choose between str::contains and map::contains_key
            // Only use contains_key when we KNOW the object is a Map.
            // Default to str::contains since it works on String and &str.
            let contains_rust = if method_name.as_str() == "contains" {
                match object.as_ref() {
                    Expr::Ident(name) => {
                        let obj_is_map = self
                            .local_var_types
                            .get(name)
                            .map(|ty| matches!(ty, Type::Map(_, _)))
                            .unwrap_or(false);
                        if obj_is_map {
                            Some("contains_key")
                        } else {
                            Some("contains")
                        }
                    }
                    Expr::Dot(inner_obj, inner_field) => {
                        // Check if the inner field is a known Map type in any struct
                        let field_is_map = if let Expr::Ident(_) = inner_obj.as_ref() {
                            self.struct_field_types.values().any(|fields| {
                                fields.iter().any(|(fname, fty)| {
                                    fname == inner_field && matches!(fty, Type::Map(_, _))
                                })
                            })
                        } else {
                            false
                        };
                        if field_is_map {
                            Some("contains_key")
                        } else {
                            Some("contains")
                        }
                    }
                    _ => Some("contains"),
                }
            } else {
                None
            };

            let rust_method = match method_name.as_str() {
                // String methods
                "to_lower" | "lower" => Some("to_lowercase"),
                "to_upper" | "upper" => Some("to_uppercase"),
                "length" | "len" => Some("len"),
                "is_empty" => Some("is_empty"),
                "trim" => Some("trim"),
                "trim_left" => Some("trim_start"),
                "trim_right" => Some("trim_end"),
                "starts_with" => Some("starts_with"),
                "ends_with" => Some("ends_with"),
                "find_last" => Some("rfind"),
                "to_str" => Some("to_str"),
                // Plan 393 E1: only remap to push_str when receiver is NOT a known
                // struct. A struct method named `append` (e.g. ChatSession.append)
                // must pass through unchanged. Unknown receiver keeps the legacy
                // remap (String is the common case for .append).
                "append" => {
                    let lhs_is_struct = if let Expr::Ident(name) = object.as_ref() {
                        self.local_var_types
                            .get(name)
                            .map(|ty| {
                                matches!(
                                    ty,
                                    Type::User(_)
                                        | Type::Tag(_)
                                        | Type::Enum(_)
                                        | Type::GenericInstance(_)
                                )
                            })
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if !lhs_is_struct {
                        Some("push_str")
                    } else {
                        None
                    }
                }
                // Collection methods
                "push" => Some("push"),
                "pop" => Some("pop"),
                "drop" => Some("take"),
                "clear" => Some("clear"),
                "to_array" => Some("clone"),
                "retain" => Some("retain"),
                // HashMap methods
                // Plan 514 W1 (洞 B, P511-5 探针暴露): only remap set → insert
                // when the receiver is NOT a known user-type instance — a type
                // method named `set` (99_idiom2/m09 `Holder.set`) must pass
                // through unchanged. Mirror of the Plan 393 E1 `append` guard.
                // PLAN-019 缺陷修正:List 接收者的 .set(idx, v) 是替换语义,
                // 不得映射 Vec::insert(插入语义,右移后续元素)——db.at 多 Tab
                // 模型表腐坏实测(tab_root_node [3,1] vs 期望 [3,4],UI 挂起)。
                // List 接收者改走下方 "set 索引赋值" 特化臂。
                "set" => {
                    if self.recv_is_list_like(object) {
                        Some("__list_set_idx__")
                    } else {
                        let lhs_is_struct = if let Expr::Ident(name) = object.as_ref() {
                            self.local_var_types
                                .get(name)
                                .map(|ty| {
                                    matches!(
                                        ty,
                                        Type::User(_)
                                            | Type::Tag(_)
                                            | Type::Enum(_)
                                            | Type::GenericInstance(_)
                                    )
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        if !lhs_is_struct {
                            Some("insert")
                        } else {
                            None
                        }
                    }
                }
                // Plan 384 A9: keep `.delete()` as-is (see note at the other
                // match site) — axum Router `.delete()` must not become remove.
                "delete" => Some("delete"),
                // String methods that need special handling
                "split" => Some("split"),
                // Type conversion
                "to_string" => Some("to_string"),
                // Plan 433 A1: Auto's universal `.str()` stringify (VM native)
                // maps to Rust's Display to_string.
                "str" => Some("to_string"),
                // Plan 013 (B16): pass-through so Map.get() reaches the
                // auto-borrow handler below (rust_method is otherwise None,
                // skipping the get-borrow path).
                "get" => Some("get"),
                _ => contains_rust,
            };

            if let Some(rust_name) = rust_method {
                // PLAN-019:List 接收者的 .set(idx, v) → 索引赋值(替换语义;
                // 镜像 Plan 514 W3 的 get 索引形特化)。
                if rust_name == "__list_set_idx__" && call.args.args.len() == 2 {
                    self.expr(object, out)?;
                    write!(out, "[(")?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        self.expr(a, out)?;
                    }
                    write!(out, ") as usize] = ")?;
                    if let Some(Arg::Pos(a)) = call.args.args.get(1) {
                        self.expr(a, out)?;
                    }
                    writeln!(out, ";")?;
                    return Ok(());
                }
                // Plan 514 W3:List/Vec 接收者的 .get(i) → 索引形
                // `recv[(i) as usize]`(Vec::get 返 Option<&T>,直接字段访问
                // E0609;lib 方法体 `.toks.get(.pos).kind` 位)。
                if rust_name == "get" && call.args.args.len() == 1 {
                    // Plan 531 真缺陷②:统一经 recv_is_list_like(self 字段
                    // 实型解析),见其注释。
                    let recv_is_list2 = self.recv_is_list_like(object);
                    if recv_is_list2 {
                        self.expr(object, out)?;
                        write!(out, "[(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr(a, out)?;
                        }
                        write!(out, ") as usize]")?;
                        if self.list_elem_needs_index_clone(object) {
                            write!(out, ".clone()")?;
                        }
                        return Ok(());
                    }
                }
                // Plan 379: parenthesize unary-deref receivers too —
                // `(*x).clone()` would otherwise emit `*x.clone()` which Rust
                // parses as `*(x.clone())` (wrong precedence).
                let obj_parens = matches!(object.as_ref(),
                    Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
                ) || matches!(object.as_ref(), Expr::Unary(Op::Mul, _))
                    || Self::is_char_at_call(object.as_ref());
                // PLAN-681 T-05（N2）：str 族方法的接收器是 Value 读
                // （`nd.label.contains(kw)`——P2 发射 nd["label"] 为 Value,
                // Value 自身无 contains/starts_with 等 str 方法）——接收器
                // 与方法之间插 as_str 降链。仅 str 方法族触发,其余方法
                // （Value/JSON 语义）直发。
                let recv_is_value_read_str_method = matches!(
                    method_name.as_str(),
                    "contains"
                        | "contains_key"
                        | "starts_with"
                        | "ends_with"
                        | "to_lower"
                        | "lower"
                        | "to_upper"
                        | "upper"
                        | "trim"
                        | "trim_left"
                        | "trim_right"
                        | "split"
                        | "replace"
                        | "replace_first"
                        | "find_last"
                        | "is_empty"
                        | "length"
                        | "len"
                ) && self.expr_is_json_value_read(object);
                if needs_i32_cast && !self.len_i32_cast_suppressed {
                    write!(out, "(")?;
                }
                if obj_parens {
                    write!(out, "(")?;
                }
                self.expr(object, out)?;
                if obj_parens {
                    write!(out, ")")?;
                }
                if recv_is_value_read_str_method {
                    write!(out, ".as_str().unwrap_or_default()")?;
                }
                write!(out, ".{}", rust_name)?;
                // Plan 395: explicit generic type args → Rust turbofish
                self.emit_turbofish_args(call, out)?;
                write!(out, "(")?;
                // Auto-borrow string args for pattern-matching and map lookup methods
                // P532 W1 G19: replace 同入(同上 Pattern 界)
                if matches!(
                    rust_name,
                    "contains" | "contains_key" | "starts_with" | "ends_with" | "split" | "replace"
                ) {
                    for (i, arg) in call.args.args.iter().enumerate() {
                        // Only add & for String-typed args, not &str params or literals
                        // Note: local_var_types has StrSlice for ALL str vars (params AND locals),
                        // but only fn params declared as `str` are truly &str in Rust.
                        // Local vars of type str are String in Rust and still need &.
                        // Plan 380: char/&str literals are already valid Patterns —
                        // `&'"'` would be `&char` (E0277).
                        let already_borrowed =
                            matches!(arg, Arg::Pos(Expr::Str(_) | Expr::CStr(_) | Expr::Char(_)))
                                || if let Arg::Pos(Expr::Ident(name)) = arg {
                                    self.current_fn_str_params.contains(name)
                                } else {
                                    false
                                };
                        if !already_borrowed {
                            write!(out, "&")?;
                        }
                        self.arg(arg, out)?;
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                } else if rust_name == "get" {
                    // Plan 013 (B16): Map.get(&Q) needs a reference for owned
                    // String keys. Borrow only OWNED-String args (local vars /
                    // field access). Skip: str params (already &str in Rust →
                    // &&str would be wrong), and Int args (Vec::get takes usize).
                    // Plan 514 W3: List/Array 接收者（Vec::get(usize)）不做
                    // 借用——`.toks.get(.pos)` 此前经 Expr::Dot 臂误发
                    // `.get(&self.pos)`（E0277，lib 方法体字段位实参）。
                    // Plan 531 真缺陷②:同上——self.<field> 经 recv_is_list_like
                    // 实型解析(`.toks.get(.pos)` 的 List 字段照常走值位)。
                    let recv_is_list = self.recv_is_list_like(object);
                    for (i, arg) in call.args.args.iter().enumerate() {
                        let is_owned_string_arg = if let Arg::Pos(e) = arg {
                            match e {
                                // Plan 016 Phase A A4 cat 3: string literals are
                                // already &'static str — Value::get accepts them
                                // directly. Adding & makes &&str (E0277 trait bound).
                                Expr::Str(_) | Expr::CStr(_) => false,
                                Expr::Ident(name) => {
                                    // Owned String local, but NOT a str param
                                    // (params declared `str` are &str in Rust).
                                    // Plan 018 §14 W1: composite keys (Type::Tuple)
                                    // also need & — HashMap::get(&Q) where K:
                                    // Borrow<Q>; `&tuple` is the direct key ref
                                    // (no Display needed). Unknown types get &
                                    // conservatively (borrow is always safe for
                                    // HashMap lookups).
                                    // Plan 032 G2-core: non-primitive KEY types
                                    // (enum/struct/tag/generic — e.g. a
                                    // HashMap<ModelTier, _> lookup) also need
                                    // `&`: `K: Borrow<K>` holds for every K, so
                                    // borrowing is always safe. Int/Uint stay
                                    // excluded (Vec::get takes usize by value).
                                    !self.current_fn_str_params.contains(name)
                                        && self
                                            .local_var_types
                                            .get(name)
                                            .map(|ty| {
                                                matches!(
                                                    ty,
                                                    Type::StrOwned
                                                        | Type::StrSlice
                                                        | Type::StrFixed(_)
                                                        | Type::CStrLit
                                                        | Type::Tuple(_)
                                                        | Type::User(_)
                                                        | Type::Tag(_)
                                                        | Type::Enum(_)
                                                        | Type::GenericInstance(_)
                                                )
                                            })
                                            .unwrap_or(true)
                                }
                                // Field access (cfg.default_provider) — a String
                                // field; borrow it (Map receivers only —
                                // Plan 514 W3 List/Vec receivers take usize).
                                Expr::Dot(_, _) => !recv_is_list,
                                _ => false,
                            }
                        } else {
                            false
                        };
                        if is_owned_string_arg {
                            write!(out, "&")?;
                        }
                        // Plan 514 W3: List/Vec 接收者首参是 usize 位——
                        // i64 实参(含 self 字段位)内联 as usize 转换。
                        if recv_is_list && i == 0 {
                            write!(out, "(")?;
                            self.arg(arg, out)?;
                            write!(out, ") as usize")?;
                        } else {
                            self.arg(arg, out)?;
                        }
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                } else {
                    let is_push_or_insert = matches!(method_name.as_str(), "push" | "set");
                    let is_insert = method_name.as_str() == "set";
                    for (i, arg) in call.args.args.iter().enumerate() {
                        // 2026-08-23 auto-ai report defect 3: an object/array
                        // record literal pushed into a `List<Value>` (Value =
                        // serde_json::Value) has no Rust struct to name — emit
                        // it as a serde_json::json! invocation instead of bare
                        // braces ("struct literal body without path").
                        // PLAN-681 T-05（N2/P3）：扩容——标量/串字面量与
                        // 非 Value 型表达式（String/int 局部、Vec 局部、算术）
                        // 同样包 json! 构造 Value；Value 型读（推断 Value）
                        // 与 Unknown（可能是 Value 读）保持直发。
                        let is_json_value_push = is_push_or_insert
                            && !is_insert
                            && self.receiver_elem_is_json_value(object)
                            && matches!(arg, Arg::Pos(e) if self.value_push_arg_needs_json_wrap(e));
                        if is_json_value_push {
                            if let Arg::Pos(e) = arg {
                                self.emit_json_macro(e, out)?;
                            }
                        } else {
                            self.arg(arg, out)?;
                            // PLAN-681 T-05（N2）：Value 读实参直发（未包
                            // json!）是索引位 place——move 出共享引用索引即
                            // E0507（`out.push(nd["id"])` 株），补 .clone()
                            //（arg() 已带 .clone() 尾缀的索引克隆形不叠加）。
                            if is_push_or_insert
                                && matches!(arg, Arg::Pos(e)
                                    if self.expr_is_json_value_read(e)
                                        && !matches!(e, Expr::Call(c)
                                            if matches!(c.name.as_ref(),
                                                Expr::Dot(_, m) if m.as_str() == "clone")))
                            {
                                write!(out, ".clone()")?;
                            }
                        }
                        // set(idx, val) -> insert(idx, val): add 'as usize' for int-typed idx
                        if is_insert && i == 0 {
                            if let Arg::Pos(expr) = arg {
                                if let Expr::Int(_) = expr {
                                    write!(out, " as usize")?;
                                } else if let Expr::Ident(name) = expr {
                                    let is_int_type = self
                                        .local_var_types
                                        .get(name)
                                        .map(|ty| matches!(ty, Type::Int | Type::Uint))
                                        .unwrap_or(false);
                                    if is_int_type {
                                        write!(out, " as usize")?;
                                    }
                                }
                            }
                        }
                        // push/insert with string literal -> .to_string() for Vec<String>/HashMap<String,_>
                        // PLAN-681 T-05（N2/P3）：json! 包裹已产 Value,外层
                        // .to_string() 会把 Value 序列化成 String（E0308）——跳过。
                        if is_push_or_insert && !is_json_value_push {
                            if let Arg::Pos(expr) = arg {
                                if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                    write!(out, ".to_string()")?;
                                }
                            }
                        }
                        // Auto-clone: .push() and .insert() take ownership, clone non-Copy ident args
                        // Conservative: unknown types are treated as non-Copy (safer for ownership)
                        // PLAN-681 T-05（N2/P3）：同上,json! 包裹（借用语义,
                        // to_value(&$e)）后不得再叠 .to_string()/.clone()。
                        if is_push_or_insert && !is_json_value_push {
                            if let Arg::Pos(Expr::Ident(name)) = arg {
                                // Plan 434 (242 #18-2): a str-typed ident is
                                // &str at runtime when it's a `str` param /
                                // str-return fn — pushing into a String elem
                                // Vec needs `.to_string()` (`.clone()` keeps
                                // &str; is_copy_type treats str as Copy so
                                // neither clone branch below fires).
                                let is_str_ty = self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| {
                                        matches!(
                                            ty,
                                            Type::StrFixed(_)
                                                | Type::StrSlice
                                                | Type::StrOwned
                                                | Type::CStrLit
                                        )
                                    })
                                    .unwrap_or(false)
                                    || (self.local_var_types.get(name).is_none()
                                        && self.current_fn_str_params.contains(name));
                                if is_str_ty {
                                    write!(out, ".to_string()")?;
                                } else {
                                    let is_copy = self
                                        .local_var_types
                                        .get(name)
                                        .map(|ty| Self::is_copy_type(ty))
                                        .unwrap_or(false);
                                    if !is_copy {
                                        write!(out, ".clone()")?;
                                    }
                                }
                            }
                        }
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                }
                write!(out, ")")?;
                if needs_i32_cast && !self.len_i32_cast_suppressed {
                    write!(out, " as i64)")?;
                }
                // trim/trim_start/trim_end return &str, auto-convert to String
                // Plan 380: skip when the callee `trim` returns void (a struct
                // method named trim, e.g. Memory.trim() — `.to_string()` on `()`
                // is E0599).
                if matches!(method_name.as_str(), "trim" | "trim_left" | "trim_right") {
                    let trim_ret_is_void = self
                        .fn_ret_types
                        .get(method_name.as_str())
                        .map(|t| matches!(t, Type::Void))
                        .unwrap_or(false);
                    if !trim_ret_is_void {
                        write!(out, ".to_string()")?;
                    }
                }
                // split returns iterator in Rust, collect into Vec so .len()/.get() work.
                // If the Auto source needs raw iterator semantics, it should use split() without
                // assigning to a variable that later uses Vec operations.
                // Plan 019 Phase 1: Auto's List<str> is owned strings — collect
                // into Vec<String> (mirrors a2ts semantics). The previous
                // Vec<_> inferred Vec<&str>, clashing with every declared
                // List<str> consumer (params, typed locals, returns).
                if method_name.as_str() == "split" {
                    write!(out, ".map(|s| s.to_string()).collect::<Vec<String>>()")?;
                }
                return Ok(());
            }

            // Check for type name static method: Type.method(args) -> Type::method(args)
            // (stdlib modules already handled by the early check above)
            if let Expr::Ident(type_name) = object.as_ref() {
                // PLAN-010 T2: Auto's optional-type constructors. `May.val(v)`
                // lowered verbatim emits `May::val(v)` — unresolved in a
                // standalone product (May lives in a2r_std; 009 ledger E0433).
                // May IS Rust Option: lower to the bare constructors.
                if type_name.as_str() == "May" {
                    if method_name.as_str() == "val" || method_name.as_str() == "some" {
                        write!(out, "Some(")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr(a, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(());
                    }
                    if matches!(method_name.as_str(), "nil" | "none") {
                        write!(out, "None")?;
                        return Ok(());
                    }
                }
                // PLAN-596 T-07 (DIV-DEP-13): a SCREAMING_CASE use.rs import is
                // a const receiver (`STANDARD.encode(..)`) — a trait-method
                // VALUE call, not a type-associated fn. Fall through to the
                // regular method-call path, which emits `.` instead of `::`.
                let is_const_receiver = self.is_dep_constant_name(type_name.as_str());
                let is_type = !is_const_receiver
                    && (type_name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                        || matches!(
                            type_name.as_str(),
                            "u8" | "u16"
                                | "u32"
                                | "u64"
                                | "u128"
                                | "usize"
                                | "i8"
                                | "i16"
                                | "i32"
                                | "i64"
                                | "i128"
                                | "isize"
                                | "f32"
                                | "f64"
                                | "bool"
                                | "char"
                                | "str"
                        )
                        || Self::auto_type_to_rust(type_name.as_str()).is_some());
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
                    } else if self.local_struct_types.contains(type_name.as_str())
                        || self.tag_types.contains(type_name.as_str())
                        || self.known_enum_names.contains(type_name.as_str())
                        || self.union_types.contains(type_name.as_str())
                    {
                        // Plan 013 (B1/BUG3): the type is declared locally in
                        // this file (struct/enum/tag/union). Never qualify it
                        // with an external crate prefix — a `use.rust <crate>`
                        // import must not leak onto local type construction.
                        rust_type_name.to_string()
                    } else if !self.uses.contains(type_name.as_str()) {
                        // PLAN-010 T1: builtin collection/type mappings (List→Vec,
                        // Map→HashMap, Set→HashSet) are never crate-qualified —
                        // `Vec::new()` is std, and a lowercase use.rs item (e.g. a
                        // brace-expanded fn import like `engine_write_line`) must
                        // not be misread as its source crate (009 T8 evidence:
                        // `engine_write_line::Vec::new()`).
                        if Self::auto_type_to_rust(type_name.as_str()).is_some() {
                            rust_type_name.to_string()
                        } else {
                            // Type not in uses at all — qualify with the best matching
                            // external crate. Prefer the most specific (longest named) crate.
                            let source_crate = self
                                .uses
                                .iter()
                                .filter(|u| {
                                    let u_str = u.as_str();
                                    !u_str.contains("::")
                                        && !u_str.contains('.')
                                        && u_str != "a2r_std"
                                        && !u_str.starts_with("std")
                                        && !u_str.starts_with("auto_lang")
                                        && !Self::auto_type_to_rust(u_str).is_some()
                                        && !self.glob_imported_modules.contains(u_str)
                                        && u_str.chars().next().map_or(true, |c| c.is_lowercase())
                                })
                                .max_by_key(|u| u.as_str().len())
                                .map(|u| u.as_str())
                                .unwrap_or("");
                            if !source_crate.is_empty() {
                                format!("{}::{}", source_crate, rust_type_name)
                            } else {
                                rust_type_name.to_string()
                            }
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
                            for (i, (arg, field_name)) in
                                call.args.args.iter().zip(fields.iter()).enumerate()
                            {
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
                                if i < call.args.args.len().min(fields.len()) - 1 {
                                    write!(out, ", ")?;
                                }
                            }
                            write!(out, " }}")?;
                        } else if call.args.args.is_empty()
                            && !self.enum_tuple_variants.contains_key(&key)
                        {
                            // Plan 016 Phase 4: zero-arg call on a variant with no
                            // registered payload is a UNIT variant (Auto spells it
                            // `Value.Null()` for a2ts parity) — `Type::Variant` is
                            // already written; parens would be E0618.
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
                                        let field_is_string = tuple_field_types
                                            .as_ref()
                                            .and_then(|types| types.get(i))
                                            .map(|ty| {
                                                matches!(
                                                    ty,
                                                    Type::StrOwned
                                                        | Type::StrFixed(_)
                                                        | Type::StrSlice
                                                )
                                            })
                                            .unwrap_or(false);
                                        let var_is_str_slice = self
                                            .local_var_types
                                            .get(name)
                                            .map(|ty| matches!(ty, Type::StrSlice))
                                            .unwrap_or(false);
                                        if field_is_string && var_is_str_slice {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                }
                                if i < call.args.args.len() - 1 {
                                    write!(out, ", ")?;
                                }
                            }
                            write!(out, ")")?;
                        }
                        return Ok(());
                    }
                    // Static method: Type::method(args)
                    write!(out, "{}::{}", qualified_type, method_name)?;
                    // Plan 016 Phase 4: `Value.Null()` — a zero-arg call on a
                    // KNOWN ENUM whose CamelCase "method" is no registered
                    // struct/tuple variant is a UNIT-variant construction
                    // (Auto spells zero-payload variants as zero-arg calls for
                    // a2ts parity). Emit `Enum::Variant` without parens —
                    // calling a unit variant is E0618. The CamelCase gate
                    // keeps real snake_case associated fns untouched.
                    if call.args.args.is_empty()
                        && method_name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                        && self.known_enum_names.contains(type_name.as_str())
                        && !self
                            .enum_struct_variants
                            .contains_key(&(type_name.clone(), method_name.clone()))
                        && !self
                            .enum_tuple_variants
                            .contains_key(&(type_name.clone(), method_name.clone()))
                    {
                        return Ok(());
                    }
                    write!(out, "(")?;
                    // Add `move` for thread::spawn closures (captured locals need 'static).
                    // Plan 599: skip when the closure already opted in via
                    // `move (..) => ..` — emitting both produced `move move ||`.
                    if method_name == "spawn"
                        && call.args.args.first().map_or(
                            false,
                            |a| matches!(a, Arg::Pos(Expr::Closure(c)) if !c.is_move),
                        )
                    {
                        write!(out, "move ")?;
                    }
                    // Prefer qualified key "Type.method" for accurate lookup
                    let qualified_key: AutoStr = format!("{}.{}", type_name, method_name).into();
                    let static_str_flags = self
                        .fn_str_param_indices
                        .get(&qualified_key)
                        .cloned()
                        .or_else(|| self.fn_str_param_indices.get(method_name.as_str()).cloned());
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if let Arg::Pos(expr) = arg {
                            self.expr(expr, out)?;
                            // Auto-borrow for str params
                            let is_str_param = static_str_flags
                                .as_ref()
                                .and_then(|f| f.get(i))
                                .copied()
                                .unwrap_or(false);
                            if is_str_param
                                && !matches!(
                                    expr,
                                    Expr::Str(_) | Expr::CStr(_) | Expr::Int(_) | Expr::Float(_, _)
                                )
                            {
                                let is_fn_str_param = if let Expr::Ident(name) = expr {
                                    self.current_fn_str_params.contains(name)
                                } else {
                                    false
                                };
                                if !is_fn_str_param {
                                    write!(out, ".as_str()")?;
                                }
                            }
                            // Auto-borrow for external crate type static methods
                            // Plan 381 (Layer 2): skip for known ENUM types —
                            // `OutputContentBlock.ToolUse(id, name, input)` is a
                            // variant CONSTRUCTION (owned String payloads), not a
                            // static method call; .as_str() on the payloads is
                            // E0308 (`String` vs `&str`).
                            let is_enum_ctor = self.known_enum_names.contains(type_name.as_str());
                            if !is_str_param && !is_enum_ctor {
                                if let Expr::Ident(name) = expr {
                                    if self
                                        .local_var_types
                                        .get(name)
                                        .map(|ty| matches!(ty, Type::StrOwned | Type::StrFixed(_)))
                                        .unwrap_or(false)
                                    {
                                        write!(out, ".as_str()")?;
                                    }
                                }
                            }
                            // Plan 381 (Layer 2): enum-ctor String payloads fed a
                            // string literal need `.to_string()` (Text("") →
                            // Text("".to_string()), E0308 `String` vs `&str`).
                            if is_enum_ctor
                                && matches!(expr, Expr::Str(_) | Expr::CStr(_))
                                && !is_str_param
                            {
                                write!(out, ".to_string()")?;
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
                    // Plan 384 S2: a known local variable / param / `self` is a
                    // value, not a type/module — short-circuit even if its name
                    // matches a use.rust path leaf (e.g. local `sse` vs module
                    // `axum::response::sse`).
                    let is_local = name == "self" || self.local_var_types.contains_key(name);
                    // PLAN-596 T-07 (DIV-DEP-13): const receivers (STANDARD)
                    // call methods with `.` — only type/module chains take `::`.
                    !is_local
                        && !self.is_dep_constant_name(name)
                        && (
                            Self::auto_type_to_rust(name).is_some()
                                || self.uses.iter().any(|u| {
                                    let u_str = u.as_str();
                                    u_str == name || u_str.ends_with(&format!("::{}", name))
                                })
                                || self.dep_crates.contains(id)
                                || self.module_types.contains_key(name)
                            // Plan 264
                        )
                }
                Expr::Dot(il, _) => {
                    // Plan 391 §7 follow-up: a multi-segment `::` path like
                    // `std::env::var` parses as Dot(Dot(Ident("std"),"env"),"var").
                    // The old check only inspected the root ident `std`, which
                    // is lowercase and doesn't match use "std::env" as a suffix
                    // → emitted `std.env.var` (invalid Rust). Now also test the
                    // full dotted path of `object` against use.rust imports.
                    let root_is_typeish = matches!(il.as_ref(), Expr::Ident(id) if {
                        let name = id.as_str();
                        name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                            || self.uses.iter().any(|u| {
                                let u_str = u.as_str();
                                u_str == name || u_str.ends_with(&format!("::{}", name))
                            })
                            || self.module_types.contains_key(name) // Plan 264
                    });
                    if root_is_typeish {
                        true
                    } else if let Some(path) = Self::dot_chain_path(object.as_ref()) {
                        // object is Dot(Dot(Ident("std"),"env")) → "std.env";
                        // match against use.rust imports (std::env).
                        self.path_matches_use_rust(&path)
                    } else {
                        false
                    }
                }
                Expr::Bina(il, Op::Dot, _) => {
                    matches!(il.as_ref(), Expr::Ident(id) if {
                        let name = id.as_str();
                        name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                            || self.uses.iter().any(|u| {
                                let u_str = u.as_str();
                                u_str == name || u_str.ends_with(&format!("::{}", name))
                            })
                            || self.module_types.contains_key(name) // Plan 264
                    })
                }
                _ => false,
            };

            // Regular method call: object.method(args)
            let is_insert = method_name.as_str() == "insert";
            // Look up str-param flags for auto-borrow at method call sites
            // Try qualified key "Type.method" first. Only fall back to bare "method"
            // when the method name is NOT a generic Rust method (get, insert, push, etc.)
            // to avoid false positive .as_str() on non-string args.
            let generic_rust_methods = [
                "get", "insert", "push", "remove", "contains", "len", "is_empty", "iter", "keys",
                "values", "clone", "new",
            ];
            let method_str_flags = if let Expr::Ident(obj_name) = object.as_ref() {
                // Try to infer the type from local_var_types
                let obj_type: String = self
                    .local_var_types
                    .get(obj_name)
                    .map(|ty| match ty {
                        Type::User(name) => name.to_string(),
                        Type::Enum(decl) => decl.borrow().name.to_string(),
                        _ => String::new(),
                    })
                    .unwrap_or_default();
                let qualified: AutoStr = format!("{}.{}", obj_type, method_name).into();
                let from_qualified = self.fn_str_param_indices.get(&qualified).cloned();
                if from_qualified.is_some() {
                    from_qualified
                } else if !generic_rust_methods.contains(&method_name.as_str()) {
                    self.fn_str_param_indices.get(method_name.as_str()).cloned()
                } else {
                    None
                }
            } else {
                // For non-simple objects (e.g., self.field, module.Type), don't fall back to
                // bare method name lookup — it may match wrong function signatures.
                // Only use qualified lookups.
                if let Expr::Dot(inner, type_field) = object.as_ref() {
                    if let Expr::Ident(obj_name) = inner.as_ref() {
                        let obj_type: String = self
                            .local_var_types
                            .get(obj_name)
                            .map(|ty| match ty {
                                Type::User(name) => name.to_string(),
                                Type::Enum(decl) => decl.borrow().name.to_string(),
                                _ => String::new(),
                            })
                            .unwrap_or_default();
                        // Try "obj_type.method" first, then "type_field.method"
                        let qualified: AutoStr = format!("{}.{}", obj_type, method_name).into();
                        let result = self.fn_str_param_indices.get(&qualified).cloned();
                        if result.is_some() {
                            result
                        } else {
                            // module.Type.method() — try "Type.method"
                            let type_qualified: AutoStr =
                                format!("{}.{}", type_field, method_name).into();
                            self.fn_str_param_indices
                                .get(&type_qualified)
                                .cloned()
                                .or_else(|| {
                                    self.fn_str_param_indices.get(method_name.as_str()).cloned()
                                })
                        }
                    } else {
                        // Nested: expr.Type.method() — try "Type.method"
                        let type_qualified: AutoStr =
                            format!("{}.{}", type_field, method_name).into();
                        self.fn_str_param_indices
                            .get(&type_qualified)
                            .cloned()
                            .or_else(|| {
                                self.fn_str_param_indices.get(method_name.as_str()).cloned()
                            })
                    }
                } else if let Expr::Bina(_inner, Op::Dot, rhs) = object.as_ref() {
                    // module.Type.method() via Bina — try "Type.method"
                    let type_name = if let Expr::Ident(name) = rhs.as_ref() {
                        name.to_string()
                    } else {
                        String::new()
                    };
                    if !type_name.is_empty() {
                        let type_qualified: AutoStr =
                            format!("{}.{}", type_name, method_name).into();
                        self.fn_str_param_indices
                            .get(&type_qualified)
                            .cloned()
                            .or_else(|| {
                                self.fn_str_param_indices.get(method_name.as_str()).cloned()
                            })
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            // Parenthesize object if it's a binary op (e.g., (a / b).method())
            // or a unary deref (Plan 379: (*x).clone() — `*x.clone()` would
            // parse as *(x.clone()) in Rust).
            let obj_needs_parens = matches!(object.as_ref(),
                Expr::Bina(_, op, _) if !matches!(op, Op::Dot)
            ) || matches!(object.as_ref(), Expr::Unary(Op::Mul, _));
            // Plan 395-followup: calling a task-state FIELD (`(self.cb)(ev)`)
            // needs parens — `self.cb(ev)` parses as a method call (E0599
            // "field, not a method"). The parser drops the source parens, so
            // re-add them around `self.cb` for known task state fields.
            let is_task_state_field = self.task_state_fields.contains(method_name.as_str());
            if obj_needs_parens || is_task_state_field {
                write!(out, "(")?;
            }
            // Plan 264: When object is a known module name and this is a type chain,
            // output crate::module instead of bare module name
            if obj_is_type_chain {
                if let Expr::Ident(obj_name) = object.as_ref() {
                    if self.module_types.contains_key(obj_name.as_str()) {
                        if self.merge_mode || obj_name.as_str() == self.current_module_name {
                            write!(out, "{}", obj_name)?;
                        } else {
                            write!(out, "crate::{}", obj_name)?;
                        }
                    } else {
                        self.expr(object, out)?;
                    }
                } else {
                    self.expr(object, out)?;
                }
            } else {
                self.expr(object, out)?;
            }
            if obj_needs_parens {
                write!(out, ")")?;
            }
            write!(
                out,
                "{}{}",
                if obj_is_type_chain { "::" } else { "." },
                method_name
            )?;
            if is_task_state_field {
                write!(out, ")")?;
            }
            // Plan 395: explicit generic type args → Rust turbofish
            self.emit_turbofish_args(call, out)?;
            write!(out, "(")?;
            // Add `move` for thread::spawn closures (captured locals need 'static).
            // Plan 599: skip when already `move (..) => ..` (double `move move ||`).
            if obj_is_type_chain
                && method_name == "spawn"
                && call.args.args.first().map_or(
                    false,
                    |a| matches!(a, Arg::Pos(Expr::Closure(c)) if !c.is_move),
                )
            {
                write!(out, "move ")?;
            }
            // Plan 390 §11 Phase E (D-A2): spec-param auto-boxing for method
            // calls. The method-call arg loop below is a SEPARATE emission path
            // from the free-fn arg loop (~7267) — `r.register(t)` returns from
            // this Expr::Dot handler before reaching the free-fn path. So we
            // compute spec flags here too (method_name → fn_spec_param_indices,
            // populated by the prescan for Type/Ext methods) and wrap matching
            // args in Box::new. A spec-bound ident (already Box<dyn Trait>) is
            // cloned; a concrete struct value is moved — mirroring the free-fn path.
            let method_spec_flags = self.fn_spec_param_indices.get(method_name).cloned();
            for (i, arg) in call.args.args.iter().enumerate() {
                let is_method_spec_param = method_spec_flags
                    .as_ref()
                    .and_then(|f| f.get(i))
                    .copied()
                    .unwrap_or(false);
                match arg {
                    Arg::Pos(expr) => {
                        // Auto-borrow for HashMap.contains_key(): key arg needs &
                        if i == 0 && method_name.as_str() == "contains_key" {
                            if let Expr::Ident(name) = expr {
                                let is_str_slice = self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false);
                                if !is_str_slice {
                                    write!(out, "&")?;
                                }
                            }
                        }
                        // Plan 016 Phase A A5: auto-borrow for HashMap.get() —
                        // the key arg needs & when it's a String ident. HashMap
                        // get accepts &Q where K: Borrow<Q>, so borrowing is
                        // always safe (covers owned String, for-loop vars whose
                        // type a2r didn't record, etc.). Only skip when the
                        // ident is a known &str slice (already a reference).
                        if i == 0 && method_name.as_str() == "get" {
                            if let Expr::Ident(name) = expr {
                                let is_str_slice = self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false);
                                if !is_str_slice {
                                    write!(out, "&")?;
                                }
                            }
                        }
                        // Plan 514 W3-15: is_str_param 上提(原在发射后计算,
                        // `.str()` 分支需要先行判定)+已 stringify 旗标。
                        let is_str_param = if obj_is_type_chain {
                            method_str_flags
                                .as_ref()
                                .and_then(|f| f.get(i))
                                .copied()
                                .unwrap_or(false)
                        } else {
                            method_str_flags
                                .as_ref()
                                .and_then(|f| f.get(i))
                                .copied()
                                .unwrap_or(false)
                                || method_str_flags
                                    .as_ref()
                                    .and_then(|f| f.get(i + 1))
                                    .copied()
                                    .unwrap_or(false)
                        };
                        let mut m_str_stringified_here = false;
                        // Plan 514 W3-15: Auto 通用 `.str()` 入 &str 方法参数位
                        // ——按接收者类型发射(镜像自由函数环 447-③前置同款
                        // 臂):str 系直借 .as_str(),数值/未知走 format!。
                        // 方法环此前缺此臂,方法化 lib 的 `c.emit(op, n.str(),
                        // n)` 族裸发 to_string() → E0308。
                        if is_str_param
                            && matches!(expr, Expr::Call(c2)
                                if matches!(c2.name.as_ref(), Expr::Dot(_, m2) if m2.as_str() == "str")
                                    && c2.args.args.is_empty())
                        {
                            if let Expr::Call(c2) = expr {
                                if let Expr::Dot(recv2, _) = c2.name.as_ref() {
                                    let recv_ty = self.infer_type_from_expr(recv2);
                                    let borrow_direct = matches!(
                                        recv_ty,
                                        Type::StrOwned
                                            | Type::StrSlice
                                            | Type::StrFixed(_)
                                            | Type::CStrLit
                                    );
                                    let recv_parens = matches!(recv2.as_ref(),
                                        Expr::Bina(_, op3, _) if !matches!(op3, Op::Dot));
                                    if borrow_direct {
                                        if recv_parens {
                                            write!(out, "(")?;
                                        }
                                        self.expr(recv2, out)?;
                                        if recv_parens {
                                            write!(out, ")")?;
                                        }
                                        write!(out, ".as_str()")?;
                                    } else {
                                        write!(out, "format!(\"{{}}\", ")?;
                                        self.expr(recv2, out)?;
                                        write!(out, ").as_str()")?;
                                    }
                                    m_str_stringified_here = true;
                                }
                            }
                        }
                        // Plan 390 §11 Phase E (D-A2): wrap spec-param args in
                        // Box::new. A spec-bound ident (already Box<dyn Trait>,
                        // e.g. from `Some(prof)`) is cloned to stay usable; any
                        // other expr (concrete struct value like `MyTool`, or a
                        // call/field) is moved into the box.
                        let m_spec_is_bound_ident = matches!(expr, Expr::Ident(name) if self.spec_bound_idents.contains(name));
                        if is_method_spec_param {
                            write!(out, "Box::new(")?;
                        }
                        // 2026-08-23 auto-ai report defect 3: an object/array
                        // record literal pushed into a `List<Value>` (Value =
                        // serde_json::Value) has no Rust struct to name, so the
                        // default bare-brace emission is invalid Rust ("struct
                        // literal body without path"). Emit it as a
                        // serde_json::json! invocation instead.
                        // PLAN-681 T-05（N2/P3）：同 :8676 主门扩容——
                        // 标量/串字面量与非 Value 型表达式同包 json!。
                        if method_name.as_str() == "push"
                            && self.receiver_elem_is_json_value(object)
                            && self.value_push_arg_needs_json_wrap(expr)
                        {
                            self.emit_json_macro(expr, out)?;
                        } else if !m_str_stringified_here {
                            // Plan 514 W3-15: `.str()` 分支已完整发射本实参。
                            self.expr(expr, out)?;
                        }
                        if is_method_spec_param {
                            if m_spec_is_bound_ident {
                                write!(out, ".clone())")?;
                            } else {
                                write!(out, ")")?;
                            }
                        }
                        // For .get(): auto-borrow handling done via is_str_param below.
                        // Post-processing (fix_vec_i32_index) converts .get(var) to [var as usize]
                        // for Vec accesses, so we don't add as usize here.
                        // For Map.insert(): auto-convert to String based on Map value type.
                        // - Key (i==0): add .to_string() ONLY for string-like args
                        //   (str literals / &str vars / String locals). Composite keys
                        //   (tuples, Plan 018 §14 W1) must NOT get .to_string() —
                        //   they're already the exact key type and tuples have no
                        //   Display (E0277).
                        // - Value (i==1): only add .to_string() when Map value type is String
                        if is_insert && !matches!(expr, Expr::Int(_) | Expr::Bool(_)) {
                            let should_to_string = if i == 0 {
                                matches!(expr, Expr::Str(_) | Expr::CStr(_))
                                    || if let Expr::Ident(name) = expr {
                                        self.local_var_types
                                            .get(name)
                                            .map(|ty| {
                                                matches!(
                                                    ty,
                                                    Type::StrFixed(_)
                                                        | Type::StrSlice
                                                        | Type::StrOwned
                                                )
                                            })
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    }
                            } else {
                                // value arg: check Map value type from local_var_types
                                self.expr_map_value_is_string(object)
                            };
                            if should_to_string {
                                write!(out, ".to_string()")?;
                            }
                        }
                        // Auto-borrow: add .as_str() when passing String to &str method param
                        // (is_str_param 已在循环体首上提——Plan 514 W3-15)
                        // Plan 371 (defect C): the i+1 lookahead above reads the
                        // NEXT param's flag, which wrongly flags enum/struct args
                        // as str params. Guard: only auto-borrow when the arg is
                        // genuinely a string-like value (a str literal, or a local
                        // var whose type is a str variant). Enum/struct/int args
                        // are skipped regardless of the flag.
                        let arg_is_str_like = matches!(expr, Expr::Str(_) | Expr::CStr(_))
                            || self.is_str_slice_var(arg)
                            // Plan 514 W3:字符串拼接(Bina+Add → 发射 format!
                            // 返 String)入 str 参数位需 .as_str()(lib 方法体
                            // `self.fail(".." + x)` 实证)
                            || matches!(expr, Expr::Bina(_, auto_val::Op::Add, _))
                            // Plan 514 W3:str 载荷复合表达式(List<str> 的
                            // get/index、str 返回的用户方法调用——`p.bind(
                            // binds.get(i2), ...)` 族)经 infer 解析。
                            || matches!(self.infer_type_from_expr(expr),
                                Type::StrOwned | Type::StrSlice
                                    | Type::StrFixed(_) | Type::CStrLit)
                            || if let Expr::Ident(name) = expr {
                                // Plan 514 W3:类型未知的局部(lib 的
                                // `var name = p.text()` 族——未注解 var 经
                                // 类型推断注册为 Type::Unknown)按 owned
                                // String 处理——非 str 参数(那些已是
                                // &str)即 as_str。
                                if matches!(
                                    self.local_var_types.get(name),
                                    None | Some(Type::Unknown)
                                ) && !self.current_fn_str_params.contains(name) {
                                    true
                                } else {
                                    self.local_var_types.get(name).map(|ty| matches!(
                                        ty,
                                        Type::StrFixed(_) | Type::StrSlice | Type::StrOwned
                                    )).unwrap_or(false)
                                }
                            } else {
                                false
                            };
                        if is_str_param
                            && arg_is_str_like
                            && !m_str_stringified_here
                            // Plan 514 W3:字符串字面量本身就是 &str——
                            // 追加 .as_str() 触发 E0658(str_as_str 未稳定);
                            // 自由函数环 is_string_literal_arg 同款排除。
                            && !matches!(expr, Expr::Str(_) | Expr::CStr(_))
                            && !matches!(expr, Expr::Int(_) | Expr::Float(_, _))
                            && !Self::is_int_var(arg, &self.local_var_types)
                        {
                            // Plan 376 Pass 7: skip .as_str() when the arg variable
                            // is already &str (StrSlice) — adding .as_str() on a &str
                            // triggers E0658 (unstable str_as_str feature).
                            let arg_already_str_slice = if let Expr::Ident(name) = expr {
                                // Plan 514 W3:仅 str 参数(current_fn_str_params)
                                // 真是 &str;StrSlice 注册的局部来自 str 返回值
                                // 调用(lib `var name = p.text()` 族)在 Rust 侧
                                // 是 String——需要 .as_str(),不得跳过。
                                self.local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false)
                                    && self.current_fn_str_params.contains(name)
                            } else {
                                false
                            };
                            if !arg_already_str_slice {
                                write!(out, ".as_str()")?;
                            }
                        }
                        // Auto-borrow for external crate calls: when calling crate::method()
                        // with a String-typed variable, add .as_str() since most Rust
                        // APIs accept &str rather than String.
                        if obj_is_type_chain && !is_str_param {
                            if let Expr::Ident(name) = expr {
                                if self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::StrOwned | Type::StrFixed(_)))
                                    .unwrap_or(false)
                                {
                                    write!(out, ".as_str()")?;
                                }
                            }
                        }
                        // Auto-clone: .push() takes ownership, clone non-Copy ident args
                        if method_name.as_str() == "push" {
                            if let Expr::Ident(name) = expr {
                                // Plan 434 (242 #18-2): a `str`-slice ident is
                                // &str at runtime (str param / str-return fn) —
                                // pushing into a String element Vec needs
                                // `.to_string()` (`.clone()` keeps &str).
                                let str_slice = self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| {
                                        matches!(
                                            ty,
                                            Type::StrFixed(_)
                                                | Type::StrSlice
                                                | Type::StrOwned
                                                | Type::CStrLit
                                        )
                                    })
                                    .unwrap_or(false)
                                    || (self.local_var_types.get(name).is_none()
                                        && self.current_fn_str_params.contains(name));
                                if str_slice {
                                    write!(out, ".to_string()")?;
                                } else {
                                    let is_copy = self
                                        .local_var_types
                                        .get(name)
                                        .map(|ty| Self::is_copy_type(ty))
                                        .unwrap_or(false);
                                    if !is_copy {
                                        write!(out, ".clone()")?;
                                    }
                                }
                            }
                        }
                        // Auto-clone: .insert() takes ownership of value (2nd arg), clone non-Copy ident args
                        // Skip 1st arg (key) — it's usually String/Copy. Only clone the value arg.
                        if is_insert && i >= 1 {
                            if let Expr::Ident(name) = expr {
                                let is_copy = self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| Self::is_copy_type(ty))
                                    .unwrap_or(false);
                                if !is_copy {
                                    write!(out, ".clone()")?;
                                }
                            }
                        }
                    }
                    other => self.arg(other, out)?,
                }
                if i < call.args.args.len() - 1 {
                    write!(out, ", ")?;
                }
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
        // Also handles module.Type.Variant(value) → module::Type::Variant(value)
        // Parser produces both Expr::Bina and Expr::Dot for dot paths:
        //   Type.Variant → Expr::Bina(Type, Dot, Variant) or Expr::Dot(Type, Variant)
        //   module.Type.Variant → Expr::Dot(Expr::Bina(module, Dot, Type), Variant)
        {
            let mut tag_match: Option<(Option<AutoStr>, AutoStr, AutoStr)> = None;
            // Pattern A: Expr::Bina(lhs, Dot, rhs) — Type.Variant or module.Type.Variant
            if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
                if matches!(op, Op::Dot) {
                    // Two-level: Type.Variant
                    if let Expr::Ident(type_name) = lhs.as_ref() {
                        if let Expr::Ident(variant_name) = rhs.as_ref() {
                            if self.tag_types.contains(type_name) {
                                tag_match = Some((None, type_name.clone(), variant_name.clone()));
                            }
                        }
                    }
                    // Three-level via Bina: module.Type.Variant
                    if tag_match.is_none() {
                        if let Expr::Bina(inner_lhs, inner_op, inner_rhs) = lhs.as_ref() {
                            if matches!(inner_op, Op::Dot) {
                                if let Expr::Ident(mod_name) = inner_lhs.as_ref() {
                                    if let Expr::Ident(type_name) = inner_rhs.as_ref() {
                                        if let Expr::Ident(variant_name) = rhs.as_ref() {
                                            if self.tag_types.contains(type_name)
                                                || self.module_types.contains_key(mod_name.as_str())
                                            {
                                                tag_match = Some((
                                                    Some(mod_name.clone()),
                                                    type_name.clone(),
                                                    variant_name.clone(),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Pattern B: Expr::Dot(object, field) — Type.Variant or module.Type.Variant
            if tag_match.is_none() {
                if let Expr::Dot(obj, field_name) = call.name.as_ref() {
                    // Two-level: Type.Variant via Dot — only match if type_name is a known tag type
                    if let Expr::Ident(type_name) = obj.as_ref() {
                        if self.tag_types.contains(type_name) {
                            tag_match = Some((None, type_name.clone(), field_name.clone()));
                        }
                    }
                    // Three-level: module.Type.Variant via Dot(Bina(module, Dot, Type), Variant)
                    if tag_match.is_none() {
                        if let Expr::Bina(inner_lhs, inner_op, inner_rhs) = obj.as_ref() {
                            if matches!(inner_op, Op::Dot) {
                                if let Expr::Ident(mod_name) = inner_lhs.as_ref() {
                                    if let Expr::Ident(type_name) = inner_rhs.as_ref() {
                                        if self.tag_types.contains(type_name)
                                            || self.module_types.contains_key(mod_name.as_str())
                                        {
                                            tag_match = Some((
                                                Some(mod_name.clone()),
                                                type_name.clone(),
                                                field_name.clone(),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Three-level: module.Type.Variant via Dot(Dot(module, Type), Variant)
                    if tag_match.is_none() {
                        if let Expr::Dot(inner_obj, inner_type_name) = obj.as_ref() {
                            if let Expr::Ident(mod_name) = inner_obj.as_ref() {
                                if self.tag_types.contains(inner_type_name)
                                    || self.module_types.contains_key(mod_name.as_str())
                                {
                                    tag_match = Some((
                                        Some(mod_name.clone()),
                                        inner_type_name.clone(),
                                        field_name.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            if let Some((mod_prefix, type_name, variant_name)) = tag_match {
                let key = (type_name.clone(), variant_name.clone());
                let struct_fields = self.enum_struct_variants.get(&key).cloned();
                // Tag construction with optional module prefix
                if let Some(ref mp) = mod_prefix {
                    if self.merge_mode || mp.as_str() == self.current_module_name {
                        write!(out, "{}::{}::{}", mp, type_name, variant_name)?;
                    } else if self.module_types.contains_key(mp.as_str()) {
                        write!(out, "crate::{}::{}::{}", mp, type_name, variant_name)?;
                    } else {
                        write!(out, "{}::{}::{}", mp, type_name, variant_name)?;
                    }
                } else {
                    write!(out, "{}::{}", type_name, variant_name)?;
                }
                if let Some(fields) = struct_fields {
                    // Struct variant: Type::Variant { field1: val1, field2: val2 }
                    write!(out, " {{ ")?;
                    for (i, (arg, field_name)) in
                        call.args.args.iter().zip(fields.iter()).enumerate()
                    {
                        if let Arg::Pos(expr) = arg {
                            write!(out, "{}: ", field_name)?;
                            self.expr(expr, out)?;
                            if matches!(expr, Expr::Str(_) | Expr::CStr(_)) {
                                write!(out, ".to_string()")?;
                            }
                        }
                        if i < call.args.args.len().min(fields.len()) - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, " }}")?;
                } else if call.args.args.is_empty() && !self.enum_tuple_variants.contains_key(&key)
                {
                    // Plan 016 Phase 4: zero-arg call on a variant with no
                    // registered payload is a UNIT variant (Auto spells it
                    // `Value.Null()` for a2ts parity) — `Type::Variant` is
                    // already written; parens would be E0618.
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
                                let field_is_string = tuple_field_types
                                    .as_ref()
                                    .and_then(|types| types.get(i))
                                    .map(|ty| {
                                        matches!(
                                            ty,
                                            Type::StrOwned | Type::StrFixed(_) | Type::StrSlice
                                        )
                                    })
                                    .unwrap_or(false);
                                let var_is_str_slice = self
                                    .local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false);
                                if field_is_string && var_is_str_slice {
                                    write!(out, ".to_string()")?;
                                }
                            }
                        }
                        if i < call.args.args.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, ")")?;
                }
                return Ok(());
            }
        }

        // Check if this is a struct construction call: Type(args)
        // Heuristic: If the callee name starts with uppercase, treat as type construction
        // This works because Rust convention: TypeNames are CamelCase, functions are snake_case
        // Exception: SCREAMING_CASE names (OP_XXX, BOOL_XXX) are constants/functions, not types
        if let Expr::Ident(type_name) = call.name.as_ref() {
            let first_char_upper = type_name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);
            let is_screaming_case = type_name
                .chars()
                .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
                && type_name.contains('_');
            if first_char_upper && !is_screaming_case {
                // This is a struct construction: Type { field1: value1, ... }
                return self.struct_init(type_name, &call.args, out);
            }
        }

        // Plan 204 Phase 5: Auto stdlib free function -> Rust equivalents
        if let Expr::Ident(fn_name) = call.name.as_ref() {
            match fn_name.as_str() {
                "min" => {
                    // min(a, b) -> a2r_std::math::min(a, b)
                    self.a2r_std_used.set(true);
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
                    self.a2r_std_used.set(true);
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

        // Plan 264: Handle module.Type(args) constructor calls
        // e.g., types.ToolChatRequest(a, b, c) → crate::types::ToolChatRequest { field1: a, ... }
        if let Expr::Dot(obj, type_name) = call.name.as_ref() {
            if let Expr::Ident(module_name) = obj.as_ref() {
                if type_name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    let is_module = self.module_types.contains_key(module_name.as_str())
                        || self.uses.iter().any(|u| {
                            let u_str = u.as_str();
                            u_str == module_name.as_str()
                                || u_str.ends_with(&format!("::{}", module_name))
                        });
                    if is_module {
                        let qualified = if self.merge_mode
                            || module_name.as_str() == self.current_module_name
                        {
                            format!("{}::{}", module_name, type_name)
                        } else {
                            format!("crate::{}::{}", module_name, type_name)
                        };
                        // Use bare type name for struct_fields lookup
                        let field_names = self
                            .struct_fields
                            .get(type_name)
                            .cloned()
                            .unwrap_or_default();
                        let field_types = self
                            .struct_field_types
                            .get(type_name)
                            .cloned()
                            .unwrap_or_default();

                        if call.args.args.is_empty() {
                            write!(out, "{} {{}}", qualified)?;
                            return Ok(());
                        }
                        write!(out, "{} {{ ", qualified)?;
                        for (i, arg) in call.args.args.iter().enumerate() {
                            let field_name = field_names
                                .get(i)
                                .map(|n| n.as_str())
                                .unwrap_or_else(|| if i == 0 { "field0" } else { "fieldN" });
                            write!(out, "{}: ", field_name)?;
                            self.arg(arg, out)?;
                            // Auto .to_string() when assigning &str to String field
                            if let Some((_, ft)) = field_types.get(i) {
                                if matches!(ft, Type::StrOwned | Type::StrFixed(_)) {
                                    if let Arg::Pos(expr) = arg {
                                        if self.needs_as_str(expr) {
                                            write!(out, ".to_string()")?;
                                        }
                                    }
                                }
                            }
                            if i < call.args.args.len() - 1 {
                                write!(out, ", ")?;
                            }
                        }
                        write!(out, " }}")?;
                        return Ok(());
                    }
                }
            }
        }

        // Normal function call
        // In merge mode, if callee is a known const name with no args, emit bare const reference
        if call.args.args.is_empty() {
            if let Expr::Ident(fn_name) = call.name.as_ref() {
                if self.const_names.contains(fn_name) {
                    self.expr(&call.name, out)?;
                    return Ok(());
                }
            }
        }
        self.expr(&call.name, out)?;
        // Plan 395: explicit generic type args → Rust turbofish
        self.emit_turbofish_args(call, out)?;
        write!(out, "(")?;

        // Look up str-param flags for auto-borrow at call sites
        let str_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_str_param_indices.get(fn_name).cloned()
        } else {
            // Try to extract the last segment of a qualified path like crate::forge::func
            let last_seg = match call.name.as_ref() {
                Expr::Dot(_, field) => Some(field.as_str()),
                Expr::Bina(_, Op::Dot, rhs) => {
                    if let Expr::Ident(name) = rhs.as_ref() {
                        Some(name.as_str())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(name) = last_seg {
                self.fn_str_param_indices.get(name).cloned()
            } else {
                None
            }
        };

        // Look up struct-param flags for auto-clone at call sites
        let struct_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_struct_param_indices.get(fn_name).cloned()
        } else {
            None
        };

        // Look up merge-mode &mut flags (context types skip .clone())
        let merge_mut_flags = if self.merge_mode {
            if let Expr::Ident(fn_name) = call.name.as_ref() {
                self.fn_merge_mut_params.get(fn_name).cloned()
            } else {
                None
            }
        } else {
            None
        };

        // C11 (Plan 018 §12 a2r-11): callee `mut p T` params → pass `&mut arg`.
        let mut_param_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_mut_params.get(fn_name).cloned()
        } else {
            None
        };

        // Look up spec-param flags for auto-boxing at call sites
        // Plan 390 §11 Phase E (D-B): handle method calls (Expr::Dot) via the same
        // last-segment fallback used by str_flags above, so `r.register(t)` resolves
        // to the "Type.register" / "register" key populated in the prescan (Fix A).
        let spec_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_spec_param_indices.get(fn_name).cloned()
        } else {
            // Try to extract the last segment of a qualified path / method call.
            let last_seg = match call.name.as_ref() {
                Expr::Dot(_, field) => Some(field.as_str()),
                Expr::Bina(_, Op::Dot, rhs) => {
                    if let Expr::Ident(name) = rhs.as_ref() {
                        Some(name.as_str())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(name) = last_seg {
                self.fn_spec_param_indices.get(name).cloned()
            } else {
                None
            }
        };

        // Look up int-param flags for enum→i32 cast at call sites
        let int_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_int_param_indices.get(fn_name).cloned()
        } else {
            None
        };

        // Look up full param types for type-aware call site generation
        let param_types = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_param_types.get(fn_name).cloned()
        } else {
            None
        };

        for (i, arg) in call.args.args.iter().enumerate() {
            let is_str_param = str_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            let needs_borrow =
                is_str_param && !Self::is_string_literal_arg(arg) && !self.is_str_slice_var(arg);

            // Plan 384 A3: reference-param borrow injection. When the callee's
            // declared param type is `Type::Reference(T)` (e.g. extern_impl
            // stubs declared via extern_sigs.at with `@T`), borrow a owned
            // argument (`&arg`) so the call compiles against `&T` params.
            // String literals, already-`&str` slice vars, and `self` are left
            // untouched (they coerce or are already borrowed).
            let needs_ref_borrow = param_types
                .as_ref()
                .and_then(|pts| pts.get(i))
                .map(|pt| matches!(pt, Type::Reference(_)))
                .unwrap_or(false)
                && !Self::is_string_literal_arg(arg)
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    // Borrow owned locals/params; skip str-slice vars (already &str)
                    // and `self` (autoref) and already-clone suffixed.
                    name.as_str() != "self" && !self.is_str_slice_var(arg)
                } else if let Arg::Pos(expr) = arg {
                    // Borrow struct-literal / method-call / field-access args
                    !matches!(
                        expr,
                        Expr::Str(_) | Expr::CStr(_) | Expr::Int(_) | Expr::Float(_, _)
                    )
                } else {
                    false
                };

            // Plan 347: Fallback auto-borrow for cross-module / imported
            // function calls. When the callee's parameter types are not in the
            // local cache (`str_flags` is None — typical for functions imported
            // via `use crate::<mod>:<fn>`), and the argument is a `String`-
            // typed local variable, assume the callee takes `&str` (the Auto
            // `str` convention) and borrow the argument. This mirrors what the
            // same-module str-param path does above. String literals and
            // already-`&str` params are left untouched.
            // Plan 381 (Layer 2): enum variant CONSTRUCTION
            // (`OutputContentBlock.ToolUse(id, name, input)`) parses as a
            // dotted call whose callee param types are unknown — the fallback
            // below would append .as_str() to owned String payloads (E0308
            // `String` vs `&str`). Skip the fallback for known enum ctors.
            let is_enum_ctor = self.is_enum_variant_ctor(call);
            let needs_borrow_unknown_callee = str_flags.is_none()
                && !is_enum_ctor
                && !Self::is_string_literal_arg(arg)
                && !self.is_str_slice_var(arg)
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    // Only borrow owned String locals; never borrow params
                    // (those are &str already) or non-string variables. A
                    // local declared `str` is recorded as StrSlice but renders
                    // as an owned `String` in Rust, so it still needs borrowing.
                    !self.current_fn_str_params.contains(name)
                        && self
                            .local_var_types
                            .get(name)
                            .map(|ty| {
                                matches!(
                                    ty,
                                    Type::StrOwned
                                        | Type::StrFixed(_)
                                        | Type::CStrLit
                                        | Type::StrSlice
                                )
                            })
                            .unwrap_or(false)
                } else if let Arg::Pos(expr) = arg {
                    // Plan 368 FU-2: an inline string concatenation
                    // (e.g. `base + "/x"`) renders to an owned `String`
                    // (format!(...)) — borrow it when calling an imported fn
                    // that takes &str. Matches the same convention as the Ident
                    // branch above. expr_contains_string detects any concat
                    // involving a string operand.
                    self.expr_contains_string(expr)
                } else {
                    false
                };

            // Auto-cast enum→i32 when passing an enum variable to an Int param
            let is_int_param = int_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            let needs_enum_cast = is_int_param
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    self.local_var_types
                        .get(name)
                        .map(|ty| match ty {
                            Type::Enum(_) => true,
                            Type::User(td) => self.known_enum_names.contains(&td.name),
                            _ => false,
                        })
                        .unwrap_or(false)
                } else {
                    false
                };

            // Auto-clone when passing a variable to a function that takes a struct param
            let is_struct_param = struct_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            // C11 (Plan 018 §12 a2r-11): callee param is `mut p T` (&mut T) →
            // pass `&mut arg` (never arg.clone()).
            let is_mut_param = mut_param_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            // Skip .clone() for merge-mode context types (they use &mut instead)
            let is_merge_mut = merge_mut_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            // Plan 347: StringBuilder params are passed by &mut reference. The
            // callee declares `sb: &mut a2r_std::StringBuilder`, so the caller
            // must pass `&mut sb` (never `sb.clone()` — that would break the
            // shared accumulator). This is independent of merge_mode.
            let is_sb_param = if matches!(arg, Arg::Pos(Expr::Ident(_))) {
                param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| Self::is_sb_ref_type(pt))
                    .unwrap_or(false)
            } else {
                false
            };
            // Check param type from fn_param_types for auto &mut insertion
            // Skip if the variable is already a &mut param of the current function
            let is_already_mut_param = if let Arg::Pos(Expr::Ident(name)) = arg {
                self.current_fn_mut_params.contains(name)
            } else {
                false
            };
            let needs_mut_borrow = if self.merge_mode
                && matches!(arg, Arg::Pos(Expr::Ident(_)))
                && !is_already_mut_param
            {
                param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| Self::is_merge_mut_type(pt))
                    .unwrap_or(false)
            } else {
                false
            };
            // Plan 433 A1: passing an &mut param of the current fn to a
            // BY-VALUE param (non-mut callee) must deref-clone (&mut Vec<T> →
            // Vec<T>); a raw ident would move the reference (E0308).
            let needs_mut_param_clone = is_already_mut_param
                && !is_mut_param
                && !needs_mut_borrow
                && !is_merge_mut
                && !is_sb_param
                && matches!(arg, Arg::Pos(Expr::Ident(_)));
            // Plan 016 (auto-down R4): passing an owned `Vec<T>` ident (e.g.
            // the current fn's own `List` param, `repl` in a recursive tree
            // walk) to a by-value `List` param MOVES it — inside a loop the
            // move fires on the first iteration; outside, the second use is
            // E0382 (e.g. `attrGet(ialAttrs, ..)` twice). Clone instead.
            // Guarded to idents whose local type is itself an owned List (so
            // &T/&mut params and iter vars are untouched).
            let is_owned_list_arg = !is_struct_param
                && param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| matches!(pt, Type::List(_)))
                    .unwrap_or(false)
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    self.local_var_types.get(name)
                        // Plan 514 W1 (m05): 数组字面量绑定是 Type::Array,
                        // Rust 侧同为 owned Vec<T>——按值传入 List 参数同样
                        // move,复用即 E0382;与 List 同列。
                        .map(|ty| matches!(ty, Type::List(_) | Type::Array(_)))
                        .unwrap_or(false)
                        // PLAN-018: 全局 List 读作 owned 实参同 move
                        // (E0507 cannot move out of MutexGuard)——同款克隆。
                        || self.global_var_types.get(name.as_str())
                            .map(|ty| matches!(ty, Type::List(_) | Type::Array(_)))
                            .unwrap_or(false)
                } else {
                    false
                };
            // Plan 016 Phase 4: a field-read (`s.marks`, `node.attrs`, `a.sel`)
            // passed to a by-value OWNED param (Vec/struct/enum/Map — anything
            // param_takes_ownership flags) MOVES the field out of its binding;
            // the next use of the same path is E0382 (or E0507 when the source
            // is behind a shared ref). Clone the field instead. Str params are
            // excluded (the .as_str() borrow path handles them), as are
            // borrowed-iter-var fields — the defect-4 branch below decides
            // those first.
            // Plan 531 真缺陷①(pointer/004): `x.@` 取址伪字段发射 `x as *mut _`
            // ——裸指针 Copy 无需 clone,且 as 强转后接 `.clone()` 是语法错
            // (cast cannot be followed by a method call),排除出本支路。
            let dot_arg_owned_param = matches!(arg, Arg::Pos(Expr::Dot(_, f)) if f.as_str() != "@")
                && !needs_ref_borrow
                && param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| Self::param_takes_ownership(pt))
                    .unwrap_or(false);
            let needs_clone = (is_struct_param || needs_mut_param_clone || is_owned_list_arg
                    || dot_arg_owned_param)
                && !is_merge_mut && !needs_mut_borrow
                && !is_mut_param
                && !is_sb_param
                && !needs_ref_borrow
                && (matches!(arg, Arg::Pos(Expr::Ident(_))) || dot_arg_owned_param)
                // Plan 380: spec-bound idents (`Some(prof)` from an Option<Spec>
                // scrutinee) are `Box<dyn Trait>` — no Clone impl (E0599).
                // Pass them by value (move) instead.
                && !(if let Arg::Pos(Expr::Ident(name)) = arg {
                    self.spec_bound_idents.contains(name)
                } else { false })
                // PLAN-010 T2 (009 ledger 033): Result<_, Box<dyn Error>>
                // locals — Box<dyn Error> is not Clone, so the safety clone is
                // E0599 ("trait bounds not satisfied"). Pass by value.
                && !(if let Arg::Pos(Expr::Ident(name)) = arg {
                    let err_not_clone = |err: &Type| matches!(err,
                        Type::GenericInstance(g) if g.base_name.as_str() == "Box")
                        || matches!(err, Type::Unknown);
                    match self.local_var_types.get(name) {
                        Some(Type::Result(err)) => err_not_clone(err),
                        // `Result<i64, Box<dyn Error>>` annotates as a
                        // GenericInstance (base "Result"), not Type::Result.
                        Some(Type::GenericInstance(inst))
                            if inst.base_name.as_str() == "Result" && inst.args.len() == 2 =>
                            err_not_clone(&inst.args[1]),
                        _ => false,
                    }
                } else { false });

            // Auto-box when passing a value to a function that takes a spec param
            let is_spec_param = spec_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);

            if is_spec_param {
                write!(out, "Box::new(")?;
            }

            // Auto &mut for context-type params in merge mode
            if needs_mut_borrow {
                write!(out, "&mut ")?;
            }
            // C11 (Plan 018 §12 a2r-11): `mut p T` callee param → pass `&mut arg`.
            // Plan 433 A1: when the arg ident is itself an &mut param of the
            // current fn, re-borrow (`&mut *arg`) — `&mut arg` on a `p: &mut T`
            // binding is E0596 (binding not declared mut) and would move the
            // reference, killing later uses. The ident itself is emitted by
            // arg() below, so only the `*` differs here.
            if is_mut_param {
                if is_already_mut_param {
                    write!(out, "&mut *")?;
                } else {
                    write!(out, "&mut ")?;
                }
            }
            // Plan 384 A3: borrow owned arg for `&T` reference params.
            if needs_ref_borrow {
                write!(out, "&")?;
            }
            // Plan 347: StringBuilder params take &mut at the call site.
            if is_sb_param {
                write!(out, "&mut ")?;
            }

            // Plan 013 (B16): if the ident was bound inside an `is` pattern for
            // a bridge-crate variant (e.g. `Kid.Node(child)`), it's a Box<T> in
            // Rust — emit `(*ident).clone()` so the inner value is cloned, not
            // the Box wrapper. Handle this before self.arg() writes the ident.
            let bridge_box_ident = if needs_clone {
                if let Arg::Pos(Expr::Ident(name)) = arg {
                    if self.bridge_pattern_bound_idents.contains(name) {
                        Some(name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            // Plan 447 ③-前置: set when a branch below already emitted the arg
            // WITH its `.as_str()` borrow — the generic borrow append at the end
            // of the loop iteration must not fire again.
            let mut str_stringified_here = false;
            if let Some(name) = bridge_box_ident {
                // Rust method resolution: `(*ident).clone()` on a Box<T> still
                // autorefs back to `Box::<T>::clone()` → returns Box<T>, not T.
                // The double-deref `*(*ident).clone()` unwraps the re-cloned
                // Box, yielding T. (Plan 013 B16.)
                write!(out, "*(*{}).clone()", Self::rust_ident(name.as_str()))?;
            } else {
                // Plan 380: when passing a trim*() call (which the expr handler
                // renders with a trailing .to_string()) to a `&str` param, emit
                // via expr_as_str so the suffix is stripped — trim() already
                // returns &str (`clean_field_value(r.trim())` → E0308 String).
                let trim_arg = if let Arg::Pos(expr) = arg {
                    Self::is_trim_method_call(expr)
                } else {
                    false
                };
                if is_str_param && trim_arg {
                    if let Arg::Pos(expr) = arg {
                        self.expr_as_str(expr, out)?;
                    }
                } else if is_str_param
                    && matches!(arg, Arg::Pos(Expr::Call(c))
                        if matches!(c.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "str")
                            && c.args.args.is_empty())
                {
                    // Plan 447 ③-前置(②列基线修复): Auto's universal
                    // `.str()` in a &str slot — emit by RECEIVER type. The
                    // plain path wrote `x.to_string()` for both str and int
                    // receivers, then the borrow append gave
                    // `x.to_string().as_str()` — text-reducer
                    // fix_string_str_mismatches collapses that to
                    // `x.as_str()`, E0599 on i64/char/bool receivers
                    // (codegen.at `emit("const.i32", cp.str(), cp)`).
                    // Str receivers keep the collapsed form byte-for-byte;
                    // known numeric receivers go through format! (same shape
                    // fix_int_as_str produces).
                    if let Arg::Pos(Expr::Call(c)) = arg {
                        if let Expr::Dot(recv, _) = c.name.as_ref() {
                            let recv_ty = self.infer_type_from_expr(recv);
                            // Numeric AND unresolvable receivers go through
                            // format! — the text reducer collapses the plain
                            // `x.to_string().as_str()` form back to `x.as_str()`
                            // (E0599 on i64). Str-typed receivers keep the
                            // direct borrow (byte-identical to the collapsed
                            // legacy form, zero golden drift).
                            let borrow_direct = matches!(
                                recv_ty,
                                Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                            );
                            let recv_parens = matches!(recv.as_ref(),
                                Expr::Bina(_, op, _) if !matches!(op, Op::Dot));
                            if borrow_direct {
                                if recv_parens {
                                    write!(out, "(")?;
                                }
                                self.expr(recv, out)?;
                                if recv_parens {
                                    write!(out, ")")?;
                                }
                                write!(out, ".as_str()")?;
                            } else {
                                write!(out, "format!(\"{{}}\", ")?;
                                self.expr(recv, out)?;
                                write!(out, ").as_str()")?;
                            }
                            str_stringified_here = true;
                        }
                    }
                } else if needs_ref_borrow && matches!(arg, Arg::Pos(Expr::Dot(_, _))) {
                    // Plan 018 §Phase 3.5: `&self.field` passed to a `@T` (&T)
                    // param must NOT get the self-dot `.clone()` that `arg()`
                    // appends — that produces `&self.field.clone()` (E0599 no
                    // clone on Mutex). The borrow is a shared reference, so the
                    // field is moved-by-borrow, not cloned. Emit the expr
                    // directly and let the `&` written above apply.
                    if let Arg::Pos(expr) = arg {
                        self.expr(expr, out)?;
                    }
                } else if {
                    // 2026-08-23 auto-ai report defect 4: a field of a
                    // borrowed loop var (`m.content` in `for m in &req.messages`)
                    // passed to a str/List param — arg()'s §2.1 conservative
                    // clone (`m.content.clone()`) regressed these call sites
                    // (daemon's content_blocks_to_anthropic / openai_content).
                    // Copy-ish callee params (str borrows, List passed by
                    // reference per the is_copy_type convention) don't take
                    // ownership, so no clone is needed. Non-Copy params (e.g.
                    // `extract_path(tc.args)` with a Value param) still clone
                    // via arg() below.
                    let iter_field = matches!(arg, Arg::Pos(Expr::Dot(_, _)))
                        && if let Arg::Pos(Expr::Dot(obj, _)) = arg {
                            matches!(obj.as_ref(), Expr::Ident(n) if self.borrowed_iter_vars.contains(n))
                        } else {
                            false
                        };
                    iter_field
                        && match param_types.as_ref().and_then(|pts| pts.get(i)) {
                            // Known callee signature: skip the clone only
                            // when the param does NOT take ownership. Plan
                            // 016 Phase 4: `List<T>` params are emitted as
                            // owned `Vec<T>` (not `&[T]`), so a List field of
                            // a borrowed loop var must clone (E0507) —
                            // is_copy_type's List-is-borrowed convention does
                            // not hold at fn boundaries.
                            Some(pt) => !Self::param_takes_ownership(pt),
                            // Unknown callee (imported fn — single-file
                            // transpile can't see its signature): keep the
                            // pre-§2.1 shape (no clone), matching the sed
                            // contract of cross-file consumers.
                            None => true,
                        }
                } {
                    // 2026-08-23 auto-ai report defect 4: a field of a
                    // borrowed loop var (`m.content` in `for m in &req.messages`)
                    // passed to a str/List param — arg()'s §2.1 conservative
                    // clone (`m.content.clone()`) regressed these call sites
                    // (daemon's content_blocks_to_anthropic / openai_content).
                    // Copy-ish callee params (str borrows, List passed by
                    // reference per the is_copy_type convention) don't take
                    // ownership, so no clone is needed. Non-Copy params (e.g.
                    // `extract_path(tc.args)` with a Value param) still clone
                    // via arg() below.
                    if let Arg::Pos(expr) = arg {
                        self.expr(expr, out)?;
                    }
                } else if needs_clone {
                    // 2026-08-23 auto-ai report defect 4: a borrowed loop
                    // var (`t` in `for t in &coll`) already gets its deref
                    // clone from arg()'s maybe_clone_borrowed_iter_field —
                    // running self.arg() here and appending another produced
                    // `t.clone().clone()`. Emit the raw expression with
                    // exactly one clone.
                    if let Arg::Pos(expr) = arg {
                        self.expr(expr, out)?;
                    }
                    write!(out, ".clone()")?;
                } else {
                    self.arg(arg, out)?;
                }
            }

            // After expression: add .as_str() for String→&str conversion
            // Plan 376 Pass 7: skip .as_str() when the callee param is already &str
            // (the callee declared `param str` which renders as &str in Rust).
            // In this case needs_borrow is true (is_str_param), but the argument
            // is already &str — adding .as_str() causes E0658 (unstable feature).
            // The fix: if the arg variable's type IS StrSlice (it's a &str param),
            // don't add .as_str() even when is_str_param says to borrow.
            let arg_is_str_slice = if let Arg::Pos(Expr::Ident(name)) = arg {
                // Only fn params declared `str` render as &str. local_var_types
                // records EVERY str-typed var (incl. owned String locals) as
                // StrSlice, so consulting it here would wrongly skip .as_str()
                // on locals → E0308 (String vs &str). Check the fn-param set
                // (mirrors is_str_slice_var).
                self.current_fn_str_params.contains(name)
            } else {
                false
            };
            let arg_is_str_literal =
                matches!(arg, Arg::Pos(Expr::Str(_)) | Arg::Pos(Expr::CStr(_)));
            // Plan 380: only add .as_str() for plain String-typed locals or
            // string-concat exprs. Method-call results (trim(), to_str().unwrap(),
            // ...) and char literals must NOT get .as_str() — they're already
            // &str or char (E0658 str_as_str / E0599 char.as_str).
            let arg_is_ident = matches!(arg, Arg::Pos(Expr::Ident(_)));
            let arg_is_concat = if let Arg::Pos(expr) = arg {
                self.expr_contains_string(expr)
            } else {
                false
            };
            // Plan 417-E1 rider: a plain call to a user fn whose DECLARED
            // return is `str` renders as an owned String — borrowing it for
            // a &str param needs the same `.as_str()` as ident args. Plan
            // 380's restriction to idents/concats skipped call args, so
            // parity string_utils' `check_str(1, ..., reverse("abc"), ...)`
            // failed E0308 (String vs &str). Only Ident-named calls with a
            // known str return type qualify (method chains are already &str
            // and stay excluded).
            let arg_is_str_returning_call = if let Arg::Pos(Expr::Call(c)) = arg {
                if let Expr::Ident(fname) = c.name.as_ref() {
                    self.fn_ret_types
                        .get(fname)
                        .map(|ty| {
                            matches!(
                                ty,
                                Type::StrOwned | Type::StrFixed(_) | Type::CStrLit | Type::StrSlice
                            )
                        })
                        .unwrap_or(false)
                } else if let Expr::Dot(_, m) = c.name.as_ref() {
                    // Plan 433 A1: `.str()` / `.to_string()` method calls return
                    // an owned String — a &str param still needs the borrow.
                    // Plan 019 Phase 1: `.slice(...)` also renders as an owned
                    // String (byte-range .to_string() or chars().collect()).
                    // Plan 514 W3: str 返回的用户方法(方法化 lib 的
                    // `int_val(p.text())` 族)渲染同为 owned String——经
                    // infer(Fix C 的 qualified 键)解析声明返回。
                    matches!(m.as_str(), "str" | "to_string" | "slice")
                        || matches!(arg,
                            Arg::Pos(e) if matches!(
                                self.infer_type_from_expr(e),
                                Type::StrOwned | Type::StrSlice
                                    | Type::StrFixed(_) | Type::CStrLit))
                } else {
                    false
                }
            } else {
                false
            };
            // Plan 433 A1: a struct-field read of a str field is an owned
            // String (t.kind) — borrow it for &str params.
            let arg_is_str_field = if let Arg::Pos(Expr::Dot(obj, field)) = arg {
                // Plan 433 A1: field reads of str fields — the owner's type is
                // often unavailable (chained/indexed objects), so accept the
                // field name being a str field of ANY known struct. Borrowed
                // loop-var fields are EXCLUDED — the defect-4 branch above
                // already emits them raw (&String coerces to &str directly).
                let is_borrowed_iter_field = matches!(obj.as_ref(),
                    Expr::Ident(n) if self.borrowed_iter_vars.contains(n));
                !is_borrowed_iter_field
                    && self.struct_field_types.values().any(|fields| {
                        fields.iter().any(|(f, t)| {
                            f.as_str() == field.as_str()
                                && matches!(
                                    t,
                                    Type::StrOwned
                                        | Type::StrSlice
                                        | Type::StrFixed(_)
                                        | Type::CStrLit
                                )
                        })
                    })
            } else {
                false
            };
            // Plan 032 第七批合并后复验发现的 plan 016 Phase 4 试点缺口（两处，
            // auto-ai 轨暴露，autodown-core 无此形态）：
            // (a) 裸 `x.trim()` 家族方法调用本来就产 `&str`——Phase 4 把 trim 加进
            //     expr_contains_string（concat 检测用）后，arg_is_concat 复用同一
            //     函数误borrow，`f(x.trim().as_str())` 触发 E0658（str_as_str）。
            //     排除 trim 家族与 to_str（产 &str 的方法）；to_string/to_lowercase
            //     等产 owned String 的不在此列。
            // (b) 未知被调者的 .as_str() 兜底不能叠在 .at 显式 `X.to_string()` 上
            //     ——作者已显式物化 owned String（如 fn-pointer 字段
            //     `handler: fn(String)`），兜底假定「&str 更常见」会反转意图
            //     （auto-ai agent driver.at `handler(step_id.to_string())`）。
            let arg_is_bare_refstr_method = if let Arg::Pos(Expr::Call(c)) = arg {
                matches!(c.name.as_ref(), Expr::Dot(_, m)
                    if matches!(m.as_str(),
                        "trim" | "trim_start" | "trim_end" | "trim_matches" | "to_str"))
            } else {
                false
            };
            let arg_is_explicit_to_string = if let Arg::Pos(Expr::Call(c)) = arg {
                matches!(c.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "to_string")
            } else {
                false
            };
            // Plan 447 ③-前置(②列基线修复): `xs.get(i)` on a List<str> —
            // the get→index conversion emits `xs[i as usize].clone()` (an
            // owned String); a &str param slot needs the same borrow as
            // plain str idents (E0308 otherwise). infer_type_from_expr's
            // get-arm resolves the element type.
            let arg_is_str_get = if let Arg::Pos(expr) = arg {
                matches!(expr, Expr::Call(c)
                    if c.args.args.len() == 1
                        && matches!(c.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "get"))
                    && matches!(
                        self.infer_type_from_expr(expr),
                        Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                    )
            } else {
                false
            };
            // Plan 019 Phase 1: `lines[i]` on a List<str> — the index
            // conversion emits an owned String (`.clone()`); a &str param
            // slot needs the same borrow as the get-arm above.
            let arg_is_str_index = if let Arg::Pos(expr) = arg {
                matches!(expr, Expr::Index(_, _))
                    && matches!(
                        self.infer_type_from_expr(expr),
                        Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                    )
            } else {
                false
            };
            // Plan 019 Phase 1: `x.field ?? fallback` yields an owned String
            // (the coalesce strips the Option) — a &str param slot needs the
            // borrow, same as the field read itself.
            let arg_is_str_coalesce = if let Arg::Pos(expr) = arg {
                matches!(expr, Expr::NullCoalesce(_, _))
                    && matches!(
                        self.infer_type_from_expr(expr),
                        Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                    )
            } else {
                false
            };
            if (needs_borrow || needs_borrow_unknown_callee)
                && !arg_is_str_slice
                && !arg_is_str_literal
                && !arg_is_bare_refstr_method
                && !(needs_borrow_unknown_callee && !needs_borrow && arg_is_explicit_to_string)
                && !str_stringified_here
                && (arg_is_ident
                    || arg_is_concat
                    || arg_is_str_returning_call
                    || arg_is_str_field
                    || arg_is_str_get
                    || arg_is_str_index
                    || arg_is_str_coalesce)
            {
                write!(out, ".as_str()")?;
            }

            // PLAN-681 T-05（N2/P5）：Value 读实参按形参型收口——实参为
            // Value 读（Value 局部字段读/bare List 索引读,P2 已发射
            // name["field"]/list[i] 索引形）而形参是标量/List 声明型时,
            // 按形参型追加降链。仅 bare fn（Ident callee,形参表在档）触发。
            if let Arg::Pos(vexpr) = arg {
                if self.expr_is_json_value_read(vexpr) {
                    if let Some(pt) = param_types.as_ref().and_then(|pts| pts.get(i)) {
                        match pt {
                            Type::StrSlice | Type::CStrLit | Type::StrFixed(_) => {
                                write!(out, ".as_str().unwrap_or_default()")?;
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
                }
            }

            // Plan 376 Pass 5: &str → String conversion when param expects owned
            // String (StrOwned/StrFixed). Auto-detect from param_types: if param
            // is StrOwned and arg is &str (StrSlice param or string literal), add .to_string().
            if !needs_borrow && !needs_borrow_unknown_callee {
                // Only when we didn't already handle it via borrow path
                if let Some(pts) = &param_types {
                    if let Some(pt) = pts.get(i) {
                        let param_is_owned_str = matches!(pt, Type::StrOwned | Type::StrFixed(_));
                        let arg_is_str_value = arg_is_str_slice
                            || arg_is_str_literal
                            || (if let Arg::Pos(Expr::Ident(name)) = arg {
                                self.local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false)
                            } else {
                                false
                            });
                        if param_is_owned_str && arg_is_str_value {
                            write!(out, ".to_string()")?;
                        }
                    }
                }
            }

            // Enum→i32 cast for int-expecting params
            if needs_enum_cast {
                write!(out, " as i64")?;
            }

            // Plan 376E: Broad type-aware argument conversion using local_var_types.
            // When neither the str-borrow path nor the str-to_string path fired,
            // check for other type mismatches between the arg's inferred type and
            // the param's declared type.
            if !needs_borrow && !needs_borrow_unknown_callee && !needs_enum_cast && !is_spec_param {
                if let (Some(pts), Arg::Pos(Expr::Ident(name))) = (&param_types, arg) {
                    if let Some(pt) = pts.get(i) {
                        let arg_ty = self.local_var_types.get(name);
                        if let Some(aty) = arg_ty {
                            // Option<T> → T: need .unwrap() or .cloned().unwrap_or_default()
                            if matches!(pt, Type::User(td) if td.name.as_str() != "Option")
                                && matches!(aty, Type::Option(_))
                                && !matches!(pt, Type::Option(_))
                            {
                                write!(out, ".unwrap()")?;
                            }
                            // u32 param, i32 arg: cast
                            else if matches!(pt, Type::Uint) && matches!(aty, Type::Int) {
                                write!(out, " as u32")?;
                            }
                            // i32 param, u32 arg: cast
                            else if matches!(pt, Type::Int) && matches!(aty, Type::Uint) {
                                write!(out, " as i64")?;
                            }
                            // usize param, i32/u32 arg: cast
                            else if matches!(pt, Type::USize)
                                && (matches!(aty, Type::Int) || matches!(aty, Type::Uint))
                            {
                                write!(out, " as usize")?;
                            }
                        }
                    }
                }
            }

            if is_spec_param {
                // Plan 390 §11 Phase E (D-C): the close paren of `Box::new(<arg>...)`.
                // A spec-bound ident (`Some(prof)` scrutinee — already `Box<dyn Trait>`)
                // must clone the box to stay usable; any other argument (a concrete
                // struct value like `EchoTool`, or a non-ident expression) is moved
                // into the box — matching the array-element pattern at ~9454 (pure
                // `Box::new(elem)`, no clone).
                if let Arg::Pos(Expr::Ident(name)) = arg {
                    if self.spec_bound_idents.contains(name) {
                        write!(out, ".clone())")?;
                    } else {
                        write!(out, ")")?;
                    }
                } else {
                    write!(out, ")")?;
                }
            }

            if i < call.args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, ")")?;

        // Plan 373: Auto-insert .await for calls to methods returning ~Result/Future.
        if let Expr::Dot(object, mname) = call.name.as_ref() {
            if self.call_needs_await(object, mname) {
                write!(out, ".await")?;
            }
        } else if let Expr::Ident(fname) = call.name.as_ref() {
            if let Some(ret) = self.fn_ret_types.get(fname.as_str()) {
                if Self::type_is_async(ret) {
                    write!(out, ".await")?;
                }
            }
        }

        Ok(())
    }

    /// Plan 610 ⑤: the pointer bridge kit — raw-buffer/out-param access from
    /// `#[export]` bodies is confined to this generated module (004 §3.5:
    /// unsafe lives in generated code only; Auto bodies stay safe). Emitted
    /// at most once per product, ahead of the first export wrapper that has
    /// any pointer in its face.
    pub(super) fn emit_cabi_kit(&mut self, sink: &mut Sink) -> AutoResult<()> {
        if self.cabi_kit_emitted {
            return Ok(());
        }
        self.cabi_kit_emitted = true;
        writeln!(
            sink.body,
            r#"/// Plan 610 ⑤: pointer bridge kit — raw-buffer/out-param access from #[export]
/// bodies is confined here (004 §3.5: unsafe lives in generated code only;
/// Auto bodies stay safe). All writes are null-tolerant no-ops.
mod auto_cabi_kit {{
    /// Null pointer of the statically expected pointee type (spawn-failure paths).
    pub fn cabi_null<T>() -> *mut T {{
        std::ptr::null_mut()
    }}

    /// Default shell name (COMSPEC → fallback "cmd"), mirroring engine ffi.rs.
    pub fn cabi_default_shell() -> String {{
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_owned())
    }}

    /// Copy UTF-8 bytes of `s` + NUL into `p` (≤cap). Returns total bytes incl.
    /// NUL (truncation mirrors the engine row_text contract); null p is a
    /// length-only probe.
    pub fn cabi_copy_cstr(p: *mut u8, cap: i64, s: &str) -> i64 {{
        if p.is_null() || cap <= 0 {{
            return (s.len() + 1) as i64;
        }}
        let bytes = s.as_bytes();
        let n = bytes.len().min(cap as usize - 1);
        unsafe {{
            let out = std::slice::from_raw_parts_mut(p, n + 1);
            out[..n].copy_from_slice(&bytes[..n]);
            out[n] = 0;
        }}
        (bytes.len() + 1) as i64
    }}

    /// Write one u32 element at `idx` (C `unsigned int`/`int` out-params — the
    /// engine face models non-negative int outs as u32, bit-compatible).
    pub fn cabi_write_u32(p: *mut u32, idx: i64, v: u32) {{
        if p.is_null() {{
            return;
        }}
        unsafe {{ *p.add(idx as usize) = v; }}
    }}

    /// Write one u8 element at `idx` (byte buffers).
    pub fn cabi_write_u8(p: *mut u8, idx: i64, v: u8) {{
        if p.is_null() {{
            return;
        }}
        unsafe {{ *p.add(idx as usize) = v; }}
    }}

    /// Copy `len` bytes from a raw const buffer into an owned Vec (host→child
    /// input path); null/empty buffer → empty Vec.
    pub fn cabi_bytes_vec(p: *const u8, len: i64) -> Vec<u8> {{
        if p.is_null() || len <= 0 {{
            return Vec::new();
        }}
        unsafe {{ std::slice::from_raw_parts(p, len as usize).to_vec() }}
    }}

    /// Reclaim a boxed value through its raw handle (free paths); null no-op.
    pub fn cabi_drop_boxed<T>(p: *mut T) {{
        if p.is_null() {{
            return;
        }}
        drop(unsafe {{ Box::from_raw(p) }});
    }}

    /// Bridge for `Option<(usize, usize)>` payloads (cursor): Some → write both
    /// u32 slots, return 1; None → 0. Auto has no tuple face, so foreign
    /// option-tuple values cross through this kit helper untouched.
    pub fn cabi_write_opt_tuple2_u32(cur: Option<(usize, usize)>, p0: *mut u32, p1: *mut u32) -> i64 {{
        match cur {{
            Some((a, b)) => {{
                cabi_write_u32(p0, 0, a as u32);
                cabi_write_u32(p1, 0, b as u32);
                1
            }}
            None => 0,
        }}
    }}
}}
pub use auto_cabi_kit::*;"#
        )?;
        Ok(())
    }

}
