//! PLAN-798 T-07: the `ac-av/core-run-v1` probe recipe — formal AutoDown
//! execution of the av probe through the unified probe/run seam landed by
//! PLAN-796 (which itself rides the PLAN-788 TestCase v2 runner).
//!
//! Same contract shape as `sema.rs` (the seam's first recipe): the case
//! declares a recipe + one declarative `run` step + an `av` binding
//! (entry/caps); this adapter spawns the plan's own worker
//! (`av_probe_worker case-serve`, envelope `avworker/1`) under Job
//! containment, admits the envelope/nonce/request-hash/recipe/version
//! echoes, and judges the observed outcome PARENT-SIDE against the
//! declared expectations. The worker never sees the expected outcomes.
//!
//! Recipe semantics (v1): one run = the full front-door → HIR → MIR →
//! ABC → AV pipeline on the case source. Judgement faces:
//! - `completion: "success"` + `exit: <i32>` — AV Success value;
//! - `completion: { stage: "run", message: { contains: <trap code> } }` —
//!   a language trap (an executed-outcome, never an error completion);
//! - `completion: { stage: "parse" | "verify", code: <diag code> }` — a
//!   diagnosed admission/frontend/pipeline rejection.

use crate::test_cases::sema::containment;
use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, Completion, MessageMatcher, Stage};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;

use super::avm::Verdict;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

thread_local! {
    static MODULE_PRIVATE_SESSIONS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Isolate explicit sessions only for an independent module-case invocation.
/// Ordinary suite callers retain their cross-invocation restart contract.
pub fn with_module_private_sessions<T>(run: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            MODULE_PRIVATE_SESSIONS.with(|scope| scope.set(self.0));
        }
    }
    let _restore = Restore(MODULE_PRIVATE_SESSIONS.with(|scope| scope.replace(true)));
    run()
}

#[cfg(feature = "test-case-batch")]
pub mod parent_observations;
#[cfg(feature = "test-case-batch")]
pub mod test_batch;

const READ_CAP: usize = (4 << 20) + (64 << 10);
const STDERR_CAP: usize = 1 << 20;
const REQUEST_CAP: usize = 4 << 20;
const READER_GRACE: Duration = Duration::from_secs(2);
const SESSION_WALL: Duration = Duration::from_secs(60);
const BUILD_WALL: Duration = Duration::from_secs(300);

pub const AV_RECIPE: &str = "ac-av/core-run-v1";
/// PLAN-863: independent typed host-console recipe and profile.
pub const AV_CONSOLE_RECIPE: &str = "ac-av/console-run-v1";
pub const EXPECTED_AV_CONSOLE_CORE_VERSION: &str = "av.host-console/1";
/// PLAN-820: the data-profile recipe — record source → data MIR → AVAD →
/// AV through the same worker/executor contract; judgement stays
/// parent-side. Registered as its own recipe identity (append-only; the
/// core recipe contract is untouched).
pub const AV_DATA_RECIPE: &str = "ac-av/data-run-v1";
pub const EXPECTED_AV_DATA_CORE_VERSION: &str = "av.data/1";
/// PLAN-821: the owned-profile recipe (`av.owned/1`, profile id 3) —
/// text-record source → owned Typed MIR → AVAD v2 → AV with the arena
/// ledger face. Same append-only registration rules; the owned face is
/// judged parent-side against THIS crate's frozen expectations (result +
/// live=0 + alloc/free order), never the worker's word.
pub const AV_OWNED_RECIPE: &str = "ac-av/owned-run-v1";
pub const EXPECTED_AV_OWNED_CORE_VERSION: &str = "av.owned/1";
/// PLAN-822: the collections-profile recipe (`av.collections/1`, profile
/// id 4) — programmatic typed fixture -> collections Typed MIR -> AVAD v3
/// -> AV with the arena-ledger face. Same append-only registration rules;
/// the collections face is judged parent-side against THIS crate's frozen
/// expectations, never the worker's word.
pub const AV_COLLECTIONS_RECIPE: &str = "ac-av/collections-run-v1";
/// PLAN-864: the collections SOURCE recipe — real .at sources through the
/// collections frontend onto the frozen 822 execution (profile 4 consumed
/// unchanged; the 822 fixture case files stay byte-identical).
pub const AV_COLLECTIONS_SOURCE_RECIPE: &str = "ac-av/collections-source-run-v1";
/// PLAN-823 (R1-F02): the closures-profile av recipe (append-only
/// registration; earlier recipe contracts untouched).
pub const AV_CLOSURES_RECIPE: &str = "ac-av/closures-run-v1";
/// PLAN-865: source-backed closures cases run through the per-function
/// update session; the existing typed fixture recipe remains unchanged.
pub const AV_CLOSURES_SOURCE_RECIPE: &str = "ac-av/closures-source-run-v1";
pub const EXPECTED_AV_COLLECTIONS_CORE_VERSION: &str = "av.collections/1";
pub const AV_LOADED_RECIPE: &str = "ac-av/loaded-run-v1";

/// PLAN-824: the dynamic-profile recipe (`av.dynamic/1`, profile id 5) —
/// programmatic typed fixture -> dynamic Typed MIR -> AVAD v4 -> AV with
/// the arena-ledger + Dyn/portable face. Same append-only registration
/// rules; the dynamic face is judged parent-side against THIS crate's
/// frozen expectations, never the worker's word.
pub const AV_DYNAMIC_RECIPE: &str = "ac-av/dynamic-run-v1";
pub const EXPECTED_AV_DYNAMIC_CORE_VERSION: &str = "av.dynamic/1";
/// PLAN-826: the compact-wire recipe — the SAME scalar `av.core-i32/1`
/// profile (same core identity) over the AVAC wire generation
/// (`auto.abc.av-compact-core-i32/1`): codec-dimensioned cache namespace,
/// compact admission and AVAC persist/restore. Append-only registration;
/// earlier recipe contracts untouched. The wire face is judged parent-side
/// against the compact-codec counters, never the worker's word.
pub const AV_COMPACT_RECIPE: &str = "ac-av/compact-run-v1";

/// PLAN-866: the resident-online recipe (`av.core-i32/1` scalar profile) —
/// ONE continuously-running guest chain driven by fuel slices, edits
/// applied exactly at guest call-boundary safety points (prepare → armed
/// park → commit → swap-resolution resume). Append-only registration;
/// earlier recipe contracts untouched. Judged parent-side against the
/// finish step's outcome plus the structural resident counters.
pub const AV_RESIDENT_RECIPE: &str = "ac-av/resident-run-v1";
/// PLAN-862: the entry-unit execution profile the core recipe serves when a
/// check selects `av: { profile: "entry-unit" }`. It is the SAME worker and
/// the SAME recipe seam — a different SOURCE front door (`auto.source.
/// entry-unit.draft`), a different MIR/ABC identity (`av.entry-unit/1`,
/// `AVAU` v1) and the added Unit result channel.
pub const EXPECTED_AV_ENTRY_UNIT_CORE_VERSION: &str = "av.entry-unit/1";

/// Which registered av recipe a check runs under.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecipeFamily {
    Core,
    Console,
    Data,
    Owned,
    Collections,
    /// PLAN-864: real `.at` sources through the collections frontend onto
    /// the frozen 822 execution (entry = the source function name).
    CollectionsSource,
    /// PLAN-865: real `.at` source through closure lowering and the existing
    /// per-function closures update/link/runtime session.
    ClosuresSource,
    /// PLAN-866: the resident-online recipe — one continuously-running
    /// chain, edits at call-boundary safety points.
    Resident,
    Loaded,
    Closures,
    Dynamic,
    Compact,
}

impl RecipeFamily {
    pub fn recipe(&self) -> &'static str {
        match self {
            RecipeFamily::Core => AV_RECIPE,
            RecipeFamily::Console => AV_CONSOLE_RECIPE,
            RecipeFamily::Data => AV_DATA_RECIPE,
            RecipeFamily::Owned => AV_OWNED_RECIPE,
            RecipeFamily::Collections => AV_COLLECTIONS_RECIPE,
            RecipeFamily::CollectionsSource => AV_COLLECTIONS_SOURCE_RECIPE,
            RecipeFamily::ClosuresSource => AV_CLOSURES_SOURCE_RECIPE,
            RecipeFamily::Resident => AV_RESIDENT_RECIPE,
            RecipeFamily::Loaded => AV_LOADED_RECIPE,
            RecipeFamily::Closures => AV_CLOSURES_RECIPE,
            RecipeFamily::Dynamic => AV_DYNAMIC_RECIPE,
            RecipeFamily::Compact => AV_COMPACT_RECIPE,
        }
    }
    pub fn expected_core_version(&self) -> &'static str {
        match self {
            RecipeFamily::Core | RecipeFamily::Loaded => EXPECTED_AV_CORE_VERSION,
            RecipeFamily::Data => EXPECTED_AV_DATA_CORE_VERSION,
            RecipeFamily::Console => EXPECTED_AV_CONSOLE_CORE_VERSION,
            RecipeFamily::Owned => EXPECTED_AV_OWNED_CORE_VERSION,
            RecipeFamily::Collections => EXPECTED_AV_COLLECTIONS_CORE_VERSION,
            RecipeFamily::CollectionsSource => EXPECTED_AV_COLLECTIONS_CORE_VERSION,
            RecipeFamily::ClosuresSource => EXPECTED_AV_CLOSURES_CORE_VERSION,
            // PLAN-866: the resident recipe rides the scalar profile
            // identity (the 828 session pipeline).
            RecipeFamily::Resident => EXPECTED_AV_CORE_VERSION,
            RecipeFamily::Closures => EXPECTED_AV_CLOSURES_CORE_VERSION,
            RecipeFamily::Dynamic => EXPECTED_AV_DYNAMIC_CORE_VERSION,
            // PLAN-826: the compact wire is a WIRE generation of the same
            // scalar profile — the core identity is unchanged.
            RecipeFamily::Compact => EXPECTED_AV_CORE_VERSION,
        }
    }
}
/// PLAN-823: the closures worker mirrors the core identity at the same
/// profile identity (the avworker/1 protocol is profile-neutral).
pub const EXPECTED_AV_CLOSURES_CORE_VERSION: &str = "av.closures/1";
pub const AV_WORKER_PROTOCOL: &str = "avworker/1";
/// Mirrored worker identity expectations — admission constants; a worker
/// bump without a mirror update fails loudly.
pub const EXPECTED_AV_WORKER_VERSION: &str = "avworker/1.0";
pub const EXPECTED_AV_CORE_VERSION: &str = "av.core-i32/1";
/// PLAN-868: admission mirror of `auto_mir_lower::MIR_LOWERING_VERSION` —
/// every av recipe family lowers HIR→MIR through the shared crate; a
/// lowering contract bump without a mirror update fails the identity axis.
pub const EXPECTED_MIR_LOWERING_VERSION: &str = "4";

/// Current av toolchain identity (the axes stale adjudication binds probe
/// rows of THIS recipe to).
pub fn current_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    if let Ok(sha) = worker_exe_sha256() {
        m.insert("av-worker-exe-sha256".to_string(), sha);
    }
    m.insert(
        "av-worker-version".to_string(),
        EXPECTED_AV_WORKER_VERSION.to_string(),
    );
    m.insert("av-protocol".to_string(), AV_WORKER_PROTOCOL.to_string());
    m.insert("recipe".to_string(), AV_RECIPE.to_string());
    m.insert(
        "mir-lowering".to_string(),
        EXPECTED_MIR_LOWERING_VERSION.to_string(),
    );
    m
}

fn probe_toolchain_axes(
    exe_sha: &str,
    family: RecipeFamily,
    core_override: Option<&str>,
) -> Vec<(String, String)> {
    let mut v = vec![
        ("av-protocol".into(), AV_WORKER_PROTOCOL.into()),
        ("recipe".into(), family.recipe().into()),
        ("av-worker-exe-sha256".into(), exe_sha.to_string()),
        (
            "av-worker-version".into(),
            EXPECTED_AV_WORKER_VERSION.into(),
        ),
        (
            "av-core-version".into(),
            core_override
                .unwrap_or_else(|| family.expected_core_version())
                .to_string(),
        ),
        // PLAN-868: the lowering contract identity shared by all av
        // families — reported per row, mandatory (old reports lacking the
        // axis adjudicate STALE by declaration).
        ("mir-lowering".into(), EXPECTED_MIR_LOWERING_VERSION.into()),
    ];
    // PLAN-862: the entry-unit generation carries its own profile id (7) and
    // source profile in the receipt — the axis names what actually ran, so a
    // scalar receipt can never be confused with an entry-unit one.
    if core_override == Some(EXPECTED_AV_ENTRY_UNIT_CORE_VERSION) {
        v.push(("av-profile".into(), "7".into()));
        v.push(("av-source-profile".into(), "entry-unit".into()));
    }
    // PLAN-821 contract-decision §10: the owned tool-identity axis carries
    // the frozen profile id (3) explicitly (core/data receipts unchanged).
    if family == RecipeFamily::Console {
        v.push(("av-profile".into(), "8".into()));
        v.push(("av-source-profile".into(), "host-console".into()));
        v.push(("av-wire-codec".into(), "auto.abc.av-host-console/1".into()));
    }
    if family == RecipeFamily::Owned {
        v.push(("av-profile".into(), "3".into()));
    }
    // PLAN-826: the compact-wire tool-identity axis carries the AVAC wire
    // schema explicitly — the receipt names the wire generation the run
    // actually used (same scalar core identity as the loaded recipe).
    if family == RecipeFamily::Compact {
        v.push((
            "av-wire-codec".into(),
            "auto.abc.av-compact-core-i32/1".into(),
        ));
    }
    // PLAN-824: the dynamic tool-identity axis carries profile id 5.
    if family == RecipeFamily::Dynamic {
        v.push(("av-profile".into(), "5".into()));
    }
    if family == RecipeFamily::ClosuresSource {
        v.push(("av-profile".into(), "6".into()));
    }
    v
}

fn worker_exe_sha256() -> Result<String, String> {
    worker_file_sha256(&worker_binary()?)
}

fn worker_file_sha256(exe: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    // Re-read the executable on every admission: same-path replacement
    // must change the tool identity. Use the existing accelerated digest
    // implementation rather than the contract crate's debug-mode scalar
    // reference; the SHA-256 bytes and admission policy stay identical.
    let bytes = std::fs::read(exe).map_err(|e| format!("read worker exe: {e}"))?;
    Ok(hex(&Sha256::digest(&bytes)))
}

#[cfg(all(test, feature = "test-ace"))]
mod worker_identity_tests {
    use super::*;

    #[test]
    fn av_worker_sha256_matches_contract_and_same_path_replacement() {
        let path = std::env::temp_dir().join(format!(
            "av798-hash-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        // Empty, known vector, block/padding boundaries and a multi-block
        // artifact all use the SAME path: no metadata/hash cache may hide
        // a replacement, even when its length remains unchanged.
        let mut inputs = vec![Vec::new(), b"abc".to_vec()];
        for len in [55, 56, 63, 64, 65, (1 << 20) + 17] {
            inputs.push((0..len).map(|i| (i % 251) as u8).collect());
        }
        inputs.push(vec![b'x'; 65]);
        inputs.push(vec![b'y'; 65]);
        for bytes in inputs {
            std::fs::write(&path, &bytes).unwrap();
            let observed = worker_file_sha256(&path).unwrap();
            assert_eq!(observed, hex(&sha256(&bytes)));
            if bytes == b"abc" {
                assert_eq!(
                    observed,
                    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                );
            }
        }
        std::fs::remove_file(&path).unwrap();
        assert!(worker_file_sha256(&path)
            .unwrap_err()
            .starts_with("read worker exe:"));
    }
}

/// Locate (and on first use build) the av worker binary. Env override
/// first (trusted host configuration binds the tool identity);
/// otherwise find `experimental/ac-av-probe` from this crate's source
/// root and build its worker release (bounded).
pub fn worker_binary() -> Result<PathBuf, String> {
    if std::env::var("AUTO_TEST_DOMAIN").as_deref() == Ok("legacy") {
        return Err(
            "ACE worker requested by a legacy test group; explicitly select the ACE group".into(),
        );
    }
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(locate_worker).clone()
}

fn locate_worker() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("AC_AV_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AC_AV_CASE_WORKER points at a missing file: {}",
            env_path
        ));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "av probe worker not located: no experimental/ac-av-probe above the crate and no AC_AV_CASE_WORKER override".to_string(),
            );
        };
        let m = d
            .join("experimental")
            .join("ac-av-probe")
            .join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    let ws_root = manifest.parent().expect("workspace root");
    let exe = ws_root
        .join("target")
        .join("release")
        .join(av_worker_exe_name());
    // F798-R3-10: ALWAYS build (a no-op incremental build is sub-second)
    // under the BUILD_WALL deadline — testing a stale binary that happens
    // to exist at the target path silently unbinds evidence from source.
    // F798-R15-05: the build runs under the SAME contained-executor
    // discipline as `call_worker`: suspend→Job→resume (dropping the job
    // reaps the whole tree), BOTH output streams drained by capped reader
    // threads (a full stdout pipe must surface as a stream-cap error, not
    // a backpressure wall timeout), ONE absolute deadline, and on breach
    // the TREE is killed and reaped — never a bare child.kill that leaves
    // linker/compiler descendants behind.
    // F798-R15-05 control seams (test-only, mirroring AV798_NATIVE_*):
    // an injectable cargo launcher + wall, so the stdout-cap/timeout/
    // kill-tree failure shapes are testable without a real build.
    let cargo = std::env::var("AV798_BUILD_CARGO").unwrap_or_else(|_| env!("CARGO").to_string());
    let build_wall = Duration::from_millis(
        std::env::var("AV798_BUILD_WALL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(BUILD_WALL.as_millis() as u64),
    );
    let build_started = Instant::now();
    let mut cmd = Command::new(&cargo);
    cmd.args([
        "build",
        "--release",
        "-p",
        "auto-av-probe",
        "--bin",
        "av_probe_worker",
    ])
    .current_dir(ws_root)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn cargo for the av worker: {e}"))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained av worker build tree".to_string(),
            );
        }
    };
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("resume cargo build: {e}"));
    }
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;
    // Capped reader threads for BOTH streams — a capped-out stream fails
    // the build as a REPORTED stream error, never a silently blocked pipe.
    fn capped_reader<R: std::io::Read + Send + 'static>(
        mut pipe: R,
        tx: std::sync::mpsc::Sender<Result<Vec<u8>, usize>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 16384];
            let mut exceeded = false;
            loop {
                match pipe.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > STDERR_CAP {
                            exceeded = true;
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = tx.send(if exceeded { Err(buf.len()) } else { Ok(buf) });
        })
    }
    let (out_tx, out_rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let (err_tx, err_rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let out_reader = capped_reader(child.stdout.take().expect("stdout"), out_tx);
    let err_reader = capped_reader(child.stderr.take().expect("stderr"), err_tx);
    let deadline_at = build_started + build_wall;
    let mut timed_out = false;
    let mut status: Option<std::process::ExitStatus> = None;
    let mut stdout_res: Option<Result<Vec<u8>, usize>> = None;
    let mut stderr_res: Option<Result<Vec<u8>, usize>> = None;
    fn stream_capped(
        out: &Option<Result<Vec<u8>, usize>>,
        err: &Option<Result<Vec<u8>, usize>>,
    ) -> bool {
        matches!(out, Some(Err(_))) || matches!(err, Some(Err(_)))
    }
    loop {
        if Instant::now() > deadline_at {
            timed_out = true;
            break;
        }
        if stdout_res.is_none() {
            stdout_res = out_rx.try_recv().ok();
        }
        if stderr_res.is_none() {
            stderr_res = err_rx.try_recv().ok();
        }
        match child.try_wait() {
            Ok(st @ Some(_)) => {
                status = st;
                break;
            }
            Ok(None) => {}
            Err(e) => return Err(format!("poll cargo build: {e}")),
        }
        if stream_capped(&stdout_res, &stderr_res) {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if timed_out || stream_capped(&stdout_res, &stderr_res) || status.is_none() {
        // Kill the whole TREE (closing the job reaps every descendant),
        // reap the direct child, then drain the bounded readers.
        let _ = child.kill();
        drop(job.take());
        let _ = child.wait();
        let _ = out_rx.recv_timeout(READER_GRACE);
        let _ = err_rx.recv_timeout(READER_GRACE);
        let _ = out_reader.join();
        let _ = err_reader.join();
        if timed_out {
            return Err("av worker build exceeded the BUILD_WALL budget".to_string());
        }
        if let Some(Err(n)) = &stdout_res {
            return Err(format!("av worker build stdout exceeded the cap ({n})"));
        }
        if let Some(Err(n)) = &stderr_res {
            return Err(format!("av worker build stderr exceeded the cap ({n})"));
        }
        return Err("av worker build ended without an exit status".to_string());
    }
    // Child exited inside the wall: close the job (reaps descendants
    // holding the inherited pipes), then drain both bounded readers.
    drop(job.take());
    let mut log = match out_rx.recv_timeout(READER_GRACE) {
        Ok(Ok(b)) => b,
        Ok(Err(n)) => return Err(format!("av worker build stdout exceeded the cap ({n})")),
        Err(_) => Vec::new(),
    };
    if let Ok(Ok(e)) = err_rx.recv_timeout(READER_GRACE) {
        log.extend_from_slice(&e);
    } else if let Ok(Err(n)) = err_rx.recv_timeout(READER_GRACE) {
        return Err(format!("av worker build stderr exceeded the cap ({n})"));
    }
    let _ = out_reader.join();
    let _ = err_reader.join();
    if !status.expect("set on exit").success() {
        return Err(format!(
            "av worker build failed:\n{}",
            String::from_utf8_lossy(&log)
        ));
    }
    if exe.is_file() {
        Ok(exe)
    } else {
        Err(format!(
            "av worker build produced no binary at {}",
            exe.display()
        ))
    }
}

#[cfg(windows)]
fn av_worker_exe_name() -> &'static str {
    "av_probe_worker.exe"
}
#[cfg(not(windows))]
fn av_worker_exe_name() -> &'static str {
    "av_probe_worker"
}

// ---- typed response mirror ----------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerResponse {
    schema: String,
    #[serde(default)]
    nonce: String,
    #[serde(default)]
    request_hash: String,
    #[serde(default)]
    ok: Option<OkOut>,
    #[serde(default)]
    error: Option<WorkerError>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerError {
    code: String,
    message: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OkOut {
    recipe: String,
    steps: Vec<StepOut>,
    worker_version: String,
    av_core_version: String,
}

/// PLAN-820 F820-R6-01: the typed-result report of a `typed:<id>` data
/// run — canonical kind tag + exact raw bits (lowercase hex; i64/u64/f64
/// 16 digits, f32 8, i32/bool unpadded). The parent compares BOTH against
/// its own frozen expectation; the worker's word is never the oracle.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TypedOut {
    kind: String,
    bits: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StepOut {
    op: String,
    outcome: String,
    #[serde(default)]
    value: Option<i32>,
    #[serde(default)]
    typed: Option<TypedOut>,
    #[serde(default)]
    trap: Option<String>,
    #[serde(default)]
    diag: Option<serde_json::Value>,
    #[serde(default)]
    counters: std::collections::BTreeMap<String, u64>,
    #[serde(default)]
    source_hash: String,
    #[serde(default)]
    wall_ms: f64,
    /// mark trace events ("a"/"b" sequence; observation only).
    #[serde(default)]
    trace: Vec<String>,
    /// Stage spans of the run step (name, own-duration ms) — bounded
    /// observability carried into the receipt.
    #[serde(default)]
    spans: Vec<(String, f64)>,
    /// PLAN-821: the owned face of an owned-recipe run step — the
    /// canonical result plus the post-release arena ledger
    /// ({result, live_after_release, allocs:[{id,bytes}], frees:[{id}]}).
    /// An OWNED-recipe increment only; its presence on any other recipe
    /// is an admission violation.
    #[serde(default)]
    owned: Option<serde_json::Value>,
    /// PLAN-822: the collections face of a collections-recipe run step —
    /// the canonical result plus the post-release arena ledger. A
    /// collections-recipe increment only.
    #[serde(default)]
    collections: Option<serde_json::Value>,
    /// PLAN-823 (R1-F02): the closures face of a closures-recipe run
    /// step — the canonical result plus the post-release arena ledger.
    /// A closures-recipe increment only.
    #[serde(default)]
    closures: Option<serde_json::Value>,
    /// PLAN-865: source closures' independently checked resource summary.
    #[serde(default)]
    closures_source: Option<serde_json::Value>,
    #[serde(default)]
    dynamic: Option<serde_json::Value>,
    /// PLAN-862: the AV result channel of a run step — `"value"` for an
    /// `i32` payload, `"unit"` for the Unit payload (which has no i32 form).
    /// Absent on older responses and on trap/diag steps.
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    console: Option<ConsoleOut>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ConsoleOut {
    schema: String,
    mir_lowering: String,
    stdout_b64: String,
    stderr_b64: String,
    stdout_present: bool,
    stderr_present: bool,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

/// Spawn the av worker and run ONE `case-serve` request under the bounded
/// executor contract (F798-R3-05/06 — mirrors the sema executor and
/// ac-core's `run_deadlined_captured` shape):
/// - contained start (suspended → job before the first instruction), so
///   the kill-on-close job reaps the whole tree on any exit path;
/// - the request WRITE runs on its own thread (a worker that never reads
///   stdin cannot wedge the parent);
/// - stdout/stderr stream on dedicated reader threads with hard caps —
///   stderr overflow kills the tree immediately, never a silent partial;
/// - ONE absolute deadline covers spawn → write → read → exit → drain;
///   blocking waits only happen with the deadline already decided (the
///   reader-grace bounded joins after a child exit, when the job close has
///   already reaped descendants).
/// After a normal exit the response must be a SUCCESS exit status and
/// EXACTLY one non-empty stdout line — trailing protocol frames and
/// non-zero exits are Infra, never green (F798-R3-05).
fn call_worker(request_json: &str, deadline: Duration) -> Result<WorkerResponse, String> {
    if request_json.len() > REQUEST_CAP {
        return Err(format!(
            "av worker request exceeds the {} byte metadata cap ({} bytes)",
            REQUEST_CAP,
            request_json.len()
        ));
    }
    let exe = worker_binary()?;
    let started = Instant::now();
    let mut cmd = Command::new(&exe);
    cmd.arg("case-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn av worker ({}): {}", exe.display(), e))?;
    #[cfg(feature = "test-case-batch")]
    parent_observations::spawned(child.id());
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained av worker tree".to_string(),
            );
        }
    };
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("resume av worker: {e}"));
    }
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;

    // Request writer thread: the write can block on a worker that never
    // reads — the parent never does.
    let (write_tx, write_rx) = std::sync::mpsc::channel::<()>();
    {
        let mut stdin = child.stdin.take().expect("stdin");
        let bytes = request_json.as_bytes().to_vec();
        std::thread::spawn(move || {
            let _ = stdin.write_all(&bytes);
            let _ = stdin.flush();
            drop(stdin);
            let _ = write_tx.send(());
        });
    }
    let mut stdout_pipe = child.stdout.take().expect("stdout");
    let mut stderr_pipe = child.stderr.take().expect("stderr");
    let (out_tx, out_rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let (err_tx, err_rx) = std::sync::mpsc::channel::<(bool, Vec<u8>)>();
    let _out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        let mut exceeded = false;
        loop {
            match stdout_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > READ_CAP {
                        exceeded = true;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = out_tx.send(if exceeded { Err(buf.len()) } else { Ok(buf) });
    });
    let _err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 16384];
        let mut overflow = false;
        loop {
            match stderr_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > STDERR_CAP {
                        overflow = true;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = err_tx.send((overflow, buf));
    });

    // Deadline loop: response AND exit, killing on wall or stderr overflow.
    // The stdout reader reads to EOF (not first-newline): the protocol is
    // exactly one line, and TRAILING frames must be visible to the judge.
    let deadline_at = started + deadline;
    let mut received: Option<Result<Vec<u8>, usize>> = None;
    let mut stderr_overflow = false;
    let mut stderr_result: Option<(bool, Vec<u8>)> = None;
    let mut timed_out = false;
    let mut exit_status: Option<std::process::ExitStatus> = None;
    loop {
        if Instant::now() > deadline_at {
            timed_out = true;
            break;
        }
        if stderr_overflow {
            break;
        }
        if received.is_none() {
            match out_rx.try_recv() {
                Ok(v) => received = Some(v),
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => received = Some(Err(0)),
            }
        }
        if stderr_result.is_none() {
            match err_rx.try_recv() {
                Ok(v) => {
                    stderr_overflow = v.0;
                    stderr_result = Some(v);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
            }
        }
        if exit_status.is_none() {
            match child.try_wait() {
                Ok(st @ Some(_)) => exit_status = st,
                Ok(None) => {}
                Err(e) => return Err(format!("poll av worker: {e}")),
            }
        }
        if exit_status.is_some() {
            #[cfg(feature = "test-case-batch")]
            let _cleanup = parent_observations::Phase::start("cleanup_ms");
            // Child gone: closing the job reaps descendants holding the
            // inherited pipes, then bounded reader joins.
            drop(job.take());
            if received.is_none() {
                received = match out_rx.recv_timeout(READER_GRACE) {
                    Ok(v) => Some(v),
                    Err(_) => Some(Err(0)),
                };
            }
            if stderr_result.is_none() {
                stderr_result = err_rx.recv_timeout(READER_GRACE).ok();
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    if timed_out || stderr_overflow {
        let _ = child.kill();
        drop(job.take());
        let _ = child.wait();
        let _ = write_rx.recv_timeout(READER_GRACE);
        let _ = out_rx.recv_timeout(READER_GRACE);
        let _ = err_rx.recv_timeout(READER_GRACE);
        return Err(if timed_out {
            "av worker exceeded the session wall (single absolute deadline over spawn/write/read/exit/drain)".to_string()
        } else {
            format!(
                "av worker stderr exceeded the {}-byte cap — tree killed",
                STDERR_CAP
            )
        });
    }
    // Normal exit: job already dropped (descendants reaped).
    #[cfg(feature = "test-case-batch")]
    let cleanup = parent_observations::Phase::start("cleanup_ms");
    drop(job.take());
    if write_rx.recv_timeout(READER_GRACE).is_err() {
        return Err("av worker request writer did not finish within the drain grace".into());
    }
    let out_buf = match received {
        Some(Ok(b)) => b,
        Some(Err(n)) => {
            return Err(format!(
                "av worker response exceeded the {}-byte read cap (saw {})",
                READ_CAP, n
            ));
        }
        None => match out_rx.recv_timeout(READER_GRACE) {
            Ok(Ok(b)) => b,
            Ok(Err(n)) => return Err(format!("av worker response exceeded the read cap ({n})")),
            Err(_) => {
                return Err("av worker stdout reader did not finish within the drain grace".into());
            }
        },
    };
    let err_buf = match stderr_result {
        Some((_, b)) => b,
        None => err_rx
            .recv_timeout(READER_GRACE)
            .map(|(_, b)| b)
            .unwrap_or_default(),
    };
    // F798-R3-05: process identity — a non-zero exit is Infra even when a
    #[cfg(feature = "test-case-batch")]
    drop(cleanup);
    // plausible JSON line preceded it.
    let status = exit_status.unwrap_or_else(|| child.wait().unwrap_or_default());
    if !status.success() {
        return Err(format!(
            "av worker exited with {:?} (non-zero worker exits are infra, never a language verdict); stderr: {:.300}",
            status.code(),
            String::from_utf8_lossy(&err_buf)
        ));
    }
    // F798-R3-05: protocol completeness — exactly ONE non-empty line; any
    // trailing frame (or zero lines) is Infra.
    let text = String::from_utf8_lossy(&out_buf);
    let non_empty: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.len() != 1 {
        return Err(format!(
            "av worker response must be exactly one non-empty protocol line, saw {} ({:.200})",
            non_empty.len(),
            text
        ));
    }
    serde_json::from_str(non_empty[0]).map_err(|e| {
        format!(
            "bad av worker response ({e}): {:.200} | stderr: {:.300}",
            non_empty[0],
            String::from_utf8_lossy(&err_buf)
        )
    })
}

// ---- parent-side judgement ----------------------------------------------

/// Map the av pipeline's diagnostic stage onto the contract's language
/// stages: front-door/frontend rejections are parse-stage; MIR/ABC/
/// closure admission rejections are verify-stage.
fn map_diag_stage(stage: &str) -> Stage {
    match stage {
        "admission" | "frontend" | "parse" | "text" => Stage::Parse,
        "resolve" | "bind" => Stage::Resolve,
        "typecheck" | "verify" => Stage::Typecheck,
        _ => Stage::Verify,
    }
}

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Core,
    )
}

pub fn run_loaded_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Loaded,
    )
}

/// PLAN-826 T-02: the compact-wire recipe entry — same executor/judgement
/// contract, scalar profile over the AVAC wire generation.
pub fn run_compact_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Compact,
    )
}

pub fn run_console_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Console,
    )
}

/// PLAN-820: the data-recipe entry — same executor/judgement contract,
/// data-profile pipeline (record source front door in the worker).
/// PLAN-820 F820-R6-01: the PARENT's frozen av.data-fixture/1 expectation
/// table — an INDEPENDENT copy of the worker-side fixture truths (tag +
/// exact raw bits). The typed judge compares against THIS, never against
/// anything the worker reports as expected.
const DATA_TYPED_FIXTURES: &[(&str, &str, &str)] = &[
    ("u64-max", "u64", "ffffffffffffffff"),
    ("i64-min", "i64", "8000000000000000"),
    ("fneg-nan", "f64", "fff80000deadbeef"),
    ("neg-zero", "f64", "8000000000000000"),
    ("f32-one", "f32", "3f800000"),
    ("f32-nan-payload", "f32", "7f80ad00"),
    ("bool-true", "bool", "1"),
    ("i32-42", "i32", "2a"),
];

pub fn run_data_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Data,
    )
}

/// PLAN-821 T-02: the PARENT's frozen av.owned-fixture/1 expectation
/// table — an INDEPENDENT hand-audited copy of the frozen fixture truths
/// (contract-decision §11): the canonical result face, the owned
/// allocation byte sizes in CREATION order, and the free ids in RELEASE
/// order (live must reach 0 after the host-side release). The owned judge
/// compares against THIS, never against anything the worker reports as
/// expected.
const OWNED_CASES: &[(&str, &str, &[usize], &[u64])] = &[
    // first-case: Note { body: clone of "Auto", n = len("Auto") = 4 }.
    // Hand-audited: two 4-byte buffers (ctor literal + clone), the ctor
    // buffer frees at the field replace, the clone frees at the host
    // release (creation-order LIFO); live=0 after the release.
    (
        "first-case",
        r#"{"fields":[{"kind":"str","utf8":"Auto"},{"kind":"i32","bits":"4"}],"kind":"record","name":"t_rec_Note"}"#,
        &[4, 4],
        &[0, 1],
    ),
];

/// PLAN-822 T-02: the PARENT's frozen av-collections expectation table —
/// an INDEPENDENT hand-audited copy of the frozen fixture truths: the
/// canonical result face, allocation byte sizes in CREATION order, and
/// free ids in RELEASE order (live must reach 0). The collections judge
/// compares against THIS, never against anything the worker reports.
const COLLECTIONS_CASES: &[(&str, &str, &[usize], &[u64])] = &[
    // first-case: Note { text: "first" replaced in place, n: 1 };
    // push/field-replace/remove/Ok-pack/guard/project/drop. Hand-audited:
    // "first"(5B) materializes then frees at the field replace; the list
    // buffer (0-elem ledger entry) frees at frame exit; "replaced"(8B)
    // frees at the explicit projected-value drop. live=0 after release.
    (
        "first-case",
        r#"{"bits":"0","kind":"i32"}"#,
        &[5, 0, 8],
        &[0, 2, 1],
    ),
];

/// PLAN-823 (R1-F02): the frozen closures fixture table — the PARENT'S
/// hand-audited expectations (never the worker's word). first-case:
/// result 29 = 0x1d; allocs [4 "auto", 4 "auto"(note field), 0 list
/// (0-elem ledger entry at mint), 4 capture clone, 6 "pushed"]; the
/// deterministic free order [0, 4, 1, 2, 3]; live=0 after release.
const CLOSURES_CASES: &[(&str, &str, &[usize], &[u64])] = &[(
    "first-case",
    r#"{"bits":"1d","kind":"i32"}"#,
    &[4, 4, 0, 4, 6],
    &[0, 4, 1, 2, 3],
)];

/// PLAN-865 source-case expectations held independently by the parent.
/// The TestCase oracle repeats these values so either side can catch a lie in
/// the other; diagnostics bind the explicitly unsupported source cases.
const CLOSURES_SOURCE_CASES: &[(&str, Option<i32>, Option<&str>, usize)] = &[
    ("K01", Some(42), None, 2),
    ("K02", Some(42), None, 3),
    ("K03", Some(42), None, 2),
    ("K04", None, Some("source.closure-borrow-escape"), 0),
    ("K05", Some(42), None, 3),
    ("K06", None, Some("source.closure-shared-mutable"), 0),
];

pub fn run_closures_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Closures,
    )
}

/// PLAN-865: source-backed closure check. The parent validates the
/// `source:<K-id>` binding, independently judges the case oracle, and checks
/// the closures-session resource face.
pub fn run_closures_source_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::ClosuresSource,
    )
}
/// PLAN-824: the frozen dynamic-fixture faces the PARENT judges against.
/// Same tuple shape as the collections table; the judge ADDITIONALLY
/// asserts the Dyn face's portable axes when the fixture returns a Dyn
/// (`dyn-out`), hand-audited below.
const DYNAMIC_CASES: &[(&str, &str, &[usize], &[u64])] = &[
    // first-case: pack/is/take over Note/List<Note>/Opt — all hits score
    // 7+1+1=9. Hand-audited: the "dyn" Str (3 bytes) materializes and
    // frees at frame exit; the list buffer (0-elem at LIST_NEW) frees
    // FIRST (LIFO) reporting its final 1 element; live=0 after release.
    (
        "first-case",
        r#"{"bits":"9","kind":"i32"}"#,
        &[3, 0],
        &[1, 0],
    ),
    // dyn-out: returns the packed Note{ "portable"(8B), 11 } as a Dyn; the
    // Str allocation frees at frame exit; live=0 after host release; the
    // portable round-trip axis is asserted true by the judge.
    (
        "dyn-out",
        r#"{"kind":"dyn","ty":"record dynamic824/Note[2]","value":{"fields":[{"kind":"str","utf8":"portable"},{"kind":"i32","bits":"b"}],"kind":"record","name":"Note"}}"#,
        &[8],
        &[0],
    ),
    // take-probe (T-11 D02 formal face): pack Note{"prb",7}, probe WRONG
    // (is List<Note> — false, never a trap), take RIGHT, read n → 7. The
    // success proves the failed probe left the container takeable; the
    // "prb" Str (3 bytes) frees at frame exit; live=0.
    ("take-probe", r#"{"bits":"7","kind":"i32"}"#, &[3], &[0]),
];

/// PLAN-866: the resident-online recipe check — one continuously-running
/// guest chain (fuel slices, never restarted), queued edits applied at
/// call-boundary safety points; the judge binds the finish step's outcome
/// to the case oracle AND asserts the structural resident faces from the
/// reported counters (chain entries == 1; installed swaps; boundary
/// steps; zero production work for unchanged functions).
pub fn run_resident_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Resident,
    )
}

/// PLAN-864: the collections SOURCE recipe check — same request/judge
/// shape as the other av families; the entry is the source function name
/// and the worker compiles the real source through the collections
/// frontend (no fixture table).
pub fn run_collections_source_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::CollectionsSource,
    )
}

pub fn run_collections_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Collections,
    )
}

pub fn run_dynamic_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Dynamic,
    )
}

pub fn run_owned_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Owned,
    )
}

fn run_check_impl(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
    family: RecipeFamily,
) -> Result<Verdict, String> {
    run_check_with_transport(
        case,
        check,
        oracle,
        _project_dir,
        artifact_dir,
        _run_dir,
        family,
        call_worker,
        None,
    )
}

// Transport changes do not create a second admission/expectation comparator.
fn run_check_with_transport(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
    family: RecipeFamily,
    mut transport: impl FnMut(&str, Duration) -> Result<WorkerResponse, String>,
    leased_worker_sha: Option<&str>,
) -> Result<Verdict, String> {
    #[cfg(feature = "test-case-batch")]
    let mut parent_clock = parent_observations::Clock::new();
    let started = Instant::now();
    let recipe = family.recipe();
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "{recipe} consumes .at sources; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    if family == RecipeFamily::Console {
        auto_test_case::contract::validate_console_oracle(
            oracle.stdout.as_ref(),
            oracle.stderr.as_ref(),
            oracle.exit,
            oracle.av_channel.as_deref(),
            &oracle.completion,
        )?;
        let binding = check
            .options
            .probe_av
            .as_ref()
            .ok_or("console binding missing")?;
        if check.options.probe_recipe.as_deref() != Some(AV_CONSOLE_RECIPE)
            || binding.entry != "d_main"
            || binding.session.is_some()
            || binding.profile.is_some()
            || !matches!(
                check.options.probe_steps.as_deref(),
                Some([auto_test_case::contract::ProbeStep::Run])
            )
        {
            return Err(
                "console-run-v1 requires d_main, one run and no session/profile override".into(),
            );
        }
    } else if oracle.stdout.is_some() || oracle.stderr.is_some() {
        return Err("this probe recipe does not expose console streams".into());
    }
    let av = check
        .options
        .probe_av
        .as_ref()
        .ok_or("probe check without an av binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if check.options.probe_recipe.as_deref() != Some(recipe) {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    // PLAN-862: the SOURCE-profile selector (append-only). Absent = the
    // historical scalar profile, byte-identical for every existing case.
    // `entry-unit` changes the front door, the wire family and the reported
    // core identity — the admission below binds to THAT identity, so a
    // worker reporting the scalar profile for an entry-unit request is a
    // protocol violation, never a green.
    let entry_unit_profile = match av.profile.as_deref() {
        None => false,
        Some("entry-unit") => true,
        Some(other) => {
            return Err(format!(
                "unknown av profile `{other}` (the contract admits only entry-unit)"
            ))
        }
    };
    let expected_core_version = if entry_unit_profile {
        EXPECTED_AV_ENTRY_UNIT_CORE_VERSION
    } else {
        family.expected_core_version()
    };
    // F798-R3-07 shape: leading edit-source allowed (measured window);
    // the judged last step must be a run. PLAN-866: the resident recipe
    // judges the resident-finish step instead.
    let wants_run_tail = family != RecipeFamily::Resident;
    let tail_ok = match steps.last() {
        Some(auto_test_case::contract::ProbeStep::Run) => wants_run_tail,
        Some(auto_test_case::contract::ProbeStep::ResidentFinish) => !wants_run_tail,
        _ => false,
    };
    if !tail_ok {
        return Err(format!(
            "{recipe} step sequences must end with a {} step (contract violation)",
            if wants_run_tail {
                "run"
            } else {
                "resident-finish"
            }
        ));
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    // Store root (F798-R3-07): with an explicit SESSION id the store lives
    // under a deterministic shared temp root keyed by that id, so calls of
    // the same session (warm-up + measured edit) share one warm store
    // across worker processes; the PARENT owns cleanup. Without a session
    // the check's own artifact dir hosts a private per-request store.
    let store_root = match &av.session {
        // Explicit module-case execution owns a private run. Ordinary suite
        // callers may intentionally restart the same session across runs.
        Some(_)
            if family == RecipeFamily::ClosuresSource
                && MODULE_PRIVATE_SESSIONS.with(|scope| scope.get()) =>
        {
            _run_dir.join("av-closures-sessions")
        }
        Some(s) => std::env::temp_dir().join(format!("av798-session-{s}")),
        None => artifact_dir.join("store"),
    };
    std::fs::create_dir_all(&store_root).map_err(|e| format!("probe store root: {}", e))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("probe artifact dir: {}", e))?;

    let nonce = hex(&sha256(
        format!("{}|{:?}|av", std::process::id(), started).as_bytes(),
    ));
    let steps_json: Vec<serde_json::Value> = steps
        .iter()
        .map(|st| match st {
            auto_test_case::contract::ProbeStep::Run => {
                serde_json::json!({ "op": "run" })
            }
            auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                serde_json::json!({ "op": "edit-source", "from": from, "to": to })
            }
            auto_test_case::contract::ProbeStep::ResidentSlice { slices } => {
                serde_json::json!({ "op": "resident-slice", "slices": slices })
            }
            auto_test_case::contract::ProbeStep::ResidentApply => {
                serde_json::json!({ "op": "resident-apply" })
            }
            auto_test_case::contract::ProbeStep::ResidentFinish => {
                serde_json::json!({ "op": "resident-finish" })
            }
            _ => serde_json::json!({ "op": "run" }),
        })
        .collect();
    // F820-R6-01: a `typed:<id>` entry (data recipe only) selects a frozen
    // av.data-fixture/1 input; the PARENT validates the id against its OWN
    // frozen table before any request — an unregistered fixture is a check
    // error, never a green run.
    let typed_fixture = if family == RecipeFamily::Data {
        match av.entry.strip_prefix("typed:") {
            None => None,
            Some(id) => match DATA_TYPED_FIXTURES.iter().find(|(fid, _, _)| *fid == id) {
                Some(_) => Some(id.to_string()),
                None => {
                    return Err(format!(
                        "av.data typed fixture `{id}` is not registered (parent table)"
                    ));
                }
            },
        }
    } else {
        None
    };
    // PLAN-821: an `owned:<id>` entry (owned recipe only) selects a frozen
    // av.owned-fixture/1 input; the PARENT validates the id against its OWN
    // frozen table before any request — an unregistered fixture is a check
    // error, never a green run.
    let owned_fixture = if family == RecipeFamily::Owned {
        match av.entry.strip_prefix("owned:") {
            None => None,
            Some(id) => match OWNED_CASES.iter().find(|(fid, ..)| *fid == id) {
                Some(_) => Some(id.to_string()),
                None => {
                    return Err(format!(
                        "av.owned fixture `{id}` is not registered (parent table)"
                    ));
                }
            },
        }
    } else {
        None
    };
    // PLAN-822: a `collections:<id>` entry (collections recipe only)
    // selects a frozen collections fixture; the PARENT validates the id
    // against its OWN frozen table before any request.
    // PLAN-824: a `dynamic:<id>` entry (dynamic recipe only) selects a
    // frozen dynamic fixture; the PARENT validates the id against its OWN
    // frozen table before any request.
    let dynamic_fixture = if family == RecipeFamily::Dynamic {
        match av.entry.strip_prefix("dynamic:") {
            None => None,
            Some(id) => match DYNAMIC_CASES.iter().find(|(fid, ..)| *fid == id) {
                Some(_) => Some(id.to_string()),
                None => {
                    return Err(format!(
                        "av dynamic fixture `{id}` is not registered (parent table)"
                    ));
                }
            },
        }
    } else {
        None
    };
    if family == RecipeFamily::CollectionsSource {
        if av.entry.starts_with("collections:") {
            return Err(
                "collections-source takes a source function name, not a collections:<id> fixture"
                    .into(),
            );
        }
        if av.entry.is_empty() {
            return Err("collections-source requires a non-empty av.entry".into());
        }
    }
    let collections_fixture = if family == RecipeFamily::Collections {
        match av.entry.strip_prefix("collections:") {
            None => None,
            Some(id) => match COLLECTIONS_CASES.iter().find(|(fid, ..)| *fid == id) {
                Some(_) => Some(id.to_string()),
                None => {
                    return Err(format!(
                        "av collections fixture `{id}` is not registered (parent table)"
                    ));
                }
            },
        }
    } else {
        None
    };
    // PLAN-823 (R1-F02): a `closures:<id>` entry (closures recipe only)
    // selects a frozen closures fixture; the PARENT validates the id
    // against its OWN frozen table before any request.
    let closures_fixture = if family == RecipeFamily::Closures {
        match av.entry.strip_prefix("closures:") {
            None => None,
            Some(id) => match CLOSURES_CASES.iter().find(|(fid, ..)| *fid == id) {
                Some(_) => Some(id.to_string()),
                None => {
                    return Err(format!(
                        "av closures fixture `{id}` is not registered (parent table)"
                    ));
                }
            },
        }
    } else {
        None
    };
    let closures_source_id = if family == RecipeFamily::ClosuresSource {
        let selector = av.entry.strip_prefix("source:").ok_or_else(|| {
            "closures-source requires a registered source:<K-id> av.entry".to_string()
        })?;
        let mut parts = selector.split(':');
        let id = parts.next().unwrap_or_default();
        let mode = parts.next();
        if parts.next().is_some()
            || mode.is_some_and(|mode| !matches!(mode, "fresh" | "restart" | "body-edit"))
            || (mode == Some("body-edit") && id != "K01")
        {
            return Err(format!("invalid closures-source selector `{selector}`"));
        }
        if !CLOSURES_SOURCE_CASES
            .iter()
            .any(|(case_id, ..)| *case_id == id)
        {
            return Err(format!(
                "closures source `{id}` is not registered (parent table)"
            ));
        }
        Some((id.to_string(), mode.map(str::to_string)))
    } else {
        None
    };
    let mut request = serde_json::json!({
        "recipe": recipe,
        "nonce": nonce,
        "source_b64": b64(&input_bytes),
        "entry": av.entry,
        "caps": av.caps,
        "store_root": store_root.display().to_string().replace('/', "\\"),
        "steps": steps_json,
    });
    if let Some(id) = &typed_fixture {
        request["typed_fixture"] = serde_json::json!(id);
    }
    if let Some(id) = &owned_fixture {
        request["owned_fixture"] = serde_json::json!(id);
    }
    if let Some(id) = &dynamic_fixture {
        request["dynamic_fixture"] = serde_json::json!(id);
    }
    if let Some(id) = &collections_fixture {
        request["collections_fixture"] = serde_json::json!(id);
    }
    if let Some(id) = &closures_fixture {
        request["closures_fixture"] = serde_json::json!(id);
    }
    if let Some(session) = &av.session {
        request["session"] = serde_json::json!(session);
    }
    if entry_unit_profile {
        request["profile"] = serde_json::json!("entry-unit");
    }
    let request_hash = hex(&sha256(request.to_string().as_bytes()));
    request["request_hash"] = serde_json::json!(request_hash);

    let worker_started = Instant::now();
    // Session wall (test seam: AV798_SESSION_WALL_MS shortens the absolute
    // deadline for the delay counter-control; default = the frozen 60s).
    let wall = std::env::var("AV798_SESSION_WALL_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(SESSION_WALL);
    let request_text = request.to_string();
    #[cfg(feature = "test-case-batch")]
    parent_clock.lap("prepare_ms");
    let resp_result = transport(&request_text, wall);
    #[cfg(feature = "test-case-batch")]
    parent_clock.lap("transport_ms");
    let worker_us = worker_started.elapsed().as_micros() as u64;

    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut infra_detail: Option<String> = None;
    let mut step_out: Option<StepOut> = None;
    // A test batch may reuse the digest ONLY while its Windows file lease
    // denies writes/deletion/replacement. Independent admission still re-reads.
    let exe_sha = match leased_worker_sha {
        Some(sha) => sha.to_owned(),
        None => worker_exe_sha256()?,
    };

    'admit: {
        let resp = match resp_result {
            Ok(r) => r,
            Err(e) => {
                infra_detail = Some(e);
                break 'admit;
            }
        };
        if resp.schema != AV_WORKER_PROTOCOL {
            infra_detail = Some(format!("av worker schema mismatch: {}", resp.schema));
            break 'admit;
        }
        if resp.nonce != nonce || resp.request_hash != request_hash {
            infra_detail = Some("av worker nonce/request_hash echo mismatch".to_string());
            break 'admit;
        }
        let ok = match resp.ok.clone() {
            Some(ok) => ok,
            None => {
                let e = resp.error.unwrap_or(WorkerError {
                    code: "missing".into(),
                    message: "neither ok nor error".into(),
                });
                infra_detail = Some(format!("av worker error {}: {}", e.code, e.message));
                break 'admit;
            }
        };
        if ok.recipe != recipe {
            infra_detail = Some(format!("av worker recipe mismatch: {}", ok.recipe));
            break 'admit;
        }
        if family == RecipeFamily::Console {
            if let Some(step) = ok.steps.first() {
                // Preserve raw observations for resource/infra diagnostics too; no PASS is trusted.
                std::fs::write(artifact_dir.join("console-observation.json"), serde_json::to_vec_pretty(&serde_json::json!({"outcome":step.outcome,"diag":step.diag,"console":step.console})).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            }
        }
        if let Some(error) = console_admission(family, &ok.steps) {
            infra_detail = Some(error);
            break 'admit;
        }
        // F820-R7-01: the typed report is a DATA-recipe protocol
        // increment — its presence on any non-data (core/owned) response
        // is an admission violation, never a pass-shaped extra channel.
        if typed_on_non_data(family, &ok.steps) {
            infra_detail =
                Some("typed report on a non-data recipe (protocol violation)".to_string());
            break 'admit;
        }
        // F820-R8-01: data typed shape is an ORACLE-INDEPENDENT admission
        // rule — no exit/value oracle match can bypass it.
        if let Some(v) = data_malformed_typed(&ok.steps) {
            infra_detail = Some(v);
            break 'admit;
        }
        // PLAN-821: the owned face is an OWNED-recipe increment — its
        // presence on any non-owned response is an admission violation,
        // and every owned report is SHAPE-validated here (oracle-independent:
        // {result: object, live_after_release: u64, allocs: [{id,bytes}],
        // frees: [{id}]}), so no oracle choice can let a malformed ledger
        // face through.
        if owned_on_non_owned(family, &ok.steps) {
            infra_detail =
                Some("owned report on a non-owned recipe (protocol violation)".to_string());
            break 'admit;
        }
        if let Some(v) = malformed_owned(&ok.steps) {
            infra_detail = Some(v);
            break 'admit;
        }
        if closures_source_on_non_recipe(family, &ok.steps) {
            infra_detail = Some(
                "closures_source report on a non-closures-source recipe (protocol violation)"
                    .to_string(),
            );
            break 'admit;
        }
        if let Some(v) = malformed_closures_source(&ok.steps) {
            infra_detail = Some(v);
            break 'admit;
        }
        if ok.worker_version != EXPECTED_AV_WORKER_VERSION {
            infra_detail = Some(format!(
                "av worker version `{}` does not match the admitted `{}`",
                ok.worker_version, EXPECTED_AV_WORKER_VERSION
            ));
            break 'admit;
        }
        if ok.av_core_version != expected_core_version {
            infra_detail = Some(format!(
                "av core version `{}` does not match the admitted `{}`",
                ok.av_core_version, expected_core_version
            ));
            break 'admit;
        }
        let last_run = if family == RecipeFamily::Resident {
            ok.steps
                .iter()
                .rev()
                .find(|s| s.op == "resident-finish")
                .cloned()
        } else {
            ok.steps.iter().rev().find(|s| s.op == "run").cloned()
        };
        match last_run {
            Some(s) => step_out = Some(s),
            _ => {
                infra_detail = Some("av worker returned no run step".to_string());
                break 'admit;
            }
        }
        // F798-R3-05: the response is bound to THIS request and revision —
        // step count, op echo, edit outcomes, and every run step's
        // source_hash re-derived parent-side over the request's own source
        // state (a relay echoing a valid envelope over somebody else's or a
        // stale revision's result is Infra, never a green judgement).
        if ok.steps.len() != steps.len() {
            infra_detail = Some(format!(
                "av worker returned {} steps for a {}-step request",
                ok.steps.len(),
                steps.len()
            ));
            break 'admit;
        }
        let mut current_source = String::from_utf8_lossy(&input_bytes).to_string();
        for (i, (want, got)) in steps.iter().zip(ok.steps.iter()).enumerate() {
            let want_op = match want {
                auto_test_case::contract::ProbeStep::Run => "run",
                auto_test_case::contract::ProbeStep::ResidentSlice { .. } => {
                    if got.outcome != "sliced" {
                        infra_detail = Some(format!(
                            "step {i}: resident-slice outcome `{}` is not `sliced`",
                            got.outcome
                        ));
                        break 'admit;
                    }
                    "resident-slice"
                }
                auto_test_case::contract::ProbeStep::ResidentApply => {
                    if got.outcome != "applied" {
                        infra_detail = Some(format!(
                            "step {i}: resident-apply outcome `{}` is not `applied`",
                            got.outcome
                        ));
                        break 'admit;
                    }
                    "resident-apply"
                }
                auto_test_case::contract::ProbeStep::ResidentFinish => "resident-finish",
                auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                    let hits = current_source.matches(from.as_str()).count();
                    if hits != 1 {
                        infra_detail = Some(format!(
                            "step {i}: parent-side edit literal matched {hits} times"
                        ));
                        break 'admit;
                    }
                    current_source = current_source.replacen(from.as_str(), to.as_str(), 1);
                    if got.outcome != "edited" {
                        infra_detail = Some(format!(
                            "step {i}: edit-source outcome `{}` is not `edited`",
                            got.outcome
                        ));
                        break 'admit;
                    }
                    "edit-source"
                }
                _ => {
                    infra_detail = Some(format!("step {i}: unsupported probe step in av recipe"));
                    break 'admit;
                }
            };
            if got.op != want_op {
                infra_detail = Some(format!(
                    "step {i}: op `{}` does not echo the requested `{}`",
                    got.op, want_op
                ));
                break 'admit;
            }
            if want_op == "run" || want_op == "resident-finish" {
                // A typed/owned fixture run's revision identity is the
                // FIXTURE ENTRY (the frozen input that actually ran); a
                // source run binds the current source revision
                // (F820-R6-01; PLAN-821 same rule for owned).
                let expect_hash = if typed_fixture.is_some()
                    || owned_fixture.is_some()
                    || collections_fixture.is_some()
                    || closures_fixture.is_some()
                    || dynamic_fixture.is_some()
                {
                    fnv1a64_hex(av.entry.as_bytes())
                } else {
                    fnv1a64_hex(current_source.as_bytes())
                };
                if got.source_hash != expect_hash {
                    infra_detail = Some(format!(
                        "step {i}: source_hash `{}` does not bind the request's current revision ({})",
                        got.source_hash, expect_hash
                    ));
                    break 'admit;
                }
            }
        }
    }

    if let Some(e) = infra_detail {
        // Infrastructure lifecycle failure — an ERROR outcome, never a
        // language verdict.
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![Mismatch {
                observation: "worker".into(),
                kind: "infra".into(),
                expected: "a contained worker session".into(),
                actual: e,
                first_diff: None,
            }],
            detail: Some(format!("{recipe}: infra failure")),
            artifacts: vec![],
            native: None,
        });
    }

    let step = step_out.expect("admitted");
    if let Some(id) = &typed_fixture {
        // The typed judge face: tag + exact raw bits against the parent's
        // frozen expectation — independent of `value` (which cannot carry
        // wide/bool results) and of anything the worker claims.
        let (_, want_kind, want_bits) = DATA_TYPED_FIXTURES
            .iter()
            .find(|(fid, _, _)| fid == id)
            .expect("validated above");
        // Counter-control seam (V-10 wrong-bits / wrong-type-tag): the
        // test-only sabotage mutates the PARENT's expectation copy so a
        // real run of a legal fixture must FAIL. Never set in production.
        let (want_kind, want_bits) =
            match std::env::var("AV820_TYPED_EXPECT_SABOTAGE").ok().as_deref() {
                Some("kind") => (
                    if *want_kind == "i64" { "u64" } else { "i64" },
                    want_bits.to_string(),
                ),
                Some("bits") => {
                    let mut b = want_bits.to_string();
                    let last = b.pop().expect("non-empty bits");
                    b.push(if last == '0' { '1' } else { '0' });
                    (*want_kind, b)
                }
                _ => (*want_kind, want_bits.to_string()),
            };
        if step.outcome != "success" {
            mismatches.push(Mismatch {
                observation: "completion".into(),
                kind: "typed-outcome".into(),
                expected: "success".into(),
                actual: step_summary(&step),
                first_diff: None,
            });
        }
        match &step.typed {
            None => mismatches.push(Mismatch {
                observation: "typed".into(),
                kind: "typed-report".into(),
                expected: format!("{want_kind}:{want_bits}"),
                actual: "no typed result on the step".into(),
                first_diff: None,
            }),
            Some(t) => {
                if t.kind != want_kind {
                    mismatches.push(Mismatch {
                        observation: "typed".into(),
                        kind: "typed-tag".into(),
                        expected: want_kind.to_string(),
                        actual: t.kind.clone(),
                        first_diff: None,
                    });
                }
                if t.bits != want_bits {
                    mismatches.push(Mismatch {
                        observation: "typed".into(),
                        kind: "typed-bits".into(),
                        expected: want_bits.clone(),
                        actual: t.bits.clone(),
                        first_diff: None,
                    });
                }
            }
        }
    } else if let Some(id) = &owned_fixture {
        // The owned-fixture judge face: canonical result + live=0 +
        // alloc/free order against the parent's frozen table — fully
        // independent of `value` and of anything the worker claims.
        judge_owned_fixture(id, &step, &mut mismatches);
    } else if let Some(id) = &collections_fixture {
        // PLAN-822: the collections-fixture judge face — same axes as the
        // owned judge (result + live=0 + alloc/free order), its own table.
        judge_collections_fixture(id, &step, &mut mismatches);
    } else if let Some(id) = &closures_fixture {
        judge_closures_fixture(id, &step, &mut mismatches);
    } else if let Some(id) = &closures_source_id {
        judge_closures_source(&id.0, id.1.as_deref(), &step, &mut mismatches);
        judge_expectation(family, &step, oracle, &mut mismatches);
    } else if let Some(id) = &dynamic_fixture {
        judge_dynamic_fixture(id, &step, &mut mismatches);
    } else {
        judge_expectation(family, &step, oracle, &mut mismatches);
        if family == RecipeFamily::Loaded {
            judge_loaded_runtime(&step, &mut mismatches);
        }
        if family == RecipeFamily::Compact {
            judge_compact_runtime(&step, &mut mismatches);
        }
        if family == RecipeFamily::Resident {
            judge_resident_faces(&step, &mut mismatches);
        }
    }

    if family == RecipeFamily::Console {
        let capture = step.console.as_ref().expect("console admitted");
        let run =
            serde_json::json!({"stdout_b64":capture.stdout_b64,"stderr_b64":capture.stderr_b64});
        super::ac::compare_stream_field(&run, "stdout", oracle.stdout.as_ref(), &mut mismatches)?;
        super::ac::compare_stream_field(&run, "stderr", oracle.stderr.as_ref(), &mut mismatches)?;
    }
    // Receipt (probe-receipt.json in the artifact dir). `request_wall_ms`
    // is the continuous parent-side window from request build to judged
    // response — spawn, request write, worker session, response read and
    // admission all inside (F798-R3-07: the formal turnaround gate
    // measures THIS, never a subtraction of parent call time).
    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let receipt = serde_json::json!({
        "recipe": recipe,
        "protocol": AV_WORKER_PROTOCOL,
        "nonce": nonce,
        "request_hash": request_hash,
        "step": {
            "op": step.op,
            "outcome": step.outcome,
            "value": step.value,
            "channel": step.channel,
            "console": step.console,
            "trap": step.trap,
            "diag": step.diag,
            "owned": step.owned,
            "closures_source": step.closures_source,
            "counters": step.counters,
            "source_hash": step.source_hash,
            "wall_ms": step.wall_ms,
            "spans": step.spans,
            "trace": step.trace,
        },
        "worker_us": worker_us,
        "request_wall_ms": request_wall_ms,
        "toolchain": probe_toolchain_axes(&exe_sha, family, entry_unit_profile.then_some(EXPECTED_AV_ENTRY_UNIT_CORE_VERSION)),
    });
    std::fs::write(
        artifact_dir.join("probe-receipt.json"),
        serde_json::to_string_pretty(&receipt).unwrap_or_default(),
    )
    .map_err(|e| format!("write receipt: {e}"))?;

    let native = NativeRecord {
        exit_code: match step.outcome.as_str() {
            "success" => step.value,
            "trap" => Some(70),
            _ => None,
        },
        stage_spans: vec![
            ("worker_us".to_string(), worker_us as u64),
            ("wall_ms".to_string(), step.wall_ms as u64),
            ("request_wall_ms".to_string(), request_wall_ms as u64),
        ],
        cache_mode: None,
        cache_hits: vec![],
        counters: step
            .counters
            .iter()
            .map(|(k, v)| (k.clone(), *v as u32))
            .collect(),
        toolchain: probe_toolchain_axes(
            &exe_sha,
            family,
            entry_unit_profile.then_some(EXPECTED_AV_ENTRY_UNIT_CORE_VERSION),
        ),
        gaps: vec![],
    };
    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    let verdict = Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "{recipe}: run step judged (outcome {}; channel {}); receipt probe-receipt.json",
            step.outcome,
            step.channel.as_deref().unwrap_or("-")
        )),
        artifacts: vec![],
        native: Some(native),
    };
    #[cfg(feature = "test-case-batch")]
    parent_clock.lap("admission_compare_receipt_ms");
    Ok(verdict)
}

/// Judge the observed step outcome against the declared expectations.
/// The oracle faces: success+exit (value), error-run+message (trap),
/// error-parse/verify+code (diagnosed rejection).
/// Canonical typed-shape validation (F820-R7-01): a data-family typed
/// report must name one of the six scalar tags and carry the kind's
/// canonical lowercase-hex width — malformed reports are mismatches, not
/// silent passes. Returns the violation message, if any.
fn typed_shape_violation(t: &TypedOut) -> Option<String> {
    let width_ok = match t.kind.as_str() {
        "i64" | "u64" | "f64" => t.bits.len() == 16,
        "f32" => t.bits.len() == 8,
        // Canonical UNPADDED hex (contract-decision §3: bool/i32 不补零) —
        // a leading zero is a non-canonical encoding, and bool is exactly
        // the two value digits 0/1 (F820-R8-01: `2`/`00` must not pass).
        // The single digit `0` is the UNIQUE canonical encoding of ZERO
        // (F820-R9-01): forbidding padding must not forbid the zero value.
        "i32" => t.bits == "0" || ((1..=8).contains(&t.bits.len()) && !t.bits.starts_with('0')),
        "bool" => t.bits == "0" || t.bits == "1",
        other => return Some(format!("unknown typed kind `{other}`")),
    };
    if !width_ok
        || !t
            .bits
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Some(format!(
            "typed bits `{}` are not canonical for kind `{}`",
            t.bits, t.kind
        ));
    }
    None
}

/// Admission rule (F820-R7-01): any typed report on a non-data recipe.
fn typed_on_non_data(family: RecipeFamily, steps: &[StepOut]) -> bool {
    family != RecipeFamily::Data && steps.iter().any(|s| s.typed.is_some())
}

/// Admission rule (PLAN-821): any owned report on a non-owned recipe.
fn owned_on_non_owned(family: RecipeFamily, steps: &[StepOut]) -> bool {
    family != RecipeFamily::Owned && steps.iter().any(|s| s.owned.is_some())
}

/// PLAN-865: the source-session face belongs only to the source closures
/// recipe; fixture closures keep their original ledger contract.
fn closures_source_on_non_recipe(family: RecipeFamily, steps: &[StepOut]) -> bool {
    family != RecipeFamily::ClosuresSource && steps.iter().any(|s| s.closures_source.is_some())
}

fn malformed_closures_source(steps: &[StepOut]) -> Option<String> {
    steps.iter().find_map(|step| {
        let face = step.closures_source.as_ref()?;
        let required = [
            "live_after_release",
            "allocs",
            "frees",
            "submitted_units",
            "verified",
            "encoded",
            "admitted",
            "store_writes",
            "cache_hits",
            "cache_misses",
            "current_hot_bytes",
            "history_hot_bytes",
            "process_peak_bytes",
        ];
        let malformed = required
            .iter()
            .any(|field| face.get(field).and_then(|value| value.as_u64()).is_none());
        malformed.then(|| {
            "malformed closures_source face (expected u64 runtime/cache/resource axes)".into()
        })
    })
}

/// Admission rule (PLAN-821): EVERY owned report on an owned response is
/// shape-validated HERE — before and independent of any oracle branch.
/// The face must carry {result: <object>, live_after_release: <u64>,
/// allocs: [{id: u64, bytes: u64}], frees: [{id: u64}]}.
fn malformed_owned(steps: &[StepOut]) -> Option<String> {
    steps.iter().find_map(|s| {
        s.owned.as_ref().and_then(|o| {
            let bad = |why: &str| {
                Some(format!(
                    "malformed owned report on step `{}`: {}",
                    s.op, why
                ))
            };
            // Every structural defect is an EXPLICIT rejection — a
            // missing key must never `?`-skip the validator (the merge
            // gate caught an alloc entry without `bytes` slipping
            // through as "not malformed").
            let Some(o) = o.as_object() else {
                return bad("owned face is not an object");
            };
            match o.get("result") {
                Some(v) if v.is_object() => {}
                Some(_) => return bad("result is not an object"),
                None => return bad("missing result"),
            }
            match o.get("live_after_release") {
                Some(v) if v.is_u64() => {}
                Some(_) => return bad("live_after_release is not a u64"),
                None => return bad("missing live_after_release"),
            }
            for key in ["allocs", "frees"] {
                let arr = match o.get(key) {
                    Some(v) => match v.as_array() {
                        Some(a) => a,
                        None => return bad(&format!("{key} is not an array")),
                    },
                    None => return bad(&format!("missing {key}")),
                };
                if arr.len() > 65536 {
                    return bad(&format!("{key} exceeds the table-entry family cap"));
                }
                for ev in arr {
                    let Some(e) = ev.as_object() else {
                        return bad(&format!("{key} entry is not an object"));
                    };
                    match e.get("id") {
                        Some(v) if v.is_u64() => {}
                        Some(_) => return bad(&format!("{key} entry id is not a u64")),
                        None => return bad(&format!("{key} entry missing id")),
                    }
                    if key == "allocs" {
                        match e.get("bytes") {
                            Some(v) if v.is_u64() => {}
                            Some(_) => return bad("allocs entry bytes is not a u64"),
                            None => return bad("allocs entry missing bytes"),
                        }
                    }
                }
            }
            None
        })
    })
}

/// The owned-fixture judge: canonical result face (structural equality),
/// live=0 after the host release, allocation byte sizes in creation order
/// and free ids in release order — all against the PARENT's frozen table.
/// Counter-control seam (V-10 wrong-result / wrong-ledger): the test-only
/// sabotage mutates the PARENT's expectation copy so a legal fixture run
/// must FAIL. Never set in production.
/// The collections-fixture judge: same face axes as the owned judge
/// (canonical result, live=0, alloc bytes in creation order, free ids in
/// release order) against the PARENT's frozen collections table.
/// Counter-control seam (V-10): AV822_COLLECTIONS_EXPECT_SABOTAGE.
fn judge_collections_fixture(id: &str, step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    let (_, want_result, want_allocs, want_frees) = COLLECTIONS_CASES
        .iter()
        .find(|(fid, ..)| *fid == id)
        .expect("validated above");
    let mut want_result: serde_json::Value =
        serde_json::from_str(want_result).expect("frozen table JSON parses");
    let mut want_frees: Vec<u64> = want_frees.to_vec();
    match std::env::var("AV822_COLLECTIONS_EXPECT_SABOTAGE")
        .ok()
        .as_deref()
    {
        Some("result") => {
            want_result["bits"] =
                serde_json::json!(format!("{}!", want_result["bits"].as_str().unwrap_or("")));
        }
        Some("ledger") => {
            want_frees.rotate_left(1);
        }
        _ => {}
    }
    if step.outcome != "success" {
        mismatches.push(Mismatch {
            observation: "completion".into(),
            kind: "collections-outcome".into(),
            expected: "success".into(),
            actual: step_summary(step),
            first_diff: None,
        });
        return;
    }
    let collections = match &step.collections {
        Some(o) => o,
        None => {
            mismatches.push(Mismatch {
                observation: "collections".into(),
                kind: "collections-report".into(),
                expected: "the collections face on the step".into(),
                actual: "no collections report".into(),
                first_diff: None,
            });
            return;
        }
    };
    let got_result = collections
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if got_result != want_result {
        mismatches.push(Mismatch {
            observation: "collections".into(),
            kind: "collections-result".into(),
            expected: want_result.to_string(),
            actual: got_result.to_string(),
            first_diff: None,
        });
    }
    let live = collections
        .get("live_after_release")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::MAX);
    if live != 0 {
        mismatches.push(Mismatch {
            observation: "collections".into(),
            kind: "collections-live".into(),
            expected: "0 (host release empties the ledger)".into(),
            actual: format!("{live}"),
            first_diff: None,
        });
    }
    let got_allocs: Vec<usize> = collections
        .get("allocs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("bytes").and_then(|b| b.as_u64()).map(|b| b as usize))
                .collect()
        })
        .unwrap_or_default();
    if got_allocs != want_allocs.to_vec() {
        mismatches.push(Mismatch {
            observation: "collections".into(),
            kind: "collections-allocs".into(),
            expected: format!("{:?}", want_allocs),
            actual: format!("{:?}", got_allocs),
            first_diff: None,
        });
    }
    let got_frees: Vec<u64> = collections
        .get("frees")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    if got_frees != want_frees {
        mismatches.push(Mismatch {
            observation: "collections".into(),
            kind: "collections-frees".into(),
            expected: format!("{:?}", want_frees),
            actual: format!("{:?}", got_frees),
            first_diff: None,
        });
    }
}

/// PLAN-823 (R1-F02): the PARENT-side closures-fixture judgement — the
/// frozen table above decides; the worker's word is never trusted. The
/// AV823_CLOSURES_EXPECT_SABOTAGE seam is the V-10 negative control
/// (a lying expectation must turn the verdict red).
fn judge_closures_fixture(id: &str, step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    let (_, want_result, want_allocs, want_frees) = CLOSURES_CASES
        .iter()
        .find(|(fid, ..)| *fid == id)
        .expect("validated above");
    let mut want_result: serde_json::Value =
        serde_json::from_str(want_result).expect("frozen table JSON parses");
    let mut want_frees: Vec<u64> = want_frees.to_vec();
    match std::env::var("AV823_CLOSURES_EXPECT_SABOTAGE")
        .ok()
        .as_deref()
    {
        Some("result") => {
            want_result["bits"] =
                serde_json::json!(format!("{}!", want_result["bits"].as_str().unwrap_or("")));
        }
        Some("ledger") => {
            want_frees.rotate_left(1);
        }
        _ => {}
    }
    if step.outcome != "success" {
        mismatches.push(Mismatch {
            observation: "completion".into(),
            kind: "closures-outcome".into(),
            expected: "success".into(),
            actual: step_summary(step),
            first_diff: None,
        });
        return;
    }
    let closures = match &step.closures {
        Some(o) => o,
        None => {
            mismatches.push(Mismatch {
                observation: "closures".into(),
                kind: "closures-report".into(),
                expected: "the closures face on the step".into(),
                actual: "no closures report".into(),
                first_diff: None,
            });
            return;
        }
    };
    let got_result = closures
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if got_result != want_result {
        mismatches.push(Mismatch {
            observation: "closures".into(),
            kind: "closures-result".into(),
            expected: want_result.to_string(),
            actual: got_result.to_string(),
            first_diff: None,
        });
    }
    let live = closures
        .get("live_after_release")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::MAX);
    if live != 0 {
        mismatches.push(Mismatch {
            observation: "closures".into(),
            kind: "closures-live".into(),
            expected: "0 (host release empties the ledger)".into(),
            actual: format!("{live}"),
            first_diff: None,
        });
    }
    let got_allocs: Vec<usize> = closures
        .get("allocs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("bytes").and_then(|b| b.as_u64()).map(|b| b as usize))
                .collect()
        })
        .unwrap_or_default();
    if got_allocs != want_allocs.to_vec() {
        mismatches.push(Mismatch {
            observation: "closures".into(),
            kind: "closures-allocs".into(),
            expected: format!("{:?}", want_allocs),
            actual: format!("{:?}", got_allocs),
            first_diff: None,
        });
    }
    let got_frees: Vec<u64> = closures
        .get("frees")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    if got_frees != want_frees {
        mismatches.push(Mismatch {
            observation: "closures".into(),
            kind: "closures-frees".into(),
            expected: format!("{:?}", want_frees),
            actual: format!("{:?}", got_frees),
            first_diff: None,
        });
    }
}

/// PLAN-824: judge the dynamic fixture face (canonical result, live=0,
/// alloc bytes in creation order, free ids in release order) against the
/// PARENT's frozen dynamic table — plus the DYN axes: a Dyn-valued fixture
/// must report a successful portable round-trip. Counter-control seam
/// (V-10): AV824_DYNAMIC_EXPECT_SABOTAGE.
/// PLAN-865: source closure semantics and resource release are judged from
/// the parent's own K-case table, independently of the worker report.
fn judge_closures_source(
    id: &str,
    mode: Option<&str>,
    step: &StepOut,
    mismatches: &mut Vec<Mismatch>,
) {
    let (_, base_value, want_code, function_count) = CLOSURES_SOURCE_CASES
        .iter()
        .find(|(case_id, ..)| *case_id == id)
        .expect("source id validated above");
    if let Some(want_code) = want_code {
        if step.outcome != "diag" {
            mismatches.push(Mismatch {
                observation: "closures_source".into(),
                kind: "source-rejection".into(),
                expected: format!("compile-time diagnostic {want_code}"),
                actual: step_summary(step),
                first_diff: None,
            });
            return;
        }
        let got_code = step
            .diag
            .as_ref()
            .and_then(|diag| diag.get("code"))
            .and_then(|code| code.as_str())
            .unwrap_or_default();
        if got_code != *want_code {
            mismatches.push(Mismatch {
                observation: "closures_source".into(),
                kind: "source-diagnostic".into(),
                expected: (*want_code).to_string(),
                actual: got_code.to_string(),
                first_diff: None,
            });
        }
        if step.closures_source.is_some() {
            mismatches.push(Mismatch {
                observation: "closures_source".into(),
                kind: "source-rejection-face".into(),
                expected: "no runtime resource face for a rejected source".into(),
                actual: "runtime resource face present".into(),
                first_diff: None,
            });
        }
        return;
    }

    let want_value = if mode == Some("body-edit") {
        43
    } else {
        base_value.expect("success case has an expected value")
    };
    if step.outcome != "success" {
        mismatches.push(Mismatch {
            observation: "closures_source".into(),
            kind: "source-outcome".into(),
            expected: "success".into(),
            actual: step_summary(step),
            first_diff: None,
        });
        return;
    }
    if step.value != Some(want_value) {
        mismatches.push(Mismatch {
            observation: "closures_source".into(),
            kind: "source-result".into(),
            expected: want_value.to_string(),
            actual: format!("{:?}", step.value),
            first_diff: None,
        });
    }
    let Some(face) = &step.closures_source else {
        mismatches.push(Mismatch {
            observation: "closures_source".into(),
            kind: "source-resource-report".into(),
            expected: "resource summary from the closures update session".into(),
            actual: "no closures_source report".into(),
            first_diff: None,
        });
        return;
    };
    let live = face
        .get("live_after_release")
        .and_then(|value| value.as_u64())
        .unwrap_or(u64::MAX);
    if live != 0 {
        mismatches.push(Mismatch {
            observation: "closures_source".into(),
            kind: "source-live".into(),
            expected: "0 after frame and host release".into(),
            actual: live.to_string(),
            first_diff: None,
        });
    }
    let allocs = face
        .get("allocs")
        .and_then(|value| value.as_u64())
        .unwrap_or(u64::MAX);
    let frees = face
        .get("frees")
        .and_then(|value| value.as_u64())
        .unwrap_or(u64::MAX);
    if allocs != frees {
        mismatches.push(Mismatch {
            observation: "closures_source".into(),
            kind: "source-alloc-free-balance".into(),
            expected: format!("allocs == frees ({allocs})"),
            actual: frees.to_string(),
            first_diff: None,
        });
    }

    let expected_submitted = if mode == Some("body-edit") {
        1
    } else {
        *function_count
    } as u64;
    for (field, expected) in [
        ("submitted_units", expected_submitted),
        ("verified", expected_submitted),
        ("encoded", expected_submitted),
        ("admitted", expected_submitted),
    ] {
        let actual = face
            .get(field)
            .and_then(|value| value.as_u64())
            .unwrap_or(u64::MAX);
        if actual != expected {
            mismatches.push(Mismatch {
                observation: format!("closures_source.{field}"),
                kind: "counter",
                expected: expected.to_string(),
                actual: actual.to_string(),
                first_diff: None,
            });
        }
    }
    let expected_hits = if mode == Some("restart") {
        expected_submitted
    } else {
        0
    };
    let expected_misses = if mode == Some("restart") {
        0
    } else {
        expected_submitted
    };
    for (field, expected) in [
        ("cache_hits", expected_hits),
        ("cache_misses", expected_misses),
        ("store_writes", expected_misses),
    ] {
        let actual = face
            .get(field)
            .and_then(|value| value.as_u64())
            .unwrap_or(u64::MAX);
        if actual != expected {
            mismatches.push(Mismatch {
                observation: format!("closures_source.{field}"),
                kind: "counter",
                expected: expected.to_string(),
                actual: actual.to_string(),
                first_diff: None,
            });
        }
    }
    let current_hot = face
        .get("current_hot_bytes")
        .and_then(|value| value.as_u64())
        .unwrap_or(u64::MAX);
    let history_hot = face
        .get("history_hot_bytes")
        .and_then(|value| value.as_u64())
        .unwrap_or(u64::MAX);
    const CLOSURES_HOT_LIMIT: u64 = 8 * 1024 * 1024;
    if current_hot.saturating_add(history_hot) > CLOSURES_HOT_LIMIT {
        mismatches.push(Mismatch {
            observation: "closures_source".into(),
            kind: "source-hot-budget".into(),
            expected: format!("current + history <= {CLOSURES_HOT_LIMIT} bytes"),
            actual: format!("{} bytes", current_hot.saturating_add(history_hot)),
            first_diff: None,
        });
    }
}

fn judge_dynamic_fixture(id: &str, step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    let (_, want_result, want_allocs, want_frees) = DYNAMIC_CASES
        .iter()
        .find(|(fid, ..)| *fid == id)
        .expect("validated above");
    let mut want_result: serde_json::Value =
        serde_json::from_str(want_result).expect("frozen table JSON parses");
    let mut want_frees: Vec<u64> = want_frees.to_vec();
    match std::env::var("AV824_DYNAMIC_EXPECT_SABOTAGE")
        .ok()
        .as_deref()
    {
        Some("result") => {
            want_result["bits"] =
                serde_json::json!(format!("{}!", want_result["bits"].as_str().unwrap_or("")));
        }
        Some("ledger") => {
            want_frees.rotate_left(1);
        }
        _ => {}
    }
    if step.outcome != "success" {
        mismatches.push(Mismatch {
            observation: "completion".into(),
            kind: "dynamic-outcome".into(),
            expected: "success".into(),
            actual: step_summary(step),
            first_diff: None,
        });
        return;
    }
    let dynamic = match &step.collections {
        Some(o) => o,
        None => {
            mismatches.push(Mismatch {
                observation: "dynamic".into(),
                kind: "dynamic-report".into(),
                expected: "the dynamic face on the step".into(),
                actual: "no dynamic report".into(),
                first_diff: None,
            });
            return;
        }
    };
    let got_result = dynamic
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if got_result != want_result {
        mismatches.push(Mismatch {
            observation: "dynamic".into(),
            kind: "dynamic-result".into(),
            expected: want_result.to_string(),
            actual: got_result.to_string(),
            first_diff: None,
        });
    }
    let live = dynamic
        .get("live_after_release")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::MAX);
    if live != 0 {
        mismatches.push(Mismatch {
            observation: "dynamic".into(),
            kind: "dynamic-live".into(),
            expected: "0 (host release empties the ledger)".into(),
            actual: format!("{live}"),
            first_diff: None,
        });
    }
    let got_allocs: Vec<usize> = dynamic
        .get("allocs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("bytes").and_then(|b| b.as_u64()).map(|b| b as usize))
                .collect()
        })
        .unwrap_or_default();
    if got_allocs != want_allocs.to_vec() {
        mismatches.push(Mismatch {
            observation: "dynamic".into(),
            kind: "dynamic-allocs".into(),
            expected: format!("{:?}", want_allocs),
            actual: format!("{:?}", got_allocs),
            first_diff: None,
        });
    }
    let got_frees: Vec<u64> = dynamic
        .get("frees")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    if got_frees != want_frees {
        mismatches.push(Mismatch {
            observation: "dynamic".into(),
            kind: "dynamic-frees".into(),
            expected: format!("{:?}", want_frees),
            actual: format!("{:?}", got_frees),
            first_diff: None,
        });
    }
    // The DYN axes: a Dyn-valued fixture must carry a successful portable
    // round-trip on its step face (V-14's byte-equivalence control at the
    // recipe layer).
    if let Some(dyn_face) = step.dynamic.as_ref().filter(|d| !d.is_null()) {
        let roundtrip = dyn_face.get("portable_roundtrip").and_then(|v| v.as_bool());
        if roundtrip != Some(true) {
            mismatches.push(Mismatch {
                observation: "dynamic".into(),
                kind: "dynamic-portable".into(),
                expected: "true (frozen payload restores bit-exactly)".into(),
                actual: format!("{roundtrip:?}"),
                first_diff: None,
            });
        }
    }
}

fn judge_owned_fixture(id: &str, step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    let (_, want_result, want_allocs, want_frees) = OWNED_CASES
        .iter()
        .find(|(fid, ..)| *fid == id)
        .expect("validated above");
    let mut want_result: serde_json::Value =
        serde_json::from_str(want_result).expect("frozen table JSON parses");
    let mut want_frees: Vec<u64> = want_frees.to_vec();
    match std::env::var("AV821_OWNED_EXPECT_SABOTAGE").ok().as_deref() {
        Some("result") => {
            // Flip one observable leaf: the record's first str field bytes.
            want_result["fields"][0]["utf8"] = serde_json::json!(format!(
                "{}!",
                want_result["fields"][0]["utf8"].as_str().unwrap_or("")
            ));
        }
        Some("ledger") => {
            want_frees.rotate_left(1);
        }
        _ => {}
    }
    if step.outcome != "success" {
        mismatches.push(Mismatch {
            observation: "completion".into(),
            kind: "owned-outcome".into(),
            expected: "success".into(),
            actual: step_summary(step),
            first_diff: None,
        });
        return;
    }
    let owned = match &step.owned {
        Some(o) => o,
        None => {
            mismatches.push(Mismatch {
                observation: "owned".into(),
                kind: "owned-report".into(),
                expected: "the owned face on the step".into(),
                actual: "no owned report".into(),
                first_diff: None,
            });
            return;
        }
    };
    let got_result = owned
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if got_result != want_result {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-result".into(),
            expected: want_result.to_string(),
            actual: got_result.to_string(),
            first_diff: None,
        });
    }
    let live = owned
        .get("live_after_release")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::MAX);
    if live != 0 {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-live".into(),
            expected: "0 (host release empties the ledger)".into(),
            actual: format!("{live}"),
            first_diff: None,
        });
    }
    let alloc_bytes: Vec<u64> = owned
        .get("allocs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("bytes").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    let want_allocs: Vec<u64> = want_allocs.iter().map(|b| *b as u64).collect();
    if alloc_bytes != want_allocs {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-allocs".into(),
            expected: format!("{want_allocs:?}"),
            actual: format!("{alloc_bytes:?}"),
            first_diff: None,
        });
    }
    // Creation order = strictly ascending ids (the ledger's monotonic
    // allocation identity).
    let alloc_ids: Vec<u64> = owned
        .get("allocs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    if !alloc_ids.windows(2).all(|w| w[0] < w[1]) {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-alloc-order".into(),
            expected: "strictly ascending allocation ids".into(),
            actual: format!("{alloc_ids:?}"),
            first_diff: None,
        });
    }
    let free_ids: Vec<u64> = owned
        .get("frees")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    if free_ids != want_frees {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-frees".into(),
            expected: format!("{want_frees:?}"),
            actual: format!("{free_ids:?}"),
            first_diff: None,
        });
    }
}

/// Admission rule (F820-R8-01): EVERY typed report on a data response is
/// shape-validated HERE — before and independent of any oracle branch, so
/// a matching exit expectation (or any other oracle choice) can never let
/// an illegal bool encoding or an unknown kind tag through.
fn data_malformed_typed(steps: &[StepOut]) -> Option<String> {
    steps.iter().find_map(|s| {
        s.typed
            .as_ref()
            .and_then(typed_shape_violation)
            .map(|v| format!("malformed typed report on step `{}`: {}", s.op, v))
    })
}

fn judge_loaded_runtime(step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    if step.outcome != "success" {
        return;
    }
    let sabotage = std::env::var("AV825_JUDGE_SABOTAGE").unwrap_or_default();
    let observed = |key: &str| {
        if sabotage == key {
            0
        } else {
            step.counters.get(key).copied().unwrap_or(0)
        }
    };
    for (key, minimum) in [
        ("loaded_index_builds", 1),
        ("loaded_pc_lookups", 1),
        ("loaded_param_map_queries", 1),
    ] {
        if observed(key) < minimum {
            mismatches.push(Mismatch {
                observation: "loaded-runtime".into(),
                kind: key.into(),
                expected: format!(">= {minimum}"),
                actual: observed(key).to_string(),
                first_diff: None,
            });
        }
    }
    for key in ["reference_pc_lookups", "reference_param_map_copies"] {
        if observed(key) != 0 {
            mismatches.push(Mismatch {
                observation: "loaded-runtime".into(),
                kind: key.into(),
                expected: "0".into(),
                actual: observed(key).to_string(),
                first_diff: None,
            });
        }
    }
}

/// PLAN-826 T-02: the compact-wire judge face. A run under
/// `ac-av/compact-run-v1` is only green when the WORKER'S OWN COUNTERS
/// prove the compact generation actually carried the units AND the whole
/// closure (emission + admission under the AVAC codec) and the executed
/// closure rode the D825 loaded representation — a legacy-wire run of the
/// same source must redden this face.
fn judge_compact_runtime(step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    if step.outcome != "success" {
        return;
    }
    let sabotage = std::env::var("AV826_JUDGE_SABOTAGE").unwrap_or_default();
    let observed = |key: &str| {
        if sabotage == key {
            0
        } else {
            step.counters.get(key).copied().unwrap_or(0)
        }
    };
    for (key, minimum) in [
        ("abc_wire_compact_units", 1),
        ("abc_wire_compact_closure", 1),
        ("loaded_index_builds", 1),
        ("loaded_pc_lookups", 1),
    ] {
        if observed(key) < minimum {
            mismatches.push(Mismatch {
                observation: "compact-wire".into(),
                kind: key.into(),
                expected: format!(">= {minimum}"),
                actual: observed(key).to_string(),
                first_diff: None,
            });
        }
    }
    if observed("reference_pc_lookups") != 0 {
        mismatches.push(Mismatch {
            observation: "compact-wire".into(),
            kind: "reference_pc_lookups".into(),
            expected: "0".into(),
            actual: observed("reference_pc_lookups").to_string(),
            first_diff: None,
        });
    }
}

/// PLAN-866: the resident structural faces — judged from the worker's
/// reported counters against the frozen contract, independent of the
/// value oracle: the chain was NEVER restarted (exactly one chain entry),
/// no whole-module recompute ran, and zero production stages moved for
/// unchanged functions. A restart/pause-only implementation cannot
/// satisfy these together with the value oracle.
fn judge_resident_faces(step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    let chain_entries = step.counters.get("resident:chain_entries").copied();
    match chain_entries {
        Some(1) => {}
        other => mismatches.push(Mismatch {
            observation: "counters".into(),
            kind: "resident-chain-entries".into(),
            expected: "1 (the resident chain is never restarted)".into(),
            actual: format!("{other:?}"),
            first_diff: None,
        }),
    }
    let unaffected = step
        .counters
        .get("resident:unaffected_stages")
        .copied()
        .unwrap_or(u64::MAX);
    if unaffected != 0 {
        mismatches.push(Mismatch {
            observation: "counters".into(),
            kind: "resident-unaffected-stages".into(),
            expected: "0 (unchanged functions do zero production work)".into(),
            actual: unaffected.to_string(),
            first_diff: None,
        });
    }
    let whole = step
        .counters
        .get("resident:whole_module_stages")
        .copied()
        .unwrap_or(u64::MAX);
    if whole != 0 {
        mismatches.push(Mismatch {
            observation: "counters".into(),
            kind: "resident-whole-module".into(),
            expected: "0 (no whole-module recompute on the online path)".into(),
            actual: whole.to_string(),
            first_diff: None,
        });
    }
}

fn judge_expectation(
    family: RecipeFamily,
    step: &StepOut,
    oracle: &ResolvedOracle,
    mismatches: &mut Vec<Mismatch>,
) {
    let expect = oracle;
    match &expect.completion {
        Completion::Success => {
            if step.outcome != "success" {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: "success".into(),
                    actual: format!("{} ({})", step.outcome, step_summary(step)),
                    first_diff: None,
                });
                return;
            }
            // PLAN-862: the RESULT CHANNEL oracle. A Unit success has no i32
            // payload, so the channel name is the only observable that
            // separates it from an int entry returning 0; the payload rules
            // below keep the two faces from being mixed (an int value on a
            // unit channel, or a missing value on a value channel, is red).
            if let Some(want) = &expect.av_channel {
                if step.channel.as_deref() != Some(want.as_str()) {
                    mismatches.push(Mismatch {
                        observation: "av_channel".into(),
                        kind: "channel".into(),
                        expected: want.clone(),
                        actual: step.channel.clone().unwrap_or_default(),
                        first_diff: None,
                    });
                }
                match want.as_str() {
                    "unit" => {
                        if step.value.is_some() {
                            mismatches.push(Mismatch {
                                observation: "av_channel".into(),
                                kind: "payload".into(),
                                expected: "no i32 payload on the unit channel".into(),
                                actual: format!("{:?}", step.value),
                                first_diff: None,
                            });
                        }
                    }
                    "value" => {
                        if step.value.is_none() {
                            mismatches.push(Mismatch {
                                observation: "av_channel".into(),
                                kind: "payload".into(),
                                expected: "an i32 payload on the value channel".into(),
                                actual: "none".into(),
                                first_diff: None,
                            });
                        }
                    }
                    _ => {}
                }
            }
            if let Some(want) = expect.exit {
                if step.value != Some(want) {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "exit".into(),
                        expected: format!("{want}"),
                        actual: format!("{:?}", step.value),
                        first_diff: None,
                    });
                }
            } else if expect.av_channel.is_none()
                && expect.stdout.is_none()
                && expect.stderr.is_none()
            {
                // Success without a value oracle still requires a value
                // present — FAMILY-ISOLATED (F820-R7-01): the typed
                // channel is a DATA-recipe increment only. A core/i32
                // success must carry the OLD value channel (a typed
                // report on a core response is an admission violation,
                // checked there); a data success may satisfy presence
                // via either channel, and a data typed report must have
                // the canonical tag/width shape. PLAN-821: an owned
                // success satisfies presence via the owned face (the
                // record/Str results have no i32 form).
                let present = step.value.is_some()
                    || (family == RecipeFamily::Data && step.typed.is_some())
                    || (family == RecipeFamily::Owned && step.owned.is_some());
                if !present {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "missing".into(),
                        expected: "an AV success value".into(),
                        actual: "none".into(),
                        first_diff: None,
                    });
                }
                // (Typed SHAPE is an admission rule — see
                // `data_malformed_typed`; nothing here re-validates it.)
            }
        }
        Completion::Error {
            stage,
            code,
            message,
        } => match step.outcome.as_str() {
            "trap" => {
                if *stage != Stage::Run {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{:?}", stage),
                        actual: "run (trap)".into(),
                        first_diff: None,
                    });
                }
                if let Some(want_code) = code {
                    if step.trap.as_deref() != Some(want_code.as_str()) {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want_code.clone(),
                            actual: step.trap.clone().unwrap_or_default(),
                            first_diff: None,
                        });
                    }
                }
                if let Some(matcher) = message {
                    let text = step.trap.clone().unwrap_or_default();
                    if !match_message(matcher, &text) {
                        mismatches.push(Mismatch {
                            observation: "message".into(),
                            kind: "message".into(),
                            expected: format!("{matcher:?}"),
                            actual: text,
                            first_diff: None,
                        });
                    }
                }
            }
            "diag" => {
                let diag = step.diag.clone().unwrap_or_default();
                let got_stage = diag["stage"].as_str().unwrap_or_default();
                let want_stage = map_diag_stage(got_stage);
                if want_stage != *stage {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{:?}", stage),
                        actual: format!("{got_stage} (→ {want_stage:?})"),
                        first_diff: None,
                    });
                }
                let got_code = diag["code"].as_str().unwrap_or_default().to_string();
                if let Some(want_code) = code {
                    if got_code != *want_code {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want_code.clone(),
                            actual: got_code,
                            first_diff: None,
                        });
                    }
                }
                if let Some(matcher) = message {
                    let text = diag["message"].as_str().unwrap_or_default().to_string();
                    if !match_message(matcher, &text) {
                        mismatches.push(Mismatch {
                            observation: "message".into(),
                            kind: "message".into(),
                            expected: format!("{matcher:?}"),
                            actual: text,
                            first_diff: None,
                        });
                    }
                }
            }
            other => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: "an error completion".into(),
                    actual: format!("{other} ({})", step_summary(step)),
                    first_diff: None,
                });
            }
        },
    }
}

fn match_message(matcher: &MessageMatcher, text: &str) -> bool {
    use MessageMatcher::*;
    match matcher {
        Exact(w) => text == w,
        Contains(w) => text.contains(w.as_str()),
    }
}

fn step_summary(step: &StepOut) -> String {
    match step.outcome.as_str() {
        "trap" => format!("trap {}", step.trap.clone().unwrap_or_default()),
        "diag" => format!(
            "diag {}",
            step.diag
                .as_ref()
                .and_then(|d| d.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("?")
        ),
        other => other.to_string(),
    }
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// FNV-1a 64 over the request's source bytes, hex-16 — the SAME
/// fingerprint the worker's driver stamps per run step, re-derived
/// parent-side to bind every reported result to the current revision
/// (F798-R3-05).
fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Test-only seam for the prep-executor controls (F798-R15-05).
/// tests_seam 可见性（815 T-04/814 T-04 两计划同款修复）：消费测试
/// （auto-lang 侧 ac_av/probe_adapter 探针族）在独立 crate——pub(crate)
/// 在 M814-A 目录迁移后跨 crate 不可达（f6e07acee 起 latent 断点）。
/// 缝整体在 test-cases feature 门内（814 交付语义：测试基建面），
/// pub 化零生产面变化，仅测试定位器。
pub mod tests_seam {
    pub fn locate_worker_for_test() -> Result<std::path::PathBuf, String> {
        super::locate_worker()
    }
}

#[cfg(all(test, feature = "test-ace"))]
mod judge_tests {
    //! F820-R7-01 pins: the typed channel is a DATA-recipe increment —
    //! the shared judge and admission rules are exercised HERE, on the
    //! production functions, with synthetic steps (the reviewer's
    //! fault-worker e2e shapes: core success missing the i32 value, with
    //! and without an attached typed object).

    use super::*;

    fn step(value: Option<i32>, typed: Option<TypedOut>) -> StepOut {
        StepOut {
            op: "run".into(),
            outcome: "success".into(),
            value,
            typed,
            trap: None,
            diag: None,
            counters: Default::default(),
            source_hash: String::new(),
            wall_ms: 0.0,
            trace: Vec::new(),
            spans: Vec::new(),
            owned: None,
            collections: None,
            closures: None,
            closures_source: None,
            dynamic: None,
            channel: None,
            console: None,
        }
    }

    fn typed(kind: &str, bits: &str) -> Option<TypedOut> {
        Some(TypedOut {
            kind: kind.into(),
            bits: bits.into(),
        })
    }

    fn oracle_no_exit() -> ResolvedOracle {
        ResolvedOracle {
            completion: auto_test_case::contract::Completion::Success,
            result: None,
            stdout: None,
            stderr: None,
            artifacts: Vec::new(),
            exit: None,
            perf: None,
            probe: None,
            av_channel: None,
            oracle_sha256: String::new(),
        }
    }

    fn mismatches(family: RecipeFamily, step: &StepOut) -> Vec<Mismatch> {
        let oracle = oracle_no_exit();
        let mut ms = Vec::new();
        judge_expectation(family, step, &oracle, &mut ms);
        ms
    }

    /// PLAN-862 (V-02): the Unit channel oracle is asserted on the CHANNEL,
    /// and a Unit payload may never carry an i32 value — the negative
    /// control proves the judge reddens a "unit→0" collapse instead of
    /// accepting it.
    #[test]
    fn av_channel_oracle_separates_unit_from_zero() {
        let unit_oracle = ResolvedOracle {
            av_channel: Some("unit".into()),
            ..oracle_no_exit()
        };
        let mut unit_step = step(None, None);
        unit_step.channel = Some("unit".into());
        let mut ms = Vec::new();
        judge_expectation(RecipeFamily::Core, &unit_step, &unit_oracle, &mut ms);
        assert!(ms.is_empty(), "a real unit channel must pass: {ms:?}");

        // THE mutation: the pipeline starts reporting I32(0) for the unit
        // entry — same payload shape an old consumer would print, different
        // channel. The oracle must redden it.
        let mut collapsed = step(Some(0), None);
        collapsed.channel = Some("value".into());
        let mut ms = Vec::new();
        judge_expectation(RecipeFamily::Core, &collapsed, &unit_oracle, &mut ms);
        assert!(
            ms.iter().any(|m| m.kind == "channel"),
            "unit→0 collapse must be red on the channel: {ms:?}"
        );

        // A value channel claiming a payload but reporting none is red too.
        let value_oracle = ResolvedOracle {
            av_channel: Some("value".into()),
            ..oracle_no_exit()
        };
        let mut lying = step(None, None);
        lying.channel = Some("value".into());
        let mut ms = Vec::new();
        judge_expectation(RecipeFamily::Core, &lying, &value_oracle, &mut ms);
        assert!(
            ms.iter().any(|m| m.kind == "payload"),
            "value channel without a payload must be red: {ms:?}"
        );
    }

    #[test]
    fn core_success_requires_the_old_value_channel() {
        // Without typed: the pre-R6 semantics — a missing value mismatches.
        let ms = mismatches(RecipeFamily::Core, &step(None, None));
        assert!(ms.iter().any(|m| m.kind == "missing"));
        // With a typed object attached (the R7 fault-worker shape): the
        // typed channel must NOT satisfy a core success — still missing.
        let ms = mismatches(RecipeFamily::Core, &step(None, typed("bool", "1")));
        assert!(
            ms.iter().any(|m| m.kind == "missing"),
            "typed must not satisfy a core value presence: {ms:?}"
        );
        // And the admission gate rejects the typed-on-core shape outright
        // (infra, never a pass-shaped extra channel).
        let steps = vec![step(None, typed("bool", "1"))];
        assert!(super::typed_on_non_data(RecipeFamily::Core, &steps));
        // A clean core response carries no typed at all.
        assert!(!super::typed_on_non_data(
            RecipeFamily::Core,
            &[step(Some(42), None)]
        ));
        // Positive: value present, no typed — no presence mismatch.
        assert!(mismatches(RecipeFamily::Core, &step(Some(42), None)).is_empty());
    }

    #[test]
    fn data_success_presence_accepts_the_typed_channel() {
        // bool has no i32 form: presence via typed (the F820-R6-01 fix,
        // now family-isolated to data).
        assert!(mismatches(RecipeFamily::Data, &step(None, typed("bool", "1"))).is_empty());
        assert!(mismatches(RecipeFamily::Data, &step(Some(42), None)).is_empty());
        assert!(mismatches(RecipeFamily::Data, &step(None, None))
            .iter()
            .any(|m| m.kind == "missing"));
    }

    #[test]
    fn data_typed_reports_must_have_the_canonical_shape() {
        // The shape rule is an ORACLE-INDEPENDENT admission check
        // (`data_malformed_typed` takes only the steps): the R8 fault
        // table — illegal bool encodings, unknown tags — is red even
        // when a value is present and an exit oracle would match.
        for (kind, bits) in [
            ("bool", "2"),               // bool is exactly 0/1 (contract §3)
            ("bool", "00"),              // non-canonical padded encoding
            ("i32", "02a"),              // leading zero = non-canonical
            ("weird", "1"),              // unknown kind tag
            ("f32", "3f80"),             // wrong fixed width
            ("u64", "zzzzzzzzzzzzzzzz"), // non-hex
        ] {
            assert!(
                typed_shape_violation(&typed(kind, bits).unwrap()).is_some(),
                "{kind}/{bits} must be rejected"
            );
            // ... and the admission gate reports it with a VALUE present
            // (the exit-oracle bypass shape from R8: value=42 + matching
            // exit would have passed the judge — admission still reddens).
            let steps = [step(Some(42), typed(kind, bits))];
            assert!(
                data_malformed_typed(&steps).is_some(),
                "{kind}/{bits} must be an admission violation regardless of oracle"
            );
        }
        // Legal shapes: bool 0/1 both (success-only asserts no value
        // semantics), the frozen NaN payload, wide fixed widths, i32.
        for (kind, bits) in [
            ("bool", "0"),
            ("bool", "1"),
            ("i32", "0"), // the UNIQUE canonical zero (F820-R9-01)
            ("i32", "2a"),
            ("i32", "ffffffff"), // -1
            ("i32", "80000000"), // i32::MIN
            ("f32", "7f80ad00"),
            ("f32", "3f800000"),
            ("f32", "00000000"), // +0.0 (fixed width: zeros are the width)
            ("f64", "8000000000000000"),
            ("f64", "0000000000000000"), // wide zero (fixed width)
            ("u64", "ffffffffffffffff"),
            ("u64", "0000000000000000"),
        ] {
            assert!(typed_shape_violation(&typed(kind, bits).unwrap()).is_none());
        }
        assert!(data_malformed_typed(&[step(None, typed("bool", "1"))]).is_none());
        assert!(data_malformed_typed(&[step(Some(42), None)]).is_none());
    }

    /// PLAN-821 pins: the owned channel is an OWNED-recipe increment — the
    /// admission rules and the fixture judge are exercised on the
    /// production functions with synthetic faces.
    #[test]
    fn owned_face_is_family_isolated_and_shape_validated() {
        let owned_face = serde_json::json!({
            "result": { "kind": "str", "utf8": "Auto" },
            "live_after_release": 0,
            "allocs": [ { "id": 0, "bytes": 4 } ],
            "frees": [ { "id": 0 } ],
        });
        let mut s = step(None, None);
        s.owned = Some(owned_face.clone());
        // Present on a non-owned recipe → admission violation.
        assert!(owned_on_non_owned(RecipeFamily::Core, &[s.clone()]));
        assert!(owned_on_non_owned(RecipeFamily::Data, &[s.clone()]));
        // Present on the owned recipe → fine, and the shape is canonical.
        assert!(!owned_on_non_owned(RecipeFamily::Owned, &[s.clone()]));
        assert!(malformed_owned(&[s.clone()]).is_none());
        // Typed stays data-only even on owned steps.
        let mut with_typed = step(None, typed("bool", "1"));
        with_typed.owned = Some(owned_face);
        assert!(typed_on_non_data(RecipeFamily::Owned, &[with_typed]));
        // Malformed faces: missing/typed result leaf, non-u64 live, an
        // alloc entry without bytes, a free id that is a string.
        for bad in [
            serde_json::json!({ "result": "str", "live_after_release": 0,
                                "allocs": [], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": -1, "allocs": [], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": 0,
                                "allocs": [ { "id": 0 } ], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": 0,
                                "allocs": [], "frees": [ { "id": "0" } ] }),
            // Missing-key faces (merge-gate repair): a `?`-skip used to
            // let these through as "not malformed".
            serde_json::json!({ "live_after_release": 0,
                                "allocs": [], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": 0, "allocs": [] }),
        ] {
            let mut b = step(None, None);
            b.owned = Some(bad);
            assert!(
                malformed_owned(&[b]).is_some(),
                "malformed owned face must be an admission violation"
            );
        }
    }

    #[test]
    fn owned_fixture_judge_distinguishes_every_face() {
        let good = serde_json::json!({
            "result": {
                "kind": "record", "name": "t_rec_Note",
                "fields": [
                    { "kind": "str", "utf8": "Auto" },
                    { "kind": "i32", "bits": "4" },
                ],
            },
            "live_after_release": 0,
            "allocs": [ { "id": 0, "bytes": 4 }, { "id": 1, "bytes": 4 } ],
            "frees": [ { "id": 0 }, { "id": 1 } ],
        });
        fn with(face: serde_json::Value) -> StepOut {
            let mut s = step(None, None);
            s.owned = Some(face);
            s
        }
        let mut ms = Vec::new();
        judge_owned_fixture("first-case", &with(good.clone()), &mut ms);
        assert!(
            ms.is_empty(),
            "the frozen first-case face must be green: {ms:?}"
        );
        // Wrong result bytes / live≠0 / an extra allocation / reversed
        // free order — each mutation alone (from the pristine face) must
        // redden its own axis.
        let pristine = good.clone();
        let mut m_result = pristine.clone();
        m_result["result"]["fields"][0]["utf8"] = serde_json::json!("Autp");
        let mut m_live = pristine.clone();
        m_live["live_after_release"] = serde_json::json!(1);
        let mut m_allocs = pristine.clone();
        m_allocs["allocs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id": 2, "bytes": 4
            }));
        let mut m_frees = pristine.clone();
        m_frees["frees"].as_array_mut().unwrap().reverse();
        for (axis, face) in [
            ("result", m_result),
            ("live", m_live),
            ("allocs", m_allocs),
            ("frees", m_frees),
        ] {
            let mut ms = Vec::new();
            judge_owned_fixture("first-case", &with(face), &mut ms);
            assert!(!ms.is_empty(), "{axis} mutation must not pass: {ms:?}");
        }
        // No owned report at all → the report mismatch (not a silent pass).
        let mut ms = Vec::new();
        judge_owned_fixture("first-case", &step(None, None), &mut ms);
        assert!(ms.iter().any(|m| m.kind == "owned-report"));
    }
}

#[cfg(all(test, feature = "test-ace"))]
mod closures_judge_tests {
    //! PLAN-823 (R1-F02): the PARENT-side closures judgement — the frozen
    //! table decides; a lying worker face turns the verdict red.

    use super::*;

    fn cstep(face: serde_json::Value) -> StepOut {
        StepOut {
            op: "run".into(),
            outcome: "success".into(),
            value: None,
            typed: None,
            trap: None,
            diag: None,
            counters: Default::default(),
            source_hash: String::new(),
            wall_ms: 0.0,
            trace: Vec::new(),
            spans: Vec::new(),
            owned: None,
            collections: None,
            closures: Some(face),
            closures_source: None,
            dynamic: None,
            channel: None,
            console: None,
        }
    }

    #[test]
    fn av_closures_judge_passes_the_frozen_face() {
        let face = serde_json::json!({
            "result": { "kind": "i32", "bits": "1d" },
            "live_after_release": 0,
            "allocs": [
                { "id": 0, "bytes": 4 }, { "id": 1, "bytes": 4 }, { "id": 2, "bytes": 0 },
                { "id": 3, "bytes": 4 }, { "id": 4, "bytes": 6 },
            ],
            "frees": [
                { "id": 0, "bytes": 4 }, { "id": 4, "bytes": 6 }, { "id": 1, "bytes": 4 },
                { "id": 2, "bytes": 0 }, { "id": 3, "bytes": 4 },
            ],
        });
        let mut ms = Vec::new();
        judge_closures_fixture("first-case", &cstep(face), &mut ms);
        assert!(ms.is_empty(), "{ms:?}");
    }

    #[test]
    fn av_closures_judge_detects_lies() {
        // wrong result bits
        let mut face = serde_json::json!({
            "result": { "kind": "i32", "bits": "2a" },
            "live_after_release": 0,
            "allocs": [], "frees": [],
        });
        let mut ms = Vec::new();
        judge_closures_fixture("first-case", &cstep(face.clone()), &mut ms);
        assert!(!ms.is_empty(), "a wrong result must go red");
        // leak + wrong free order + missing face
        face["live_after_release"] = serde_json::json!(1);
        let mut ms2 = Vec::new();
        judge_closures_fixture("first-case", &cstep(face), &mut ms2);
        assert!(ms2.len() >= 2, "leak AND result both red: {ms2:?}");
        let mut ms3 = Vec::new();
        judge_closures_fixture("first-case", &cstep(serde_json::json!({})), &mut ms3);
        assert!(!ms3.is_empty(), "an empty face must go red");
        // non-success outcome
        let mut ms4 = Vec::new();
        let mut bad = cstep(serde_json::json!({}));
        bad.outcome = "trap".into();
        judge_closures_fixture("first-case", &bad, &mut ms4);
        assert!(!ms4.is_empty(), "a trapped step must go red");
    }
}

#[cfg(all(test, feature = "test-ace"))]
mod closures_source_judge_tests {
    use super::*;

    fn successful_step(value: Option<i32>) -> StepOut {
        StepOut {
            op: "run".into(),
            outcome: "success".into(),
            value,
            typed: None,
            trap: None,
            diag: None,
            counters: Default::default(),
            source_hash: String::new(),
            wall_ms: 0.0,
            trace: Vec::new(),
            spans: Vec::new(),
            owned: None,
            collections: None,
            closures: None,
            closures_source: None,
            dynamic: None,
            channel: None,
            console: None,
        }
    }

    #[test]
    fn source_judge_checks_result_release_and_unsupported_diagnostics() {
        let mut good = successful_step(Some(42));
        good.closures_source = Some(serde_json::json!({
            "live_after_release": 0,
            "allocs": 2,
            "frees": 2,
            "submitted_units": 2,
            "verified": 2,
            "encoded": 2,
            "admitted": 2,
            "store_writes": 2,
            "cache_hits": 0,
            "cache_misses": 2,
            "current_hot_bytes": 4096,
            "history_hot_bytes": 0,
            "process_peak_bytes": 64 * 1024 * 1024,
        }));
        let mut mismatches = Vec::new();
        judge_closures_source("K01", Some("fresh"), &good, &mut mismatches);
        assert!(mismatches.is_empty(), "valid source face: {mismatches:?}");

        let mut leaked = good.clone();
        leaked.closures_source.as_mut().unwrap()["live_after_release"] = serde_json::json!(1);
        let mut mismatches = Vec::new();
        judge_closures_source("K01", Some("fresh"), &leaked, &mut mismatches);
        assert!(
            mismatches.iter().any(|m| m.kind == "source-live"),
            "a live closure environment must fail the parent oracle: {mismatches:?}"
        );

        let mut unbalanced = good.clone();
        unbalanced.closures_source.as_mut().unwrap()["frees"] = serde_json::json!(1);
        let mut mismatches = Vec::new();
        judge_closures_source("K01", Some("fresh"), &unbalanced, &mut mismatches);
        assert!(
            mismatches
                .iter()
                .any(|m| m.kind == "source-alloc-free-balance"),
            "an unmatched closure allocation must fail: {mismatches:?}"
        );

        let mut restarted = successful_step(Some(42));
        restarted.closures_source = Some(serde_json::json!({
            "live_after_release": 0,
            "allocs": 2,
            "frees": 2,
            "submitted_units": 2,
            "verified": 2,
            "encoded": 2,
            "admitted": 2,
            "store_writes": 0,
            "cache_hits": 2,
            "cache_misses": 0,
            "current_hot_bytes": 4096,
            "history_hot_bytes": 0,
            "process_peak_bytes": 64 * 1024 * 1024,
        }));
        let mut mismatches = Vec::new();
        judge_closures_source("K01", Some("restart"), &restarted, &mut mismatches);
        assert!(
            mismatches.is_empty(),
            "file cache restart face: {mismatches:?}"
        );

        let mut edited = successful_step(Some(43));
        edited.closures_source = Some(serde_json::json!({
            "live_after_release": 0,
            "allocs": 2,
            "frees": 2,
            "submitted_units": 1,
            "verified": 1,
            "encoded": 1,
            "admitted": 1,
            "store_writes": 1,
            "cache_hits": 0,
            "cache_misses": 1,
            "current_hot_bytes": 4096,
            "history_hot_bytes": 1024,
            "process_peak_bytes": 64 * 1024 * 1024,
        }));
        let mut mismatches = Vec::new();
        judge_closures_source("K01", Some("body-edit"), &edited, &mut mismatches);
        assert!(
            mismatches.is_empty(),
            "body edit must rebuild one unit: {mismatches:?}"
        );

        let mut rejected = successful_step(None);
        rejected.outcome = "diag".into();
        rejected.diag = Some(serde_json::json!({
            "stage": "bind",
            "code": "source.closure-borrow-escape",
            "message": "borrow-capturing closure escapes",
        }));
        let mut mismatches = Vec::new();
        judge_closures_source("K04", None, &rejected, &mut mismatches);
        assert!(mismatches.is_empty(), "valid rejection: {mismatches:?}");

        rejected.diag.as_mut().unwrap()["code"] = serde_json::json!("source.other");
        let mut mismatches = Vec::new();
        judge_closures_source("K04", None, &rejected, &mut mismatches);
        assert!(
            mismatches.iter().any(|m| m.kind == "source-diagnostic"),
            "the source diagnostic code is independently frozen: {mismatches:?}"
        );
    }
}

/// PLAN-863: exact current identity used by stale adjudication for console rows.
pub fn current_console_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    match worker_exe_sha256() {
        Ok(sha) => probe_toolchain_axes(&sha, RecipeFamily::Console, None)
            .into_iter()
            .collect(),
        Err(_) => std::collections::BTreeMap::new(),
    }
}

/// Console transport admission is independent of a matching oracle.
fn console_admission(family: RecipeFamily, steps: &[StepOut]) -> Option<String> {
    if family != RecipeFamily::Console {
        return steps
            .iter()
            .any(|s| s.console.is_some())
            .then(|| "console capture on a non-console recipe".into());
    }
    if steps.len() != 1 || steps[0].op != "run" {
        return Some("console response requires exactly one run".into());
    }
    let step = &steps[0];
    let Some(c) = &step.console else {
        return Some("console capture is missing; never an empty stream".into());
    };
    if c.schema != "av.console-capture/1" || c.mir_lowering != EXPECTED_MIR_LOWERING_VERSION {
        return Some("console schema/lowering identity mismatch".into());
    }
    if !c.stdout_present || !c.stderr_present {
        return Some("console stream was not captured".into());
    }
    if c.stdout_truncated || c.stderr_truncated {
        return Some("console stream is truncated; never successful output".into());
    }
    let mut total = 0usize;
    for raw in [&c.stdout_b64, &c.stderr_b64] {
        if raw.len() > ((super::ac::MAX_STREAM_BYTES + 2) / 3) * 4 {
            return Some("encoded console stream exceeds its cap".into());
        }
        let bytes = match super::ac::b64d_strict(raw) {
            Ok(b) => b,
            Err(e) => return Some(format!("invalid console base64: {e}")),
        };
        if base64::engine::general_purpose::STANDARD.encode(&bytes) != *raw {
            return Some("non-canonical console base64".into());
        }
        total = total.saturating_add(bytes.len());
        if bytes.len() > super::ac::MAX_STREAM_BYTES || total > super::ac::MAX_STREAM_BYTES {
            return Some("decoded console output exceeds the combined 1 MiB cap".into());
        }
    }
    if step.typed.is_some()
        || step.owned.is_some()
        || step.collections.is_some()
        || step.closures.is_some()
        || step.dynamic.is_some()
        || step.closures_source.is_some()
    {
        return Some("foreign result face on a console recipe".into());
    }
    match step.outcome.as_str() {
        "success"
            if matches!(
                (step.channel.as_deref(), step.value),
                (Some("value"), Some(_)) | (Some("unit"), None)
            ) && step.trap.is_none()
                && step.diag.is_none() =>
        {
            None
        }
        "trap"
            if step.trap.as_deref().is_some_and(|t| !t.is_empty())
                && step.value.is_none()
                && step.channel.is_none()
                && step.diag.is_none() =>
        {
            None
        }
        "diag"
            if step.value.is_none()
                && step.channel.is_none()
                && step.trap.is_none()
                && step.diag.as_ref().is_some_and(|d| {
                    d["stage"].is_string()
                        && d["code"].as_str().is_some_and(|c| !c.is_empty())
                        && d["message"].is_string()
                }) =>
        {
            None
        }
        "resource" | "cancelled" | "infra" => Some(format!(
            "console {} outcome: {}",
            step.outcome,
            step.diag
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        )),
        _ => Some("malformed console outcome/result channel".into()),
    }
}

#[cfg(all(test, feature = "test-ace"))]
mod console_863_tests {
    use super::*;

    fn observation() -> serde_json::Value {
        serde_json::json!({"op":"run","outcome":"success","channel":"value","value":7,
            "console":{"schema":"av.console-capture/1","mir_lowering":EXPECTED_MIR_LOWERING_VERSION,
            "stdout_b64":"YQBiCg==","stderr_b64":"","stdout_present":true,"stderr_present":true,
            "stdout_truncated":false,"stderr_truncated":false}})
    }
    fn admit(v: serde_json::Value) -> Option<String> {
        console_admission(RecipeFamily::Console, &[serde_json::from_value(v).unwrap()])
    }
    #[test]
    fn console_863_admission_rejects_missing_truncated_malformed_and_foreign_faces() {
        assert!(admit(observation()).is_none());
        let mut missing = observation();
        missing.as_object_mut().unwrap().remove("console");
        assert!(admit(missing).is_some());
        for field in ["stdout_present", "stderr_present"] {
            let mut v = observation();
            v["console"][field] = false.into();
            assert!(admit(v).is_some(), "{field}");
        }
        for field in ["stdout_truncated", "stderr_truncated"] {
            let mut v = observation();
            v["console"][field] = true.into();
            assert!(admit(v).is_some(), "{field}");
        }
        for raw in ["!", "YQ", "YR==", "YQ==\n"] {
            let mut v = observation();
            v["console"]["stdout_b64"] = raw.into();
            assert!(admit(v).is_some(), "{raw:?}");
        }
        let mut v = observation();
        v["console"]["mir_lowering"] = "2".into();
        assert!(admit(v).is_some());
        let mut v = observation();
        v["owned"] = serde_json::json!({});
        assert!(admit(v).is_some());
        let step = serde_json::from_value(observation()).unwrap();
        assert!(console_admission(RecipeFamily::Core, &[step]).is_some());
        for field in [
            "stdout_b64",
            "stderr_b64",
            "stdout_present",
            "stderr_truncated",
        ] {
            let mut v = observation();
            v["console"].as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<StepOut>(v).is_err(), "{field}");
        }
    }
    #[test]
    fn console_863_admission_caps_combined_bytes_and_keeps_resources_errors() {
        let mut v = observation();
        let encoded = base64::engine::general_purpose::STANDARD.encode(vec![0u8; (1 << 19) + 1]);
        v["console"]["stdout_b64"] = encoded.clone().into();
        v["console"]["stderr_b64"] = encoded.into();
        assert!(admit(v).unwrap().contains("combined"));
        for outcome in ["resource", "cancelled", "infra"] {
            let mut v = observation();
            v["outcome"] = outcome.into();
            v["diag"] = serde_json::json!({"stage":"run","code":"resource.console-output","message":"limit"});
            assert!(admit(v).unwrap().contains(outcome));
        }
        let mut v = observation();
        v["channel"] = "unit".into();
        assert!(admit(v.clone()).is_some());
        v["value"] = serde_json::Value::Null;
        assert!(admit(v).is_none());
        let mut v = observation();
        v["outcome"] = "trap".into();
        v["trap"] = "av.trap.div-by-zero".into();
        assert!(admit(v.clone()).is_some());
        v["value"] = serde_json::Value::Null;
        v["channel"] = serde_json::Value::Null;
        assert!(admit(v).is_none());
    }
}
