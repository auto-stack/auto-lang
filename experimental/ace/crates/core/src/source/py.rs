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
//! * the frozen float helpers `__py_chunk_*` (PLAN-1129). A float operand
//!   of a Python operation is ONE call of `__py_chunk_d`/`__py_chunk_f`, so
//!   it is evaluated exactly once. These helpers read the float's exact bit
//!   pattern with ordinary Auto arithmetic and render it as three
//!   21/21/22-bit decimal words — only i32 is ever displayed, so they
//!   resolve under every profile the py lane serves. The single step plain
//!   source cannot write — the sign bit of a `-0.0` — is the AV-lane
//!   intrinsic [`PY_SIGN_BIT`], which only these helpers call;
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
fn __py_chunk_t(v Str) Str {
return f"t ${v.len_bytes()}\n$v\n" }
fn __py_chunk_d(v double) Str {
var p = __py_chunk_words(__py_chunk_f64(v))
return f"d ${p.len_bytes()}\n$p\n" }
fn __py_chunk_f(v float) Str {
var p = __py_chunk_words(__py_chunk_f32(v))
return f"f ${p.len_bytes()}\n$p\n" }
fn __py_chunk_words(b u64) Str {
var w0 = (b & 0x3FFFFFu64).as(i32)
var w1 = (b.shr(22u64) & 0x1FFFFFu64).as(i32)
var w2 = b.shr(43u64).as(i32)
return f"$w2 $w1 $w0" }
fn __py_chunk_f64(v double) u64 {
if v != v { return 0x7FF8000000000000u64 }
var a = v
if v < 0.0d { a = -v }
var mag = 0u64
if a != 0.0d {
if a * 2.0d == a { mag = 0x7FF0000000000000u64 } else {
var e = 0i64
for a >= 2.0d { a = a * 0.5d; e = e + 1i64 }
for a < 1.0d { a = a * 2.0d; e = e - 1i64 }
var m = ((a - 1.0d) * 4503599627370496.0d).as(u64)
var big = e + 1023i64
if big <= 0i64 { mag = (m + 0x10000000000000u64).shr((1i64 - big).as(u64)) } else { mag = (big.shl(52i64) | m.as(i64)).as(u64) } } }
return __py_sign_bit(v).as(u64).shl(63u64) | mag }
fn __py_chunk_f32(v float) u64 {
var b = __py_chunk_f64(v.as(double))
var exp = b.shr(52u64) & 0x7FFu64
var frac = b & 0xFFFFFFFFFFFFFu64
var sign = b.shr(63u64)
if exp == 0x7FFu64 { if frac == 0u64 { return sign.shl(31u64) | 0x7F800000u64 } else { return 0x7FC00000u64 } }
if exp == 0u64 { return sign.shl(31u64) }
if exp >= 897u64 { return sign.shl(31u64) | (exp - 896u64).shl(23u64) | frac.shr(29u64) }
return sign.shl(31u64) | (frac | 0x10000000000000u64).shr(926u64 - exp) }
"#;

/// The `p967.frame/1` wire header: frames that carry no user value (the
/// static import frame, receiver-only extraction frames).
pub const FRAME_V1_HEADER: &str = "p967.frame/1\n";
/// The `p967.frame/2` wire header (PLAN-1129): every frame the resolver
/// builds around user VALUES — a Python call's arguments, an index key, a
/// binary operand. It is the `/1` superset: floats cross as the typed tags
/// `d`/`f` (payload `<w2> <w1> <w0>`, the 64-bit pattern split 21 / 21 / 22
/// bits from the top) and a text payload is always text (no `F:` literal
/// decoding).
pub const FRAME_V2_HEADER: &str = "p967.frame/2\n";

/// The AV-lane primitive the frozen float helper calls for a float's sign
/// bit; the lowering emits it inline. The resolver admits the name only
/// inside the frozen helper [`PY_SIGN_BIT_OWNER`]: a user call of it is an
/// unknown callee, on every target alike (the 1026 reconstruction
/// intrinsics are not user-callable either).
pub use crate::py_ops::PY_SIGN_BIT;
/// The one function whose body may call [`PY_SIGN_BIT`] — the frozen helper
/// `__py_chunk_f64` above.
pub const PY_SIGN_BIT_OWNER: &str = "__py_chunk_f64";

/// `<w2> <w1> <w0>`: a 64-bit pattern as three unsigned decimals, 21 / 21 /
/// 22 bits from the top — the wire payload of a typed float. Every word fits
/// an `i32` and stays below its bound, so one value has one encoding (the
/// frozen helper `__py_chunk_words` renders the same three words at run
/// time).
pub fn float_words(bits: u64) -> String {
    format!("{} {} {}", bits >> 43, (bits >> 22) & 0x1F_FFFF, bits & 0x3F_FFFF)
}

/// The typed frame text of a float LITERAL whose bits are known at compile
/// time: `<tag> <len>\n<w2> <w1> <w0>\n` (`tag` is `d` for an f64, `f` for
/// an f32). Static text, exactly like every other literal in a frame — a
/// literal costs the running program nothing. A float the program computes
/// is never written this way; it crosses through the frozen helper instead.
pub fn typed_float_text(tag: u8, bits: u64) -> String {
    let payload = float_words(bits);
    format!("{} {}\n{}\n", tag as char, payload.len(), payload)
}

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

#[cfg(test)]
mod tests {
    use super::{HELPERS, PY_SIGN_BIT, PY_SIGN_BIT_OWNER};

    /// The admission the resolver grants — `PY_SIGN_BIT` inside
    /// `PY_SIGN_BIT_OWNER` — names the helper that actually calls it: the
    /// owner is one defined helper and holds the primitive's only call site.
    #[test]
    fn the_sign_bit_owner_is_the_one_helper_that_calls_it() {
        assert_eq!(
            HELPERS.matches(&format!("fn {PY_SIGN_BIT_OWNER}(")).count(),
            1,
            "the owner is one defined helper"
        );
        assert_eq!(
            HELPERS.matches(&format!("{PY_SIGN_BIT}(")).count(),
            1,
            "the primitive has exactly one call site"
        );
        let owner = HELPERS
            .split(&format!("fn {PY_SIGN_BIT_OWNER}("))
            .nth(1)
            .and_then(|rest| rest.split("\nfn ").next())
            .expect("the owner's body");
        assert!(
            owner.contains(&format!("{PY_SIGN_BIT}(")),
            "the call site is inside the owner's body"
        );
    }
}
