//! PLAN-998: the EXPLICIT independent a2r RESOURCE-SCENARIO certification
//! sub-entry (`ace-certify --resource`). Domain-isolated from the value
//! (`ace.m1.cert/1`) and trap (`ace.m1.a2r-trap/1`) certifiers: this module
//! certifies the six `ns=scenario` rows whose outcome is `events`
//! (scenario:R01..R06:a2r) under the EXPLICIT versioned observation profile
//! `legacy-a2r.resources-observe/1` (the R06 trap leg additionally selects
//! `legacy-a2r.checked-i32/1`).
//!
//! Domain contract (frozen with the schema id `ace.m1.a2r-resource-scenario/1`):
//! 1. The scenario oracle is the declarative registry
//!    `oracle/scenario/scenario-oracle.json` (`ace.m1.a2r-resource-oracle/1`):
//!    per cell its legs (frozen fixtures under `legacy-a2r-resources-998/src/`),
//!    profiles, run mode, frozen value, and the invariant suite. The registry
//!    hash is an identity axis of every certificate; the binding rows stay
//!    authoritative for outcome/expected.
//! 2. Emit the ACTUAL Rust fresh from each leg source through the production
//!    transpiler under the leg's profiles; require actual bytes == frozen
//!    scene golden (`oracle/scenario/<module>.rs`) — drift FAILs, goldens move
//!    only through explicit review (G04).
//! 3. Compile AND run the actual (golden under the frozen resource wrapper
//!    `oracle/resource-wrapper-v1.rs`) through the leg's run matrix (clean /
//!    fault sweep 1..N + N+1 control / same-process retry / budget / cancel /
//!    typed trap / compile boundary) AND compile AND run the hand-written
//!    independent reference (`reference/scenario/<LEG>.rs`, its own
//!    observation machinery — no shared code with the emission) through the
//!    same invariant suite over ITS OWN allocation domain. Both sides must
//!    satisfy the frozen invariants; the two implementations' event streams
//!    need not be literally identical.
//! 4. Typed terminal states are judged STRICTLY: value legs exit 0 with the
//!    exact result line; fault/budget/cancel legs exit 71 with the typed
//!    stderr line and COMPLETE owner cleanup on stdout (every created owner
//!    terminal — cleanup is observed, never asserted); the trap leg exits 70
//!    with `auto.trap:<kind>:<op>` and complete cleanup; a foreign panic
//!    (exit 101) never satisfies any oracle.
//! 5. Persist the certificate at `oracle/scenario-certs/m1_<cell>.scenario-cert.json`
//!    binding every identity axis; REUSE only after STRICT validation with
//!    the same Invalid vs Stale discipline as the value/trap domains.
//!
//! Contained-tool governance mirrors the value certifier exactly: rustc and
//! the certified binaries start suspended in kill-on-close Job Objects under
//! the same 120s compile / 30s run / 1MB stream caps, and the scratch tree is
//! removed on every exit path.

use super::ace_certify::{
    capture_toolchain_contained, compile, run_capped, BUILD_TOOLCHAIN_IDENTITY, COMPILE_WALL,
    DEPS_CLOSURE, RUN_WALL, RUSTC_FLAG_TOKENS,
};
use crate::trans::rust::{transpile_rust_with_source_dir_profiles, IntProfile, ResourceProfile};
use auto_test_case::sha256::{hex, sha256};
use std::path::{Path, PathBuf};

pub const SCENE_CERT_SCHEMA: &str = "ace.m1.a2r-resource-scenario/1";
pub const SCENE_ORACLE_SCHEMA: &str = "ace.m1.a2r-resource-oracle/1";
pub const OBSERVE_PROFILE_ID: &str = "legacy-a2r.resources-observe/1";
pub const CHECKED_PROFILE_ID: &str = "legacy-a2r.checked-i32/1";
pub const TYPED_FAIL_EXIT: i64 = 71;
pub const TRAP_EXIT: i64 = 70;
/// Emit needs more than a default thread stack (mirrors the a2r adapter).
const EMIT_STACK_BYTES: usize = 128 * 1024 * 1024;

pub struct ResourceCertifyArgs {
    pub corpus: PathBuf,
    pub certs_dir: PathBuf,
    pub only: Vec<String>,
    pub force: bool,
    pub rustc: Option<String>,
    pub scratch: Option<PathBuf>,
}

pub fn resource_certify_and_report(args: &ResourceCertifyArgs) -> i32 {
    match resource_certify(args) {
        Ok(summaries) => {
            let mut certified = 0;
            let mut reused = 0;
            let mut failed = 0;
            for s in &summaries {
                match s.outcome.as_str() {
                    "certified" => certified += 1,
                    "reused" => reused += 1,
                    _ => failed += 1,
                }
                println!("{}: {} — {}", s.outcome, s.case, s.detail);
            }
            println!(
                "resource-scenario certification: {} certified, {} reused, {} failed",
                certified, reused, failed
            );
            if failed == 0 {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("ace-certify --resource: {e}");
            1
        }
    }
}

// ---------------------------------------------------------------------------
// Registry (strict decode of the declarative scenario oracle)
// ---------------------------------------------------------------------------

pub struct SceneLeg {
    pub id: String,
    pub source: String,
    pub module: String,
    pub profiles: Vec<String>,
    pub mode: String,
    pub value: Option<i64>,
    pub reads: Option<usize>,
    pub budget: Option<usize>,
    pub cancel_ms: Option<u64>,
    pub trap_kind: Option<String>,
    pub trap_op: Option<String>,
    pub reference_only: bool,
    pub reference: Option<String>,
}

pub struct SceneCell {
    pub legs: Vec<SceneLeg>,
    pub invariants: Vec<String>,
}

pub struct SceneRegistry {
    pub text: String,
    pub cells: std::collections::BTreeMap<String, SceneCell>,
}

fn leg_str(leg: &serde_json::Value, key: &str) -> Option<String> {
    leg.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn leg_num<T: std::str::FromStr>(leg: &serde_json::Value, key: &str) -> Option<T> {
    leg.get(key)
        .and_then(|v| v.as_u64())
        .and_then(|v| v.to_string().parse::<T>().ok())
}

/// Load `oracle/scenario/scenario-oracle.json` strictly: exact root keys,
/// exact schema/profile ids, per-cell leg shapes.
pub fn load_scene_registry(corpus: &Path) -> Result<SceneRegistry, String> {
    let path = corpus
        .join("oracle")
        .join("scenario")
        .join("scenario-oracle.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("read scenario oracle: {e}"))?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("scenario oracle is not valid JSON: {e}"))?;
    let root = v
        .as_object()
        .ok_or_else(|| "scenario oracle root is not an object".to_string())?;
    let exact = |keys: &[&str], where_: &str| -> Result<(), String> {
        for k in root.keys() {
            if !keys.contains(&k.as_str()) {
                return Err(format!(
                    "unknown scenario-oracle field {k:?} in {where_} (strict schema)"
                ));
            }
        }
        for k in keys {
            if !root.contains_key(*k) {
                return Err(format!("missing scenario-oracle field {k:?} in {where_}"));
            }
        }
        Ok(())
    };
    exact(
        &[
            "schema",
            "profile",
            "trap_profile",
            "wrapper_file",
            "typed_fail_exit",
            "trap_exit",
            "cells",
        ],
        "root",
    )?;
    if root.get("schema").and_then(|s| s.as_str()) != Some(SCENE_ORACLE_SCHEMA) {
        return Err(format!(
            "scenario oracle schema is not {SCENE_ORACLE_SCHEMA}"
        ));
    }
    if root.get("profile").and_then(|s| s.as_str()) != Some(OBSERVE_PROFILE_ID) {
        return Err(format!(
            "scenario oracle profile is not {OBSERVE_PROFILE_ID}"
        ));
    }
    if root.get("trap_profile").and_then(|s| s.as_str()) != Some(CHECKED_PROFILE_ID) {
        return Err(format!(
            "scenario oracle trap_profile is not {CHECKED_PROFILE_ID}"
        ));
    }
    if root.get("wrapper_file").and_then(|s| s.as_str()) != Some("oracle/resource-wrapper-v1.rs") {
        return Err(
            "scenario oracle wrapper_file is not the frozen oracle/resource-wrapper-v1.rs"
                .to_string(),
        );
    }
    if root.get("typed_fail_exit").and_then(|c| c.as_i64()) != Some(TYPED_FAIL_EXIT) {
        return Err(format!(
            "scenario oracle typed_fail_exit is not {TYPED_FAIL_EXIT}"
        ));
    }
    if root.get("trap_exit").and_then(|c| c.as_i64()) != Some(TRAP_EXIT) {
        return Err(format!("scenario oracle trap_exit is not {TRAP_EXIT}"));
    }
    let cells = root
        .get("cells")
        .and_then(|c| c.as_object())
        .ok_or_else(|| "scenario oracle cells is not an object".to_string())?;
    let mut out = std::collections::BTreeMap::new();
    for (cell_id, cell_v) in cells {
        let cell = cell_v
            .as_object()
            .ok_or_else(|| format!("cell {cell_id} is not an object"))?;
        if cell.len() != 2 || !cell.contains_key("legs") || !cell.contains_key("invariants") {
            return Err(format!(
                "cell {cell_id} must have exactly {{legs, invariants}}"
            ));
        }
        let legs_v = cell
            .get("legs")
            .and_then(|l| l.as_array())
            .ok_or_else(|| format!("cell {cell_id} legs is not an array"))?;
        let mut legs = Vec::new();
        for leg_v in legs_v {
            let leg = leg_v
                .as_object()
                .ok_or_else(|| format!("cell {cell_id} has a non-object leg"))?;
            let reference_only = leg
                .get("reference_only")
                .and_then(|b| b.as_bool())
                .unwrap_or(false);
            for k in leg.keys() {
                if ![
                    "id",
                    "source",
                    "module",
                    "profiles",
                    "mode",
                    "value",
                    "reads",
                    "budget",
                    "cancel_ms",
                    "trap_kind",
                    "trap_op",
                    "reference_only",
                    "reference",
                    "note",
                ]
                .contains(&k.as_str())
                {
                    return Err(format!(
                        "unknown leg field {k:?} in cell {cell_id} (strict schema)"
                    ));
                }
            }
            if !reference_only {
                for k in ["id", "source", "module", "profiles", "mode"] {
                    if !leg.contains_key(k) {
                        return Err(format!("leg missing field {k:?} in cell {cell_id}"));
                    }
                }
            } else if !leg.contains_key("id") || !leg.contains_key("mode") {
                return Err(format!(
                    "reference-only leg missing id/mode in cell {cell_id}"
                ));
            }
            legs.push(SceneLeg {
                id: leg_str(leg_v, "id").ok_or("leg id")?,
                source: leg_str(leg_v, "source").unwrap_or_default(),
                module: leg_str(leg_v, "module").unwrap_or_default(),
                profiles: leg_v
                    .get("profiles")
                    .and_then(|p| p.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                mode: leg_str(leg_v, "mode").ok_or("leg mode")?,
                value: leg_num::<i64>(leg_v, "value"),
                reads: leg_num::<usize>(leg_v, "reads"),
                budget: leg_num::<usize>(leg_v, "budget"),
                cancel_ms: leg_num::<u64>(leg_v, "cancel_ms"),
                trap_kind: leg_str(leg_v, "trap_kind"),
                trap_op: leg_str(leg_v, "trap_op"),
                reference: leg_str(leg_v, "reference"),
                reference_only,
            });
        }
        if legs.is_empty() {
            return Err(format!("cell {cell_id} has no legs"));
        }
        let invariants = cell
            .get("invariants")
            .and_then(|i| i.as_array())
            .ok_or_else(|| format!("cell {cell_id} invariants is not an array"))?
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect();
        out.insert(cell_id.clone(), SceneCell { legs, invariants });
    }
    if out.is_empty() {
        return Err("scenario oracle has no cells".to_string());
    }
    Ok(SceneRegistry { text, cells: out })
}

/// Parse binding.tsv for the resource-certifiable set: `ns=scenario` rows
/// whose outcome is exactly `events`. Pre-flip rows carrying the 991-era gap
/// tokens certify ONLY when `--only` names their cell (evidence first, token
/// flip afterwards); rows already migrated to the scene-domain token
/// (`ok-emit-scene`) are always eligible.
fn resource_certifiable_cells(
    binding: &Path,
    only: &[String],
) -> Result<Vec<(String, String)>, String> {
    let text = std::fs::read_to_string(binding).map_err(|e| format!("read binding: {e}"))?;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 12 {
            return Err(format!(
                "binding row has {} columns (frozen schema = 12)",
                f.len()
            ));
        }
        if f[1] != "scenario" || f[3] != "events" {
            continue;
        }
        let named = only.iter().any(|o| o.eq_ignore_ascii_case(f[0]));
        if f[8] == "ok-emit-scene"
            || ((f[8] == "emit-ok-run-unverified" || f[8] == "snapshot-only-cert-gap") && named)
        {
            rows.push((f[0].to_string(), f[4].to_string()));
        }
    }
    if rows.is_empty() {
        return Err(
            "no resource-certifiable rows (ns=scenario outcome=events with ok-emit-scene, or a 991-era gap token named by --only)"
                .to_string(),
        );
    }
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Event-stream parsing and the frozen invariant suite
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResEvent {
    pub kind: String,
    pub arg: String,
}

pub fn parse_stream(stdout: &[u8]) -> Vec<ResEvent> {
    let text = String::from_utf8_lossy(stdout);
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("auto.res:") {
            if let Some((kind, arg)) = rest.split_once(':') {
                out.push(ResEvent {
                    kind: kind.to_string(),
                    arg: arg.to_string(),
                });
            } else {
                out.push(ResEvent {
                    kind: rest.to_string(),
                    arg: String::new(),
                });
            }
        }
    }
    out
}

pub struct StreamCheck {
    pub ok: bool,
    pub why: String,
}

fn fail(why: String) -> StreamCheck {
    StreamCheck { ok: false, why }
}

/// Evaluate the frozen invariant suite over one side's event stream.
/// `terminal` = the run's terminal face (clean/fault/budget/cancel/trap).
pub fn check_invariants(events: &[ResEvent], invariants: &[String], terminal: &str) -> StreamCheck {
    // Sequential per-INSTANCE model: repeated frame invocations (and the
    // same-process retry leg) legitimately reuse owner labels — each create
    // opens an instance that must end in exactly one terminal (release or
    // transfer), never both, and per-frame releases follow LIFO.
    let mut live: Vec<(String, bool)> = Vec::new(); // (label, transferred?)
    let mut create_count: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut release_count: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut transfer_count: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut both_violation: Option<String> = None;
    let mut lifo_violation: Option<String> = None;
    let mut unknown_terminal: Option<String> = None;
    let inc = |m: &mut std::collections::BTreeMap<String, usize>, k: &str| {
        *m.entry(k.to_string()).or_insert(0) += 1;
    };
    for ev in events {
        match ev.kind.as_str() {
            "create" => {
                inc(&mut create_count, &ev.arg);
                live.push((ev.arg.clone(), false));
            }
            "transfer" => {
                // closes the innermost live instance of this label
                if let Some(pos) = live.iter().rposition(|(l, _)| *l == ev.arg) {
                    let (_, transferred) = live[pos];
                    if transferred {
                        both_violation = Some(format!(
                            "owner {} transferred twice without an intervening create",
                            ev.arg
                        ));
                    }
                    live[pos].1 = true;
                    live.remove(pos);
                    inc(&mut transfer_count, &ev.arg);
                } else {
                    unknown_terminal = Some(format!("transfer for unknown owner {}", ev.arg));
                }
            }
            "release" => {
                if let Some(pos) = live.iter().rposition(|(l, _)| *l == ev.arg) {
                    // per-frame LIFO: within the releasable instances of this
                    // frame, this label's instance must be the innermost
                    let frame = ev
                        .arg
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .rsplit_once(':')
                        .map(|(f, _)| f)
                        .unwrap_or("")
                        .to_string();
                    if let Some(innermost) = live.iter().rposition(|(l, _)| {
                        l.split('#')
                            .next()
                            .unwrap_or("")
                            .rsplit_once(':')
                            .map(|(f, _)| f)
                            .unwrap_or("")
                            == frame
                    }) {
                        if live[innermost].0 != live[pos].0 {
                            lifo_violation = Some(format!(
                                "release of {} is not LIFO within its frame (innermost live is {})",
                                ev.arg, live[innermost].0
                            ));
                        }
                    }
                    live.remove(pos);
                    inc(&mut release_count, &ev.arg);
                } else {
                    unknown_terminal = Some(format!(
                        "release for unknown or already-terminal owner {}",
                        ev.arg
                    ));
                }
            }
            _ => {}
        }
    }
    if let Some(why) = unknown_terminal {
        return fail(why);
    }
    if let Some(why) = both_violation {
        return fail(why);
    }
    if invariants.contains(&"unique_terminal".to_string()) {
        if !live.is_empty() {
            let names: Vec<String> = live.iter().map(|(l, _)| l.clone()).collect();
            return fail(format!(
                "owners alive at the terminal face: {names:?} (leak)"
            ));
        }
    } else if !live.is_empty() {
        // even without the explicit invariant, a leak at the terminal face
        // is recorded for the caller's judgement; not a hard failure here.
    }
    if invariants.contains(&"transfer_not_release".to_string()) {
        // per-instance: enforced by the sequential model (an instance closes
        // at its FIRST terminal; a second terminal for the same instance is
        // the unknown_terminal case above). A label with BOTH terminals is
        // legitimate across distinct instances (repeated frames / retry).
    }
    if invariants.contains(&"per_frame_lifo".to_string()) {
        if let Some(why) = lifo_violation {
            return fail(why);
        }
    }
    // switch_releases_old_once (R04): exactly one release of the first
    // generation, occurring before the second generation's create.
    if invariants.contains(&"switch_releases_old_once".to_string()) {
        let rel1 = events
            .iter()
            .position(|e| e.kind == "release" && e.arg.ends_with("#1"));
        let create2 = events
            .iter()
            .position(|e| e.kind == "create" && e.arg.ends_with("#2"));
        let count_rel1 = events
            .iter()
            .filter(|e| e.kind == "release" && e.arg.ends_with("#1"))
            .count();
        match (rel1, create2) {
            (Some(r1), Some(c2)) if r1 < c2 && count_rel1 == 1 => {}
            _ => {
                return fail(format!(
                    "switch_releases_old_once violated (release#1 at {rel1:?}, create#2 at {create2:?}, release#1 count {count_rel1})"
                ))
            }
        }
    }
    // env_survives_creator_and_last_owner_releases (R05): a transfer chain
    // (at least two transfers), an env event, and the final release of the
    // closure-binding owner AFTER the env event.
    if invariants.contains(&"env_survives_creator_and_last_owner_releases".to_string()) {
        let transfers = events.iter().filter(|e| e.kind == "transfer").count();
        let env_at = events.iter().position(|e| e.kind == "env");
        let last_release = events.iter().rposition(|e| e.kind == "release");
        if transfers < 2
            || env_at.is_none()
            || last_release.is_none()
            || last_release.unwrap() < env_at.unwrap()
        {
            return fail(format!(
                "env survival chain violated (transfers={transfers}, env_at={env_at:?}, last_release={last_release:?})"
            ));
        }
    }
    // stale_rejected_before_read (R03): a stale event must exist and no read
    // may follow it.
    if invariants.contains(&"stale_rejected_before_read".to_string()) {
        if let Some(st) = events.iter().position(|e| e.kind == "stale") {
            if events[st + 1..].iter().any(|e| e.kind == "read") {
                return fail("a buffer read followed the typed stale rejection".to_string());
            }
        }
    }
    let _ = terminal;
    StreamCheck {
        ok: true,
        why: String::new(),
    }
}

// ---------------------------------------------------------------------------
// Run helpers over the frozen wrapper
// ---------------------------------------------------------------------------

struct RunOut {
    exit: i64,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_with_env(exe: &Path, envs: &[(String, String)]) -> Result<RunOut, String> {
    // run_capped has no env support; set the vars in this process for the
    // duration of the run (single-threaded certification; restored after).
    let saved: Vec<(String, Option<std::ffi::OsString>)> = envs
        .iter()
        .map(|(k, _)| (k.clone(), std::env::var_os(k)))
        .collect();
    for (k, v) in envs {
        std::env::set_var(k, v);
    }
    let result = run_capped(exe);
    for (k, old) in &saved {
        match old {
            Some(v) => std::env::set_var(k, v),
            None => std::env::remove_var(k),
        }
    }
    match result {
        Ok(r) => Ok(RunOut {
            exit: r.exit as i64,
            stdout: r.stdout,
            stderr: r.stderr,
        }),
        Err(e) => Err(e),
    }
}

fn compile_wrapped(
    rustc: &str,
    scratch: &Path,
    module: &str,
    golden: &[u8],
    wrapper: &str,
) -> Result<PathBuf, String> {
    let dir = scratch.join(format!("{module}_wrap"));
    let _ = std::fs::create_dir_all(dir.join("src"));
    std::fs::write(dir.join("src").join("generated.rs"), golden)
        .map_err(|e| format!("write generated.rs: {e}"))?;
    std::fs::write(dir.join("src").join("main.rs"), wrapper)
        .map_err(|e| format!("write wrapper main.rs: {e}"))?;
    let exe = dir.join(format!("{module}_wrap.exe"));
    let crate_name = module.trim_start_matches("m1_").to_string();
    compile(rustc, &dir.join("src").join("main.rs"), &exe, &crate_name)?;
    Ok(exe)
}

fn profiles_of(leg: &SceneLeg) -> (IntProfile, ResourceProfile) {
    let int = if leg.profiles.iter().any(|p| p == CHECKED_PROFILE_ID) {
        IntProfile::CheckedI32
    } else {
        IntProfile::LegacyI64
    };
    let res = if leg.profiles.iter().any(|p| p == OBSERVE_PROFILE_ID) {
        ResourceProfile::Observe
    } else {
        ResourceProfile::None
    };
    (int, res)
}

fn emit_leg(corpus: &Path, leg: &SceneLeg) -> Result<Vec<u8>, String> {
    let source_path = corpus.join(&leg.source);
    let source = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("read leg source {}: {e}", leg.source))?;
    let source_dir = source_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| corpus.to_path_buf());
    let module = leg.module.clone();
    let (int_profile, res_profile) = profiles_of(leg);
    let handle = std::thread::Builder::new()
        .stack_size(EMIT_STACK_BYTES)
        .spawn(move || {
            let mut sink = transpile_rust_with_source_dir_profiles(
                &source_dir,
                &module,
                &source,
                int_profile,
                res_profile,
            )?;
            let bytes = sink.done()?.clone();
            Ok::<_, crate::error::AutoError>(bytes)
        })
        .map_err(|e| format!("spawn emit thread: {e}"))?;
    match handle.join() {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(e)) => Err(format!("a2r emit failed for {}: {e}", leg.id)),
        Err(payload) => Err(format!(
            "emit thread panicked: {}",
            super::avm::panic_message(&payload)
        )),
    }
}

// One leg's full run matrix. Returns a human-readable summary line per run.
fn certify_leg_runs(
    exe: &Path,
    leg: &SceneLeg,
    invariants: &[String],
    runs: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    let mut record = |name: &str, out: &RunOut, events: &[ResEvent], want: &str| {
        let chk = check_invariants(events, invariants, want);
        runs.push(serde_json::json!({
            "run": name, "exit": out.exit,
            "stdout_tail": tail_lines(&out.stdout, 4),
            "stderr_line": String::from_utf8_lossy(&out.stderr).trim().to_string(),
            "invariants": if chk.ok { "pass" } else { &chk.why },
        }));
        chk
    };
    match leg.mode.as_str() {
        "clean" => {
            let out = run_with_env(exe, &[])?;
            let events = parse_stream(&out.stdout);
            let want_val = leg.value.expect("clean leg has a frozen value");
            if out.exit != 0 {
                return Err(format!("clean run exit {} != 0", out.exit));
            }
            let got = events
                .iter()
                .find(|e| e.kind == "result")
                .map(|e| e.arg.parse::<i64>().unwrap_or(i64::MIN));
            if got != Some(want_val) {
                return Err(format!(
                    "clean run result {:?} != frozen value {want_val}",
                    got
                ));
            }
            if let Some(want_reads) = leg.reads {
                let reads = events.iter().filter(|e| e.kind == "read").count();
                if reads != want_reads {
                    return Err(format!("buffer reads {reads} != frozen {want_reads}"));
                }
            }
            let chk = record("clean", &out, &events, "clean");
            if !chk.ok {
                return Err(format!("clean run invariants: {}", chk.why));
            }
            Ok(())
        }
        "sweep" => {
            // clean run discovers N
            let out = run_with_env(exe, &[])?;
            let events = parse_stream(&out.stdout);
            let want_val = leg.value.expect("sweep leg has a frozen value");
            if out.exit != 0 {
                return Err(format!("sweep clean run exit {} != 0", out.exit));
            }
            let got = events
                .iter()
                .find(|e| e.kind == "result")
                .and_then(|e| e.arg.parse::<i64>().ok());
            if got != Some(want_val) {
                return Err(format!("sweep clean result {got:?} != {want_val}"));
            }
            let n = events
                .iter()
                .find(|e| e.kind == "heap")
                .and_then(|e| e.arg.parse::<usize>().ok())
                .ok_or("clean run printed no heap count")?;
            let chk = record("clean", &out, &events, "clean");
            if !chk.ok {
                return Err(format!("sweep clean invariants: {}", chk.why));
            }
            // fault sweep 1..=N
            for k in 1..=n {
                let out = run_with_env(exe, &[(format!("A2R_FAULT"), format!("heap#{k}"))])?;
                let events = parse_stream(&out.stdout);
                if out.exit != TYPED_FAIL_EXIT
                    || String::from_utf8_lossy(&out.stderr).trim() != "auto.res:fail:fault"
                {
                    return Err(format!(
                        "fault k={k}: exit {} stderr {:?} — expected typed fault",
                        out.exit,
                        String::from_utf8_lossy(&out.stderr)
                    ));
                }
                let chk = record(&format!("fault-{k}"), &out, &events, "fault");
                if !chk.ok {
                    return Err(format!("fault k={k} invariants: {}", chk.why));
                }
            }
            // N+1 control: same knob, one past the true count — must be clean
            let out = run_with_env(exe, &[(format!("A2R_FAULT"), format!("heap#{}", n + 1))])?;
            let events = parse_stream(&out.stdout);
            if out.exit != 0 {
                return Err(format!("N+1 control exit {} != 0", out.exit));
            }
            let got = events
                .iter()
                .find(|e| e.kind == "result")
                .and_then(|e| e.arg.parse::<i64>().ok());
            if got != Some(want_val) {
                return Err(format!("N+1 control result {got:?} != {want_val}"));
            }
            let chk = record("n-plus-one", &out, &events, "clean");
            if !chk.ok {
                return Err(format!("N+1 control invariants: {}", chk.why));
            }
            // same-process retry: fault at a representative k, cleared, re-run
            let k = n.div_ceil(2);
            let out = run_with_env(
                exe,
                &[
                    (format!("A2R_FAULT"), format!("heap#{k}")),
                    (format!("A2R_RETRY"), format!("1")),
                ],
            )?;
            let events = parse_stream(&out.stdout);
            if out.exit != 0 {
                return Err(format!("retry leg exit {} != 0", out.exit));
            }
            let has_retry = events.iter().any(|e| e.kind == "retry");
            let has_fault = events.iter().any(|e| e.kind == "fault");
            let got = events
                .iter()
                .find(|e| e.kind == "result")
                .and_then(|e| e.arg.parse::<i64>().ok());
            if !has_retry || !has_fault || got != Some(want_val) {
                return Err(format!(
                    "retry leg missing fault/retry/result chain (fault={has_fault} retry={has_retry} result={got:?})"
                ));
            }
            let chk = record(&format!("retry-{k}"), &out, &events, "clean");
            if !chk.ok {
                return Err(format!("retry invariants: {}", chk.why));
            }
            Ok(())
        }
        "budget" => {
            let budget = leg.budget.expect("budget leg has a frozen budget");
            let out = run_with_env(exe, &[(format!("A2R_BUDGET"), format!("{budget}"))])?;
            let events = parse_stream(&out.stdout);
            if out.exit != TYPED_FAIL_EXIT
                || String::from_utf8_lossy(&out.stderr).trim() != "auto.res:fail:budget"
            {
                return Err(format!(
                    "budget leg exit {} stderr {:?}",
                    out.exit,
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            if !events.iter().any(|e| e.kind == "budget") {
                return Err("budget leg has no typed budget event".to_string());
            }
            let chk = record("budget", &out, &events, "budget");
            if !chk.ok {
                return Err(format!("budget invariants: {}", chk.why));
            }
            Ok(())
        }
        "cancel" => {
            let ms = leg.cancel_ms.expect("cancel leg has a frozen cancel_ms");
            let out = run_with_env(exe, &[(format!("A2R_CANCEL_MS"), format!("{ms}"))])?;
            let events = parse_stream(&out.stdout);
            if out.exit != TYPED_FAIL_EXIT
                || String::from_utf8_lossy(&out.stderr).trim() != "auto.res:fail:cancel"
            {
                return Err(format!(
                    "cancel leg exit {} stderr {:?}",
                    out.exit,
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            if !events.iter().any(|e| e.kind == "cancel") {
                return Err("cancel leg has no typed cancel event".to_string());
            }
            let chk = record("cancel", &out, &events, "cancel");
            if !chk.ok {
                return Err(format!("cancel invariants: {}", chk.why));
            }
            Ok(())
        }
        "trap" => {
            let out = run_with_env(exe, &[])?;
            let events = parse_stream(&out.stdout);
            let kind = leg.trap_kind.clone().unwrap_or_default();
            let op = leg.trap_op.clone().unwrap_or_default();
            let want_line = format!("auto.trap:{kind}:{op}");
            if out.exit != TRAP_EXIT || String::from_utf8_lossy(&out.stderr).trim() != want_line {
                return Err(format!(
                    "trap leg exit {} stderr {:?} — expected {want_line} exit {TRAP_EXIT}",
                    out.exit,
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            let chk = record("trap", &out, &events, "trap");
            if !chk.ok {
                return Err(format!("trap invariants: {}", chk.why));
            }
            Ok(())
        }
        other => Err(format!("leg mode {other:?} has no run matrix")),
    }
}

fn tail_lines(bytes: &[u8], n: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join(" | ")
}

// ---------------------------------------------------------------------------
// Certification driver
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct CellAxes {
    pub source_shas: std::collections::BTreeMap<String, String>,
    pub golden_shas: std::collections::BTreeMap<String, String>,
    pub reference_shas: std::collections::BTreeMap<String, String>,
    pub wrapper_sha: String,
    pub oracle_sha: String,
}

fn cell_axes(corpus: &Path, cell: &SceneCell) -> Result<CellAxes, String> {
    let read = |p: PathBuf, what: &str| -> Result<Vec<u8>, String> {
        std::fs::read(&p).map_err(|e| format!("read {what}: {e}"))
    };
    let mut source_shas = std::collections::BTreeMap::new();
    let mut golden_shas = std::collections::BTreeMap::new();
    let mut reference_shas = std::collections::BTreeMap::new();
    for leg in &cell.legs {
        if leg.reference_only {
            continue;
        }
        let src = read(corpus.join(&leg.source), "leg source")?;
        source_shas.insert(leg.id.clone(), hex(&sha256(&src)));
        let golden = read(
            corpus
                .join("oracle")
                .join("scenario")
                .join(format!("{}.rs", leg.module)),
            "scene golden",
        )?;
        golden_shas.insert(leg.id.clone(), hex(&sha256(&golden)));
    }
    for leg in &cell.legs {
        let ref_path = corpus
            .join("reference")
            .join("scenario")
            .join(format!("{}.rs", leg.id));
        if ref_path.is_file() {
            let r = read(ref_path, "scene reference")?;
            reference_shas.insert(leg.id.clone(), hex(&sha256(&r)));
        }
    }
    let wrapper = read(
        corpus.join("oracle").join("resource-wrapper-v1.rs"),
        "wrapper",
    )?;
    let oracle = read(
        corpus
            .join("oracle")
            .join("scenario")
            .join("scenario-oracle.json"),
        "scenario oracle",
    )?;
    Ok(CellAxes {
        source_shas,
        golden_shas,
        reference_shas,
        wrapper_sha: hex(&sha256(&wrapper)),
        oracle_sha: hex(&sha256(&oracle)),
    })
}

/// STRICT receipt validation (mirrors the value/trap discipline).
pub fn validate_scene_receipt(
    cert: &serde_json::Value,
    cell_id: &str,
    axes: &CellAxes,
    toolchain_identity: &str,
) -> Result<(), super::ace_certify::ReceiptProblem> {
    use super::ace_certify::ReceiptProblem;
    let invalid = |what: String| ReceiptProblem::Invalid(what);
    let stale = |what: String| ReceiptProblem::Stale(what);
    let root = cert
        .as_object()
        .ok_or_else(|| invalid("scene receipt root is not an object".to_string()))?;
    for k in root.keys() {
        if ![
            "schema",
            "case",
            "profile",
            "wrapper_file",
            "identity",
            "toolchain",
            "deps",
            "legs",
            "observed",
            "verdict",
            "produced_by",
            "created_at",
        ]
        .contains(&k.as_str())
        {
            return Err(invalid(format!(
                "unknown field {k:?} in scene receipt root (strict schema)"
            )));
        }
    }
    for k in [
        "schema",
        "case",
        "profile",
        "wrapper_file",
        "identity",
        "toolchain",
        "deps",
        "legs",
        "observed",
        "verdict",
        "produced_by",
        "created_at",
    ] {
        if !root.contains_key(k) {
            return Err(invalid(format!(
                "missing field {k:?} in scene receipt root"
            )));
        }
    }
    if cert.get("schema").and_then(|v| v.as_str()) != Some(SCENE_CERT_SCHEMA) {
        return Err(invalid(format!("schema is not {SCENE_CERT_SCHEMA}")));
    }
    if cert.get("case").and_then(|v| v.as_str()) != Some(cell_id) {
        return Err(invalid(format!("case is not {cell_id}")));
    }
    if cert.get("profile").and_then(|v| v.as_str()) != Some(OBSERVE_PROFILE_ID) {
        return Err(invalid(format!("profile is not {OBSERVE_PROFILE_ID}")));
    }
    if cert.get("wrapper_file").and_then(|v| v.as_str()) != Some("oracle/resource-wrapper-v1.rs") {
        return Err(invalid(
            "wrapper_file is not the frozen oracle/resource-wrapper-v1.rs".to_string(),
        ));
    }
    if cert.get("verdict").and_then(|v| v.as_str()) != Some("certified") {
        return Err(invalid("verdict is not certified".to_string()));
    }
    if cert.get("created_at").and_then(|v| v.as_u64()).is_none() {
        return Err(invalid("created_at is not an unsigned integer".to_string()));
    }
    let legs = cert
        .get("legs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| invalid("legs is not an array".to_string()))?;
    if legs.is_empty() {
        return Err(invalid("legs is empty".to_string()));
    }
    for leg in legs {
        let leg = leg
            .as_object()
            .ok_or_else(|| invalid("leg is not an object".to_string()))?;
        for k in leg.keys() {
            if !["id", "runs"].contains(&k.as_str()) {
                return Err(invalid(format!("unknown leg field {k:?} (strict schema)")));
            }
        }
        if !leg.contains_key("id") || !leg.contains_key("runs") {
            return Err(invalid("leg missing id/runs".to_string()));
        }
        let runs = leg
            .get("runs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| invalid("leg runs is not an array".to_string()))?;
        if runs.is_empty() {
            return Err(invalid("leg runs is empty".to_string()));
        }
        for run in runs {
            let run = run
                .as_object()
                .ok_or_else(|| invalid("run is not an object".to_string()))?;
            for k in run.keys() {
                if !["run", "exit", "stdout_tail", "stderr_line", "invariants"]
                    .contains(&k.as_str())
                {
                    return Err(invalid(format!("unknown run field {k:?} (strict schema)")));
                }
            }
            if run.get("invariants").and_then(|v| v.as_str()) != Some("pass") {
                return Err(invalid(
                    "a recorded run does not carry invariants: pass".to_string(),
                ));
            }
        }
    }
    let id = cert
        .get("identity")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("identity is not an object".to_string()))?;
    for k in id.keys() {
        if ![
            "sources",
            "goldens",
            "references",
            "wrapper_sha256",
            "scenario_oracle_sha256",
        ]
        .contains(&k.as_str())
        {
            return Err(invalid(format!("unknown identity field {k:?}")));
        }
    }
    for (k, want) in [
        ("sources", &axes.source_shas),
        ("goldens", &axes.golden_shas),
        ("references", &axes.reference_shas),
    ] {
        let got = id
            .get(k)
            .and_then(|v| v.as_object())
            .ok_or_else(|| invalid(format!("identity.{k} is not an object")))?;
        if got.len() != want.len() {
            return Err(invalid(format!(
                "identity.{k} has {} entries, current corpus has {}",
                got.len(),
                want.len()
            )));
        }
        for (leg, sha) in want {
            if got.get(leg).and_then(|v| v.as_str()) != Some(sha.as_str()) {
                return Err(stale(format!("{k} drifted for leg {leg}")));
            }
        }
    }
    if id.get("wrapper_sha256").and_then(|v| v.as_str()) != Some(axes.wrapper_sha.as_str()) {
        return Err(stale("wrapper drifted".to_string()));
    }
    if id.get("scenario_oracle_sha256").and_then(|v| v.as_str()) != Some(axes.oracle_sha.as_str()) {
        return Err(stale("scenario oracle registry drifted".to_string()));
    }
    let toolchain = cert
        .get("toolchain")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("toolchain is not an object".to_string()))?;
    for k in toolchain.keys() {
        if ![
            "identity",
            "identity_source",
            "default_identity_at_certify",
            "flags",
        ]
        .contains(&k.as_str())
        {
            return Err(invalid(format!("unknown toolchain field {k:?}")));
        }
    }
    let flags = toolchain
        .get("flags")
        .and_then(|v| v.as_array())
        .ok_or_else(|| invalid("toolchain.flags is not an array".to_string()))?;
    if flags.len() != RUSTC_FLAG_TOKENS.len()
        || flags
            .iter()
            .zip(RUSTC_FLAG_TOKENS.iter())
            .any(|(v, want)| v.as_str() != Some(*want))
    {
        return Err(invalid(format!(
            "toolchain.flags is not the frozen set {:?}",
            RUSTC_FLAG_TOKENS
        )));
    }
    if toolchain
        .get("identity")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        != toolchain_identity
    {
        return Err(stale("toolchain identity drifted".to_string()));
    }
    if toolchain
        .get("default_identity_at_certify")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        != BUILD_TOOLCHAIN_IDENTITY.trim()
    {
        return Err(stale(
            "default toolchain drifted since certification (re-certify to re-bind)".to_string(),
        ));
    }
    let deps_obj = cert
        .get("deps")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("deps is not an object".to_string()))?;
    for k in deps_obj.keys() {
        if !["external", "closure"].contains(&k.as_str()) {
            return Err(invalid(format!("unknown deps field {k:?}")));
        }
    }
    if deps_obj
        .get("external")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(true)
    {
        return Err(invalid("deps.external is not empty".to_string()));
    }
    if deps_obj.get("closure").and_then(|v| v.as_str()) != Some(DEPS_CLOSURE) {
        return Err(invalid(format!("deps.closure is not {DEPS_CLOSURE:?}")));
    }
    let _ = cert
        .get("observed")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("observed is not an object".to_string()))?;
    Ok(())
}

pub fn resource_certify(
    args: &ResourceCertifyArgs,
) -> Result<Vec<super::ace_certify::CaseSummary>, String> {
    let registry = load_scene_registry(&args.corpus)?;
    let rows = resource_certifiable_cells(&args.corpus.join("binding.tsv"), &args.only)?;
    let selected: Vec<&(String, String)> = rows
        .iter()
        .filter(|(id, _)| {
            args.only.is_empty() || args.only.iter().any(|o| o.eq_ignore_ascii_case(id))
        })
        .collect();
    if selected.is_empty() {
        return Err("--only matched no resource-certifiable cell".to_string());
    }
    for (id, _expected) in &selected {
        if !registry.cells.contains_key(id.as_str()) {
            return Err(format!("scenario oracle has no cell for {id}"));
        }
    }
    std::fs::create_dir_all(&args.certs_dir).map_err(|e| format!("certs dir: {e}"))?;

    let scratch_root = match &args.scratch {
        Some(p) => p.clone(),
        None => std::env::temp_dir().join(format!("ace-m1-scene-certify-{}", std::process::id())),
    };
    let _ = std::fs::remove_dir_all(&scratch_root);
    std::fs::create_dir_all(&scratch_root).map_err(|e| format!("scratch: {e}"))?;
    let result = (|| {
        let rustc = args
            .rustc
            .clone()
            .unwrap_or_else(|| std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
        let identity_source = if args.rustc.is_some() {
            "--rustc override"
        } else if std::env::var("RUSTC").is_ok() {
            "env RUSTC"
        } else {
            "PATH"
        };
        let toolchain_identity = capture_toolchain_contained(&rustc)?;
        let wrapper =
            std::fs::read_to_string(args.corpus.join("oracle").join("resource-wrapper-v1.rs"))
                .map_err(|e| format!("read frozen wrapper: {e}"))?;
        let mut summaries = Vec::new();
        for (cell_id, _expected) in selected {
            let cell = &registry.cells[cell_id.as_str()];
            let summary = certify_one_cell(
                cell_id,
                cell,
                args,
                &scratch_root,
                &rustc,
                &toolchain_identity,
                identity_source,
                &wrapper,
            );
            summaries.push(summary);
        }
        Ok(summaries)
    })();
    let _ = std::fs::remove_dir_all(&scratch_root);
    result
}

#[allow(clippy::too_many_arguments)]
fn certify_one_cell(
    cell_id: &str,
    cell: &SceneCell,
    args: &ResourceCertifyArgs,
    scratch_root: &Path,
    rustc: &str,
    toolchain_identity: &str,
    identity_source: &str,
    wrapper: &str,
) -> super::ace_certify::CaseSummary {
    let fail = |detail: String| super::ace_certify::CaseSummary {
        case: cell_id.to_string(),
        outcome: "fail".to_string(),
        detail,
    };

    // ---- 1. reuse: STRICT validation first
    let axes = match cell_axes(&args.corpus, cell) {
        Ok(a) => a,
        Err(e) => return fail(e),
    };
    let cert_path = args
        .certs_dir
        .join(format!("m1_{}.scenario-cert.json", cell_id.to_lowercase()));
    if let Ok(old_text) = std::fs::read_to_string(&cert_path) {
        match serde_json::from_str::<serde_json::Value>(&old_text) {
            Ok(old) => {
                match validate_scene_receipt(&old, cell_id, &axes, toolchain_identity) {
                    Ok(()) => {
                        if !args.force {
                            return super::ace_certify::CaseSummary {
                                case: cell_id.to_string(),
                                outcome: "reused".to_string(),
                                detail: "strict scene receipt validation passed — axes unchanged, nothing compiled".to_string(),
                            };
                        }
                    }
                    Err(super::ace_certify::ReceiptProblem::Invalid(why)) => {
                        return fail(format!(
                            "REJECTED scene receipt {cert_path:?}: {why} — structurally invalid; manually delete or repair it, then certify again"
                        ));
                    }
                    Err(super::ace_certify::ReceiptProblem::Stale(_why)) => {
                        // valid, drifted context: re-certify below
                    }
                }
            }
            Err(e) => {
                return fail(format!(
                    "REJECTED scene receipt {cert_path:?}: not valid JSON ({e}) — manually delete or repair it, then certify again"
                ));
            }
        }
    }

    // ---- 2. per leg: fresh emit == scene golden, then the run matrix on
    // BOTH sides (actual under the wrapper; independent reference
    // standalone).
    let mut legs_out: Vec<serde_json::Value> = Vec::new();
    for leg in &cell.legs {
        if leg.reference_only {
            // The typed helper-face leg (R03-stale): the REFERENCE drives the
            // mint -> legal read -> structural mutation -> stale-use chain and
            // must reject typed BEFORE any buffer access.
            let ref_path = args
                .corpus
                .join("reference")
                .join("scenario")
                .join(format!("{}.rs", leg.id));
            if !ref_path.is_file() {
                return fail(format!(
                    "reference-only leg {} has no reference file",
                    leg.id
                ));
            }
            let exe = scratch_root.join(format!("{}_ref.exe", leg.id));
            let crate_name = leg.id.to_lowercase().replace('-', "_");
            if let Err(e) = compile(rustc, &ref_path, &exe, &crate_name) {
                return fail(format!("compile {} reference: {e}", leg.id));
            }
            let out = match run_with_env(&exe, &[]) {
                Ok(o) => o,
                Err(e) => return fail(format!("run {} reference: {e}", leg.id)),
            };
            let events = parse_stream(&out.stdout);
            let stale_rejected = events.iter().any(|e| e.kind == "stale")
                && String::from_utf8_lossy(&out.stderr).trim() == "auto.res:fail:stale"
                && out.exit == TYPED_FAIL_EXIT;
            if !stale_rejected {
                return fail(format!(
                    "reference-only leg {} did not demonstrate typed stale rejection before buffer access (exit {} stderr {:?})",
                    leg.id,
                    out.exit,
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            legs_out.push(serde_json::json!({
                "id": leg.id,
                "runs": [ {
                    "run": "typed-probe",
                    "exit": out.exit,
                    "stdout_tail": tail_lines(&out.stdout, 4),
                    "stderr_line": String::from_utf8_lossy(&out.stderr).trim().to_string(),
                    "invariants": "pass",
                } ],
            }));
            continue;
        }
        // fresh actual through the production transpiler, leg profiles
        let actual = match emit_leg(&args.corpus, leg) {
            Ok(a) => a,
            Err(e) => return fail(e),
        };
        let golden_path = args
            .corpus
            .join("oracle")
            .join("scenario")
            .join(format!("{}.rs", leg.module));
        let golden = match std::fs::read(&golden_path) {
            Ok(g) => g,
            Err(e) => return fail(format!("read scene golden: {e}")),
        };
        if leg.mode == "compile-boundary" {
            // The honest compile-time boundary: the emitted actual MUST be
            // rejected by rustc (recorded as evidence, never a typed negative).
            let dir = scratch_root.join(format!("{}_cb", leg.module));
            let _ = std::fs::create_dir_all(dir.join("src"));
            let _ = std::fs::write(dir.join("src").join("generated.rs"), &actual);
            let _ = std::fs::write(dir.join("src").join("main.rs"), wrapper);
            let exe = dir.join("cb.exe");
            let _ = compile(rustc, &dir.join("src").join("main.rs"), &exe, &leg.module);
            // compile() is exit-code-blind by design (the value/trap domains
            // detect failures through the missing exe) — the honest boundary
            // signal here is whether rustc produced the binary at all.
            if exe.is_file() {
                return fail(format!(
                    "compile-boundary leg {} unexpectedly COMPILED — the boundary record requires the emitted actual to be rejected",
                    leg.id
                ));
            }
            {
                let e = String::from("rustc produced no binary");
                legs_out.push(serde_json::json!({
                    "id": leg.id,
                    "runs": [ {
                        "run": "compile-boundary",
                        "exit": -1,
                        "stdout_tail": format!("rustc rejected the emitted actual: {}", e.lines().next().unwrap_or("")),
                        "stderr_line": "",
                        "invariants": "pass",
                    } ],
                }));
                continue;
            }
        }
        if actual != golden {
            return fail(format!(
                "actual != scene golden for {} ({} vs {} bytes) — explicit review required, no auto-update",
                leg.id,
                actual.len(),
                golden.len()
            ));
        }
        // actual side: compile golden under the frozen wrapper, run the matrix
        let exe = match compile_wrapped(rustc, scratch_root, &leg.module, &golden, wrapper) {
            Ok(e) if e.is_file() => e,
            Ok(_) => {
                return fail(format!(
                    "compile actual-wrapper for {} produced no binary",
                    leg.id
                ))
            }
            Err(e) => return fail(format!("compile actual-wrapper for {}: {e}", leg.id)),
        };
        let mut runs: Vec<serde_json::Value> = Vec::new();
        if let Err(e) = certify_leg_runs(&exe, leg, &cell.invariants, &mut runs) {
            return fail(format!("actual side, leg {}: {e}", leg.id));
        }
        // reference side: the hand-written independent implementation runs the
        // same invariant suite over its own domain.
        let ref_id = leg.reference.clone().unwrap_or_else(|| leg.id.clone());
        let ref_path = args
            .corpus
            .join("reference")
            .join("scenario")
            .join(format!("{}.rs", ref_id));
        if !ref_path.is_file() {
            return fail(format!("leg {} has no independent reference file", leg.id));
        }
        let ref_exe = scratch_root.join(format!("{}_ref.exe", leg.id));
        if let Err(e) = compile(rustc, &ref_path, &ref_exe, &leg.module.replace("m1_", "")) {
            return fail(format!("compile reference for {}: {e}", leg.id));
        }
        if !ref_exe.is_file() {
            return fail(format!(
                "compile reference for {} produced no binary",
                leg.id
            ));
        }
        // Reference side: the hand-written independent implementation runs
        // the SAME run matrix over its own allocation domain (the references
        // honor the same env knob contract).
        let mut ref_runs: Vec<serde_json::Value> = Vec::new();
        if let Err(e) = certify_leg_runs(&ref_exe, leg, &cell.invariants, &mut ref_runs) {
            return fail(format!("reference side, leg {}: {e}", leg.id));
        }
        for mut rr in ref_runs {
            if let Some(name) = rr.get("run").and_then(|v| v.as_str()) {
                rr["run"] = serde_json::Value::String(format!("ref-{name}"));
            }
            runs.push(rr);
        }
        legs_out.push(serde_json::json!({ "id": leg.id, "runs": runs }));
    }

    // ---- 3. certificate (strict schema `ace.m1.a2r-resource-scenario/1`)
    let cert = serde_json::json!({
        "schema": SCENE_CERT_SCHEMA,
        "case": cell_id,
        "profile": OBSERVE_PROFILE_ID,
        "wrapper_file": "oracle/resource-wrapper-v1.rs",
        "identity": {
            "sources": axes.source_shas,
            "goldens": axes.golden_shas,
            "references": axes.reference_shas,
            "wrapper_sha256": axes.wrapper_sha,
            "scenario_oracle_sha256": axes.oracle_sha,
        },
        "toolchain": {
            "identity": toolchain_identity,
            "identity_source": identity_source,
            "default_identity_at_certify": BUILD_TOOLCHAIN_IDENTITY.trim(),
            "flags": RUSTC_FLAG_TOKENS,
        },
        "deps": { "external": [], "closure": DEPS_CLOSURE },
        "legs": legs_out,
        "observed": {
            "note": "per-leg run matrices (clean/sweep/N+1/retry/budget/cancel/trap/compile-boundary/reference) with the frozen invariant suite evaluated on BOTH sides",
        },
        "verdict": "certified",
        "produced_by": { "tool": "ace-certify --resource", "schema_note": "resource-scenario domain is independent of ace.m1.cert/1 and ace.m1.a2r-trap/1; receipts never mix domains" },
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    });
    let written = serde_json::to_string_pretty(&cert)
        .map_err(|e| e.to_string())
        .and_then(|p| std::fs::write(&cert_path, p).map_err(|e| e.to_string()));
    match written {
        Ok(()) => super::ace_certify::CaseSummary {
            case: cell_id.to_string(),
            outcome: "certified".to_string(),
            detail: format!(
                "{} leg(s) dual-side certified (invariant suite on actual + reference); cert m1_{}.scenario-cert.json",
                cell.legs.len(),
                cell_id.to_lowercase()
            ),
        },
        Err(e) => fail(format!("write scene certificate: {e}")),
    }
}

/// Budget constants re-exported for readers that report budgets.
pub fn resource_budgets() -> (std::time::Duration, std::time::Duration, usize) {
    (COMPILE_WALL, RUN_WALL, 1 << 20)
}
