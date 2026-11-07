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
use ace_a2x_core::{A2xReject, Ty};
use ace_core::hir::ParamMode;
use std::collections::HashMap;

/// Rendering context: nominal record names by type-table index.
struct PrintCtx<'p> {
    variants: HashMap<u32, &'p crate::lower::RustVariant>,
    records: HashMap<u32, &'p RustRecord>,
}

impl<'p> PrintCtx<'p> {
    fn new(program: &'p RustProgram) -> Self {
        let mut records = HashMap::new();
        for r in &program.records {
            records.insert(r.type_index, r);
        }
        let mut variants = HashMap::new();
        for v in &program.variants {
            variants.insert(v.type_index, v);
        }
        PrintCtx { records, variants }
    }

    /// Rust type text of a target-neutral type. Scalar types render
    /// exactly as before (byte-identical scalar output).
    fn ty_str(&self, ty: Ty) -> String {
        match ty {
            Ty::I32 => "i32".to_string(),
            // PLAN-961: the numeric names are their Rust spellings.
            Ty::I8 => "i8".to_string(),
            Ty::U8 => "u8".to_string(),
            Ty::U32 => "u32".to_string(),
            Ty::I64 => "i64".to_string(),
            Ty::U64 => "u64".to_string(),
            Ty::F32 => "f32".to_string(),
            Ty::F64 => "f64".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Char => "char".to_string(),
            Ty::Str => "&str".to_string(),
            Ty::StrOwned => "A2xStr".to_string(),
            Ty::Record(id) => self
                .records
                .get(&id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Rec_#{}", id)),
            Ty::Variant(id) => self
                .variants
                .get(&id)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format!("A2xv_#{}", id)),
            Ty::List(_) => "A2xList".to_string(),
            Ty::Unit => "()".to_string(),
        }
    }
}

/// Render a whole program to the `generated.rs` body (record declarations
/// + deep-clone helpers + function definitions only; module wiring lives in
/// the wrapper emitted by artifacts.rs).
pub fn print_program(p: &RustProgram) -> String {
    let ctx = PrintCtx::new(p);
    let mut out = String::new();
    for r in &p.records {
        out.push_str(&print_record(&ctx, r));
        out.push('\n');
    }
    for h in clone_helper_closure(p) {
        out.push_str(&h);
        out.push('\n');
    }
    // PLAN-904: nominal variants + elem impls + free/clone/take helpers
    for v in &p.variants {
        out.push_str(&print_variant(&ctx, v));
        out.push('\n');
    }
    if p.coll {
        for r in &p.records {
            out.push_str(&coll_record_helpers(&ctx, r, p));
            out.push('\n');
        }
        for v in &p.variants {
            out.push_str(&coll_variant_helpers(&ctx, v, p));
            out.push('\n');
        }
    }
    for f in &p.functions {
        out.push_str(&print_fn_in(&ctx, f, p.borrowed));
        out.push('\n');
    }
    out
}

/// The transitive deep-clone helper set for the records the program clones
/// (a helper that clones a nested resource-record field needs that field's
/// own helper). One allocation per non-empty owned-Str field; scalars and
/// Copy records copy by value.
fn clone_helper_closure(p: &RustProgram) -> Vec<String> {
    let mut needed: Vec<u32> = p.clones.clone();
    let mut i = 0;
    while i < needed.len() {
        let id = needed[i];
        if let Some(rec) = p.records.iter().find(|r| r.type_index == id) {
            for (_, ty) in &rec.fields {
                if let Ty::Record(inner) = ty {
                    if !needed.contains(inner) {
                        needed.push(*inner);
                    }
                }
            }
        }
        i += 1;
    }
    needed
        .iter()
        .filter_map(|id| p.records.iter().find(|r| r.type_index == *id))
        .map(|rec| clone_helper(rec, p))
        .collect()
}

fn clone_helper(rec: &RustRecord, p: &RustProgram) -> String {
    let mut inits = Vec::new();
    for (fname, ty) in &rec.fields {
        let init = match ty {
            Ty::StrOwned => format!("{fname}: a2x_str_clone(src.{fname}.as_str())?"),
            Ty::Record(inner) => {
                let inner_name = p
                    .records
                    .iter()
                    .find(|r| r.type_index == *inner)
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| "Rec".to_string());
                format!("{fname}: a2x_rt_clone_{inner_name}(&src.{fname})?")
            }
            _ => format!("{fname}: src.{fname}"),
        };
        inits.push(init);
    }
    format!(
        "pub fn a2x_rt_clone_{}(src: &{}) -> A2xResult<{}> {{\n    Ok({} {{ {} }})\n}}\n",
        rec.name,
        rec.name,
        rec.name,
        rec.name,
        inits.join(", ")
    )
}

/// Render one function (records context needed only for record/text
/// programs; kept for scalar-compatible external use).
pub fn print_fn(f: &RustFn) -> String {
    print_fn_in(
        &PrintCtx {
            records: HashMap::new(),
            variants: HashMap::new(),
        },
        f,
        false,
    )
}

/// Render one function against the module's RECORD table (the fragment
/// path: record-typed params/results/locals render their nominal names).
pub fn print_fn_records(records: &[crate::lower::RustRecord], f: &RustFn) -> String {
    let mut map = HashMap::new();
    for r in records {
        map.insert(r.type_index, r);
    }
    print_fn_in(
        &PrintCtx {
            records: map,
            variants: HashMap::new(),
        },
        f,
        false,
    )
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
        render_result(ctx, f.result, borrowed)
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

/// Mode-legalized parameter text (PLAN-902 §5.2): text view AND mut-text
/// params borrow content (`&'a str` — mut text is an exclusive view whose
/// exclusivity the VERIFIER proved at the call sites; no runtime mutation
/// surface exists for text), view record params shared-borrow, mut record
/// params exclusive-borrow (inherent write-back), move params take the
/// value.
fn render_param(ctx: &PrintCtx<'_>, p: &RustParam) -> String {
    let ty = match (p.mode, p.ty) {
        (_, Ty::Str | Ty::StrOwned) if p.mode != ParamMode::MoveOwn => "&'a str".to_string(),
        (ParamMode::View, Ty::Record(_)) => format!("&'a {}", ctx.ty_str(p.ty)),
        (ParamMode::MutBorrow, Ty::Record(_)) => format!("&'a mut {}", ctx.ty_str(p.ty)),
        (ParamMode::MoveOwn, Ty::StrOwned) => "A2xStr".to_string(),
        (ParamMode::MoveOwn, Ty::Record(_)) => ctx.ty_str(p.ty),
        (_, other) => ctx.ty_str(other),
    };
    format!("{}: {}", p.name, ty)
}

/// Result text. A borrowed `str` result borrows from the view params
/// (`&'a str`, provenance-verified); a literal-only str result from a
/// borrowing-free signature is `&'static str` (a promoted literal outlives
/// everything — the verifier's borrow-escape rules admit nothing else).
fn render_result(ctx: &PrintCtx<'_>, ty: Ty, borrowed: bool) -> String {
    match ty {
        Ty::Str if borrowed => "&'a str".to_string(),
        Ty::Str => "&'static str".to_string(),
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
        RStmt::Free { name, path } => {
            let mut target = name.clone();
            for f in path {
                target.push('.');
                target.push_str(f);
            }
            format!("{}a2x_str_free(&mut {});\n", pad, target)
        }
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
        // ---- PLAN-904 collections surface ----
        RStmt::FreeSlot { name, call } => format!("{}{}(&mut {});\n", pad, call, name),
        RStmt::CollLet {
            name,
            ty_str,
            mutable,
            init,
        } => {
            let m = if *mutable { "mut " } else { "" };
            format!(
                "{}let {}{}: {} = {};\n",
                pad,
                m,
                name,
                ty_str,
                render_val(init)
            )
        }
        RStmt::EvalExpr { expr } => format!("{}{};\n", pad, render_val(expr)),
    }
}

/// Local slot type text (record locals name their nominal type; text
/// locals use the runtime types).
fn local_ty_text(ctx: &PrintCtx<'_>, ty: Ty) -> String {
    match ty {
        Ty::I32 => "i32".to_string(),
        // PLAN-961: numeric locals use their Rust spellings.
        Ty::I8 => "i8".to_string(),
        Ty::U8 => "u8".to_string(),
        Ty::U32 => "u32".to_string(),
        Ty::I64 => "i64".to_string(),
        Ty::U64 => "u64".to_string(),
        Ty::F32 => "f32".to_string(),
        Ty::F64 => "f64".to_string(),
        Ty::Bool => "bool".to_string(),
        Ty::Char => "char".to_string(),
        Ty::Str => "&str".to_string(),
        Ty::StrOwned => "A2xStr".to_string(),
        Ty::Record(id) => ctx.ty_str(Ty::Record(id)),
        Ty::Variant(id) => ctx.ty_str(Ty::Variant(id)),
        Ty::List(id) => ctx.ty_str(Ty::List(id)),
        Ty::Unit => "()".to_string(),
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
        // ---- PLAN-904 collections surface ----
        RExpr::Wrap { inner, frees } => {
            if frees.is_empty() {
                return format!("{}?", render_expr(inner));
            }
            let mut out = String::from("match ");
            out.push_str(&render_expr(inner));
            out.push_str(" { Ok(v) => v, Err(e) => {");
            for (name, call) in frees {
                out.push_str(&format!(" {}(&mut {});", call, name));
            }
            out.push_str(" return Err(e) } }");
            out
        }
        RExpr::UnitExpr => "()".to_string(),
        RExpr::CollCall { func, turbo, args } => {
            let mut out = String::from(func);
            if let Some(t) = turbo {
                out.push_str("::<");
                out.push_str(t);
                out.push('>');
            }
            out.push('(');
            let rendered: Vec<String> = args.iter().map(render_operand).collect();
            out.push_str(&rendered.join(", "));
            out.push(')');
            out
        }
        RExpr::VariantIs {
            variant,
            arm,
            payload,
            root,
        } => {
            let pat = if *payload {
                format!("{}::{}(_)", variant, arm)
            } else {
                format!("{}::{}", variant, arm)
            };
            format!("::std::matches!(&{}, {})", render_operand(root), pat)
        }
        RExpr::VariantTake { helper, root } => {
            format!("{}({})", helper, render_operand(root))
        }
        RExpr::VariantCtorExpr {
            variant,
            arm,
            payload,
        } => match payload {
            Some(p) => format!("{}::{}({})", variant, arm, render_operand(p)),
            None => format!("{}::{}", variant, arm),
        },
    }
}

/// An SC side's pinned statements + tail value, rendered as the branch
/// body of an if-expression. Empty preludes stay on one line; statements
/// render as a real block with the value as its tail expression.
fn render_block_value(side: &RBlock) -> String {
    if side.stmts.is_empty() {
        render_val(&side.value)
    } else {
        let ctx = PrintCtx {
            records: HashMap::new(),
            variants: HashMap::new(),
        };
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

/// PLAN-902 T-05: the record-text ASSEMBLY prelude — nominal record
/// declarations plus the deep-clone helpers the given FRAGMENT texts
/// actually use (transitively for nested resource records). Derived from
/// the view + the session's own generated fragment texts, so a fully
/// resident assembly (zero lowering) still emits a complete `generated.rs`.
pub fn print_rt_prelude(
    view: &ace_a2x_core::SemanticView<'_>,
    fragment_texts: &[&str],
) -> Result<String, A2xReject> {
    let records = crate::lower::lower_records(view)?;
    Ok(print_rt_prelude_from(&records, fragment_texts))
}

/// The same prelude over a RECORD SNAPSHOT (the warm-assembly path keeps
/// the lowered record table alive after the fragment view's borrow ends).
pub fn print_rt_prelude_from(
    records: &[crate::lower::RustRecord],
    fragment_texts: &[&str],
) -> String {
    let mut out = String::new();
    rt_prelude_into(records, fragment_texts, &mut out);
    out
}

fn rt_prelude_into(
    records: &[crate::lower::RustRecord],
    fragment_texts: &[&str],
    out: &mut String,
) {
    // scan OUR OWN generated fragment texts for clone-helper uses
    let mut needed: Vec<String> = Vec::new();
    for text in fragment_texts {
        let mut rest = *text;
        while let Some(pos) = rest.find("a2x_rt_clone_") {
            let after = &rest[pos + "a2x_rt_clone_".len()..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && !needed.contains(&name) {
                needed.push(name);
            }
            rest = after;
        }
    }
    // resolve names to records, close transitively over nested fields
    let mut ids: Vec<u32> = needed
        .iter()
        .filter_map(|n| records.iter().find(|r| &r.name == n).map(|r| r.type_index))
        .collect();
    let mut i = 0;
    while i < ids.len() {
        if let Some(rec) = records.iter().find(|r| r.type_index == ids[i]) {
            for (_, ty) in &rec.fields {
                if let Ty::Record(inner) = ty {
                    if !ids.contains(inner) {
                        ids.push(*inner);
                    }
                }
            }
        }
        i += 1;
    }
    let fake = RustProgram {
        functions: vec![],
        entry: String::new(),
        marks: vec![],
        records: records.to_vec(),
        clones: ids.clone(),
        borrowed: false,
        variants: Vec::new(),
        coll: false,
    };
    let ctx = PrintCtx::new(&fake);
    for r in &fake.records {
        out.push_str(&print_record(&ctx, r));
        out.push('\n');
    }
    for id in ids {
        if let Some(rec) = fake.records.iter().find(|r| r.type_index == id) {
            out.push_str(&clone_helper(rec, &fake));
            out.push('\n');
        }
    }
}

/// PLAN-904: one nominal variant enum (arms in declaration order; the arm
/// INDEX is the runtime discriminant; payloads by value).
fn print_variant(ctx: &PrintCtx<'_>, v: &crate::lower::RustVariant) -> String {
    let arms: Vec<String> = v
        .arms
        .iter()
        .map(|(name, payload)| match payload {
            Some(t) => format!("    {}({}),", name, ctx.ty_str(*t)),
            None => format!("    {},", name),
        })
        .collect();
    format!(
        "#[allow(dead_code)]\npub enum {} {{\n{}\n}}\n",
        v.name,
        arms.join("\n")
    )
}

/// PLAN-904: per-record explicit helpers + the list-element impl.
fn coll_record_helpers(
    ctx: &PrintCtx<'_>,
    r: &crate::lower::RustRecord,
    p: &RustProgram,
) -> String {
    let _ = ctx;
    if !r.resource {
        // Copy records hold no resources: free is a no-op, clone is a value
        // copy with the ctor charge only.
        return format!(
            "fn a2x_coll_free_{n}(_v: &mut {n}) {{}}\nfn a2x_coll_clone_{n}(src: &{n}) -> A2xResult<{n}> {{ a2x_coll_charge_ctor()?; Ok(*src) }}\nimpl A2xCollElem for {n} {{\n    fn a2x_elem_free(&mut self) {{}}\n    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {{ a2x_coll_clone_{n}(self) }}\n    fn a2x_elem_str_bytes(&self) -> u64 {{ 0 }}\n}}\n",
            n = r.name
        );
    }
    // resource record: explicit frees in field order; sequential clone with
    // partial-output rollback (field k's failure frees the cloned k-1..0)
    let mut free_body = String::new();
    let mut bytes_body = String::new();
    for (fname, ty) in &r.fields {
        match ty {
            Ty::StrOwned => {
                free_body.push_str(&format!("    a2x_str_free(&mut v.{});\n", fname));
                bytes_body.push_str(&format!("    b += self.{}.as_str().len() as u64;\n", fname));
            }
            Ty::Record(id) => {
                let inner = record_name_of(p, *id);
                free_body.push_str(&format!("    a2x_coll_free_{}(&mut v.{});\n", inner, fname));
                bytes_body.push_str(&format!("    b += self.{}.a2x_elem_str_bytes();\n", fname));
            }
            _ => {}
        }
    }
    let mut clone_stmts = String::from("    a2x_coll_charge_ctor()?;\n");
    let mut owned: Vec<(String, String)> = Vec::new();
    for (fname, ty) in &r.fields {
        match ty {
            Ty::StrOwned => {
                clone_stmts.push_str(&format!(
                    "    let c_{f} = match a2x_str_clone(src.{f}.as_str()) {{ Ok(v) => v, Err(e) => {{ {rollback} return Err(e) }} }};\n",
                    f = fname,
                    rollback = rollback_stmts(&owned)
                ));
                owned.push((fname.clone(), "a2x_str_free".to_string()));
            }
            Ty::Record(id) => {
                let inner = record_name_of(p, *id);
                let call = format!("a2x_coll_free_{}", inner);
                clone_stmts.push_str(&format!(
                    "    let c_{f} = match a2x_coll_clone_{inner}(&src.{f}) {{ Ok(v) => v, Err(e) => {{ {rollback} return Err(e) }} }};\n",
                    f = fname,
                    inner = inner,
                    rollback = rollback_stmts(&owned)
                ));
                owned.push((fname.clone(), call));
            }
            _ => {}
        }
    }
    let inits: Vec<String> = r
        .fields
        .iter()
        .map(|(fname, ty)| match ty {
            Ty::StrOwned | Ty::Record(_) => format!("{}: c_{}", fname, fname),
            _ => format!("{}: src.{}", fname, fname),
        })
        .collect();
    clone_stmts.push_str(&format!("    Ok({} {{ {} }})\n", r.name, inits.join(", ")));
    format!(
        "fn a2x_coll_free_{n}(v: &mut {n}) {{\n{fb}}}\nfn a2x_coll_clone_{n}(src: &{n}) -> A2xResult<{n}> {{\n{cs}}}\nimpl A2xCollElem for {n} {{\n    fn a2x_elem_free(&mut self) {{ a2x_coll_free_{n}(self) }}\n    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {{ a2x_coll_clone_{n}(self) }}\n    fn a2x_elem_str_bytes(&self) -> u64 {{ let mut b = 0u64; {bb}    b }}\n}}\n",
        n = r.name,
        fb = free_body,
        cs = clone_stmts,
        bb = bytes_body
    )
}

fn record_name_of(p: &RustProgram, id: u32) -> String {
    p.records
        .iter()
        .find(|x| x.type_index == id)
        .map(|x| x.name.clone())
        .unwrap_or_else(|| "Rec".to_string())
}

fn variant_name_of(p: &RustProgram, id: u32) -> String {
    p.variants
        .iter()
        .find(|x| x.type_index == id)
        .map(|x| x.name.clone())
        .unwrap_or_else(|| "A2xv".to_string())
}

/// The error-path free statements for already-cloned owned fields.
fn rollback_stmts(owned: &[(String, String)]) -> String {
    let mut out = String::new();
    for (name, call) in owned.iter().rev() {
        out.push_str(&format!("{}(&mut c_{}); ", call, name));
    }
    out
}

/// PLAN-904: per-variant explicit helpers (free by active arm, per-arm
/// take with the runtime discriminant defense, deep clone) + the elem
/// impl.
fn coll_variant_helpers(
    ctx: &PrintCtx<'_>,
    v: &crate::lower::RustVariant,
    p: &RustProgram,
) -> String {
    let mut out = String::new();
    let mut free_arms = String::new();
    for (arm, payload) in &v.arms {
        let frees = match payload {
            Some(Ty::StrOwned) => "    a2x_str_free(p);\n".to_string(),
            Some(Ty::Record(id)) => {
                format!("    a2x_coll_free_{}(p);\n", record_name_of(p, *id))
            }
            Some(Ty::Variant(id)) => {
                format!("    a2x_coll_free_{}(p);\n", variant_name_of(p, *id))
            }
            _ => String::new(),
        };
        match payload {
            Some(_) => free_arms.push_str(&format!(
                "            {}::{}(ref mut p) => {{\n{}            }}\n",
                v.name, arm, frees
            )),
            None => free_arms.push_str(&format!("            {}::{} => {{}}\n", v.name, arm)),
        }
    }
    out.push_str(&format!(
        "fn a2x_coll_free_{n}(v: &mut {n}) {{\n    match v {{\n{arms}    }}\n}}\n",
        n = v.name,
        arms = free_arms
    ));
    for (arm, payload) in v.arms.iter() {
        if let Some(ty) = payload {
            out.push_str(&format!(
                "fn a2x_coll_take_{n}_{a}(v: {n}) -> A2xResult<{t}> {{\n    match v {{\n        {n}::{a}(p) => Ok(p),\n        _ => Err(AutoTrap::VariantBranch),\n    }}\n}}\n",
                n = v.name,
                a = arm,
                t = ctx.ty_str(*ty)
            ));
        }
    }
    let mut clone_arms = String::new();
    for (arm, payload) in &v.arms {
        match payload {
            Some(Ty::StrOwned) => clone_arms.push_str(&format!(
                "            {n}::{a}(p) => Ok({n}::{a}(a2x_str_clone(p.as_str())?)),\n",
                n = v.name,
                a = arm
            )),
            Some(Ty::Record(id)) => clone_arms.push_str(&format!(
                "            {n}::{a}(p) => Ok({n}::{a}(a2x_coll_clone_{inner}(p)?)),\n",
                n = v.name,
                a = arm,
                inner = record_name_of(p, *id)
            )),
            Some(Ty::Variant(id)) => clone_arms.push_str(&format!(
                "            {n}::{a}(p) => Ok({n}::{a}(a2x_coll_clone_{inner}(p)?)),\n",
                n = v.name,
                a = arm,
                inner = variant_name_of(p, *id)
            )),
            Some(_) => clone_arms.push_str(&format!(
                "            {n}::{a}(p) => Ok({n}::{a}(p.clone())),\n",
                n = v.name,
                a = arm
            )),
            None => clone_arms.push_str(&format!(
                "            {n}::{a} => Ok({n}::{a}),\n",
                n = v.name,
                a = arm
            )),
        }
    }
    out.push_str(&format!(
        "fn a2x_coll_clone_{n}(src: &{n}) -> A2xResult<{n}> {{\n    a2x_coll_charge_ctor()?;\n    match src {{\n{arms}    }}\n}}\n",
        n = v.name,
        arms = clone_arms
    ));
    out.push_str(&format!(
        "impl A2xCollElem for {n} {{\n    fn a2x_elem_free(&mut self) {{ a2x_coll_free_{n}(self) }}\n    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {{ a2x_coll_clone_{n}(self) }}\n    fn a2x_elem_str_bytes(&self) -> u64 {{ match self {{",
        n = v.name
    ));
    for (arm, payload) in &v.arms {
        match payload {
            Some(Ty::StrOwned) => out.push_str(&format!(
                " {n}::{a}(p) => p.as_str().len() as u64,",
                n = v.name,
                a = arm
            )),
            Some(Ty::Record(_)) | Some(Ty::Variant(_)) => out.push_str(&format!(
                " {n}::{a}(p) => p.a2x_elem_str_bytes(),",
                n = v.name,
                a = arm
            )),
            Some(_) => out.push_str(&format!(" {n}::{a}(_) => 0,", n = v.name, a = arm)),
            None => out.push_str(&format!(" {n}::{a} => 0,", n = v.name, a = arm)),
        }
    }
    out.push_str(" }}\n}\n");
    out
}

/// PLAN-904: render ONE collections function against the module's record
/// and variant tables (the store's per-fragment text).
pub fn print_coll_fn(
    records: &[crate::lower::RustRecord],
    variants: &[crate::lower::RustVariant],
    f: &RustFn,
) -> String {
    let mut map = HashMap::new();
    for r in records {
        map.insert(r.type_index, r);
    }
    let mut vmap = HashMap::new();
    for v in variants {
        vmap.insert(v.type_index, v);
    }
    print_fn_in(
        &PrintCtx {
            records: map,
            variants: vmap,
        },
        f,
        false,
    )
}

/// PLAN-904: the collections assembly prelude — nominal record structs,
/// variant enums and the explicit free/clone/take helpers + elem impls
/// (everything except the function texts).
pub fn print_coll_prelude_from(
    records: &[crate::lower::RustRecord],
    variants: &[crate::lower::RustVariant],
) -> String {
    let fake = RustProgram {
        functions: vec![],
        entry: String::new(),
        marks: vec![],
        records: records.to_vec(),
        clones: Vec::new(),
        borrowed: false,
        variants: variants.to_vec(),
        coll: true,
    };
    let ctx = PrintCtx::new(&fake);
    let mut out = String::new();
    for r in records {
        out.push_str(&print_record(&ctx, r));
        out.push('\n');
    }
    for v in variants {
        out.push_str(&print_variant(&ctx, v));
        out.push('\n');
    }
    for r in records {
        out.push_str(&coll_record_helpers(&ctx, r, &fake));
        out.push('\n');
    }
    for v in variants {
        out.push_str(&coll_variant_helpers(&ctx, v, &fake));
        out.push('\n');
    }
    out
}
