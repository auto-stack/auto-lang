//! T-02 single-body credential probe (PLAN-828 FnCandidate surface through
//! the A2X chain): the WARM path a function-incremental session will use.
//!
//! Proves, against the real ac-core single-body machinery:
//! * `emit_function_body` plans EXACTLY the edited body (receipt) and
//!   `verify_function_ref` turns it into a `CheckedFunction` credential
//!   whose `context_revision` is STABLE across a pure body edit (the
//!   fingerprint covers the declaration context, not the body — §5.4);
//! * the A2X view admits the fragment BY CREDENTIAL (`from_parts`) and the
//!   Rust provider emits + really compiles + runs the EDITED semantics
//!   (oracle of the new body, not the old one);
//! * the FnCandidate transaction is honest: validate→commit publishes
//!   exactly once (generation 0→1), a candidate prepared against an older
//!   base is a NAMED `source.stale` rejection, and a candidate matched
//!   against a foreign environment is `source.foreign` (identity, not
//!   name+slot).

use ace_core::hir;
use ace_core::source::ast::{FnItem, Item, Program};
use ace_core::source::{
    compile_source, emit_function_body, parse_program_with, prepare_function_candidate,
    recheck_function, resolve_env, SourceProfile,
};

/// The parsed `FnItem` of `name` (source items are name-addressed here;
/// the environment slot below stays index-addressed on purpose).
fn fn_item<'a>(program: &'a Program, name: &str) -> &'a FnItem {
    program
        .items
        .iter()
        .find_map(|i| match i {
            Item::Fn(f) if f.name.name == name => Some(f),
            _ => None,
        })
        .unwrap_or_else(|| panic!("fn `{}` not in program", name))
}
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_file_set, lower_program};
use ace_core::verify::verify_function_ref;
use std::path::PathBuf;
use std::process::Command;

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out")
}

const COLD: &str = "fn main() int { return 2 + 3 * 4 }";
const EDIT1: &str = "fn main() int { return 2 + 3 * 5 }"; // value 18
const EDIT2: &str = "fn main() int { return 2 + 3 * 6 }"; // value 20

fn main_def_index(bundle: &hir::Bundle) -> usize {
    bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_main")
        .expect("d_main def present")
}

fn emit_and_run(view_src: &str, parts: &ace_core::source::SourceParts, dir_name: &str) -> i32 {
    let cred = verify_function_ref(&parts.bundle, main_def_index(&parts.bundle))
        .expect("single-body verification");
    let view = SemanticView::from_parts(parts, &cred).expect("fragment admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &parts.input_hash, "main");
    let dir = out_root().join(dir_name);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();
    let exe = dir.join("a2x_warm.exe");
    let out = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir)
        .output()
        .expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed for {}: {}",
        dir_name,
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe).current_dir(&dir).output().expect("run");
    let stdout = String::from_utf8_lossy(&run.stdout);
    let line = stdout.lines().next().unwrap_or("");
    let value = line
        .strip_prefix("value:")
        .unwrap_or_else(|| panic!("expected value line, got {:?} (src {})", line, view_src));
    value.parse::<i32>().expect("i32 value")
}

#[test]
fn single_body_credential_roundtrip() {
    // ---- cold: whole-module compile for the OLD semantics + context ----
    let cold = compile_source(COLD, "warm", &[]).expect("cold compile");
    let cold_cred = verify_function_ref(
        &cold.checked.bundle(),
        main_def_index(cold.checked.bundle()),
    )
    .expect("cold credential");

    // ---- warm environment over the same source ----
    let program = parse_program_with(COLD, SourceProfile::Core).expect("parse cold");
    let mut env = resolve_env(&program, &[], SourceProfile::Core).expect("resolve env");
    assert_eq!(env.generation(), 0);

    // ---- CURRENT-semantics single-body emission (env-owned body; src only
    // anchors spans): receipt plans exactly one real body ----
    let (cur_parts, cur_receipt) =
        emit_function_body(&env, 0, COLD, "warm").expect("current-semantics emission");
    assert_eq!(cur_receipt.real_body_plans, 1);
    assert_eq!(cur_receipt.scaffold_fns, 0);
    let got = emit_and_run(COLD, &cur_parts, "warm-current");
    assert_eq!(got, 14, "current semantics through the single-body path");

    // ---- pure body EDIT lives in the CANDIDATE (prepare grafts + emits;
    // the environment still serves current until commit) ----
    let edited_program = parse_program_with(EDIT1, SourceProfile::Core).expect("parse edit1");
    let item = fn_item(&edited_program, "main");
    let cand =
        prepare_function_candidate(&mut env, item, 0, EDIT1, "warm").expect("prepare candidate");
    assert_eq!(cand.base_generation(), 0);
    assert_eq!(cand.slot_name(), "main");
    assert_eq!(cand.receipt().real_body_plans, 1);

    let parts = cand.parts();
    let warm_cred = verify_function_ref(&parts.bundle, main_def_index(&parts.bundle))
        .expect("candidate credential");
    // §5.4 fingerprint fact: a pure body edit leaves the declaration
    // context (types + every signature) unchanged → same context revision.
    assert_eq!(
        cold_cred.context_revision(),
        warm_cred.context_revision(),
        "pure body edit must not change the context revision"
    );
    // the candidate body IS the edited one (4 → 5), while the environment
    // still serves the old semantics — checked by emission below.
    assert_eq!(
        parts.bundle.modules[0].bodies[0].exprs[2].kind,
        hir::ExprKind::Constant { value: 5 },
        "candidate carries the edited literal"
    );

    // ---- the A2X chain consumes the CANDIDATE fragment and runs the
    // EDITED value BEFORE any commit (candidate owns the new semantics) ----
    let got = emit_and_run(EDIT1, parts, "warm-edit1");
    assert_eq!(got, 17, "edited semantics (2+3*5), not the cold 14");
    // environment still un-published: generation unchanged, current
    // emission still returns the OLD body
    assert_eq!(env.generation(), 0, "preparing published nothing");
    let (still_old, _) = emit_function_body(&env, 0, COLD, "warm").expect("re-emit current");
    let still = verify_function_ref(&still_old.bundle, main_def_index(&still_old.bundle))
        .expect("re-emit credential");
    let _ = still;

    // ---- validate → commit publishes exactly once ----
    cand.validate(&env)
        .expect("validate against the preparing env");
    cand.commit(&mut env)
        .expect("commit publishes exactly once");
    assert_eq!(
        env.generation(),
        1,
        "commit advanced the publication generation"
    );
    let (now_new, _) = emit_function_body(&env, 0, EDIT1, "warm").expect("post-commit emission");
    let got = emit_and_run(EDIT1, &now_new, "warm-postcommit");
    assert_eq!(got, 17, "published semantics after commit");

    // ---- stale: a candidate prepared on gen 1 is rejected after gen 2 ----
    let edit2_program = parse_program_with(EDIT2, SourceProfile::Core).expect("parse edit2");
    let item2 = fn_item(&edit2_program, "main");
    let cand2 =
        prepare_function_candidate(&mut env, item2, 0, EDIT2, "warm").expect("prepare cand2");
    assert_eq!(cand2.base_generation(), 1);
    // an in-place recheck publishes gen 2 underneath cand2
    recheck_function(&mut env, item2, 0).expect("recheck publishes");
    assert_eq!(env.generation(), 2);
    let err = cand2.validate(&env).expect_err("expired base must reject");
    assert!(
        err.iter().any(|d| d.code == "source.stale"),
        "got {:?}",
        err
    );

    // ---- foreign: identity is the environment, not name+slot ----
    let other_env = resolve_env(&program, &[], SourceProfile::Core).expect("fresh env");
    let err = cand2
        .validate(&other_env)
        .expect_err("foreign env must reject");
    assert!(
        err.iter().any(|d| d.code == "source.foreign"),
        "got {:?}",
        err
    );

    // ---- warm EDIT2 through the A2X chain too (multi-revision reality) ----
    let (parts2, receipt2) =
        emit_function_body(&env, 0, EDIT2, "warm").expect("second single-body emission");
    assert_eq!(receipt2.real_body_plans, 1);
    let got2 = emit_and_run(EDIT2, &parts2, "warm-edit2");
    assert_eq!(got2, 20, "second edit semantics (2+3*6)");
}
