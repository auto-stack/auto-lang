//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_foreign_value_ctor(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                        return Ok(CallShape::Handled);
                    }
                }
            }
        }
        // Detect Rust macro patterns: name!("...") was parsed as name.collect()("...")
        // because '!' is the eager collection operator in Auto.
        // Parser creates: Expr::Bina(lhs, Dot, "collect") then wraps in Call.
        // AST: Call { name: Call { name: Bina(Ident(name), Dot, "collect"), args: [] }, args: [...] }

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_collection_ctor_strip(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                            return Ok(CallShape::Handled);
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

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_generic_json_decode(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                    return Ok(CallShape::Handled);
                }
            }
        }

        // Special case for print / write function

        Ok(CallShape::Miss)
    }
}
