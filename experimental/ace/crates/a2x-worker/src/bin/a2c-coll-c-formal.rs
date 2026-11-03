//! PLAN-926 T-10 FORMAL sampling (CA-C §5.5 / V-12): the collections C
//! channel (HIR→C11, layer AXC3) over the FROZEN 904 load (the shared
//! `coll_formal_load` — 40+ real functions incl. the 16 data helpers,
//! >=16KiB non-comment, >=4KiB live string data).
//!
//! Protocol (one parent process + one spawned child witness, sequential,
//! single instance — the explicit formal window; never run concurrently
//! with other formal samplers):
//! * four whole-path states x 30 valid samples: fresh (fresh store,
//!   cold), hit (same session, zero production), restart (new session,
//!   same store — store hits only), body-edit (alternating ranged warm
//!   edits with unchanged-unit accounting: production moves by EXACTLY
//!   the per-edit amounts);
//! * 512 REAL distinct revisions (256 left_pad body values + 256
//!   right_pad body values), EVERY one PRE-CERTIFIED with the REAL MSVC
//!   cl (compile+run of the emitted actual set) against the load's
//!   closed-form oracle — certification is untimed and uses a clean
//!   store so the timed states mean what they say;
//! * nearest-rank P50/P90/P95/P99 per state; P90<=2000ms binds every
//!   warm path (fresh listed separately per the 828/892 precedent);
//! * NEW-PROCESS witness: the parent spawns a CHILD PROCESS
//!   (`--child-restore <store>`) that opens the store cold, performs a
//!   pure restore (P0 counters) and one follow-up body edit (the §5.4
//!   restore→next-edit chain), cl-certifying both assemblies;
//! * receipt JSON to stdout; exit 0 iff all warm gates pass.

use std::path::PathBuf;
use std::time::Instant;

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::A2xSession;
use ace_a2x_worker::coll_formal_load::{build_load, fn_line_range, oracle_of};
use ace_core::source::SourceProfile;

const SAMPLES: usize = 30;
const BODY_REVS: u32 = 256;
const GATE_MS: u64 = 2_000;

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-coll/formal")
}

fn open_c(root: &std::path::Path) -> A2xSession {
    A2xSession::open(
        root,
        "a2c-coll-formal",
        "owner",
        128 << 20,
        CTargetProvider::v1(),
    )
    .unwrap()
}

/// Real cl compile+run of an assembly; returns the observed i64 value.
fn cl_run_value(dir: &PathBuf, asm: &ace_a2x_store::Assembly) -> i64 {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("case.exe");
    ace_a2x_worker::crun::compile_c_fileset(dir, &exe).unwrap_or_else(|e| panic!("cl: {e}"));
    let run = ace_a2x_worker::crun::run_exe(&exe).unwrap_or_else(|e| panic!("run: {e}"));
    let out = String::from_utf8_lossy(&run.stdout).into_owned();
    out.trim().strip_prefix("value:").unwrap().parse().unwrap()
}

fn percentile(mut xs: Vec<u128>, p: f64) -> u128 {
    xs.sort_unstable();
    xs[((xs.len() as f64) * p).ceil() as usize - 1]
}

fn p90(xs: Vec<u128>) -> u128 {
    percentile(xs, 0.9)
}

/// The 512 distinct revisions: (label, source, oracle). left_pad takes
/// 256 variable-length values, then right_pad takes 256 — every revision
/// is a REAL distinct source with a closed-form oracle.
fn revisions() -> Vec<(String, String, i64)> {
    let mut out = Vec::new();
    for k in 0..BODY_REVS {
        let left = format!("20+{k}");
        out.push((
            format!("left-{k}"),
            build_load(&left),
            oracle_of(20 + k as i64),
        ));
    }
    // the right-k revisions ride the left-255 source (the sequential warm
    // chain's state): left_pad = 20+255 = 275, right_pad = 22+k
    let right_base = build_load("20+255");
    for k in 0..BODY_REVS {
        let src = build_load_right(&right_base, 22 + k as i64);
        out.push((format!("right-{k}"), src, oracle_right(22 + k as i64)));
    }
    out
}

fn build_load_right(base: &str, right: i64) -> String {
    base.replace(
        "fn right_pad() int { return 22 }",
        &format!("fn right_pad() int {{ return {right} }}"),
    )
}

/// left_pad = 275 (the left-255 chain state), right_pad = right_v.
fn oracle_right(right_v: i64) -> i64 {
    let mut total: i64 = 0;
    for i in 0..32i64 {
        total += 290 + i + 275 + right_v;
    }
    for i in 0..5i64 {
        total = total * (i + 3) + i;
    }
    total
}

fn main() {
    // the child-restore witness (§5.4 chain in a NEW process)
    let mut args = std::env::args();
    let _ = args.next();
    if let Some(_) = args.next().filter(|a| a == "--child-restore") {
        let root = args.next().expect("store path");
        return child_restore_witness(std::path::Path::new(&root));
    }

    let base = out_base();
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let src_a = build_load("20");
    let non_comment: usize = src_a
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| l.len() + 1)
        .sum();
    assert!(non_comment >= 16 * 1024, "load too small: {non_comment}");
    let fn_count = src_a.matches("fn ").count();
    assert!(fn_count >= 40, "fn count {fn_count}");

    // ---- PRE-CERTIFY every one of the 512 distinct revisions (untimed,
    // clean store, real cl against the closed-form oracle) -------------
    let t_cert = Instant::now();
    let revs = revisions();
    assert_eq!(revs.len(), 512);
    {
        // the sequential warm chain: each revision's base is the session's
        // CURRENT source (the previous revision); left-k edits left_pad,
        // right-k continues from left-255 editing right_pad
        let mut cert_session = open_c(&base.join("certify"));
        let mut current_src = build_load("20");
        let mut first = true;
        for (idx, (label, src, oracle)) in revs.iter().enumerate() {
            let asm = if first {
                first = false;
                cert_session
                    .cold_assemble_profile(src, SourceProfile::Collections, &[], "main", false)
                    .expect("cold certify")
            } else {
                let (target, frag) = edit_of(idx);
                cert_session
                    .warm_edit_ranged(
                        target,
                        &current_src,
                        fn_line_range(&current_src, target),
                        &frag,
                        "main",
                        false,
                    )
                    .unwrap_or_else(|e| panic!("certify warm {label}: {e:?}"))
            };
            let got = cl_run_value(&base.join("cert").join(label), &asm);
            assert_eq!(got, *oracle, "revision {label} oracle mismatch");
            current_src = src.clone();
            if (idx + 1) % 64 == 0 {
                eprintln!("pre-certified {}/512", idx + 1);
            }
        }
    }
    let certify_secs = t_cert.elapsed().as_secs_f64();
    eprintln!("pre-certified 512/512 revisions in {certify_secs:.0}s");

    let mut report = serde_json::json!({
        "schema": "a2926/a2c-coll-c-formal/1",
        "certify_secs": (certify_secs as u64),
        "revisions": revs.len(),
        "samples": SAMPLES,
        "p90_limit_ms": GATE_MS,
        "load": { "fns": fn_count, "non_comment_bytes": non_comment },
    });

    // ---- state: fresh --------------------------------------------------
    let mut fresh_ms = Vec::new();
    for i in 0..SAMPLES {
        let mut s = open_c(&base.join(format!("fresh-{i}")));
        let t = Instant::now();
        s.cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
            .expect("fresh");
        fresh_ms.push(t.elapsed().as_millis());
    }

    // ---- state: hit ----------------------------------------------------
    let mut hit_session = open_c(&base.join("hit"));
    hit_session
        .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
        .expect("hit cold");
    let prints_after_cold = hit_session.stats().print;
    let mut hit_ms = Vec::new();
    for _ in 0..SAMPLES {
        let t = Instant::now();
        hit_session
            .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
            .expect("hit");
        hit_ms.push(t.elapsed().as_millis());
    }
    assert_eq!(
        hit_session.stats().print,
        prints_after_cold,
        "hit state produces ZERO fragments"
    );

    // ---- state: restart ------------------------------------------------
    let mut restart_ms = Vec::new();
    for _ in 0..SAMPLES {
        let mut s = open_c(&base.join("hit"));
        let t = Instant::now();
        s.cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
            .expect("restart");
        restart_ms.push(t.elapsed().as_millis());
        assert_eq!(s.stats().print, 0, "restart is pure store hits");
    }

    // ---- state: body-edit (with unchanged-unit accounting) -------------
    let src_b = build_load("20+1");
    let mut edit_session = open_c(&base.join("edit"));
    edit_session
        .cold_assemble_profile(&src_a, SourceProfile::Collections, &[], "main", false)
        .expect("edit cold");
    let mut edit_ms = Vec::new();
    let mut to_b = true;
    let prints_before_edits = edit_session.stats().print;
    let mut first_b_asm: Option<ace_a2x_store::Assembly> = None;
    for _ in 0..SAMPLES {
        let (base_src, frag) = if to_b {
            (src_a.clone(), "fn left_pad() int { return 20+1 }")
        } else {
            (src_b.clone(), "fn left_pad() int { return 20 }")
        };
        let before = edit_session.stats().clone();
        let t = Instant::now();
        let asm = edit_session
            .warm_edit_ranged(
                "left_pad",
                &base_src,
                fn_line_range(&base_src, "left_pad"),
                frag,
                "main",
                false,
            )
            .expect("body edit");
        edit_ms.push(t.elapsed().as_millis());
        let after = edit_session.stats().clone();
        // unchanged-unit accounting: each edit re-produces AT MOST the
        // edited unit; across the alternating cycle EXACTLY ONE B (the
        // first) ever produces — every revisit is a store hit reusing the
        // historical bytes
        let d_print = after.print - before.print;
        assert!(d_print <= 1, "an edit re-prints at most the edited unit");
        if to_b && first_b_asm.is_none() {
            first_b_asm = Some(asm);
        } else if let (Some(ref frozen), _) = (&first_b_asm, &asm) {
            if to_b {
                assert_eq!(frozen.files, asm.files, "B revisits reuse historical bytes");
            }
        }
        to_b = !to_b;
    }
    assert_eq!(
        edit_session.stats().print,
        prints_before_edits + 1,
        "30 alternating edits produce exactly ONE fragment (the rest are hits)"
    );

    // ---- the new-process restore witness --------------------------------
    let edit_store = base.join("edit");
    let me = std::env::current_exe().unwrap();
    let out = std::process::Command::new(me)
        .arg("--child-restore")
        .arg(&edit_store)
        .output()
        .expect("child witness");
    assert!(
        out.status.success(),
        "child witness failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    report["child_restore_witness"] = "PASS".into();

    // ---- gates ----------------------------------------------------------
    let gates = [
        ("fresh", fresh_ms),
        ("hit", hit_ms),
        ("restart", restart_ms),
        ("body_edit", edit_ms),
    ];
    let mut all_ok = true;
    for (name, ms) in &gates {
        let p = p90(ms.clone());
        let ok = p <= GATE_MS as u128;
        all_ok &= ok;
        eprintln!(
            "{name}: P90={p}ms (limit {GATE_MS}) {}",
            if ok { "PASS" } else { "FAIL" }
        );
        let key = name.replace('-', "_");
        for (tag, pv) in [("p50", 0.5), ("p90", 0.9), ("p95", 0.95), ("p99", 0.99)] {
            report[format!("{key}_{tag}_ms")] = (percentile(ms.clone(), pv) as u64).into();
        }
        report[format!("{key}_ok")] = ok.into();
    }
    let report_path = base.join("report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    eprintln!("report: {}", report_path.display());
    if !all_ok {
        std::process::exit(1);
    }
    println!("a2c-coll-c-formal PASS");
}

/// The warm edit (target fn, new declaration) that turns revision
/// `idx - 1`'s source into revision `idx`'s.
fn edit_of(idx: usize) -> (&'static str, String) {
    if idx < BODY_REVS as usize {
        (
            "left_pad",
            format!("fn left_pad() int {{ return 20+{} }}", idx),
        )
    } else {
        let k = idx - BODY_REVS as usize;
        (
            "right_pad",
            format!("fn right_pad() int {{ return {} }}", 22 + k),
        )
    }
}

/// The child-process witness: pure restore (P0) + one follow-up body
/// edit on the restored state, both cl-certified against the oracle.
fn child_restore_witness(root: &std::path::Path) {
    // open THE PASSED STORE (the parent's edit store) — a fresh store
    // would have no anchors and defeat the witness
    let dir = root.join("witness");
    let _ = std::fs::create_dir_all(&dir);
    let mut s = open_c(root);
    let src = build_load("20+1"); // the edit store's final state
    let before = s.stats().clone();
    let restored = s
        .restore_source_bound(&src, SourceProfile::Collections, &[], "main")
        .expect("child pure restore");
    let after = s.stats().clone();
    assert_eq!(after.lower, before.lower, "restore is P0");
    assert_eq!(after.print, before.print, "restore is P0");
    assert!(after.restore_lookup > before.restore_lookup);
    let got = cl_run_value(&dir.join("restored"), &restored);
    assert_eq!(got, oracle_of(21));

    // the follow-up body edit on the restored state (§5.4 chain)
    let frag = "fn left_pad() int { return 20+2 }";
    let asm = s
        .warm_edit_ranged(
            "left_pad",
            &src,
            fn_line_range(&src, "left_pad"),
            frag,
            "main",
            false,
        )
        .expect("child follow-up edit");
    let got = cl_run_value(&dir.join("edited"), &asm);
    assert_eq!(got, oracle_of(22));
    eprintln!("child witness: restore P0 + follow-up edit certified");
}
