//! PLAN-926 CA-C (T-07): the C collections PURE-RESTORE chain — AXP3
//! seed-carrying anchor publish (the nominal type table rides the
//! anchor), NEW-SESSION restore with P0/R>0 accounting and byte-identical
//! BLIND re-assembly (view-less, seed-driven), the next FREE-FUNCTION
//! body edit after a restore (env rebuild counted, edited-only
//! production), restore of the EDITED source pure again, bad-edit
//! repair-then-restore, read-only publish refusal, lease+GC liveness,
//! and cross-target isolation (the Rust collections target keeps its
//! frozen no-anchor behavior).

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::{Path, PathBuf};

fn root(tag: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/a2x-out-c-coll/restore-{}", tag))
}

/// Three free functions around a record + variant main (the warm/restore
/// corpus shape; the seed carries records + variants + the list table).
const SRC: &str = "fn left_value() int { return 20 }\nfn right_value() int { return 22 }\ntype Item { name Str; code int }\nenum Choice { Data(Item)\nEnd }\nfn main() int {\n    var xs = List<Choice>.new()\n    xs.push(Choice.Data(Item(\"ab\", left_value() + right_value())))\n    let t = xs.remove(0)\n    is t {\n        Choice.Data(item) -> { return item.name.len() + item.code }\n        Choice.End -> { return 0 }\n    }\n}";

fn open_c(root: &Path) -> A2xSession {
    A2xSession::open(
        root,
        "a2c-coll-restore",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap()
}

fn compile_run(dir: &Path, files: &[(String, String)]) -> String {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), "").unwrap();
    let (out, _) = ace_a2x_worker::crun::compile_and_run_c(dir, "case.exe").unwrap();
    out
}

/// V-10 core: cold publish → FRESH session restore is PURE (P0 / R>0)
/// and byte-identical (blind: the seed from the anchor drives the type
/// table, no view, no Checked deserialization); the next body edit works
/// after the restore and rebinds the anchor; restore of the EDITED
/// source is pure again.
#[test]
fn coll_restore_pure_p0_then_next_edit_then_restore_again() {
    let root = root("p0");
    let _ = std::fs::remove_dir_all(&root);
    // writer: cold publish (the type-table seed rides the AXP3 anchor)
    let mut w = open_c(&root);
    let cold = w
        .cold_assemble_profile(SRC, SourceProfile::Collections, &[], "main", false)
        .unwrap();
    assert!(w.last_anchor().is_some(), "cold publishes the AXP3 anchor");
    // 2 + 42 = 44 ("ab".len()=2 + code 42)
    assert_eq!(compile_run(&root.join("cold"), &cold.files), "value:44\n");

    // restorer: fresh session, pure restore — no parse/lower/print/
    // persist, bounded reads + structural/semantic/content verifies only
    let mut r = open_c(&root);
    let before = r.stats().clone();
    let restored = r
        .restore_source_bound(SRC, SourceProfile::Collections, &[], "main")
        .expect("pure restore");
    let after = r.stats().clone();
    assert_eq!(after.lower, before.lower, "restore must not lower");
    assert_eq!(after.print, before.print, "restore must not print");
    assert_eq!(after.persist, before.persist, "restore must not persist");
    assert_eq!(after.parse, before.parse, "restore must not parse");
    assert_eq!(after.whole_fallback, before.whole_fallback);
    assert!(after.restore_lookup > before.restore_lookup);
    assert!(after.restore_binding_checks > before.restore_binding_checks);
    assert!(after.restore_payload_verifies > before.restore_payload_verifies);
    assert!(after.restore_fileset_verifies > before.restore_fileset_verifies);
    assert_eq!(
        restored.files, cold.files,
        "blind re-assembly == cold bytes"
    );
    assert_eq!(restored.manifest, cold.manifest);
    assert_eq!(
        compile_run(&root.join("restored"), &restored.files),
        "value:44\n"
    );

    // the NEXT body edit after the restore: ranged authorization (the
    // anchor rebinds; the env rebuild is counted, edited-only production)
    let old_body = "fn left_value() int { return 20 }";
    let new_body = "fn left_value() int { return 20+1 }";
    let (start, end) = (
        SRC.find(old_body).unwrap(),
        SRC.find(old_body).unwrap() + old_body.len(),
    );
    let edited_src = format!("{}{}{}", &SRC[..start], new_body, &SRC[end..]);
    let before_edit = r.stats().clone();
    let asm = r
        .warm_edit_ranged("left_value", SRC, (start, end), new_body, "main", false)
        .expect("body edit after restore");
    let after_edit = r.stats().clone();
    assert_eq!(after_edit.env_rebuilds, before_edit.env_rebuilds + 1);
    assert_eq!(after_edit.lower, before_edit.lower + 1, "edited fn only");
    assert_eq!(after_edit.print, before_edit.print + 1);
    assert!(after_edit.persist > before_edit.persist);
    assert_eq!(compile_run(&root.join("edit"), &asm.files), "value:45\n");

    // restore of the EDITED source is pure again (the rebind chain)
    let mut r2 = open_c(&root);
    let restored2 = r2
        .restore_source_bound(&edited_src, SourceProfile::Collections, &[], "main")
        .expect("restore of the edited source");
    assert_eq!(r2.stats().lower, 0, "still P0");
    assert!(r2.stats().restore_binding_checks >= 1);
    assert_eq!(restored2.files, asm.files, "restored == edited assembly");
    assert_eq!(
        compile_run(&root.join("restored2"), &restored2.files),
        "value:45\n"
    );
}

/// V-10 negatives + repair: a different source has no anchor under its
/// sha; a FAILED edit publishes nothing (the old anchor stays
/// restorable); the fix lands and rebinds, and the fixed source restores
/// to the fixed bytes.
#[test]
fn coll_restore_rejects_and_repairs() {
    let root = root("neg");
    let _ = std::fs::remove_dir_all(&root);
    let mut w = open_c(&root);
    w.cold_assemble_profile(SRC, SourceProfile::Collections, &[], "main", false)
        .unwrap();
    drop(w);

    let mut r = open_c(&root);
    // different source → no anchor under its sha
    let bad_src = "fn main() int { return 1 }";
    let err = r
        .restore_source_bound(bad_src, SourceProfile::Collections, &[], "main")
        .expect_err("no anchor for a different source");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(j) => {
            assert_eq!(j.code, "a2x.restore.no-anchor", "got: {}", j);
        }
        other => panic!("{:?}", other),
    }

    // a BAD edit keeps the old anchor restorable (the failed edit
    // publishes nothing)
    let mut e = open_c(&root);
    e.restore_source_bound(SRC, SourceProfile::Collections, &[], "main")
        .expect("restore");
    let old_body = "fn left_value() int { return 20 }";
    let (start, end) = (
        SRC.find(old_body).unwrap(),
        SRC.find(old_body).unwrap() + old_body.len(),
    );
    let bad_edit = "fn left_value() int { return unknown_name }";
    assert!(
        e.warm_edit_ranged("left_value", SRC, (start, end), bad_edit, "main", false)
            .is_err(),
        "unknown name must reject"
    );
    let mut r3 = open_c(&root);
    r3.restore_source_bound(SRC, SourceProfile::Collections, &[], "main")
        .expect("the OLD anchor still restores after the failed edit");

    // the fix lands and rebinds
    let fixed = "fn left_value() int { return 20+2 }";
    let fixed_src = format!("{}{}{}", &SRC[..start], fixed, &SRC[end..]);
    let asm = e
        .warm_edit_ranged("left_value", SRC, (start, end), fixed, "main", false)
        .expect("fixed edit");
    let mut r4 = open_c(&root);
    let restored = r4
        .restore_source_bound(&fixed_src, SourceProfile::Collections, &[], "main")
        .expect("restore after the fix");
    assert_eq!(restored.files, asm.files);
}

/// V-10: read-only colds publish nothing (no anchor); lease + GC keep
/// the anchored set alive.
#[test]
fn coll_readonly_lease_gc_liveness() {
    let root = root("gc");
    let _ = std::fs::remove_dir_all(&root);
    // read-only cold: no anchor
    let mut ro = open_c(&root.join("ro"));
    ro.cold_assemble_profile(SRC, SourceProfile::Collections, &[], "main", true)
        .unwrap();
    assert!(ro.last_anchor().is_none(), "read-only publishes nothing");
    let mut r0 = open_c(&root.join("ro"));
    let err = r0
        .restore_source_bound(SRC, SourceProfile::Collections, &[], "main")
        .expect_err("no anchor under read-only publication");
    assert!(matches!(
        err,
        ace_a2x_store::A2xRejectOrResource::Reject(ref j) if j.code == "a2x.restore.no-anchor"
    ));
    // anchoring cold + lease + GC: the anchored set survives
    let mut s = open_c(&root);
    s.cold_assemble_profile(SRC, SourceProfile::Collections, &[], "main", false)
        .unwrap();
    let lease = s.lease_current("restore-witness").expect("lease");
    let _receipt = s.gc().expect("gc runs");
    let mut r = open_c(&root);
    r.restore_source_bound(SRC, SourceProfile::Collections, &[], "main")
        .expect("anchored set survives GC under the lease");
    drop(lease);
}

/// V-13 isolation: the Rust collections target keeps its frozen
/// no-anchor behavior (restore is a NAMED rejection); a C SCALAR anchor
/// never serves a collections restore (the profile keys the anchor).
#[test]
fn coll_restore_cross_target_isolation() {
    let root = root("iso");
    let _ = std::fs::remove_dir_all(&root);
    // Rust collections: cold works, but no anchor is published and the
    // restore is a NAMED rejection (the frozen no-anchor contract)
    let mut rust_s = A2xSession::open(
        &root,
        "a2c-coll-restore",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    rust_s
        .cold_assemble_profile(SRC, SourceProfile::Collections, &[], "main", false)
        .expect("rust collections cold works");
    assert!(
        rust_s.last_anchor().is_none(),
        "rust target publishes no collections anchor"
    );
    let mut rust_r = A2xSession::open(
        &root,
        "a2c-coll-restore",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let err = rust_r
        .restore_source_bound(SRC, SourceProfile::Collections, &[], "main")
        .expect_err("rust collections restore is a named rejection");
    assert!(
        format!("{err:?}").contains("anchor"),
        "rust rejection: {err:?}"
    );

    // a C SCALAR anchor under the same owner never serves a collections
    // restore (the profile label keys the anchor)
    let scalar_src = "fn main() int { return 40 + 2 }";
    let mut cs = open_c(&root);
    cs.cold_assemble(scalar_src, true, &[], "main", false)
        .expect("C scalar cold");
    let mut cr = open_c(&root);
    let err = cr
        .restore_source_bound(SRC, SourceProfile::Collections, &[], "main")
        .expect_err("scalar anchor must not serve a collections restore");
    assert!(matches!(
        err,
        ace_a2x_store::A2xRejectOrResource::Reject(ref j) if j.code == "a2x.restore.no-anchor"
    ));
}
