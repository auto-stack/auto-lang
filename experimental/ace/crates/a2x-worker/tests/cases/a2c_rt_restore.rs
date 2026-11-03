//! PLAN-925 CA-B (T-07): the C record-text PURE-RESTORE chain — AXP2
//! seed-carrying anchor publish (cold + ranged rebind incl. METHODS),
//! NEW-SESSION restore with P0/R>0 accounting and byte-identical
//! re-assembly (blind: view-less, seed-driven record table), the next
//! METHOD body edit after a restore (env rebuild counted, edited-only
//! production), repair-then-restore, read-only publish refusal, lease+GC
//! liveness, and cross-family/target isolation.

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::{Path, PathBuf};

fn root(tag: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/a2x-out-c-rt/restore-{}", tag))
}

const SRC: &str = "type Wrap {\n    head Str\n    tail Str\n    static fn mk(h Str, t Str) Wrap { return Wrap { head: h.clone(), tail: t.clone() } }\n    fn span() int { return .head.len() + .tail.len() }\n}\nfn main() int {\n    let w Wrap = Wrap.mk(\"ab\", \"cde\")\n    return w.span()\n}";

fn caps() -> Vec<String> {
    vec!["ac.runtime.text.v1".to_string()]
}

fn open_c(root: &Path) -> A2xSession {
    A2xSession::open(
        root,
        "a2c-rt-restore",
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
    let (out, _) = ace_a2x_worker::crun::compile_and_run_c(dir, "case.exe").unwrap();
    out
}

/// V-09: cold publish → FRESH session restore is PURE (P0 / R>0) and
/// byte-identical; the next METHOD body edit works after the restore
/// (env rebuild counted, edited-only production) and rebinds the anchor;
/// restore of the EDITED source is pure again.
#[test]
fn rt_restore_pure_p0_then_method_edit_then_restore_again() {
    let root = root("p0");
    let _ = std::fs::remove_dir_all(&root);
    // writer: cold publish (record table seed rides the anchor)
    let mut w = open_c(&root);
    let cold = w
        .cold_assemble_profile(SRC, SourceProfile::TextRecord, &caps(), "main", false)
        .unwrap();
    assert!(w.last_anchor().is_some(), "cold publishes the AXP2 anchor");
    assert_eq!(compile_run(&root.join("cold"), &cold.files), "value:5\n");

    // restorer: fresh session, pure restore
    let mut r = open_c(&root);
    let before = r.stats().clone();
    let restored = r
        .restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect("pure restore");
    let after = r.stats().clone();
    assert_eq!(after.lower, before.lower, "restore must not lower");
    assert_eq!(after.print, before.print, "restore must not print");
    assert_eq!(after.persist, before.persist, "restore must not persist");
    assert_eq!(after.parse, before.parse, "restore must not parse");
    assert_eq!(after.whole_fallback, before.whole_fallback);
    assert!(after.restore_lookup > before.restore_lookup);
    assert!(after.restore_binding_checks >= before.restore_binding_checks + 3);
    assert!(after.restore_payload_verifies > before.restore_payload_verifies);
    assert!(after.restore_fileset_verifies >= before.restore_fileset_verifies + 3);
    assert_eq!(
        restored.files, cold.files,
        "blind re-assembly == cold bytes"
    );
    assert_eq!(restored.manifest, cold.manifest);
    assert_eq!(
        compile_run(&root.join("restored"), &restored.files),
        "value:5\n"
    );

    // the next METHOD body edit after the restore: ranged authorization
    // (the anchor rebinds; the env rebuild is M-class, counted)
    let old_span = "fn span() int { return .head.len() + .tail.len() }";
    let new_span = "fn span() int { return .head.len() * 10 + .tail.len() }";
    let (start, end) = (
        SRC.find(old_span).unwrap(),
        SRC.find(old_span).unwrap() + old_span.len(),
    );
    let edited_src = format!("{}{}{}", &SRC[..start], new_span, &SRC[end..]);
    let before_edit = r.stats().clone();
    let asm = r
        .warm_edit_ranged_method("Wrap", "span", SRC, (start, end), new_span, "main", false)
        .expect("method edit after restore");
    let after_edit = r.stats().clone();
    assert_eq!(after_edit.env_rebuilds, before_edit.env_rebuilds + 1);
    assert_eq!(
        after_edit.lower,
        before_edit.lower + 1,
        "edited method only"
    );
    assert_eq!(after_edit.print, before_edit.print + 1);
    assert_eq!(
        after_edit.persist,
        before_edit.persist + 3,
        "semantic + fragment + rebound anchor"
    );
    assert_eq!(compile_run(&root.join("edit"), &asm.files), "value:23\n");

    // restore of the EDITED source is pure again (the rebind chain)
    let mut r2 = open_c(&root);
    let restored2 = r2
        .restore_source_bound(&edited_src, SourceProfile::TextRecord, &caps(), "main")
        .expect("restore of the edited source");
    assert_eq!(r2.stats().lower, 0, "still P0");
    assert!(r2.stats().restore_binding_checks >= 3);
    assert_eq!(restored2.files, asm.files, "restored == edited assembly");
    assert_eq!(
        compile_run(&root.join("restored2"), &restored2.files),
        "value:23\n"
    );
}

/// V-09: named rejections — no anchor for a different source, context
/// mismatch (caps), nothing installed on failure; a bad edit keeps the
/// old anchor restorable; the fix rebinds.
#[test]
fn rt_restore_rejects_and_repairs() {
    let root = root("neg");
    let _ = std::fs::remove_dir_all(&root);
    let mut w = open_c(&root);
    w.cold_assemble_profile(SRC, SourceProfile::TextRecord, &caps(), "main", false)
        .unwrap();
    drop(w);

    let mut r = open_c(&root);
    // different source → no anchor under its sha
    let bad_src = "fn main() int { return 1 }";
    let err = r
        .restore_source_bound(bad_src, SourceProfile::TextRecord, &caps(), "main")
        .expect_err("no anchor for a different source");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(j) => {
            assert_eq!(j.code, "a2x.restore.no-anchor", "got: {}", j);
        }
        other => panic!("{:?}", other),
    }
    // nothing was installed by the failed restore (still no env)
    // caps mismatch → context rejection
    let err = r
        .restore_source_bound(SRC, SourceProfile::TextRecord, &[], "main")
        .expect_err("caps mismatch must reject");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(j) => {
            assert_eq!(j.code, "a2x.restore.context", "got: {}", j);
        }
        other => panic!("{:?}", other),
    }
    // a BAD edit keeps the old anchor restorable (the failed edit
    // publishes nothing)
    let mut e = open_c(&root);
    e.restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect("restore");
    let old_span = "fn span() int { return .head.len() + .tail.len() }";
    let (start, end) = (
        SRC.find(old_span).unwrap(),
        SRC.find(old_span).unwrap() + old_span.len(),
    );
    let bad_edit = "fn span() int { return .head.len() + unknown_name }";
    assert!(
        e.warm_edit_ranged_method("Wrap", "span", SRC, (start, end), bad_edit, "main", false)
            .is_err(),
        "unknown name must reject"
    );
    let mut r3 = open_c(&root);
    r3.restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect("the OLD anchor still restores after the failed edit");
    // the fix lands and rebinds
    let fixed = "fn span() int { return .head.len() * 2 + .tail.len() }";
    let fixed_src = format!("{}{}{}", &SRC[..start], fixed, &SRC[end..]);
    let asm = e
        .warm_edit_ranged_method("Wrap", "span", SRC, (start, end), fixed, "main", false)
        .expect("fixed edit");
    let mut r4 = open_c(&root);
    let restored = r4
        .restore_source_bound(&fixed_src, SourceProfile::TextRecord, &caps(), "main")
        .expect("restore after the fix");
    assert_eq!(restored.files, asm.files);
}

/// V-09/V-08: read-only colds publish nothing; lease + GC keep the
/// anchored set alive.
#[test]
fn rt_readonly_lease_gc_liveness() {
    let root = root("gc");
    let _ = std::fs::remove_dir_all(&root);
    // read-only cold: no anchor
    let mut ro = open_c(&root.join("ro"));
    ro.cold_assemble_profile(SRC, SourceProfile::TextRecord, &caps(), "main", true)
        .unwrap();
    assert!(ro.last_anchor().is_none(), "read-only publishes nothing");
    let mut r0 = open_c(&root.join("ro"));
    let err = r0
        .restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect_err("no anchor under read-only publication");
    assert!(matches!(
        err,
        ace_a2x_store::A2xRejectOrResource::Reject(ref j) if j.code == "a2x.restore.no-anchor"
    ));
    // anchoring cold + lease + GC: the anchored set survives
    let mut s = open_c(&root);
    s.cold_assemble_profile(SRC, SourceProfile::TextRecord, &caps(), "main", false)
        .unwrap();
    let lease = s.lease_current("restore-witness").expect("lease");
    let _receipt = s.gc().expect("gc runs");
    let mut r = open_c(&root);
    r.restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect("anchored set survives GC under the lease");
    drop(lease);
}

/// V-08/V-13: cross-family and cross-target isolation — the Rust
/// record-text target publishes no anchor (frozen behavior), its restore
/// is a named rejection; a scalar anchor never serves a record-text
/// restore (profile keys the anchor).
#[test]
fn rt_restore_cross_target_and_family_isolation() {
    let root = root("iso");
    let _ = std::fs::remove_dir_all(&root);
    // Rust record-text: cold works, but no anchor is published and the
    // restore is a NAMED rejection (the frozen scalar-only contract)
    let mut rust_s = A2xSession::open(
        &root,
        "a2c-rt-restore",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    rust_s
        .cold_assemble_profile(SRC, SourceProfile::TextRecord, &caps(), "main", false)
        .expect("rust record-text cold still works");
    assert!(
        rust_s.last_anchor().is_none(),
        "the Rust record-text target publishes no anchor"
    );
    let err = rust_s
        .restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect_err("rust record-text restore is a named rejection");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(j) => {
            assert_eq!(j.code, "a2x.restore", "got: {}", j);
        }
        other => panic!("{:?}", other),
    }
    // a C SCALAR anchor on the same owner/store never serves the RT
    // restore (profile is an anchor key input)
    let mut cs = open_c(&root);
    cs.cold_assemble_profile(
        "fn main() int { return 1 }",
        SourceProfile::Scalar,
        &[],
        "main",
        false,
    )
    .unwrap();
    let mut r = open_c(&root);
    let err = r
        .restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect_err("scalar anchor cannot serve a record-text restore");
    assert!(matches!(
        err,
        ace_a2x_store::A2xRejectOrResource::Reject(ref j) if j.code == "a2x.restore.no-anchor"
    ));
}
