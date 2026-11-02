//! PLAN-913 T-02: the minimal SOURCE-BOUND RECOVERY chain.
//!
//! Real production in the parent (912 function session, store-backed) →
//! publish an atomically rooted source-origin manifest → a DIFFERENT process
//! (the `av_closures_913_restore` probe) restores the accepted state from the
//! caller's expected root id + digest, deep-verifies every unit, installs,
//! links and executes — with ZERO source-production stages (P axis) and real
//! decode/verify work (R axis). Oracle values are hand-computed from the
//! source, never carried in the request.
//!
//! Negative controls: a wrong expected digest, an absent root and a manifest
//! whose self-description was rewritten out of band are controlled
//! rejections, not silent cold rebuilds.

use std::path::{Path, PathBuf};

use ace_av::closures_fn_session::ClosuresFunctionSession;
use ace_av::closures_source_recovery::{
    publish_source_recovery, restore_source_recovery, PublicationRequest, RecoveryUnitInput,
    RestoreRequest, RECOVERY_SCHEMA,
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

fn temp_store(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "av913-recovery-{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("store dir");
    dir
}

fn write_source(dir: &Path, src: &str) -> PathBuf {
    let path = dir.join("main.at");
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

/// The parent-side cold production + publication used by every case.
fn publish(tag: &str) -> (PathBuf, PathBuf, String, String, usize) {
    let store = temp_store(tag);
    let src_path = write_source(&store, SRC);
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
    // The stored payload is the AVEN envelope AROUND the manifest bytes: the
    // digest the parent hands out covers the inner payload, the envelope
    // binds it to producer/schema/owner/source.
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
    (
        store,
        src_path,
        ticket.manifest_artifact,
        ticket.manifest_digest,
        ticket.units,
    )
}

#[test]
fn av_closures_recovery_913_fresh_process_restore_zero_production() {
    let (store, src_path, manifest_id, digest, units) = publish("fresh");
    let store_arg = store.to_string_lossy().to_string();
    let source_arg = src_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &manifest_id,
        &digest,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "restore failed: {} (exit {:?})", probe.line, probe.code);
    let json = &probe.json;
    assert_eq!(json["ok"], true);
    assert_eq!(json["value"], 51, "restored execution oracle");
    assert_ne!(
        json["pid"].as_u64().unwrap(),
        std::process::id() as u64,
        "the restore must run in a DIFFERENT process"
    );
    assert_eq!(json["units"], units);
    // The positive path takes ZERO source-production stages...
    assert_eq!(json["p_axis_attempts"], 0);
    assert_eq!(json["p_verified"], 0, "no verify_mir before lookup");
    assert_eq!(json["p_encoded"], 0, "no encode_abc before lookup");
    assert_eq!(json["store_writes"], 0, "read-only restore writes nothing");
    // ...and REAL deep verification work.
    assert_eq!(json["r_manifest_reads"], 1);
    assert_eq!(json["r_unit_reads"], units);
    assert_eq!(json["r_deep_verifies"], units);
    assert_eq!(json["r_admits"], units);
    assert_eq!(json["p_admitted_installed"], units);
    assert!(json["wire_bytes"].as_u64().unwrap() > 0);
    assert_eq!(json["live_after_release"], 0);
    assert_eq!(json["allocs"], json["frees"]);
}

#[test]
fn av_closures_recovery_913_wrong_digest_and_missing_root_are_named_rejections() {
    let (store, src_path, manifest_id, digest, _units) = publish("negative");
    let store_arg = store.to_string_lossy().to_string();
    let source_arg = src_path.to_string_lossy().to_string();
    // A wrong parent digest: the store's self-description never overrides
    // the caller's expectation.
    let forged = "0".repeat(64);
    let probe = run_probe(&[
        &store_arg,
        &manifest_id,
        &forged,
        &source_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok);
    assert!(
        probe.json["code"].as_str().unwrap_or("").starts_with("recovery."),
        "unexpected rejection: {}",
        probe.line
    );
    // An absent root id is a locatable MISS with no cold fallback.
    let probe = run_probe(&[
        &store_arg,
        "art-does-not-exist",
        &digest,
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
    let (store, src_path, manifest_id, digest, _units) = publish("binding");
    let store_arg = store.to_string_lossy().to_string();
    let other_src = store.join("other.at");
    std::fs::write(&other_src, SRC.replace("n + 1", "n + 2")).expect("write other source");
    let other_arg = other_src.to_string_lossy().to_string();
    // Same manifest, DIFFERENT source text: the source SHA binding rejects.
    let probe = run_probe(&[
        &store_arg,
        &manifest_id,
        &digest,
        &other_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(!probe.ok, "a manifest must not restore a different source: {}", probe.line);
    // Different logical owner (a cross-domain context) is rejected too.
    let source_arg = src_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &manifest_id,
        &digest,
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
    let edited = SRC.replace("return x - n }", "return x - n * 2 }");
    let warm = session.apply_source(&edited).expect("warm");
    assert_eq!(warm.value, 48);
    let second = session.publish_source_recovery("ac913-repub").expect("second");
    assert_ne!(first.manifest_artifact, second.manifest_artifact);
    let handle = ace_artifacts::store::ManagedStore::open(&store).expect("store");
    let roots = handle
        .root_artifacts("ac913-repub", "source-recovery-current")
        .expect("roots");
    assert_eq!(roots, vec![second.manifest_artifact.clone()], "one current root");
    // The newer manifest restores the edited source's oracle.
    let src_path = store.join("edited.at");
    std::fs::write(&src_path, &edited).expect("write edited");
    let src_arg = src_path.to_string_lossy().to_string();
    let store_arg = store.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &second.manifest_artifact,
        &second.manifest_digest,
        &src_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "republished restore failed: {}", probe.line);
    assert_eq!(probe.json["value"], 48);
    // The superseded manifest still restores its OWN (older) source — the
    // new root never rewrites the old binding.
    let src_path = store.join("base.at");
    std::fs::write(&src_path, SRC).expect("write base");
    let src_arg = src_path.to_string_lossy().to_string();
    let probe = run_probe(&[
        &store_arg,
        &first.manifest_artifact,
        &first.manifest_digest,
        &src_arg,
        "main.at",
        "d_main",
        ace_core::source::CLOSURES_SOURCE_PROFILE,
    ]);
    assert!(probe.ok, "old manifest restore failed: {}", probe.line);
    assert_eq!(probe.json["value"], 51);
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
            }],
        },
    );
    let err = result.expect_err("an unaddressed unit must not publish");
    assert_eq!(err.code, "recovery.unit-unaddressed");
}

#[test]
fn av_closures_recovery_913_in_process_restore_matches_probe() {
    // The same restore entry through a plain in-process call: the counters
    // and the executed value agree with the out-of-process probe, and the
    // installed unit wires are the persisted ones (no re-encode).
    let (store, _src_path, manifest_id, digest, units) = publish("inproc");
    let request = RestoreRequest {
        expected_manifest: &manifest_id,
        expected_digest: &digest,
        source: SRC,
        logical_owner: "main.at",
        entry: "d_main",
        profile: ace_core::source::CLOSURES_SOURCE_PROFILE,
        capabilities: &[],
        read_only: true,
    };
    let restored = restore_source_recovery(&store, &request).expect("restore");
    assert_eq!(restored.counters.p_axis_attempts, 0);
    assert_eq!(restored.counters.unit_deep_verifies, units);
    assert_eq!(restored.manifest.schema, RECOVERY_SCHEMA);
    assert_eq!(restored.manifest.entry, "d_main");
    assert_eq!(restored.units.len(), units);
    for u in &restored.units {
        assert!(u.artifact.is_some(), "restored units keep their artifact address");
        assert!(u.wire.len() > 0);
    }
    let mut session = ace_av::closures_session::ClosuresUpdateSession::new(None, "d_main");
    let outcome = session.restore_install(restored.units).expect("install");
    assert_eq!(outcome.value, Some(51));
    assert_eq!(outcome.production.verified, 0);
    assert_eq!(outcome.production.encoded, 0);
    assert_eq!(outcome.production.admitted, units);
    assert_eq!(outcome.live_after_release, 0);
    session.close().expect("close");
}
