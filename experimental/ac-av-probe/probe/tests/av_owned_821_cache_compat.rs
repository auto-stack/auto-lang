//! PLAN-821 T-05 cache-dependency and old-consumer compatibility
//! (`av_owned_821` family — V-08/V-09).
//!
//! V-08 oracles: same-store restores re-verify and reproduce identical
//! results; a CHANGED type table is a clean miss (never a stale reuse);
//! a tampered payload fails the envelope checksum (a good store address
//! never resurrects bad bytes); owned artifacts never satisfy a foreign
//! producer/profile identity.
//!
//! V-09 oracles: owned AVAD v2 bytes under the OLD consumers (data /
//! core owner scopes, the data restore path) are explicit identity
//! rejections — no silent profile widening; the old corpora replay in
//! the script's compatibility scope.

use auto_av_core::contract::{
    ExecOutcome, ABC_DATA_SCHEMA, ABC_OWNED_SCHEMA, MIR_DATA_SCHEMA, MIR_SCHEMA, OWNED_PRODUCER_ID,
};
use std::path::PathBuf;

const SRC_A: &str = "\
type Note { body Str; n int }
fn main() Note {
    let a Note = Note { body: \"Auto\", n: 1 }
    return a.move
}
";

// Same entry, DIFFERENT type table (extra field): the types-fingerprint
// axis must part the keys.
const SRC_B: &str = "\
type Note { body Str; n int; k int }
fn main() Note {
    let a Note = Note { body: \"Auto\", n: 1, k: 2 }
    return a.move
}
";

fn tmp_store(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "av821-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn owned_mir_of(src: &str) -> auto_av_core::mir::MirModule {
    let compiled =
        auto_ac_prototype::source::compile_text_record_source(src, "c.at", &[]).expect("compile");
    let snap = auto_av_core::contract::fnv1a64(src.as_bytes());
    auto_mir_lower::lower_checked_owned(&compiled.checked, "d_main", snap).expect("lower")
}

fn types_fp_of(src: &str) -> String {
    auto_av_probe::owned_driver::types_fp_of(&owned_mir_of(src))
}

fn expect_success_note(out: &ExecOutcome) {
    let ExecOutcome::Success { value, .. } = out else {
        panic!("expected success, got {out:?}")
    };
    let auto_av_core::contract::Value::Record(r) = value else {
        panic!("expected a record, got {value:?}")
    };
    match &r.fields[0] {
        auto_av_core::contract::Value::Str(h) => assert_eq!(h.bytes(), b"Auto"),
        other => panic!("body must be Str, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// V-08: dependency-partitioned restores
// ---------------------------------------------------------------------------

#[test]
fn av_owned_821_v08_changed_types_is_clean_miss_not_stale_reuse() {
    let tmp = tmp_store("v08");
    let ra = auto_av_probe::owned_driver::request_owned(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("A persists");
    let rb = auto_av_probe::owned_driver::request_owned(Some(&tmp), "p", SRC_B, "d_main", &[])
        .expect("B persists");
    expect_success_note(ra.av.as_ref().unwrap());
    expect_success_note(rb.av.as_ref().unwrap());
    assert_ne!(
        ra.snapshot, rb.snapshot,
        "different sources, different snapshots"
    );
    // Both restore under their OWN fingerprints — identical results.
    for r in [&ra, &rb] {
        let out =
            auto_av_probe::owned_driver::restore_owned(&tmp, &r.owner, "d_main", r.snapshot, &{
                // the fp of the source that produced r
                if r.snapshot == ra.snapshot {
                    types_fp_of(SRC_A)
                } else {
                    types_fp_of(SRC_B)
                }
            })
            .expect("own-fp restore re-verifies and executes");
        expect_success_note(&out);
    }
    // Cross-fingerprint restores are CLEAN MISSES — the changed type
    // table never serves the other shape's artifact.
    let err = auto_av_probe::owned_driver::restore_owned(
        &tmp,
        &ra.owner,
        "d_main",
        ra.snapshot,
        &types_fp_of(SRC_B),
    )
    .unwrap_err();
    assert!(err.contains("not found"), "cross-fp miss, got {err}");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn av_owned_821_v08_good_hash_bad_payload_rejected() {
    // Tamper ONE byte inside the stored artifact's envelope: the envelope
    // checksum rejects — a good (layer/hash) address never resurrects
    // corrupted bytes, and nothing executes.
    use ac_artifact_probe::store::ManagedStore;
    let tmp = tmp_store("tamper");
    let r = auto_av_probe::owned_driver::request_owned(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("persist");
    let store = ManagedStore::open(&tmp).expect("open");
    let key = auto_av_probe::owned_driver::owned_module_key(
        &r.owner,
        "d_main",
        &types_fp_of(SRC_A),
        r.snapshot,
    );
    let art = store
        .get_artifact("A21", &key)
        .expect("get")
        .expect("present");
    assert_eq!(art.kind, ac_artifact_probe::codec::ChunkKind::LayoutData);
    let mut tampered = art.payload.clone();
    let n = tampered.len();
    tampered[n - 1] ^= 0xFF;
    let ident = auto_av_probe::cache::EnvelopeIdentity {
        kind: auto_av_probe::cache::EnvelopeKind::AbcUnit,
        producer: OWNED_PRODUCER_ID,
        schema: ABC_OWNED_SCHEMA,
        owner_module: r.owner.as_str(),
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: r.snapshot,
        unit: "d_main",
        body_fp: "",
    };
    let err = auto_av_probe::cache::envelope_unwrap(&tampered, &ident)
        .expect_err("tampered envelope rejects");
    assert!(
        format!("{err:?}").to_lowercase().contains("checksum"),
        "checksum rejection, got {err:?}"
    );
    // The pristine payload still unwraps (control).
    auto_av_probe::cache::envelope_unwrap(&art.payload, &ident).expect("pristine unwraps");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn av_owned_821_v08_profile_identity_isolation() {
    // An owned payload under the DATA (or core) producer/schema identity
    // fails the envelope admission — artifact identities never cross
    // profiles; the same holds for the owned identity over a data payload.
    let payload = b"AVAD-v2-owned-bytes".to_vec();
    let owned_ident = auto_av_probe::cache::EnvelopeIdentity {
        kind: auto_av_probe::cache::EnvelopeKind::AbcUnit,
        producer: OWNED_PRODUCER_ID,
        schema: ABC_OWNED_SCHEMA,
        owner_module: "m",
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: 0,
        unit: "d_main",
        body_fp: "",
    };
    let data_ident = auto_av_probe::cache::EnvelopeIdentity {
        kind: auto_av_probe::cache::EnvelopeKind::AbcUnit,
        producer: auto_av_core::contract::DATA_PRODUCER_ID,
        schema: ABC_DATA_SCHEMA,
        owner_module: "m",
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: 0,
        unit: "d_main",
        body_fp: "",
    };
    let core_ident = auto_av_probe::cache::EnvelopeIdentity::abc("m");
    let wrapped = auto_av_probe::cache::envelope_wrap(&payload, &owned_ident);
    assert!(
        auto_av_probe::cache::envelope_unwrap(&wrapped, &data_ident).is_err(),
        "owned bytes under the data identity must reject"
    );
    assert!(auto_av_probe::cache::envelope_wrap(&payload, &data_ident).len() > 0);
    let core_wrapped = auto_av_probe::cache::envelope_wrap(&payload, &core_ident);
    assert!(
        auto_av_probe::cache::envelope_unwrap(&core_wrapped, &owned_ident).is_err(),
        "core-wrapped bytes under the owned identity must reject"
    );
    // Control: the owned identity unwraps its own bytes.
    auto_av_probe::cache::envelope_unwrap(&wrapped, &owned_ident).expect("own identity unwraps");
}

// ---------------------------------------------------------------------------
// V-09: the old consumers reject owned bytes (no silent widening)
// ---------------------------------------------------------------------------

#[test]
fn av_owned_821_v09_owned_bytes_rejected_by_old_consumers() {
    let mir = owned_mir_of(SRC_A);
    let wire = auto_av_core::abc::encode_abc(&mir).expect("encode AVAD v2");
    let flat = wire.to_flat();
    assert_eq!(&flat[0..4], b"AVAD", "the owned wire is AVAD v2");

    // (a) The DATA admission path refuses the owned module (schema/
    // profile family mismatch — the data consumer never executes a
    // profile it does not know).
    let data_owner = auto_av_core::mir::OwnerScope {
        schema: MIR_DATA_SCHEMA.to_string(),
        module: mir.owner.module.clone(),
    };
    assert!(
        auto_av_core::abc_verify::verify_abc_bytes(&flat, &data_owner).is_err(),
        "the data consumer must refuse owned AVAD v2 bytes"
    );

    // (b) The CORE (av.core-i32/1, AVAB) admission path refuses likewise.
    let core_owner = auto_av_core::mir::OwnerScope {
        schema: MIR_SCHEMA.to_string(),
        module: mir.owner.module.clone(),
    };
    assert!(
        auto_av_core::abc_verify::verify_abc_bytes(&flat, &core_owner).is_err(),
        "the core consumer must refuse owned AVAD v2 bytes"
    );

    // (c) The DATA restore path over an OWNED store is a clean miss (the
    // A21 owned layer is not the data A20 layer — no cross-layer service).
    let tmp = tmp_store("v09");
    let r = auto_av_probe::owned_driver::request_owned(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("owned persist");
    let data_fp = {
        let compiled = auto_ac_prototype::source::compile_record_source(
            "type P { x int }\nfn main() int {\n    var p = P { x: 1 }\n    return p.x\n}\n",
            "d.at",
            &[],
        )
        .expect("data source compiles");
        let dm = auto_mir_lower::lower_checked_data(
            &compiled.checked,
            "d_main",
            auto_av_core::contract::fnv1a64(b"type P"),
        )
        .expect("lower data");
        auto_av_probe::data_driver::types_fp_of(&dm)
    };
    let err =
        auto_av_probe::data_driver::restore_data(&tmp, &r.owner, "d_main", r.snapshot, &data_fp)
            .unwrap_err();
    assert!(
        err.contains("not found"),
        "the data restore over an owned store is a clean miss, got {err}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// The owned store's own replay stays green after the cross-profile
/// probes above touched the same store directories (control).
#[test]
fn av_owned_821_v09_control_owned_restore_still_green() {
    let tmp = tmp_store("ctl");
    let r = auto_av_probe::owned_driver::request_owned(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("persist");
    let out = auto_av_probe::owned_driver::restore_owned(
        &tmp,
        &r.owner,
        "d_main",
        r.snapshot,
        &types_fp_of(SRC_A),
    )
    .expect("restore");
    expect_success_note(&out);
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// R1-F10: the REAL new-worker restore + a self-consistent bad inner
// ---------------------------------------------------------------------------

/// The owned store restores in a REAL NEW PROCESS (the worker's
/// `owned-restore-run`): a fresh executable re-admits the envelope,
/// re-decodes and re-runs the FULL deep verification before executing —
/// the same value comes back, exit 0.
#[test]
fn av_owned_821_r1_new_process_restore_executes() {
    let tmp = tmp_store("r1newproc");
    let r = auto_av_probe::owned_driver::request_owned(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("persist");
    let fp = types_fp_of(SRC_A);
    // Locate and build the release worker (contained, bounded — same
    // discipline as the av case worker).
    let worker = locate_release_worker();
    let out = std::process::Command::new(&worker)
        .args([
            "owned-restore-run",
            "--store",
            tmp.to_str().unwrap(),
            "--owner",
            &r.owner,
            "--entry",
            "d_main",
            "--snapshot",
            &format!("{:016x}", r.snapshot),
            "--types-fp",
            &fp,
        ])
        .output()
        .expect("spawn the fresh restore worker");
    assert!(
        out.status.success(),
        "the new-process restore succeeds: {} | stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let line = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(
        line.lines()
            .find(|l| !l.trim().is_empty())
            .expect("one line"),
    )
    .expect("json");
    assert_eq!(v["ok"], serde_json::json!(true));
    assert_eq!(v["restored"], serde_json::json!(true));
    assert_eq!(
        v["result"]["kind"], "record",
        "the canonical record face comes back: {}",
        v["result"]
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// A SELF-CONSISTENT bad inner: the tampered ABC payload is re-wrapped
/// with a CORRECT envelope checksum under the SAME good key (the outer
/// hash/checksum passes), published into the store, and restored in a
/// NEW process — the DEEP verification (not the checksum) rejects it and
/// nothing executes (non-zero exit, restored=false at the deep gate).
#[test]
fn av_owned_821_r1_self_consistent_bad_inner_never_executes() {
    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::store::{ManagedStore, PutRequest};

    let tmp = tmp_store("r1badinner");
    let r = auto_av_probe::owned_driver::request_owned(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("persist");
    let fp = types_fp_of(SRC_A);

    // The good artifact + key.
    let store = ManagedStore::open(&tmp).expect("open");
    let key = auto_av_probe::owned_driver::owned_module_key(&r.owner, "d_main", &fp, r.snapshot);
    let art = store
        .get_artifact("A21", &key)
        .expect("get")
        .expect("present");
    let ident = auto_av_probe::cache::EnvelopeIdentity {
        kind: auto_av_probe::cache::EnvelopeKind::AbcUnit,
        producer: OWNED_PRODUCER_ID,
        schema: ABC_OWNED_SCHEMA,
        owner_module: r.owner.as_str(),
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: r.snapshot,
        unit: "d_main",
        body_fp: "",
    };
    let good_payload = auto_av_probe::cache::envelope_unwrap(&art.payload, &ident)
        .expect("the pristine payload unwraps");
    // Tamper the INNER ABC (flip a byte in the body region, away from the
    // header), then re-wrap with a FRESH CORRECT envelope — the outer
    // checksum is valid by construction; only the deep verification can
    // catch it.
    let mut bad = good_payload.clone();
    let n = bad.len();
    assert!(n > 64, "a real AVAD payload");
    bad[n - 17] ^= 0xFF;
    let bad_env = auto_av_probe::cache::envelope_wrap(&bad, &ident);
    // The tampered+re-wrapped payload PASSES the envelope gate...
    auto_av_probe::cache::envelope_unwrap(&bad_env, &ident)
        .expect("the self-consistent envelope unwraps (checksum valid)");
    // ...and is PUBLISHED under the SAME good key (a fresh scratch store
    // seeded with it — the store's own republish-conflict semantics stay
    // out of this oracle's way).
    let tmp2 = tmp_store("r1badinner2");
    let store2 = ManagedStore::open(&tmp2).expect("open 2");
    store2
        .put_artifact(PutRequest {
            layer: "A21",
            key: key.clone(),
            kind: ChunkKind::LayoutData,
            payload: bad_env,
            edges: vec![],
            project: Some("p".to_string()),
        })
        .expect("publish the bad inner under the good key");

    // Restore in a REAL NEW PROCESS: the deep gate rejects, exit 4.
    let worker = locate_release_worker();
    let out = std::process::Command::new(&worker)
        .args([
            "owned-restore-run",
            "--store",
            tmp2.to_str().unwrap(),
            "--owner",
            &r.owner,
            "--entry",
            "d_main",
            "--snapshot",
            &format!("{:016x}", r.snapshot),
            "--types-fp",
            &fp,
        ])
        .output()
        .expect("spawn");
    assert!(
        !out.status.success(),
        "the self-consistent bad inner MUST be refused: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let line = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(line.lines().find(|l| !l.trim().is_empty()).unwrap_or(""))
            .unwrap_or(serde_json::Value::Null);
    assert_eq!(
        v.get("dispatched").and_then(|d| d.as_bool()),
        Some(false),
        "dispatched=0 — nothing executed on the bad inner"
    );
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::remove_dir_all(&tmp2);
}

/// Locate (and build on first use) the release worker binary — the same
/// lookup the av case worker uses (env override → probe workspace).
fn locate_release_worker() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("AC_AV_CASE_WORKER") {
        let p = std::path::PathBuf::from(p);
        if p.is_file() {
            return p;
        }
    }
    // CARGO_MANIFEST_DIR is the probe CRATE dir; the workspace root (and
    // its target/) is one level up.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    assert!(
        manifest.join("Cargo.toml").is_file(),
        "workspace manifest at {}",
        manifest.display()
    );
    let exe = manifest
        .join("target")
        .join("release")
        .join(if cfg!(windows) {
            "av_probe_worker.exe"
        } else {
            "av_probe_worker"
        });
    // Build (bounded no-op when fresh).
    let status = std::process::Command::new(env!("CARGO"))
        .args([
            "build",
            "--release",
            "-p",
            "auto-av-probe",
            "--bin",
            "av_probe_worker",
        ])
        .current_dir(&manifest)
        .output()
        .expect("cargo build");
    assert!(
        status.status.success(),
        "worker build: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(exe.is_file(), "worker at {}", exe.display());
    exe
}
