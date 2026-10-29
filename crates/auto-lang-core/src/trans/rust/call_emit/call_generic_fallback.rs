//! PLAN-884 T-05：call_inner 分支族主题模块（深度0语句边界原样迁入）。
use super::*;

impl RustTrans {
    /// PLAN-884 T-05：自 call_inner 顺序链按深度0语句边界原样迁入的分支族
    /// （审计图 attachments/884/t01-audit-call.md）。Miss=未命中落穿；
    /// Handled=已发射。首匹配序是 a2r 金样的载荷，不得重排。
    pub(super) fn emit_call_generic_fallback(
        &mut self,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<CallShape> {
        self.expr(&call.name, out)?;
        // Plan 395: explicit generic type args → Rust turbofish
        self.emit_turbofish_args(call, out)?;
        write!(out, "(")?;

        // Look up str-param flags for auto-borrow at call sites
        let str_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_str_param_indices.get(fn_name).cloned()
        } else {
            // Try to extract the last segment of a qualified path like crate::forge::func
            let last_seg = match call.name.as_ref() {
                Expr::Dot(_, field) => Some(field.as_str()),
                Expr::Bina(_, Op::Dot, rhs) => {
                    if let Expr::Ident(name) = rhs.as_ref() {
                        Some(name.as_str())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(name) = last_seg {
                self.fn_str_param_indices.get(name).cloned()
            } else {
                None
            }
        };

        // Look up struct-param flags for auto-clone at call sites
        let struct_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_struct_param_indices.get(fn_name).cloned()
        } else {
            None
        };

        // Look up merge-mode &mut flags (context types skip .clone())
        let merge_mut_flags = if self.merge_mode {
            if let Expr::Ident(fn_name) = call.name.as_ref() {
                self.fn_merge_mut_params.get(fn_name).cloned()
            } else {
                None
            }
        } else {
            None
        };

        // C11 (Plan 018 §12 a2r-11): callee `mut p T` params → pass `&mut arg`.
        let mut_param_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_mut_params.get(fn_name).cloned()
        } else {
            None
        };

        // Look up spec-param flags for auto-boxing at call sites
        // Plan 390 §11 Phase E (D-B): handle method calls (Expr::Dot) via the same
        // last-segment fallback used by str_flags above, so `r.register(t)` resolves
        // to the "Type.register" / "register" key populated in the prescan (Fix A).
        let spec_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_spec_param_indices.get(fn_name).cloned()
        } else {
            // Try to extract the last segment of a qualified path / method call.
            let last_seg = match call.name.as_ref() {
                Expr::Dot(_, field) => Some(field.as_str()),
                Expr::Bina(_, Op::Dot, rhs) => {
                    if let Expr::Ident(name) = rhs.as_ref() {
                        Some(name.as_str())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(name) = last_seg {
                self.fn_spec_param_indices.get(name).cloned()
            } else {
                None
            }
        };

        // Look up int-param flags for enum→i32 cast at call sites
        let int_flags = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_int_param_indices.get(fn_name).cloned()
        } else {
            None
        };

        // Look up full param types for type-aware call site generation
        let param_types = if let Expr::Ident(fn_name) = call.name.as_ref() {
            self.fn_param_types.get(fn_name).cloned()
        } else {
            None
        };

        for (i, arg) in call.args.args.iter().enumerate() {
            let is_str_param = str_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            let needs_borrow =
                is_str_param && !Self::is_string_literal_arg(arg) && !self.is_str_slice_var(arg);

            // Plan 384 A3: reference-param borrow injection. When the callee's
            // declared param type is `Type::Reference(T)` (e.g. extern_impl
            // stubs declared via extern_sigs.at with `@T`), borrow a owned
            // argument (`&arg`) so the call compiles against `&T` params.
            // String literals, already-`&str` slice vars, and `self` are left
            // untouched (they coerce or are already borrowed).
            let needs_ref_borrow = param_types
                .as_ref()
                .and_then(|pts| pts.get(i))
                .map(|pt| matches!(pt, Type::Reference(_)))
                .unwrap_or(false)
                && !Self::is_string_literal_arg(arg)
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    // Borrow owned locals/params; skip str-slice vars (already &str)
                    // and `self` (autoref) and already-clone suffixed.
                    name.as_str() != "self" && !self.is_str_slice_var(arg)
                } else if let Arg::Pos(expr) = arg {
                    // Borrow struct-literal / method-call / field-access args
                    !matches!(
                        expr,
                        Expr::Str(_) | Expr::CStr(_) | Expr::Int(_) | Expr::Float(_, _)
                    )
                } else {
                    false
                };

            // Plan 347: Fallback auto-borrow for cross-module / imported
            // function calls. When the callee's parameter types are not in the
            // local cache (`str_flags` is None — typical for functions imported
            // via `use crate::<mod>:<fn>`), and the argument is a `String`-
            // typed local variable, assume the callee takes `&str` (the Auto
            // `str` convention) and borrow the argument. This mirrors what the
            // same-module str-param path does above. String literals and
            // already-`&str` params are left untouched.
            // Plan 381 (Layer 2): enum variant CONSTRUCTION
            // (`OutputContentBlock.ToolUse(id, name, input)`) parses as a
            // dotted call whose callee param types are unknown — the fallback
            // below would append .as_str() to owned String payloads (E0308
            // `String` vs `&str`). Skip the fallback for known enum ctors.
            let is_enum_ctor = self.is_enum_variant_ctor(call);
            let needs_borrow_unknown_callee = str_flags.is_none()
                && !is_enum_ctor
                && !Self::is_string_literal_arg(arg)
                && !self.is_str_slice_var(arg)
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    // Only borrow owned String locals; never borrow params
                    // (those are &str already) or non-string variables. A
                    // local declared `str` is recorded as StrSlice but renders
                    // as an owned `String` in Rust, so it still needs borrowing.
                    !self.current_fn_str_params.contains(name)
                        && self
                            .local_var_types
                            .get(name)
                            .map(|ty| {
                                matches!(
                                    ty,
                                    Type::StrOwned
                                        | Type::StrFixed(_)
                                        | Type::CStrLit
                                        | Type::StrSlice
                                )
                            })
                            .unwrap_or(false)
                } else if let Arg::Pos(expr) = arg {
                    // Plan 368 FU-2: an inline string concatenation
                    // (e.g. `base + "/x"`) renders to an owned `String`
                    // (format!(...)) — borrow it when calling an imported fn
                    // that takes &str. Matches the same convention as the Ident
                    // branch above. expr_contains_string detects any concat
                    // involving a string operand.
                    self.expr_contains_string(expr)
                } else {
                    false
                };

            // Auto-cast enum→i32 when passing an enum variable to an Int param
            let is_int_param = int_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            let needs_enum_cast = is_int_param
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    self.local_var_types
                        .get(name)
                        .map(|ty| match ty {
                            Type::Enum(_) => true,
                            Type::User(td) => self.known_enum_names.contains(&td.name),
                            _ => false,
                        })
                        .unwrap_or(false)
                } else {
                    false
                };

            // Auto-clone when passing a variable to a function that takes a struct param
            let is_struct_param = struct_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            // C11 (Plan 018 §12 a2r-11): callee param is `mut p T` (&mut T) →
            // pass `&mut arg` (never arg.clone()).
            let is_mut_param = mut_param_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            // Skip .clone() for merge-mode context types (they use &mut instead)
            let is_merge_mut = merge_mut_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);
            // Plan 347: StringBuilder params are passed by &mut reference. The
            // callee declares `sb: &mut a2r_std::StringBuilder`, so the caller
            // must pass `&mut sb` (never `sb.clone()` — that would break the
            // shared accumulator). This is independent of merge_mode.
            let is_sb_param = if matches!(arg, Arg::Pos(Expr::Ident(_))) {
                param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| Self::is_sb_ref_type(pt))
                    .unwrap_or(false)
            } else {
                false
            };
            // Check param type from fn_param_types for auto &mut insertion
            // Skip if the variable is already a &mut param of the current function
            let is_already_mut_param = if let Arg::Pos(Expr::Ident(name)) = arg {
                self.current_fn_mut_params.contains(name)
            } else {
                false
            };
            let needs_mut_borrow = if self.merge_mode
                && matches!(arg, Arg::Pos(Expr::Ident(_)))
                && !is_already_mut_param
            {
                param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| Self::is_merge_mut_type(pt))
                    .unwrap_or(false)
            } else {
                false
            };
            // Plan 433 A1: passing an &mut param of the current fn to a
            // BY-VALUE param (non-mut callee) must deref-clone (&mut Vec<T> →
            // Vec<T>); a raw ident would move the reference (E0308).
            let needs_mut_param_clone = is_already_mut_param
                && !is_mut_param
                && !needs_mut_borrow
                && !is_merge_mut
                && !is_sb_param
                && matches!(arg, Arg::Pos(Expr::Ident(_)));
            // Plan 016 (auto-down R4): passing an owned `Vec<T>` ident (e.g.
            // the current fn's own `List` param, `repl` in a recursive tree
            // walk) to a by-value `List` param MOVES it — inside a loop the
            // move fires on the first iteration; outside, the second use is
            // E0382 (e.g. `attrGet(ialAttrs, ..)` twice). Clone instead.
            // Guarded to idents whose local type is itself an owned List (so
            // &T/&mut params and iter vars are untouched).
            let is_owned_list_arg = !is_struct_param
                && param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| matches!(pt, Type::List(_)))
                    .unwrap_or(false)
                && if let Arg::Pos(Expr::Ident(name)) = arg {
                    self.local_var_types.get(name)
                        // Plan 514 W1 (m05): 数组字面量绑定是 Type::Array,
                        // Rust 侧同为 owned Vec<T>——按值传入 List 参数同样
                        // move,复用即 E0382;与 List 同列。
                        .map(|ty| matches!(ty, Type::List(_) | Type::Array(_)))
                        .unwrap_or(false)
                        // PLAN-018: 全局 List 读作 owned 实参同 move
                        // (E0507 cannot move out of MutexGuard)——同款克隆。
                        || self.global_var_types.get(name.as_str())
                            .map(|ty| matches!(ty, Type::List(_) | Type::Array(_)))
                            .unwrap_or(false)
                } else {
                    false
                };
            // Plan 016 Phase 4: a field-read (`s.marks`, `node.attrs`, `a.sel`)
            // passed to a by-value OWNED param (Vec/struct/enum/Map — anything
            // param_takes_ownership flags) MOVES the field out of its binding;
            // the next use of the same path is E0382 (or E0507 when the source
            // is behind a shared ref). Clone the field instead. Str params are
            // excluded (the .as_str() borrow path handles them), as are
            // borrowed-iter-var fields — the defect-4 branch below decides
            // those first.
            // Plan 531 真缺陷①(pointer/004): `x.@` 取址伪字段发射 `x as *mut _`
            // ——裸指针 Copy 无需 clone,且 as 强转后接 `.clone()` 是语法错
            // (cast cannot be followed by a method call),排除出本支路。
            let dot_arg_owned_param = matches!(arg, Arg::Pos(Expr::Dot(_, f)) if f.as_str() != "@")
                && !needs_ref_borrow
                && param_types
                    .as_ref()
                    .and_then(|pts| pts.get(i))
                    .map(|pt| Self::param_takes_ownership(pt))
                    .unwrap_or(false);
            let needs_clone = (is_struct_param || needs_mut_param_clone || is_owned_list_arg
                    || dot_arg_owned_param)
                && !is_merge_mut && !needs_mut_borrow
                && !is_mut_param
                && !is_sb_param
                && !needs_ref_borrow
                && (matches!(arg, Arg::Pos(Expr::Ident(_))) || dot_arg_owned_param)
                // Plan 380: spec-bound idents (`Some(prof)` from an Option<Spec>
                // scrutinee) are `Box<dyn Trait>` — no Clone impl (E0599).
                // Pass them by value (move) instead.
                && !(if let Arg::Pos(Expr::Ident(name)) = arg {
                    self.spec_bound_idents.contains(name)
                } else { false })
                // PLAN-010 T2 (009 ledger 033): Result<_, Box<dyn Error>>
                // locals — Box<dyn Error> is not Clone, so the safety clone is
                // E0599 ("trait bounds not satisfied"). Pass by value.
                && !(if let Arg::Pos(Expr::Ident(name)) = arg {
                    let err_not_clone = |err: &Type| matches!(err,
                        Type::GenericInstance(g) if g.base_name.as_str() == "Box")
                        || matches!(err, Type::Unknown);
                    match self.local_var_types.get(name) {
                        Some(Type::Result(err)) => err_not_clone(err),
                        // `Result<i64, Box<dyn Error>>` annotates as a
                        // GenericInstance (base "Result"), not Type::Result.
                        Some(Type::GenericInstance(inst))
                            if inst.base_name.as_str() == "Result" && inst.args.len() == 2 =>
                            err_not_clone(&inst.args[1]),
                        _ => false,
                    }
                } else { false });

            // Auto-box when passing a value to a function that takes a spec param
            let is_spec_param = spec_flags
                .as_ref()
                .and_then(|f| f.get(i))
                .copied()
                .unwrap_or(false);

            if is_spec_param {
                write!(out, "Box::new(")?;
            }

            // Auto &mut for context-type params in merge mode
            if needs_mut_borrow {
                write!(out, "&mut ")?;
            }
            // C11 (Plan 018 §12 a2r-11): `mut p T` callee param → pass `&mut arg`.
            // Plan 433 A1: when the arg ident is itself an &mut param of the
            // current fn, re-borrow (`&mut *arg`) — `&mut arg` on a `p: &mut T`
            // binding is E0596 (binding not declared mut) and would move the
            // reference, killing later uses. The ident itself is emitted by
            // arg() below, so only the `*` differs here.
            if is_mut_param {
                if is_already_mut_param {
                    write!(out, "&mut *")?;
                } else {
                    write!(out, "&mut ")?;
                }
            }
            // Plan 384 A3: borrow owned arg for `&T` reference params.
            if needs_ref_borrow {
                write!(out, "&")?;
            }
            // Plan 347: StringBuilder params take &mut at the call site.
            if is_sb_param {
                write!(out, "&mut ")?;
            }

            // Plan 013 (B16): if the ident was bound inside an `is` pattern for
            // a bridge-crate variant (e.g. `Kid.Node(child)`), it's a Box<T> in
            // Rust — emit `(*ident).clone()` so the inner value is cloned, not
            // the Box wrapper. Handle this before self.arg() writes the ident.
            let bridge_box_ident = if needs_clone {
                if let Arg::Pos(Expr::Ident(name)) = arg {
                    if self.bridge_pattern_bound_idents.contains(name) {
                        Some(name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            // Plan 447 ③-前置: set when a branch below already emitted the arg
            // WITH its `.as_str()` borrow — the generic borrow append at the end
            // of the loop iteration must not fire again.
            let mut str_stringified_here = false;
            if let Some(name) = bridge_box_ident {
                // Rust method resolution: `(*ident).clone()` on a Box<T> still
                // autorefs back to `Box::<T>::clone()` → returns Box<T>, not T.
                // The double-deref `*(*ident).clone()` unwraps the re-cloned
                // Box, yielding T. (Plan 013 B16.)
                write!(out, "*(*{}).clone()", Self::rust_ident(name.as_str()))?;
            } else {
                // Plan 380: when passing a trim*() call (which the expr handler
                // renders with a trailing .to_string()) to a `&str` param, emit
                // via expr_as_str so the suffix is stripped — trim() already
                // returns &str (`clean_field_value(r.trim())` → E0308 String).
                let trim_arg = if let Arg::Pos(expr) = arg {
                    Self::is_trim_method_call(expr)
                } else {
                    false
                };
                if is_str_param && trim_arg {
                    if let Arg::Pos(expr) = arg {
                        self.expr_as_str(expr, out)?;
                    }
                } else if is_str_param
                    && matches!(arg, Arg::Pos(Expr::Call(c))
                        if matches!(c.name.as_ref(), Expr::Dot(_, m)
                            if matches!(m.as_str(), "str" | "to_string"))
                            && c.args.args.is_empty())
                {
                    // Plan 447 ③-前置(②列基线修复): Auto's universal
                    // `.str()` in a &str slot — emit by RECEIVER type. The
                    // plain path wrote `x.to_string()` for both str and int
                    // receivers, then the borrow append gave
                    // `x.to_string().as_str()` — text-reducer
                    // fix_string_str_mismatches collapses that to
                    // `x.as_str()`, E0599 on i64/char/bool receivers
                    // (codegen.at `emit("const.i32", cp.str(), cp)`).
                    // Str receivers keep the collapsed form byte-for-byte;
                    // known numeric receivers go through format! (same shape
                    // fix_int_as_str produces).
                    if let Arg::Pos(Expr::Call(c)) = arg {
                        if let Expr::Dot(recv, _) = c.name.as_ref() {
                            let recv_ty = self.infer_type_from_expr(recv);
                            // Numeric AND unresolvable receivers go through
                            // format! — the text reducer collapses the plain
                            // `x.to_string().as_str()` form back to `x.as_str()`
                            // (E0599 on i64). Str-typed receivers keep the
                            // direct borrow (byte-identical to the collapsed
                            // legacy form, zero golden drift).
                            let borrow_direct = matches!(
                                recv_ty,
                                Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                            );
                            let recv_parens = matches!(recv.as_ref(),
                                Expr::Bina(_, op, _) if !matches!(op, Op::Dot));
                            if borrow_direct {
                                if recv_parens {
                                    write!(out, "(")?;
                                }
                                self.expr(recv, out)?;
                                if recv_parens {
                                    write!(out, ")")?;
                                }
                                write!(out, ".as_str()")?;
                            } else {
                                write!(out, "format!(\"{{}}\", ")?;
                                self.expr(recv, out)?;
                                write!(out, ").as_str()")?;
                            }
                            str_stringified_here = true;
                        }
                    }
                } else if needs_ref_borrow && matches!(arg, Arg::Pos(Expr::Dot(_, _))) {
                    // Plan 018 §Phase 3.5: `&self.field` passed to a `@T` (&T)
                    // param must NOT get the self-dot `.clone()` that `arg()`
                    // appends — that produces `&self.field.clone()` (E0599 no
                    // clone on Mutex). The borrow is a shared reference, so the
                    // field is moved-by-borrow, not cloned. Emit the expr
                    // directly and let the `&` written above apply.
                    if let Arg::Pos(expr) = arg {
                        self.expr(expr, out)?;
                    }
                } else if {
                    // 2026-08-23 auto-ai report defect 4: a field of a
                    // borrowed loop var (`m.content` in `for m in &req.messages`)
                    // passed to a str/List param — arg()'s §2.1 conservative
                    // clone (`m.content.clone()`) regressed these call sites
                    // (daemon's content_blocks_to_anthropic / openai_content).
                    // Copy-ish callee params (str borrows, List passed by
                    // reference per the is_copy_type convention) don't take
                    // ownership, so no clone is needed. Non-Copy params (e.g.
                    // `extract_path(tc.args)` with a Value param) still clone
                    // via arg() below.
                    let iter_field = matches!(arg, Arg::Pos(Expr::Dot(_, _)))
                        && if let Arg::Pos(Expr::Dot(obj, _)) = arg {
                            matches!(obj.as_ref(), Expr::Ident(n) if self.borrowed_iter_vars.contains(n))
                        } else {
                            false
                        };
                    iter_field
                        && match param_types.as_ref().and_then(|pts| pts.get(i)) {
                            // Known callee signature: skip the clone only
                            // when the param does NOT take ownership. Plan
                            // 016 Phase 4: `List<T>` params are emitted as
                            // owned `Vec<T>` (not `&[T]`), so a List field of
                            // a borrowed loop var must clone (E0507) —
                            // is_copy_type's List-is-borrowed convention does
                            // not hold at fn boundaries.
                            Some(pt) => !Self::param_takes_ownership(pt),
                            // Unknown callee (imported fn — single-file
                            // transpile can't see its signature): keep the
                            // pre-§2.1 shape (no clone), matching the sed
                            // contract of cross-file consumers.
                            None => true,
                        }
                } {
                    // 2026-08-23 auto-ai report defect 4: a field of a
                    // borrowed loop var (`m.content` in `for m in &req.messages`)
                    // passed to a str/List param — arg()'s §2.1 conservative
                    // clone (`m.content.clone()`) regressed these call sites
                    // (daemon's content_blocks_to_anthropic / openai_content).
                    // Copy-ish callee params (str borrows, List passed by
                    // reference per the is_copy_type convention) don't take
                    // ownership, so no clone is needed. Non-Copy params (e.g.
                    // `extract_path(tc.args)` with a Value param) still clone
                    // via arg() below.
                    if let Arg::Pos(expr) = arg {
                        self.expr(expr, out)?;
                    }
                } else if needs_clone {
                    // 2026-08-23 auto-ai report defect 4: a borrowed loop
                    // var (`t` in `for t in &coll`) already gets its deref
                    // clone from arg()'s maybe_clone_borrowed_iter_field —
                    // running self.arg() here and appending another produced
                    // `t.clone().clone()`. Emit the raw expression with
                    // exactly one clone.
                    if let Arg::Pos(expr) = arg {
                        self.expr(expr, out)?;
                    }
                    write!(out, ".clone()")?;
                } else {
                    self.arg(arg, out)?;
                }
            }

            // After expression: add .as_str() for String→&str conversion
            // Plan 376 Pass 7: skip .as_str() when the callee param is already &str
            // (the callee declared `param str` which renders as &str in Rust).
            // In this case needs_borrow is true (is_str_param), but the argument
            // is already &str — adding .as_str() causes E0658 (unstable feature).
            // The fix: if the arg variable's type IS StrSlice (it's a &str param),
            // don't add .as_str() even when is_str_param says to borrow.
            let arg_is_str_slice = if let Arg::Pos(Expr::Ident(name)) = arg {
                // Only fn params declared `str` render as &str. local_var_types
                // records EVERY str-typed var (incl. owned String locals) as
                // StrSlice, so consulting it here would wrongly skip .as_str()
                // on locals → E0308 (String vs &str). Check the fn-param set
                // (mirrors is_str_slice_var).
                self.current_fn_str_params.contains(name)
            } else {
                false
            };
            let arg_is_str_literal =
                matches!(arg, Arg::Pos(Expr::Str(_)) | Arg::Pos(Expr::CStr(_)));
            // Plan 380: only add .as_str() for plain String-typed locals or
            // string-concat exprs. Method-call results (trim(), to_str().unwrap(),
            // ...) and char literals must NOT get .as_str() — they're already
            // &str or char (E0658 str_as_str / E0599 char.as_str).
            let arg_is_ident = matches!(arg, Arg::Pos(Expr::Ident(_)));
            let arg_is_concat = if let Arg::Pos(expr) = arg {
                self.expr_contains_string(expr)
            } else {
                false
            };
            // Plan 417-E1 rider: a plain call to a user fn whose DECLARED
            // return is `str` renders as an owned String — borrowing it for
            // a &str param needs the same `.as_str()` as ident args. Plan
            // 380's restriction to idents/concats skipped call args, so
            // parity string_utils' `check_str(1, ..., reverse("abc"), ...)`
            // failed E0308 (String vs &str). Only Ident-named calls with a
            // known str return type qualify (method chains are already &str
            // and stay excluded).
            let arg_is_str_returning_call = if let Arg::Pos(Expr::Call(c)) = arg {
                if let Expr::Ident(fname) = c.name.as_ref() {
                    self.fn_ret_types
                        .get(fname)
                        .map(|ty| {
                            matches!(
                                ty,
                                Type::StrOwned | Type::StrFixed(_) | Type::CStrLit | Type::StrSlice
                            )
                        })
                        .unwrap_or(false)
                } else if let Expr::Dot(_, m) = c.name.as_ref() {
                    // Plan 433 A1: `.str()` / `.to_string()` method calls return
                    // an owned String — a &str param still needs the borrow.
                    // Plan 019 Phase 1: `.slice(...)` also renders as an owned
                    // String (byte-range .to_string() or chars().collect()).
                    // Plan 514 W3: str 返回的用户方法(方法化 lib 的
                    // `int_val(p.text())` 族)渲染同为 owned String——经
                    // infer(Fix C 的 qualified 键)解析声明返回。
                    matches!(m.as_str(), "str" | "to_string" | "slice")
                        || matches!(arg,
                            Arg::Pos(e) if matches!(
                                self.infer_type_from_expr(e),
                                Type::StrOwned | Type::StrSlice
                                    | Type::StrFixed(_) | Type::CStrLit))
                } else {
                    false
                }
            } else {
                false
            };
            // Plan 433 A1: a struct-field read of a str field is an owned
            // String (t.kind) — borrow it for &str params.
            let arg_is_str_field = if let Arg::Pos(Expr::Dot(obj, field)) = arg {
                // Plan 433 A1: field reads of str fields — the owner's type is
                // often unavailable (chained/indexed objects), so accept the
                // field name being a str field of ANY known struct. Borrowed
                // loop-var fields are EXCLUDED — the defect-4 branch above
                // already emits them raw (&String coerces to &str directly).
                let is_borrowed_iter_field = matches!(obj.as_ref(),
                    Expr::Ident(n) if self.borrowed_iter_vars.contains(n));
                !is_borrowed_iter_field
                    && self.struct_field_types.values().any(|fields| {
                        fields.iter().any(|(f, t)| {
                            f.as_str() == field.as_str()
                                && matches!(
                                    t,
                                    Type::StrOwned
                                        | Type::StrSlice
                                        | Type::StrFixed(_)
                                        | Type::CStrLit
                                )
                        })
                    })
            } else {
                false
            };
            // Plan 032 第七批合并后复验发现的 plan 016 Phase 4 试点缺口（两处，
            // auto-ai 轨暴露，autodown-core 无此形态）：
            // (a) 裸 `x.trim()` 家族方法调用本来就产 `&str`——Phase 4 把 trim 加进
            //     expr_contains_string（concat 检测用）后，arg_is_concat 复用同一
            //     函数误borrow，`f(x.trim().as_str())` 触发 E0658（str_as_str）。
            //     排除 trim 家族与 to_str（产 &str 的方法）；to_string/to_lowercase
            //     等产 owned String 的不在此列。
            // (b) 未知被调者的 .as_str() 兜底不能叠在 .at 显式 `X.to_string()` 上
            //     ——作者已显式物化 owned String（如 fn-pointer 字段
            //     `handler: fn(String)`），兜底假定「&str 更常见」会反转意图
            //     （auto-ai agent driver.at `handler(step_id.to_string())`）。
            let arg_is_bare_refstr_method = if let Arg::Pos(Expr::Call(c)) = arg {
                matches!(c.name.as_ref(), Expr::Dot(_, m)
                    if matches!(m.as_str(),
                        "trim" | "trim_start" | "trim_end" | "trim_matches" | "to_str"))
            } else {
                false
            };
            let arg_is_explicit_to_string = if let Arg::Pos(Expr::Call(c)) = arg {
                matches!(c.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "to_string")
            } else {
                false
            };
            // Plan 447 ③-前置(②列基线修复): `xs.get(i)` on a List<str> —
            // the get→index conversion emits `xs[i as usize].clone()` (an
            // owned String); a &str param slot needs the same borrow as
            // plain str idents (E0308 otherwise). infer_type_from_expr's
            // get-arm resolves the element type.
            let arg_is_str_get = if let Arg::Pos(expr) = arg {
                matches!(expr, Expr::Call(c)
                    if c.args.args.len() == 1
                        && matches!(c.name.as_ref(), Expr::Dot(_, m) if m.as_str() == "get"))
                    && matches!(
                        self.infer_type_from_expr(expr),
                        Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                    )
            } else {
                false
            };
            // Plan 019 Phase 1: `lines[i]` on a List<str> — the index
            // conversion emits an owned String (`.clone()`); a &str param
            // slot needs the same borrow as the get-arm above.
            let arg_is_str_index = if let Arg::Pos(expr) = arg {
                matches!(expr, Expr::Index(_, _))
                    && matches!(
                        self.infer_type_from_expr(expr),
                        Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                    )
            } else {
                false
            };
            // Plan 019 Phase 1: `x.field ?? fallback` yields an owned String
            // (the coalesce strips the Option) — a &str param slot needs the
            // borrow, same as the field read itself.
            let arg_is_str_coalesce = if let Arg::Pos(expr) = arg {
                matches!(expr, Expr::NullCoalesce(_, _))
                    && matches!(
                        self.infer_type_from_expr(expr),
                        Type::StrOwned | Type::StrSlice | Type::StrFixed(_) | Type::CStrLit
                    )
            } else {
                false
            };
            if (needs_borrow || needs_borrow_unknown_callee)
                && !arg_is_str_slice
                && !arg_is_str_literal
                && !arg_is_bare_refstr_method
                && !(needs_borrow_unknown_callee && !needs_borrow && arg_is_explicit_to_string)
                && !str_stringified_here
                && (arg_is_ident
                    || arg_is_concat
                    || arg_is_str_returning_call
                    || arg_is_str_field
                    || arg_is_str_get
                    || arg_is_str_index
                    || arg_is_str_coalesce)
            {
                write!(out, ".as_str()")?;
            }

            // PLAN-681 T-05（N2/P5）：Value 读实参按形参型收口——实参为
            // Value 读（Value 局部字段读/bare List 索引读,P2 已发射
            // name["field"]/list[i] 索引形）而形参是标量/List 声明型时,
            // 按形参型追加降链。仅 bare fn（Ident callee,形参表在档）触发。
            if let Arg::Pos(vexpr) = arg {
                if self.expr_is_json_value_read(vexpr) {
                    if let Some(pt) = param_types.as_ref().and_then(|pts| pts.get(i)) {
                        match pt {
                            Type::StrSlice | Type::CStrLit | Type::StrFixed(_) => {
                                write!(out, ".as_str().unwrap_or_default()")?;
                            }
                            Type::Int | Type::I64 => {
                                write!(out, ".as_i64().unwrap_or(0)")?;
                            }
                            Type::Bool => {
                                write!(out, ".as_bool().unwrap_or(false)")?;
                            }
                            Type::List(_) | Type::Array(_) => {
                                write!(out, ".as_array().cloned().unwrap_or_default()")?;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Plan 376 Pass 5: &str → String conversion when param expects owned
            // String (StrOwned/StrFixed). Auto-detect from param_types: if param
            // is StrOwned and arg is &str (StrSlice param or string literal), add .to_string().
            if !needs_borrow && !needs_borrow_unknown_callee {
                // Only when we didn't already handle it via borrow path
                if let Some(pts) = &param_types {
                    if let Some(pt) = pts.get(i) {
                        let param_is_owned_str = matches!(pt, Type::StrOwned | Type::StrFixed(_));
                        let arg_is_str_value = arg_is_str_slice
                            || arg_is_str_literal
                            || (if let Arg::Pos(Expr::Ident(name)) = arg {
                                self.local_var_types
                                    .get(name)
                                    .map(|ty| matches!(ty, Type::StrSlice))
                                    .unwrap_or(false)
                            } else {
                                false
                            });
                        if param_is_owned_str && arg_is_str_value {
                            write!(out, ".to_string()")?;
                        }
                    }
                }
            }

            // Enum→i32 cast for int-expecting params
            if needs_enum_cast {
                write!(out, " as i64")?;
            }

            // Plan 376E: Broad type-aware argument conversion using local_var_types.
            // When neither the str-borrow path nor the str-to_string path fired,
            // check for other type mismatches between the arg's inferred type and
            // the param's declared type.
            if !needs_borrow && !needs_borrow_unknown_callee && !needs_enum_cast && !is_spec_param {
                if let (Some(pts), Arg::Pos(Expr::Ident(name))) = (&param_types, arg) {
                    if let Some(pt) = pts.get(i) {
                        let arg_ty = self.local_var_types.get(name);
                        if let Some(aty) = arg_ty {
                            // Option<T> → T: need .unwrap() or .cloned().unwrap_or_default()
                            if matches!(pt, Type::User(td) if td.name.as_str() != "Option")
                                && matches!(aty, Type::Option(_))
                                && !matches!(pt, Type::Option(_))
                            {
                                write!(out, ".unwrap()")?;
                            }
                            // u32 param, i32 arg: cast
                            else if matches!(pt, Type::Uint) && matches!(aty, Type::Int) {
                                write!(out, " as u32")?;
                            }
                            // i32 param, u32 arg: cast
                            else if matches!(pt, Type::Int) && matches!(aty, Type::Uint) {
                                write!(out, " as i64")?;
                            }
                            // usize param, i32/u32 arg: cast
                            else if matches!(pt, Type::USize)
                                && (matches!(aty, Type::Int) || matches!(aty, Type::Uint))
                            {
                                write!(out, " as usize")?;
                            }
                        }
                    }
                }
            }

            if is_spec_param {
                // Plan 390 §11 Phase E (D-C): the close paren of `Box::new(<arg>...)`.
                // A spec-bound ident (`Some(prof)` scrutinee — already `Box<dyn Trait>`)
                // must clone the box to stay usable; any other argument (a concrete
                // struct value like `EchoTool`, or a non-ident expression) is moved
                // into the box — matching the array-element pattern at ~9454 (pure
                // `Box::new(elem)`, no clone).
                if let Arg::Pos(Expr::Ident(name)) = arg {
                    if self.spec_bound_idents.contains(name) {
                        write!(out, ".clone())")?;
                    } else {
                        write!(out, ")")?;
                    }
                } else {
                    write!(out, ")")?;
                }
            }

            if i < call.args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, ")")?;

        // Plan 373: Auto-insert .await for calls to methods returning ~Result/Future.
        if let Expr::Dot(object, mname) = call.name.as_ref() {
            if self.call_needs_await(object, mname) {
                write!(out, ".await")?;
            }
        } else if let Expr::Ident(fname) = call.name.as_ref() {
            if let Some(ret) = self.fn_ret_types.get(fname.as_str()) {
                if Self::type_is_async(ret) {
                    write!(out, ".await")?;
                }
            }
        }

        Ok(CallShape::Handled)
    }
}
