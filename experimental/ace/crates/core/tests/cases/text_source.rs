//! PLAN-770 T-04 text source frontend tests: the check-source legs of
//! V-02 (Unicode surface), V-03 (char conversions), V-08 (static
//! legality/source spans/shadowing) and V-09 (deep chains). In-process
//! `compile_text_source` (parse → resolve → typecheck → atom → text
//! descriptor bind → verify_text incl. the resource walk). Native PE
//! execution of the same fixtures lives in text_native.rs (T-05).

use ace_core::atom_text::Diagnostic;
use ace_core::source::compile_text_source;

fn check(src: &str) -> Result<ace_core::source::CompiledSource, Vec<Diagnostic>> {
    compile_text_source(src, "t.at", &[])
}

fn ok(src: &str) {
    check(src).unwrap_or_else(|ds| {
        let rendered: Vec<String> = ds.iter().map(|d| d.render("t.at", src)).collect();
        panic!("expected OK, got:\n{}", rendered.join("\n"))
    });
}

fn err_has(src: &str, code: &str) {
    let ds = check(src).expect_err("expected rejection");
    assert!(
        ds.iter().any(|d| d.code == code),
        "expected code `{}`, got: {:?}",
        code,
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn v02_unicode_surface_compiles() {
    ok(r#"
fn lens(s str) int { return s.len() }
fn bytes(s Str) int { return s.len_bytes() }
fn slicer(s str) str { return s.slice(1, 3) }
fn main() int {
    let n int = lens("A中😀")
    let o Str = "e\u{0301}"
    let m int = bytes(o)
    let v str = slicer("A中😀")
    return n + m + v.len()
}
"#);
}

#[test]
fn v02_cross_line_literal_keeps_line_break() {
    // CRLF source: the break enters the value verbatim (774 r2 R2-02)
    ok("fn f() int {\n    let s str = \"a\r\nb\"\n    return s.len()\n}\nfn main() int { return f() }");
}

#[test]
fn v03_char_surface_compiles() {
    ok(r#"
fn cp(c char) int { return c.to_codepoint() }
fn mk(n int) char { return char.from_codepoint(n) }
fn eqc(a char, b char) bool { return a == b }
fn main() int {
    let a char = 'A'
    let b char = char.from_codepoint(20013)
    if eqc(a, b) { return 0 }
    return cp(a) + cp(b)
}
"#);
}

#[test]
fn v03_invalid_literals_reject_at_lex() {
    err_has(
        "fn f() int { let s str = \"a\\qb\" return s.len() }\nfn main() int { return f() }",
        "text.invalid-escape",
    );
    err_has(
        "fn f() int { let s str = \"\\u{D800}\" return s.len() }\nfn main() int { return f() }",
        "text.invalid-scalar",
    );
    err_has(
        "fn f() int { let c char = 'ab' return c.to_codepoint() }\nfn main() int { return f() }",
        "text.char-multi",
    );
    err_has(
        "fn f() int { let c char = '' return c.to_codepoint() }\nfn main() int { return f() }",
        "text.char-empty",
    );
}

#[test]
fn v03_no_implicit_char_int_mixing() {
    err_has(
        "fn f(c char) int {\n    return c + 1\n}\nfn main() int { return f('a') }",
        "source.type-mismatch",
    );
    err_has(
        "fn f(n int) int {\n    let c char = n\n    return c.to_codepoint()\n}\nfn main() int { return f(1) }",
        "source.type-mismatch",
    );
}

#[test]
fn v08_text_methods_and_promotions() {
    ok(r#"
fn main() int {
    let o Str = "abc"
    let cat Str = o + "def"
    let n int = cat.len()
    let v str = o.to(str)
    let w Str = v.to(Str)
    let c Str = o.clone()
    return n + v.len() + w.len_bytes() + c.len()
}
"#);
    // mixed concat shapes reject
    err_has(
        "fn main() int {\n    let x Str = \"a\" + 1\n    return x.len()\n}",
        "source.type-mismatch",
    );
}

#[test]
fn v08_move_and_mode_params() {
    ok(r#"
fn consume(move o Str) int { return o.len_bytes() }
fn peek(mut m str) int { return m.len() }
fn main() int {
    let x Str = "xyz"
    let a int = consume(x.move)
    let b int = peek("q")
    return a + b
}
"#);
    // missing .move on a move param rejects (verify.ownership-required)
    err_has(
        "fn consume(move o Str) int { return o.len_bytes() }\nfn main() int { return consume(\"q\") }",
        "verify.ownership-required",
    );
    // use after move rejects
    err_has(
        "fn consume(move o Str) int { return o.len_bytes() }\nfn main() int {\n    let x Str = \"xyz\"\n    let a int = consume(x.move)\n    return a + x.len()\n}",
        "verify.use-after-move",
    );
}

#[test]
fn v08_shadowing_and_static_legs() {
    // a user function named like a builtin method keeps its name; methods
    // bind on receiver type and never hijack the function namespace
    ok(r#"
fn len(n int) int { return n }
fn main() int {
    let s str = "abcd"
    return len(s.len())
}
"#);
    // unknown method on an i32 receiver rejects
    err_has(
        "fn main() int {\n    let n int = 3\n    return n.len()\n}",
        "source.undefined",
    );
}

#[test]
fn v09_deep_chains_parse_and_check() {
    // 4096-deep method chain + 4096-deep concat chain compile through the
    // iterative spine (V-09 scale leg; native execution lands in T-05)
    let chain = "s".to_string() + &".clone()".repeat(4096);
    let src =
        format!("fn f(s str) int {{ return {chain}.len() }}\nfn main() int {{ return f(\"x\") }}");
    ok(&src);

    let concat = "\"a\"".to_string();
    let src = format!(
        "fn main() int {{
    let s Str = {concat}{}
    return s.len()
}}",
        " + \"b\"".repeat(4096)
    );
    ok(&src);
}

#[test]
fn old_profiles_still_reject_text_surface() {
    // str type word under the scalar profile is an unsupported surface
    let ds = ace_core::source::compile_scalar_source(
        "fn f(s str) int { return 1 }\nfn main() int { return f(\"x\") }",
        "t.at",
        &[],
    )
    .expect_err("scalar must reject text");
    assert!(
        ds.iter().any(|d| d.code == "source.unsupported"),
        "{:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}
