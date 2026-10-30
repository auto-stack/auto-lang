//! PLAN-894 T-07 V-09 formal measurement: the four §5.4 states × 30 full
//! valid formal samples each on the owned-source native session, release
//! gate P90 ≤ 2000ms per state (sequential order statistics, sample count
//! and window unchanged from the frozen contract). Cold tool build is
//! single-listed OUTSIDE the samples (the cargo build precedes this
//! binary); each sample is the CONTINUOUS request-entry → real-PE-result
//! interval. Structure gates ride along: untouched-unit production stays
//! zero on body edits and every state's PE result is judged against the
//! independent oracle (65/66 variants).

mod common;

use ac_mn::owned_source_incremental::OwnedSourceMnSession;
use std::time::Instant;

const SUPPLEMENT: &str = "\
fn text_job() int {
    let a Str = \"ab\"
    let b Str = a.clone()
    var c Str = b.move
    c = c + \"中\"
    return a.len() * 10 + c.len()
}
fn cmp_job() int {
    let a str = \"A\\0B\"
    if a == \"A\" { return 1 }
    if a != \"A\\0B\" { return 2 }
    return 42
}
fn disjoint() int { return 100 }
fn main() int { return text_job() + cmp_job() }
";

const SAMPLES: usize = 30;
const P90_LIMIT_MS: f64 = 2000.0;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn report(state: &str, ms: &mut Vec<f64>) {
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    eprintln!(
        "{state}: n={} P50={:.0}ms P90={:.0}ms P95={:.0}ms P99={:.0}ms max={:.0}ms",
        ms.len(),
        percentile(ms, 0.50),
        percentile(ms, 0.90),
        percentile(ms, 0.95),
        percentile(ms, 0.99),
        ms.last().copied().unwrap_or(0.0),
    );
}

fn fresh_root(tag: &str, i: usize) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("mn894-perf-{tag}-{i}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
#[cfg(windows)]
fn owned_894_v09_four_state_30_samples_p90_gate() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — not run");
        return;
    }
    // ---- state 1: fresh (cold session per sample) --------------------
    let mut fresh: Vec<f64> = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let root = fresh_root("fresh", i);
        let t = Instant::now();
        let (s, obs) =
            OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &root).expect("warmup sample");
        let w = t.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(obs.exit_code, Some(65), "fresh sample {i}");
        let _ = s;
        fresh.push(w);
        let _ = std::fs::remove_dir_all(&root);
    }
    report("fresh", &mut fresh);

    // One long-lived session for hit / body-edit / restart.
    let live_root = fresh_root("live", 0);
    let (mut s, obs) =
        OwnedSourceMnSession::warmup(SUPPLEMENT, "d_main", &[], &live_root).expect("live warmup");
    assert_eq!(obs.exit_code, Some(65));

    // ---- state 2: same-identity hit (zero production, real PE) --------
    let mut hit: Vec<f64> = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let t = Instant::now();
        let o = s.rerun();
        let w = t.elapsed().as_secs_f64() * 1000.0;
        assert!(o.ok && o.exit_code == Some(65), "hit sample {i}");
        assert_eq!(o.codegen_units, 0, "hit sample {i}: zero COFF production");
        hit.push(w);
    }
    report("hit", &mut hit);

    // ---- state 3: warm body miss (single-body re-production) -----------
    // 30 DISTINCT bodies (`中0`..`中29`) so every sample is a real miss —
    // no content hit may shortcut the measured path. Expected value:
    // 20 + len("ab" + payload) + 42 (len counts unicode scalars).
    let mut edit: Vec<f64> = Vec::with_capacity(SAMPLES);
    let mut expect = 65i32;
    for i in 0..SAMPLES {
        let payload = format!("中{i}");
        expect = 20 + (2 + payload.chars().count() as i32) + 42;
        let cur = s.src();
        let cur_pos = cur.find("c + \"中").expect("needle in current");
        let cur_byte = cur[..cur_pos].len() + "c + \"".len();
        let end = cur[cur_byte..]
            .find('"')
            .map(|k| cur_byte + k)
            .expect("close quote");
        let t = Instant::now();
        let o = s.edit(cur_byte as u64, end as u64, payload.as_bytes().to_vec());
        let w = t.elapsed().as_secs_f64() * 1000.0;
        assert!(o.ok, "edit sample {i}: {:?}", o.refused_reason);
        assert_eq!(o.exit_code, Some(expect), "edit sample {i}");
        assert_eq!(o.codegen_units, 1, "edit sample {i}: exactly one COFF");
        assert_eq!(
            o.whole_module_stage_delta, 0,
            "edit sample {i}: no whole work"
        );
        // structure gate: untouched units never re-produce (parse = rounds)
        for name in s.unit_names() {
            if name != "d_text_job" {
                let rounds = s.production().get(name).map(|p| p.parse).unwrap_or(0);
                assert_eq!(rounds, 1, "edit sample {i}: {name} untouched");
            }
        }
        edit.push(w);
    }
    report("body-edit", &mut edit);

    // ---- state 4: restart (decode + verify + re-link + run) ------------
    // The last edit already persisted the manifest; drop the session and
    // restore (re-derives every unit fingerprint through its production
    // path, verifies every payload hash, re-links, runs) — the restore
    // re-verification is counted in this window and never mixed into
    // warm-edit zeros.
    drop(s);
    let mut restart: Vec<f64> = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let t = Instant::now();
        let (s2, obs) = OwnedSourceMnSession::restore(&live_root).expect("restart sample");
        let w = t.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            obs.exit_code,
            Some(expect),
            "restart sample {i}: the live revision reproduces"
        );
        for name in s2.unit_names() {
            let rounds = s2.production().get(name).map(|p| p.parse).unwrap_or(0);
            assert_eq!(rounds, 0, "restart sample {i}: {name} counters zero");
        }
        drop(s2);
        restart.push(w);
    }
    report("restart", &mut restart);
    let _ = std::fs::remove_dir_all(&live_root);

    // ---- the release gate: P90 ≤ 2000ms on EVERY state -----------------
    for (state, ms) in [
        ("fresh", &fresh),
        ("hit", &hit),
        ("body-edit", &edit),
        ("restart", &restart),
    ] {
        let p90 = percentile(ms, 0.90);
        assert!(
            p90 <= P90_LIMIT_MS,
            "{state}: P90 {p90:.0}ms exceeds the {P90_LIMIT_MS}ms gate"
        );
    }
}
