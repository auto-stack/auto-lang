//! PLAN-826 T-02: the compact AVAC wire first case. The SAME complete
//! module as the delivered legacy first case (direct call + short-circuit
//! CFG) is carried over the new wire generation end to end: compact
//! encode/decode, REAL compact admission, execution on the D825 loaded
//! representation, codec-dimensioned cache rows, and AVAC persist ->
//! fresh-worker restore re-verify. The independent result/trace oracle is
//! frozen with the case source: 42 and a,b,a.

use auto_av_core::av::{execute, execute_reference, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value, AV_PROFILE, TRACE_CAPABILITY};
use auto_av_core::mir::TraceEvent;
use auto_av_core::{
    decode_abc, decode_compact_abc, encode_abc, encode_compact_abc, verify_abc_bytes,
    verify_compact_abc_bytes, COMPACT_ABC_MAGIC,
};
use auto_av_probe::cache::{manifest_key, open_store, split_units};
use auto_av_probe::driver::Driver;
use auto_av_probe::wire::WireCodec;

const FIRST_CASE: &str =
    include_str!("../../../../crates/auto-lang/test/cases/av-abc/826/src/first-case.at");
const EXPECTED_TRACE: &[TraceEvent] = &[TraceEvent::MarkA, TraceEvent::MarkB, TraceEvent::MarkA];

fn lower_first_case_mir() -> auto_av_core::mir::MirModule {
    let compiled = auto_ac_prototype::source::compile_scalar_source(
        FIRST_CASE,
        "first-case.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("scalar source compiles");
    let mir = auto_mir_lower::lower_checked(&compiled.checked, "d_main", 826)
        .expect("lower real HIR");
    assert_eq!(mir.profile, AV_PROFILE);
    auto_av_core::verify_mir(&mir).expect("MIR verifies");
    mir
}

fn admit_compact() -> auto_av_core::LoadedModule {
    let mir = lower_first_case_mir();
    let wire = encode_compact_abc(&mir).expect("compact encode");
    verify_compact_abc_bytes(&wire, &mir.owner).expect("compact admission")
}

fn assert_42(out: ExecOutcome, events: &[TraceEvent]) {
    let ExecOutcome::Success { value, .. } = out else {
        panic!("first case must succeed: {out:?}")
    };
    assert_eq!(value, Value::I32(42));
    assert_eq!(events, EXPECTED_TRACE);
}

#[test]
fn av_wire_826_first_case_compact_roundtrip_admission_and_execution() {
    let mir = lower_first_case_mir();
    let wire = encode_compact_abc(&mir).expect("compact encode");
    assert!(wire.starts_with(&COMPACT_ABC_MAGIC), "AVAC magic");
    // The two wire generations never cross-decode: the legacy decoder
    // refuses AVAC bytes by magic.
    assert!(decode_abc(&wire).is_err(), "legacy decoder rejects AVAC");
    assert!(
        !WireCodec::LegacyAbc.is_current_generation(&wire)
            && WireCodec::CompactAbc.is_current_generation(&wire)
    );
    let decoded = decode_compact_abc(&wire).expect("compact decode");
    assert_eq!(decoded.module.profile, AV_PROFILE);
    assert_eq!(decoded.module.funcs.len(), mir.funcs.len());

    let loaded = verify_compact_abc_bytes(&wire, &mir.owner).expect("compact admission");
    assert!(loaded.executable(), "complete closure is executable");

    let cancel = CancelFlag::new(false);
    let (new_out, new_trace) = execute(&loaded, &ExecLimits::default(), &cancel);
    let (ref_out, ref_trace) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
    assert_42(new_out, &new_trace.events);
    assert_42(ref_out, &ref_trace.events);
    // The executed closure rides the D825 loaded representation.
    assert!(new_trace.loaded_index_builds >= 2);
    assert!(new_trace.loaded_pc_lookups > 0);
    assert_eq!(new_trace.reference_pc_lookups, 0);
    assert!(ref_trace.reference_pc_lookups > 0);
    assert_eq!(ref_trace.loaded_pc_lookups, 0);
}

#[test]
fn av_wire_826_first_case_wire_generations_never_cross_admit() {
    let mir = lower_first_case_mir();
    let compact = encode_compact_abc(&mir).expect("compact encode");
    let legacy = encode_abc(&mir).expect("legacy encode").to_flat();
    // Magic isolation holds in BOTH directions at the admission entries.
    assert!(verify_abc_bytes(&compact, &mir.owner).is_err());
    assert!(verify_compact_abc_bytes(&legacy, &mir.owner).is_err());
    assert!(decode_compact_abc(&legacy).is_err());
}

#[test]
fn av_wire_826_first_case_persist_cache_hit_and_new_worker_restore() {
    let root = std::env::temp_dir().join(format!("av826-first-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let caps = vec![TRACE_CAPABILITY.to_string()];
    let mut driver = Driver::with_store(&root, "av826-first").expect("store driver");
    driver.set_wire_compact();
    let first = driver.request(FIRST_CASE, "d_main", &caps);
    let first_av = match first.av.clone() {
        Some(av) => av,
        None => panic!(
            "no AV outcome; diag={:?} spans={:?}",
            first.diag, first.spans
        ),
    };
    assert_42(first_av, &first.trace);
    assert!(*first.counters.get("abc_wire_compact_closure").unwrap_or(&0) >= 1);
    assert!(*first.counters.get("cache_hit_abc").unwrap_or(&0) == 0);
    assert!(first.execution.loaded_index_builds >= 2);
    assert!(first.execution.loaded_pc_lookups > 0);

    let second = driver.request(FIRST_CASE, "d_main", &caps);
    let second_av = match second.av.clone() {
        Some(av) => av,
        None => panic!(
            "no AV outcome on cache-hit request; diag={:?} spans={:?}",
            second.diag, second.spans
        ),
    };
    assert_42(second_av, &second.trace);
    assert!(*second.counters.get("cache_hit_abc").unwrap_or(&0) >= 2);
    assert!(
        *second
            .counters
            .get("abc_wire_compact_closure")
            .unwrap_or(&0)
            >= 1,
        "a cache-hit request still admits the whole closure over AVAC"
    );
    assert!(second.execution.loaded_pc_lookups > 0);

    // The persisted manifest names the compact codec; the fresh worker
    // restore re-verifies every AVAC unit and the closure before running.
    let manifest = second.manifest.as_ref().expect("persisted manifest");
    assert_eq!(
        manifest.abc_codec,
        auto_av_core::COMPACT_ABC_SCHEMA,
        "manifest routes restore over the compact generation"
    );
    let store = open_store(&root).expect("reopen store");
    let key = manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let artifact = store
        .get_artifact("A09", &key)
        .expect("manifest lookup")
        .expect("manifest artifact");
    drop(store);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_av_probe_worker"))
        .args([
            "restore-run",
            "--store",
            root.to_str().unwrap(),
            "--owner",
            &manifest.owner,
            "--entry",
            &manifest.entry,
            "--manifest-artifact",
            &artifact.artifact_id,
            "--snapshot",
            &format!("{:016x}", second.snapshot),
            "--caps",
            TRACE_CAPABILITY,
        ])
        .output()
        .expect("fresh worker restore");
    assert!(
        output.status.success(),
        "worker failed: {}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let restored: auto_av_probe::worker_restore::RestoreOutcome =
        serde_json::from_slice(&output.stdout).expect("worker JSON");
    assert!(restored.executed_ok, "{:?}", restored.error);
    assert_eq!(restored.value, Some(42));
    assert_eq!(restored.units_verified, 2);
    assert_eq!(restored.trace, ["a", "b", "a"]);
    assert!(restored.loaded_index_builds >= 2);
    assert!(restored.loaded_pc_lookups > 0);
    assert_eq!(restored.reference_pc_lookups, 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn av_wire_826_first_case_legacy_driver_and_codec_namespaces() {
    // The delivered legacy AVAB path is unchanged by the compact policy,
    // and the two generations never share a cache namespace: over a store
    // already warm under the compact policy, a legacy request determinis-
    // tically misses every compact row (rebuild + re-verify), never reads
    // AVAC bytes as AVAB.
    let root = std::env::temp_dir().join(format!("av826-mixed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let caps = vec![TRACE_CAPABILITY.to_string()];
    let mut compact_driver = Driver::with_store(&root, "av826-mixed").expect("store driver");
    compact_driver.set_wire_compact();
    let warmed = compact_driver.request(FIRST_CASE, "d_main", &caps);
    assert_42(warmed.av.clone().expect("AV outcome"), &warmed.trace);
    drop(compact_driver);

    let mut legacy_driver = Driver::with_store(&root, "av826-mixed").expect("store driver");
    let legacy = legacy_driver.request(FIRST_CASE, "d_main", &caps);
    assert_42(legacy.av.clone().expect("AV outcome"), &legacy.trace);
    assert_eq!(
        *legacy.counters.get("cache_hit_abc").unwrap_or(&0),
        0,
        "legacy keys never hit compact rows"
    );
    assert_eq!(
        *legacy
            .counters
            .get("abc_wire_compact_closure")
            .unwrap_or(&0),
        0
    );

    // And the compact policy over the same store hits ITS OWN rows from
    // the warm phase (the legacy request neither consumed nor overwrote
    // them) — the two namespaces coexist without cross-reads.
    let mut compact_again = Driver::with_store(&root, "av826-mixed").expect("store driver");
    compact_again.set_wire_compact();
    let replay = compact_again.request(FIRST_CASE, "d_main", &caps);
    assert_42(replay.av.clone().expect("AV outcome"), &replay.trace);
    assert_eq!(
        *replay.counters.get("cache_hit_abc").unwrap_or(&0),
        2,
        "compact keys hit their own generation's rows, not the legacy rows"
    );
    assert!(
        *replay
            .counters
            .get("abc_wire_compact_closure")
            .unwrap_or(&0)
            >= 1
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn av_wire_826_first_case_unit_never_dispatches() {
    let loaded = admit_compact();
    let unit = split_units(loaded.module()).expect("split").remove(0);
    let wire = encode_compact_abc(&unit).expect("unit compact encode");
    let admitted = verify_compact_abc_bytes(&wire, &loaded.module().owner).expect("unit admission");
    assert!(!admitted.executable());
    let (out, trace) = execute(&admitted, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Infra(_)));
    assert_eq!(trace.loaded_index_builds, 0);
    assert_eq!(trace.loaded_pc_lookups, 0);
    assert_eq!(trace.reference_pc_lookups, 0);
    assert!(trace.events.is_empty());
}
