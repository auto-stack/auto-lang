//! PLAN-892 T-06 FORMAL sampling (plan §5.6 / V-12): the frozen load256
//! load (ORIGINAL generator bytes — 273,575B / 115 functions, scalar
//! surface) through the A2X session channel.
//!
//! Protocol (one process, sequential, single instance):
//! * basis assertions (frozen size/fn count) first;
//! * every distinct revision is PRE-CERTIFIED before it is timed: the
//!   emitted actual set is really compiled and run, and the observed
//!   value must equal the generator's INDEPENDENT closed-form expected
//!   (its mirror is computed without any backend run — that IS the
//!   independent oracle);
//! * states, each with warmup and 30 samples: fresh (fresh store, cold
//!   request), hit (same session re-request — zero production), restart
//!   (new session, same store — store hits only), body-edit (30 distinct
//!   fn_037 fragment revisions);
//! * nearest-rank P50/P90/P95/P99 per state; the P90<=2000ms gate binds
//!   the warm paths (hit / restart / body-edit) per the 828 precedent
//!   (cold/init is reported separately); raw samples are kept in the
//!   receipt;
//! * receipt JSON to stdout and to the receipts dir (local, never
//!   committed as evidence bulk).

use std::path::PathBuf;
use std::time::Instant;

use ace_a2x_store::A2xSession;
use ace_a2x_worker::load256;

const SAMPLES: usize = 30;
const GATE_MS: u64 = 2_000;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("a2x-formal-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn fragment_of(revision: u32) -> String {
    let src = load256::generate_revision(revision).source;
    let start = src.find("fn fn_037").expect("fn_037 present");
    let end = src.find("fn fn_038").expect("fn_038 follows");
    src[start..end].trim_end().to_string()
}

/// REAL rustc compile+run of an emitted assembly; the observed value line.
fn certify_assembly(asm: &ace_a2x_store::Assembly, tag: &str) -> String {
    let dir = tmp(&format!("cert-{}", tag));
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let exe = dir.join("a2x_formal.exe");
    let out = std::process::Command::new(&rustc)
        .args([
            "--edition",
            "2021",
            "-O",
            "-C",
            "debug-assertions=off",
            "-o",
        ])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir)
        .output()
        .expect("spawn rustc");
    assert!(
        out.status.success(),
        "certify compile failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = std::process::Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run");
    assert!(run.status.success());
    let line = String::from_utf8_lossy(&run.stdout).trim().to_string();
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

fn main() {
    let (bytes, fns) = load256::frozen_facts_ok();
    let base = load256::generate_revision(0);
    let t_all = Instant::now();

    // ---- pre-certification of EVERY distinct revision (0..=30) ----
    let root = tmp("store");
    let mut cert_ok = 0u32;
    {
        let mut s = A2xSession::open(&root, "a2x-formal", "formal", 128 * 1024 * 1024).unwrap();
        let asm = s
            .cold_assemble(&base.source, true, &[], "main", false)
            .unwrap();
        let line = certify_assembly(&asm, "rev-0");
        assert_eq!(
            line,
            format!("value:{}", base.expected_main),
            "base certification vs the independent closed-form oracle"
        );
        cert_ok += 1;
        for r in 1..=SAMPLES as u32 {
            let rev = load256::generate_revision(r);
            let asm = s
                .warm_edit("fn_037", &fragment_of(r), "main", false)
                .unwrap_or_else(|e| panic!("edit r{r}: {e:?}"));
            let line = certify_assembly(&asm, &format!("rev-{r}"));
            assert_eq!(
                line,
                format!("value:{}", rev.expected_main),
                "revision {r} certification vs its independent oracle"
            );
            cert_ok += 1;
        }
    }
    // certification polluted the store with all revisions — formal timing
    // uses a CLEAN store so states mean what they say
    let _ = std::fs::remove_dir_all(&root);

    // ---- warmup (untimed) ----
    {
        let w = tmp("warmup");
        let mut s = A2xSession::open(&w, "a2x-formal", "formal", 128 * 1024 * 1024).unwrap();
        s.cold_assemble(&base.source, true, &[], "main", false)
            .unwrap();
        s.restart_assemble(&base.source, true, &[], "main").unwrap();
        let _ = std::fs::remove_dir_all(&w);
    }

    // ---- fresh x30: fresh store + cold request each sample ----
    let mut fresh: Vec<u64> = Vec::new();
    for i in 0..SAMPLES {
        let dir = tmp(&format!("fresh-{i}"));
        let mut s = A2xSession::open(&dir, "a2x-formal", "formal", 128 * 1024 * 1024).unwrap();
        let t = Instant::now();
        s.cold_assemble(&base.source, true, &[], "main", false)
            .unwrap_or_else(|e| panic!("fresh {i}: {e:?}"));
        fresh.push(t.elapsed().as_millis() as u64);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- hit x30: SAME session re-request (resident reuse, 0 production) ----
    let hit_root = tmp("hit");
    let mut s = A2xSession::open(&hit_root, "a2x-formal", "formal", 128 * 1024 * 1024).unwrap();
    s.cold_assemble(&base.source, true, &[], "main", false)
        .unwrap();
    let mut hit: Vec<u64> = Vec::new();
    for _ in 0..SAMPLES {
        let t = Instant::now();
        s.cold_assemble(&base.source, true, &[], "main", false)
            .unwrap();
        hit.push(t.elapsed().as_millis() as u64);
    }
    assert_eq!(s.stats().lower - 115, 0, "hit state produced nothing new");
    let _ = std::fs::remove_dir_all(&hit_root);

    // ---- restart x30: NEW session, SAME store (store hits only) ----
    let rs_root = tmp("restart");
    {
        let mut s = A2xSession::open(&rs_root, "a2x-formal", "formal", 128 * 1024 * 1024).unwrap();
        s.cold_assemble(&base.source, true, &[], "main", false)
            .unwrap();
    }
    let mut restart: Vec<u64> = Vec::new();
    for _ in 0..SAMPLES {
        let mut s = A2xSession::open(&rs_root, "a2x-formal", "formal", 128 * 1024 * 1024).unwrap();
        let t = Instant::now();
        s.restart_assemble(&base.source, true, &[], "main").unwrap();
        restart.push(t.elapsed().as_millis() as u64);
    }
    let _ = std::fs::remove_dir_all(&rs_root);

    // ---- body-edit x30 distinct revisions (warm fragment path) ----
    let ed_root = tmp("edit");
    let mut s = A2xSession::open(&ed_root, "a2x-formal", "formal", 128 * 1024 * 1024).unwrap();
    s.cold_assemble(&base.source, true, &[], "main", false)
        .unwrap();
    let mut edits: Vec<u64> = Vec::new();
    for r in 1..=SAMPLES as u32 {
        let frag = fragment_of(r);
        let t = Instant::now();
        s.warm_edit("fn_037", &frag, "main", false)
            .unwrap_or_else(|e| panic!("timed edit r{r}: {e:?}"));
        edits.push(t.elapsed().as_millis() as u64);
    }
    let _ = std::fs::remove_dir_all(&ed_root);

    // ---- gates: the P90<=2000ms binds the warm paths (828 precedent:
    // cold/init reported separately) ----
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
        "fresh_p90_ms_reported_separately": nearest_rank(&{ let mut x = fresh.clone(); x.sort_unstable(); x }, 90),
    });
    let all_pass = gate(&hit) && gate(&restart) && gate(&edits);

    let receipt = serde_json::json!({
        "schema": "a2x-formal-sampling/1",
        "load": { "bytes": bytes, "functions": fns, "profile": "scalar",
                  "source_sha256": ace_a2x_core::sha256::sha256_hex(base.source.as_bytes()) },
        "pre_certified_revisions": cert_ok,
        "samples_per_state": SAMPLES,
        "states": {
            "fresh": stats(&fresh),
            "hit": stats(&hit),
            "restart": stats(&restart),
            "body_edit": stats(&edits),
        },
        "gates": gates,
        "all_warm_gates_pass": all_pass,
        "total_wall_s": t_all.elapsed().as_secs(),
    });
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    std::process::exit(if all_pass { 0 } else { 1 });
}
