//! PLAN-871 T-05 gap-closure counterexamples found by the consumer
//! migration (the 771/782 script legs running through `ac-native`):
//!
//! 1. **782 TP02** — `t.clone()` on a VIEW-typed record parameter (plain
//!    `t: Tok` params are RecordView): the clone must deep-copy field by
//!    field (Str fields via `StrToOwned` over a field view) instead of
//!    feeding a view into CLONE (`mir.clone-type`).
//! 2. **771 r01 (HIR)** — hand-authored HIR declares methods with PLAIN ids
//!    (`method d_new …`) and calls a mut-record-param function BEFORE the
//!    callee body is lowered: the method identity must accept the plain id
//!    and the `t_MutRet` wrapper ABI must be derived from the DEF, not from
//!    the already-lowered function table.
//! 3. **782 HP01 (HIR)** — the same forward-reference wrapper path on the
//!    text-record HIR fixture.
//!
//! Every case is judged by the SAME frozen exit oracle the retired CLI's
//! scripts use (9 / 0 / 2), so a regression cannot pass as "different but
//! green".

#![cfg(windows)]

#[path = "../common/mod.rs"]
mod common;

use ace_native::driver::MnDriver;
use std::path::PathBuf;

fn repo(rel: &str) -> PathBuf {
    common::repo_root().join(rel)
}

fn scratch(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ac871-gap-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn run_exe(exe: &std::path::Path) -> Option<i32> {
    std::process::Command::new(exe)
        .status()
        .ok()
        .and_then(|s| s.code())
}

#[test]
fn gap871_tp02_view_record_clone_deep_copies_owned_fields() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let src = std::fs::read_to_string(repo(
        "experimental/ace/crates/core/fixtures/text-record-source/valid/p02-overlap.at",
    ))
    .expect("p02-overlap.at");
    let caps = vec!["ac.runtime.text.v1".to_string()];
    let compiled = ace_core::source::compile_text_record_source(&src, "main.at", &caps)
        .expect("TP02 compiles");
    let dir = scratch("tp02");
    let mut d = MnDriver::ephemeral();
    let built = d
        .build_checked(
            &compiled.checked,
            "d_main",
            &dir,
            None,
            None,
            ace_av_core::contract::fnv1a64(src.as_bytes()),
        )
        .expect("TP02 lowers (view-record clone must deep-copy)");
    assert_eq!(
        run_exe(&built.exe_path),
        Some(9),
        "TP02 frozen oracle (kind 2 + len 7)"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn gap871_r01_hir_plain_method_ids_and_forward_mutret_wrapper() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let atom = std::fs::read_to_string(repo(
        "experimental/ace/crates/core/fixtures/record-hir/valid/r01-records.atom",
    ))
    .expect("r01-records.atom");
    let bundle = ace_core::bind_document(&atom).expect("r01 binds");
    let checked = ace_core::verify::verify_record(bundle).expect("r01 verifies");
    let dir = scratch("r01");
    let mut d = MnDriver::ephemeral();
    let built = d
        .build_checked(
            &checked,
            "d_main",
            &dir,
            None,
            None,
            ace_av_core::contract::fnv1a64(atom.as_bytes()),
        )
        .expect("r01 lowers (plain method ids + forward t_MutRet wrapper)");
    // The hand-authored static method must land under the owner-based unit
    // identity, exactly like the source-derived form.
    assert!(
        built.unit_sha256.iter().any(|(n, _)| n == "m.t_pair.new"),
        "method unit identity: {:?}",
        built.unit_sha256.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
    assert_eq!(run_exe(&built.exe_path), Some(0), "r01 happy path");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn gap871_ctor_operand_snapshot_survives_a_sibling_mut_call() {
    // 771 SP05 (F-771-R2-01): a constructor operand is captured at its own
    // source-order evaluation point; the later `.mut` sibling must not
    // retroactively change it. Exit 11 is the fixture's own assertion of the
    // captured field, 10 the sibling's post-call view.
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let named = "type Q {\n    v int\n}\ntype P {\n    a Q\n    b int\n}\nfn bump(mut q Q) int {\n    q.v = 2\n    return 0\n}\nfn main() int {\n    var q = Q(1)\n    let p = P { a: q, b: bump(q.mut) }\n    if q.v != 2 { return 10 }\n    if p.a.v != 1 { return 11 }\n    return 0\n}\n";
    let positional = "type Q {\n    v int\n}\ntype P {\n    a Q\n    b int\n}\nfn bump(mut q Q) int {\n    q.v = 2\n    return 0\n}\nfn main() int {\n    var q = Q(1)\n    let p = P(q, bump(q.mut))\n    if q.v != 2 { return 10 }\n    if p.a.v != 1 { return 11 }\n    return 0\n}\n";
    for (tag, src) in [("named", named), ("positional", positional)] {
        let compiled =
            ace_core::source::compile_record_source(src, "main.at", &[]).expect("SP05 compiles");
        let dir = scratch(&format!("ctor-{tag}"));
        let mut d = MnDriver::ephemeral();
        let built = d
            .build_checked(
                &compiled.checked,
                "d_main",
                &dir,
                None,
                None,
                ace_av_core::contract::fnv1a64(src.as_bytes()),
            )
            .expect("SP05 lowers");
        assert_eq!(run_exe(&built.exe_path), Some(0), "SP05 {tag} snapshot");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn gap871_hp01_text_record_hir_entry_runs() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let atom = std::fs::read_to_string(repo(
        "experimental/ace/crates/core/fixtures/text-record-hir/valid/tr01-combination.atom",
    ))
    .expect("tr01-combination.atom");
    let bundle = ace_core::bind_document(&atom).expect("HP01 binds");
    let checked = ace_core::verify::verify_text_record(bundle).expect("HP01 verifies");
    let dir = scratch("hp01");
    let mut d = MnDriver::ephemeral();
    let built = d
        .build_checked(
            &checked,
            "d_main",
            &dir,
            None,
            None,
            ace_av_core::contract::fnv1a64(atom.as_bytes()),
        )
        .expect("HP01 lowers");
    assert_eq!(run_exe(&built.exe_path), Some(2), "HP01 frozen oracle");
    let _ = std::fs::remove_dir_all(&dir);
}
