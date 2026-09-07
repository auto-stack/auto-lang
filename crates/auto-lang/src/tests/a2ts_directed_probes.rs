//! PLAN-577: a2ts directed compile-level probes (.at → emit → `tsc --noEmit`).
//!
//! Source: auto-down DEBTS 016 a2ts T1-T4 (转介单 052 附件⑤). The legacy
//! a2ts golden corpus is `#[ignore]`d wholesale; these four probes follow the
//! Plan 427 `a2r_compile_smoke` precedent instead — text goldens cannot catch
//! compile-level breakage, so each probe typechecks its fresh product with a
//! real `tsc`. `#[ignore]`d because they shell out to node/tsc; run on demand:
//! `cargo test -p auto-lang --lib --features test-trans a2ts_directed -- --ignored --nocapture`
//!
//! tsc resolution order: `$AUTO_TSC` (path to tsc.js) → the widgets-gallery
//! (auto-os, PLAN-590 迁址) / 015-notes node_modules of the MAIN checkout
//! (worktrees have no node_modules — red line: no junctions; read-only
//! reference to the main checkout is the documented cross-repo resolution
//! order) → skip loudly if none exists.

use crate::{
    error::AutoResult,
    parser::Parser,
    trans::{Sink, Trans, typescript::TypeScriptTrans},
};

fn transpile_inline(name: &str, src: &str) -> String {
    let _scope = crate::scope_manager::ScopeManager::new();
    let mut parser = Parser::from(src);
    let ast = parser.parse().expect("parse failed");
    let mut sink = Sink::new(name.into());
    let mut trans = TypeScriptTrans::new(name.into());
    trans.trans(ast, &mut sink).expect("trans failed");
    let bytes = sink.done().expect("sink failed");
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Resolve a runnable tsc. Returns (node, tsc_js) or None when unavailable.
fn find_tsc() -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    if let Ok(p) = std::env::var("AUTO_TSC") {
        let tsc = std::path::PathBuf::from(p);
        return Some((tsc.clone(), tsc));
    }
    // PLAN-590:widgets-gallery 迁 auto-os 顶层(gen 产物随画廊;015-notes
    // 仍留框架仓 examples/ui)。绝对主检出锚沿既有跨仓只读引用纪律。
    let candidates = [
        "D:/autostack/auto-os/widgets-gallery/gen/front/vue/node_modules/typescript/lib/tsc.js",
        "D:/autostack/auto-lang/examples/ui/015-notes/gen/front/vue/node_modules/typescript/lib/tsc.js",
    ];
    for c in candidates {
        let p = std::path::PathBuf::from(c);
        if p.is_file() {
            return Some((std::path::PathBuf::from("node"), p));
        }
    }
    None
}

enum TscOutcome {
    /// No tsc installation found — probe skips loudly (env hint included).
    Unavailable,
    Ok,
    Errors(String),
}

fn tsc_check(name: &str, ts_src: &str) -> TscOutcome {
    let Some((node, tsc)) = find_tsc() else {
        return TscOutcome::Unavailable;
    };
    let file = std::env::temp_dir().join(format!("a2ts_probe_{name}.ts"));
    std::fs::write(&file, ts_src).expect("write probe ts");
    let out = std::process::Command::new(&node)
        .arg(&tsc)
        .arg("--noEmit")
        .arg("--strict")
        .arg("--isolatedModules")
        .arg("--target")
        .arg("es2020")
        .arg("--moduleResolution")
        .arg("node")
        .arg(&file)
        .output();
    match out {
        Err(e) => TscOutcome::Errors(format!("failed to spawn node/tsc: {e}")),
        Ok(o) if o.status.success() => TscOutcome::Ok,
        Ok(o) => TscOutcome::Errors(format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        )),
    }
}

fn assert_tsc_clean(name: &str, src: &str) {
    let ts = transpile_inline(name, src);
    match tsc_check(name, &ts) {
        TscOutcome::Unavailable => {
            eprintln!(
                "SKIP (no tsc found — set AUTO_TSC to a tsc.js path to enable): {name}"
            );
        }
        TscOutcome::Errors(err) => {
            panic!("tsc reported errors for {name}:\n{err}\n--- emitted TS ---\n{ts}");
        }
        TscOutcome::Ok => {}
    }
}

/// T1: multi-payload enum constructor call sites pass args spread
/// (`Op.Add("x", 1)`), while the emitted factory takes the payload tuple as a
/// single parameter — arity mismatch is TS2554 (auto-down DEBTS 016 T1).
#[test]
#[ignore = "shells out to tsc; on-demand compile-level guard (Plan 577)"]
fn a2ts_probe_t1_multi_payload_enum_ctor() {
    assert_tsc_clean(
        "t1_multi_payload_enum",
        "enum Op {\n    Add(str, int)\n    Nil\n}\n\nfn mk() Op {\n    return Op.Add(\"x\", 1)\n}\n\nfn main() {\n    print(mk())\n}\n",
    );
}

/// T2: `is` with an `else ->` arm must emit a working else branch — both the
/// int-literal form and the enum-discriminated form (auto-down DEBTS 016 T2:
/// the breakage was on enum matches, where the discipline was "enumerate all
/// variants + trailing fallback return").
#[test]
#[ignore = "shells out to tsc; on-demand compile-level guard (Plan 577)"]
fn a2ts_probe_t2_is_else_arm() {
    assert_tsc_clean(
        "t2_is_else_arm",
        "fn cls(v int) {\n    is v {\n        1 -> print(\"one\")\n        else -> print(\"other\")\n    }\n}\n\nfn main() {\n    cls(1)\n    cls(2)\n}\n",
    );
    assert_tsc_clean(
        "t2_is_else_enum",
        "enum Op {\n    Add(str)\n    Nil\n}\n\nfn d(o Op) {\n    is o {\n        Op.Add(s) -> print(s)\n        else -> print(\"other\")\n    }\n}\n\nfn main() {\n    d(Op.Add(\"x\"))\n    d(Op.Nil)\n}\n",
    );
}

/// T3: `is` on an optional must typecheck and compare the wrapped value, not
/// emit a 恒假 identity comparison — the literal-arm form and the
/// Some/None-arm form (auto-down DEBTS 016 T3).
#[test]
#[ignore = "shells out to tsc; on-demand compile-level guard (Plan 577)"]
fn a2ts_probe_t3_is_on_optional() {
    assert_tsc_clean(
        "t3_is_on_optional",
        "fn dflt(x int?) {\n    is x {\n        1 -> print(\"one\")\n        else -> print(\"other\")\n    }\n}\n\nfn main() {\n    dflt(1)\n    dflt(None)\n}\n",
    );
    assert_tsc_clean(
        "t3_is_optional_some_none",
        "fn d(x int?) {\n    is x {\n        Some(v) -> print(v)\n        None -> print(\"none\")\n    }\n}\n\nfn main() {\n    d(1)\n    d(None)\n}\n",
    );
}

/// T4: struct constructor in return/let position needs `new` (gen.mjs B1
/// post-fix retired), and scalar enums must not be `const enum` under
/// --isolatedModules (gen.mjs B2 post-fix retired).
#[test]
#[ignore = "shells out to tsc; on-demand compile-level guard (Plan 577)"]
fn a2ts_probe_t4_struct_new_and_plain_enum() {
    assert_tsc_clean(
        "t4_struct_new",
        "type P {\n    x int\n}\n\nenum Color {\n    Red\n    Green\n}\n\nfn mkP() P {\n    return P(1)\n}\n\nfn mkC() Color {\n    return Color.Red\n}\n\nfn main() {\n    let p = mkP()\n    let q = P(2)\n    print(p.x + q.x)\n    print(mkC() == Color.Red)\n}\n",
    );
}
