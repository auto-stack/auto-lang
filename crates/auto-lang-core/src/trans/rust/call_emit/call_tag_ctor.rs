//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_union_ctor(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                    return Ok(CallShape::Handled);
                }
                // Positional arg: use the first union field name
                if let Some(field) = call.args.args.first() {
                    write!(out, "{}::new_0(", type_name)?;
                    self.arg(field, out)?;
                    write!(out, ")")?;
                    return Ok(CallShape::Handled);
                }
            }
        }

        // Detect Rust macro calls imported via use.rust (e.g., use.rust log::debug → debug!("..."))
        // When call.name is Ident("debug") and self.uses contains "log::debug", emit debug!(...)

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_struct_construction(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                return self
                    .struct_init(type_name, &call.args, out)
                    .map(|_| CallShape::Handled);
            }
        }

        // Plan 204 Phase 5: Auto stdlib free function -> Rust equivalents

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_qualified_type_ctor(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                            return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                }
            }
        }

        // Normal function call
        // In merge mode, if callee is a known const name with no args, emit bare const reference

        Ok(CallShape::Miss)
    }
}
