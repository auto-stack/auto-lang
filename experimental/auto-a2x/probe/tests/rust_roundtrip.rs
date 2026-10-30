//! PLAN-892 T-02 PRIVATE true-chain probe (P surface only — no H wiring):
//! real `.at` source → existing checked frontend (`compile_source` /
//! `compile_scalar_source`) → shared semantic view → Rust provider → full
//! file set on disk → REAL rustc compile → REAL run → parent-side oracle.
//!
//! This is the certification-shaped leg (explicit compile+run), not the
//! daily spawn-zero channel; toolchain spawning here is expected and
//! observed. Per-case budget follows the 852 lines (compile ≤120s, run
//! ≤30s) as a recorded measurement, not yet an enforced deadline — the
//! formal worker (T-05) owns deadline discipline.
//!
//! Oracle provenance: B-case expectations are the frozen ACE-M1 binding
//! table values; trap names are the frozen §5.1 taxonomy; mark order is
//! source evaluation order; mark return values are the frozen native
//! contract (mark_a→1, mark_b→2).

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
    name: &'static str,
    /// "core" (745 surface) or "scalar" (767 surface).
    profile: &'static str,
    trace: bool,
    src: &'static str,
    expect: Expect,
    /// Expected stderr marks, in execution order.
    marks: Vec<&'static str>,
    /// Also compile/run the release flavor (`-O -C debug-assertions=off`)
    /// and require an identical outcome (V-02 debug/release control).
    release_too: bool,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "B01-precedence",
            profile: "core",
            trace: false,
            src: "fn main() int { return 2 + 3 * 4 }",
            expect: Expect::Value(14),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "B02-signed-div-rem",
            profile: "scalar",
            trace: false,
            src: "fn main() int { return (-7 / 3) * 10 + (-7 % 3) }",
            expect: Expect::Value(-21), // -7/3 = -2 (toward zero), -7%3 = -1 (sign of dividend)
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "B04-shortcircuit-and",
            profile: "scalar",
            trace: false,
            src: "fn main() int {\n    if false && 1 / 0 > 0 { return 1 }\n    return 42\n}",
            expect: Expect::Value(42),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "B09-recursion",
            profile: "scalar",
            trace: false,
            src: "fn fact(n int) int {\n    if n <= 1 { return 1 }\n    return n * fact(n - 1)\n}\n\
                   fn main() int { return fact(5) }",
            expect: Expect::Value(120),
            marks: vec![],
            release_too: false,
        },
        Case {
            name: "B13-add-overflow",
            profile: "core",
            trace: false,
            src: "fn main() int { return 2147483647 + 1 }",
            expect: Expect::Trap("Overflow"),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "B14-div-by-zero",
            profile: "scalar",
            trace: false,
            src: "fn main() int { return 10 / 0 }",
            expect: Expect::Trap("DivByZero"),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "MIN-neg-overflow",
            profile: "scalar",
            trace: false,
            src: "fn main() int { return -2147483648 }", // MIN literal via 745 negative-literal form
            expect: Expect::Value(-2147483648),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "MIN-div-minus-one",
            profile: "scalar",
            trace: false,
            src: "fn main() int { return -2147483648 / -1 }",
            expect: Expect::Trap("DivOverflow"),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "MIN-rem-minus-one",
            profile: "scalar",
            trace: false,
            src: "fn main() int { return -2147483648 % -1 }",
            expect: Expect::Trap("RemOverflow"),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "rem-by-zero",
            profile: "scalar",
            trace: false,
            src: "fn main() int { return 10 % 0 }",
            expect: Expect::Trap("RemByZero"),
            marks: vec![],
            release_too: false,
        },
        Case {
            name: "value70-is-a-value",
            profile: "core",
            trace: false,
            src: "fn main() int { return 70 }",
            expect: Expect::Value(70),
            marks: vec![],
            release_too: false,
        },
        Case {
            name: "unexecuted-branch-trap-not-lifted",
            profile: "core",
            trace: false,
            src: "fn main() int { if false { return 2147483647 + 1 }; return 7 }",
            expect: Expect::Value(7),
            marks: vec![],
            release_too: true,
        },
        Case {
            name: "while-retest",
            profile: "core",
            trace: false,
            src: "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }",
            expect: Expect::Value(3),
            marks: vec![],
            release_too: false,
        },
        Case {
            name: "conditional-for",
            profile: "scalar",
            trace: false,
            src: "fn main() int { var i = 0; for i < 3 { i = i + 1 }; return i }",
            expect: Expect::Value(3),
            marks: vec![],
            release_too: false,
        },
        Case {
            name: "break-nearest-loop",
            profile: "core",
            trace: false,
            src: "fn main() int { var n = 0; loop { loop { n = n + 2; break }; n = n + 3; break }; return n }",
            expect: Expect::Value(5),
            marks: vec![],
            release_too: false,
        },
        Case {
            name: "continue-retests",
            profile: "core",
            trace: false,
            src: "fn main() int { var i = 0; var s = 0; while i < 5 { i = i + 1; if i < 4 { s = s + i } else { continue } }; return s }",
            expect: Expect::Value(6), // 1+2+3; i=4 takes the continue
            marks: vec![],
            release_too: false,
        },
        Case {
            name: "named-args-source-order",
            profile: "scalar",
            trace: true,
            src: "fn pair(a int, b int) int { return a * 10 + b }\n\
                   fn main() int { return pair(b: mark_b(), a: mark_a()) }",
            expect: Expect::Value(12), // a=mark_a()=1, b=mark_b()=2 → 1*10+2
            marks: vec!["mark_b", "mark_a"], // SOURCE order, not placement order
            release_too: false,
        },
        Case {
            name: "observable-before-trap",
            profile: "scalar",
            trace: true,
            src: "fn main() int { let observed = mark_a(); return 2147483647 + 1 }",
            expect: Expect::Trap("Overflow"),
            marks: vec!["mark_a"],
            release_too: false,
        },
        Case {
            name: "or-shortcircuit-skips-rhs",
            profile: "scalar",
            trace: true,
            src: "fn main() int { if true || mark_b() > 99 { return 8 }; return 9 }",
            expect: Expect::Value(8),
            marks: vec![], // RHS never evaluated → no mark_b
            release_too: false,
        },
        Case {
            name: "bool-params-and-eq",
            profile: "scalar",
            trace: false,
            src: "fn flag(a bool, b bool) bool { return a && !b }\n\
                   fn main() int { if flag(true, false) == true { return 1 }; return 2 }",
            expect: Expect::Value(1),
            marks: vec![],
            release_too: false,
        },
    ]
}

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/a2x-out")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// One full emit → compile → run → observe pass for a case and flavor.
fn run_case(case: &Case, flavor: &str, extra_flags: &[&str]) {
    let caps = if case.trace {
        vec![TRACE_CAPABILITY.to_string()]
    } else {
        vec![]
    };
    let compiled = if case.profile == "core" {
        compile_source(case.src, "probe", &caps).expect("frontend compile")
    } else {
        compile_scalar_source(case.src, "probe", &caps).expect("frontend compile")
    };

    let view = SemanticView::from_checked(&compiled.checked).expect("view admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = out_root().join(case.name).join(flavor);
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
        Expect::Value(v) => format!("value:{}\n", v),
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
fn true_chain_roundtrip() {
    let cases = cases();
    for case in &cases {
        run_case(case, "debug", &[]);
        if case.release_too {
            run_case(case, "release", &["-O", "-C", "debug-assertions=off"]);
        }
    }
}

/// Admission negatives (V-03 shape, view level): wider HIR never passes the
/// view into a target — a text-domain module (str values) is a NAMED
/// domain rejection, not a passthrough.
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
/// named rejections, not silent picks.
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
