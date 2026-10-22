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
    /// B3-C03: the record's AV execution sequence vs the process-current
    /// one at publication — equal ⇒ the published record IS the latest
    /// execution; a later redundant execution (e.g. a deep copy run
    /// after the recorded one) makes them differ and the gate reddens.
    #[serde(default)]
    pub run_seq: u64,
    #[serde(default)]
    pub av_seq: u64,
}

/// B3-C03 (F-02 axis at the real consumer): one function's dispatched
/// payload identity — the object address, generation, and content digest
/// of the `FunctionCode` the AV's unit dispatch actually read, taken
/// from THE execution's trace. An unchanged function must keep the SAME
/// (address, generation, content) across revisions; only the install
/// target may change (or legitimately reuse on a content hit).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DispatchedIdentity {
    pub name: String,
    pub ptr: u64,
    pub generation: u64,
    #[serde(default)]
    pub content: u64,
    /// B3-C03 resident-object binding: the dispatched payload IS the
    /// session's resident code object (live-address identity — a deep
    /// copy can never claim this). Absent (old payloads) = false =
    /// fail-closed.
    #[serde(default)]
    pub resident: bool,
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
    /// B3-C04 (§5.5 phase walls): the edit's per-phase wall times in µs
    /// at the session's real production seams (absent key = the phase
    /// did not run — e.g. a content hit performs no lower/emit work).
    #[serde(default)]
    pub phase_walls: BTreeMap<String, u64>,
    /// B3-C04 (§5.5 phase IO): the edit's real IO bytes — rescan bytes;
    /// store payload bytes when a store is attached (absent = not
    /// performed; `store_attached=false` ⇒ the warm edit did NO disk
    /// IO, structurally).
    #[serde(default)]
    pub phase_io: BTreeMap<String, u64>,
    #[serde(default)]
    pub store_attached: bool,
    /// B3-C03: same record-vs-current AV execution pairing as the ready
    /// message — the published identities must be the LATEST execution's.
    #[serde(default)]
    pub run_seq: u64,
    #[serde(default)]
    pub av_seq: u64,
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
    /// B3-C04: the edit's per-phase walls (µs) and real IO bytes.
    pub phase_walls: BTreeMap<String, u64>,
    pub phase_io: BTreeMap<String, u64>,
    pub store_attached: bool,
    /// B3-C03: the publication's record-vs-current AV execution pairing.
    pub run_seq: u64,
    pub av_seq: u64,
    pub seq_ok: bool,
    /// B3-C03: every dispatched payload was the RESIDENT object.
    pub resident_ok: bool,
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

/// B3-C03 directory gate (pure — component-testable): the ready phase's
/// dispatched-identity list must be a COMPLETE directory — one entry per
/// function the worker reports (`functions`), no duplicates, and the
/// entry function present. A missing observation (e.g. an omitted
/// `d_main`) must refuse PASS, not silently compare a truncated set.
pub fn ready_directory_gate(
    dispatched: &[DispatchedIdentity],
    functions: usize,
    entry: &str,
) -> Result<(), String> {
    if dispatched.len() != functions {
        return Err(format!(
            "dispatched directory incomplete: {} identities for {functions} functions",
            dispatched.len()
        ));
    }
    let mut names: Vec<&str> = dispatched.iter().map(|d| d.name.as_str()).collect();
    names.sort_unstable();
    if names.windows(2).any(|w| w[0] == w[1]) {
        return Err("duplicate dispatched identities".to_string());
    }
    if !names.contains(&entry) {
        return Err(format!(
            "entry `{entry}` missing from the dispatched directory"
        ));
    }
    Ok(())
}

/// B3-C03 identity gate (pure — component-testable): across one
/// publication, every function OTHER than the install target must keep
/// its dispatched (address, generation, content); the name set must be
/// unchanged; and the target must have taken a DIFFERENT payload
/// (address/generation/content) unless this revision was a legitimate
/// content hit (`reuse_hit` moved for the target's source-name key).
pub fn edit_identity_gate(
    prev: &BTreeMap<String, (u64, u64, u64)>,
    cur: &BTreeMap<String, (u64, u64, u64)>,
    raw_len: usize,
    expected_install: &str,
    reuse_hit_delta: u64,
) -> Result<(), String> {
    if cur.len() != raw_len {
        return Err("duplicate dispatched identities".to_string());
    }
    if cur.len() != prev.len() || !cur.keys().zip(prev.keys()).all(|(a, b)| a == b) {
        return Err("dispatched directory changed between revisions".to_string());
    }
    for (name, id) in cur {
        if name.as_str() == expected_install {
            continue;
        }
        if prev.get(name) != Some(id) {
            return Err(format!(
                "dispatched payload identity changed for `{name}` (not the install target)"
            ));
        }
    }
    let target_unchanged = cur.get(expected_install) == prev.get(expected_install);
    if target_unchanged && reuse_hit_delta == 0 {
        return Err(format!(
            "the install target `{expected_install}` dispatched the PREVIOUS revision's payload with no recorded content hit — the identities did not come from this revision's real execution"
        ));
    }
    Ok(())
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
    let mut prev_identity: BTreeMap<String, (u64, u64, u64)>;
    let mut worker_peak = 0u64;
    let session_id: Option<u64>;

    // Ready handshake: one initialization, one constant worker identity,
    // the initial value judged against the independent expectation. The
    // init window is SINGLE-LISTED (init_ms) and excluded from the edit
    // statistics (§5.5 initialization is not a sample).
    let init_started = Instant::now();
    // B3-C04 R3: handshake SEND/RECEIVE/PARSE failures feed the SAME
    // failure-report path as every other failure — the caller always
    // gets a report (0 samples, the handshake reason, input and worker
    // identity) it can persist before any verdict. Only a spawn failure
    // stays Err: nothing ran, there is no session to report.
    let handshake: Result<WorkerReply, String> = (|| {
        link.send(&WorkerRequest::Init(InitOp {
            src: cfg.src.clone(),
            entry: cfg.entry.clone(),
            caps: cfg.caps.clone(),
        }))
        .map_err(|e| format!("init send failed: {e}"))?;
        link.recv().map_err(|e| format!("init receive failed: {e}"))
    })();
    let mut handshake_failed = false;
    match handshake {
        Ok(WorkerReply::Ready(r)) => {
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
            // B3-C03: the ready directory must be COMPLETE — identities
            // for every reported function, no duplicates, entry present.
            if let Err(e) = ready_directory_gate(&r.dispatched, r.functions, &cfg.entry) {
                failures.push(format!("ready: {e}"));
            }
            // B3-C03 resident-object binding: every dispatched payload
            // must BE the session's resident object.
            if let Some(bad) = r.dispatched.iter().find(|d| !d.resident) {
                failures.push(format!(
                    "ready: dispatched payload `{}` is not the resident object (a copy cannot claim residency)",
                    bad.name
                ));
            }
            // B3-C03: the ready record must be the LATEST AV execution.
            if r.run_seq != r.av_seq {
                failures.push(format!(
                    "ready: published record seq {} != current AV seq {} — a later execution is not the published one",
                    r.run_seq, r.av_seq
                ));
            }
            prev_identity = r
                .dispatched
                .iter()
                .map(|d| (d.name.clone(), (d.ptr, d.generation, d.content)))
                .collect();
        }
        // B3-C04: an abnormal ready reply still yields a REPORT (with
        // the failure and zero samples) so the caller can persist
        // evidence — never a bare Err that drops everything.
        Ok(other) => {
            failures.push(format!("ready: expected a ready reply, got {other:?}"));
            session_id = Some(0);
            prev_identity = BTreeMap::new();
            handshake_failed = true;
        }
        // B3-C04 R3: EOF / bad JSON / send failure on the handshake —
        // the same reportable failure, with the stream's own reason.
        Err(e) => {
            failures.push(format!("ready: {e}"));
            session_id = Some(0);
            prev_identity = BTreeMap::new();
            handshake_failed = true;
        }
    }
    let init_ms = init_started.elapsed().as_secs_f64() * 1000.0;

    for (i, e) in cfg.edits.iter().enumerate() {
        if handshake_failed {
            // Nothing to judge — the completeness failure above owns the
            // verdict; the report keeps the handshake reason.
            break;
        }
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
        // B3-C04: an abnormal reply type records the failure and keeps
        // the already-collected samples — the report (and the caller's
        // evidence write) still happens.
        let WorkerReply::EditResult(r) = reply else {
            failures.push(format!(
                "edit {req_id}: expected a result reply, got {reply:?}"
            ));
            break;
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
        // real consumer (the identities THE execution's trace recorded —
        // address, generation AND content digest). Every function other
        // than the parent's expected target must dispatch the SAME
        // payload; the name set must be unchanged; the target must have
        // taken a DIFFERENT payload unless this revision was a legal
        // content hit (`reuse_hit:{owner}` — the producer's real
        // per-function key — moved).
        let cur_identity: BTreeMap<String, (u64, u64, u64)> = r
            .dispatched
            .iter()
            .map(|d| (d.name.clone(), (d.ptr, d.generation, d.content)))
            .collect();
        // B3-C03 resident-object binding: the executed payloads must BE
        // the session's resident code objects (live-address identity —
        // a deep copy can never claim residency, however its heap
        // address recycles).
        let resident_ok = r.ok && r.dispatched.iter().all(|d| d.resident);
        if r.ok && !resident_ok {
            let bad = r
                .dispatched
                .iter()
                .find(|d| !d.resident)
                .map(|d| d.name.clone())
                .unwrap_or_default();
            failures.push(format!(
                "edit {req_id}: dispatched payload `{bad}` is not the resident object — the executed program is not the session's own code"
            ));
        }
        // B3-C03: the published record must be the LATEST AV execution —
        // a redundant later execution (e.g. a deep copy run after the
        // recorded one) leaves the sequence numbers unequal.
        let seq_ok = r.ok && r.run_seq == r.av_seq;
        if r.ok && !seq_ok {
            failures.push(format!(
                "edit {req_id}: published record seq {} != current AV seq {} — the identities did not come from the latest real execution",
                r.run_seq, r.av_seq
            ));
        }
        let reuse_hit_delta = counters_delta
            .get(&format!("reuse_hit:{expected_owner}"))
            .copied()
            .unwrap_or(0);
        let identity_ok = r.ok
            && edit_identity_gate(
                &prev_identity,
                &cur_identity,
                r.dispatched.len(),
                &e.expected_install,
                reuse_hit_delta,
            )
            .is_ok();
        if r.ok && !identity_ok {
            failures.push(format!(
                "edit {req_id}: {}",
                edit_identity_gate(
                    &prev_identity,
                    &cur_identity,
                    r.dispatched.len(),
                    &e.expected_install,
                    reuse_hit_delta
                )
                .unwrap_err()
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
            && resident_ok
            && seq_ok
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
            phase_walls: r.phase_walls.clone(),
            phase_io: r.phase_io.clone(),
            store_attached: r.store_attached,
            run_seq: r.run_seq,
            av_seq: r.av_seq,
            seq_ok,
            resident_ok,
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
            && s.resident_ok
            && s.seq_ok
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
    // B3-C04: even an aborted handshake yields a report (session_id 0)
    // so the caller's evidence write always has this run's artifacts.
    let session_id = session_id.unwrap_or(0);
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
