//! PLAN-828 T-08 formal worker protocol + parent-side judge (F-12
//! scaffold).
//!
//! The FORMAL entry is ONE persistent worker process and a parent-side
//! judge: the worker holds the resident session (warm-up once, then one
//! real body edit per request), the parent owns the measurement window
//! (edit ENQUEUE → parent-side verification of the published result —
//! IPC, queueing and the run are inside the window, exactly as §5.5
//! requires) and the independent expectations. The dual gate stays the
//! plan's: P95 < `gate_ms` AND every result correct AND the structural
//! axes green — a timing pass can never cover a granularity failure,
//! and `gate_ms == 0` MUST fail (V-09 countercontrol).
//!
//! Scaffold boundary: the protocol, judge, gates and cross-process tests
//! below are the production surfaces; the FORMAL measurement campaign
//! (frozen 273 575 B load, ≥30 revisions, P95 verdict, dual-line memory
//! budget) runs on the reviewed execution core after the batch-2
//! re-review — not from this scaffold.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

/// Parent → worker: initialize the resident session (once).
#[derive(Serialize, Deserialize, Debug)]
pub struct InitOp {
    pub src: String,
    pub entry: String,
    #[serde(default)]
    pub caps: Vec<String>,
}

/// Parent → worker: one real body edit (old-source byte coordinates).
#[derive(Serialize, Deserialize, Debug)]
pub struct EditOp {
    pub req_id: u64,
    pub start: u64,
    pub end: u64,
    pub insert: String,
}

/// Parent → worker: terminal close.
#[derive(Serialize, Deserialize, Debug)]
pub struct CloseOp {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op")]
pub enum WorkerRequest {
    Init(InitOp),
    Edit(EditOp),
    Close(CloseOp),
}

/// Worker → parent: the session is warm. `session_id` is the CONSTANT
/// worker identity (verified once here, per §5.5); the per-publication
/// ACTUAL admission stays in every `EditResult`.
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadyMsg {
    pub session_id: u64,
    pub functions: usize,
    pub revision: u64,
    pub value: i32,
}

/// Worker → parent: one publication — the applied edit, the CURRENT
/// revision's verified result, the worker-internal phase timings (for
/// attribution only; the judged window is the PARENT's), the cumulative
/// counter snapshot (the judge takes deltas), and the worker's sampled
/// memory peak (the worker line of the dual-line budget).
#[derive(Serialize, Deserialize, Debug)]
pub struct EditResultMsg {
    pub req_id: u64,
    pub ok: bool,
    pub revision: u64,
    pub value: Option<i32>,
    pub changed_fn: Option<String>,
    pub install_set: Vec<String>,
    pub diagnostics: Vec<String>,
    pub non_fast_path_reason: Option<String>,
    pub worker_edit_us: u64,
    pub worker_edit_to_result_us: u64,
    pub counters: BTreeMap<String, u64>,
    pub worker_peak_bytes: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClosedMsg {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WorkerReply {
    Ready(ReadyMsg),
    EditResult(EditResultMsg),
    Closed(ClosedMsg),
}

/// One judged sample: the PARENT-measured enqueue→verified-publication
/// window (ms) plus the raw axes the report preserves.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JudgeSample {
    pub req_id: u64,
    pub edit_to_verified_ms: f64,
    pub expected: i32,
    pub value: Option<i32>,
    pub correct: bool,
    pub ok: bool,
    pub revision: u64,
    pub worker_edit_to_result_us: u64,
    pub worker_peak_bytes: u64,
    /// Structural deltas over the UNCHANGED functions (stage sums,
    /// whole-module axes, install-set size, rescan evidence) computed by
    /// the judge from cumulative counter snapshots.
    pub unaffected_stage_total: u64,
    pub whole_module_total: u64,
    pub install_set_len: usize,
    pub reparsed_decls: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JudgeReport {
    /// The CONSTANT worker identity from the ready handshake (one
    /// process, one session — the formal campaign pins this).
    pub session_id: u64,
    pub samples: Vec<JudgeSample>,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub gate_ms: f64,
    pub gate_pass: bool,
    pub all_correct: bool,
    pub structural_ok: bool,
    pub worker_peak_bytes: u64,
    pub parent_peak_bytes: u64,
    /// Every failure axis that fired (V-09: each axis must be able to
    /// redden the gate on its own).
    pub failures: Vec<String>,
}

/// The judge's expected-edit script (independent of the worker).
#[derive(Clone, Debug)]
pub struct JudgeEdit {
    pub start: u64,
    pub end: u64,
    pub insert: String,
    pub expected: i32,
}

pub struct JudgeConfig {
    pub src: String,
    pub entry: String,
    pub caps: Vec<String>,
    /// The independent expectation for the READY (initial) value.
    pub expected_initial: i32,
    pub edits: Vec<JudgeEdit>,
    /// The enforced P95 bound. `0.0` MUST fail (V-09 countercontrol).
    pub gate_ms: f64,
}

/// Spawn the persistent worker and run the judged series. Every axis —
/// correctness, structure, sample completeness, gate — is evaluated from
/// the PARENT side against independent expectations; the worker's own
/// numbers are attribution, never the verdict.
pub fn run_formal_session_judge(
    worker_exe: &Path,
    cfg: &JudgeConfig,
) -> Result<JudgeReport, String> {
    let mut link = WorkerLink::spawn(worker_exe)?;
    let parent_peak = crate::mempeek::PeakSampler::start(20);

    let mut failures: Vec<String> = Vec::new();
    let mut samples: Vec<JudgeSample> = Vec::with_capacity(cfg.edits.len());
    let mut prev_counters: BTreeMap<String, u64> = BTreeMap::new();
    let mut worker_peak = 0u64;
    let session_id: Option<u64>;

    // Ready handshake: one initialization, one constant worker identity,
    // the initial value judged against the independent expectation.
    link.send(&WorkerRequest::Init(InitOp {
        src: cfg.src.clone(),
        entry: cfg.entry.clone(),
        caps: cfg.caps.clone(),
    }))?;
    match link.recv()? {
        WorkerReply::Ready(r) => {
            session_id = Some(r.session_id);
            if r.functions == 0 || r.revision != 0 {
                failures.push(format!("ready: not a fresh session (rev {})", r.revision));
            }
            if r.value != cfg.expected_initial {
                failures.push(format!(
                    "ready value {} != independent expected {}",
                    r.value, cfg.expected_initial
                ));
            }
        }
        other => return Err(format!("expected ready, got {other:?}")),
    }

    for (i, e) in cfg.edits.iter().enumerate() {
        let req_id = (i + 1) as u64;
        // The judged window opens at ENQUEUE (IPC included) and closes at
        // the parent's verification of the published result.
        let enqueued = Instant::now();
        if let Err(e) = link.send(&WorkerRequest::Edit(EditOp {
            req_id,
            start: e.start,
            end: e.end,
            insert: e.insert.clone(),
        })) {
            // A worker that died mid-stream leaves MISSING samples — the
            // completeness gate reddens below (V-09 缺样本), never a
            // silently shrunk campaign.
            failures.push(format!("edit {req_id}: worker stream lost (send): {e}"));
            break;
        }
        let reply = match link.recv() {
            Ok(r) => r,
            Err(e) => {
                failures.push(format!("edit {req_id}: worker stream lost: {e}"));
                break;
            }
        };
        let verified = enqueued.elapsed().as_secs_f64() * 1000.0;
        let WorkerReply::EditResult(r) = reply else {
            return Err(format!("edit {req_id}: expected a result, got {reply:?}"));
        };
        if r.req_id != req_id {
            failures.push(format!("edit {req_id}: reply carried req_id {}", r.req_id));
        }
        let correct = r.value == Some(e.expected);
        if !r.ok {
            failures.push(format!(
                "edit {req_id}: refused: {:?} {:?}",
                r.non_fast_path_reason, r.diagnostics
            ));
        } else if !correct {
            failures.push(format!(
                "edit {req_id}: published {:?} != independent expected {}",
                r.value, e.expected
            ));
        }
        // Structural deltas over the UNCHANGED functions, from cumulative
        // counter snapshots (the changed function's own stage work is
        // expected; everything else must be zero).
        let (unaffected_stage_total, whole_module_total, reparsed) =
            structural_deltas(&prev_counters, &r.counters, r.changed_fn.as_deref());
        // The structural GATES (§5.5): unchanged functions do zero stage
        // work, no whole-module axis moves, the install set is exactly
        // one function. `reparsed` is EVIDENCE (单列): a bounded rescan
        // window may legitimately cover an adjacent decl's tokens.
        let structural_ok_here = r.ok
            && unaffected_stage_total == 0
            && whole_module_total == 0
            && r.install_set.len() == 1;
        if r.ok && !structural_ok_here {
            failures.push(format!(
                "edit {req_id}: structural gate red (unaffected {unaffected_stage_total}, whole {whole_module_total}, install {})",
                r.install_set.len()
            ));
        }
        worker_peak = worker_peak.max(r.worker_peak_bytes);
        prev_counters = r.counters.clone();
        samples.push(JudgeSample {
            req_id,
            edit_to_verified_ms: verified,
            expected: e.expected,
            value: r.value,
            correct,
            ok: r.ok,
            revision: r.revision,
            worker_edit_to_result_us: r.worker_edit_to_result_us,
            worker_peak_bytes: r.worker_peak_bytes,
            unaffected_stage_total,
            whole_module_total,
            install_set_len: r.install_set.len(),
            reparsed_decls: reparsed,
        });
    }

    // Best-effort close: after a recorded stream loss the worker may
    // already be gone — the completeness failure above owns the verdict.
    if link.send(&WorkerRequest::Close(CloseOp {})).is_ok() {
        let _ = link.recv();
    }
    let parent_peak_bytes = parent_peak.stop();
    let _ = link.child.wait();

    // Completeness: one sample per scripted edit, none skipped.
    if samples.len() != cfg.edits.len() {
        failures.push(format!(
            "missing samples: {} of {}",
            samples.len(),
            cfg.edits.len()
        ));
    }
    let all_correct = samples.iter().all(|s| s.ok && s.correct);
    let structural_ok = samples.iter().all(|s| {
        s.ok && s.unaffected_stage_total == 0 && s.whole_module_total == 0 && s.install_set_len == 1
    });
    let mut times: Vec<f64> = samples.iter().map(|s| s.edit_to_verified_ms).collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let (p50, p95, p99) = (
        percentile(&times, 0.50),
        percentile(&times, 0.95),
        percentile(&times, 0.99),
    );
    // The dual gate. gate_ms == 0 is the V-09 countercontrol and MUST
    // fail — a zero gate can never pass by accident.
    let positive_gate = cfg.gate_ms > 0.0;
    let complete = samples.len() == cfg.edits.len();
    let gate_pass = failures.is_empty()
        && all_correct
        && structural_ok
        && complete
        && positive_gate
        && p95 < cfg.gate_ms;
    if !positive_gate {
        failures.push(format!("gate-ms {} is not a positive bound", cfg.gate_ms));
    }
    let Some(session_id) = session_id else {
        return Err("ready handshake never completed".to_string());
    };
    Ok(JudgeReport {
        session_id,
        samples,
        p50_ms: p50,
        p95_ms: p95,
        p99_ms: p99,
        gate_ms: cfg.gate_ms,
        gate_pass,
        all_correct,
        structural_ok,
        worker_peak_bytes: worker_peak,
        parent_peak_bytes,
        failures,
    })
}

impl JudgeReport {
    /// The RAW observation stream: one JSON line per judged sample
    /// (kept in full — failures included; the report never replaces raw).
    pub fn raw_jsonl(&self) -> String {
        self.samples
            .iter()
            .map(|s| serde_json::to_string(s).unwrap_or_default())
            .collect::<Vec<_>>()
            .join(
                "
",
            )
    }
}

/// Sum the stage-axis deltas of every function OTHER than the changed
/// one, plus the whole-module axes and the rescan evidence.
fn structural_deltas(
    prev: &BTreeMap<String, u64>,
    now: &BTreeMap<String, u64>,
    changed: Option<&str>,
) -> (u64, u64, u64) {
    let mut unaffected = 0u64;
    let mut whole = 0u64;
    let mut reparsed = 0u64;
    for (k, v) in now {
        let d = v.saturating_sub(prev.get(k).copied().unwrap_or(0));
        if d == 0 {
            continue;
        }
        if k.starts_with("whole_module_") {
            whole += d;
        } else if k == "syntax_reparsed_decls" {
            reparsed = d; // per-edit evidence: exactly the edited decl
        } else if k == "syntax_reused_decls" || k == "syntax_rescan_bytes" {
            // evidence axes, not gates
        } else if let Some(stage) = k.split(':').next() {
            if crate::session::STAGES.contains(&stage) {
                // stage counter "<stage>:<fn>" — the changed function's
                // own work is expected; any other function moving is a
                // structural violation.
                let owner = k.split_once(':').map(|(_, f)| f).unwrap_or("");
                if Some(owner) != changed {
                    unaffected += d;
                }
            }
        }
    }
    (unaffected, whole, reparsed)
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// The persistent worker link: one child, one buffered reader (a fresh
/// BufReader per read could drop buffered-ahead bytes — never create one
/// per line).
struct WorkerLink {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl WorkerLink {
    fn spawn(worker_exe: &Path) -> Result<WorkerLink, String> {
        let mut child = Command::new(worker_exe)
            .arg("session-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("spawn worker {}: {e}", worker_exe.display()))?;
        let reader = BufReader::new(child.stdout.take().ok_or("worker stdout closed")?);
        Ok(WorkerLink { child, reader })
    }

    fn send(&mut self, req: &WorkerRequest) -> Result<(), String> {
        let line = serde_json::to_string(req).map_err(|e| e.to_string())?;
        let stdin = self.child.stdin.as_mut().ok_or("worker stdin closed")?;
        writeln!(stdin, "{line}").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())
    }

    fn recv(&mut self) -> Result<WorkerReply, String> {
        let mut line = String::new();
        let n = self
            .reader
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("worker closed the protocol stream".to_string());
        }
        serde_json::from_str(line.trim()).map_err(|e| format!("bad reply `{}`: {e}", line.trim()))
    }
}
