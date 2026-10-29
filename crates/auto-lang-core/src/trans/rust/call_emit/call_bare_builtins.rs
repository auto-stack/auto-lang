//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_bare_diff_frame(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                return Ok(CallShape::Handled);
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

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_print_write_family(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
        // Special case for print / write function
        if let Expr::Ident(name) = call.name.as_ref() {
            if name == "print" {
                return self
                    .output_call(call, out, true)
                    .map(|_| CallShape::Handled);
            }
            if name == "write" {
                return self
                    .output_call(call, out, false)
                    .map(|_| CallShape::Handled);
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
                return Ok(CallShape::Handled);
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
                return Ok(CallShape::Handled);
            }
        }

        // Plan 310 Phase 0.2: Union construction `Union(field: val)` →
        // `Union::new_field(val)`. Rust requires `unsafe` to construct a union
        // via `Union { field: val }`; we route through the safe `new_<f>`
        // accessor generated in union_decl. Only the first named field is used
        // (union semantics: only one variant active at a time).

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_assert_family(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                return Ok(CallShape::Handled);
            }
        }

        // Plan 223: Function name mappings for external calls

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_plain_name_map(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
        // Plan 223: Function name mappings for external calls
        if let Expr::Ident(name) = call.name.as_ref() {
            if name == "not" {
                write!(out, "!(")?;
                if let Some(Arg::Pos(expr)) = call.args.args.first() {
                    self.expr(expr, out)?;
                }
                write!(out, ")")?;
                return Ok(CallShape::Handled);
            }
            match name.as_str() {
                "sleep_ms" => {
                    write!(out, "std::thread::sleep(std::time::Duration::from_millis(")?;
                    if let Some(arg) = call.args.args.first() {
                        self.arg(arg, out)?;
                    }
                    write!(out, " as u64))")?;
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
                }
                // PLAN-724 T-06：裸名流生产者（use http: get_stream 形态）。
                "get_stream" | "post_stream" | "post_stream_with_headers" => {
                    self.a2r_std_used.set(true);
                    emit_http_verb_call(self, out, name.as_str(), call)?;
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
                }
                "simple_hash" => {
                    self.a2r_std_used.set(true);
                    write!(out, "a2r_std::simple_hash(")?;
                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                        self.expr_as_str(a, out)?;
                    }
                    write!(out, ")")?;
                    return Ok(CallShape::Handled);
                }
                "time_now" => {
                    self.a2r_std_used.set(true);
                    write!(out, "a2r_std::time_now()")?;
                    return Ok(CallShape::Handled);
                }
                _ => {}
            }
        }

        // Handle Expr::Dot calls: http.post_sync(...), http.last_status(), env.get(...), etc.
        // Parser generates Expr::Dot(Ident("http"), "post_sync") for two-segment module calls.

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_min_max(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
                }
                _ => {}
            }
        }

        // Plan 264: Handle module.Type(args) constructor calls
        // e.g., types.ToolChatRequest(a, b, c) → crate::types::ToolChatRequest { field1: a, ... }

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_const_reference(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
        // Normal function call
        // In merge mode, if callee is a known const name with no args, emit bare const reference
        if call.args.args.is_empty() {
            if let Expr::Ident(fn_name) = call.name.as_ref() {
                if self.const_names.contains(fn_name) {
                    self.expr(&call.name, out)?;
                    return Ok(CallShape::Handled);
                }
            }
        }

        Ok(CallShape::Miss)
    }
}
