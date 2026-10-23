//! The turnaround bench (PLAN-817 T-07, V-09): eight scenarios x N raw
//! samples through the REAL driver path, with the two EDIT hard gates
//! (helper-body edit and Meta-set edit, warm store, P95 < gate) and a
//! JSONL raw report whose rows bind each sample to the hash of the
//! exact deck text it executed. Native observables are OFF here — the
//! native reference is reported separately (contract-decision §7).
//!
//! F-817-R5-01: every PROCESS scenario's worker gate reads the child's
//! SELF-REPORTED `worker_private_bytes_peak` (0/missing = sampling
//! unavailable = red, never green), and the 512MiB tree gate consumes
//! a REAL parent+children simultaneous sampler (`procmem::TreeSampler`)
//! — a parent under the cap plus a worker under the cap proves nothing
//! about the tree. F-817-R5-03: the warm and spawn paths share ONE
//! panic-free admission (`admit_response`) verifying protocol echoes,
//! step count, per-step op / source_hash / outcome / binding key /
//! consumer type / value / HIR constant, the edit's exactly-once
//! application, the worker peak and the managed verdict; rejected rows
//! keep their failure reasons and a raw response excerpt.

use crate::driver::CteDriver;
use serde_json::json;
use serde_json::Value;
use std::io::Write;
use std::time::Instant;

pub const DEFAULT_SAMPLES: usize = 30;
pub const DEFAULT_GATE_MS: f64 = 1000.0;

/// Independent closed form for the 115-fn baseline fold (hand-derived,
/// never executed): h_k(20) = 81*20 + 81*k + 3160 (+rev on k==57) and
/// cte_sum folds all 115 over (20, 22).
pub const fn base_closed_form() -> i64 {
    42 + 115 * 4780 + 81 * (114 * 115 / 2)
}

/// The specified baseline generator (contract-decision §8; >=256KiB,
/// 115 functions, bounded + only arithmetic). `rev` rides ONE helper.
pub fn baseline_source_bench(fns: usize, rev: i64) -> String {
    let mut s = String::new();
    for k in 0..fns {
        let rev_here = if k == fns / 2 { rev } else { 0 };
        s.push_str(&format!(
            "fn h{k}(x int) int {{\n    var acc = x + {k} + {rev_here}\n"
        ));
        for j in 0..80 {
            s.push_str(&format!("    acc = acc + {j} + x + {k}\n"));
        }
        s.push_str("    return acc\n}\n\n");
    }
    s.push_str("fn cte_sum(a int, b int) int {\n    var acc = a + b\n");
    for k in 0..fns {
        s.push_str(&format!("    acc = acc + h{k}(20)\n"));
    }
    s.push_str("    return acc\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n");
    s
}

/// The Meta-edit gate's deck at the SPECIFIED load (F-817-R5-02: the
/// 256KiB / 115-fn workload is contract-decision §8's load for the
/// Meta/DAG gate too — a compact-deck substitution is a contract
/// reduction, not a bench choice). The CONTRACTED Meta->AV path rides a
/// multi-query DAG: `field_count Pair` (meta fact) feeds `meta_fold`,
/// which folds the field count through a REAL callee closure of
/// baseline helpers, so each field-set revision re-executes real AV
/// work with a fresh argument.
pub fn big_meta_deck(fields: usize) -> String {
    let src = baseline_source_bench(115, 0);
    let mut fold = String::from("fn meta_fold(n int) int {\n    var acc = n * 3\n");
    for k in 0..META_CALLEES {
        fold.push_str(&format!("    acc = acc + h{k}(n)\n"));
    }
    fold.push_str("    return acc\n}\n");
    let mut extras = String::new();
    for i in 0..fields {
        extras.push_str(&format!("    extra{i} int\n"));
    }
    format!(
        "deck cte817/1\nmodule main\ncaps cte.meta.fields/v1\nmeta <<\ntype Pair {{\n    left int\n    right bool\n{extras}}}\n>>\nsource <<\n{src}\n{fold}>>\nquery n = field_count Pair\nquery q = call meta_fold(n)\nbind main.ct = q expect i32 placeholder 0\nentry main\nnative off\n"
    )
}

/// Independent closed form for the meta gate (hand-derived): with
/// n = field_count(Pair) = 2 + fields and h_k(x) = 81x + 81k + 3160
/// (the rev rider sits on k==57, OUTSIDE the meta callee window 0..32),
/// meta_fold(n) = 3n + Σ_{k<32} h_k(n) = 2595n + 141296.
pub fn meta_closed_form(fields: usize) -> i64 {
    let n = 2 + fields as i64;
    2595 * n + 141_296
}

const META_CALLEES: usize = 32;

pub fn big_deck(rev: i64) -> String {
    // exec snapshot: the incremental lane is the production path for
    // edit-heavy sessions (T-11/SD-06) — the warm resident session makes
    // local helper-body edits cheap on the specified load.
    let src = baseline_source_bench(115, rev);
    format!(
        "deck cte817/1\nmodule main\nexec snapshot\nsource <<\n{src}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n"
    )
}

struct Row {
    scenario: &'static str,
    sample: usize,
    wall_ms: f64,
    input_hash: String,
    cache: &'static str,
    ct_execute: u64,
    ok: bool,
    /// The worker PID for process scenarios (restart asserts PIDs
    /// differ); None for in-process rows.
    pid: Option<u32>,
    /// The measured VALUE (rendered) — raw rows are independently
    /// recomputable (F-817-R3-04).
    value: String,
    /// The child's self-reported private-bytes peak for process rows
    /// (F-817-R5-01); None in-process.
    worker_peak: Option<u64>,
    /// Admission rejection reasons for failed rows (the failure report
    /// travels WITH the raw row, F-817-R5-03).
    failures: Vec<String>,
    /// Raw response excerpt preserved on rejection (truncated).
    raw_excerpt: Option<String>,
}

/// Measure one in-process sample at the specified load: the deck BUILD
/// is inside the window; the row carries the measured VALUE, HIR
/// constant, consumer type and managed verdict against the independent
/// expected form.
fn measure_built(
    scenario: &'static str,
    sample: usize,
    rev: i64,
    drv: &mut CteDriver,
    rows: &mut Vec<Row>,
    expected: i64,
) {
    let t = Instant::now();
    let deck = big_deck(rev);
    let o = drv.query_step(&deck, None);
    let wall = t.elapsed().as_secs_f64() * 1000.0;
    let value = o
        .values
        .get("main.ct")
        .map(|v| v.render())
        .unwrap_or_default();
    let ok = o.status == crate::driver::StepStatus::Ok
        && o.managed_target_met
        && !value.is_empty()
        && value == format!("i32:{expected}")
        && o.hir_proof.get("main.ct").map(String::as_str) == Some(&expected.to_string())
        && o.consumer_types.get("main.ct").map(String::as_str) == Some("i32");
    let failures = if ok {
        Vec::new()
    } else {
        vec![format!(
            "in-process oracle mismatch (status ok={}, managed={}, value={value}, want i32:{expected})",
            o.status == crate::driver::StepStatus::Ok,
            o.managed_target_met
        )]
    };
    rows.push(Row {
        scenario,
        sample,
        wall_ms: wall,
        input_hash: crate::fnv1a64_hex(deck.as_bytes()),
        cache: o.cache.unwrap_or("-"),
        ct_execute: o.ct_execute,
        ok,
        pid: None,
        value,
        worker_peak: None,
        failures,
        raw_excerpt: None,
    });
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ---- the shared step-expectation oracle (F-817-R5-03) ------------------

/// One query step's exact expectation: binding key, rendered value,
/// HIR proof constant and consumer type — checked for EVERY query step
/// (prior steps included), not only the last one.
pub struct QueryExpect {
    pub binding: &'static str,
    pub value: String,
    pub hir_const: String,
    pub ty: &'static str,
}

pub enum StepKind {
    Query(QueryExpect),
    Edit { from: String, to: String },
}

pub struct StepExpect {
    pub op: &'static str,
    pub kind: StepKind,
}

/// The deck + step plan a scenario sample runs.
pub enum DeckKind {
    /// ONE query on the specified baseline at `rev` (restart scenario).
    QueryBig { rev: i64 },
    /// query -> edit(rev -> rev+1) -> query on the baseline at `rev`
    /// (body-edit / worker-edit scenarios).
    EditRev(i64),
    /// single query on the CONTRACTED Meta->AV deck with `fields`
    /// extras in the meta block (meta-edit scenario).
    Meta { fields: usize },
}

impl DeckKind {
    pub fn materialize(&self) -> (String, serde_json::Value) {
        match self {
            DeckKind::QueryBig { rev } => {
                let deck = big_deck(*rev);
                let steps = serde_json::json!([{ "op": "query" }]);
                (deck, steps)
            }
            DeckKind::EditRev(rev) => {
                let deck = big_deck(*rev);
                let steps = serde_json::json!([
                    { "op": "query" },
                    { "op": "edit-source", "from": format!("var acc = x + 57 + {rev}"), "to": format!("var acc = x + 57 + {}", rev + 1) },
                    { "op": "query" },
                ]);
                (deck, steps)
            }
            DeckKind::Meta { fields } => {
                let deck = big_meta_deck(*fields);
                let steps = serde_json::json!([{ "op": "query" }]);
                (deck, steps)
            }
        }
    }

    /// Per-step exact expectations (the independent closed forms —
    /// PRIOR query values included, F-817-R5-03).
    pub fn expects(&self) -> Vec<StepExpect> {
        let base = base_closed_form();
        let query = |value: i64| StepExpect {
            op: "query",
            kind: StepKind::Query(QueryExpect {
                binding: "main.ct",
                value: format!("i32:{value}"),
                hir_const: value.to_string(),
                ty: "i32",
            }),
        };
        match self {
            DeckKind::QueryBig { rev } => vec![query(base + rev)],
            DeckKind::EditRev(rev) => vec![
                query(base + rev),
                StepExpect {
                    op: "edit-source",
                    kind: StepKind::Edit {
                        from: format!("var acc = x + 57 + {rev}"),
                        to: format!("var acc = x + 57 + {}", rev + 1),
                    },
                },
                query(base + rev + 1),
            ],
            DeckKind::Meta { fields } => vec![query(meta_closed_form(*fields))],
        }
    }
}

/// The verdict of the shared parent-side admission.
pub struct AdmissionOutcome {
    pub ok: bool,
    pub failures: Vec<String>,
    pub got_value: String,
    pub ct_execute: u64,
    pub cache: String,
    pub worker_peak: Option<u64>,
    pub managed_ok: bool,
    pub pid: Option<u32>,
}

/// F-817-R5-03: ONE admission for BOTH process paths (warm stream and
/// per-sample spawn). PANIC-FREE: a malformed, truncated, mutated or
/// lying response is a REJECTED row with named reasons and the raw
/// bytes preserved — never a swallowed unwrap. Checks: protocol
/// echoes; step count; per-step op, source_hash (re-derived from the
/// deck text with prior edits applied), outcome, language_class,
/// values/consumer_types/hir maps against the EXACT binding key with
/// the exact type / value / HIR constant (every query step, prior
/// steps included); the edit's exactly-once literal application and
/// its own hash; the worker's self-reported peak (0/missing =
/// sampling unavailable) and the managed verdict.
pub fn admit_response(
    raw: &str,
    nonce: &str,
    request_hash: &str,
    deck: &str,
    expects: &[StepExpect],
    exit_ok: bool,
) -> AdmissionOutcome {
    let mut failures: Vec<String> = Vec::new();
    let mut out = AdmissionOutcome {
        ok: false,
        failures: Vec::new(),
        got_value: String::new(),
        ct_execute: 0,
        cache: "-".to_string(),
        worker_peak: None,
        managed_ok: false,
        pid: None,
    };
    let v: Value = match serde_json::from_str(raw.trim()) {
        Ok(v) => v,
        Err(_) => {
            failures.push("response is not JSON".into());
            out.failures = failures;
            return out;
        }
    };
    if v["schema"] != "cteworker/1" {
        failures.push("schema echo mismatch".into());
    }
    if v["nonce"] != json!(nonce) {
        failures.push(format!("nonce echo mismatch (want {nonce})"));
    }
    if v["request_hash"] != json!(request_hash) {
        failures.push(format!("request_hash echo mismatch (want {request_hash})"));
    }
    if !exit_ok {
        failures.push("worker exit code non-zero".into());
    }
    out.pid = v["ok"]["worker_pid"].as_u64().map(|p| p as u32);
    let arr = v["ok"]["steps"].as_array().cloned();
    match &arr {
        None => failures.push("no ok envelope / steps array".into()),
        Some(arr) => {
            if arr.len() != expects.len() {
                failures.push(format!(
                    "step count {} != expected {}",
                    arr.len(),
                    expects.len()
                ));
            } else {
                let mut edited_deck = deck.to_string();
                let mut edited_seen = false;
                for (si, st) in arr.iter().enumerate() {
                    let e = &expects[si];
                    let op = st["op"].as_str().unwrap_or("");
                    if op != e.op {
                        failures.push(format!("step {si}: op {op:?} != {:?}", e.op));
                        continue;
                    }
                    match &e.kind {
                        StepKind::Edit { from, to } => {
                            if st["outcome"] != "edited" {
                                failures.push(format!(
                                    "step {si}: edit outcome {} != edited",
                                    st["outcome"]
                                ));
                            }
                            if edited_deck.matches(from.as_str()).count() != 1 {
                                failures.push(format!(
                                    "step {si}: edit literal does not match the deck exactly once"
                                ));
                            } else {
                                edited_deck = edited_deck.replacen(from.as_str(), to.as_str(), 1);
                            }
                            let want = crate::fnv1a64_hex(edited_deck.as_bytes());
                            if st["source_hash"].as_str() != Some(want.as_str()) {
                                failures.push(format!("step {si}: edit source_hash mismatch"));
                            }
                            edited_seen = true;
                        }
                        StepKind::Query(q) => {
                            let want_hash = crate::fnv1a64_hex(edited_deck.as_bytes());
                            if st["source_hash"].as_str() != Some(want_hash.as_str()) {
                                failures.push(format!("step {si}: source_hash mismatch"));
                            }
                            if st["outcome"] != "ok" {
                                failures.push(format!(
                                    "step {si}: outcome {} inside a valid envelope",
                                    st["outcome"]
                                ));
                            }
                            if st["language_class"] != json!(false) {
                                failures.push(format!("step {si}: language_class not false"));
                            }
                            out.ct_execute += st["ct_execute"].as_u64().unwrap_or(0);
                            let vals_ok = match st["values"].as_object() {
                                Some(m) if m.len() == 1 && m.contains_key(q.binding) => {
                                    m.get(q.binding).and_then(|x| x.as_str())
                                        == Some(q.value.as_str())
                                }
                                _ => false,
                            };
                            if !vals_ok {
                                failures.push(format!(
                                    "step {si}: values[binding {}] wrong key/set or wrong value (expected {})",
                                    q.binding, q.value
                                ));
                            }
                            let ty_ok = match st["consumer_types"].as_object() {
                                Some(m) if m.len() == 1 && m.contains_key(q.binding) => {
                                    m.get(q.binding).and_then(|x| x.as_str()) == Some(q.ty)
                                }
                                _ => false,
                            };
                            if !ty_ok {
                                failures.push(format!(
                                    "step {si}: consumer_types[{}] != {}",
                                    q.binding, q.ty
                                ));
                            }
                            let hir_ok = match st["hir"].as_object() {
                                Some(m) if m.len() == 1 && m.contains_key(q.binding) => {
                                    m.get(q.binding).and_then(|x| x.as_str())
                                        == Some(q.hir_const.as_str())
                                }
                                _ => false,
                            };
                            if !hir_ok {
                                failures.push(format!(
                                    "step {si}: hir[{}] != constant {}",
                                    q.binding, q.hir_const
                                ));
                            }
                            if si + 1 == arr.len() {
                                out.got_value = st["values"]
                                    .as_object()
                                    .and_then(|m| m.get(q.binding))
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                out.cache = st["cache"].as_str().unwrap_or("-").to_string();
                            }
                        }
                    }
                }
                let want_edit = expects
                    .iter()
                    .any(|e| matches!(e.kind, StepKind::Edit { .. }));
                if want_edit && !edited_seen {
                    failures.push("planned edit never observed".into());
                }
            }
        }
    }
    match v["ok"]["worker_private_bytes_peak"].as_u64() {
        Some(p) if p > 0 => out.worker_peak = Some(p),
        _ => failures.push("worker peak sampling unavailable (missing/zero)".into()),
    }
    if v["ok"]["managed_target_met"] != json!(true) {
        failures.push("managed budget exceeded (managed_target_met=false)".into());
    } else {
        out.managed_ok = true;
    }
    out.failures = failures;
    out.ok = out.failures.is_empty();
    out
}

fn truncate_raw(raw: &str) -> String {
    const KEEP: usize = 2048;
    if raw.len() <= KEEP {
        raw.trim().to_string()
    } else {
        format!("{}…[truncated {}B]", &raw[..KEEP], raw.len())
    }
}

/// Serialize one `cteworker/1` request line for `kind` — the REAL
/// request format the bench spawn paths emit; exposed so the
/// wire-mutation regression (F-817-R5-03) drives actual bytes through
/// actual workers.
pub fn build_worker_request_line(
    nonce: &str,
    request_hash: &str,
    store_root: &std::path::Path,
    session: &str,
    kind: &DeckKind,
) -> String {
    use base64::Engine as _;
    let (deck, steps) = kind.materialize();
    serde_json::json!({
        "recipe": "cte/scalar-query-v1",
        "nonce": nonce,
        "deck_b64": base64::engine::general_purpose::STANDARD.encode(&deck),
        "store_root": store_root.display().to_string().replace('/', "\\"),
        "session": session,
        "steps": steps,
        "request_hash": request_hash,
    })
    .to_string()
}

pub fn run_bench(
    samples: usize,
    gate_ms: f64,
    inject_delay_ms: u64,
    out: Option<&std::path::Path>,
    root: &std::path::Path,
) -> (i32, serde_json::Value) {
    run_bench_ext(samples, gate_ms, inject_delay_ms, false, &[], out, root)
}

fn selected(only: &[&str], name: &str) -> bool {
    only.is_empty() || only.contains(&name)
}

/// `inject_rebuild`: the forced whole-module rebuild control (the
/// resident-edit degrade gate must go red on it). `only`: restrict the
/// run to the named scenarios (the formal gate runs ALL of them; the
/// counter-controls use this to reach one memory/admission gate
/// cheaply through the very same machinery).
pub fn run_bench_ext(
    samples: usize,
    gate_ms: f64,
    _inject_delay_ms: u64,
    inject_rebuild: bool,
    only: &[&str],
    out: Option<&std::path::Path>,
    root: &std::path::Path,
) -> (i32, serde_json::Value) {
    let _ = std::fs::create_dir_all(root);
    // F-817-R1-04: the gate REFUSES malformed inputs outright — zero,
    // negative or insufficient sample counts and non-finite thresholds
    // exit non-zero with target_met=false (never a green empty run).
    let invalid: Option<String> = if samples == 0 || samples > 100_000 {
        Some(format!("samples {samples} must be 1..=100000"))
    } else if !gate_ms.is_finite() {
        Some(format!("gate-ms {gate_ms} must be finite"))
    } else if gate_ms < 0.0 {
        Some(format!("gate-ms {gate_ms} must be >= 0"))
    } else {
        None
    };
    if let Some(msg) = invalid {
        let summary = serde_json::json!({
            "target_met": false,
            "hard_gate_failures": [format!("invalid input: {msg}")],
        });
        if let Some(path) = out {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut f) = std::fs::File::create(path) {
                use std::io::Write;
                let _ = writeln!(f, "{summary}");
            }
        }
        return (2, summary);
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut extra_failures: Vec<String> = Vec::new();
    let bench_start = Instant::now();
    // F-817-R5-01: the TREE gate's input — the parent plus every
    // registered live worker child, sampled SIMULTANEOUSLY (5ms ticks).
    // The parent's solo peak rides the same sampler (no separate
    // parent-only bench sampler: one sampler, both numbers, one scope).
    let tree_sampler = crate::procmem::TreeSampler::start(5);

    // per-scenario worker peaks: in-process scenarios feed the parent's
    // scenario sampler; process scenarios feed the CHILD's self-reported
    // wire peak (with a sample count of real readings — F-817-R5-01).
    let mut scenario_peaks: std::collections::BTreeMap<&'static str, u64> =
        std::collections::BTreeMap::new();
    let mut scenario_sampled: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();

    // 1) fresh: a brand-new store per sample (cold everything) on the
    //    SPECIFIED LOAD; the deck build is INSIDE the measured window.
    if selected(only, "fresh") {
        for i in 0..samples {
            let dir = root.join(format!("fresh-{i}"));
            let _ = std::fs::remove_dir_all(&dir);
            let mut d = CteDriver::with_store(&dir, "fresh").unwrap();
            let sp = auto_av_probe::mempeek::PeakSampler::start(1);
            measure_built("fresh", i, 0, &mut d, &mut rows, base_closed_form());
            let pk = sp.stop();
            scenario_peaks
                .entry("fresh")
                .and_modify(|c| *c = (*c).max(pk))
                .or_insert(pk);
        }
    }

    // 2) hot: one warm driver, repeated identical requests (hit path) on
    //    the specified load. Dropped at scenario end so later process
    //    scenarios do not carry its footprint in the tree sum.
    if selected(only, "hot") {
        let hot_dir = root.join("hot");
        let _ = std::fs::remove_dir_all(&hot_dir);
        let mut hot = CteDriver::with_store(&hot_dir, "hot").unwrap();
        {
            let sp = auto_av_probe::mempeek::PeakSampler::start(1);
            let probe_out = hot.query_step(&big_deck(0), None);
            assert_eq!(probe_out.status, crate::driver::StepStatus::Ok);
            let pk = sp.stop();
            scenario_peaks
                .entry("hot")
                .and_modify(|c| *c = (*c).max(pk))
                .or_insert(pk);
        }
        {
            let sp = auto_av_probe::mempeek::PeakSampler::start(1);
            for i in 0..samples {
                measure_built("hot", i, 0, &mut hot, &mut rows, base_closed_form());
            }
            let pk = sp.stop();
            scenario_peaks
                .entry("hot")
                .and_modify(|c| *c = (*c).max(pk))
                .or_insert(pk);
        }
        drop(hot);
    }

    // 3) restart: a REAL worker process per sample (F-817-R3-03) on the
    //    SPECIFIED LOAD; the window covers request build -> serialize ->
    //    spawn -> IPC -> response parse + the shared admission.
    if selected(only, "restart") {
        let rs_dir = root.join("restart");
        let _ = std::fs::remove_dir_all(&rs_dir);
        {
            let mut warm = CteDriver::with_store(&rs_dir, "rs").unwrap();
            let warm_out = warm.query_step(&big_deck(0), None);
            assert_eq!(warm_out.status, crate::driver::StepStatus::Ok);
        }
        spawn_process_samples(
            "restart",
            samples,
            &rs_dir,
            &DeckKind::QueryBig { rev: 0 },
            &mut rows,
            &mut scenario_peaks,
            &mut scenario_sampled,
            &tree_sampler,
            &mut extra_failures,
        );
    }

    // 4) body-edit (HARD GATE, WARM SESSION over real IPC): ONE worker
    //    process, warmed outside the window; each sample's window runs
    //    per-sample deck generation -> serialization -> IPC -> response
    //    -> COMPLETE shared admission (prior-step values included).
    if selected(only, "body-edit") {
        let be_dir = root.join("body-edit");
        let _ = std::fs::remove_dir_all(&be_dir);
        let mut kinds: Vec<DeckKind> = Vec::with_capacity(samples + 1);
        kinds.push(DeckKind::EditRev(0));
        for i in 0..samples {
            kinds.push(DeckKind::EditRev(i as i64));
        }
        spawn_warm_samples(
            "body-edit",
            samples,
            &be_dir,
            &kinds,
            &mut rows,
            &mut scenario_peaks,
            &mut scenario_sampled,
            &tree_sampler,
            _inject_delay_ms,
            &mut extra_failures,
        );
    }

    // 5) meta-edit (HARD GATE, WARM SESSION over real IPC): the
    //    CONTRACTED Meta->AV path on the SPECIFIED load — each sample a
    //    deck whose meta block grew one field; field_count flips, the AV
    //    call re-executes meta_fold with the new argument through its
    //    real callee closure.
    if selected(only, "meta-edit") {
        let me_dir = root.join("meta-edit");
        let _ = std::fs::remove_dir_all(&me_dir);
        let mut kinds: Vec<DeckKind> = Vec::with_capacity(samples + 1);
        kinds.push(DeckKind::Meta { fields: 0 });
        for i in 0..samples {
            kinds.push(DeckKind::Meta { fields: i + 1 });
        }
        spawn_warm_samples(
            "meta-edit",
            samples,
            &me_dir,
            &kinds,
            &mut rows,
            &mut scenario_peaks,
            &mut scenario_sampled,
            &tree_sampler,
            0,
            &mut extra_failures,
        );
    }

    // 6) gc-concurrent: queries under a live GC loop on the same store
    //    (its OWN store handle), on the specified load; exact per-sample
    //    closed form (rev = i % 3).
    if selected(only, "gc") {
        let gc_dir = root.join("gc");
        let _ = std::fs::remove_dir_all(&gc_dir);
        let mut gd = CteDriver::with_store(&gc_dir, "gc").unwrap();
        {
            let probe_out = gd.query_step(&big_deck(0), None);
            assert_eq!(probe_out.status, crate::driver::StepStatus::Ok);
        }
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop2 = stop.clone();
        let gc_root = gc_dir.clone();
        let handle = std::thread::spawn(move || {
            let st = match crate::store::CteStore::open(&gc_root, "gc") {
                Ok(s) => s,
                Err(_) => return,
            };
            while !stop2.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = ac_artifact_probe::store::run_gc(
                    st.raw(),
                    ac_artifact_probe::store::GcConfig { high: 1, low: 0 },
                );
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        });
        {
            let sp = auto_av_probe::mempeek::PeakSampler::start(1);
            for i in 0..samples {
                measure_built(
                    "gc",
                    i,
                    (i % 3) as i64,
                    &mut gd,
                    &mut rows,
                    base_closed_form() + (i % 3) as i64,
                );
            }
            let pk = sp.stop();
            scenario_peaks
                .entry("gc")
                .and_modify(|c| *c = (*c).max(pk))
                .or_insert(pk);
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = handle.join();
        drop(gd);
    }

    // 7) worker-edit: a REAL worker process per sample (query -> edit ->
    //    query) on the specified load; every round edits a NEW revision
    //    literal (per-round fresh hash), expected base + rev + 1.
    if selected(only, "worker-edit") {
        let we_dir = root.join("worker-edit");
        let _ = std::fs::remove_dir_all(&we_dir);
        for i in 0..samples {
            let kind = DeckKind::EditRev(i as i64);
            spawn_process_samples(
                "worker-edit",
                1,
                &we_dir,
                &kind,
                &mut rows,
                &mut scenario_peaks,
                &mut scenario_sampled,
                &tree_sampler,
                &mut extra_failures,
            );
            if let Some(r) = rows.last_mut() {
                r.sample = i;
            }
        }
    }

    // 8) resident-edit (FORMAL GATE): 30 distinct local body revisions
    //    through ONE resident driver at the specified load. TWO hard
    //    gates (F-817-R3-02): ZERO whole-module re-warms AND P95 < gate.
    //    --inject-rebuild changes the STRUCTURE at the midpoint; the
    //    SAME normal gates go red from the DETECTOR VALUE (no marker
    //    failure) — with the detector disabled (env control) the red
    //    must vanish, proving the gate is load-bearing.
    let mut rewarms_before: u64 = 0;
    let mut rewarms_after: u64 = 0;
    if selected(only, "resident-edit") {
        let re_dir = root.join("resident-edit");
        let _ = std::fs::remove_dir_all(&re_dir);
        let mut re = CteDriver::with_store(&re_dir, "re").unwrap();
        let re_start = re.query_step(&big_deck(0), None);
        assert_eq!(re_start.status, crate::driver::StepStatus::Ok);
        rewarms_before = re.snapshot_rewarms();
        let sp = auto_av_probe::mempeek::PeakSampler::start(1);
        for i in 0..samples {
            let rev = (i + 1) as i64;
            let deck = if inject_rebuild && i == samples / 2 {
                // Structural change at the SAME function count: a rename is a
                // declaration-set change the single-hunk sync cannot absorb —
                // the whole module re-warms (the detector's trigger) without
                // growing the source past the memory gate.
                let structural = baseline_source_bench(115, rev).replace("h57", "hx57");
                format!(
                    "deck cte817/1\nmodule main\nexec snapshot\nsource <<\n{structural}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n"
                )
            } else {
                big_deck(rev)
            };
            let t = Instant::now();
            let o = re.query_step(&deck, None);
            let wall = t.elapsed().as_secs_f64() * 1000.0;
            let value = o
                .values
                .get("main.ct")
                .map(|v| v.render())
                .unwrap_or_default();
            let structural_round = inject_rebuild && i == samples / 2;
            let ok = o.status == crate::driver::StepStatus::Ok
                && o.managed_target_met
                && (structural_round
                    || (value == format!("i32:{}", base_closed_form() + rev)
                        && o.hir_proof.get("main.ct").map(String::as_str)
                            == Some(&(base_closed_form() + rev).to_string())
                        && o.consumer_types.get("main.ct").map(String::as_str) == Some("i32")));
            let failures = if ok {
                Vec::new()
            } else {
                vec![format!(
                    "in-process oracle mismatch (value={value}, want i32:{})",
                    base_closed_form() + rev
                )]
            };
            rows.push(Row {
                scenario: "resident-edit",
                sample: i,
                wall_ms: wall,
                input_hash: crate::fnv1a64_hex(deck.as_bytes()),
                cache: o.cache.unwrap_or("-"),
                ct_execute: o.ct_execute,
                ok,
                pid: None,
                value,
                worker_peak: None,
                failures,
                raw_excerpt: None,
            });
        }
        let pk = sp.stop();
        scenario_peaks
            .entry("resident-edit")
            .and_modify(|c| *c = (*c).max(pk))
            .or_insert(pk);
        rewarms_after = re.snapshot_rewarms();
        drop(re);
    }

    // ---- aggregate ---------------------------------------------------
    let bench_wall_ms = bench_start.elapsed().as_secs_f64() * 1000.0;
    let tree = tree_sampler.stop();
    let scenarios: [&str; 8] = [
        "fresh",
        "hot",
        "restart",
        "body-edit",
        "meta-edit",
        "gc",
        "worker-edit",
        "resident-edit",
    ];
    let run_scenarios: Vec<&str> = scenarios
        .iter()
        .copied()
        .filter(|s| selected(only, s))
        .collect();
    let mut summary = serde_json::json!({
        "samples_per_scenario": samples,
        "gate_ms": gate_ms,
        "bench_wall_ms": bench_wall_ms,
        "bench_parent_private_bytes_peak": tree.parent_peak,
        "tree_private_bytes_peak": tree.tree_peak,
        "tree_sample_interval_ms": 5,
    });
    let mut failures: Vec<String> = Vec::new();
    failures.append(&mut extra_failures);
    for sc in &run_scenarios {
        let mut walls: Vec<f64> = rows
            .iter()
            .filter(|r| r.scenario == *sc && r.sample != usize::MAX && r.sample < 10_000)
            .map(|r| r.wall_ms)
            .collect();
        walls.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = percentile(&walls, 0.95);
        let max = walls.last().copied().unwrap_or(0.0);
        let ok = rows
            .iter()
            .filter(|r| r.scenario == *sc && r.sample != usize::MAX && r.sample < 10_000)
            .all(|r| r.ok);
        if !ok {
            failures.push(format!(
                "{sc} value/admission mismatch (independent oracle; see failed rows)"
            ));
        }
        summary[sc] = json!({
            "p95_ms": p95,
            "max_ms": max,
            "samples": walls.len(),
            "all_values_ok": ok,
        });
        // Strict: P95 must be strictly BELOW the gate (equality fails).
        // The edit hard gates AND the resident-edit lane gate (F-817-R3-02)
        // are enforced here.
        if (*sc == "body-edit" || *sc == "meta-edit" || *sc == "resident-edit") && p95 >= gate_ms {
            failures.push(format!("{sc} p95 {:.1}ms >= gate {gate_ms}ms", p95));
        }
        if walls.len() != samples {
            failures.push(format!(
                "{sc} measured {} samples, expected {samples}",
                walls.len()
            ));
        }
    }
    if gate_ms <= 0.0 {
        failures.push("gate-ms 0 always fails (counter-control)".into());
    }
    // F-817-R3-04: the degrade gate fires ONLY from the DETECTOR VALUE —
    // the same normal check in control and normal runs alike (no marker
    // failure manufactured by the inject flag). The env control disables
    // the detector; the TEST then asserts the red vanishes — proving the
    // gate, not the flag, produces the failure.
    let detector_disabled = std::env::var("AUTO_CTE_DISABLE_RESIDENT_GATE")
        .map(|v| v == "1")
        .unwrap_or(false);
    if selected(only, "resident-edit") {
        let resident_rows_ok = rows
            .iter()
            .filter(|r| r.scenario == "resident-edit")
            .all(|r| r.ok);
        let re_delta = rewarms_after.saturating_sub(rewarms_before);
        summary["resident_edit_rewarms"] = json!(re_delta);
        if !resident_rows_ok {
            failures.push("resident-edit sample value mismatch".into());
        }
        if !detector_disabled && re_delta != 0 {
            failures.push(format!(
                "resident-edit degraded: {re_delta} whole-module re-warm(s) during incremental traffic"
            ));
        }
        if inject_rebuild && re_delta == 0 && !detector_disabled {
            failures.push("forced-rebuild control did NOT re-warm (detector saw nothing)".into());
        }
    }
    // F-817-R5-01: the memory hard gates. Worker gate = per-scenario
    // peak (child's self-reported wire peak for process scenarios, the
    // scenario sampler for in-process ones); a ZERO peak or a missing
    // child reading is SAMPLING UNAVAILABLE — red, never green. Tree
    // gate = the simultaneous parent+children sampler.
    const WORKER_GATE: u64 = 384 * 1024 * 1024;
    const TREE_GATE: u64 = 512 * 1024 * 1024;
    let mut peak_map = serde_json::Map::new();
    for (name, pk) in &scenario_peaks {
        let sampled = scenario_sampled.get(name);
        peak_map.insert(
            name.to_string(),
            json!({
                "bytes": pk,
                "mib": (*pk as f64) / 1048576.0,
                "child_reported_samples": sampled,
            }),
        );
    }
    summary["scenario_peaks"] = Value::Object(peak_map);
    for sc in &run_scenarios {
        let Some(pk) = scenario_peaks.get(sc) else {
            failures.push(format!(
                "{sc} memory sampling unavailable (no peak recorded)"
            ));
            continue;
        };
        if *pk == 0 {
            failures.push(format!("{sc} memory sampling unavailable (zero peak)"));
            continue;
        }
        if *pk > WORKER_GATE {
            failures.push(format!(
                "{sc} worker peak {:.1}MiB > 384MiB gate",
                *pk as f64 / 1048576.0
            ));
        }
        // Process scenarios: every measured sample must have carried a
        // real child peak (a missing reading is unavailable, not zero).
        if let Some(n) = scenario_sampled.get(*sc) {
            if *n < samples {
                failures.push(format!(
                    "{sc} worker peak sampling unavailable on {} of {samples} samples",
                    samples - *n
                ));
            }
        }
    }
    if tree.tree_peak == 0 {
        failures.push("tree memory sampling unavailable (zero peak)".into());
    }
    if tree.tree_peak > TREE_GATE {
        failures.push(format!(
            "process tree peak {:.1}MiB > 512MiB gate (parent + live worker children, simultaneous sampling)",
            tree.tree_peak as f64 / 1048576.0
        ));
    }
    summary["hard_gate_failures"] = json!(failures);
    summary["target_met"] = json!(failures.is_empty());

    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut f = std::fs::File::create(path).expect("raw report file");
        for r in &rows {
            let mut line = json!({
                "scenario": r.scenario,
                "sample": if r.sample == usize::MAX { serde_json::Value::Null } else { json!(r.sample) },
                "wall_ms": r.wall_ms,
                "input_hash": r.input_hash,
                "cache": r.cache,
                "ct_execute": r.ct_execute,
                "ok": r.ok,
                "pid": r.pid,
                "value": r.value,
                "worker_private_bytes_peak": r.worker_peak,
            });
            if !r.failures.is_empty() {
                line["failures"] = json!(r.failures);
                line["raw"] = json!(r.raw_excerpt.clone().unwrap_or_default());
            }
            writeln!(f, "{line}").ok();
        }
        writeln!(f, "{summary}").ok();
    }

    (if failures.is_empty() { 0 } else { 5 }, summary)
}

/// F-817-R3-03/04 + R5-01/03: one REAL worker process per sample. The
/// wall covers request build + serialization, spawn, IPC, response
/// parse AND the complete shared admission; the scenario's worker peak
/// is the CHILD's self-reported wire peak (registered with the tree
/// sampler for its whole lifetime).
fn spawn_process_samples(
    scenario: &'static str,
    samples: usize,
    store: &std::path::Path,
    kind: &DeckKind,
    rows: &mut Vec<Row>,
    peaks: &mut std::collections::BTreeMap<&'static str, u64>,
    sampled: &mut std::collections::BTreeMap<&'static str, usize>,
    tree: &crate::procmem::TreeSampler,
    extra_failures: &mut Vec<String>,
) {
    use base64::Engine as _;
    use std::io::Write;
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe().expect("exe");
    for i in 0..samples {
        // Deck build INSIDE the window (same rule as the warm path).
        let t = Instant::now();
        let (deck, steps) = kind.materialize();
        let expects = kind.expects();
        let nonce = format!("{scenario}-{i}");
        let rh = format!("rh-{scenario}-{i}");
        let req = serde_json::json!({
            "recipe": "cte/scalar-query-v1",
            "nonce": nonce,
            "deck_b64": base64::engine::general_purpose::STANDARD.encode(&deck),
            "store_root": store.display().to_string().replace('/', "\\"),
            "session": scenario,
            "steps": steps,
            "request_hash": rh,
        })
        .to_string();
        let mut child = Command::new(&exe)
            .arg("case-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("worker spawn");
        let child_pid = child.id();
        tree.register(child_pid);
        child
            .stdin
            .take()
            .unwrap()
            .write_all(req.as_bytes())
            .unwrap();
        let out = child.wait_with_output().expect("worker completes");
        tree.unregister(child_pid);
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        // Admission ends INSIDE the window (F-817-R5-03 shared oracle).
        let adm = admit_response(&text, &nonce, &rh, &deck, &expects, out.status.success());
        if let Some(p) = adm.worker_peak {
            peaks
                .entry(scenario)
                .and_modify(|c| *c = (*c).max(p))
                .or_insert(p);
            *sampled.entry(scenario).or_insert(0) += 1;
        }
        let wall = t.elapsed().as_secs_f64() * 1000.0;
        let ok = adm.ok;
        let cache_leaked: &'static str = Box::leak(adm.cache.clone().into_boxed_str());
        rows.push(Row {
            scenario,
            sample: i,
            wall_ms: wall,
            input_hash: crate::fnv1a64_hex(deck.as_bytes()),
            cache: if ok { cache_leaked } else { "proto-err" },
            ct_execute: adm.ct_execute,
            ok,
            pid: adm.pid.or(Some(child_pid)),
            value: adm.got_value,
            worker_peak: adm.worker_peak,
            failures: adm.failures.clone(),
            raw_excerpt: if ok { None } else { Some(truncate_raw(&text)) },
        });
    }
    let _ = extra_failures; // spawn path reports through rows only
}

/// F-817-R4-02/03 + R5-01/03: the WARM-SESSION hard-gate window. ONE
/// worker process (case-serve-stream, one resident driver + store),
/// warmed outside the window. Each sample's window runs from PER-SAMPLE
/// DECK GENERATION through request serialization, IPC, response read,
/// and the COMPLETE shared admission; the child is registered with the
/// tree sampler for its whole lifetime and its self-reported peak (and
/// presence per sample) feeds the worker gate.
fn spawn_warm_samples(
    scenario: &'static str,
    samples: usize,
    store: &std::path::Path,
    deck_kinds: &[DeckKind],
    rows: &mut Vec<Row>,
    peaks: &mut std::collections::BTreeMap<&'static str, u64>,
    sampled: &mut std::collections::BTreeMap<&'static str, usize>,
    tree: &crate::procmem::TreeSampler,
    inject_delay_ms: u64,
    extra_failures: &mut Vec<String>,
) {
    use base64::Engine as _;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe().expect("exe");
    let mut child = Command::new(&exe)
        .arg("case-serve-stream")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("warm worker spawn");
    let child_pid = child.id();
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let build_req = |i: usize, deck: &str, steps: &serde_json::Value| -> serde_json::Value {
        serde_json::json!({
            "recipe": "cte/scalar-query-v1",
            "nonce": format!("{scenario}-{i}"),
            "deck_b64": base64::engine::general_purpose::STANDARD.encode(deck),
            "store_root": store.display().to_string().replace('/', "\\"),
            "session": scenario,
            "steps": steps,
            "request_hash": format!("rh-{scenario}-{i}"),
        })
    };

    // Warm-up OUTSIDE the window — but still fully admitted: a lying
    // warm-up is a bench failure (F-817-R5-03), and it tells us the
    // child PID for tree registration.
    {
        let (deck, steps) = deck_kinds[0].materialize();
        let expects = deck_kinds[0].expects();
        let req = build_req(usize::MAX, &deck, &steps);
        stdin
            .write_all(format!("{}\n", req.to_string()).as_bytes())
            .expect("warm write");
        stdin.flush().expect("warm flush");
        let mut line = String::new();
        reader.read_line(&mut line).expect("warm read");
        let adm = admit_response(
            &line,
            &format!("{scenario}-{}", usize::MAX),
            &format!("rh-{scenario}-{}", usize::MAX),
            &deck,
            &expects,
            true,
        );
        if !adm.ok {
            extra_failures.push(format!(
                "{scenario} warm-up rejected by admission: {}",
                adm.failures.join("; ")
            ));
        }
        tree.register(adm.pid.unwrap_or(child_pid));
    }

    for i in 0..samples {
        let t = Instant::now();
        // Per-sample generation INSIDE the window (F-817-R4-02).
        let (deck, steps) = deck_kinds[i + 1].materialize();
        let expects = deck_kinds[i + 1].expects();
        let nonce = format!("{scenario}-{i}");
        let rh = format!("rh-{scenario}-{i}");
        let req = build_req(i, &deck, &steps);
        stdin
            .write_all(format!("{}\n", req.to_string()).as_bytes())
            .expect("write");
        stdin.flush().expect("flush");
        let mut line = String::new();
        reader.read_line(&mut line).expect("read");
        let adm = admit_response(&line, &nonce, &rh, &deck, &expects, true);
        if let Some(p) = adm.worker_peak {
            peaks
                .entry(scenario)
                .and_modify(|c| *c = (*c).max(p))
                .or_insert(p);
            *sampled.entry(scenario).or_insert(0) += 1;
        }
        let mut wall = t.elapsed().as_secs_f64() * 1000.0;
        if inject_delay_ms > 0 && i < 2 {
            std::thread::sleep(std::time::Duration::from_millis(inject_delay_ms));
            wall += inject_delay_ms as f64;
        }
        let ok = adm.ok;
        let cache_leaked: &'static str = Box::leak(adm.cache.clone().into_boxed_str());
        rows.push(Row {
            scenario,
            sample: i,
            wall_ms: wall,
            input_hash: crate::fnv1a64_hex(deck.as_bytes()),
            cache: if ok { cache_leaked } else { "proto-err" },
            ct_execute: adm.ct_execute,
            ok,
            pid: adm.pid,
            value: adm.got_value,
            worker_peak: adm.worker_peak,
            failures: adm.failures.clone(),
            raw_excerpt: if ok { None } else { Some(truncate_raw(&line)) },
        });
    }
    tree.unregister(child_pid);
    drop(stdin);
    let _ = child.wait();
}
