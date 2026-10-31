//! Render the lowered Rust IR to source text (plan §2: provider-side
//! legalization/type mapping/printing — the last provider stage).
//!
//! Rendering rules frozen here:
//! * `i32::MIN` prints as the named constant (a plain `-2147483648` literal
//!   is a negation of an out-of-range literal in Rust; the boundary value
//!   must print correctly, plan §5.1).
//! * A fallible root consumed as a value gets `?` appended (`render_val`);
//!   pure expressions render bare.
//! * Short-circuit sides always render parenthesized `(if lhs { … } else {
//!   false })` so they are safe in any operator/condition position.
//! * No tail `unreachable!`: the Auto verifier's all-paths-return guarantee
//!   is re-checked by rustc itself — a missing tail is a lowering bug that
//!   must surface as a compile error, not be masked.
//! * PLAN-902: record/text renderings — `Rec_<Name>`/`f_<field>`, owned
//!   `A2xStr`, borrowed `&'a str`, mode-legalized parameters, explicit
//!   `a2x_str_free` cleanup statements, string literals escaped to valid
//!   Rust (UTF-8 kept, control bytes escaped) so NUL etc. round-trip.

use crate::lower::{CmpOp, RBlock, RExpr, RStmt, RustFn, RustParam, RustProgram, RustRecord};
use ace_a2x_core::Ty;
use ace_core::hir::ParamMode;
use std::collections::HashMap;

/// Rendering context: nominal record names by type-table index.
struct PrintCtx<'p> {
    records: HashMap<u32, &'p RustRecord>,
}

impl<'p> PrintCtx<'p> {
    fn new(program: &'p RustProgram) -> Self {
        let mut records = HashMap::new();
        for r in &program.records {
            records.insert(r.type_index, r);
        }
        PrintCtx { records }
    }

    /// Rust type text of a target-neutral type. Scalar types render
    /// exactly as before (byte-identical scalar output).
    fn ty_str(&self, ty: Ty) -> String {
        match ty {
            Ty::I32 => "i32".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Char => "char".to_string(),
            Ty::Str => "&str".to_string(),
            Ty::StrOwned => "A2xStr".to_string(),
            Ty::Record(id) => self
                .records
                .get(&id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Rec_#{}", id)),
        }
    }
}

/// Render a whole program to the `generated.rs` body (record declarations
/// + function definitions only; module wiring lives in the wrapper emitted
/// by artifacts.rs).
pub fn print_program(p: &RustProgram) -> String {
    let ctx = PrintCtx::new(p);
    let mut out = String::new();
    for r in &p.records {
        out.push_str(&print_record(&ctx, r));
        out.push('\n');
    }
    for f in &p.functions {
        out.push_str(&print_fn_in(&ctx, f, p.borrowed));
        out.push('\n');
    }
    out
}

/// Render one function (records context needed only for record/text
/// programs; kept for scalar-compatible external use).
pub fn print_fn(f: &RustFn) -> String {
    print_fn_in(&PrintCtx { records: HashMap::new() }, f, false)
}

fn print_record(ctx: &PrintCtx<'_>, r: &RustRecord) -> String {
    // Copy records clone/move by value; resource records (transitive owned
    // Str) derive nothing — clone and cleanup are explicit typed helpers.
    let derive = if r.resource {
        ""
    } else {
        "#[derive(Clone, Copy)]\n"
    };
    let fields: Vec<String> = r
        .fields
        .iter()
        .map(|(name, ty)| format!("    pub {}: {},", name, ctx.ty_str(*ty)))
        .collect();
    format!(
        "{}pub struct {} {{\n{}\n}}\n",
        derive,
        r.name,
        fields.join("\n")
    )
}

fn print_fn_in(ctx: &PrintCtx<'_>, f: &RustFn, _program_borrowed: bool) -> String {
    let mut out = format!(
        "// origin: {}..{} (def `{}`, byte span in the compiled source)\n",
        f.origin.0, f.origin.1, f.name
    );
    let params: Vec<String> = f.params.iter().map(|p| render_param(ctx, p)).collect();
    // per-function: only signatures that actually carry references get 'a
    let borrowed = f.params.iter().any(|p| param_borrows(p)) || matches!(f.result, Ty::Str);
    let life = if borrowed { "<'a>" } else { "" };
    out.push_str(&format!(
        "pub fn {}{}({}) -> A2xResult<{}> {{\n",
        f.name,
        life,
        params.join(", "),
        render_result(ctx, f.result)
    ));
    for s in &f.body {
        out.push_str(&render_stmt(ctx, s, 1));
    }
    out.push_str("}\n");
    out
}

/// Whether a parameter passes by reference (mirror of the lowering's
/// `borrows` rule).
fn param_borrows(p: &RustParam) -> bool {
    match p.mode {
        ParamMode::MutBorrow => true,
        ParamMode::MoveOwn => false,
        ParamMode::View => matches!(p.ty, Ty::Str | Ty::StrOwned | Ty::Record(_)),
    }
}

/// Mode-legalized parameter text (PLAN-902 §5.2): view text params borrow
/// content (`&'a str`), view record params shared-borrow (`&'a Rec_X`),
/// mut params exclusive-borrow, move params take the value.
fn render_param(ctx: &PrintCtx<'_>, p: &RustParam) -> String {
    let ty = match (p.mode, p.ty) {
        (ParamMode::View, Ty::Str | Ty::StrOwned) => "&'a str".to_string(),
        (ParamMode::View, Ty::Record(_)) => format!("&'a {}", ctx.ty_str(p.ty)),
        (ParamMode::MutBorrow, Ty::Record(_)) => format!("&'a mut {}", ctx.ty_str(p.ty)),
        (ParamMode::MutBorrow, Ty::StrOwned) => "&'a mut A2xStr".to_string(),
        (ParamMode::MutBorrow, Ty::Str) => "&'a mut A2xStr".to_string(),
        (ParamMode::MoveOwn, Ty::StrOwned) => "A2xStr".to_string(),
        (ParamMode::MoveOwn, Ty::Record(_)) => ctx.ty_str(p.ty),
        (_, Ty::Str) => "&'a str".to_string(),
        (_, other) => ctx.ty_str(other),
    };
    format!("{}: {}", p.name, ty)
}

fn render_result(ctx: &PrintCtx<'_>, ty: Ty) -> String {
    match ty {
        Ty::Str => "&'a str".to_string(),
        other => ctx.ty_str(other),
    }
}

fn indent(n: usize) -> String {
    "    ".repeat(n)
}

fn render_stmt(ctx: &PrintCtx<'_>, s: &RStmt, ind: usize) -> String {
    let pad = indent(ind);
    match s {
        RStmt::Let {
            name,
            ty,
            mutable,
            init,
        } => {
            let m = if *mutable { "mut " } else { "" };
            format!(
                "{}let {}{}: {} = {};\n",
                pad,
                m,
                name,
                local_ty_text(ctx, *ty),
                render_val(init)
            )
        }
        RStmt::Assign { name, value } => format!("{}{} = {};\n", pad, name, render_val(value)),
        RStmt::AssignField { name, path, value } => {
            let mut target = name.clone();
            for f in path {
                target.push('.');
                target.push_str(f);
            }
            format!("{}{} = {};\n", pad, target, render_val(value))
        }
        RStmt::Free { name } => format!("{}a2x_str_free(&mut {});\n", pad, name),
        RStmt::If { cond, then, els } => {
            let mut out = format!("{}if {} {{\n", pad, render_cond(cond));
            for s in then {
                out.push_str(&render_stmt(ctx, s, ind + 1));
            }
            if let Some(e) = els {
                out.push_str(&format!("{}}} else {{\n", pad));
                for s in e {
                    out.push_str(&render_stmt(ctx, s, ind + 1));
                }
            }
            out.push_str(&format!("{}}}\n", pad));
            out
        }
        RStmt::Loop { body } => {
            let mut out = format!("{}loop {{\n", pad);
            for s in body {
                out.push_str(&render_stmt(ctx, s, ind + 1));
            }
            out.push_str(&format!("{}}}\n", pad));
            out
        }
        RStmt::Break => format!("{}break;\n", pad),
        RStmt::Continue => format!("{}continue;\n", pad),
        RStmt::Return { value } => format!("{}return Ok({});\n", pad, render_val(value)),
    }
}

/// Local slot type text (record locals name their nominal type; text
/// locals use the runtime types).
fn local_ty_text(ctx: &PrintCtx<'_>, ty: Ty) -> String {
    match ty {
        Ty::I32 => "i32".to_string(),
        Ty::Bool => "bool".to_string(),
        Ty::Char => "char".to_string(),
        Ty::Str => "&str".to_string(),
        Ty::StrOwned => "A2xStr".to_string(),
        Ty::Record(id) => ctx.ty_str(Ty::Record(id)),
    }
}

/// Value rendering: bare expression, `?` appended for fallible roots.
fn render_val(e: &RExpr) -> String {
    let base = render_expr(e);
    if e.is_fallible_root() {
        format!("{}?", base)
    } else {
        base
    }
}

/// Condition/operand rendering: short-circuit forms are already
/// parenthesized by `render_expr`; used where a bool is consumed.
fn render_cond(e: &RExpr) -> String {
    render_val(e)
}

/// Escape a UTF-8 string into a valid Rust literal: printable ASCII and
/// non-ASCII UTF-8 stay literal; quotes/backslash/control bytes escape
/// (`\0`, `\n`, …, `\u{..}`) — NUL bytes round-trip exactly (T05).
fn escape_str_lit(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\u{{{:x}}}", c as u32))
            }
            c => out.push(c),
        }
    }
    out
}

fn escape_char_lit(c: char) -> String {
    match c {
        '\'' => "'\\''".to_string(),
        '\\' => "'\\\\'".to_string(),
        '\n' => "'\\n'".to_string(),
        '\r' => "'\\r'".to_string(),
        '\t' => "'\\t'".to_string(),
        '\0' => "'\\0'".to_string(),
        c if (c as u32) < 0x20 || c as u32 == 0x7f => format!("'\\u{{{:x}}}'", c as u32),
        c => format!("'{}'", c),
    }
}

fn render_expr(e: &RExpr) -> String {
    match e {
        RExpr::I32Lit(v) => {
            if *v == i32::MIN {
                "i32::MIN".to_string()
            } else {
                format!("{}", v)
            }
        }
        RExpr::BoolLit(b) => format!("{}", b),
        RExpr::StrLit(s) => format!("\"{}\"", escape_str_lit(s)),
        RExpr::CharLit(c) => escape_char_lit(*c),
        RExpr::Local(n) => n.clone(),
        RExpr::Checked { func, args } => {
            let args: Vec<String> = args.iter().map(render_val).collect();
            format!("{}({})", func, args.join(", "))
        }
        RExpr::Cmp { op, lhs, rhs } => {
            format!(
                "{} {} {}",
                render_operand(lhs),
                cmp_text(*op),
                render_operand(rhs)
            )
        }
        RExpr::Not(o) => format!("!{}", render_operand(o)),
        RExpr::AndSC { lhs, rhs } => {
            // lhs && rhs  =>  (if lhs { <rhs> } else { false })
            format!(
                "(if {} {{ {} }} else {{ false }})",
                render_cond(lhs),
                render_block_value(rhs)
            )
        }
        RExpr::OrSC { lhs, rhs } => {
            // lhs || rhs  =>  (if lhs { true } else { <rhs> } )
            format!(
                "(if {} {{ true }} else {{ {} }})",
                render_cond(lhs),
                render_block_value(rhs)
            )
        }
        RExpr::Call { func, args } => {
            let args: Vec<String> = args.iter().map(render_val).collect();
            format!("{}({})", func, args.join(", "))
        }
        RExpr::Trace { func } => format!("{}()", func),
        RExpr::StrFn { func, args } => {
            let args: Vec<String> = args.iter().map(render_val).collect();
            format!("{}({})", func, args.join(", "))
        }
        RExpr::StrCmp { eq, lhs, rhs } => {
            let f = if *eq { "a2x_str_eq" } else { "a2x_str_ne" };
            format!("{}({}, {})", f, render_val(lhs), render_val(rhs))
        }
        RExpr::AsStr { val, owned } => {
            if *owned {
                format!("{}.as_str()", render_val(val))
            } else {
                render_val(val)
            }
        }
        RExpr::RecCtor { record, fields } => {
            let inits: Vec<String> = fields
                .iter()
                .map(|(f, v)| format!("{}: {}", f, render_val(v)))
                .collect();
            format!("{} {{ {} }}", record, inits.join(", "))
        }
        RExpr::FieldPath { base, path } => {
            let mut out = render_operand(base);
            for f in path {
                out.push('.');
                out.push_str(f);
            }
            out
        }
        RExpr::Ref { inner } => format!("&{}", render_operand(inner)),
        RExpr::RefMut { inner } => format!("&mut {}", render_operand(inner)),
        RExpr::CharToI32 { inner } => format!("{} as i32", render_operand(inner)),
    }
}

/// An SC side's pinned statements + tail value, rendered as the branch
/// body of an if-expression. Empty preludes stay on one line; statements
/// render as a real block with the value as its tail expression.
fn render_block_value(side: &RBlock) -> String {
    if side.stmts.is_empty() {
        render_val(&side.value)
    } else {
        let ctx = PrintCtx { records: HashMap::new() };
        let mut inner = String::new();
        for s in &side.stmts {
            inner.push_str(&render_stmt(&ctx, s, 1));
        }
        format!("{{\n{}    {}\n}}", inner, render_val(&side.value))
    }
}

/// Operand rendering with parens around every non-leaf form so precedence
/// can never bind wrongly (nested comparisons — bool literals arrive as
/// `0 < 1` comparisons in the HIR — would otherwise chain).
fn render_operand(e: &RExpr) -> String {
    match e {
        RExpr::Local(_) | RExpr::I32Lit(_) | RExpr::BoolLit(_) => render_val(e),
        other => format!("({})", render_val(other)),
    }
}

fn cmp_text(op: CmpOp) -> &'static str {
    crate::lower::cmp_rust_str(op)
}
