//! PLAN-843 结构拆分（自 rust.rs 原样搬移；算法/行为零变化）。
use super::*;

impl RustTrans {
    /// PLAN-738 T-11 引用闭包收集器：六核心 producer 的每条真实发射路径
    /// 都在此收敛——限定 Dot 形状（含 Json 别名）、裸名导入/wildcard 拼写
    /// （重建限定文本验证）、裸名专用臂（发射文本自带 a2r_std::<core>::
    /// 限定）。无公共面的私有 producer 不获得公共声明证明（与 P1 同义）。
    pub(super) fn collect_core_reference(
        &mut self,
        call: &Call,
        text: &str,
        args_text: &str,
    ) -> AutoResult<()> {
        let core = crate::stdlib_assembly::validate::CORE_MODULES;
        if let Expr::Dot(object, symbol) = call.name.as_ref() {
            if let Expr::Ident(module) = object.as_ref() {
                if !self.local_var_types.contains_key(module)
                    && !self.local_modules.contains(module.as_str())
                    && !self.sibling_modules.contains(module.as_str())
                {
                    let module = if module.as_str() == "Json" {
                        "json"
                    } else {
                        module.as_str()
                    };
                    if core.contains(&module) {
                        let proof = crate::stdlib_assembly::host::verify_rust_reference(
                            module,
                            symbol.as_str(),
                            text,
                            self.assembly_runtime,
                        )?;
                        if let Some(proof) = proof {
                            self.assembly_references.push(proof);
                            return Ok(());
                        }
                        // P738-R6-01 落穿：裸名无公共顶层面（如 json.keys——
                        // 公共面是 JsonValue.keys 方法）时按方法分母归属再
                        // 验证，不静默放行。
                        return self.collect_core_method_reference(
                            object,
                            symbol.as_str(),
                            text,
                            args_text,
                        );
                    }
                }
                return self.collect_core_method_reference(
                    object,
                    symbol.as_str(),
                    text,
                    args_text,
                );
            }
            // P738-R5-02：receiver 方法调用（对象为变量/表达式）——发射
            // 文本带核心 FQN 时按该模块公共方法分母归属验证（JsonValue
            // 十二个公共方法面）；无公共面映射的 legacy 发射变体维持已
            // 登记边界（P738-D2），不冒称证明。
            return self.collect_core_method_reference(object, symbol.as_str(), text, args_text);
        }
        // 三段限定形状 `auto.<core>.<method>()`（Expr::Bina 链）：核心模块
        // 面同样走 strict 门——io/net 等无 Rust producer 的模块在此诚实
        // 拒绝，不再发射幻影 `a2r_std::io::...`。
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if let (Expr::Bina(inner_lhs, Op::Dot, inner_rhs), Expr::Ident(method)) =
                    (lhs.as_ref(), rhs.as_ref())
                {
                    if let (Expr::Ident(auto_name), Expr::Ident(module)) =
                        (inner_lhs.as_ref(), inner_rhs.as_ref())
                    {
                        if auto_name == "auto"
                            && crate::stdlib_assembly::validate::CORE_MODULES
                                .contains(&module.as_str())
                        {
                            if let Some(proof) =
                                crate::stdlib_assembly::host::verify_rust_reference(
                                    module,
                                    method,
                                    text,
                                    self.assembly_runtime,
                                )?
                            {
                                self.assembly_references.push(proof);
                            }
                        }
                    }
                }
            }
            return Ok(());
        }
        // 裸名拼写：本地函数遮蔽优先（用户同名符号不是 stdlib 引用）。
        let Expr::Ident(name) = call.name.as_ref() else {
            return Ok(());
        };
        if self.fn_ret_types.contains_key(name) || self.fn_param_types.contains_key(name) {
            return Ok(());
        }
        if let Some(module) = self.stdlib_named_imports.get(name.as_str()).cloned() {
            // 具名导入：裸名调用重建限定文本携带真实实参面验证（元数/
            // async 证据）；专用臂已限定的文本原样验证，不重复前缀。
            let already_qualified =
                text.contains(&format!("a2r_std::{module}::{}(", name.as_str()));
            let qualified = if already_qualified {
                text.to_string()
            } else {
                text.replacen(
                    &format!("{}(", name.as_str()),
                    &format!("a2r_std::{module}::{}(", name.as_str()),
                    1,
                )
            };
            if qualified != text {
                if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                    &module,
                    name,
                    &qualified,
                    self.assembly_runtime,
                )? {
                    self.assembly_references.push(proof);
                }
            }
            return Ok(());
        }
        if text.contains("a2r_std::") {
            // 裸名专用臂（如流生产者）：发射文本自带限定——按 FQN 归属验证。
            let mut found = None;
            for module in core {
                if text.contains(&format!("a2r_std::{module}::")) {
                    found = Some((*module).to_string());
                    break;
                }
            }
            if let Some(module) = found {
                if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                    &module,
                    name,
                    text,
                    self.assembly_runtime,
                )? {
                    self.assembly_references.push(proof);
                }
            }
            return Ok(());
        }
        if !self.stdlib_wildcard_modules.is_empty() {
            // wildcard 裸名：公共面唯一归属才可证明；多模块同名=歧义拒绝。
            let owners: Vec<&str> = self
                .stdlib_wildcard_modules
                .iter()
                .filter(|module| crate::stdlib_assembly::host::public_symbol_exists(module, name))
                .map(String::as_str)
                .collect();
            match owners.as_slice() {
                [module] => {
                    let qualified = text.replacen(
                        &format!("{}(", name.as_str()),
                        &format!("a2r_std::{module}::{}(", name.as_str()),
                        1,
                    );
                    if qualified != text {
                        if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                            module,
                            name,
                            &qualified,
                            self.assembly_runtime,
                        )? {
                            self.assembly_references.push(proof);
                        }
                    } else {
                        return Err(format!(
                            "STDASSEMBLY.SIGNATURE_UNVERIFIED: {module}.{}: bare wildcard call has no reconstructable argument face",
                            name.as_str()
                        )
                        .into());
                    }
                }
                [] => {}
                many => {
                    return Err(format!(
                        "STDASSEMBLY.SIGNATURE_UNVERIFIED: {}: bare name is ambiguous across wildcard modules {many:?}",
                        name.as_str()
                    )
                    .into());
                }
            }
        }
        Ok(())
    }

    /// Plan 396 §2.1: `tc.field` passed as a CALL ARGUMENT where `tc` is a
    /// borrowed loop variable (`for tc in &coll`). Moving a non-Copy, non-str
    /// field out of a shared ref is E0507 — append `.clone()`. Str fields are
    /// excluded: fn params adapt to `&str` (see the str-param path), so a
    /// String clone would break the call site. Unknown inferred types clone
    /// conservatively (mirrors the struct-field site in
    /// write_expr_for_struct_field, Plan 399 P11.4).
    pub(super) fn maybe_clone_borrowed_iter_field(
        &self,
        expr: &Expr,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        // Plan 396 §2.1 (bare loop var): passing the borrowed loop variable
        // ITSELF (`for t in &coll { f(t) }` with t: &Arc<..>) to an owned
        // param also needs .clone() — same gate as fields below.
        if let Expr::Ident(name) = expr {
            if self.borrowed_iter_vars.contains(name) {
                let ty = self
                    .local_var_types
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or(Type::Unknown);
                let str_like = matches!(
                    ty,
                    Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice
                );
                let copy_like = matches!(
                    ty,
                    Type::Int
                        | Type::Uint
                        | Type::U64
                        | Type::I64
                        | Type::USize
                        | Type::Float
                        | Type::Double
                        | Type::Bool
                        | Type::Char
                );
                if !str_like && !copy_like {
                    write!(out, ".clone()")?;
                }
                return Ok(());
            }
        }
        if let Expr::Dot(obj, _field) = expr {
            if let Expr::Ident(obj_name) = obj.as_ref() {
                if self.borrowed_iter_vars.contains(obj_name) {
                    // Resolve the field's declared type via the loop var's
                    // element struct (registered at the loop site); fall back
                    // to expression inference when the chain is unavailable.
                    let field_ty = self
                        .local_var_types
                        .get(obj_name.as_str())
                        .and_then(|obj_ty| {
                            if let Type::User(type_decl) = obj_ty {
                                self.struct_field_types
                                    .get(type_decl.name.as_str())
                                    .and_then(|fs| {
                                        fs.iter()
                                            .find(|(n, _)| n.as_str() == _field.as_str())
                                            .map(|(_, t)| t.clone())
                                    })
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| self.infer_type_from_expr(expr));
                    let ty = field_ty;
                    let str_like = matches!(
                        ty,
                        Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice
                    );
                    let copy_like = matches!(
                        ty,
                        Type::Int
                            | Type::Uint
                            | Type::U64
                            | Type::I64
                            | Type::USize
                            | Type::Float
                            | Type::Double
                            | Type::Bool
                            | Type::Char
                    );
                    if !str_like && !copy_like {
                        write!(out, ".clone()")?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Plan 610 ⑤: emit the C-ABI export wrapper module for an `#[export]` fn.
    ///
    /// Shape: `mod <name>_c_export { #[unsafe(no_mangle)] pub extern "<abi>"
    /// fn <name>(C-ABI params) { super::<name>(boundary conversions) } }`.
    /// The symbol equals the fn name (no_mangle ignores the module path), the
    /// safe body below keeps its original name so every call site in the
    /// product is untouched (zero whole-program rename mapping). Type
    /// fidelity (strategy A, plan §2 T-01): Auto `int` crosses the boundary
    /// as i32 with explicit `as` casts to/from the body's i64; `cstr` params
    /// arrive as `*const c_char` (null-tolerant → "") and cstr returns hand
    /// ownership to C via `CString::into_raw` (interior-NUL degrades to an
    /// empty C string rather than panicking across the FFI boundary).
    pub(super) fn export_wrapper_module(
        &mut self,
        fn_decl: &Fn,
        abi: &str,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        let unsupported = |what: &str| -> crate::AutoError {
            crate::AutoError::Msg(format!(
                "Plan 610 ⑤: #[export] 暂不支持 {}（fn {}）",
                what, fn_decl.name
            ))
        };
        if fn_decl.parent.is_some() {
            return Err(unsupported("方法（仅顶层函数可导出）"));
        }
        if !fn_decl.type_params.is_empty() || !fn_decl.const_params.is_empty() {
            return Err(unsupported("泛型函数"));
        }
        if fn_decl.is_test {
            return Err(unsupported("#[test] 组合"));
        }
        if fn_decl.api_attrs.is_some() {
            return Err(unsupported("#[api] 组合"));
        }
        if matches!(fn_decl.ret, Type::Handle { .. })
            || matches!(&fn_decl.ret, Type::GenericInstance(inst) if inst.base_name == "Future")
        {
            return Err(unsupported("async 返回（~T）"));
        }

        // C-ABI param types + boundary conversion. cstr params convert through
        // a `let …_conv: String` local because the safe body's str-family
        // params are `&str` (rust_param_type_name) — a borrow needs an owner
        // that outlives the call. `*T` with a named pointee is a HANDLE: the
        // wrapper takes the raw `*mut T` and derefs null-safely, passing
        // `&mut T` to the body (see the body-side emission twin below); at
        // most one handle param per fn (engine-face shape).
        let mut handle_param: Option<String> = None;
        let mut abi_params: Vec<String> = Vec::new();
        let mut conv_prelude: Vec<String> = Vec::new();
        let mut call_args: Vec<String> = Vec::new();
        let mut has_any_ptr = false;
        for param in &fn_decl.params {
            let name = param.name.to_string();
            if matches!(&param.ty, Type::Ptr(_)) {
                has_any_ptr = true;
            }
            match &param.ty {
                // Width bridge: C `int` (i32) ↔ Auto int (i64 body).
                Type::Int => {
                    abi_params.push(format!("{}: i32", name));
                    call_args.push(format!("{} as i64", name));
                }
                // Borrowed C string → body's &str; null-tolerant (→ "").
                // unsafe is scoped to CStr::from_ptr exactly (wrapper fns
                // without cstr params emit no unsafe block at all).
                Type::CStrLit => {
                    abi_params.push(format!("{}: *const std::os::raw::c_char", name));
                    conv_prelude.push(format!(
                        "let {n}_conv: String = if {n}.is_null() {{ String::new() }} else {{ unsafe {{ std::ffi::CStr::from_ptr({n}) }}.to_string_lossy().into_owned() }};",
                        n = name
                    ));
                    call_args.push(format!("&{}_conv", name));
                }
                // Opaque session handle → null-safe &mut deref in the wrapper;
                // the body's param becomes `&mut <T>`.
                Type::Ptr(_) if Self::export_ptr_pointee_is_handle(&param.ty) => {
                    let pointee = match &param.ty {
                        Type::Ptr(p) => self.rust_type_name(&p.of.borrow()),
                        _ => unreachable!(),
                    };
                    if handle_param.is_some() {
                        return Err(crate::AutoError::Msg(format!(
                            "Plan 610 ⑤: #[export] fn {} 至多一个句柄形参（*T 具名指向）",
                            fn_decl.name
                        )));
                    }
                    abi_params.push(format!("{}: *mut {}", name, pointee));
                    // Shadowed inside the Some(h) arm below.
                    call_args.push(name.clone());
                    handle_param = Some(name);
                }
                // Buffer pointers and ABI-identical scalars pass through.
                Type::Ptr(_) => {
                    has_any_ptr = true;
                    let t = self.rust_type_name(&param.ty);
                    abi_params.push(format!("{}: {}", name, t));
                    call_args.push(name);
                }
                Type::I64
                | Type::Uint
                | Type::U64
                | Type::USize
                | Type::Byte
                | Type::Bool
                | Type::Char
                | Type::Float
                | Type::Double => {
                    let t = self.rust_type_name(&param.ty);
                    abi_params.push(format!("{}: {}", name, t));
                    call_args.push(name);
                }
                other => {
                    return Err(crate::AutoError::Msg(format!(
                        "Plan 610 ⑤: #[export] 参数 {} 的类型 {} 暂不在 ABI 面（支持 int/i64/uint/u64/usize/byte/bool/char/float/cstr/*T）",
                        name, other
                    )));
                }
            }
        }
        if matches!(&fn_decl.ret, Type::Ptr(_)) {
            has_any_ptr = true;
        }
        // The kit fronts every pointer-facing export (helpers + null/abort
        // paths used by bodies); once per product, ahead of this wrapper.
        if has_any_ptr {
            self.emit_cabi_kit(sink)?;
        }

        // Return-position ABI type + wrapping of the body call. Handle fns
        // wrap the call in a null-safe match (None → -1 sentinel for int
        // returns, no-op for void — uniform simplification of the engine's
        // per-fn -1/-2 null sentinels, documented in the corpus).
        let handle = handle_param.as_deref();
        let body_call = |args: &str| format!("super::{}({})", fn_decl.name, args);
        let (abi_ret, wrap_call) = match &fn_decl.ret {
            Type::Void => match handle {
                Some(h) => (
                    None,
                    format!(
                        "match unsafe {{ {h}.as_mut() }} {{ Some({h}) => {call}, None => () }}",
                        h = h,
                        call = body_call(&call_args.join(", "))
                    ),
                ),
                None => (None, body_call(&call_args.join(", "))),
            },
            Type::Int => match handle {
                Some(h) => (
                    Some("i32".to_string()),
                    format!(
                        "match unsafe {{ {h}.as_mut() }} {{ Some({h}) => {call} as i32, None => -1 }}",
                        h = h,
                        call = body_call(&call_args.join(", "))
                    ),
                ),
                None => (
                    Some("i32".to_string()),
                    format!("{} as i32", body_call(&call_args.join(", "))),
                ),
            },
            // Ownership passes to the C caller (free side = C); interior NUL
            // degrades to "" instead of panicking across the boundary.
            Type::CStrLit => (
                Some("*mut std::os::raw::c_char".to_string()),
                format!(
                    "std::ffi::CString::new({}).unwrap_or_default().into_raw()",
                    body_call(&call_args.join(", "))
                ),
            ),
            Type::Ptr(_)
            | Type::I64
            | Type::Uint
            | Type::U64
            | Type::USize
            | Type::Byte
            | Type::Bool
            | Type::Char
            | Type::Float
            | Type::Double => (
                Some(self.rust_type_name(&fn_decl.ret)),
                body_call(&call_args.join(", ")),
            ),
            other => {
                return Err(crate::AutoError::Msg(format!(
                    "Plan 610 ⑤: #[export] 返回类型 {} 暂不在 ABI 面（支持 int/i64/uint/u64/usize/byte/bool/char/float/cstr/*T/void）",
                    other
                )));
            }
        };

        writeln!(
            sink.body,
            "/// Plan 610 ⑤: C ABI export face for `{}` — the sibling wrapper bridges C widths\n/// to the safe body below; the exported symbol equals the fn name (no_mangle\n/// ignores the module path).",
            fn_decl.name
        )?;
        writeln!(sink.body, "mod {}_c_export {{", fn_decl.name)?;
        // Bring the outer scope in (handle pointee types like the session
        // struct live outside this module).
        writeln!(sink.body, "    use super::*;")?;
        writeln!(sink.body, "    #[unsafe(no_mangle)]")?;
        write!(
            sink.body,
            "    pub extern \"{}\" fn {}({})",
            abi,
            fn_decl.name,
            abi_params.join(", ")
        )?;
        match &abi_ret {
            Some(t) => writeln!(sink.body, " -> {} {{", t)?,
            None => writeln!(sink.body, " {{")?,
        }
        for line in &conv_prelude {
            writeln!(sink.body, "        {}", line)?;
        }
        writeln!(sink.body, "        {}", wrap_call)?;
        writeln!(sink.body, "    }}")?;
        writeln!(sink.body, "}}")?;
        Ok(())
    }

    // For loop
    pub(super) fn for_stmt(&mut self, for_stmt: &For, sink: &mut Sink) -> AutoResult<()> {
        match &for_stmt.iter {
            Iter::Named(name) => {
                // PLAN-724 T-06：HTTP 流 for-in（`for c in s.iter()` /
                // `for c in stream_iter(s)` / 直接生产调用）。EOF 哨兵 ""
                // 与 VM 手工接口契约一致；局部 break 不冒充关闭（消费者变量
                // 仍归用户所有，显式 close 或作用域 Drop 回收）。
                if let Some(stream_is_async) = for_range_is_http_stream(self, &for_stmt.range) {
                    let var = name.as_str();
                    self.local_var_types.insert(name.clone(), Type::StrOwned);
                    self.current_fn_str_params.insert(name.clone());
                    self.print_indent(&mut sink.body)?;
                    // 生产调用形态 → 临时拥有绑定；变量借用形态 → 直接引用。
                    let owned_temp = !matches!(&for_stmt.range,
                        Expr::Call(c) if matches!(c.name.as_ref(),
                            Expr::Dot(_, m) if m.as_str() == "iter")
                            || matches!(c.name.as_ref(), Expr::Ident(f) if f.as_str() == "stream_iter"));
                    let mut recv_expr = String::new();
                    if owned_temp {
                        write!(sink.body, "let __hs = ")?;
                        self.expr(&for_stmt.range, &mut sink.body)?;
                        sink.body.write(
                            b";
",
                        )?;
                        recv_expr = "__hs".to_string();
                    } else if let Expr::Call(c) = &for_stmt.range {
                        if let Expr::Dot(obj, _) = c.name.as_ref() {
                            let mut buf = std::io::Cursor::new(Vec::new());
                            self.expr(obj, &mut buf)?;
                            recv_expr = String::from_utf8(buf.into_inner()).unwrap_or_default();
                        } else if let Expr::Ident(_) = c.name.as_ref() {
                            if let Some(Arg::Pos(e)) = c.args.args.first() {
                                let mut buf = std::io::Cursor::new(Vec::new());
                                self.expr(e, &mut buf)?;
                                recv_expr = String::from_utf8(buf.into_inner()).unwrap_or_default();
                            }
                        }
                    }
                    sink.body.write(
                        b"loop {
",
                    )?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    // PLAN-738 T-11：for-in 流反糖的 stream_next 发射是核心
                    // producer 的真实引用——按 stream_next 契约验证（async 面
                    // 契约 stream_next_async/同步面走同一 strict 门）。
                    if stream_is_async {
                        write!(
                            sink.body,
                            "let {var} = a2r_std::http::stream_next_async(&{recv_expr}).await;
"
                        )?;
                        if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                            "http",
                            "stream_next",
                            &format!("a2r_std::http::stream_next_async(&{recv_expr}).await"),
                            self.assembly_runtime,
                        )? {
                            self.assembly_references.push(proof);
                        }
                    } else {
                        write!(
                            sink.body,
                            "let {var} = a2r_std::http::stream_next(&{recv_expr});
"
                        )?;
                        if let Some(proof) = crate::stdlib_assembly::host::verify_rust_reference(
                            "http",
                            "stream_next",
                            &format!("a2r_std::http::stream_next(&{recv_expr})"),
                            self.assembly_runtime,
                        )? {
                            self.assembly_references.push(proof);
                        }
                    }
                    self.print_indent(&mut sink.body)?;
                    write!(
                        sink.body,
                        "if {var}.is_empty() {{ break; }}
"
                    )?;
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                    return Ok(());
                }
                // Plan 364 Phase 8 F1: if the iterable is a ~Stream<T> generator
                // call, emit `while let Some(x) = s.next().await` instead of a
                // `for` loop — `impl futures::Stream` does not implement
                // `IntoIterator`, so a plain `for` won't compile.
                if self.iterable_is_stream(&for_stmt.range) {
                    // let mut __s = <range>;
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "let mut __s = ")?;
                    self.expr(&for_stmt.range, &mut sink.body)?;
                    sink.body.write(b";\n")?;
                    // tokio::pin!(__s);
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"tokio::pin!(__s);\n")?;
                    // while let Some(<name>) = __s.next().await { ... }
                    // Fully-qualified futures::StreamExt::next avoids needing a
                    // `use futures::StreamExt;` import in the generated file.
                    self.print_indent(&mut sink.body)?;
                    write!(
                        sink.body,
                        "while let Some({}) = futures::StreamExt::next(&mut __s).await {{\n",
                        name
                    )?;
                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                    return Ok(());
                }

                sink.body.write(b"for ")?;
                sink.body.write(name.as_bytes())?;
                sink.body.write(b" in ")?;

                // Check if it's a range or array iteration
                if let Expr::Range(range) = &for_stmt.range {
                    // Range iteration: for x in start..end
                    self.expr(&range.start, &mut sink.body)?;
                    sink.body.write(b"..")?;
                    self.expr(&range.end, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    // Body
                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                } else {
                    // Array iteration: for x in arr.
                    // Plan 016 Phase A A.4: bind the loop variable's type into
                    // local_var_types so downstream codegen can make correct
                    // borrow/coercion decisions (e.g. .as_str() skipping for
                    // &str loop vars from .split()). Key iterable patterns:
                    //   X.split(...) → items are &str
                    //   X.lines() / X.chars() → items are &str / char
                    // We only bind StrSlice for split/lines (the common .at
                    // pattern); other iterables leave the var untyped (the
                    // pre-existing conservative behavior).
                    let iter_yields_str = match &for_stmt.range {
                        Expr::Call(c) => {
                            if let Expr::Dot(_, m) = c.name.as_ref() {
                                matches!(m.as_str(), "split" | "lines" | "split_whitespace")
                            } else {
                                false
                            }
                        }
                        // split().collect::<Vec<_>>() wrapper
                        Expr::Index(inner, _) => {
                            if let Expr::Call(c) = inner.as_ref() {
                                if let Expr::Dot(_, m) = c.name.as_ref() {
                                    matches!(m.as_str(), "split" | "lines" | "split_whitespace")
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    if iter_yields_str {
                        self.local_var_types.insert(name.clone(), Type::StrSlice);
                        // Also register in current_fn_str_params so auto-borrow
                        // logic treats it as &str (skip .as_str()/.to_string()).
                        self.current_fn_str_params.insert(name.clone());
                    }
                    // Borrow collection by reference to avoid moving it:
                    //   `for x in self.sections` → `for x in &self.sections`
                    //   `for x in some_vec`      → `for x in &some_vec`
                    // This mirrors the Destructured branch. Method calls
                    // (e.g. `.clone()`, iterator-yielding fns) stay un-borrowed.
                    let is_borrowable = matches!(&for_stmt.range, Expr::Ident(_) | Expr::Dot(_, _))
                        && !matches!(&for_stmt.range, Expr::Ident(n)
                        if self.by_value_iter_bindings.contains(n));
                    if is_borrowable {
                        sink.body.write(b"&")?;
                        // Plan 399 Phase 11.4: record this loop var as borrowed
                        // (&T) so struct-field reads `x.field` of non-Copy types
                        // can emit .clone() (avoiding E0507 move out of shared ref).
                        self.borrowed_iter_vars.insert(name.clone());
                        // Plan 396 §2.1: also resolve the loop var's ELEMENT
                        // type (`for tc in result.tool_calls` with
                        // tool_calls: List<ToolCallRec> → tc: ToolCallRec) so
                        // later field-type queries (clone decisions in call
                        // args / struct literals) resolve instead of Unknown.
                        if let Some(elem_ty) = self.iter_elem_type(&for_stmt.range) {
                            // Only register NON-str element types: str-yielding
                            // loops have their own mechanism above (StrSlice +
                            // current_fn_str_params), and registering StrOwned
                            // here would flip bare  coercion to explicit
                            // .as_str() at unrelated sites.
                            let is_str = matches!(
                                elem_ty,
                                Type::StrFixed(_) | Type::StrOwned | Type::CStrLit | Type::StrSlice
                            );
                            if !is_str {
                                self.local_var_types.insert(name.clone(), elem_ty);
                            }
                        }
                        // PLAN-681 T-05（N2/P1b）：bare List 迭代元素=动态记录
                        // 桶——循环变量登记 Value 占位（str 例外逻辑在前,
                        // 已登记者不受影响；字段读经 P2 走 name["field"]）。
                        if matches!(
                            self.local_var_types.get(name.as_str()),
                            None | Some(Type::Unknown)
                        ) && matches!(&for_stmt.range, Expr::Ident(_)
                            if self.receiver_elem_is_json_value(&for_stmt.range))
                        {
                            self.local_var_types
                                .insert(name.clone(), Self::placeholder_user_type("Value".into()));
                        }
                    }
                    self.expr(&for_stmt.range, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    // Body
                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
            Iter::Destructured(key, val) => {
                // for (k, v) in <expr>
                // - If <expr> is a collection (map/variable/field), iterate by
                //   reference: `for (k, v) in &map` (Rust idiom for HashMap).
                // - If <expr> is an iterator-yielding method call (e.g.
                //   `node.kids_iter()`, `tr_node.props_iter()`), the call already
                //   returns an iterator — borrowing it with `&` would try to
                //   iterate a reference to a temporary, which doesn't compile.
                //   Emit `for (k, v) in node.kids_iter()` with no `&`.
                //   (Plan 013 B1/BUG4.)
                let is_iter_call = matches!(&for_stmt.range, Expr::Call(_))
                    || matches!(&for_stmt.range, Expr::Dot(_, _));
                sink.body.write(b"for (")?;
                sink.body.write(key.as_bytes())?;
                sink.body.write(b", ")?;
                sink.body.write(val.as_bytes())?;
                if is_iter_call {
                    sink.body.write(b") in ")?;
                } else {
                    sink.body.write(b") in &")?;
                }
                self.expr(&for_stmt.range, &mut sink.body)?;
                sink.body.write(b" {\n")?;
                self.indent();
                self.emit_loop_body(&for_stmt.body, sink)?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            Iter::Ever => {
                // Infinite loop: loop { body }
                sink.body.write(b"loop {\n")?;
                self.indent();
                self.emit_loop_body(&for_stmt.body, sink)?;
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            Iter::Cond => {
                // Conditional loop: while condition { ... }
                // Optimize: for true { ... } -> loop { ... }
                if let Expr::Bool(true) = &for_stmt.range {
                    sink.body.write(b"loop {\n")?;
                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                } else {
                    // Check if there's an init statement
                    if let Some(init_stmt) = &for_stmt.init {
                        // Emit init statement before the loop
                        match &**init_stmt {
                            Stmt::Store(store) => {
                                self.store(store, &mut sink.body)?;
                                sink.body.write(b";\n")?;
                            }
                            _ => {
                                self.stmt(&**init_stmt, sink)?;
                                sink.body.write(b"\n")?;
                            }
                        }
                    }

                    sink.body.write(b"while ")?;
                    self.expr(&for_stmt.range, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    self.indent();
                    self.emit_loop_body(&for_stmt.body, sink)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Inline for-loop for closures: outputs compact single-line for loop
    pub(super) fn for_stmt_inline(
        &mut self,
        for_stmt: &For,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        match &for_stmt.iter {
            Iter::Named(name) => {
                // Plan 364 Phase 8 F1: for-over-Stream in inline (closure) context.
                if self.iterable_is_stream(&for_stmt.range) {
                    write!(out, "{{ let mut __s = ")?;
                    self.expr(&for_stmt.range, out)?;
                    write!(out, "; tokio::pin!(__s); while let Some({}) = futures::StreamExt::next(&mut __s).await {{", name)?;
                    for stmt in &for_stmt.body.stmts {
                        match stmt {
                            Stmt::Expr(expr) => {
                                self.expr(expr, out)?;
                                write!(out, "; ")?;
                            }
                            Stmt::Store(store) => {
                                self.store(store, out)?;
                                write!(out, "; ")?;
                            }
                            _ => {}
                        }
                    }
                    write!(out, "}} }}")?;
                    return Ok(());
                }
                write!(out, "for {} in ", name)?;
                if let Expr::Range(range) = &for_stmt.range {
                    self.expr(&range.start, out)?;
                    write!(out, "..")?;
                    self.expr(&range.end, out)?;
                } else {
                    self.expr(&for_stmt.range, out)?;
                }
                write!(out, " {{ ")?;
                for stmt in &for_stmt.body.stmts {
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                            write!(out, "; ")?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                            write!(out, "; ")?;
                        }
                        _ => {}
                    }
                }
                write!(out, "}}")?;
            }
            Iter::Cond => {
                write!(out, "while ")?;
                self.expr(&for_stmt.range, out)?;
                write!(out, " {{ ")?;
                for stmt in &for_stmt.body.stmts {
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                            write!(out, "; ")?;
                        }
                        _ => {}
                    }
                }
                write!(out, "}}")?;
            }
            _ => {
                write!(out, "/* unsupported for in closure */")?;
            }
        }
        Ok(())
    }

    // If statement
    pub(super) fn if_stmt(&mut self, if_: &If, sink: &mut Sink) -> AutoResult<()> {
        // If there's no else branch, the if block can't be used as an expression,
        // so all Call tail expressions need semicolons to avoid type mismatches.
        let has_else = if_.else_.is_some();

        // Plan 417-E4 调查: `value_ctx` = this if IS the fn-body tail value
        // (set by body()'s Stmt::If tail arm). In that case branch tail
        // expressions omit the `;` below (they carry the fn's return value);
        // nested ifs start in statement mode and are re-flagged only when
        // they are themselves an arm's last statement.
        let value_ctx = self.value_if_tail;
        self.value_if_tail = false;
        for (i, branch) in if_.branches.iter().enumerate() {
            if i == 0 {
                sink.body.write(b"if ")?;
            } else {
                sink.body.write(b" else if ")?;
            }

            self.expr(&branch.cond, &mut sink.body)?;
            sink.body.write(b" ")?;

            // Process branch body - use body() method for proper formatting
            sink.body.write(b"{\n")?;
            self.indent();
            let stmt_count = branch.body.stmts.len();
            for (i, stmt) in branch.body.stmts.iter().enumerate() {
                sink.record();
                if i < branch.body.source_lines.len() {
                    sink.set_source_line(branch.body.source_lines[i]);
                }
                self.print_indent(&mut sink.body)?;
                let is_last = i == stmt_count - 1;
                match stmt {
                    Stmt::Expr(Expr::If(inner_if)) => {
                        // Nested if expression - handle recursively
                        self.expr(&Expr::If(inner_if.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        if is_last
                            && self.ret_type_needs_string_coercion()
                            && self.expr_needs_string_coercion(expr)
                        {
                            sink.body.write(b".to_string()")?;
                        }
                        if !is_last {
                            sink.body.write(b";\n")?;
                        } else if is_last && value_ctx && has_else {
                            // Plan 417-E4 调查: 值位尾 if 的分支尾表达式是
                            // 函数返回值——不加 `;`(加了则 fn 返回 unit,E0308)。
                            sink.body.write(b"\n")?;
                        } else {
                            // Plan 393 E3: 语句上下文 if 的分支尾表达式也需要 `;`
                            // (值被丢弃)。原逻辑仅在 !has_else && Call 时补 `;`,
                            // 遗漏 has_else 的情况,导致 m.insert(...) 等
                            // Option<V>-返回调用泄漏为分支尾类型 (E0308)。
                            sink.body.write(b";\n")?;
                        }
                    }
                    Stmt::If(inner_if) => {
                        // Nested if statement - handle recursively. When the
                        // nested if is this arm's LAST statement in a value
                        // context, it carries the arm's value — re-flag it.
                        let saved = self.value_if_tail;
                        self.value_if_tail = value_ctx && has_else && is_last;
                        self.if_stmt(inner_if, sink)?;
                        self.value_if_tail = saved;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::Break => {
                        sink.body.write(b"break;\n")?;
                    }
                    Stmt::Continue => {
                        sink.body.write(b"continue;\n")?;
                    }
                    Stmt::Return(ret) => {
                        self.write_return_expr(ret, &mut sink.body, true)?;
                        sink.body.write(b"\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        sink.body.write(
                            b"
",
                        )?;
                    }
                }
            }
            sink.record();
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}")?;
        }

        if let Some(else_body) = &if_.else_ {
            sink.body.write(b" else ")?;
            sink.body.write(b"{\n")?;
            self.indent();
            let stmt_count = else_body.stmts.len();
            for (i, stmt) in else_body.stmts.iter().enumerate() {
                sink.record();
                if i < else_body.source_lines.len() {
                    sink.set_source_line(else_body.source_lines[i]);
                }
                self.print_indent(&mut sink.body)?;
                let is_last = i == stmt_count - 1;
                match stmt {
                    Stmt::Expr(Expr::If(inner_if)) => {
                        // Nested if expression in else
                        self.expr(&Expr::If(inner_if.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        if is_last
                            && self.ret_type_needs_string_coercion()
                            && self.expr_needs_string_coercion(expr)
                        {
                            sink.body.write(b".to_string()")?;
                        }
                        // Plan 393 E3: else 分支尾表达式也一律 `;` (语句上下文)
                        // Plan 417-E4 调查: 值位尾 if 的 else 分支尾表达式是
                        // 函数返回值——不加 `;`。
                        if is_last && value_ctx && has_else {
                            sink.body.write(b"\n")?;
                        } else {
                            sink.body.write(b";\n")?;
                        }
                    }
                    Stmt::If(inner_if) => {
                        // Nested if statement in else — re-flag when it is the
                        // else tail in a value context (carries the value).
                        let saved = self.value_if_tail;
                        self.value_if_tail = value_ctx && has_else && is_last;
                        self.if_stmt(inner_if, sink)?;
                        self.value_if_tail = saved;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::Break => {
                        sink.body.write(b"break;\n")?;
                    }
                    Stmt::Return(ret) => {
                        sink.body.write(b"return ")?;
                        self.expr(ret, &mut sink.body)?;
                        // Plan 433 A1: mirror stmt()'s Return arm — a
                        // String-returning fn returning a &str expr in an
                        // else branch needs the .to_string() coercion.
                        if self.ret_type_needs_string_coercion()
                            && self.expr_needs_string_coercion(ret)
                        {
                            sink.body.write(b".to_string()")?;
                        }
                        sink.body.write(b";\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        sink.body.write(
                            b"
",
                        )?;
                    }
                }
            }
            sink.record();
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }

        Ok(())
    }

    pub(super) fn is_stmt(&mut self, is_stmt: &Is, sink: &mut Sink) -> AutoResult<()> {
        // Plan 016 Phase A A.4: detect `is json.parse(x) { Some/None ... }` —
        // Auto's json.parse returns ?JsonValue (Option), but a2r_std::json::parse
        // returns Value. Use parse_opt (which returns Option<Value>) when the
        // scrutinee is json.parse AND branches include Some/None patterns.
        let is_jp = self.is_json_parse_scrutinee(&is_stmt.target);
        let has_some_none = is_stmt.branches.iter().any(|b| {
            matches!(b, IsBranch::EqBranch(patterns, _)
            if patterns.iter().any(|p| match p {
                Expr::Ident(n) => n.as_str() == "Some" || n.as_str() == "None",
                Expr::Call(c) => matches!(c.name.as_ref(), Expr::Ident(n) if n.as_str() == "Some"),
                // Auto's Some(x)/None patterns are Expr::OptionPattern
                Expr::OptionPattern(_) => true,
                _ => false,
            }))
        });
        let parse_as_opt = is_jp && has_some_none;
        let prev = self.json_parse_as_opt;
        self.json_parse_as_opt = parse_as_opt;

        // Plan 447 H4: complex scrutinee + guard arms — hoist to a temp local
        // first (`let __is_N = expr; match __is_N { ... }`) so the scrutinee
        // evaluates once and guard arms can bind it (`__is_N if <cond> =>`).
        // Only when IfBranch arms exist: goldens cover zero guard arms, so
        // this never touches existing emission.
        let has_guard_arm = is_stmt
            .branches
            .iter()
            .any(|b| matches!(b, IsBranch::IfBranch(_, _)));
        let hoisted_scrutinee: Option<AutoStr> =
            if has_guard_arm && !matches!(is_stmt.target, Expr::Ident(_)) {
                let name: AutoStr = format!("__is_{}", self.is_hoist_counter).into();
                self.is_hoist_counter += 1;
                sink.body.write(b"let ")?;
                sink.body.write(name.as_bytes())?;
                sink.body.write(b" = ")?;
                self.expr(&is_stmt.target, &mut sink.body)?;
                sink.body.write(b";\n")?;
                self.print_indent(&mut sink.body)?;
                Some(name)
            } else {
                None
            };

        sink.body.write(b"match ")?;

        // Check if any arm pattern is a string literal — if so, match on &str
        // (computed BEFORE the H5 `&`: `.as_str()` already borrows, prefixing
        // `&` would double-reference — `match &text.as_str()` is E0308)
        let has_str_pattern = is_stmt.branches.iter().any(|branch| {
            if let IsBranch::EqBranch(patterns, _) = branch {
                patterns
                    .iter()
                    .any(|p| matches!(p, Expr::Str(_) | Expr::CStr(_)))
            } else {
                false
            }
        });

        if let Some(name) = &hoisted_scrutinee {
            sink.body.write(name.as_bytes())?;
        } else {
            // Plan 447 H5: Fix double match E0382 — `match &v` only when the same
            // identifier scrutinee is matched >= 2 times in this function (first
            // by-value match would move it). Single-use keeps plain `match v` so
            // arm payload bindings stay owned (and goldens stay stable).
            // str-pattern arms borrow via .as_str() already — skip the `&`.
            let needs_ref_match = match &is_stmt.target {
                Expr::Ident(name) if !has_str_pattern => {
                    self.fn_is_scrutinee_counts
                        .get(name.as_str())
                        .copied()
                        .unwrap_or(0)
                        >= 2
                }
                _ => false,
            };

            if needs_ref_match {
                sink.body.write(b"&")?;
            }

            // Check if scrutinee is self.field (needs .clone() in &self methods)
            let is_self_field = Self::is_self_dot(&is_stmt.target);

            if has_str_pattern {
                // Use match target.as_str() to allow &str patterns against String —
                // but NOT when the target is already `&str` (a str param / StrSlice
                // local): `.as_str()` on `&str` is E0658 (str_as_str, unstable).
                // e.g. `match name.as_str()` where `name: &str` (load_builtin).
                self.expr(&is_stmt.target, &mut sink.body)?;
                let target_is_str = match &is_stmt.target {
                    Expr::Ident(name) => {
                        self.current_fn_str_params.contains(name)
                            || self
                                .local_var_types
                                .get(name)
                                .map(|t| matches!(t, Type::StrSlice))
                                .unwrap_or(false)
                    }
                    _ => false,
                };
                if !target_is_str {
                    sink.body.write(b".as_str()")?;
                }
            } else if is_self_field {
                // self.field needs .clone() to avoid move in &self methods
                self.expr(&is_stmt.target, &mut sink.body)?;
                sink.body.write(b".clone()")?;
            } else {
                self.expr(&is_stmt.target, &mut sink.body)?;
            }
        }
        sink.body.write(b" {\n")?;
        self.indent();

        // Plan 032 G2-core: statement-position is with MIXED arm shapes —
        // one arm a multi-stmt block (unit type), another a bare
        // value-bearing expression (e.g. `map.insert(...)` → Option) — is
        // E0308 (arms have incompatible types). The match's value is
        // discarded here (the dispatcher appends `;`), so wrap single-Expr
        // arms in `{ ...; }` when any arm is a block.
        let has_block_arm = is_stmt.branches.iter().any(|b| match b {
            IsBranch::EqBranch(_, body)
            | IsBranch::IfBranch(_, body)
            | IsBranch::ElseBranch(body) => body.stmts.len() > 1,
        });

        for branch in &is_stmt.branches {
            self.print_indent(&mut sink.body)?;

            match branch {
                IsBranch::EqBranch(patterns, body) => {
                    // Multi-pattern: 1 | 2 | 3 => ...
                    // Plan 447 ③-前置: int-typed scrutinee — char literal
                    // patterns emit as code-point decimals; scoped to the
                    // pattern list only (bodies keep the D37 char bridge).
                    let saved_int_scrut = self.int_match_scrutinee;
                    self.int_match_scrutinee = matches!(
                        self.infer_type_from_expr(&is_stmt.target),
                        Type::Int | Type::Uint | Type::I64 | Type::U64 | Type::USize
                    );
                    for (i, pat) in patterns.iter().enumerate() {
                        if i > 0 {
                            sink.body.write(b" | ")?;
                        }
                        // In match patterns, Some(ident) binds by value (Auto semantics)
                        if let Expr::Some(inner) = pat {
                            sink.body.write(b"Some(")?;
                            // Plan 380: `Some(x)` binding from an &str-returning
                            // scrutinee (strip_prefix / to_str / …) is `&str` —
                            // record so call-site auto-borrow doesn't append
                            // `.as_str()` (E0658 str_as_str).
                            if let Expr::Ident(binding) = inner.as_ref() {
                                if Self::is_str_returning_scrutinee(&is_stmt.target) {
                                    self.local_var_types.insert(binding.clone(), Type::StrSlice);
                                    // Plan 427: the precise &str marker consumed
                                    // by is_str_slice_var (see its comment).
                                    self.str_slice_pattern_bindings.insert(binding.clone());
                                }
                                // Plan 380: `Some(x)` from an Option<Spec>
                                // scrutinee binds a Box<dyn Trait> — record so
                                // call-site auto-clone skips it (E0599).
                                if self.is_spec_returning_scrutinee(&is_stmt.target) {
                                    self.spec_bound_idents.insert(binding.clone());
                                }
                            }
                            self.expr(inner, &mut sink.body)?;
                            sink.body.write(b")")?;
                        } else if let Expr::Call(call) = pat {
                            if let Expr::Ident(name) = call.name.as_ref() {
                                if name == "Some" && !call.args.args.is_empty() {
                                    sink.body.write(b"Some(")?;
                                    if let Some(Arg::Pos(inner)) = call.args.args.first() {
                                        self.expr(inner, &mut sink.body)?;
                                    }
                                    if let Some(Arg::Pos(inner)) = call.args.args.first() {
                                        self.expr(inner, &mut sink.body)?;
                                    }
                                    sink.body.write(b")")?;
                                } else {
                                    self.expr(pat, &mut sink.body)?;
                                }
                            } else {
                                // Plan 013 (B16): a variant pattern like
                                // `auto_val.Kid.Node(child)` binds `child` to a
                                // Box<T> in Rust. Record bare-ident args so the
                                // call-site auto-clone can deref before cloning.
                                for arg in &call.args.args {
                                    if let Arg::Pos(Expr::Ident(name)) = arg {
                                        self.bridge_pattern_bound_idents.insert(name.clone());
                                    }
                                }
                                // Plan 447 H6 (DEBT-113): `Enum.Variant(bound)`
                                // call-shaped payload patterns — backfill
                                // local_var_types from enum_tuple_field_types.
                                if let crate::ast::Expr::Bina(l, op, r) = call.name.as_ref() {
                                    if let Op::Dot = op {
                                        if let (Expr::Ident(kind), Expr::Ident(tag)) =
                                            (l.as_ref(), r.as_ref())
                                        {
                                            if let Some(field_types) = self
                                                .enum_tuple_field_types
                                                .get(&(kind.clone(), tag.clone()))
                                            {
                                                for (arg, ty) in
                                                    call.args.args.iter().zip(field_types.iter())
                                                {
                                                    if let Arg::Pos(Expr::Ident(name)) = arg {
                                                        if name.as_str() != "_" {
                                                            self.local_var_types
                                                                .insert(name.clone(), ty.clone());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                self.expr(pat, &mut sink.body)?;
                            }
                        } else if let Expr::OptionPattern(oc) = pat {
                            // Some(text) / None parsed as OptionPattern in is branches
                            match oc.variant {
                                crate::ast::cover::OptionVariant::Some => {
                                    sink.body.write(b"Some(")?;
                                    if let Some(binding) = &oc.binding {
                                        sink.body.write(binding.as_bytes())?;
                                        // Plan 380: a `Some(x)` binding from an
                                        // &str-returning scrutinee (strip_prefix /
                                        // to_str / …) is `&str` — record it so the
                                        // call-site auto-borrow doesn't append
                                        // `.as_str()` (E0658 str_as_str).
                                        if Self::is_str_returning_scrutinee(&is_stmt.target) {
                                            self.local_var_types
                                                .insert(binding.clone(), Type::StrSlice);
                                            // Plan 427: the precise &str marker
                                            // consumed by is_str_slice_var.
                                            self.str_slice_pattern_bindings.insert(binding.clone());
                                        }
                                        // Plan 380: Option<Spec> scrutinee → the
                                        // binding is a Box<dyn Trait>; skip
                                        // call-site auto-clone (E0599).
                                        if self.is_spec_returning_scrutinee(&is_stmt.target) {
                                            self.spec_bound_idents.insert(binding.clone());
                                        }
                                    }
                                    sink.body.write(b")")?;
                                }
                                crate::ast::cover::OptionVariant::None => {
                                    self.expr(pat, &mut sink.body)?;
                                }
                            }
                        } else if let Expr::ResultPattern(cover) = pat {
                            // Plan 396 §2.2: `Ok(x)` from a by-value-iterator
                            // scrutinee (fs.read_dir) binds x as ReadDir —
                            // later `for x in ...` must NOT borrow (&ReadDir is
                            // not an iterator, E0277).
                            if let Some(binding) = &cover.binding {
                                if Self::is_by_value_iter_scrutinee(&is_stmt.target) {
                                    self.by_value_iter_bindings.insert(binding.clone());
                                }
                            }
                            self.expr(pat, &mut sink.body)?;
                        } else {
                            // Plan 013 (B16): record bare-ident args in any
                            // remaining pattern shape (e.g. a Dot-chain variant
                            // `auto_val.Kid.Node(child)`), so call-site auto-
                            // clone can deref Box<T> before cloning.
                            // Plan 447 H6 (DEBT-113): enum-payload Cover::Tag
                            // patterns — backfill local_var_types for arm
                            // bindings from enum_tuple_field_types.
                            if let Expr::Cover(crate::ast::Cover::Tag(tc)) = pat {
                                if let Some(field_types) = self
                                    .enum_tuple_field_types
                                    .get(&(tc.kind.clone(), tc.tag.clone()))
                                {
                                    for (b, ty) in tc.bindings.iter().zip(field_types.iter()) {
                                        if b.as_str() != "_" {
                                            self.local_var_types.insert(b.clone(), ty.clone());
                                        }
                                    }
                                }
                            }
                            self.expr(pat, &mut sink.body)?;
                        }
                    }
                    self.int_match_scrutinee = saved_int_scrut;
                    sink.body.write(b" => ")?;
                    // Plan 032 G2-core: register this arm's Some(x) bindings
                    // when the scrutinee is a `.get()` (binding is `&V`), so
                    // the body's return/let-init emitters can auto-clone.
                    // Scoped to this arm only (restored after the body).
                    let saved_get_refs = self.get_ref_bindings.clone();
                    if Self::is_borrowing_get_scrutinee(&is_stmt.target) {
                        self.register_get_ref_bindings(patterns);
                    }
                    self.write_is_arm_body(body, sink, has_block_arm)?;
                    self.get_ref_bindings = saved_get_refs;
                    sink.body.write(b",\n")?;
                }
                IsBranch::IfBranch(expr, body) => {
                    // Plan 447 H4: 卫语句臂生成 `<绑定> if <cond> =>`
                    // — hoisted scrutinee binds __is_N, plain ident uses _guard.
                    if let Some(name) = &hoisted_scrutinee {
                        sink.body.write(name.as_bytes())?;
                    } else {
                        sink.body.write(b"_guard")?;
                    }
                    sink.body.write(b" if ")?;
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b" => ")?;
                    self.write_is_arm_body(body, sink, has_block_arm)?;
                    sink.body.write(b",\n")?;
                }
                IsBranch::ElseBranch(body) => {
                    sink.body.write(b"_ => ")?;
                    self.write_is_arm_body(body, sink, has_block_arm)?;
                    sink.body.write(b",\n")?;
                }
            }
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        self.json_parse_as_opt = prev;
        Ok(())
    }

    // Use statement
    /// Plan 417-D2: single-file transpiles have no project module discovery,
    /// so `use auto.<mod>: f, g` leaves the imported fns' signatures unknown —
    /// for-in generator rewrites (`iterable_is_stream`) and the String→&str
    /// call-site coercion both key off `fn_ret_types`/`fn_str_param_indices`
    /// and missed imported names (the parity string_utils / generators a2r
    /// builds failed on exactly this). When the module source is locatable
    /// relative to the working directory (`./auto/<mod>.at` — the parity lib
    /// layout — or `./<mod>.at`), parse it and register the imported items'
    /// fn signatures. Best-effort: missing files leave behavior unchanged.
    pub(super) fn register_import_signatures(&mut self, use_stmt: &Use) {
        // Only structured `use auto.<mod>: ...` / bare `<mod>` imports.
        let mod_name: String = if use_stmt.paths.first().map(|s| s.as_str()) == Some("auto") {
            match use_stmt.paths.get(1) {
                Some(m) => m.as_str().to_string(),
                None => return,
            }
        } else if use_stmt.paths.len() == 1 && !use_stmt.is_wildcard {
            use_stmt.paths[0].as_str().to_string()
        } else {
            return;
        };
        if mod_name.is_empty() || mod_name.contains("::") {
            return;
        }
        let cwd = match std::env::current_dir() {
            Ok(d) => d,
            Err(_) => return,
        };
        // PLAN-885 G2: prefer the transpiled file's own directory (back/
        // modules resolve sibling imports like `use works: …`); fall back to
        // CWD for legacy single-file entry points.
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Some(dir) = self.source_dir.clone() {
            candidates.push(dir.join(format!("{}.at", mod_name)));
            candidates.push(dir.join("auto").join(format!("{}.at", mod_name)));
        }
        candidates.push(cwd.join("auto").join(format!("{}.at", mod_name)));
        candidates.push(cwd.join(format!("{}.at", mod_name)));
        let source = candidates
            .iter()
            .find_map(|p| std::fs::read_to_string(p).ok());
        let source = match source {
            Some(s) => s,
            None => return,
        };
        let mut parser = crate::parser::Parser::new(&source);
        let code = match parser.parse() {
            Ok(c) => c,
            Err(_) => return,
        };
        for stmt in &code.stmts {
            if let crate::ast::Stmt::Fn(fn_decl) = stmt {
                let wanted = use_stmt.is_wildcard
                    || use_stmt
                        .items
                        .iter()
                        .any(|i| i.as_str() == fn_decl.name.as_str());
                if !wanted {
                    continue;
                }
                let str_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| {
                        matches!(
                            p.ty,
                            Type::StrFixed(_) | Type::StrSlice | Type::StrOwned | Type::CStrLit
                        )
                    })
                    .collect();
                if str_flags.iter().any(|f| *f) {
                    self.fn_str_param_indices
                        .insert(fn_decl.name.clone(), str_flags);
                }
                self.fn_ret_types
                    .insert(fn_decl.name.clone(), fn_decl.ret.clone());
                // Plan 016 Phase 4: register the FULL signature, not just
                // str/ret — the Dot-arg clone decision (param takes ownership)
                // and the defect-4 borrowed-iter-field gate key off
                // fn_param_types, which stayed None for imported fns.
                let param_types: Vec<Type> = fn_decl.params.iter().map(|p| p.ty.clone()).collect();
                self.fn_param_types
                    .insert(fn_decl.name.clone(), param_types);
                let struct_param_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| !Self::is_copy_type(&p.ty))
                    .collect();
                self.fn_struct_param_indices
                    .insert(fn_decl.name.clone(), struct_param_flags);
                let int_param_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| matches!(p.ty, Type::Int))
                    .collect();
                self.fn_int_param_indices
                    .insert(fn_decl.name.clone(), int_param_flags);
                let mut_param_flags: Vec<bool> = fn_decl
                    .params
                    .iter()
                    .map(|p| p.mode == crate::ast::ParamMode::Mut)
                    .collect();
                self.fn_mut_params
                    .insert(fn_decl.name.clone(), mut_param_flags);
            }
            // Plan 016 Phase 4: imported STRUCT field types — the call-site
            // str-field borrow (`node.id` → `node.id.as_str()`, Plan 433 A1)
            // and iter-var field type resolution scan struct_field_types,
            // which stayed empty for types from another module (E0308).
            if let crate::ast::Stmt::TypeDecl(td) = &stmt {
                let wanted = use_stmt.is_wildcard
                    || use_stmt
                        .items
                        .iter()
                        .any(|i| i.as_str() == td.name.as_str());
                if !wanted {
                    continue;
                }
                let fields: Vec<(AutoStr, Type)> = td
                    .members
                    .iter()
                    .map(|m| (m.name.clone(), m.ty.clone()))
                    .collect();
                if !fields.is_empty() {
                    self.struct_field_types.insert(td.name.clone(), fields);
                }
            }
            // Plan 016 Phase 4: imported ENUM variants — zero-payload variant
            // construction (`Value.Null()`) needs the payload-variant maps to
            // tell unit variants from tuple/struct ones (E0618), and String
            // payload literals need the enum-ctor `.to_string()` (E0308).
            if let crate::ast::Stmt::EnumDecl(ed) = &stmt {
                let wanted = use_stmt.is_wildcard
                    || use_stmt
                        .items
                        .iter()
                        .any(|i| i.as_str() == ed.name.as_str());
                if !wanted {
                    continue;
                }
                self.known_enum_names.insert(ed.name.clone());
                for item in &ed.items {
                    let item_key = (ed.name.clone(), item.name.clone());
                    if item.has_fields() {
                        let field_names: Vec<AutoStr> =
                            item.fields.iter().map(|f| f.name.clone()).collect();
                        self.enum_struct_variants.insert(item_key, field_names);
                    } else if item.has_tuple_payload() {
                        self.enum_tuple_variants
                            .insert(item_key.clone(), item.payload_types.len());
                        self.enum_tuple_field_types
                            .insert(item_key, item.payload_types.clone());
                    } else if let Some(ref payload) = item.payload_type {
                        self.enum_tuple_variants.insert(item_key.clone(), 1);
                        self.enum_tuple_field_types
                            .insert(item_key, vec![payload.clone()]);
                    }
                }
            }
        }
    }

    /// Plan 610 ⑥: lower `use.c <header>` to a generated FFI face (manifest
    /// shared IR — 004 §3.5 "one semantics, three lowerings"). S form
    /// (default): `#[link(name)] extern "<abi>"` block + safe wrappers; D
    /// form (manifest `"link": "dynamic"`): libloading with env
    /// `<LIBRARY>_DLL` → exe same-dir → bare-name resolution. All unsafe
    /// stays inside the generated module; Auto call sites use the bare
    /// manifest function names.
    pub(super) fn emit_use_c_ffi(
        &mut self,
        use_stmt: &Use,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        let raw = use_stmt
            .paths
            .first()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let clean = raw
            .trim_start_matches('<')
            .trim_end_matches('>')
            .trim_matches('"')
            .to_string();

        let manifest = if let Some(m) = crate::vm::ffi::c_ffi::load_builtin_manifest(&clean) {
            Some(m)
        } else if clean.ends_with(".json") {
            // Plan 610 ⑥: relative manifest paths resolve against the source
            // file's directory first, then CWD.
            let mut loaded = None;
            if let Some(dir) = self.source_dir.clone() {
                let p = dir.join(&clean);
                if p.is_file() {
                    loaded = crate::vm::ffi::c_ffi::load_manifest_file(&p.to_string_lossy());
                }
            }
            if loaded.is_none() {
                loaded = crate::vm::ffi::c_ffi::load_manifest_file(&clean);
            }
            loaded
        } else {
            None
        };
        let Some(manifest) = manifest else {
            // Unknown header — keep the pre-610 behavior (ignored import).
            return Ok(());
        };

        for f in &manifest.functions {
            let has_fnptr = matches!(
                f.return_type,
                auto_bindgen::manifest::CTypeDesc::FnPtr { .. }
            ) || f
                .params
                .iter()
                .any(|p| matches!(p.ty, auto_bindgen::manifest::CTypeDesc::FnPtr { .. }));
            if has_fnptr {
                return Err(crate::AutoError::Msg(format!(
                    "Plan 610 ⑥: use.c 函数 {} 含回调 FnPtr——暂不支持（trampoline 选型见计划 §5/T-10）",
                    f.name
                )));
            }
        }

        let suffix = if manifest.link == "dynamic" {
            "_dyn"
        } else {
            ""
        };
        let module = format!(
            "{}_c{}",
            manifest
                .header
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect::<String>(),
            suffix
        )
        .trim_matches('_')
        .to_string();
        let abi = if manifest.abi == "system" {
            "system"
        } else {
            "C"
        };

        writeln!(
            out,
            "/// Plan 610 ⑥: `use.c {}` → {} FFI face (manifest shared IR, auto-bindgen).",
            raw,
            if manifest.link == "dynamic" {
                "D-form dynamic"
            } else {
                "S-form static"
            }
        )?;
        writeln!(
            out,
            "/// Safe wrappers carry Auto-side widths (int = i64, cast at the ABI edge);
/// every unsafe op stays inside this module."
        )?;
        writeln!(out, "mod {} {{", module)?;

        let fns: Vec<&auto_bindgen::manifest::CFunction> =
            manifest.functions.iter().filter(|f| !f.variadic).collect();

        if manifest.link == "dynamic" {
            let lib_env = format!(
                "{}_DLL",
                manifest
                    .library
                    .to_uppercase()
                    .replace(|c: char| !c.is_ascii_alphanumeric(), "_")
            );
            writeln!(
                out,
                "    fn lib() -> &'static libloading::Library {{\n        static LIB: std::sync::OnceLock<libloading::Library> = std::sync::OnceLock::new();\n        LIB.get_or_init(|| unsafe {{\n            // Plan 610 ⑥ D form: env {env} → exe same-dir → bare name (PATH).\n            if let Ok(p) = std::env::var(\"{env}\") {{\n                if let Ok(l) = libloading::Library::new(&p) {{ return l; }}\n            }}\n            let file = if cfg!(windows) {{ \"{lib}.dll\".to_string() }} else {{ \"lib{lib}.so\".to_string() }};\n            if let Ok(exe) = std::env::current_exe() {{\n                if let Some(dir) = exe.parent() {{\n                    if let Ok(l) = libloading::Library::new(dir.join(&file)) {{ return l; }}\n                }}\n            }}\n            libloading::Library::new(&file).expect(\"failed to load library {lib} (set {env})\")\n        }})\n    }}",
                env = lib_env,
                lib = manifest.library
            )?;
        } else {
            writeln!(out, "    mod ffi_raw {{")?;
            writeln!(out, "        #[link(name = \"{}\")]", manifest.library)?;
            writeln!(out, "        extern \"{}\" {{", abi)?;
            for f in &fns {
                let params: Vec<String> = f
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, Self::c_abi_ffi_type(&p.ty)))
                    .collect();
                let ret = &f.return_type;
                if matches!(ret, auto_bindgen::manifest::CTypeDesc::Void) {
                    writeln!(out, "            pub fn {}({});", f.name, params.join(", "))?;
                } else {
                    writeln!(
                        out,
                        "            pub fn {}({}) -> {};",
                        f.name,
                        params.join(", "),
                        Self::c_abi_ffi_type(ret)
                    )?;
                }
            }
            writeln!(out, "        }}")?;
            writeln!(out, "    }}")?;
        }

        // Safe wrappers: one per manifest fn, Auto-side widths.
        for f in &fns {
            let mut sig: Vec<String> = Vec::new();
            let mut prelude: Vec<String> = Vec::new();
            let mut args: Vec<String> = Vec::new();
            for p in &f.params {
                match Self::c_abi_wrapper_param(&p.ty) {
                    Some(wt) => {
                        sig.push(format!("{}: {}", p.name, wt));
                        match &p.ty {
                            auto_bindgen::manifest::CTypeDesc::CStr => {
                                prelude.push(format!(
                                    "let {}_c = std::ffi::CString::new({}).unwrap_or_default();",
                                    p.name, p.name
                                ));
                                args.push(format!("{}_c.as_ptr()", p.name));
                            }
                            auto_bindgen::manifest::CTypeDesc::Int => {
                                args.push(format!("{} as i32", p.name));
                            }
                            auto_bindgen::manifest::CTypeDesc::Float => {
                                args.push(format!("{} as f32", p.name));
                            }
                            _ => args.push(p.name.clone()),
                        }
                    }
                    None => {
                        sig.push(format!("{}: {}", p.name, Self::c_abi_ffi_type(&p.ty)));
                        args.push(p.name.clone());
                    }
                }
            }
            let ret = &f.return_type;
            let ret_sig = match ret {
                auto_bindgen::manifest::CTypeDesc::Void => None,
                auto_bindgen::manifest::CTypeDesc::Int => Some("i64".to_string()),
                auto_bindgen::manifest::CTypeDesc::Float => Some("f64".to_string()),
                auto_bindgen::manifest::CTypeDesc::CStr => Some("String".to_string()),
                other => Some(Self::c_abi_ffi_type(other)),
            };
            let call_args = args.join(", ");
            let raw_call = if manifest.link == "dynamic" {
                format!(
                    "let sym: libloading::Symbol<unsafe extern \"{abi}\" fn({pt}) -> {rt}> = lib().get(b\"{name}\").expect(\"symbol {name} missing\");
        sym({ca})",
                    abi = abi,
                    pt = f
                        .params
                        .iter()
                        .map(|p| Self::c_abi_ffi_type(&p.ty))
                        .collect::<Vec<_>>()
                        .join(", "),
                    rt = Self::c_abi_ffi_type(ret),
                    name = f.name,
                    ca = call_args
                )
            } else {
                format!("ffi_raw::{}({})", f.name, call_args)
            };
            let body_expr = match ret {
                auto_bindgen::manifest::CTypeDesc::Void => {
                    format!("unsafe {{\n            {}\n        }}", raw_call)
                }
                auto_bindgen::manifest::CTypeDesc::Int => {
                    format!("(unsafe {{\n            {}\n        }}) as i64", raw_call)
                }
                auto_bindgen::manifest::CTypeDesc::Float => {
                    format!("(unsafe {{\n            {}\n        }}) as f64", raw_call)
                }
                auto_bindgen::manifest::CTypeDesc::CStr => format!(
                    "let r = unsafe {{\n            {}\n        }};
        if r.is_null() {{ String::new() }} else {{ unsafe {{ std::ffi::CStr::from_ptr(r) }}.to_string_lossy().into_owned() }}",
                    raw_call
                ),
                _ => format!("unsafe {{\n            {}\n        }}", raw_call),
            };
            match &ret_sig {
                Some(rt) => writeln!(
                    out,
                    "    pub fn {}({}) -> {} {{",
                    f.name,
                    sig.join(", "),
                    rt
                )?,
                None => writeln!(out, "    pub fn {}({}) {{", f.name, sig.join(", "))?,
            }
            for line in &prelude {
                writeln!(out, "        {}", line)?;
            }
            writeln!(out, "        {}", body_expr)?;
            writeln!(out, "    }}")?;
        }

        // Pointer/buffer helper kit — emitted when the face has any opaque
        // pointer params (out-param and handle shapes need them; pure-scalar
        // faces like stdio.h stay lean). Buffers leak by design (driver
        // scope, process lifetime).
        let has_ptr = manifest.functions.iter().any(|f| {
            f.params.iter().any(|p| {
                matches!(
                    p.ty,
                    auto_bindgen::manifest::CTypeDesc::Ptr
                        | auto_bindgen::manifest::CTypeDesc::PtrMut
                )
            })
        });
        if has_ptr {
            writeln!(
                out,
                "    // Plan 610 ⑥ helper kit: pointer/buffer access from safe Auto code."
            )?;
            writeln!(
                out,
                "    pub fn cffi_handle_is_null(h: *const std::os::raw::c_void) -> bool {{"
            )?;
            writeln!(out, "        h.is_null()")?;
            writeln!(out, "    }}")?;
            writeln!(
                out,
                "    pub fn cffi_buf_new_u32(len: i64) -> *mut std::os::raw::c_void {{"
            )?;
            writeln!(
                out,
                "        if len <= 0 {{ return std::ptr::null_mut(); }}"
            )?;
            writeln!(
                out,
                "        Box::into_raw(vec![0u32; len as usize].into_boxed_slice()) as *mut _"
            )?;
            writeln!(out, "    }}")?;
            writeln!(
                out,
                "    pub fn cffi_buf_get_u32(p: *const std::os::raw::c_void, idx: i64) -> u32 {{"
            )?;
            writeln!(out, "        if p.is_null() {{ return 0; }}")?;
            writeln!(
                out,
                "        unsafe {{ *(p as *const u32).add(idx as usize) }}"
            )?;
            writeln!(out, "    }}")?;
            writeln!(
                out,
                "    pub fn cffi_buf_new_u8(len: i64) -> *mut std::os::raw::c_void {{"
            )?;
            writeln!(
                out,
                "        if len <= 0 {{ return std::ptr::null_mut(); }}"
            )?;
            writeln!(
                out,
                "        Box::into_raw(vec![0u8; len as usize].into_boxed_slice()) as *mut _"
            )?;
            writeln!(out, "    }}")?;
            writeln!(
                out,
                "    pub fn cffi_cstr_read(p: *const std::os::raw::c_void) -> String {{"
            )?;
            writeln!(out, "        if p.is_null() {{ return String::new(); }}")?;
            writeln!(out, "        unsafe {{ std::ffi::CStr::from_ptr(p as *const std::os::raw::c_char) }}.to_string_lossy().into_owned()")?;
            writeln!(out, "    }}")?;
        }
        writeln!(out, "}}")?;
        writeln!(out, "pub use {}::*;", module)?;
        Ok(())
    }

    // Ext block (type extension) - transpiles to impl block
    pub(super) fn ext_decl(&mut self, ext: &Ext, sink: &mut Sink) -> AutoResult<()> {
        // Plan 364 W1: impl-level attribute macros (#[zbus::interface]) before `impl`
        for attr in &ext.attrs {
            write!(sink.body, "#[{}]\n", attr)?;
        }
        // Plan 164: Support "ext Type for Trait" → impl Trait for Type
        // Plan 6B-2.7: Support generic args on trait: ext Type for From<String> → impl From<String> for Type
        match &ext.trait_name {
            Some(trait_name) => {
                write!(sink.body, "impl {}", trait_name)?;
                if !ext.trait_generic_args.is_empty() {
                    write!(sink.body, "<")?;
                    for (i, arg) in ext.trait_generic_args.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        write!(sink.body, "{}", self.rust_type_name(arg))?;
                    }
                    write!(sink.body, ">")?;
                }
                write!(sink.body, " for {}", ext.target)?;
            }
            None => {
                write!(sink.body, "impl {}", ext.target)?;
            }
        }

        // Add generic parameters if present
        if !ext.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in ext.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => {
                        write!(sink.body, "{}", tp.name)?;
                        // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                        if !tp.constraint.is_empty() {
                            write!(sink.body, ": ")?;
                            for (ci, ct) in tp.constraint.iter().enumerate() {
                                if ci > 0 {
                                    write!(sink.body, " + ")?;
                                }
                                write!(sink.body, "{}", self.rust_bound_name(ct))?;
                            }
                        }
                    }
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        writeln!(sink.body, " {{")?;
        self.indent();

        // C8: associated consts (`[pub] const NAME TYPE = value` inside ext)
        for c in &ext.consts {
            self.print_indent(&mut sink.body)?;
            let ty_name = if matches!(c.ty, Type::StrFixed(_) | Type::StrSlice | Type::StrOwned) {
                "&str".to_string()
            } else {
                self.rust_type_name(&c.ty)
            };
            if c.is_pub {
                write!(sink.body, "pub const {}: {} = ", c.name, ty_name)?;
            } else {
                write!(sink.body, "const {}: {} = ", c.name, ty_name)?;
            }
            self.expr(&c.expr, &mut sink.body)?;
            sink.body.write(b";\n")?;
        }

        // Generate methods
        // Plan 514 W3: ext 方法同款传递性可变方法集预计算。
        let ext_methods: Vec<&Fn> = ext.methods.iter().collect();
        self.current_type_mut_methods = Some(Self::compute_type_mut_methods(&ext_methods));
        if let Some(s) = &self.current_type_mut_methods {
            self.all_mut_method_names.extend(s.iter().cloned());
        }
        self.current_impl_type = Some(ext.target.clone());
        for method in &ext.methods {
            self.fn_decl(method, sink)?;
            sink.body.write(b"\n")?;
        }
        self.current_impl_type = None;
        self.current_type_mut_methods = None;

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n")?;

        // Plan 382: synthesize std::error::Error + Display for ENUM types with
        // an inherent message() method (single source of truth — Display
        // delegates to message(), so format strings live in one place).
        // This makes transpiled error enums real `std::error::Error` values
        // (interop with `?` chains / Box<dyn Error> / logging), matching the
        // Rust reference's `#[derive(Error)]` without a thiserror dependency.
        let has_message =
            ext.trait_name.is_none() && ext.methods.iter().any(|m| m.name.as_str() == "message");
        if has_message && self.known_enum_names.contains(ext.target.as_str()) {
            writeln!(sink.body, "impl std::fmt::Display for {} {{", ext.target)?;
            self.indent();
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{"
            )?;
            self.indent();
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "write!(f, \"{{}}\", self.message())")?;
            self.dedent();
            self.print_indent(&mut sink.body)?;
            writeln!(sink.body, "}}")?;
            self.dedent();
            writeln!(sink.body, "}}")?;
            writeln!(sink.body, "impl std::error::Error for {} {{}}", ext.target)?;
            sink.body.write(b"\n")?;
        }

        Ok(())
    }

    // Spec/trait declaration
    // Plan 204 Phase 4: spec → Rust trait mapping
    pub(super) fn spec_decl(&mut self, spec_decl: &SpecDecl, sink: &mut Sink) -> AutoResult<()> {
        // Cache spec methods for later use in impl Trait for Type
        self.spec_decls
            .insert(spec_decl.name.clone(), spec_decl.methods.clone());

        // Plan 390 §15.10: `spec Fn` is a phantom spec — it maps to Rust's
        // builtin `Fn` trait (std prelude). Emitting a local `trait Fn {}`
        // would SHADOW the prelude `Fn` and break `Box<dyn Fn(A)>` (the local
        // trait has no `Args` type param). Skip the trait definition; the
        // spec_decls entry above keeps `Fn` resolvable as a spec (Type::Spec →
        // `Box<dyn Fn>` / the Box/Arc special case).
        if spec_decl.name.as_str() == "Fn" {
            return Ok(());
        }

        // Plan 380: async spec methods (~Result / Future) need #[async_trait]
        // on the TRAIT declaration too — a bare `-> Future<...>` return type in
        // a trait is E0782. Plan 373 G2 only annotated the impl blocks.
        // Plan 382 (A.1): `!T` → Type::Result is SYNC — excluded.
        let has_async_method = spec_decl
            .methods
            .iter()
            .any(|m| matches!(&m.ret, Type::GenericInstance(inst) if inst.base_name == "Future"));
        if has_async_method {
            write!(sink.body, "#[async_trait::async_trait]\n")?;
        }

        // Plan 163: Output pub prefix
        if spec_decl.is_pub {
            write!(sink.body, "pub ")?;
        }

        // Generate trait definition with generic parameters
        write!(sink.body, "trait {}", spec_decl.name)?;

        // Add generic parameters if present
        if !spec_decl.generic_params.is_empty() {
            write!(sink.body, "<")?;
            for (i, param) in spec_decl.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(sink.body, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => {
                        write!(sink.body, "{}", tp.name)?;
                        // Plan 364 W3: multi-bound `#[with(T as A + B)]` → `T: A + B`
                        if !tp.constraint.is_empty() {
                            write!(sink.body, ": ")?;
                            for (ci, ct) in tp.constraint.iter().enumerate() {
                                if ci > 0 {
                                    write!(sink.body, " + ")?;
                                }
                                write!(sink.body, "{}", self.rust_bound_name(ct))?;
                            }
                        }
                    }
                    GenericParam::Const(cp) => {
                        write!(sink.body, "{}: {}", cp.name, self.rust_type_name(&cp.typ))?
                    }
                }
            }
            write!(sink.body, ">")?;
        }

        // Plan 397: supertrait bounds — `spec Tool: Send + Sync { }` → `trait Tool: Send + Sync { }`.
        // Bounds are opaque identifier strings from the .at source, emitted verbatim.
        if !spec_decl.bounds.is_empty() {
            write!(sink.body, ": {}", spec_decl.bounds.join(" + "))?;
        }

        writeln!(sink.body, " {{")?;
        self.indent();

        // Plan 417-E2 (DIV-TRAIT-LANG-1): associated types emit as Rust
        // associated type declarations (`type Item;`) before the methods.
        // Method signatures below reference them via `Self::Item`: rewrite
        // each bare Type::User(assoc) reference to an opaque User type named
        // "Self::Item" (rendered verbatim — see the Self:: guard in
        // rust_type_name), reusing Type::substitute for nested positions
        // (List<Item> → List<Self::Item>).
        let assoc_self_names: Vec<Name> = spec_decl
            .associated_types
            .iter()
            .map(|at| at.name.clone())
            .collect();
        let assoc_self_types: Vec<Type> = spec_decl
            .associated_types
            .iter()
            .map(|at| {
                Type::User(TypeDecl {
                    consts: Vec::new(),
                    name: Name::from(format!("Self::{}", at.name).as_str()),
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
                })
            })
            .collect();
        let subst_assoc_to_self = |ty: &Type| -> Type {
            if assoc_self_names.is_empty() {
                ty.clone()
            } else {
                ty.substitute(&assoc_self_names, &assoc_self_types)
            }
        };
        for at in &spec_decl.associated_types {
            self.print_indent(&mut sink.body)?;
            write!(sink.body, "type {};", at.name)?;
            if let Some(ref bound) = at.bound {
                write!(sink.body, " // bound: {}", bound)?;
            }
            writeln!(sink.body)?;
        }

        // Plan 417-followup (DIV-TRAIT-A2R-2, checker-adjacent): the spec's
        // generic params act as the "current fn" type params while emitting
        // trait method signatures, so a return type referencing one renders
        // as the bare param (`-> T`), never `impl T` (the PascalCase-trait
        // heuristic misfires on single-letter params). Restored after the
        // method loop (see the assignment just below the loop).
        let saved_spec_type_params = std::mem::take(&mut self.current_fn_type_params);
        for tp_name in spec_decl.generic_params.iter().filter_map(|gp| match gp {
            GenericParam::Type(tp) => Some(tp.name.clone()),
            _ => None,
        }) {
            self.current_fn_type_params.insert(tp_name);
        }
        for method in &spec_decl.methods {
            // Plan 380: async spec methods emit `async fn` (mirrors fn_decl).
            // With `#[async_trait]` (added above when any method is async), a
            // bare `-> Future<...>` in the trait would be E0782 — async_trait
            // rewrites `async fn -> Result<...>` into the boxed-Future form.
            // Plan 382 (A.1): `!T` → Type::Result is SYNC — excluded.
            let method_is_async =
                matches!(&method.ret, Type::GenericInstance(inst) if inst.base_name == "Future");
            self.print_indent(&mut sink.body)?;
            if method_is_async {
                write!(sink.body, "async ")?;
            }
            write!(sink.body, "fn {}(&self", method.name)?;

            // Parameters (skip self which is already added as &self)
            // Plan 417-E2: associated-type references render as Self::Item.
            for param in &method.params {
                write!(
                    sink.body,
                    ", {}: {}",
                    param.name,
                    self.rust_param_type_name(&subst_assoc_to_self(&param.ty))
                )?;
            }

            // Return type — use rust_return_type_name for correct str→String mapping
            // Plan 204 Phase 4: !T (Type::Result) → Result<T, String>
            // Plan 417-E2: associated-type references render as Self::Item.
            let method_ret = subst_assoc_to_self(&method.ret);
            if !matches!(method_ret, Type::Void) {
                let ret_str = if method_is_async {
                    match &method_ret {
                        // ~Result / Future<Result<...>> → Result<...> for `async fn`
                        Type::GenericInstance(inst) if inst.base_name == "Future" => {
                            self.rust_return_type_name(inst.args.first().unwrap_or(&Type::Unknown))
                        }
                        other => self.rust_return_type_name(other),
                    }
                } else {
                    self.rust_return_type_name(&method_ret)
                };
                write!(sink.body, ") -> {}", ret_str)?;
            } else {
                write!(sink.body, ")")?;
            }

            // Default method implementation (Plan 019 Stage 8.5; fixed Plan 359 DIV-TRAIT-A2R-1).
            // SpecMethod.body is Option<Box<Expr>>. For a block body we delegate to the
            // generic body() emitter so a value-returning default method keeps its tail
            // expression (instead of becoming `{ expr; }` = unit, which caused E0308).
            // body() writes its own { ... } and uses method.ret for string coercion.
            if let Some(ref default_body) = method.body {
                match **default_body {
                    Expr::Block(ref block_body) => {
                        self.body(block_body, sink, &method_ret, "")?;
                    }
                    _ => {
                        // Single-expression default body: wrap minimally.
                        sink.body.write(b" {\n")?;
                        self.indent();
                        self.print_indent(&mut sink.body)?;
                        self.expr(default_body, &mut sink.body)?;
                        sink.body.write(b"\n")?;
                        self.dedent();
                        self.print_indent(&mut sink.body)?;
                        sink.body.write(b"}\n")?;
                    }
                }
            } else {
                writeln!(sink.body, ";")?;
            }
        }
        self.current_fn_type_params = saved_spec_type_params; // Plan 417-followup

        self.dedent();
        writeln!(sink.body, "}}\n")?;

        Ok(())
    }

    // Body and block management
    pub(super) fn body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
        ret_type: &Type,
        _insert: &str,
    ) -> AutoResult<()> {
        // Set current_fn_ret_type so that return statements can check if .to_string() is needed
        self.current_fn_ret_type = Some(ret_type.clone());
        let has_return = !matches!(ret_type, Type::Void);

        sink.body.write(b"{\n")?;
        self.indent();

        // Plan 387: actor `fn main` prologue (e.g. `let mut __rt = ...;`).
        if let Some(prologue) = &self.main_actor_prologue {
            self.print_indent(&mut sink.body)?;
            sink.body.write(prologue.as_bytes())?;
        }

        // Plan 018 §14 W2: track Mutex guards (`var g = ...lock().unwrap()`)
        // so an `is g.get(k) { None -> {} }` scrutinee can `drop(g)` after the
        // match. a2r's match (unlike hw's `if let`) keeps the guard alive until
        // fn end → a second `lock()` in the same fn deadlocks. We drop the
        // guard right after the is-stmt when it isn't used later in the body.
        let mut lock_guards: std::collections::HashSet<AutoStr> = std::collections::HashSet::new();

        // PLAN-010 T2: per-statement later-read sets (see the field doc).
        // A None from the collector (unwalked variant) means "assume used".
        let stmt_reads: Vec<Option<std::collections::HashSet<AutoStr>>> =
            body.stmts.iter().map(Self::stmt_read_idents).collect();

        // Process statements
        for (i, stmt) in body.stmts.iter().enumerate() {
            self.later_used_locals =
                stmt_reads[i + 1..]
                    .iter()
                    .fold(std::collections::HashSet::new(), |mut acc, s| {
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
                    });
            // W2: record guard bindings from `var g = ...lock().unwrap()`
            if let Stmt::Store(store) = stmt {
                if Self::store_is_lock_guard(store) {
                    lock_guards.insert(store.name.clone());
                }
            }
            sink.record();
            // Set source line for mapping
            if i < body.source_lines.len() {
                sink.set_source_line(body.source_lines[i]);
            }
            if !matches!(stmt, Stmt::EmptyLine(_)) {
                self.print_indent(&mut sink.body)?;
            }

            let is_last = i == body.stmts.len() - 1;

            // PLAN-010 T2: `fn f() ?T { nil }` — a Nil tail in an
            // Option-returning fn IS the return value (None), it must not fall
            // to the statement path where `None;` types the block as `()`.
            let is_option_nil_tail = is_last
                && matches!(ret_type, Type::Option(_))
                && matches!(stmt, Stmt::Expr(Expr::Nil | Expr::Null));

            if is_last && has_return && (self.is_returnable(stmt) || is_option_nil_tail) {
                // Last statement in a non-void function: expression position (no semicolon)
                match stmt {
                    Stmt::Expr(expr) => {
                        // PLAN-010 T2: bare-scalar tail of an Option-returning fn
                        // is Auto's implicit Some(value) — mirror write_return_expr's
                        // Plan 531 P525-4 wrap so `fn f() ?int { x }` typechecks
                        // (E0308 otherwise). Option-shaped exprs are untouched.
                        let may_wrap_some = matches!(ret_type, Type::Option(_))
                            && !matches!(expr, Expr::Nil | Expr::Null)
                            && !Self::expr_is_option_shaped(expr)
                            && (matches!(self.infer_type_from_expr(expr),
                                    Type::Int | Type::Uint | Type::USize | Type::I64 | Type::U64
                                    | Type::Float | Type::Double | Type::Bool | Type::Char | Type::Byte
                                    | Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit)
                                // `g()?` tail in a `?T` fn: the `?` unwraps to
                                // the bare inner value — re-wrap as the Option
                                // the fn returns (009 ledger 011/019).
                                || matches!(expr, Expr::ErrorPropagate(_)));
                        if may_wrap_some {
                            sink.body.write(b"Some(")?;
                        }
                        if matches!(expr, Expr::Nil | Expr::Null) {
                            sink.body.write(b"None")?;
                        } else {
                            self.expr(expr, &mut sink.body)?;
                        }
                        // If return type is String and expr produces &str, add .to_string()
                        let needs_to_string = self.ret_type_needs_string_coercion()
                            && self.expr_needs_string_coercion(expr);
                        if needs_to_string {
                            sink.body.write(b".to_string()")?;
                        }
                        // Plan 013 (B1/BUG2): tail-position `self.field` of an
                        // owned non-Copy type in a &self method needs .clone()
                        // (E0507 otherwise). Mirrors write_return_expr.
                        if !needs_to_string
                            && Self::is_self_dot(expr)
                            && self.ret_type_is_owned_noncopy()
                        {
                            sink.body.write(b".clone()")?;
                        }
                        if may_wrap_some {
                            sink.body.write(b")")?;
                        }
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Node(node) => {
                        // Node (struct constructor) as tail expression — no semicolon
                        self.expr(&Expr::Node(node.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::If(inner_if) => {
                        // Plan 417-E4 investigation: a tail if WITH else in a non-void
                        // fn is a VALUE expression — branch tail exprs must NOT get
                        // the statement-context `;` (if_stmt's Plan 393 E3 rule).
                        let saved = self.value_if_tail;
                        self.value_if_tail = true;
                        self.if_stmt(inner_if, sink)?;
                        self.value_if_tail = saved;
                        sink.body.write(
                            b"
",
                        )?;
                    }
                    Stmt::Is(is_stmt) => {
                        // `is` (match) as tail expression — emit WITHOUT the
                        // trailing semicolon that statement-position `is` adds
                        // (fn stmt(), line ~7670). A match as the last stmt of
                        // a value-returning fn is a tail expression: its value
                        // is the fn's return value. Adding `;` makes the fn
                        // return `()` (E0308). (Plan 016 Phase A A2.)
                        self.is_stmt(is_stmt, sink)?;
                        sink.body.write(b"\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n")?;
                    }
                }
            } else {
                // Regular statement: add semicolon if needed
                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::EmptyLine(n) => {
                        for _ in 0..*n {
                            sink.body.write(b"\n")?;
                        }
                    }
                    Stmt::Break => {
                        sink.body.write(b"break;\n")?;
                    }
                    Stmt::Is(is_stmt) => {
                        // W2: after the match, drop a Mutex guard used only as
                        // the scrutinee receiver (see lock_guards note above).
                        self.is_stmt(is_stmt, sink)?;
                        sink.body.write(b";\n")?;
                        if let Some(guard) = Self::is_scrutinee_receiver(is_stmt) {
                            if lock_guards.contains(&guard)
                                && !Self::stmts_reference_ident(&body.stmts[i + 1..], &guard)
                            {
                                self.print_indent(&mut sink.body)?;
                                sink.body.write(b"drop(")?;
                                sink.body.write(guard.as_bytes())?;
                                sink.body.write(b");\n")?;
                            }
                        }
                    }
                    _ => {
                        // For other statement types that handle their own formatting
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n")?;
                    }
                }
            }
        }
        sink.record();

        // For Result-returning functions, append Ok(()) if the last
        // statement is not a tail expression (e.g., ends with a semicolon).
        // An empty body also needs Ok(()) — otherwise `fn f() -> Result<...> { }`
        // is a type error (E0308). This matters for actor hooks like
        // `fn start() ! { }` (Plan 387 §12.4).
        if matches!(ret_type, Type::Result(_)) {
            let needs_ok = if body.stmts.is_empty() {
                true
            } else {
                let last = &body.stmts[body.stmts.len() - 1];
                !self.is_returnable(last)
            };
            if needs_ok {
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"Ok(())\n")?;
            }
        }

        // Plan 387: actor `fn main` epilogue (e.g. `drop(h); __rt.run_to_completion().await;`).
        // Each line gets its own indent (epilogue may be multi-line).
        if let Some(epilogue) = &self.main_actor_epilogue {
            for line in epilogue.lines() {
                if !line.is_empty() {
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(line.as_bytes())?;
                    sink.body.write(b"\n")?;
                }
            }
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        Ok(())
    }

    /// PLAN-009 T1 (F1): post-processing that knows which type names had
    /// their auto-derived comparison set restricted (explicit-derive
    /// surfaces), so fix_non_ord_derives never re-widens them. Entry points
    /// holding a RustTrans pass `transpiler.ord_restricted_names`.
    pub fn post_process_with(output: &mut Vec<u8>, ord_restricted: &HashSet<AutoStr>) {
        let mut content = String::from_utf8(std::mem::take(output)).unwrap_or_default();

        // B3: Remove duplicate `use self::X;` when `pub mod X;` exists
        Self::remove_duplicate_module_uses(&mut content);

        // B3b: Remove duplicate imports that import locally-defined symbols
        Self::remove_duplicate_imports(&mut content);

        // A7: Vec.get(i32_var) → Vec[i32_var as usize] (heuristic)
        Self::fix_counted(&mut content, "fix_vec_i32_index", Self::fix_vec_i32_index);

        // A8: HashMap.get(key).field → HashMap.get(key).unwrap().field
        Self::fix_counted(
            &mut content,
            "fix_option_unwrapping",
            Self::fix_option_unwrapping,
        );

        // A9: vec.get(0.as_str()) → vec[0], vec.get(N.as_str()) → vec[N as usize]
        Self::fix_counted(
            &mut content,
            "fix_numeric_get_as_str",
            Self::fix_numeric_get_as_str,
        );

        // A10: self.sessions.get(X) { Some(var) => → self.sessions.get(X).cloned() { Some(var) =>
        Self::fix_counted(
            &mut content,
            "fix_get_cloned_for_match",
            Self::fix_get_cloned_for_match,
        );

        // B2: String/&str heuristic fixes
        Self::fix_counted(
            &mut content,
            "fix_string_str_mismatches",
            Self::fix_string_str_mismatches,
        );

        // B13: Fix derive macros on structs with dyn Trait fields
        Self::fix_counted(
            &mut content,
            "fix_dyn_trait_derives",
            Self::fix_dyn_trait_derives,
        );

        // B14: Fix integer type mismatches (u32 vs i32 vs usize)
        Self::fix_counted(
            &mut content,
            "fix_integer_type_mismatches",
            Self::fix_integer_type_mismatches,
        );

        // B16: Add `mut` to let bindings that are later reassigned
        Self::fix_counted(
            &mut content,
            "fix_mutable_bindings",
            Self::fix_mutable_bindings,
        );

        // B16b: Add `mut` to fn params that are mutated in the body
        // (a2r only handles `let` locals; Rust params default to immutable →
        // E0596 for e.g. `fn f(seen: HashMap, names: Vec) { seen.insert(..); }`).
        Self::fix_counted(&mut content, "fix_mutable_params", Self::fix_mutable_params);

        // B17: Fix return None; in void functions → return;
        Self::fix_counted(
            &mut content,
            "fix_void_return_none",
            Self::fix_void_return_none,
        );

        // B18: Fix borrowing issues (&Vec → Vec.clone(), etc.)
        Self::fix_counted(
            &mut content,
            "fix_borrowing_issues",
            Self::fix_borrowing_issues,
        );

        // B19: Fix HashMap.keys() used as indexable collection (Auto List → Rust iterator)
        Self::fix_counted(
            &mut content,
            "fix_map_keys_indexing",
            Self::fix_map_keys_indexing,
        );

        // B20: Fix push move errors — add .clone() when pushing reused variables
        Self::fix_counted(&mut content, "fix_push_move", Self::fix_push_move);

        // B21: Fix &str params assigned to String fields / pushed to Vec<String>
        Self::fix_counted(
            &mut content,
            "fix_str_to_string_assignments",
            Self::fix_str_to_string_assignments,
        );

        // B22: Fix Option<String>.unwrap_or("") → .unwrap_or_default()
        Self::fix_counted(
            &mut content,
            "fix_option_unwrap_or_empty",
            Self::fix_option_unwrap_or_empty,
        );

        // B23: Fix String passed where &_ is expected (map.get(var) → map.get(&var))
        Self::fix_counted(&mut content, "fix_string_to_ref", Self::fix_string_to_ref);

        // B15: Fix enum == "str" comparisons — Auto enums can compare with str, Rust can't
        Self::fix_counted(
            &mut content,
            "fix_enum_str_comparisons",
            Self::fix_enum_str_comparisons,
        );

        // B7: Fix vec![(str, str, str)] where return type is Vec<(String,...)>
        Self::fix_counted(
            &mut content,
            "fix_vec_tuple_string_literals",
            Self::fix_vec_tuple_string_literals,
        );

        // B8: Fix tuple.get_N() -> tuple.N
        Self::fix_counted(&mut content, "fix_tuple_get_n", Self::fix_tuple_get_n);

        // B4: Fix u32/i32 cast mismatches
        Self::fix_counted(&mut content, "fix_u32_i32_casts", Self::fix_u32_i32_casts);

        // B5: Fix Vec/HashMap .insert() first arg needs usize
        Self::fix_counted(&mut content, "fix_insert_usize", Self::fix_insert_usize);

        // B6: Fix bool-returning functions used with == 0 / != 0
        Self::fix_counted(
            &mut content,
            "fix_bool_int_comparisons",
            Self::fix_bool_int_comparisons,
        );

        // B9: Fix map.get(key).as_str() → map.get(key).map(|s| s.as_str()).unwrap_or("")
        Self::fix_counted(&mut content, "fix_map_get_as_str", Self::fix_map_get_as_str);

        // B10: Fix integer.as_str() → integer.to_string().as_str()
        Self::fix_counted(&mut content, "fix_int_as_str", Self::fix_int_as_str);

        // B11: Fix str.split(X).len() → str.split(X).count()
        //     and str.split(X).get(i) → str.split(X).nth(i)
        Self::fix_counted(&mut content, "fix_split_methods", Self::fix_split_methods);

        // Plan 373 backports — additional B1 codegen papercuts.
        // Lower Auto-VM str/numeric methods + structural fixes that the main
        // trans() pass doesn't emit correctly yet (see docs/plans/373).
        Self::fix_counted(
            &mut content,
            "fix_substring_method",
            Self::fix_substring_method,
        );
        Self::fix_counted(
            &mut content,
            "fix_numeric_conversion_methods",
            Self::fix_numeric_conversion_methods,
        );
        Self::fix_counted(
            &mut content,
            "fix_residual_error_box",
            Self::fix_residual_error_box,
        );
        Self::fix_counted(
            &mut content,
            "fix_result_none_unit",
            Self::fix_result_none_unit,
        );
        Self::fix_counted(&mut content, "fix_fn_field_calls", Self::fix_fn_field_calls);
        // fix_non_ord_derives carries PLAN-009 F1 state (restricted type
        // names), so it can't go through the fn-pointer fix_counted helper;
        // count inline with the same convention.
        {
            let before = content.len();
            Self::fix_non_ord_derives(&mut content, ord_restricted);
            if content.len() != before {
                if let Ok(mut m) = FIX_COUNTS.lock() {
                    *m.entry("fix_non_ord_derives".to_string()).or_insert(0) += 1;
                }
            }
        }
        Self::fix_counted(
            &mut content,
            "fix_missing_trait_impl_uses",
            Self::fix_missing_trait_impl_uses,
        );
        Self::fix_counted(
            &mut content,
            "fix_string_literal_enum_args",
            Self::fix_string_literal_enum_args,
        );
        // Plan 376: Type-flow analysis post_process passes
        Self::fix_counted(
            &mut content,
            "fix_for_in_self_field_borrow",
            Self::fix_for_in_self_field_borrow,
        );
        Self::fix_counted(
            &mut content,
            "fix_option_get_field_access",
            Self::fix_option_get_field_access,
        );
        Self::fix_counted(
            &mut content,
            "fix_some_str_to_string",
            Self::fix_some_str_to_string,
        );
        Self::fix_counted(
            &mut content,
            "fix_a2r_std_fs_result_patterns",
            Self::fix_a2r_std_fs_result_patterns,
        );
        Self::fix_counted(
            &mut content,
            "fix_spec_trait_boxing",
            Self::fix_spec_trait_boxing,
        );
        Self::fix_counted(&mut content, "fix_pathbuf_as_str", Self::fix_pathbuf_as_str);
        Self::fix_counted(&mut content, "fix_tuple_index", Self::fix_tuple_index);

        if !content.ends_with('\n') {
            content.push('\n');
        }

        // Plan 014 Layer 3: A2R_FIX_COUNTS=1 → print which fixes rewrote output.
        if std::env::var("A2R_FIX_COUNTS")
            .map(|v| v == "1")
            .unwrap_or(false)
        {
            if let Ok(m) = FIX_COUNTS.lock() {
                for (k, v) in m.iter() {
                    eprintln!("[fix-count] {}={}", k, v);
                }
            }
        }

        *output = content.into_bytes();
    }

    /// Fix Vec.get(i32_var) → Vec[i32_var as usize] using heuristic i32 variable names.
    fn fix_vec_i32_index(content: &mut String) {
        let hash_map_names = [
            "map",
            "dict",
            "env",
            "vars",
            "cache",
            "sessions",
            "entries",
            "headers",
            "params",
            "options",
            "metadata",
            "config",
            "routes",
            "data",
            "properties",
            "fields",
            "professions",
            "souls",
            "flows",
            "agents",
            "providers",
            "runs",
            "checkpoints",
            "project_locks",
            // Plan 376: ToolRegistry.tools is HashMap<String, Arc<...>>;
            // lookups are string-keyed, never integer-indexed.
            "tools",
        ];
        let vec_field_names = [
            "tool_call_ids",
            "tool_call_names",
            "tool_call_args",
            "tool_call_started",
            "items",
            "steps",
            "events",
            "messages",
        ];

        // Pattern 1: self.field.get(var) → self.field[var as usize] for known Vec fields
        // But ONLY when `var` is NOT a string type (String vars can't cast to usize)
        if let Some(re) = cached_regex(r"(self\.(\w+))\.get\((\w+)\)") {
            let new_content = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let full = caps.get(1).unwrap().as_str();
                    let field = caps.get(2).unwrap().as_str();
                    let var = caps.get(3).unwrap().as_str();
                    if vec_field_names.contains(&field) {
                        // Check if var looks like a string (starts with a letter and isn't i/idx/n/index)
                        let is_likely_index =
                            var.starts_with('i') || var == "idx" || var == "index" || var == "n";
                        if is_likely_index {
                            format!("{}[{} as usize]", full, var)
                        } else {
                            // Keep .get() for string-keyed lookups
                            format!("{}.get({})", full, var)
                        }
                    } else {
                        format!("{}.get({})", full, var)
                    }
                })
                .to_string();
            if new_content != *content {
                *content = new_content;
            }
        }

        // Pattern 2: vecname.get(var) → vecname[var as usize] for non-HashMap, non-self.field
        let int_like_vars = [
            "i", "j", "k", "ci", "ti", "ki", "ri", "ei", "pi", "si", "ti2", "ri2", "tri", "tc_i",
            "step_idx", "idx", "offset", "pos", "n", "count", "len", "start", "end", "index",
            "from", "slot", "col", "gii", "pii", "ppi", "ii", "iii", "di", "li", "mi", "ni", "qi",
            "vi", "wi", "xi", "yi", "zi", "gi", "si2", "hi", "fi", "ai", "bi", "ci2", "no",
        ];

        for var in &int_like_vars {
            // vecname.get(var.as_str()) → vecname[var as usize]
            let pattern_str = format!(r"(\w+)\.get\({}\.as_str\(\)\)", regex::escape(var));
            if let Some(re) = cached_regex(&pattern_str) {
                let new_content = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let vec_name = caps.get(1).unwrap().as_str();
                        if hash_map_names.contains(&vec_name) {
                            format!("{}.get({}.as_str())", vec_name, var)
                        } else {
                            format!("{}[{} as usize]", vec_name, var)
                        }
                    })
                    .to_string();
                if new_content != *content {
                    *content = new_content;
                }
            }

            // vecname.get(var) where not already followed by .as_str or as usize
            // Note: Rust regex crate doesn't support lookahead, so we match broadly
            // and filter in the replacement callback
            let pattern = format!(r"(\w+)\.get\({}\)", regex::escape(var));
            if let Some(re) = cached_regex(&pattern) {
                let new_content = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let full_match = caps.get(0).unwrap();
                        let after = &content[full_match.end()..];
                        // Skip if already followed by " as usize" or ".as_str"
                        if after.starts_with(" as usize") || after.starts_with(".as_str") {
                            return full_match.as_str().to_string();
                        }
                        let vec_name = caps.get(1).unwrap().as_str();
                        if hash_map_names.contains(&vec_name) {
                            format!("{}.get({})", vec_name, var)
                        } else {
                            format!("{}[{} as usize]", vec_name, var)
                        }
                    })
                    .to_string();
                if new_content != *content {
                    *content = new_content;
                }
            }
        }

        // Pattern 3: .get(0) or .get(NUM) → [NUM] for Vec-like collections
        // DISABLED: AST-level handling now correctly converts Auto List.get(N) → [N as usize].clone()
        // This pattern was incorrectly converting Rust Vec::get(N) (returns Option) to [N] (returns T)
        // For use.rust code, .get(NUM) should remain as-is.

        // Pattern 4: expr.field.get(var) → expr.field[var as usize] for Vec fields
        // Handles cases like goal.items.get(gii), plan.sections.get(pi), etc.
        for var in &int_like_vars {
            let pattern = format!(r"(\w+)\.(\w+)\.get\({}\)", regex::escape(var));
            if let Some(re) = cached_regex(&pattern) {
                let new_content = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let obj = caps.get(1).unwrap().as_str();
                        let field = caps.get(2).unwrap().as_str();
                        if vec_field_names.contains(&field) {
                            format!("{}.{}[{} as usize]", obj, field, var)
                        } else {
                            format!("{}.{}.get({})", obj, field, var)
                        }
                    })
                    .to_string();
                if new_content != *content {
                    *content = new_content;
                }
            }
        }
    }

    /// Fix HashMap.get(key).field → HashMap.get(key).unwrap().field
    fn fix_option_unwrapping(content: &mut String) {
        let known_fields = [
            "id",
            "name",
            "status",
            "content",
            "section_type",
            "items",
            "profession_id",
            "title",
            "model",
            "role",
            "kind",
            "stop_reason",
            "provider",
            "api_key_env",
            "base_url",
            "is_available",
            "models",
            "soul_id",
            "api_source_id",
            "model_tier",
            "is_default",
            "temperature",
            "max_tokens",
            "description",
            "steps",
            "exit",
            "gate",
            "avatar_url",
            "project_path",
            "messages",
            "system_prompt",
            "tools",
            "input_tokens",
            "output_tokens",
            "usage",
            "error",
        ];

        for field in &known_fields {
            let pattern = format!(r"\.get\(([^)]+)\)\.{}\b", regex::escape(field));
            if let Some(re) = cached_regex(&pattern) {
                let new_content = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let inner = caps.get(1).unwrap().as_str();
                        if inner.contains(".unwrap()") {
                            caps.get(0).unwrap().as_str().to_string()
                        } else {
                            format!(".get({}).unwrap().{}", inner, field)
                        }
                    })
                    .to_string();
                *content = new_content;
            }
        }
        // Also handle .get(X).as_str() → .get(X).unwrap().as_str()
        // (Option<&String> doesn't have .as_str(), need to unwrap first)
        if let Some(re) = cached_regex(r"\.get\(([^)]+)\)\.as_str\(\)") {
            let new_content = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let inner = caps.get(1).unwrap().as_str();
                    if inner.contains(".unwrap()") {
                        caps.get(0).unwrap().as_str().to_string()
                    } else {
                        format!(".get({}).unwrap().as_str()", inner)
                    }
                })
                .to_string();
            *content = new_content;
        }
    }

    /// Fix u32/i32 cast mismatches:
    /// 1. `let ... : u32 = (... as i64)` → `as u32`
    /// 2. `while var < (... as i64)` where var was declared as u32 → `as u32`
    fn fix_u32_i32_casts(content: &mut String) {
        use std::collections::HashMap;
        // Build a map of variable names declared as u32
        let u32_vars: HashMap<String, ()> = {
            let mut map = HashMap::new();
            if let Some(re) = cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*:\s*u32\s*=") {
                for caps in re.captures_iter(content) {
                    map.insert(caps.get(1).unwrap().as_str().to_string(), ());
                }
            }
            map
        };
        if u32_vars.is_empty() {
            return;
        }

        // Pattern 1: `let ... : u32 = (... as i64)` → `as u32`
        if let Some(re) =
            cached_regex(r"(let\s+(?:mut\s+)?\w+\s*:\s*u32\s*=\s*\()(.+?)\s+as\s+i32\)")
        {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let prefix = caps.get(1).unwrap().as_str();
                    let expr = caps.get(2).unwrap().as_str();
                    format!("{}{} as u32)", prefix, expr)
                })
                .to_string();
            *content = new;
        }

        // Pattern 2: `while var < (... as i64)` where var is a u32 var → `as u32`
        for var_name in u32_vars.keys() {
            let pattern = format!(
                r"(while\s+{}\s*<\s*\()(.+?)\s+as\s+i32\)",
                regex::escape(var_name)
            );
            if let Some(re) = cached_regex(&pattern) {
                let _vn = var_name.clone(); // used in closure if needed
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let prefix = caps.get(1).unwrap().as_str();
                        let expr = caps.get(2).unwrap().as_str();
                        format!("{}{} as u32)", prefix, expr)
                    })
                    .to_string();
                *content = new;
            }
        }

        // Pattern 3: struct field assignment `self.field: u32 = (... as i64)` for known u32 fields
        // Detected via struct field declarations: `pub field_name: u32,`
        let u32_fields: Vec<String> = {
            let mut fields = Vec::new();
            if let Some(re) = cached_regex(r"pub\s+(\w+)\s*:\s*u32\s*,") {
                for caps in re.captures_iter(content) {
                    fields.push(caps.get(1).unwrap().as_str().to_string());
                }
            }
            fields
        };
        for field_name in &u32_fields {
            // `self.field_name = (... as i64)` → `as u32`
            let pattern = format!(
                r"(self\.{}\s*=\s*\()(.+?)\s+as\s+i32\)",
                regex::escape(field_name)
            );
            if let Some(re) = cached_regex(&pattern) {
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let prefix = caps.get(1).unwrap().as_str();
                        let expr = caps.get(2).unwrap().as_str();
                        format!("{}{} as u32)", prefix, expr)
                    })
                    .to_string();
                *content = new;
            }
        }
    }

    /// Fix bool-returning functions compared with integer literals.
    /// a2r_std::fs::exists/is_dir now return bool, but Auto code uses == 0 / != 0.
    fn fix_bool_int_comparisons(content: &mut String) {
        // Pattern: `a2r_std::fs::exists(X) == 0` → `!a2r_std::fs::exists(X)`
        // Use non-greedy match to handle nested parens like `file_path.as_str()`
        if let Some(re) = cached_regex(r"a2r_std::fs::exists\((.+?)\)\s*==\s*0") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    format!("!a2r_std::fs::exists({})", caps.get(1).unwrap().as_str())
                })
                .to_string();
            *content = new;
        }
        // Pattern: `a2r_std::fs::exists(X) != 0` → `a2r_std::fs::exists(X)`
        if let Some(re) = cached_regex(r"a2r_std::fs::exists\((.+?)\)\s*!=\s*0") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    format!("a2r_std::fs::exists({})", caps.get(1).unwrap().as_str())
                })
                .to_string();
            *content = new;
        }
        // Pattern: `a2r_std::fs::is_dir(X) == 0` → `!a2r_std::fs::is_dir(X)`
        if let Some(re) = cached_regex(r"a2r_std::fs::is_dir\((.+?)\)\s*==\s*0") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    format!("!a2r_std::fs::is_dir({})", caps.get(1).unwrap().as_str())
                })
                .to_string();
            *content = new;
        }
        // Pattern: `a2r_std::fs::is_dir(X) != 0` → `a2r_std::fs::is_dir(X)`
        if let Some(re) = cached_regex(r"a2r_std::fs::is_dir\((.+?)\)\s*!=\s*0") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    format!("a2r_std::fs::is_dir({})", caps.get(1).unwrap().as_str())
                })
                .to_string();
            *content = new;
        }
        // Pattern: `!(a2r_std::fs::is_dir(X))` → `!a2r_std::fs::is_dir(X)`
        // Only if the closing parens match — avoid removing extra parens
        // Skip this for now — `!(bool_expr)` is valid Rust

        // Pattern: `let VAR = a2r_std::fs::is_dir(X); ... if VAR != 0` → `if VAR`
        // Find variables assigned from is_dir and replace `VAR != 0` with just `VAR`
        if let Some(re) = cached_regex(r"let\s+(\w+)\s*=\s*a2r_std::fs::is_dir\(") {
            let bool_vars: Vec<String> = re
                .captures_iter(content.as_str())
                .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                .collect();
            for var in &bool_vars {
                // var is a simple identifier, safe to embed directly
                let pattern_ne = format!(r"if\s+{}\s*!=\s*0\s*\{{", var);
                if let Some(re) = cached_regex(&pattern_ne) {
                    let replacement = format!("if {} {{", var);
                    let new = re
                        .replace_all(content.as_str(), replacement.as_str())
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
                let pattern_eq = format!(r"if\s+{}\s*==\s*0\s*\{{", var);
                if let Some(re) = cached_regex(&pattern_eq) {
                    let replacement = format!("if !{} {{", var);
                    let new = re
                        .replace_all(content.as_str(), replacement.as_str())
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
            }
        }
    }

    /// Fix integer type mismatches (u32 vs i32 vs usize).
    fn fix_integer_type_mismatches(content: &mut String) {
        // Collect u32 and i32 variable names
        let u32_vars: std::collections::HashSet<String> = {
            let mut vars = std::collections::HashSet::new();
            if let Some(re) = cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*:\s*u32\s*=") {
                for caps in re.captures_iter(content.as_str()) {
                    vars.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            if let Some(re) = cached_regex(r"let\s+(\w+)\s*=\s*\(.+?\s+as\s+u32\)") {
                for caps in re.captures_iter(content.as_str()) {
                    vars.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            // Also track struct fields declared as u32 (accessed via self.field)
            if let Some(re) = cached_regex(r"pub\s+(\w+):\s*u32") {
                for caps in re.captures_iter(content.as_str()) {
                    vars.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            vars
        };
        let i32_vars: std::collections::HashSet<String> = {
            let mut vars = std::collections::HashSet::new();
            if let Some(re) = cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*:\s*i32\s*=") {
                for caps in re.captures_iter(content.as_str()) {
                    vars.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            if let Some(re) = cached_regex(r"let\s+(\w+)\s*=\s*\(.+?\s+as\s+i32\)") {
                for caps in re.captures_iter(content.as_str()) {
                    vars.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            // Also track struct fields declared as i32
            if let Some(re) = cached_regex(r"pub\s+(\w+):\s*i32") {
                for caps in re.captures_iter(content.as_str()) {
                    vars.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            vars
        };

        // Fix comparison operators: u32_var op (expr as i64) -> u32_var op (expr as u32)
        for var in &u32_vars {
            for op in &["<=", ">=", "<", ">"] {
                let pattern = format!(
                    r"{}\s*{}\s*\((.+?)\s+as\s+i32\)",
                    regex::escape(var),
                    regex::escape(op)
                );
                if let Some(re) = cached_regex(&pattern) {
                    let new = re
                        .replace_all(content.as_str(), |caps: &regex::Captures| {
                            let expr = caps.get(1).unwrap().as_str();
                            format!("{} {} ({} as u32)", var, op, expr)
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
            }
        }

        // Fix comparisons between u32 and i32 vars: add `as u32` to i32 side
        for uvar in &u32_vars {
            for ivar in &i32_vars {
                for op in &[" < ", " > ", " <= ", " >= "] {
                    let pat = format!("{}{}{}", uvar, op, ivar);
                    let repl = format!("{}{}{} as u32", uvar, op, ivar);
                    *content = content.replace(&pat, &repl);
                }
            }
        }

        // Fix u32 vars used as usize index: vec[u32_var] -> vec[u32_var as usize]
        for var in &u32_vars {
            let pattern = format!(r"\[{}\]", regex::escape(var));
            if let Some(re) = cached_regex(&pattern) {
                let orig = content.clone();
                let new = re
                    .replace_all(content.as_str(), |_caps: &regex::Captures| {
                        format!("[{} as usize]", var)
                    })
                    .to_string();
                if new != orig {
                    *content = new;
                }
            }
        }

        // Fix u32 vars passed where i32 expected (enum variant args)
        let enum_variants_needing_i32 = [
            "ContentBlockStart",
            "ContentBlockDelta",
            "ContentBlockStop",
            "StepStarted",
            "GateWaiting",
            "RunFailed",
        ];
        for variant in &enum_variants_needing_i32 {
            for var in &u32_vars {
                // Pattern: Variant(var, or Variant(var)
                let pat = format!(r"::{}\({},\s*", regex::escape(variant), regex::escape(var));
                if let Some(re) = cached_regex(&pat) {
                    let new = re
                        .replace_all(content.as_str(), |_caps: &regex::Captures| {
                            format!("::{}({} as i32, ", variant, var)
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
                let pat = format!(
                    r"::{}\(\s*{}\s*\)",
                    regex::escape(variant),
                    regex::escape(var)
                );
                if let Some(re) = cached_regex(&pat) {
                    let new = re
                        .replace_all(content.as_str(), |_caps: &regex::Captures| {
                            format!("::{}({} as i64)", variant, var)
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
                // Also handle self.var patterns: Variant(self.var,
                let self_var = format!("self.{}", var);
                let pat = format!(
                    r"::{}\({},\s*",
                    regex::escape(variant),
                    regex::escape(&self_var)
                );
                if let Some(re) = cached_regex(&pat) {
                    let new = re
                        .replace_all(content.as_str(), |_caps: &regex::Captures| {
                            format!("::{}({} as i32, ", variant, self_var)
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
                let pat = format!(
                    r"::{}\(\s*{}\s*\)",
                    regex::escape(variant),
                    regex::escape(&self_var)
                );
                if let Some(re) = cached_regex(&pat) {
                    let new = re
                        .replace_all(content.as_str(), |_caps: &regex::Captures| {
                            format!("::{}({} as i64)", variant, self_var)
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
            }
        }

        // Fix i32 vars passed where u32 expected
        let functions_needing_u32 = ["ensure_tool_call"];
        for func in &functions_needing_u32 {
            for var in &i32_vars {
                let pat = format!(r"{}\({}\)", regex::escape(func), regex::escape(var));
                if let Some(re) = cached_regex(&pat) {
                    let new = re
                        .replace_all(content.as_str(), |_caps: &regex::Captures| {
                            format!("{}({} as u32)", func, var)
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
            }
        }

        // Fix self.u32_field used as usize index: vec[self.field] -> vec[self.field as usize]
        for var in &u32_vars {
            let pattern = format!(r"\[self\.{}\]", regex::escape(var));
            if let Some(re) = cached_regex(&pattern) {
                let orig = content.clone();
                let new = re
                    .replace_all(content.as_str(), |_caps: &regex::Captures| {
                        format!("[self.{} as usize]", var)
                    })
                    .to_string();
                if new != orig {
                    *content = new;
                }
            }
            // Also: .insert(self.u32_field, -> .insert(self.u32_field as usize,
            let pattern = format!(r"\.insert\(self\.{},\s*", regex::escape(var));
            if let Some(re) = cached_regex(&pattern) {
                let orig = content.clone();
                let new = re
                    .replace_all(content.as_str(), |_caps: &regex::Captures| {
                        format!(".insert(self.{} as usize, ", var)
                    })
                    .to_string();
                if new != orig {
                    *content = new;
                }
            }
        }
    }

    /// Add `mut` to `let` bindings that are later reassigned (x.field = ... or x = ...).
    /// Auto variables are mutable by default; Rust requires explicit `mut`.
    fn fix_mutable_bindings(content: &mut String) {
        // Find all `let name = ` bindings (without mut) and check if name.field or name = appears later
        let lines: Vec<&str> = content.lines().collect();
        let mut needs_mut: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Match `let name = ` (without mut)
            if let Some(caps) = cached_regex(r"^let\s+(\w+)\s*=").unwrap().captures(trimmed) {
                let var_name = caps.get(1).unwrap().as_str();
                // Skip if already mut
                if trimmed.starts_with("let mut") {
                    continue;
                }
                // Plan 419 §4.5: Rc<RefCell<T>> 写捕获绑定自带内部可变性,
                // 追加 mut 只会产生 unused_mut 告警。
                if trimmed.contains("std::rc::Rc::new(std::cell::RefCell::new(") {
                    continue;
                }
                // Look ahead for assignments to this variable
                let assign_pat = format!(r"\b{}\s*[.\[]", var_name);
                let direct_pat = format!(r"\b{}\s*=[^=>]", var_name);
                // Methods that take &mut self (require mut binding)
                let mut_methods = [
                    "push", "pop", "insert", "remove", "clear", "next", "extend", "truncate",
                    "retain", "sort", "sort_by", "reverse", "dedup", "swap", "splice", "drain",
                    "append", "resize",
                ];
                if let Some(re) = cached_regex(&assign_pat) {
                    for future_line in lines.iter().skip(i + 1) {
                        // Stop at function boundary
                        let fl = future_line.trim();
                        if fl.starts_with("pub fn ")
                            || fl.starts_with("fn ")
                            || fl.starts_with("pub async fn ")
                            || fl.starts_with("async fn ")
                        {
                            break;
                        }
                        if re.is_match(fl) {
                            // Check if it's an actual assignment: var.field = or var[idx] =
                            let field_assign = format!(r"\b{}\.\w+\s*=", var_name);
                            let idx_assign = format!(r"\b{}\[[^\]]*\]\s*=", var_name);
                            if let Some(re2) = cached_regex(&field_assign) {
                                if re2.is_match(fl) {
                                    needs_mut.insert(i);
                                    break;
                                }
                            }
                            if let Some(re2) = cached_regex(&idx_assign) {
                                if re2.is_match(fl) {
                                    needs_mut.insert(i);
                                    break;
                                }
                            }
                            // Check for &mut self method calls: var.push(...), var.insert(...), etc.
                            for method in &mut_methods {
                                let method_pat = format!(r"\b{}\.{}\s*\(", var_name, method);
                                if let Some(re3) = cached_regex(&method_pat) {
                                    if re3.is_match(fl) {
                                        needs_mut.insert(i);
                                        break;
                                    }
                                }
                            }
                            if needs_mut.contains(&i) {
                                break;
                            }
                        }
                    }
                }
                if let Some(re) = cached_regex(&direct_pat) {
                    for future_line in lines.iter().skip(i + 1) {
                        let fl = future_line.trim();
                        if fl.starts_with("pub fn ")
                            || fl.starts_with("fn ")
                            || fl.starts_with("pub async fn ")
                            || fl.starts_with("async fn ")
                        {
                            break;
                        }
                        if re.is_match(fl) && !fl.starts_with(&format!("let {}", var_name)) {
                            // Exclude == and !=
                            if let Some(eq_check) =
                                cached_regex(&format!(r"\b{}\s*=[^=>]", var_name))
                            {
                                if eq_check.is_match(fl) {
                                    needs_mut.insert(i);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if needs_mut.is_empty() {
            return;
        }

        let new_lines: Vec<String> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if needs_mut.contains(&i) {
                    line.replacen("let ", "let mut ", 1)
                } else {
                    line.to_string()
                }
            })
            .collect();
        *content = new_lines.join("\n");
    }

    /// B16b: Add `mut` to fn params that are mutated inside the body.
    ///
    /// a2r's `fix_mutable_bindings` only covers `let` locals; Rust fn params
    /// are immutable bindings, so mutating a param (e.g. `seen.insert(..)`,
    /// `names.push(..)`, `param.field = x`) is E0596. Detect mutated params
    /// and prefix the declaration with `mut `.
    fn fix_mutable_params(content: &mut String) {
        let mut_methods: &[&str] = &[
            "push", "pop", "insert", "remove", "clear", "next", "extend", "truncate", "retain",
            "sort", "sort_by", "reverse", "dedup", "swap", "splice", "drain", "append", "resize",
            "set", "update", "merge",
        ];
        let lines: Vec<&str> = content.lines().collect();
        let mut result: Vec<String> = lines.iter().map(|l| l.to_string()).collect();

        for (i, line) in lines.iter().enumerate() {
            let sig = line.trim();
            let is_sig = sig.starts_with("pub fn ")
                || sig.starts_with("pub async fn ")
                || sig.starts_with("async fn ")
                || sig.starts_with("fn ")
                || sig.starts_with("pub(crate) fn ")
                || sig.starts_with("unsafe fn ");
            if !is_sig {
                continue;
            }

            let open = match line.find('(') {
                Some(p) => p,
                None => continue,
            };
            // ')' that precedes the return arrow / body — params span
            // (..) up to the first ')'; a2r emits one-line signatures so
            // rfind(')') is safe unless an arrow type contains ')'. Prefer
            // the first ')' that is followed (after whitespace) by "->" or "{".
            let rest = &line[open + 1..];
            let close = rest.find(')');
            let close = match close {
                Some(c) => open + 1 + c,
                None => continue,
            };

            // Split params at depth-0 commas.
            let params_str = &line[open + 1..close];
            let mut params: Vec<&str> = Vec::new();
            let mut depth = 0i32;
            let mut cur_start = 0usize;
            for (k, ch) in params_str.char_indices() {
                match ch {
                    '<' | '[' | '(' => depth += 1,
                    '>' | ']' | ')' => depth -= 1,
                    ',' if depth == 0 => {
                        let p = params_str[cur_start..k].trim();
                        if !p.is_empty() {
                            params.push(p);
                        }
                        cur_start = k + 1;
                    }
                    _ => {}
                }
            }
            let last_p = params_str[cur_start..].trim();
            if !last_p.is_empty() {
                params.push(last_p);
            }

            // Locate the end of the body via brace balance. i64 so lines with
            // more `}` than `{` (block closes) can't underflow. Trait/impl
            // method DECLARATIONS without a body (`fn name(&self) -> String;`)
            // have no `{` — skip them entirely.
            if !line.contains('{') {
                continue;
            }
            let mut brace: i64 = line.bytes().filter(|b| *b == b'{').count() as i64
                - line.bytes().filter(|b| *b == b'}').count() as i64;
            let mut j = i;
            while brace > 0 && j + 1 < lines.len() {
                j += 1;
                brace += lines[j].bytes().filter(|b| *b == b'{').count() as i64;
                brace -= lines[j].bytes().filter(|b| *b == b'}').count() as i64;
            }
            // One-liner (`fn f(mut_me: Vec<String>) { mut_me.push(x); }`) has
            // brace 0 → body is just the signature line; otherwise the lines
            // between the sig and the balanced `}`.
            let body = if j > i {
                &lines[i + 1..=j.min(lines.len() - 1)]
            } else {
                &lines[i..=i]
            };

            // For each plain `name: Type` param, check for mutation in body.
            let mut mutated: Vec<String> = Vec::new();
            for p in params {
                // Skip receiver forms (&self / &mut self / self) and bindings
                // without a type annotation (e.g. `_`, `mut x`? a2r never
                // emits `mut` on params, so only plain forms appear).
                let name = match p.split_once(':') {
                    Some((n, _)) => n.trim().to_string(),
                    None => continue,
                };
                let name = name
                    .trim_start_matches("&")
                    .trim_start_matches("mut ")
                    .trim()
                    .to_string();
                if name.is_empty() || name == "self" {
                    continue;
                }

                let mut is_mut = false;
                // Plan 032 G2.1 (auto-ai consumer debt): any method call on an
                // OWNED non-primitive param conservatively needs `mut` — whether
                // the method takes `&mut self` is unknowable at transpile time
                // (custom types like `eng.push_gate_feedback(...)` never hit the
                // builtin mutating-method list). Only for owned value params:
                // uppercase type name (local struct/String/Vec/Box<dyn>…),
                // excluding references (&T) and shared handles (Arc/Rc), where a
                // mut binding could never help and would only add noise.
                let owned_value_type = {
                    let t = p.split_once(':').map(|(_, t)| t.trim()).unwrap_or("");
                    t.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                        && !t.starts_with('&')
                        && !t.starts_with("Arc<")
                        && !t.starts_with("Rc<")
                };
                for bl in body {
                    let fl = bl.trim();
                    if owned_value_type {
                        let any_call = format!(r"\b{}\.\w+\s*\(", name);
                        if let Some(re) = cached_regex(&any_call) {
                            if re.is_match(fl) {
                                is_mut = true;
                                break;
                            }
                        }
                    }
                    // name.push/insert/... (mutating method calls)
                    for m in mut_methods {
                        let pat = format!(r"\b{}\.{}\s*\(", name, m);
                        if let Some(re) = cached_regex(&pat) {
                            if re.is_match(fl) {
                                is_mut = true;
                                break;
                            }
                        }
                    }
                    if is_mut {
                        break;
                    }
                    // name.field = ... (field assignment through the param)
                    let field_assign = format!(r"\b{}\.\w+\s*=", name);
                    if let Some(re) = cached_regex(&field_assign) {
                        if re.is_match(fl) {
                            is_mut = true;
                            break;
                        }
                    }
                    // name[idx] = ... (indexed assignment)
                    let idx_assign = format!(r"\b{}\[[^\]]*\]\s*=", name);
                    if let Some(re) = cached_regex(&idx_assign) {
                        if re.is_match(fl) {
                            is_mut = true;
                            break;
                        }
                    }
                    // name = ... (direct reassignment; exclude == and `let name`;
                    // also `=>` — match-arm arrows like `_guard if a == b =>`
                    // end with `<param> =>` and falsely read as assignment,
                    // 447-③ Y1 t_unify `mut b` 噪声实证)
                    let direct = format!(r"\b{}\s*=[^=>]", name);
                    if let Some(re) = cached_regex(&direct) {
                        if re.is_match(fl) && !fl.starts_with("let ") {
                            is_mut = true;
                            break;
                        }
                    }
                }
                if is_mut {
                    mutated.push(name);
                }
            }

            if mutated.is_empty() {
                continue;
            }
            // Rewrite the params span: `name: Type` → `mut name: Type`.
            let mut new_params = params_str.to_string();
            for name in &mutated {
                let marker = format!("{}:", name);
                let marker_mut = format!("mut {}:", name);
                if let Some(pos) = new_params.find(&marker) {
                    // Only replace if not already `mut `-prefixed.
                    let before = &new_params[..pos];
                    if !before.ends_with("mut ") {
                        new_params = new_params.replacen(&marker, &marker_mut, 1);
                    }
                }
            }
            result[i] = format!("{}{}{}", &line[..open + 1], new_params, &line[close..]);
        }

        let joined = result.join("\n");
        if joined != *content {
            *content = joined;
        }
    }

    /// Fix common borrowing issues:
    /// 1. `.insert(key, &vec_var)` → `.insert(key, vec_var.clone())`
    /// 2. `.field = &var` where field is Vec/struct → `.field = var.clone()`
    /// 3. map.get(X).unwrap_or(vec![]) → map.get(X).cloned().unwrap_or_default()
    /// 4. let var = map.get(X).unwrap_or(default) → needs .cloned()
    fn fix_borrowing_issues(content: &mut String) {
        // Fix: map.get(X).unwrap_or(vec![]) → map.get(X).cloned().unwrap_or_default()
        // Also: map.get(X).unwrap_or(&[]) → map.get(X).cloned().unwrap_or_default()
        if let Some(re) = cached_regex(r"\.get\(([^)]+)\)\.unwrap_or\(vec!\[\]\)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let key = caps.get(1).unwrap().as_str();
                    format!(".get({}).cloned().unwrap_or_default()", key)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Fix: .get(X).unwrap_or(&[]) → .get(X).cloned().unwrap_or_default()
        if let Some(re) = cached_regex(r"\.get\(([^)]+)\)\.unwrap_or\(&\[\]\)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let key = caps.get(1).unwrap().as_str();
                    format!(".get({}).cloned().unwrap_or_default()", key)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Fix: let var = map.get(X).unwrap_or(vec![...]) → add .cloned()
        if let Some(re) =
            cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*=\s*(\w+\.get\([^)]+\))\.unwrap_or\(vec!")
        {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let var = caps.get(1).unwrap().as_str();
                    let get_expr = caps.get(2).unwrap().as_str();
                    format!("let mut {} = {}.cloned().unwrap_or(vec!", var, get_expr)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Fix: map.insert(key, &variable) → map.insert(key, variable.clone())
        if let Some(re) = cached_regex(r"\.insert\(([^,]+),\s+&(\w+)\)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let key = caps.get(1).unwrap().as_str();
                    let var = caps.get(2).unwrap().as_str();
                    format!(".insert({}, {}.clone())", key, var)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Fix: .field = &variable; → .field = variable.clone();
        if let Some(re) = cached_regex(r"(\.\w+)\s*=\s+&(\w+);") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let field = caps.get(1).unwrap().as_str();
                    let var = caps.get(2).unwrap().as_str();
                    format!("{} = {}.clone();", field, var)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Fix: .push(&variable) → .push(variable.clone())
        if let Some(re) = cached_regex(r"\.push\(&(\w+)\)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let var = caps.get(1).unwrap().as_str();
                    format!(".push({}.clone())", var)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix vec![(str, str, str)] where the return type is Vec<(String, String, String)>.
    /// Adds .to_string() to string literals inside tuples in vec![] macros.
    fn fix_vec_tuple_string_literals(content: &mut String) {
        // Strategy: find vec![ ... ]; regions and add .to_string() to bare string literals.
        // Track paren depth — inside function call args, don't add .to_string().
        // Heuristic: if ( is preceded by an identifier (Name(), Type::method()), it's a function call.
        // If ( is preceded by , or [ or (, it's a tuple — those still need .to_string().
        let bytes = content.as_bytes();
        let len = bytes.len();
        let mut result = Vec::new();
        let mut i = 0;
        let mut in_vec = false;
        let mut vec_depth = 0;
        let mut paren_depth: i32 = 0;
        let mut func_paren_depths: std::collections::HashSet<i32> =
            std::collections::HashSet::new();

        while i < len {
            // Plan 523 W3(path4 首曝的既有洞):外层扫描须跳过双引号字符串
            // 字面量——字面量内的文本 "vec!["(AA2R ar_array_literal 的 out
            // 初值 "vec![")会伪触发 vec 域,后续所有字符串被误插
            // .to_string()(s8 复现:format!("{}, ", out) →
            // format!(".to_string(){}, ", out);既有自 511 期,AA2R-in-bin
            // 首次运行时触达)。
            if !in_vec && bytes[i] == b'"' {
                let start = i;
                i += 1;
                while i < len && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                result.extend_from_slice(&content[start..i].as_bytes());
                continue;
            }
            if !in_vec && i + 5 <= len && &bytes[i..i + 5] == b"vec![" {
                in_vec = true;
                vec_depth = 1;
                paren_depth = 0;
                func_paren_depths.clear();
                result.extend_from_slice(b"vec![");
                i += 5;
                continue;
            }

            if in_vec {
                match bytes[i] {
                    b'[' => {
                        vec_depth += 1;
                        result.push(b'[');
                        i += 1;
                        continue;
                    }
                    b']' => {
                        vec_depth -= 1;
                        result.push(b']');
                        i += 1;
                        if vec_depth == 0 {
                            in_vec = false;
                        }
                        continue;
                    }
                    b'(' => {
                        // Check if this ( is a function call: preceded by identifier or ::
                        let before = content[..i].trim_end();
                        let is_func = before
                            .chars()
                            .last()
                            .map(|c| c.is_alphanumeric() || c == '_' || c == ':')
                            .unwrap_or(false);
                        if is_func {
                            func_paren_depths.insert(paren_depth + 1);
                        }
                        paren_depth += 1;
                        result.push(b'(');
                        i += 1;
                        continue;
                    }
                    b')' => {
                        func_paren_depths.remove(&(paren_depth));
                        paren_depth -= 1;
                        result.push(b')');
                        i += 1;
                        continue;
                    }
                    b'"' => {
                        let start = i;
                        i += 1;
                        while i < len && bytes[i] != b'"' {
                            if bytes[i] == b'\\' {
                                i += 1;
                            }
                            i += 1;
                        }
                        if i < len {
                            i += 1;
                        }
                        let lit = &content[start..i];
                        let rest = &content[i..];
                        let already_has = rest.trim_start().starts_with(".to_string()");
                        let inside_func_call = func_paren_depths.contains(&paren_depth);
                        if inside_func_call || already_has {
                            result.extend_from_slice(lit.as_bytes());
                        } else {
                            result.extend_from_slice(lit.as_bytes());
                            result.extend_from_slice(b".to_string()");
                        }
                        continue;
                    }
                    _ => {
                        result.push(bytes[i]);
                        i += 1;
                        continue;
                    }
                }
            }

            result.push(bytes[i]);
            i += 1;
        }

        let new = String::from_utf8(result).unwrap_or_else(|_| content.clone());
        if new != *content {
            *content = new;
        }
    }

    /// Fix E0382 move errors when pushing a variable that's reused later.
    /// Pattern: `vec.push(var)` where var is a `let var = expr.clone()` or loop variable
    /// that gets reassigned in the next iteration.
    /// Solution: Add .clone() to the push argument.
    fn fix_push_move(content: &mut String) {
        // Pattern: within a while loop, a variable declared as `let var = collection[i].clone()`
        // is pushed to a vec and then reassigned in the next iteration.
        // The push needs .clone() if the var is used after the push.

        // Strategy: find lines like `result.push(var)` or `goals.push(s)` where
        // the pushed variable is a local binding used after the push.
        // We check if the same variable appears on a later line in the same scope.

        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::with_capacity(content.len());
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            // Check for push patterns: `something.push(varname)`
            if let Some(_rest) = line.trim().strip_suffix(")") {
                // Match `something.push(varname)` or `something.push(varname.field)`
                if let Some(re) = cached_regex(r"^(\s*\S+\.push\()(\w+)(\))$") {
                    if let Some(caps) = re.captures(line) {
                        let prefix = caps.get(1).unwrap().as_str();
                        let var = caps.get(2).unwrap().as_str();
                        let suffix = caps.get(3).unwrap().as_str();

                        // Skip if already has .clone()
                        if prefix.contains(".clone()") {
                            result.push_str(line);
                            result.push('\n');
                            i += 1;
                            continue;
                        }

                        // Check if var is used after this line in the same or nearby scope
                        let mut var_used_again = false;
                        let indent = line.len() - line.trim_start().len();
                        for j in (i + 1)..std::cmp::min(i + 20, lines.len()) {
                            let later = lines[j];
                            // Stop at lines with less or equal indentation that are closing braces or new statements
                            let later_indent = later.len() - later.trim_start().len();
                            if later.trim().starts_with('}') && later_indent <= indent {
                                break;
                            }
                            // Check if var appears as a standalone identifier (not just substring)
                            // Simple heuristic: var followed by . or = or ( or [ or , or )
                            if let Some(var_re) =
                                cached_regex(&format!(r"\b{}\b", regex::escape(var)))
                            {
                                if var_re.is_match(later) {
                                    // Exclude the case where var appears in the same push
                                    if !later.contains(&format!(".push({})", var)) {
                                        var_used_again = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if var_used_again {
                            result.push_str(&format!("{}{}.clone(){}\n", prefix, var, suffix));
                            i += 1;
                            continue;
                        }
                    }
                }
            }

            result.push_str(line);
            result.push('\n');
            i += 1;
        }

        if result != *content {
            // Remove trailing newline if original didn't have one
            if !content.ends_with('\n') && result.ends_with('\n') {
                result.pop();
            }
            *content = result;
        }
    }

    /// Plan 373/393 E2: `Result<None, E>` → `Result<(), E>` (type position, always),
    /// and `Ok(None)` → `Ok(())` **only inside functions whose return type is
    /// `Result<(), _>`**. Functions returning `Result<Option<T>, _>` legitimately
    /// use `Ok(None)` (success-but-no-value), so the global replace was an E0308
    /// bug. We track brace depth to bound each fn body.
    fn fix_result_none_unit(content: &mut String) {
        // Type position: `Result<None,` or `Result<None>` → unit first arg (always safe).
        let reduced = content
            .replace("Result<None,", "Result<(),")
            .replace("Result<None>", "Result<(),>");
        if reduced != *content {
            *content = reduced;
        }

        // Value position: only replace `Ok(None)` → `Ok(())` inside fn bodies
        // whose signature returns `Result<(), _>` (Ok-type is unit). Walk the
        // content line by line, tracking fn boundaries + brace depth.
        let lines: Vec<&str> = content.lines().collect();
        let mut out = String::with_capacity(content.len());
        // Regex to detect a fn returning Result<(), ...> (Ok-type is `()` or `None`
        // already normalized to `()` above). Match `-> Result<(),` or `-> Result<()>`.
        let unit_fn_re = cached_regex(r"fn\s+\w+[^{]*->\s*Result\s*<\s*\(\s*\)\s*,");
        let mut in_unit_fn = false;
        let mut depth: i32 = 0;
        let mut fn_header = String::new(); // accumulate multi-line fn signature
        let mut header_done = false;
        for line in &lines {
            if !in_unit_fn {
                // Detect fn start. Signatures may span lines, so accumulate until `{`.
                if !header_done && line.contains("fn ") {
                    fn_header.clear();
                    fn_header.push_str(line);
                    if line.contains('{') {
                        header_done = true;
                    } else {
                        out.push_str(line);
                        out.push('\n');
                        continue;
                    }
                } else if !header_done && !fn_header.is_empty() {
                    fn_header.push('\n');
                    fn_header.push_str(line);
                    if line.contains('{') {
                        header_done = true;
                    } else {
                        out.push_str(line);
                        out.push('\n');
                        continue;
                    }
                }

                if header_done {
                    if unit_fn_re
                        .as_ref()
                        .map(|r| r.is_match(&fn_header))
                        .unwrap_or(false)
                    {
                        in_unit_fn = true;
                        depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                        if depth <= 0 {
                            in_unit_fn = false;
                        }
                    }
                    header_done = false;
                    fn_header.clear();
                    out.push_str(line);
                    out.push('\n');
                    continue;
                }
                out.push_str(line);
                out.push('\n');
            } else {
                // Inside a Result<(), _> fn body: replace Ok(None) → Ok(()).
                let replaced = line.replace("Ok(None)", "Ok(())");
                out.push_str(&replaced);
                out.push('\n');
                depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
                if depth <= 0 {
                    in_unit_fn = false;
                }
            }
        }
        // Trim trailing extra newline added by the loop.
        if out.ends_with('\n') && !content.ends_with('\n') {
            out.pop();
        }
        if out != *content {
            *content = out;
        }
    }

    /// Plan 373: calls on function-typed struct fields need parenthesization.
    /// Auto stores callbacks as fields and calls them as `self.field(args)`;
    /// Rust requires `(self.field)(args)` when the field is a `fn(...)` type.
    /// Heuristic: scan struct definitions for `field: fn(...) -> ...` and rewrite
    /// `self.field(` calls into `(self.field)(`.
    fn fix_fn_field_calls(content: &mut String) {
        // Collect field names declared as function types in structs.
        let fn_fields: std::collections::HashSet<String> = {
            let mut set = std::collections::HashSet::new();
            // `field: fn(Args) -> Ret` or `field ?fn(Args) Ret` (Auto lowered)
            if let Some(re) = cached_regex(r"\b(\w+):\s*fn\s*\(") {
                for caps in re.captures_iter(content.as_str()) {
                    set.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            if let Some(re) = cached_regex(r"\b(\w+):\s*\?fn\s*\(") {
                for caps in re.captures_iter(content.as_str()) {
                    set.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            set
        };
        for field in &fn_fields {
            // self.field(args) → (self.field)(args)  (avoid double-wrap)
            // Plan 376V: also match without space (self.field(args) directly)
            let pat = format!(r"self\.{}\s*\(", regex::escape(field));
            if let Some(re) = cached_regex(&pat) {
                let fld = field.clone();
                let new = re
                    .replace_all(content.as_str(), |_caps: &regex::Captures| {
                        format!("(self.{})(", fld)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
            // plain `field(args)` calls (non-self receiver) — also wrap
            let pat2 = format!(r"\b{} \(", regex::escape(field));
            if let Some(re) = cached_regex(&pat2) {
                let fld = field.clone();
                let new = re
                    .replace_all(content.as_str(), |_caps: &regex::Captures| {
                        format!("({})(", fld)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
        }
    }

    /// Plan 373: relax `#[derive(... Eq/PartialOrd/Ord ...)]` on enums/structs
    /// whose variants/fields hold a non-Ord type. The generator can't always
    /// tell that a variant carries `serde_json::Value`, `HashMap`, a foreign
    /// crate type (e.g. `auto_ai_client::Message`, `ai_config::ModelTier`), or
    /// another non-Ord local type — so it emits the full derive set, which then
    /// fails to compile (E0277). Downgrade those to `Clone, Debug, PartialEq`.
    /// (No transpiled type is ever used as a BTreeMap key or sort key, so
    /// dropping Ord/PartialOrd/Eq is always safe here.)
    // ========================================================================
    // PLAN-009 T1 (F1, 006 S2): auto-derive restriction machinery.
    //
    // a2r auto-adds comparison derives to types without explicit attrs. When
    // such a container references a type whose EXPLICIT derive set lacks a
    // trait (e.g. `#[derive(Clone, Copy, Debug)] struct GridSize`), the
    // emitted derive fails rustc with E0369. The handlers intersect the
    // shape-derived candidate set with the constraint masks below.
    // ========================================================================

    fn fix_non_ord_derives(content: &mut String, ord_restricted: &HashSet<AutoStr>) {
        // Markers that indicate a non-Ord payload. If any appears between a
        // derive line and the end of the type body, downgrade the derive.
        // We scan the two lines that often follow the derive: the type kind line
        // and its field/variant lines up to the closing brace.
        let non_ord_markers = [
            "JsonValue",
            "serde_json::Value",
            "Value",
            "HashMap",
            "BTreeMap",
            "Box<dyn",
            "Message",
            "ClientError",
            "ToolError",
            "HandoffDocument",
            "AgentError",
            "AgentResult",
            "ToolCallRecord",
            "RoleConfig",
            // Plan 016 Phase A A6: ModelTier removed — it IS Ord (fieldless enum,
            // and rust-ref tier.rs now derives PartialOrd+Ord).
            // RoleConfig added: it contains Option<f64> (non-Ord) and is defined
            // cross-file in auto-ai-agent (role_config.at), so the single-file
            // derived-marker collection can't see it from roles.at.
        ];
        // (?s) dotall: capture derive + whole body up to matching closing brace.
        // We approximate the body as everything up to the first "\n}\n" at column 0.
        let re_str = r"(?s)(#\[derive\(([^)]*)\)\]\n)(pub )?(enum|struct) (\w+) \{(.*?)\n\}";
        if let Some(re) = cached_regex(re_str) {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let derive_attr = caps.get(1).unwrap().as_str();
                    let derives = caps.get(2).unwrap().as_str();
                    let body = caps.get(6).unwrap().as_str();
                    let has_non_ord = non_ord_markers.iter().any(|m| body.contains(m));
                    if !has_non_ord {
                        return caps.get(0).unwrap().as_str().to_string();
                    }
                    let needs_relax = derives.contains("Ord")
                        || derives.contains("PartialOrd")
                        || derives.contains("Eq");
                    if !needs_relax {
                        return caps.get(0).unwrap().as_str().to_string();
                    }
                    // Rebuild the type with a relaxed derive.
                    let kind_kw = caps.get(4).unwrap().as_str();
                    let pub_kw = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                    let name = caps.get(5).unwrap().as_str();
                    format!(
                        "#[derive(Clone, Debug, PartialEq)]\n{}{} {} {{{}\n}}",
                        pub_kw, kind_kw, name, body
                    )
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Plan 016 Phase A A6 (upgrade pass): structs that were conservatively
        // downgraded to PartialEq (because they have an enum field — see
        // has_enum_field in type_decl) but whose body contains NO non-Ord
        // marker can safely UPGRADE to the full derive set. This restores
        // Eq/Ord for structs like ModelDefinition { tier: ModelTier } where
        // ModelTier is a fieldless (Ord-safe) enum. Only structs (not enums):
        // enums derive Eq/Ord via payload_is_eq_safe in enum_decl, and a
        // PartialEq enum was deliberately downgraded because its payload is
        // genuinely non-Ord (e.g. ContentBlock carries JsonValue).
        // We also propagate non-Ord-ness: collect names of types whose body
        // contains a base marker, then treat those names as markers too, so a
        // struct containing a non-Ord enum (e.g. Message { content: Vec<ContentBlock> })
        // is NOT upgraded.
        let upgrade_re_str =
            r"(?s)(#\[derive\(Clone, Debug, PartialEq\)\]\n)(pub )?struct (\w+) \{(.*?)\n\}";
        if let Some(re) = cached_regex(upgrade_re_str) {
            // First pass: collect derived markers (type names whose own body
            // contains a base non_ord marker — these are non-Ord enums/structs
            // that should block upgrade of anything referencing them).
            let mut derived_markers: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let collect_re_str = r"(?s)(?:enum|struct) (\w+) \{(.*?)\n\}";
            if let Some(collect_re) = cached_regex(collect_re_str) {
                for caps in collect_re.captures_iter(content.as_str()) {
                    let name = caps.get(1).unwrap().as_str();
                    let body = caps.get(2).unwrap().as_str();
                    if non_ord_markers.iter().any(|m| body.contains(m)) {
                        derived_markers.insert(name.to_string());
                    }
                }
            }
            // Plan 016 Phase A A3: also collect names of structs whose derive
            // is PartialEq-only (no Eq/Ord) — these are non-Ord (e.g. contain
            // f64, or were downgraded by type_decl's has_float_field/has_map_field).
            // A struct referencing such a type must not be upgraded to Ord.
            if let Some(partial_eq_re) = cached_regex(
                r"(?s)#\[derive\(Clone, Debug, PartialEq\)\]\n(?:pub )?struct (\w+) \{",
            ) {
                for caps in partial_eq_re.captures_iter(content.as_str()) {
                    if let Some(m) = caps.get(1) {
                        derived_markers.insert(m.as_str().to_string());
                    }
                }
            }
            let all_markers: Vec<String> = non_ord_markers
                .iter()
                .map(|s| s.to_string())
                .chain(derived_markers.iter().cloned())
                // PLAN-009 T1 (F1): types whose comparison derives were
                // restricted at the AST level (explicit-derive surfaces) are
                // non-Eq/Ord by construction — treat them as markers so the
                // upgrade pass never re-widens them nor anything referencing
                // them.
                .chain(ord_restricted.iter().map(|s| s.as_str().to_string()))
                .collect();

            let new = re.replace_all(content.as_str(), |caps: &regex::Captures| {
                let body = caps.get(4).unwrap().as_str();
                // If body references any non-Ord marker (base or derived),
                // keep the conservative PartialEq.
                if all_markers.iter().any(|m| body.contains(m.as_str())) {
                    return caps.get(0).unwrap().as_str().to_string();
                }
                // Plan 016 Phase A A3: float fields (f32/f64) are non-Ord.
                // type_decl downgrades them to PartialEq via has_float_field,
                // but this text pass doesn't see types — check for float type
                // names in the body to avoid upgrading float-bearing structs
                // (which would fail E0277 "f64: Ord not satisfied").
                if body.contains("f64") || body.contains("f32")
                    || body.contains("float") || body.contains("double")
                {
                    return caps.get(0).unwrap().as_str().to_string();
                }
                // Safe to upgrade: no non-Ord payload detected.
                let pub_kw = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let name = caps.get(3).unwrap().as_str();
                format!("#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]\n{}struct {} {{{}\n}}",
                        pub_kw, name, body)
            }).to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376 Pass 3: Fix `Some(ident)` where target is Option<String>.
    /// Adds `.to_string()` to the inner value.
    fn fix_some_str_to_string(content: &mut String) {
        // Add .to_string() to `self.field = Some(ident)` ONLY when the payload
        // ident is str-typed IN THE SAME FUNCTION. The field is Option<String>
        // in that case and Rust won't coerce `Some(&str)` → `Some(String)`.
        //
        // Plan 016 Phase A A7a: the str-ident set must be FUNCTION-SCOPED, not
        // file-global. A file-global set caused cross-function name collisions
        // (e.g. `fn text(t: &str)` poisoned `Some(t)` in an unrelated
        // `fn with_temperature(t: f64)`, inserting a bogus `.to_string()` on a
        // float → E0308 Option<f64> vs Option<String>). We split the content
        // into function chunks and process each independently.
        let re = cached_regex(r"(self\.\w+\s*=\s*Some\()(\w+)(\))");
        if let Some(re) = re {
            // Split into function chunks. A function starts at a line matching
            // `fn ` (at any indent) and ends right before the next such line or
            // EOF. We keep the boundaries so reconstruction is byte-faithful.
            let lines: Vec<&str> = content.lines().collect();
            let mut chunk_starts: Vec<usize> = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("fn ")
                    || trimmed.starts_with("pub async fn ")
                    || trimmed.starts_with("async fn ")
                {
                    chunk_starts.push(i);
                }
            }
            chunk_starts.push(lines.len()); // sentinel end

            let mut out_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            for c in 0..chunk_starts.len() - 1 {
                let start = chunk_starts[c];
                let end = chunk_starts[c + 1];
                let chunk = &lines[start..end];

                // Collect str-typed idents WITHIN this function chunk only.
                let mut str_idents = std::collections::HashSet::new();
                let patterns = [
                    r"\b(\w+):\s*&str\b",
                    r"\b(\w+):\s*String\b",
                    r"let (?:mut )?(\w+):\s*(?:&str|String)\b",
                ];
                for pat in &patterns {
                    if let Some(pr) = cached_regex(pat) {
                        for cl in chunk {
                            for caps in pr.captures_iter(cl) {
                                if let Some(m) = caps.get(1) {
                                    str_idents.insert(m.as_str().to_string());
                                }
                            }
                        }
                    }
                }

                // Apply the Some(ident) → Some(ident.to_string()) rewrite in-chunk.
                for (i, cl) in chunk.iter().enumerate() {
                    if !cl.contains("= Some(") {
                        continue;
                    }
                    let rewritten = re.replace(cl, |caps: &regex::Captures| {
                        let prefix = caps.get(1).unwrap().as_str();
                        let ident = caps.get(2).unwrap().as_str();
                        let suffix = caps.get(3).unwrap().as_str();
                        if !str_idents.contains(ident) {
                            return format!("{}{}{}", prefix, ident, suffix);
                        }
                        format!("{}{}.to_string(){}", prefix, ident, suffix)
                    });
                    out_lines[start + i] = rewritten.to_string();
                }
            }
            let new = out_lines.join("\n") + "\n";
            if new != *content {
                *content = new;
            }
        }
        let re2 = cached_regex(r#"(self\.\w+\s*=\s*Some\()("(?:[^"\\]|\\.)*")(\))"#);
        if let Some(re2) = re2 {
            let new = re2
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    format!(
                        "{}{}.to_string(){}",
                        caps.get(1).unwrap().as_str(),
                        caps.get(2).unwrap().as_str(),
                        caps.get(3).unwrap().as_str()
                    )
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376V: PathBuf has no .as_str() (unstable feature). a2r's auto-borrow
    /// adds .as_str() when passing to &str params, but for PathBuf variables this
    /// fails E0599. Convert <pathlike>.as_str() → <pathlike>.to_str().unwrap().
    ///
    /// Plan 016 Phase A A.4: type-aware — only rewrite when the variable is
    /// genuinely PathBuf. The old name-heuristic (_path/path/dir/sidecar) wrongly
    /// rewrote String variables named `aaid_path`/`path`. Now we scan for PathBuf
    /// declarations first: explicit annotations, PathBuf::from, .join() chains
    /// from PathBuf, or fn params typed PathBuf.
    fn fix_pathbuf_as_str(content: &mut String) {
        // Collect names of variables known to be PathBuf.
        let mut pathbuf_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Pattern 1: let X: PathBuf = ...  or  fn f(X: PathBuf)
        if let Some(re) = cached_regex(r"\b(\w+):\s*(?:std::path::)?PathBuf\b") {
            for caps in re.captures_iter(content.as_str()) {
                if let Some(m) = caps.get(1) {
                    pathbuf_vars.insert(m.as_str().to_string());
                }
            }
        }
        // Pattern 2: let X = PathBuf::from(...) or std::path::PathBuf::from(...)
        if let Some(re) =
            cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*=\s*(?:std::path::)?PathBuf::from")
        {
            for caps in re.captures_iter(content.as_str()) {
                if let Some(m) = caps.get(1) {
                    pathbuf_vars.insert(m.as_str().to_string());
                }
            }
        }
        // Pattern 3: let X = Y.join(...) — in Rust, .join() on a path-like
        // returns PathBuf (PathBuf::join). Auto's str.join is not a thing in
        // the transpiled output, so any .join() result is PathBuf.
        // (Plan 016 Phase A A.4: don't require rhs to be known PathBuf —
        //  agent's `let path = home.join(...)` where home comes from home_dir()
        //  still produces a PathBuf path.)
        if let Some(re) = cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*=\s*\w+\.join\(") {
            for caps in re.captures_iter(content.as_str()) {
                if let Some(m) = caps.get(1) {
                    pathbuf_vars.insert(m.as_str().to_string());
                }
            }
        }
        // Pattern 4: is fn { Some(X) => ... } where fn returns ?PathBuf —
        // detect by the match arm binding X and fn being home_dir/find_aaid etc.
        // (Skip: too fragile for a text pass. Agent's validate.at path comes
        //  from home_dir().join() which Pattern 3 covers if home_dir's return
        //  is tracked — but it isn't at text level. Keep the name heuristic as
        //  a FALLBACK only for names NOT seen as String.)
        // Collect known String/str vars to EXCLUDE from the name heuristic.
        let mut string_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Some(re) = cached_regex(r"\b(\w+):\s*(?:std::string::)?String\b") {
            for caps in re.captures_iter(content.as_str()) {
                if let Some(m) = caps.get(1) {
                    string_vars.insert(m.as_str().to_string());
                }
            }
        }
        // Plan 016 Phase A A.4: also &str params (a name shared between a match
        // arm binding and a &str fn param in the same file means the name is
        // string-like, not PathBuf).
        if let Some(re) = cached_regex(r"\b(\w+):\s*&str\b") {
            for caps in re.captures_iter(content.as_str()) {
                if let Some(m) = caps.get(1) {
                    string_vars.insert(m.as_str().to_string());
                }
            }
        }
        if let Some(re) =
            cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*=\s*(?:std::path::)?PathBuf::from")
        {
            for caps in re.captures_iter(content.as_str()) {
                if let Some(m) = caps.get(1) {
                    string_vars.remove(&m.as_str().to_string());
                }
            }
        }

        // Now rewrite: for each <name>.as_str(), rewrite to .to_str().unwrap()
        // only if name is in pathbuf_vars OR (matches the old heuristic AND is
        // NOT a known String var).
        if let Some(re) = cached_regex(r"(\b\w*_path|\bpath|\bdir|\bsidecar)\.as_str\(\)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let name = caps.get(1).unwrap().as_str();
                    // Definitely PathBuf → rewrite.
                    if pathbuf_vars.contains(name) {
                        return format!("{}.to_str().unwrap()", name);
                    }
                    // Known String → do NOT rewrite (keep .as_str()).
                    if string_vars.contains(name) {
                        return format!("{}.as_str()", name);
                    }
                    // Unknown (not confirmed PathBuf, not confirmed String):
                    // Plan 016 Phase A A.4 — conservatively keep .as_str().
                    // String has a valid .as_str() method; &str params/literals
                    // wouldn't reach here (a2r skips .as_str() for them at codegen).
                    // Only PathBuf needs .to_str().unwrap(), and confirmed PathBufs
                    // are in pathbuf_vars (handled above). This avoids wrongly
                    // rewriting String vars named path/aaid_path (E0599 to_str).
                    format!("{}.as_str()", name)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376 Phase 2: Run Auto's type inference engine over each function body
    /// to determine local variable types, then populate `local_var_types`.
    /// This gives the codegen type context for decisions like:
    /// - `.to_string()` when assigning &str to String field
    /// - `.unwrap()` when HashMap.get() returns Option
    /// - `.as_str()` when passing String to &str param
    pub(super) fn run_type_inference(&mut self, stmts: &[Stmt]) {
        use crate::infer::InferenceContext;

        // Plan 376D: Use the shared TypeStore (from all modules) if available;
        // otherwise build a local one from the current AST only.
        let type_store = if let Some(ref shared) = self.shared_type_store {
            // Enrich the shared store with the current module's declarations
            // (the shared store may have been built before this module was parsed).
            if let Ok(mut store) = shared.write() {
                for stmt in stmts {
                    match stmt {
                        Stmt::Fn(fn_decl) => {
                            store.register_fn_decl(fn_decl);
                        }
                        Stmt::TypeDecl(td) => {
                            store.register_type_decl(td);
                        }
                        Stmt::SpecDecl(sd) => {
                            store.register_spec_decl(sd);
                        }
                        Stmt::EnumDecl(ed) => {
                            store.register_enum_decl(ed.clone());
                        }
                        Stmt::Ext(ext) => {
                            store.register_ext_methods(ext);
                        }
                        _ => {}
                    }
                }
            }
            shared.clone()
        } else {
            // Single-file mode: build from current AST only.
            let mut store = crate::types::TypeStore::new();
            for stmt in stmts {
                match stmt {
                    Stmt::Fn(fn_decl) => {
                        store.register_fn_decl(fn_decl);
                    }
                    Stmt::TypeDecl(td) => {
                        store.register_type_decl(td);
                    }
                    Stmt::SpecDecl(sd) => {
                        store.register_spec_decl(sd);
                    }
                    Stmt::EnumDecl(ed) => {
                        store.register_enum_decl(ed.clone());
                    }
                    Stmt::Ext(ext) => {
                        store.register_ext_methods(ext);
                    }
                    _ => {}
                }
            }
            std::sync::Arc::new(std::sync::RwLock::new(store))
        };

        let mut ctx = InferenceContext::with_type_store(type_store);

        // Process each top-level function and each method inside type declarations
        for stmt in stmts {
            match stmt {
                Stmt::Fn(fn_decl) => {
                    self.infer_fn_body(&mut ctx, fn_decl);
                }
                Stmt::TypeDecl(td) => {
                    for method in &td.methods {
                        self.infer_fn_body(&mut ctx, method);
                    }
                }
                _ => {}
            }
        }
    }
}
