//! PLAN-843 结构拆分（自 codegen.rs 原样搬移；算法/行为零变化）。
use super::*;

impl Codegen {
    /// Plan 203 Phase 2: Process use statement and populate import scope
    ///
    /// Handles `use module.path: name1, name2` style imports by mapping
    /// each imported name to its fully qualified path in `import_scope`.
    ///
    /// Plan 339 Phase 6: resolve a call's bare name to the symbol name the
    /// linker will bind. Order:
    ///   1. `import_scope` alias (from `use back.api: create_note`)
    ///   2. exact export key (already qualified, e.g. `db.all_notes`)
    ///   3. unique bare-name → module-qualified fallback: if exactly one export
    ///      ends with `.name`, use it. Ambiguous names stay bare so the
    ///      explicit `use` alias remains authoritative.
    /// Returns the input unchanged if no better resolution is found.
    pub(super) fn resolve_call_symbol(&self, name: &str) -> String {
        // 1. import_scope alias. Don't require the export to exist yet — this
        //    is the relocation SYMBOL (the name the linker will later bind),
        //    and forward references (e.g. day_style declared below its caller)
        //    are exactly the case where the export isn't populated yet.
        if let Some(qualified) = self.import_scope.get(name) {
            return qualified.clone();
        }
        // 2. exact export key.
        if self.exports.contains_key(name) {
            return name.to_string();
        }
        // 2.5 PLAN-019 T-06 (vm delegate): own-module binding for bare calls
        // inside module-qualified fns. Flattened synthesis renames every
        // imported fn to `mod.fn`, so a bare intra-module call (`mux_tab_id_at`
        // inside db.mux_tab_title_at) is ambiguous at module scope (api.X and
        // db.X both define the bare name) and step 3 leaves it bare — a bare
        // CALL reloc that can never bind against dotted-only entry-module
        // exports ("Undefined symbol" at link). Bind to the current fn's own
        // module first, mirroring the loader's Plan 322/545 own-module
        // semantics at codegen level. fn_return_types covers forward refs
        // (pre-registered for every flattened import before bodies compile).
        if !name.contains('.') {
            if let Some(mod_name) = &self.current_fn_module {
                let qualified = format!("{}.{}", mod_name, name);
                if self.exports.contains_key(&qualified)
                    || self.fn_return_types.contains_key(&qualified)
                {
                    return qualified;
                }
            }
        }
        // 3. unique bare-name → module-qualified fallback.
        if !name.contains('.') {
            let mut hits = self
                .exports
                .keys()
                .filter(|k| k.split('.').last() == Some(name));
            if let Some(first) = hits.next() {
                if hits.next().is_none() {
                    return first.clone();
                }
            }
        }
        name.to_string()
    }

    /// PLAN-093 G-7 诊断（AUTO_DEBUG_G7 门控）：ext 方法体内裸调用解析
    /// 观测——import_scope 内容与最终解析结果。
    pub(super) fn debug_resolve_call_symbol(&self, name: &str, resolved: &str) {
        if std::env::var_os("AUTO_DEBUG_G7").is_some() && !name.contains('.') {
            let scope: Vec<&String> = self.import_scope.keys().take(12).collect();
            eprintln!(
                "[G7] call `{}` -> `{}` (module={:?} fn_mod={:?} scope={:?})",
                name, resolved, self.current_module, self.current_fn_module, scope
            );
        }
    }

    /// Plan 340: Rewrite a bare API function call into an HTTP request.
    ///
    /// Emits bytecode equivalent to:
    ///   GET    /api/notes                   → http.get_json(url)    → json.to_value
    ///   POST   /api/notes, body             → http.post_json(url,body) → json.to_value
    ///   PUT    /api/notes/:id, body         → http.put_json(url,body)  → json.to_value
    ///   DELETE /api/notes/:id               → http.delete_json(url)  → json.to_value
    ///
    /// Path params (`:id`) are spliced from the matching positional argument
    /// (stringified via str concatenation). Remaining args form the JSON body
    /// via `json.from_value`.
    /// Plan 060 M3:裸 #[api] 调用 → auto.host.call(name, args_json) 直调
    /// 宿主桥(ash-runner 注册),响应 JSON 经 auto.json.to_value 落栈。
    /// 参数序列化与 HTTP body 同构;返回类型映射与 HTTP 版一致。
    pub(super) fn emit_api_host_call(
        &mut self,
        api: &ApiCallInfo,
        call: &crate::ast::Call,
    ) -> AutoResult<()> {
        let arg_exprs: Vec<Expr> = call.args.args.iter().map(|a| a.get_expr()).collect();

        // 1. 桥函数名(= api.at 契约函数裸名;ash-runner 按端点名注册。
        //    ApiCallInfo.fn_name 可能带模块限定前缀,取最后一段)。
        let bare = api
            .fn_name
            .rsplit('.')
            .next()
            .unwrap_or(&api.fn_name)
            .to_string();
        self.emit_str_const_push(&bare);

        // 2. 参数 JSON 对象:{"p1":<v1>,...}(json.from_value 逐个序列化)。
        //    零参数直接推常量 "{}" —— 不经 STR_CAT(与 HTTP GET 路径同构,
        //    规避疑似 STR_CAT 栈协议问题)。
        if arg_exprs.is_empty() {
            self.emit_str_const_push("{}");
        } else {
            self.emit_str_const_push("{");
            for (i, expr) in arg_exprs.iter().enumerate() {
                if i > 0 {
                    self.emit_str_const_push(",");
                    self.emit(OpCode::STR_CAT);
                }
                let pname = api
                    .params
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("arg{}", i));
                self.emit_str_const_push(&format!("\"{}\":", pname));
                self.emit(OpCode::STR_CAT);
                self.compile_expr(expr)?;
                self.emit_call_nat_by_name("auto.json.from_value", 1)?;
                self.emit(OpCode::STR_CAT);
            }
            self.emit_str_const_push("}");
            self.emit(OpCode::STR_CAT);
        }

        // 3. 直调宿主桥:name, args_json → 响应 JSON 串。
        self.emit_call_nat_by_name("auto.host.call", 2)?;
        // 4. json.to_value(响应) → VM 值(调用点直接消费,不过 .at fn return)。
        self.emit_call_nat_by_name("auto.json.to_value", 1)?;

        // 5. 返回类型映射(与 HTTP 版同表)。
        self.last_expr_type = match &api.ret_type {
            crate::ast::Type::Array(_) | crate::ast::Type::List(_) => ObjectType::NestedObject,
            crate::ast::Type::StrFixed(_)
            | crate::ast::Type::StrOwned
            | crate::ast::Type::CStrLit
            | crate::ast::Type::StrSlice => ObjectType::String,
            crate::ast::Type::Int | crate::ast::Type::I64 => ObjectType::Int,
            crate::ast::Type::Uint | crate::ast::Type::U64 | crate::ast::Type::USize => {
                ObjectType::Int
            }
            crate::ast::Type::Bool => ObjectType::Bool,
            crate::ast::Type::Float => ObjectType::Float,
            crate::ast::Type::Double => ObjectType::Double,
            _ => ObjectType::NestedObject,
        };
        Ok(())
    }

    pub(super) fn emit_api_http_call(
        &mut self,
        api: &ApiCallInfo,
        call: &crate::ast::Call,
    ) -> AutoResult<()> {
        // Collect positional arg expressions in order.
        let arg_exprs: Vec<Expr> = call.args.args.iter().map(|a| a.get_expr()).collect();

        // 1. Build the URL. Start from the path template; for each `:param`
        //    segment, splice the corresponding positional arg (stringified).
        //    The arg consumed by a path param is NOT included in the body.
        //    Plan 340 fix: prepend the backend base URL (host:port from
        //    AUTO_HTTP_PORT, default 8080) — api.path is just "/api/notes".
        // Plan 057 (ash-gui): AUTO_BACKEND (e.g. http://127.0.0.1:3000) takes
        // precedence as the base URL — the VM app talks to a real backend
        // (ash-server) rather than the split-mode scaffold server.
        let base_url = std::env::var("AUTO_BACKEND")
            .ok()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                let port = std::env::var("AUTO_HTTP_PORT").unwrap_or_else(|_| "8080".to_string());
                format!("http://127.0.0.1:{}", port)
            });
        let mut path = format!("{}{}", base_url, api.path);
        let mut body_arg_indices: Vec<usize> = Vec::new();
        for (ai, expr) in arg_exprs.iter().enumerate() {
            // Only positional args map to path params by position. Three
            // placeholder spellings: `{param}` (musk/vue convention, e.g.
            // "/api/chats/session/{id}"), `{*param}` (axum wildcard, e.g.
            // "/api/wiki/{*path}" — plan-022), and `:param` (legacy). Brace
            // wins when both would match.
            let param_name = api.params.get(ai).cloned().unwrap_or_default();
            let brace = format!("{{{}}}", param_name);
            let wild = format!("{{*{}}}", param_name);
            let colon = format!(":{}", param_name);
            // Single segments get full percent-encoding (axum Path decodes
            // on the wire — identity for unreserved chars, so existing
            // plain-id URLs are unchanged); wildcards keep `/` separators
            // (server-side per-segment decode reconstructs the path).
            let (placeholder, encode_native) = if !param_name.is_empty() && path.contains(&brace) {
                (brace, Some("auto.url.encode"))
            } else if !param_name.is_empty() && path.contains(&wild) {
                (wild, Some("auto.url.encode_path"))
            } else if path.contains(&colon) {
                (colon, Some("auto.url.encode"))
            } else {
                body_arg_indices.push(ai);
                continue;
            };
            // Splice the arg into the path at the placeholder: emit a
            // LOAD_STR + arg + STR_CAT chain via emit_url_with_param.
            self.emit_url_with_param(&path, &placeholder, expr, encode_native)?;
            path = String::new(); // marker: URL already emitted
        }
        // If no path-param substitution happened (path unchanged), emit the
        // literal URL now.
        if !path.is_empty() {
            self.emit_str_const_push(&path);
        }

        // 2. Determine the HTTP native name and whether a body is needed.
        let method = api.method.to_uppercase();
        let needs_body = matches!(method.as_str(), "POST" | "PUT" | "PATCH");

        // 2.5 (plan-022): bodyless methods carry their non-path args as a
        // query string. Previously the args were collected for the JSON
        // body and then silently dropped here, leaving `?q=…&limit=…`
        // style endpoints unreachable from the VM front. Values are
        // percent-encoded (identity for int/bool stringifications).
        if !needs_body && !body_arg_indices.is_empty() {
            for (qi, &ai) in body_arg_indices.iter().enumerate() {
                let pname = api
                    .params
                    .get(ai)
                    .cloned()
                    .unwrap_or_else(|| format!("arg{}", ai));
                let sep = if qi == 0 { "?" } else { "&" };
                self.emit_str_const_push(&format!("{}{}=", sep, pname));
                self.emit(OpCode::STR_CAT);
                self.compile_expr(&arg_exprs[ai])?;
                self.emit(OpCode::TO_STR);
                self.emit_call_nat_by_name("auto.url.encode", 1)?;
                self.emit(OpCode::STR_CAT);
            }
        }

        let http_native = match method.as_str() {
            "GET" => "auto.http.get_json",
            "POST" => "auto.http.post_json",
            "PUT" => "auto.http.put_json",
            "DELETE" => "auto.http.delete_json",
            "PATCH" => "auto.http.patch_json",
            other => {
                return Err(format!(
                    "emit_api_http_call: unsupported method '{}' for {}",
                    other, api.path
                )
                .into());
            }
        };

        // 3. For POST/PUT, build the JSON body from the non-path args.
        if needs_body {
            // Build a JSON object string: {"p1":<v1>,"p2":<v2>,...}
            // Emit: for each param, key literal + json.from_value(arg) + concat.
            // Start with "{" then append "\"p1\":" + from_value(v1) + "," ...
            self.emit_str_const_push("{");
            let body_params: Vec<(String, &Expr)> = body_arg_indices
                .iter()
                .map(|&i| {
                    (
                        api.params
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| format!("arg{}", i)),
                        &arg_exprs[i],
                    )
                })
                .collect();
            for (i, (pname, expr)) in body_params.iter().enumerate() {
                if i > 0 {
                    self.emit_str_const_push(",");
                    self.emit(OpCode::STR_CAT);
                }
                // key: "\"name\":"
                self.emit_str_const_push(&format!("\"{}\":", pname));
                self.emit(OpCode::STR_CAT);
                // value: json.from_value(arg) → string
                self.compile_expr(expr)?;
                self.emit_call_nat_by_name("auto.json.from_value", 1)?;
                self.emit(OpCode::STR_CAT);
            }
            self.emit_str_const_push("}");
            self.emit(OpCode::STR_CAT);
        }

        // 4. Call the HTTP native. Stack (POST/PUT): [url, body]
        //    (GET/DELETE): [url]. Native pops body then url.
        let arg_count: u16 = if needs_body { 2 } else { 1 };
        self.emit_call_nat_by_name(http_native, arg_count)?;
        // Stack: [response_body_string]

        // 5. json.to_value(response_body) → VM value.
        self.emit_call_nat_by_name("auto.json.to_value", 1)?;

        // 6. Set last_expr_type from the API fn's return type.
        //
        // The value pushed by `auto.json.to_value` is a SINGLE 1-slot NanoValue
        // for ints/uints/bools/strings/objects (json_to_vm_value uses push_nv),
        // and a 2-slot f64 pair for doubles. So we map by the ACTUAL stack
        // layout, not the declared type: uint/U64/USize JSON integers land as
        // encode_i32 (1 slot), so they must be ObjectType::Int here — using
        // ObjectType::Uint would make Stmt::Expr / Stmt::Store emit POP_N(2)
        // / two STORE_LOCs and corrupt the stack (only 1 slot was pushed).
        // Double stays Double because json_to_vm_value pushes a 2-slot f64.
        self.last_expr_type = match &api.ret_type {
            crate::ast::Type::Array(_) | crate::ast::Type::List(_) => ObjectType::NestedObject,
            crate::ast::Type::StrFixed(_)
            | crate::ast::Type::StrOwned
            | crate::ast::Type::CStrLit
            | crate::ast::Type::StrSlice => ObjectType::String,
            crate::ast::Type::Int | crate::ast::Type::I64 => ObjectType::Int,
            crate::ast::Type::Uint | crate::ast::Type::U64 | crate::ast::Type::USize => {
                ObjectType::Int
            }
            crate::ast::Type::Bool => ObjectType::Bool,
            crate::ast::Type::Float => ObjectType::Float,
            crate::ast::Type::Double => ObjectType::Double,
            _ => ObjectType::NestedObject, // struct / Option / unknown
        };

        // NOTE: do NOT emit a POP here. This function is invoked from
        // `compile_expr` (Expr::Call branch), so the result must stay on the
        // stack for the consumer — a Stmt::Store RHS reads it via STORE_LOC,
        // and a bare statement-expr is popped by `Stmt::Expr` (compile_stmt,
        // the `should_pop_expr_result` / `last_was_native_void` check). Emitting
        // POP here too double-consumes the value: when this call is a store
        // RHS the inherited `should_pop_expr_result` is often true (set for a
        // sibling statement), so the POP would discard the return value and
        // leave STORE_LOC reading garbage, corrupting the stack frame (bp).
        Ok(())
    }

    /// Emit a CALL_NAT by resolved native name with the given arg count.
    pub(super) fn emit_call_nat_by_name(&mut self, name: &str, _arg_count: u16) -> AutoResult<()> {
        let native_id = BIGVM_NATIVES
            .lock()
            .unwrap()
            .resolve_qualified(name)
            .ok_or_else(|| format!("emit_call_nat_by_name: native '{}' not registered", name))?;
        self.emit(OpCode::CALL_NAT);
        self.emit_u16(native_id);
        Ok(())
    }

    /// Push a string literal onto the stack (intern in the strings pool).
    pub(super) fn emit_str_const_push(&mut self, s: &str) {
        let idx = self.add_string(s);
        self.emit(OpCode::LOAD_STR);
        self.code.extend_from_slice(&idx.to_le_bytes());
    }

    /// Emit URL construction with a path-param substitution.
    /// Produces a single string on the stack: the URL with `:param` replaced
    /// by the runtime value of `arg_expr` (stringified).
    fn emit_url_with_param(
        &mut self,
        path: &str,
        placeholder: &str,
        arg_expr: &Expr,
        encode_native: Option<&str>,
    ) -> AutoResult<()> {
        // Split path at the placeholder.
        let (before, after) = path.split_once(placeholder).unwrap_or((path, ""));
        self.emit_str_const_push(before);
        // Stringify the arg: compile it, then to_string via str conversion.
        // For int args, use I32_TO_STR; for others, assume string-compatible.
        self.compile_expr(arg_expr)?;
        // Stringify the arg (TO_STR handles int/bool/str/f32; object args would
        // produce a debug string, so path params should be int/str — the common
        // case for :id segments).
        self.emit(OpCode::TO_STR);
        // Optional percent-encoding pass (plan-022): full encode for single
        // segments, slash-preserving encode for `{*path}` wildcards (the
        // native no-ops on unreserved chars, so plain-id URLs are unchanged).
        if let Some(native) = encode_native {
            self.emit_call_nat_by_name(native, 1)?;
        }
        self.emit(OpCode::STR_CAT); // before + arg
        if !after.is_empty() {
            self.emit_str_const_push(after);
            self.emit(OpCode::STR_CAT); // (before+arg) + after
        }
        Ok(())
    }

    /// Plan 539 (T19): unified dispatch for py special call forms. Returns
    /// true when the call was fully compiled (caller returns immediately).
    pub(super) fn try_py_call_special_form(&mut self, call: &crate::ast::Call) -> AutoResult<bool> {
        // py_with inline bracket (T14)
        if self.is_py_with_inline_form(call) {
            self.compile_py_with_inline(call)?;
            return Ok(true);
        }
        // item-import kwargs (T17)
        if self.is_py_item_kw_form(call) {
            self.compile_py_item_kw_form(call)?;
            return Ok(true);
        }
        // py_call with named args (T03)
        if self.is_py_call_kw_form(call) {
            self.compile_py_call_kw_form(call)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Plan 539 W2 (T17): does this call site match an item-import direct
    /// call carrying named args (`Linear(784, 10, bias: false)`)? The callee
    /// must NOT be one of the py_* builtins (those have their own kw forms).
    fn is_py_item_kw_form(&self, call: &crate::ast::Call) -> bool {
        match call.name.as_ref() {
            Expr::Ident(n) => {
                let name = n.as_str();
                if name.starts_with("py_") {
                    return false;
                }
                match self.py_native_map.get(name) {
                    Some((module_path, _full)) => {
                        // item imports carry a real module path; the fixed
                        // builtins use the __pyobj__ placeholder.
                        module_path != "__pyobj__"
                            && call.args.args.len() >= 1
                            && call
                                .args
                                .args
                                .iter()
                                .any(|a| matches!(a, crate::ast::Arg::Pair(..)))
                    }
                    None => false,
                }
            }
            _ => false,
        }
    }

    /// Plan 539 W2 (T17): emit the py_item_kw 5-slot convention —
    /// (module_name, func_name, posargs, kw_names, kw_vals).
    fn compile_py_item_kw_form(&mut self, call: &crate::ast::Call) -> AutoResult<()> {
        const NATIVE_PY_ITEM_KW: u16 = 464;
        let name = match call.name.as_ref() {
            Expr::Ident(n) => n.to_string(),
            _ => return Ok(()),
        };
        let module_path = self
            .py_native_map
            .get(&name)
            .map(|(m, _)| m.clone())
            .unwrap_or_default();

        // module name + func name as pool constants
        for text in [&module_path, &name] {
            let bytes = text.as_bytes().to_vec();
            let idx = self.strings.len() as u32;
            self.strings.push(bytes);
            self.emit(OpCode::LOAD_STR);
            self.code.extend_from_slice(&idx.to_le_bytes());
        }
        // positional args -> list
        let pos_count = call
            .args
            .args
            .iter()
            .filter(|a| matches!(a, crate::ast::Arg::Pos(_)))
            .count();
        for arg in call.args.args.iter() {
            if let crate::ast::Arg::Pos(e) = arg {
                self.compile_expr(e)?;
            }
        }
        self.emit(OpCode::CREATE_ARRAY);
        self.code.push(pos_count as u8);
        // kw names -> list of str
        let kw_names: Vec<String> = call
            .args
            .args
            .iter()
            .filter_map(|a| match a {
                crate::ast::Arg::Pair(k, _) => Some(k.to_string()),
                _ => None,
            })
            .collect();
        for kn in &kw_names {
            let bytes = kn.as_bytes().to_vec();
            let idx = self.strings.len() as u32;
            self.strings.push(bytes);
            self.emit(OpCode::LOAD_STR);
            self.code.extend_from_slice(&idx.to_le_bytes());
        }
        self.emit(OpCode::CREATE_ARRAY);
        self.code.push(kw_names.len() as u8);
        // kw values -> list (source order)
        for arg in call.args.args.iter() {
            if let crate::ast::Arg::Pair(_, e) = arg {
                self.compile_expr(e)?;
            }
        }
        self.emit(OpCode::CREATE_ARRAY);
        self.code.push(kw_names.len() as u8);
        // CALL_NAT_COUNTED py_item_kw with count = 5
        self.emit(OpCode::CALL_NAT_COUNTED);
        self.emit_u16(NATIVE_PY_ITEM_KW);
        self.code.push(5u8);
        Ok(())
    }

    /// Plan 539 W1 (T14): does this call site match `py_with(ctx, () => { body })`?
    fn is_py_with_inline_form(&self, call: &crate::ast::Call) -> bool {
        matches!(call.name.as_ref(), Expr::Ident(n) if n.as_ref() == "py_with")
            && call.args.args.len() == 2
            && matches!(call.args.args.first(), Some(crate::ast::Arg::Pos(_)))
            && matches!(
                call.args.args.get(1),
                Some(crate::ast::Arg::Pos(Expr::Closure(cl))) if cl.params.is_empty()
            )
    }

    /// Plan 539 W1 (T14): emit the INLINE with-bracket —
    /// py_enter(ctx); <body statements>; py_exit(ctx). Py handles in closure
    /// locals degrade to raw ids (DIV-PY-CLOSURE-1, pre-existing), so the
    /// body runs in the enclosing scope with normal local semantics.
    fn compile_py_with_inline(&mut self, call: &crate::ast::Call) -> AutoResult<()> {
        const NATIVE_PY_ENTER: u16 = 462;
        const NATIVE_PY_EXIT: u16 = 463;
        // temp local holding the ctx handle across the body
        self.push_scope();
        let ctx_idx = self.add_var("_with_ctx");
        if let Some(crate::ast::Arg::Pos(e)) = call.args.args.first() {
            self.compile_expr(e)?;
        }
        self.emit_store_loc(ctx_idx);
        // py_enter(ctx)
        self.emit_load_loc(ctx_idx);
        self.emit(OpCode::CALL_NAT_COUNTED);
        self.emit_u16(NATIVE_PY_ENTER);
        self.code.push(1u8);
        // body statements inline (discard inner expression results)
        if let Some(crate::ast::Arg::Pos(Expr::Closure(cl))) = call.args.args.get(1) {
            if let Expr::Block(block) = cl.body.as_ref() {
                let old_pop = self.should_pop_expr_result;
                self.should_pop_expr_result = true;
                for st in &block.stmts {
                    self.compile_stmt(st)?;
                }
                self.should_pop_expr_result = old_pop;
            } else {
                self.compile_expr(&cl.body)?;
                self.emit(OpCode::POP);
            }
        }
        // py_exit(ctx)
        self.emit_load_loc(ctx_idx);
        self.emit(OpCode::CALL_NAT_COUNTED);
        self.emit_u16(NATIVE_PY_EXIT);
        self.code.push(1u8);
        self.pop_scope();
        self.last_expr_type = ObjectType::Void;
        self.last_was_native_void = true;
        Ok(())
    }

    /// Plan 539 W0 (DIV-PY-KWARGS-1): does this call site match
    /// `py_call(obj, "m", pos..., k: v...)`? Requires the callee to be the
    /// py_call builtin, >=3 args, obj/method positional, and at least one
    /// named arg in the tail.
    fn is_py_call_kw_form(&self, call: &crate::ast::Call) -> bool {
        // Plan 567 T09: py_call_may + Pair 实参同走 5 槽 kwargs 约定
        // （native id 由 compile_py_call_kw_form 按 may 名路由 478）。
        matches!(call.name.as_ref(), Expr::Ident(n)
            if n.as_ref() == "py_call" || n.as_ref() == "py_call_may")
            && call.args.args.len() >= 3
            && matches!(call.args.args.first(), Some(crate::ast::Arg::Pos(_)))
            && matches!(call.args.args.get(1), Some(crate::ast::Arg::Pos(_)))
            && call.args.args[2..]
                .iter()
                .any(|a| matches!(a, crate::ast::Arg::Pair(..)))
    }

    /// Plan 539 W0 (DIV-PY-KWARGS-1): emit the py_call_kw 5-slot convention
    /// (obj, method, posargs_list, kw_names_list, kw_vals_list) for a matched
    /// call site. Kept out of the native-call arm to bound its stack frame.
    fn compile_py_call_kw_form(&mut self, call: &crate::ast::Call) -> AutoResult<()> {
        // obj + method (slots 1-2 of the convention)
        if let Some(crate::ast::Arg::Pos(e)) = call.args.args.first() {
            self.compile_expr(e)?;
        }
        if let Some(crate::ast::Arg::Pos(e)) = call.args.args.get(1) {
            self.compile_expr(e)?;
        }
        // positional tail -> Auto list
        let pos_count = call.args.args[2..]
            .iter()
            .filter(|a| matches!(a, crate::ast::Arg::Pos(_)))
            .count();
        for arg in call.args.args[2..].iter() {
            if let crate::ast::Arg::Pos(e) = arg {
                self.compile_expr(e)?;
            }
        }
        self.emit(OpCode::CREATE_ARRAY);
        self.code.push(pos_count as u8);
        // kw names -> list of str (pool constants)
        let kw_names: Vec<String> = call.args.args[2..]
            .iter()
            .filter_map(|a| match a {
                crate::ast::Arg::Pair(k, _) => Some(k.to_string()),
                _ => None,
            })
            .collect();
        for name in &kw_names {
            let name_bytes = name.as_bytes().to_vec();
            let name_idx = self.strings.len() as u32;
            self.strings.push(name_bytes);
            self.emit(OpCode::LOAD_STR);
            self.code.extend_from_slice(&name_idx.to_le_bytes());
        }
        self.emit(OpCode::CREATE_ARRAY);
        self.code.push(kw_names.len() as u8);
        // kw values -> Auto list (source order, matching names)
        for arg in call.args.args[2..].iter() {
            if let crate::ast::Arg::Pair(_, e) = arg {
                self.compile_expr(e)?;
            }
        }
        self.emit(OpCode::CREATE_ARRAY);
        self.code.push(kw_names.len() as u8);
        // The fixed 5-slot convention of py_call_kw (id 452); the may variant
        // (Plan 567 T09) routes py_call_kw_may (id 478) — same ABI, Err value out.
        let is_may = matches!(call.name.as_ref(), Expr::Ident(n) if n.as_ref() == "py_call_may");
        self.emit(OpCode::CALL_NAT_COUNTED);
        self.emit_u16(if is_may { 478 } else { 452 });
        self.code.push(5u8);
        Ok(())
    }

    /// Plan 369 Task 12: register py_call / py_getattr in py_native_map so the
    /// codegen treats them as py-FFI calls (emitting CALL_NAT_COUNTED with runtime arg
    /// count). Idempotent.
    pub(super) fn register_py_object_builtins(&mut self) {
        // Use fixed IDs directly to avoid importing py_ffi (which is feature-gated).
        // NATIVE_PY_CALL = 450, NATIVE_PY_GETATTR = 451, NATIVE_PY_CALL_KW = 452
        // (defined in py_ffi.rs).
        const NATIVE_PY_CALL: u16 = 450;
        const NATIVE_PY_GETATTR: u16 = 451;
        const NATIVE_PY_CALL_KW: u16 = 452;
        const NATIVE_PY_CALL_MAY: u16 = 453;
        const NATIVE_PY_ITER: u16 = 454;
        const NATIVE_PY_NEXT: u16 = 455;
        const NATIVE_PY_MATMUL: u16 = 456;
        const NATIVE_PY_GETITEM: u16 = 457;
        const NATIVE_PY_SETITEM: u16 = 458;
        const NATIVE_PY_SLICE: u16 = 459;
        const NATIVE_PY_CALL0: u16 = 460;
        const NATIVE_PY_WITH: u16 = 461;
        const NATIVE_PY_ENTER: u16 = 462;
        const NATIVE_PY_EXIT: u16 = 463;
        const NATIVE_PY_ITEM_KW: u16 = 464;
        // Plan 567 T06 (P560-D2): may 值通道变体。
        const NATIVE_PY_GETATTR_MAY: u16 = 476;
        const NATIVE_PY_GETITEM_MAY: u16 = 477;
        // Plan 567 T07 (P539-D5): kwargs×may 组合。
        const NATIVE_PY_CALL_KW_MAY: u16 = 478;
        // Plan 567 T12 (P560-D1): with-as catch 臂再抛通道。
        const NATIVE_PY_RAISE: u16 = 479;
        // Plan 567 T18 (W3 D4): GIL int() 显式标量提取。
        const NATIVE_PY_INT: u16 = 480;
        // Plan 602 (D2): py_subclass 类派生工厂。
        const NATIVE_PY_SUBCLASS: u16 = 481;
        // Insert placeholder entries; the (module, full_path) tuple is unused for
        // dispatch since the native IDs are fixed constants. The qualified lookup
        // below uses the entry's existence to set is_py_ffi_call = true.
        if !self.py_native_map.contains_key("py_call") {
            self.py_native_map.insert(
                "py_call".to_string(),
                ("__pyobj__".to_string(), "__pyobj__.py_call".to_string()),
            );
            self.py_return_types
                .insert("py_call".to_string(), crate::py_ffi_types::PyType::Auto);
            // Pin the fixed native ID so resolve_qualified() finds it without
            // needing a runtime import.
            if let Ok(mut reg) = BIGVM_NATIVES.lock() {
                reg.register_with_id("py.py_call", NATIVE_PY_CALL);
                reg.register_with_id("py.py_getattr", NATIVE_PY_GETATTR);
                reg.register_with_id("py.py_call_kw", NATIVE_PY_CALL_KW);
                reg.register_with_id("py.py_call_may", NATIVE_PY_CALL_MAY);
                reg.register_with_id("py.py_iter", NATIVE_PY_ITER);
                reg.register_with_id("py.py_next", NATIVE_PY_NEXT);
                reg.register_with_id("py.py_matmul", NATIVE_PY_MATMUL);
                reg.register_with_id("py.py_getitem", NATIVE_PY_GETITEM);
                reg.register_with_id("py.py_setitem", NATIVE_PY_SETITEM);
                reg.register_with_id("py.py_slice", NATIVE_PY_SLICE);
                reg.register_with_id("py.py_call0", NATIVE_PY_CALL0);
                reg.register_with_id("py.py_with", NATIVE_PY_WITH);
                reg.register_with_id("py.py_enter", NATIVE_PY_ENTER);
                reg.register_with_id("py.py_exit", NATIVE_PY_EXIT);
                reg.register_with_id("py.py_item_kw", NATIVE_PY_ITEM_KW);
                // Plan 567 T06: may 值通道变体。
                reg.register_with_id("py.py_getattr_may", NATIVE_PY_GETATTR_MAY);
                reg.register_with_id("py.py_getitem_may", NATIVE_PY_GETITEM_MAY);
                reg.register_with_id("py.py_call_kw_may", NATIVE_PY_CALL_KW_MAY);
                reg.register_with_id("py.py_raise", NATIVE_PY_RAISE);
                reg.register_with_id("py.py_int", NATIVE_PY_INT);
                // Plan 602 (D2): Python 类派生工厂。
                reg.register_with_id("py.py_subclass", NATIVE_PY_SUBCLASS);
            }
        }
        if !self.py_native_map.contains_key("py_getattr") {
            self.py_native_map.insert(
                "py_getattr".to_string(),
                ("__pyobj__".to_string(), "__pyobj__.py_getattr".to_string()),
            );
            self.py_return_types
                .insert("py_getattr".to_string(), crate::py_ffi_types::PyType::Auto);
        }
        // Plan 539 W0 (DIV-PY-KWARGS-1): py_call_kw is the fixed 5-slot
        // keyword-argument channel that py_call-with-named-args lowers into
        // (posargs / kw_names / kw_vals as marshalled Auto lists).
        // Plan 539 W0 (DIV-PY-EXCEPT-1): py_call_may is the May-valued
        // variant (Result.Ok wrap / Result.Err on Python exceptions).
        for builtin in [
            "py_call_kw",
            "py_call_may",
            "py_iter",
            "py_next",
            "py_matmul",
            "py_getitem",
            "py_setitem",
            "py_slice",
            "py_call0",
            "py_with",
            // Plan 567 T12: with-as 块形态的桥半（原仅内联 codegen 通道，
            // 源级调用不可解析——E0401）。
            "py_enter",
            "py_exit",
            "py_item_kw",
            "py_float",
            "py_callable",
            // Plan 555 T04: 分发组合子配套三桥（B2 桥半/B6/D8）——
            // 固定 id 注册见 lib.rs init_py_ffi（467-469）。
            "py_setattr",
            "py_len",
            "py_type_name",
            // Plan 560 T04: B7/B8。
            "py_contains",
            "py_module",
            // Plan 560 T06 (D7) + T07 (C6)。
            "py_str",
            "py_pow",
            // Plan 560 T08。
            "py_truthy",
            "py_is",
            // Plan 567 T06 (P560-D2): may 值通道变体。
            "py_getattr_may",
            "py_getitem_may",
            "py_call_kw_may",
            "py_raise",
            "py_int",
            // Plan 602 (D2): Python 类派生工厂（exec 模板 + 回调挂载，481）。
            "py_subclass",
        ] {
            if !self.py_native_map.contains_key(builtin) {
                self.py_native_map.insert(
                    builtin.to_string(),
                    ("__pyobj__".to_string(), format!("__pyobj__.{}", builtin)),
                );
                self.py_return_types
                    .insert(builtin.to_string(), crate::py_ffi_types::PyType::Auto);
            }
        }
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> AutoResult<()> {
        self.assembly_compile_depth += 1;
        let result = self.compile_expr_inner(expr);
        self.assembly_compile_depth -= 1;
        result?;
        if self.assembly_compile_depth == 0 {
            self.verify_assembly_references()?;
        }
        Ok(())
    }

    fn compile_expr_inner(&mut self, expr: &Expr) -> AutoResult<()> {
        self.last_was_native_void = false; // reset for each expression
        self.last_expr_may_py = false; // Plan 569 D2: 粘性位随表达式边界重置（仅 py 臂/Ident 镜像置位）
        match expr {
            Expr::Int(i) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*i);
            }
            Expr::Bool(b) => {
                self.last_expr_type = ObjectType::Bool;
                // Plan 318: emit PUSH_BOOL so the value carries the bool nanbox tag
                // (not CONST_I32). Without this, `.editing = false` stores Int(0) and
                // `.editing == false` comparisons in the view builder fail.
                self.emit(OpCode::PUSH_BOOL);
                self.code.push(if *b { 1 } else { 0 });
            }
            // Plan 073 Stage A.5: Float literal support
            // Plan 118: Track type for output formatting
            Expr::Float(f, _) => {
                self.last_expr_type = ObjectType::Float;
                self.emit(OpCode::CONST_F32);
                self.emit_f32(*f as f32);
            }
            // Plan 073 Stage A.5: Double literal support
            Expr::Double(d, _) => {
                self.last_expr_type = ObjectType::Double;
                self.emit(OpCode::CONST_F64);
                self.emit_f64(*d);
            }
            // Plan 073 Stage B: I64 literal support
            Expr::I64(i) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I64);
                self.emit_i64(*i);
            }
            // Plan 073 Stage B: U64 literal support
            Expr::U64(u) => {
                self.emit(OpCode::CONST_U64);
                self.emit_u64(*u);
            }
            // Plan 073 Stage B: Uint literal support (use CONST_I32)
            // Plan 118: Track type for output formatting
            Expr::Uint(u) => {
                self.last_expr_type = ObjectType::Uint;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*u as i32);
            }
            // Plan 073 Stage B: I8 literal support (use CONST_I32)
            Expr::I8(i) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*i as i32);
            }
            // Plan 073 Stage B: U8 literal support (use CONST_I32)
            Expr::U8(u) => {
                self.last_expr_type = ObjectType::Int;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*u as i32);
            }
            // Plan 073 Stage B: Byte literal support (use CONST_I32)
            // Plan 118: Track type for output formatting (hex display)
            Expr::Byte(_b) => {
                self.last_expr_type = ObjectType::Byte;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*_b as i32);
            }
            // Plan 073 Stage B: Char literal support (use CONST_I32 for UTF-32 codepoint)
            Expr::Char(c) => {
                self.last_expr_type = ObjectType::Char;
                self.emit(OpCode::CONST_I32);
                self.emit_i32(*c as i32);
            }
            // Plan 073 Stage B: CStr literal support (use LOAD_STR like regular strings)
            Expr::CStr(s) => {
                // Add C string to constant pool and emit LOAD_STR <index>
                let bytes = s.as_bytes().to_vec();
                let idx = self.strings.len() as u32;
                self.strings.push(bytes);
                self.emit(OpCode::LOAD_STR);
                self.code.extend_from_slice(&idx.to_le_bytes());
            }
            _ => return self.expr_containers(expr),
        }
        Ok(())
    }

    /// PLAN-057 T7：web 内建编译期门禁的行级豁免——当前调用点（按
    /// current_source_line）同行或上一行含 `// vm-safe-allow` 即豁免。
    /// source_text 缺席（纯 AST 编译）时不可豁免（保守报错）。
    pub(super) fn web_builtin_call_exempt(&self) -> bool {
        let Some(text) = self.source_text.as_ref() else {
            return false;
        };
        let line1 = self.current_source_line as usize; // 1-based
        if line1 == 0 {
            return false;
        }
        let cur = text.split('\n').nth(line1 - 1);
        let prev = if line1 >= 2 {
            text.split('\n').nth(line1 - 2)
        } else {
            None
        };
        let mark = |l: Option<&str>| l.map(|s| s.contains("// vm-safe-allow")).unwrap_or(false);
        mark(cur) || mark(prev)
    }

    /// Plan 447 H1: multi-pattern (`A | B -> ...`) arm emission — per-pattern
    /// EQ with short-circuit OR (each hit JMP_IF_NZs to the shared matched
    /// label), then the arm body + end-jump. Single emission point shared by
    /// the literal-pattern path and the scalar-enum Cover::Tag path so the
    /// two stay in lockstep (双副本同源: Stmt::Is 与 Expr::Is 均经此)。
    pub(super) fn emit_is_or_arm(
        &mut self,
        target_var: usize,
        patterns: &[crate::ast::Expr],
        body: &crate::ast::Body,
        end_jumps: &mut Vec<usize>,
    ) -> AutoResult<()> {
        let mut match_jumps = Vec::new();
        for pat in patterns {
            self.emit_load_loc(target_var);
            self.compile_expr(pat)?;
            self.emit(OpCode::EQ);
            self.emit(OpCode::JMP_IF_NZ);
            match_jumps.push(self.emit_placeholder_i16());
        }
        // No pattern matched — jump to next branch
        self.emit(OpCode::JMP);
        let jump_to_next = self.emit_placeholder_i16();
        // === Matched label === — patch all JMP_IF_NZ to jump here
        let matched_pos = self.code.len();
        for j in &match_jumps {
            self.patch_jump_to(*j, matched_pos);
        }
        self.compile_stmt(&crate::ast::Stmt::Block(body.clone()))?;
        // Jump to end of is statement
        self.emit(OpCode::JMP);
        let jump_to_end = self.emit_placeholder_i16();
        end_jumps.push(jump_to_end);
        self.patch_jump(jump_to_next);
        Ok(())
    }

    // === Symbol Table Helpers ===

    /// Get parameter information for a function at a specific index
    /// Returns (type, mode) if found, None otherwise
    fn get_param_info(&self, func_name: &str, param_index: usize) -> Option<(Type, ParamMode)> {
        if let Some(params) = self.fn_params.get(func_name) {
            if param_index < params.len() {
                let param = &params[param_index];
                return Some((param.ty.clone(), param.mode));
            }
        }
        None
    }

    /// Compile a single argument for a function call with smart parameter passing
    /// This implements the Plan 088 ABO-01 strategy: "Semantic View, Implementation Copy"
    pub(super) fn compile_call_arg(
        &mut self,
        arg: &Expr,
        func_name: &str,
        param_index: usize,
    ) -> AutoResult<()> {
        // Get target parameter info (type and mode)
        let param_info = self.get_param_info(func_name, param_index);

        if let Some((param_ty, param_mode)) = param_info {
            // We have parameter info, use smart parameter passing
            match arg {
                Expr::Ident(var_name) => {
                    // Argument is a variable/identifier
                    // === BUG FIX === Use lookup_var() instead of self.locals.get()
                    // to properly search the scope stack
                    if let Some(var_index) = self.lookup_var(var_name.as_ref()) {
                        // Local variable: choose loading strategy based on type and mode
                        match param_mode {
                            ParamMode::View => {
                                // View mode: immutable reference
                                // TODO: Once LOAD_REF/STORE_REF are fully supported on the callee side,
                                // restore the reference passing optimization for large objects.
                                // Currently all parameters are passed by value (the instance ID / heap object ID).
                                self.emit_load_loc(var_index);
                            }
                            ParamMode::Mut => {
                                // Mut mode: mutable reference
                                // TODO: Once LOAD_MUT_REF is fully supported on the callee side,
                                // restore the reference passing optimization for large objects.
                                // Currently all parameters are passed by value (the instance ID / heap object ID).
                                self.emit_load_loc(var_index);
                            }
                            ParamMode::Copy => {
                                // Copy mode: explicit value passing
                                if param_ty.is_optimized_by_value() {
                                    // Small object: direct value passing (LOAD_LOC)
                                    self.emit_load_loc(var_index);
                                } else {
                                    // Large object + Copy: needs clone
                                    // For now, use LOAD_LOC (TODO: implement clone in future)
                                    self.emit_load_loc(var_index);
                                }
                            }
                            #[allow(deprecated)]
                            ParamMode::Take => {
                                // Take mode: move semantics (value passing)
                                // DEPRECATED: Use Move instead
                                self.emit_load_loc(var_index);
                            }
                            ParamMode::Move => {
                                // Move mode: ownership transfer (value passing)
                                self.emit_load_loc(var_index);
                            }
                        }
                        return Ok(());
                    }
                }
                _ => {
                    // Argument is a complex expression (constant, operation, etc.)
                    // Just compile it normally
                }
            }
        }

        // Fallback: compile argument as expression
        self.compile_expr(arg)
    }

    // === Closure Support (Plan 071) ===

    /// Get the current captured_vars map (top of stack)
    /// Plan 071 Phase 6.2: Helper for accessing captured variables
    pub(super) fn current_captured_vars(&self) -> &HashMap<String, usize> {
        self.captured_vars_stack.last().unwrap_or_else(|| {
            // If stack is empty, return empty map (not in a closure)
            static EMPTY_MAP: std::sync::OnceLock<std::collections::HashMap<String, usize>> =
                std::sync::OnceLock::new();
            EMPTY_MAP.get_or_init(|| HashMap::new())
        })
    }

    /// Get mutable reference to current captured_vars map (top of stack)
    /// Plan 071 Phase 6.2: Helper for modifying captured variables
    #[allow(dead_code)]
    fn current_captured_vars_mut(&mut self) -> &mut HashMap<String, usize> {
        if self.captured_vars_stack.is_empty() {
            // If stack is empty, push a new map
            self.captured_vars_stack.push(HashMap::new());
        }
        self.captured_vars_stack.last_mut().unwrap()
    }

    /// Push a new captured_vars level (for compiling a closure body)
    /// Plan 071 Phase 6.2: Support nested closures
    pub(super) fn push_captured_vars(&mut self, vars: HashMap<String, usize>) {
        self.captured_vars_stack.push(vars);
    }

    /// Pop the current captured_vars level (after compiling a closure body)
    /// Plan 071 Phase 6.2: Support nested closures
    pub(super) fn pop_captured_vars(&mut self) -> Option<HashMap<String, usize>> {
        if self.captured_vars_stack.is_empty() {
            None
        } else {
            self.captured_vars_stack.pop()
        }
    }

    /// Find free variables in an expression (variables that should be captured)
    /// Excludes: parameters and locally-defined variables
    fn find_free_vars(&self, expr: &Expr, params: &HashSet<String>) -> Vec<String> {
        let mut free_vars = HashSet::new();
        self.collect_free_vars(expr, params, &mut free_vars);
        free_vars.into_iter().collect()
    }

    /// Recursively collect free variables from an expression
    pub(super) fn collect_free_vars(
        &self,
        expr: &Expr,
        exclude: &HashSet<String>,
        free_vars: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Ident(name) => {
                let name_str = name.to_string();
                // Only collect if not in exclude list (parameters/locals)
                if !exclude.contains(&name_str) {
                    free_vars.insert(name_str);
                }
            }
            Expr::Bina(lhs, _op, rhs) => {
                self.collect_free_vars(lhs, exclude, free_vars);
                self.collect_free_vars(rhs, exclude, free_vars);
            }
            Expr::Unary(_op, rhs) => {
                self.collect_free_vars(rhs, exclude, free_vars);
            }
            Expr::Call(call) => {
                self.collect_free_vars(&call.name, exclude, free_vars);
                for arg in &call.args.args {
                    if let crate::ast::Arg::Pos(expr) = arg {
                        self.collect_free_vars(expr, exclude, free_vars);
                    }
                }
            }
            Expr::Array(elems) => {
                for elem in elems {
                    self.collect_free_vars(elem, exclude, free_vars);
                }
            }
            Expr::Block(body) => {
                // For block expressions, exclude local variables defined in the block
                // PLAN-598 T-03 (DIV-PY-CLOSURE-1 VM facet): `let` bindings
                // (Stmt::Store with StoreKind::Let/Const) were never excluded,
                // so a block-local name was misclassified as a captured var —
                // the store went to the local slot but subsequent loads read
                // the closure's (empty) capture env as garbage. Walk the
                // initializer FIRST (its free vars belong to the enclosing
                // scope), then bind the name for the rest of the block.
                // Remaining gap (pre-existing): If/For/Block statements nested
                // in a block are still not walked here.
                let mut inner_exclude = exclude.clone();
                for stmt in &body.stmts {
                    match stmt {
                        Stmt::Store(store) => {
                            self.collect_free_vars(&store.expr, &inner_exclude, free_vars);
                            if matches!(
                                store.kind,
                                crate::ast::StoreKind::Let | crate::ast::StoreKind::Const
                            ) {
                                inner_exclude.insert(store.name.to_string());
                            }
                        }
                        Stmt::Expr(e) => {
                            self.collect_free_vars(e, &inner_exclude, free_vars);
                        }
                        Stmt::Return(e) => {
                            self.collect_free_vars(e, &inner_exclude, free_vars);
                        }
                        _ => {}
                    }
                }
            }
            Expr::If(if_expr) => {
                for branch in &if_expr.branches {
                    self.collect_free_vars(&branch.cond, exclude, free_vars);
                    for stmt in &branch.body.stmts {
                        if let Stmt::Expr(e) = stmt {
                            self.collect_free_vars(e, exclude, free_vars);
                        }
                    }
                }
            }
            Expr::Closure(inner_closure) => {
                // For nested closures, process inner body with updated excludes
                let mut inner_exclude = exclude.clone();
                for p in &inner_closure.params {
                    inner_exclude.insert(p.name.to_string());
                }
                self.collect_free_vars(&inner_closure.body, &inner_exclude, free_vars);
            }
            // Dot expressions - check object (e.g., x in x.view)
            Expr::View(inner) | Expr::Mut(inner) | Expr::Move(inner) | Expr::Take(inner) => {
                self.collect_free_vars(inner, exclude, free_vars);
            }
            Expr::Dot(obj, _method) => {
                self.collect_free_vars(obj, exclude, free_vars);
            }
            // Index expressions
            Expr::Index(arr, idx) => {
                self.collect_free_vars(arr, exclude, free_vars);
                self.collect_free_vars(idx, exclude, free_vars);
            }
            // Primitives - no identifiers to collect
            Expr::Int(_)
            | Expr::Float(_, _)
            | Expr::Str(_)
            | Expr::Bool(_)
            | Expr::Nil
            | Expr::Byte(_) => {}
            // Other expressions - add more cases as needed
            _ => {}
        }
    }

    /// Plan 417-E3-P4: call-site bound checking for generic functions.
    ///
    /// For each positional argument whose corresponding parameter is declared
    /// as a BOUNDED type param of the callee (`fn max_of<T has Comparable>(a
    /// T, ...)`), verify the argument's static type implements the bound.
    ///
    /// Conservative by design — an error is raised ONLY when all of the
    /// following hold; everything else passes (deferred to the runtime
    /// CALL_SPEC dispatch or left unchecked):
    /// - the argument is a plain identifier with a known static type;
    /// - that type is a concrete user type, not itself a type param of the
    ///   caller (pass-through generics stay dynamic);
    /// - the bound names a spec known to the TypeStore;
    /// - the argument's TypeDecl is resolvable and lists neither the spec
    ///   (`type X as Spec`) nor a generic spec impl for it.
    pub(super) fn check_generic_call_bounds(
        &self,
        callee: &str,
        args: &[crate::ast::Arg],
    ) -> AutoResult<()> {
        let Some(bounds) = self.fn_type_param_bounds.get(callee) else {
            return Ok(());
        };
        if bounds.iter().all(|(_, c)| c.is_empty()) {
            return Ok(());
        }
        let Some(params) = self.fn_params.get(callee) else {
            return Ok(());
        };
        for (i, arg) in args.iter().enumerate() {
            let crate::ast::Arg::Pos(Expr::Ident(var_name)) = arg else {
                continue;
            };
            // Parameter i's declared type names a bounded type param?
            let Some(param) = params.get(i) else { continue };
            let Type::User(ptd) = &param.ty else {
                continue;
            };
            let Some((_, constraints)) =
                bounds.iter().find(|(n, _)| n.as_str() == ptd.name.as_str())
            else {
                continue;
            };
            if constraints.is_empty() {
                continue;
            }
            // Argument's static type must be a concrete user type. Typed
            // locals register as GenericInstance{base_name: T, args: []} when
            // constructed (`var s Score = Score {...}`) — treat the base name
            // as the concrete type.
            let arg_ty_name = match self.var_types.get(var_name.as_ref()) {
                Some(Type::User(atd)) => atd.name.to_string(),
                Some(Type::GenericInstance(inst)) if inst.args.is_empty() => {
                    inst.base_name.to_string()
                }
                _ => {
                    continue;
                }
            };
            if self.current_fn_type_params.contains(&arg_ty_name) {
                continue; // pass-through generic — stays dynamic
            }
            for constraint in constraints {
                // A bound parses as Type::Spec when the spec is declared
                // before the fn (`T has Comparable`), Type::User otherwise
                // (forward reference placeholder) — accept both.
                let spec_name = match constraint {
                    Type::Spec(s) => s.borrow().name.to_string(),
                    Type::User(u) => u.name.to_string(),
                    _ => continue,
                };
                let (spec_known, arg_decl) = {
                    let ts = self.type_store.read().unwrap();
                    (
                        ts.lookup_spec_decl_str(&spec_name).is_some(),
                        ts.lookup_type_decl_str(&arg_ty_name),
                    )
                };
                if !spec_known {
                    continue; // not a spec bound (builtin/unknown) — unchecked
                }
                let Some(arg_decl) = arg_decl else {
                    continue; // type unresolvable here (cross-module) — unchecked
                };
                let implements = arg_decl
                    .specs
                    .iter()
                    .any(|s| s.as_str() == spec_name.as_str())
                    || arg_decl
                        .spec_impls
                        .iter()
                        .any(|si| si.spec_name.as_str() == spec_name.as_str());
                if !implements {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "Type '{}' does not implement spec '{}' required by the bound of generic function '{}' (argument {})",
                            arg_ty_name, spec_name, callee, i + 1
                        ),
                        span: SourceSpan::new(0_usize.into(), 0_usize.into()),
                    }
                    .into());
                }
            }
        }
        Ok(())
    }

    /// Check if a variable is captured with unsafe borrowing (.view or .mut)
    /// Returns the expression that uses unsafe borrowing, if found
    /// Plan 071 Phase 6.1: Borrow Checking Integration
    fn check_unsafe_capture<'a>(&'a self, var_name: &str, expr: &'a Expr) -> Option<&'a Expr> {
        match expr {
            // Direct unsafe borrow: x.view or x.mut
            Expr::View(inner) => {
                // Check if this is borrowing the target variable
                if let Expr::Ident(name) = inner.as_ref() {
                    if name.to_string() == var_name {
                        return Some(expr); // Found unsafe capture
                    }
                }
                // Recursively check inner expression
                self.check_unsafe_capture(var_name, inner)
            }
            Expr::Mut(inner) => {
                // Check if this is borrowing the target variable
                if let Expr::Ident(name) = inner.as_ref() {
                    if name.to_string() == var_name {
                        return Some(expr); // Found unsafe capture
                    }
                }
                // Recursively check inner expression
                self.check_unsafe_capture(var_name, inner)
            }

            // Binary expressions - check both sides
            Expr::Bina(lhs, _op, rhs) => {
                // First check left side
                if let Some(found) = self.check_unsafe_capture(var_name, lhs) {
                    return Some(found);
                }
                // Then check right side
                self.check_unsafe_capture(var_name, rhs)
            }

            // Unary expressions - check operand
            Expr::Unary(_op, rhs) => self.check_unsafe_capture(var_name, rhs),

            // Function calls - check arguments
            Expr::Call(call) => {
                // Check function name
                if let Some(found) = self.check_unsafe_capture(var_name, &call.name) {
                    return Some(found);
                }
                // Check arguments
                for arg in &call.args.args {
                    if let crate::ast::Arg::Pos(arg_expr) = arg {
                        if let Some(found) = self.check_unsafe_capture(var_name, arg_expr) {
                            return Some(found);
                        }
                    }
                }
                None
            }

            // Arrays - check elements
            Expr::Array(elems) => {
                for elem in elems {
                    if let Some(found) = self.check_unsafe_capture(var_name, elem) {
                        return Some(found);
                    }
                }
                None
            }

            // Block expressions - check statements
            Expr::Block(body) => {
                for stmt in &body.stmts {
                    match stmt {
                        Stmt::Expr(e) => {
                            if let Some(found) = self.check_unsafe_capture(var_name, e) {
                                return Some(found);
                            }
                        }
                        Stmt::Return(e) => {
                            if let Some(found) = self.check_unsafe_capture(var_name, e) {
                                return Some(found);
                            }
                        }
                        // Other statements don't contain expressions we care about
                        _ => {}
                    }
                }
                None
            }

            // Closure expressions - recurse into closure body
            Expr::Closure(inner_closure) => {
                self.check_unsafe_capture(var_name, &inner_closure.body)
            }

            // If expressions - check branches
            Expr::If(if_expr) => {
                // Check all branches
                for branch in &if_expr.branches {
                    // Check condition
                    if let Some(found) = self.check_unsafe_capture(var_name, &branch.cond) {
                        return Some(found);
                    }
                    // Check branch body
                    if let Some(found) = self.check_unsafe_capture_in_body(var_name, &branch.body) {
                        return Some(found);
                    }
                }
                // Check else branch
                if let Some(else_body) = &if_expr.else_ {
                    self.check_unsafe_capture_in_body(var_name, else_body)
                } else {
                    None
                }
            }

            // Index expressions - check both parts
            Expr::Index(arr, idx) => {
                if let Some(found) = self.check_unsafe_capture(var_name, arr) {
                    return Some(found);
                }
                self.check_unsafe_capture(var_name, idx)
            }

            // Dot expressions - check both object and method
            Expr::Dot(obj, _method) => {
                // Check object (e.g., x in x.view)
                if let Expr::Ident(name) = obj.as_ref() {
                    if name.to_string() == var_name {
                        // This is a direct reference to the variable
                        // Check if this dot expression is part of a borrow
                        // We need to look at the outer context to determine this
                        // For now, we'll be conservative and check the method name
                        return None; // Safe - just accessing a field
                    }
                }
                self.check_unsafe_capture(var_name, obj)
            }

            // Identifiers - direct references are safe (copy semantics)
            // Only .view/.mut are unsafe, not direct variable access
            Expr::Ident(_name) => None,

            // Other expressions - no unsafe borrowing possible
            _ => None,
        }
    }

    /// Check for unsafe captures in a Body (block)
    /// Plan 071 Phase 6.1: Helper for checking if/branch bodies
    fn check_unsafe_capture_in_body<'a>(
        &'a self,
        var_name: &str,
        body: &'a crate::ast::Body,
    ) -> Option<&'a Expr> {
        for stmt in &body.stmts {
            if let Stmt::Expr(expr) = stmt {
                if let Some(found) = self.check_unsafe_capture(var_name, expr) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Compile closure expression (Plan 071)
    pub fn compile_closure(&mut self, closure: &Closure) -> AutoResult<()> {
        // Step 1: Find free variables to capture
        let param_names: HashSet<String> =
            closure.params.iter().map(|p| p.name.to_string()).collect();
        let free_vars = self.find_free_vars(&closure.body, &param_names);

        // Plan 071 Phase 6.1: Borrow Checking - Check for unsafe captures
        // Block .view/.mut in closure capture to prevent dangling references
        for var_name in &free_vars {
            if let Some(unsafe_expr) = self.check_unsafe_capture(var_name, &closure.body) {
                // Found unsafe capture - emit compiler error
                let span = self.get_expr_span(unsafe_expr);
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Cannot capture borrowed value '{0}' in closure. \
                        Closures may outlive their parent scope, causing dangling references. \
                        Use .take to transfer ownership, or remove .view/.mut. \
                        Note: Default capture semantics copy the value, which is safe.",
                        var_name
                    ),
                    span,
                }
                .into());
            }
        }

        // Step 2: For each captured variable, emit code to load its current value
        // For MVP, we emit LOAD_LOCAL for each captured variable
        // TODO: In Phase 3.6, emit proper variable loading based on scope
        for var_name in &free_vars {
            // Try to look up the variable in current scope
            if let Some(var_index) = self.lookup_var(var_name) {
                self.emit_load_loc(var_index);
            } else if var_name == "self" && self.lookup_var("__state").is_some() {
                // PLAN-622 (e2): `self` inside a closure body is the enclosing
                // handler's state object. The handler frame holds it as the
                // `__state` param (slot 0), which is not visible under the
                // name "self" — the capture previously degraded to CONST_0 and
                // every `self.field` read inside the closure resolved nil
                // (comparators silently missed). Capture the state value by
                // env (slot metadata stays 0xFFFF) under the name "self".
                let state_idx = self.lookup_var("__state").unwrap();
                self.emit_load_loc(state_idx);
            } else {
                // Variable not found - emit 0 as fallback
                self.emit(OpCode::CONST_0);
            }
        }

        // Step 3: Emit CLOSURE opcode at current position (Plan 071 Phase 6.2)
        // Save position for reloc entry
        let closure_opcode_offset = self.code.len();

        // Create unique symbol name for this closure
        let closure_symbol = format!("closure_{}", closure_opcode_offset);

        // Emit CLOSURE opcode with placeholder address
        self.emit(OpCode::CLOSURE);
        let func_addr_offset = self.code.len() as u32; // Position where func_addr will be
        self.code.extend_from_slice(&(0u32).to_le_bytes()); // Placeholder - will be filled later
        self.code.push(free_vars.len() as u8); // capture_count
        self.code.push(closure.params.len() as u8); // n_args (for CALL_CLOSURE)

        // Emit variable name indices + slot offsets for each captured variable
        // Plan 385: slot_offset lets LOAD/STORE_CAPTURED directly access the
        // creator's stack frame (by-reference capture instead of value copy).
        for var_name in &free_vars {
            let var_idx = self.add_string(var_name);
            self.code.extend_from_slice(&var_idx.to_le_bytes());
            // Plan 385: emit the variable's local slot offset (relative to bp+1)
            // Plan 454 E5a: 函数参数域打 0x8000 旗标,运行期按 bp-(n_args-real+1)
            // 负向寻址解析为绝对槽位。
            // PLAN-622 (e1): 槽位换算必须与 emit_store_loc/emit_load_loc 的
            // 编址一致 —— local_index = idx - fn_scope_start - n_args、参数域
            // real = idx - fn_scope_start。此前用裸 scope idx(含全局/参数域
            // 偏移)直接当槽位,widget handler(fn_scope_start=1)里捕获的局部
            // 变量差一位:by-ref 读到相邻槽的垃圾,还遮蔽了正确的 env 值,
            // 闭包比较恒不命中。fss=0 的旧语料两者恰好相等,故 385/454 未爆。
            let slot_offset = self
                .lookup_var(var_name)
                .map(|idx| {
                    let fss = self.fn_scope_start;
                    let n_args = self.current_fn_n_args;
                    if idx >= fss && idx < fss.saturating_add(n_args) {
                        // parameter domain — real = relative param index
                        0x8000u16 | (((idx - fss) as usize) as u16 & 0x3FFF)
                    } else {
                        idx.saturating_sub(fss).saturating_sub(n_args) as u16
                    }
                })
                .unwrap_or(0xFFFF); // 0xFFFF = no slot (fallback to env)
                                    // Plan 419/454 E5a: 记录 by-ref 捕获槽,块尾释放跳过。
                                    // 0x8000 旗标 = 函数参数域(运行期负向寻址),既不进释放组,
                                    // 由引擎在 CLOSURE 执行期解析为创建帧绝对槽位。
            if slot_offset != 0xFFFF && slot_offset & 0x8000 == 0 {
                self.byref_captured_slots.insert(slot_offset as usize);
            }
            self.code.extend_from_slice(&slot_offset.to_le_bytes());
        }

        // Step 3.5: Emit JMP to skip closure body during normal execution
        // After CLOSURE opcode, we need to jump over the closure body
        // JMP offset will be patched later after we know body size
        self.emit(OpCode::JMP);
        let jmp_offset_pos = self.code.len();
        self.code.extend_from_slice(&(0i16).to_le_bytes()); // Placeholder - will be filled after body

        // Step 4: Compile closure body as separate function (Plan 071 Phase 6.2)
        // Closure body is compiled AFTER the CLOSURE opcode (at the end of current code)

        // Create captured variable map for this closure
        let mut new_captured_vars = HashMap::new();
        for (idx, var_name) in free_vars.iter().enumerate() {
            new_captured_vars.insert(var_name.clone(), idx);
        }

        // Push new captured_vars level for nested closures
        self.push_captured_vars(new_captured_vars);

        // Compile closure body at the END of current code
        let func_addr = self.code.len() as u32;

        // Save old current_fn_n_args and set new value for closure
        let old_fn_n_args = self.current_fn_n_args;
        let old_fn_scope_start = self.fn_scope_start;
        // PLAN-598 T-03 (DIV-PY-CLOSURE-1 VM facet): per-closure max_locals
        // accounting, mirroring the per-fn reset in compile_fn — the closure's
        // local frame size is measured in its own index space.
        let old_max_locals = self.max_locals;
        self.max_locals = 0;
        self.current_fn_n_args = closure.params.len();

        // Enter new scope for closure parameters
        self.push_scope();

        // Record starting index for closure scope (for correct parameter indexing)
        self.fn_scope_start = self.scope_stack.iter().map(|s| s.len()).sum();

        for param in &closure.params {
            self.add_var(&param.name);
        }
        let locals_base = self.max_locals; // = fn_scope_start + n_args

        // Compile closure body expression
        self.compile_expr(&closure.body)?;

        // PLAN-598 T-03: closures never reserved stack for their locals — a
        // `let` inside a closure body addressed bp-relative memory that was
        // never allocated, reading back garbage (None/0). Fns reserve locals
        // via FN_PROLOG + RESERVE_STACK; closures now insert RESERVE_STACK
        // n_locals at body entry. The body (including nested closures) was
        // just compiled at >= func_addr, so those bytes shift by 2: jump
        // placeholders, relocs and exports created during the body compile
        // move with it (same post-insertion dance compile_fn does).
        let n_locals = self.max_locals.saturating_sub(locals_base);
        if n_locals > 0 {
            let n_locals = n_locals.min(255);
            for idx in &mut self.jump_placeholders {
                if *idx >= func_addr as usize {
                    *idx += 2;
                }
            }
            for reloc in &mut self.relocs {
                if reloc.offset >= func_addr {
                    reloc.offset += 2;
                }
            }
            let shifted: Vec<(String, u32)> = self
                .exports
                .iter()
                .filter(|(_, addr)| **addr >= func_addr)
                .map(|(name, addr)| (name.clone(), addr + 2))
                .collect();
            for (name, addr) in shifted {
                self.exports.insert(name, addr);
            }
            self.code.insert(func_addr as usize, n_locals as u8);
            self.code
                .insert(func_addr as usize, OpCode::RESERVE_STACK as u8);
        }
        self.max_locals = old_max_locals;

        // Emit RET for closure
        self.emit(OpCode::RET);
        self.code.push(closure.params.len() as u8);

        // Exit closure scope
        self.pop_scope();

        // Restore old current_fn_n_args and fn_scope_start
        self.current_fn_n_args = old_fn_n_args;
        self.fn_scope_start = old_fn_scope_start;

        // Pop captured_vars (restore outer closure's captured vars)
        self.pop_captured_vars();

        // Step 4.5: Back-fill JMP offset to skip closure body during normal execution
        let body_end_addr = self.code.len() as u32;
        let jmp_offset = ((body_end_addr as i32) - (jmp_offset_pos as i32 + 2)) as i16; // +2 for the i16 offset itself
        let jmp_bytes = jmp_offset.to_le_bytes();
        for (i, byte) in jmp_bytes.iter().enumerate() {
            self.code[jmp_offset_pos as usize + i] = *byte;
        }

        // Step 5: Back-fill the func_addr in the CLOSURE opcode
        // Now we know the actual function address, so we can fill it in
        let func_addr_bytes = func_addr.to_le_bytes();
        for (i, byte) in func_addr_bytes.iter().enumerate() {
            let idx = func_addr_offset as usize + i;
            self.code[idx] = *byte;
        }

        // Step 6: Create reloc entry for this closure (Plan 071 Phase 6.2)
        self.relocs.push(crate::vm::loader::RelocEntry {
            offset: func_addr_offset,
            symbol_name: closure_symbol.clone(),
            reloc_type: crate::vm::loader::RelocType::FuncCall,
            source_pos: None,
        });

        // Export the closure function address
        self.exports.insert(closure_symbol, func_addr);

        Ok(())
    }

    // ========== Plan 073: Instance Method Call Support ==========
}
