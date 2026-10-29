//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_dot_module_methods(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                    ("http", "last_status") => {
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::http::last_status() as i64")?;
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                    // PLAN-724 T-06：普通动词族（两参 post 同型）——async 上下文
                    // 发射 *_async().await；同步上下文发射同步桥接面。
                    ("http", "get") | ("http", "put") | ("http", "delete") => {
                        self.a2r_std_used.set(true);
                        let Expr::Dot(_, method) = call.name.as_ref() else {
                            unreachable!("http verb dispatch requires Dot name");
                        };
                        emit_http_verb_call(self, out, method.as_str(), call)?;
                        return Ok(CallShape::Handled);
                    }
                    // PLAN-724 T-06：流生产者——async 上下文产出 AsyncHTTPStream
                    // （内部记录变量分型），同步上下文产出 HTTPStream。
                    ("http", "get_stream") | ("http", "post_stream") => {
                        self.a2r_std_used.set(true);
                        let Expr::Dot(_, method) = call.name.as_ref() else {
                            unreachable!("http stream dispatch requires Dot name");
                        };
                        emit_http_verb_call(self, out, method.as_str(), call)?;
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                    ("json", "encode") | ("Json", "encode") => {
                        // Keep generic serialization behind the declared JSON producer.
                        self.a2r_std_used.set(true);
                        write!(out, "a2r_std::json::encode(&")?;
                        if let Some(Arg::Pos(a)) = call.args.args.first() {
                            self.expr(a, out)?;
                        }
                        write!(out, ")")?;
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                    _ => {}
                }
            }
        }

        // Plan 223: Method call mappings for env.x / fs.x

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_bina_module_methods(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                                    return Ok(CallShape::Handled);
                                }
                                ("env", "args") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::env::args()")?;
                                    return Ok(CallShape::Handled);
                                }
                                ("io", "read_line") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::io::read_line()")?;
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
                                }
                                ("sqlite", "last_error") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::sqlite::last_error()")?;
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
                                }
                                ("File", "read_bytes") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::fs::read_bytes_list(")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr_as_str(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
                                }
                                ("fs", "delete") | ("File", "delete") => {
                                    write!(out, "File::delete(")?;
                                    if let Some(arg) = call.args.args.first() {
                                        self.arg(arg, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
                                }
                                ("http", "last_status") => {
                                    // http.last_status() → a2r_std::http::last_status()
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::http::last_status()")?;
                                    return Ok(CallShape::Handled);
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
                                ("json", "as_int") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::as_int(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(CallShape::Handled);
                                }
                                ("json", "as_string") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::as_string(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(CallShape::Handled);
                                }
                                ("json", "as_bool") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::as_bool(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
                                    return Ok(CallShape::Handled);
                                }
                                ("json", "len") => {
                                    self.a2r_std_used.set(true);
                                    write!(out, "a2r_std::json::len(&")?;
                                    if let Some(Arg::Pos(a)) = call.args.args.first() {
                                        self.expr(a, out)?;
                                    }
                                    write!(out, ")")?;
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "last_error" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::sqlite::last_error()")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "last_error" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::redis::last_error()")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "exists" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::exists(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "create_dir" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::create_dir(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "is_dir" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::is_dir(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "is_binary" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::is_binary(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "file_size" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::file_size(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "walk" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::walk(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "mkdir_all" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::mkdir_all(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "remove_file" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::remove_file(")?;
                                if let Some(arg) = call.args.args.first() {
                                    self.arg(arg, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "now" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time_now()")?;
                                return Ok(CallShape::Handled);
                            }
                            "now_secs" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time::now_sec().to_string()")?;
                                return Ok(CallShape::Handled);
                            }
                            "now_sec" => {
                                // Plan 376U: bare numeric now_sec() (i32) for
                                // timestamp math (no .to_string() — caller casts).
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time::now_sec()")?;
                                return Ok(CallShape::Handled);
                            }
                            "now_ms" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::time::now_ms()")?;
                                return Ok(CallShape::Handled);
                            }
                            _ => {}
                        },
                        // str module: str.uuid() → a2r_std::uuid(), str.from_uint(x) → x.to_string()
                        "str" => match method.as_str() {
                            "uuid" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::uuid()")?;
                                return Ok(CallShape::Handled);
                            }
                            "from_uint" | "from_int" => {
                                // str.from_uint(x) -> x.to_string()
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ".to_string()")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "as_string" => {
                                // Json.as_string(val) -> a2r_std::json::as_string(val)
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_string(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "as_int" => {
                                // Json.as_int(val) -> a2r_std::json::as_int(&val) as i32
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_int(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ") as i64")?;
                                return Ok(CallShape::Handled);
                            }
                            "is_null" => {
                                // Json.is_null(val) -> a2r_std::json::is_null(&val)
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::is_null(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "encode" => {
                                // Keep generic serialization behind the declared JSON producer.
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::encode(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "as_string" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_string(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "as_int" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_int(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ") as i64")?;
                                return Ok(CallShape::Handled);
                            }
                            "as_bool" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_bool(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "is_valid" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::is_valid(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "keys" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::keys(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "as_int" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::as_int(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ") as i64")?;
                                return Ok(CallShape::Handled);
                            }
                            "is_null" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::is_null(&")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
                            }
                            "type_of" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::json::value_type(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "last_status" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::http::last_status()")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "exists" => {
                                write!(out, "std::fs::metadata(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ").is_ok()")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                            "remove_dir" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::fs::remove_dir(")?;
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr_as_str(a, out)?;
                                }
                                write!(out, ")")?;
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                                return Ok(CallShape::Handled);
                            }
                        }
                        "str" => match method_name.as_str() {
                            "uuid" => {
                                self.a2r_std_used.set(true);
                                write!(out, "a2r_std::uuid()")?;
                                return Ok(CallShape::Handled);
                            }
                            "from_uint" | "from_int" => {
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ".to_string()")?;
                                return Ok(CallShape::Handled);
                            }
                            "to_uint" => {
                                if let Some(Arg::Pos(a)) = call.args.args.first() {
                                    self.expr(a, out)?;
                                }
                                write!(out, ".parse::<u64>().unwrap_or(0)")?;
                                return Ok(CallShape::Handled);
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

        Ok(CallShape::Miss)
    }
}
