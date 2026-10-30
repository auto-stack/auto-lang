//! T-04 P-private store/session probes (V-05/V-07 shapes over the REAL
//! 783 ManagedStore/CAS; no H wiring). Frozen session facts asserted:
//!
//! * cold assembly persists per-function layered artifacts (AXS1 semantic /
//!   AXR1 fragment) with producer accounting;
//! * a WARM pure body edit produces exactly ONE new fragment — the
//!   unchanged function has ZERO production (resident reuse, §5.4 "未改
//!   函数生产/工件写入均0") and keeps its fragment byte-identical;
//! * A→B→A revisit REUSES the real historical payload through a store hit
//!   (no mint of a same-hash fake);
//! * semantically equivalent source (whitespace-only change) keeps every
//!   semantic fingerprint — spans never enter the key;
//! * RESTART (fresh session, empty residents, same store) rebuilds purely
//!   from store hits ("重建后下一restart必须真hit");
//! * read-only recomputes misses but writes NOTHING;
//! * the quota gate is a controlled `a2x.resource` rejection leaving prior
//!   state and residents intact.

use auto_a2x_store::{A2xRejectOrResource, A2xSession};
use std::path::PathBuf;
use std::process::Command;

fn tmp_root(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/a2x-stores")
        .join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_assembly(asm: &auto_a2x_store::Assembly, tag: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/a2x-out")
        .join(format!("store-{}", tag));
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("a2x_session.exe");
    let out = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir)
        .output()
        .expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed ({}): {}",
        tag,
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe).current_dir(&dir).output().expect("run");
    assert!(run.status.success());
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

const COLD: &str = "fn helper(x int) int { return x * 2 }\nfn main() int { return helper(20) + 1 }";
/// warm edits take the EDITED FUNCTION'S TEXT ONLY (the fragment contract)
const HELPER_A: &str = "fn helper(x int) int { return x * 2 }";
const HELPER_B: &str = "fn helper(x int) int { return x * 3 }";
const PADDED: &str = "fn helper(x int) int {\n    return x * 2\n}\n\nfn main() int {\n    return helper(20) + 1\n}\n";

fn generated_of(asm: &auto_a2x_store::Assembly) -> &str {
    &asm.files
        .iter()
        .find(|(p, _)| p == "generated.rs")
        .unwrap()
        .1
}

fn fragment_text(generated: &str, marker: &str) -> String {
    let start = generated.find(marker).expect("fragment marker present");
    let from = generated[start..].find("pub fn").expect("pub fn") + start;
    let end = generated[from..].find("}\n").expect("fragment end") + from + 2;
    generated[from..end].to_string()
}

#[test]
fn layered_session_lifecycle() {
    let root = tmp_root("lifecycle");
    let mut s = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();

    // ---- cold: 2 fragments produced + persisted; oracle through rustc ----
    let cold = s.cold_assemble(COLD, false, &[], "main", false).unwrap();
    let st = s.stats();
    assert_eq!(st.lower, 2, "two fragments lowered cold");
    assert_eq!(st.print, 2);
    assert_eq!(st.persist, 4, "semantic+fragment per function");
    assert_eq!(run_assembly(&cold, "cold"), "value:41", "helper(20)*2+1");

    let cold_generated = generated_of(&cold).to_string();

    // ---- warm edit of helper ONLY: main has ZERO production ----
    let before = s.stats().clone();
    let warm = s.warm_edit("helper", HELPER_B, "main", false).unwrap();
    let after = s.stats();
    assert_eq!(after.lower, before.lower + 1, "ONLY helper re-lowered");
    assert_eq!(after.print, before.print + 1);
    assert!(after.persist > before.persist, "helper repersisted");
    assert_eq!(run_assembly(&warm, "warm"), "value:61", "helper(20)*3+1");

    // main's fragment text is byte-identical to cold's main fragment
    let cold_main = fragment_text(&cold_generated, "fn_main");
    let warm_main = fragment_text(generated_of(&warm), "fn_main");
    assert_eq!(
        cold_main, warm_main,
        "unchanged function keeps its fragment"
    );
    // and the helper fragment really changed
    let cold_helper = fragment_text(&cold_generated, "fn_helper");
    let warm_helper = fragment_text(generated_of(&warm), "fn_helper");
    assert_ne!(cold_helper, warm_helper);

    // ---- A→B→A: revert helper; the store HIT reuses cold's payload ----
    let before = s.stats().clone();
    let back = s.warm_edit("helper", HELPER_A, "main", false).unwrap();
    let after = s.stats();
    assert_eq!(
        after.store_hit_fragment,
        before.store_hit_fragment + 1,
        "A revisit hits AXR1"
    );
    assert_eq!(after.lower, before.lower, "no re-lowering on hit");
    assert_eq!(after.persist, before.persist, "no re-persist on hit");
    let back_helper = fragment_text(generated_of(&back), "fn_helper");
    assert_eq!(back_helper, cold_helper, "historical payload bytes reused");
    assert_eq!(run_assembly(&back, "aba"), "value:41");

    // ---- semantic equivalence: whitespace-only source keeps every fp ----
    let cold_fps: Vec<String> = extract_fps(&cold.manifest);
    let padded = {
        let mut s2 = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
        s2.cold_assemble(PADDED, false, &[], "main", false).unwrap()
    };
    let padded_fps = extract_fps(&padded.manifest);
    assert_eq!(
        cold_fps, padded_fps,
        "span/padding changes never change semantic fps"
    );

    // ---- restart: fresh session over the SAME store rebuilds from hits ----
    let mut s3 = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let restarted = s3.restart_assemble(COLD, false, &[], "main").unwrap();
    let st3 = s3.stats();
    assert_eq!(st3.lower, 0, "restart produces nothing new");
    assert_eq!(st3.persist, 0);
    assert_eq!(st3.store_hit_fragment, 2, "both fragments hit the store");
    assert_eq!(
        generated_of(&restarted),
        &cold_generated,
        "restart assembly is byte-identical"
    );
}

fn extract_fps(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .filter(|l| l.contains("sem_fp"))
        .map(|l| {
            let fp = l.split("sem_fp\": \"").nth(1).unwrap();
            fp.split('"').next().unwrap().to_string()
        })
        .collect()
}

#[test]
fn read_only_writes_nothing() {
    let root = tmp_root("readonly");
    // read-only session on an EMPTY namespace: misses recompute, writes 0
    let mut ro = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let asm = ro.cold_assemble(COLD, false, &[], "main", true).unwrap();
    let st = ro.stats();
    assert_eq!(st.lower, 2, "misses really recompute");
    assert_eq!(st.persist, 0, "read-only writes nothing");
    assert_eq!(run_assembly(&asm, "ro"), "value:41");
    // the store namespace still holds no A2X layers (nothing persisted)
    let mut rw = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let before = rw.space().total;
    let _ = rw.cold_assemble(COLD, false, &[], "main", false).unwrap();
    assert!(rw.space().total > before, "the write session does persist");
}

#[test]
fn quota_gate_is_controlled_and_preserves_state() {
    let root = tmp_root("quota");
    let mut s = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    s.cold_assemble(COLD, false, &[], "main", false).unwrap();

    // shrink the quota below what a big helper edit would need
    // a WIDE body (many statements, bounded depth) — payload grows without
    // tripping the a2x.too-deep structure budget
    let mut body = String::from("fn helper(x int) int { var s = x ");
    for i in 0..600 {
        body.push_str(&format!("; s = s + {} * 7", i));
    }
    body.push_str("; return s }");
    let big_helper = body; // the edited function's text only (fragment contract)
    drop(s);
    // measure the REOPENED namespace (WAL/trash settle across opens), then
    // open the gated session with only 64B of headroom
    let probe = A2xSession::open(&root, "a2x-probe", "probe", u64::MAX / 2).unwrap();
    let settled = probe.space().total;
    drop(probe);
    let tight = settled + 64;
    let mut s2 = A2xSession::open(&root, "a2x-probe", "probe", tight).unwrap();
    // re-cold over the same store (hits) then a big edit must hit the gate
    let _ = s2.cold_assemble(COLD, false, &[], "main", false).unwrap();
    let err = s2.warm_edit("helper", &big_helper, "main", false);
    match err {
        Err(A2xRejectOrResource::Resource(r)) => {
            assert_eq!(r.code, "a2x.resource");
        }
        other => panic!("expected resource rejection, got {:?}", other.map(|_| ())),
    }
    // prior state intact: the namespace did not grow past the quota, and
    // the surviving artifacts still serve a full restart (real hits)
    assert!(
        s2.space().total <= tight,
        "quota held: {} > {}",
        s2.space().total,
        tight
    );
    drop(s2);
    let mut s3 = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let restarted = s3.restart_assemble(COLD, false, &[], "main").unwrap();
    assert_eq!(
        s3.stats().lower,
        0,
        "post-rejection restart still hits everything"
    );
    assert_eq!(run_assembly(&restarted, "quota-restart"), "value:41");
    // and with a LEGAL quota the small edit assembles normally
    let small = s3.warm_edit("helper", HELPER_B, "main", false).unwrap();
    assert_eq!(run_assembly(&small, "quota-small"), "value:61");
}

#[test]
fn lease_and_gc_smoke() {
    let root = tmp_root("gc");
    let mut s = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let _ = s.cold_assemble(COLD, false, &[], "main", false).unwrap();
    let _lease = s.lease_current("a2x-probe-current").unwrap();
    let receipt = s.gc().unwrap();
    assert!(!receipt.is_empty());
    // after GC the project-rooted artifacts still serve restarts
    let mut s2 = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let asm = s2.restart_assemble(COLD, false, &[], "main").unwrap();
    assert_eq!(s2.stats().lower, 0, "post-GC restart still hits everything");
    assert_eq!(run_assembly(&asm, "postgc"), "value:41");
}

/// V-10 candidate-transaction controls, session face. Matrix rows and
/// where each is proven:
/// * stale (expired base)      -> incremental_cache.rs `source.stale`
/// * foreign environment       -> incremental_cache.rs `source.foreign`
/// * cancel/no-commit leaves current semantics untouched
///                             -> incremental_cache.rs (prepare alone: gen 0,
///                                current emission still OLD body)
/// * quota failure mid-edit    -> `quota_gate_is_controlled_and_preserves_state`
///                                (named a2x.resource, prior state intact)
/// * CLOSED environment        -> HERE: close_env makes every later warm edit
///   a controlled `source.closed` rejection; residents/store keep serving.
#[test]
fn closed_environment_rejects_warm_edits_but_keeps_serving() {
    let root = tmp_root("closed");
    let mut s = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let cold = s.cold_assemble(COLD, false, &[], "main", false).unwrap();
    assert!(!s.env_closed());
    s.close_env();
    assert!(s.env_closed());
    // every later warm edit is a controlled rejection (source.closed
    // surfaces through the prepare wrapper's detail)
    let err = s
        .warm_edit(
            "helper",
            "fn helper(x int) int { return x * 5 }",
            "main",
            false,
        )
        .expect_err("closed env must reject warm edits");
    match err {
        A2xRejectOrResource::Reject(r) => {
            assert!(
                r.detail.contains("source.closed"),
                "expected the 828 source.closed rejection, got: {}",
                r
            );
        }
        other => panic!("expected rejection, got {:?}", other),
    }
    // the published residents + store keep serving assemblies: a restart
    // over the same store rebuilds from hits with no production
    drop(s);
    let mut s2 = A2xSession::open(&root, "a2x-probe", "probe", 128 * 1024 * 1024).unwrap();
    let asm = s2.restart_assemble(COLD, false, &[], "main").unwrap();
    assert_eq!(s2.stats().lower, 0);
    assert_eq!(generated_of(&asm), generated_of(&cold));
}
