//! PLAN-798 T-07: the `ac-av/core-run-v1` probe recipe — formal AutoDown
//! execution of the av probe through the unified probe/run seam landed by
//! PLAN-796 (which itself rides the PLAN-788 TestCase v2 runner).
//!
//! Same contract shape as `sema.rs` (the seam's first recipe): the case
//! declares a recipe + one declarative `run` step + an `av` binding
//! (entry/caps); this adapter spawns the plan's own worker
//! (`av-worker case-serve`, envelope `avworker/1`) under Job
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
use auto_test_case::compare::{compare_bytes, Mismatch};
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

pub mod parent_observations;
pub mod resource_events;
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
/// PLAN-914 (I04/I09): the DATA-profile incremental recipe (a retained
/// DataIncrementalSession; edits report their dependency-precise rebuild set
/// and per-def resident code identities).
pub const AV_DATA_INCREMENTAL_RECIPE: &str = "ac-av/data-incremental-v1";
pub const EXPECTED_AV_DATA_CORE_VERSION: &str = "av.data/1";
/// PLAN-821: the owned-profile recipe (`av.owned/1`, profile id 3) —
/// text-record source → owned Typed MIR → AVAD v2 → AV with the arena
/// ledger face. Same append-only registration rules; the owned face is
/// judged parent-side against THIS crate's frozen expectations (result +
/// live=0 + alloc/free order), never the worker's word.
pub const AV_OWNED_RECIPE: &str = "ac-av/owned-run-v1";
pub const EXPECTED_AV_OWNED_CORE_VERSION: &str = "av.owned/1";
/// PLAN-919: the per-owner lifecycle recipe (scenario R01/R04/R06 av cells)
/// — a real source on the `owned` or `collections` production profile with
/// the bounded `av.lifecycle/1` channel on; the core version follows the
/// requested profile (owned → `av.owned/1`, collections →
/// `av.collections/1`).
pub const AV_RESOURCE_EVENTS_RECIPE: &str = "ac-av/resource-events-v1";
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
/// PLAN-912: the single-function closures source recipe — the same corpus
/// faces through the FUNCTION session (retained frontend + registry + the
/// per-unit zero-production axes the parent judges below).
pub const AV_CLOSURES_FN_RECIPE: &str = "ac-av/closures-fn-run-v1";
/// PLAN-913: the source-bound recovery recipe (publish / restore / restore-edit).
pub const AV_CLOSURES_RECOVERY_RECIPE: &str = "ac-av/closures-recovery-run-v1";
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

/// PLAN-915: the R02 allocation-fault sweep / R03 stale-Place scenario
/// recipe. Two faces selected by `av.entry`: `source:<l04|l05|v05|k02>`
/// sweeps every allocation attempt of a frozen corpus program,
/// `place:<push|insert|remove|replace>` runs the typed stale-Place
/// sequence. The parent judges the frozen attempt counts and place facts;
/// the worker never sees them.
pub const AV_RESOURCE_RECIPE: &str = "ac-av/resource-run-v1";

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
    /// PLAN-914 (I04/I09): the DATA-profile incremental recipe (a retained
    /// `DataIncrementalSession`; edits report their dependency-precise
    /// rebuild set and per-def resident code identities).
    DataIncremental,
    /// PLAN-912: the single-function closures source recipe.
    ClosuresFn,
    /// PLAN-913: the source-bound recovery recipe (publish -> fresh-process
    /// restore -> the next warm edit through the restored context).
    ClosuresRecovery,
    /// PLAN-866: the resident-online recipe — one continuously-running
    /// chain, edits at call-boundary safety points.
    Resident,
    /// PLAN-915: the R02 fault sweep / R03 stale-Place scenario recipe.
    Resource,
    Loaded,
    Closures,
    Dynamic,
    Compact,
    /// PLAN-919: the per-owner lifecycle recipe (`av.lifecycle/1`).
    ResourceEvents,
}

impl RecipeFamily {
    pub fn recipe(&self) -> &'static str {
        match self {
            RecipeFamily::Core => AV_RECIPE,
            RecipeFamily::Console => AV_CONSOLE_RECIPE,
            RecipeFamily::Data => AV_DATA_RECIPE,
            RecipeFamily::DataIncremental => AV_DATA_INCREMENTAL_RECIPE,
            RecipeFamily::Owned => AV_OWNED_RECIPE,
            RecipeFamily::Collections => AV_COLLECTIONS_RECIPE,
            RecipeFamily::CollectionsSource => AV_COLLECTIONS_SOURCE_RECIPE,
            RecipeFamily::ClosuresSource => AV_CLOSURES_SOURCE_RECIPE,
            RecipeFamily::ClosuresFn => AV_CLOSURES_FN_RECIPE,
            RecipeFamily::ClosuresRecovery => AV_CLOSURES_RECOVERY_RECIPE,
            RecipeFamily::Resident => AV_RESIDENT_RECIPE,
            RecipeFamily::Resource => AV_RESOURCE_RECIPE,
            RecipeFamily::Loaded => AV_LOADED_RECIPE,
            RecipeFamily::Closures => AV_CLOSURES_RECIPE,
            RecipeFamily::Dynamic => AV_DYNAMIC_RECIPE,
            RecipeFamily::Compact => AV_COMPACT_RECIPE,
            RecipeFamily::ResourceEvents => AV_RESOURCE_EVENTS_RECIPE,
        }
    }
    pub fn expected_core_version(&self) -> &'static str {
        match self {
            RecipeFamily::Core | RecipeFamily::Loaded => EXPECTED_AV_CORE_VERSION,
            RecipeFamily::Data => EXPECTED_AV_DATA_CORE_VERSION,
            RecipeFamily::DataIncremental => EXPECTED_AV_DATA_CORE_VERSION,
            RecipeFamily::Console => EXPECTED_AV_CONSOLE_CORE_VERSION,
            RecipeFamily::Owned => EXPECTED_AV_OWNED_CORE_VERSION,
            RecipeFamily::Collections => EXPECTED_AV_COLLECTIONS_CORE_VERSION,
            RecipeFamily::CollectionsSource => EXPECTED_AV_COLLECTIONS_CORE_VERSION,
            RecipeFamily::ClosuresSource
            | RecipeFamily::ClosuresFn
            | RecipeFamily::ClosuresRecovery => EXPECTED_AV_CLOSURES_CORE_VERSION,
            // PLAN-866: the resident recipe rides the scalar profile
            // identity (the 828 session pipeline).
            RecipeFamily::Resident => EXPECTED_AV_CORE_VERSION,
            // PLAN-915: the resource recipe admits the TWO real profiles it
            // drives (collections sources/place witnesses → av.collections/1,
            // the K02 capture source → av.closures/1); the per-entry check
            // below pins which entry may report which identity.
            RecipeFamily::Resource => EXPECTED_AV_COLLECTIONS_CORE_VERSION,
            RecipeFamily::Closures => EXPECTED_AV_CLOSURES_CORE_VERSION,
            RecipeFamily::Dynamic => EXPECTED_AV_DYNAMIC_CORE_VERSION,
            // PLAN-826: the compact wire is a WIRE generation of the same
            // scalar profile — the core identity is unchanged.
            RecipeFamily::Compact => EXPECTED_AV_CORE_VERSION,
            // PLAN-919: profile-dependent — see the special case at the
            // admission site (owned vs collections).
            RecipeFamily::ResourceEvents => EXPECTED_AV_CORE_VERSION,
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
/// PLAN-868: admission mirror of `ace_lowering::MIR_LOWERING_VERSION` —
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
    if matches!(family, RecipeFamily::ClosuresSource | RecipeFamily::ClosuresFn) {
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

#[cfg(test)]
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
/// otherwise find `experimental/ace` from this crate's source
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
                "av probe worker not located: no experimental/ace above the crate and no AC_AV_CASE_WORKER override".to_string(),
            );
        };
        let m = d.join("experimental/ace").join("Cargo.toml");
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
    cmd.args(["build", "--release", "-p", "ace-av", "--bin", "av-worker"])
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
    "av-worker.exe"
}
#[cfg(not(windows))]
fn av_worker_exe_name() -> &'static str {
    "av-worker"
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
    /// PLAN-914 (I09): the typed refusal of a controlled partial rebuild
    /// (`{kind, reason}`); present only on a refused step.
    #[serde(default)]
    refusal: Option<serde_json::Value>,
    /// PLAN-914 (I04): the dependency-precise rebuild set the incremental
    /// edit step really produced (AFTER the edit committed).
    #[serde(default)]
    rebuilt_defs: Vec<String>,
    /// PLAN-914 (I04): per-def resident code identities around the step
    /// (`[ptr, generation]` pairs) — the carry-over evidence.
    #[serde(default)]
    identities_before: std::collections::BTreeMap<String, (u64, u64)>,
    #[serde(default)]
    identities_after: std::collections::BTreeMap<String, (u64, u64)>,
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
    /// PLAN-913: the source-bound recovery face (phase publish|restore|edit,
    /// manifest/digest binding, P/R axes, warm-edit identity counters).
    #[serde(default)]
    recovery: Option<serde_json::Value>,
    #[serde(default)]
    dynamic: Option<serde_json::Value>,
    /// PLAN-862: the AV result channel of a run step — `"value"` for an
    /// `i32` payload, `"unit"` for the Unit payload (which has no i32 form).
    /// Absent on older responses and on trap/diag steps.
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    console: Option<ConsoleOut>,
    /// PLAN-915: the R02 sweep face of a resource-recipe run step
    /// ({source_value, arena_attempts, value_attempts, arena_points,
    /// value_points, arena_control, value_control, after}). A
    /// resource-recipe increment only.
    #[serde(default)]
    resource: Option<serde_json::Value>,
    /// PLAN-915: the R03 stale-Place face ({mutation, stale, fresh, legal}).
    /// A resource-recipe increment only.
    #[serde(default)]
    place: Option<serde_json::Value>,
    /// PLAN-915: the R02 capture-clone control face ({clean, points,
    /// control}) at the typed prepare boundary. A resource-recipe increment
    /// only.
    #[serde(default)]
    capture: Option<serde_json::Value>,
    /// PLAN-919: the resource face of a lifecycle run step
    /// (`av.resource.instr-steps` & co); a resource-events increment only.
    #[serde(default)]
    exit_kind: Option<String>,
    /// PLAN-919: the `av.lifecycle/1` facts object; a resource-events
    /// increment only — its presence on any other recipe is an admission
    /// violation, and its absence on a non-diag resource-events step too.
    #[serde(default)]
    lifecycle: Option<serde_json::Value>,
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

/// PLAN-915 (V-05): the worker observation line must not carry DUPLICATE
/// object keys — `serde_json` silently keeps the last one, so a relay could
/// smuggle a second `resource`/`value` past the typed mirror. This bounded
/// scanner validates the frame BEFORE parsing: one pass, depth <= 32,
/// <= 4096 keys per object, decoded keys compared (so `"\u0061"` and `"a"`
/// collide), and every object's key set must be duplicate-free.
fn reject_duplicate_keys(line: &str) -> Result<(), String> {
    struct P<'a> {
        b: &'a [u8],
        i: usize,
        depth: usize,
        keys: usize,
    }
    impl P<'_> {
        fn ws(&mut self) {
            while let Some(&c) = self.b.get(self.i) {
                if matches!(c, b' ' | b'\t' | b'\r' | b'\n') {
                    self.i += 1;
                } else {
                    break;
                }
            }
        }
        fn lit(&mut self, want: &[u8]) -> Result<(), String> {
            if self.b.get(self.i..self.i + want.len()) == Some(want) {
                self.i += want.len();
                Ok(())
            } else {
                Err("bad literal".into())
            }
        }
        fn number(&mut self) -> Result<(), String> {
            let start = self.i;
            while let Some(&c) = self.b.get(self.i) {
                if c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E') {
                    self.i += 1;
                } else {
                    break;
                }
            }
            if self.i == start {
                Err("empty number".into())
            } else {
                Ok(())
            }
        }
        fn string(&mut self) -> Result<String, String> {
            if self.b.get(self.i) != Some(&b'"') {
                return Err("expected a string".into());
            }
            self.i += 1;
            let mut out = String::new();
            loop {
                let c = *self.b.get(self.i).ok_or("unterminated string")?;
                match c {
                    b'"' => {
                        self.i += 1;
                        return Ok(out);
                    }
                    b'\\' => {
                        self.i += 1;
                        let e = *self.b.get(self.i).ok_or("escape eof")?;
                        match e {
                            b'"' => out.push('"'),
                            b'\\' => out.push('\\'),
                            b'/' => out.push('/'),
                            b'b' => out.push('\u{8}'),
                            b'f' => out.push('\u{c}'),
                            b'n' => out.push('\n'),
                            b'r' => out.push('\r'),
                            b't' => out.push('\t'),
                            b'u' => {
                                let hex = self
                                    .b
                                    .get(self.i + 1..self.i + 5)
                                    .ok_or("unicode escape eof")?;
                                let text =
                                    std::str::from_utf8(hex).map_err(|_| "unicode escape")?;
                                let cp =
                                    u32::from_str_radix(text, 16).map_err(|_| "unicode escape")?;
                                out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                                self.i += 4;
                            }
                            _ => return Err("bad escape".into()),
                        }
                        self.i += 1;
                    }
                    _ => {
                        let start = self.i;
                        let len = if c < 0x80 {
                            1
                        } else if c >> 5 == 0b110 {
                            2
                        } else if c >> 4 == 0b1110 {
                            3
                        } else {
                            4
                        };
                        self.i += len;
                        let piece = self
                            .b
                            .get(start..self.i)
                            .ok_or_else(|| "truncated utf-8".to_string())?;
                        out.push_str(std::str::from_utf8(piece).map_err(|_| "bad utf-8")?);
                    }
                }
            }
        }
        fn value(&mut self) -> Result<(), String> {
            self.ws();
            if self.depth > 32 {
                return Err("nesting too deep".into());
            }
            match self.b.get(self.i) {
                Some(b'{') => self.object(),
                Some(b'[') => self.array(),
                Some(b'"') => {
                    self.string()?;
                    Ok(())
                }
                Some(b't') => self.lit(b"true"),
                Some(b'f') => self.lit(b"false"),
                Some(b'n') => self.lit(b"null"),
                Some(c) if c.is_ascii_digit() || *c == b'-' => self.number(),
                _ => Err("bad value".into()),
            }
        }
        fn object(&mut self) -> Result<(), String> {
            self.i += 1;
            self.depth += 1;
            let mut seen = std::collections::BTreeSet::new();
            self.ws();
            if self.b.get(self.i) == Some(&b'}') {
                self.i += 1;
                self.depth -= 1;
                return Ok(());
            }
            loop {
                self.ws();
                let key = self.string()?;
                self.keys += 1;
                if self.keys > 4096 {
                    return Err("too many object keys".into());
                }
                if !seen.insert(key) {
                    return Err("duplicate object key".into());
                }
                self.ws();
                if self.b.get(self.i) != Some(&b':') {
                    return Err("missing colon".into());
                }
                self.i += 1;
                self.value()?;
                self.ws();
                match self.b.get(self.i) {
                    Some(b',') => self.i += 1,
                    Some(b'}') => {
                        self.i += 1;
                        self.depth -= 1;
                        return Ok(());
                    }
                    _ => return Err("bad object separator".into()),
                }
            }
        }
        fn array(&mut self) -> Result<(), String> {
            self.i += 1;
            self.depth += 1;
            self.ws();
            if self.b.get(self.i) == Some(&b']') {
                self.i += 1;
                self.depth -= 1;
                return Ok(());
            }
            loop {
                self.value()?;
                self.ws();
                match self.b.get(self.i) {
                    Some(b',') => self.i += 1,
                    Some(b']') => {
                        self.i += 1;
                        self.depth -= 1;
                        return Ok(());
                    }
                    _ => return Err("bad array separator".into()),
                }
            }
        }
    }
    let mut p = P {
        b: line.as_bytes(),
        i: 0,
        depth: 0,
        keys: 0,
    };
    p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err("trailing content after the JSON frame".into());
    }
    Ok(())
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
    reject_duplicate_keys(non_empty[0]).map_err(|e| {
        format!(
            "bad av worker response ({e}): {:.200} | stderr: {:.300}",
            non_empty[0],
            String::from_utf8_lossy(&err_buf)
        )
    })?;
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
        "typecheck" => Stage::Typecheck,
        // PLAN-909: the HIR verifier's own rejections (verify.*) keep their
        // verify stage — the completion vocabulary distinguishes them from
        // typecheck-face rejections.
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

/// PLAN-915 R02 (capture clone): (id, first clone attempt, last clone
/// attempt, control attempt, expected live after the plan drops). The two
/// source buffers are attempts 1/2, so the clone attempts start at 3.
const CAPTURE_CASES: &[(&str, u64, u64, u64, usize)] = &[("clone", 3, 4, 5, 2)];

/// PLAN-915 R02: the PARENT's frozen per-corpus expectation table —
/// (entry id, frozen i32 result, owned-arena allocation attempts, value
/// allocation attempts). Hand-audited from the frozen corpus programs at
/// T-02 and then re-verified by the crate-side controls; the judge compares
/// against THIS, never against anything the worker reports.
const RESOURCE_SWEEP_CASES: &[(&str, i32, u64, u64)] = &[
    // L04: owned-record field view — "hello" buffer + list buffer.
    ("l04", 42, 2, 6),
    // L05: remove transfers the owner out — same two buffers.
    ("l05", 42, 2, 5),
    // V05: owned variant payload — one arena attempt (the payload buffer).
    ("v05", 42, 1, 6),
    // LSTR: List<Str> push/remove — three owned buffers, five value writes.
    ("lstr", 51, 3, 5),
    // K02: creator-exit owned closure — one arena attempt (the captured Str).
    ("k02", 42, 1, 5),
];

/// PLAN-915 R03: the PARENT's frozen stale-Place sequence table —
/// (mutation, trap code, fresh-token value, legal-face value). The legal
/// face is the no-mutation control (mint → read = 10); the fresh face is a
/// token minted AFTER the mutation reading the current element; the stale
/// face is the pre-mutation token used after it (must trap).
const PLACE_SEQUENCES: &[(&str, &str, i32, i32)] = &[
    ("push", "av.trap.stale-place", 30, 10),
    ("insert", "av.trap.stale-place", 30, 10),
    ("remove", "av.trap.stale-place", 20, 10),
    ("replace", "av.trap.stale-place", 99, 10),
];

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
    // PLAN-909: K06's original move-closure source now EXECUTES to 42
    // (private writable env); the unsupported non-scalar shared-write
    // rejection moved to the shared-mutable-909 corpus (S07/S08).
    ("K06", Some(42), None, 2),
    // PLAN-909 shared mutable captures — the dedicated corpus (fresh value,
    // no diagnostic, fresh submitted-unit count).
    ("S01", Some(43), None, 2),
    ("S02", Some(42), None, 3),
    ("S03", Some(42), None, 3),
    ("S04", Some(44), None, 2),
    ("S05", Some(82), None, 3),
    ("S06", Some(84), None, 3),
    ("S07", None, Some("source.closure-shared-mutable"), 0),
    ("S08", None, Some("verify.immutable-write"), 0),
];

/// PLAN-909: the expected value of each source case's `body-edit` mode
/// (the edit pair lives in the case file; the parent holds the oracle).
fn closures_source_body_edit_value(id: &str) -> i32 {
    match id {
        "K01" => 43,
        "S01" => 46,
        "S02" => 44,
        "S03" => 45,
        // Fail closed: an id without a recorded body-edit oracle is a
        // parent-table defect, never K01's value in disguise.
        other => unreachable!("body-edit oracle missing for source `{other}`"),
    }
}

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

/// PLAN-912: the single-function closures source check (same corpus, the
/// FUNCTION session's observation axes).
pub fn run_closures_fn_check(
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
        RecipeFamily::ClosuresFn,
    )
}

/// PLAN-913: the source-bound recovery check (publish -> fresh-process
/// restore -> the next warm edit; the parent judges the P/R axes and the
/// restored-context identity).
pub fn run_closures_recovery_check(
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
        RecipeFamily::ClosuresRecovery,
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

/// PLAN-914 (I04/I09): the data-incremental recipe rides the data family's
/// value/presence faces; the invalidation facts are judged separately.
pub fn run_data_incremental_check(
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
        RecipeFamily::DataIncremental,
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

/// PLAN-915: the R02/R03 scenario recipe entry.
pub fn run_resource_check(
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
        RecipeFamily::Resource,
    )
}

/// PLAN-919: the per-owner lifecycle recipe's parent entry.
#[allow(clippy::too_many_arguments)]
pub fn run_resource_events_check(
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
        RecipeFamily::ResourceEvents,
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
    // PLAN-919: the lifecycle recipe's core version follows the requested
    // source profile (both are admitted production identities).
    let expected_core_version = if family == RecipeFamily::ResourceEvents {
        match av.resource.as_ref().map(|r| r.profile.as_str()) {
            Some("owned") => EXPECTED_AV_OWNED_CORE_VERSION,
            Some("collections") => EXPECTED_AV_COLLECTIONS_CORE_VERSION,
            other => {
                return Err(format!(
                    "resource-events requires av.resource profile owned|collections (got {other:?})"
                ))
            }
        }
    } else if entry_unit_profile {
        EXPECTED_AV_ENTRY_UNIT_CORE_VERSION
    } else {
        family.expected_core_version()
    };
    // PLAN-915: the resource recipe drives two REAL profiles; the entry
    // pins which identity it may report, and the receipt's toolchain axis
    // records that same identity instead of the family default.
    let resource_core_axis: Option<&str> = if family == RecipeFamily::Resource {
        Some(
            if av.entry == "source:k02" || av.entry.starts_with("capture:") {
                EXPECTED_AV_CLOSURES_CORE_VERSION
            } else {
                EXPECTED_AV_COLLECTIONS_CORE_VERSION
            },
        )
    } else {
        None
    };
    let receipt_core_override =
        resource_core_axis.or(entry_unit_profile.then_some(EXPECTED_AV_ENTRY_UNIT_CORE_VERSION));
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
            if matches!(family, RecipeFamily::ClosuresSource | RecipeFamily::ClosuresFn)
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
            // PLAN-914 (I09): the controlled partial rebuild carries its own
            // rebuild set (never silently downgraded to a run).
            auto_test_case::contract::ProbeStep::PartialRebuild { from, to, defs } => {
                serde_json::json!({ "op": "partial-rebuild", "from": from, "to": to, "defs": defs })
            }
            // PLAN-914 (I04): declaration rounds are not an AV step shape.
            auto_test_case::contract::ProbeStep::DeclarationInvalidate { from, to } => {
                serde_json::json!({ "op": "declaration-invalidate", "from": from, "to": to })
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
    let is_closures_source_family = matches!(
        family,
        RecipeFamily::ClosuresSource
            | RecipeFamily::ClosuresFn
            | RecipeFamily::ClosuresRecovery
    );
    let closures_source_id = if is_closures_source_family {
        let selector = av.entry.strip_prefix("source:").ok_or_else(|| {
            "closures-source requires a registered source:<K-id> av.entry".to_string()
        })?;
        let mut parts = selector.split(':');
        let id = parts.next().unwrap_or_default();
        let mode = parts.next();
        // PLAN-909: body-edit stays K01-frozen in the K corpus; the
        // shared-mutable-909 (S*) and PLAN-912 (F*) sources may edit bodies.
        // PLAN-912 adds the `hit` mode (an identical re-apply is judged as a
        // zero-production hit).
        let fn_family = family == RecipeFamily::ClosuresFn;
        let recovery_family = family == RecipeFamily::ClosuresRecovery;
        let modes: &[&str] = if recovery_family {
            &["publish", "restore", "restore-edit"]
        } else if fn_family {
            &["fresh", "restart", "body-edit", "hit"]
        } else {
            &["fresh", "restart", "body-edit"]
        };
        if parts.next().is_some()
            || mode.is_some_and(|mode| !modes.contains(&mode))
            || (mode == Some("body-edit")
                && id != "K01"
                && !id.starts_with('S')
                && !id.starts_with('F'))
        {
            return Err(format!("invalid closures-source selector `{selector}`"));
        }
        let registered = if recovery_family {
            closures_recovery_case(id).is_some()
        } else if fn_family {
            closures_fn_case(id).is_some()
        } else {
            CLOSURES_SOURCE_CASES.iter().any(|(case_id, ..)| *case_id == id)
        };
        if !registered {
            return Err(format!(
                "closures source `{id}` is not registered (parent table)"
            ));
        }
        Some((id.to_string(), mode.map(str::to_string)))
    } else {
        None
    };
    // PLAN-915: the resource recipe's entry selector is validated against
    // the parent's OWN frozen tables before any request — an unregistered
    // source id or place mutation is a check error, never a green run.
    if family == RecipeFamily::Resource {
        match av.entry.strip_prefix("source:") {
            Some(id) if RESOURCE_SWEEP_CASES.iter().any(|(fid, ..)| *fid == id) => {}
            Some(id) => {
                return Err(format!(
                    "av resource source `{id}` is not registered (parent table)"
                ))
            }
            None => match av.entry.strip_prefix("place:") {
                Some(name) if PLACE_SEQUENCES.iter().any(|(fid, ..)| *fid == name) => {}
                Some(name) => {
                    return Err(format!(
                        "av resource place mutation `{name}` is not registered (parent table)"
                    ))
                }
                None => match av.entry.strip_prefix("capture:") {
                    Some(id) if CAPTURE_CASES.iter().any(|(fid, ..)| *fid == id) => {}
                    Some(id) => {
                        return Err(format!(
                            "av resource capture `{id}` is not registered (parent table)"
                        ))
                    }
                    None => {
                        return Err(
                            "the resource recipe requires source:<id>, place:<mutation> or capture:<id>"
                                .into(),
                        )
                    }
                },
            },
        }
    }
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
    // PLAN-919: the lifecycle recipe's profile + exit knobs (operations
    // and budgets only — expectations never reach the worker).
    if family == RecipeFamily::ResourceEvents {
        let res = av
            .resource
            .as_ref()
            .ok_or_else(|| "resource-events requires av.resource".to_string())?;
        if av.profile.is_some() {
            return Err("resource-events does not take av.profile (use av.resource.profile)".into());
        }
        request["profile"] = serde_json::json!(res.profile);
        let mut limits = serde_json::Map::new();
        if let Some(n) = res.max_steps {
            limits.insert("max_steps".into(), serde_json::json!(n));
        }
        if let Some(n) = res.wall_ms {
            limits.insert("wall_ms".into(), serde_json::json!(n));
        }
        request["limits"] = serde_json::Value::Object(limits);
        if let Some(n) = res.cancel_after_ms {
            request["cancel_after_ms"] = serde_json::json!(n);
        }
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
    parent_clock.lap("prepare_ms");
    let resp_result = transport(&request_text, wall);
    parent_clock.lap("transport_ms");
    let worker_us = worker_started.elapsed().as_micros() as u64;

    let mut mismatches: Vec<Mismatch> = Vec::new();
    // PLAN-914 (I04): the admitted step records (the invalidation facts live
    // on the EDIT steps, not on the judged run).
    let mut admitted_steps: Vec<StepOut> = Vec::new();
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
        if lifecycle_on_non_resource(family, &ok.steps) {
            infra_detail = Some("lifecycle facts on a non-resource-events recipe".to_string());
            break 'admit;
        }
        if family == RecipeFamily::ResourceEvents {
            if let Some(v) = malformed_resource_events(&ok.steps) {
                infra_detail = Some(v);
                break 'admit;
            }
        }
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
        if resource_on_non_resource(family, &ok.steps) {
            infra_detail = Some(
                "resource/place face on a non-resource recipe (protocol violation)".to_string(),
            );
            break 'admit;
        }
        if let Some(v) = malformed_resource(family, &ok.steps) {
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
        if let Some(want_core) = resource_core_axis {
            // PLAN-915: the resource recipe attaches the real profile of the
            // entry it drove; the parent pins WHICH entry may report which
            // identity (the K02 capture source is the closures profile, all
            // collections sources and place witnesses the collections one).
            if ok.av_core_version != want_core {
                infra_detail = Some(format!(
                    "av core version `{}` does not match the entry's admitted `{}`",
                    ok.av_core_version, want_core
                ));
                break 'admit;
            }
        } else if ok.av_core_version != expected_core_version {
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
        admitted_steps = ok.steps.clone();
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
                auto_test_case::contract::ProbeStep::PartialRebuild { from, to, defs } => {
                    // PLAN-914 (I09): a CONTROLLED partial rebuild MUST be
                    // refused by the link/env-contract gate; the accepted
                    // snapshot stays the session's state (the parent's
                    // current source is NOT advanced).
                    let hits = current_source.matches(from.as_str()).count();
                    if hits != 1 {
                        infra_detail = Some(format!(
                            "step {i}: partial-rebuild literal matched {hits} times"
                        ));
                        break 'admit;
                    }
                    let _ = to; // the candidate is the refused one; never installed
                    if got.outcome != "refused" {
                        mismatches.push(Mismatch {
                            observation: "partial-rebuild".into(),
                            kind: "refusal".into(),
                            expected: "the link/env-contract gate REFUSES the incompatible mix"
                                .into(),
                            actual: format!("outcome `{}`", got.outcome),
                            first_diff: None,
                        });
                    }
                    let kind = got
                        .refusal
                        .as_ref()
                        .and_then(|r| r.get("kind"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if kind != "env-contract" && kind != "link-rejected" {
                        mismatches.push(Mismatch {
                            observation: "partial-rebuild".into(),
                            kind: "refusal-kind".into(),
                            expected: "a typed link refusal (env-contract or link-rejected)".into(),
                            actual: if kind.is_empty() {
                                "no typed refusal".to_string()
                            } else {
                                kind.to_string()
                            },
                            first_diff: None,
                        });
                    }
                    let _ = defs;
                    "partial-rebuild"
                }
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
    // PLAN-914 (I04, D-05 criterion): the data-incremental invalidation
    // oracle — the dependency-precise rebuild set and the resident code
    // identities of the EDIT steps, judged parent-side (the judged run only
    // carries the outcome).
    if check.options.probe_recipe.as_deref() == Some(AV_DATA_INCREMENTAL_RECIPE) {
        match av.invalidation.as_ref() {
            Some(spec) => judge_invalidation(&admitted_steps, spec, &mut mismatches),
            // A refusal-only check (I09) carries no invalidation oracle: the
            // contract already rejects an oracle without an edit step.
            None => {}
        }
    }
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
        if family == RecipeFamily::ClosuresRecovery {
            judge_closures_recovery(&id.0, id.1.as_deref(), &step, &input_bytes, &mut mismatches);
        } else if family == RecipeFamily::ClosuresFn {
            judge_closures_fn(&id.0, id.1.as_deref(), &step, &mut mismatches);
        } else {
            judge_closures_source(&id.0, id.1.as_deref(), &step, &mut mismatches);
        }
        judge_expectation(family, &step, oracle, &mut mismatches);
    } else if let Some(id) = &dynamic_fixture {
        judge_dynamic_fixture(id, &step, &mut mismatches);
    } else if family == RecipeFamily::Resource {
        // PLAN-915: the R02 sweep / R03 place judgement against the
        // parent's frozen tables (the worker's word is never the oracle),
        // plus the case's own declared completion face.
        judge_resource(&av.entry, &step, &mut mismatches);
        judge_expectation(family, &step, oracle, &mut mismatches);
    } else if family == RecipeFamily::ResourceEvents {
        // PLAN-919: the frozen per-case table lookup + generic invariants
        // (admission already ran `admit_lifecycle` — a red there is infra).
        let oracle_key = format!("{}/{}", case.spec.id, check.id);
        match resource_events::oracle_for(&oracle_key) {
            Some(o) => {
                resource_events::judge_resource_events(o, &step, &mut mismatches);
                judge_expectation(family, &step, oracle, &mut mismatches);
            }
            None => {
                mismatches.push(Mismatch {
                    observation: "lifecycle".into(),
                    kind: "case-registration".into(),
                    expected: "a frozen resource-events oracle row".into(),
                    actual: format!("check {oracle_key} is not in the parent table"),
                    first_diff: None,
                });
            }
        }
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
        "recovery": step.recovery,
            "counters": step.counters,
            "source_hash": step.source_hash,
            "wall_ms": step.wall_ms,
            "spans": step.spans,
            "trace": step.trace,
        },
        "worker_us": worker_us,
        "request_wall_ms": request_wall_ms,
        "toolchain": probe_toolchain_axes(&exe_sha, family, receipt_core_override),
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
/// PLAN-891: parent-side canonical-shape check shared by the native typed
/// judgement (same rules, single source of truth).
pub(crate) fn typed_expect_shape_violation(kind: &str, bits: &str) -> Option<String> {
    typed_shape_violation(&TypedOut {
        kind: kind.to_string(),
        bits: bits.to_string(),
    })
}

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

/// PLAN-919: the lifecycle face belongs ONLY to the resource-events recipe.
fn lifecycle_on_non_resource(family: RecipeFamily, steps: &[StepOut]) -> bool {
    family != RecipeFamily::ResourceEvents && steps.iter().any(|s| s.lifecycle.is_some())
}

/// PLAN-919: a resource-events RUN step must carry the lifecycle facts
/// (except frontend/verify diag outcomes, which never executed).
fn malformed_resource_events(steps: &[StepOut]) -> Option<String> {
    steps.iter().find_map(|step| {
        if step.op != "run" || step.outcome == "diag" {
            return None;
        }
        match &step.lifecycle {
            None => Some("resource-events run step carries no lifecycle facts".into()),
            Some(v) => super::ac_av::resource_events::admit_lifecycle(v).err(),
        }
    })
}

/// PLAN-865: the source-session face belongs only to the source closures
/// recipe; fixture closures keep their original ledger contract.
fn closures_source_on_non_recipe(family: RecipeFamily, steps: &[StepOut]) -> bool {
    !matches!(family, RecipeFamily::ClosuresSource | RecipeFamily::ClosuresFn)
        && steps.iter().any(|s| s.closures_source.is_some())
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

/// Admission rule (PLAN-915): the `resource`/`place` faces belong to the
/// resource recipe only — their presence on any other recipe is a protocol
/// violation, never a pass-shaped extra channel.
fn resource_on_non_resource(family: RecipeFamily, steps: &[StepOut]) -> bool {
    family != RecipeFamily::Resource
        && steps
            .iter()
            .any(|s| s.resource.is_some() || s.place.is_some() || s.capture.is_some())
}

/// Admission rule (PLAN-915): every resource/place face is SHAPE-validated
/// here, before and independent of any oracle. Wrong types, unknown keys,
/// point counts that disagree with the reported attempt totals, and
/// half-present control faces are all explicit rejections.
fn malformed_resource(family: RecipeFamily, steps: &[StepOut]) -> Option<String> {
    if family != RecipeFamily::Resource {
        return None;
    }
    let step = steps.iter().rev().find(|s| s.op == "run")?;
    let bad = |why: String| Some(format!("malformed resource report: {why}"));
    let faces = [
        step.resource.is_some(),
        step.place.is_some(),
        step.capture.is_some(),
    ];
    if faces.iter().filter(|present| **present).count() != 1 {
        return bad("the run step must carry exactly one resource/place/capture face".into());
    }
    if let Some(face) = &step.resource {
        let Some(obj) = face.as_object() else {
            return bad("resource face is not an object".into());
        };
        let known = [
            "source_value",
            "arena_attempts",
            "arena_successful",
            "arena_frees",
            "value_attempts",
            "arena_points",
            "value_points",
            "arena_control",
            "value_control",
            "after",
            "after_recompile",
        ];
        if let Some(key) = obj.keys().find(|k| !known.contains(&k.as_str())) {
            return bad(format!("resource face carries unknown key `{key}`"));
        }
        if obj.get("source_value").and_then(|v| v.as_i64()).is_none() {
            return bad("source_value is not an integer".into());
        }
        for field in ["arena_attempts", "value_attempts"] {
            if obj.get(field).and_then(|v| v.as_u64()).is_none() {
                return bad(format!("{field} is not an unsigned integer"));
            }
        }
        for (points_field, attempts_field) in [
            ("arena_points", "arena_attempts"),
            ("value_points", "value_attempts"),
        ] {
            let Some(points) = obj.get(points_field).and_then(|v| v.as_array()) else {
                return bad(format!("{points_field} is not an array"));
            };
            let attempts = obj
                .get(attempts_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if points.len() as u64 != attempts {
                return bad(format!(
                    "{points_field} has {} points for {attempts} attempts",
                    points.len()
                ));
            }
            for (i, point) in points.iter().enumerate() {
                if point.get("attempt").and_then(|v| v.as_u64()) != Some(i as u64 + 1) {
                    return bad(format!("{points_field}[{i}] attempt ordinal mismatch"));
                }
                if !point.get("faulted").and_then(|v| v.as_bool()).is_some() {
                    return bad(format!("{points_field}[{i}] faulted is not a bool"));
                }
                if point.get("live").and_then(|v| v.as_u64()).is_none() {
                    return bad(format!(
                        "{points_field}[{i}] live is not an unsigned integer"
                    ));
                }
            }
        }
        for control in ["arena_control", "value_control"] {
            let Some(c) = obj.get(control) else {
                return bad(format!("{control} is missing"));
            };
            if c.get("attempt").and_then(|v| v.as_u64()).is_none() {
                return bad(format!("{control}.attempt is not an unsigned integer"));
            }
            if c.get("value").and_then(|v| v.as_i64()).is_none() {
                return bad(format!("{control}.value is not an integer"));
            }
            if c.get("live").and_then(|v| v.as_u64()).is_none() {
                return bad(format!("{control}.live is not an unsigned integer"));
            }
        }
        for field in ["after", "after_recompile"] {
            let Some(after) = obj.get(field) else {
                return bad(format!("{field} face is missing"));
            };
            if after.get("value").and_then(|v| v.as_i64()).is_none()
                || after.get("live").and_then(|v| v.as_u64()).is_none()
            {
                return bad(format!("{field}.value/live are not integers"));
            }
        }
    }
    if let Some(face) = &step.place {
        let Some(obj) = face.as_object() else {
            return bad("place face is not an object".into());
        };
        let face_keys = ["mutation", "stale", "fresh", "legal"];
        if let Some(key) = obj.keys().find(|k| !face_keys.contains(&k.as_str())) {
            return bad(format!("place face carries unknown key `{key}`"));
        }
        for key in face_keys {
            if obj.get(key).is_none() {
                return bad(format!("place face is missing `{key}`"));
            }
        }
        if obj
            .get("mutation")
            .and_then(|v| v.as_str())
            .map(str::is_empty)
            .unwrap_or(true)
        {
            return bad("place mutation is not a non-empty string".into());
        }
        for face_name in ["stale", "fresh", "legal"] {
            let Some(f) = obj.get(face_name) else {
                continue;
            };
            if f.get("outcome").and_then(|v| v.as_str()).is_none() {
                return bad(format!("place.{face_name}.outcome is not a string"));
            }
            if f.get("live").and_then(|v| v.as_u64()).is_none() {
                return bad(format!("place.{face_name}.live is not an unsigned integer"));
            }
            let sub_keys = ["outcome", "value", "trap", "facts", "live"];
            if let Some(inner) = f.as_object() {
                if let Some(key) = inner.keys().find(|k| !sub_keys.contains(&k.as_str())) {
                    return bad(format!("place.{face_name} carries unknown key `{key}`"));
                }
            }
            let Some(facts) = f.get("facts").and_then(|v| v.as_object()) else {
                return bad(format!("place.{face_name}.facts is not an object"));
            };
            let fact_keys = [
                "mints",
                "get_ok",
                "get_stale",
                "set_ok",
                "set_stale",
                "buffer_reads",
            ];
            if let Some(key) = facts.keys().find(|k| !fact_keys.contains(&k.as_str())) {
                return bad(format!(
                    "place.{face_name}.facts carries unknown key `{key}`"
                ));
            }
            for key in fact_keys {
                if facts.get(key).and_then(|v| v.as_u64()).is_none() {
                    return bad(format!("place.{face_name}.facts.{key} is not unsigned"));
                }
            }
        }
    }
    if let Some(face) = &step.capture {
        let Some(obj) = face.as_object() else {
            return bad("capture face is not an object".into());
        };
        let capture_keys = ["clean", "points", "control"];
        if let Some(key) = obj.keys().find(|k| !capture_keys.contains(&k.as_str())) {
            return bad(format!("capture face carries unknown key `{key}`"));
        }
        for key in capture_keys {
            if obj.get(key).is_none() {
                return bad(format!("capture face is missing `{key}`"));
            }
        }
        let entry_keys = [
            "attempt",
            "outcome",
            "fault_attempt",
            "code",
            "live",
            "source_readable",
        ];
        let Some(points) = obj.get("points").and_then(|v| v.as_array()) else {
            return bad("capture.points is not an array".into());
        };
        for (i, point) in points.iter().enumerate() {
            if let Some(key) = point
                .as_object()
                .and_then(|o| o.keys().find(|k| !entry_keys.contains(&k.as_str())))
            {
                return bad(format!("capture.points[{i}] carries unknown key `{key}`"));
            }
            if point.get("outcome").and_then(|v| v.as_str()).is_none()
                || point
                    .get("fault_attempt")
                    .and_then(|v| v.as_u64())
                    .is_none()
                || point.get("live").and_then(|v| v.as_u64()).is_none()
                || point
                    .get("source_readable")
                    .and_then(|v| v.as_bool())
                    .is_none()
            {
                return bad(format!("capture.points[{i}] is malformed"));
            }
        }
        for face_name in ["clean", "control"] {
            let Some(f) = obj.get(face_name) else {
                continue;
            };
            if let Some(key) = f
                .as_object()
                .and_then(|o| o.keys().find(|k| !entry_keys.contains(&k.as_str())))
            {
                return bad(format!("capture.{face_name} carries unknown key `{key}`"));
            }
            if f.get("outcome").and_then(|v| v.as_str()).is_none()
                || f.get("live").and_then(|v| v.as_u64()).is_none()
                || f.get("source_readable").and_then(|v| v.as_bool()).is_none()
            {
                return bad(format!("capture.{face_name} is malformed"));
            }
        }
    }
    None
}

/// PLAN-915: judge the R02 sweep / R03 place sequence against the parent's
/// frozen tables. Every axis is checked against the table, never against the
/// worker's reported expectations.
fn judge_resource(entry: &str, step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    fn push(
        mismatches: &mut Vec<Mismatch>,
        observation: &str,
        kind: &'static str,
        expected: String,
        actual: String,
    ) {
        mismatches.push(Mismatch {
            observation: observation.to_string(),
            kind,
            expected,
            actual,
            first_diff: None,
        });
    }
    if let Some(id) = entry.strip_prefix("source:") {
        let Some((_, want_value, want_arena, want_values)) =
            RESOURCE_SWEEP_CASES.iter().find(|(fid, ..)| *fid == id)
        else {
            return;
        };
        let Some(face) = &step.resource else {
            push(
                mismatches,
                "resource",
                "missing-face",
                "the R02 sweep face".into(),
                "absent".into(),
            );
            return;
        };
        let got_value = face
            .get("source_value")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        if got_value != *want_value as i64 {
            push(
                mismatches,
                "resource",
                "source-value",
                want_value.to_string(),
                got_value.to_string(),
            );
        }
        let got_arena = face
            .get("arena_attempts")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        if got_arena != *want_arena {
            push(
                mismatches,
                "resource",
                "arena-attempts",
                want_arena.to_string(),
                got_arena.to_string(),
            );
        }
        let got_values = face
            .get("value_attempts")
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX);
        if got_values != *want_values {
            push(
                mismatches,
                "resource",
                "value-attempts",
                want_values.to_string(),
                got_values.to_string(),
            );
        }
        let points = |name: &str| {
            face.get(name)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        };
        for (field, count) in [
            ("arena_points", *want_arena),
            ("value_points", *want_values),
        ] {
            let got = points(field);
            if got.len() as u64 != count {
                push(
                    mismatches,
                    "resource",
                    field,
                    count.to_string(),
                    got.len().to_string(),
                );
                continue;
            }
            for (i, point) in got.iter().enumerate() {
                let faulted = point.get("faulted").and_then(|v| v.as_bool()) == Some(true);
                let live = point.get("live").and_then(|v| v.as_u64()).unwrap_or(1);
                let message = point
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let marker = if field == "arena_points" {
                    "fault-injected at owned allocation attempt #"
                } else {
                    "fault-injected at value allocation #"
                };
                if !faulted || live != 0 || !message.contains(marker) {
                    push(
                        mismatches,
                        "resource",
                        "point",
                        format!("#{} {marker}… with live=0", i + 1),
                        format!("{point}"),
                    );
                }
                if field == "arena_points" {
                    // The arena ledger must be internally consistent: the
                    // faulted attempt exists, no successful allocation was
                    // produced by it, and the released ids are exactly the
                    // prior attempts.
                    let ledger_attempts = point
                        .get("attempts")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(u64::MAX);
                    let successful = point
                        .get("successful")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(u64::MAX);
                    let frees = point
                        .get("frees")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(u64::MAX);
                    let want = (i + 1) as u64;
                    if ledger_attempts != want || successful != want - 1 || frees != want - 1 {
                        push(
                            mismatches,
                            "resource",
                            "arena-ledger",
                            format!("attempts={want} successful={} frees={}", want - 1, want - 1),
                            format!(
                                "attempts={ledger_attempts} successful={successful} frees={frees}"
                            ),
                        );
                    }
                }
            }
        }
        for (field, attempt) in [
            ("arena_control", *want_arena + 1),
            ("value_control", *want_values + 1),
        ] {
            let got_attempt = face
                .get(field)
                .and_then(|v| v.get("attempt"))
                .and_then(|v| v.as_u64())
                .unwrap_or(u64::MAX);
            let got_value = face
                .get(field)
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_i64())
                .unwrap_or(-1);
            let got_live = face
                .get(field)
                .and_then(|v| v.get("live"))
                .and_then(|v| v.as_u64())
                .unwrap_or(1);
            if got_attempt != attempt || got_value != *want_value as i64 || got_live != 0 {
                push(
                    mismatches,
                    "resource",
                    field,
                    format!("attempt {attempt} → {want_value} live=0"),
                    format!("attempt {got_attempt} → {got_value} live={got_live}"),
                );
            }
        }
        let after_value = face
            .get("after")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        let after_live = face
            .get("after")
            .and_then(|v| v.get("live"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        if after_value != *want_value as i64 || after_live != 0 {
            push(
                mismatches,
                "resource",
                "after",
                format!("{want_value} live=0"),
                format!("{after_value} live={after_live}"),
            );
        }
        // The recovery run must be a FULL re-verification of the same
        // source (never a cached result) and must reproduce the frozen value.
        let recompiled_value = face
            .get("after_recompile")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        let recompiled_live = face
            .get("after_recompile")
            .and_then(|v| v.get("live"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        if recompiled_value != *want_value as i64 || recompiled_live != 0 {
            push(
                mismatches,
                "resource",
                "after-recompile",
                format!("{want_value} live=0"),
                format!("{recompiled_value} live={recompiled_live}"),
            );
        }
        return;
    }
    if let Some(id) = entry.strip_prefix("capture:") {
        let Some((_, first, last, control_attempt, want_live)) =
            CAPTURE_CASES.iter().find(|(fid, ..)| *fid == id)
        else {
            return;
        };
        let Some(face) = &step.capture else {
            push(
                mismatches,
                "capture",
                "missing-face",
                "the R02 capture-clone face".into(),
                "absent".into(),
            );
            return;
        };
        let text = |face_name: &str, key: &str| {
            face.get(face_name)
                .and_then(|f| f.get(key))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };
        let number = |face_name: &str, key: &str| {
            face.get(face_name)
                .and_then(|f| f.get(key))
                .and_then(|v| v.as_u64())
                .unwrap_or(u64::MAX)
        };
        let readable = |face_name: &str| {
            face.get(face_name)
                .and_then(|f| f.get("source_readable"))
                .and_then(|v| v.as_bool())
                == Some(true)
        };
        if text("clean", "outcome") != "success"
            || number("clean", "live") != *want_live as u64
            || !readable("clean")
        {
            push(
                mismatches,
                "capture",
                "clean-control",
                format!("success live={want_live} source readable"),
                format!("{face:?}"),
            );
        }
        let points = face
            .get("points")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let want_points = last - first + 1;
        if points.len() as u64 != want_points {
            push(
                mismatches,
                "capture",
                "point-count",
                want_points.to_string(),
                points.len().to_string(),
            );
        }
        for (i, point) in points.iter().enumerate() {
            let want_attempt = first + i as u64;
            let got_attempt = point
                .get("fault_attempt")
                .and_then(|v| v.as_u64())
                .unwrap_or(u64::MAX);
            let outcome = point
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let code = point
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let live = point
                .get("live")
                .and_then(|v| v.as_u64())
                .unwrap_or(u64::MAX);
            let src_ok = point.get("source_readable").and_then(|v| v.as_bool()) == Some(true);
            if outcome != "faulted-clone"
                || got_attempt != want_attempt
                || code != "env.capture.faulted-clone"
                || live != *want_live as u64
                || !src_ok
            {
                push(
                    mismatches,
                    "capture",
                    "point",
                    format!(
                        "clone attempt {want_attempt} → env.capture.faulted-clone, live={want_live}, source readable"
                    ),
                    format!("{point}"),
                );
            }
        }
        let control_ok = text("control", "outcome") == "success"
            && number("control", "live") == *want_live as u64
            && number("control", "attempt") == *control_attempt
            && readable("control");
        if !control_ok {
            push(
                mismatches,
                "capture",
                "control",
                format!(
                    "the N+1 control (attempt {control_attempt}) clones successfully with live={want_live}"
                ),
                format!("{face:?}"),
            );
        }
        return;
    }
    let Some(name) = entry.strip_prefix("place:") else {
        return;
    };
    let Some((_, want_trap, want_fresh, want_legal)) =
        PLACE_SEQUENCES.iter().find(|(fid, ..)| *fid == name)
    else {
        return;
    };
    let Some(face) = &step.place else {
        push(
            mismatches,
            "place",
            "missing-face",
            "the R03 place sequence".into(),
            "absent".into(),
        );
        return;
    };
    let got_mutation = face.get("mutation").and_then(|v| v.as_str()).unwrap_or("");
    if got_mutation != name {
        push(
            mismatches,
            "place",
            "mutation",
            name.to_string(),
            got_mutation.to_string(),
        );
    }
    let facts = |face_name: &str, key: &str| {
        face.get(face_name)
            .and_then(|f| f.get("facts"))
            .and_then(|f| f.get(key))
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::MAX)
    };
    // Stale face: the pre-mutation token must be REJECTED by the stale
    // validation; mints=1, one legal read happened, the rejected use added
    // NO buffer access.
    let stale_outcome = face
        .get("stale")
        .and_then(|f| f.get("outcome"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let stale_trap = face
        .get("stale")
        .and_then(|f| f.get("trap"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let stale_live = face
        .get("stale")
        .and_then(|f| f.get("live"))
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    if stale_outcome != "trap" || stale_trap != *want_trap || stale_live != 0 {
        push(
            mismatches,
            "place",
            "stale-trap",
            format!("{want_trap} with live=0"),
            format!("{stale_outcome}/{stale_trap} live={stale_live}"),
        );
    }
    for (key, want) in [
        ("mints", 1u64),
        ("get_ok", 1),
        ("get_stale", 1),
        ("set_ok", 0),
        ("set_stale", 0),
        // the legal pre-mutation read is the ONLY buffer access.
        ("buffer_reads", 1),
    ] {
        let got = facts("stale", key);
        if got != want {
            push(
                mismatches,
                "place",
                "stale-facts",
                format!("{key}={want}"),
                format!("{key}={got}"),
            );
        }
    }
    // Fresh face: a token minted after the mutation reads the current value.
    let fresh_outcome = face
        .get("fresh")
        .and_then(|f| f.get("outcome"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let fresh_value = face
        .get("fresh")
        .and_then(|f| f.get("value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    if fresh_outcome != "success" || fresh_value != *want_fresh as i64 {
        push(
            mismatches,
            "place",
            "fresh-read",
            format!("success {want_fresh}"),
            format!("{fresh_outcome} {fresh_value}"),
        );
    }
    for (key, want) in [("get_ok", 1u64), ("get_stale", 0), ("buffer_reads", 1)] {
        let got = facts("fresh", key);
        if got != want {
            push(
                mismatches,
                "place",
                "fresh-facts",
                format!("{key}={want}"),
                format!("{key}={got}"),
            );
        }
    }
    // Legal face: no mutation at all — the token stays valid.
    let legal_outcome = face
        .get("legal")
        .and_then(|f| f.get("outcome"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let legal_value = face
        .get("legal")
        .and_then(|f| f.get("value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    if legal_outcome != "success" || legal_value != *want_legal as i64 {
        push(
            mismatches,
            "place",
            "legal-read",
            format!("success {want_legal}"),
            format!("{legal_outcome} {legal_value}"),
        );
    }
    if facts("legal", "get_stale") != 0 {
        push(
            mismatches,
            "place",
            "legal-facts",
            "get_stale=0".into(),
            format!("get_stale={}", facts("legal", "get_stale")),
        );
    }
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
/// PLAN-912 T-05: the FUNCTION-session closures source judge (parent-side
/// independent oracle). Values and diagnostics come from the parent's own
/// frozen tables; the zero-production axes (route, warm gate, whole/mir
/// attempts, retained pool) are judged from the worker's observation, never
/// from the worker's claim about itself.
/// PLAN-912: the dedicated function-session corpus (the 912 suite). The
/// K01..K06 / S01..S08 rows are shared verbatim with the source table
/// (V-01: the original corpus through the new chain).
const CLOSURES_FN_CASES: &[(&str, Option<i32>, Option<&str>, usize)] = &[
    // F01: two sibling lambdas + two unrelated functions; fresh = 51
    // (21 + 21 + 7 + 2), body-edit of the SECOND lambda's body (x - n →
    // x - n * 2) = 48, 5 units.
    ("F01", Some(51), None, 5),
    // F02: the same source; body-edit is the OWNER body (x: 10 → 20) = 61
    // with both lambdas reused.
    ("F02", Some(51), None, 5),
];

fn closures_fn_case(id: &str) -> Option<(&'static str, Option<i32>, Option<&'static str>, usize)> {
    if let Some((cid, v, d, n)) = CLOSURES_FN_CASES.iter().find(|(c, ..)| *c == id) {
        return Some((cid, *v, *d, *n));
    }
    CLOSURES_SOURCE_CASES
        .iter()
        .find(|(c, ..)| *c == id)
        .map(|(c, v, d, n)| (*c, *v, *d, *n))
}

/// PLAN-912: the expected `body-edit` value per source (the edit pair lives
/// in the case file; the parent holds the oracle).
fn closures_fn_body_edit_value(id: &str) -> i32 {
    match id {
        "F01" => 48,
        "F02" => 61,
        other => closures_source_body_edit_value(other),
    }
}

fn face_u64(face: &serde_json::Value, field: &str) -> Option<u64> {
    face.get(field).and_then(|v| v.as_u64())
}

fn judge_closures_fn(
    id: &str,
    mode: Option<&str>,
    step: &StepOut,
    mismatches: &mut Vec<Mismatch>,
) {
    let Some((_, base_value, want_code, function_count)) = closures_fn_case(id) else {
        mismatches.push(Mismatch {
            observation: "closures_fn".into(),
            kind: "source-registration".into(),
            expected: format!("source `{id}` in the parent table"),
            actual: "unregistered".into(),
            first_diff: None,
        });
        return;
    };
    let obs = "closures_fn";
    let bad =
        |mismatches: &mut Vec<Mismatch>, kind: &'static str, expected: String, actual: String| {
            mismatches.push(Mismatch {
                observation: obs.to_string(),
                kind,
                expected,
                actual,
                first_diff: None,
            });
        };
    if let Some(want_code) = want_code {
        if step.outcome != "diag" {
            bad(mismatches, "source-rejection", format!("compile-time diagnostic {want_code}"), step_summary(step));
            return;
        }
        let got_code = step
            .diag
            .as_ref()
            .and_then(|d| d.get("code"))
            .and_then(|c| c.as_str())
            .unwrap_or_default();
        if got_code != want_code {
            bad(mismatches, "source-diagnostic", want_code.to_string(), got_code.to_string());
        }
        if step.closures_source.is_some() {
            bad(mismatches, "source-rejection-face", "no runtime face for a rejected source".into(), "face present".into());
        }
        return;
    }
    let want_value = match mode {
        Some("body-edit") => closures_fn_body_edit_value(id),
        // hit/fresh/restart all run the base source.
        _ => base_value.expect("success case has an expected value"),
    };
    if step.outcome != "success" {
        bad(mismatches, "source-outcome", "success".into(), step_summary(step));
        return;
    }
    if step.value != Some(want_value) {
        bad(mismatches, "source-result", want_value.to_string(), format!("{:?}", step.value));
    }
    let Some(face) = &step.closures_source else {
        bad(mismatches, "source-resource-report", "function-session face".into(), "no closures_source report".into());
        return;
    };
    if face_u64(face, "live_after_release") != Some(0) {
        bad(mismatches, "source-live", "0".into(), format!("{:?}", face_u64(face, "live_after_release")));
    }
    let (allocs, frees) = (face_u64(face, "allocs"), face_u64(face, "frees"));
    if allocs.is_none() || frees.is_none() || allocs != frees {
        bad(mismatches, "source-alloc-free-balance", format!("{allocs:?}"), format!("{frees:?}"));
    }
    // The 912 axes.
    let route = face.get("route").and_then(|r| r.as_str()).unwrap_or("");
    let gate = face.get("warm_gate_pass").and_then(|g| g.as_bool());
    let whole = face_u64(face, "whole_attempts");
    let mir_lower = face_u64(face, "mir_lower_attempts");
    let retained = face_u64(face, "retained_bytes").unwrap_or(u64::MAX);
    let units = function_count as u64;
    // The warm gate verdict exists exactly on the WARM routes; a cold or
    // non-fast apply claiming a warm gate would be a lie.
    match mode {
        Some("body-edit") | Some("hit") => {
            if gate.is_none() {
                bad(mismatches, "warm-gate", "warm gate verdict present".into(), "absent".into());
            }
        }
        _ => {
            if face.get("warm_gate_pass").map(|v| !v.is_null()).unwrap_or(false) {
                bad(mismatches, "warm-gate", "null on a cold route".into(), "present".into());
            }
        }
    }
    // §5.5 pool: the retained frontend estimate stays within 16 MiB.
    if retained > 16 * 1024 * 1024 {
        bad(mismatches, "retained-pool", "<= 16 MiB".into(), retained.to_string());
    }
    let counters = |field: &str| face_u64(face, field);
    match mode {
        Some("body-edit") => {
            if route != "fast-unit" && route != "fast-owner" {
                bad(mismatches, "route", "fast-unit|fast-owner".into(), route.to_string());
            }
            if gate != Some(true) {
                bad(mismatches, "warm-gate", "pass".into(), format!("{gate:?}"));
            }
            if whole != Some(0) {
                bad(mismatches, "whole-attempts", "0".into(), whole.unwrap_or_default().to_string());
            }
            let submitted = counters("submitted_units");
            if submitted.is_none() || submitted.unwrap() == 0 || submitted.unwrap() >= units {
                bad(mismatches, "submitted", format!("1..{} (strictly fewer than whole)", units - 1), format!("{submitted:?}"));
            }
            for field in ["verified", "encoded", "admitted"] {
                if counters(field) != submitted {
                    bad(mismatches, field, format!("{submitted:?}"), format!("{:?}", counters(field)));
                }
            }
            if counters("cache_hits") != Some(0) {
                bad(mismatches, "cache-hits", "0".into(), format!("{:?}", counters("cache_hits")));
            }
        }
        Some("hit") => {
            if route != "zero-hit" {
                bad(mismatches, "route", "zero-hit".into(), route.to_string());
            }
            if gate != Some(true) {
                bad(mismatches, "warm-gate", "pass".into(), format!("{gate:?}"));
            }
            if whole != Some(0) {
                bad(mismatches, "whole-attempts", "0".into(), whole.unwrap_or_default().to_string());
            }
            for field in ["submitted_units", "verified", "encoded", "admitted", "store_writes", "cache_misses", "cache_hits"] {
                if counters(field) != Some(0) {
                    bad(mismatches, field, "0".into(), format!("{:?}", counters(field)));
                }
            }
            if mir_lower != Some(0) {
                bad(mismatches, "mir-lower", "0 on a hit".into(), mir_lower.unwrap_or_default().to_string());
            }
        }
        Some("restart") => {
            if route != "cold" {
                bad(mismatches, "route", "cold".into(), route.to_string());
            }
            for field in ["verified", "encoded", "admitted"] {
                if counters(field) != Some(units) {
                    bad(mismatches, field, units.to_string(), format!("{:?}", counters(field)));
                }
            }
            if counters("cache_hits") != Some(units) {
                bad(mismatches, "cache-hits", units.to_string(), format!("{:?}", counters("cache_hits")));
            }
            if counters("store_writes") != Some(0) {
                bad(mismatches, "store-writes", "0".into(), format!("{:?}", counters("store_writes")));
            }
            // R3 stays OPEN and is REPORTED, not hidden: a restart still
            // re-lowers every unit before the content key is computable.
            if mir_lower != Some(units) {
                bad(mismatches, "mir-lower", format!("{units} (R3 open: honestly non-zero)"), mir_lower.unwrap_or_default().to_string());
            }
            if whole.unwrap_or_default() < 1 {
                bad(mismatches, "whole-attempts", ">= 1 (the truth path is whole)".into(), whole.unwrap_or_default().to_string());
            }
        }
        _ => {
            // fresh
            if route != "cold" {
                bad(mismatches, "route", "cold".into(), route.to_string());
            }
            for field in ["submitted_units", "verified", "encoded", "admitted", "store_writes", "cache_misses"] {
                if counters(field) != Some(units) {
                    bad(mismatches, field, units.to_string(), format!("{:?}", counters(field)));
                }
            }
            if counters("cache_hits") != Some(0) {
                bad(mismatches, "cache-hits", "0".into(), format!("{:?}", counters("cache_hits")));
            }
            if whole.unwrap_or_default() < 1 {
                bad(mismatches, "whole-attempts", ">= 1 (the truth path is whole)".into(), whole.unwrap_or_default().to_string());
            }
        }
    }
}

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
        closures_source_body_edit_value(id)
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
            // PLAN-914: the MARK TRACE oracle. The worker reports the
            // executed `hir.test.trace.v1` events in execution order; the
            // parent compares that observed sequence to the declared bytes
            // (the worker never sees this expectation). A recipe that does
            // not report a trace can never satisfy it (empty actual is red
            // for any non-empty oracle).
            if let Some(want) = &expect.trace {
                let actual: String = step.trace.concat();
                if let Some(m) =
                    compare_bytes(want.text.as_bytes(), actual.as_bytes(), want.compare, "trace")
                {
                    mismatches.push(m);
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
                    || (family == RecipeFamily::DataIncremental && step.typed.is_some())
                    || (family == RecipeFamily::Owned && step.owned.is_some())
                    // PLAN-915: the resource recipe's typed witnesses report
                    // their own faces (a stale-Place sequence or a capture
                    // clone sweep has no i32 result); the face IS the
                    // success evidence and is judged separately.
                    || (family == RecipeFamily::Resource
                        && (step.place.is_some() || step.capture.is_some()));
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
            // PLAN-919: the lifecycle recipe's exit faces — a resource
            // refusal reports its kind code, a cancellation its fixed
            // code, both at the run stage.
            "resource" => {
                if *stage != Stage::Run {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{stage:?}"),
                        actual: "run (resource)".into(),
                        first_diff: None,
                    });
                }
                if let Some(want_code) = code {
                    if step.exit_kind.as_deref() != Some(want_code.as_str()) {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want_code.clone(),
                            actual: step.exit_kind.clone().unwrap_or_default(),
                            first_diff: None,
                        });
                    }
                }
            }
            "cancelled" => {
                if *stage != Stage::Run {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{stage:?}"),
                        actual: "run (cancelled)".into(),
                        first_diff: None,
                    });
                }
                if let Some(want_code) = code {
                    if want_code != "av.cancelled" {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want_code.clone(),
                            actual: "av.cancelled".into(),
                            first_diff: None,
                        });
                    }
                }
            }
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

#[cfg(test)]
mod judge_tests {
    //! F820-R7-01 pins: the typed channel is a DATA-recipe increment —
    //! the shared judge and admission rules are exercised HERE, on the
    //! production functions, with synthetic steps (the reviewer's
    //! fault-worker e2e shapes: core success missing the i32 value, with
    //! and without an attached typed object).

    use super::*;

    fn step(value: Option<i32>, typed: Option<TypedOut>) -> StepOut {
        StepOut {
            rebuilt_defs: Vec::new(),
            identities_before: Default::default(),
            identities_after: Default::default(),
            refusal: None,
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
            recovery: None,
            dynamic: None,
            channel: None,
            console: None,
            resource: None,
            place: None,
            capture: None,
            exit_kind: None,
            lifecycle: None,
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
            trace: None,
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

#[cfg(test)]
mod closures_judge_tests {
    //! PLAN-823 (R1-F02): the PARENT-side closures judgement — the frozen
    //! table decides; a lying worker face turns the verdict red.

    use super::*;

    fn cstep(face: serde_json::Value) -> StepOut {
        StepOut {
            rebuilt_defs: Vec::new(),
            identities_before: Default::default(),
            identities_after: Default::default(),
            refusal: None,
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
            recovery: None,
            dynamic: None,
            channel: None,
            console: None,
            resource: None,
            place: None,
            capture: None,
            exit_kind: None,
            lifecycle: None,
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

#[cfg(test)]
mod closures_source_judge_tests {
    use super::*;

    fn successful_step(value: Option<i32>) -> StepOut {
        StepOut {
            rebuilt_defs: Vec::new(),
            identities_before: Default::default(),
            identities_after: Default::default(),
            refusal: None,
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
            recovery: None,
            dynamic: None,
            channel: None,
            console: None,
            resource: None,
            place: None,
            capture: None,
            exit_kind: None,
            lifecycle: None,
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

#[cfg(test)]
mod console_tests {
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
    fn console_admission_rejects_missing_truncated_malformed_and_foreign_faces() {
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
    fn console_admission_caps_combined_bytes_and_keeps_resources_errors() {
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




/// PLAN-914 (I04, D-05 criterion): judge the data-incremental edits against
/// the declared invalidation oracle — every declared rebuilt def must really
/// be in the edit's dependency-precise rebuild set, and every def declared
/// stable must keep its resident code identity (and never appear in the
/// rebuild set).
fn judge_invalidation(
    steps: &[StepOut],
    spec: &auto_test_case::contract::ProbeAvInvalidationSpec,
    mismatches: &mut Vec<Mismatch>,
) {
    let edits: Vec<&StepOut> = steps.iter().filter(|s| s.op == "edit-source").collect();
    if edits.is_empty() {
        mismatches.push(Mismatch {
            observation: "invalidation".into(),
            kind: "steps".into(),
            expected: "at least one edit-source step".into(),
            actual: "none".into(),
            first_diff: None,
        });
        return;
    }
    let rebuilt: std::collections::BTreeSet<&str> = edits
        .iter()
        .flat_map(|e| e.rebuilt_defs.iter().map(String::as_str))
        .collect();
    for def in &spec.rebuilt_defs {
        if !rebuilt.contains(def.as_str()) {
            mismatches.push(Mismatch {
                observation: format!("rebuilt[{def}]"),
                kind: "invalidation".into(),
                expected: "def in the dependency-precise rebuild set".into(),
                actual: format!("build set {:?}", rebuilt),
                first_diff: None,
            });
        }
    }
    for def in &spec.stable_defs {
        if rebuilt.contains(def.as_str()) {
            mismatches.push(Mismatch {
                observation: format!("rebuilt[{def}]"),
                kind: "invalidation".into(),
                expected: "an UNAFFECTED def (never rebuilt)".into(),
                actual: "present in the rebuild set".into(),
                first_diff: None,
            });
        }
        let mut seen = false;
        for e in &edits {
            if let (Some(b), Some(a)) = (
                e.identities_before.get(def),
                e.identities_after.get(def),
            ) {
                seen = true;
                if b != a {
                    mismatches.push(Mismatch {
                        observation: format!("identities[{def}]"),
                        kind: "identity".into(),
                        expected: format!("carried code identity {b:?}"),
                        actual: format!("{a:?}"),
                        first_diff: None,
                    });
                }
            }
        }
        if !seen {
            mismatches.push(Mismatch {
                observation: format!("identities[{def}]"),
                kind: "missing".into(),
                expected: "a resident code identity around the edit".into(),
                actual: "absent".into(),
                first_diff: None,
            });
        }
    }
}


// -- PLAN-913: source-bound recovery ---------------------------------------

/// The ids the recovery recipe may publish/restore. The BASE value and the
/// unit count come from the shared closures tables (one corpus, one oracle);
/// this table only adds the value AFTER the post-restore body edit.
const CLOSURES_RECOVERY_EDIT_VALUES: &[(&str, i32)] = &[
    // F01: the second lambda's body (x - n) -> (x - n * 2) = 48.
    ("F01", 48),
    // K06: no body edit is exercised; the restore leg re-runs the source.
    ("K06", 42),
];

fn closures_recovery_case(id: &str) -> Option<(&'static str, i32, i32, usize)> {
    let (cid, base, want_code, units) = closures_fn_case(id)?;
    if want_code.is_some() {
        return None;
    }
    let base = base?;
    let edited = CLOSURES_RECOVERY_EDIT_VALUES
        .iter()
        .find(|(c, _)| *c == id)
        .map(|(_, v)| *v)
        .unwrap_or(base);
    Some((cid, base, edited, units))
}

/// PLAN-913: judge one recovery step. The parent holds the value oracle; the
/// worker's face is only a claim. A positive restore MUST report zero
/// source-production attempts and real deep-verification work, and the
/// post-restore edit MUST be the warm single-unit route with every unchanged
/// unit keeping its object and producing nothing.
fn judge_closures_recovery(
    id: &str,
    mode: Option<&str>,
    step: &StepOut,
    source_bytes: &[u8],
    mismatches: &mut Vec<Mismatch>,
) {
    let Some((_, base_value, edited_value, units)) = closures_recovery_case(id) else {
        mismatches.push(Mismatch {
            observation: "closures_recovery".into(),
            kind: "source-registration".into(),
            expected: format!("recovery source `{id}` in the parent table"),
            actual: "unregistered".into(),
            first_diff: None,
        });
        return;
    };
    let obs = "closures_recovery";
    let bad =
        |mismatches: &mut Vec<Mismatch>, kind: &'static str, expected: String, actual: String| {
            mismatches.push(Mismatch {
                observation: obs.to_string(),
                kind,
                expected,
                actual,
                first_diff: None,
            });
        };
    if step.outcome != "success" {
        bad(mismatches, "recovery-outcome", "success".into(), step_summary(step));
        return;
    }
    let Some(face) = &step.recovery else {
        bad(
            mismatches,
            "recovery-face",
            "a recovery face".into(),
            "no recovery report".into(),
        );
        return;
    };
    let units_want = units as u64;
    let phase = face
        .get("phase")
        .and_then(|p| p.as_str())
        .unwrap_or("");
    if !matches!(phase, "publish" | "restore" | "edit") {
        bad(
            mismatches,
            "recovery-phase",
            "publish|restore|edit".into(),
            phase.to_string(),
        );
        return;
    }
    // The BINDING legs carry the unit count AND the caller-held binding; the
    // edit face reports the warm edit's identity counters instead (the
    // catalog was already verified). A malformed or missing binding is a
    // mismatch, not a pass.
    if phase != "edit" {
        let manifest = face.get("manifest").and_then(|m| m.as_str()).unwrap_or("");
        let digest = face.get("digest").and_then(|d| d.as_str()).unwrap_or("");
        let well_formed = manifest.starts_with("art")
            && manifest.len() > 16
            && digest.len() == 64
            && digest.chars().all(|c| c.is_ascii_hexdigit());
        if !well_formed {
            bad(
                mismatches,
                "recovery-binding",
                "a manifest artifact id + 64-hex digest".into(),
                format!("manifest={manifest:?} digest={digest:?}"),
            );
        }
        // V-13: the SOURCE hash is parent-verified against the case bytes —
        // the worker's claim is never taken on its own word.
        let want_sha = hex(&sha256(source_bytes));
        let got_sha = face
            .get("source_sha256")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if got_sha != want_sha {
            bad(
                mismatches,
                "recovery-source-hash",
                want_sha,
                got_sha.to_string(),
            );
        }
    }
    if phase != "edit" && face_u64(face, "units") != Some(units_want) {
        bad(
            mismatches,
            "recovery-units",
            units_want.to_string(),
            format!("{:?}", face_u64(face, "units")),
        );
    }
    let want_value = if mode == Some("restore-edit") {
        edited_value
    } else {
        base_value
    };
    if step.value != Some(want_value) {
        bad(
            mismatches,
            "recovery-result",
            want_value.to_string(),
            format!("{:?}", step.value),
        );
    }
    match phase {
        "publish" => {
            // The publication leg is a REAL production: every unit went
            // through verify + encode + admit.
            for field in ["p_verified", "p_encoded", "p_admitted"] {
                if face_u64(face, field) != Some(units_want) {
                    bad(
                        mismatches,
                        "recovery-publish-production",
                        format!("{field}={units_want}"),
                        format!("{:?}", face_u64(face, field)),
                    );
                }
            }
        }
        "restore" => {
            if face_u64(face, "p_axis_attempts") != Some(0) {
                bad(
                    mismatches,
                    "recovery-zero-production",
                    "0 source-production attempts".into(),
                    format!("{:?}", face_u64(face, "p_axis_attempts")),
                );
            }
            for (field, want) in [
                ("r_manifest_reads", 1),
                ("r_unit_reads", units_want),
                ("r_deep_verifies", units_want),
                ("r_admits", units_want),
                ("r_context_reads", 1),
            ] {
                if face_u64(face, field) != Some(want) {
                    bad(
                        mismatches,
                        "recovery-verification",
                        format!("{field}={want}"),
                        format!("{:?}", face_u64(face, field)),
                    );
                }
            }
        }
        _ => {
            if face.get("route").and_then(|r| r.as_str()) != Some("fast-unit") {
                bad(
                    mismatches,
                    "recovery-edit-route",
                    "fast-unit".into(),
                    format!("{:?}", face.get("route")),
                );
            }
            if face.get("warm_gate_pass").and_then(|g| g.as_bool()) != Some(true) {
                bad(
                    mismatches,
                    "recovery-edit-gate",
                    "warm gate pass".into(),
                    format!("{:?}", face.get("warm_gate_violations")),
                );
            }
            for (field, want) in [
                ("whole_attempts", 0),
                ("submitted_units", 1),
                ("unchanged_total", units_want.saturating_sub(1)),
                ("unchanged_same_object", units_want.saturating_sub(1)),
                ("unchanged_production_attempts", 0),
                ("store_writes", 0),
                ("live_after_release", 0),
            ] {
                if face_u64(face, field) != Some(want) {
                    bad(
                        mismatches,
                        "recovery-edit-identity",
                        format!("{field}={want}"),
                        format!("{:?}", face_u64(face, field)),
                    );
                }
            }
            let allocs = face_u64(face, "allocs");
            let frees = face_u64(face, "frees");
            if allocs.is_none() || frees.is_none() || allocs != frees {
                bad(
                    mismatches,
                    "recovery-edit-arena",
                    format!("{allocs:?}"),
                    format!("{frees:?}"),
                );
            }
        }
    }
}

#[cfg(test)]
mod resource_915_tests {
    use super::*;

    fn step_with(resource: Option<serde_json::Value>, place: Option<serde_json::Value>) -> StepOut {
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
            trace: vec![],
            refusal: None,
            rebuilt_defs: vec![],
            identities_before: Default::default(),
            identities_after: Default::default(),
            spans: vec![],
            owned: None,
            collections: None,
            closures: None,
            closures_source: None,
            recovery: None,
            dynamic: None,
            channel: None,
            console: None,
            resource,
            place,
            exit_kind: None,
            lifecycle: None,
            capture: None,
        }
    }

    fn point(attempt: u64, arena: bool) -> serde_json::Value {
        serde_json::json!({
            "attempt": attempt,
            "faulted": true,
            "live": 0,
            "attempts": attempt,
            "successful": attempt - 1,
            "frees": attempt - 1,
            "message": if arena {
                format!("fault-injected at owned allocation attempt #{attempt}")
            } else {
                format!("fault-injected at value allocation #{attempt}")
            },
        })
    }

    /// A complete, well-formed R02 face for the frozen `l04` row (2 arena +
    /// 6 value attempts) — the accepting control.
    fn clean_sweep() -> serde_json::Value {
        serde_json::json!({
            "source_value": 42,
            "arena_attempts": 2,
            "arena_successful": 2,
            "arena_frees": 2,
            "value_attempts": 6,
            "arena_points": (1..=2).map(|n| point(n, true)).collect::<Vec<_>>(),
            "value_points": (1..=6).map(|n| point(n, false)).collect::<Vec<_>>(),
            "arena_control": {"attempt": 3, "faulted": false, "live": 0, "value": 42},
            "value_control": {"attempt": 7, "faulted": false, "live": 0, "value": 42},
            "after": {"value": 42, "live": 0},
            "after_recompile": {"value": 42, "live": 0},
        })
    }

    fn place_face(mutation: &str) -> serde_json::Value {
        serde_json::json!({
            "mutation": mutation,
            "stale": {
                "outcome": "trap",
                "value": null,
                "trap": "av.trap.stale-place",
                "facts": {"mints":1,"get_ok":1,"get_stale":1,"set_ok":0,"set_stale":0,"buffer_reads":1},
                "live": 0,
            },
            "fresh": {
                "outcome": "success",
                "value": 30,
                "trap": null,
                "facts": {"mints":1,"get_ok":1,"get_stale":0,"set_ok":0,"set_stale":0,"buffer_reads":1},
                "live": 0,
            },
            "legal": {
                "outcome": "success",
                "value": 10,
                "trap": null,
                "facts": {"mints":1,"get_ok":2,"get_stale":0,"set_ok":0,"set_stale":0,"buffer_reads":2},
                "live": 0,
            },
        })
    }

    fn capture_face() -> serde_json::Value {
        serde_json::json!({
            "clean": {"attempt": null, "outcome": "success", "fault_attempt": null,
                      "code": null, "live": 2, "source_readable": true},
            "points": [
                {"attempt": 3, "outcome": "faulted-clone", "fault_attempt": 3,
                 "code": "env.capture.faulted-clone", "live": 2, "source_readable": true},
                {"attempt": 4, "outcome": "faulted-clone", "fault_attempt": 4,
                 "code": "env.capture.faulted-clone", "live": 2, "source_readable": true},
            ],
            "control": {"attempt": 5, "outcome": "success", "fault_attempt": null,
                        "code": null, "live": 2, "source_readable": true},
        })
    }

    #[test]
    fn resource_capture_face_is_admitted_and_judged_green() {
        let step = step_with(None, None);
        let mut step = step;
        step.capture = Some(capture_face());
        assert!(malformed_resource(RecipeFamily::Resource, &[step.clone()]).is_none());
        let mut mismatches = Vec::new();
        judge_resource("capture:clone", &step, &mut mismatches);
        assert!(mismatches.is_empty(), "{mismatches:?}");
    }

    #[test]
    fn resource_capture_skipped_point_is_rejected() {
        let mut face = capture_face();
        face["points"].as_array_mut().unwrap().pop();
        let mut step = step_with(None, None);
        step.capture = Some(face);
        let mut mismatches = Vec::new();
        judge_resource("capture:clone", &step, &mut mismatches);
        assert!(!mismatches.is_empty());
        let mut face2 = capture_face();
        face2["points"][1]["code"] = serde_json::json!("env.capture.unsupported-kind");
        let mut step2 = step_with(None, None);
        step2.capture = Some(face2);
        let mut mismatches2 = Vec::new();
        judge_resource("capture:clone", &step2, &mut mismatches2);
        assert!(!mismatches2.is_empty());
    }

    #[test]
    fn resource_clean_faces_are_admitted_and_judged_green() {
        let sweep = step_with(Some(clean_sweep()), None);
        assert!(malformed_resource(RecipeFamily::Resource, &[sweep.clone()]).is_none());
        let mut mismatches = Vec::new();
        judge_resource("source:l04", &sweep, &mut mismatches);
        assert!(mismatches.is_empty(), "{mismatches:?}");

        let place = step_with(None, Some(place_face("push")));
        assert!(malformed_resource(RecipeFamily::Resource, &[place.clone()]).is_none());
        let mut mismatches = Vec::new();
        judge_resource("place:push", &place, &mut mismatches);
        assert!(mismatches.is_empty(), "{mismatches:?}");
    }

    /// V-01 distinguishing control: a worker that skips the LAST allocation
    /// point must fail — both the shape gate (points != attempts) and the
    /// judge (attempt count) must catch it.
    #[test]
    fn resource_skipped_point_is_rejected() {
        let mut face = clean_sweep();
        face["arena_points"].as_array_mut().unwrap().pop();
        let step = step_with(Some(face), None);
        assert!(malformed_resource(RecipeFamily::Resource, &[step.clone()]).is_some());
        let mut mismatches = Vec::new();
        judge_resource("source:l04", &step, &mut mismatches);
        assert!(!mismatches.is_empty());
    }

    /// Only the FIRST point is not enough (the plan's explicit control).
    #[test]
    fn resource_first_point_only_is_rejected() {
        let mut face = clean_sweep();
        face["arena_points"] = serde_json::json!([point(1, true)]);
        let step = step_with(Some(face), None);
        let mut mismatches = Vec::new();
        judge_resource("source:l04", &step, &mut mismatches);
        assert!(!mismatches.is_empty());
    }

    /// V-03: a stale use that "trap"s for another reason must not pass.
    #[test]
    fn resource_non_stale_trap_is_rejected() {
        let mut face = place_face("push");
        face["stale"]["trap"] = serde_json::json!("av.trap.list-index");
        let step = step_with(None, Some(face));
        let mut mismatches = Vec::new();
        judge_resource("place:push", &step, &mut mismatches);
        assert!(!mismatches.is_empty());
    }

    /// The stale use must not have read the buffer: a claim of an extra
    /// buffer read fails the frozen facts.
    #[test]
    fn resource_stale_buffer_access_is_rejected() {
        let mut face = place_face("push");
        face["stale"]["facts"]["buffer_reads"] = serde_json::json!(2);
        let step = step_with(None, Some(face));
        let mut mismatches = Vec::new();
        judge_resource("place:push", &step, &mut mismatches);
        assert!(!mismatches.is_empty());
    }

    /// The unknown-key gate covers all three faces (F-915-R8).
    #[test]
    fn resource_faces_reject_unknown_keys() {
        let mut sweep = clean_sweep();
        sweep["unexpected"] = serde_json::json!(1);
        assert!(
            malformed_resource(RecipeFamily::Resource, &[step_with(Some(sweep), None)]).is_some()
        );

        let mut place = place_face("push");
        place["extra"] = serde_json::json!(true);
        assert!(
            malformed_resource(RecipeFamily::Resource, &[step_with(None, Some(place))]).is_some()
        );

        let mut place_inner = place_face("push");
        place_inner["stale"]["extra"] = serde_json::json!(1);
        assert!(malformed_resource(
            RecipeFamily::Resource,
            &[step_with(None, Some(place_inner))]
        )
        .is_some());

        let mut capture = capture_face();
        capture["points"][0]["extra"] = serde_json::json!(1);
        let mut step = step_with(None, None);
        step.capture = Some(capture);
        assert!(malformed_resource(RecipeFamily::Resource, &[step]).is_some());
    }

    /// The arena ledger axes are judged, not just reported (F-915-R4).
    #[test]
    fn resource_arena_ledger_is_judged() {
        let mut sweep = clean_sweep();
        sweep["arena_points"][0]["successful"] = serde_json::json!(1);
        let step = step_with(Some(sweep), None);
        let mut mismatches = Vec::new();
        judge_resource("source:l04", &step, &mut mismatches);
        assert!(!mismatches.is_empty());
    }

    /// A resource face on another recipe is a protocol violation, and a
    /// resource step must carry exactly one face.
    #[test]
    fn resource_face_admission_rules() {
        let step = step_with(Some(clean_sweep()), None);
        assert!(resource_on_non_resource(
            RecipeFamily::Core,
            &[step.clone()]
        ));
        assert!(!resource_on_non_resource(
            RecipeFamily::Resource,
            &[step.clone()]
        ));
        let both = step_with(Some(clean_sweep()), Some(place_face("push")));
        let mut capture_mixed = step_with(Some(clean_sweep()), None);
        capture_mixed.capture = Some(capture_face());
        assert!(malformed_resource(RecipeFamily::Resource, &[capture_mixed]).is_some());
        assert!(malformed_resource(RecipeFamily::Resource, &[both]).is_some());
        let neither = step_with(None, None);
        assert!(malformed_resource(RecipeFamily::Resource, &[neither]).is_some());
    }
}

#[cfg(test)]
mod worker_json_tests {
    use super::reject_duplicate_keys;

    #[test]
    fn valid_response_passes() {
        let line = r#"{"schema":"avworker/1","nonce":"n","request_hash":"h","ok":{"recipe":"ac-av/resource-run-v1","steps":[{"op":"run","outcome":"success","value":42,"resource":{"source_value":42,"arena_points":[{"attempt":1,"faulted":true,"live":0}]}}],"worker_version":"avworker/1.0","av_core_version":"av.collections/1"}}"#;
        assert!(reject_duplicate_keys(line).is_ok());
    }

    #[test]
    fn duplicate_key_is_rejected() {
        let line = r#"{"ok":{"resource":{"source_value":42,"source_value":0}}}"#;
        let err = reject_duplicate_keys(line).expect_err("duplicate must reject");
        assert!(err.contains("duplicate"), "{err}");
    }

    #[test]
    fn escaped_duplicate_key_is_rejected() {
        // "\u0076alue" decodes to "value" — the decoded form collides.
        let line = r#"{"value":1,"\u0076alue":2}"#;
        assert!(reject_duplicate_keys(line).is_err());
    }

    #[test]
    fn truncated_or_trailing_frames_are_rejected() {
        assert!(reject_duplicate_keys(r#"{"a":1"#).is_err());
        assert!(reject_duplicate_keys(r#"{"a":1} {}"#).is_err());
    }
}
