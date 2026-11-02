//! PLAN-913 T-02/T-05: the source-bound RECOVERY chain end to end.
//!
//! Real production in the parent (912 function session, store-backed) →
//! publish an atomically rooted source-origin manifest (units + declaration
//! context + spanless identity wires) → a DIFFERENT process restores the
//! accepted state from the caller's expected root id + digest, deep-verifies
//! every unit, installs, links and executes — with ZERO source-production
//! stages (P axis) and real decode/verify work (R axis) — and then takes ONE
//! more single-lambda body edit through the restored declaration context.
//!
//! Oracle values are hand-computed from the source and live only in the
//! parent's assertions; the request carries no expected value.

use std::path::{Path, PathBuf};

use ace_av::closures_fn_session::ClosuresFunctionSession;
use ace_av::closures_source_recovery::{
    publish_source_recovery, restore_source_recovery, restore_source_session, PublicationRequest,
    RecoveryUnitInput, RestoreRequest, RECOVERY_SCHEMA,
};

/// The 909/912 shared K/S corpus (value oracle 51: f writes the shared cell).
const SRC: &str = "\
fn helper(n int) int { return n + 1 }
fn other() int { return 7 }
fn main() int {
    var x int = 10
    var base int = 1
    var f = (n int) => { base = base + n; return base }
    var g = (n int) => { return x - n }
    return f(20) + f(0) + g(3) + helper(1)
}
";

/// `g`'s body doubled: f(20)=21, f(0)=21, g(3)=10-6=4, helper(1)=2 → 48.
fn edited_source() -> String {
    SRC.replace("return x - n }", "return x - n * 2 }")
}

fn temp_store(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "av913-recovery-{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("store dir");
    dir
}

fn write_source(dir: &Path, name: &str, src: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, src).expect("write source");
    path
}

struct Probe {
    ok: bool,
    line: String,
    json: serde_json::Value,
    code: Option<i32>,
}

fn run_probe(args: &[&str]) -> Probe {
    let exe = env!("CARGO_BIN_EXE_av_closures_913_restore");
    let out = std::process::Command::new(exe)
        .args(args)
        .output()
        .expect("spawn restore probe");
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let json: serde_json::Value = serde_json::from_str(
        stdout
            .lines()
            .last()
            .unwrap_or("{\"ok\":false,\"code\":\"no-output\"}"),
    )
    .unwrap_or_else(|e| panic!("probe output not JSON ({e}): {stdout}"));
    Probe {
        ok: out.status.success(),
        line: stdout,
        code: out.status.code(),
        json,
    }
}

struct Published {
    store: PathBuf,
    source: PathBuf,
    manifest: String,
    digest: String,
    units: usize,
    source_sha256: String,
    /// `(unit wire identity, artifact id)` for every published unit.
    unit_artifacts: Vec<(String, String)>,
}

/// The parent-side cold production + publication used by every case.
fn publish(tag: &str) -> Published {
    let store = temp_store(tag);
    let src_path = write_source(&store, "main.at", SRC);
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    let cold = session.apply_source(SRC).expect("cold production");
    assert_eq!(cold.value, 51, "hand-computed oracle");
    assert_eq!(cold.submitted_units, 5, "3 functions + 2 lambdas");
    let ticket = session
        .publish_source_recovery("ac913-recovery")
        .expect("publish source recovery");
    let state = session.accepted_state().expect("accepted state");
    let source_sha256 = state.source_sha256.clone();
    let unit_artifacts: Vec<(String, String)> = state
        .units
        .iter()
        .map(|u| {
            (
                u.unit.clone(),
                u.store_ref.clone().expect("persisted unit artifact"),
            )
        })
        .collect();
    assert_eq!(ticket.source_sha256.len(), 64);
    assert_eq!(ticket.units, 5);
    assert!(ticket.manifest_bytes > 0 && ticket.manifest_bytes <= 4 * 1024 * 1024);
    assert_eq!(ticket.manifest_digest.len(), 64);
    // The published root is visible under the project's recovery kind and
    // points at the ticket's manifest.
    let store_handle = ace_artifacts::store::ManagedStore::open(&store).expect("open store");
    let roots = store_handle
        .root_artifacts("ac913-recovery", "source-recovery-current")
        .expect("read roots");
    assert!(roots.contains(&ticket.manifest_artifact), "roots: {roots:?}");
    assert_eq!(roots.len(), 1, "one current root per project");
    let artifact = store_handle
        .get_artifact_by_id(&ticket.manifest_artifact)
        .expect("read manifest")
        .expect("manifest present");
    assert!(
        artifact.payload.len() >= ticket.manifest_bytes,
        "framed manifest {} < payload {}",
        artifact.payload.len(),
        ticket.manifest_bytes
    );
    session.close().expect("close");
    Published {
        store,
        source: src_path,
        manifest: ticket.manifest_artifact,
        digest: ticket.manifest_digest,
        units: ticket.units,
        source_sha256,
        unit_artifacts,
    }
}

#[test]
fn av_closures_recovery_913_fresh_process_restore_zero_production() {
    let p = publish("fresh");
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(
        probe.ok,
        "restore failed: {} (exit {:?})",
        probe.line, probe.code
    );
    let json = &probe.json;
    assert_eq!(json["ok"], true);
    assert_eq!(json["restore_value"], 51, "restored execution oracle");
    assert_ne!(
        json["pid"].as_u64().unwrap(),
        std::process::id() as u64,
        "the restore must run in a DIFFERENT process"
    );
    assert_eq!(json["units"], p.units);
    // The positive path takes ZERO source-production stages...
    assert_eq!(json["p_axis_attempts"], 0);
    // ...and REAL deep verification work, including the declaration context.
    assert_eq!(json["r_manifest_reads"], 1);
    assert_eq!(json["r_unit_reads"], p.units);
    assert_eq!(json["r_deep_verifies"], p.units);
    assert_eq!(json["r_admits"], p.units);
    assert_eq!(json["r_context_reads"], 1);
    assert!(json["wire_bytes"].as_u64().unwrap() > 0);
}

#[test]
fn av_closures_recovery_913_restore_then_single_lambda_edit() {
    let p = publish("edit-after-restore");
    let edited = edited_source();
    let edit_path = write_source(&p.store, "edited.at", &edited);
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    let edit_arg = edit_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
        "",
        &edit_arg,
    ]);
    assert!(
        probe.ok,
        "restore+edit failed: {} (exit {:?})",
        probe.line, probe.code
    );
    let base = &probe.json["base"];
    let edit = &probe.json["edit"];
    // The restore itself produced nothing.
    assert_eq!(base["restore_value"], 51);
    assert_eq!(base["p_axis_attempts"], 0);
    assert_eq!(base["r_context_reads"], 1);
    // The NEXT edit is the 912 warm single-unit route on the restored context.
    assert_eq!(edit["value"], 48, "hand-computed edited oracle");
    assert_eq!(edit["route"], "fast-unit");
    assert_eq!(edit["submitted"], 1);
    assert_eq!(
        edit["submitted_names"],
        serde_json::json!(["d___closure_main_1"])
    );
    assert_eq!(edit["whole_total"], 0, "no whole-module fallback");
    assert_eq!(edit["warm_gate_pass"], true, "gate: {:?}", edit["warm_gate_violations"]);
    // Only the edited unit did body work / production.
    let body = edit["body"].as_object().expect("body map");
    assert_eq!(body.len(), 1, "body work: {body:?}");
    assert!(body.contains_key("__closure_main_1"));
    assert_eq!(edit["unchanged_production_attempts"], 0);
    assert!(edit["stages"]["mir_lower"].as_u64().unwrap() >= 1);
    assert!(edit["stages"]["abc_encode"].as_u64().unwrap() >= 1);
    // Every unchanged unit kept its OBJECT and its generation; the edited one
    // is a new object.
    assert_eq!(edit["unchanged_total"], p.units - 1);
    assert_eq!(edit["unchanged_same_object"], p.units - 1);
    assert_eq!(edit["changed_total"], 1);
    assert_eq!(edit["changed_new_object"], true);
    // The recovery session does not persist: the edit writes nothing.
    assert_eq!(edit["store_writes"], 0);
    assert_eq!(edit["live_after_release"], 0);
    assert_eq!(edit["allocs"], edit["frees"]);
}

#[test]
fn av_closures_recovery_913_wrong_digest_and_missing_root_are_named_rejections() {
    let p = publish("negative");
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    // A wrong parent digest: the store's self-description never overrides
    // the caller's expectation.
    let forged = "0".repeat(64);
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &forged,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok);
    assert!(
        probe.json["code"]
            .as_str()
            .unwrap_or("")
            .starts_with("recovery."),
        "unexpected rejection: {}",
        probe.line
    );
    // An absent root id is a locatable MISS with no cold fallback.
    let probe = run_probe(&[
        &store_arg,
        "art-does-not-exist",
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok);
    assert_eq!(probe.json["code"], "recovery.miss");
    assert_ne!(probe.json["pid"].as_u64().unwrap(), 0, "the probe still ran");
}

#[test]
fn av_closures_recovery_913_manifest_binds_source_and_config() {
    let p = publish("binding");
    let store_arg = p.store.to_string_lossy().to_string();
    let other_src = write_source(&p.store, "other.at", &SRC.replace("n + 1", "n + 2"));
    let other_arg = other_src.to_string_lossy().to_string();
    // Same manifest, DIFFERENT source text: the source SHA binding rejects.
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &other_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(
        !probe.ok,
        "a manifest must not restore a different source: {}",
        probe.line
    );
    // Different logical owner (a cross-domain context) is rejected too.
    let source_arg = p.source.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "elsewhere.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok, "a foreign owner must not restore: {}", probe.line);
}

#[test]
fn av_closures_recovery_913_publish_republishes_root_atomically() {
    // Two accepted states in one project: the second publication replaces the
    // current root; exactly one root remains and it is the newer manifest.
    let store = temp_store("republish");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    session.apply_source(SRC).expect("cold");
    let first = session.publish_source_recovery("ac913-repub").expect("first");
    let edited = edited_source();
    let warm = session.apply_source(&edited).expect("warm");
    assert_eq!(warm.value, 48);
    let second = session
        .publish_source_recovery("ac913-repub")
        .expect("second");
    assert_ne!(
        first.manifest_artifact, second.manifest_artifact,
        "the edited accepted state is a different manifest"
    );
    let handle = ace_artifacts::store::ManagedStore::open(&store).expect("store");
    let roots = handle
        .root_artifacts("ac913-repub", "source-recovery-current")
        .expect("roots");
    assert_eq!(
        roots,
        vec![second.manifest_artifact.clone()],
        "one current root"
    );
    // The newer manifest restores the edited source's oracle.
    let store_arg = store.to_string_lossy().to_string();
    let edited_path = write_source(&store, "edited.at", &edited);
    let edited_arg = edited_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &second.manifest_artifact,
        &second.manifest_digest,
        &edited_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "republished restore failed: {}", probe.line);
    assert_eq!(probe.json["restore_value"], 48);
    // The superseded manifest still restores its OWN (older) source — the
    // new root never rewrites the old binding.
    let base_path = write_source(&store, "base.at", SRC);
    let base_arg = base_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &first.manifest_artifact,
        &first.manifest_digest,
        &base_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "old manifest restore failed: {}", probe.line);
    assert_eq!(probe.json["restore_value"], 51);
    session.close().expect("close");
}

#[test]
fn av_closures_recovery_913_publish_requires_real_artifacts() {
    // Publication is a decision over REAL persisted units: a manifest for an
    // artifact the store never wrote is rejected (no invented addresses).
    let store = temp_store("unaddressed");
    let result = publish_source_recovery(
        &store,
        PublicationRequest {
            project: "ac913-check",
            source_sha256: &"a".repeat(64),
            source_len: SRC.len(),
            logical_owner: "main.at",
            owner_module: "m_src",
            owner_schema: "auto.mir.av-closures/1",
            entry: "d_main",
            profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
            capabilities: &[],
            accepted_generation: 0,
            commit_point: 0,
            context_revision: "ctx",
            accepted_state_schema: RECOVERY_SCHEMA,
            context_payload: b"{}",
            context_schema: "ace.closures.context-dump/1",
            units: vec![RecoveryUnitInput {
                unit: "d_main".into(),
                logical: "owner".into(),
                body_digest: "b".into(),
                dependency_digest: "c".into(),
                env_layout_identity: None,
                mir_identity: "d".into(),
                body_fp: "e".into(),
                wire_bytes: 4,
                artifact: String::new(),
                mint_span: (0, 1),
                current_span: (0, 1),
                identity_wire: vec![1, 2, 3, 4],
            }],
        },
    );
    let err = result.expect_err("an unaddressed unit must not publish");
    assert_eq!(err.code, "recovery.unit-unaddressed");
}

#[test]
fn av_closures_recovery_913_failed_publication_keeps_previous_root() {
    // A failed publication must leave the PREVIOUS root valid and restorable:
    // the new root is registered only after every required artifact exists,
    // and the superseded root is retired last.
    let p = publish("failed-publish");
    let handle = ace_artifacts::store::ManagedStore::open(&p.store).expect("store");
    let before = handle
        .root_artifacts("ac913-recovery", "source-recovery-current")
        .expect("roots");
    assert_eq!(before, vec![p.manifest.clone()]);
    drop(handle);
    // A publication attempt that cannot complete (no declaration context).
    let failed = publish_source_recovery(
        &p.store,
        PublicationRequest {
            project: "ac913-recovery",
            source_sha256: &"a".repeat(64),
            source_len: SRC.len(),
            logical_owner: "main.at",
            owner_module: "m_src",
            owner_schema: "auto.mir.av-closures/1",
            entry: "d_main",
            profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
            capabilities: &[],
            accepted_generation: 1,
            commit_point: 1,
            context_revision: "ctx",
            accepted_state_schema: RECOVERY_SCHEMA,
            context_payload: b"",
            context_schema: "ace.closures.context-dump/1",
            units: Vec::new(),
        },
    );
    assert_eq!(
        failed
            .expect_err("an incomplete publication must fail")
            .code,
        "recovery.unit-count",
        "the first publication guard a context-less, unit-less attempt trips"
    );
    let handle = ace_artifacts::store::ManagedStore::open(&p.store).expect("store");
    let after = handle
        .root_artifacts("ac913-recovery", "source-recovery-current")
        .expect("roots");
    assert_eq!(after, before, "the previous root survives a failed publication");
    drop(handle);
    // ...and it still restores for real.
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "the previous root must still restore: {}", probe.line);
    assert_eq!(probe.json["restore_value"], 51);
}

#[test]
fn av_closures_recovery_913_old_path_control_reports_production() {
    // CONTROL (V-01): the same "restart" scenario WITHOUT a source-bound
    // binding is the legacy shape — it re-produces the module. This proves
    // the restore's P=0 assertion is a real difference and not a counter
    // nobody records.
    let store = temp_store("old-path");
    let mut parent = ClosuresFunctionSession::new(&store, "main.at");
    let cold = parent.apply_source(SRC).expect("cold production");
    assert_eq!(cold.value, 51);
    assert_eq!(cold.production.verified, 5);
    assert_eq!(cold.production.encoded, 5);
    parent.close().expect("close");
    // A new process/session with NO caller-held anchor: cold again.
    let mut restarted = ClosuresFunctionSession::new(&store, "main.at");
    let again = restarted.apply_source(SRC).expect("cold restart");
    assert_eq!(again.value, 51);
    assert!(
        again.production.verified >= 5 && again.production.encoded >= 5,
        "the legacy restart re-produces every unit: verified {} encoded {}",
        again.production.verified,
        again.production.encoded
    );
    assert_eq!(
        again.production.cache_hits, 5,
        "the store still answers, but only AFTER verify+encode"
    );
    restarted.close().expect("close");
}

#[test]
fn av_closures_recovery_913_missing_required_data_is_a_named_miss() {
    // The declaration context and every unit are REQUIRED data: losing them is
    // a locatable MISS, never a silent cold rebuild that would rewrite a PASS.
    let p = publish("missing-required");
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    // Drop the recovery layer's metadata (the manifest + context artifacts).
    let handle = ace_artifacts::store::ManagedStore::open(&p.store).expect("store");
    handle.purge_layer("A23-R").expect("purge recovery layer");
    drop(handle);
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok, "a purged root must not restore: {}", probe.line);
    assert_eq!(probe.json["code"], "recovery.miss");
}

/// A synthetic publication request: enough shape to reach the guards that do
/// not require a real production (CAS, caps, addressing). The unit's identity
/// wire and fingerprint are self-consistent, so only the guard under test can
/// reject it.
fn synthetic_publication<'a>(
    project: &'a str,
    generation: u64,
    source_sha256: &'a str,
    artifact: &'a str,
    context_payload: &'a [u8],
) -> PublicationRequest<'a> {
    PublicationRequest {
        project,
        source_sha256,
        source_len: SRC.len(),
        logical_owner: "main.at",
        owner_module: "m_src",
        owner_schema: "auto.mir.av-closures/1",
        entry: "d_main",
        profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
        capabilities: &[],
        accepted_generation: generation,
        commit_point: generation,
        context_revision: "ctx",
        accepted_state_schema: RECOVERY_SCHEMA,
        context_payload,
        context_schema: "ace.closures.context-dump/1",
        units: vec![RecoveryUnitInput {
            unit: "d_main".into(),
            logical: "owner".into(),
            body_digest: "b".into(),
            dependency_digest: "c".into(),
            env_layout_identity: None,
            mir_identity: format!("{:016x}", ace_av_core::contract::fnv1a64(&[1, 2, 3, 4])),
            body_fp: "e".into(),
            wire_bytes: 4,
            artifact: artifact.to_string(),
            mint_span: (0, 1),
            current_span: (0, 1),
            identity_wire: vec![1, 2, 3, 4],
        }],
    }
}

#[test]
fn av_closures_recovery_913_stale_generation_cannot_overwrite_newer_root() {
    // V-07: a slow publication carrying an OLD accepted generation must not
    // overwrite a newer root, and must not disturb its required closure.
    let store = temp_store("cas");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    session.apply_source(SRC).expect("cold");
    let first = session.publish_source_recovery("ac913-cas").expect("first");
    assert_eq!(first.accepted_generation, 0);
    let warm = session.apply_source(&edited_source()).expect("warm");
    assert_eq!(warm.value, 48);
    let second = session.publish_source_recovery("ac913-cas").expect("second");
    assert_eq!(second.accepted_generation, 1, "the edit advanced the generation");
    // A stale (generation 0) publication attempt is refused before any write.
    let stale = publish_source_recovery(
        &store,
        synthetic_publication("ac913-cas", 0, &p_source_sha(&second), &second.manifest_artifact, b"{}"),
    );
    assert_eq!(
        stale.expect_err("a stale generation must not publish").code,
        "recovery.stale-generation"
    );
    let handle = ace_artifacts::store::ManagedStore::open(&store).expect("store");
    let roots = handle
        .root_artifacts("ac913-cas", "source-recovery-current")
        .expect("roots");
    assert_eq!(
        roots,
        vec![second.manifest_artifact.clone()],
        "the newer root survives a stale publication"
    );
    drop(handle);
    // The surviving root restores the EDITED source with zero production.
    let store_arg = store.to_string_lossy().to_string();
    let edited_path = write_source(&store, "edited.at", &edited_source());
    let edited_arg = edited_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &second.manifest_artifact,
        &second.manifest_digest,
        &edited_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "the newer root must restore: {}", probe.line);
    assert_eq!(probe.json["restore_value"], 48);
    session.close().expect("close");
}

#[test]
fn av_closures_recovery_913_gc_keeps_rooted_required_closure() {
    // V-08 minimal: a rooted manifest keeps its required closure alive across
    // a real GC pass; without the root the same data is collectable.
    let p = publish("gc-rooted");
    let handle = ace_artifacts::store::ManagedStore::open(&p.store).expect("store");
    ace_artifacts::store::gc::run_gc(
        &handle,
        ace_artifacts::store::gc::GcConfig { high: 0, low: 0 },
    )
    .expect("gc");
    drop(handle);
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(
        probe.ok,
        "a rooted recovery closure must survive GC: {}",
        probe.line
    );
    assert_eq!(probe.json["restore_value"], 51);
    assert_eq!(probe.json["p_axis_attempts"], 0);
    // CONTROL: without a root the required data is no longer protected.
    let handle = ace_artifacts::store::ManagedStore::open(&p.store).expect("store");
    handle
        .remove_root("ac913-recovery", "source-recovery-current", &p.manifest)
        .expect("remove root");
    ace_artifacts::store::gc::run_gc(
        &handle,
        ace_artifacts::store::gc::GcConfig { high: 0, low: 0 },
    )
    .expect("gc");
    drop(handle);
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(
        !probe.ok,
        "unrooted required data must be collectable: {}",
        probe.line
    );
}

#[test]
fn av_closures_recovery_913_a_commit_b_publish_failure_keeps_old_binding() {
    // V-07: the source commit (A) and the recovery publication (B) are
    // SEPARATE states. A failed B never claims an A rollback and never hands
    // out a new ticket: the old binding still restores the OLD source, and
    // the NEW source has no binding at all.
    let store = temp_store("ab-split");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    session.apply_source(SRC).expect("cold");
    let first = session.publish_source_recovery("ac913-ab").expect("first");
    // A: the parent accepts the edited source (source commit happened).
    let warm = session.apply_source(&edited_source()).expect("warm");
    assert_eq!(warm.value, 48);
    // B: the publication of the NEW state fails.
    let failed = publish_source_recovery(
        &store,
        synthetic_publication("ac913-ab", 99, &p_source_sha(&first), &first.manifest_artifact, b""),
    );
    assert!(
        failed.is_err(),
        "the failed publication must report an error, not a ticket"
    );
    session.close().expect("close");
    // The project root still points at the FIRST manifest and the store still
    // protects it (closing the session does not retire the project's root).
    let handle = ace_artifacts::store::ManagedStore::open(&store).expect("store");
    let roots = handle
        .root_artifacts("ac913-ab", "source-recovery-current")
        .expect("roots");
    assert_eq!(roots, vec![first.manifest_artifact.clone()]);
    drop(handle);
    // The OLD binding restores the OLD source...
    let store_arg = store.to_string_lossy().to_string();
    let base_arg = p_source_arg(&store, "base.at", SRC);
    let probe = run_probe(&[
        &store_arg,
        &first.manifest_artifact,
        &first.manifest_digest,
        &base_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "the old binding must still restore: {}", probe.line);
    assert_eq!(probe.json["restore_value"], 51);
    // ...and the NEW source is NOT restoreable through it (no fake rollback,
    // no hidden re-binding).
    let edited_arg = p_source_arg(&store, "edited2.at", &edited_source());
    let probe = run_probe(&[
        &store_arg,
        &first.manifest_artifact,
        &first.manifest_digest,
        &edited_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok, "the new source has no binding: {}", probe.line);
    assert!(
        matches!(
            probe.json["code"].as_str(),
            Some("recovery.envelope") | Some("recovery.source-mismatch")
        ),
        "unexpected rejection: {}",
        probe.line
    );
}

fn p_source_arg(dir: &Path, name: &str, src: &str) -> String {
    write_source(dir, name, src).to_string_lossy().to_string()
}

#[test]
fn av_closures_recovery_913_bad_state_cold_repair_then_restore_again() {
    // V-10: a missing/bad accepted state is a named MISS; the caller repairs
    // by an explicit COLD rebuild and a fresh publication, and the NEXT fresh
    // process restores with zero production again — the earlier failure is
    // not papered over.
    let store = temp_store("repair");
    let src_path = write_source(&store, "main.at", SRC);
    let mut parent = ClosuresFunctionSession::new(&store, "main.at");
    parent.apply_source(SRC).expect("cold");
    let first = parent.publish_source_recovery("ac913-repair").expect("publish");
    parent.close().expect("close");
    // Break the accepted state.
    let handle = ace_artifacts::store::ManagedStore::open(&store).expect("store");
    handle.purge_layer("A23-R").expect("purge recovery layer");
    drop(handle);
    let request = RestoreRequest {
        expected_manifest: &first.manifest_artifact,
        expected_digest: &first.manifest_digest,
        source: SRC,
        logical_owner: "main.at",
        entry: "d_main",
        profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
        capabilities: &[],
        read_only: true,
    };
    let err = restore_source_recovery(&store, &request).err().expect("miss");
    assert!(
        err.code == "recovery.miss",
        "a broken accepted state is a named miss: {err:?}"
    );
    // REPAIR: an explicit cold rebuild + a NEW publication (never a silent
    // reuse of the broken root).
    let mut repaired = ClosuresFunctionSession::new(&store, "main.at");
    let cold = repaired.apply_source(SRC).expect("cold repair");
    assert_eq!(cold.value, 51);
    let second = repaired.publish_source_recovery("ac913-repair").expect("republish");
    assert_eq!(
        first.manifest_artifact, second.manifest_artifact,
        "a byte-identical accepted state is the SAME content-addressed artifact"
    );
    repaired.close().expect("close");
    // The next fresh process restores the repaired state with P=0.
    let store_arg = store.to_string_lossy().to_string();
    let source_arg = src_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &second.manifest_artifact,
        &second.manifest_digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "the repaired root must restore: {}", probe.line);
    assert_eq!(probe.json["restore_value"], 51);
    assert_eq!(probe.json["p_axis_attempts"], 0);
    assert_eq!(probe.json["r_deep_verifies"], 5);
}

#[test]
fn av_closures_recovery_913_read_only_restore_leaves_the_store_bytes_alone() {
    // V-09 minimal (data/root write zero): a restore runs against the caller's
    // PRE-AUTHORIZED root and performs no data or root write. The evidence is
    // the store's own files: their size and modification time must not move.
    let p = publish("readonly");
    let snapshot = |dir: &Path| {
        let mut rows = Vec::new();
        for entry in std::fs::read_dir(dir).expect("read store dir") {
            let entry = entry.expect("dir entry");
            let meta = entry.metadata().expect("metadata");
            rows.push((
                entry.file_name().to_string_lossy().to_string(),
                meta.len(),
                meta.modified().ok(),
            ));
        }
        rows.sort();
        rows
    };
    let before = snapshot(&p.store);
    let request = RestoreRequest {
        expected_manifest: &p.manifest,
        expected_digest: &p.digest,
        source: SRC,
        logical_owner: "main.at",
        entry: "d_main",
        profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
        capabilities: &[],
        read_only: true,
    };
    let restored = restore_source_session(&p.store, &request).expect("read-only restore");
    assert_eq!(restored.restore_value, Some(51));
    let after = snapshot(&p.store);
    // The store's own liveness LOCK directory is refreshed by opening the
    // store (an OS-held shared lock, not a data/root write). Everything that
    // carries DATA — the database and every content/staging/trash directory —
    // must be byte- and time-identical.
    let data_rows = |rows: &[(String, u64, Option<std::time::SystemTime>)]| {
        rows.iter()
            .filter(|(name, _, _)| name != "locks")
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        data_rows(&before),
        data_rows(&after),
        "a read-only restore must not write the store's data or root state"
    );
    let db = before
        .iter()
        .find(|(name, _, _)| name == "journal.db")
        .expect("store database");
    assert!(db.1 > 0);
}

fn p_source_sha(t: &ace_av::closures_source_recovery::RecoveryTicket) -> String {
    t.source_sha256.clone()
}

#[test]
fn av_closures_recovery_913_config_mismatch_is_named() {
    // V-02: the binding covers entry/profile/capabilities, not just the source.
    let p = publish("config");
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    for (entry, profile, caps, expected) in [
        (
            "d_other",
            ace_core::source::CLOSURES_SOURCE_PROFILE,
            "",
            "recovery.context-mismatch",
        ),
        ("d_main", "auto.source.core", "", "recovery.context-mismatch"),
        (
            "d_main",
            ace_core::source::CLOSURES_SOURCE_PROFILE,
            "trace",
            "recovery.capability",
        ),
    ] {
        let probe = run_probe(&[
            &store_arg,
            &p.manifest,
            &p.digest,
            &source_arg,
            "main.at",
            entry,
            profile,
            caps,
        ]);
        assert!(
            !probe.ok,
            "a changed config must be rejected: {}",
            probe.line
        );
        assert_eq!(probe.json["code"], expected, "probe: {}", probe.line);
    }
}

#[test]
fn av_closures_recovery_913_store_root_index_is_not_an_authority() {
    // V-03: the store's root/index is a HINT. The caller's expected id +
    // digest is the authority, so a restore still works when the project has
    // NO current root — and a wrong expectation still fails.
    let p = publish("index-hint");
    let handle = ace_artifacts::store::ManagedStore::open(&p.store).expect("store");
    handle
        .remove_root("ac913-recovery", "source-recovery-current", &p.manifest)
        .expect("remove root");
    let roots = handle
        .root_artifacts("ac913-recovery", "source-recovery-current")
        .expect("roots");
    assert!(roots.is_empty(), "no current root remains");
    drop(handle);
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &p.digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(
        probe.ok,
        "the caller-held binding must not depend on the store index: {}",
        probe.line
    );
    assert_eq!(probe.json["restore_value"], 51);
    // The caller's expectation is authoritative: a different digest fails even
    // though the store still holds the artifact.
    let forged = "1".repeat(64);
    let probe = run_probe(&[
        &store_arg,
        &p.manifest,
        &forged,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok);
    assert!(
        probe.json["code"]
            .as_str()
            .unwrap_or("")
            .starts_with("recovery."),
        "unexpected rejection: {}",
        probe.line
    );
}

#[test]
fn av_closures_recovery_913_caps_are_enforced_before_any_write() {
    // V-12: bounded data is refused at the publication door, before a single
    // artifact or root is written.
    let p = publish("caps");
    let big = vec![b'{'; 4 * 1024 * 1024 + 1];
    let err = publish_source_recovery(
        &p.store,
        synthetic_publication("ac913-caps", 1, &p.source_sha256, &p.manifest, &big),
    )
    .err()
    .expect("an oversized context must be refused");
    assert_eq!(err.code, "recovery.context-too-large");
    let mut request =
        synthetic_publication("ac913-caps", 1, &p.source_sha256, &p.manifest, b"{}");
    request.units[0].wire_bytes = 8 * 1024 * 1024 + 1;
    let err = publish_source_recovery(&p.store, request)
        .err()
        .expect("an oversized unit must be refused");
    assert_eq!(err.code, "recovery.unit-too-large");
    // Neither attempt moved the project's root.
    let handle = ace_artifacts::store::ManagedStore::open(&p.store).expect("store");
    let roots = handle
        .root_artifacts("ac913-recovery", "source-recovery-current")
        .expect("roots");
    assert_eq!(roots, vec![p.manifest.clone()]);
}

#[test]
fn av_closures_recovery_913_cross_unit_artifact_is_rejected() {
    // V-04: a manifest that self-consistently points at ANOTHER unit's valid
    // artifact is rejected by the envelope's unit binding, not silently
    // installed as that unit.
    let p = publish("cross-unit");
    let helper = p
        .unit_artifacts
        .iter()
        .find(|(unit, _)| unit == "d_helper")
        .expect("helper artifact")
        .1
        .clone();
    let request = synthetic_publication("ac913-cross", 0, &p.source_sha256, &helper, b"{}");
    let ticket = publish_source_recovery(&p.store, request).expect("synthetic publication");
    let store_arg = p.store.to_string_lossy().to_string();
    let source_arg = p.source.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &ticket.manifest_artifact,
        &ticket.manifest_digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(
        !probe.ok,
        "a cross-unit artifact must be rejected: {}",
        probe.line
    );
    assert_eq!(probe.json["code"], "recovery.envelope");
}

#[test]
fn av_closures_recovery_913_restart_runs_from_initial_values() {
    // V-11: a restored session executes from the source's INITIAL values every
    // time (the shared cell f writes is created and released per run), and no
    // live env/cell survives a run.
    let p = publish("restart-values");
    let request = RestoreRequest {
        expected_manifest: &p.manifest,
        expected_digest: &p.digest,
        source: SRC,
        logical_owner: "main.at",
        entry: "d_main",
        profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
        capabilities: &[],
        read_only: true,
    };
    let restored = restore_source_session(&p.store, &request).expect("restore session");
    assert_eq!(restored.restore_value, Some(51));
    let mut session = restored.session;
    // Re-running the SAME source in the restored session is a fresh execution
    // from the initial values (51), not a cached result.
    let again = session.apply_source(SRC).expect("re-run");
    assert_eq!(again.value, 51);
    assert_eq!(again.live_after_release, 0, "cells cleaned after the run");
    assert_eq!(again.allocs, again.frees);
    // The edit then runs from the same initial values.
    let edited = session.apply_source(&edited_source()).expect("edit");
    assert_eq!(edited.value, 48);
    assert_eq!(edited.live_after_release, 0);
    // ...and reverting re-runs the original semantics.
    let reverted = session.apply_source(SRC).expect("revert");
    assert_eq!(reverted.value, 51);
    assert_eq!(reverted.live_after_release, 0);
    session.close().expect("close");
}

#[test]
fn av_closures_recovery_913_bounded_by_id_read_enforces_the_budget() {
    // V-12: recovery data is addressed BY ARTIFACT ID, so the read itself must
    // be bounded — a payload over budget is refused before its bytes are
    // materialized (the recovery entry never reads unbounded data).
    let store = temp_store("bounded-read");
    let handle = ace_artifacts::store::ManagedStore::open(&store).expect("store");
    let big = vec![7u8; 2 * 1024 * 1024];
    let id = handle
        .put_artifact(ace_artifacts::store::PutRequest {
            layer: ace_av::closures_source_recovery::RECOVERY_LAYER.id,
            key: ace_artifacts::identity::QueryKey::new(
                &ace_av::closures_source_recovery::RECOVERY_LAYER,
                vec![("test".into(), "bounded".into())],
            ),
            kind: ace_artifacts::codec::ChunkKind::LayoutData,
            payload: big.clone(),
            edges: Vec::new(),
            project: None,
        })
        .expect("put oversized artifact");
    // A small budget refuses it...
    let bounded = handle.get_artifact_by_id_bounded(&id, 4096);
    assert!(
        bounded.is_err() || matches!(bounded, Ok(None)),
        "an over-budget payload must not be materialized"
    );
    // ...a sufficient budget reads it.
    let read = handle
        .get_artifact_by_id_bounded(&id, 4 * 1024 * 1024)
        .expect("bounded read")
        .expect("artifact present");
    assert_eq!(read.payload.len(), big.len());
}

#[test]
fn av_closures_recovery_913_session_entry_matches_probe() {
    // The same restore entry through a plain in-process call: the counters,
    // the restored value and the installed wires agree with the probe.
    let p = publish("inproc");
    let request = RestoreRequest {
        expected_manifest: &p.manifest,
        expected_digest: &p.digest,
        source: SRC,
        logical_owner: "main.at",
        entry: "d_main",
        profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
        capabilities: &[],
        read_only: true,
    };
    let restored = restore_source_recovery(&p.store, &request).expect("restore");
    assert_eq!(restored.counters.p_axis_attempts, 0);
    assert_eq!(restored.counters.unit_deep_verifies, p.units);
    assert_eq!(restored.counters.context_reads, 1);
    assert_eq!(restored.manifest.schema, RECOVERY_SCHEMA);
    assert_eq!(restored.manifest.entry, "d_main");
    assert_eq!(restored.units.len(), p.units);
    assert_eq!(restored.identity_wires.len(), p.units);
    assert_eq!(
        restored.frontend.units().len(),
        p.units,
        "the declaration context restores the full catalog"
    );
    for u in &restored.units {
        assert!(
            u.artifact.is_some(),
            "restored units keep their artifact address"
        );
        assert!(!u.wire.is_empty());
    }
    // The session entry installs those units and executes them.
    let restored = restore_source_session(&p.store, &request).expect("restore session");
    assert_eq!(restored.restore_value, Some(51));
    let mut session = restored.session;
    assert_eq!(session.unit_names().len(), p.units);
    session.close().expect("close");
}
