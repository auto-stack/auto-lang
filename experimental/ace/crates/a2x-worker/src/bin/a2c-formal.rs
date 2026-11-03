//! PLAN-921 T-11 FORMAL sampling (CA-A §5.4/V-10): the frozen load256
//! load (ORIGINAL generator bytes — 273,575B / 115 functions, scalar
//! surface) through the C target channel (HIR→C11, layer AXC1).
//!
//! Protocol (one parent process + one spawned child witness, sequential,
//! single instance):
//! * basis assertions (frozen size/fn count) first;
//! * every distinct revision is PRE-CERTIFIED with the REAL MSVC cl
//!   (compile+run of the emitted actual set) against the generator's
//!   INDEPENDENT closed-form expected value;
//! * states, each with warmup and 30 samples: fresh (fresh store, cold),
//!   hit (same session re-request — zero production), restart (new
//!   session, same store — store hits only), body-edit (30 distinct
//!   fn_037 ranged fragment revisions — the C anchor-rebind chain),
//!   restore (new session pure source-bound restore — P0);
//! * nearest-rank P50/P90/P95/P99 per state; P90<=2000ms binds every
//!   warm path (fresh reported separately); raw samples stay in the
//!   receipt;
//! * NEW-PROCESS witness: the parent spawns a CHILD PROCESS
//!   (`--child-restore <store>`) that opens the store cold, performs a
//!   pure restore and one follow-up body edit, asserting P0 restore
//!   counters — the cross-process proof of the restore chain.
//! * receipt JSON to stdout; exit 0 iff all warm gates pass.

use std::path::PathBuf;
use std::time::Instant;

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::A2xSession;
use ace_a2x_worker::load256;

const SAMPLES: usize = 30;
const GATE_MS: u64 = 2_000;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("a2c-formal-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn open(root: &std::path::Path) -> A2xSession {
    A2xSession::open(
        root,
        "a2c-formal",
        "formal",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap()
}

/// The fn_037 declaration fragment of revision `r` (generator-truth).
fn fragment_of(revision: u32) -> String {
    let src = load256::generate_revision(revision).source;
    let start = src.find("fn fn_037").expect("fn_037 present");
    let end = src.find("fn fn_038").expect("fn_038 follows");
    src[start..end].trim_end().to_string()
}

/// The fn_037 byte range in `src` (ranged-edit authorization).
fn range_of_fn037(src: &str) -> (usize, usize) {
    let start = src.find("fn fn_037").expect("fn_037 present");
    let end = src.find("fn fn_038").expect("fn_038 follows");
    (start, end)
}

/// REAL cl compile+run of an emitted C assembly; the observed stdout line.
fn certify_assembly(asm: &ace_a2x_store::Assembly, tag: &str) -> String {
    let dir = tmp(&format!("cert-{}", tag));
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let (out, _) = ace_a2x_worker::crun::compile_and_run_c(&dir, "a2c_formal.exe")
        .unwrap_or_else(|e| panic!("certify {tag}: {e}"));
    let line = out.trim().to_string();
    let _ = std::fs::remove_dir_all(&dir);
    line
}

fn nearest_rank(sorted: &[u64], p: u64) -> u64 {
    let n = sorted.len();
    let idx = (((p as usize * n) + 99) / 100).clamp(1, n) - 1;
    sorted[idx]
}

fn stats(samples: &[u64]) -> serde_json::Value {
    let mut s = samples.to_vec();
    s.sort_unstable();
    serde_json::json!({
        "n": s.len(),
        "min": s[0],
        "p50": nearest_rank(&s, 50),
        "p90": nearest_rank(&s, 90),
        "p95": nearest_rank(&s, 95),
        "p99": nearest_rank(&s, 99),
        "max": *s.last().unwrap(),
        "raw_ms": s,
    })
}

/// The child-process restore witness: pure restore (P0 asserted) + one
/// follow-up ranged body edit, printing its counters and exiting 0.
fn child_restore(store: &str) -> ! {
    use ace_core::source::SourceProfile;
    let base = load256::generate_revision(0);
    let mut s = open(std::path::Path::new(store));
    let t = Instant::now();
    let asm = s
        .restore_source_bound(&base.source, SourceProfile::Scalar, &[], "main")
        .unwrap_or_else(|e| panic!("child restore: {e:?}"));
    let restore_ms = t.elapsed().as_millis() as u64;
    let st = s.stats().clone();
    assert_eq!(st.lower, 0, "child restore is PURE (P0)");
    assert!(st.restore_binding_checks >= 115);
    // follow-up body edit works after the cross-process restore
    let (start, end) = range_of_fn037(&base.source);
    let frag = fragment_of(1);
    let asm2 = s
        .warm_edit_ranged("fn_037", &base.source, (start, end), &frag, "main", false)
        .unwrap_or_else(|e| panic!("child edit-after-restore: {e:?}"));
    let expected = format!("value:{}", load256::generate_revision(1).expected_main);
    assert_eq!(
        certify_assembly(&asm, "child-restore"),
        format!("value:{}", base.expected_main)
    );
    assert_eq!(certify_assembly(&asm2, "child-edit"), expected);
    println!(
        "{}",
        serde_json::json!({
            "child": true,
            "restore_ms": restore_ms,
            "restore_counters": {
                "lookup": st.restore_lookup,
                "binding_checks": st.restore_binding_checks,
                "payload_verifies": st.restore_payload_verifies,
                "fileset_verifies": st.restore_fileset_verifies,
                "lower": st.lower,
                "env_rebuilds": st.env_rebuilds,
            }
        })
    );
    std::process::exit(0);
}

fn main() {
    // child witness mode
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "--child-restore" {
            let store = std::env::args().nth(2).expect("store path");
            child_restore(&store);
        }
    }

    let (bytes, fns) = load256::frozen_facts_ok();
    let base = load256::generate_revision(0);
    let t_all = Instant::now();

    // ---- pre-certification of EVERY distinct revision (0..=30) with
    // the REAL MSVC cl against the independent closed-form oracle ----
    let root = tmp("store");
    let mut cert_ok = 0u32;
    {
        use ace_core::source::SourceProfile;
        let mut s = open(&root);
        let asm = s
            .cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
            .unwrap();
        assert_eq!(
            certify_assembly(&asm, "rev-0"),
            format!("value:{}", base.expected_main),
            "base certification vs the independent closed-form oracle"
        );
        cert_ok += 1;
        let mut cur_src = base.source.clone();
        for r in 1..=SAMPLES as u32 {
            let rev = load256::generate_revision(r);
            let (start, end) = range_of_fn037(&cur_src);
            let frag = fragment_of(r);
            let asm = s
                .warm_edit_ranged("fn_037", &cur_src, (start, end), &frag, "main", false)
                .unwrap_or_else(|e| panic!("edit r{r}: {e:?}"));
            assert_eq!(
                certify_assembly(&asm, &format!("rev-{r}")),
                format!("value:{}", rev.expected_main),
                "revision {r} certification vs its independent oracle"
            );
            cert_ok += 1;
            cur_src = format!("{}{}{}", &cur_src[..start], frag, &cur_src[end..]);
        }
    }
    // certification polluted the store with all revisions — formal timing
    // uses a CLEAN store so states mean what they say
    let _ = std::fs::remove_dir_all(&root);

    // ---- warmup (untimed) ----
    {
        use ace_core::source::SourceProfile;
        let w = tmp("warmup");
        let mut s = open(&w);
        s.cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
            .unwrap();
        s.restart_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main")
            .unwrap();
        let _ = std::fs::remove_dir_all(&w);
    }

    use ace_core::source::SourceProfile;

    // ---- fresh x30: fresh store + cold request each sample ----
    let mut fresh: Vec<u64> = Vec::new();
    for i in 0..SAMPLES {
        let dir = tmp(&format!("fresh-{i}"));
        let mut s = open(&dir);
        let t = Instant::now();
        s.cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
            .unwrap_or_else(|e| panic!("fresh {i}: {e:?}"));
        fresh.push(t.elapsed().as_millis() as u64);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- hit x30: SAME session re-request (resident reuse, 0 production) --
    let hit_root = tmp("hit");
    let mut hit: Vec<u64> = Vec::new();
    {
        let mut s = open(&hit_root);
        s.cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
            .unwrap();
        for _ in 0..SAMPLES {
            let t = Instant::now();
            s.cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
                .unwrap();
            hit.push(t.elapsed().as_millis() as u64);
        }
        assert_eq!(s.stats().lower - 115, 0, "hit state produced nothing new");
        let _ = std::fs::remove_dir_all(&hit_root);
    }

    // ---- restart x30: NEW session, SAME store (store hits only) ----
    let rs_root = tmp("restart");
    {
        let mut s = open(&rs_root);
        s.cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
            .unwrap();
    }
    let mut restart: Vec<u64> = Vec::new();
    for _ in 0..SAMPLES {
        let mut s = open(&rs_root);
        let t = Instant::now();
        s.restart_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main")
            .unwrap();
        restart.push(t.elapsed().as_millis() as u64);
    }
    let _ = std::fs::remove_dir_all(&rs_root);

    // ---- body-edit x30 distinct revisions (ranged anchor-rebind chain) --
    let ed_root = tmp("edit");
    let mut edits: Vec<u64> = Vec::new();
    {
        let mut s = open(&ed_root);
        s.cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
            .unwrap();
        let mut cur_src = base.source.clone();
        for r in 1..=SAMPLES as u32 {
            let (start, end) = range_of_fn037(&cur_src);
            let frag = fragment_of(r);
            let t = Instant::now();
            s.warm_edit_ranged("fn_037", &cur_src, (start, end), &frag, "main", false)
                .unwrap_or_else(|e| panic!("timed edit r{r}: {e:?}"));
            edits.push(t.elapsed().as_millis() as u64);
            cur_src = format!("{}{}{}", &cur_src[..start], frag, &cur_src[end..]);
        }
        let _ = std::fs::remove_dir_all(&ed_root);
    }

    // ---- restore x30: NEW session, pure source-bound restore (P0) ----
    let rt_root = tmp("restore");
    {
        let mut s = open(&rt_root);
        s.cold_assemble_profile(&base.source, SourceProfile::Scalar, &[], "main", false)
            .unwrap();
    }
    let mut restores: Vec<u64> = Vec::new();
    for _ in 0..SAMPLES {
        let mut s = open(&rt_root);
        let t = Instant::now();
        s.restore_source_bound(&base.source, SourceProfile::Scalar, &[], "main")
            .unwrap();
        restores.push(t.elapsed().as_millis() as u64);
    }

    // ---- NEW-PROCESS witness: spawn THIS binary as a child that
    // restores the anchored store purely and edits once (asserts P0) ----
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("--child-restore")
        .arg(&rt_root)
        .output()
        .expect("spawn child restore witness");
    assert!(
        child.status.success(),
        "child restore witness failed: {}",
        String::from_utf8_lossy(&child.stderr)
    );
    let child_receipt: serde_json::Value =
        serde_json::from_slice(&child.stdout).expect("child receipt json");
    let _ = std::fs::remove_dir_all(&rt_root);

    // ---- gates ----
    let gate = |v: &[u64]| {
        nearest_rank(
            &{
                let mut x = v.to_vec();
                x.sort_unstable();
                x
            },
            90,
        ) <= GATE_MS
    };
    let gates = serde_json::json!({
        "hit_p90_le_2000": gate(&hit),
        "restart_p90_le_2000": gate(&restart),
        "body_edit_p90_le_2000": gate(&edits),
        "restore_p90_le_2000": gate(&restores),
        "fresh_p90_ms_reported_separately": nearest_rank(
            &{ let mut x = fresh.clone(); x.sort_unstable(); x }, 90),
    });
    let all_pass = gate(&hit) && gate(&restart) && gate(&edits) && gate(&restores);

    let receipt = serde_json::json!({
        "schema": "a2c-formal-sampling/1",
        "target": "c",
        "load": { "bytes": bytes, "functions": fns, "profile": "scalar",
                  "source_sha256": ace_a2x_core::sha256::sha256_hex(base.source.as_bytes()) },
        "pre_certified_revisions": cert_ok,
        "samples_per_state": SAMPLES,
        "states": {
            "fresh": stats(&fresh),
            "hit": stats(&hit),
            "restart": stats(&restart),
            "body_edit": stats(&edits),
            "restore": stats(&restores),
        },
        "gates": gates,
        "all_warm_gates_pass": all_pass,
        "child_process_restore_witness": child_receipt,
        "total_wall_s": t_all.elapsed().as_secs(),
    });
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    std::process::exit(if all_pass { 0 } else { 1 });
}
