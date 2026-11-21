//! PLAN-1026: the `use.py` front-door transform.
//!
//! Runs BETWEEN parse and resolve on the AGGREGATES py lane only (the
//! compile entry calls it when the program carries `use.py` items). It
//! injects, in source order:
//!
//! * the frozen runtime helper (`__py_dlen` — a plain if-chain fn; the
//!   frame wire only carries NON-NEGATIVE i32 patterns, so the negative
//!   domain is unreachable by construction) — the f64/text reconstruction
//!   lives in the lowering emitters (`__py_f64_bits`/`__py_staged_text`
//!   intrinsics), NOT in user-surface helpers;
//! * one hoisted `var <member> = py_import("<static frame>")` per
//!   imported member, prepended to the script body (each import runs
//!   exactly once at entry, in source order — R-01/V-03).
//!
//! The resolver's py-op sugar recognizes the hoisted bindings (an i32
//! local whose initializer is a `py_import` foreign call) and rewrites
//! uses through the frozen Python op set; nothing here parses or types.

use super::ast::{Arg, Expr, ExprKind, Ident, Item, Program, Stmt};
use crate::atom_text::Span;
use crate::py_ops::{PyOpFact, PyOpKind};

/// The frozen helpers (aggregates surface; `2147483647 + 1` split avoids
/// the i32 literal-range rejection while staying exact at runtime; the
/// word frame f-string carries `\n` escapes — single line).
const HELPERS: &str = r#"fn __py_dlen(n int) int {
var d = 1
if n >= 10 { d = d + 1 }
if n >= 100 { d = d + 1 }
if n >= 1000 { d = d + 1 }
if n >= 10000 { d = d + 1 }
if n >= 100000 { d = d + 1 }
if n >= 1000000 { d = d + 1 }
if n >= 10000000 { d = d + 1 }
if n >= 100000000 { d = d + 1 }
if n >= 1000000000 { d = d + 1 }
return d }
fn __py_chunk_o(n int) Str {
return f"o ${__py_dlen(n)}\n$n\n" }
fn __py_chunk_i(n int) Str {
return f"i ${__py_dlen(n)}\n$n\n" }
fn __py_chunk_b(n int) Str {
return f"b 1\n$n\n" }
"#;

/// One `p967.frame/1` static import frame (the frozen wire layout for a
/// module path + member name; every payload LF-terminated, a kwarg name
/// abuts its value header line — byte-identical to the delivered
/// validator's frozen format).
pub fn import_frame(module: &str, member: &str) -> String {
    format!(
        "p967.frame/1\nr n 0\n\np 0\nk 2\n3\nmodt {mlen}\n{module}\n6\nmembert {elen}\n{member}\n",
        mlen = module.len(),
        elen = member.len(),
    )
}

/// Whether the program carries at least one `use.py` item.
pub fn has_use_py(program: &Program) -> bool {
    program.items.iter().any(|i| matches!(i, Item::UsePy(_)))
}

/// The transform: inject helpers + hoisted member imports. Returns the
/// number of hoisted imports (0 when the program has no use.py items —
/// the helpers are then not injected either; the py lane is opt-in by
/// the import's presence).
pub fn front_door(program: &mut Program, user_source_len: usize) -> usize {
    let mut imports: Vec<(String, String, Span)> = Vec::new();
    for item in &program.items {
        if let Item::UsePy(u) = item {
            let module = u
                .module
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>()
                .join(".");
            for m in &u.members {
                imports.push((module.clone(), m.name.clone(), m.span));
            }
        }
    }
    let count = imports.len();
    if imports.is_empty() {
        return 0;
    }
    // 1) the frozen helpers as fn items, parsed once from their own
    //    program so the AST construction is identical to user sources.
    let helper_program = super::parse_program_with(HELPERS, super::SourceProfile::Aggregates)
        .expect("the frozen __py helpers parse under the aggregates surface");
    let mut helper_items: Vec<Item> = helper_program
        .items
        .into_iter()
        .filter(|i| matches!(i, Item::Fn(_)))
        .collect();
    // PLAN-1094: the helpers parse from their OWN frozen text, so their
    // spans live in the helpers' byte coordinates. Re-base them past the
    // user source — the shared HIR span gate clamps such internal spans to
    // the source end (approximate diagnostics) instead of mapping helper
    // coordinates onto user bytes, which breaks UTF-8 boundaries for
    // multibyte user sources.
    for item in &mut helper_items {
        if let Item::Fn(helper) = item {
            super::cst::shift_fn_item(helper, user_source_len);
        }
    }
    let mut new_items: Vec<Item> = Vec::with_capacity(program.items.len() + helper_items.len());
    new_items.append(&mut helper_items);
    new_items.extend(std::mem::take(&mut program.items));
    program.items = new_items;

    // 2) hoisted imports prepended to the SCRIPT body (the parser always
    //    appends the script FnItem last).
    // the hoisted imports run at entry: the script item when present,
    // otherwise the `main` fn's body (the py lane entry).
    let script_idx = program
        .items
        .iter()
        .rposition(|i| matches!(i, Item::Fn(f) if f.script))
        .or_else(|| {
            program
                .items
                .iter()
                .rposition(|i| matches!(i, Item::Fn(f) if f.name.name == "main" && !f.script))
        })
        .expect("the script item exists");
    let Item::Fn(script) = &mut program.items[script_idx] else {
        unreachable!("indexed a fn item");
    };
    let mut hoisted: Vec<Stmt> = Vec::with_capacity(imports.len());
    for (module, member, span) in imports {
        // PLAN-1107 T-02: the shared py-op fact — the import is recorded HERE,
        // where the front door admits it (module string + member + span), so
        // no target has to recover it from the generated frame text.
        if crate::py_ops::recording_active() {
            crate::py_ops::record(
                PyOpFact::new(PyOpKind::Import, span)
                    .with_module(module.clone())
                    .with_member(member.clone()),
            );
        }
        let frame = import_frame(&module, &member);
        hoisted.push(Stmt::Let {
            kw: span,
            name: Ident {
                name: member.clone(),
                span,
            },
            ann: None,
            mutable: true,
            value: Expr {
                kind: ExprKind::Call {
                    callee: Ident {
                        name: "py_import".to_string(),
                        span,
                    },
                    args: vec![Arg::Pos(Expr {
                        kind: ExprKind::StrLit(frame),
                        span,
                    })],
                },
                span,
            },
            span,
        });
    }
    script.body.stmts.splice(0..0, hoisted);
    count
}
