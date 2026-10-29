//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_task_spawn(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                            return Ok(CallShape::Handled);
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

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_task_handle_send(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                        return Ok(CallShape::Handled);
                    }
                    // String-message task: bare string literal → owned String.
                    if matches!(arg0, Expr::Str(_) | Expr::CStr(_)) {
                        self.expr(_obj, out)?;
                        write!(out, ".send(")?;
                        self.expr(arg0, out)?;
                        write!(out, ".to_string())")?;
                        return Ok(CallShape::Handled);
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

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_send_await_ask(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                }
            }
        }

        // Plan 151 Phase 1.3 + Plan 204 Phase 5: Method call translations
        // Translate Auto method names to Rust equivalents

        Ok(CallShape::Miss)
    }
}
