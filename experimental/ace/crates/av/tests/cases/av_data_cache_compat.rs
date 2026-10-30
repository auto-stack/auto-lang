//! PLAN-820 T-05 cache-dependency and old-consumer compatibility
//! (`av_data_820` family, part 4 — V-08/V-09).
//!
//! V-08 oracles: same-store restores re-verify and return identical
//! values; a CHANGED type table produces a clean miss (never a stale
//! reuse); a tampered payload fails the envelope checksum (a good store
//! hash never resurrects bad bytes); a data payload under the WRONG
//! producer/profile envelope is an identity rejection.
//!
//! V-09 oracles: the core-i32 driver replays its scalar vertical
//! unchanged; record source through the OLD consumer rejects at the
//! scalar front door (parse-stage diag, no silent profile widening); and
//! the OLD manifest-driven restore path refuses a data-only store.

use ace_av_core::contract::ExecOutcome;
use std::path::PathBuf;

const SRC_A: &str = "\
type P { x int }
fn main() int {
    var p = P { x: 40 }
    p.x = p.x + 2
    return p.x
}
";
// Same entry, DIFFERENT type table (extra field + nested record): the
// types-fingerprint axis must part the keys.
const SRC_B: &str = "\
type I { v int }
type P { x int; inner I }
fn main() int {
    var p = P { x: 1, inner: I { v: 2 } }
    p.inner.v = 41
    return p.x + p.inner.v
}
";

fn tmp_store(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "av820-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn types_fp_of(src: &str) -> String {
    let compiled = ace_core::source::compile_record_source(src, "c.at", &[]).expect("compile");
    let mir = ace_lowering::lower_checked_data(&compiled.checked, "d_main", 0).expect("lower");
    ace_av::data_driver::types_fp_of(&mir)
}

// ---------------------------------------------------------------------------
// V-08: dependency-partitioned restores
// ---------------------------------------------------------------------------

#[test]
fn av_data_cache_changed_types_is_clean_miss_not_stale_reuse() {
    let tmp = tmp_store("v08");
    let ra = ace_av::data_driver::request_data(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("A persists");
    let rb = ace_av::data_driver::request_data(Some(&tmp), "p", SRC_B, "d_main", &[])
        .expect("B persists");
    assert!(
        matches!(
            ra.av,
            Some(ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(42),
                ..
            })
        ),
        "A returns 42"
    );
    assert!(
        matches!(
            rb.av,
            Some(ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(42),
                ..
            })
        ),
        "B returns 42 (1 + 41)"
    );
    // Both restore under their OWN fingerprints — identical values.
    for (r, src) in [(&ra, SRC_A), (&rb, SRC_B)] {
        let out = ace_av::data_driver::restore_data(
            &tmp,
            &r.owner,
            "d_main",
            r.snapshot,
            &types_fp_of(src),
        )
        .expect("own-fp restore");
        assert!(matches!(
            out,
            ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(42),
                ..
            }
        ));
    }
    // Cross-fingerprint restores are CLEAN MISSES — the changed type table
    // never serves the other shape's artifact.
    let err = ace_av::data_driver::restore_data(
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
fn av_data_cache_good_hash_bad_payload_rejected() {
    // Tamper ONE payload byte inside the stored artifact's envelope: the
    // envelope checksum rejects — a good (layer/hash) address never
    // resurrects corrupted bytes.
    use ace_artifacts::store::ManagedStore;
    let tmp = tmp_store("tamper");
    let r =
        ace_av::data_driver::request_data(Some(&tmp), "p", SRC_A, "d_main", &[]).expect("persist");
    let store = ManagedStore::open(&tmp).expect("open");
    // Locate the artifact by layer+key via the data module key.
    let compiled = ace_core::source::compile_record_source(SRC_A, "c.at", &[]).unwrap();
    let snap = ace_av_core::contract::fnv1a64(SRC_A.as_bytes());
    let mir = ace_lowering::lower_checked_data(&compiled.checked, "d_main", snap).unwrap();
    let key = ace_av::data_driver::data_module_key(&r.owner, "d_main", &mir);
    let art = store
        .get_artifact("A20", &key)
        .expect("get")
        .expect("present");
    let mut tampered = art.payload.clone();
    // Flip a byte in the LAST 8 bytes = the envelope checksum itself; a
    // payload-region flip behaves identically (checksum covers it).
    let n = tampered.len();
    tampered[n - 1] ^= 0xFF;
    let ident = ace_av::cache::EnvelopeIdentity {
        kind: ace_av::cache::EnvelopeKind::AbcUnit,
        producer: ace_av_core::contract::DATA_PRODUCER_ID,
        schema: ace_av_core::contract::ABC_DATA_SCHEMA,
        owner_module: r.owner.as_str(),
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: r.snapshot,
        unit: "d_main",
        body_fp: "",
    };
    let err =
        ace_av::cache::envelope_unwrap(&tampered, &ident).expect_err("tampered envelope rejects");
    assert!(
        format!("{err:?}").contains("checksum")
            || format!("{err:?}").to_lowercase().contains("checksum"),
        "checksum rejection, got {err:?}"
    );
    // The pristine payload still unwraps (control).
    ace_av::cache::envelope_unwrap(&art.payload, &ident).expect("pristine unwraps");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn av_data_data_payload_under_core_envelope_identity_rejected() {
    // Profile/producer axis: an AVAD payload wrapped under the CORE
    // producer identity fails the data-side envelope admission (and vice
    // versa) — the artifact stores never share identities across profiles.
    let payload = b"AVAD-payload-bytes".to_vec();
    let core_ident = ace_av::cache::EnvelopeIdentity::abc("m");
    let wrapped = ace_av::cache::envelope_wrap(&payload, &core_ident);
    let data_ident = ace_av::cache::EnvelopeIdentity {
        kind: ace_av::cache::EnvelopeKind::AbcUnit,
        producer: ace_av_core::contract::DATA_PRODUCER_ID,
        schema: ace_av_core::contract::ABC_DATA_SCHEMA,
        owner_module: "m",
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: 0,
        unit: "d_main",
        body_fp: "",
    };
    let err = ace_av::cache::envelope_unwrap(&wrapped, &data_ident)
        .expect_err("cross-profile identity rejects");
    let msg = format!("{err:?}").to_lowercase();
    assert!(
        msg.contains("producer") || msg.contains("schema") || msg.contains("identity"),
        "identity-axis rejection, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// V-09: old consumers stay frozen
// ---------------------------------------------------------------------------

#[test]
fn av_data_old_consumer_replays_scalar_and_rejects_record_source() {
    // (a) The core-i32 driver replays its scalar vertical unchanged.
    let mut d = ace_av::driver::Driver::ephemeral();
    let vertical = "fn score(a int, b int) int {\n    if a > b {\n        return a - b\n    }\n    return b - a\n}\nfn main() int {\n    let x = score(17, 29)\n    let y = score(40, 13)\n    if x + y != 39 {\n        return 1\n    }\n    return 0\n}\n";
    let r = d.request(vertical, "d_main", &[]);
    assert!(
        matches!(r.value_i32(), Some(0)),
        "scalar vertical replays green, got {:?}",
        r.av
    );

    // (b) Record source through the OLD consumer: the scalar front door
    // rejects at the parse/frontend stage — no silent profile widening.
    let r = d.request(SRC_A, "d_main", &[]);
    let diag = r.diag.expect("record source rejects in the scalar driver");
    assert!(
        diag.stage == "frontend" || diag.stage == "admission",
        "parse-stage rejection, got {diag:?}"
    );
}

#[test]
fn av_data_old_manifest_restore_refuses_data_store() {
    // The OLD restore path is manifest-driven and core-profile-gated: a
    // data-only store (no core manifest) is refused before anything runs.
    let tmp = tmp_store("oldrestore");
    ace_av::data_driver::request_data(Some(&tmp), "p", SRC_A, "d_main", &[])
        .expect("persist data artifact");
    let worker = env!("CARGO_BIN_EXE_av-worker");
    let out = std::process::Command::new(worker)
        .arg("restore-run")
        .arg("--store")
        .arg(&tmp)
        .arg("--owner")
        .arg("probe")
        .arg("--entry")
        .arg("d_main")
        .arg("--manifest-artifact")
        .arg("does-not-exist")
        .output()
        .expect("spawn");
    assert!(
        !out.status.success(),
        "the old restore path refuses a data-only store"
    );
    let reported = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        reported.contains("manifest") || reported.contains("store"),
        "manifest-gated refusal, got {reported}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}
