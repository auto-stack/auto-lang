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
    assert_ne!(first.manifest_artifact, second.manifest_artifact);
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
