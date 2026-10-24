//! PLAN-818 T-03 parity tier (`mir_native_parity_`): every case runs
//! through THREE real paths — the AV executor, the landed old HIR-native
//! reference, and the NEW MIR-native pipeline — and EACH leg is checked
//! against the INDEPENDENT hand-computed expectation (V-02: "每条独立
//! 符合值/trap/trace；不只比两后端同错"). Trap legs: the AV leg must
//! report the exact TrapKind; both native legs must exit 70.

mod common;

use auto_av_core::contract::{ExecOutcome, TrapKind};

/// Value/trap/latent matrix across the three paths.
#[test]
#[cfg(windows)]
fn mir_native_parity_values_traps_matrix() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    struct Case {
        name: &'static str,
        src: String,
        caps: &'static [&'static str],
        /// Independent expectation: a value, or a trap kind.
        want: Result<i32, TrapKind>,
    }
    let min_src =
        "fn main() int {\n    var m = 0 - 2147483647\n    m = m - 1\n    return BODY\n}\n";
    let cases = vec![
        Case {
            name: "value-neg-trunc",
            src: "fn main() int {\n    return (0 - 7) / 2\n}\n".to_string(),
            caps: &[],
            want: Ok(-3),
        },
        Case {
            name: "value-compound",
            // a=1,t=28→arm false→29; b: a=1,t=46→arm true,b=2→44; 29+44=73≠72→1.
            src: "fn gate(v int) int {\n    let a = mark_a()\n    let t = v * 3 + a\n    if t > 40 {\n        return t - mark_b()\n    }\n    return t + a\n}\n\nfn main() int {\n    let x = gate(9)\n    let y = gate(15)\n    if x + y != 72 {\n        return 1\n    }\n    return 0\n}\n".to_string(),
            caps: &["hir.test.trace.v1"],
            want: Ok(1),
        },
        Case {
            name: "trap-add-overflow",
            src: "fn main() int {\n    var a = 2147483647\n    a = a + 1\n    return a\n}\n".to_string(),
            caps: &[],
            want: Err(TrapKind::IntOverflow),
        },
        Case {
            name: "trap-div-by-zero",
            src: "fn main() int {\n    return 5 / 0\n}\n".to_string(),
            caps: &[],
            want: Err(TrapKind::DivByZero),
        },
        Case {
            name: "trap-div-min-minus-one",
            src: min_src.replace("BODY", "m / (0 - 1)"),
            caps: &[],
            want: Err(TrapKind::DivOverflow),
        },
        Case {
            name: "trap-rem-min-minus-one",
            src: min_src.replace("BODY", "m % (0 - 1)"),
            caps: &[],
            want: Err(TrapKind::RemOverflow),
        },
        Case {
            name: "latent-trap-value",
            src: "fn main() int {\n    if 1 > 2 {\n        return 1 / 0\n    }\n    return 41\n}\n".to_string(),
            caps: &[],
            want: Ok(41),
        },
    ];

    for c in cases {
        // Leg 1 — AV: exact value or exact trap kind.
        let av = common::run_av(&c.src, c.caps);
        match &c.want {
            Ok(v) => {
                assert!(
                    av.diag.is_none(),
                    "{}: AV leg rejected: {:?}",
                    c.name,
                    av.diag
                );
                assert_eq!(av.value_i32(), Some(*v), "{}: AV value", c.name);
            }
            Err(kind) => match &av.av {
                Some(ExecOutcome::Trap { kind: got, .. }) => {
                    assert_eq!(got, kind, "{}: AV trap kind", c.name);
                }
                other => panic!("{}: AV must trap with {kind:?}, got {other:?}", c.name),
            },
        }
        // Leg 2 — old HIR-native reference: value exit, or 70 on a trap.
        let old = common::run_old(&c.src, c.caps, c.name);
        match &c.want {
            Ok(v) => assert_eq!(old.exit_code, *v, "{}: old-native exit", c.name),
            Err(_) => assert_eq!(old.exit_code, 70, "{}: old-native trap exit", c.name),
        }
        // Leg 3 — NEW MIR-native: same expectations, independently derived.
        let mn = common::run_mn(&c.src, c.caps, c.name);
        assert!(mn.ok, "{}: new-native failed: {:?}", c.name, mn.error);
        match &c.want {
            Ok(v) => assert_eq!(mn.exit_code, Some(*v), "{}: new-native exit", c.name),
            Err(_) => assert_eq!(mn.exit_code, Some(70), "{}: new-native trap exit", c.name),
        }
        // Trace bytes: both native legs agree with the hand-computed order
        // (the compound case emits "aab"; trap/latent cases emit nothing).
        let want_bytes: &[u8] = match c.name {
            "value-compound" => b"aab",
            _ => b"",
        };
        assert_eq!(
            old.stderr.as_bytes(),
            want_bytes,
            "{}: old-native marks",
            c.name
        );
        assert_eq!(mn.stderr, want_bytes, "{}: new-native marks", c.name);
    }
}

/// Call/loop matrix across the three paths (V-03): deep argument lists,
/// recursion, loop merging — each leg against the hand-computed value.
#[test]
#[cfg(windows)]
fn mir_native_parity_call_loop_matrix() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let cases: Vec<(&str, String, i32)> = vec![
        (
            // add7 = 5+10+20+30+40+50 +1 = 156.
            "seven-args",
            "fn add7(a int, b int, c int, d int, e int, f int, g bool) int {\n    var t = a + b + c + d + e + f\n    if g {\n        t = t + 1\n    }\n    return t\n}\n\nfn main() int {\n    return add7(2 + 3, 10, 20, 30, 40, 50, true)\n}\n".to_string(),
            156,
        ),
        (
            // f(6) = 7 + 1+2+3+4+5+6 = 28.
            "self-recursion",
            "fn f(n int) int {\n    if n <= 0 {\n        return 7\n    }\n    return f(n - 1) + n\n}\n\nfn main() int {\n    return f(6)\n}\n".to_string(),
            28,
        ),
        (
            // f(4)→g(3)→f(2)→g(1)→f(0)=1.
            "mutual-recursion",
            "fn f(n int) int {\n    if n == 0 {\n        return 1\n    }\n    return g(n - 1)\n}\n\nfn g(n int) int {\n    if n == 0 {\n        return 2\n    }\n    return f(n - 1)\n}\n\nfn main() int {\n    return f(4)\n}\n".to_string(),
            1,
        ),
        (
            // skip 2, break at 4: acc = 1+3 = 4.
            "loop-break-continue",
            "fn main() int {\n    var i = 0\n    var acc = 0\n    for i < 4 {\n        i = i + 1\n        if i == 2 {\n            continue\n        }\n        if i == 4 {\n            break\n        }\n        acc = acc + i\n    }\n    return acc\n}\n".to_string(),
            4,
        ),
        (
            // 1→2→4→8→16→116.
            "loop-merge-reassign",
            "fn main() int {\n    var i = 0\n    var acc = 1\n    for i < 5 {\n        if acc > 8 {\n            acc = acc + 100\n        } else {\n            acc = acc * 2\n        }\n        i = i + 1\n    }\n    return acc\n}\n".to_string(),
            116,
        ),
    ];
    for (name, src, want) in cases {
        let av = common::run_av(&src, &[]);
        assert!(av.diag.is_none(), "{name}: AV: {:?}", av.diag);
        assert_eq!(av.value_i32(), Some(want), "{name}: AV value");
        let old = common::run_old(&src, &[], name);
        assert_eq!(old.exit_code, want, "{name}: old-native exit");
        let mn = common::run_mn(&src, &[], name);
        assert!(mn.ok, "{name}: new-native: {:?}", mn.error);
        assert_eq!(mn.exit_code, Some(want), "{name}: new-native exit");
    }
}
