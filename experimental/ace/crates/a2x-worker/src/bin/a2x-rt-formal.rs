//! PLAN-902 T-08 FORMAL sampling (plan §5.6 / V-12): the REPRESENTATIVE
//! record/text load through the record-text A2X session channel (AXS2/AXR2).
//!
//! Protocol (one process, sequential, single instance — the explicit
//! formal window; never run concurrently with other formal samplers):
//! * the RECORD load (a resource record with static/view methods plus a
//!   caller fleet, real Str data built by loops) drives the whole-path
//!   states: fresh (fresh store, cold), hit (same session, zero
//!   production), restart (new session, same store — pure store hits);
//! * the TEXT load (type-free owned-Str functions) drives the body-edit
//!   state: 30 DISTINCT revisions of the edit target with varying string
//!   data, through the warm FRAGMENT path (single-body text credential);
//! * every distinct revision is PRE-CERTIFIED before any timing: the
//!   emitted actual set is really compiled (rustc -O) and run, and the
//!   observed value must equal the generator's INDEPENDENT closed-form
//!   expectation (computed from the generation parameters alone);
//! * nearest-rank P50/P90/P95/P99 per state; the P90<=2000ms gate binds
//!   the warm paths (hit/restart/body-edit; cold is reported separately
//!   per the 828/892 precedent); the budget ledger records store
//!   namespace totals and per-state production accounting.
//!
//! Receipt JSON to stdout (the caller saves it; never committed as bulk).

use std::path::PathBuf;
use std::time::Instant;

use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;

const SAMPLES: usize = 30;
const GATE_MS: u64 = 2_000;
const CALLERS: usize = 24;
const RT_CAP: &str = "ac.runtime.text.v1";

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("a2x-rt-formal-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------------------
// The RECORD load: W.make(n) builds a Str of scalar length 1 + 2n ("x"
// then n appends of "yz"); W.size() reports its byte length. Caller cI(k)
// = size + k with n = k + I + 1; main sums cI(I+1).
// ---------------------------------------------------------------------------

fn record_load() -> String {
    let mut src = String::new();
    src.push_str("type W { label Str\n    n int\n");
    src.push_str("    static fn make(n int) W {\n");
    src.push_str("        var s Str = \"x\"\n");
    src.push_str("        var i int = 0\n");
    src.push_str("        loop {\n            if i - n >= 0 { break }\n            s = s + \"yz\"\n            i = i + 1\n        }\n");
    src.push_str("        return W(s.move, n)\n    }\n");
    src.push_str("    fn size() int { return .label.len_bytes() + .n }\n}\n");
    for i in 0..CALLERS {
        src.push_str(&format!(
            "fn c{i}(k int) int {{\n    let w W = W.make(k + {}/{one} + 1)\n    return w.size() + k\n}}\n",
            i,
            one = 1
        ));
    }
    src.push_str("fn main() int {\n    return ");
    let terms: Vec<String> = (0..CALLERS).map(|i| format!("c{i}({})", i + 1)).collect();
    src.push_str(&terms.join(" + "));
    src.push_str("\n}\n");
    src
}

/// Closed-form expectation of the record load's main (computed from the
/// generation parameters alone — the independent oracle): cI(k) with
/// n = k+I+1 returns byte_len("x" + n*"yz") + n + k = (1 + 2n) + n + k.
fn record_expected() -> i64 {
    (0..CALLERS as i64)
        .map(|i| {
            let k = i + 1;
            let n = k + i + 1;
            (1 + 2 * n) + n + k
        })
        .sum()
}

// ---------------------------------------------------------------------------
// The TEXT load (type-free, warm-fragment path): the edit target `tgt`
// owns two Strs, replaces one, and returns a length arithmetic; revision r
// varies the literal length L(r) = 8 + r and the multiplier stays fixed.
// ---------------------------------------------------------------------------

fn text_load_base() -> String {
    text_load_rev(0)
}

fn text_load_rev(r: u32) -> String {
    let lit = "a".repeat(8 + r as usize);
    format!(
        "fn tgt(k int) int {{\n    let s Str = \"{lit}\"\n    let t Str = s.clone()\n    var u Str = t.move\n    u = u + \"中\"\n    return u.len() * k + s.len_bytes()\n}}\nfn main() int {{\n    let base Str = \"anchor\"\n    return tgt(3) * 2 + base.len_bytes()\n}}\n"
    )
}

fn tgt_fragment(rev: u32) -> String {
    let lit = "a".repeat(8 + rev as usize);
    format!(
        "fn tgt(k int) int {{\n    let s Str = \"{lit}\"\n    let t Str = s.clone()\n    var u Str = t.move\n    u = u + \"中\"\n    return u.len() * k + s.len_bytes()\n}}"
    )
}

/// Closed-form: L = 8 + r; u.len = L+1 scalars; s.len_bytes = L;
/// tgt(3) = 3(L+1) + L = 4L+3; main = 2(4L+3) + 6 ("anchor" = 6 bytes).
fn text_expected(r: u32) -> i64 {
    let l = 8 + r as i64;
    2 * (4 * l + 3) + 6
}

/// REAL rustc compile+run of an emitted assembly; the observed value line.
fn certify_assembly(asm: &ace_a2x_store::Assembly, tag: &str) -> String {
    let dir = tmp(&format!("cert-{}", tag));
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let exe = dir.join("a2x_rt_formal.exe");
    let mut owned_compiler = std::process::Command::new(&rustc);
    owned_compiler
        .args(["--edition", "2021", "-O", "-C", "debug-assertions=off", "-o"])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    let out = auto_tool_execution::run_command_output(owned_compiler).expect("spawn rustc");
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
    let rec_src = record_load();
    let t_all = Instant::now();
    let caps = [RT_CAP.to_string()];

    // ---- pre-certification of every distinct revision (rustc -O + run) ----
    let cert_root = tmp("cert-store");
    let mut cert_ok = 0u32;
    {
        let mut s =
            A2xSession::open(&cert_root, "a2x-rt-formal", "formal", 128 * 1024 * 1024).unwrap();
        let asm = s
            .cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main", false)
            .unwrap();
        let line = certify_assembly(&asm, "rec-0");
        assert_eq!(line, format!("value:{}", record_expected()));
        cert_ok += 1;
        let asm = s
            .cold_assemble_profile(&text_load_base(), SourceProfile::TextRecord, &caps, "main", false)
            .unwrap();
        let line = certify_assembly(&asm, "txt-0");
        assert_eq!(line, format!("value:{}", text_expected(0)));
        cert_ok += 1;
        for r in 1..=SAMPLES as u32 {
            let asm = s
                .warm_edit("tgt", &tgt_fragment(r), "main", false)
                .unwrap_or_else(|e| panic!("cert edit r{r}: {e:?}"));
            let line = certify_assembly(&asm, &format!("txt-{r}"));
            assert_eq!(line, format!("value:{}", text_expected(r)));
            cert_ok += 1;
        }
    }
    let _ = std::fs::remove_dir_all(&cert_root);

    // ---- warmup (untimed) ----
    {
        let w = tmp("warmup");
        let mut s = A2xSession::open(&w, "a2x-rt-formal", "formal", 128 * 1024 * 1024).unwrap();
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main", false)
            .unwrap();
        s.restart_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main")
            .unwrap();
        let _ = std::fs::remove_dir_all(&w);
    }

    // ---- fresh x30 (record load, whole path) ----
    let mut fresh: Vec<u64> = Vec::new();
    for i in 0..SAMPLES {
        let dir = tmp(&format!("fresh-{i}"));
        let mut s = A2xSession::open(&dir, "a2x-rt-formal", "formal", 128 * 1024 * 1024).unwrap();
        let t = Instant::now();
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main", false)
            .unwrap();
        fresh.push(t.elapsed().as_millis() as u64);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- hit x30 (same session, zero production) ----
    let hit_root = tmp("hit");
    let mut s = A2xSession::open(&hit_root, "a2x-rt-formal", "formal", 128 * 1024 * 1024).unwrap();
    s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main", false)
        .unwrap();
    // residents after cold: measure via the first hit-state reuse count
    let lower_before_hit = s.stats().lower;
    s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main", false).unwrap();
    let fns_total = s.stats().fragments_reused_resident;
    assert_eq!(s.stats().lower, lower_before_hit, "hit probe produced nothing");
    let lower_after_cold = s.stats().lower;
    let mut hit: Vec<u64> = Vec::new();
    for _ in 0..SAMPLES {
        let t = Instant::now();
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main", false)
            .unwrap();
        hit.push(t.elapsed().as_millis() as u64);
    }
    assert_eq!(
        s.stats().lower, lower_after_cold,
        "hit state produced nothing new (all residents reused)"
    );
    let hit_store_bytes = s.space().total;
    let _ = std::fs::remove_dir_all(&hit_root);

    // ---- restart x30 (new session, same store — store hits only) ----
    let rs_root = tmp("restart");
    {
        let mut s =
            A2xSession::open(&rs_root, "a2x-rt-formal", "formal", 128 * 1024 * 1024).unwrap();
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main", false)
            .unwrap();
    }
    let mut restart: Vec<u64> = Vec::new();
    let mut restart_store_bytes = 0u64;
    for i in 0..SAMPLES {
        let mut s =
            A2xSession::open(&rs_root, "a2x-rt-formal", "formal", 128 * 1024 * 1024).unwrap();
        let t = Instant::now();
        s.restart_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps, "main")
            .unwrap();
        restart.push(t.elapsed().as_millis() as u64);
        if i == 0 {
            assert_eq!(
                s.stats().lower, 0,
                "restart re-admits by pure store hits (zero lowering)"
            );
            restart_store_bytes = s.space().total;
        }
    }
    let _ = std::fs::remove_dir_all(&rs_root);

    // ---- body-edit x30 distinct revisions (warm fragment path) ----
    let ed_root = tmp("edit");
    let mut s = A2xSession::open(&ed_root, "a2x-rt-formal", "formal", 128 * 1024 * 1024).unwrap();
    s.cold_assemble_profile(&text_load_base(), SourceProfile::TextRecord, &caps, "main", false)
        .unwrap();
    let mut edits: Vec<u64> = Vec::new();
    for r in 1..=SAMPLES as u32 {
        let frag = tgt_fragment(r);
        let t = Instant::now();
        s.warm_edit("tgt", &frag, "main", false)
            .unwrap_or_else(|e| panic!("timed edit r{r}: {e:?}"));
        edits.push(t.elapsed().as_millis() as u64);
    }
    let edit_store_bytes = s.space().total;
    let _ = std::fs::remove_dir_all(&ed_root);

    // ---- gates (P90 <= 2000ms binds the warm paths) ----
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
    let mut fresh_sorted = fresh.clone();
    fresh_sorted.sort_unstable();
    let gates = serde_json::json!({
        "hit_p90_le_2000": gate(&hit),
        "restart_p90_le_2000": gate(&restart),
        "body_edit_p90_le_2000": gate(&edits),
        "fresh_p90_ms_reported_separately": nearest_rank(&fresh_sorted, 90),
    });
    let all_pass = gate(&hit) && gate(&restart) && gate(&edits);

    let receipt = serde_json::json!({
        "schema": "a2x-rt-formal-sampling/1",
        "load": {
            "record_bytes": rec_src.len(),
            "record_functions": fns_total,
            "record_callers": CALLERS,
            "profile": "text-record",
            "record_source_sha256": ace_a2x_core::sha256::sha256_hex(rec_src.as_bytes()),
            "text_edit_target": "tgt",
            "text_revisions": SAMPLES + 1,
        },
        "pre_certified_revisions": cert_ok,
        "samples_per_state": SAMPLES,
        "states": {
            "fresh": stats(&fresh),
            "hit": stats(&hit),
            "restart": stats(&restart),
            "body_edit": stats(&edits),
        },
        "budget_ledger": {
            "store_namespace_bytes_hit_state": hit_store_bytes,
            "store_namespace_bytes_restart_state": restart_store_bytes,
            "store_namespace_bytes_edit_state": edit_store_bytes,
            "quota_bytes": 128 * 1024 * 1024,
            "note": "compile/run walls are PRE-timed (certification never inside a sample); store totals are the 783 namespace accounting",
        },
        "gates": gates,
        "all_warm_gates_pass": all_pass,
        "total_wall_s": t_all.elapsed().as_secs(),
    });
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    std::process::exit(if all_pass { 0 } else { 1 });
}
