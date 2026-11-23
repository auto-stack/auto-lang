//! PLAN-1129: the shared front door's float operands — what the operation
//! facts say about them, and which wire version each frame speaks. The
//! facts are the one input every target consumes (AV builds frames from
//! them, A2PY lowers from them), so they are checked here without a
//! provider: the neutral declaration contract admits the program.
#![cfg(windows)]

use ace_core::py_ops::{PyLiteral, PyOpKind, PyValClass};
use ace_core::source::compile_language_source_with_foreign;
use ace_core::verify::CheckedModule;
use ace_foreign_contract::manual::{AdmittedForeign, SOURCE_CAP};
use ace_foreign_contract::resource::RESOURCE_CAP;

fn contract() -> AdmittedForeign {
    ace_foreign_contract::py_ops::language_contract().expect("the neutral contract admits")
}

type Compile = fn(
    &str,
    &str,
    &[String],
    &AdmittedForeign,
)
    -> Result<ace_core::source::CompiledSource, Vec<ace_core::atom_text::Diagnostic>>;

fn compile_with(src: &str, compile: Compile) -> CheckedModule {
    let caps = [SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
    compile(src, "main.as", &caps, &contract())
        .unwrap_or_else(|d| panic!("compile: {d:?}"))
        .checked
}

fn compile(src: &str) -> CheckedModule {
    compile_with(src, compile_language_source_with_foreign)
}

/// Every text literal of the checked program, in module order.
fn texts(checked: &CheckedModule) -> Vec<String> {
    let mut out = Vec::new();
    for module in &checked.bundle().modules {
        for body in &module.bodies {
            for expr in &body.exprs {
                if let ace_core::hir::ExprKind::StrLit { value } = &expr.kind {
                    out.push(String::from_utf8_lossy(value.as_bytes()).into_owned());
                }
            }
        }
    }
    out
}

/// How many frame fragments open with each wire version.
fn headers(checked: &CheckedModule) -> (usize, usize) {
    let all = texts(checked);
    let v1 = all
        .iter()
        .filter(|t| t.starts_with("p967.frame/1\n"))
        .count();
    let v2 = all
        .iter()
        .filter(|t| t.starts_with("p967.frame/2\n"))
        .count();
    (v1, v2)
}

const FACTS: &str = "use.py struct: pack
fn main() int {
    var rate = 0.1
    rate = 0.2
    var lr = 0.1d * 2.0d
    print(pack(\">f\", rate).hex().to(str))
    print(pack(\">d\", lr).hex().to(str))
    print(pack(\">d\", 0.25d).hex().to(str))
    print(pack(\">f\", 0.1).hex().to(str))
    return 0
}
";

#[test]
fn every_float_operand_is_a_float_and_only_a_literal_carries_a_literal() {
    let checked = compile(FACTS);
    let plan = checked.py_ops().expect("the py lane pins an op plan");
    let mut slots: Vec<(String, PyValClass, Option<PyLiteral>)> = Vec::new();
    for fact in plan.of_kind(PyOpKind::Call) {
        // the four direct `pack(<format>, <value>)` calls
        if fact.member.is_empty() && fact.positionals().len() == 2 {
            let value = fact.positionals()[1];
            let text = &FACTS[value.span.0 as usize..value.span.1 as usize];
            slots.push((text.to_string(), value.class, value.literal.clone()));
        }
    }
    assert_eq!(
        slots,
        vec![
            // a local the program reassigned: the CURRENT value, never the
            // initializer's literal
            ("rate".to_string(), PyValClass::Float, None),
            // a runtime computation
            ("lr".to_string(), PyValClass::Float, None),
            // literals keep their exact value — an f32 as its one widening
            (
                "0.25d".to_string(),
                PyValClass::Float,
                Some(PyLiteral::Float(0.25))
            ),
            (
                "0.1".to_string(),
                PyValClass::Float,
                Some(PyLiteral::Float(0.1f32 as f64))
            ),
        ],
        "float operands: class, and a literal only for a literal"
    );
    assert_ne!(0.1f32 as f64, 0.1f64, "the f32 literal is not the double");
}

#[test]
fn value_carrying_frames_speak_version_two_and_valueless_frames_stay_on_one() {
    // a call whose only argument is an int is still a value-carrying frame
    let (v1, v2) = headers(&compile(
        "use.py builtins: repr\nfn main() int {\n    print(repr(7).to(str))\n    return 0\n}\n",
    ));
    assert_eq!(
        (v1, v2),
        (2, 1),
        "import + receiver frames on /1, the call on /2"
    );

    // no operation that receives a user value: nothing moves to /2
    let (v1, v2) = headers(&compile(
        "use.py math: pi\nfn main() int {\n    print(pi.to(str))\n    return 0\n}\n",
    ));
    assert_eq!(v2, 0, "a receiver-only program emits no version-2 frame");
    assert!(v1 >= 2);

    // calls, a binary operand and an index key — every value-carrying shape
    let mixed = compile(
        "use.py builtins: dict, repr
fn main() int {
    var d = dict()
    var s = repr(\"a\")
    var t = s + \"x\"
    var k = d[0.5]
    var p = d.get(0.5, \"x\")
    print(t.to(str))
    print(p.to(str))
    return 0
}
",
    );
    let (v1, v2) = headers(&mixed);
    assert_eq!(v2, 5, "dict(), repr(\"a\"), `+`, `[0.5]` and `get(...)`");
    assert!(v1 >= 3, "two imports and the receiver frames: {v1}");
    // the float literals are static typed text — the 21/21/22-bit words of
    // 0.1f32… of 0.5f32 — and the retired `F:` disguise is nowhere
    let all = texts(&mixed).concat();
    assert!(
        all.contains(&format!(
            "\n{}",
            ace_core::source::py::typed_float_text(b'f', u64::from(0.5f32.to_bits()))
        )),
        "a float literal is static typed text: {all:?}"
    );
    assert!(!all.contains("F:"), "no `F:` float-literal text is emitted");
}

#[test]
fn the_typed_payload_is_three_bounded_words_with_one_spelling_per_value() {
    use ace_core::source::py::{float_words, typed_float_text};
    assert_eq!(float_words(0), "0 0 0");
    assert_eq!(float_words(u64::MAX), "2097151 2097151 4194303");
    assert_eq!(float_words(1.5f64.to_bits()), "524032 0 0");
    // a value can be read back from its words by the wire rule
    for bits in [
        0u64,
        1,
        (1 << 43) - 1,
        1 << 43,
        u64::MAX,
        2.25f64.to_bits(),
        (-0.0f64).to_bits(),
    ] {
        let words: Vec<u64> = float_words(bits)
            .split(' ')
            .map(|w| w.parse().expect("decimal"))
            .collect();
        assert_eq!(words.len(), 3);
        assert!(words[0] < 1 << 21 && words[1] < 1 << 21 && words[2] < 1 << 22);
        assert_eq!((words[0] << 43) | (words[1] << 22) | words[2], bits);
    }
    assert_eq!(typed_float_text(b'd', 0), "d 5\n0 0 0\n");
    assert_eq!(
        typed_float_text(b'f', u64::from(0.5f32.to_bits())),
        "f 7\n0 252 0\n"
    );
}

#[test]
fn unsupported_value_kinds_stay_a_named_rejection_at_their_own_span() {
    let src =
        "use.py builtins: repr\nfn main() int {\n    print(repr(1i64).to(str))\n    return 0\n}\n";
    let caps = [SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
    let diags = compile_language_source_with_foreign(src, "main.as", &caps, &contract())
        .expect_err("a 64-bit integer does not cross");
    let diag = diags
        .iter()
        .find(|d| d.code == "source.py-domain")
        .unwrap_or_else(|| panic!("missing the py-domain rejection: {diags:?}"));
    assert_eq!(diag.stage, ace_core::atom_text::Stage::Bind);
    assert_eq!(&src[diag.span.start..diag.span.end], "1i64");

    // the float forms are admitted: f32, f64, a parameter, a call result
    compile(
        "use.py builtins: repr
fn scale(v float) float { return v }
fn main() int {
    print(repr(0.5).to(str))
    print(repr(0.5d).to(str))
    print(repr(scale(0.25)).to(str))
    return 0
}
",
    );
}

#[test]
fn the_float_helpers_belong_to_the_py_lane_only() {
    let plain = ace_core::source::compile_language_source(
        "fn main() int {\n    var x = 0.5\n    print(x)\n    return 0\n}\n",
        "main.as",
        &[],
    )
    .expect("a program without use.py compiles")
    .checked;
    let names: Vec<&str> = plain
        .bundle()
        .modules
        .iter()
        .flat_map(|m| m.defs.iter().map(|d| d.id_text.as_str()))
        .collect();
    assert!(
        names.iter().all(|n| !n.contains("__py_")),
        "no Python helper enters a program without use.py: {names:?}"
    );
    assert!(plain.py_ops().is_none());

    let py = compile(FACTS);
    let names: Vec<&str> = py
        .bundle()
        .modules
        .iter()
        .flat_map(|m| m.defs.iter().map(|d| d.id_text.as_str()))
        .collect();
    for helper in [
        "__py_chunk_d",
        "__py_chunk_f",
        "__py_chunk_words",
        "__py_chunk_f64",
        "__py_chunk_f32",
    ] {
        assert!(
            names.iter().any(|n| n.ends_with(helper)),
            "the frozen float helper `{helper}` is injected: {names:?}"
        );
    }
}

/// The sign-bit primitive belongs to the frozen helper alone. A user call of
/// the name is not a value the language defines: the shared front door
/// refuses it like any unknown callee, so no target ever sees it. (Before
/// this admission the AV lane ran `__py_sign_bit(-1.0d)` and printed `1`
/// while the A2PY worker failed on the same program.)
#[test]
fn the_sign_bit_primitive_is_not_user_callable() {
    for (call, what) in [
        ("__py_sign_bit(-1.0d)", "one double"),
        ("__py_sign_bit(-1.0)", "one float"),
        ("__py_sign_bit(0.0d, 0.0d)", "two doubles"),
        ("__py_sign_bit(1)", "an int"),
        ("__py_sign_bit()", "no argument"),
    ] {
        let src = format!(
            "use.py builtins: repr\nfn probe(v double) int {{\n    return {call}\n}}\nfn main() int {{\n    var s = probe(0.0d)\n    print(s)\n    print(repr(0.5).to(str))\n    return 0\n}}\n"
        );
        let caps = [SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
        let diags = compile_language_source_with_foreign(&src, "main.as", &caps, &contract())
            .err()
            .unwrap_or_else(|| panic!("{what}: a user call of `{call}` must not resolve"));
        let at_call = diags.iter().any(|d| {
            d.span.end > d.span.start
                && src[d.span.start as usize..d.span.end as usize].contains("__py_sign_bit")
        });
        assert!(
            at_call,
            "{what}: the refusal names the user call, got {diags:?}"
        );
    }
    // the ordinary float forms still cross: the helper itself keeps the name
    compile("use.py builtins: repr\nfn main() int {\n    print(repr(0.5 + 0.25).to(str))\n    return 0\n}\n");
}

#[test]
fn the_aggregates_entry_admits_runtime_float_operands() {
    // The 1026 product entry and the 978 engineering lane compile `use.py`
    // sources under Aggregates: the frozen helpers and a runtime float
    // operand must resolve there too, with the same facts and the same wire.
    // (The argument shape stays inside what the Aggregates grammar admits.)
    let src = "use.py struct: pack
use.py math: ldexp

fn main() int {
    var fmt = \">f\"
    var rate = 0.1
    rate = 0.2
    var b = pack(fmt, rate)
    var c = ldexp(0.25d, 3)
    print(b)
    print(c)
    return 0
}
";
    let checked = compile_with(
        src,
        ace_core::source::compile_aggregates_source_with_foreign,
    );
    let plan = checked.py_ops().expect("the py lane pins an op plan");
    let pack = plan
        .of_kind(PyOpKind::Call)
        .find(|f| f.member.is_empty() && f.positionals().len() == 2)
        .expect("a direct two-positional call");
    let ldexp = plan
        .of_kind(PyOpKind::Call)
        .find(|f| f.member.is_empty() && f.positionals().len() == 2 && f.span != pack.span)
        .expect("the other direct two-positional call");
    // source order: `pack(fmt, rate)` precedes `ldexp(0.25d, 3)`
    assert!(pack.span.0 < ldexp.span.0);
    let value = pack.positionals()[1];
    assert_eq!(&src[value.span.0 as usize..value.span.1 as usize], "rate");
    assert_eq!(value.class, PyValClass::Float);
    assert_eq!(value.literal, None, "a reassigned local is never a literal");
    assert_eq!(
        ldexp.positionals()[0].literal,
        Some(PyLiteral::Float(0.25)),
        "a literal keeps its value under Aggregates"
    );
    let (v1, v2) = headers(&checked);
    assert_eq!(v2, 2, "the two value-carrying calls");
    assert!(v1 >= 2, "the import frames stay on /1: {v1}");
}
