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

use crate::lower::{CmpOp, RBlock, RExpr, RStmt, RustFn, RustProgram};
use ace_a2x_core::Ty;

fn ty_str(ty: Ty) -> &'static str {
    match ty {
        Ty::I32 => "i32",
        Ty::Bool => "bool",
    }
}

/// Render a whole program to the `generated.rs` body (function definitions
/// only; module wiring lives in the wrapper emitted by artifacts.rs).
pub fn print_program(p: &RustProgram) -> String {
    let mut out = String::new();
    for f in &p.functions {
        out.push_str(&print_fn(f));
        out.push('\n');
    }
    out
}

pub fn print_fn(f: &RustFn) -> String {
    let mut out = format!(
        "// origin: {}..{} (def `{}`, byte span in the compiled source)
",
        f.origin.0, f.origin.1, f.name
    );
    let params: Vec<String> = f
        .params
        .iter()
        .map(|(name, ty)| format!("{}: {}", name, ty_str(*ty)))
        .collect();
    out.push_str(&format!(
        "pub fn {}({}) -> A2xResult<{}> {{
",
        f.name,
        params.join(", "),
        ty_str(f.result)
    ));
    for s in &f.body {
        out.push_str(&render_stmt(s, 1));
    }
    out.push_str("}\n");
    out
}

fn indent(n: usize) -> String {
    "    ".repeat(n)
}

fn render_stmt(s: &RStmt, ind: usize) -> String {
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
                ty_str(*ty),
                render_val(init)
            )
        }
        RStmt::Assign { name, value } => format!("{}{} = {};\n", pad, name, render_val(value)),
        RStmt::If { cond, then, els } => {
            let mut out = format!("{}if {} {{\n", pad, render_cond(cond));
            for s in then {
                out.push_str(&render_stmt(s, ind + 1));
            }
            if let Some(e) = els {
                out.push_str(&format!("{}}} else {{\n", pad));
                for s in e {
                    out.push_str(&render_stmt(s, ind + 1));
                }
            }
            out.push_str(&format!("{}}}\n", pad));
            out
        }
        RStmt::Loop { body } => {
            let mut out = format!("{}loop {{\n", pad);
            for s in body {
                out.push_str(&render_stmt(s, ind + 1));
            }
            out.push_str(&format!("{}}}\n", pad));
            out
        }
        RStmt::Break => format!("{}break;\n", pad),
        RStmt::Continue => format!("{}continue;\n", pad),
        RStmt::Return { value } => format!("{}return Ok({});\n", pad, render_val(value)),
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
            // lhs || rhs  =>  (if lhs { true } else { <rhs> })
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
    }
}

/// An SC side's pinned statements + tail value, rendered as the branch
/// body of an if-expression. Empty preludes stay on one line; statements
/// render as a real block with the value as its tail expression.
fn render_block_value(side: &RBlock) -> String {
    if side.stmts.is_empty() {
        render_val(&side.value)
    } else {
        let mut inner = String::new();
        for s in &side.stmts {
            inner.push_str(&render_stmt(s, 1));
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
