//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_dot_method_dispatch(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                return Ok(CallShape::Handled);
            }
            // Plan 162: Pointer intrinsic methods (only unique names that won't conflict)
            // ptr.is_null() -> ptr.is_null()
            // ptr.is_not_null() -> !ptr.is_null()
            match method_name.as_str() {
                "is_null" => {
                    self.expr(object, out)?;
                    write!(out, ".is_null()")?;
                    return Ok(CallShape::Handled);
                }
                "is_not_null" => {
                    write!(out, "(!")?;
                    self.expr(object, out)?;
                    write!(out, ".is_null())")?;
                    return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
                }
                "not" => {
                    // val.not() -> (!val), no arguments.
                    write!(out, "(!")?;
                    self.expr(object, out)?;
                    write!(out, ")")?;
                    return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
                }
                "slice" => {
                    // s.slice(n) -> s[n..].to_string()
                    // s.slice(start, end) -> s[start..end].to_string()
                    // Plan 019 Phase 1: string receivers slice by CHARS;
                    // non-string receivers keep the byte-index form.
                    if self.expr_is_string_like(object) {
                        return self
                            .emit_str_slice_chars(object, &call.args.args, out)
                            .map(|_| CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        "args" => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::args()")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("json", "to_string") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::to_string(&")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("json", "keys") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::keys(&")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("json", "is_valid") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::is_valid(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
                        }
                        ("json", "is_null") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::is_null(&")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
                        }
                        ("json", "type_of") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::json::value_type(&a2r_std::json::parse(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, "))")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("fs", "create_dir") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::create_dir(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("fs", "is_dir") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::is_dir(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
                        }
                        ("fs", "is_binary") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::is_binary(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
                        }
                        ("fs", "file_size") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::file_size(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
                        }
                        ("fs", "walk") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::fs::walk(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr_as_str(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("env", "get") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::env::get(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
                        }
                        ("io", "read_line") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::io::read_line()")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("sqlite", "last_error") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::sqlite::last_error()")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        ("redis", "last_error") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::redis::last_error()")?;
                            return Ok(CallShape::Handled);
                        }
                        ("Map", "new") => {
                            write!(out, "std::collections::HashMap::new()")?;
                            return Ok(CallShape::Handled);
                        }
                        // Plan 016 Phase A A4 cat 1: time builtin Dot-path dispatch
                        // (mirrors the Bina-path arms). Without this, `time.now_ms()`
                        // emits literally → E0423 "found module time".
                        ("time", "now_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::now_ms()")?;
                            return Ok(CallShape::Handled);
                        }
                        ("time", "sleep_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::sleep_ms(")?;
                            if let Some(Arg::Pos(a)) = call.args.args.first() {
                                self.expr(a, out)?;
                            }
                            write!(out, " as u64)")?;
                            return Ok(CallShape::Handled);
                        }
                        ("time", "now_sec") | ("time", "now_secs") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::now_sec()")?;
                            return Ok(CallShape::Handled);
                        }
                        ("time", "now") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::time::now()")?;
                            return Ok(CallShape::Handled);
                        }
                        // PLAN-716 Phase 2 (供⑨-b): frame 二段名 Dot-path 臂
                        // （time A4 先例同款——缺失时原文发射 → E0425 cannot
                        // find value `frame`，下游 plan-022 实录；裸名姊妹臂
                        // 在 call() 的 Expr::Ident 映射表）。
                        ("frame", "begin_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::frame::begin_ms()")?;
                            return Ok(CallShape::Handled);
                        }
                        ("frame", "present_ms") => {
                            self.a2r_std_used.set(true);
                            write!(out, "a2r_std::frame::present_ms()")?;
                            return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
                }
                "insert_str" => {
                    self.expr(object, out)?;
                    write!(out, ".insert(")?;
                    self.arg(&call.args.args[0], out)?;
                    write!(out, ".to_string(), (")?;
                    self.arg(&call.args.args[1], out)?;
                    write!(out, ").to_string())")?;
                    return Ok(CallShape::Handled);
                }
                "get_str" => {
                    write!(out, "(")?;
                    self.expr(object, out)?;
                    write!(out, ".get(&")?;
                    self.arg(&call.args.args[0], out)?;
                    write!(out, ".to_string()).cloned().unwrap_or_default())")?;
                    return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                }
            }

            // Plan 433 A1: struct `len` field read (Tok.len) — property,
            // not the collection-length method.
            if self.try_emit_struct_len_field(object, method_name, call.args.args.len(), out)? {
                return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                    if matches!(method_name.as_str(), "nil" | "none") {
                        write!(out, "None")?;
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                    return Ok(CallShape::Handled);
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
            return Ok(CallShape::Handled);
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
                return Ok(CallShape::Handled);
            }
        }

        // Check if this is a struct construction call: Type(args)
        // Heuristic: If the callee name starts with uppercase, treat as type construction
        // This works because Rust convention: TypeNames are CamelCase, functions are snake_case
        // Exception: SCREAMING_CASE names (OP_XXX, BOOL_XXX) are constants/functions, not types

        Ok(CallShape::Miss)
    }
}
