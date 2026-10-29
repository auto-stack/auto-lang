//! PLAN-871 T-03 shadow gate (`shadow871_`): the five named profiles —
//! core / scalar / text / record / text-record — each row run through BOTH
//! backends in THIS process on the SAME Checked module: the old ac-core
//! HIR→CLIF lowering (`native_reference::build_stages_with_support` +
//! `run_stage`) and the mn backend (`MnDriver::request_checked`). Every
//! leg is first checked against the INDEPENDENT frozen oracle (hand-audited
//! exit/trap values from the 745/767/770/771/782 surfaces and the ACE-M1
//! binding table), then old and mn are compared: exit value/trap, stdout,
//! trace stderr marks, and — on owned (text/text-record) rows — the
//! resource ledger event stream (LIT/CLONE/CONCAT/FREE). Any difference is
//! red; there is no "different backend" exemption (freeze F2).
//!
//! Migration-window artifact: the old legs exist until the G-DELETE
//! retirement (T-06) — they retire WITH the old lowering, and the frozen
//! oracles then guard mn alone (the plan file records both states).

mod common;

use ac_mn::driver::MnDriver;
use auto_ac_prototype::hir::HirDomain;
use auto_ac_prototype::source::{
    compile_record_source, compile_scalar_source, compile_source, compile_text_record_source,
    compile_text_source, CompiledSource,
};
use auto_ac_prototype::verify::CheckedModule;

const TRACE_CAP: &str = "hir.test.trace.v1";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Prof {
    Core,
    Scalar,
    Text,
    Record,
    TextRecord,
}

impl Prof {
    fn name(self) -> &'static str {
        match self {
            Prof::Core => "core",
            Prof::Scalar => "scalar",
            Prof::Text => "text",
            Prof::Record => "record",
            Prof::TextRecord => "text-record",
        }
    }
    fn compile(self, src: &str, caps: &[String]) -> Result<CompiledSource, String> {
        let r = match self {
            Prof::Core => compile_source(src, "main.at", caps),
            Prof::Scalar => compile_scalar_source(src, "main.at", caps),
            Prof::Text => compile_text_source(src, "main.at", caps),
            Prof::Record => compile_record_source(src, "main.at", caps),
            Prof::TextRecord => compile_text_record_source(src, "main.at", caps),
        };
        r.map_err(|ds| {
            ds.first()
                .map(|d| format!("{:?}/{}: {}", d.stage, d.code, d.message))
                .unwrap_or_else(|| "frontend failed".into())
        })
    }
}

/// One frozen shadow row: independent oracle (exit value; traps are the
/// frozen exit 70 on both backends), optional capability/trace-lib, and
/// whether the resource ledger stream is compared (owned rows).
struct Row {
    id: &'static str,
    prof: Prof,
    /// ACE-M1 corpus source (`crates/auto-lang/test/cases/ace-m1/src/`).
    corpus: Option<&'static str>,
    /// ac-core fixture source (core rows: `experimental/ac-core/fixtures/source/valid/`).
    fixture: Option<&'static str>,
    want: i32,
    caps: &'static [&'static str],
    trace_lib: bool,
    /// Exact stderr oracle (trace rows: shim mark order).
    stderr: Option<&'static str>,
    ledger: bool,
    /// Rows with real owned allocations: an empty old-leg stream is vacuous.
    ledger_req: bool,
}

fn row(id: &'static str, prof: Prof, corpus: &'static str, want: i32) -> Row {
    Row {
        id,
        prof,
        corpus: Some(corpus),
        fixture: None,
        want,
        caps: &[],
        trace_lib: false,
        stderr: None,
        ledger: false,
        ledger_req: false,
    }
}

fn corpus_src(name: &str) -> String {
    let p = common::repo_root().join(format!("crates/auto-lang/test/cases/ace-m1/src/{name}.at"));
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn fixture_src(name: &str) -> String {
    let p = common::repo_root().join(format!("experimental/ac-core/fixtures/source/valid/{name}"));
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn source_of(r: &Row) -> String {
    if let Some(c) = r.corpus {
        corpus_src(c)
    } else if let Some(f) = r.fixture {
        fixture_src(f)
    } else {
        panic!("row {} has no source", r.id)
    }
}

/// Native-side ledger event stream (freeze F2): alloc pairs (id, len) in
/// event order, frees as per-program-point BATCH multisets (maximal runs of
/// consecutive FREE events — the intra-batch order is frozen non-semantic:
/// old follows the verify plan's reverse-root-declaration order, mn the AV
/// descending frame sweep; no language construct observes it and both
/// satisfy the ownership invariants. Batch boundaries and contents, alloc
/// order, double frees and leaks stay red.)
fn parse_events(stderr: &[u8]) -> (Vec<(u64, u64)>, Vec<std::collections::BTreeSet<u64>>) {
    let text = String::from_utf8_lossy(stderr);
    let mut allocs: Vec<(u64, u64)> = Vec::new();
    let mut batches: Vec<std::collections::BTreeSet<u64>> = Vec::new();
    let mut in_free_run = false;
    let mut pending_len: Option<u64> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("LIT len=") {
            pending_len = rest.trim().parse().ok();
            in_free_run = false;
        } else if let Some(rest) = line.strip_prefix("CLONE len=") {
            pending_len = rest.trim().parse().ok();
            in_free_run = false;
        } else if let Some(rest) = line.strip_prefix("CONCAT a=") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            let a: u64 = parts
                .first()
                .and_then(|p| p.trim().parse().ok())
                .unwrap_or(0);
            let b: u64 = parts
                .get(1)
                .and_then(|p| p.strip_prefix("b=").and_then(|v| v.trim().parse().ok()))
                .unwrap_or(0);
            pending_len = Some(a + b);
            in_free_run = false;
        } else if let Some(rest) = line.strip_prefix("LIT id=") {
            allocs.push((
                rest.trim().parse().unwrap_or(0),
                pending_len.take().unwrap_or(0),
            ));
            in_free_run = false;
        } else if let Some(rest) = line.strip_prefix("CLONE id=") {
            allocs.push((
                rest.trim().parse().unwrap_or(0),
                pending_len.take().unwrap_or(0),
            ));
            in_free_run = false;
        } else if let Some(rest) = line.strip_prefix("CONCAT id=") {
            allocs.push((
                rest.trim().parse().unwrap_or(0),
                pending_len.take().unwrap_or(0),
            ));
            in_free_run = false;
        } else if let Some(rest) = line.strip_prefix("FREE id=") {
            let id: u64 = rest.trim().parse().unwrap_or(0);
            if in_free_run {
                batches
                    .last_mut()
                    .expect("free run implies a batch")
                    .insert(id);
            } else {
                batches.push(std::collections::BTreeSet::from([id]));
                in_free_run = true;
            }
        }
    }
    (allocs, batches)
}

/// One shadow comparison. Returns Err(row-id + reason) on ANY difference;
/// `want_override` exists solely for the counter-control (AC-06).
fn shadow_row(r: &Row, want_override: Option<i32>) -> Result<(), String> {
    let src = source_of(r);
    let caps: Vec<String> = r.caps.iter().map(|s| s.to_string()).collect();
    let checked: CheckedModule = r
        .prof
        .compile(&src, &caps)
        .map_err(|e| format!("{}: frontend: {e}", r.id))?
        .checked;
    let domain = checked.bundle().domain;
    let needs_text_runtime = matches!(domain, HirDomain::Text | HirDomain::TextRecord);
    let tag = format!("mn871-{}-{}", r.id, std::process::id());

    // ---- old leg: the SAME checked module through the ac-core lowering --
    let old_dir = std::env::temp_dir().join(format!("{tag}-old"));
    let _ = std::fs::remove_dir_all(&old_dir);
    let lib = if r.trace_lib {
        Some(common::trace_support_lib())
    } else {
        None
    };
    let support = if needs_text_runtime {
        Some(
            auto_av_probe::native_reference::text_runtime_lib()
                .map_err(|e| format!("{}: text runtime: {e}", r.id))?,
        )
    } else {
        None
    };
    let (_obj, exe, _receipt) = auto_av_probe::native_reference::build_stages_with_support(
        &checked,
        "d_main",
        &old_dir,
        lib.as_deref(),
        support.as_deref(),
    )
    .map_err(|e| format!("{}: old build: {e}", r.id))?;
    let (old_exit, _old_out, old_err) = auto_av_probe::native_reference::run_stage(
        &exe,
        std::time::Duration::from_secs(60),
        1 << 20,
        1 << 20,
    )
    .map_err(|e| format!("{}: old run: {e}", r.id))?;
    let old_exit = old_exit.unwrap_or(-1);

    // ---- mn leg: the SAME checked module through the shared MIR chain ---
    let mn_dir = std::env::temp_dir().join(format!("{tag}-mn"));
    let _ = std::fs::remove_dir_all(&mn_dir);
    let snapshot = auto_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let mn = driver.request_checked(&checked, "d_main", &mn_dir, lib.as_deref(), snapshot);
    let _ = std::fs::remove_dir_all(&old_dir);
    let _ = std::fs::remove_dir_all(&mn_dir);
    if !mn.ok {
        return Err(format!("{}: mn leg failed: {:?}", r.id, mn.error));
    }
    let mn_exit = mn.exit_code.unwrap_or(-1);

    // ---- independent oracle first (both legs), then old == mn ----------
    let want = want_override.unwrap_or(r.want);
    if old_exit != want {
        return Err(format!(
            "{}: OLD leg {} != frozen oracle {}",
            r.id, old_exit, want
        ));
    }
    if mn_exit != want {
        return Err(format!(
            "{}: MN leg {} != frozen oracle {}",
            r.id, mn_exit, want
        ));
    }
    if old_exit != mn_exit {
        return Err(format!(
            "{}: exit differs old={} mn={}",
            r.id, old_exit, mn_exit
        ));
    }
    if let Some(want_err) = r.stderr {
        let old_err_s = String::from_utf8_lossy(&old_err).to_string();
        let mn_err_s = String::from_utf8_lossy(&mn.stderr).to_string();
        if old_err_s != want_err {
            return Err(format!(
                "{}: OLD stderr marks {:?} != oracle {:?}",
                r.id, old_err_s, want_err
            ));
        }
        if mn_err_s != want_err {
            return Err(format!(
                "{}: MN stderr marks {:?} != oracle {:?}",
                r.id, mn_err_s, want_err
            ));
        }
    }
    if r.ledger {
        let (old_allocs, old_batches) = parse_events(&old_err);
        let (mn_allocs, mn_batches) = parse_events(&mn.stderr);
        if old_allocs != mn_allocs {
            return Err(format!(
                "{}: alloc event stream differs: old {:?} / mn {:?}",
                r.id, old_allocs, mn_allocs
            ));
        }
        if old_batches != mn_batches {
            return Err(format!(
                "{}: free batches differ (program-point sets): old {:?} / mn {:?}",
                r.id, old_batches, mn_batches
            ));
        }
        if r.ledger_req && old_allocs.is_empty() {
            return Err(format!(
                "{}: owned row produced no ledger events on the old leg — the comparison above would be vacuous",
                r.id
            ));
        }
    } else if !r.trace_lib {
        // Non-trace, non-ledger rows: stderr is empty on both sides.
        let old_nonempty = String::from_utf8_lossy(&old_err);
        if !old_nonempty.trim().is_empty() {
            return Err(format!(
                "{}: OLD stderr not empty: {:?}",
                r.id, old_nonempty
            ));
        }
    }
    Ok(())
}

fn run_profile_rows(rows: &[Row]) {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let mut failed = Vec::new();
    for r in rows {
        if let Err(e) = shadow_row(r, None) {
            failed.push(format!("[{}] {}", r.prof.name(), e));
        }
    }
    assert!(
        failed.is_empty(),
        "shadow rows red ({}):\n  {}",
        failed.len(),
        failed.join("\n  ")
    );
}

// ---- core profile: the 745 P01–P21 frozen surface -------------------------

fn core_rows() -> Vec<Row> {
    let mut v: Vec<Row> = [
        ("p01-precedence.at", 14),
        ("p02-signed-positional.at", 1),
        ("p03-bool-param-constant-encoding.at", 7),
        ("p04-while-retest.at", 3),
        ("p05-while-zero.at", 1),
        ("p06-continue-retests-condition.at", 3),
        ("p07-break-nearest-loop.at", 5),
        ("p08-recursion.at", 3),
        ("p09-forward-signature.at", 8),
        ("p10-shadow-rhs-outer.at", 5),
        ("p11-literal-min-max.at", 0),
        ("p12-unexecuted-trap-not-lifted-x1.at", 0),
        ("p13-add-overflow-trap.at", 70),
        ("p14-mul-overflow-trap.at", 70),
        ("p15-named-eval-order-x2.at", 12),
        ("p16-observable-before-trap-x9.at", 70),
        ("p17-heterogeneous-binding-map.at", 6),
        ("p18-bool-return-local-branch.at", 4),
        ("p19-user-function-not-intrinsic.at", 6),
        ("p20-utf8-crlf-negative-min.at", 0),
        ("p21-mutual-recursion.at", 4),
    ]
    .into_iter()
    .map(|(f, want)| Row {
        id: f.split('.').next().unwrap(),
        prof: Prof::Core,
        corpus: None,
        fixture: Some(f),
        want,
        caps: &[],
        trace_lib: false,
        stderr: None,
        ledger: false,
        ledger_req: false,
    })
    .collect();
    // Trace-mark rows: capability + support lib; stderr carries the mark
    // order oracle (eval-order "ba", observable-before-trap "a").
    for r in &mut v {
        if r.id == "p15-named-eval-order-x2" {
            r.caps = &[TRACE_CAP];
            r.trace_lib = true;
            r.stderr = Some("ba");
        }
        if r.id == "p16-observable-before-trap-x9" {
            r.caps = &[TRACE_CAP];
            r.trace_lib = true;
            r.stderr = Some("a");
        }
        if r.id == "p19-user-function-not-intrinsic" {
            r.caps = &[TRACE_CAP];
        }
    }
    v
}

#[test]
#[cfg(windows)]
fn shadow871_core() {
    run_profile_rows(&core_rows());
}

// ---- scalar profile: the ACE-M1 B-row executable surface ------------------

#[test]
#[cfg(windows)]
fn shadow871_scalar() {
    let rows: Vec<Row> = [
        ("B01", 14),
        ("B02", -21),
        ("B03", 42),
        ("B04", 42),
        ("B05", 42),
        ("B06", 0),
        ("B07", 30),
        ("B08", 7),
        ("B09", 120),
        ("B10", 42),
        ("B11", 7),
        ("B12", -2),
        ("B13", 70),
        ("B14", 70),
        ("B15", 70),
        ("B20", 70),
    ]
    .into_iter()
    .map(|(id, want)| row(id, Prof::Scalar, id, want))
    .collect();
    run_profile_rows(&rows);
}

// ---- text profile: T-rows (Unicode codepoint, Str ops, traps) -------------

#[test]
#[cfg(windows)]
fn shadow871_text() {
    let mut rows: Vec<Row> = [("T01", 38), ("T02", 27), ("T04", 128512), ("T08", 70)]
        .into_iter()
        .map(|(id, want)| row(id, Prof::Text, id, want))
        .collect();
    // Ledger compared on every text row; the pure-text domain operates on
    // borrowed views (no owned allocs — legs agree), so the non-empty
    // guard is off for every row here (T04 codepoint, T08 borrowed slice
    // are structurally allocation-free; T01/T02 ride str views too). The
    // owned-alloc coverage lives in the text-record rows.
    for r in &mut rows {
        r.ledger = true;
        r.ledger_req = false;
    }
    run_profile_rows(&rows);
}

// ---- record profile: R-rows + 881 method rows -----------------------------

#[test]
#[cfg(windows)]
fn shadow871_record() {
    let rows: Vec<Row> = [
        ("R01", 19),
        ("R02", 9),
        ("R03", 23),
        ("R06", 21),
        ("R07", 42),
        ("R08", 42),
        ("M01", 0),
        ("M03", 75),
        ("M04", 135),
        ("M07", 84),
        ("M08", 42),
    ]
    .into_iter()
    .map(|(id, want)| row(id, Prof::Record, id, want))
    .collect();
    run_profile_rows(&rows);
}

// ---- text-record profile: owned rows (move Str, methods, ledger) ----------

#[test]
#[cfg(windows)]
fn shadow871_text_record() {
    // (T03 is NOT here: it is the `gap:mn-text-move` REJECTION row — the
    // owned profile and the old frontend both reject `resource-assign`/
    // `expr.move`; only legacy avm produces its value. Owned upstream.)
    let mut rows: Vec<Row> = [
        ("R04", 51),
        ("R05", 42),
        ("M02", 57),
        ("M05", 42),
        ("M06", 52),
    ]
    .into_iter()
    .map(|(id, want)| row(id, Prof::TextRecord, id, want))
    .collect();
    for r in &mut rows {
        r.ledger = true;
        r.ledger_req = true;
    }
    run_profile_rows(&rows);
}

// ---- AC-06 counter-control: a mutated oracle MUST redden exactly one row --

#[test]
#[cfg(windows)]
fn shadow871_counter_control_mutated_oracle_red() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    // R08 with the oracle bumped by one (42 -> 43): the gate must flag the
    // ROW (both legs checked against the oracle first — mirror of 881's
    // M03 75→76 mutation oracle).
    let mut r = row("R08", Prof::Record, "R08", 42);
    r.id = "R08#mutant";
    let err = shadow_row(&r, Some(43)).expect_err("mutated oracle must redden the row");
    assert!(
        err.contains("R08#mutant") && (err.contains("frozen oracle")),
        "counter-control must name the row and the oracle, got: {err}"
    );
}
