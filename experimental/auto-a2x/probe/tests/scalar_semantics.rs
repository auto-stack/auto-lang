//! T-02/T-03 PRIVATE true-chain semantics probe (P surface only — no H
//! wiring): the FULL frozen 745/767 positive corpus plus the ACE-M1
//! B-cases and boundary synthetics, each through
//! source → checked frontend → semantic view → Rust provider → real
//! rustc → real run → parent-side oracle.
//!
//! Oracle provenance: the frozen corpus fixtures (byte-identical files
//! from `experimental/ac-core/fixtures/source|scalar-source/valid`) carry
//! the 745/767 scripts' exit-oracle semantics; here the VALUE itself is
//! asserted (exit codes only cover 0..255). Trap names follow the frozen
//! §5.1 taxonomy; trace-mark order is source evaluation order; mark
//! return values are the frozen native contract (mark_a→1, mark_b→2).
//!
//! This is the certification-shaped leg (explicit compile+run) — toolchain
//! spawns are expected here; the daily spawn-zero channel is T-05.

use auto_a2x_core::SemanticView;
use auto_a2x_rust::{assemble_file_set, lower_program};
use auto_ac_prototype::source::{compile_scalar_source, compile_source, TRACE_CAPABILITY};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
enum Expect {
    Value(i32),
    Trap(&'static str),
}

struct Case {
    name: String,
    profile: &'static str,
    trace: bool,
    src: String,
    expect: Expect,
    marks: Vec<&'static str>,
    release_too: bool,
}

fn fixture(rel: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ac-core/fixtures");
    let path = base.join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn case(name: &str, profile: &'static str, src: String, expect: Expect) -> Case {
    Case {
        name: name.to_string(),
        profile,
        trace: false,
        src,
        expect,
        marks: vec![],
        release_too: false,
    }
}

fn trace_case(
    name: &str,
    profile: &'static str,
    src: String,
    expect: Expect,
    marks: Vec<&'static str>,
) -> Case {
    Case {
        name: name.to_string(),
        profile,
        trace: true,
        src,
        expect,
        marks,
        release_too: false,
    }
}

fn v(x: i32) -> Expect {
    Expect::Value(x)
}

fn cases() -> Vec<Case> {
    let mut cs = Vec::new();
    // ---- frozen 745 core corpus (valid/) ----
    let core = |f: &str| fixture(&format!("source/valid/{}", f));
    cs.push(case(
        "core-c02-paren-newline",
        "core",
        core("c02-paren-newline.at"),
        v(3),
    ));
    cs.push(case(
        "core-c05-ac-start-name",
        "core",
        core("c05-ac-start-name.at"),
        v(3),
    ));
    cs.push(case(
        "core-c06-exitprocess-name",
        "core",
        core("c06-exitprocess-name.at"),
        v(4),
    ));
    cs.push(case(
        "core-d01-call-multiline",
        "core",
        core("d01-call-multiline-then-add.at"),
        v(5),
    ));
    cs.push(case(
        "core-d02-paren-multiline",
        "core",
        core("d02-paren-multiline-then-add.at"),
        v(6),
    ));
    cs.push(case(
        "core-d03-negative-multiline",
        "core",
        core("d03-negative-multiline-arg.at"),
        v(1),
    ));
    cs.push(case(
        "core-d04-nested-paren",
        "core",
        core("d04-nested-paren-multiline.at"),
        v(6),
    ));
    cs.push(case(
        "core-d05-call-no-tail",
        "core",
        core("d05-call-multiline-no-tail.at"),
        v(2),
    ));
    cs.push(case(
        "core-d07-colliding-prefix",
        "core",
        core("d07-colliding-prefix-names.at"),
        v(7),
    ));
    cs.push(case(
        "core-e01-nesting-16",
        "core",
        core("e01-paren-nesting-16.at"),
        v(1),
    ));
    cs.push(case(
        "core-f04-parens-99",
        "core",
        core("f04-parens-99.at"),
        v(1),
    ));
    cs.push(case(
        "core-f11-add-chain-100",
        "core",
        core("f11-deep-add-chain.at"),
        v(300),
    ));
    cs.push(case(
        "core-g02-parens-200",
        "core",
        core("g02-parens-200.at"),
        v(1),
    ));
    cs.push(case(
        "core-m01-comma-args",
        "core",
        core("m01-comma-binop-args.at"),
        v(42),
    ));
    cs.push(case(
        "core-m02-named-binop",
        "core",
        core("m02-named-binop-args.at"),
        v(42),
    ));
    cs.push(case(
        "core-m06-nested-call",
        "core",
        core("m06-nested-call-binop-args.at"),
        v(192),
    ));
    cs.push(case(
        "core-m07-bool-arg",
        "core",
        core("m07-bool-binop-arg.at"),
        v(5),
    ));
    cs.push(case(
        "core-m10-paren-args",
        "core",
        core("m10-parenthesized-binop-args.at"),
        v(42),
    ));
    cs.push(case(
        "core-m22-named-newline",
        "core",
        core("m22-named-args-newline.at"),
        v(33),
    ));
    cs.push(case(
        "core-p01-precedence",
        "core",
        core("p01-precedence.at"),
        v(14),
    ));
    cs.push(case(
        "core-p02-signed-pos",
        "core",
        core("p02-signed-positional.at"),
        v(1),
    ));
    cs.push(case(
        "core-p03-bool-param",
        "core",
        core("p03-bool-param-constant-encoding.at"),
        v(7),
    ));
    cs.push(case(
        "core-p04-while",
        "core",
        core("p04-while-retest.at"),
        v(3),
    ));
    cs.push(case(
        "core-p05-while-zero",
        "core",
        core("p05-while-zero.at"),
        v(1),
    ));
    cs.push(case(
        "core-p06-continue",
        "core",
        core("p06-continue-retests-condition.at"),
        v(3),
    ));
    cs.push(case(
        "core-p07-break-nearest",
        "core",
        core("p07-break-nearest-loop.at"),
        v(5),
    ));
    cs.push(case(
        "core-p08-recursion",
        "core",
        core("p08-recursion.at"),
        v(3),
    ));
    cs.push(case(
        "core-p09-forward-sig",
        "core",
        core("p09-forward-signature.at"),
        v(8),
    ));
    cs.push(case(
        "core-p10-shadow",
        "core",
        core("p10-shadow-rhs-outer.at"),
        v(5),
    ));
    cs.push(case(
        "core-p11-min-max",
        "core",
        core("p11-literal-min-max.at"),
        v(0),
    ));
    cs.push(case(
        "core-p12-unexecuted",
        "core",
        core("p12-unexecuted-trap-not-lifted-x1.at"),
        v(0),
    ));
    {
        let mut c = case(
            "core-p13-add-overflow",
            "core",
            core("p13-add-overflow-trap.at"),
            Expect::Trap("Overflow"),
        );
        c.release_too = true;
        cs.push(c);
    }
    {
        let mut c = case(
            "core-p14-mul-overflow",
            "core",
            core("p14-mul-overflow-trap.at"),
            Expect::Trap("Overflow"),
        );
        c.release_too = true;
        cs.push(c);
    }
    cs.push(case(
        "core-p17-hetero-named",
        "core",
        core("p17-heterogeneous-binding-map.at"),
        v(6),
    ));
    cs.push(case(
        "core-p18-bool-return",
        "core",
        core("p18-bool-return-local-branch.at"),
        v(4),
    ));
    cs.push(case(
        "core-p19-user-mark-a",
        "core",
        core("p19-user-function-not-intrinsic.at"),
        v(6),
    ));
    cs.push(case(
        "core-p20-utf8-crlf-min",
        "core",
        core("p20-utf8-crlf-negative-min.at"),
        v(0),
    ));

    // ---- frozen 767 scalar corpus (scalar-source/valid/) ----
    let scalar = |f: &str| fixture(&format!("scalar-source/valid/{}", f));
    cs.push(case(
        "sc-a-minus-1",
        "scalar",
        scalar("p-a-minus-1.at"),
        v(0),
    ));
    cs.push(case(
        "sc-binop-args",
        "scalar",
        scalar("p-binop-args.at"),
        v(0),
    ));
    cs.push(case(
        "sc-cmp-matrix",
        "scalar",
        scalar("p-cmp-matrix.at"),
        v(0),
    ));
    cs.push(case(
        "sc-count-digits",
        "scalar",
        scalar("p-count-digits.at"),
        v(0),
    ));
    cs.push(case("sc-crlf", "scalar", scalar("p-crlf.at"), v(0)));
    cs.push(case(
        "sc-for-loops",
        "scalar",
        scalar("p-for-loops.at"),
        v(0),
    ));
    cs.push(case(
        "sc-gcd-euclid",
        "scalar",
        scalar("p-gcd-euclid.at"),
        v(0),
    ));
    cs.push(case("sc-is-digit", "scalar", scalar("p-is-digit.at"), v(0)));
    // 'abababaaaabab' (frozen 767 oracle) as mark lines in execution order
    cs.push(trace_case(
        "sc-short-circuit-contexts",
        "scalar",
        scalar("p-short-circuit-contexts.at"),
        v(0),
        vec![
            "mark_a", "mark_b", "mark_a", "mark_b", "mark_a", "mark_b", "mark_a", "mark_a",
            "mark_a", "mark_a", "mark_b", "mark_a", "mark_b",
        ],
    ));
    // 'aaaba' (frozen 767 oracle)
    cs.push(trace_case(
        "sc-short-circuit-trace",
        "scalar",
        scalar("p-short-circuit-trace.at"),
        v(0),
        vec!["mark_a", "mark_a", "mark_a", "mark_b", "mark_a"],
    ));
    {
        let mut c = case(
            "sc-short-circuit-trap",
            "scalar",
            scalar("p-short-circuit-trap.at"),
            Expect::Trap("DivByZero"),
        );
        c.release_too = true;
        cs.push(c);
    }

    // ---- ACE-M1 B-cases (frozen binding.tsv values) ----
    let b = |f: &str| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../crates/auto-lang/test/cases/ace-m1/src")
            .join(f);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
    };
    {
        let mut c = case("ace-B01", "core", b("B01.at"), v(14));
        c.release_too = true;
        cs.push(c);
    }
    {
        let mut c = case("ace-B02-signed-div-rem", "scalar", b("B02.at"), v(-21));
        c.release_too = true;
        cs.push(c);
    }
    {
        let mut c = case("ace-B04-shortcircuit", "scalar", b("B04.at"), v(42));
        c.release_too = true;
        cs.push(c);
    }
    cs.push(case("ace-B09-recursion", "scalar", b("B09.at"), v(120)));
    {
        let mut c = case(
            "ace-B13-overflow",
            "core",
            b("B13.at"),
            Expect::Trap("Overflow"),
        );
        c.release_too = true;
        cs.push(c);
    }
    {
        let mut c = case(
            "ace-B14-div-zero",
            "scalar",
            b("B14.at"),
            Expect::Trap("DivByZero"),
        );
        c.release_too = true;
        cs.push(c);
    }

    // ---- boundary synthetics (frozen §5.1 taxonomy; hand oracles) ----
    cs.push(case(
        "bd-min-literal",
        "scalar",
        "fn main() int { return -2147483648 }".into(),
        v(-2147483648),
    ));
    cs.push(case(
        "bd-min-div-minus-1",
        "scalar",
        "fn main() int { return -2147483648 / -1 }".into(),
        Expect::Trap("DivOverflow"),
    ));
    cs.push(case(
        "bd-min-rem-minus-1",
        "scalar",
        "fn main() int { return -2147483648 % -1 }".into(),
        Expect::Trap("RemOverflow"),
    ));
    cs.push(case(
        "bd-rem-by-zero",
        "scalar",
        "fn main() int { return 10 % 0 }".into(),
        Expect::Trap("RemByZero"),
    ));
    cs.push(case(
        "bd-value70",
        "core",
        "fn main() int { return 70 }".into(),
        v(70),
    ));
    cs.push(trace_case("bd-named-source-order", "scalar",
        "fn pair(a int, b int) int { return a * 10 + b }\nfn main() int { return pair(b: mark_b(), a: mark_a()) }".into(),
        v(12), vec!["mark_b", "mark_a"]));
    cs.push(trace_case(
        "bd-or-skips-rhs",
        "scalar",
        "fn main() int { if true || mark_b() > 99 { return 8 }; return 9 }".into(),
        v(8),
        vec![],
    ));
    cs
}

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/a2x-out")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

fn run_case(case: &Case, flavor: &str, extra_flags: &[&str]) {
    let caps = if case.trace {
        vec![TRACE_CAPABILITY.to_string()]
    } else {
        vec![]
    };
    let compiled = if case.profile == "core" {
        compile_source(&case.src, "probe", &caps).expect("frontend compile")
    } else {
        compile_scalar_source(&case.src, "probe", &caps).expect("frontend compile")
    };

    let view = SemanticView::from_checked(&compiled.checked).expect("view admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = out_root().join(&case.name).join(flavor);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();

    let exe = dir.join("a2x_probe.exe");
    let compile_out = Command::new(rustc())
        .arg("--edition")
        .arg("2021")
        .args(extra_flags)
        .arg("-o")
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir)
        .output()
        .expect("spawn rustc");
    assert!(
        compile_out.status.success(),
        "rustc failed for {} ({}):\n{}",
        case.name,
        flavor,
        String::from_utf8_lossy(&compile_out.stderr)
    );

    let run = Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run exe");
    assert!(
        run.status.success(),
        "wrapper exit non-zero for {} ({}): {}",
        case.name,
        flavor,
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run.stderr).to_string();
    let expected_line = match &case.expect {
        Expect::Value(x) => format!("value:{}\n", x),
        Expect::Trap(t) => format!("trap:{}\n", t),
    };
    assert_eq!(
        stdout, expected_line,
        "stdout mismatch for {} ({}) — marks observed: {:?}",
        case.name, flavor, stderr
    );
    let marks: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        marks, case.marks,
        "trace marks mismatch for {} ({})",
        case.name, flavor
    );
}

#[test]
fn full_corpus_true_chain() {
    let cases = cases();
    assert!(cases.len() >= 55, "corpus table shrank: {}", cases.len());
    for case in &cases {
        run_case(case, "debug", &[]);
        if case.release_too {
            run_case(case, "release", &["-O", "-C", "debug-assertions=off"]);
        }
    }
}

/// Admission negatives: wider HIR never passes the view into a target — a
/// text-domain module is a NAMED domain rejection.
#[test]
fn view_admission_rejects_wider_domains() {
    use auto_ac_prototype::source::compile_text_source;
    let compiled = compile_text_source(
        "fn pick(a str, b str) str { if a == b { return a }; return b }",
        "neg",
        &[],
    )
    .expect("text compile");
    let err = SemanticView::from_checked(&compiled.checked)
        .expect_err("text domain must be rejected by the view");
    assert_eq!(err.code, "a2x.domain", "got: {}", err);
}

/// Entry-shape rejections (§5.1): nonzero-param or non-i32 entries are
/// named rejections, never silent picks.
#[test]
fn entry_shape_rejections() {
    let src = "fn helper(a int) int { return a }\nfn main() int { return helper(3) }";
    let compiled = compile_scalar_source(src, "neg", &[]).unwrap();
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    let err = view
        .entry("helper")
        .expect_err("nonzero-param entry must reject");
    assert_eq!(err.code, "a2x.entry");
    let err = view
        .entry("no_such")
        .expect_err("unknown entry must reject");
    assert_eq!(err.code, "a2x.entry");
}

/// Deeply-nested synthetic beyond the frozen walk budget is a NAMED
/// resource rejection (a2x.too-deep), not a host stack overflow (V-11
/// shape, view/lower level).
#[test]
fn depth_budget_named_rejection() {
    let mut body = String::from("fn deep(x int) int { return ");
    body.push_str(&"(x + ".repeat(500));
    body.push('1');
    body.push_str(&")".repeat(500));
    body.push_str(" }");
    body.push_str("fn main() int { return deep(1) }");
    let compiled = compile_scalar_source(&body, "neg", &[]).unwrap();
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    let err = lower_program(&view, "main").expect_err("beyond-budget nesting must reject");
    assert_eq!(err.code, "a2x.too-deep", "got: {}", err);
}
