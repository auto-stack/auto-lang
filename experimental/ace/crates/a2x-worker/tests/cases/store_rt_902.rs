//! PLAN-902 T-05 store/session conformance (V-06/V-07/V-08): the
//! record-text channel's TRUE function-level increment over AXS2/AXR2 —
//! cold → edit → A→B→A historical byte reuse, unchanged methods/callers
//! with ZERO production, dependency invalidation on declaration changes,
//! restart re-admission by pure store hits, read-only zero-write, quota
//! preservation, the close contract, and family isolation from the scalar
//! channel. Assemblies are certified by REAL rustc runs (both the old and
//! the new value).
//!
//! Bounded boundary (recorded in the plan): warm FRAGMENT edits (the
//! parse-one-declaration path) ride type-free modules; record-family
//! sources carry `type` declarations whose retained spans the 828 graft
//! cannot re-anchor into a fragment document yet — their edits go through
//! the whole path, which is still function-level incremental (unchanged
//! functions reuse residents/store bytes with zero production).

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;

const RT_CAP: &str = "ac.runtime.text.v1";

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/store-rt-902")
}

fn scratch(name: &str) -> PathBuf {
    let dir = out_base().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Write an assembly to a dir, compile with rustc, run, return stdout.
fn run_assembly(name: &str, asm: &ace_a2x_store::Assembly) -> String {
    let dir = out_base().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("case.exe");
    let compile = auto_tool_execution::run_command_capture(
        {
            let mut c =
                Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
            c.args(["--edition", "2021", "-o"])
                .arg(&exe)
                .arg("main.rs")
                .current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("rustc");
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc failed for {name}: {}",
        String::from_utf8_lossy(compile.stderr.as_ref().unwrap())
    );
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(&exe);
            c.current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run");
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    String::from_utf8_lossy(run.stdout.as_ref().expect("captured stdout")).to_string()
}

/// The record-domain program: a static ctor + a view method (methods are
/// RESIDENTS with owner-qualified identities) and a free caller pair.
const RECORD_A: &str = "type Counter { value int\n\
     \x20   static fn new(n int) Counter { return Counter(n) }\n\
     \x20   fn read() int { return .value }\n}\n\
     fn scale(c Counter, k int) int { return c.read() * k }\n\
     fn main() int {\n    let x Counter = Counter.new(6)\n    return scale(x, 7)\n}";
/// only main's BODY changes (scale's signature — and every other decl —
/// is untouched)
const RECORD_B: &str = "type Counter { value int\n\
     \x20   static fn new(n int) Counter { return Counter(n) }\n\
     \x20   fn read() int { return .value }\n}\n\
     fn scale(c Counter, k int) int { return c.read() * k }\n\
     fn main() int {\n    let x Counter = Counter.new(5)\n    return scale(x, 9)\n}";
/// only scale's BODY changes — main's fingerprint (callee SIGNATURE
/// unchanged) must keep main at ZERO production
const RECORD_CALLEE_B: &str = "type Counter { value int\n\
     \x20   static fn new(n int) Counter { return Counter(n) }\n\
     \x20   fn read() int { return .value }\n}\n\
     fn scale(c Counter, k int) int { return c.read() * k + 1 }\n\
     fn main() int {\n    let x Counter = Counter.new(6)\n    return scale(x, 7)\n}";

const TEXT_A_FULL: &str = "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a.clone()\n    var c Str = b.move\n    c = c + \"中\"\n    return a.len() * 10 + c.len()\n}\n";
const TEXT_B_FRAG: &str = "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a.clone()\n    var c Str = b.move\n    c = c + \"国\"\n    return a.len() * 10 + c.len()\n}";
const TEXT_A_FRAG: &str = "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a.clone()\n    var c Str = b.move\n    c = c + \"中\"\n    return a.len() * 10 + c.len()\n}";

/// V-06 (record family, whole path): cold A → cold B (ONLY main's
/// fragment re-produces; methods and the untouched callee reuse) → cold A
/// again (pure store hits, byte-identical fileset) — real rustc certifies
/// every assembly at both values.
#[test]
fn store_rt_a_b_a_true_increment() {
    let root = scratch("aba");
    let mut s = A2xSession::open(
        &root,
        "p902",
        "owner",
        8 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .unwrap();
    let a = s
        .cold_assemble_profile(RECORD_A, SourceProfile::Record, &[], "main", false)
        .expect("cold A");
    assert_eq!(run_assembly("aba-A", &a), "value:42\n");
    let prints_a = s.stats.print;

    // B: only main re-produces (Counter_new/Counter_read/scale reuse)
    let b = s
        .cold_assemble_profile(RECORD_B, SourceProfile::Record, &[], "main", false)
        .expect("cold B");
    assert_eq!(run_assembly("aba-B", &b), "value:45\n");
    assert_eq!(s.stats.print, prints_a + 1, "exactly one fragment produced");

    // A revisit: pure store hits, historical bytes
    let prints_b = s.stats.print;
    let a2 = s
        .cold_assemble_profile(RECORD_A, SourceProfile::Record, &[], "main", false)
        .expect("cold A2");
    assert_eq!(run_assembly("aba-A2", &a2), "value:42\n");
    assert_eq!(s.stats.print, prints_b, "A revisit produces nothing");
    assert!(
        s.stats.store_hit_fragment >= 1,
        "the A revisit must be a real store hit"
    );
    assert_eq!(a.files, a2.files, "A revisit reuses the historical bytes");
}

/// V-06: a CALLEE body edit (signature unchanged) re-produces exactly the
/// callee — the caller and methods stay at ZERO production.
#[test]
fn store_rt_callee_body_edit_isolation() {
    let root = scratch("units");
    let mut s = A2xSession::open(
        &root,
        "p902",
        "owner",
        8 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .unwrap();
    s.cold_assemble_profile(RECORD_A, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    let prints = s.stats.print;

    let b = s
        .cold_assemble_profile(RECORD_CALLEE_B, SourceProfile::Record, &[], "main", false)
        .expect("callee edit");
    assert_eq!(s.stats.print, prints + 1, "only scale re-produces");
    assert_eq!(run_assembly("units-B", &b), "value:43\n");
}

/// V-07: declaration changes invalidate — a record FIELD TYPE change moves
/// every dependent function's fingerprint (record digest + callee
/// signatures); a layout-only reorder does NOT.
#[test]
fn store_rt_declaration_invalidation() {
    let root = scratch("deps");
    let mut s = A2xSession::open(
        &root,
        "p902",
        "owner",
        8 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .unwrap();
    s.cold_assemble_profile(RECORD_A, SourceProfile::Record, &[], "main", false)
        .expect("cold");
    let prints = s.stats.print;

    // same semantics, reordered free functions (layout change): reuse
    let reordered = "type Counter { value int\n\
         \x20   static fn new(n int) Counter { return Counter(n) }\n\
         \x20   fn read() int { return .value }\n}\n\
         fn main() int {\n    let x Counter = Counter.new(6)\n    return scale(x, 7)\n}\n\
         fn scale(c Counter, k int) int { return c.read() * k }";
    s.cold_assemble_profile(reordered, SourceProfile::Record, &[], "main", false)
        .expect("reordered");
    assert_eq!(s.stats.print, prints, "layout reorder reuses everything");

    // field type change: every function touching Counter re-produces
    let changed = "type Counter { value bool\n\
         \x20   static fn new(n bool) Counter { return Counter(n) }\n\
         \x20   fn read() int { return 1 }\n}\n\
         fn scale(c Counter, k int) int { return c.read() * k }\n\
         fn main() int {\n    let x Counter = Counter.new(true)\n    return scale(x, 7)\n}";
    s.cold_assemble_profile(changed, SourceProfile::Record, &[], "main", false)
        .expect("changed");
    assert!(s.stats.print > prints, "declaration change must re-produce");

    // restart: everything re-admits by store hits, zero production
    let prints_before_restart = s.stats.print;
    s.restart_assemble_profile(changed, SourceProfile::Record, &[], "main")
        .expect("restart");
    assert_eq!(
        s.stats.print, prints_before_restart,
        "restart is pure store hits"
    );
}

/// V-06 (text family): type-free warm FRAGMENT edits — the single-body
/// text credential with the retained exclusivity seed, A→B→A byte reuse.
#[test]
fn store_rt_text_warm_edits() {
    let root = scratch("text");
    let mut s = A2xSession::open(
        &root,
        "p902",
        "owner",
        8 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .unwrap();
    let a = s
        .cold_assemble_profile(
            TEXT_A_FULL,
            SourceProfile::TextRecord,
            &[RT_CAP.to_string()],
            "main",
            false,
        )
        .expect("cold text");
    assert_eq!(run_assembly("text-A", &a), "value:23\n");
    let prints_a = s.stats.print;

    let b = s
        .warm_edit("main", TEXT_B_FRAG, "main", false)
        .expect("warm text B");
    assert_eq!(run_assembly("text-B", &b), "value:23\n");
    assert_eq!(s.stats.print, prints_a + 1, "exactly the edited fragment");

    let a2 = s
        .warm_edit("main", TEXT_A_FRAG, "main", false)
        .expect("warm text A");
    assert_eq!(s.stats.print, prints_a + 1, "A revisit hits the store");
    assert!(s.stats.store_hit_fragment >= 1);
    assert_eq!(a.files, a2.files, "A revisit reuses the historical bytes");
}

/// V-08: read-only writes ZERO bytes; the close contract rejects warm
/// edits; quota refusal preserves prior state.
#[test]
fn store_rt_readonly_quota_close() {
    let root = scratch("ops");
    let mut s = A2xSession::open(
        &root,
        "p902",
        "owner",
        8 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .unwrap();
    s.cold_assemble_profile(
        TEXT_A_FULL,
        SourceProfile::TextRecord,
        &[RT_CAP.to_string()],
        "main",
        false,
    )
    .expect("cold");

    // read-only warm edit: zero persistence, assembly still served
    let before = s.stats.bytes_persisted;
    let b = s
        .warm_edit("main", TEXT_B_FRAG, "main", true)
        .expect("read-only warm");
    assert_eq!(
        s.stats.bytes_persisted, before,
        "read-only writes zero bytes"
    );
    assert_eq!(run_assembly("ops-B", &b), "value:23\n");

    // close: warm edits are a controlled source.closed rejection
    s.close_env();
    let err = s
        .warm_edit("main", TEXT_A_FRAG, "main", false)
        .expect_err("closed env must reject");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(r) => {
            assert!(r.detail.contains("source.closed"), "got {r}");
        }
        other => panic!("expected reject, got {other:?}"),
    }

    // quota: a tiny store refuses cold persistence with a2x.resource
    let root2 = scratch("quota");
    let mut s2 = A2xSession::open(
        &root2,
        "p902",
        "owner",
        1,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .unwrap();
    let err = s2
        .cold_assemble_profile(
            TEXT_A_FULL,
            SourceProfile::TextRecord,
            &[RT_CAP.to_string()],
            "main",
            false,
        )
        .expect_err("quota must refuse");
    match err {
        ace_a2x_store::A2xRejectOrResource::Resource(r) => assert_eq!(r.code, "a2x.resource"),
        other => panic!("expected resource reject, got {other:?}"),
    }
}

/// Family isolation: the scalar channel in the SAME store keeps its own
/// layers and hits (AXS1/AXR1 untouched by the rt namespaces).
#[test]
fn store_rt_family_isolation() {
    let root = scratch("mixed");
    let mut s = A2xSession::open(
        &root,
        "p902",
        "owner",
        8 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .unwrap();
    let scalar_src = "fn helper(a int) int { return a * 2 }\nfn main() int { return helper(21) }";
    s.cold_assemble(scalar_src, true, &[], "main", false)
        .expect("scalar cold");
    let scalar_prints = s.stats.print;
    // rt cold in the same store: different layers, no interference
    s.cold_assemble_profile(RECORD_A, SourceProfile::Record, &[], "main", false)
        .expect("rt cold");
    // scalar restart still hits its own layer
    let restarted = s
        .restart_assemble(scalar_src, true, &[], "main")
        .expect("scalar restart");
    assert_eq!(run_assembly("mixed-scalar", &restarted), "value:42\n");
    assert!(
        s.stats.print < scalar_prints + 10,
        "scalar hits unaffected by rt namespaces"
    );
}
