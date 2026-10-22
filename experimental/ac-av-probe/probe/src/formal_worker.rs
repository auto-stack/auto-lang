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
    /// B3-C03: the identity of every function's dispatched payload on the
    /// INITIAL snapshot — the parent's cross-revision stability baseline.
    #[serde(default)]
    pub dispatched: Vec<DispatchedIdentity>,
}

/// B3-C03 (F-02 axis at the real consumer): one function's dispatched
/// payload identity — the object address and generation of the
/// `FunctionCode` Arc the AV's unit dispatch actually reads. An
/// unchanged function must keep the SAME (address, generation) across
/// revisions; only the install target may change.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DispatchedIdentity {
    pub name: String,
    pub ptr: u64,
    pub generation: u64,
}

/// Worker → parent: one publication — the applied edit, the CURRENT
/// revision's verified result, the worker-internal phase timings (for
/// attribution only; the judged window is the PARENT's), the cumulative
/// counter snapshot (the judge takes deltas), the worker's sampled
/// memory peak (the worker line of the dual-line budget), and the
/// dispatched-payload identities (B3-C03).
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
    #[serde(default)]
    pub dispatched: Vec<DispatchedIdentity>,
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
/// window (ms — B3-C04: the endpoint is taken AFTER every parent-side
/// check of this request has completed) plus the FULL observation the
/// evidence retains (worker phases, target, diagnostics, the per-edit
/// counter delta, and the dispatched-payload identities).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JudgeSample {
    pub req_id: u64,
    pub edit_to_verified_ms: f64,
    pub expected: i32,
    pub value: Option<i32>,
    pub correct: bool,
    pub ok: bool,
    pub revision: u64,
    pub revision_ok: bool,
    pub worker_edit_us: u64,
    pub worker_edit_to_result_us: u64,
    pub changed_fn: Option<String>,
    pub install_set: Vec<String>,
    /// B3-C02: the parent's INDEPENDENT expected install target (def id).
    pub expected_install: String,
    pub install_ok: bool,
    /// B3-C03: every function other than the install target kept its
    /// dispatched (address, generation) identity; the name set is stable.
    pub identity_ok: bool,
    pub diagnostics: Vec<String>,
    pub non_fast_path_reason: Option<String>,
    /// The per-edit counter delta (stage axes + evidence axes).
    pub counters_delta: BTreeMap<String, u64>,
    pub worker_peak_bytes: u64,
    pub dispatched: Vec<DispatchedIdentity>,
    /// Structural deltas over the UNCHANGED functions (stage sums,
    /// whole-module axes, install-set size, rescan evidence) computed by
    /// the judge from cumulative counter snapshots — excluding by the
    /// PARENT's expected target, never the worker's self-report.
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
    /// B3-C04: the initialization window (init → validated ready),
    /// single-listed and EXCLUDED from the edit statistics.
    pub init_ms: f64,
    /// B3-C04: input / worker-binary / configuration identity for replay.
    pub worker_exe: String,
    pub worker_identity: String,
    pub input_identity: String,
    pub entry: String,
    pub caps: Vec<String>,
    pub requested_revisions: usize,
}

/// The judge's expected-edit script (independent of the worker). B3-C02:
/// `expected_install` is the PARENT's own expectation of the exact
/// install target (def id) — the publication gate binds revision
/// progression (revision == req_id) and this target; the worker's
/// self-reported `changed_fn`/`install_set` must MATCH it, and the
/// stage-delta exclusion and the identity exemption use it too.
#[derive(Clone, Debug)]
pub struct JudgeEdit {
    pub start: u64,
    pub end: u64,
    pub insert: String,
    pub expected: i32,
    pub expected_install: String,
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
/// correctness, revision progression, install target, dispatched-payload
/// identity, structure, sample completeness, gate — is evaluated from
/// the PARENT side against independent expectations; the worker's own
/// numbers are attribution, never the verdict. B3-C04: each sample's
/// window closes only after ALL parent-side checks of that request have
/// completed (the endpoint is the publication, not the recv).
pub fn run_formal_session_judge(
    worker_exe: &Path,
    cfg: &JudgeConfig,
) -> Result<JudgeReport, String> {
    let mut link = WorkerLink::spawn(worker_exe)?;
    let parent_peak = crate::mempeek::PeakSampler::start(20);
    // B3-C04: replayable input / binary identity.
    let worker_identity = file_identity(worker_exe)?;
    let input_identity = format!(
        "len={} fnv64={:016x}",
        cfg.src.len(),
        fnv64(cfg.src.as_bytes())
    );

    let mut failures: Vec<String> = Vec::new();
    let mut samples: Vec<JudgeSample> = Vec::with_capacity(cfg.edits.len());
    let mut prev_counters: BTreeMap<String, u64> = BTreeMap::new();
    let mut prev_identity: BTreeMap<String, (u64, u64)>;
    let mut worker_peak = 0u64;
    let session_id: Option<u64>;

    // Ready handshake: one initialization, one constant worker identity,
    // the initial value judged against the independent expectation. The
    // init window is SINGLE-LISTED (init_ms) and excluded from the edit
    // statistics (§5.5 initialization is not a sample).
    let init_started = Instant::now();
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
            prev_identity = r
                .dispatched
                .iter()
                .map(|d| (d.name.clone(), (d.ptr, d.generation)))
                .collect();
            if prev_identity.len() != r.dispatched.len() {
                failures.push("ready: duplicate dispatched identities".into());
            }
        }
        other => return Err(format!("expected ready, got {other:?}")),
    }
    let init_ms = init_started.elapsed().as_secs_f64() * 1000.0;

    for (i, e) in cfg.edits.iter().enumerate() {
        let req_id = (i + 1) as u64;
        // The judged window opens at ENQUEUE (IPC included) and closes at
        // the parent's COMPLETED verification of the publication.
        let enqueued = Instant::now();
        if let Err(err) = link.send(&WorkerRequest::Edit(EditOp {
            req_id,
            start: e.start,
            end: e.end,
            insert: e.insert.clone(),
        })) {
            // A worker that died mid-stream leaves MISSING samples — the
            // completeness gate reddens below (V-09 缺样本), never a
            // silently shrunk campaign.
            failures.push(format!("edit {req_id}: worker stream lost (send): {err}"));
            break;
        }
        let reply = match link.recv() {
            Ok(r) => r,
            Err(err) => {
                failures.push(format!("edit {req_id}: worker stream lost: {err}"));
                break;
            }
        };
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
        // B3-C02: the publication's revision must be the CURRENT one —
        // the script's own progression (revision 0 at ready, +1 per
        // applied edit) is the parent's independent expectation.
        let revision_ok = r.ok && r.revision == req_id;
        if r.ok && !revision_ok {
            failures.push(format!(
                "edit {req_id}: published revision {} != expected {req_id}",
                r.revision
            ));
        }
        // B3-C02: the install set must be EXACTLY the parent's expected
        // target (the def id, e.g. `d_helper`) — cardinality proves
        // nothing about identity. The worker's self-reported `changed_fn`
        // (a SOURCE name per the pipeline's convention) must agree with
        // the same expectation, and the stage-delta exclusion below keys
        // on the source name too (`<stage>:<fn>` counters).
        let expected_owner = e
            .expected_install
            .strip_prefix("d_")
            .unwrap_or(e.expected_install.as_str());
        let install_ok = r.ok
            && r.install_set.len() == 1
            && r.install_set[0] == e.expected_install
            && r.changed_fn.as_deref() == Some(expected_owner);
        if r.ok && !install_ok {
            failures.push(format!(
                "edit {req_id}: install target {:?}/changed {:?} != expected {} ({})",
                r.install_set, r.changed_fn, e.expected_install, expected_owner
            ));
        }
        // B3-C04: the per-edit counter delta — computed BEFORE the
        // identity gate (which consults the reuse evidence) and retained
        // as the observation of what actually moved in THIS revision.
        let counters_delta: BTreeMap<String, u64> = r
            .counters
            .iter()
            .filter_map(|(k, v)| {
                let d = v.saturating_sub(prev_counters.get(k).copied().unwrap_or(0));
                (d > 0).then_some((k.clone(), d))
            })
            .collect();
        // B3-C03: dispatched-payload identity stability, observed at the
        // real consumer (the identities `run` recorded). Every function
        // OTHER than the expected install target must dispatch the SAME
        // (object address, generation) as the previous publication; the
        // name set must be stable; and the install target itself must
        // have TAKEN A NEW OBJECT unless this revision was a legitimate
        // content hit (`reuse_hit` moved). A caller that executes some
        // other program (e.g. a deep copy) without going through the
        // session's `run` leaves the record at the PREVIOUS revision —
        // caught here by the target rule even though values and stage
        // counters stay correct.
        let cur_identity: BTreeMap<String, (u64, u64)> = r
            .dispatched
            .iter()
            .map(|d| (d.name.clone(), (d.ptr, d.generation)))
            .collect();
        let reuse_hit_delta = counters_delta.get("reuse_hit").copied().unwrap_or(0);
        let identity_ok = r.ok
            && cur_identity.len() == r.dispatched.len()
            && cur_identity.len() == prev_identity.len()
            && cur_identity
                .iter()
                .zip(prev_identity.iter())
                .all(|(a, b)| a.0 == b.0)
            && cur_identity.iter().all(|(name, id)| {
                name.as_str() == e.expected_install.as_str() || prev_identity.get(name) == Some(id)
            })
            && (cur_identity.get(&e.expected_install) != prev_identity.get(&e.expected_install)
                || reuse_hit_delta >= 1);
        if r.ok && !identity_ok {
            let target_stale = cur_identity.get(&e.expected_install)
                == prev_identity.get(&e.expected_install)
                && reuse_hit_delta == 0;
            failures.push(format!(
                "edit {req_id}: dispatched payload identity gate red ({}; expected target {} {})",
                if target_stale {
                    "the target's dispatched object is the PREVIOUS revision's — the reported identities did not come from this revision's real run"
                } else {
                    "a function other than the install target changed its dispatched payload object"
                },
                e.expected_install,
                if reuse_hit_delta >= 1 {
                    "(content hit)"
                } else {
                    "(new object required)"
                }
            ));
        }
        // Structural deltas over the UNCHANGED functions, from cumulative
        // counter snapshots, EXCLUDING BY THE PARENT'S expected target
        // (source-name key) — never the worker's self-report.
        let (unaffected_stage_total, whole_module_total, reparsed) =
            structural_deltas(&prev_counters, &r.counters, Some(expected_owner));
        // The structural GATES (§5.5): unchanged functions do zero stage
        // work, no whole-module axis moves, the install set is exactly
        // the expected target, identities are stable. `reparsed` is
        // EVIDENCE (单列): a bounded rescan window may legitimately cover
        // an adjacent decl's tokens.
        let structural_ok_here = r.ok
            && revision_ok
            && install_ok
            && identity_ok
            && unaffected_stage_total == 0
            && whole_module_total == 0;
        if r.ok && !structural_ok_here {
            failures.push(format!(
                "edit {req_id}: structural gate red (revision_ok {revision_ok}, install_ok {install_ok}, identity_ok {identity_ok}, unaffected {unaffected_stage_total}, whole {whole_module_total})"
            ));
        }
        worker_peak = worker_peak.max(r.worker_peak_bytes);
        // The judged window CLOSES HERE: every parent-side check of this
        // request (type, req_id, value, revision, install target,
        // identity, structural deltas) has completed — the publication.
        let verified = enqueued.elapsed().as_secs_f64() * 1000.0;
        prev_counters = r.counters.clone();
        prev_identity = cur_identity;
        samples.push(JudgeSample {
            req_id,
            edit_to_verified_ms: verified,
            expected: e.expected,
            value: r.value.clone(),
            correct,
            ok: r.ok,
            revision: r.revision,
            revision_ok,
            worker_edit_us: r.worker_edit_us,
            worker_edit_to_result_us: r.worker_edit_to_result_us,
            changed_fn: r.changed_fn.clone(),
            install_set: r.install_set.clone(),
            expected_install: e.expected_install.clone(),
            install_ok,
            identity_ok,
            diagnostics: r.diagnostics.clone(),
            non_fast_path_reason: r.non_fast_path_reason.clone(),
            counters_delta,
            worker_peak_bytes: r.worker_peak_bytes,
            dispatched: r.dispatched.clone(),
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
        s.ok && s.revision_ok
            && s.install_ok
            && s.identity_ok
            && s.unaffected_stage_total == 0
            && s.whole_module_total == 0
            && s.install_set.len() == 1
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
        parent_peak_bytes: parent_peak_bytes,
        failures,
        init_ms,
        worker_exe: worker_exe.display().to_string(),
        worker_identity,
        input_identity,
        entry: cfg.entry.clone(),
        caps: cfg.caps.clone(),
        requested_revisions: cfg.edits.len(),
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

    /// B3-C04: durable evidence. Writes the FULL report (identity,
    /// init window, verdicts, failures) and every raw sample — success
    /// and failure alike — under `dir`, returning the report path. The
    /// formal entry never generates-and-drops its observations.
    pub fn write_evidence(&self, dir: &Path) -> Result<std::path::PathBuf, String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("evidence dir: {e}"))?;
        let report_path = dir.join("formal-report.json");
        let body = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&report_path, body).map_err(|e| format!("write report: {e}"))?;
        let samples_path = dir.join("formal-samples.jsonl");
        let mut raw = self.raw_jsonl();
        raw.push('\n');
        std::fs::write(&samples_path, raw).map_err(|e| format!("write samples: {e}"))?;
        Ok(report_path)
    }
}

/// B3-C04: a compact replayable file identity (size + FNV-1a over the
/// bytes) for the worker binary.
fn file_identity(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(format!("size={} fnv64={:016x}", bytes.len(), fnv64(&bytes)))
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
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
