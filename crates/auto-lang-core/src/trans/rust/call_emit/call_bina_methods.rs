//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_bina_method_translate(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
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
                                return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                                    return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        "shl" => {
                            // val.shl(n) -> val.wrapping_shl(n as u32) (wrapping)
                            self.expr(lhs, out)?;
                            write!(out, ".wrapping_shl((")?;
                            if let Some(Arg::Pos(arg)) = call.args.args.first() {
                                self.expr(arg, out)?;
                            }
                            write!(out, ") as u32) as i64")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        "not" => {
                            // val.not() -> (!val), no arguments.
                            write!(out, "(!")?;
                            self.expr(lhs, out)?;
                            write!(out, ")")?;
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        "slice" => {
                            // s.slice(n) -> s[n..].to_string()
                            // s.slice(start, end) -> s[start..end].to_string()
                            // Plan 019 Phase 1: string receivers slice by
                            // CHARS (Auto str positions are code-unit based);
                            // non-string receivers keep the byte-index form.
                            if self.expr_is_string_like(lhs) {
                                return self
                                    .emit_str_slice_chars(lhs, &call.args.args, out)
                                    .map(|_| CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
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
                            return Ok(CallShape::Handled);
                        }
                        // Tuple field access: .get_0() -> .0, .get_1() -> .1, etc.
                        "get_0" => {
                            self.expr(lhs, out)?;
                            write!(out, ".0")?;
                            return Ok(CallShape::Handled);
                        }
                        "get_1" => {
                            self.expr(lhs, out)?;
                            write!(out, ".1")?;
                            return Ok(CallShape::Handled);
                        }
                        "get_2" => {
                            self.expr(lhs, out)?;
                            write!(out, ".2")?;
                            return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
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
                        return Ok(CallShape::Handled);
                    }
                }
            }
        }

        // Also handle Expr::Dot method calls (parser emits Dot for method calls)

        Ok(CallShape::Miss)
    }
}
