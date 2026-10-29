//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_rust_macro_shape(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                                    return Ok(CallShape::Handled);
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

        Ok(CallShape::Miss)
    }

    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_imported_rust_macro(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                return Ok(CallShape::Handled);
            }
        }

        // Plan 204 Phase 1A: Rust assert/assert_eq/assert_ne/panic are macros, need ! suffix
        // Special: when 2nd arg is an f-string, inline it directly (not format!())
        // because Rust assert! expects a string literal as the format arg.

        Ok(CallShape::Miss)
    }
}
