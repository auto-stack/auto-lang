//! PLAN-925 T-10 FORMAL sampling (CA-B §5.5 / V-12): the record/text C
//! channel (HIR→C11, layer AXC2) over the 902-shaped resource load.
//!
//! Protocol (one parent process + one spawned child witness, sequential,
//! single instance — the explicit formal window; never run concurrently
//! with other formal samplers):
//! * the RECORD load (a resource record with a static ctor + a view
//!   method plus a 24-caller fleet, real Str data built by loops) drives
//!   the whole-path states: fresh (fresh store, cold), hit (same session,
//!   zero production), restart (new session, same store — store hits
//!   only), restore (new session, pure source-bound restore — P0);
//! * the TEXT load (type-free owned-Str functions with THREE unchanged
//!   anchor units beside the edit target) drives the free-body-edit
//!   state; the record load's view method drives the METHOD-edit state;
//! * 512 REAL distinct revisions (256 free-body + 256 method), EVERY one
//!   PRE-CERTIFIED with the REAL MSVC cl (compile+run of the emitted
//!   actual set) against the generator's INDEPENDENT closed-form oracle
//!   — certification is untimed and uses a clean store so the timed
//!   states mean what they say;
//! * unchanged-unit accounting: across each timed edit state the store's
//!   production counters move by EXACTLY the per-edit amounts (lower/
//!   print/persist), proving the untouched units (24 callers + the
//!   static ctor + main / the three anchors) reused residents;
//! * nearest-rank P50/P90/P95/P99 per state; P90<=2000ms binds every
//!   warm path (fresh reported separately per the 828/892 precedent);
//! * NEW-PROCESS witness: the parent spawns a CHILD PROCESS
//!   (`--child-restore <store>`) that opens the store cold, performs a
//!   pure restore and one follow-up METHOD edit (the §5.4
//!   restore→next-method-edit chain), asserting P0 restore counters and
//!   cl-certifying both assemblies;
//! * receipt JSON to stdout; exit 0 iff all warm gates pass.

use std::path::PathBuf;
use std::time::Instant;

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;

const SAMPLES: usize = 30;
const BODY_REVS: u32 = 256;
const METHOD_REVS: u32 = 256;
const GATE_MS: u64 = 2_000;
const CALLERS: usize = 24;
const RT_CAP: &str = "ac.runtime.text.v1";

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("a2c-rt-formal-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn open(root: &std::path::Path) -> A2xSession {
    A2xSession::open(
        root,
        "a2c-rt-formal",
        "formal",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap()
}

fn caps() -> Vec<String> {
    [RT_CAP.to_string()].to_vec()
}

// ---------------------------------------------------------------------------
// The RECORD load: W.make(n) builds a Str of scalar length 1 + 2n ("x"
// then n appends of "yz"); W.size() reports its byte length. Caller cI(k)
// = size + k with n = k + I + 1; main sums cI(I+1). The view method
// W.size() is the METHOD-edit target (revision m appends `+ m`).
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
            "fn c{i}(k int) int {{\n    let w W = W.make(k + {i} + 1)\n    return w.size() + k\n}}\n"
        ));
    }
    src.push_str("fn main() int {\n    return ");
    let terms: Vec<String> = (0..CALLERS).map(|i| format!("c{i}({})", i + 1)).collect();
    src.push_str(&terms.join(" + "));
    src.push_str("\n}\n");
    src
}

/// The METHOD-edit fragment of revision `m` (m = 0 is the base body).
fn method_frag(m: u32) -> String {
    if m == 0 {
        "fn size() int { return .label.len_bytes() + .n }".to_string()
    } else {
        format!("fn size() int {{ return .label.len_bytes() + .n + {m} }}")
    }
}

/// Closed-form expectation of the record load's main at method revision
/// `m`: cI(k) with n = k+I+1 returns byte_len("x" + n*"yz") + n + m + k
/// = (1 + 2n) + n + m + k; main adds one `+ m` per caller (24 total).
fn record_expected(m: u32) -> i64 {
    (0..CALLERS as i64)
        .map(|i| {
            let k = i + 1;
            let n = k + i + 1;
            (1 + 2 * n) + n + m as i64 + k
        })
        .sum()
}

// ---------------------------------------------------------------------------
// The TEXT load (type-free, free-body-edit path): the edit target `tgt`
// owns two Strs, replaces one, and returns a length arithmetic; revision
// r varies the literal length L(r) = 8 + r. The THREE anchors a1/a2/main
// are the unchanged units of this state.
// ---------------------------------------------------------------------------

fn text_load_rev(r: u32) -> String {
    let lit = "a".repeat(8 + r as usize);
    format!(
        "fn a1(k int) int {{\n    let s Str = \"anchor-one\"\n    return s.len_bytes() + k\n}}\nfn a2(k int) int {{\n    let s Str = \"anchor-two!!\"\n    return s.len_bytes() - k\n}}\nfn tgt(k int) int {{\n    let s Str = \"{lit}\"\n    let t Str = s.clone()\n    var u Str = t.move\n    u = u + \"中\"\n    return u.len() * k + s.len_bytes()\n}}\nfn main() int {{\n    return tgt(3) * 2 + a1(1) * 100 + a2(2)\n}}\n"
    )
}

fn tgt_fragment(rev: u32) -> String {
    let lit = "a".repeat(8 + rev as usize);
    format!(
        "fn tgt(k int) int {{\n    let s Str = \"{lit}\"\n    let t Str = s.clone()\n    var u Str = t.move\n    u = u + \"中\"\n    return u.len() * k + s.len_bytes()\n}}"
    )
}

/// Closed-form: L = 8 + r; u.len = L+1 scalars; s.len_bytes = L;
/// tgt(3) = 3(L+1) + L = 4L+3; main = 2(4L+3) + 11*100 + (12-2)
/// ("anchor-one" = 10 bytes, "anchor-two!!" = 12 bytes).
fn text_expected(r: u32) -> i64 {
    let l = 8 + r as i64;
    2 * (4 * l + 3) + 1100 + 10
}

/// REAL cl compile+run of an emitted C assembly; the observed stdout line.
fn certify_assembly(asm: &ace_a2x_store::Assembly, tag: &str) -> String {
    let dir = tmp(&format!("cert-{}", tag));
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let (out, _) = ace_a2x_worker::crun::compile_and_run_c(&dir, "a2c_rt_formal.exe")
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
/// follow-up METHOD edit, cl-certifying both, printing counters, exit 0.
fn child_restore(store: &str) -> ! {
    let src = record_load();
    let mut s = open(std::path::Path::new(store));
    let t = Instant::now();
    let asm = s
        .restore_source_bound(&src, SourceProfile::TextRecord, &caps(), "main")
        .unwrap_or_else(|e| panic!("child restore: {e:?}"));
    let restore_ms = t.elapsed().as_millis() as u64;
    let st = s.stats().clone();
    assert_eq!(st.lower, 0, "child restore is PURE (P0)");
    // follow-up METHOD edit works after the cross-process restore: the
    // RANGED authorization (the anchor rebinds; env rebuild is M-class)
    let old_body = method_frag(0);
    let new_body = method_frag(1);
    let start = src.find(&old_body).expect("W.size body present");
    let end = start + old_body.len();
    let env_before = s.stats().env_rebuilds;
    let asm2 = s
        .warm_edit_ranged_method("W", "size", &src, (start, end), &new_body, "main", false)
        .unwrap_or_else(|e| panic!("child method-edit-after-restore: {e:?}"));
    assert_eq!(
        s.stats().env_rebuilds,
        env_before + 1,
        "the whole-source env rebuild is counted (M-class)"
    );
    assert_eq!(
        certify_assembly(&asm, "child-restore"),
        format!("value:{}", record_expected(0))
    );
    assert_eq!(
        certify_assembly(&asm2, "child-method-edit"),
        format!("value:{}", record_expected(1))
    );
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

    let rec_src = record_load();
    let t_all = Instant::now();

    // ---- pre-certification of EVERY distinct revision (1 + 256 body +
    // 256 method = 513 cl rounds) against the closed-form oracles ----
    let cert_start = Instant::now();
    let root = tmp("cert-store");
    let mut cert_ok = 0u32;
    {
        // record base + the method chain
        let mut s = open(&root);
        let asm = s
            .cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
            .unwrap();
        assert_eq!(
            certify_assembly(&asm, "rec-0"),
            format!("value:{}", record_expected(0)),
            "record base vs the independent closed-form oracle"
        );
        cert_ok += 1;
        for m in 1..=METHOD_REVS {
            let asm = s
                .warm_edit_method("W", "size", &method_frag(m), "main", false)
                .unwrap_or_else(|e| panic!("cert method edit m{m}: {e:?}"));
            assert_eq!(
                certify_assembly(&asm, &format!("method-{m}")),
                format!("value:{}", record_expected(m)),
                "method revision {m} vs its independent oracle"
            );
            cert_ok += 1;
        }
        // the free-body chain on its own session
        let asm = s
            .cold_assemble_profile(
                &text_load_rev(0),
                SourceProfile::TextRecord,
                &caps(),
                "main",
                false,
            )
            .unwrap();
        assert_eq!(
            certify_assembly(&asm, "txt-0"),
            format!("value:{}", text_expected(0))
        );
        cert_ok += 1;
        for r in 1..=BODY_REVS {
            let asm = s
                .warm_edit("tgt", &tgt_fragment(r), "main", false)
                .unwrap_or_else(|e| panic!("cert body edit r{r}: {e:?}"));
            assert_eq!(
                certify_assembly(&asm, &format!("body-{r}")),
                format!("value:{}", text_expected(r)),
                "body revision {r} vs its independent oracle"
            );
            cert_ok += 1;
        }
    }
    // certification polluted the store with every revision — formal
    // timing uses CLEAN stores so the states mean what they say
    let _ = std::fs::remove_dir_all(&root);
    let cert_wall_s = cert_start.elapsed().as_secs();

    // ---- warmup (untimed) ----
    {
        let w = tmp("warmup");
        let mut s = open(&w);
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
            .unwrap();
        s.restart_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main")
            .unwrap();
        let _ = std::fs::remove_dir_all(&w);
    }

    // ---- fresh x30 (record load, whole path; reported separately) ----
    let mut fresh: Vec<u64> = Vec::new();
    for i in 0..SAMPLES {
        let dir = tmp(&format!("fresh-{i}"));
        let mut s = open(&dir);
        let t = Instant::now();
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
            .unwrap();
        fresh.push(t.elapsed().as_millis() as u64);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- hit x30 (same session, zero production) ----
    let hit_root = tmp("hit");
    let hit_store_bytes;
    let mut hit: Vec<u64> = Vec::new();
    {
        let mut s = open(&hit_root);
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
            .unwrap();
        let lower_after_cold = s.stats().lower;
        for _ in 0..SAMPLES {
            let t = Instant::now();
            s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
                .unwrap();
            hit.push(t.elapsed().as_millis() as u64);
        }
        assert_eq!(
            s.stats().lower,
            lower_after_cold,
            "hit state produced nothing new (all residents reused)"
        );
        hit_store_bytes = s.space().total;
        let _ = std::fs::remove_dir_all(&hit_root);
    }

    // ---- restart x30 (new session, same store — store hits only) ----
    let rs_root = tmp("restart");
    let restart_store_bytes;
    {
        let mut s = open(&rs_root);
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
            .unwrap();
    }
    let mut restart: Vec<u64> = Vec::new();
    {
        restart_store_bytes = {
            let mut s0 = open(&rs_root);
            s0.restart_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main")
                .unwrap();
            assert_eq!(
                s0.stats().lower,
                0,
                "restart re-admits by pure store hits (zero lowering)"
            );
            s0.space().total
        };
        for _ in 0..SAMPLES {
            let mut s = open(&rs_root);
            let t = Instant::now();
            s.restart_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main")
                .unwrap();
            restart.push(t.elapsed().as_millis() as u64);
        }
    }
    let _ = std::fs::remove_dir_all(&rs_root);

    // ---- free-body-edit x30 (of the 256 distinct revisions; the three
    // anchors a1/a2/main are the unchanged units) ----
    let ed_root = tmp("edit");
    let edit_store_bytes;
    let mut edits: Vec<u64> = Vec::new();
    {
        let mut s = open(&ed_root);
        s.cold_assemble_profile(
            &text_load_rev(0),
            SourceProfile::TextRecord,
            &caps(),
            "main",
            false,
        )
        .unwrap();
        let before = s.stats().clone();
        for r in 1..=SAMPLES as u32 {
            let frag = tgt_fragment(r);
            let t = Instant::now();
            s.warm_edit("tgt", &frag, "main", false)
                .unwrap_or_else(|e| panic!("timed body edit r{r}: {e:?}"));
            edits.push(t.elapsed().as_millis() as u64);
        }
        let after = s.stats().clone();
        // unchanged-unit accounting: exactly ONE unit produced per edit
        // (the edited tgt) — lower/print move by 30, persist by 60; the
        // three anchors + everything else reused residents untouched
        assert_eq!(after.lower - before.lower, SAMPLES as u64);
        assert_eq!(after.print - before.print, SAMPLES as u64);
        assert_eq!(after.persist - before.persist, 2 * SAMPLES as u64);
        edit_store_bytes = s.space().total;
        let _ = std::fs::remove_dir_all(&ed_root);
    }

    // ---- method-edit x30 (of the 256 distinct method revisions; the 24
    // callers + the static ctor + main are the unchanged units) ----
    let me_root = tmp("method-edit");
    let method_store_bytes;
    let mut method_edits: Vec<u64> = Vec::new();
    {
        let mut s = open(&me_root);
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
            .unwrap();
        let before = s.stats().clone();
        for m in 1..=SAMPLES as u32 {
            let t = Instant::now();
            s.warm_edit_method("W", "size", &method_frag(m), "main", false)
                .unwrap_or_else(|e| panic!("timed method edit m{m}: {e:?}"));
            method_edits.push(t.elapsed().as_millis() as u64);
        }
        let after = s.stats().clone();
        // exactly ONE unit produced per edit (the edited view method)
        assert_eq!(after.lower - before.lower, SAMPLES as u64);
        assert_eq!(after.print - before.print, SAMPLES as u64);
        assert_eq!(after.persist - before.persist, 2 * SAMPLES as u64);
        method_store_bytes = s.space().total;
        let _ = std::fs::remove_dir_all(&me_root);
    }

    // ---- restore x30 (new session, pure source-bound restore — P0) ----
    let rt_root = tmp("restore");
    {
        let mut s = open(&rt_root);
        s.cold_assemble_profile(&rec_src, SourceProfile::TextRecord, &caps(), "main", false)
            .unwrap();
    }
    let mut restores: Vec<u64> = Vec::new();
    for _ in 0..SAMPLES {
        let mut s = open(&rt_root);
        let t = Instant::now();
        s.restore_source_bound(&rec_src, SourceProfile::TextRecord, &caps(), "main")
            .unwrap();
        restores.push(t.elapsed().as_millis() as u64);
    }

    // ---- NEW-PROCESS witness: child restores the anchored store purely
    // and performs one METHOD edit (asserts P0 + cl-certifies both) ----
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

    // ---- gates (P90 <= 2000ms binds every warm path) ----
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
        "method_edit_p90_le_2000": gate(&method_edits),
        "restore_p90_le_2000": gate(&restores),
        "fresh_p90_ms_reported_separately": nearest_rank(&fresh_sorted, 90),
    });
    let all_pass =
        gate(&hit) && gate(&restart) && gate(&edits) && gate(&method_edits) && gate(&restores);

    let a2x_working_set =
        hit_store_bytes + restart_store_bytes + edit_store_bytes + method_store_bytes;
    let receipt = serde_json::json!({
        "schema": "a2c-rt-formal-sampling/1",
        "target": "c",
        "load": {
            "record_bytes": rec_src.len(),
            "record_callers": CALLERS,
            "record_units": CALLERS + 3, // + W.make + W.size + main
            "profile": "text-record",
            "record_source_sha256": ace_a2x_core::sha256::sha256_hex(rec_src.as_bytes()),
            "text_edit_target": "tgt",
            "text_unchanged_anchors": ["a1", "a2", "main"],
            "method_edit_target": "W.size",
            "body_revisions": BODY_REVS,
            "method_revisions": METHOD_REVS,
            "distinct_revisions_total": BODY_REVS + METHOD_REVS,
        },
        "pre_certified_revisions": cert_ok,
        "certification_wall_s": cert_wall_s,
        "samples_per_state": SAMPLES,
        "states": {
            "fresh": stats(&fresh),
            "hit": stats(&hit),
            "restart": stats(&restart),
            "body_edit": stats(&edits),
            "method_edit": stats(&method_edits),
            "restore": stats(&restores),
        },
        "unchanged_unit_accounting": {
            "body_edit": {"lower": SAMPLES, "print": SAMPLES, "persist": 2 * SAMPLES,
                          "unchanged": ["a1", "a2", "main"]},
            "method_edit": {"lower": SAMPLES, "print": SAMPLES, "persist": 2 * SAMPLES,
                            "unchanged": ["W.make", "c0..c23", "main"]},
        },
        "budget_ledger": {
            "store_namespace_bytes_hit_state": hit_store_bytes,
            "store_namespace_bytes_restart_state": restart_store_bytes,
            "store_namespace_bytes_body_edit_state": edit_store_bytes,
            "store_namespace_bytes_method_edit_state": method_store_bytes,
            "a2x_working_set_bytes_sum": a2x_working_set,
            "a2x_working_set_budget_bytes": 48 * 1024 * 1024,
            "quota_bytes": 128 * 1024 * 1024,
            "note": "compile/run walls are PRE-timed (certification never inside a sample); per-spawn budgets: compile<=120s run<=30s via the frozen crun deadlines",
        },
        "gates": gates,
        "all_warm_gates_pass": all_pass,
        "child_process_restore_witness": child_receipt,
        "total_wall_s": t_all.elapsed().as_secs(),
    });
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    std::process::exit(if all_pass { 0 } else { 1 });
}
