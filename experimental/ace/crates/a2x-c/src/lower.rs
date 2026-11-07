//! PLAN-921 CA-A: the HIR → C11 scalar lowering.
//!
//! Mirrors the Rust target's semantic discipline exactly (plan §5.1):
//! * **Checked arithmetic** — `a2x_add`/`a2x_sub`/`a2x_mul`/`a2x_neg`/
//!   `a2x_div`/`a2x_rem` compute in `int64_t` and range-check BEFORE the
//!   narrowing conversion (C signed overflow is never executed);
//!   `div`/`rem` reject zero and `INT32_MIN / -1` first; C's truncation
//!   toward zero preserves `-7/3 == -2`, `-7%3 == -1`.
//! * **Source order, single evaluation** — every call argument (and every
//!   fallible operand) is pinned into a fresh temporary at its
//!   source-order position; the call receives only side-effect-free
//!   locals, so C's unspecified argument evaluation order is irrelevant.
//! * **Short-circuit** — `and`/`or` lower to a branch-guarded temporary:
//!   the RHS's statements live inside `if (t) { ... }` / `if (!t) { ... }`
//!   and never execute when the LHS decides.
//! * **Trap propagation** — every fallible operation checks
//!   `t.trap != A2X_TRAP_NONE` and returns the trap result immediately;
//!   no unwind, no longjmp.
//! * **Hygiene** — `fn_`/`mt_`/`lN_` prefixed identifiers (never a C
//!   keyword, never a reserved leading-underscore name), ASCII-sanitized.
//!
//! Fragment shape contract (the fileset assembler depends on it): each
//! fragment is `// origin: ...\n` then its PROTOTYPE line, then the
//! definition — the assembler collects every prototype line into the
//! forward-declaration block of `generated.c` (same scan-the-texts
//! pattern the Rust record-text prelude uses).

use ace_a2x_core::{A2xReject, FnFacts, FnKind, SemanticView, Ty};
use ace_core::hir;
use std::collections::HashMap;

/// Hygiene: ASCII letters/digits/`_` only, everything else → `_` (same
/// sanitizer as the Rust target).
pub(crate) fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    out
}

/// Hygienic C name of a module function (free functions and
/// owner-qualified methods alike — the `fn_`/`mt_` prefix can never be a
/// C keyword).
pub fn c_fn_name(name: &str) -> String {
    let s = sanitize(name);
    if s.is_empty() {
        "fn_".to_string()
    } else {
        format!("fn_{}", s)
    }
}

/// Hygienic C name of an owner-qualified method.
pub fn c_method_name(owner_qualified: &str) -> String {
    let s = sanitize(owner_qualified);
    format!("mt_{}", s)
}

fn c_name_of(f: &FnFacts<'_>) -> String {
    match f.kind {
        FnKind::Free => c_fn_name(f.source_name),
        FnKind::Method { .. } => c_method_name(f.source_name),
    }
}

/// The C type of a scalar view type (the compute surface only).
fn c_ty(ty: Ty) -> Result<&'static str, A2xReject> {
    match ty {
        Ty::I32 => Ok("int32_t"),
        Ty::Bool => Ok("bool"),
        // PLAN-961: the numeric C locals in their native widths.
        Ty::I8 => Ok("int8_t"),
        Ty::U8 => Ok("uint8_t"),
        Ty::U32 => Ok("uint32_t"),
        Ty::I64 => Ok("int64_t"),
        Ty::U64 => Ok("uint64_t"),
        Ty::F32 => Ok("float"),
        Ty::F64 => Ok("double"),
        other => Err(A2xReject::unsupported_type(format!(
            "the C scalar channel serves the numeric/bool locals, not `{:?}`",
            other
        ))),
    }
}

/// PLAN-961: the typed result CHANNEL of a value type (the trap-carrying
/// struct family); i32/bool keep the frozen `a2x_result`.
pub(crate) fn c_result_ty(ty: Ty) -> Result<&'static str, A2xReject> {
    Ok(match ty {
        Ty::I32 | Ty::Bool => "a2x_result",
        Ty::I8 | Ty::I64 => "a2x_result_i64",
        Ty::U8 | Ty::U32 | Ty::U64 => "a2x_result_u64",
        Ty::F32 => "a2x_result_f32",
        Ty::F64 => "a2x_result_f64",
        other => {
            return Err(A2xReject::unsupported_type(format!(
                "no numeric result channel for {other:?}"
            )))
        }
    })
}

/// PLAN-961: the domain type the typed helpers take (i8/u8/u32 widen to
/// the i64/u64 helper domain; the helpers range-check internally).
pub(crate) fn c_dom_ty(ty: Ty) -> Result<&'static str, A2xReject> {
    Ok(match ty {
        Ty::I8 | Ty::I64 => "int64_t",
        Ty::U8 | Ty::U32 | Ty::U64 => "uint64_t",
        _ => c_ty(ty)?,
    })
}

/// PLAN-961: helper suffix per value type (i32 keeps the frozen unsuffixed
/// i32 helpers for arithmetic; bit/shift ops gain _i32).

/// One lowered C function (name, params, result, origin, body text).
#[derive(Clone, Debug)]
pub struct CFn {
    pub name: String,
    /// (c type, c name) per parameter, in parameter order.
    pub params: Vec<(String, String)>,
    pub result_ty: Ty,
    pub origin: (u32, u32),
    /// The function body statements (indentation-free lines).
    pub body: Vec<String>,
}

/// Lower every function of the view (the batched whole-view pass; the
/// session calls this once per produce batch).
pub fn lower_functions_c<'a>(
    view: &SemanticView<'a>,
    fns: &[&FnFacts<'a>],
) -> Result<Vec<CFn>, A2xReject> {
    let mut out = Vec::with_capacity(fns.len());
    for f in fns {
        out.push(lower_fn_c(view, f)?);
    }
    Ok(out)
}

/// Render one fragment: origin comment + prototype + definition. The
/// prototype line is the assembler's forward-declaration input.
///
/// PLAN-938: `write_fragment` is the ONE formatter; the legacy entry keeps
/// its exact historical bytes, and the admitted entry reserves the exact
/// output size with the host before allocating the payload.
pub fn print_fragment(f: &CFn) -> String {
    let mut out = String::new();
    write_fragment(f, &mut out).expect("formatting into a String cannot fail");
    out
}

/// Same bytes as [`print_fragment`], with the exact size admitted by `admit`
/// BEFORE the single exactly-sized allocation.
pub fn print_fragment_admitted(
    f: &CFn,
    admit: impl FnOnce(usize) -> Result<(), String>,
) -> Result<String, A2xReject> {
    crate::admitted_text::render_admitted(|out| write_fragment(f, out), admit)
        .map_err(|e| A2xReject::new("a2c.foreign-resource", e))
}

fn write_params(f: &CFn, out: &mut dyn std::fmt::Write) -> std::fmt::Result {
    if f.params.is_empty() {
        return out.write_str("void");
    }
    for (index, (ty, name)) in f.params.iter().enumerate() {
        if index > 0 {
            out.write_str(", ")?;
        }
        write!(out, "{ty} {name}")?;
    }
    Ok(())
}

/// The single formatter behind both entries (counting pass allocates
/// nothing; the writer is bounded by the admitted size).
pub fn write_fragment(f: &CFn, out: &mut dyn std::fmt::Write) -> std::fmt::Result {
    write!(
        out,
        "// origin: {}..{} (def `{}`, byte span in the compiled source)\n",
        f.origin.0, f.origin.1, f.name
    )?;
    out.write_str("a2x_result ")?;
    out.write_str(&f.name)?;
    out.write_str("(")?;
    write_params(f, out)?;
    out.write_str(");\n")?;
    out.write_str("a2x_result ")?;
    out.write_str(&f.name)?;
    out.write_str("(")?;
    write_params(f, out)?;
    out.write_str(") {\n")?;
    for line in &f.body {
        out.write_str("    ")?;
        out.write_str(line)?;
        out.write_str("\n")?;
    }
    out.write_str("}\n")
}

struct FnLowering<'v, 'a> {
    view: &'v SemanticView<'a>,
    facts: &'v FnFacts<'a>,
    /// PLAN-961: the function's own trap-return channel (typed results
    /// return their typed struct; i32/bool keep a2x_result).
    ret_chan: &'static str,
    local_names: HashMap<u32, String>,
    out: Vec<String>,
    temp_counter: usize,
    loop_stack: Vec<u32>,
}

impl<'v, 'a> FnLowering<'v, 'a> {
    fn local_name(&self, id: u32) -> Result<&str, A2xReject> {
        self.local_names
            .get(&id)
            .map(|s| s.as_str())
            .ok_or_else(|| A2xReject::new("a2x.internal", "unknown local index"))
    }

    fn fresh_temp(&mut self) -> String {
        let t = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        t
    }

    /// Emit the immediate trap check of a pinned fallible result.
    fn check_trap(&mut self, t: &str) {
        self.out.push(format!(
            "if ({}.trap != A2X_TRAP_NONE) {{ return {}; }}",
            t, t
        ));
    }

    /// PLAN-961: pin one typed fallible numeric call into a fresh typed
    /// temp (checked, early-returned) and yield `tN.value`.
    fn typed_call(&mut self, chan: &str, call: String) -> String {
        let t = self.fresh_temp();
        self.out.push(format!("{chan} {t} = {call};"));
        // The trap propagates through the FUNCTION's channel (a typed
        // temp inside an i32-result function narrows to a2x_result{0,trap}).
        self.out.push(format!(
            "if ({t}.trap != A2X_TRAP_NONE) {{ return ({}){{ 0, {t}.trap }}; }}",
            self.ret_chan
        ));
        format!("{t}.value")
    }

    /// PLAN-961: typed_call + an explicit narrowing cast to the value's
    /// own C width (u8/u32 helpers return the u64 channel — W4-clean
    /// assignment needs the cast; range was checked inside the helper).
    fn typed_call_as(&mut self, chan: &str, call: String, cast: &str) -> String {
        let v = self.typed_call(chan, call);
        format!("({cast})({v})")
    }

    /// PLAN-961: typed numeric binary emission. Integers ride the typed
    /// checked helpers; comparisons use native C operators (width/
    /// signedness correct); float +,-,* are infallible native ops, / and %
    /// go through the trap/fmod helpers.
    fn num_bin_c(
        &mut self,
        op: hir::NumOpKind,
        bty: hir::BuiltinType,
        l: String,
        r: String,
    ) -> Result<String, A2xReject> {
        use hir::NumOpKind as K;
        let name = c_bname(bty);
        let is_cmp = matches!(op, K::Lt | K::Le | K::Gt | K::Ge | K::Eq | K::Ne);
        if is_cmp {
            let o = match op {
                K::Lt => "<",
                K::Le => "<=",
                K::Gt => ">",
                K::Ge => ">=",
                K::Eq => "==",
                _ => "!=",
            };
            return Ok(format!("(({}) {} ({}))", l, o, r));
        }
        if matches!(bty, hir::BuiltinType::F32 | hir::BuiltinType::F64) {
            let chan = if matches!(bty, hir::BuiltinType::F32) {
                "a2x_result_f32"
            } else {
                "a2x_result_f64"
            };
            let f: String = match op {
                K::Add => return Ok(format!("(({}) + ({}))", l, r)),
                K::Sub => return Ok(format!("(({}) - ({}))", l, r)),
                K::Mul => return Ok(format!("(({}) * ({}))", l, r)),
                K::Div => format!("a2x_div_{name}"),
                K::Rem => format!("a2x_rem_{name}"),
                _ => return Err(A2xReject::unsupported_op(format!("float bit op {op:?}"))),
            };
            return Ok(self.typed_call(chan, format!("{f}({l}, {r})")));
        }
        let ty = builtin_ty_of(bty)?;
        let dom = c_dom_ty(ty)?;
        let chan = c_result_ty(ty)?;
        let f = match op {
            K::Add => format!("a2x_add_{name}"),
            K::Sub => format!("a2x_sub_{name}"),
            K::Mul => format!("a2x_mul_{name}"),
            K::Div => format!("a2x_div_{name}"),
            K::Rem => format!("a2x_rem_{name}"),
            K::And => format!("a2x_and_{name}"),
            K::Or => format!("a2x_or_{name}"),
            K::Xor => format!("a2x_xor_{name}"),
            K::Shl => format!("a2x_shl_{name}"),
            K::Shr => format!("a2x_shr_{name}"),
            _ => unreachable!("cmp handled above"),
        };
        let cast = c_ty(ty)?;
        Ok(self.typed_call_as(chan, format!("{f}(({dom})({l}), ({dom})({r}))"), cast))
    }

    /// Lower an expression whose RESULT must be side-effect-free at its
    /// use site: fallible roots (checked ops, user calls) pin into fresh
    /// `a2x_result` temporaries (checked + returned on trap), trace marks
    /// pin into `int32_t` temporaries — exactly the Rust target's
    /// `needs_pin` discipline.
    fn lower_pure(&mut self, id: hir::ExprId) -> Result<String, A2xReject> {
        let e = self
            .facts
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => Ok(self.local_name(local.0)?.to_string()),
            hir::ExprKind::Constant { value } => {
                // INT32_MIN needs a constant expression that never
                // round-trips through a wider type: -2147483647 - 1
                if *value == i32::MIN {
                    Ok("(int32_t)(-2147483647 - 1)".to_string())
                } else {
                    Ok(format!("(int32_t){}", value))
                }
            }
            // PLAN-961: typed numeric constants — exact payload spellings
            // (MIN via constant expressions; floats via raw-bit
            // constructors so NaN payloads and -0.0 survive verbatim).
            hir::ExprKind::NumericConst { ty, bits } => Ok(match ty {
                hir::BuiltinType::I8 => {
                    let v = *bits as u8 as i8;
                    c_min_or_cast_i8(v)
                }
                hir::BuiltinType::I32 => c_min_or_cast_i32(*bits as i32),
                hir::BuiltinType::U8 => format!("(uint8_t){}", *bits as u8),
                hir::BuiltinType::U32 => format!("(uint32_t){}u", *bits as u32),
                hir::BuiltinType::I64 => c_min_or_cast_i64(*bits as i64),
                hir::BuiltinType::U64 => format!("(uint64_t){}ull", *bits),
                hir::BuiltinType::F32 => {
                    format!("a2x_f32_bits({}u)", *bits as u32)
                }
                hir::BuiltinType::F64 => {
                    format!("a2x_f64_bits({}ull)", *bits)
                }
                other => {
                    return Err(A2xReject::unsupported_type(format!(
                        "{other:?} is not a numeric builtin"
                    )))
                }
            }),
            // PLAN-961: conversion through the frozen helper matrix.
            hir::ExprKind::Cast { operand, to } => {
                let v = self.lower_pure(*operand)?;
                let from = self
                    .view
                    .resolve_use_ty(self.facts.body.exprs[operand.0 as usize].ty)
                    .map_err(|_| A2xReject::unsupported_type("cast source type"))?;
                let from_b = c_builtin_of(from)?;
                if from_b == *to {
                    return Ok(v);
                }
                let to_ty = builtin_ty_of(*to)?;
                let chan = c_result_ty(to_ty)?;
                let dom = c_dom_ty(builtin_ty_of(from_b)?)?;
                Ok(self.typed_call(
                    chan,
                    format!("a2x_c_{}_{}(({}){})", c_bname(from_b), c_bname(*to), dom, v),
                ))
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                // pin BOTH operands at their source-order positions first
                let l = self.lower_pure(*lhs)?;
                let r = self.lower_pure(*rhs)?;
                match op {
                    hir::BinOp::AddI32
                    | hir::BinOp::SubI32
                    | hir::BinOp::MulI32
                    | hir::BinOp::DivI32
                    | hir::BinOp::RemI32 => {
                        let func = match op {
                            hir::BinOp::AddI32 => "a2x_add",
                            hir::BinOp::SubI32 => "a2x_sub",
                            hir::BinOp::MulI32 => "a2x_mul",
                            hir::BinOp::DivI32 => "a2x_div",
                            _ => "a2x_rem",
                        };
                        let t = self.fresh_temp();
                        self.out
                            .push(format!("a2x_result {} = {}({}, {});", t, func, l, r));
                        self.check_trap(&t);
                        Ok(format!("{}.value", t))
                    }
                    hir::BinOp::LtI32
                    | hir::BinOp::LeI32
                    | hir::BinOp::GtI32
                    | hir::BinOp::GeI32
                    | hir::BinOp::Eq
                    | hir::BinOp::Ne => {
                        let o = match op {
                            hir::BinOp::LtI32 => "<",
                            hir::BinOp::LeI32 => "<=",
                            hir::BinOp::GtI32 => ">",
                            hir::BinOp::GeI32 => ">=",
                            hir::BinOp::Eq => "==",
                            _ => "!=",
                        };
                        Ok(format!("(({}) {} ({}))", l, o, r))
                    }
                    hir::BinOp::Num { op, ty } => self.num_bin_c(*op, *ty, l, r),
                }
            }
            hir::ExprKind::Unary { op, operand } => {
                let o = self.lower_pure(*operand)?;
                match op {
                    hir::UnaryOp::NegI32 => {
                        let t = self.fresh_temp();
                        self.out.push(format!("a2x_result {} = a2x_neg({});", t, o));
                        self.check_trap(&t);
                        Ok(format!("{}.value", t))
                    }
                    hir::UnaryOp::NotBool => Ok(format!("(!({}))", o)),
                    // PLAN-961: typed negation (signed checked; float
                    // sign flip inline).
                    hir::UnaryOp::NegNumeric(bty) => {
                        if matches!(bty, hir::BuiltinType::F32 | hir::BuiltinType::F64) {
                            return Ok(format!("(-({}))", o));
                        }
                        let name = match bty {
                            hir::BuiltinType::I8 => "a2x_neg_i8",
                            hir::BuiltinType::I64 => "a2x_neg_i64",
                            other => {
                                return Err(A2xReject::unsupported_op(format!(
                                    "typed negation on {other:?}"
                                )))
                            }
                        };
                        Ok(self.typed_call("a2x_result_i64", format!("{name}((int64_t){})", o)))
                    }
                }
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // LHS always evaluates (its statements go to the outer
                // stream); the RHS's statements live inside the guard so
                // an untaken side never runs (short-circuit).
                let l = self.lower_pure(*lhs)?;
                let mut side = FnLowering {
                    view: self.view,
                    facts: self.facts,
                    ret_chan: self.ret_chan,
                    local_names: clone_map(&self.local_names),
                    out: Vec::new(),
                    temp_counter: self.temp_counter,
                    loop_stack: self.loop_stack.clone(),
                };
                let r = side.lower_pure(*rhs)?;
                let side_stmts = side.out;
                self.temp_counter = side.temp_counter;
                let t = self.fresh_temp();
                self.out.push(format!("bool {} = (bool)({});", t, l));
                // the guard reads the MATERIALIZED temp (never the raw
                // expression — a constant LHS would re-trigger C4127)
                let guard = match op {
                    hir::LogicOp::And => format!("if ({}) {{", t),
                    hir::LogicOp::Or => format!("if (!{}) {{", t),
                };
                self.out.push(guard);
                for s in &side_stmts {
                    self.out.push(format!("    {}", s));
                }
                self.out.push(format!("{} = (bool)({});", t, r));
                self.out.push("}".to_string());
                Ok(t)
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                let target_id = callee.0 as usize;
                if let Some(declaration) = self.view.foreign_by_def(target_id).cloned() {
                    declaration
                        .validate()
                        .map_err(|e| A2xReject::new("a2c.foreign-contract", e))?;
                    let mut evaluated = Vec::new();
                    for arg in eval_args {
                        let expression = self.lower_pure(*arg)?;
                        let temp = self.fresh_temp();
                        self.out.push(format!("int32_t {temp} = {expression};"));
                        evaluated.push(temp);
                    }
                    let mut placed = vec![None; declaration.params().len()];
                    for binding in bindings {
                        placed[binding.param as usize] =
                            Some(evaluated[binding.arg as usize].clone());
                    }
                    let mut arguments = Vec::new();
                    self.out.push(crate::foreign::prototype(&declaration));
                    self.out
                        .push("extern void ace_foreign_fail(int32_t status);".into());
                    if let Some(symbol) = &declaration.context_symbol {
                        self.out.push(format!("extern unsigned char {symbol};"));
                        arguments.push(format!("(void *)&{symbol}"));
                    }
                    for (value, lane) in placed.into_iter().zip(declaration.params()) {
                        let value = value.ok_or_else(|| {
                            A2xReject::new("a2c.foreign-binding", "missing foreign argument")
                        })?;
                        if *lane == ace_foreign_contract::LaneKind::Bool {
                            self.out.push(format!(
                                "if ((uint32_t){value} > 1u) ace_foreign_fail(INT32_MIN + 1);"
                            ));
                            arguments.push(format!("(uint8_t){value}"));
                        } else {
                            arguments.push(value);
                        }
                    }
                    let result = self.fresh_temp();
                    let c_type = if declaration.result() == ace_foreign_contract::LaneKind::Bool {
                        "uint8_t"
                    } else {
                        "int32_t"
                    };
                    if matches!(
                        declaration.function.form,
                        ace_foreign_contract::FnForm::Fallible { .. }
                    ) {
                        self.out.push(format!("{c_type} {result};"));
                        arguments.push(format!("&{result}"));
                        let status = self.fresh_temp();
                        self.out.push(format!(
                            "int32_t {status} = {}({});",
                            declaration.function.symbol,
                            arguments.join(", ")
                        ));
                        self.out
                            .push(format!("if ({status} != 0) ace_foreign_fail({status});"));
                    } else {
                        self.out.push(format!(
                            "{c_type} {result} = {}({});",
                            declaration.function.symbol,
                            arguments.join(", ")
                        ));
                    }
                    if declaration.result() == ace_foreign_contract::LaneKind::Bool {
                        self.out.push(format!(
                            "if ({result} > 1u) ace_foreign_fail(INT32_MIN + 1);"
                        ));
                    }
                    return Ok(format!("(int32_t){result}"));
                }
                if let Some(sym) = self.view.trace_symbol(target_id) {
                    if !eval_args.is_empty() {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "trace intrinsic call carries arguments",
                        ));
                    }
                    let func = match sym {
                        "hir.test.mark_a" => "a2x_mark_a",
                        _ => "a2x_mark_b",
                    };
                    let t = self.fresh_temp();
                    self.out.push(format!("int32_t {} = {}();", t, func));
                    return Ok(t);
                }
                // resolve the callee (internal function or fragment
                // external) and its parameter list
                let (c_name, target_params) = if let Some(t) = self.view.fn_by_def(target_id) {
                    (c_name_of(t), t.params.clone())
                } else if let Some(x) = self.view.external_by_def(target_id) {
                    let n = match x.kind {
                        FnKind::Free => c_fn_name(&x.source_name),
                        FnKind::Method { .. } => c_method_name(&x.source_name),
                    };
                    (
                        n,
                        x.params
                            .iter()
                            .map(|ty| ace_a2x_core::ParamFacts {
                                name: String::new(),
                                ty: *ty,
                                mode: hir::ParamMode::View,
                            })
                            .collect(),
                    )
                } else {
                    return Err(A2xReject::unsupported_expr(format!(
                        "call target def #{} is not an admitted function",
                        target_id
                    )));
                };
                // Source order first: EVERY argument pins at its
                // source-order position (single evaluation, effects before
                // placement)
                let mut slots: Vec<String> = Vec::with_capacity(eval_args.len());
                for &arg in eval_args {
                    slots.push(self.lower_pure(arg)?);
                }
                // Placement by the binding map, in PARAM order
                let n = target_params.len();
                let mut placed: Vec<Option<String>> = vec![None; n];
                for b in bindings {
                    let arg = slots.get(b.arg as usize).cloned().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "binding arg index out of range")
                    })?;
                    let p = b.param as usize;
                    if p >= n {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "binding param index out of range",
                        ));
                    }
                    placed[p] = Some(arg);
                }
                let args = placed
                    .into_iter()
                    .map(|a| a.unwrap_or_else(|| "0".to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                let t = self.fresh_temp();
                self.out
                    .push(format!("a2x_result {} = {}({});", t, c_name, args));
                self.check_trap(&t);
                Ok(format!("{}.value", t))
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "the C scalar channel does not serve `{:?}`",
                std::mem::discriminant(other)
            ))),
        }
    }

    fn lower_block(&mut self, block_id: hir::BlockId) -> Result<(), A2xReject> {
        let block = self
            .facts
            .body
            .blocks
            .get(block_id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "block index out of range"))?;
        for stmt in &block.stmts {
            self.lower_stmt(stmt)?;
        }
        Ok(())
    }

    fn lower_stmt(&mut self, s: &hir::Stmt) -> Result<(), A2xReject> {
        match s {
            hir::Stmt::Let { local, value } => {
                let v = self.lower_pure(*value)?;
                let name = self.local_name(local.0)?.to_string();
                // the local is DECLARED at function top; a let is its
                // initializing assignment (C11 forbids same-scope redecl)
                self.out.push(format!("{} = {};", name, v));
            }
            hir::Stmt::Return { value } => {
                let v = self.lower_pure(*value)?;
                // PLAN-961: the return channel follows the expr's own
                // type (typed numerics wrap in their own OK struct).
                let rty = self
                    .view
                    .resolve_use_ty(self.facts.body.exprs[value.0 as usize].ty)
                    .map_err(|_| A2xReject::unsupported_type("return type outside the channel"))?;
                match rty {
                    Ty::I32 | Ty::Bool => {
                        self.out.push(format!("return a2x_value({});", v));
                    }
                    Ty::I8 | Ty::I64 => {
                        self.out.push(format!(
                            "return (a2x_result_i64){{ (int64_t)({}), A2X_TRAP_NONE }};",
                            v
                        ));
                    }
                    Ty::U8 | Ty::U32 | Ty::U64 => {
                        self.out.push(format!(
                            "return (a2x_result_u64){{ (uint64_t)({}), A2X_TRAP_NONE }};",
                            v
                        ));
                    }
                    Ty::F32 => {
                        self.out.push(format!(
                            "return (a2x_result_f32){{ (float)({}), A2X_TRAP_NONE }};",
                            v
                        ));
                    }
                    Ty::F64 => {
                        self.out.push(format!(
                            "return (a2x_result_f64){{ (double)({}), A2X_TRAP_NONE }};",
                            v
                        ));
                    }
                    _ => {
                        return Err(A2xReject::unsupported_type(
                            "return type outside the numeric channels",
                        ))
                    }
                }
            }
            hir::Stmt::If { cond, then, els } => {
                // the condition materializes into a bool temporary first:
                // uniform single-evaluation, and a source-level constant
                // condition (e.g. `while false`) stops being a
                // syntactically-constant controlling expression (C4127)
                let c = self.lower_pure(*cond)?;
                let t = self.fresh_temp();
                self.out.push(format!("bool {} = (bool)({});", t, c));
                self.out.push(format!("if ({}) {{", t));
                self.lower_block(*then)?;
                if let Some(eb) = els {
                    self.out.push("} else {".to_string());
                    self.lower_block(*eb)?;
                }
                self.out.push("}".to_string());
            }
            hir::Stmt::Loop { id, body } => {
                self.loop_stack.push(id.0);
                self.out.push("for (;;) {".to_string());
                self.lower_block(*body)?;
                self.out.push("}".to_string());
                self.loop_stack.pop();
            }
            hir::Stmt::Assign { place, value } => {
                let place = self
                    .facts
                    .body
                    .places
                    .get(place.0 as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "place index out of range"))?;
                match &place.kind {
                    hir::PlaceKind::Local => {
                        let v = self.lower_pure(*value)?;
                        let name = self.local_name(place.local.0)?.to_string();
                        self.out.push(format!("{} = {};", name, v));
                    }
                    hir::PlaceKind::FieldPath(_) => {
                        return Err(A2xReject::unsupported_place(
                            "field-path assignment is outside the C scalar channel",
                        ))
                    }
                }
            }
            hir::Stmt::Break { target } => {
                self.assert_nearest(target.0, "break")?;
                self.out.push("break;".to_string());
            }
            hir::Stmt::Continue { target } => {
                self.assert_nearest(target.0, "continue")?;
                self.out.push("continue;".to_string());
            }
            other => {
                return Err(A2xReject::unsupported_stmt(format!(
                    "in `{}`: statement kind `{:?}` is outside the C scalar surface",
                    self.facts.name, other
                )))
            }
        }
        Ok(())
    }

    fn assert_nearest(&self, target: u32, what: &str) -> Result<(), A2xReject> {
        match self.loop_stack.last() {
            Some(top) if *top == target => Ok(()),
            _ => Err(A2xReject::new(
                "a2x.internal",
                format!("{} target is not the nearest loop", what),
            )),
        }
    }
}

fn clone_map(m: &HashMap<u32, String>) -> HashMap<u32, String> {
    m.iter().map(|(k, v)| (*k, v.clone())).collect()
}

/// Lower ONE function of the view to its [`CFn`].
pub fn lower_fn_c(view: &SemanticView<'_>, facts: &FnFacts<'_>) -> Result<CFn, A2xReject> {
    if view.foreign_declarations().next().is_some() {
        return Err(A2xReject::new(
            "a2c.foreign-authority",
            "foreign HIR requires the explicit checked foreign target",
        ));
    }
    lower_foreign_fn_c(view, facts)
}

pub(crate) fn lower_foreign_fn_c(
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
) -> Result<CFn, A2xReject> {
    // local names: index-prefixed sanitized (stable across whole-module
    // and single-body bundles — same scheme as the Rust target)
    let mut local_names = HashMap::new();
    for (idx, local) in facts.body.locals.iter().enumerate() {
        let s = sanitize(&local.name);
        let name = if s.is_empty() {
            format!("l{}", idx)
        } else {
            format!("l{}_{}", idx, s)
        };
        local_names.insert(idx as u32, name);
    }
    // parameter locals in parameter order
    let mut param_locals: Vec<(u32, u32)> = facts
        .body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
        .map(|(i, l)| (l.index.unwrap_or(u32::MAX), i as u32))
        .collect();
    param_locals.sort();
    let param_locals: Vec<u32> = param_locals.into_iter().map(|(_, i)| i).collect();
    if param_locals.len() != facts.params.len() {
        return Err(A2xReject::new(
            "a2x.internal",
            "param locals disagree with collected params",
        ));
    }
    let mut params = Vec::with_capacity(param_locals.len());
    for &id in &param_locals {
        let ty = view
            .resolve_use_ty(facts.body.locals[id as usize].ty)
            .map_err(|_| {
                A2xReject::unsupported_type("parameter type outside the C scalar channel")
            })?;
        params.push((c_ty(ty)?.to_string(), local_names[&id].clone()));
    }

    let ret_chan = c_result_ty(facts.result).unwrap_or("a2x_result");
    let ret_chan: &'static str = Box::leak(ret_chan.to_string().into_boxed_str());
    let mut lowering = FnLowering {
        view,
        facts,
        ret_chan,
        local_names,
        out: Vec::new(),
        temp_counter: 0,
        loop_stack: Vec::new(),
    };
    // declarations for every NON-param local, in index order (params are
    // the function's own parameters)
    for (idx, local) in facts.body.locals.iter().enumerate() {
        if matches!(local.role, hir::LocalRole::Param) {
            continue;
        }
        let ty = view.resolve_use_ty(local.ty)?;
        lowering.out.push(format!(
            "{} {};",
            c_ty(ty)?,
            lowering.local_names[&(idx as u32)]
        ));
    }
    // the entry block. Every path returns per the verifier (the same
    // trust the Rust target places in it); a would-be fall-off fails
    // LOUDLY at compile time (C4715 under /W4 /WX) instead of silently
    // producing a value.
    lowering.lower_block(hir::BlockId(0))?;

    Ok(CFn {
        name: c_name_of(facts),
        params,
        result_ty: facts.result,
        origin: facts.origin,
        body: lowering.out,
    })
}

/// PLAN-961 helpers mapping the HIR builtin axis onto C spellings.
pub(crate) fn builtin_ty_of(b: hir::BuiltinType) -> Result<Ty, A2xReject> {
    Ok(match b {
        hir::BuiltinType::I32 => Ty::I32,
        hir::BuiltinType::I8 => Ty::I8,
        hir::BuiltinType::U8 => Ty::U8,
        hir::BuiltinType::U32 => Ty::U32,
        hir::BuiltinType::I64 => Ty::I64,
        hir::BuiltinType::U64 => Ty::U64,
        hir::BuiltinType::F32 => Ty::F32,
        hir::BuiltinType::F64 => Ty::F64,
        other => {
            return Err(A2xReject::unsupported_type(format!(
                "{other:?} is not a numeric builtin"
            )))
        }
    })
}

pub(crate) fn c_builtin_of(t: Ty) -> Result<hir::BuiltinType, A2xReject> {
    Ok(match t {
        Ty::I32 => hir::BuiltinType::I32,
        Ty::I8 => hir::BuiltinType::I8,
        Ty::U8 => hir::BuiltinType::U8,
        Ty::U32 => hir::BuiltinType::U32,
        Ty::I64 => hir::BuiltinType::I64,
        Ty::U64 => hir::BuiltinType::U64,
        Ty::F32 => hir::BuiltinType::F32,
        Ty::F64 => hir::BuiltinType::F64,
        other => {
            return Err(A2xReject::unsupported_type(format!(
                "{other:?} is not numeric"
            )))
        }
    })
}

pub(crate) fn c_bname(b: hir::BuiltinType) -> &'static str {
    match b {
        hir::BuiltinType::I8 => "i8",
        hir::BuiltinType::I32 => "i32",
        hir::BuiltinType::U8 => "u8",
        hir::BuiltinType::U32 => "u32",
        hir::BuiltinType::I64 => "i64",
        hir::BuiltinType::U64 => "u64",
        hir::BuiltinType::F32 => "f32",
        hir::BuiltinType::F64 => "f64",
        _ => "i32",
    }
}

/// MIN spellings never round-trip through a wider type.
fn c_min_or_cast_i8(v: i8) -> String {
    if v == i8::MIN {
        "(int8_t)(-127 - 1)".to_string()
    } else {
        format!("(int8_t){v}")
    }
}
fn c_min_or_cast_i32(v: i32) -> String {
    if v == i32::MIN {
        "(int32_t)(-2147483647 - 1)".to_string()
    } else {
        format!("(int32_t){v}")
    }
}
fn c_min_or_cast_i64(v: i64) -> String {
    if v == i64::MIN {
        "(int64_t)(-9223372036854775807LL - 1)".to_string()
    } else {
        format!("(int64_t){v}LL")
    }
}
